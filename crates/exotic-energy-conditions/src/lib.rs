//! Classical and quantum energy-condition auditing on the 3+1 foliation.
//!
//! Holographic boundary stress-energy framework imported from
//! `shbt-precision` (`src/shbt/boundary.rs`) and the coordinate stress
//! tensors of `shbt-ghost` (`ghost-multiseed-gravity`), specialized to the
//! shbt-exotic co-simulation role: continuous WEC / NEC / SEC / DEC audits
//! across ADM slices plus Ford-Roman quantum-inequality (QI) sampling for
//! the negative-energy pocket in the warp bubble boundary.
//!
//! Ford-Roman Lorentzian sampling:
//!   int <T_mn n^m n^n> tau0 / (pi (tau^2 + tau0^2)) dt >= -C / tau0^4
//! with C = 3 / (32 pi^2).

/// Ford-Roman bound constant C = 3 / (32 pi^2).
pub const FORD_ROMAN_C: f64 = 3.0 / (32.0 * std::f64::consts::PI * std::f64::consts::PI);

/// Planck sampling timescale floor (s) below which the QI does not apply.
pub const TAU_PLANCK_S: f64 = 5.39e-44;

/// Diagonal coordinate stress tensor on one foliation cell, in the
/// orthonormal Eulerian frame: T = diag(rho, p_r, p_t, p_t).
#[derive(Clone, Copy, Debug)]
pub struct StressTensor {
    /// Energy density rho.
    pub rho: f64,
    /// Radial pressure p_r.
    pub p_r: f64,
    /// Transverse pressure p_t.
    pub p_t: f64,
}

impl StressTensor {
    /// Minkowski vacuum.
    pub fn vacuum() -> Self {
        Self { rho: 0.0, p_r: 0.0, p_t: 0.0 }
    }

    /// Warp bubble boundary sample: negative energy density `rho_neg`
    /// screened by the QI budget, with anisotropic wall pressures.
    pub fn warp_wall(rho_neg: f64) -> Self {
        Self { rho: rho_neg, p_r: rho_neg.abs(), p_t: -rho_neg.abs() }
    }
}

/// Weak Energy Condition: T_mn u^m u^n >= 0 for every timelike u.
/// For the diagonal Eulerian tensor this reduces to rho >= 0 together with
/// rho + p_i >= 0 on every principal direction.
pub fn wec(t: &StressTensor) -> bool {
    t.rho >= 0.0
        && t.rho + t.p_r >= 0.0
        && t.rho + t.p_t >= 0.0
}

/// Null Energy Condition: T_mn k^m k^n >= 0 for every null k.
/// Reduces to rho + p_i >= 0 in each principal direction.
pub fn nec(t: &StressTensor) -> bool {
    t.rho + t.p_r >= 0.0 && t.rho + t.p_t >= 0.0
}

/// Strong Energy Condition: (T_mn - 1/2 T g_mn) u^m u^n >= 0.
/// For the diagonal tensor: rho + p_r + 2 p_t >= 0 and each
/// rho + p_i >= 0.
pub fn sec(t: &StressTensor) -> bool {
    t.rho + t.p_r + 2.0 * t.p_t >= 0.0 && nec(t)
}

/// Dominant Energy Condition: energy flux is non-spacelike, i.e.
/// |p_i| <= rho (requires rho >= 0).
pub fn dec(t: &StressTensor) -> bool {
    t.rho >= 0.0 && t.p_r.abs() <= t.rho + f64::EPSILON * t.rho.abs().max(1.0)
        && t.p_t.abs() <= t.rho + f64::EPSILON * t.rho.abs().max(1.0)
}

/// Classical energy-condition verdict for one cell.
#[derive(Clone, Copy, Debug)]
pub struct ConditionAudit {
    pub wec: bool,
    pub nec: bool,
    pub sec: bool,
    pub dec: bool,
}

pub fn audit_classical(t: &StressTensor) -> ConditionAudit {
    ConditionAudit {
        wec: wec(t),
        nec: nec(t),
        sec: sec(t),
        dec: dec(t),
    }
}

/// Lorentzian sampling kernel g(tau; tau0) = tau0 / (pi (tau^2 + tau0^2)).
pub fn lorentzian_kernel(tau: f64, tau0: f64) -> f64 {
    tau0 / (std::f64::consts::PI * (tau * tau + tau0 * tau0))
}

/// Ford-Roman QI: sample the energy-density profile `rho(tau)` over a
/// Lorentzian window of width `tau0` and compare against -C / tau0^4.
///
/// `rho_samples` are evaluations on a uniform grid `tau = k*dt` covering
/// +-`half_span`; `dt` is the sample spacing in the same units as `tau0`.
/// Returns the sampled value of the QI integral (must be >= bound).
pub fn ford_roman_integral(rho_samples: &[f64], dt: f64, tau0: f64) -> f64 {
    let n = rho_samples.len() as f64;
    rho_samples
        .iter()
        .enumerate()
        .map(|(k, r)| {
            let tau = (k as f64 - (n - 1.0) / 2.0) * dt;
            *r * lorentzian_kernel(tau, tau0) * dt
        })
        .sum()
}

/// The Ford-Roman lower bound for sampling width `tau0` (same units as the
/// integral): -C / tau0^4.
pub fn ford_roman_bound(tau0: f64) -> f64 {
    -FORD_ROMAN_C / tau0.powi(4)
}

/// QI compliance check: the sampled integral must meet the bound whenever
/// `tau0 >= tau_planck`.
pub fn qi_compliant(rho_samples: &[f64], dt: f64, tau0: f64) -> bool {
    if tau0 < TAU_PLANCK_S {
        return true;
    }
    ford_roman_integral(rho_samples, dt, tau0) >= ford_roman_bound(tau0)
}

/// Negative-energy warp-wall audit: a constant density `rho_neg` sustained
/// over duration `t_dur` must satisfy rho_neg * t_dur^4 >= -C (the
/// compact-window form of the Ford-Roman bound used in EXT-01).
pub fn warp_wall_qi(rho_neg: f64, t_dur: f64) -> bool {
    rho_neg * t_dur.powi(4) >= -FORD_ROMAN_C
}

/// Continuous foliation audit: evaluate the conditions on every slice of a
/// 1-D profile and return the count of NEC violations plus the worst
/// (lowest) QI margin across the window.
pub fn foliation_nec_violations(profile: &[StressTensor]) -> usize {
    profile.iter().filter(|t| !nec(t)).count()
}
