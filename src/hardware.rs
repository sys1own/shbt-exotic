//! Hardware-in-the-Loop synthesis and invariant auditing.
//!
//! Verifies that the universal backend operates within the InP/InGaAs SHBT
//! transistor limits (`f_max = 72 GHz`) and the boundary state routing bandwidth
//! (`B = 40 Gb/s`).

use pyo3::prelude::*;

use crate::constants::{F_MAX_HZ, ROUTING_BANDWIDTH_BPS};
use crate::gmp_memory;

#[pyclass(name = "HardwareSynthesisAuditor")]
#[derive(Clone, Debug)]
pub struct HardwareSynthesisAuditor {
    pub f_max_hz: f64,
    pub routing_bandwidth_bps: f64,
}

impl HardwareSynthesisAuditor {
    pub fn new() -> Self {
        gmp_memory::init();
        Self {
            f_max_hz: F_MAX_HZ,
            routing_bandwidth_bps: ROUTING_BANDWIDTH_BPS,
        }
    }

    /// Clock period of the InP/InGaAs SHBT transistor (s).
    pub fn clock_period_s_impl(&self) -> f64 {
        1.0 / self.f_max_hz
    }

    /// Check that the provided clock rate does not exceed the 72 GHz limit.
    pub fn clock_rate_passes_impl(&self, clock_hz: f64) -> bool {
        clock_hz > 0.0 && clock_hz <= self.f_max_hz
    }

    /// Check that the provided routing bandwidth does not exceed 40 Gb/s.
    pub fn routing_bandwidth_passes_impl(&self, bandwidth_bps: f64) -> bool {
        bandwidth_bps > 0.0 && bandwidth_bps <= self.routing_bandwidth_bps
    }

    /// Combined hardware invariant audit.
    pub fn audit_impl(&self, clock_hz: f64, bandwidth_bps: f64) -> String {
        if !self.clock_rate_passes_impl(clock_hz) {
            return "STATUS_EMERGENCY_SHUTDOWN".to_string();
        }
        if !self.routing_bandwidth_passes_impl(bandwidth_bps) {
            return "STATUS_EMERGENCY_SHUTDOWN".to_string();
        }
        "STATUS_NOMINAL_PASS".to_string()
    }

    /// Maximum theoretical number of state routing operations per clock edge
    /// if one bit is transferred per clock cycle.
    pub fn max_bits_per_clock_cycle_impl(&self) -> f64 {
        // One bit per full clock cycle requires at least a toggling edge;
        // the Nyquist limit for a 72 GHz clock is 72 Gb/s.
        self.f_max_hz
    }
}

#[pymethods]
impl HardwareSynthesisAuditor {
    #[new]
    pub fn py_new() -> Self {
        Self::new()
    }

    fn clock_period_s(&self) -> f64 {
        self.clock_period_s_impl()
    }

    fn clock_rate_passes(&self, clock_hz: f64) -> bool {
        self.clock_rate_passes_impl(clock_hz)
    }

    fn routing_bandwidth_passes(&self, bandwidth_bps: f64) -> bool {
        self.routing_bandwidth_passes_impl(bandwidth_bps)
    }

    fn audit(&self, clock_hz: f64, bandwidth_bps: f64) -> String {
        self.audit_impl(clock_hz, bandwidth_bps)
    }

    fn max_bits_per_clock_cycle(&self) -> f64 {
        self.max_bits_per_clock_cycle_impl()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clock_and_bandwidth_nominal() {
        let hw = HardwareSynthesisAuditor::new();
        assert!(hw.clock_rate_passes_impl(F_MAX_HZ));
        assert!(hw.routing_bandwidth_passes_impl(ROUTING_BANDWIDTH_BPS));
        assert_eq!(hw.audit_impl(F_MAX_HZ, ROUTING_BANDWIDTH_BPS), "STATUS_NOMINAL_PASS");
    }

    #[test]
    fn excessive_clock_fails() {
        let hw = HardwareSynthesisAuditor::new();
        assert!(!hw.clock_rate_passes_impl(F_MAX_HZ * 1.1));
    }

    #[test]
    fn excessive_bandwidth_fails() {
        let hw = HardwareSynthesisAuditor::new();
        assert!(!hw.routing_bandwidth_passes_impl(ROUTING_BANDWIDTH_BPS * 1.1));
    }
}
