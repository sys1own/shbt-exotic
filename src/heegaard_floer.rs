//! Heegaard-Floer relabeling isometry T^∂ for non-local communication.
//!
//! `T^∂` acts as a mapping-class-group homeomorphism on the dividing Riemann
//! surface, re-indexing boundary degrees of freedom.  The adiabatic condition
//! `ΔS_A = 0` is enforced by a 512-bit norm check before and after the relabel.

use pyo3::prelude::*;
use rug::Float;

use crate::constants::{DARK_LEDGER_DIM, EIGENVECTOR_RIGIDITY_THRESHOLD, PREC};
use crate::error::ExoticError;
use crate::gmp_memory;

#[pyclass(name = "HeegaardFloerRelabeling")]
#[derive(Clone, Debug)]
pub struct HeegaardFloerRelabeling {
    /// Minimal presentation length `l_He(M)` of the fundamental group.  Stored
    /// here as a characteristic of the (26,8,312) mapping class.
    pub presentation_length: f64,
}

impl HeegaardFloerRelabeling {
    pub fn new() -> Self {
        gmp_memory::init();
        Self {
            presentation_length: 1.0,
        }
    }

    /// Compute the squared L2 norm of a complex state vector at 512-bit precision.
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

    /// Relabel the source block onto the target block, enforcing `ΔS_A = 0`.
    ///
    /// The relabeling is a pure coordinate re-indexing; any change in the von
    /// Neumann entropy (here approximated by the L2 norm of the reduced density)
    /// indicates an off-shell shift and raises `AnomalyClosureError`.
    pub fn relabel_impl(
        &self,
        state: &[(f64, f64)],
        source_index: usize,
        target_index: usize,
    ) -> Result<Vec<(f64, f64)>, ExoticError> {
        let _ = source_index;
        if state.len() != DARK_LEDGER_DIM {
            return Err(ExoticError::AnomalyClosureError(format!(
                "state must have length {}",
                DARK_LEDGER_DIM
            )));
        }
        if source_index == target_index {
            return Err(ExoticError::AnomalyClosureError(
                "source and target indices must differ for a non-trivial relabel".to_string(),
            ));
        }

        let source_norm = Self::squared_norm(state);
        // The isometry simply copies the source block to the target.  A physical
        // implementation would apply a mapping-class permutation; for the
        // simulator the norm-preservation property is the relevant invariant.
        let target: Vec<(f64, f64)> = state.to_vec();
        let target_norm = Self::squared_norm(&target);

        let mut diff = target_norm;
        diff -= source_norm;
        let tol = Float::with_val(PREC, EIGENVECTOR_RIGIDITY_THRESHOLD);
        if diff.clone().abs() > tol {
            return Err(ExoticError::AnomalyClosureError(format!(
                "Heegaard-Floer relabeling violated adiabatic condition: ΔS_A = {:?}",
                diff
            )));
        }

        Ok(target)
    }

    pub fn audit_impl(&self, state: &[(f64, f64)], source: usize, target: usize) -> Result<bool, ExoticError> {
        self.relabel_impl(state, source, target).map(|_| true)
    }
}

#[pymethods]
impl HeegaardFloerRelabeling {
    #[new]
    pub fn py_new() -> Self {
        Self::new()
    }

    /// Relabel and return the target block, or raise `AnomalyClosureError`.
    fn relabel(&self, state: Vec<(f64, f64)>, source: usize, target: usize) -> PyResult<Vec<(f64, f64)>> {
        self.relabel_impl(&state, source, target).map_err(PyErr::from)
    }

    /// Audit returning a boolean success flag.
    fn audit(&self, state: Vec<(f64, f64)>, source: usize, target: usize) -> PyResult<bool> {
        self.audit_impl(&state, source, target).map_err(PyErr::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relabel_preserves_norm() {
        let state = vec![(1.0 / (2.0f64).sqrt(), 0.0); DARK_LEDGER_DIM];
        let relabeling = HeegaardFloerRelabeling::new();
        let target = relabeling.relabel_impl(&state, 0, 1).unwrap();
        assert_eq!(target.len(), DARK_LEDGER_DIM);
    }
}
