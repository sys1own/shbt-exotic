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
  - `UnifiedStinespringMap.branching_matrix_b()` exposes the explicit 33x33 branching matrix `B` with three 11x11 blocks derived from the eigendecomposition of the reconstructed Choi matrix `C`.
- **Heegaard-Floer relabeling isometry** `T^∂`
  - Re-indexes boundary degrees of freedom while enforcing the adiabatic condition `ΔS_A = 0`.
  - `HeegaardMappingTorus` checks Kojima's inequality `Ent(φ) ≤ C · Vol(M)` with `C = 10^20` and the arithmetic bound `Ent(φ_1) ≤ [M_1 : M] · (ℓ_He - 1) · log 3`.
- **Newton-lock stationarity**
  - `T_dot ∝ 1 / C_get`; the GET cost `C_get` is modulated against the cosmic Landauer bound `5.34 × 10^{-175}` J/bit.
- **Mass-Congestion Coupling Identity**
  - `M_seed = α_seed (N_local - N_limit)` with `α_seed = 1.3258 × 10^{-51}` `M_☉` per bit, derived from the Planck mass and the lattice divisor `d_1 = gcd(26, 312) = 26`.
- **Entropic refrigeration**
  - `P_cool = Γ_de · ΔS · T_c` with `ΔS = k_B ln 2` per bit.
- **Ghost-seed entropy-debt**
  - `P_debt = (M_seed / M_☉) · 906 GW` continuous power requirement.
- **Multi-seed interference**
  - `g_{μν} = η_{μν} + Σ_i h_{μν}^{(i)} + I_{μν}` with 512-bit interference coefficients `I_00, I_11, I_22, I_33`.
  - `R_congestion = 2.954 × 10^15 m` bit-congestion radius; overlap safety audit raises `AnomalyClosureError` if `|Δμ| > 10^{-12}`.
- **Fibonacci anyon braid compiler**
  - Maps `V_unified` transition weights `sqrt(10/33)` and `sqrt(23/33)` to an abelian `B_3` representation.
  - Base word `β = σ1^2 σ2^{-2} σ1 σ2^2 σ1^{-1} σ2^{-1}` has exponent sum `1`, compiling to `U_target`.
  - Solovay-Kitaev expansion to `n = 9` yields 124 physical `u3` gates with approximation error `≤ 1.5 × 10^{-10}`.
  - `compile_openqasm(n, qubit)` emits OpenQASM 2.0 in parallel over Rayon thread pools (`O(N log N)`).
- **Closed-loop InP/InGaAs calibration**
  - Calibration tone `V_cal(t) = 3.3 V + 50 mV · sin(2π · 10 MHz · t + δφ(t))`.
  - PID bias regulator for the 3.3 V base with `Kp = 1.85 V/rad`, `Ki = 9.12 × 10^3 V/(rad·s)`, `Kd = 3.45 × 10^{-7} V·s/rad`.
  - Enforces HIL phase-jitter limit `|δφ| ≤ 5.05 × 10^{-5} rad`; returns `STATUS_EMERGENCY_SHUTDOWN` if the regulator cannot correct the jitter.
- **Thermal-fatigue reliability audit**
  - Coffin-Manson model for the Alumina/InP interface: plastic strain `Δεp = 6.0 × 10^{-6}` from `15 K` thermal swings.
  - Cycle-to-failure limit `Nf = 4.0 × 10^6` cycles; equivalent de-rendering lifetime budget `1.514 × 10^16` bits.
  - Returns `STATUS_QUENCH_WARNING` when cumulative de-rendering exceeds the budget and reports the shifted acoustic impedance `Z → 1.3250 MRayl` that raises the superconducting niobium quench risk.
- **CAD/EDA export synthesis**
  - `GdsiiMaskExporter` writes an 8×8 SHBT array GDSII mask with 50 μm pitch, Layer 10 `SUBSTRATE_INP` (350 μm), Layer 20 `AIRBRIDGE_SPAN` (1.5×5.0 μm), and Layer 25 `MET_NB_TRACE` (300 nm Niobium). Coordinates are stored at 1 pm per database unit for sub-nanometer precision.
  - `StepSolidModel` exports ISO 10303-21 B-Rep `MANIFOLD_SOLID_BREP` geometry for the sapphire waveguide, sized to the 1.1512 MRayl nominal impedance interface.

## Hardware Architecture

- InP/InGaAs SHBT transistors: `f_max = 72 GHz`.
- 2D topological-insulator edge-state waveguides for backscattering-free anyon transport.
- 2D topological surface-code lattice for micro-scale heat-sink operation.
- State routing bandwidth: `B = 40 Gb/s`, clocked by the 72 GHz SHBT array.

## HIL Safety

The dual-target Hardware-in-the-Loop monitor concurrently samples the Stasis Control Register (`C_get`) and the Mass-Congestion Register (`N_local / N_limit`).

