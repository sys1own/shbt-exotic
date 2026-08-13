//! Newton-lock stationarity for temporal stasis.
//!
//! Temporal flow `T_dot` is proportional to `1 / C_get`.  By locally biasing the
//! density multiplier `mu(x) = mu0 + delta_mu(x)`, the GET cost is increased
//! until `C_get` reaches the cosmic Landauer bound `5.34e-175` J/bit, producing
//! `gamma_stasis = C_get_local / C_get_bound`.

use pyo3::prelude::*;
use rug::Float;

use crate::constants::{C_GET_THERMODYNAMIC_BOUND_J, EIGENVECTOR_RIGIDITY_THRESHOLD, PREC};
use crate::error::ExoticError;
use crate::gmp_memory;

#[pyclass(name = "NewtonLockStasis")]
#[derive(Clone, Debug)]
pub struct NewtonLockStasis {
    mu0: f64,
    c_get_scale: f64,
}

impl NewtonLockStasis {
    pub fn new() -> Self {
        gmp_memory::init();
        Self {
            mu0: 1.0,
            // Use the cosmic Landauer bound as the reference scale.  The local
            // C_get is modulated around this value.
            c_get_scale: C_GET_THERMODYNAMIC_BOUND_J,
        }
    }

    /// Validate that the spatial integral of the bias is zero (global modular
    /// invariance).  For a discrete list, this is the plain sum.
    pub fn validate_zero_mean_bias(bias: &[f64]) -> Result<(), ExoticError> {
        let sum: f64 = bias.iter().sum();
        if sum.abs() > 1e-30 {
            return Err(ExoticError::AnomalyClosureError(format!(
                "density-multiplier bias must have zero mean; got sum = {}",
                sum
            )));
        }
        Ok(())
    }

    /// Local GET cost for a given density bias.
    ///
    /// `C_get(x) = C_get_bound * exp(bias(x) / 1e-13)` so that a bias at the
    /// HIL threshold `1e-12` yields `gamma_stasis = exp(10) ≈ 2.2e4`.
    pub fn local_c_get_impl(&self, bias: f64) -> Result<f64, ExoticError> {
        let _ = self.mu0;
        if bias.abs() >= EIGENVECTOR_RIGIDITY_THRESHOLD {
            return Err(ExoticError::RigidityViolationError(format!(
                "density bias {} exceeds HIL rigidity threshold {}",
                bias, EIGENVECTOR_RIGIDITY_THRESHOLD
            )));
        }
        let scale = 1e-13;
        let gamma = (bias / scale).exp();
        Ok(self.c_get_scale * gamma)
    }

    /// Dilation factor `gamma_stasis = C_get_local / C_get_bound`.
    pub fn gamma_stasis_impl(&self, bias: f64) -> Result<f64, ExoticError> {
        self.local_c_get_impl(bias).map(|c| c / self.c_get_scale)
    }

    /// Power required to maintain a stasis volume.
    /// `P_stasis = N_active * S_dot_frozen * T_boundary`.
    pub fn stasis_power_impl(&self, n_active: f64, s_dot_frozen: f64, t_boundary: f64) -> f64 {
        n_active * s_dot_frozen * t_boundary
    }

    /// 512-bit verification that `T_dot ∝ 1 / C_get` vanishes when `C_get` is at
    /// the bound (history crystallization halted).
    pub fn is_locked_impl(&self, bias: f64) -> Result<bool, ExoticError> {
        let c_get = self.local_c_get_impl(bias)?;
        let bound = Float::with_val(PREC, self.c_get_scale);
        let c = Float::with_val(PREC, c_get);
        Ok(c >= bound)
    }
}

#[pymethods]
impl NewtonLockStasis {
    #[new]
    pub fn py_new() -> Self {
        Self::new()
    }

    /// Set and validate a list of local density biases.
    fn set_bias_field(&mut self, bias: Vec<f64>) -> PyResult<()> {
        Self::validate_zero_mean_bias(&bias).map_err(PyErr::from)
    }

    fn gamma_stasis(&self, bias: f64) -> PyResult<f64> {
        self.gamma_stasis_impl(bias).map_err(PyErr::from)
    }

    fn local_c_get(&self, bias: f64) -> PyResult<f64> {
        self.local_c_get_impl(bias).map_err(PyErr::from)
    }

    fn stasis_power(&self, n_active: f64, s_dot_frozen: f64, t_boundary: f64) -> f64 {
        self.stasis_power_impl(n_active, s_dot_frozen, t_boundary)
    }

    fn is_locked(&self, bias: f64) -> PyResult<bool> {
        self.is_locked_impl(bias).map_err(PyErr::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stasis_factor_increases_with_bias() {
        let stasis = NewtonLockStasis::new();
        let gamma0 = stasis.gamma_stasis_impl(0.0).unwrap();
        let gamma1 = stasis.gamma_stasis_impl(1e-15).unwrap();
        assert!(gamma1 > gamma0);
    }

    #[test]
    fn zero_mean_bias_passes() {
        NewtonLockStasis::validate_zero_mean_bias(&[0.1, -0.05, -0.05]).unwrap();
    }

    #[test]
    fn non_zero_mean_bias_fails() {
        assert!(NewtonLockStasis::validate_zero_mean_bias(&[0.1, 0.05, 0.05]).is_err());
    }
}
