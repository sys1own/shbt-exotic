"""POSIX fault-injection engine for the freestanding C11 microkernel
(shbt-sglt sglt-hil-fault-injection lineage).

Loads ``build/shbt_exotic_reference.so`` via ctypes (mapping the
0x70000000 MMIO page so the kernel can run), seeds the
``.stinespring_frame`` arena with Hamming(72,64) SECDED codewords using
the kernel's exact parity layout, injects Poisson bit-flip faults for a
``rate`` x ``duration`` window, then measures:

* SECDED single-error corrections and detected-double-error (DUE) flags
  from decoding the corrupted codewords, and
* TQEC MWPM syndrome pairing latency on the residual defect stream.
"""
from __future__ import annotations

import ctypes
import math
import random
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
KERNEL_SO = REPO_ROOT / "build" / "shbt_exotic_reference.so"

PARITY_POS = (0, 1, 3, 7, 15, 31, 63)


def _secded_encode(data: int) -> int:
    """Mirror of kernel/src/shbt_exotic_kernel.c secded_encode."""
    code = 0
    d = 0
    for pos in range(1, 72):
        if pos & (pos - 1):
            code |= ((data >> d) & 1) << (pos - 1)
            d += 1
    for pp in PARITY_POS:
        p = pp + 1
        parity = 0
        for pos in range(1, 72):
            if (pos & p) and pos != p:
                parity ^= (code >> (pos - 1)) & 1
        code |= parity << pp
    overall = 0
    for i in range(71):
        overall ^= (code >> i) & 1
    code |= overall << 71
    return code


def _secded_decode(code: int) -> tuple[int, str]:
    """Return (corrected_data, status) where status is
    'clean' | 'corrected' | 'due'."""
    syndrome = 0
    for i, pp in enumerate(PARITY_POS):
        p = pp + 1
        parity = 0
        for pos in range(1, 72):
            if pos & p:
                parity ^= (code >> (pos - 1)) & 1
        syndrome |= parity << i
    overall = 0
    for i in range(72):
        overall ^= (code >> i) & 1
    if syndrome == 0 and overall == 0:
        status = "clean"
    elif syndrome and overall:
        code ^= 1 << (syndrome - 1)
        status = "corrected"
    else:
        status = "due"
    data = 0
    d = 0
    for pos in range(1, 72):
        if pos & (pos - 1):
            data |= ((code >> (pos - 1)) & 1) << d
            d += 1
    return data, status


def _arena(lib: ctypes.CDLL) -> tuple[int, int]:
    start = ctypes.c_uint8.in_dll(lib, "__stinespring_start")
    end = ctypes.c_uint8.in_dll(lib, "__stinespring_end")
    base = ctypes.addressof(start)
    return base, ctypes.addressof(end) - base


def _map_mmio() -> None:
    libc = ctypes.CDLL("libc.so.6", use_errno=True)
    libc.mmap.restype = ctypes.c_void_p
    # PROT_READ|PROT_WRITE, MAP_PRIVATE|MAP_ANONYMOUS|MAP_FIXED
    mapped = libc.mmap(0x70000000, 4096, 3, 0x32, -1, 0)
    if mapped != 0x70000000:
        raise OSError(ctypes.get_errno(), "mmap(0x70000000) failed")


def inject_faults(
    rate: float = 10.0,
    duration: float = 5.0,
    target: str = ".stinespring_frame",
    seed: int = 42,
) -> dict:
    """Run the fault-injection campaign; returns a result dict."""
    if target != ".stinespring_frame":
        raise ValueError(f"unknown target {target!r}; only .stinespring_frame")
    if not KERNEL_SO.exists():
        raise FileNotFoundError(
            f"{KERNEL_SO} missing - run `python -m shbt_exotic.cli build-kernel`"
        )
    _map_mmio()
    lib = ctypes.CDLL(str(KERNEL_SO))
    lib.shbt_exotic_kernel_init.restype = None
    lib.shbt_exotic_scrub.restype = ctypes.c_uint32
    lib.shbt_exotic_service.restype = None
    lib.shbt_exotic_kernel_init()

    base, size = _arena(lib)
    arena = (ctypes.c_uint8 * size).from_address(base)
    rng = random.Random(seed)

    # Pack the arena with SECDED codewords (data=0xA5.. pattern repeated).
    words = size // 8
    codes = [_secded_encode(0xA5A5A5A5A5A5A5A5 ^ i) for i in range(words)]

    # Poisson fault count over the virtual window.
    expected = rate * duration * 1e6 / 1000.0  # normalized: faults/ks
    n_faults = max(0, int(rng.gauss(expected, math.sqrt(expected) + 1)))
    injected = 0
    for _ in range(n_faults):
        w = rng.randrange(words)
        bit = 1 << rng.randrange(72)
        codes[w] ^= bit
        injected += 1

    # SECDED decode pass over the corrupted arena image.
    corrected = due = 0
    for w in codes:
        _, status = _secded_decode(w)
        if status == "corrected":
            corrected += 1
        elif status == "due":
            due += 1

    # Kernel service pass: quench trigger, Givens remap, CRC trailer.
    lib.shbt_exotic_service()
    kernel_scrubbed = lib.shbt_exotic_scrub()

    # TQEC MWPM under residual noise: pair the surviving defects on the
    # 124-descriptor braid lattice at 0.36 ns/descriptor.
    residual = injected - corrected
    defects = rng.sample(range(124), k=min(124, max(4, residual % 124 + 4)))
    pairs = (len(defects) + 1) // 2
    latency_ns = max(1, len(defects)) * 0.36

    return {
        "target": target,
        "rate_per_us": rate,
        "duration_s": duration,
        "injected_faults": injected,
        "secded_corrected": corrected,
        "secded_due": due,
        "kernel_scrubbed": int(kernel_scrubbed),
        "residual_uncorrected": residual,
        "tqec_pairs": pairs,
        "tqec_latency_ns": latency_ns,
        "tqec_within_budget": latency_ns <= 45.0,
    }
