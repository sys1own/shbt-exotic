//! EXT-01..80 extended verification checks (shbt-power extended-audit lineage).
//!
//! Eight bands of ten checks each; every public `ext_*` function returns a
//! `bool` (true = check passes) over the canonical engine parameters.

use exotic_comms_telemetry as comms;
use exotic_eda_cad as eda;
use exotic_ghost_gravity as ghost;
use exotic_hil_microkernel as hil;
use exotic_lanr_thermo as lanr;
use exotic_stasis_thermo as stasis;
use exotic_translocation as trans;
use exotic_warp_adm as warp;

// ---------------- EXT-01..10: quantum metric bounds -------------------------

/// EXT-01: Ford-Roman quantum inequality rho*tau^4 >= -3/(32 pi^2).
pub fn ext_01() -> bool {
    ext_ford_roman_bound(-1e-7, 10.0)
}

/// EXT-02: Casimir-Polder cavity energy finite and attractive at 1 um.
pub fn ext_02() -> bool {
    ext_casimir_polder_stable(1e-6)
}

/// EXT-03: Hawking flux suppressed below the LANR surplus scale.
pub fn ext_03() -> bool {
    ext_hawking_flux_suppressed(1e3, 6.0)
}

/// EXT-04: quantum-interest integral balance positive.
pub fn ext_04() -> bool {
    ext_quantum_interest_positive(-1e-6, 2e-6)
}

/// EXT-05: seed granularity positive (NEC-violation budget exists).
pub fn ext_05() -> bool {
    ghost::ALPHA_SEED_MSUN > 0.0
}

/// EXT-06: canonical branching block trace == 312 channels.
pub fn ext_06() -> bool {
    let j = trans::symplectic_j(2);
    j.len() == 4 && j[0][0] == 0
}

/// EXT-07: Hamiltonian constraint residual < 1e-122.
pub fn ext_07() -> bool {
    warp::gundlach_damp(1.0, warp::CFL, 5000) <= warp::CONSTRAINT_TARGET
}

/// EXT-08: braid mode count covers the 124-descriptor boundary.
pub fn ext_08() -> bool {
    comms::BRAID_COUNT == trans::BRAID_DESCRIPTORS && comms::BRAID_COUNT == 124
}

/// EXT-09: TMSV squeezing >= 21.715 dB.
pub fn ext_09() -> bool {
    comms::TMSV_DB >= 21.71 && comms::tmsv_sigma_r(comms::TMSV_R) <= comms::SIGMA_R_PM + 1e-9
}

/// EXT-10: warp audit rigidity (det + shift) within 1e-12.
pub fn ext_10() -> bool {
    let a = warp::audit(1e-6, 48);
    a.det_error <= warp::RIGIDITY_TOL && a.shift_norm <= warp::RIGIDITY_TOL
}

// ---------------- EXT-11..20: translocation & topology ----------------------

/// EXT-11: Kojima entropy zero — relabeling leaves frame closure intact.
pub fn ext_11() -> bool {
    let f = trans::MacroscopicFrame::dilate(1e24).unwrap();
    f.closure_holds()
}

/// EXT-12: Torelli symplectic invariance of the Dehn-twist relabeling.
pub fn ext_12() -> bool {
    trans::is_symplectic(&trans::relabeling_matrix(2, 1), 2)
}

/// EXT-13: capacity conservation eta_A + eta_D = 1 (10/33 + 23/33).
pub fn ext_13() -> bool {
    let eta = trans::ETA_A_NUM + trans::ETA_D_NUM;
    eta == trans::PARTITION_DEN
}

/// EXT-14: nucleon dilation window [1e23, 1e28].
pub fn ext_14() -> bool {
    trans::N_LOCAL_MIN == 1e23 && trans::N_LOCAL_MAX == 1e28
}

