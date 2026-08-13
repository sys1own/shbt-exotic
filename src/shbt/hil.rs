//! Professional Debye T^3 thermal model for the InP HIL safety monitor.
//!
//! The acoustic phonon heat capacity per unit volume for a crystalline InP
//! substrate at cryogenic temperature is
//!
//!   C_{v,vol}(T) = a_InP * T^3,
//!
//! with `a_InP = 3.87759483 J/(m^3 K^4)`.  A 142.08 MW magnetic-field collapse
//! over a 2.5 ns shunt window dumps `E = P * tau` into the InP volume `V`.
//! Using the exact Debye integral,
//!
//!   E = a_InP * V * (T_f^4 - T_i^4) / 4,
//!
//! the final temperature is
//!
//!   T_f = ((4 * E) / (a_InP * V) + T_i^4)^{1/4}.
//!
//! The HIL monitor triggers `STATUS_EMERGENCY_SHUTDOWN` and shunts all bias
//! currents if `T_f` exceeds the niobium superconducting transition `T_c = 9.3 K`
//! or if the dissipation volume is below `48.98 cm^3`.

use pyo3::prelude::*;

use crate::constants::{
    BASELINE_TEMPERATURE_K, GHOST_SEED_TRANSIENT_W, INP_DEBYE_A_J_PER_M3_K4,
    MIN_DISSIPATION_VOLUME_CM3, NIOBIUM_TRANSITION_TEMP_K,
};

/// Thermal HIL monitor using the Debye T^3 model.
#[pyclass(name = "ThermalHILMonitor")]
#[derive(Clone, Debug)]
pub struct ThermalHILMonitor {
    /// Debye T^3 coefficient `a_InP` (J/(m^3 K^4)).
    pub a_inp: f64,
    /// Baseline InP substrate temperature `T_i` (K).
    pub t_i: f64,
    /// Niobium superconducting transition temperature `T_c` (K).
    pub t_c: f64,
    /// Power of the field collapse being shunted (W).
    pub power_w: f64,
    /// Shunt latency `tau` (s).
    pub tau_s: f64,
    /// Dissipation volume `V` (m^3).
    pub volume_m3: f64,
}

impl ThermalHILMonitor {
    pub fn new() -> Self {
        // Use a 50 cm^3 design volume by default — safely above the 48.98 cm^3
        // Debye threshold and still within a compact cryogenic interconnect.
        Self::with_params(
            INP_DEBYE_A_J_PER_M3_K4,
            BASELINE_TEMPERATURE_K,
            NIOBIUM_TRANSITION_TEMP_K,
            GHOST_SEED_TRANSIENT_W,
            2.5e-9,
            50.0,
        )
    }

    pub fn with_params(
        a_inp: f64,
        t_i: f64,
        t_c: f64,
        power_w: f64,
        tau_s: f64,
        volume_cm3: f64,
    ) -> Self {
        Self {
            a_inp,
            t_i,
            t_c,
            power_w,
            tau_s,
            volume_m3: volume_cm3 * 1.0e-6,
        }
    }

    /// Dissipation volume in cm^3.
    pub fn volume_cm3_impl(&self) -> f64 {
        self.volume_m3 * 1.0e6
    }

    /// Energy dumped by the shunt, `E = P * tau` (J).
    pub fn energy_dissipative_j_impl(&self) -> f64 {
        self.power_w * self.tau_s
    }

    /// Volumetric heat capacity at temperature `T`, `C_v,vol = a_InP * T^3`.
    pub fn volumetric_heat_capacity_impl(&self, t: f64) -> f64 {
        self.a_inp * t.powi(3)
    }

    /// Final substrate temperature after the shunt using the exact Debye integral.
    pub fn final_temperature_k_impl(&self) -> f64 {
        let e = self.energy_dissipative_j_impl();
        let t_i4 = self.t_i.powi(4);
        let term = (4.0 * e) / (self.a_inp * self.volume_m3) + t_i4;
        term.max(t_i4).powf(0.25)
    }

    /// Temperature rise above baseline (K).
    pub fn temperature_rise_k_impl(&self) -> f64 {
        self.final_temperature_k_impl() - self.t_i
    }

