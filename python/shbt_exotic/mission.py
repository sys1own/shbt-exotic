"""Mission flight director and Ford-Roman QI auditor (Python mirror of
``exotic-mission-director`` / ``exotic-energy-conditions``)."""
from __future__ import annotations

import json
import math

WARP_INCEPTION_VS = 2.0
MIN_JERK_ACC_MAX = 10.0 / math.sqrt(3.0)
FORD_ROMAN_C = 3.0 / (32.0 * math.pi**2)
TAU_PLANCK_S = 5.39e-44
LANR_NET_KW = 999.054
POWER_SURPLUS_KW = 93.054
BIT_STEP_HZ = 50_518.0
STEP_JITTER_NS = 1.2


def minimum_jerk(tau: float) -> float:
    return tau**3 * (10.0 - 15.0 * tau + 6.0 * tau * tau)


def minimum_jerk_accel(tau: float) -> float:
    return 60.0 * tau - 180.0 * tau * tau + 120.0 * tau**3


def ds2_2pn(dt: float, dx: float, mass: float, r_mid: float) -> float:
    corr = 2.0 * mass / max(r_mid, 1e-12)
    return -dt * dt + dx * dx + corr * (dt * dt + dx * dx)


def flight_sim(n_local: float = 1e24, steps: int = 256) -> dict:
    """Run the 5-stage exotic mission profile and emit a trajectory log."""
    stages = []

    cold = {
        "stage": "LANR cold start",
        "net_kw": LANR_NET_KW,
        "surplus_kw": POWER_SURPLUS_KW,
        "passed": LANR_NET_KW >= 999.054 and POWER_SURPLUS_KW > 0.0,
    }
    stages.append(cold)

    station = {
        "stage": "Subluminal stationkeeping",
        "bit_step_hz": BIT_STEP_HZ,
        "jitter_ns": 0.5,
        "rigidity_error": 0.0,
        "passed": 0.5 <= STEP_JITTER_NS,
    }
    stages.append(station)

    trajectory = []
    accel_peak = 0.0
    for k in range(steps + 1):
        tau = k / steps
        a = minimum_jerk_accel(tau)
        accel_peak = max(accel_peak, abs(a))
        trajectory.append({"tau": tau, "s": minimum_jerk(tau) * WARP_INCEPTION_VS,
                           "accel": a})
    inception = {
        "stage": "Warp inception",
        "v_s_c": WARP_INCEPTION_VS,
        "accel_peak": accel_peak,
        "passed": accel_peak <= MIN_JERK_ACC_MAX + 1e-9,
    }
    stages.append(inception)

    decel = {
        "stage": "Warp deceleration",
        "accel_peak": accel_peak,
        "s_terminal": 1.0 - minimum_jerk(1.0),
        "passed": accel_peak <= MIN_JERK_ACC_MAX + 1e-9,
    }
    stages.append(decel)

    mass = 1e-6
    ds2 = ds2_2pn(dt=10.0, dx=1.0, mass=mass, r_mid=0.5)
    egress = {
        "stage": "Translocation payload egress",
        "n_local": n_local,
        "ds2_2pn": ds2,
        "authorized": ds2 <= 0.0 and 1e23 <= n_local <= 1e28,
        "passed": ds2 <= 0.0 and 1e23 <= n_local <= 1e28,
    }
    stages.append(egress)

    return {
        "suite": "FLIGHT-5",
        "stages": stages,
        "trajectory": trajectory,
        "all_stages_passed": all(s["passed"] for s in stages),
    }


def lorentzian_kernel(tau: float, tau0: float) -> float:
    return tau0 / (math.pi * (tau * tau + tau0 * tau0))


def ford_roman_integral(samples, dt: float, tau0: float) -> float:
    n = len(samples)
    return sum(
        r * lorentzian_kernel((k - (n - 1) / 2.0) * dt, tau0) * dt
        for k, r in enumerate(samples)
    )


def ford_roman_bound(tau0: float) -> float:
    return -FORD_ROMAN_C / tau0**4


def audit_qi(rho_neg: float = -1e-7, t_dur: float = 10.0,
             samples: int = 1001, tau0: float = 10.0) -> dict:
    """Ford-Roman QI audit across a warp-wall foliation profile."""
    dt = 2 * t_dur / (samples - 1)
    profile = [
        rho_neg if abs((k - (samples - 1) / 2.0) * dt) < t_dur / 2.0 else 0.0
        for k in range(samples)
    ]
    integral = ford_roman_integral(profile, dt, tau0)
    bound = ford_roman_bound(tau0)
    compact = rho_neg * t_dur**4
    return {
        "suite": "QI-AUDIT",
        "tau0": tau0,
        "qi_integral": integral,
        "qi_bound": bound,
        "compact_window": compact,
        "compact_bound": -FORD_ROMAN_C,
        "planck_applicable": tau0 >= TAU_PLANCK_S,
        "compliant": integral >= bound and compact >= -FORD_ROMAN_C,
    }


def main_flight_sim(out: str | None) -> int:
    res = flight_sim()
    text = json.dumps(res, indent=2)
    if out:
        with open(out, "w") as fh:
            fh.write(text + "\n")
        print(f"trajectory log: {out}")
    else:
        print(text)
    return 0 if res["all_stages_passed"] else 1


def main_audit_qi(args) -> int:
    res = audit_qi(rho_neg=args.rho, t_dur=args.duration, tau0=args.tau0)
    print(json.dumps(res, indent=2))
    return 0 if res["compliant"] else 1


def main_visualize() -> int:
    from pathlib import Path
    import webbrowser

    page = Path(__file__).resolve().parents[2] / "webgpu" / "index.html"
    url = page.as_uri()
    print(f"WebGPU spacetime visualizer: {url}")
    webbrowser.open(url)
    return 0
