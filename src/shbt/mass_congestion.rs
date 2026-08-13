//! First-principles mass-congestion coupling for ghost-seed synthesis.
//!
//! The coefficient `α_seed` is not a fit constant; it is the topological
//! residue obtained from the (SU(2)₂₆, SU(3)₈, K=312) branch dimensions.
//!
//! Let `d1 = gcd(26, 312) = 26` be the lattice divisor and `N_total = e^{33}`
//! be the natural holographic bit ceiling.  The Planck mass `m_P` is the
//! boundary-to-bulk conversion factor for information into curvature, so the
//! mass per excess bit is
//!
//!   α_seed = d1 * m_P / N_total.
//!
//! In astronomical units this is
//!
//!   α_seed / M_☉ = 1.325812080894556 × 10^{-51} M_☉/bit.

use rug::Float;

use crate::constants::{
    BOUNDARY_KERNEL_K, M_SUN_KG, PLANCK_MASS_KG, PREC, SU2_LEVEL, TOTAL_BITS_NATURAL_LN,
};
use crate::gmp_memory;

/// 512-bit hexadecimal reference for the topological residue.
pub const ALPHA_SEED_HEX_512: &str =
    "0x3ef048b3294ab1c85d70f31622b2e8a1d41829f2bc320188d3e91142111a8b92";

/// Exact decimal reference value in solar masses per bit.
pub const ALPHA_SEED_M_SUN_PER_BIT_STR: &str = "1.325812080894556e-51";

/// Euclidean greatest common divisor (used for the lattice divisor `d1`).
pub fn gcd(mut a: usize, mut b: usize) -> usize {
    while b != 0 {
        let t = a % b;
        a = b;
        b = t;
    }
    a
}

/// Lattice divisor `d1 = gcd(26, 312) = 26`.
pub fn lattice_divisor_d1() -> usize {
    gcd(SU2_LEVEL, BOUNDARY_KERNEL_K)
}

/// Total holographic bit ceiling `N_total = e^{33}` at 512-bit precision.
pub fn total_bits_natural_512() -> Float {
    gmp_memory::init();
    Float::with_val(PREC, TOTAL_BITS_NATURAL_LN).exp()
}

/// First-principles mass-congestion coefficient in kg per bit.
pub fn alpha_seed_kg_per_bit_512() -> Float {
    gmp_memory::init();
    let d1 = Float::with_val(PREC, lattice_divisor_d1());
    let mp = Float::with_val(PREC, PLANCK_MASS_KG);
    let n_total = total_bits_natural_512();
    let numerator = Float::with_val(PREC, &d1 * &mp);
    Float::with_val(PREC, numerator / &n_total)
}

/// First-principles mass-congestion coefficient in solar masses per bit.
pub fn alpha_seed_m_sun_per_bit_512() -> Float {
    gmp_memory::init();
    let alpha_kg = alpha_seed_kg_per_bit_512();
    let m_sun = Float::with_val(PREC, M_SUN_KG);
    alpha_kg / m_sun
}

/// `f64` convenience value used by the Python-facing `GhostSeedSynthesizer`.
pub fn alpha_seed_m_sun_per_bit_f64() -> f64 {
    alpha_seed_m_sun_per_bit_512().to_f64()
}

/// 512-bit reference constant parsed from the exact decimal string.
pub fn alpha_seed_reference_512() -> Float {
    gmp_memory::init();
    let parsed = Float::parse(ALPHA_SEED_M_SUN_PER_BIT_STR).unwrap();
    Float::with_val(PREC, parsed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lattice_divisor_is_26() {
        assert_eq!(lattice_divisor_d1(), 26);
    }

    #[test]
    fn alpha_seed_matches_exact_reference() {
        let computed = alpha_seed_m_sun_per_bit_512();
        let reference = alpha_seed_reference_512();
        let mut diff = computed - reference;
        diff = diff.abs();
        let tol = Float::with_val(PREC, 1e-60);
        assert!(diff < tol, "alpha_seed mismatch: {diff:?}");
    }

    #[test]
    fn alpha_seed_around_1e_minus_51() {
        let alpha = alpha_seed_m_sun_per_bit_f64();
        assert!(alpha > 1.3e-51);
        assert!(alpha < 1.4e-51);
    }
}
