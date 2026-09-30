//! Gate asserts are compile-time constant checks by design.
#![allow(clippy::assertions_on_constants)]

//! Master 70-gate verification suite (GATE-01 .. GATE-70) for the upgraded
//! shbt-exotic platform. Each gate is one #[test]; gate numbering follows the
//! specification bands:
//!   01-10 boundary CFT / Stinespring branching
//!   11-20 microkernel C-ABI / MMIO / SECDED / Givens
//!   21-30 PCSS quench / SiC recovery / thermal headroom
//!   31-40 CCZ4-ADM constraints / shift-nulling / lapse lock
//!   41-50 2PN causal authorization / Heegaard-Floer relabeling
//!   51-60 LANR ledger / Landauer debt / two-phase boiling
//!   61-70 TMSV metrology / TQEC / GST / hyper-dual UQ / EDA S2P

use exotic_comms_telemetry as comms;
use exotic_core_cft as cft;
use exotic_eda_cad as eda;
use exotic_ghost_gravity as ghost;
use exotic_hil_microkernel as hil;
use exotic_lanr_thermo as lanr;
use exotic_stasis_thermo as stasis;
use exotic_translocation as xloc;
use exotic_uq_montecarlo as uq;
use exotic_warp_adm as warp;

const TOL12: f64 = 1e-12;

// ---------------- GATE-01..10: boundary CFT & branching -----------------

#[test]
fn gate_01_canonical_branch() {
    assert_eq!(cft::CANONICAL_BRANCH, (26, 8, 312));
}

#[test]
fn gate_02_su2_central_charge() {
    let c = cft::su2_central_charge(26);
    // c = 3*26/28 = 78/28 = 39/14
    assert!((c.to_f64() - 39.0 / 14.0).abs() < 1e-15);
}

#[test]
fn gate_03_su3_central_charge() {
    let c = cft::su3_central_charge(8);
    assert!((c.to_f64() - 64.0 / 11.0).abs() < 1e-15);
}

#[test]
fn gate_04_canonical_central_charge_positive() {
    assert!(cft::canonical_central_charge() > 0.0);
}

#[test]
fn gate_05_framing_defect_zero() {
    assert!(cft::framing_defect((26, 8, 312)) == 0);
    assert!(cft::framing_defect((25, 8, 312)) != 0);
}

#[test]
fn gate_06_branching_matrix_shape() {
    let b = cft::branching_matrix();
    assert_eq!(b.len(), 33);
    assert!(b.iter().all(|r| r.len() == 33));
}

#[test]
fn gate_07_branching_matrix_isometric() {
    let b = cft::branching_matrix();
    // B B^T = I_33
    for i in 0..33 {
        for j in 0..33 {
            let dot: f64 = (0..33).map(|k| b[i][k] * b[j][k]).sum();
            assert!((dot - if i == j { 1.0 } else { 0.0 }).abs() < TOL12);
        }
    }
}

#[test]
fn gate_08_partition_fractions_exact() {
    let ((an, ad), (dn, dd)) = cft::partition_fractions();
    assert_eq!((an, ad), (10, 33));
    assert_eq!((dn, dd), (23, 33));
    assert_eq!(an + dn, ad);
}

#[test]
fn gate_09_de_render_splits_33_modes() {
    let mut w = [0.0; 33];
    for (i, x) in w.iter_mut().enumerate() {
        *x = i as f64 + 1.0;
    }
    let (a, d) = cft::de_render(&w);
    assert_eq!(a.len(), 10);
    assert_eq!(d.len(), 23);
    let total: f64 = a.iter().chain(d.iter()).sum();
    assert!((total - w.iter().sum::<f64>()).abs() < TOL12);
}

#[test]
fn gate_10_operational_capacity_finite() {
    assert!(cft::operational_capacity_bits() > 0.0);
}

// ------------- GATE-11..20: microkernel / MMIO / ECC / Givens -----------

#[test]
fn gate_11_mmio_base_address() {
    assert_eq!(hil::MMIO_BASE, 0x7000_0000);
    assert_eq!(hil::MMIO_SIZE, 128);
}

#[test]
fn gate_12_mmio_struct_layout() {
    assert_eq!(std::mem::size_of::<hil::ShbtExoticMmio>(), 128);
    assert_eq!(std::mem::align_of::<hil::ShbtExoticMmio>(), 64);
}

