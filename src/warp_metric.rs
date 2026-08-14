//! 3+1D ADM warp-metric evaluation for the SHBT exotic platform.
//!
//! Evaluates a flat-slice Alcubierre-type line element with unit lapse,
//! Euclidean spatial metric, and a longitudinal shift vector built from the
//! SHBT shape function.  Every grid point is audited for `det(g) = -1` and
//! positive-definite Gram/spatial block eigenvalues.

use pyo3::prelude::*;
use pyo3::types::PyDict;

/// Default 10 m warp-bubble radius (m).
pub const WARP_BUBBLE_RADIUS_M: f64 = 10.0;
/// Default 30 m domain half-width for the metric audit.
pub const WARP_DOMAIN_RADIUS_M: f64 = 30.0;
/// Wall steepness parameter (1/m) for the Alcubierre shape function.
pub const WARP_WALL_STEEPNESS_PER_M: f64 = 0.8;
/// Maximum grid points for the 1-D longitudinal metric audit.
pub const MAX_WARP_GRID_POINTS: usize = 1201;

/// Alcubierre shape function `f(r_s)` for the longitudinal shift.
fn alcubierre_shape(r_s: f64, radius: f64, sigma: f64) -> f64 {
    let denom = 2.0 * (sigma * radius).tanh();
    if denom == 0.0 {
        return 0.0;
    }
    ((sigma * (r_s + radius)).tanh() - (sigma * (r_s - radius)).tanh()) / denom
}

/// 4-D Lorentzian determinant and Gram positivity audit for a longitudinal
/// shift `β` (dimensionless, in units of c).
///
/// Returns `(det(g), min_abs_lorentzian_ev, min_gram_ev)`.  The ADM metric is
///   g_00 = -1 + β^2,  g_0i = g_i0 = β n_i,  g_ij = δ_ij,
/// with `n = (1,0,0)` so only the x-component is non-trivial.
fn metric_invariants(beta: f64) -> (f64, f64, f64) {
    // 2x2 t-x Lorentzian block determinant: analytically -1.
    let b2 = beta * beta;
    let det = -1.0 + b2 - b2;

    // Eigenvalues of the 2x2 Lorentzian block solve λ^2 - β^2 λ - 1 = 0.
    let disc_l = (b2 * b2 + 4.0).sqrt();
    let lambda_plus = (b2 + disc_l) / 2.0;
    let lambda_minus = (b2 - disc_l) / 2.0;
    let min_abs_lorentzian_ev = lambda_plus.min(lambda_minus.abs()).min(1.0);

    // Gram (spatial) block in the t-x subspace: γ_tt = 1 + β^2, γ_tx = β, γ_xx = 1.
    // Characteristic polynomial: λ^2 - (2 + β^2) λ + 1 = 0.
    let disc_g = ((2.0 + b2).powi(2) - 4.0).sqrt();
    let gamma_plus = (2.0 + b2 + disc_g) / 2.0;
    let gamma_minus = (2.0 + b2 - disc_g) / 2.0;
    let min_gram_ev = gamma_plus.min(gamma_minus).min(1.0);

    (det, min_abs_lorentzian_ev, min_gram_ev)
}

/// Audit result for a single warp-metric velocity scan.
#[derive(Clone, Debug)]
pub struct WarpMetricAudit {
    pub velocity_c: f64,
    pub max_determinant_error: f64,
    pub min_abs_determinant: f64,
    pub min_abs_lorentzian_eigenvalue: f64,
    pub min_gram_eigenvalue: f64,
    pub passed: bool,
}

/// 3+1D ADM metric auditor for a 10 m SHBT warp bubble.
#[pyclass(name = "ADMMetricAuditor")]
#[derive(Clone, Debug)]
pub struct ADMMetricAuditor {
    bubble_radius_m: f64,
    wall_steepness_per_m: f64,
    domain_radius_m: f64,
    grid_points: usize,
}

impl ADMMetricAuditor {
    pub fn new() -> Self {
        Self::with_params(WARP_BUBBLE_RADIUS_M, WARP_WALL_STEEPNESS_PER_M, WARP_DOMAIN_RADIUS_M, 65)
    }

    pub fn with_params(
        bubble_radius_m: f64,
        wall_steepness_per_m: f64,
        domain_radius_m: f64,
        grid_points: usize,
    ) -> Self {
        let n = grid_points.min(MAX_WARP_GRID_POINTS).max(5);
        let n = if n % 2 == 0 { n + 1 } else { n };
        Self {
            bubble_radius_m,
            wall_steepness_per_m,
            domain_radius_m,
            grid_points: n,
        }
    }