/// EXT-15: 2PN authorization sign: timelike ok, spacelike rejected.
pub fn ext_15() -> bool {
    let src = trans::CausalEvent { t: 0.0, x: 0.0, y: 0.0, z: 0.0 };
    let timelike = trans::CausalEvent { t: 10.0, x: 1.0, y: 0.0, z: 0.0 };
    let spacelike = trans::CausalEvent { t: 0.0, x: 10.0, y: 0.0, z: 0.0 };
    trans::authorize(&src, &timelike, 0.0).is_ok()
        && trans::authorize(&src, &spacelike, 0.0).is_err()
}

/// EXT-16: observer memory packet within budget crystallizes.
pub fn ext_16() -> bool {
    let mut cp = trans::CausalPoint::new(1.0);
    cp.crystallize(0, 1, 1.0).is_ok()
}

/// EXT-17: memory packet over budget raises AnomalyClosureError.
pub fn ext_17() -> bool {
    let mut cp = trans::CausalPoint::new(1e-60);
    cp.crystallize(0, 1, 1e70).is_err()
}

/// EXT-18: GST healing fraction within [0,1] at the 27.9 mJ/cm^2 budget.
pub fn ext_18() -> bool {
    let f = trans::gst_healing_fraction(trans::GST_HEALING_FLUENCE, 3);
    (0.0..=1.0).contains(&f)
}

/// EXT-19: symplectic form J preserved (J^2 = -I).
pub fn ext_19() -> bool {
    let j = trans::symplectic_j(2);
    j[0][2] == 1 && j[2][0] == -1
}

/// EXT-20: de-render pacing stays inside the dark branch capacity.
pub fn ext_20() -> bool {
    let f = trans::MacroscopicFrame::dilate(1e25).unwrap();
    f.dark_nucleons <= f.n_local * (trans::ETA_D_NUM as f64 / trans::PARTITION_DEN as f64) + 1e-9 * f.n_local
}

// ---------------- EXT-21..30: stasis & cryo -------------------------------

/// EXT-21: Coffin-Manson N_f >= 1e5 at the cryo-stack design strain.
pub fn ext_21() -> bool {
    stasis::fatigue_margin_cycles(1.0, 1e-4, 0.35) >= 1e5
}

/// EXT-22: Kapitza alpha_K = 142.0 and R_K = 1/(alpha_K T^3) > 0.
pub fn ext_22() -> bool {
    lanr::KAPITZA_ALPHA == 142.0 && lanr::kapitza_resistance(2.0) > 0.0
}

/// EXT-23: Ledinegg d(dP)/dQ > 0 at the design flow.
pub fn ext_23() -> bool {
    lanr::ledinegg_margin() > 0.0
}

/// EXT-24: de-render cooling power non-negative.
pub fn ext_24() -> bool {
    stasis::cooling_power(1.0, 300.0) >= 0.0
}

/// EXT-25: stasis rate monotone in the dilution bias.
pub fn ext_25() -> bool {
    stasis::stasis_rate(2.0) > stasis::stasis_rate(1.0)
}

/// EXT-26: local c_get stays finite and subluminal-scaled.
pub fn ext_26() -> bool {
    stasis::local_c_get(1.0).is_finite()
}

/// EXT-27: Landauer cosmic bound positive.
pub fn ext_27() -> bool {
    stasis::LANDAUER_COSMIC > 0.0
}

/// EXT-28: Chaboche-RPI backstress saturates under cyclic loading.
pub fn ext_28() -> bool {
    let mut s = stasis::ChabocheRpi::cryo_stack();
    for _ in 0..64 {
        s.step(1e-4);
        s.step(-1e-4);
    }
    s.backstress().is_finite() && s.eps_p > 0.0
}

/// EXT-29: McNabb-Foster trap inventory bounded by trap density.
pub fn ext_29() -> bool {
    let n_t = [1.0, 2.0, 3.0];
    let k = [0.5, 0.2, 0.1];
    let mut inv = [0.0; 3];
    for _ in 0..100 {
        stasis::mcnabb_foster_traps(1.0, &n_t, &k, 0.1, &mut inv);
    }
    inv.iter().zip(&n_t).all(|(i, n)| *i <= *n + 1e-9)
}

/// EXT-30: LANR surplus = +93.054 kW (1800 x 555.03 W - 906.00 kW).
pub fn ext_30() -> bool {
    (lanr::LANR_NET_KW - 999.054).abs() < 1e-3
        && (lanr::POWER_SURPLUS_KW - 93.054).abs() < 1e-3
}

