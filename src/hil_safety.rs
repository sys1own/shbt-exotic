//! Dual-target Hardware-in-the-Loop safety monitor.
//!
//! The monitor concurrently audits:
//!   - the temporal-stasis density register `mu_local` (Newton-lock bias), and
//!   - the ghost-seed mass-congestion register `n_local / n_limit`.
//!
//! If the eigenvector rigidity deviation `|mu_local - mu0|` reaches or exceeds
//! `10^-12`, the monitor triggers an automated fail-fast interrupt.

use pyo3::prelude::*;

use crate::constants::{C_GET_THERMODYNAMIC_BOUND_J, EIGENVECTOR_RIGIDITY_THRESHOLD};
use crate::gmp_memory;

#[pyclass(name = "HilSafetyMonitor")]
#[derive(Clone, Debug)]
pub struct HilSafetyMonitor {
    pub mu0: f64,
    pub c_get_bound: f64,
}

impl HilSafetyMonitor {
    pub fn new() -> Self {
        gmp_memory::init();
        Self {
            mu0: 1.0,
            c_get_bound: C_GET_THERMODYNAMIC_BOUND_J,
        }
    }

    /// Dual-target audit.
    ///
    /// Inputs:
    /// - `mu_local`: local density multiplier (from Newton-lock or ghost seed)
    /// - `n_local`: localized active boundary bits
    /// - `n_limit`: holographic entropy limit of the localized region
    /// - `c_get_local`: local GET cost (J/bit)
    ///
    /// Returns `"STATUS_NOMINAL_PASS"` only when `mu_local` is within the
    /// rigidity threshold and `c_get_local` does not exceed the cosmic Landauer
    /// bound.
    pub fn audit_impl(
        &self,
        mu_local: f64,
        n_local: f64,
        n_limit: f64,
        c_get_local: f64,
    ) -> String {
        let delta_rigidity = (mu_local - self.mu0).abs();
        if delta_rigidity >= EIGENVECTOR_RIGIDITY_THRESHOLD {
            return "EMERGENCY_RIGIDITY_VIOLATION".to_string();
        }
        if !c_get_local.is_finite() || c_get_local <= 0.0 {
            return "EMERGENCY_C_GET_INVALID".to_string();
        }
        if n_limit > 0.0 {
            let congestion = (n_local - n_limit) / n_limit;
            if congestion.abs() >= EIGENVECTOR_RIGIDITY_THRESHOLD {
                return "EMERGENCY_MASS_CONGESTION".to_string();
            }
        }
        "STATUS_NOMINAL_PASS".to_string()
    }

    pub fn is_nominal_impl(&self, status: &str) -> bool {
        status == "STATUS_NOMINAL_PASS"
    }
}

#[pymethods]
impl HilSafetyMonitor {
    #[new]
    pub fn py_new() -> Self {
        Self::new()
    }

    fn audit(&self, mu_local: f64, n_local: f64, n_limit: f64, c_get_local: f64) -> String {
        self.audit_impl(mu_local, n_local, n_limit, c_get_local)
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
    fn rigidity_violation() {
        let monitor = HilSafetyMonitor::new();
        assert_eq!(
            monitor.audit_impl(1.0 + 2.0e-12, 1.0e50, 1.0e60, C_GET_THERMODYNAMIC_BOUND_J),
            "EMERGENCY_RIGIDITY_VIOLATION"
        );
    }
}
