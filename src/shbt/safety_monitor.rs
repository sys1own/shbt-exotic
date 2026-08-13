//! Gate-cycle and thermal-dissipation safety monitor for HIL shunts.
//!
//! Simulates the 142.08 MW field-collapse emergency shunt at the 72 GHz
//! InP/InGaAs SHBT clock-cycle level and verifies that the dumped energy does
//! not thermally quench the dilution refrigerator.

use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::constants::{
    F_MAX_HZ, GHOST_SEED_TRANSIENT_W, KB_J_PER_K, LN2, N_LOCAL_BITS, TEMPERATURE_K,
};
use crate::hil_safety::HilSafetyMonitor;

/// Hard-coded emergency shutdown latency budget (s).
const SHUNT_LATENCY_S: f64 = 2.5e-9;

/// Gate-cycle-level shunt simulator.
#[pyclass(name = "GateCycleShunt")]
#[derive(Debug, Clone)]
pub struct GateCycleShunt {
    /// SHBT transistor clock rate (Hz).
    pub f_max_hz: f64,
    /// Maximum allowed shunt latency (s).
    pub latency_budget_s: f64,
    /// Number of gate cycles available within the latency budget.
    pub max_cycles: usize,
}

impl GateCycleShunt {
    pub fn new() -> Self {
        let period_s = 1.0 / F_MAX_HZ;
        let max_cycles = ((SHUNT_LATENCY_S / period_s).floor() as usize).max(1);
        Self {
            f_max_hz: F_MAX_HZ,
            latency_budget_s: SHUNT_LATENCY_S,
            max_cycles,
        }
    }

    /// Clock period (s).
    pub fn cycle_period_s(&self) -> f64 {
        1.0 / self.f_max_hz
    }

    /// Latency for a given number of gate cycles.
    pub fn latency_for_cycles_s(&self, cycles: usize) -> f64 {
        (cycles as f64) * self.cycle_period_s()
    }

    /// Latency for the maximum allowed number of cycles (s).
    pub fn max_latency_s(&self) -> f64 {
        self.latency_for_cycles_s(self.max_cycles)
    }

    /// Simulate the gate-cycle sampling path.  At each clock cycle the HIL
    /// monitor samples the registers; if a shutdown is triggered, the loop
    /// aborts and returns the guaranteed shunt latency and the cycle at which
    /// the fault was detected.
    pub fn simulate_shutdown(
        &self,
        hil: &HilSafetyMonitor,
        mu_local: f64,
        n_local: f64,
        n_limit: f64,
        c_get_local: f64,
    ) -> (String, f64, usize) {
        for cycle in 1..=self.max_cycles {
            let (status, latency_ns, _corrected) =
                hil.sample_impl(mu_local, n_local, n_limit, c_get_local);
            if status == "STATUS_EMERGENCY_SHUTDOWN" {
                return (status, latency_ns, cycle);
            }
            if status == "STATUS_CORRECTION_APPLIED" {
                return (status, latency_ns, cycle);
            }
        }
        (
            "STATUS_NOMINAL_PASS".to_string(),
            self.max_latency_s() * 1e9,
            self.max_cycles,
        )
    }
}

#[pymethods]
impl GateCycleShunt {
    #[new]
    pub fn py_new() -> Self {
        Self::new()
    }

    #[getter]
    fn get_cycle_period_s(&self) -> f64 {
        self.cycle_period_s()
    }

    #[getter]
    fn get_max_cycles(&self) -> usize {
        self.max_cycles
    }

    #[getter]
    fn get_max_latency_s(&self) -> f64 {
        self.max_latency_s()
    }
}

/// Thermal-dissipation auditor for the emergency field-collapse shunt.
#[pyclass(name = "ThermalShuntAuditor")]
#[derive(Debug, Clone)]
pub struct ThermalShuntAuditor {
    /// Power of the rendered field being collapsed (W).
    pub field_power_w: f64,
    /// Shunt latency (s).
    pub shunt_latency_s: f64,
    /// Cryogenic base temperature (K).
    pub base_temperature_k: f64,
    /// Local bit budget used for holographic heat capacity.
    pub n_local_bits: f64,
}

impl ThermalShuntAuditor {
    pub fn new() -> Self {
        Self::with_params(GHOST_SEED_TRANSIENT_W, SHUNT_LATENCY_S, TEMPERATURE_K, N_LOCAL_BITS)
    }