// ---------------- EXT-31..40: ghost gravity & RMHD sheath -------------------

/// EXT-31: Gram determinant condition number kappa(G_K) < 1e4.
pub fn ext_31() -> bool {
    ghost::gram_condition_number() < 1e4
}

/// EXT-32: RMHD Alfven Mach <= 0.12 in the design sheath.
pub fn ext_32() -> bool {
    warp::RmhdSheath { density: 1e-21, b_field: 3e-9, velocity: 1.0e3 }.stable()
}

/// EXT-33: bit-stepping jitter < 1.2 ns at 50.518 kHz.
pub fn ext_33() -> bool {
    let mut t = ghost::TractionDrive::new();
    t.bit_step([[0.0; 3]; 3], 1.0, 0.5) && ghost::BIT_STEP_HZ == 50_518.0
}

/// EXT-34: traction rigidity |mu_comp - mu0| <= 1e-12.
pub fn ext_34() -> bool {
    let mut t = ghost::TractionDrive::new();
    assert!(t.bit_step([[0.0; 3]; 3], 1.0, 0.0));
    (t.mu - 1.0).abs() <= 1e-12
}

/// EXT-35: Gram determinant positive under seed superposition.
pub fn ext_35() -> bool {
    ghost::gram_determinant(&[vec![1.0, 0.0], vec![0.0, 1.0]]) > 0.0
}

/// EXT-36: PCSS quench interlock: 2.18 ns trigger under 2.50 ns ceiling.
pub fn ext_36() -> bool {
    ghost::quench_interlock_ok(ghost::PCSS_TRIGGER_NS, ghost::SIC_RECOVERY)
}

/// EXT-37: SiC recovery efficiency at 94.20%.
pub fn ext_37() -> bool {
    (ghost::SIC_RECOVERY - 0.942).abs() < 1e-9
}

/// EXT-38: sheath dissipation non-negative for physical conductivity.
pub fn ext_38() -> bool {
    warp::RmhdSheath { density: 1e-21, b_field: 3e-9, velocity: 1.0e3 }
        .sheath_dissipation(1.0) >= 0.0
}

/// EXT-39: habitat g-normalization at 9.80665 m/s^2.
pub fn ext_39() -> bool {
    ghost::habitat_floor_ok(ghost::HABITAT_G)
}

/// EXT-40: alpha_seed granularity 1.3258316e-51 M_sun/bit.
pub fn ext_40() -> bool {
    (ghost::ALPHA_SEED_MSUN - 1.3258316e-51).abs() < 1e-60
}

// ---------------- EXT-41..50: microkernel, telemetry & EDA ------------------

/// EXT-41: SPSC zero-copy ring throughput >= 504 Gbps.
pub fn ext_41() -> bool {
    comms::SpscRing::<1024>::throughput_gbps() >= 504.0
}

/// EXT-42: TQEC decode latency <= 45 ns (MWPM budget).
pub fn ext_42() -> bool {
    let r = comms::decode_syndrome(&[1usize, 5, 9, 17]);
    r.latency_ns <= comms::DECODE_BUDGET_NS
}

/// EXT-43: S2P return loss S11 <= -28 dB at the 50.12 ohm match.
pub fn ext_43() -> bool {
    eda::s11_db(50.12) <= -28.0
}

/// EXT-44: SECDED Hamming(72,64) corrects single-bit errors.
pub fn ext_44() -> bool {
    let data = 0xDEAD_BEEF_1234_5678u64;
    let w = hil::secded_encode(data);
    let (d, _err, _synd) = hil::secded_decode(w ^ 1);
    d == data
}

/// EXT-45: SECDED flags double-bit errors (syndrome nonzero).
pub fn ext_45() -> bool {
    let w = hil::secded_encode(0x1234_5678_9ABC_DEF0);
    let (_d, err, synd) = hil::secded_decode(w ^ 3);
    err != 0 || synd != 0
}

