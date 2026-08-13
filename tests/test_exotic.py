import math

import pytest

from shbt_exotic import (
    MassCongestionEngine,
    FibonacciBraidCompiler,
    CalibrationEngine,
    ReliabilityAuditor,
    AnomalyClosureError,
    AcousticImpedanceEngine,
    CoordinatePerturbationSweep,
    EntropicRefrigerator,
    ExoticEngine,
    ExportPhaseModulationTable,
    GhostSeedSynthesizer,
    HardwareSynthesisAuditor,
    HeegaardFloerRelabeling,
    HeegaardMappingTorus,
    HilSafetyMonitor,
    NewtonLockStasis,
    SafetyMonitor,
    ThermalFluxReport,
    ThermalHILMonitor,
    ThermalShuntAuditor,
    UnifiedStinespringMap,
)


def _state(dim: int = 8):
    return [(1.0 / math.sqrt(dim), 0.0) for _ in range(dim)]


def test_exotic_engine_kernel():
    engine = ExoticEngine()
    assert engine.kernel == (26, 8, 312)


def test_stinespring_isometric():
    st = UnifiedStinespringMap()
    state = _state()
    active, dark = st.de_render(state)
    assert len(active) == len(dark) == 8
    _, _, iso, _, _ = st.audit(state)
    assert iso is True


def test_stinespring_partition_from_branch():
    st = UnifiedStinespringMap()
    n_local, n_active, n_dark, eta_a, eta_d = st.partition()
    assert n_local == 33
    assert n_active == 10
    assert n_dark == 23
    assert eta_a == (10, 33)
    assert eta_d == (23, 33)


def test_alpha_seed_is_topological_residue():
    ghost = GhostSeedSynthesizer()
    alpha = ghost.alpha_seed()
    assert 1.3e-51 < alpha < 1.4e-51


def test_heegaard_floer_relabel():
    relabel = HeegaardFloerRelabeling()
    state = _state()
    target = relabel.relabel(state, 0, 1)
    assert len(target) == 8
    assert relabel.audit(state, 0, 1) is True


def test_heegaard_mapping_torus_kojima():
    torus = HeegaardMappingTorus()
    state = _state()
    target = state.copy()
    ell, delta_s, ok = torus.evaluate(state, target)
    assert ell == pytest.approx(1.0, rel=1e-15)
    assert abs(delta_s) < 1e-15
    assert ok is True

    perturbed = [(v[0] + 1e-3, v[1]) for v in state]
    _, delta_s_bad, ok_bad = torus.evaluate(state, perturbed)
    assert ok_bad is False


def test_newton_lock_stasis():
    stasis = NewtonLockStasis()
    gamma0 = stasis.gamma_stasis(0.0)
    gamma1 = stasis.gamma_stasis(1.0e-15)
    assert gamma1 > gamma0
    # exp(1e-15 / 1e-12) ≈ 1.001
    assert gamma1 == pytest.approx(1.0010005, abs=1e-6)


def test_newton_lock_anomaly_at_modular_detuning_limit():
    stasis = NewtonLockStasis()
    with pytest.raises(Exception):
        stasis.local_c_get(1.0e-12)
    with pytest.raises(Exception):
        stasis.gamma_stasis(1.0e-12)


def test_ghost_seed_one_solar_mass_and_entropy_debt():
    ghost = GhostSeedSynthesizer()
    alpha = ghost.alpha_seed()
    n_limit = 1.0e65
    n_local = n_limit + 1.0 / alpha
    m_sun = ghost.seed_mass_solar(n_local, n_limit)
    m_kg = ghost.seed_mass_kg(n_local, n_limit)
    p_debt = ghost.entropy_debt_power_w(n_local, n_limit)
    assert abs(m_sun - 1.0) < 1.0e-2
    assert abs(m_kg - 1.98847e30) / 1.98847e30 < 1.0e-2
    assert abs(p_debt - 906.0e9) / 906.0e9 < 1.0e-2
    assert ghost.is_filling_factor_allowed((12, 5))


def test_entropic_refrigeration():
    fridge = EntropicRefrigerator()
    t_c = 10.0e-3
    gamma_de = fridge.de_rendering_rate(14.2e-6, t_c)
    assert gamma_de > 0.0
    p = fridge.cooling_power(gamma_de, t_c)
    assert abs(p - 14.2e-6) < 1.0e-10


