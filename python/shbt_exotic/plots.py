"""Generate Matplotlib figure PDFs for the shbt-exotic manuscript."""

from pathlib import Path

import numpy as np
from matplotlib import pyplot as plt

from shbt_exotic import GhostSeedSynthesizer, NewtonLockStasis


def _fig_dir() -> Path:
    return Path("figures")


def temporal_dilation_vs_entropy_cost() -> Path:
    """Plot Newton-lock temporal dilation factor against GET cost."""
    stasis = NewtonLockStasis()
    # Bias from zero up to 9e-13, i.e. just below the 1e-12 rigidity threshold.
    biases = np.linspace(0.0, 9.0e-13, 200)
    gammas = np.array([stasis.gamma_stasis(float(b)) for b in biases])
    c_gets = np.array([stasis.local_c_get(float(b)) for b in biases])

    out = _fig_dir() / "temporal_dilation_vs_entropy_cost.pdf"
    out.parent.mkdir(parents=True, exist_ok=True)

    fig, ax = plt.subplots(figsize=(5.0, 3.5))
    ax.semilogy(c_gets, gammas, color="#1f4e79", lw=1.5)
    ax.axvline(5.34e-175, color="#b22222", ls="--", lw=1.0, label="Landauer bound $C_{get}^{(0)}$")
    ax.set_xlabel("GET cost $C_{get}$ (J/bit)", fontsize=10)
    ax.set_ylabel("Temporal dilation factor $\\gamma_{\\mathrm{stasis}}$", fontsize=10)
    ax.set_title("Temporal Dilation vs. Entropy Cost", fontsize=11)
    ax.legend(fontsize=8)
    ax.grid(True, ls=":", alpha=0.5)
    fig.tight_layout()
    fig.savefig(out, format="pdf")
    plt.close(fig)
    return out


def seed_mass_vs_bit_overflow() -> Path:
    """Plot ghost-seed mass as a function of bit overflow."""
    ghost = GhostSeedSynthesizer()
    alpha = ghost.alpha_seed()  # M_sun per bit
    n_limit = 1.0e65
    # Bit overflow from 0 up to 5 / alpha, i.e. 5 solar masses.
    overflows = np.linspace(0.0, 5.0 / alpha, 200)
    n_locals = n_limit + overflows
    masses = np.array([ghost.seed_mass_solar(float(nl), n_limit) for nl in n_locals])

    out = _fig_dir() / "seed_mass_vs_bit_overflow.pdf"
    out.parent.mkdir(parents=True, exist_ok=True)

    fig, ax = plt.subplots(figsize=(5.0, 3.5))
    ax.plot(overflows, masses, color="#1f4e79", lw=1.5)
    ax.set_xlabel("Bit overflow $\\Delta N = N_{\\mathrm{local}} - N_{\\mathrm{limit}}$", fontsize=10)
    ax.set_ylabel("Seed mass $M_{\\mathrm{seed}}$ ($M_{\\odot}$)", fontsize=10)
    ax.set_title("Artificial Seed Mass vs. Bit Overflow", fontsize=11)
    ax.grid(True, ls=":", alpha=0.5)
    fig.tight_layout()
    fig.savefig(out, format="pdf")
    plt.close(fig)
    return out


def generate_all() -> list[Path]:
    return [temporal_dilation_vs_entropy_cost(), seed_mass_vs_bit_overflow()]


def main() -> int:
    for path in generate_all():
        print(path)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