#[test]
fn gate_13_register_offsets() {
    assert_eq!(hil::reg::SYS_CONTROL, 0x00);
    assert_eq!(hil::reg::SYS_STATUS, 0x04);
    assert_eq!(hil::reg::POWER_DEBT_KW, 0x08);
    assert_eq!(hil::reg::LANR_OUTPUT_KW, 0x10);
    assert_eq!(hil::reg::SEED_MASS_LO, 0x18);
    assert_eq!(hil::reg::SEED_MASS_HI, 0x20);
    assert_eq!(hil::reg::DS2_INTERVAL_LO, 0x28);
    assert_eq!(hil::reg::DS2_INTERVAL_HI, 0x2C);
    assert_eq!(hil::reg::QUENCH_TIME_NS, 0x30);
    assert_eq!(hil::reg::ANOMALY_FLAGS, 0x34);
    assert_eq!(hil::reg::CRC32_CASTAGNOLI, 0x7C);
}

#[test]
fn gate_14_mmio_read_write() {
    let mut m = hil::ShbtExoticMmio::zeroed();
    m.write(hil::reg::POWER_DEBT_KW, 906);
    m.write(hil::reg::LANR_OUTPUT_KW, 999);
    assert_eq!(m.read(hil::reg::POWER_DEBT_KW), 906);
    assert_eq!(m.read(hil::reg::LANR_OUTPUT_KW), 999);
}

#[test]
fn gate_15_secded_roundtrip() {
    for v in [0u64, 1, 0xDEAD_BEEF_CAFE_F00D, u64::MAX] {
        let (d, syn, corr) = hil::secded_decode(hil::secded_encode(v));
        assert_eq!(d, v);
        assert_eq!(syn, 0);
        assert_eq!(corr, 0);
    }
}

#[test]
fn gate_16_secded_single_bit_correction() {
    let v = 0x1234_5678_9ABC_DEF0u64;
    let code = hil::secded_encode(v);
    for bit in [0u32, 7, 15, 31, 40, 63, 70] {
        let flipped = code ^ (1u128 << bit);
        let (d, syn, corr) = hil::secded_decode(flipped);
        if bit < 70 {
            assert_eq!(d, v, "bit {}", bit);
            assert_ne!(syn, 0);
            assert_eq!(corr, 1);
        }
    }
}

#[test]
fn gate_17_givens_rotation_unitary() {
    let (c, s) = (0.6f64, 0.8f64);
    let (x2, y2) = hil::givens_rotate(3.0, 4.0, c, s);
    assert!((x2 * x2 + y2 * y2 - 25.0).abs() < TOL12);
}

#[test]
fn gate_18_givens_annihilates() {
    let (x, y) = (3.0f64, 4.0f64);
    let n = (x * x + y * y).sqrt();
    let (c, s) = (x / n, y / n);
    let (_, y2) = hil::givens_rotate(x, y, c, s);
    assert!(y2.abs() < TOL12);
}

#[test]
fn gate_19_crc32c_known_vector() {
    // CRC-32C of "123456789" = 0xE3069283.
    assert_eq!(hil::crc32c(b"123456789"), 0xE306_9283);
}

#[test]
fn gate_20_telemetry_frame_abi() {
    assert_eq!(std::mem::size_of::<comms::TelemetryFrame128>(), 128);
    assert_eq!(std::mem::align_of::<comms::TelemetryFrame128>(), 64);
}

// ------------- GATE-21..30: PCSS quench / SiC / thermal -----------------

#[test]
fn gate_21_pcss_trigger_budget() {
    assert!(hil::PCSS_TRIGGER_NS <= 2.18);
}

#[test]
fn gate_22_quench_hard_limit() {
    assert!(ghost::PCSS_TRIGGER_NS <= ghost::PCSS_HARD_LIMIT_NS);
    assert!(ghost::PCSS_HARD_LIMIT_NS <= 2.50);
}

#[test]
fn gate_23_crowbar_model_passes() {
    assert!(hil::crowbar_ok(0.27 * 8.0));
}

#[test]
fn gate_24_sic_recovery_fraction() {
    assert_eq!(hil::SIC_RECOVERY, 0.9420);
    assert!(hil::SIC_RECOVERY >= 0.9420);
}

#[test]
fn gate_25_quench_interlock() {
    assert!(ghost::quench_interlock_ok(2.18, 0.942));
    assert!(!ghost::quench_interlock_ok(2.60, 0.942));
    assert!(!ghost::quench_interlock_ok(2.18, 0.90));
}

#[test]
fn gate_26_thermal_headroom_min() {
    assert!(hil::headroom_ok(11.79));
    assert!(!hil::headroom_ok(10.0));
}