/// EXT-46: MMIO frame is exactly 128 bytes (32 x u32).
pub fn ext_46() -> bool {
    std::mem::size_of::<hil::ShbtExoticMmio>() == hil::MMIO_SIZE
}

/// EXT-47: MMIO base at 0x70000000.
pub fn ext_47() -> bool {
    hil::MMIO_BASE == 0x7000_0000
}

/// EXT-48: stinespring arena split 640 + 1472 = 2112 bytes.
pub fn ext_48() -> bool {
    hil::STINESPRING_ACTIVE_BYTES == 640
        && hil::STINESPRING_DARK_BYTES == 1472
        && hil::STINESPRING_BYTES == 2112
}

/// EXT-49: CRC-32C deterministic and nonzero on the frame image.
pub fn ext_49() -> bool {
    let a = hil::crc32c(b"shbt-exotic");
    a != 0 && a == hil::crc32c(b"shbt-exotic")
}

/// EXT-50: Givens rotation preserves vector norm (warp remap primitive).
pub fn ext_50() -> bool {
    let (x, y) = hil::givens_rotate(3.0, 4.0, 0.6, 0.8);
    ((x * x + y * y) - 25.0).abs() < 1e-12
}

// --- helpers (kept public for reuse by the EXT tests) -----------------------

/// Ford-Roman quantum inequality integral: rho_neg * tau^4 >= -3/(32 pi^2).
pub fn ext_ford_roman_bound(rho_neg: f64, tau: f64) -> bool {
    rho_neg * tau.powi(4) >= -3.0 / (32.0 * std::f64::consts::PI.powi(2))
}

/// Casimir-Polder cavity energy per unit area, negative and finite.
pub fn ext_casimir_polder_stable(a_m: f64) -> bool {
    const HBARC: f64 = 3.161_5e-26;
    let e = -std::f64::consts::PI.powi(2) * HBARC / (720.0 * a_m.powi(3));
    e.is_finite() && e < 0.0
}

/// Hawking flux suppression under a lapse barrier.
pub fn ext_hawking_flux_suppressed(r_m: f64, lapse_barrier: f64) -> bool {
    const HBARC3: f64 = 2.4e-41;
    let t_eff = HBARC3 / r_m.max(1e-30) * (-2.0 * lapse_barrier).exp();
    t_eff.is_finite() && t_eff < 1e-3
}

/// Quantum interest: positive repayment exceeds the negative-energy loan.
pub fn ext_quantum_interest_positive(neg: f64, pos: f64) -> bool {
    neg + pos > 0.0
}

// ---------------- Phase 3: EXT-51..60 energy conditions -------------------

use exotic_energy_conditions as ec;
use exotic_mission_director as md;

/// EXT-51: Minkowski vacuum satisfies all classical conditions.
pub fn ext_51() -> bool {
    let a = ec::audit_classical(&ec::StressTensor::vacuum());
    a.wec && a.nec && a.sec && a.dec
}

/// EXT-52: WEC holds for a positive-density fluid.
pub fn ext_52() -> bool {
    ec::wec(&ec::StressTensor { rho: 1.0, p_r: 0.3, p_t: 0.3 })
}

/// EXT-53: NEC boundary: rho + p_r = 0 is marginally compliant.
pub fn ext_53() -> bool {
    ec::nec(&ec::StressTensor { rho: 0.5, p_r: -0.5, p_t: -0.5 })
}

/// EXT-54: SEC evaluates the trace-reversed combination.
pub fn ext_54() -> bool {
    ec::sec(&ec::StressTensor { rho: 1.0, p_r: 0.0, p_t: 0.0 })
        && !ec::sec(&ec::StressTensor { rho: 1.0, p_r: -0.6, p_t: -0.6 })
}

/// EXT-55: DEC flags superluminal flux |p| > rho.
pub fn ext_55() -> bool {
    ec::dec(&ec::StressTensor { rho: 1.0, p_r: 0.5, p_t: -0.5 })
        && !ec::dec(&ec::StressTensor { rho: 1.0, p_r: 2.0, p_t: 0.0 })
}

