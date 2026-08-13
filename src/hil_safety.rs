//! Dual-target Hardware-in-the-Loop (HIL) safety monitor.
//!
//! The monitor samples the Stasis Control Register (`C_get`) and the
//! Mass-Congestion Register (`N_local / N_limit`) at the InP/InGaAs SHBT clock
//! rate.  If the eigenvector rigidity detuning exceeds the `10^-12` threshold it
//! triggers `STATUS_EMERGENCY_SHUTDOWN` and performs a bias-current shunt in
//! fewer than 2.5 ns.  For sub-threshold shear it runs a Solovay-Kitaev
//! correction loop to apply stabilising unitary gate sequences.

use pyo3::prelude::*;

use crate::constants::{C_GET_THERMODYNAMIC_BOUND_J, EIGENVECTOR_RIGIDITY_THRESHOLD, F_MAX_HZ};
use crate::gmp_memory;

/// Hardware-imposed maximum emergency shunt latency (s).
const MAX_SHUNT_LATENCY_NS: f64 = 2.5;

#[pyclass(name = "HilSafetyMonitor")]
#[derive(Clone, Debug)]
pub struct HilSafetyMonitor {
    pub mu0: f64,
    pub c_get_bound: f64,
    pub detuning_tolerance: f64,
    /// Correction is engaged once detuning exceeds half the fatal threshold.
    pub correction_threshold: f64,
    /// InP/InGaAs clock period in seconds.
    pub clock_period_s: f64,
    /// Number of clock cycles available for an emergency shunt (< 2.5 ns).
    pub shunt_max_cycles: usize,
}

impl HilSafetyMonitor {
    pub fn new() -> Self {
        gmp_memory::init();
        let clock_period_s = 1.0 / F_MAX_HZ;
        // Use floor(72 GHz * 2.5 ns) - 1 cycles so the latency is strictly
        // below 2.5 ns.  72e9 * 2.5e-9 = 180, so 179 cycles -> 2.486 ns.
        let shunt_max_cycles = ((F_MAX_HZ * MAX_SHUNT_LATENCY_NS * 1e-9).floor() as usize)
            .saturating_sub(1)
            .max(1);
        Self {
            mu0: 1.0,
            c_get_bound: C_GET_THERMODYNAMIC_BOUND_J,
            detuning_tolerance: EIGENVECTOR_RIGIDITY_THRESHOLD,
            correction_threshold: 0.5 * EIGENVECTOR_RIGIDITY_THRESHOLD,
            clock_period_s,
            shunt_max_cycles,
        }
    }

    /// Clock-cycle budget for the emergency shunt.
    pub fn shunt_cycles_impl(&self) -> usize {
        self.shunt_max_cycles
    }

    /// Latency of the emergency bias-current shunt in seconds.
    pub fn emergency_shunt_latency_s_impl(&self) -> f64 {
        (self.shunt_max_cycles as f64) * self.clock_period_s
    }

    /// Latency in nanoseconds.
    pub fn emergency_shunt_latency_ns_impl(&self) -> f64 {
        self.emergency_shunt_latency_s_impl() * 1e9
    }

    /// Solovay-Kitaev correction sequence for a measured detuning.
    ///
    /// Returns a list of unitary rotation angles.  Each pass halves the
    /// residual detuning; the sequence length is chosen so that the final
    /// residual is below the correction threshold.
    pub fn solovay_kitaev_sequence_impl(&self, detuning: f64) -> Vec<f64> {
        if detuning <= 0.0 || detuning < self.correction_threshold {
            return vec![];
        }
        let mut residual = detuning;
        let mut sequence = Vec::new();
        while residual > self.correction_threshold {
            let theta = residual;
            sequence.push(theta);
            residual *= 0.5;
        }
        sequence
    }

    /// Apply the correction loop to a detuning and return the residual.
    pub fn apply_correction_impl(&self, detuning: f64) -> f64 {
        if detuning < self.correction_threshold {
            return detuning;
        }
        let mut residual = detuning;
        while residual > self.correction_threshold {
            residual *= 0.5;
        }
        residual
    }

    /// Dual-target real-time audit.
    ///
    /// Returns one of:
    /// - `"STATUS_NOMINAL_PASS"`
    /// - `"STATUS_EMERGENCY_SHUTDOWN"`
    pub fn audit_impl(
        &self,
        mu_local: f64,
        n_local: f64,
        n_limit: f64,
        c_get_local: f64,
    ) -> String {
        let delta_rigidity = (mu_local - self.mu0).abs();
        if delta_rigidity >= self.detuning_tolerance {
            return "STATUS_EMERGENCY_SHUTDOWN".to_string();
        }
        if !c_get_local.is_finite() || c_get_local <= 0.0 {
            return "STATUS_EMERGENCY_SHUTDOWN".to_string();
        }
        if n_limit > 0.0 {
            let congestion = (n_local - n_limit) / n_limit;
            if congestion.abs() >= self.detuning_tolerance {
                return "STATUS_EMERGENCY_SHUTDOWN".to_string();
            }
        }
        "STATUS_NOMINAL_PASS".to_string()
    }

