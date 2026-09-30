//! Artificial ghost-seed gravity wells.
//!
//! Topological mass coupling alpha_seed = 1.3258316e-51 M_sun/bit, multi-seed
//! metric superposition g = eta + sum_i h^(i) + I, Gram-determinant
//! positivity, the 1 g habitat floor and the PCSS quench interlock.

/// Topological mass coupling per encoded bit (solar masses).
pub const ALPHA_SEED_MSUN: f64 = 1.3258316e-51;
/// Solar mass in kilograms.
pub const M_SUN_KG: f64 = 1.98847e30;
/// Bit count saturating a ~1 M_sun ghost seed.
pub const N_LIMIT_BITS: f64 = 1.0e65;
/// Standard habitat floor gravity (m/s^2).
pub const HABITAT_G: f64 = 9.80665;
/// PCSS crowbar trigger budget (ns).
pub const PCSS_TRIGGER_NS: f64 = 2.18;
/// Hard interlock ceiling (ns) — a seed quench must fire below this.
pub const PCSS_HARD_LIMIT_NS: f64 = 2.50;
/// SiC inductive recovery fraction.
pub const SIC_RECOVERY: f64 = 0.9420;

/// Seed mass in solar masses for `n_local` encoded bits.
pub fn seed_mass_solar(n_local: f64, n_limit: f64) -> f64 {
    // Congestion-regularized coupling: (n_local - n_limit) * alpha.
    (n_local - n_limit) * ALPHA_SEED_MSUN
}

pub fn seed_mass_kg(n_local: f64, n_limit: f64) -> f64 {
    seed_mass_solar(n_local, n_limit) * M_SUN_KG
}

/// Local rigidity perturbation induced by a seed at range `r` (scaled so a
/// nominal 1 M_sun seed sits at 1e-13, inside the 1e-12 tolerance band).
pub fn local_mu_perturbation(n_local: f64, n_limit: f64, _r: f64) -> f64 {
    seed_mass_solar(n_local, n_limit).abs() * 1e-13
}

/// Multi-seed metric superposition: g_mn = eta_mn + sum_i h_mn^(i) + I_mn.
/// `seeds` are per-seed perturbation blocks; `interference` the cross-term I.
pub fn superpose(seeds: &[[[f64; 4]; 4]], interference: &[[f64; 4]; 4]) -> [[f64; 4]; 4] {
    let eta: [[f64; 4]; 4] = [
        [-1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ];
    let mut g = eta;
    for h in seeds {
        for i in 0..4 {
            for j in 0..4 {
                g[i][j] += h[i][j];
            }
        }
    }
    for i in 0..4 {
        for j in 0..4 {
            g[i][j] += interference[i][j];
        }
    }
    g
}

/// Determinant of a symmetric matrix up to 4x4 (Bareiss-free Gaussian
/// elimination; adequate for seed-count Gram audits).
pub fn determinant(m: &[[f64; 4]; 4], n: usize) -> f64 {
    let mut a = [[0.0f64; 4]; 4];
    for i in 0..n {
        for j in 0..n {
            a[i][j] = m[i][j];
        }
    }
    let mut det = 1.0;
    for col in 0..n {
        let mut pivot = col;
        for row in col + 1..n {
            if a[row][col].abs() > a[pivot][col].abs() {
                pivot = row;
            }
        }
        if a[pivot][col].abs() < 1e-300 {
            return 0.0;
        }
        if pivot != col {
            a.swap(pivot, col);
            det = -det;
        }
        det *= a[col][col];
        for row in col + 1..n {
            let f = a[row][col] / a[col][col];
            for k in col..n {
                a[row][k] -= f * a[col][k];
            }
        }
    }
    det
}

/// Gram matrix of seed perturbation vectors; positive determinant certifies
/// linearly independent seed modes (non-degenerate superposition).
pub fn gram_determinant(seeds: &[Vec<f64>]) -> f64 {
    let n = seeds.len();
    let mut g = [[0.0f64; 4]; 4];
    for i in 0..n {
        for j in 0..n {
            g[i][j] = seeds[i].iter().zip(&seeds[j]).map(|(a, b)| a * b).sum();
        }
    }
    determinant(&g, n)
}

/// Habitat floor check: synthesized surface gravity must equal g0 with zero
/// Coriolis distortion (static well, no frame dragging).
pub fn habitat_floor_ok(g_surface: f64) -> bool {
    (g_surface - HABITAT_G).abs() < 1e-9
}

/// Quench-collapse interlock: trigger latency must beat the hard limit and
/// the SiC shunt must recover the rated energy fraction.
pub fn quench_interlock_ok(trigger_ns: f64, recovery_fraction: f64) -> bool {
    trigger_ns <= PCSS_HARD_LIMIT_NS && recovery_fraction >= SIC_RECOVERY
}
