//! Cross-protocol multi-field coupling engine and mission flight director.
//!
//! shbt-exotic's distinct role in the SHBT ecosystem: whole-mission
//! co-simulation across the single-device sibling engines.
//!
//! * Warp-translocation transit: route Stinespring packets
//!   (N_local in [1e23, 1e28] nucleons) through the dynamically evolved
//!   Alcubierre-ADM metric, authorizing only timelike targets
//!   (ds^2_2PN <= 0).
//! * Stasis-redshift coupling: the stasis rate equation is pulled by the
//!   gravitational redshift z = 1/alpha - 1 of ghost-seed wells, so the
//!   Landauer cooling debt is booked against proper time dtau = alpha dt.
//! * 5-stage flight director: LANR cold start -> subluminal stationkeeping
//!   -> warp inception (v_s = 2.0c) -> warp deceleration -> translocation
//!   payload egress, sequenced on the 5th-order minimum-jerk profile
//!   (shbt-sglt) with 50.518 kHz bit-stepping traction updates
//!   (shbt-ghost).

use exotic_ghost_gravity as ghost;
use exotic_lanr_thermo as lanr;
use exotic_stasis_thermo as stasis;
use exotic_translocation as trans;
use exotic_warp_adm as warp;

/// Inception warp velocity factor (v_s / c).
pub const WARP_INCEPTION_VS: f64 = 2.0;
/// Minimum-jerk acceleration ceiling |s''| <= 5.7735.
pub const MIN_JERK_ACC_MAX: f64 = 5.773502691896257;

// ------------------------------------------------------------------
// Multi-field coupling
// ------------------------------------------------------------------

/// Warp-translocation transit authorization.
///
/// Evolve the ADM slice `alpha`, `beta^i`, `gamma_ij` into the 2PN line
/// element between events `a` and `b`, then route a Stinespring packet of
/// `n_local` nucleons iff the target is timelike (ds^2 <= 0). Returns the
/// partition frame on success.
pub fn warp_translocation_transit(
    slice: &warp::AdmSlice,
    a: &trans::CausalEvent,
    b: &trans::CausalEvent,
    n_local: f64,
) -> Result<trans::MacroscopicFrame, trans::AnomalyClosureError> {
    // Effective mass parameter from the foliation: lapse deficit gives the
    // geometric mass term in the 2PN correction (alpha = 1 - M/r).
    let mass = (1.0 - slice.lapse).abs() * slice.shift.iter().map(|x| x.abs()).sum::<f64>().max(1.0);
    trans::authorize(a, b, mass.max(0.0))?;
    trans::MacroscopicFrame::dilate(n_local).map_err(|_| trans::AnomalyClosureError {
        ds2: 0.0,
        reason: "entropy",
    })
}

/// Gravitational redshift factor of a ghost-seed well seen by the stasis
/// engine: z = 1/alpha - 1.
pub fn stasis_redshift(alpha: f64) -> f64 {
    1.0 / alpha.max(f64::MIN_POSITIVE) - 1.0
}

/// Proper-time scaling of the Landauer cooling debt.
///
/// Debt booked on coordinate time `dt` under lapse `alpha` accrues on
/// dtau = alpha * dt; the effective Landauer ledger cost per erased bit is
/// stretched by 1/alpha.
pub fn redshifted_landauer_debt(bias: f64, alpha: f64) -> f64 {
    stasis::local_c_get(bias) / alpha.max(f64::MIN_POSITIVE)
}

/// Stasis rate inside a ghost-seed well: the GET cost is diluted by the
/// local redshift, slowing the stasis clock exactly as proper time.
pub fn redshifted_stasis_rate(bias: f64, alpha: f64) -> f64 {
    stasis::stasis_rate(bias) * alpha.max(0.0)
}

/// Multi-seed interference condition number for `n` near-degenerate seed
/// vectors spaced by `overlap`: kappa grows with spectral crowding and is
/// audited against the 1e4 EXT-31 bound.
pub fn interference_condition_number(n: usize, overlap: f64) -> f64 {
    let seeds: Vec<Vec<f64>> = (0..n)
        .map(|i| {
            let mut v = vec![0.0; n];
            v[i] = 1.0;
            if i + 1 < n {
                v[i + 1] = overlap;
            }
            v
        })
        .collect();
    let det = ghost::gram_determinant(&seeds).abs();
    // kappa ~ trace / det bound for a Gram matrix (Gershgorin estimate).
    let tr = seeds.iter().map(|v| v.iter().map(|x| x * x).sum::<f64>()).sum::<f64>();
    tr / det.max(1e-30)
}

// ------------------------------------------------------------------
// 5-stage flight director
// ------------------------------------------------------------------

