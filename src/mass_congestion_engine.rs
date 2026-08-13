//! Multi-seed mass congestion, metric superposition, and interference safety audit.
//!
//! Supports arrays of ghost seeds by linearly superposing their individual metric
//! perturbations `h_{μν}^{(i)}` and adding the non-linear interference correction
//! tensor `I_{μν}`.  The bit-congestion radius and the `10^{-12}` density-multiplier
//! rigidity limit enforce that overlapping seeds cannot detune `μ` past the HIL
//! threshold.

use pyo3::prelude::*;
use rug::Float;

use crate::constants::{EIGENVECTOR_RIGIDITY_THRESHOLD, PREC};
use crate::error::ExoticError;
use crate::gmp_memory;
use crate::shbt::mass_congestion::alpha_seed_m_sun_per_bit_f64;

/// Bit-congestion radius (m).  Seeds separated by less than this are treated as
/// overlapping and subject to the interference safety audit.
pub const BIT_CONGESTION_RADIUS_M: f64 = 2.954e15;

/// 512-bit decimal coefficient `I_{00}` for the interference correction tensor.
pub const I_00_STR: &str = "0.1415926535897932384626433832795028841971693993751058209749445923078164062862089986280348253421170679821480865132823066470938446";

/// 512-bit decimal coefficient `I_{11}` for the interference correction tensor.
pub const I_11_STR: &str = "-0.2718281828459045235360287471352662497757247093699959574966967627724076630353547594571382178525166427427466391932003059921817413";

/// 512-bit decimal coefficient `I_{22}` for the interference correction tensor.
pub const I_22_STR: &str = "0.5772156649015328606065120900824024310421593359399235988057672348848677267776620160803038285701620311288931215134811111102909722";

/// 512-bit decimal coefficient `I_{33}` for the interference correction tensor.
pub const I_33_STR: &str = "-0.3183098861837906715377675267450287240689184754980934520815024494944017305351480336210082987141528627814859173234568124230101825";

/// Isotropic seed perturbation template used for each ghost seed.
///
/// In the linearized regime each seed adds a uniform scalar strain `A_i` to all
/// diagonal metric components; the component-dependent `I_{μν}` then provides
/// the non-linear correction that keeps the metric from collapsing.
const SEED_PERTURBATION_TEMPLATE: [f64; 4] = [1.0, 1.0, 1.0, 1.0];

fn parse_512bit(s: &str) -> Float {
    gmp_memory::init();
    let parsed = Float::parse(s).expect("valid 512-bit coefficient");
    Float::with_val(PREC, parsed)
}

/// Multi-seed mass-congestion engine.
#[pyclass(name = "MassCongestionEngine")]
#[derive(Clone, Debug)]
pub struct MassCongestionEngine;

impl MassCongestionEngine {
    pub fn new() -> Self {
        gmp_memory::init();
        Self
    }

    /// Interference correction tensor `I_{μν}` as 512-bit `Float` values.
    ///
    /// Returns a 4x4 diagonal matrix with entries `(I_00, I_11, I_22, I_33)`.
    pub fn interference_tensor_512(&self) -> Vec<Vec<Float>> {
        let i_vals = [
            parse_512bit(I_00_STR),
            parse_512bit(I_11_STR),
            parse_512bit(I_22_STR),
            parse_512bit(I_33_STR),
        ];
        let mut matrix = vec![vec![Float::with_val(PREC, 0); 4]; 4];
        for (mu, val) in i_vals.iter().enumerate() {
            matrix[mu][mu] = val.clone();
        }
        matrix
    }

    /// Interference correction tensor `I_{μν}` as `f64` values for Python use.
    pub fn interference_tensor_f64_impl(&self) -> Vec<Vec<f64>> {
        let i_vals = [
            parse_512bit(I_00_STR).to_f64(),
            parse_512bit(I_11_STR).to_f64(),
            parse_512bit(I_22_STR).to_f64(),
            parse_512bit(I_33_STR).to_f64(),
        ];
        let mut matrix = vec![vec![0.0; 4]; 4];
        for (mu, val) in i_vals.iter().enumerate() {
            matrix[mu][mu] = *val;
        }
        matrix
    }

    /// Bit-congestion radius `R_congestion` in metres.
    pub fn bit_congestion_radius_m_impl(&self) -> f64 {
        BIT_CONGESTION_RADIUS_M
    }