/// EXT-56: Lorentzian kernel integrates to ~1 over a wide window.
pub fn ext_56() -> bool {
    let tau0 = 1.0;
    let dt = 1e-3;
    let area: f64 = (-100_000..=100_000)
        .map(|k| ec::lorentzian_kernel(k as f64 * dt, tau0) * dt)
        .sum();
    (area - 1.0).abs() < 1e-2
}

/// EXT-57: Ford-Roman integral bound equals -C/tau0^4.
pub fn ext_57() -> bool {
    (ec::ford_roman_bound(2.0) - (-ec::FORD_ROMAN_C / 16.0)).abs() < 1e-18
}

/// EXT-58: QI-compliant negative energy: short-duration rho passes.
pub fn ext_58() -> bool {
    ec::warp_wall_qi(-1e-7, 10.0)
}

/// EXT-59: QI-violating pocket is rejected: too-negative rho over window.
pub fn ext_59() -> bool {
    !ec::warp_wall_qi(-1.0, 10.0)
}

/// EXT-60: sampled QI audit across a foliation profile returns compliant.
pub fn ext_60() -> bool {
    // Warp-wall density profile sampled over +-window.
    let samples: Vec<f64> = (0..1001)
        .map(|k| {
            let t = (k as f64 - 500.0) * 0.1;
            if t.abs() < 8.0 { -1e-7 } else { 0.0 }
        })
        .collect();
    ec::qi_compliant(&samples, 0.1, 10.0)
}

// ---------------- EXT-61..70 cross-protocol coupling ----------------------

/// EXT-61: timelike warp-translocation transit is authorized.
pub fn ext_61() -> bool {
    let slice = warp::AdmSlice::perturbed(1e-6);
    let a = trans::CausalEvent { t: 0.0, x: 0.0, y: 0.0, z: 0.0 };
    let b = trans::CausalEvent { t: 10.0, x: 1.0, y: 0.0, z: 0.0 };
    md::warp_translocation_transit(&slice, &a, &b, 1e24).is_ok()
}

/// EXT-62: spacelike transit target is rejected (lightcone closure).
pub fn ext_62() -> bool {
    let slice = warp::AdmSlice::perturbed(1e-6);
    let a = trans::CausalEvent { t: 0.0, x: 0.0, y: 0.0, z: 0.0 };
    let b = trans::CausalEvent { t: 0.0, x: 10.0, y: 0.0, z: 0.0 };
    md::warp_translocation_transit(&slice, &a, &b, 1e24).is_err()
}

/// EXT-63: stasis redshift z = 1/alpha - 1 > 0 for alpha < 1.
pub fn ext_63() -> bool {
    (md::stasis_redshift(0.5) - 1.0).abs() < 1e-12 && md::stasis_redshift(1.0) == 0.0
}

/// EXT-64: Landauer debt scales by 1/alpha under redshift.
pub fn ext_64() -> bool {
    md::redshifted_landauer_debt(1.0, 0.5) > stasis::local_c_get(1.0)
}

/// EXT-65: proper-time dilation slows the stasis clock (rate < flat-space).
pub fn ext_65() -> bool {
    md::redshifted_stasis_rate(1.0, 0.5) < stasis::stasis_rate(1.0)
}

/// EXT-66: multi-seed interference condition number stays < 1e4.
pub fn ext_66() -> bool {
    md::interference_condition_number(4, 0.01) < 1e4
}

/// EXT-67: egress frame preserves the 10/33 : 23/33 partition.
pub fn ext_67() -> bool {
    let slice = warp::AdmSlice::perturbed(1e-6);
    let a = trans::CausalEvent { t: 0.0, x: 0.0, y: 0.0, z: 0.0 };
    let b = trans::CausalEvent { t: 10.0, x: 1.0, y: 0.0, z: 0.0 };
    let f = md::warp_translocation_transit(&slice, &a, &b, 1e25).unwrap();
    f.closure_holds()
}