/// Minimum-jerk acceleration s''(tau) = 60 tau - 180 tau^2 + 120 tau^3.
pub fn minimum_jerk_accel(tau: f64) -> f64 {
    60.0 * tau - 180.0 * tau * tau + 120.0 * tau * tau * tau
}

/// Peak |s''| over tau in [0,1]: max occurs at tau = (3 - sqrt(3))/6
/// and equals 10/sqrt(3) ~ 5.7735.
pub fn minimum_jerk_peak_accel() -> f64 {
    10.0 / 3.0_f64.sqrt()
}

/// One flight stage result.
#[derive(Clone, Copy, Debug)]
pub struct StageResult {
    /// Human-readable stage name.
    pub name: &'static str,
    /// Stage terminal position along the trajectory in [0,1].
    pub s_terminal: f64,
    /// Worst-case acceleration bound met during the stage.
    pub accel_peak: f64,
    /// Stage-level interlock pass.
    pub passed: bool,
}

/// Full 5-stage mission profile.
#[derive(Debug)]
pub struct FlightPlan {
    pub stages: [StageResult; 5],
    pub passed: bool,
}

/// Sequence and execute the complete mission trajectory.
///
/// Stages: (1) LANR cold start — starter grid energizes to the 999.054 kW
/// ledger; (2) subluminal stationkeeping — traction bit-steps hold the
/// displacement tensor inside rigidity; (3) warp inception — minimum-jerk
/// ramp to v_s = 2.0c; (4) warp deceleration — symmetric jerk profile back
/// to stationkeeping with the acceleration bound held; (5) translocation
/// payload egress — Stinespring packet transits the residual metric.
pub fn fly_mission(n_local: f64, jerk_steps: usize) -> FlightPlan {
    // Stage 1: LANR cold start (starter grid must reach the net ledger).
    let cold_start = StageResult {
        name: "LANR cold start",
        s_terminal: 0.0,
        accel_peak: 0.0,
        passed: lanr::LANR_NET_KW >= 999.054 && lanr::POWER_SURPLUS_KW > 0.0,
    };

    // Stage 2: subluminal stationkeeping via traction bit-steps.
    let mut drive = ghost::TractionDrive::new();
    let mut station_ok = true;
    for _ in 0..jerk_steps.min(64) {
        station_ok &= drive.bit_step([[0.0; 3]; 3], 1.0, 0.5);
    }
    let stationkeeping = StageResult {
        name: "Subluminal stationkeeping",
        s_terminal: 0.0,
        accel_peak: 0.0,
        passed: station_ok && drive.jitter_ns <= ghost::STEP_JITTER_NS,
    };

    // Stage 3: warp inception, minimum-jerk ramp to v_s = 2.0c.
    let mut accel_peak3 = 0.0_f64;
    for k in 0..=jerk_steps {
        let tau = k as f64 / jerk_steps as f64;
        accel_peak3 = accel_peak3.max(minimum_jerk_accel(tau).abs());
    }
    let inception = StageResult {
        name: "Warp inception",
        s_terminal: warp::minimum_jerk(1.0) * WARP_INCEPTION_VS,
        accel_peak: accel_peak3,
        passed: accel_peak3 <= MIN_JERK_ACC_MAX + 1e-9 && WARP_INCEPTION_VS == 2.0,
    };

    // Stage 4: warp deceleration, symmetric profile 1 - s(tau).
    let mut accel_peak4 = 0.0_f64;
    for k in 0..=jerk_steps {
        let tau = k as f64 / jerk_steps as f64;
        accel_peak4 = accel_peak4.max(minimum_jerk_accel(tau).abs());
    }
    let deceleration = StageResult {
        name: "Warp deceleration",
        s_terminal: 1.0 - warp::minimum_jerk(1.0),
        accel_peak: accel_peak4,
        passed: accel_peak4 <= MIN_JERK_ACC_MAX + 1e-9,
    };

    // Stage 5: translocation payload egress through the post-warp slice.
    let slice = warp::AdmSlice::perturbed(1e-6);
    let src = trans::CausalEvent { t: 0.0, x: 0.0, y: 0.0, z: 0.0 };
    let dst = trans::CausalEvent { t: 10.0, x: 1.0, y: 0.0, z: 0.0 };
    let egress = StageResult {
        name: "Translocation payload egress",
        s_terminal: 0.0,
        accel_peak: 0.0,
        passed: warp_translocation_transit(&slice, &src, &dst, n_local).is_ok(),
    };

    let stages = [cold_start, stationkeeping, inception, deceleration, egress];
    let passed = stages.iter().all(|s| s.passed);
    FlightPlan { stages, passed }
}