    /// Heat capacity of the dissipating volume evaluated at the final temperature (J/K).
    pub fn heat_capacity_j_per_k_impl(&self) -> f64 {
        self.volumetric_heat_capacity_impl(self.final_temperature_k_impl()) * self.volume_m3
    }

    /// Thermal audit status.
    pub fn audit_impl(&self) -> &'static str {
        if self.volume_cm3_impl() < MIN_DISSIPATION_VOLUME_CM3 {
            "STATUS_EMERGENCY_SHUTDOWN"
        } else if self.final_temperature_k_impl() > self.t_c {
            "STATUS_EMERGENCY_SHUTDOWN"
        } else {
            "STATUS_NOMINAL_PASS"
        }
    }

    /// True if the shunt would not thermally quench the interconnects.
    pub fn is_nominal_impl(&self) -> bool {
        self.audit_impl() == "STATUS_NOMINAL_PASS"
    }
}

#[pymethods]
impl ThermalHILMonitor {
    #[new]
    #[pyo3(signature = (
        a_inp = INP_DEBYE_A_J_PER_M3_K4,
        t_i = BASELINE_TEMPERATURE_K,
        t_c = NIOBIUM_TRANSITION_TEMP_K,
        power_w = GHOST_SEED_TRANSIENT_W,
        tau_s = 2.5e-9,
        volume_cm3 = 50.0
    ))]
    pub fn py_new(
        a_inp: f64,
        t_i: f64,
        t_c: f64,
        power_w: f64,
        tau_s: f64,
        volume_cm3: f64,
    ) -> Self {
        Self::with_params(a_inp, t_i, t_c, power_w, tau_s, volume_cm3)
    }

    fn volume_cm3(&self) -> f64 {
        self.volume_cm3_impl()
    }

    fn energy_dissipative_j(&self) -> f64 {
        self.energy_dissipative_j_impl()
    }

    fn volumetric_heat_capacity(&self, t: f64) -> f64 {
        self.volumetric_heat_capacity_impl(t)
    }

    fn final_temperature_k(&self) -> f64 {
        self.final_temperature_k_impl()
    }

    fn temperature_rise_k(&self) -> f64 {
        self.temperature_rise_k_impl()
    }

    fn heat_capacity_j_per_k(&self) -> f64 {
        self.heat_capacity_j_per_k_impl()
    }

    fn audit(&self) -> &'static str {
        self.audit_impl()
    }

    fn is_nominal(&self) -> bool {
        self.is_nominal_impl()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn debye_t3_final_temperature_at_threshold_volume_is_tc() {
        let monitor = ThermalHILMonitor::with_params(
            INP_DEBYE_A_J_PER_M3_K4,
            BASELINE_TEMPERATURE_K,
            NIOBIUM_TRANSITION_TEMP_K,
            GHOST_SEED_TRANSIENT_W,
            2.5e-9,
            MIN_DISSIPATION_VOLUME_CM3,
        );
        let tf = monitor.final_temperature_k_impl();
        // At the exact 48.98 cm^3 threshold the designed transient lands on 9.3 K.
        assert!((tf - NIOBIUM_TRANSITION_TEMP_K).abs() < 1e-6);
    }

    #[test]
    fn smaller_volume_quenches() {
        let monitor = ThermalHILMonitor::with_params(
            INP_DEBYE_A_J_PER_M3_K4,
            BASELINE_TEMPERATURE_K,
            NIOBIUM_TRANSITION_TEMP_K,
            GHOST_SEED_TRANSIENT_W,
            2.5e-9,
            1.0,
        );
        assert_eq!(monitor.audit_impl(), "STATUS_EMERGENCY_SHUTDOWN");
        assert!(monitor.final_temperature_k_impl() > NIOBIUM_TRANSITION_TEMP_K);
    }

    #[test]
    fn larger_volume_stays_nominal() {
        let monitor = ThermalHILMonitor::with_params(
            INP_DEBYE_A_J_PER_M3_K4,
            BASELINE_TEMPERATURE_K,
            NIOBIUM_TRANSITION_TEMP_K,
            GHOST_SEED_TRANSIENT_W,
            2.5e-9,
            100.0,
        );
        assert_eq!(monitor.audit_impl(), "STATUS_NOMINAL_PASS");
        assert!(monitor.final_temperature_k_impl() < NIOBIUM_TRANSITION_TEMP_K);
    }
}
