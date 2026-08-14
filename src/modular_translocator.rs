//! Modular state translocator: de-render / causal-authorize / re-render cycle.
//!
//! Implements the reconstruction operator
//!
//!   R^rerender = T^∂ O^excitation D^derender†
//!
//! where `D^derender` is the Stinespring dark-ledger projection, `O` is a
//! U(1) phase-locked excitation, and `T^∂` is a boundary-address relabeling
//! isometry.  Causal authorization `x_tar ∈ J^+(x_src)` is enforced before
//! any state is moved.

use pyo3::prelude::*;

use crate::causal_coordinate::{CausalCoordinate, verify_future_cone_fatal};
use crate::constants::{DARK_LEDGER_DIM, HOLOGRAPHIC_NOISE_FLOOR, PREC};
use crate::error::ExoticError;
use crate::gmp_memory;
use crate::heegaard_floer::HeegaardFloerRelabeling;
use crate::phase_rotation;
use crate::stinespring::UnifiedStinespringMap;
use rug::Float;

/// Production-grade modular state translocator.
#[pyclass(name = "ModularStateTranslocator")]
#[derive(Clone, Debug)]
pub struct ModularStateTranslocator {
    stinespring: UnifiedStinespringMap,
    relabeling: HeegaardFloerRelabeling,
}

impl ModularStateTranslocator {
    pub fn new() -> Self {
        gmp_memory::init();
        Self {
            stinespring: UnifiedStinespringMap::new(),
            relabeling: HeegaardFloerRelabeling::new(),
        }
    }

    fn norm_squared(state: &[(f64, f64)]) -> Float {
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

    /// Verify that `state` is normalized to within the eigenvector rigidity tolerance.
    pub fn check_rigidity(&self, state: &[(f64, f64)]) -> Result<(), ExoticError> {
        let norm = Self::norm_squared(state);
        let one = Float::with_val(PREC, 1);
        let mut diff = norm;
        diff -= &one;
        if diff.abs() > Float::with_val(PREC, crate::constants::EIGENVECTOR_RIGIDITY_THRESHOLD) {
            return Err(ExoticError::RigidityViolationError(
                "input state detuned past 10^-12".to_string(),
            ));
        }
        Ok(())
    }

    /// Run the full de-render / re-render translocation pipeline.
    ///
    /// `state` is an 8-component visible residual; `src` and `tar` are causal
    /// coordinates; `theta` is the phase-locked excitation angle.  The passive
    /// stress-energy component (active residual) is returned alongside the
    /// reconstructed dark-ledger state so that `T_{μν}^{passive}` is preserved.
    pub fn translocate_impl(
        &self,
        state: &[(f64, f64)],
        src: &CausalCoordinate,
        tar: &CausalCoordinate,
        theta: f64,
    ) -> Result<(Vec<(f64, f64)>, Vec<(f64, f64)>), ExoticError> {
        if state.len() != DARK_LEDGER_DIM {
            return Err(ExoticError::AnomalyClosureError(format!(
                "state must have length {}",
                DARK_LEDGER_DIM
            )));
        }

        self.check_rigidity(state)?;

        // Causal authorization: the target must lie in the future light-cone.
        verify_future_cone_fatal(src, tar)?;

        // D^derender: split into active/passive components.  The active
        // residual is the passive stress-energy that remains behind.
        let (active, dark) = self.stinespring.apply(state)?;

        // T^∂ O^excitation: relabel and phase-rotate the dark component.
        let relabeled = self.relabeling.relabel_impl(&dark, 0, 1)?;
        let mut re = [0.0; DARK_LEDGER_DIM];
        let mut im = [0.0; DARK_LEDGER_DIM];
        for (i, &(r, j)) in relabeled.iter().enumerate() {
            re[i] = r;
            im[i] = j;
        }
        phase_rotation::rotate_block(theta, &mut re, &mut im);

        // D^derender†: project the rotated dark ledger back to visible space.
        // The Stinespring isometry uses sqrt(23/33) for the dark branch, so
        // the adjoint multiplies by the same factor.
        let dark_weight = self.stinespring.dark_weight_f64();
        let mut reconstructed = Vec::with_capacity(DARK_LEDGER_DIM);
        for i in 0..DARK_LEDGER_DIM {
            reconstructed.push((re[i] * dark_weight, im[i] * dark_weight));
        }

        // Guard against collapse below the holographic noise floor.
        let mut amp_sq = Float::with_val(PREC, 0);
        for &(r, j) in &reconstructed {
            let mut a = Float::with_val(PREC, r);
            a.square_mut();
            let mut b = Float::with_val(PREC, j);
            b.square_mut();
            amp_sq += a;
            amp_sq += b;
        }
        if amp_sq < Float::with_val(PREC, HOLOGRAPHIC_NOISE_FLOOR) {
            return Err(ExoticError::PrecisionLossError(
                "Reconstructed state collapsed below holographic noise floor".to_string(),
            ));
        }

        Ok((reconstructed, active))
    }

    /// Convenience: attempt translocation and return only the reconstructed state.
    pub fn translocate_state_impl(
        &self,
        state: &[(f64, f64)],
        src: &CausalCoordinate,
        tar: &CausalCoordinate,
        theta: f64,
    ) -> Result<Vec<(f64, f64)>, ExoticError> {
        self.translocate_impl(state, src, tar, theta).map(|(rec, _)| rec)
    }
}

#[pymethods]
impl ModularStateTranslocator {
    #[new]
    pub fn py_new() -> Self {
        Self::new()
    }

    /// Python-facing translocation returning `(reconstructed, passive_active)`.
    fn translocate(
        &self,
        state: Vec<(f64, f64)>,
        src: CausalCoordinate,
        tar: CausalCoordinate,
        theta: f64,
    ) -> PyResult<(Vec<(f64, f64)>, Vec<(f64, f64)>)> {
        self.translocate_impl(&state, &src, &tar, theta)
            .map_err(PyErr::from)
    }

    /// Convenience Python method returning only the reconstructed state.
    fn translocate_state(
        &self,
        state: Vec<(f64, f64)>,
        src: CausalCoordinate,
        tar: CausalCoordinate,
        theta: f64,
    ) -> PyResult<Vec<(f64, f64)>> {
        self.translocate_state_impl(&state, &src, &tar, theta)
            .map_err(PyErr::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unit_state() -> Vec<(f64, f64)> {
        let n = (DARK_LEDGER_DIM as f64).sqrt();
        vec![(1.0 / n, 0.0); DARK_LEDGER_DIM]
    }

    #[test]
    fn translocate_future_point_passes() {
        let trans = ModularStateTranslocator::new();
        let src = CausalCoordinate::new(0.0, 0.0, 0.0, 0.0);
        let tar = CausalCoordinate::new(1.0, 0.0, 0.0, 0.0);
        let result = trans.translocate_impl(&unit_state(), &src, &tar, 0.421);
        assert!(result.is_ok());
    }

    #[test]
    fn translocate_rejects_spacelike_target() {
        let trans = ModularStateTranslocator::new();
        let src = CausalCoordinate::new(0.0, 0.0, 0.0, 0.0);
        let tar = CausalCoordinate::new(0.0, 2.0, 0.0, 0.0);
        let result = trans.translocate_impl(&unit_state(), &src, &tar, 0.0);
        assert!(result.is_err());
    }
}
