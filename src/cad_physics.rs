//! CAD-to-physics consistency checks.
//!
//! Cross-references exported GDSII layout dimensions against simulated
//! structural resonance modes to catch geometries that would excite critical
//! modes during high-energy transients.

use pyo3::prelude::*;

use crate::error::ExoticError;
use crate::harmonic_audit::{HarmonicAuditor, FLEXURAL_MODE_HZ};

/// Relative tolerance for flexural-mode resonance coincidence.
pub const RESONANCE_TOLERANCE_HZ: f64 = 0.10 * FLEXURAL_MODE_HZ;

/// Validator that links GDSII feature dimensions to the FEA resonance audit.
#[pyclass(name = "CadPhysicsValidator")]
#[derive(Clone, Debug)]
pub struct CadPhysicsValidator;

impl CadPhysicsValidator {
    pub fn new() -> Self {
        Self
    }

    /// Estimate the natural flexural frequency of a rectangular beam with the
    /// given length, width, and height (all in metres) and raise a
    /// `DesignRuleViolation` if it lies within the resonance tolerance of the
    /// critical 19.82 MHz flexural mode during a power transient.
    pub fn validate_airbridge_resonance_impl(
        &self,
        length_m: f64,
        width_m: f64,
        height_m: f64,
    ) -> Result<f64, ExoticError> {
        if length_m <= 0.0 || width_m <= 0.0 || height_m <= 0.0 {
            return Err(ExoticError::DesignRuleViolation(
                "airbridge dimensions must be positive".to_string(),
            ));
        }
        let auditor = HarmonicAuditor::new();
        let freqs = auditor.estimate_natural_frequencies_hz_impl(length_m, width_m, height_m);
        let flex_hz = freqs
            .into_iter()
            .find(|(name, _)| name.eq_ignore_ascii_case("flexural"))
            .map(|(_, f)| f)
            .ok_or_else(|| {
                ExoticError::DesignRuleViolation(
                    "could not estimate flexural frequency".to_string(),
                )
            })?;
        if (flex_hz - FLEXURAL_MODE_HZ).abs() <= RESONANCE_TOLERANCE_HZ {
            return Err(ExoticError::DesignRuleViolation(format!(
                "airbridge flexural mode {:.3e} Hz coincides with critical {:.3e} Hz",
                flex_hz, FLEXURAL_MODE_HZ
            )));
        }
        Ok(flex_hz)
    }

    /// Cross-check the exported airbridge dimensions (μm) against the critical
    /// 19.82 MHz flexural mode.  The default 1.5×5.0 μm airbridge is safe;
    /// longer bridges may excite the mode.
    pub fn validate_airbridge_um_impl(
        &self,
        length_um: f64,
        width_um: f64,
        height_um: f64,
    ) -> Result<f64, ExoticError> {
        self.validate_airbridge_resonance_impl(
            length_um * 1e-6,
            width_um * 1e-6,
            height_um * 1e-6,
        )
    }
}

#[pymethods]
impl CadPhysicsValidator {
    #[new]
    fn py_new() -> Self {
        Self::new()
    }

    /// Validate a rectangular airbridge feature against the critical flexural mode.
    ///
    /// Dimensions are in metres.  Returns the estimated flexural frequency (Hz)
    /// if the feature is safe, or raises `DesignRuleViolation`.
    fn validate_airbridge_resonance(
        &self,
        length_m: f64,
        width_m: f64,
        height_m: f64,
    ) -> PyResult<f64> {
        self.validate_airbridge_resonance_impl(length_m, width_m, height_m)
            .map_err(PyErr::from)
    }

    /// Convenience validator accepting micrometre dimensions.
    fn validate_airbridge_um(
        &self,
        length_um: f64,
        width_um: f64,
        height_um: f64,
    ) -> PyResult<f64> {
        self.validate_airbridge_um_impl(length_um, width_um, height_um)
            .map_err(PyErr::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_airbridge_is_safe() {
        let validator = CadPhysicsValidator::new();
        let f = validator
            .validate_airbridge_um_impl(5.0, 1.5, 0.3)
            .unwrap();
        assert!(f > 0.0);
    }

    #[test]
    fn long_airbridge_excites_flexural_mode() {
        let validator = CadPhysicsValidator::new();
        // A ~12 μm airbridge length drops the flexural mode close to 19.82 MHz.
        let result = validator.validate_airbridge_um_impl(12.0, 1.5, 0.3);
        assert!(result.is_err());
    }
}
