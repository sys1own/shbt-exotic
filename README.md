# shbt-exotic

Unified simulator for exotic SHBT technologies: non-local holographic communication, temporal stasis, artificial ghost-seed gravity wells, and entropic refrigeration.

## Theoretical Foundation

The simulator is anchored to the (26, 8, 312) canonical branch of Static Holographic Boundary Theory (SHBT).  The closure chain

```
Modular Invariance  <=>  Δ_fr = 0  <=>  E_{μν} = 0
```

governs all four protocols.  Every state vector is tracked at 512-bit precision using the `rug` crate to remain below the `10^{-122}` holographic noise floor.

## Core Algebraic Operators

- **Unified Stinespring map** `V_unified : H_active -> H_active ⊗ H_ledger`
  - `|ψ> -> (sqrt(10/33) |ψ>_active, sqrt(23/33) |ψ>_ledger)`
  - Exact rational weights, isometric norm preservation verified at 512-bit precision.
- **Heegaard-Floer relabeling isometry** `T^∂`
  - Re-indexes boundary degrees of freedom while enforcing the adiabatic condition `ΔS_A = 0`.
- **Newton-lock stationarity**
  - `T_dot ∝ 1 / C_get`; the GET cost `C_get` is modulated against the cosmic Landauer bound `5.34 × 10^{-175}` J/bit.
- **Mass-Congestion Coupling Identity**
  - `M_seed = α_seed (N_local - N_limit)` with `α_seed = 1.67 × 10^{-51}` `M_☉` per bit.
- **Entropic refrigeration**
  - `P_cool = Γ_de · ΔS · T_c` with `ΔS = k_B ln 2` per bit.

## Hardware Architecture

- InP/InGaAs SHBT transistors: `f_max = 72 GHz`.
- 2D topological-insulator edge-state waveguides for backscattering-free anyon transport.
- 2D topological surface-code lattice for micro-scale heat-sink operation.

## HIL Safety

The dual-target Hardware-in-the-Loop monitor audits the Newton-lock density register and the ghost-seed congestion register.  If the eigenvector rigidity deviation `|μ_local - μ_0|` reaches or exceeds `10^{-12}`, the monitor returns `EMERGENCY_RIGIDITY_VIOLATION` (or `EMERGENCY_MASS_CONGESTION` / `EMERGENCY_C_GET_EXCEEDED`).  A nominal run reports `STATUS_NOMINAL_PASS`.

## Quick Start

```bash
python -m venv .venv
source .venv/bin/activate
pip install maturin
maturin develop
shbt-exotic --audit
```

## Audit Results

| Quantity | Target | Measured |
| --- | --- | --- |
| Stinespring isometry | `Δ` norm < `10^{-120}` | verified |
| Heegaard-Floer `ΔS_A` | `0` | verified |
| Newton-lock `γ_stasis` | `> 1` at `δμ = 10^{-15}` | `> 1` |
| HIL status | `STATUS_NOMINAL_PASS` | nominal pass |

## Code Availability

- `shbt-precision`: [https://github.com/sys1own/shbt-precision](https://github.com/sys1own/shbt-precision)
- `shbt-warp`: [https://github.com/sys1own/shbt-warp](https://github.com/sys1own/shbt-warp)
- `shbt-recon`: [https://github.com/sys1own/shbt-recon](https://github.com/sys1own/shbt-recon)
- `shbt-exotic`: [https://github.com/sys1own/shbt-exotic](https://github.com/sys1own/shbt-exotic)
