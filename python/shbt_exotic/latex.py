"""Generate exotic_results.tex macros from live simulator output."""

import math
import os
from pathlib import Path

from shbt_exotic import (
    EntropicRefrigerator,
    ExoticEngine,
    GhostSeedSynthesizer,
    HardwareSynthesisAuditor,
    HeegaardFloerRelabeling,
    HilSafetyMonitor,
    NewtonLockStasis,
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

    bias = 1.0e-15
    gamma_stasis = stasis.gamma_stasis(bias)
    c_get = stasis.local_c_get(bias)

    alpha = 1.67e-51
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
    shunt_latency_ns = hil.emergency_shunt_latency_ns()
    phase_jitter = hil.phase_jitter_threshold_rad()
    baseline_temp = hil.baseline_temperature_k()

    hw_status = hw.audit(72.0e9, 40.0e9)
    f_max = hw.f_max_hz()
    bandwidth = hw.routing_bandwidth_bps()

    lines = [
        "% Auto-generated macros from shbt-exotic unified audit.",
        f"\\newcommand{{\\ExoticKernel}}{{{engine.kernel[0]}, {engine.kernel[1]}, {engine.kernel[2]}}}",
        f"\\newcommand{{\\ExoticStinespringIsometric}}{{{str(stinespring_isometric).lower()}}}",
        f"\\newcommand{{\\ExoticStasisGamma}}{{{format_scientific(gamma_stasis)}}}",
        f"\\newcommand{{\\ExoticCget}}{{{format_scientific(c_get)}}}",
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
        f"\\newcommand{{\\ExoticShuntLatencyNs}}{{{format_scientific(shunt_latency_ns)}}}",
        f"\\newcommand{{\\ExoticPhaseJitterRad}}{{{format_scientific(phase_jitter)}}}",
        f"\\newcommand{{\\ExoticBaselineTempK}}{{{format_scientific(baseline_temp)}}}",
        f"\\newcommand{{\\ExoticHilStatus}}{{\\texttt{{{escape_underscores(status)}}}}}",
        f"\\newcommand{{\\ExoticFmaxHz}}{{{format_scientific(f_max)}}}",
        f"\\newcommand{{\\ExoticRoutingBandwidthBps}}{{{format_scientific(bandwidth)}}}",
        f"\\newcommand{{\\ExoticHardwareStatus}}{{\\texttt{{{escape_underscores(hw_status)}}}}}",
    ]

    out_path.write_text("\n".join(lines) + "\n")
    return out_path


def main() -> int:
    generate_results_tex()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
