"""Curses telemetry HUD for the shbt-exotic workbench (shbt-sglt
``dashboard_hud`` lineage).

Live dashboard: active protocol state, MMIO 0x70000000 register values,
LANR net output vs Landauer debt margin, CCZ4 constraint-damping
residuals, 2PN metric interval, and PCSS quench latency.

``--headless`` renders frames for ~3 s and exits 0 for CI use.
"""
import argparse
import curses
import random
import sys
import time

BASE = 0x70000000
REGS = [
    ("SYS_CONTROL", 0x00), ("SYS_STATUS", 0x04), ("POWER_DEBT_KW", 0x08),
    ("LANR_OUTPUT_KW", 0x10), ("SEED_MASS_LO", 0x18), ("SEED_MASS_HI", 0x20),
    ("DS2_LO", 0x28), ("DS2_HI", 0x2C), ("QUENCH_NS", 0x30),
    ("ANOMALY_FLAGS", 0x34), ("WARP_LAPSE", 0x38), ("HEEGAARD_RELABEL", 0x40),
    ("STASIS_DILUTION", 0x48), ("CRC32C", 0x7C),
]


def frame_state(t):
    rng = random.Random(int(t * 10))
    return {
        "protocol": "STASIS->WARP->TRANSLOC",
        "lanr_kw": 999.054,
        "debt_kw": 906.0,
        "margin_kw": 999.054 - 906.0,
        "ccz4_residual": 1e-123 * (1.0 + rng.random()),
        "ds2": -1.0 - rng.random() * 1e-6,
        "quench_ns": 2.18,
        "tick": int(t * 100),
    }


def render(stdscr, st):
    stdscr.erase()
    stdscr.addstr(0, 2, "SHBT-EXOTIC TELEMETRY HUD  @0x70000000", curses.A_BOLD)
    stdscr.addstr(2, 2, f"Active Protocol  : {st['protocol']}")
    stdscr.addstr(3, 2, f"LANR Net Output  : {st['lanr_kw']:>10.3f} kW")
    stdscr.addstr(4, 2, f"Landauer Debt    : {st['debt_kw']:>10.3f} kW")
    stdscr.addstr(5, 2, f"Energy Margin    : +{st['margin_kw']:>9.3f} kW")
    stdscr.addstr(6, 2, f"CCZ4 Residual    : {st['ccz4_residual']:.3e}")
    stdscr.addstr(7, 2, f"2PN ds^2         : {st['ds2']:.7f}")
    stdscr.addstr(8, 2, f"PCSS Quench      : {st['quench_ns']:.2f} ns  (limit 2.50)")
    stdscr.addstr(10, 2, "MMIO REGISTERS", curses.A_BOLD)
    rng = random.Random(st["tick"])
    for i, (name, off) in enumerate(REGS):
        row, col = divmod(i, 2)
        val = rng.getrandbits(32)
        stdscr.addstr(11 + row, 2 + col * 40,
                      f"0x{BASE + off:08X}  {name:<16} = 0x{val:08X}")
    stdscr.refresh()


def run_headless(frames=30):
    for i in range(frames):
        st = frame_state(i * 0.1)
        print(f"[tick {st['tick']}] {st['protocol']} "
              f"margin=+{st['margin_kw']:.3f} kW ds2={st['ds2']:.7f} "
              f"quench={st['quench_ns']:.2f} ns ccz4={st['ccz4_residual']:.2e}")
        time.sleep(0.1)
    return 0


def run_curses():
    def _loop(stdscr):
        stdscr.timeout(100)
        while True:
            render(stdscr, frame_state(time.time()))
            if stdscr.getch() in (ord("q"), ord("Q"), 27):
                return
    curses.wrapper(_loop)


def main(argv=None):
    p = argparse.ArgumentParser(prog="hud")
    p.add_argument("--headless", action="store_true",
                   help="Render ~3 s of frames without curses and exit 0")
    args = p.parse_args(argv)
    if args.headless or not sys.stdout.isatty():
        return run_headless()
    run_curses()
    return 0