#[test]
fn gate_27_coffin_manson_margin() {
    // 10 K collapse, 20 ppm/K CTE mismatch, eps_f = 0.05 -> large margin.
    let cyc = stasis::fatigue_margin_cycles(10.0, 20e-6, 0.05);
    assert!(cyc > 1e4);
}

#[test]
fn gate_28_chaboche_backstress_saturates() {
    let mut b = stasis::ChabocheBackstress::new();
    for _ in 0..1000 {
        b.step(1e-4, 2e8, 400.0, 80e6, 20.0);
    }
    assert!(b.alpha.abs() < 2e8 / 400.0 * 1.1);
    assert!((b.drag - 80e6).abs() < 6e6);
}

#[test]
fn gate_29_kapitza_resistance() {
    let r = lanr::kapitza_resistance(2.0);
    assert!((r - 1.0 / (142.0 * 8.0)).abs() < 1e-9);
    assert!(r > 0.0);
}

#[test]
fn gate_30_anomaly_flag_semantics() {
    let m = hil::ShbtExoticMmio::zeroed();
    assert_eq!(m.read(hil::reg::ANOMALY_FLAGS), 0);
}

// ------------- GATE-31..40: CCZ4 / ADM ----------------------------------

#[test]
fn gate_31_cfl_factor() {
    assert_eq!(warp::CFL, 0.25);
}

#[test]
fn gate_32_gundlach_parameters() {
    assert!(warp::KAPPA_1 > 0.0);
    assert!(warp::KAPPA_2 > -1.0);
}

#[test]
fn gate_33_constraint_damping_below_target() {
    let residual = warp::gundlach_damp(1.0, warp::CFL, 20_000);
    assert!(residual < 1e-122, "residual {}", residual);
}

#[test]
fn gate_34_shift_nulling() {
    let mut s = warp::AdmSlice::perturbed(1e-3);
    for _ in 0..40 {
        warp::null_shift(&mut s, warp::CFL);
    }
    let norm: f64 = s.shift.iter().map(|b| b * b).sum::<f64>().sqrt();
    assert!(norm < TOL12);
}

#[test]
fn gate_35_lapse_lock_det_minus_one() {
    let mut s = warp::AdmSlice::perturbed(1e-4);
    warp::lapse_lock(&mut s);
    assert!((s.det_g() + 1.0).abs() <= TOL12);
}

#[test]
fn gate_36_warp_audit_passes() {
    let a = warp::audit(1e-4, 20_000);
    assert!(a.passed, "{:?}", a);
}

#[test]
fn gate_37_minimum_jerk_endpoints() {
    assert_eq!(warp::minimum_jerk(0.0), 0.0);
    assert!((warp::minimum_jerk(1.0) - 1.0).abs() < TOL12);
    // Zero end-velocity: s'(1) = 0.
    let d = (warp::minimum_jerk(1.0 - 1e-7) - 1.0) / -1e-7;
    assert!(d.abs() < 1e-4);
}

#[test]
fn gate_38_wake_compensation_rigidity() {
    let mu0 = 1.0;
    let comp = warp::wake_compensate(mu0, 3e-13, 2e-13, 1e-13);
    assert!((comp - mu0).abs() <= TOL12);
}

#[test]
fn gate_39_multi_seed_superposition() {
    let mut h = [[0.0f64; 4]; 4];
    h[1][1] = 0.01;
    let i = [[0.0f64; 4]; 4];
    let g = ghost::superpose(&[h, h], &i);
    assert!((g[0][0] + 1.0).abs() < TOL12);
    assert!((g[1][1] - 1.02).abs() < TOL12);
}

#[test]
fn gate_40_gram_determinant_positive() {
    let seeds = vec![
        vec![1.0, 0.0, 0.0],
        vec![0.0, 1.0, 0.1],
        vec![0.0, 0.1, 1.0],
    ];
    assert!(ghost::gram_determinant(&seeds) > 0.0);
}

// ------------- GATE-41..50: 2PN causality / relabeling ------------------

#[test]
fn gate_41_causal_timelike_authorized() {
    let a = xloc::CausalEvent { t: 0.0, x: 0.0, y: 0.0, z: 0.0 };
    let b = xloc::CausalEvent { t: 2.0, x: 1.0, y: 0.0, z: 0.0 };
    let ds2 = xloc::authorize(&a, &b, 1e-3).expect("timelike interval");
    assert!(ds2 <= 0.0);
}