- **Rigidity check**: eigenvector detuning `|μ_local - μ_0|` is held below `10^{-12}`.
- **Correction loop**: a Solovay-Kitaev sequence is applied if detuning enters the `0.5 × 10^{-12}` correction band.
- **Emergency shutdown**: if detuning reaches `10^{-12}` the monitor returns `STATUS_EMERGENCY_SHUTDOWN` and the bias-current shunt completes in fewer than 2.5 ns.
- **Closure chain**: the scalar framing defect `Δ_fr` is exactly `0.0` for canonical unperturbed values and remains below `10^{-12}` during active modulation.
- **Engineering stress test**: `CoordinatePerturbationSweep.safety_zone_grid()` maps the 2-D `(δμ, δN_local)` parameter space and counts the cells where the `10^{-12}` rigidity limit and thermal limits stay nominal.

## Zero-Heap Runtime and SIMD Determinism

- **Stack-allocated fixed-size arrays**: all intermediate state vectors (Stinespring blocks, HIL sensor lanes, U(1) rotation buffers) are stored as `[[f64; 8]; 2]`-style arrays on the stack. No heap allocation occurs in the high-frequency HIL audit path.
- **Custom GMP/MPFR memory**: the `rug` crate is wired to `mp_set_memory_functions` through `src/gmp_memory.rs`. Limb allocations are served from a pre-resident 16 MiB arena, eliminating variable `malloc/free` latency from the 512-bit braiding loops.
- **AVX-512 sensor pipeline**: the HIL fatal-threshold compare uses `vmovaps` / `vcmpps` / `vmovmskps` / `mov [mem], 0` on a 64-byte aligned 16-lane buffer, completing in about six cycles (~1.5 ns at 4.0 GHz).
- **U(1) phase-locked excitation**: the operator `ψ_j → e^{-i θ_j} ψ_j` is vectorised for x86_64 AVX-512 and aarch64 NEON, processing an entire 8-component dark-ledger block in a single branchless pass.

## Acoustic Impedance Micro-Engineering

- **Sapphire waveguide**: single-crystal Al2O3 with acoustic impedance `Z = 44.178 MRayl` tamps the 142.08 MW / 2.5 ns transient.
- **Quarter-wave matching layer**: optimal impedance `Z_m = sqrt(Z_sapphire * Z_He4) ≈ 1.1512 MRayl` couples the waveguide to a liquid He-4 bath.
- **Alumina formulation selector**: chooses AAO-Epoxy (`Z = 9.5 MRayl`), High-Compression Composite (`6.5–9.47 MRayl`), or Colloidal Nanocomposite (sub-10 μm layers) based on operating frequency and thickness.
- **InP substrate verification**: the transmitted acoustic pressure into InP is computed from the boundary transmission coefficient and verified to stay below the InP structural yield/phase-transition limit (~10 GPa); the waveguide peak pressure of 12.6427 GPa is consistent with the 142.08 MW transient and the chosen waveguide area.

## Engineering Synthesis

- **RF phase-modulation table**: `ExportPhaseModulationTable` maps an 8x8 conformal-dimension matrix `h_ij` and effective velocity `v_eff` to a JSON/CSV table of 64 microwave phase commands `e^{i θ}`.  Phase-shifter voltages are constrained between the gate/base turn-on `3.8 V` and collector-drain `7.4 V` bias levels.
- **Thermal flux report**: `ThermalFluxReport` computes `Γ_de = P_cool / (k_B T_c ln 2)` for the `14.2 μW` core and an 8x8 thermal-flux map.  The un-engineered sapphire/He-4 Kapitza drop is `≈ 3.89 × 10^{14} K`; a quarter-wave Al2O3 matching layer reduces this drop, justifying the acoustic-impedance engineering.

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
| Ghost-seed entropy-debt | `≈ 906 GW` for `1 M_☉` | `≈ 906 GW` |
| Framing defect `Δ_fr` | `0.0` canonical, `< 10^{-12}` active | `0.0` / `< 10^{-12}` |
| HIL status | `STATUS_NOMINAL_PASS` | nominal pass |
| Hardware clock | `≤ 72 GHz` | `72 GHz` |
| Routing bandwidth | `≤ 40 Gb/s` | `40 Gb/s` |

## Code Availability

- `shbt-precision`: [https://github.com/sys1own/shbt-precision](https://github.com/sys1own/shbt-precision)
- `shbt-warp`: [https://github.com/sys1own/shbt-warp](https://github.com/sys1own/shbt-warp)
- `shbt-recon`: [https://github.com/sys1own/shbt-recon](https://github.com/sys1own/shbt-recon)
- `shbt-exotic`: [https://github.com/sys1own/shbt-exotic](https://github.com/sys1own/shbt-exotic)
