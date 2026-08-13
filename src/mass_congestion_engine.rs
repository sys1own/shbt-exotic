//! Multi-seed mass congestion, metric superposition, and interference safety audit.
//!
//! Supports arrays of ghost seeds by linearly superposing their individual metric
//! perturbations `h_{μν}^{(i)}` and adding the non-linear interference correction
//! tensor `I_{μν}`.  The bit-congestion radius and the `10^{-12}` density-multiplier
//! rigidity limit enforce that overlapping seeds cannot detune `μ` past the HIL
//! threshold.

use pyo3::prelude::*;
use rug::Float;

use crate::constants::{
    EIGENVECTOR_RIGIDITY_THRESHOLD, PREC, SPEED_OF_LIGHT_M_S, TOTAL_BITS_NATURAL_LN,
};
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

/// 512-bit decimal wake coefficient `α_wake^{(1)}` for velocity-dependent detuning compensation.
pub const WAKE_1_STR: &str = "1.77245385090551602729816760023506821816503923841029381023910293810239281039120391823019238102938102391029381023912039120391203912039120";

/// 512-bit decimal wake coefficient `α_wake^{(2)}`.
pub const WAKE_2_STR: &str = "0.03423719481239845019238410293810239102938102391023910293102938120391203912039120391203912039120391203912039123841029381023910293810230";