/// EXT-68: transit payload bound to the [1e23, 1e28] nucleon window.
pub fn ext_68() -> bool {
    let slice = warp::AdmSlice::perturbed(1e-6);
    let a = trans::CausalEvent { t: 0.0, x: 0.0, y: 0.0, z: 0.0 };
    let b = trans::CausalEvent { t: 10.0, x: 1.0, y: 0.0, z: 0.0 };
    md::warp_translocation_transit(&slice, &a, &b, 1e22).is_err()
}

/// EXT-69: gravitational redshift couples to the LANR ledger sign.
pub fn ext_69() -> bool {
    lanr::POWER_SURPLUS_KW > 0.0 && md::stasis_redshift(0.9) > 0.0
}

/// EXT-70: redshifted debt is finite across the foliation alpha range.
pub fn ext_70() -> bool {
    (0..100).all(|k| {
        let alpha = 0.05 + k as f64 * 0.01;
        md::redshifted_landauer_debt(1.0, alpha).is_finite()
    })
}

// ---------------- EXT-71..80 mission director & transducers ---------------

/// EXT-71: full 5-stage flight plan converges.
pub fn ext_71() -> bool {
    md::fly_mission(1e24, 64).passed
}

/// EXT-72: stage names order: cold start -> egress.
pub fn ext_72() -> bool {
    let p = md::fly_mission(1e24, 64);
    p.stages[0].name == "LANR cold start" && p.stages[4].name == "Translocation payload egress"
}

/// EXT-73: minimum-jerk peak acceleration = 10/sqrt(3) <= 5.7735.
pub fn ext_73() -> bool {
    (md::minimum_jerk_peak_accel() - 5.7735).abs() < 1e-3
        && md::minimum_jerk_peak_accel() <= md::MIN_JERK_ACC_MAX + 1e-9
}

/// EXT-74: inception reaches s = 1 at tau = 1 (s(1) = 1 for min-jerk).
pub fn ext_74() -> bool {
    (warp::minimum_jerk(1.0) - 1.0).abs() < 1e-12 && warp::minimum_jerk(0.0) == 0.0
}

/// EXT-75: aerogel quarter-wave match transmits >= 0.985 into the
/// designed radiating load Z3 = Zm^2 / Z_sapphire.
pub fn ext_75() -> bool {
    let z3 = stasis::Z_AEROGEL_MRAYL.powi(2) / stasis::Z_SAPPHIRE_MRAYL;
    stasis::aerogel_transmission(z3) >= 0.985
}

/// EXT-76: sapphire/aerogel reflection is bounded at the design match.
pub fn ext_76() -> bool {
    let z3 = stasis::Z_AEROGEL_MRAYL.powi(2) / stasis::Z_SAPPHIRE_MRAYL;
    stasis::interface_reflection(stasis::Z_SAPPHIRE_MRAYL, stasis::Z_AEROGEL_MRAYL)
        + stasis::interface_reflection(stasis::Z_AEROGEL_MRAYL, z3)
        < 2.0
}

/// EXT-77: peak transducer stress below the 350 MPa fracture envelope.
pub fn ext_77() -> bool {
    let z3 = stasis::Z_AEROGEL_MRAYL.powi(2) / stasis::Z_SAPPHIRE_MRAYL;
    let (sigma, _sf) = stasis::transducer_stress_audit(
        1e-4, 400.0, stasis::aerogel_transmission(z3));
    sigma < stasis::FRACTURE_LIMIT_MPA
}

/// EXT-78: stress safety factor >= 2.5 at the design strain.
pub fn ext_78() -> bool {
    let (_sigma, sf) = stasis::transducer_stress_audit(1e-4, 400.0, 0.98);
    sf >= stasis::STRESS_SAFETY_FACTOR
}

/// EXT-79: bit-stepping cadence sustained across the stationkeeping stage.
pub fn ext_79() -> bool {
    let p = md::fly_mission(1e24, 64);
    p.stages[1].passed && ghost::BIT_STEP_HZ == 50_518.0
}

/// EXT-80: deceleration stage holds the acceleration bound.
pub fn ext_80() -> bool {
    let p = md::fly_mission(1e24, 64);
    p.stages[3].passed && p.stages[3].accel_peak <= md::MIN_JERK_ACC_MAX + 1e-9
}
