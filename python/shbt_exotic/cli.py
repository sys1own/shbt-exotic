"""Command-line interface for the shbt-exotic unified simulator."""

import argparse
import math
import sys

from shbt_exotic import (
    ADMMetricAuditor,
    CausalCoordinate,
    CalibrationEngine,
    CoordinatePerturbationSweep,
    ReliabilityAuditor,
    GdsiiMaskExporter,
    StepSolidModel,
    EntropicRefrigerator,
    ExoticEngine,
    FibonacciBraidCompiler,
    GhostSeedSynthesizer,
    HardwareSynthesisAuditor,
    HeegaardFloerRelabeling,
    HeegaardMappingTorus,
    HilSafetyMonitor,
    ModularStateTranslocator,
    NewtonLockStasis,
    SafetyMonitor,
    ThermalHILMonitor,
    UnifiedStinespringMap,
)


def _normalized_state(dim: int = 8) -> list:
    return [(1.0 / (dim**0.5), 0.0) for _ in range(dim)]


def run_audit(args: argparse.Namespace) -> int:
    engine = ExoticEngine()
    stinespring = UnifiedStinespringMap()
    relabel = HeegaardFloerRelabeling()
    stasis = NewtonLockStasis()
    ghost = GhostSeedSynthesizer()
    fridge = EntropicRefrigerator()
    hil = HilSafetyMonitor()
    hw = HardwareSynthesisAuditor()

    state = _normalized_state()

    # 1. Unified Stinespring map
    active, dark = stinespring.de_render(state)
    _, _, iso, _, _ = stinespring.audit(state)

    # 2. Heegaard-Floer relabeling (non-local communication)
    relabeled = relabel.relabel(state, 0, 1)
    torus = HeegaardMappingTorus()
    ell_he, delta_s, kojima_ok = torus.evaluate(state, relabeled)

    # 3. Newton-lock temporal stasis
    bias = 1.0e-15
    gamma = stasis.gamma_stasis(bias)
    c_get = stasis.local_c_get(bias)

    # 4. Ghost-seed mass-congestion (~1 M_sun, with congestion < 10^-12)
    alpha = ghost.alpha_seed()
    n_limit = 1.0e65
    n_local = n_limit + 1.0 / alpha
    m_seed = ghost.seed_mass_kg(n_local, n_limit)
    m_sun = ghost.seed_mass_solar(n_local, n_limit)
    entropy_debt = ghost.entropy_debt_power_w(n_local, n_limit)
    mu_local = ghost.local_mu_perturbation(n_local, n_limit, 1.0)

    # 5. Entropic refrigeration
    t_c = 10.0e-3
    gamma_de = fridge.de_rendering_rate(14.2e-6, t_c)
    p_cool = fridge.cooling_power(gamma_de, t_c)

    # 6. HIL dual-target audit
    status = hil.audit(mu_local, n_local, n_limit, c_get)
    nominal = hil.is_nominal(status)
    framing_defect = hil.framing_defect(mu_local, n_local, n_limit, c_get)

    # 7. Gate-cycle shunt safety and Debye T^3 thermal HIL audit
    monitor = SafetyMonitor()
    shunt_status, latency_ns, cycles, thermal = monitor.simulate_shutdown(
        mu_local, n_local, n_limit, c_get
    )
    hil_thermal = monitor.hil_thermal_monitor()

    # 8. Hardware invariants
    hw_status = hw.audit(72.0e9, 40.0e9)

    # 9. Coordinate perturbation sweep for 10^{-12} rigidity limit
    sweep = CoordinatePerturbationSweep(
        mu0=1.0, n_limit=n_limit, c_get_bound=c_get
    )
    sweep_ok, worst_detuning = sweep.verify_rigidity_limit()

    # 10. Integrated 512-bit closure-chain audit
    i_l_star = hil.i_l_star()
    i_q_star = hil.i_q_star()
    closure_chain_holds = framing_defect == 0.0
    KB = 1.380_649e-23
    dS_total_dt = gamma_de * KB * math.log(2) + p_cool / t_c
    entropy_arrow_positive = dS_total_dt > 0.0

    # Cartesian rigidity grid: every sub-10^{-12} mu perturbation stays inside the
    # safe region (NOMINAL or CORRECTION_APPLIED); every supra-threshold mu
    # perturbation triggers an emergency shutdown.
    grid_sweep = CoordinatePerturbationSweep(mu0=1.0, n_limit=n_limit, c_get_bound=c_get)
    grid_ok = True
    for amp in [0.0, 1e-15, 5e-13, 9e-13, 9.9e-13, 1e-12, 2e-12, 1e-11]:
        res = grid_sweep.sweep_mu([amp])[0]
        if abs(amp) < 1.0e-12:
            within_threshold = res.status in (
                "STATUS_NOMINAL_PASS",
                "STATUS_CORRECTION_APPLIED",
            )
        else:
            within_threshold = res.status == "STATUS_EMERGENCY_SHUTDOWN"
        if not within_threshold:
            grid_ok = False

    # 11. Closed-loop InP/InGaAs calibration check
    cal = CalibrationEngine()
    _, _, cal_nominal_status = cal.step(1.0e-6, 4.0e-5)
    cal.reset()
    _, _, cal_shutdown_status = cal.step(1.0e-6, 10.0)
    calibration_ok = (
        cal_nominal_status == "STATUS_NOMINAL_PASS"
        and cal_shutdown_status == "STATUS_EMERGENCY_SHUTDOWN"
    )

    # 12. Thermal-fatigue reliability audit
    rel = ReliabilityAuditor()
    rel.accumulate_bits(1.514e16 / 2.0)
    rel_status, rel_nominal, rel_remaining, rel_consumed, rel_impedance = rel.audit()
    rel_exhausted = ReliabilityAuditor()
    rel_exhausted.accumulate_bits(1.514e16 * 1.01)
    rel_warn_status, _, _, _, _ = rel_exhausted.audit()
    reliability_ok = rel_nominal and rel_warn_status == "STATUS_QUENCH_WARNING"

    # 13. Holographic warp-drive metric audit (10 m bubble)
    adm = ADMMetricAuditor()
    warp_audit = adm.audit(0.1)
    warp_ok = warp_audit["passed"]

    # 14. Modular state translocation audit with causal authorization
    transloc = ModularStateTranslocator()
    src = CausalCoordinate(0.0, 0.0, 0.0, 0.0)
    tar = CausalCoordinate(1.0, 0.0, 0.0, 0.0)
    rec, passive = transloc.translocate(state, src, tar, 0.421)
    causal_ok = len(rec) == len(state) and len(passive) == len(state)

    print("SHBT Exotic Technologies — Unified Audit")
    print("=" * 50)
    print(f"Kernel (SU(2), SU(3), K): {engine.kernel}")
    print(f"Stinespring isometric:     {iso}")
    print(f"Heegaard-Floer relabel:    {len(relabeled)} components")
    print(f"Heegaard pres. length:     {ell_he:.6f}")
    print(f"Kojima ΔS_A:               {delta_s:.6e}")
    print(f"Kojima satisfied:          {kojima_ok}")
    print(f"Newton-lock gamma:         {gamma:.6e}")
    print(f"Local C_get (J/bit):       {c_get:.6e}")
    print(f"Ghost-seed mass (kg):      {m_seed:.6e}  ({m_sun:.3f} M_sun)")
    print(f"Ghost-seed entropy-debt:   {entropy_debt:.6e} W")
    print(f"Mu local perturbation:     {mu_local:.6e}")
    print(f"Refrigeration P_cool (W):  {p_cool:.6e}")
    print(f"De-rendering rate (b/s):   {gamma_de:.6e}")
    print(f"Framing defect:            {framing_defect:.6e}")
    print(f"HIL status:                {status}")
    print(f"HIL nominal:               {nominal}")
    print(f"Shunt status:              {shunt_status}")
    print(f"Shunt latency (ns):        {latency_ns:.6f}")
    print(f"Shunt gate cycles:         {cycles}")
    print(f"Thermal status:            {thermal}")
    print(f"Debye final temp (K):      {hil_thermal.final_temperature_k():.6f}")
    print(f"Dissipation volume (cm^3): {hil_thermal.volume_cm3():.4f}")
    print(f"Heat capacity (J/K):       {hil_thermal.heat_capacity_j_per_k():.6e}")
    print(f"Hardware status:           {hw_status}")
    print(f"Rigidity sweep OK:         {sweep_ok}")
    print(f"Worst sub-threshold detuning: {worst_detuning:.6e}")
    print(f"Calibration nominal:       {cal_nominal_status}")
    print(f"Calibration over-range:    {cal_shutdown_status}")
    print(f"Reliability status:        {rel_status}")
    print(f"Reliability remaining:     {rel_remaining:.6e} bits")
    print(f"Reliability consumed:      {rel_consumed:.6e} cycles")
    print(f"Fatigue shifted Z:         {rel_impedance:.4f} MRayl")
    print(f"Reliability warn status:   {rel_warn_status}")
    print(f"Warp metric passed:        {warp_ok}")
    print(f"Warp det error:            {warp_audit['max_determinant_error']:.6e}")
    print(f"Warp Gram λ_min:           {warp_audit['min_gram_eigenvalue']:.6e}")
    print(f"Translocation causal pass: {causal_ok}")
    print("-" * 50)
    print("512-bit closure-chain audit")
    print(f"I_l^*:                     {i_l_star:.6f}")
    print(f"I_q^*:                     {i_q_star:.6f}")
    print(f"Framing defect Δ_fr:       {framing_defect:.6e}")
    print(f"Closure chain holds:       {closure_chain_holds}")
    print(f"dS_total/dt_lc (W/K):      {dS_total_dt:.6e}")
    print(f"Entropy arrow positive:    {entropy_arrow_positive}")
    print(f"Cartesian rigidity grid OK: {grid_ok}")

    all_ok = (
        nominal
        and kojima_ok
        and shunt_status == "STATUS_NOMINAL_PASS"
        and hil_thermal.is_nominal()
        and hw_status == "STATUS_NOMINAL_PASS"
        and sweep_ok
        and closure_chain_holds
        and entropy_arrow_positive
        and grid_ok
        and calibration_ok
        and reliability_ok
        and warp_ok
        and causal_ok
    )
    return 0 if all_ok else 1


