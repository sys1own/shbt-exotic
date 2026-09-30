//! Modular state translocation.
//!
//! Macroscopic Stinespring dilation V_unified^macro over N_local in
//! [1e23, 1e28] nucleons with exact fractional partitioning
//! (eta_A = 10/33, eta_D = 23/33), Heegaard-Floer symplectic boundary
//! relabeling in Sp(2g, Z), 2PN causal lightcone authorization and GST
//! chalcogenide self-healing.

use std::fmt;

/// Nucleon window for macroscopic dilation.
pub const N_LOCAL_MIN: f64 = 1e23;
pub const N_LOCAL_MAX: f64 = 1e28;

/// Exact partition fractions.
pub const ETA_A_NUM: u64 = 10;
pub const ETA_D_NUM: u64 = 23;
pub const PARTITION_DEN: u64 = 33;

/// Fibonacci braid descriptors in the dark-ledger arena.
pub const BRAID_DESCRIPTORS: usize = 124;

/// GST self-healing fluence threshold (mJ/cm^2).
pub const GST_HEALING_FLUENCE: f64 = 27.9;

/// Thrown when the translocation target is spacelike (ds^2 > 0).
#[derive(Debug, Clone)]
pub struct AnomalyClosureError {
    pub ds2: f64,
}

impl fmt::Display for AnomalyClosureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "AnomalyClosureError: spacelike target, ds^2 = {:.6e} > 0",
            self.ds2
        )
    }
}

impl std::error::Error for AnomalyClosureError {}

/// Event in 2PN harmonic coordinates (t, x, y, z in geometric units).
#[derive(Clone, Copy, Debug)]
pub struct CausalEvent {
    pub t: f64,
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

/// Signed 2PN-corrected interval between two events.
///
/// ds^2 = -(dt)^2 + dr^2 + (2M/r) [ (dt)^2 + dr^2 ] evaluated at the midpoint
/// radius; matches the first post-Minkowskian 2PN harmonic correction.
pub fn ds2_2pn(a: &CausalEvent, b: &CausalEvent, mass: f64) -> f64 {
    let dt = b.t - a.t;
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    let dz = b.z - a.z;
    let dr2 = dx * dx + dy * dy + dz * dz;
    let r_mid = (((a.x + b.x) / 2.0).powi(2)
        + ((a.y + b.y) / 2.0).powi(2)
        + ((a.z + b.z) / 2.0).powi(2))
    .sqrt()
    .max(1e-12);
    let corr = 2.0 * mass / r_mid;
    -dt * dt + dr2 + corr * (dt * dt + dr2)
}

/// Causal authorization: ds^2 <= 0 required.
pub fn authorize(a: &CausalEvent, b: &CausalEvent, mass: f64) -> Result<f64, AnomalyClosureError> {
    let ds2 = ds2_2pn(a, b, mass);
    if ds2 > 0.0 {
        Err(AnomalyClosureError { ds2 })
    } else {
        Ok(ds2)
    }
}

/// Macroscopic Stinespring frame for `n_local` nucleons.
pub struct MacroscopicFrame {
    pub n_local: f64,
    pub active_nucleons: f64,
    pub dark_nucleons: f64,
}

impl MacroscopicFrame {
    /// Partition the nucleon count exactly as 10/33 : 23/33.
    pub fn dilate(n_local: f64) -> Result<Self, String> {
        if !(N_LOCAL_MIN..=N_LOCAL_MAX).contains(&n_local) {
            return Err(format!(
                "n_local {} outside [{}, {}] nucleon window",
                n_local, N_LOCAL_MIN, N_LOCAL_MAX
            ));
        }
        let active = n_local * (ETA_A_NUM as f64 / PARTITION_DEN as f64);
        let dark = n_local * (ETA_D_NUM as f64 / PARTITION_DEN as f64);
        Ok(Self {
            n_local,
            active_nucleons: active,
            dark_nucleons: dark,
        })
    }

    /// Recombination closure: active + dark must reconstruct n_local exactly.
    pub fn closure_holds(&self) -> bool {
        (self.active_nucleons + self.dark_nucleons - self.n_local).abs()
            <= self.n_local * f64::EPSILON
    }
}

/// The canonical Sp(2g, Z) symplectic form J (g x g blocks).
pub fn symplectic_j(g: usize) -> Vec<Vec<i64>> {
    let mut j = vec![vec![0i64; 2 * g]; 2 * g];
    for i in 0..g {
        j[i][g + i] = 1;
        j[g + i][i] = -1;
    }
    j
}

/// Heegaard-Floer boundary relabeling matrix T^d_ij in Sp(2g, Z).
/// Built as a Dehn-twist shear on the first handle pair: unipotent and
/// symplectic by construction.
pub fn relabeling_matrix(g: usize, twist: i64) -> Vec<Vec<i64>> {
    let mut t = vec![vec![0i64; 2 * g]; 2 * g];
    for i in 0..2 * g {
        t[i][i] = 1;
    }
    t[0][g] = twist;
    t
}

/// Verify M in Sp(2g, Z): integer entries and M^T J M = J.
pub fn is_symplectic(m: &[Vec<i64>], g: usize) -> bool {
    let j = symplectic_j(g);
    let n = 2 * g;
    // M^T J M
    for i in 0..n {
        for k in 0..n {
            let mut acc: i64 = 0;
            for a in 0..n {
                for b in 0..n {
                    acc += m[a][i] * j[a][b] * m[b][k];
                }
            }
            if acc != j[i][k] {
                return false;
            }
        }
    }
    true
}

/// GST chalcogenide self-healing audit: healed bond fraction after an
/// annealing pulse of `fluence` mJ/cm^2. Healing saturates above the
/// crystallization threshold with an Arrhenius-like approach to unity.
pub fn gst_healing_fraction(fluence: f64, anneal_cycles: u32) -> f64 {
    let drive = (fluence / GST_HEALING_FLUENCE).max(0.0);
    1.0 - (-drive * (anneal_cycles as f64) * 3.0).exp()
}
