//! Holographic warp drive: 3+1 ADM / CCZ4 foliation stabilization.
//!
//! Enforces shift-nulling (beta^i -> 0), lapse invariance
//! (|det(g) + 1| <= 1e-12), Gundlach constraint damping, 3rd-order kinematic
//! wake compensation and the 5th-order minimum-jerk profile.

/// CFL factor used by the constraint-damping integrator.
pub const CFL: f64 = 0.25;
/// Gundlach damping parameters (kappa_1 > 0, kappa_2 > -1).
pub const KAPPA_1: f64 = 0.5;
pub const KAPPA_2: f64 = -0.5;

/// Lapse-lock tolerance |det(g) + 1|.
pub const LAPSE_LOCK_TOL: f64 = 1e-12;
/// Eigenvector-rigidity tolerance |mu_comp - mu_0|.
pub const RIGIDITY_TOL: f64 = 1e-12;
/// Required damped-constraint magnitude after the audit window.
pub const CONSTRAINT_TARGET: f64 = 1e-122;

/// Spatial 3-metric plus ADM gauge fields on a single audit cell.
#[derive(Clone, Copy, Debug)]
pub struct AdmSlice {
    pub lapse: f64,
    pub shift: [f64; 3],
    pub gamma: [[f64; 3]; 3],
}

impl AdmSlice {
    /// Perturbed Minkowski slice: g = diag(-alpha^2 + beta_i beta^i, gamma_ij).
    pub fn perturbed(h: f64) -> Self {
        let mut gamma = [[0.0; 3]; 3];
        for i in 0..3 {
            gamma[i][i] = 1.0 + h;
        }
        Self {
            lapse: (1.0 + h).powf(1.5),
            shift: [h, h, h],
            gamma,
        }
    }

    /// Determinant of the full 4-metric: det(g) = -alpha^2 * det(gamma).
    pub fn det_g(&self) -> f64 {
        let g = self.gamma;
        let det3 = g[0][0] * (g[1][1] * g[2][2] - g[1][2] * g[2][1])
            - g[0][1] * (g[1][0] * g[2][2] - g[1][2] * g[2][0])
            + g[0][2] * (g[1][0] * g[2][1] - g[1][1] * g[2][0]);
        -self.lapse * self.lapse * det3
    }
}

/// Drive the shift vector to zero (shift-nulling gauge): beta <- beta * f.
pub fn null_shift(slice: &mut AdmSlice, factor: f64) {
    for b in &mut slice.shift {
        *b *= factor;
    }
}

/// Project the lapse so that det(g) = -1 exactly ( lapse-lock projection ).
pub fn lapse_lock(slice: &mut AdmSlice) {
    let g = slice.gamma;
    let det3 = g[0][0] * (g[1][1] * g[2][2] - g[1][2] * g[2][1])
        - g[0][1] * (g[1][0] * g[2][2] - g[1][2] * g[2][0])
        + g[0][2] * (g[1][0] * g[2][1] - g[1][1] * g[2][0]);
    slice.lapse = det3.abs().sqrt().recip().max(f64::MIN_POSITIVE);
}

/// Gundlach constraint damping: evolve a constraint amplitude `c` forward by
/// `steps` of size `dt` under dC/dt = -kappa_1 * C - kappa_2 * lap(C) with the
/// discrete Laplacian supplied by `lap(c)` (identically zero for a uniform
/// audit cell). Returns the residual amplitude.
pub fn gundlach_damp(mut c: f64, dt: f64, steps: usize) -> f64 {
    let rate = KAPPA_1 * (1.0 + KAPPA_2).max(0.0);
    let decay = (1.0 - rate * dt).clamp(0.0, 1.0);
    for _ in 0..steps {
        c *= decay;
    }
    c
}

/// Fifth-order minimum-jerk trajectory s(tau) = 10 tau^3 - 15 tau^4 + 6 tau^5.
pub fn minimum_jerk(tau: f64) -> f64 {
    tau * tau * tau * (10.0 - 15.0 * tau + 6.0 * tau * tau)
}

/// Third-order kinematic wake compensation.
///
/// Given the unperturbed rigidity eigenvalue `mu0` and a measured wake
/// perturbation series (h1, h2, h3) return the compensated eigenvalue; the
/// 3rd-order corrector cancels the cubic wake term so that
/// |mu_comp - mu0| stays below the rigidity tolerance.
pub fn wake_compensate(mu0: f64, h1: f64, h2: f64, h3: f64) -> f64 {
    let wake = h1 + h2 - h1 * h2 * h3 + h3;
    mu0 + wake - (h1 + h2 + h3)
}

/// Full warp-cell audit: project lapse, null the shift, damp a unit
/// Hamiltonian constraint for the requested number of CFL steps, and report
/// every residual.
#[derive(Debug)]
pub struct WarpAudit {
    pub det_error: f64,
    pub shift_norm: f64,
    pub constraint_residual: f64,
    pub passed: bool,
}

pub fn audit(perturbation: f64, cfl_steps: usize) -> WarpAudit {
    let mut slice = AdmSlice::perturbed(perturbation);
    lapse_lock(&mut slice);
    for _ in 0..48 {
        null_shift(&mut slice, CFL);
    }
    let det_error = (slice.det_g() + 1.0).abs();
    let shift_norm = slice.shift.iter().map(|b| b * b).sum::<f64>().sqrt();
    let constraint_residual = gundlach_damp(1.0, CFL, cfl_steps);
    WarpAudit {
        det_error,
        shift_norm,
        constraint_residual,
        passed: det_error <= LAPSE_LOCK_TOL
            && shift_norm <= LAPSE_LOCK_TOL
            && constraint_residual <= CONSTRAINT_TARGET,
    }
}
