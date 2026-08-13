//! Artificial ghost-seed synthesis via the Mass-Congestion Coupling Identity.
//!
//! `M_seed = α_seed (N_local - N_limit)` where `α_seed` is the UV-cutoff
//! residue coupling constant and `N_limit` is the holographic entropy limit of
//! the localized region.

use pyo3::prelude::*;

use crate::constants::{ALPHA_SEED_M_SUN_PER_BIT, M_SUN_KG};
use crate::error::ExoticError;
use crate::gmp_memory;

#[pyclass(name = "GhostSeedSynthesizer")]
#[derive(Clone, Debug)]
pub struct GhostSeedSynthesizer {
    /// Allowed non-Abelian anyon filling factors (Fibonacci-like) that can
    /// sustain the required state density.
    pub allowed_filling_factors: Vec<(i64, i64)>,
}

impl GhostSeedSynthesizer {
    pub fn new() -> Self {
        gmp_memory::init();
        Self {
            allowed_filling_factors: vec![(12, 5), (4, 7)],
        }
    }

    /// Mass-Congestion Coupling Identity.
    ///
    /// `n_local` and `n_limit` are bit counts.  The result is returned in solar
    /// masses; use `seed_mass_kg` for SI conversion.
    pub fn seed_mass_solar_impl(&self, n_local: f64, n_limit: f64) -> Result<f64, ExoticError> {
        if n_local < n_limit {
            return Err(ExoticError::AnomalyClosureError(
                "N_local must exceed N_limit to generate a ghost seed".to_string(),
            ));
        }
        let delta = n_local - n_limit;
        Ok(ALPHA_SEED_M_SUN_PER_BIT * delta)
    }

    pub fn seed_mass_kg_impl(&self, n_local: f64, n_limit: f64) -> Result<f64, ExoticError> {
        self.seed_mass_solar_impl(n_local, n_limit).map(|m| m * M_SUN_KG)
    }

    /// Density-multiplier perturbation induced by the congestion.
    ///
    /// `mu_local = mu0 + (N_local - N_limit) / N_limit`.
    pub fn local_mu_perturbation_impl(&self, n_local: f64, n_limit: f64, mu0: f64) -> Result<f64, ExoticError> {
        if n_limit <= 0.0 {
            return Err(ExoticError::AnomalyClosureError(
                "N_limit must be positive".to_string(),
            ));
        }
        let delta = n_local - n_limit;
        Ok(mu0 + delta / n_limit)
    }

    /// Check that the anyon filling factor is in the allowed non-Abelian list.
    pub fn is_filling_factor_allowed_impl(&self, nu: (i64, i64)) -> bool {
        self.allowed_filling_factors.iter().any(|&f| f == nu)
    }
}

#[pymethods]
impl GhostSeedSynthesizer {
    #[new]
    pub fn py_new() -> Self {
        Self::new()
    }

    fn seed_mass_solar(&self, n_local: f64, n_limit: f64) -> PyResult<f64> {
        self.seed_mass_solar_impl(n_local, n_limit).map_err(PyErr::from)
    }

    fn seed_mass_kg(&self, n_local: f64, n_limit: f64) -> PyResult<f64> {
        self.seed_mass_kg_impl(n_local, n_limit).map_err(PyErr::from)
    }

    fn local_mu_perturbation(&self, n_local: f64, n_limit: f64, mu0: f64) -> PyResult<f64> {
        self.local_mu_perturbation_impl(n_local, n_limit, mu0).map_err(PyErr::from)
    }

    fn is_filling_factor_allowed(&self, nu: (i64, i64)) -> bool {
        self.is_filling_factor_allowed_impl(nu)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_solar_mass_ghost_seed() {
        let synth = GhostSeedSynthesizer::new();
        // alpha_seed * delta = 1 M_sun  =>  delta = 1 / alpha_seed
        let delta = 1.0 / ALPHA_SEED_M_SUN_PER_BIT;
        let m_sun = synth.seed_mass_solar_impl(delta, 0.0).unwrap();
        assert!((m_sun - 1.0).abs() < 1e-6);
    }

    #[test]
    fn filling_factor_validation() {
        let synth = GhostSeedSynthesizer::new();
        assert!(synth.is_filling_factor_allowed_impl((12, 5)));
        assert!(!synth.is_filling_factor_allowed((1, 3)));
    }
}
