//! Heegaard mapping torus and Kojima inequality for non-local communication.
//!
//! The relabeling isometry `T^∂` moves boundary addresses along a leaf of the
//! Heegaard foliation.  Kojima's inequality bounds the entropy (translation
//! length) `Ent(φ)` of a mapping-class isometry `φ` by the hyperbolic volume
//! of the ambient mapping torus `M`:
//!
//!   Ent(φ) ≤ C · Vol(M).
//!
//! For arithmetic hyperbolic manifolds the bound can be sharpened to a linear
//! expression in the Heegaard presentation length:
//!
//!   Ent(φ_1) ≤ [M_1 : M] · (ℓ_He(M) - 1) · log 3.
//!
//! For a flat boundary leaf the hyperbolic volume vanishes, so both bounds are
//! zero and the isometry must have zero entropy.  The simulator therefore
//! requires `ΔS_A = 0` for any admissible address shift.

use pyo3::prelude::*;
use rug::Float;

use crate::constants::{DARK_LEDGER_DIM, EIGENVECTOR_RIGIDITY_THRESHOLD, PREC};
use crate::error::ExoticError;
use crate::gmp_memory;

/// Universal geometric constant used in the general-hyperbolic Kojima bound
/// `Ent(φ) ≤ C · Vol(M)`.
pub const KOJIMA_GEOMETRIC_CONSTANT: f64 = 1.0e20;

/// Heegaard mapping torus that evaluates the presentation length of a boundary
/// address shift and checks Kojima's entropy-volume inequality.
#[pyclass(name = "HeegaardMappingTorus")]
#[derive(Clone, Debug)]
pub struct HeegaardMappingTorus {
    /// Hyperbolic volume of the mapping torus.  For a flat boundary leaf this
    /// is exactly 0, making the hyperbolic Kojima bound 0.
    volume: f64,
    /// Constant of proportionality in `Ent(φ) ≤ C · Vol(M)`.
    kojima_constant: f64,
    /// Covering index `[M_1 : M]` appearing in the arithmetic bound.
    covering_index: f64,
}

impl HeegaardMappingTorus {
    pub fn new() -> Self {
        gmp_memory::init();
        Self {
            volume: 0.0,
            kojima_constant: KOJIMA_GEOMETRIC_CONSTANT,
            covering_index: 1.0,
        }
    }

    /// Squared L2 norm of a complex state vector at 512-bit precision.
    fn squared_norm(state: &[(f64, f64)]) -> Float {
        let mut norm = Float::with_val(PREC, 0);
        for &(re, im) in state.iter() {
            let mut r = Float::with_val(PREC, re);
            r.square_mut();
            let mut i = Float::with_val(PREC, im);
            i.square_mut();
            norm += r;
            norm += i;
        }
        norm
    }

    /// Heegaard presentation length `ℓ_He` of the source state.
    ///
    /// We take the translation-length proxy to be `sqrt(‖ψ‖²)`, which equals
    /// 1 for a normalized visible state.
    pub fn presentation_length_impl(&self, state: &[(f64, f64)]) -> Result<f64, ExoticError> {
        if state.len() != DARK_LEDGER_DIM {
            return Err(ExoticError::AnomalyClosureError(format!(
                "Heegaard mapping torus state must have length {}",
                DARK_LEDGER_DIM
            )));
        }
        let norm = Self::squared_norm(state);
        Ok(Float::with_val(PREC, norm).sqrt().to_f64())
    }

    /// Entropy change `ΔS_A = ‖ψ_tar‖² - ‖ψ_src‖²` along the mapping torus.
    pub fn entropy_change_impl(
        &self,
        source: &[(f64, f64)],
        target: &[(f64, f64)],
    ) -> Result<f64, ExoticError> {
        if source.len() != DARK_LEDGER_DIM || target.len() != DARK_LEDGER_DIM {
            return Err(ExoticError::AnomalyClosureError(format!(
                "source and target states must have length {}",
                DARK_LEDGER_DIM
            )));
        }
        let source_norm = Self::squared_norm(source);
        let target_norm = Self::squared_norm(target);
        let mut diff = target_norm;
        diff -= source_norm;
        Ok(diff.to_f64())
    }

    /// General hyperbolic Kojima bound `C · Vol(M)`.
    pub fn kojima_bound_impl(&self) -> f64 {
        self.kojima_constant * self.volume
    }

    /// Arithmetic Kojima bound for the entropy of a mapping-class isometry:
    ///
    ///   Ent(φ_1) ≤ [M_1 : M] · (ℓ_He(M) - 1) · log 3.
    pub fn entropy_bound_arithmetic_impl(&self, presentation_length: f64) -> f64 {
        self.covering_index * (presentation_length - 1.0).max(0.0) * 3.0_f64.ln()
    }

