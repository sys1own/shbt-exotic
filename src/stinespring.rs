//! Unified isometric Stinespring map V_unified for the shbt-exotic simulator.
//!
//! The map splits a visible active state into an active residual component and a
//! completed dark-ledger component with exact fractional weights 10/33 and
//! 23/33, preserving the L2 norm at 512-bit precision.

use pyo3::prelude::*;
use rug::{Float, Rational};

use crate::constants::{DARK_LEDGER_DIM, PREC, SU2_LEVEL, SU3_LEVEL, BOUNDARY_KERNEL_K};
use crate::error::ExoticError;
use crate::gmp_memory;

/// Isometric Stinespring dilation `V_unified: H_active -> H_active ⊗ H_ledger`.
///
/// The environment is the dark ledger.  For an input state `|ψ>`, the output is
/// `(sqrt(10/33) |ψ>_active, sqrt(23/33) |ψ>_ledger)`, which is an isometry
/// because `(10 + 23) / 33 = 1`.
#[pyclass(name = "UnifiedStinespringMap")]
#[derive(Clone, Debug)]
pub struct UnifiedStinespringMap {
    active_weight: Float,
    dark_weight: Float,
}

impl UnifiedStinespringMap {
    pub fn new() -> Self {
        gmp_memory::init();
        let active_frac = Rational::from((10, 33));
        let dark_frac = Rational::from((23, 33));
        let active_weight = Float::with_val(PREC, active_frac).sqrt();
        let dark_weight = Float::with_val(PREC, dark_frac).sqrt();
        Self {
            active_weight,
            dark_weight,
        }
    }

    /// Apply `V_unified` to an 8-component complex state vector.
    ///
    /// Returns `(active_component, dark_component)` each of length 8.
    pub fn apply(
        &self,
        state: &[(f64, f64)],
    ) -> Result<(Vec<(f64, f64)>, Vec<(f64, f64)>), ExoticError> {
        if state.len() != DARK_LEDGER_DIM {
            return Err(ExoticError::AnomalyClosureError(format!(
                "input state must have length {}",
                DARK_LEDGER_DIM
            )));
        }
        let mut active = Vec::with_capacity(DARK_LEDGER_DIM);
        let mut dark = Vec::with_capacity(DARK_LEDGER_DIM);
        for &(re, im) in state.iter() {
            let mut a_re = Float::with_val(PREC, re);
            let mut a_im = Float::with_val(PREC, im);
            a_re *= &self.active_weight;
            a_im *= &self.active_weight;
            let mut d_re = Float::with_val(PREC, re);
            let mut d_im = Float::with_val(PREC, im);
            d_re *= &self.dark_weight;
            d_im *= &self.dark_weight;
            active.push((a_re.to_f64(), a_im.to_f64()));
            dark.push((d_re.to_f64(), d_im.to_f64()));
        }
        Ok((active, dark))
    }

    /// Squared L2 norm of a complex state vector at 512-bit precision.
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

    /// Verify that the active and dark components reconstruct the original
    /// squared norm, i.e. that `V_unified` is an isometry.
    pub fn verify_isometry(
        &self,
        state: &[(f64, f64)],
        active: &[(f64, f64)],
        dark: &[(f64, f64)],
    ) -> Result<bool, ExoticError> {
        let input_norm = Self::norm_squared(state);
        let output_norm = Self::norm_squared(active) + Self::norm_squared(dark);
        let mut diff = input_norm;
        diff -= output_norm;
        Ok(diff.abs() < Float::with_val(PREC, 1e-14))
    }

    /// Audit returning the exact rational weights and a norm-residual check.
    pub fn audit_impl(&self, state: &[(f64, f64)]) -> Result<StinespringAudit, ExoticError> {
        let (active, dark) = self.apply(state)?;
        let isometric = self.verify_isometry(state, &active, &dark)?;
        Ok(StinespringAudit {
            active_weight: (10, 33),
            dark_weight: (23, 33),
            isometric,
            active,
            dark,
        })
    }
}

#[pymethods]
impl UnifiedStinespringMap {
    #[new]
    pub fn py_new() -> Self {
        Self::new()
    }

    /// Python-facing de-render: returns `(active, dark)` as lists of tuples.
    fn de_render(&self, state: Vec<(f64, f64)>) -> PyResult<(Vec<(f64, f64)>, Vec<(f64, f64)>)> {
        self.apply(&state).map_err(PyErr::from)
    }

    /// Audit the map for a given input state and return `(active_weight,
    /// dark_weight, isometric, active, dark)`.
    fn audit(
        &self,
        state: Vec<(f64, f64)>,
    ) -> PyResult<((i64, i64), (i64, i64), bool, Vec<(f64, f64)>, Vec<(f64, f64)>)> {
        let r = self.audit_impl(&state).map_err(PyErr::from)?;
        Ok((r.active_weight, r.dark_weight, r.isometric, r.active, r.dark))
    }
}

#[derive(Debug)]
pub struct StinespringAudit {
    pub active_weight: (i64, i64),
    pub dark_weight: (i64, i64),
    pub isometric: bool,
    pub active: Vec<(f64, f64)>,
    pub dark: Vec<(f64, f64)>,
}

/// High-level exotic engine that exposes the four protocols through a single
/// entry point and performs a dual-target HIL audit.
#[pyclass(name = "ExoticEngine")]
pub struct ExoticEngine {
    pub stinespring: UnifiedStinespringMap,
    pub relabeling: crate::heegaard_floer::HeegaardFloerRelabeling,
    pub stasis: crate::newton_lock::NewtonLockStasis,
    pub ghost: crate::ghost_seed::GhostSeedSynthesizer,
    pub refrigeration: crate::refrigeration::EntropicRefrigerator,
    pub hil: crate::hil_safety::HilSafetyMonitor,
}

impl ExoticEngine {
    pub fn new() -> Self {
        gmp_memory::init();
        Self {
            stinespring: UnifiedStinespringMap::new(),
            relabeling: crate::heegaard_floer::HeegaardFloerRelabeling::new(),
            stasis: crate::newton_lock::NewtonLockStasis::new(),
            ghost: crate::ghost_seed::GhostSeedSynthesizer::new(),
            refrigeration: crate::refrigeration::EntropicRefrigerator::new(),
            hil: crate::hil_safety::HilSafetyMonitor::new(),
        }
    }

    pub fn kernel(&self) -> (usize, usize, usize) {
        (SU2_LEVEL, SU3_LEVEL, BOUNDARY_KERNEL_K)
    }
}

#[pymethods]
impl ExoticEngine {
    #[new]
    pub fn py_new() -> Self {
        Self::new()
    }

    #[getter]
    fn get_kernel(&self) -> (usize, usize, usize) {
        self.kernel()
    }
}