def test_hil_nominal_and_framing_defect():
    hil = HilSafetyMonitor()
    alpha = 1.67e-51
    n_limit = 1.0e65
    n_local = n_limit + 1.0 / alpha
    mu_local = 1.0 + (n_local - n_limit) / n_limit
    c_get = 5.34e-175
    status = hil.audit(mu_local, n_local, n_limit, c_get)
    assert status == "STATUS_NOMINAL_PASS"
    assert hil.is_nominal(status) is True
    assert (
        hil.framing_defect(1.0, 1.0e65, 1.0e65, 5.34e-175) == 0.0
    )


def test_hil_rigidity_shutdown():
    hil = HilSafetyMonitor()
    status = hil.audit(1.0 + 2.0e-12, 1.0e60, 1.0e60, 5.34e-175)
    assert status == "STATUS_EMERGENCY_SHUTDOWN"


def test_hil_correction_and_shunt_latency():
    hil = HilSafetyMonitor()
    assert hil.emergency_shunt_latency_ns() < 2.5
    residual = hil.apply_correction(7.0e-13)
    assert residual < 0.5e-12


def test_hardware_invariants():
    hw = HardwareSynthesisAuditor()
    assert hw.clock_rate_passes(72.0e9) is True
    assert hw.routing_bandwidth_passes(40.0e9) is True
    assert hw.audit(72.0e9, 40.0e9) == "STATUS_NOMINAL_PASS"
    assert hw.clock_rate_passes(80.0e9) is False
    assert hw.routing_bandwidth_passes(50.0e9) is False


def test_gate_cycle_shunt_emergency_shutdown():
    monitor = SafetyMonitor()
    status, latency_ns, cycles, _thermal = monitor.simulate_shutdown(
        1.0 + 2.0e-12, 1.0e65, 1.0e65, 5.34e-175
    )
    assert status == "STATUS_EMERGENCY_SHUTDOWN"
    assert latency_ns < 2.5
    assert cycles <= 180


def test_thermal_shunt_no_quench():
    auditor = ThermalShuntAuditor()
    assert auditor.audit() == "STATUS_NOMINAL_PASS"
    assert auditor.temperature_rise_k() < 1.0e-12


def test_coordinate_perturbation_sweep_rigidity():
    sweep = CoordinatePerturbationSweep(mu0=1.0, n_limit=1.0e65, c_get_bound=5.34e-175)
    ok, worst = sweep.verify_rigidity_limit()
    assert ok is True
    assert worst < 1.0e-12


def test_thermal_hil_debye_t3_model():
    monitor = ThermalHILMonitor()
    assert monitor.audit() == "STATUS_NOMINAL_PASS"
    assert monitor.final_temperature_k() < 9.3
    assert monitor.volume_cm3() >= 48.98


def test_thermal_hil_quench_for_small_volume():
    monitor = ThermalHILMonitor(
        a_inp=3.87759483,
        t_i=15.4e-3,
        t_c=9.3,
        power_w=142.08e6,
        tau_s=2.5e-9,
        volume_cm3=1.0,
    )
    assert monitor.audit() == "STATUS_EMERGENCY_SHUTDOWN"
    assert monitor.final_temperature_k() > 9.3


def test_safety_monitor_thermal_hil_shuts_down_on_quench():
    monitor = SafetyMonitor()
    # The default design volume is 50 cm^3, so nominal mu should pass.
    _, _, _, thermal = monitor.simulate_shutdown(1.0, 1.0e65, 1.0e65, 5.34e-175)
    assert thermal == "STATUS_NOMINAL_PASS"


def test_stinespring_branching_matrix_has_33x33_block_structure():
    st = UnifiedStinespringMap()
    b = st.branching_matrix_b()
    assert len(b) == 33
    assert all(len(row) == 33 for row in b)
    # Active diagonal entries: sqrt(10/33)
    active_val = (10 / 33) ** 0.5
    for i in range(10):
        assert b[i][i] == pytest.approx(active_val, rel=1e-12)
    # Dark diagonal entries: sqrt(23/33)
    dark_val = (23 / 33) ** 0.5
    assert b[10][10] == pytest.approx(dark_val, rel=1e-12)
    for i in range(11, 33):
        assert b[i][i] == pytest.approx(dark_val, rel=1e-12)
    # Three 11x11 diagonal blocks are exposed separately.
    block0 = st.branching_block(0)
    assert len(block0) == 11
    block1 = st.branching_block(1)
    assert len(block1) == 11


