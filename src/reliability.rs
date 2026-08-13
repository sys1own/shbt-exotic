//! Thermal-fatigue reliability model for the Alumina/InP interface.
//!
//! Uses the Coffin-Manson low-cycle fatigue picture: a 15 K thermal swing induces
//! a plastic strain Δε_p = 6.0×10⁻⁶, giving a cycle-to-failure limit
//! N_f = 4.0×10⁶.  The equivalent de-rendering lifetime budget is
//! 1.514×10¹⁶ bits.  Exceeding this budget shifts the acoustic impedance toward
//! Z = 1.3250 MRayl and raises a STATUS_QUENCH_WARNING because the resulting
//! mismatch raises the probability of a superconducting niobium quench.

use pyo3::prelude::*;

/// Plastic strain amplitude induced by the 15 K thermal swing.
pub const COFFIN_MANSON_PLASTIC_STRAIN: f64 = 6.0e-6;
/// Thermal swing amplitude (K).
pub const THERMAL_FATIGUE_SWING_K: f64 = 15.0;
/// Cycle-to-failure limit for the Alumina/InP interface.
pub const CYCLES_TO_FAILURE: f64 = 4.0e6;
/// Equivalent de-rendering lifetime budget (bits).
pub const LIFETIME_BITS_DE_RENDERED: f64 = 1.514e16;
/// Acoustic impedance the interface drifts toward when fatigued (MRayl).
pub const FATIGUE_SHIFTED_IMPEDANCE_MRAYL: f64 = 1.3250;

/// Audit engine tracking cumulative de-rendering and thermal-fatigue damage.
#[pyclass(name = "ReliabilityAuditor")]
#[derive(Clone, Debug)]
pub struct ReliabilityAuditor {
    cumulative_bits: f64,
}

impl ReliabilityAuditor {
    pub fn new() -> Self {
        Self {
            cumulative_bits: 0.0,
        }
    }

    pub fn accumulate_bits_impl(&mut self, bits: f64) {
        self.cumulative_bits += bits.max(0.0);
    }

    pub fn remaining_lifetime_bits_impl(&self) -> f64 {
        (LIFETIME_BITS_DE_RENDERED - self.cumulative_bits).max(0.0)
    }

    pub fn consumed_cycles_impl(&self) -> f64 {
        if LIFETIME_BITS_DE_RENDERED > 0.0 {
            (self.cumulative_bits / LIFETIME_BITS_DE_RENDERED) * CYCLES_TO_FAILURE
        } else {
            0.0
        }
    }

    pub fn cumulative_bits_impl(&self) -> f64 {
        self.cumulative_bits
    }

    pub fn audit_impl(&self) -> (String, bool, f64, f64, f64) {
        let remaining = self.remaining_lifetime_bits_impl();
        let consumed = self.consumed_cycles_impl();
        let nominal = self.cumulative_bits <= LIFETIME_BITS_DE_RENDERED;
        let status = if nominal {
            "STATUS_NOMINAL_PASS".to_string()
        } else {
            "STATUS_QUENCH_WARNING".to_string()
        };
        (
            status,
            nominal,
            remaining,
            consumed,
            FATIGUE_SHIFTED_IMPEDANCE_MRAYL,
        )
    }
}

#[pymethods]
impl ReliabilityAuditor {
    #[new]
    fn py_new() -> Self {
        Self::new()
    }

    /// Add bits to the cumulative de-rendering total.
    fn accumulate_bits(&mut self, bits: f64) {
        self.accumulate_bits_impl(bits);
    }

    /// Cumulative de-rendered bits.
    fn cumulative_bits(&self) -> f64 {
        self.cumulative_bits_impl()
    }

    /// Remaining de-rendering lifetime budget (bits).
    fn remaining_lifetime_bits(&self) -> f64 {
        self.remaining_lifetime_bits_impl()
    }

    /// Consumed fraction of the Coffin-Manson fatigue life in cycles.
    fn consumed_cycles(&self) -> f64 {
        self.consumed_cycles_impl()
    }

    /// Run the thermal-fatigue audit.
    ///
    /// Returns (status, nominal, remaining_bits, consumed_cycles, shifted_impedance_mrayl).
    fn audit(&self) -> (String, bool, f64, f64, f64) {
        self.audit_impl()
    }

    /// Acoustic impedance the Alumina/InP interface approaches when fatigued.
    fn fatigue_shifted_impedance_mrayl(&self) -> f64 {
        FATIGUE_SHIFTED_IMPEDANCE_MRAYL
    }

    /// Plastic strain and cycle-to-failure constants as a tuple.
    fn coffin_manson_constants(&self) -> (f64, f64, f64, f64) {
        (
            COFFIN_MANSON_PLASTIC_STRAIN,
            THERMAL_FATIGUE_SWING_K,
            CYCLES_TO_FAILURE,
            LIFETIME_BITS_DE_RENDERED,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lifetime_budget_matches_spec() {
        let auditor = ReliabilityAuditor::new();
        let (_, nominal, _, _, _) = auditor.audit_impl();
        assert!(nominal);
    }

    #[test]
    fn quench_warning_after_lifetime_exceeded() {
        let mut auditor = ReliabilityAuditor::new();
        auditor.accumulate_bits_impl(LIFETIME_BITS_DE_RENDERED * 1.01);
        let (status, nominal, remaining, _, impedance) = auditor.audit_impl();
        assert_eq!(status, "STATUS_QUENCH_WARNING");
        assert!(!nominal);
        assert_eq!(remaining, 0.0);
        assert_eq!(impedance, FATIGUE_SHIFTED_IMPEDANCE_MRAYL);
    }

    #[test]
    fn consumed_cycles_linear_with_bits() {
        let mut auditor = ReliabilityAuditor::new();
        auditor.accumulate_bits_impl(LIFETIME_BITS_DE_RENDERED / 2.0);
        let consumed = auditor.consumed_cycles_impl();
        assert!((consumed - CYCLES_TO_FAILURE / 2.0).abs() < 1.0);
    }

    #[test]
    fn constants_match_spec() {
        assert_eq!(COFFIN_MANSON_PLASTIC_STRAIN, 6.0e-6);
        assert_eq!(THERMAL_FATIGUE_SWING_K, 15.0);
        assert_eq!(CYCLES_TO_FAILURE, 4.0e6);
        assert_eq!(LIFETIME_BITS_DE_RENDERED, 1.514e16);
        assert_eq!(FATIGUE_SHIFTED_IMPEDANCE_MRAYL, 1.3250);
    }
}