    /// Evaluate the Kojima inequality for an address shift from `source` to
    /// `target`.  Returns `(ℓ_He, ΔS_A, satisfies_kojima)`.
    pub fn evaluate_impl(
        &self,
        source: &[(f64, f64)],
        target: &[(f64, f64)],
    ) -> Result<(f64, f64, bool), ExoticError> {
        let ell_he = self.presentation_length_impl(source)?;
        let delta_s = self.entropy_change_impl(source, target)?;
        let hyperbolic_bound = self.kojima_bound_impl();
        let arithmetic_bound = self.entropy_bound_arithmetic_impl(ell_he);
        // For the flat boundary leaf both bounds are 0 up to the HIL noise
        // threshold.  Any non-zero ΔS_A violates the Kojima inequalities.
        let satisfies =
            delta_s <= hyperbolic_bound + EIGENVECTOR_RIGIDITY_THRESHOLD
                && delta_s <= arithmetic_bound + EIGENVECTOR_RIGIDITY_THRESHOLD;
        Ok((ell_he, delta_s, satisfies))
    }
}

#[pymethods]
impl HeegaardMappingTorus {
    #[new]
    pub fn py_new() -> Self {
        Self::new()
    }

    /// Set the covering index `[M_1 : M]` for the arithmetic Kojima bound.
    fn set_covering_index(&mut self, index: f64) {
        self.covering_index = index.max(0.0);
    }

    /// Get the universal geometric constant `C = 10^20`.
    fn kojima_geometric_constant(&self) -> f64 {
        self.kojima_constant
    }

    /// Set the hyperbolic volume `Vol(M)`.
    fn set_volume(&mut self, volume: f64) {
        self.volume = volume.max(0.0);
    }

    /// Presentation length `ℓ_He` of `state`.
    fn presentation_length(&self, state: Vec<(f64, f64)>) -> PyResult<f64> {
        self.presentation_length_impl(&state).map_err(PyErr::from)
    }

    /// Entropy change `ΔS_A` between `source` and `target`.
    fn entropy_change(&self, source: Vec<(f64, f64)>, target: Vec<(f64, f64)>) -> PyResult<f64> {
        self.entropy_change_impl(&source, &target).map_err(PyErr::from)
    }

    /// Hyperbolic Kojima bound `C · Vol(M)`.
    fn kojima_bound(&self) -> f64 {
        self.kojima_bound_impl()
    }

    /// Arithmetic Kojima bound `[M_1 : M] · (ℓ_He - 1) · log 3`.
    fn entropy_bound_arithmetic(&self, presentation_length: f64) -> f64 {
        self.entropy_bound_arithmetic_impl(presentation_length)
    }

    /// Evaluate `(ℓ_He, ΔS_A, satisfies_kojima)` for an address shift.
    fn evaluate(
        &self,
        source: Vec<(f64, f64)>,
        target: Vec<(f64, f64)>,
    ) -> PyResult<(f64, f64, bool)> {
        self.evaluate_impl(&source, &target).map_err(PyErr::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flat_leaf_kojima_inequality_satisfied_for_isometric_shift() {
        let torus = HeegaardMappingTorus::new();
        let norm = (DARK_LEDGER_DIM as f64).sqrt();
        let state = vec![(1.0 / norm, 0.0); DARK_LEDGER_DIM];
        let (ell_he, delta_s, ok) = torus.evaluate_impl(&state, &state).unwrap();
        assert!((ell_he - 1.0).abs() < 1e-15);
        assert!(delta_s.abs() < 1e-15);
        assert!(ok);
    }

    #[test]
    fn flat_leaf_kojima_fails_when_entropy_changes() {
        let torus = HeegaardMappingTorus::new();
        let norm = (DARK_LEDGER_DIM as f64).sqrt();
        let source = vec![(1.0 / norm, 0.0); DARK_LEDGER_DIM];
        let mut target = source.clone();
        target[0].0 += 1e-3;
        let (_, delta_s, ok) = torus.evaluate_impl(&source, &target).unwrap();
        assert!(!ok);
        assert!(delta_s.abs() > 1e-12);
    }

    #[test]
    fn arithmetic_bound_is_zero_for_flat_leaf() {
        let torus = HeegaardMappingTorus::new();
        let bound = torus.entropy_bound_arithmetic_impl(1.0);
        assert!(bound.abs() < 1e-15);
    }

    #[test]
    fn hyperbolic_bound_scales_with_volume() {
        let mut torus = HeegaardMappingTorus::new();
        torus.set_volume(1.0);
        assert!((torus.kojima_bound_impl() - KOJIMA_GEOMETRIC_CONSTANT).abs() < 1e-6);
    }
}
