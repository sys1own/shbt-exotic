"""End-to-end CLI regression: exercise every shbt_exotic subcommand.

Each test shells out to ``python -m shbt_exotic.cli <cmd>`` (or make /
pytest for the kernel and formal suites) and asserts the command exits
cleanly plus the expected artifact or summary line is produced.
"""
import json
import subprocess
import sys
from pathlib import Path

import pytest

REPO = Path(__file__).resolve().parents[1]
VENV_PYTHON = sys.executable


def cli(*args, timeout=600):
    return subprocess.run(
        [VENV_PYTHON, "-m", "shbt_exotic.cli", *args],
        cwd=REPO, capture_output=True, text=True, timeout=timeout,
    )


def test_build_kernel_and_reference_c():
    proc = cli("build-kernel")
    assert proc.returncode == 0, proc.stderr
    assert (REPO / "build/shbt_exotic_reference.so").exists()
    proc = subprocess.run(
        ["make", "-C", str(REPO / "kernel"), "test-c"],
        capture_output=True, text=True, timeout=120,
    )
    assert proc.returncode == 0, proc.stderr
    assert "all checks passed" in proc.stdout


def test_verify_150_matrix():
    proc = cli("verify", timeout=900)
    assert proc.returncode == 0, proc.stdout + proc.stderr
    matrix = json.loads((REPO / "verification_matrix.json").read_text())
    assert matrix["total"] == 150
    assert matrix["passed"] == 150
    assert all(v == "PASS" for v in matrix["gates"].values())
    assert all(v == "PASS" for v in matrix["ext"].values())
    assert len(matrix["gates"]) == 70 and len(matrix["ext"]) == 80


def test_formal_proofs():
    proc = subprocess.run(
        [VENV_PYTHON, "-m", "pytest", "formal/", "-x", "-q"],
        cwd=REPO, capture_output=True, text=True, timeout=300,
    )
    assert proc.returncode == 0, proc.stdout + proc.stderr


def test_sim_cosimulation():
    proc = cli("sim", timeout=300)
    assert proc.returncode == 0, proc.stdout + proc.stderr
    assert "STATUS_NOMINAL_PASS" in proc.stdout


def test_inject_faults():
    proc = cli(
        "inject-faults", "--rate", "10.0",
        "--target", ".stinespring_frame", "--duration", "3.0",
    )
    assert proc.returncode == 0, proc.stdout + proc.stderr
    res = json.loads(proc.stdout)
    assert res["secded_due"] >= 0
    assert res["tqec_within_budget"]


def test_flight_sim(tmp_path):
    out = tmp_path / "flight_log.json"
    proc = cli("flight-sim", "--out", str(out))
    assert proc.returncode == 0, proc.stdout + proc.stderr
    log = json.loads(out.read_text())
    assert len(log["stages"]) == 5
    assert log["all_stages_passed"]
    # every translocation egress was authorized (ds^2 <= 0, timelike)
    assert log["stages"][-1]["authorized"]
    assert log["stages"][-1]["ds2_2pn"] <= 0.0


def test_audit_qi():
    proc = cli("audit-qi", "--rho", "-1e-7", "--duration", "10.0", "--tau0", "10.0")
    assert proc.returncode == 0, proc.stdout + proc.stderr


def test_optimize():
    proc = cli("optimize", "--pop", "20", "--gen", "10", timeout=300)
    assert proc.returncode == 0, proc.stdout + proc.stderr


def test_hud_headless():
    proc = cli("hud", "--headless", timeout=60)
    assert proc.returncode == 0, proc.stdout + proc.stderr
    assert "margin=+93.054 kW" in proc.stdout


def test_export_fits(tmp_path):
    out = tmp_path / "exotic_ccz4.fits"
    proc = cli("export-fits", "--out", str(out))
    assert proc.returncode == 0, proc.stdout + proc.stderr
    assert out.stat().st_size > 0


def test_export_hdf5(tmp_path):
    out = tmp_path / "exotic_trajectories.h5"
    proc = cli("export-hdf5", "--out", str(out))
    assert proc.returncode == 0, proc.stdout + proc.stderr
    assert out.stat().st_size > 0


def test_export_eda(tmp_path):
    proc = cli("export-eda", "--out-dir", str(tmp_path))
    assert proc.returncode == 0, proc.stdout + proc.stderr
    for name in ("pic8x8.gds", "waveguide.step", "interposer.s2p"):
        assert (tmp_path / name).stat().st_size > 0


@pytest.mark.slow
def test_paper():
    proc = cli("paper", timeout=600)
    assert proc.returncode == 0, proc.stdout + proc.stderr
    assert (REPO / "exotic.pdf").stat().st_size > 0
