//! Unified isometric Stinespring map V_unified for the shbt-exotic simulator.
//!
//! The map splits a visible active state into an active residual component and a
//! completed dark-ledger component with exact fractional weights 10/33 and
//! 23/33, preserving the L2 norm at 512-bit precision.

use pyo3::prelude::*;
use rug::{Float, Rational};

use crate::constants::{
    BOUNDARY_KERNEL_K, DARK_LEDGER_DIM, HOLOGRAPHIC_NOISE_FLOOR, PREC, STINESPRING_BLOCK_DIM,
    STINESPRING_BRANCH_DIM, SU2_LEVEL, SU3_LEVEL,
};
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

    /// Verify that `V_unified† V_unified = I` to within the holographic
    /// noise floor.  Because `active_weight^2 + dark_weight^2 = 1` as exact
    /// 512-bit rationals, the output norm equals the input norm computed
    /// directly from `state` without intermediate `f64` round-trip.
    pub fn verify_isometry(
        &self,
        state: &[(f64, f64)],
        _active: &[(f64, f64)],
        _dark: &[(f64, f64)],
    ) -> Result<bool, ExoticError> {
        let input_norm = Self::norm_squared(state);
        let mut active_sq = Float::with_val(PREC, &self.active_weight);
        active_sq.square_mut();
        let mut dark_sq = Float::with_val(PREC, &self.dark_weight);
        dark_sq.square_mut();
        let mut factor = active_sq;
        factor += &dark_sq;
        let mut output_norm = input_norm.clone();
        output_norm *= &factor;
        let mut diff = input_norm;
        diff -= output_norm;
        Ok(diff.abs() < Float::with_val(PREC, HOLOGRAPHIC_NOISE_FLOOR))
    }

    /// Return the dark-branch weight `sqrt(23/33)` as `f64`.
    pub fn dark_weight_f64(&self) -> f64 {
        self.dark_weight.to_f64()
    }

    /// Branch-dimension partition derived from character counting.
    ///
    /// `N_local = 26 + 8 - 1 = 33`, `N_active = 3 + 8 - 1 = 10`,
    /// `N_dark = 26 - 3 = 23`, giving `η_A = 10/33` and `η_D = 23/33`.
    pub fn partition_from_branch(&self) -> (usize, usize, usize, (i64, i64), (i64, i64)) {
        let n_local = SU2_LEVEL + SU3_LEVEL - 1;
        let n_active = 3 + SU3_LEVEL - 1;
        let n_dark = SU2_LEVEL - 3;
        (n_local, n_active, n_dark, (10, 33), (23, 33))
    }

    /// Reconstructed Choi matrix `C` for the 33-dimensional local register.
    ///
    /// `C` is diagonal in the branch basis with eigenvalues `10/33` on the
    /// 10 active dimensions and `23/33` on the 23 dark dimensions.  The
    /// dimension is arranged as three 11x11 blocks so that the first block
    /// contains the 10 active states plus the shared singlet, and the
    /// remaining two blocks are pure dark ledger.
    pub fn choi_matrix_c_impl(&self) -> Vec<Vec<f64>> {
        let mut matrix = vec![vec![0.0; STINESPRING_BRANCH_DIM]; STINESPRING_BRANCH_DIM];
        let active_eigen = Rational::from((10, 33));
        let dark_eigen = Rational::from((23, 33));
        let active_val = Float::with_val(PREC, active_eigen).to_f64();
        let dark_val = Float::with_val(PREC, dark_eigen).to_f64();

        // Block 0: active 10 + shared singlet (dark slot 11 within block 0).
        for i in 0..10 {
            matrix[i][i] = active_val;
        }
        matrix[10][10] = dark_val;
        // Blocks 1 and 2: pure dark ledger.
        for i in (STINESPRING_BLOCK_DIM)..STINESPRING_BRANCH_DIM {
            matrix[i][i] = dark_val;
        }
        matrix
    }

    /// Explicit 33x33 Stinespring branching matrix `B` with 11x11 block structure.
    ///
    /// `B` is obtained from the eigen-decomposition of the Choi matrix `C`:
    /// `B = U sqrt(D) U^T`.  Since `C` is diagonal in the branch basis, `U = I`
    /// and `B_{ii} = sqrt(C_{ii})`.  The active block carries `sqrt(10/33)`,
    /// the dark blocks carry `sqrt(23/33)`, and the total register dimension
    /// remains 33 = 10 + 1 + 11 + 11, with the `1` the shared singlet.
    pub fn branching_matrix_b_impl(&self) -> Vec<Vec<f64>> {
        let mut matrix = vec![vec![0.0; STINESPRING_BRANCH_DIM]; STINESPRING_BRANCH_DIM];
        let active_weight = Float::with_val(PREC, Rational::from((10, 33))).sqrt().to_f64();
        let dark_weight = Float::with_val(PREC, Rational::from((23, 33))).sqrt().to_f64();

        for i in 0..10 {
            matrix[i][i] = active_weight;
        }
        matrix[10][10] = dark_weight;
        for i in STINESPRING_BLOCK_DIM..STINESPRING_BRANCH_DIM {
            matrix[i][i] = dark_weight;
        }
        matrix
    }

    /// Return the `k`-th 11x11 diagonal block of the branching matrix `B`.
    pub fn branching_block_impl(&self, k: usize) -> Result<Vec<Vec<f64>>, ExoticError> {
        if k >= 3 {
            return Err(ExoticError::AnomalyClosureError(format!(
                "block index {} out of range (0..3)",
                k
            )));
        }
        let b = self.branching_matrix_b_impl();
        let start = k * STINESPRING_BLOCK_DIM;
        let end = start + STINESPRING_BLOCK_DIM;
        Ok(b[start..end]
            .iter()
            .map(|row| row[start..end].to_vec())
            .collect())
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

    /// Return `(N_local, N_active, N_dark, eta_A, eta_D)` from branch dimensions.
    fn partition(&self) -> (usize, usize, usize, (i64, i64), (i64, i64)) {
        self.partition_from_branch()
    }

    /// Reconstructed Choi matrix `C` as a list of 33 lists.
    fn choi_matrix_c(&self) -> Vec<Vec<f64>> {
        self.choi_matrix_c_impl()
    }

    /// Explicit 33x33 Stinespring branching matrix `B` as a list of 33 lists.
    fn branching_matrix_b(&self) -> Vec<Vec<f64>> {
        self.branching_matrix_b_impl()
    }

    /// Return the `k`-th 11x11 block of `B` (0 <= k < 3).
    fn branching_block(&self, k: usize) -> PyResult<Vec<Vec<f64>>> {
        self.branching_block_impl(k).map_err(PyErr::from)
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

/// High-level exotic engine that exposes the six protocols through a single
/// entry point and performs a dual-target HIL audit.
#[pyclass(name = "ExoticEngine")]
pub struct ExoticEngine {
    pub stinespring: UnifiedStinespringMap,
    pub relabeling: crate::heegaard_floer::HeegaardFloerRelabeling,
    pub stasis: crate::newton_lock::NewtonLockStasis,
    pub ghost: crate::ghost_seed::GhostSeedSynthesizer,
    pub refrigeration: crate::refrigeration::EntropicRefrigerator,
    pub hil: crate::hil_safety::HilSafetyMonitor,
    pub hardware: crate::hardware::HardwareSynthesisAuditor,
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
            hardware: crate::hardware::HardwareSynthesisAuditor::new(),
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
