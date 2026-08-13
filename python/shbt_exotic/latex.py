"""Generate exotic_results.tex macros from live simulator output."""

import math
import os
from pathlib import Path

from shbt_exotic import (
    CoordinatePerturbationSweep,
    EntropicRefrigerator,
    ExoticEngine,
    ExportPhaseModulationTable,
    FibonacciBraidCompiler,
    GhostSeedSynthesizer,
    HardwareSynthesisAuditor,
    HarmonicAuditor,
    HeegaardFloerRelabeling,
    HeegaardMappingTorus,
    HilSafetyMonitor,
    LabHAL,
    LindbladSolver,
    MassCongestionEngine,
    NewtonLockStasis,
    SafetyMonitor,
    TelemetryBridge,
    ThermalFluxReport,
    ThermalHILMonitor,
    UnifiedStinespringMap,
    EngineeringStressSuite,
    CadPhysicsValidator,
)


def _state(dim: int = 8):
    return [(1.0 / math.sqrt(dim), 0.0) for _ in range(dim)]


def format_scientific(value: float, precision: int = 4) -> str:
    """Return a LaTeX-formatted scientific string."""
    if value == 0.0:
        return "0.0"
    if abs(value) >= 1e4 or abs(value) < 1e-3:
        return f"{value:.{precision}e}"
    return f"{value:.{precision}f}"


def escape_underscores(value: str) -> str:
    return value.replace("_", "\\_")