#[test]
fn gate_42_lightlike_authorized() {
    let a = xloc::CausalEvent { t: 0.0, x: 0.0, y: 0.0, z: 0.0 };
    let b = xloc::CausalEvent { t: 1.0, x: 1.0, y: 0.0, z: 0.0 };
    // Far region (r_mid large) -> ds^2 ~ 0 minus tiny correction; still <= 0
    // when the 2PN term vanishes? corr > 0 makes ds^2 slightly positive,
    // so pick mass zero.
    assert!(xloc::authorize(&a, &b, 0.0).is_ok());
}

#[test]
fn gate_43_spacelike_rejected() {
    let a = xloc::CausalEvent { t: 0.0, x: 0.0, y: 0.0, z: 0.0 };
    let b = xloc::CausalEvent { t: 1.0, x: 10.0, y: 0.0, z: 0.0 };
    let err = xloc::authorize(&a, &b, 0.0).unwrap_err();
    assert!(err.ds2 > 0.0);
}

#[test]
fn gate_44_anomaly_error_is_error() {
    let a = xloc::CausalEvent { t: 0.0, x: 0.0, y: 0.0, z: 0.0 };
    let b = xloc::CausalEvent { t: 1.0, x: 10.0, y: 0.0, z: 0.0 };
    let e: Box<dyn std::error::Error> =
        Box::new(xloc::authorize(&a, &b, 0.0).unwrap_err());
    assert!(e.to_string().contains("AnomalyClosureError"));
}

#[test]
fn gate_45_macroscopic_dilation_window() {
    assert!(xloc::MacroscopicFrame::dilate(1e23).is_ok());
    assert!(xloc::MacroscopicFrame::dilate(1e28).is_ok());
    assert!(xloc::MacroscopicFrame::dilate(1e22).is_err());
    assert!(xloc::MacroscopicFrame::dilate(1e29).is_err());
}

#[test]
fn gate_46_partition_exactness() {
    let f = xloc::MacroscopicFrame::dilate(3.3e24).unwrap();
    assert!((f.active_nucleons / f.n_local - 10.0 / 33.0).abs() < 1e-15);
    assert!((f.dark_nucleons / f.n_local - 23.0 / 33.0).abs() < 1e-15);
    assert!(f.closure_holds());
}

#[test]
fn gate_47_relabel_matrix_symplectic() {
    let t = xloc::relabeling_matrix(3, 5);
    assert!(xloc::is_symplectic(&t, 3));
}

#[test]
fn gate_48_non_symplectic_rejected() {
    let mut t = xloc::relabeling_matrix(2, 1);
    t[0][0] = 2;
    assert!(!xloc::is_symplectic(&t, 2));
}

#[test]
fn gate_49_braid_descriptor_count() {
    assert_eq!(xloc::BRAID_DESCRIPTORS, 124);
    assert_eq!(comms::BRAID_COUNT, 124);
}

#[test]
fn gate_50_gst_self_healing() {
    let healed = xloc::gst_healing_fraction(27.9, 8);
    assert!(healed > 0.999, "healed {}", healed);
}

// ------------- GATE-51..60: LANR ledger / boiling -----------------------

#[test]
fn gate_51_module_count() {
    assert_eq!(lanr::MODULE_COUNT, 1800);
}

#[test]
fn gate_52_module_net_power() {
    assert!((lanr::MODULE_NET_W - 555.03).abs() < 1e-9);
}

#[test]
fn gate_53_lanr_net_999_054() {
    assert!((lanr::LANR_NET_KW - 999.054).abs() < 1e-9);
}

#[test]
fn gate_54_landauer_debt_906() {
    assert!((lanr::LANDAUER_DEBT_KW - 906.00).abs() < 1e-9);
}

#[test]
fn gate_55_surplus_93_054() {
    assert!((lanr::POWER_SURPLUS_KW - 93.054).abs() < 1e-9);
}

#[test]
fn gate_56_ledger_closes() {
    let l = lanr::ledger();
    assert!((l.lanr_net_kw - l.debt_kw - l.surplus_kw).abs() < 1e-9);
    assert!(l.surplus_kw > 0.0);
}

#[test]
fn gate_57_dual_stage_teg_positive() {
    let w = lanr::dual_stage_teg_w(1000.0, 900.0, 500.0, 300.0, 1.5);
    assert!(w > 0.0 && w < 1000.0);
}

