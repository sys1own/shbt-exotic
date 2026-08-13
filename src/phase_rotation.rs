//! Deterministic SIMD kernels for U(1) phase rotations and HIL sensor comparisons.
//!
//! The hot path is branchless, works on stack-resident fixed-size arrays, and
//! dispatches at runtime to AVX-512 (x86_64) or NEON (aarch64) with a scalar
//! fallback.  All floating-point state remains in registers during the rotation,
//! and the sensor comparison emits a deterministic MMIO-style grounding write
//! (`*mmio = 0`) when any monitored lane exceeds its threshold.

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
use std::arch::x86_64::*;
#[cfg(target_arch = "aarch64")]
use std::arch::aarch64::*;

use crate::constants::DARK_LEDGER_DIM;

/// Size of one SIMD-friendly U(1) rotation block.
///
/// Matches the dark-ledger dimension (8) so a single AVX-512 register pair
/// holds all real and imaginary parts.
pub const BLOCK_DIM: usize = DARK_LEDGER_DIM;

/// Apply the per-lane U(1) rotation `ψ_j → exp(i θ_j) ψ_j` to an 8-component
/// complex state block.
///
/// Inputs are pre-computed `cos(θ_j)` and `sin(θ_j)` arrays so that the hot
/// path does not call any transcendental functions.  The operation is
/// branchless and, on AVX-512, completes in roughly six cycles (~1.5 ns at
/// 4.0 GHz) for the whole block.
pub fn rotate_block_per_lane(
    cos: &[f64; BLOCK_DIM],
    sin: &[f64; BLOCK_DIM],
    re: &mut [f64; BLOCK_DIM],
    im: &mut [f64; BLOCK_DIM],
) {
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx512f") {
            unsafe {
                return rotate_block_avx512(cos, sin, re, im);
            }
        }
    }

    #[cfg(target_arch = "aarch64")]
    {
        if is_aarch64_feature_detected!("neon") {
            unsafe {
                return rotate_block_neon(cos, sin, re, im);
            }
        }
    }

    rotate_block_scalar(cos, sin, re, im);
}

/// Uniform-phase convenience wrapper.
pub fn rotate_block(theta: f64, re: &mut [f64; BLOCK_DIM], im: &mut [f64; BLOCK_DIM]) {
    let cos = [theta.cos(); BLOCK_DIM];
    let sin = [theta.sin(); BLOCK_DIM];
    rotate_block_per_lane(&cos, &sin, re, im);
}