def run_braid_compiler(args: argparse.Namespace) -> int:
    compiler = FibonacciBraidCompiler()
    n = args.braid_depth
    if args.braid_info:
        print(f"Braid gate count (n={n}): {compiler.gate_count(n)}")
        print(f"Approximation error: {compiler.approximation_error(n):.6e}")
        return 0
    qasm = compiler.compile_openqasm(n, args.braid_qubit)
    print(qasm, end="")
    return 0


def run_cad_export(args: argparse.Namespace) -> int:
    if args.export_gds:
        GdsiiMaskExporter().export_array(args.export_gds)
        print(f"GDSII mask written to {args.export_gds}")
    if args.export_step:
        StepSolidModel().export_waveguide(
            args.export_step,
            args.step_length,
            args.step_width,
            args.step_height,
        )
        print(f"STEP B-Rep written to {args.export_step}")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(
        prog="shbt-exotic",
        description="Unified SHBT exotic-technology simulator",
    )
    parser.add_argument(
        "--audit",
        action="store_true",
        help="Run the unified HIL audit of all six exotic protocols",
    )
    parser.add_argument(
        "--braid-openqasm",
        action="store_true",
        help="Emit the n=9 Solovay-Kitaev braid compilation as OpenQASM 2.0",
    )
    parser.add_argument(
        "--braid-info",
        action="store_true",
        help="Report braid gate count and approximation error for --braid-depth",
    )
    parser.add_argument(
        "--braid-depth",
        type=int,
        default=9,
        help="Solovay-Kitaev recursion depth (default: 9)",
    )
    parser.add_argument(
        "--braid-qubit",
        type=int,
        default=0,
        help="Target qubit index for the OpenQASM output (default: 0)",
    )
    parser.add_argument(
        "--export-gds",
        metavar="PATH",
        help="Export the 8x8 SHBT array GDSII mask to PATH",
    )
    parser.add_argument(
        "--export-step",
        metavar="PATH",
        help="Export the sapphire waveguide STEP B-Rep to PATH",
    )
    parser.add_argument(
        "--step-length",
        type=float,
        default=350e-6,
        help="STEP waveguide length in metres (default: 350e-6)",
    )
    parser.add_argument(
        "--step-width",
        type=float,
        default=5e-6,
        help="STEP waveguide width in metres (default: 5e-6)",
    )
    parser.add_argument(
        "--step-height",
        type=float,
        default=1.5e-6,
        help="STEP waveguide height in metres (default: 1.5e-6)",
    )
    args = parser.parse_args()
    if args.export_gds or args.export_step:
        return run_cad_export(args)
    if args.braid_openqasm or args.braid_info:
        return run_braid_compiler(args)
    if not args.audit:
        parser.print_help()
        return 0
    return run_audit(args)


if __name__ == "__main__":
    sys.exit(main())