def test_export_phase_modulation_table():
    exporter = ExportPhaseModulationTable()
    h = [[(i + 1.0) / (j + 2.0) for j in range(8)] for i in range(8)]
    json_table = exporter.export_json(h, 0.5)
    csv_table = exporter.export_csv(h, 0.5)
    assert '"i"' in json_table
    assert "theta_rad" in csv_table
    entries = exporter.table_entries(h, 0.5)
    assert len(entries) == 64
    assert all(3.8 <= cmd.v_phase <= 7.4 for cmd in entries)


def test_thermal_flux_report():
    flux = ThermalFluxReport()
    gamma = flux.gamma_de
    assert gamma > 0.0
    assert flux.cooling_power_w == pytest.approx(14.2e-6, rel=1e-12)
    assert flux.kapitza_delta_t_unengineered_k == pytest.approx(3.89e14, rel=1e-6)
    assert flux.kapitza_delta_t_matched_k < flux.kapitza_delta_t_unengineered_k
    assert flux.acoustic_matching_justified is True
    assert len(flux.flux_map()) == 64


def test_acoustic_impedance_inp_within_yield():
    acoustic = AcousticImpedanceEngine()
    assert acoustic.peak_waveguide_pressure_gpa() == pytest.approx(12.6427, rel=1e-4)
    assert acoustic.inp_substrate_pressure_gpa() < 10.0
    assert acoustic.is_inp_within_yield() is True


def test_heegaard_mapping_torus_kojima_bounds():
    torus = HeegaardMappingTorus()
    assert torus.kojima_geometric_constant() == pytest.approx(1.0e20, rel=1e-12)
    assert torus.entropy_bound_arithmetic(1.0) == pytest.approx(0.0, abs=1e-12)
    # For a larger presentation length the arithmetic bound grows linearly.
    bound_2 = torus.entropy_bound_arithmetic(3.0)
    assert bound_2 > 0.0
    torus.set_volume(2.0)
    assert torus.kojima_bound() == pytest.approx(2.0e20, rel=1e-12)


def test_coordinate_perturbation_sweep_safety_zone():
    from shbt_exotic.sweep import CoordinatePerturbationSweep
    sweep = CoordinatePerturbationSweep(mu0=1.0, n_limit=1.0e65, c_get_bound=5.34e-175)
    zone = sweep.safety_zone_grid(
        mu_values=[0.0, 1e-15, 1.1e-12],
        n_offsets=[0.0, 1e50],
    )
    assert zone["total"] == 6
    assert zone["nominal"] >= 1


def test_mass_congestion_engine_interference_tensor():
    engine = MassCongestionEngine()
    i = engine.interference_tensor_f64()
    assert len(i) == 4 and all(len(row) == 4 for row in i)
    assert i[0][0] > 0.0
    assert i[1][1] < 0.0
    assert i[2][2] > 0.0
    assert i[3][3] < 0.0


def test_mass_congestion_engine_bit_congestion_radius():
    engine = MassCongestionEngine()
    assert engine.bit_congestion_radius_m() == pytest.approx(2.954e15, rel=1e-12)


def test_mass_congestion_engine_linearized_metric():
    engine = MassCongestionEngine()
    g = engine.linearized_metric_with_interference([(1.0e65 + 4e52, 1.0e65)])
    assert len(g) == 4
    for mu in range(4):
        assert g[mu][mu] != 0.0


def test_multi_seed_overlap_triggers_anomaly_closure_error():
    from shbt_exotic.sweep import CoordinatePerturbationSweep
    sweep = CoordinatePerturbationSweep(mu0=1.0, n_limit=1.0e65, c_get_bound=5.34e-175)
    # Two seeds, each contributing 6e-13, sum to 1.2e-12 > 1e-12.
    status, _, triggered = sweep.sweep_multi_seed_overlap([6e52, 6e52])
    assert triggered is True
    assert "AnomalyClosureError" in status


def test_multi_seed_overlap_passes_below_threshold():
    from shbt_exotic.sweep import CoordinatePerturbationSweep
    sweep = CoordinatePerturbationSweep(mu0=1.0, n_limit=1.0e65, c_get_bound=5.34e-175)
    # Two seeds, each contributing 4e-13, sum to 8e-13 < 1e-12.
    status, total_delta, triggered = sweep.sweep_multi_seed_overlap([4e52, 4e52])
    assert triggered is False
    assert status == "STATUS_NOMINAL_PASS"
    assert total_delta == pytest.approx(8e-13, rel=1e-6)


