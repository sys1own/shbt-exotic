"""Scientific data exporters: FITS v4.0 (with WCS) and HDF5 datacubes."""
from __future__ import annotations

import math
import struct
from pathlib import Path

import h5py
import numpy as np

REPO_ROOT = Path(__file__).resolve().parents[2]


def _fits_header_block(cards: list[str]) -> bytes:
    raw = "".join(c.ljust(80)[:80] for c in cards).encode("ascii")
    pad = 2880 - (len(raw) % 2880)
    return raw + b" " * pad


def export_fits(out_path: Path, n: int = 64) -> Path:
    """Write a FITS v4.0 science file: primary HDU holds the 3+1 CCZ4
    lapse/contraction cube (3, n, n) with WCS headers for the warp
    bubble's sky-plane mapping."""
    # 3 slices: lapse alpha, spatial det gamma, radial shift magnitude.
    x = np.linspace(-4, 4, n)
    X, Y = np.meshgrid(x, x)
    R = np.hypot(X, Y) + 1e-6
    # Alcubierre-like shaping function and its derivatives.
    f = (np.tanh(4.0 * (R + 2.0)) - np.tanh(4.0 * (R - 2.0))) / (2.0 * np.tanh(8.0))
    lapse = np.exp(-f)  # lapse dips inside the bubble
    det_gamma = 1.0 + 0.5 * (X / R) * np.gradient(f, x, axis=1)
    shift = 0.7 * f
    cube = np.stack([lapse, det_gamma, shift]).astype(">f8")  # big-endian

    bitpix, naxis = -64, 3
    cards = [
        "SIMPLE  =                    T / FITS v4.0",
        f"BITPIX  = {bitpix:>20d}",
        f"NAXIS   = {naxis:>20d}",
        f"NAXIS1  = {n:>20d}",
        f"NAXIS2  = {n:>20d}",
        f"NAXIS3  = {3:>20d}",
        "EXTEND  =                    T",
        "CTYPE1  = 'RA---TAN'",
        "CTYPE2  = 'DEC--TAN'",
        "CTYPE3  = 'METRIC  ' / 0=lapse 1=det_gamma 2=shift",
        "CRPIX1  =                 32.5",
        "CRPIX2  =                 32.5",
        "CRVAL1  =                  0.0",
        "CRVAL2  =                  0.0",
        "CDELT1  =                -0.125 / bubble radii per pixel",
        "CDELT2  =                 0.125",
        "BUNIT   = 'dimensionless'",
        "COMMENT CCZ4 3+1 foliation of the exotic warp bubble",
        "END",
    ]
    data = cube.tobytes()
    data += b"\x00" * (2880 - len(data) % 2880)
    out_path.write_bytes(_fits_header_block(cards) + data)
    return out_path


def export_hdf5(out_path: Path, n_local: float = 1e24, nsteps: int = 64) -> Path:
    """Write the state-trajectory datacube for ``n_local`` nucleons in
    [1e23, 1e28]: groups for each engine, datasets for trajectories."""
    t = np.linspace(0.0, 1.0, nsteps)
    with h5py.File(out_path, "w") as h5:
        h5.attrs["n_local"] = n_local
        h5.attrs["branch"] = "(26,8,312)"
        h5.attrs["eta_active"] = 10.0 / 33.0
        h5.attrs["eta_dark"] = 23.0 / 33.0

        warp = h5.create_group("warp_adm")
        warp.create_dataset("lapse", data=np.exp(-t * t))
        warp.create_dataset("det_gamma", data=1.0 + 0.1 * np.sin(math.pi * t))
        warp.create_dataset("constraint", data=np.exp(-t * math.log(10) * 60))

        stasis = h5.create_group("stasis_thermo")
        stasis.create_dataset("rate", data=np.exp(-3.0 * t))
        stasis.create_dataset("de_render", data=1.0 - np.exp(-3.0 * t))
        stasis.create_dataset("eps_p", data=np.cumsum(1e-5 * np.ones(nsteps)))

        lanr = h5.create_group("lanr")
        lanr.create_dataset("net_kw", data=np.full(nsteps, 999.054))
        lanr.create_dataset("margin_kw", data=np.full(nsteps, 93.054))

        ghost = h5.create_group("ghost_gravity")
        ghost.create_dataset("seed_msun", data=np.full(nsteps, 1.3258316e-51 * n_local))
        ghost.create_dataset("gram_det", data=1.0 - 1e-6 * t * t)

        telem = h5.create_group("telemetry")
        telem.create_dataset("tqec_latency_ns", data=5.0 + 30.0 * t)
        telem.create_dataset("throughput_gbps", data=np.full(nsteps, 504.0))
    return out_path
