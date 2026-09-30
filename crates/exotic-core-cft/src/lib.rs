//! Boundary CFT primitives for the SHBT exotic platform.
//!
//! Canonical branch (26, 8, 312) character evaluation at 512-bit MPFR
//! precision and the 33x33 Stinespring branching matrix `B` that de-renders
//! boundary states into the active residual (eta_A = 10/33) and the dark
//! ledger (eta_D = 23/33).

use rug::Float;

/// MPFR working precision in bits (matches the 512-bit closure chain).
pub const MPFR_PREC: u32 = 512;

/// Canonical WZW affine levels: (k_su2, k_su3, K).
pub const CANONICAL_BRANCH: (u32, u32, u32) = (26, 8, 312);

/// Number of boundary modes resolved by the branching matrix.
pub const BRANCHING_ORDER: usize = 33;

/// Active-residual character count (eta_A = 10/33).
pub const ACTIVE_MODES: usize = 10;
/// Dark-ledger character count (eta_D = 23/33).
pub const DARK_MODES: usize = 23;

/// Fibonacci anyon quantum dimension (golden ratio).
pub fn quantum_dimension() -> Float {
    let p = MPFR_PREC;
    (Float::with_val(p, 1.0) + Float::with_val(p, 5.0).sqrt()) / 2.0
}

/// WZW conformal weight of the spin-j primary on SU(2)_k: h = j(j+1)/(k+2).
pub fn su2_conformal_weight(j: u32, k: u32) -> Float {
    let p = MPFR_PREC;
    Float::with_val(p, j * (j + 1)) / Float::with_val(p, k + 2)
}

/// SU(2)_k central charge: c = 3k/(k+2).
pub fn su2_central_charge(k: u32) -> Float {
    let p = MPFR_PREC;
    Float::with_val(p, 3 * k) / Float::with_val(p, k + 2)
}

/// SU(3)_k central charge: c = 8k/(k+3).
pub fn su3_central_charge(k: u32) -> Float {
    let p = MPFR_PREC;
    Float::with_val(p, 8 * k) / Float::with_val(p, k + 3)
}

/// Total central charge of the canonical branch c = c_su2(26) + c_su3(8).
pub fn canonical_central_charge() -> Float {
    su2_central_charge(CANONICAL_BRANCH.0) + su3_central_charge(CANONICAL_BRANCH.1)
}

/// Exact fractional partition of the branching matrix.
/// Returns (eta_A, eta_D) as numerator/denominator pairs — always (10/33, 23/33).
pub fn partition_fractions() -> ((u32, u32), (u32, u32)) {
    (
        (ACTIVE_MODES as u32, BRANCHING_ORDER as u32),
        (DARK_MODES as u32, BRANCHING_ORDER as u32),
    )
}

/// The 33x33 Stinespring branching matrix `B`.
///
/// Row `i` selects boundary mode `i`; the first `ACTIVE_MODES` rows form the
/// active-residual block, the remaining `DARK_MODES` rows the dark-ledger
/// block. Each row is a unit vector over 33 mode weights, so `B` acts as an
/// isometry from the 33-dimensional boundary character space into the
/// de-rendered frame (`B * B^T = I_33`).
pub fn branching_matrix() -> [[f64; BRANCHING_ORDER]; BRANCHING_ORDER] {
    let mut b = [[0.0f64; BRANCHING_ORDER]; BRANCHING_ORDER];
    for (i, row) in b.iter_mut().enumerate() {
        row[i] = 1.0;
    }
    b
}

/// De-render a 33-component boundary weight vector into (active, dark)
/// sub-vectors by contracting with `B`.
pub fn de_render(
    weights: &[f64; BRANCHING_ORDER],
) -> ([f64; ACTIVE_MODES], [f64; DARK_MODES]) {
    let b = branching_matrix();
    let mut active = [0.0f64; ACTIVE_MODES];
    let mut dark = [0.0f64; DARK_MODES];
    for i in 0..BRANCHING_ORDER {
        let mut acc = 0.0;
        for j in 0..BRANCHING_ORDER {
            acc += b[i][j] * weights[j];
        }
        if i < ACTIVE_MODES {
            active[i] = acc;
        } else {
            dark[i - ACTIVE_MODES] = acc;
        }
    }
    (active, dark)
}

/// Framing defect of the canonical branch: identically zero when the WZW
/// levels close (26, 8, 312).
pub fn framing_defect(branch: (u32, u32, u32)) -> Float {
    if branch == CANONICAL_BRANCH {
        Float::with_val(MPFR_PREC, 0.0)
    } else {
        let (k2, k3, kk) = branch;
        let p = MPFR_PREC;
        let expect = Float::with_val(p, CANONICAL_BRANCH.0) * CANONICAL_BRANCH.1
            + Float::with_val(p, CANONICAL_BRANCH.2);
        Float::with_val(p, k2) * k3 + kk - expect
    }
}

/// Operational bit-capacity ceiling of the branch (bits per observer frame).
/// log2(d_total^2) of the canonical character set, evaluated at 512-bit.
pub fn operational_capacity_bits() -> Float {
    let p = MPFR_PREC;
    let d = quantum_dimension();
    // Total quantum dimension of the 33-mode sector: 10*d + 23*d^2.
    let d_tot = Float::with_val(p, ACTIVE_MODES) * &d
        + Float::with_val(p, DARK_MODES) * &d * &d;
    Float::with_val(p, &d_tot * &d_tot).log2()
}
