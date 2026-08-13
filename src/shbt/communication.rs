//! Heegaard mapping torus and Kojima inequality for non-local communication.
//!
//! The relabeling isometry `T^∂` moves boundary addresses along a leaf of the
//! Heegaard foliation.  Kojima's inequality bounds the entropy (translation
//! length) `Ent(φ)` of a mapping-class isometry `φ` by the hyperbolic volume
//! of the ambient mapping torus `M`:
//!
//!   Ent(φ) ≤ C · Vol(M).
//!
//! For a flat boundary leaf the hyperbolic volume vanishes, so the bound is
//! zero and the isometry must have zero entropy.  The simulator therefore
//! requires `ΔS_A = 0` for any admissible address shift.

use pyo3::prelude::*;
use rug::Float;

use crate::constants::{DARK_LEDGER_DIM, EIGENVECTOR_RIGIDITY_THRESHOLD, PREC};
use crate::error::ExoticError;
use crate::gmp_memory;

/// Heegaard mapping torus that evaluates the presentation length of a boundary
/// address shift and checks Kojima's entropy-volume inequality.
#[pyclass(name = "HeegaardMappingTorus")]
#[derive(Clone, Debug)]
pub struct HeegaardMappingTorus {
    /// Hyperbolic volume of the mapping torus.  For a flat boundary leaf this
    /// is exactly 0, making Kojima's bound 0.
    volume: f64,
    /// Constant of proportionality in `Ent(φ) ≤ C · Vol(M)`.
    kojima_constant: f64,
}

impl HeegaardMappingTorus {
    pub fn new() -> Self {
        gmp_memory::init();
        Self {
            volume: 0.0,
            kojima_constant: 1.0,
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

    /// Kojima bound `C · Vol(M)`.
    pub fn kojima_bound_impl(&self) -> f64 {
        self.kojima_constant * self.volume
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
        let bound = self.kojima_bound_impl();
        // For the flat boundary leaf Vol(M) = 0, so the bound is 0 up to the
        // HIL noise threshold.  Any non-zero ΔS_A violates Kojima's inequality.
        let satisfies = delta_s <= bound + EIGENVECTOR_RIGIDITY_THRESHOLD;
        Ok((ell_he, delta_s, satisfies))
    }
}

#[pymethods]
impl HeegaardMappingTorus {
    #[new]
    pub fn py_new() -> Self {
        Self::new()
    }

    /// Presentation length `ℓ_He` of `state`.
    fn presentation_length(&self, state: Vec<(f64, f64)>) -> PyResult<f64> {
        self.presentation_length_impl(&state).map_err(PyErr::from)
    }

    /// Entropy change `ΔS_A` between `source` and `target`.
    fn entropy_change(&self, source: Vec<(f64, f64)>, target: Vec<(f64, f64)>) -> PyResult<f64> {
        self.entropy_change_impl(&source, &target).map_err(PyErr::from)
    }

    /// Kojima bound `C · Vol(M)` for the flat boundary leaf.
    fn kojima_bound(&self) -> f64 {
        self.kojima_bound_impl()
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
}