    pub fn with_params(
        field_power_w: f64,
        shunt_latency_s: f64,
        base_temperature_k: f64,
        n_local_bits: f64,
    ) -> Self {
        Self {
            field_power_w,
            shunt_latency_s,
            base_temperature_k,
            n_local_bits,
        }
    }

    /// Energy dumped during the shunt, `E = P \tau`.
    pub fn energy_dissipative_j(&self) -> f64 {
        self.field_power_w * self.shunt_latency_s
    }

    /// Thermal capacity of the local holographic register, `C = N k_B ln 2`.
    pub fn thermal_capacity_j_per_k(&self) -> f64 {
        self.n_local_bits * KB_J_PER_K * LN2
    }

    /// Thermal flux during the shunt, `\dot{Q} = E / \tau`.
    pub fn q_dot_shunt_w(&self) -> f64 {
        self.energy_dissipative_j() / self.shunt_latency_s
    }

    /// Holographic cooling power available during the shunt,
    /// `P_cooling = C T / \tau`.
    pub fn cooling_power_w(&self) -> f64 {
        self.thermal_capacity_j_per_k() * self.base_temperature_k / self.shunt_latency_s
    }

    /// Temperature rise if the shunt energy is dumped into the local heat capacity.
    pub fn temperature_rise_k(&self) -> f64 {
        self.energy_dissipative_j() / self.thermal_capacity_j_per_k()
    }

    /// Thermal audit status.
    pub fn audit(&self) -> &'static str {
        if self.q_dot_shunt_w() <= self.cooling_power_w()
            && self.temperature_rise_k() <= self.base_temperature_k
        {
            "STATUS_NOMINAL_PASS"
        } else {
            "EMERGENCY_THERMAL_QUENCH"
        }
    }
}

#[pymethods]
impl ThermalShuntAuditor {
    #[new]
    #[pyo3(signature = (
        field_power_w = GHOST_SEED_TRANSIENT_W,
        shunt_latency_s = SHUNT_LATENCY_S,
        base_temperature_k = TEMPERATURE_K,
        n_local_bits = N_LOCAL_BITS
    ))]
    pub fn py_new(
        field_power_w: f64,
        shunt_latency_s: f64,
        base_temperature_k: f64,
        n_local_bits: f64,
    ) -> Self {
        Self::with_params(field_power_w, shunt_latency_s, base_temperature_k, n_local_bits)
    }

    /// Energy dumped during the shunt (J).
    #[pyo3(name = "energy_dissipative_j")]
    fn py_energy_dissipative_j(&self) -> f64 {
        self.energy_dissipative_j()
    }

    /// Thermal capacity of the local holographic register (J/K).
    #[pyo3(name = "thermal_capacity_j_per_k")]
    fn py_thermal_capacity_j_per_k(&self) -> f64 {
        self.thermal_capacity_j_per_k()
    }

    /// Thermal flux during the shunt (W).
    #[pyo3(name = "q_dot_shunt_w")]
    fn py_q_dot_shunt_w(&self) -> f64 {
        self.q_dot_shunt_w()
    }

    /// Holographic cooling power available during the shunt (W).
    #[pyo3(name = "cooling_power_w")]
    fn py_cooling_power_w(&self) -> f64 {
        self.cooling_power_w()
    }

    /// Temperature rise from dumping the shunt energy (K).
    #[pyo3(name = "temperature_rise_k")]
    fn py_temperature_rise_k(&self) -> f64 {
        self.temperature_rise_k()
    }

    /// Thermal audit status.
    #[pyo3(name = "audit")]
    fn py_audit(&self) -> &'static str {
        self.audit()
    }

    /// Audit the thermal shunt and return a dictionary of computed quantities.
    fn audit_py<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let d = PyDict::new(py);
        d.set_item("field_power_w", self.field_power_w)?;
        d.set_item("shunt_latency_s", self.shunt_latency_s)?;
        d.set_item("base_temperature_k", self.base_temperature_k)?;
        d.set_item("energy_dissipative_j", self.energy_dissipative_j())?;
        d.set_item("q_dot_shunt_w", self.q_dot_shunt_w())?;
        d.set_item("cooling_power_w", self.cooling_power_w())?;
        d.set_item("temperature_rise_k", self.temperature_rise_k())?;
        d.set_item("status", self.audit())?;
        Ok(d)
    }
}

impl Default for ThermalShuntAuditor {
    fn default() -> Self {
        Self::new()
    }
}