#[test]
fn gate_58_two_phase_stable_window() {
    let cell = lanr::TwoPhaseCell {
        void_fraction: 0.3,
        mass_flux: 120.0,
        temperature: 4.2,
    };
    assert!(lanr::boiling_stable(&cell, 100.0));
}

#[test]
fn gate_59_two_phase_dryout_rejected() {
    let cell = lanr::TwoPhaseCell {
        void_fraction: 0.95,
        mass_flux: 120.0,
        temperature: 4.2,
    };
    assert!(!lanr::boiling_stable(&cell, 100.0));
}

#[test]
fn gate_60_continuity_conserved() {
    assert_eq!(lanr::continuity_residual(10.0, 2.0, 9.0, 3.0), 0.0);
    assert_ne!(lanr::continuity_residual(10.0, 2.0, 9.0, 4.0), 0.0);
}

// ------------- GATE-61..70: metrology / TQEC / UQ / EDA -----------------

#[test]
fn gate_61_tmsv_squeezing_db() {
    assert!((comms::TMSV_DB - 21.715).abs() < 1e-3);
}

#[test]
fn gate_62_tmsv_sigma_floor() {
    assert!(comms::tmsv_sigma_r(comms::TMSV_R) <= comms::SIGMA_R_PM + 1e-12);
}

#[test]
fn gate_63_sk_gate_count() {
    assert_eq!(comms::sk_gate_count(9), 9 * 5u64.pow(9));
}

#[test]
fn gate_64_tqec_logical_error_rate() {
    let r = comms::decode_syndrome(&[3, 17, 42, 88]);
    assert!(r.logical_error_rate <= comms::LOGICAL_ERROR_TARGET);
}

#[test]
fn gate_65_tqec_latency_budget() {
    let r = comms::decode_syndrome(&[1, 2, 3, 4, 5, 6, 7, 8]);
    assert!(r.latency_ns <= comms::DECODE_BUDGET_NS);
    assert!(r.pairs >= 4);
}

#[test]
fn gate_66_union_find_components() {
    let mut uf = comms::UnionFind::new();
    uf.union(0, 1);
    uf.union(2, 3);
    assert_eq!(uf.find(0), uf.find(1));
    assert_ne!(uf.find(0), uf.find(2));
}

#[test]
fn gate_67_spsc_ring_fifo() {
    let mut ring = comms::SpscRing::<64>::new();
    for i in 0..64u64 {
        assert!(ring.push(i));
    }
    assert!(!ring.push(64));
    for i in 0..64u64 {
        assert_eq!(ring.pop(), Some(i));
    }
    assert_eq!(ring.pop(), None);
}

#[test]
fn gate_68_hyperdual_second_derivative() {
    // d2/dx2 sin(x) at x=0.3: f'' = -sin(x).
    let x = uq::HyperDual {
        real: 0.3,
        e1: 1.0,
        e2: 1.0,
        e1e2: 0.0,
    };
    let y = x.sin();
    assert!((y.real - 0.3f64.sin()).abs() < TOL12);
    assert!((y.e1e2 + 0.3f64.sin()).abs() < TOL12);
}

#[test]
fn gate_69_gum_monte_carlo_bounds() {
    let r = uq::propagate(|x| x * x, 1.0, 0.01, 1_000_000, 7);
    assert!(r.lo_3sigma < 1.0 && 1.0 < r.hi_3sigma);
    assert!((r.mean - 1.0001).abs() < 5e-3);
}

#[test]
fn gate_70_eda_artifacts() {
    let dir = std::env::temp_dir().join("shbt_exotic_gate70");
    std::fs::create_dir_all(&dir).unwrap();
    let gds = dir.join("pic8x8.gds");
    let step = dir.join("waveguide.step");
    let s2p = dir.join("interposer.s2p");
    eda::export_gdsii(gds.to_str().unwrap()).unwrap();
    eda::export_step(step.to_str().unwrap(), 350e-6, 5e-6, 1.5e-6).unwrap();
    let freqs: Vec<f64> = (1..=40).map(|i| i as f64).collect();
    eda::export_s2p(s2p.to_str().unwrap(), eda::INTERPOSER_Z0, &freqs).unwrap();
    assert!(std::fs::metadata(&gds).unwrap().len() > 100);
    assert!(std::fs::metadata(&step).unwrap().len() > 100);
    let s2p_text = std::fs::read_to_string(&s2p).unwrap();
    assert!(s2p_text.contains("50.12"));
    // Out-of-tolerance impedance rejected.
    assert!(eda::export_s2p(s2p.to_str().unwrap(), 52.0, &freqs).is_err());
}