    /// Check that seed-to-seed separations are at least the bit-congestion radius.
    ///
    /// `separations_m` contains the pairwise distances for the seed array.  Any
    /// distance smaller than `R_congestion` triggers `AnomalyClosureError`.
    pub fn check_bit_congestion_radius_impl(&self, separations_m: &[f64]) -> Result<(), ExoticError> {
        for (i, d) in separations_m.iter().enumerate() {
            if *d < 0.0 {
                return Err(ExoticError::AnomalyClosureError(format!(
                    "separation {} is negative",
                    d
                )));
            }
            if *d < BIT_CONGESTION_RADIUS_M {
                return Err(ExoticError::AnomalyClosureError(format!(
                    "bit-congestion violation: separation {} m < R_congestion = {} m at pair {}",
                    d, BIT_CONGESTION_RADIUS_M, i
                )));
            }
        }
        Ok(())
    }

    /// Individual seed perturbation amplitude `A_i = (N_local - N_limit) / N_limit`.
    fn seed_perturbation(&self, n_local: f64, n_limit: f64) -> Result<f64, ExoticError> {
        if n_limit <= 0.0 {
            return Err(ExoticError::AnomalyClosureError(
                "N_limit must be positive".to_string(),
            ));
        }
        let delta = n_local - n_limit;
        Ok(delta / n_limit)
    }

    /// Total seed perturbation summed over an array of seeds.
    ///
    /// `seeds` is a slice of `(N_local, N_limit)` pairs.
    pub fn total_seed_perturbation(&self, seeds: &[(f64, f64)]) -> Result<f64, ExoticError> {
        let mut total = 0.0;
        for &(n_local, n_limit) in seeds {
            total += self.seed_perturbation(n_local, n_limit)?;
        }
        Ok(total)
    }

    /// Multi-seed density-multiplier perturbation including overlap.
    ///
    /// `μ = μ_0 + Σ_i (N_local_i - N_limit_i) / N_limit_i`.
    /// If `|μ - μ_0| > 10^{-12}` an `AnomalyClosureError` is raised.
    pub fn multi_seed_mu_perturbation_impl(
        &self,
        seeds: &[(f64, f64)],
        mu0: f64,
    ) -> Result<f64, ExoticError> {
        let delta = self.total_seed_perturbation(seeds)?;
        let mu = mu0 + delta;
        if (mu - mu0).abs() > EIGENVECTOR_RIGIDITY_THRESHOLD {
            return Err(ExoticError::AnomalyClosureError(format!(
                "multi-seed overlap detunes μ from {} to {} (threshold {})",
                mu0, mu, EIGENVECTOR_RIGIDITY_THRESHOLD
            )));
        }
        Ok(mu)
    }

    /// Linearized metric with multi-seed interference correction:
    ///
    ///   g_{μν} = η_{μν} + Σ_i h_{μν}^{(i)} + I_{μν} .
    ///
    /// The Minkowski metric is `η = diag(-1, 1, 1, 1)`.  Each seed contributes
    /// the isotropic scalar strain `A_i` to all diagonal components, and the
    /// 512-bit interference tensor `I_{μν}` is added component-by-component.
    pub fn linearized_metric_with_interference_impl(
        &self,
        seeds: &[(f64, f64)],
    ) -> Result<Vec<Vec<f64>>, ExoticError> {
        let a_total = self.total_seed_perturbation(seeds)?;
        let i = self.interference_tensor_f64_impl();

        let eta = [-1.0_f64, 1.0, 1.0, 1.0];
        let mut g = vec![vec![0.0; 4]; 4];
        for mu in 0..4 {
            let h_mu = a_total * SEED_PERTURBATION_TEMPLATE[mu];
            g[mu][mu] = eta[mu] + h_mu + i[mu][mu];
        }
        Ok(g)
    }

    /// Aggregate ghost-seed mass in solar masses for a multi-seed array.
    pub fn multi_seed_mass_solar_impl(&self, seeds: &[(f64, f64)]) -> Result<f64, ExoticError> {
        let alpha = alpha_seed_m_sun_per_bit_f64();
        let mut total = 0.0;
        for &(n_local, n_limit) in seeds {
            if n_local < n_limit {
                return Err(ExoticError::AnomalyClosureError(
                    "N_local must exceed N_limit for every seed".to_string(),
                ));
            }
            total += alpha * (n_local - n_limit);
        }
        Ok(total)
    }
}

#[pymethods]
impl MassCongestionEngine {
    #[new]
    pub fn py_new() -> Self {
        Self::new()
    }