def test_fibonacci_braid_compiler_beta_sequence():
    compiler = FibonacciBraidCompiler()
    beta = compiler.beta_sequence()
    # Primitive expansion: sigma2^{-2} becomes two sigma2^{-1} factors.
    assert len(beta) == 7
    assert beta[0] == "sigma1^2"
    assert beta[1] == "sigma2^-1"
    assert beta[2] == "sigma2^-1"


def test_fibonacci_braid_sk_nine_gates():
    compiler = FibonacciBraidCompiler()
    assert compiler.gate_count(9) == 124


def test_fibonacci_braid_sk_nine_error():
    compiler = FibonacciBraidCompiler()
    err = compiler.approximation_error(9)
    assert err <= 1.5e-10


def test_fibonacci_braid_openqasm():
    compiler = FibonacciBraidCompiler()
    qasm = compiler.compile_openqasm(9, 0)
    assert qasm.startswith("OPENQASM 2.0")
    assert "u3(" in qasm
    assert qasm.count("u3(") == 124
    assert "sigma1" in qasm
    assert "sigma2" in qasm


def test_fibonacci_braid_target_unitary_weights():
    compiler = FibonacciBraidCompiler()
    u = compiler.target_unitary()
    c = (10.0 / 33.0) ** 0.5
    s = (23.0 / 33.0) ** 0.5
    assert u[0][0] == pytest.approx((c, 0.0), abs=1e-12)
    assert u[0][1] == pytest.approx((-s, 0.0), abs=1e-12)
    assert u[1][0] == pytest.approx((s, 0.0), abs=1e-12)
    assert u[1][1] == pytest.approx((c, 0.0), abs=1e-12)


def test_calibration_waveform_base_and_frequency():
    engine = CalibrationEngine()
    assert engine.calibration_waveform(0.0, 0.0) == pytest.approx(3.3, abs=1e-9)
    period = 1.0 / 10.0e6
    v0 = engine.calibration_waveform(0.0, 0.0)
    v1 = engine.calibration_waveform(period, 0.0)
    assert v0 == pytest.approx(v1, abs=1e-9)


def test_calibration_pid_gains():
    engine = CalibrationEngine()
    assert engine.pid_gains() == (1.85, 9.12e3, 3.45e-7)


def test_calibration_pid_nominal_for_small_jitter():
    engine = CalibrationEngine()
    bias, corrected, status = engine.step(1.0e-6, 4.0e-5)
    assert status == "STATUS_NOMINAL_PASS"
    assert corrected <= 5.05e-5
    assert corrected >= -5.05e-5
    assert bias == pytest.approx(3.3, abs=0.01)


def test_calibration_pid_shutdown_for_excessive_jitter():
    engine = CalibrationEngine()
    _, corrected, status = engine.step(1.0e-6, 10.0)
    assert status == "STATUS_EMERGENCY_SHUTDOWN"
    assert abs(corrected) > 5.05e-5


def test_reliability_lifetime_budget():
    rel = ReliabilityAuditor()
    eps, swing, n_f, lifetime = rel.coffin_manson_constants()
    assert eps == pytest.approx(6.0e-6, abs=1e-12)
    assert swing == pytest.approx(15.0, abs=1e-9)
    assert n_f == pytest.approx(4.0e6, abs=1.0)
    assert lifetime == pytest.approx(1.514e16, rel=1e-9)


def test_reliability_nominal_within_lifetime():
    rel = ReliabilityAuditor()
    rel.accumulate_bits(1.514e16 / 2.0)
    status, nominal, remaining, consumed, impedance = rel.audit()
    assert status == "STATUS_NOMINAL_PASS"
    assert nominal
    assert remaining == pytest.approx(1.514e16 / 2.0, rel=1e-9)
    assert consumed == pytest.approx(2.0e6, rel=1e-6)
    assert impedance == pytest.approx(1.3250, abs=1e-9)


def test_reliability_quench_warning_after_exceeding_lifetime():
    rel = ReliabilityAuditor()
    rel.accumulate_bits(1.514e16 * 1.01)
    status, nominal, remaining, consumed, impedance = rel.audit()
    assert status == "STATUS_QUENCH_WARNING"
    assert not nominal
    assert remaining == 0.0
    assert impedance == pytest.approx(1.3250, abs=1e-9)


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