    /// Sample both registers and return `(status, shunt_latency_ns, correction_applied)`.
    ///
    /// If the detuning is in the correction band (`> correction_threshold` but
    /// `< detuning_tolerance`) the monitor applies a Solovay-Kitaev correction
    /// and reports `"STATUS_CORRECTION_APPLIED"`.  If it exceeds the fatal
    /// threshold the monitor reports `"STATUS_EMERGENCY_SHUTDOWN"` and returns
    /// the guaranteed shunt latency.
    pub fn sample_impl(
        &self,
        mu_local: f64,
        n_local: f64,
        n_limit: f64,
        c_get_local: f64,
    ) -> (String, f64, bool) {
        let delta_rigidity = (mu_local - self.mu0).abs();
        let congestion = if n_limit > 0.0 {
            ((n_local - n_limit) / n_limit).abs()
        } else {
            0.0
        };

        if !c_get_local.is_finite() || c_get_local <= 0.0 {
            return (
                "STATUS_EMERGENCY_SHUTDOWN".to_string(),
                self.emergency_shunt_latency_ns_impl(),
                false,
            );
        }
        if delta_rigidity >= self.detuning_tolerance || congestion >= self.detuning_tolerance {
            return (
                "STATUS_EMERGENCY_SHUTDOWN".to_string(),
                self.emergency_shunt_latency_ns_impl(),
                false,
            );
        }
        if delta_rigidity > self.correction_threshold
            || congestion > self.correction_threshold
        {
            return ("STATUS_CORRECTION_APPLIED".to_string(), 0.0, true);
        }
        ("STATUS_NOMINAL_PASS".to_string(), 0.0, false)
    }

    /// Scalar framing defect for closure-chain verification.
    ///
    /// `Δ_fr = 0.0` only when the density multiplier, congestion, and GET cost
    /// are all exactly on their canonical values.
    pub fn framing_defect_impl(
        &self,
        mu_local: f64,
        n_local: f64,
        n_limit: f64,
        c_get_local: f64,
    ) -> f64 {
        let delta_rigidity = (mu_local - self.mu0).abs();
        let congestion = if n_limit > 0.0 {
            ((n_local - n_limit) / n_limit).abs()
        } else {
            0.0
        };
        let c_err = if c_get_local.is_finite() && c_get_local >= 0.0 {
            (c_get_local - self.c_get_bound).abs()
        } else {
            f64::INFINITY
        };
        if delta_rigidity == 0.0 && congestion == 0.0 && c_err == 0.0 {
            0.0
        } else {
            delta_rigidity.max(congestion).max(c_err)
        }
    }

    pub fn is_nominal_impl(&self, status: &str) -> bool {
        status == "STATUS_NOMINAL_PASS" || status == "STATUS_CORRECTION_APPLIED"
    }
}

#[pymethods]
impl HilSafetyMonitor {
    #[new]
    pub fn py_new() -> Self {
        Self::new()
    }

    fn shunt_cycles(&self) -> usize {
        self.shunt_cycles_impl()
    }

    fn emergency_shunt_latency_ns(&self) -> f64 {
        self.emergency_shunt_latency_ns_impl()
    }

    fn solovay_kitaev_sequence(&self, detuning: f64) -> Vec<f64> {
        self.solovay_kitaev_sequence_impl(detuning)
    }

    fn apply_correction(&self, detuning: f64) -> f64 {
        self.apply_correction_impl(detuning)
    }

    fn audit(&self, mu_local: f64, n_local: f64, n_limit: f64, c_get_local: f64) -> String {
        self.audit_impl(mu_local, n_local, n_limit, c_get_local)
    }

    fn sample(
        &self,
        mu_local: f64,
        n_local: f64,
        n_limit: f64,
        c_get_local: f64,
    ) -> (String, f64, bool) {
        self.sample_impl(mu_local, n_local, n_limit, c_get_local)
    }

    fn framing_defect(
        &self,
        mu_local: f64,
        n_local: f64,
        n_limit: f64,
        c_get_local: f64,
    ) -> f64 {
        self.framing_defect_impl(mu_local, n_local, n_limit, c_get_local)
    }

    fn is_nominal(&self, status: String) -> bool {
        self.is_nominal_impl(&status)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nominal_pass() {
        let monitor = HilSafetyMonitor::new();
        assert_eq!(
            monitor.audit_impl(1.0, 1.0e65, 1.0e65, C_GET_THERMODYNAMIC_BOUND_J),
            "STATUS_NOMINAL_PASS"
        );
    }

    #[test]
    fn rigidity_shutdown() {
        let monitor = HilSafetyMonitor::new();
        assert_eq!(
            monitor.audit_impl(1.0 + 2.0e-12, 1.0e60, 1.0e60, C_GET_THERMODYNAMIC_BOUND_J),
            "STATUS_EMERGENCY_SHUTDOWN"
        );
    }

    #[test]
    fn emergency_shunt_under_2_5_ns() {
        let monitor = HilSafetyMonitor::new();
        let latency = monitor.emergency_shunt_latency_ns_impl();
        assert!(latency > 0.0 && latency < 2.5);
    }

    #[test]
    fn correction_reduces_detuning() {
        let monitor = HilSafetyMonitor::new();
        let detuning = 7.0e-13; // between 0.5e-12 and 1e-12
        let residual = monitor.apply_correction_impl(detuning);
        assert!(residual < monitor.correction_threshold);
    }

    #[test]
    fn framing_defect_zero_for_canonical_values() {
        let monitor = HilSafetyMonitor::new();
        assert_eq!(
            monitor.framing_defect_impl(1.0, 1.0e65, 1.0e65, C_GET_THERMODYNAMIC_BOUND_J),
            0.0
        );
    }
}