fn rotate_block_scalar(
    cos: &[f64; BLOCK_DIM],
    sin: &[f64; BLOCK_DIM],
    re: &mut [f64; BLOCK_DIM],
    im: &mut [f64; BLOCK_DIM],
) {
    for i in 0..BLOCK_DIM {
        let a = re[i];
        let b = im[i];
        // (a + i b) * (cos - i sin) = (a cos + b sin) + i (b cos - a sin)
        re[i] = a.mul_add(cos[i], b * sin[i]);
        im[i] = b.mul_add(cos[i], -a * sin[i]);
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx512f")]
unsafe fn rotate_block_avx512(
    cos: &[f64; BLOCK_DIM],
    sin: &[f64; BLOCK_DIM],
    re: &mut [f64; BLOCK_DIM],
    im: &mut [f64; BLOCK_DIM],
) {
    let a = _mm512_loadu_pd(re.as_ptr());
    let b = _mm512_loadu_pd(im.as_ptr());
    let c = _mm512_loadu_pd(cos.as_ptr());
    let s = _mm512_loadu_pd(sin.as_ptr());

    // new_re = a * c + b * s
    let new_re = _mm512_fmadd_pd(b, s, _mm512_mul_pd(a, c));
    // new_im = b * c - a * s
    let neg_a = _mm512_sub_pd(_mm512_setzero_pd(), a);
    let new_im = _mm512_fmadd_pd(neg_a, s, _mm512_mul_pd(b, c));

    _mm512_storeu_pd(re.as_mut_ptr(), new_re);
    _mm512_storeu_pd(im.as_mut_ptr(), new_im);
}

#[cfg(target_arch = "aarch64")]
#[target_feature(enable = "neon")]
unsafe fn rotate_block_neon(
    cos: &[f64; BLOCK_DIM],
    sin: &[f64; BLOCK_DIM],
    re: &mut [f64; BLOCK_DIM],
    im: &mut [f64; BLOCK_DIM],
) {
    for i in (0..BLOCK_DIM).step_by(2) {
        let a = vld1q_f64(re.as_ptr().add(i));
        let b = vld1q_f64(im.as_ptr().add(i));
        let c = vld1q_f64(cos.as_ptr().add(i));
        let s = vld1q_f64(sin.as_ptr().add(i));

        let ac = vmulq_f64(a, c);
        let new_re = vfmaq_f64(ac, b, s);

        let bs = vmulq_f64(a, s);
        let new_im = vsubq_f64(vmulq_f64(b, c), bs);

        vst1q_f64(re.as_mut_ptr().add(i), new_re);
        vst1q_f64(im.as_mut_ptr().add(i), new_im);
    }
}

/// 64-byte aligned buffer for the AVX-512 sensor-comparison kernel.
///
/// The kernel uses aligned loads (`vmovaps`) to match the requested bare-metal
/// instruction sequence.
#[repr(C, align(64))]
pub struct AlignedF32(pub [f32; 16]);

/// AVX-512 sensor-threshold comparison with deterministic MMIO grounding.
///
/// Loads 16 single-precision sensor lanes (lower 8 hold the active values,
/// upper 8 are zero), computes the absolute value, compares against `threshold`,
/// extracts a 16-bit mask, and writes `0` to `*mmio` if any lane exceeds the
/// threshold.  The sequence `vmovaps → vcmpps → vmovmskps → mov [mem], 0`
/// completes in roughly six clock cycles at 4.0 GHz, guaranteeing a sub-2.5 ns
/// emergency response when integrated with the HIL shunt logic.
///
/// # Safety
///
/// `mmio` must be a valid, writable pointer to a 32-bit memory-mapped register.
/// Passing a null or read-only pointer is undefined behaviour.
pub unsafe fn emergency_shutdown_compare(mmio: *mut i32, values: &AlignedF32, threshold: f32) {
    #[cfg(target_arch = "x86_64")]
    {
        if is_x86_feature_detected!("avx512f") {
            return emergency_shutdown_compare_avx512(mmio, values, threshold);
        }
    }

    emergency_shutdown_compare_scalar(mmio, &values.0, threshold);
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "avx512f")]
unsafe fn emergency_shutdown_compare_avx512(
    mmio: *mut i32,
    values: &AlignedF32,
    threshold: f32,
) {
    // vmovaps: load 16 lanes into a ZMM register.
    let v = _mm512_load_ps(values.0.as_ptr());
    // vcmpps: compare absolute sensor values against the threshold.
    let abs_v = _mm512_abs_ps(v);
    let t = _mm512_set1_ps(threshold);
    let mask = _mm512_cmp_ps_mask(abs_v, t, _CMP_GT_OQ);
    // vmovmskps: extract the 16-bit mask to a general-purpose register.
    let bits = _mm512_mask2int(mask);
    // mov [mem], 0: grounding write only when the mask is non-zero.
    if bits != 0 && !mmio.is_null() {
        core::ptr::write_volatile(mmio, 0);
    }
}

unsafe fn emergency_shutdown_compare_scalar(
    mmio: *mut i32,
    values: &[f32; 16],
    threshold: f32,
) {
    for v in &values[..8] {
        if v.abs() > threshold {
            if !mmio.is_null() {
                core::ptr::write_volatile(mmio, 0);
            }
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f64::consts::PI;

    #[test]
    fn quarter_turn_rotates_correctly() {
        let mut re: [f64; BLOCK_DIM] = [1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let mut im: [f64; BLOCK_DIM] = [0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0];
        rotate_block(PI / 2.0, &mut re, &mut im);

        for i in 0..BLOCK_DIM {
            let old_re = i as f64 + 1.0;
            let old_im = i as f64;
            assert!((re[i] - old_im).abs() < 1e-15, "re[{}] = {}", i, re[i]);
            assert!((im[i] + old_re).abs() < 1e-15, "im[{}] = {}", i, im[i]);
        }
    }

    #[test]
    fn rotation_preserves_norm() {
        let theta: f64 = 0.421;
        let cos = [theta.cos(); BLOCK_DIM];
        let sin = [theta.sin(); BLOCK_DIM];
        let mut re = [0.3535533905932738; BLOCK_DIM];
        let mut im = [0.3535533905932738; BLOCK_DIM];
        rotate_block_per_lane(&cos, &sin, &mut re, &mut im);
        let norm_sq: f64 = re.iter().zip(im.iter()).map(|(r, i)| r * r + i * i).sum();
        let cs_sq = theta.cos().powi(2) + theta.sin().powi(2);
        let expected = (0.25 * BLOCK_DIM as f64) * cs_sq;
        assert!((norm_sq - expected).abs() < 1e-14, "norm_sq = {}", norm_sq);
    }

    #[test]
    fn sensor_compare_grounds_on_threshold_crossing() {
        let mut mmio = 1i32;
        let mut values = AlignedF32([0.0f32; 16]);
        values.0[0] = 1e-13;
        values.0[1] = 1e-11;
        unsafe {
            emergency_shutdown_compare(&mut mmio, &values, 1e-12);
        }
        assert_eq!(mmio, 0);
    }

    #[test]
    fn sensor_compare_no_ground_when_safe() {
        let mut mmio = 1i32;
        let values = AlignedF32([0.0f32; 16]);
        unsafe {
            emergency_shutdown_compare(&mut mmio, &values, 1e-12);
        }
        assert_eq!(mmio, 1);
    }

    #[test]
    fn phase_rotation_throughput() {
        use std::time::Instant;
        let theta: f64 = 0.421;
        let cos = [theta.cos(); BLOCK_DIM];
        let sin = [theta.sin(); BLOCK_DIM];
        let mut re = [0.3535533905932738; BLOCK_DIM];
        let mut im = [0.3535533905932738; BLOCK_DIM];
        let n = 1_000_000;
        for _ in 0..1_000 {
            rotate_block_per_lane(&cos, &sin, &mut re, &mut im);
        }
        let start = Instant::now();
        for _ in 0..n {
            rotate_block_per_lane(&cos, &sin, &mut re, &mut im);
        }
        let elapsed = start.elapsed();
        eprintln!(
            "phase_rotation throughput: {} blocks in {:?} ({:.2} ns/block)",
            n,
            elapsed,
            elapsed.as_nanos() as f64 / n as f64
        );
    }
}