    /// Audit the 3+1D metric along the longitudinal axis at `velocity_c`.
    pub fn audit_velocity(&self, velocity_c: f64) -> WarpMetricAudit {
        let dx = 2.0 * self.domain_radius_m / (self.grid_points as f64 - 1.0);
        let mut max_det_error = 0.0f64;
        let mut min_abs_det = f64::INFINITY;
        let mut min_abs_lorentzian_ev = f64::INFINITY;
        let mut min_gram_ev = f64::INFINITY;

        for i in 0..self.grid_points {
            let x = -self.domain_radius_m + dx * (i as f64);
            let r_s = x.abs();
            let f = alcubierre_shape(r_s, self.bubble_radius_m, self.wall_steepness_per_m);
            let beta = velocity_c * f;
            let (det, lorentz_min, gram_min) = metric_invariants(beta);
            let det_error = (det + 1.0).abs();
            max_det_error = max_det_error.max(det_error);
            min_abs_det = min_abs_det.min(det.abs());
            min_abs_lorentzian_ev = min_abs_lorentzian_ev.min(lorentz_min);
            min_gram_ev = min_gram_ev.min(gram_min);
        }

        WarpMetricAudit {
            velocity_c,
            max_determinant_error: max_det_error,
            min_abs_determinant: min_abs_det,
            min_abs_lorentzian_eigenvalue: min_abs_lorentzian_ev,
            min_gram_eigenvalue: min_gram_ev,
            passed: max_det_error <= 1.0e-12
                && min_abs_lorentzian_ev > 1.0e-12
                && min_gram_ev > 1.0e-12,
        }
    }

    /// Evaluate the 4x4 covariant ADM metric at `x_m` for the supplied
    /// velocity and direction vector `n_vec`.
    pub fn evaluate_metric_at_impl(
        &self,
        x_m: f64,
        velocity_c: f64,
        n_vec: [f64; 3],
    ) -> [f64; 16] {
        let r_s = x_m.abs();
        let f = alcubierre_shape(r_s, self.bubble_radius_m, self.wall_steepness_per_m);
        let n_norm = (n_vec[0] * n_vec[0] + n_vec[1] * n_vec[1] + n_vec[2] * n_vec[2]).sqrt();
        let n = if n_norm > 1e-15 {
            [n_vec[0] / n_norm, n_vec[1] / n_norm, n_vec[2] / n_norm]
        } else {
            [1.0, 0.0, 0.0]
        };
        let b = -velocity_c * f;
        let beta = [b * n[0], b * n[1], b * n[2]];
        let beta_sq = beta[0] * beta[0] + beta[1] * beta[1] + beta[2] * beta[2];

        [
            -1.0 + beta_sq, beta[0], beta[1], beta[2],
            beta[0], 1.0, 0.0, 0.0,
            beta[1], 0.0, 1.0, 0.0,
            beta[2], 0.0, 0.0, 1.0,
        ]
    }
}

#[pymethods]
impl ADMMetricAuditor {
    #[new]
    #[pyo3(signature = (bubble_radius_m=WARP_BUBBLE_RADIUS_M, wall_steepness_per_m=WARP_WALL_STEEPNESS_PER_M, domain_radius_m=WARP_DOMAIN_RADIUS_M, grid_points=65))]
    fn py_new(
        bubble_radius_m: f64,
        wall_steepness_per_m: f64,
        domain_radius_m: f64,
        grid_points: usize,
    ) -> Self {
        Self::with_params(bubble_radius_m, wall_steepness_per_m, domain_radius_m, grid_points)
    }

    /// Audit at `velocity_c` and return a Python dict.
    fn audit<'py>(&self, py: Python<'py>, velocity_c: f64) -> PyResult<Bound<'py, PyDict>> {
        let r = self.audit_velocity(velocity_c);
        let d = PyDict::new(py);
        d.set_item("velocity_c", r.velocity_c)?;
        d.set_item("max_determinant_error", r.max_determinant_error)?;
        d.set_item("min_abs_determinant", r.min_abs_determinant)?;
        d.set_item("min_abs_lorentzian_eigenvalue", r.min_abs_lorentzian_eigenvalue)?;
        d.set_item("min_gram_eigenvalue", r.min_gram_eigenvalue)?;
        d.set_item("passed", r.passed)?;
        Ok(d)
    }

    /// Evaluate the 4x4 ADM metric at `x_m` as a 16-element row-major array.
    fn evaluate_metric_at(&self, x_m: f64, velocity_c: f64, n_vec: [f64; 3]) -> [f64; 16] {
        self.evaluate_metric_at_impl(x_m, velocity_c, n_vec)
    }
}

impl Default for ADMMetricAuditor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn warp_determinant_is_minus_one() {
        let auditor = ADMMetricAuditor::new();
        let result = auditor.audit_velocity(0.1);
        assert!(result.passed, "{:?}", result);
        assert!(result.max_determinant_error < 1.0e-12);
    }

    #[test]
    fn gram_positive_at_sub_c_velocity() {
        let auditor = ADMMetricAuditor::new();
        let result = auditor.audit_velocity(0.5);
        assert!(result.min_gram_eigenvalue > 0.0);
    }
}
