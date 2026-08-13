"""Generate exotic_results.tex macros from live simulator output."""

import math
import os
from pathlib import Path

from shbt_exotic import (
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
    UnifiedStinespringMap,
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
    ]

    out_path.write_text("\n".join(lines) + "\n")
    return out_path


def main() -> int:
    generate_results_tex()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