/// 512-bit decimal wake coefficient `α_wake^{(3)}`.
pub const WAKE_3_STR: &str = "0.00001540911529184719238410293810239102938102392810391203918230192381029381023910293810239120391203912039120391203912384102938102391029";

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

    /// Wake coefficients `(α_wake^{(1)}, α_wake^{(2)}, α_wake^{(3)})` as `f64`.
    pub fn wake_constants_f64_impl(&self) -> [f64; 3] {
        [
            parse_512bit(WAKE_1_STR).to_f64(),
            parse_512bit(WAKE_2_STR).to_f64(),
            parse_512bit(WAKE_3_STR).to_f64(),
        ]
    }

    /// Total holographic bit ceiling `N_total = e^{33}`.
    pub fn n_total_impl(&self) -> f64 {
        TOTAL_BITS_NATURAL_LN.exp()
    }

    /// Velocity-dependent detuning compensation for a moving ghost seed.
    ///
    /// `μ_compensated(t) = μ0 - Σ_{k=1}^{3} α_wake^{(k)} (v_eff / c)^k (ΔN(t) / N_total)`.
    ///
    /// Returns `Ok(mu_comp)` if `|μ_comp - μ0| <= 10^{-12}`, otherwise raises
    /// `AnomalyClosureError`.
    pub fn compensated_mu_impl(
        &self,
        mu0: f64,
        delta_n: f64,
        n_total: f64,
        v_eff_m_s: f64,
    ) -> Result<f64, ExoticError> {
        if n_total <= 0.0 {
            return Err(ExoticError::AnomalyClosureError(
                "n_total must be positive".to_string(),
            ));
        }
        if v_eff_m_s < 0.0 {
            return Err(ExoticError::AnomalyClosureError(
                "v_eff must be non-negative".to_string(),
            ));
        }
        if v_eff_m_s > SPEED_OF_LIGHT_M_S {
            return Err(ExoticError::AnomalyClosureError(
                "v_eff cannot exceed the speed of light".to_string(),
            ));
        }
        let beta = if SPEED_OF_LIGHT_M_S > 0.0 {
            v_eff_m_s / SPEED_OF_LIGHT_M_S
        } else {
            0.0
        };
        let ratio = delta_n / n_total;
        let alphas = self.wake_constants_f64_impl();
        let mut correction = 0.0;
        for (k, alpha) in alphas.iter().enumerate() {
            let power = (k + 1) as i32;
            correction += alpha * beta.powi(power) * ratio;
        }
        let mu_comp = mu0 - correction;
        if (mu_comp - mu0).abs() > EIGENVECTOR_RIGIDITY_THRESHOLD {
            return Err(ExoticError::AnomalyClosureError(format!(
                "velocity wake detunes μ from {} to {} (threshold {})",
                mu0, mu_comp, EIGENVECTOR_RIGIDITY_THRESHOLD
            )));
        }
        Ok(mu_comp)
    }

    /// Dynamic interference Lagrangian for a moving ghost seed.
    ///
    /// `L_int = -μ0 - g_{μν} u^μ u^ν + (1/2) I_{μν} u^μ u^ν ΔN(t) - (1/6) Θ_{μνρ} u^μ u^ν u^ρ ΔN_dot`,
    /// where `Θ` is contracted with the metric as `Θ_{μνρ} u^μ u^ν u^ρ = (I_{μν} u^μ u^ν)(g_{αβ} u^α u^β)`.
    ///
    /// `g` and `u` must both be 4-dimensional.
    pub fn dynamic_interference_lagrangian_impl(
        &self,
        g: &[Vec<f64>],
        u: &[f64],
        delta_n: f64,
        delta_n_dot: f64,
        mu0: f64,
    ) -> Result<f64, ExoticError> {
        if g.len() != 4 || g.iter().any(|row| row.len() != 4) {
            return Err(ExoticError::AnomalyClosureError(
                "g must be a 4x4 matrix".to_string(),
            ));
        }
        if u.len() != 4 {
            return Err(ExoticError::AnomalyClosureError(
                "u must be a 4-vector".to_string(),
            ));
        }
        let i = self.interference_tensor_f64_impl();
        let mut g_uv = 0.0;
        let mut i_uv = 0.0;
        for mu in 0..4 {
            for nu in 0..4 {
                g_uv += g[mu][nu] * u[mu] * u[nu];
                i_uv += i[mu][nu] * u[mu] * u[nu];
            }
        }
        // Third-order contraction defined as (I_{μν} u^μ u^ν)(g_{αβ} u^α u^β).
        let theta = i_uv * g_uv;
        let l_int = -mu0 - g_uv + 0.5 * i_uv * delta_n - (1.0 / 6.0) * theta * delta_n_dot;
        Ok(l_int)
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

    /// 512-bit wake coefficients `(α_wake^{(1)}, α_wake^{(2)}, α_wake^{(3)})` as `f64`.
    fn wake_constants_f64(&self) -> [f64; 3] {
        self.wake_constants_f64_impl()
    }

    /// Total holographic bit ceiling `N_total = e^{33}`.
    fn n_total(&self) -> f64 {
        self.n_total_impl()
    }

    /// Velocity-dependent density-multiplier compensation for a moving seed.
    ///
    /// `mu_comp = mu0 - Σ_k α_wake^{(k)} (v_eff / c)^k (delta_n / n_total)`.
    fn compensated_mu(
        &self,
        mu0: f64,
        delta_n: f64,
        n_total: f64,
        v_eff_m_s: f64,
    ) -> PyResult<f64> {
        self.compensated_mu_impl(mu0, delta_n, n_total, v_eff_m_s)
            .map_err(PyErr::from)
    }

    /// Dynamic interference Lagrangian `L_int` for a moving seed.
    ///
    /// `g` is a 4x4 metric, `u` a 4-velocity, `delta_n` the bit overflow, and
    /// `delta_n_dot` its time derivative.  `mu0` is the unperturbed density multiplier.
    fn dynamic_interference_lagrangian(
        &self,
        g: Vec<Vec<f64>>,
        u: Vec<f64>,
        delta_n: f64,
        delta_n_dot: f64,
        mu0: f64,
    ) -> PyResult<f64> {
        self.dynamic_interference_lagrangian_impl(&g, &u, delta_n, delta_n_dot, mu0)
            .map_err(PyErr::from)
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

    #[test]
    fn compensated_mu_stays_within_rigidity_for_slow_transit() {
        let engine = MassCongestionEngine::new();
        let n_total = engine.n_total_impl();
        // A small overflow at 1 km/s keeps the wake correction below 1e-12.
        let v = 1.0e3;
        let delta_n = 1.0e5;
        let mu = engine.compensated_mu_impl(1.0, delta_n, n_total, v).unwrap();
        assert!((mu - 1.0).abs() < EIGENVECTOR_RIGIDITY_THRESHOLD);
    }

    #[test]
    fn compensated_mu_fails_for_extreme_velocity() {
        let engine = MassCongestionEngine::new();
        let n_total = engine.n_total_impl();
        // A large overflow at 99% c should exceed the 1e-12 limit.
        let v = 0.99 * SPEED_OF_LIGHT_M_S;
        let delta_n = 1.0e63;
        assert!(engine.compensated_mu_impl(1.0, delta_n, n_total, v).is_err());
    }

    #[test]
    fn dynamic_lagrangian_computes_finite_value() {
        let engine = MassCongestionEngine::new();
        let seeds = vec![(1.0e65, 1.0e65)];
        let g = engine.linearized_metric_with_interference_impl(&seeds).unwrap();
        // Timelike 4-velocity normalised to g_{μν} u^μ u^ν = -1 for η = diag(-1,1,1,1).
        let u = vec![1.0, 0.0, 0.0, 0.0];
        let l = engine
            .dynamic_interference_lagrangian_impl(&g, &u, 1.0, 0.0, 1.0)
            .unwrap();
        assert!(l.is_finite());
    }

    #[test]
    fn wake_constants_leading_digits_match_spec() {
        let engine = MassCongestionEngine::new();
        let w = engine.wake_constants_f64_impl();
        assert!(w[0] > 0.0);
        assert!(w[1] > 0.0);
        assert!(w[2] > 0.0);
        // Leading digits match the supplied 512-bit strings.
        assert!(w[0].to_string().starts_with("1.772453850905516"));
        assert!(w[1].to_string().starts_with("0.0342371948123984"));
        assert!(w[2].to_string().starts_with("0.00001540911529184"));
    }
}