def generate_results_tex(out_path: str | Path = "exotic_results.tex") -> Path:
    """Run the unified simulator and emit a .tex macro file."""
    out_path = Path(out_path)

    engine = ExoticEngine()
    stinespring = UnifiedStinespringMap()
    relabel = HeegaardFloerRelabeling()
    stasis = NewtonLockStasis()
    ghost = GhostSeedSynthesizer()
    fridge = EntropicRefrigerator()
    hil = HilSafetyMonitor()
    hw = HardwareSynthesisAuditor()

    state = _state()

    # Core operators
    _, _, stinespring_isometric, _, _ = stinespring.audit(state)
    relabel.audit(state, 0, 1)
    n_local_partition, n_active_partition, n_dark_partition, eta_a, eta_d = stinespring.partition()

    # Heegaard mapping torus / Kojima inequality
    torus = HeegaardMappingTorus()
    relabeled = relabel.relabel(state, 0, 1)
    ell_he, delta_s, kojima_ok = torus.evaluate(state, relabeled)

    bias = 1.0e-15
    gamma_stasis = stasis.gamma_stasis(bias)
    c_get = stasis.local_c_get(bias)

    alpha = ghost.alpha_seed()
    n_limit = 1.0e65
    n_local = n_limit + 1.0 / alpha
    m_sun = ghost.seed_mass_solar(n_local, n_limit)
    m_kg = ghost.seed_mass_kg(n_local, n_limit)
    entropy_debt = ghost.entropy_debt_power_w(n_local, n_limit)
    high_energy_transient = ghost.high_energy_transient_w()
    mu_local = ghost.local_mu_perturbation(n_local, n_limit, 1.0)

    t_c = 10.0e-3
    gamma_de = fridge.de_rendering_rate(14.2e-6, t_c)
    p_cool = fridge.cooling_power(gamma_de, t_c)
    sub_kelvin_power = fridge.sub_kelvin_cooling_power_w()
    macro_cooling_power = fridge.macro_cooling_power_w()

    status = hil.audit(mu_local, n_local, n_limit, c_get)
    framing_defect = hil.framing_defect(mu_local, n_local, n_limit, c_get)
    canonical_framing_defect = hil.framing_defect(1.0, n_limit, n_limit, 5.34e-175)
    phase_jitter = hil.phase_jitter_threshold_rad()
    baseline_temp = hil.baseline_temperature_k()

    hw_status = hw.audit(72.0e9, 40.0e9)
    f_max = hw.f_max_hz()
    bandwidth = hw.routing_bandwidth_bps()

    # Gate-cycle shunt safety and Debye T^3 thermal HIL audit
    monitor = SafetyMonitor()
    shunt_status, shunt_latency_ns, shunt_cycles, thermal_status = monitor.simulate_shutdown(
        mu_local, n_local, n_limit, c_get
    )
    hil_thermal = monitor.hil_thermal_monitor()
    thermal = monitor.thermal_shunt_auditor()
    temperature_rise_k = thermal.temperature_rise_k()
    debye_final_temp_k = hil_thermal.final_temperature_k()
    dissipation_volume_cm3 = hil_thermal.volume_cm3()
    debye_heat_capacity_j_per_k = hil_thermal.heat_capacity_j_per_k()

    sweep = CoordinatePerturbationSweep(mu0=1.0, n_limit=n_limit, c_get_bound=c_get)
    sweep_ok, worst_detuning = sweep.verify_rigidity_limit()
    rigidity_status = "STATUS_NOMINAL_PASS" if sweep_ok else "STATUS_EMERGENCY_SHUTDOWN"
    zone = sweep.safety_zone_grid()
    safety_zone_total = zone["total"]
    safety_zone_nominal = zone["nominal"]

    # Multi-seed interference and metric superposition
    mass_engine = MassCongestionEngine()
    congestion_radius_m = mass_engine.bit_congestion_radius_m()
    i00, i11, i22, i33 = mass_engine.interference_coefficients()
    g_metric = mass_engine.linearized_metric_with_interference([(n_local, n_limit)])
    g00 = g_metric[0][0]
    g11 = g_metric[1][1]
    multi_seed_overlap_status, _, overlap_triggered = sweep.sweep_multi_seed_overlap([6e52, 6e52])

    # RF phase-modulation table (8x8 SHBT array)
    phase_exporter = ExportPhaseModulationTable()
    h_phase = [[(i + 1.0) / (j + 2.0) for j in range(8)] for i in range(8)]
    v_eff = 0.5
    phase_table = phase_exporter.table_entries(h_phase, v_eff)
    phase_table_min_v = min(cmd.v_phase for cmd in phase_table)
    phase_table_max_v = max(cmd.v_phase for cmd in phase_table)

    # Thermal flux report for the 14.2 μW refrigeration core
    flux = ThermalFluxReport()
    flux_gamma_de = flux.gamma_de
    flux_cooling_power = flux.cooling_power_w
    flux_heat_flux = flux.heat_flux_w_per_m2
    kapitza_unengineered = flux.kapitza_delta_t_unengineered_k
    kapitza_matched = flux.kapitza_delta_t_matched_k
    kapitza_justified = flux.acoustic_matching_justified

    # Fibonacci anyon braid compiler
    braid = FibonacciBraidCompiler()
    braid_depth = 9
    braid_gate_count = braid.gate_count(braid_depth)
    braid_approx_error = braid.approximation_error(braid_depth)

    # Dynamic interference and wake compensation
    n_total = mass_engine.n_total()
    wake1, wake2, wake3 = mass_engine.wake_constants_f64()
    v_eff = 1.0e3  # representative slow transit
    delta_n = 1.0e5
    mu0 = 1.0
    mu_compensated = mass_engine.compensated_mu(mu0, delta_n, n_total, v_eff)
    g_metric = mass_engine.linearized_metric_with_interference([(n_local, n_limit)])
    u_4 = [1.0, 0.0, 0.0, 0.0]
    l_int = mass_engine.dynamic_interference_lagrangian(g_metric, u_4, delta_n, 0.0, mu0)

    # Lindblad master-equation solver / SK noise floor
    lindblad = LindbladSolver()
    gamma_charge = 8.42e-5
    gamma_phonon = 3.58e-5
    gamma_dec = lindblad.combined_decoherence_rate_hz()
    sk_logical_error = lindblad.sk_logical_error_default()
    sk_logical_error_log10 = math.log10(sk_logical_error) if sk_logical_error > 0 else -999.0

    # Structural resonance and harmonic audit
    harmonic = HarmonicAuditor()
    f_shear = harmonic.nominal_frequency_hz("shear")
    f_long = harmonic.nominal_frequency_hz("longitudinal")
    f_tors = harmonic.nominal_frequency_hz("torsional")
    f_flex = harmonic.nominal_frequency_hz("flexural")
    min_loss_factor = 1.15e-3
    min_damping_ratio = 6.0e-4
    support_volume_m3 = 1.0e-6
    modes_nominal = [
        ("shear", f_shear, min_loss_factor, min_damping_ratio, 1.0e-9),
        ("longitudinal", f_long, min_loss_factor, min_damping_ratio, 1.0e-9),
        ("torsional", f_tors, min_loss_factor, min_damping_ratio, 1.0e-9),
        ("flexural", f_flex, min_loss_factor, min_damping_ratio, 1.0e-9),
    ]
    harmonic_status = harmonic.audit_waveguide(modes_nominal, support_volume_m3)

    # Laboratory HAL telemetry latency
    telemetry = TelemetryBridge()
    telemetry_cycle_ns = telemetry.telemetry_cycle_ns()

    # Integrated engineering stress suite
    suite = EngineeringStressSuite()
    stress = suite.run_all()
    cad_validator = CadPhysicsValidator()
    cad_flex_hz = cad_validator.validate_airbridge_um(5.0, 1.5, 0.3)

    # 512-bit closure-chain audit
    i_l_star = hil.i_l_star()
    i_q_star = hil.i_q_star()
    kb = 1.380_649e-23
    entropy_arrow = gamma_de * kb * math.log(2) + p_cool / t_c
    closure_chain_holds = framing_defect == 0.0
    entropy_arrow_positive = entropy_arrow > 0.0

    lines = [
        "% Auto-generated macros from shbt-exotic unified audit.",
        f"\\newcommand{{\\ExoticKernel}}{{{engine.kernel[0]}, {engine.kernel[1]}, {engine.kernel[2]}}}",
        f"\\newcommand{{\\ExoticStinespringIsometric}}{{{str(stinespring_isometric).lower()}}}",
        f"\\newcommand{{\\ExoticNLocal}}{{{n_local_partition}}}",
        f"\\newcommand{{\\ExoticNActive}}{{{n_active_partition}}}",
        f"\\newcommand{{\\ExoticNDark}}{{{n_dark_partition}}}",
        f"\\newcommand{{\\ExoticEtaA}}{{{eta_a[0]}/{eta_a[1]}}}",
        f"\\newcommand{{\\ExoticEtaD}}{{{eta_d[0]}/{eta_d[1]}}}",
        f"\\newcommand{{\\ExoticHeegaardPresLength}}{{{format_scientific(ell_he)}}}",
        f"\\newcommand{{\\ExoticHeegaardEntropyChange}}{{{format_scientific(delta_s)}}}",
        f"\\newcommand{{\\ExoticKojimaSatisfied}}{{{str(kojima_ok).lower()}}}",
        f"\\newcommand{{\\ExoticStasisScale}}{{{format_scientific(1.0e-12)}}}",
        f"\\newcommand{{\\ExoticStasisGamma}}{{{format_scientific(gamma_stasis)}}}",
        f"\\newcommand{{\\ExoticCget}}{{{format_scientific(c_get)}}}",
        f"\\newcommand{{\\ExoticAlphaSeed}}{{{format_scientific(alpha)}}}",
        f"\\newcommand{{\\ExoticGhostMassSun}}{{{format_scientific(m_sun)}}}",
        f"\\newcommand{{\\ExoticGhostMassKg}}{{{format_scientific(m_kg)}}}",
        f"\\newcommand{{\\ExoticEntropyDebtPower}}{{{format_scientific(entropy_debt)}}}",
        f"\\newcommand{{\\ExoticHighEnergyTransient}}{{{format_scientific(high_energy_transient)}}}",
        f"\\newcommand{{\\ExoticMuLocal}}{{{format_scientific(mu_local)}}}",
        f"\\newcommand{{\\ExoticGammaDe}}{{{format_scientific(gamma_de)}}}",
        f"\\newcommand{{\\ExoticCoolingPower}}{{{format_scientific(p_cool)}}}",
        f"\\newcommand{{\\ExoticSubKelvinPower}}{{{format_scientific(sub_kelvin_power)}}}",
        f"\\newcommand{{\\ExoticMacroCoolingPower}}{{{format_scientific(macro_cooling_power)}}}",
        f"\\newcommand{{\\ExoticFramingDefect}}{{{format_scientific(framing_defect)}}}",
        f"\\newcommand{{\\ExoticCanonicalFramingDefect}}{{{format_scientific(canonical_framing_defect)}}}",
        f"\\newcommand{{\\ExoticPhaseJitterRad}}{{{format_scientific(phase_jitter)}}}",
        f"\\newcommand{{\\ExoticBaselineTempK}}{{{format_scientific(baseline_temp)}}}",
        f"\\newcommand{{\\ExoticHilStatus}}{{\\texttt{{{escape_underscores(status)}}}}}",
        f"\\newcommand{{\\ExoticFmaxHz}}{{{format_scientific(f_max)}}}",
        f"\\newcommand{{\\ExoticRoutingBandwidthBps}}{{{format_scientific(bandwidth)}}}",
        f"\\newcommand{{\\ExoticHardwareStatus}}{{\\texttt{{{escape_underscores(hw_status)}}}}}",
        f"\\newcommand{{\\ExoticShuntStatus}}{{\\texttt{{{escape_underscores(shunt_status)}}}}}",
        f"\\newcommand{{\\ExoticShuntLatencyNs}}{{{format_scientific(shunt_latency_ns)}}}",
        f"\\newcommand{{\\ExoticShuntCycles}}{{{shunt_cycles}}}",
        f"\\newcommand{{\\ExoticThermalStatus}}{{\\texttt{{{escape_underscores(thermal_status)}}}}}",
        f"\\newcommand{{\\ExoticTemperatureRiseK}}{{{format_scientific(temperature_rise_k)}}}",
        f"\\newcommand{{\\ExoticDebyeFinalTempK}}{{{format_scientific(debye_final_temp_k)}}}",
        f"\\newcommand{{\\ExoticDissipationVolume}}{{{format_scientific(dissipation_volume_cm3)}}}",
        f"\\newcommand{{\\ExoticMinDissipationVolume}}{{{format_scientific(48.98)}}}",
        f"\\newcommand{{\\ExoticDebyeHeatCapacityJK}}{{{format_scientific(debye_heat_capacity_j_per_k)}}}",
        f"\\newcommand{{\\ExoticRigiditySweepStatus}}{{\\texttt{{{escape_underscores(rigidity_status)}}}}}",
        f"\\newcommand{{\\ExoticWorstDetuning}}{{{format_scientific(worst_detuning)}}}",
        f"\\newcommand{{\\ExoticILStar}}{{{format_scientific(i_l_star)}}}",
        f"\\newcommand{{\\ExoticIQStar}}{{{format_scientific(i_q_star)}}}",
        f"\\newcommand{{\\ExoticEntropyArrow}}{{{format_scientific(entropy_arrow)}}}",
        f"\\newcommand{{\\ExoticClosureChainHolds}}{{{str(closure_chain_holds).lower()}}}",
        f"\\newcommand{{\\ExoticEntropyArrowPositive}}{{{str(entropy_arrow_positive).lower()}}}",
        f"\\newcommand{{\\ExoticPhaseTableVOn}}{{3.8}}",
        f"\\newcommand{{\\ExoticPhaseTableVcc}}{{7.4}}",
        f"\\newcommand{{\\ExoticPhaseTableEntries}}{{64}}",
        f"\\newcommand{{\\ExoticPhaseTableMinVPhase}}{{{format_scientific(phase_table_min_v)}}}",
        f"\\newcommand{{\\ExoticPhaseTableMaxVPhase}}{{{format_scientific(phase_table_max_v)}}}",
        f"\\newcommand{{\\ExoticFluxGammaDe}}{{{format_scientific(flux_gamma_de)}}}",
        f"\\newcommand{{\\ExoticFluxCoolingPower}}{{{format_scientific(flux_cooling_power)}}}",
        f"\\newcommand{{\\ExoticFluxHeatFlux}}{{{format_scientific(flux_heat_flux)}}}",
        f"\\newcommand{{\\ExoticKapitzaDropUnengineered}}{{{format_scientific(kapitza_unengineered)}}}",
        f"\\newcommand{{\\ExoticKapitzaDropMatched}}{{{format_scientific(kapitza_matched)}}}",
        f"\\newcommand{{\\ExoticKapitzaJustified}}{{{str(kapitza_justified).lower()}}}",
        f"\\newcommand{{\\ExoticSafetyZoneTotal}}{{{safety_zone_total}}}",
        f"\\newcommand{{\\ExoticSafetyZoneNominal}}{{{safety_zone_nominal}}}",
        f"\\newcommand{{\\ExoticBitCongestionRadius}}{{{format_scientific(congestion_radius_m)}}}",
        f"\\newcommand{{\\ExoticInterferenceZeroZero}}{{{i00}}}",
        f"\\newcommand{{\\ExoticInterferenceOneOne}}{{{i11}}}",
        f"\\newcommand{{\\ExoticInterferenceTwoTwo}}{{{i22}}}",
        f"\\newcommand{{\\ExoticInterferenceThreeThree}}{{{i33}}}",
        f"\\newcommand{{\\ExoticMultiSeedOverlapTriggered}}{{{str(overlap_triggered).lower()}}}",
        f"\\newcommand{{\\ExoticBraidDepth}}{{{braid_depth}}}",
        f"\\newcommand{{\\ExoticBraidGateCount}}{{{braid_gate_count}}}",
        f"\\newcommand{{\\ExoticBraidApproxError}}{{{format_scientific(braid_approx_error)}}}",
        # Dynamic interference
        f"\\newcommand{{\\ExoticWakeOne}}{{{format_scientific(wake1)}}}",
        f"\\newcommand{{\\ExoticWakeTwo}}{{{format_scientific(wake2)}}}",
        f"\\newcommand{{\\ExoticWakeThree}}{{{format_scientific(wake3)}}}",
        f"\\newcommand{{\\ExoticNTotal}}{{{format_scientific(n_total)}}}",
        f"\\newcommand{{\\ExoticMuCompensated}}{{{format_scientific(mu_compensated)}}}",
        f"\\newcommand{{\\ExoticDynamicLagrangian}}{{{format_scientific(l_int)}}}",
        # Lindblad / SK
        f"\\newcommand{{\\ExoticLindbladGammaCharge}}{{{format_scientific(gamma_charge)}}}",
        f"\\newcommand{{\\ExoticLindbladGammaPhonon}}{{{format_scientific(gamma_phonon)}}}",
        f"\\newcommand{{\\ExoticLindbladGammaDec}}{{{format_scientific(gamma_dec)}}}",
        f"\\newcommand{{\\ExoticSKLogicalError}}{{{format_scientific(sk_logical_error)}}}",
        f"\\newcommand{{\\ExoticSKLogicalErrorLogTen}}{{{format_scientific(-sk_logical_error_log10)}}}",
        # Harmonic audit
        f"\\newcommand{{\\ExoticShearModeHz}}{{{format_scientific(f_shear)}}}",
        f"\\newcommand{{\\ExoticLongitudinalModeHz}}{{{format_scientific(f_long)}}}",
        f"\\newcommand{{\\ExoticTorsionalModeHz}}{{{format_scientific(f_tors)}}}",
        f"\\newcommand{{\\ExoticFlexuralModeHz}}{{{format_scientific(f_flex)}}}",
        f"\\newcommand{{\\ExoticMinLossFactor}}{{{format_scientific(min_loss_factor)}}}",
        f"\\newcommand{{\\ExoticMinDampingRatio}}{{{format_scientific(min_damping_ratio)}}}",
        f"\\newcommand{{\\ExoticHarmonicStatus}}{{\\texttt{{{escape_underscores(harmonic_status)}}}}}",
        # HAL telemetry
        f"\\newcommand{{\\ExoticTelemetryCycleNs}}{{{format_scientific(telemetry_cycle_ns)}}}",
        # Engineering stress suite
        f"\\newcommand{{\\ExoticScenarioAStatus}}{{\\texttt{{{escape_underscores(stress.scenario_a_status)}}}}}",
        f"\\newcommand{{\\ExoticScenarioBStatus}}{{\\texttt{{{escape_underscores(stress.scenario_b_status)}}}}}",
        f"\\newcommand{{\\ExoticScenarioCStatus}}{{\\texttt{{{escape_underscores(stress.scenario_c_status)}}}}}",
        f"\\newcommand{{\\ExoticScenarioDStatus}}{{\\texttt{{{escape_underscores(stress.scenario_d_status)}}}}}",
        f"\\newcommand{{\\ExoticStressAllPass}}{{{str(stress.all_pass).lower()}}}",
        f"\\newcommand{{\\ExoticStressFinalTempK}}{{{format_scientific(stress.final_substrate_temp_k)}}}",
        f"\\newcommand{{\\ExoticStressSKError}}{{{format_scientific(stress.sk_logical_error)}}}",
        f"\\newcommand{{\\ExoticStressConsumedBits}}{{{format_scientific(stress.consumed_lifetime_bits)}}}",
        f"\\newcommand{{\\ExoticStressShiftedImpedance}}{{{format_scientific(stress.shifted_impedance_mrayl)}}}",
        f"\\newcommand{{\\ExoticCadAirbridgeFlexHz}}{{{format_scientific(cad_flex_hz)}}}",
    ]

    out_path.write_text("\n".join(lines) + "\n")
    return out_path


def main() -> int:
    generate_results_tex()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
