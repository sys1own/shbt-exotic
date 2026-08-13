"""Command-line interface for the shbt-exotic unified simulator."""

import argparse
import sys

from shbt_exotic import (
    CoordinatePerturbationSweep,
    EntropicRefrigerator,
    ExoticEngine,
    GhostSeedSynthesizer,
    HardwareSynthesisAuditor,
    HeegaardFloerRelabeling,
    HeegaardMappingTorus,
    HilSafetyMonitor,
    NewtonLockStasis,
    SafetyMonitor,
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

    # 7. Gate-cycle shunt safety and thermal audit
    monitor = SafetyMonitor()
    shunt_status, latency_ns, cycles, thermal = monitor.simulate_shutdown(
        mu_local, n_local, n_limit, c_get
    )
    thermal_auditor = monitor.thermal_shunt_auditor()

    # 8. Hardware invariants
    hw_status = hw.audit(72.0e9, 40.0e9)

    # 9. Coordinate perturbation sweep for 10^{-12} rigidity limit
    sweep = CoordinatePerturbationSweep(
        mu0=1.0, n_limit=n_limit, c_get_bound=c_get
    )
    sweep_ok, worst_detuning = sweep.verify_rigidity_limit()

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
    print(f"Temperature rise (K):      {thermal_auditor.temperature_rise_k():.6e}")
    print(f"Hardware status:           {hw_status}")
    print(f"Rigidity sweep OK:         {sweep_ok}")
    print(f"Worst sub-threshold detuning: {worst_detuning:.6e}")

    all_ok = (
        nominal
        and kojima_ok
        and shunt_status == "STATUS_NOMINAL_PASS"
        and hw_status == "STATUS_NOMINAL_PASS"
        and sweep_ok
    )
    return 0 if all_ok else 1


def main() -> int:
    parser = argparse.ArgumentParser(
        prog="shbt-exotic",
        description="Unified SHBT exotic-technology simulator",
    )
    parser.add_argument(
        "--audit",
        action="store_true",
        help="Run the unified HIL audit of all four exotic protocols",
    )
    args = parser.parse_args()
    if not args.audit:
        parser.print_help()
        return 0
    return run_audit(args)


if __name__ == "__main__":
    sys.exit(main())