    /// Bit-congestion radius `R_congestion` in metres.
    fn bit_congestion_radius_m(&self) -> f64 {
        self.bit_congestion_radius_m_impl()
    }

    /// Check pairwise seed separations against `R_congestion`.
    fn check_bit_congestion_radius(&self, separations_m: Vec<f64>) -> PyResult<()> {
        self.check_bit_congestion_radius_impl(&separations_m)
            .map_err(PyErr::from)
    }

    /// Interference correction tensor `I_{μν}` as a 4x4 list of lists (f64).
    fn interference_tensor_f64(&self) -> Vec<Vec<f64>> {
        self.interference_tensor_f64_impl()
    }

    /// Coefficients `(I_00, I_11, I_22, I_33)` as exact decimal strings.
    fn interference_coefficients(&self) -> (String, String, String, String) {
        (
            I_00_STR.to_string(),
            I_11_STR.to_string(),
            I_22_STR.to_string(),
            I_33_STR.to_string(),
        )
    }

    /// Multi-seed density multiplier `μ` including overlap; raises
    /// `AnomalyClosureError` if `|μ - μ_0| > 10^{-12}`.
    fn multi_seed_mu_perturbation(&self, seeds: Vec<(f64, f64)>, mu0: f64) -> PyResult<f64> {
        self.multi_seed_mu_perturbation_impl(&seeds, mu0).map_err(PyErr::from)
    }

    /// Linearized metric `g_{μν} = η_{μν} + Σ h_{μν}^{(i)} + I_{μν}` as 4x4 f64.
    fn linearized_metric_with_interference(&self, seeds: Vec<(f64, f64)>) -> PyResult<Vec<Vec<f64>>> {
        self.linearized_metric_with_interference_impl(&seeds)
            .map_err(PyErr::from)
    }

    /// Total multi-seed mass in solar masses.
    fn multi_seed_mass_solar(&self, seeds: Vec<(f64, f64)>) -> PyResult<f64> {
        self.multi_seed_mass_solar_impl(&seeds).map_err(PyErr::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interference_tensor_is_diagonal_4x4() {
        let engine = MassCongestionEngine::new();
        let i = engine.interference_tensor_f64();
        assert_eq!(i.len(), 4);
        assert!(i.iter().all(|row| row.len() == 4));
        for mu in 0..4 {
            for nu in 0..4 {
                if mu != nu {
                    assert_eq!(i[mu][nu], 0.0);
                }
            }
        }
    }

    #[test]
    fn i00_is_positive_and_i11_is_negative() {
        let engine = MassCongestionEngine::new();
        let i = engine.interference_tensor_f64();
        assert!(i[0][0] > 0.0);
        assert!(i[1][1] < 0.0);
    }

    #[test]
    fn bit_congestion_radius_matches_spec() {
        let engine = MassCongestionEngine::new();
        assert!((engine.bit_congestion_radius_m_impl() - 2.954e15).abs() < 1e10);
    }

    #[test]
    fn multi_seed_mu_passes_below_threshold() {
        let engine = MassCongestionEngine::new();
        // Two seeds, each contributing 4e-13, total 8e-13 < 1e-12
        let seeds = vec![(1.0e65 + 4e52, 1.0e65), (1.0e65 + 4e52, 1.0e65)];
        let mu = engine.multi_seed_mu_perturbation_impl(&seeds, 1.0).unwrap();
        assert!((mu - 1.0).abs() < 1e-12);
    }

    #[test]
    fn multi_seed_mu_fails_above_threshold() {
        let engine = MassCongestionEngine::new();
        // Two seeds, each contributing 6e-13, total 1.2e-12 > 1e-12
        let seeds = vec![(1.0e65 + 6e52, 1.0e65), (1.0e65 + 6e52, 1.0e65)];
        assert!(engine.multi_seed_mu_perturbation_impl(&seeds, 1.0).is_err());
    }

    #[test]
    fn linearized_metric_is_non_singular_for_small_overlap() {
        let engine = MassCongestionEngine::new();
        let seeds = vec![(1.0e65 + 4e52, 1.0e65), (1.0e65 + 4e52, 1.0e65)];
        let g = engine.linearized_metric_with_interference_impl(&seeds).unwrap();
        // Diagonal entries should not be zero after the I_{mu nu} correction.
        for mu in 0..4 {
            assert!(g[mu][mu].abs() > 0.1, "g[{}][{}] = {}", mu, mu, g[mu][mu]);
        }
    }
}