/// Unified safety monitor: gate-cycle shutdown + thermal shunt audit.
#[pyclass(name = "SafetyMonitor")]
#[derive(Debug, Clone)]
pub struct SafetyMonitor {
    pub hil: HilSafetyMonitor,
    pub gate_shunt: GateCycleShunt,
    pub thermal: ThermalShuntAuditor,
}

impl SafetyMonitor {
    pub fn new() -> Self {
        Self {
            hil: HilSafetyMonitor::new(),
            gate_shunt: GateCycleShunt::new(),
            thermal: ThermalShuntAuditor::new(),
        }
    }

    /// Simulate the full gate-cycle shutdown path and thermal audit.
    pub fn simulate_shutdown(
        &self,
        mu_local: f64,
        n_local: f64,
        n_limit: f64,
        c_get_local: f64,
    ) -> (String, f64, usize, &'static str) {
        let (status, latency_ns, cycles) =
            self.gate_shunt
                .simulate_shutdown(&self.hil, mu_local, n_local, n_limit, c_get_local);
        let thermal_status = self.thermal.audit();
        let combined = if status == "STATUS_NOMINAL_PASS" && thermal_status == "STATUS_NOMINAL_PASS"
        {
            "STATUS_NOMINAL_PASS"
        } else if status == "STATUS_EMERGENCY_SHUTDOWN" {
            "STATUS_EMERGENCY_SHUTDOWN"
        } else if thermal_status == "EMERGENCY_THERMAL_QUENCH" {
            "EMERGENCY_THERMAL_QUENCH"
        } else {
            status.as_str()
        };
        (combined.to_string(), latency_ns, cycles, thermal_status)
    }
}

#[pymethods]
impl SafetyMonitor {
    #[new]
    pub fn py_new() -> Self {
        Self::new()
    }

    /// Simulate the full gate-cycle shutdown path and thermal audit.
    #[pyo3(name = "simulate_shutdown")]
    fn simulate_shutdown_py(
        &self,
        mu_local: f64,
        n_local: f64,
        n_limit: f64,
        c_get_local: f64,
    ) -> (String, f64, usize, String) {
        let (status, latency_ns, cycles, thermal) =
            self.simulate_shutdown(mu_local, n_local, n_limit, c_get_local);
        (status, latency_ns, cycles, thermal.to_string())
    }

    /// Return the embedded gate-cycle shunt model.
    fn gate_cycle_shunt(&self) -> GateCycleShunt {
        self.gate_shunt.clone()
    }

    /// Return the embedded thermal shunt auditor.
    fn thermal_shunt_auditor(&self) -> ThermalShuntAuditor {
        self.thermal.clone()
    }
}

impl Default for SafetyMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gate_cycle_shunt_meets_latency_budget() {
        let shunt = GateCycleShunt::new();
        assert!(shunt.max_latency_s() <= SHUNT_LATENCY_S);
        assert_eq!(shunt.max_cycles, 180);
    }

    #[test]
    fn thermal_shunt_does_not_quench() {
        let auditor = ThermalShuntAuditor::new();
        assert!(auditor.q_dot_shunt_w() <= auditor.cooling_power_w());
        assert!(auditor.temperature_rise_k() <= auditor.base_temperature_k);
        assert_eq!(auditor.audit(), "STATUS_NOMINAL_PASS");
    }

    #[test]
    fn safety_monitor_passes_nominal() {
        let monitor = SafetyMonitor::new();
        let alpha = 1.67e-51;
        let n_limit = 1.0e65;
        let n_local = n_limit + 1.0 / alpha;
        let c_get = 5.34e-175;
        let (status, _latency_ns, _cycles, thermal) =
            monitor.simulate_shutdown(1.0, n_local, n_limit, c_get);
        assert_eq!(status, "STATUS_NOMINAL_PASS");
        assert_eq!(thermal, "STATUS_NOMINAL_PASS");
    }

    #[test]
    fn safety_monitor_triggers_emergency_shutdown() {
        let monitor = SafetyMonitor::new();
        let (status, latency_ns, cycles, _thermal) =
            monitor.simulate_shutdown(1.0 + 2.0e-12, 1.0e65, 1.0e65, 5.34e-175);
        assert_eq!(status, "STATUS_EMERGENCY_SHUTDOWN");
        assert!(latency_ns > 0.0);
        assert!(cycles <= 180);
    }
}
