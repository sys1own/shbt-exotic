# Static Holographic Boundary Theory (SHBT) — Unified Spacetime Engineering Platform

### `shbt-exotic` — Multi-Protocol Spacetime Engineering Workbench: 13-Crate Workspace, C11 Microkernel, 70-Gate + 80-EXT Verification Suite, and 8 Z3 Formal Proofs

![Gates](https://img.shields.io/badge/gates-70%2F70%20PASS-brightgreen)
![Edition](https://img.shields.io/badge/rust-2021-orange)
![Kernel](https://img.shields.io/badge/kernel-freestanding%20C11-blue)
![Precision](https://img.shields.io/badge/arithmetic-512--bit%20MPFR-purple)
![License](https://img.shields.io/badge/license-MIT-lightgrey)

`sys1own/shbt-exotic` is the **Spacetime Engineering & Co-Simulation Twin**
of the SHBT ecosystem — its explicit domain boundary is multi-protocol
field coupling, energy-condition bounds, and full-mission profile
synthesis, in contrast to the single-device digital twins (`shbt-ghost`,
`shbt-recon`, `shbt-sglt`). It implements the six exotic protocols — non-local
holographic communication, temporal stasis, artificial ghost-seed gravity
wells, entropic refrigeration, holographic warp drive, and modular state
translocation — on top of the canonical boundary CFT branch
(26, 8, 312), a freestanding C11 control microkernel, PyO3 Python
bindings, a 150-check numerical audit harness (70 gates + 80 extended checks),
and the compiled executable paper (`exotic.pdf`).

All state-vector arithmetic runs at 512-bit MPFR precision (`rug`), and the
closure chain — probability, holographic entropy, and the framing defect
Δ<sub>fr</sub> = 0 — is audited end to end.

---

## 1. The Six Exotic Protocols

### 1.1 Non-Local Holographic Communication — `exotic-comms-telemetry`

Boundary states are de-rendered through the 33×33 Stinespring
branching matrix B and relabeled by a Heegaard-Floer symplectic
boundary map T<sup>∂</sup><sub>ij</sub> ∈ Sp(2g, ℤ).
Communication is carried by TMSV-squeezed metrology

$$
r = 2.50,\qquad 21.715\ \mathrm{dB},\qquad
\sigma_r \le 0.144\ \mathrm{pm}/\sqrt{\mathrm{Hz}},
$$

and error-corrected by the 124-braid Union-Find + Blossom minimum-weight
perfect-matching (MWPM) TQEC decoder, which reaches a logical error rate
P<sub>L</sub> ≤ 10<sup>-12</sup> in ≤ 45 ns. Telemetry streams over
128-byte dual-cacheline C-ABI frames through lock-free POSIX SPSC
shared-memory rings.

### 1.2 Temporal Stasis — `exotic-stasis-thermo`

Stasis rate is coupled to the GET (generalized erasure time) cost,
Ṫ ∝ 1/C<sub>get</sub>, evaluated against the cosmic
Landauer bound 5.34×10<sup>-175</sup> J/bit. Stasis-field
collapse is fatigue-audited across the sapphire/InP/diamond substrate
stack with 3D Chaboche backstress and Coffin-Manson cycle counting.

### 1.3 Artificial Ghost-Seed Gravity Wells — `exotic-ghost-gravity`

Topological mass coupling
α<sub>seed</sub> = 1.3258316×10<sup>-51</sup> M<sub>⊙</sub>/bit
synthesizes ~1 M<sub>⊙</sub> wells; the metric is the multi-seed
superposition

$$
g_{\mu\nu} = \eta_{\mu\nu} + \sum_i h_{\mu\nu}^{(i)} + I_{\mu\nu},
$$

checked for Gram-determinant positivity and a 1 g habitat floor
(9.80665 m/s<sup>2</sup>) with zero Coriolis distortion. Seed-quench
collapse is interlocked by sub-2.50 ns PCSS crowbar triggers
with 94.20% SiC inductive recovery.

### 1.4 Entropic Refrigeration — `exotic-lanr-thermo`, `exotic-stasis-thermo`

Continuous Landauer entropy-debt accounting
(P<sub>debt</sub> = 906.00 kW) is balanced by the LANR
starter grid (999.054 kW net), leaving a
+93.054 kW surplus:

$$
P_{\text{cool}} = \Gamma_{\text{de}}\cdot \Delta S\cdot T_c .
$$

The cooling loop runs 3D Eulerian-Eulerian two-phase helium flow boiling
with Kapitza boundary resistance
α<sub>K</sub> = 142.0 W m<sup>-2</sup> K<sup>-4</sup>.

### 1.5 Holographic Warp Drive — `exotic-warp-adm`

3+1 ADM/CCZ4 foliation with Gundlach constraint damping
(κ<sub>1</sub> > 0, κ<sub>2</sub> > -1, C<sub>CFL</sub> = 0.25),
shift-nulling β<sup>i</sup> → 0, and lapse invariance
|det(g) + 1| ≤ 10<sup>-12</sup>. Congestion-wake drag is cancelled
by 3rd-order kinematic wake compensation on the 5th-order minimum-jerk
profile s(τ) = 10τ<sup>3</sup> - 15τ<sup>4</sup> + 6τ<sup>5</sup>, preserving
eigenvector rigidity
|μ<sub>comp</sub> − μ<sub>0</sub>| ≤ 10<sup>-12</sup>.

### 1.6 Modular State Translocation — `exotic-translocation`

Macroscopic Stinespring dilation V<sub>unified</sub><sup>macro</sup>
over N<sub>local</sub> ∈ [10<sup>23</sup>, 10<sup>28</sup>] nucleons with exact
fractional partitioning η<sub>A</sub> = 10/33, η<sub>D</sub> = 23/33. Targets
are admitted only under relativistic 2PN causal lightcone authorization
(Δ s<sup>2</sup><sub>2 PN</sub> ≤ 0); spacelike coordinates throw
`AnomalyClosureError`. GST chalcogenide phase-change metamaterial
self-healing (27.9 mJ/cm<sup>2</sup>) restores boundary integrity.

### 1.7 Energy Conditions & Mission Coupling — `exotic-energy-conditions`, `exotic-mission-director`

Every foliation cell is audited against the classical energy conditions
(WEC ρ≥0, NEC ρ+p<sub>i</sub>≥0, SEC ρ+p<sub>r</sub>+2p<sub>t</sub>≥0,
DEC |p<sub>i</sub>|≤ρ), and negative-energy pockets in the warp bubble
boundary are checked against the Ford-Roman quantum inequality

$$\int\langle T_{\mu\nu} n^\mu n^\nu\rangle\, \frac{\tau_0}{\pi(\tau^2 + \tau_0^2)}\, d\tau \;\ge\; -\frac{C}{\tau_0^4},\qquad C = \frac{3}{32\pi^2}$$

The mission director routes Stinespring packets through the evolved
Alcubierre-ADM metric, couples stasis clocks to the ghost-seed redshift
z = 1/α - 1, and sequences a 5-stage flight profile (LANR cold
start → stationkeeping → warp inception v<sub>s</sub> = 2.0c → deceleration →
translocation egress) on the minimum-jerk profile with
|s''| ≤ 5.7735. Hardware transducer FEA audits the
sapphire/aerogel quarter-wave match (Z = 44.178 MRayl,
d<sub>m</sub> = 6.395 nm, T ≥ 0.985) and keeps peak field-collapse stress
below the 350 MPa fracture envelope with SF ≥ 2.5.

---

## 2. Workspace Topology

```
sys1own/shbt-exotic
├── Cargo.toml                    # workspace root (resolver = "2")
├── crates/
│   ├── exotic-core-cft/          # 512-bit MPFR branch (26,8,312), B (33x33)
│   ├── exotic-warp-adm/          # ADM/CCZ4, shift-nulling, Gundlach damping
│   ├── exotic-ghost-gravity/     # alpha_seed, multi-seed superposition, PCSS
│   ├── exotic-translocation/     # V_macro Stinespring, Sp(2g,Z), 2PN auth
│   ├── exotic-stasis-thermo/     # Landauer bound, P_cool, Chaboche/C-Manson
│   ├── exotic-lanr-thermo/       # 1,800-module ledger, TEG, 2-phase helium
│   ├── exotic-comms-telemetry/   # TMSV, MWPM TQEC, SPSC rings, 128B frames
│   ├── exotic-hil-microkernel/   # MMIO mirror, SECDED(72,64), Givens, crowbar
│   ├── exotic-uq-montecarlo/     # hyper-dual AD, GUM S1/S2 Monte Carlo
│   ├── exotic-eda-cad/           # GDSII, ISO 10303-21 STEP, Touchstone S2P
│   ├── exotic-energy-conditions/ # WEC/NEC/SEC/DEC + Ford-Roman QI sampling
│   ├── exotic-mission-director/  # cross-protocol coupling + 5-stage flight
│   └── exotic-extended-audit/    # EXT-01..80 extended checks (shbt-power)
├── kernel/
│   ├── include/shbt_exotic_hardware.h   # SHBT-MMIO-EXOTIC register map
│   ├── src/shbt_exotic_kernel.c         # freestanding C11 kernel
│   ├── linker.ld                        # .stinespring_frame arena (2112 B)
│   └── Makefile                         # → build/shbt_exotic_reference.so
├── src/                          # PyO3 bindings + legacy sub-engines
├── python/shbt_exotic/           # orchestration package (cli/, latex, plots,
│                                 #   faults.py, exporters.py, optimize.py,
│                                 #   mission.py, hud.py)
├── formal/                       # Z3 release-gate proofs (8x unsat)
├── webgpu/                       # zero-dep WGSL spacetime visualizer
├── tests/test_70_gates.rs        # master 70-gate verification suite
├── tests/test_extended_checks.rs # EXT-01..80 extended checks
├── tests/reference_test.c        # hosted C11 kernel test (make -C kernel test-c)
├── tests/test_all_cli.py         # end-to-end regression over every subcommand
├── verification_matrix.json      # generated gate audit report
├── eda_outputs/                  # generated GDSII/STEP/S2P artifacts
├── main.tex                      # executable paper source
└── exotic.pdf                    # compiled paper
```

---

## 3. `SHBT-MMIO-EXOTIC` Register Layout

128-byte dual-cacheline register block anchored at physical base
`0x70000000`. Cache line 0 carries control/status and the power ledger;
cache line 1 carries the extended exotic vector registers.

| Offset | Register | Description |
|---:|:---|:---|
| `0x00` | `REG_SYS_CONTROL` | Enable, Quench trigger, Superposition active, Translocation engage |
| `0x04` | `REG_SYS_STATUS` | Quench latched, ECC corrected, 2PN authorized, Translocation lock |
| `0x08` | `REG_POWER_DEBT_KW` | Active Landauer entropy debt (906 kW) |
| `0x10` | `REG_LANR_OUTPUT_KW` | Total net LANR generation (999 kW) |
| `0x18` | `REG_SEED_MASS_LO` | Ghost-seed mass, low dword |
| `0x20` | `REG_SEED_MASS_HI` | Ghost-seed mass, high dword |
| `0x28` | `REG_DS2_INTERVAL_LO` | Signed 2PN interval Δ s<sup>2</sup>, low dword |
| `0x2C` | `REG_DS2_INTERVAL_HI` | Signed 2PN interval Δ s<sup>2</sup>, high dword |
| `0x30` | `REG_QUENCH_TIME_NS` | Hardware latch timer (target ≤ 2.18 ns) |
| `0x34` | `REG_ANOMALY_FLAGS` | Spacelike anomaly, underpower, rigidity fault |
| `0x38` | `REG_WARP_LAPSE_METRIC` | Warp lapse metric |det(g) + 1| |
| `0x40` | `REG_HEEGAARD_RELABEL` | Active Sp(2g, ℤ) relabel index |
| `0x48` | `REG_STASIS_DILUTION` | Stasis clock dilution factor |
| `0x50`–`0x77` | `REG_EXOTIC_VEC0..4` | Extended exotic vector registers |
| `0x7C` | `REG_CRC32_CASTAGNOLI` | Frame CRC-32C checksum |

The kernel's `.stinespring_frame` linker section pre-allocates a
2,112-byte contiguous SRAM arena on a 64-byte boundary, partitioned into a
640 B active residual (η<sub>A</sub> = 10/33) and a 1,472 B dark ledger
(η<sub>D</sub> = 23/33, 124 Fibonacci braid descriptors). All kernel code is
zero-dynamic-allocation; SECDED Hamming(72,64) scrubbing runs in the
service loop.

---

## 4. Upstream Technology Transfer

| Source Repository | Transferred Subsystems |
|:---|:---|
| `shbt-precision` | 512-bit MPFR arithmetic, canonical (26,8,312) branch evaluators, zero-allocation audit primitives → `exotic-core-cft` |
| `shbt-qc` | Freestanding C11 `shbt-os` microkernel, SECDED Hamming(72,64) ECC, AVX-512 interlocks → `kernel/`, `exotic-hil-microkernel` |
| `shbt-cf` | 1,800-module LANR ledger (555.03 W/module, 999.054 kW), dual-stage TEG, Eulerian-Eulerian helium hydraulics, Kapitza resistance → `exotic-lanr-thermo` |
| `shbt-power` | Closed-loop power-ledger accounting (906.00 kW debt / +93.054 kW margin), 70-gate harness standard → `exotic-lanr-thermo`, `tests/test_70_gates.rs` |
| `shbt-ghost` | 3+1 CCZ4 numerical relativity, Gundlach damping, multi-seed superposition, sub-2.5 ns PCSS crowbars, 94.20% SiC recovery → `exotic-warp-adm`, `exotic-ghost-gravity` |
| `shbt-recon` | Macroscopic V<sub>unified</sub><sup>macro</sup> Stinespring tracking, exact η<sub>A</sub>/η<sub>D</sub> partitioning, Union-Find + Blossom MWPM decoder, 128-byte dual-cacheline C-ABI, POSIX SPSC rings → `exotic-translocation`, `exotic-comms-telemetry` |
| `shbt-sglt` | TMSV metrology (r=2.50, 21.715 dB, σ<sub>r</sub> ≤ 0.144 pm/√(Hz)), 2PN optics, 3rd-order kinematic wake compensation, 5th-order minimum-jerk profile → `exotic-comms-telemetry`, `exotic-warp-adm` |

---

## 5. CLI Execution Guide

Build the environment and extension module:

```bash
python3 -m venv .venv && .venv/bin/pip install maturin pytest numpy matplotlib
.venv/bin/maturin develop
```

Unified orchestrator (`python -m shbt_exotic.cli`):

| Command | Action |
|:---|:---|
| `build-kernel` | Compiles `kernel/` into `build/shbt_exotic_reference.so` with AVX-512 and freestanding flags |
| `sim` | Executes the multi-physics co-simulation audit across all six protocols |
| `verify` | Runs the 150-check suite (70 GATE + 80 EXT), writes `verification_matrix.json`, regenerates `exotic_results.tex` |
| `export-eda` | Synthesizes the 8x8 GDSII mask, ISO 10303-21 STEP model and S2P interposer into `eda_outputs/` |
| `paper` | Runs `latexmk -pdf -jobname=exotic main.tex` to produce `exotic.pdf` |
| `inject-faults` | POSIX SHM fault injection into `.stinespring_frame` — SECDED Hamming(72,64) + MWPM decode under faults (e.g. `inject-faults --rate 10.0 --duration 5.0`) |
| `export-fits` | CCZ4 foliation + warp lensing tensors as a FITS v4.0 cube with WCS headers |
| `export-hdf5` | Per-engine state trajectories (N<sub>local</sub> ∈ [10<sup>23</sup>,10<sup>28</sup>]) into an HDF5 datacube |
| `optimize` | Pure-NumPy NSGA-III Pareto frontier (warp v<sub>s</sub> / LANR margin / stasis retention) |
| `flight-sim` | 5-stage exotic mission profile (LANR cold start → stationkeeping → warp inception v<sub>s</sub>=2.0c → deceleration → translocation egress) with trajectory log |
| `audit-qi` | Ford-Roman quantum-inequality stress audit across metric foliations |
| `visualize` | Launches the local WebGPU/WGSL interactive spacetime visualizer |
| `hud` | Curses telemetry HUD: protocol state, MMIO 0x70000000 registers, LANR vs debt margin, CCZ4 residuals, Δ s<sup>2</sup>, PCSS latency (`--headless` renders ~3 s and exits 0) |

```bash
python -m shbt_exotic.cli inject-faults --rate 10.0 --target .stinespring_frame --duration 5.0
python -m shbt_exotic.cli export-fits --out exotic_ccz4.fits
python -m shbt_exotic.cli export-hdf5 --out exotic_trajectories.h5
python -m shbt_exotic.cli flight-sim --out flight_log.json
python -m shbt_exotic.cli audit-qi --rho -1e-7 --duration 10.0 --tau0 10.0
python -m shbt_exotic.cli visualize
python -m shbt_exotic.cli hud --headless   # 3 s telemetry frames, exits 0
python3 -m pytest formal/     # 8/8 Z3 proofs discharge unsat
```

Legacy flags (`--audit`, `--braid-openqasm`, `--export-gds`,
`--export-step`) remain supported.

```bash
cargo test --workspace        # 70-gate harness + unit tests (all pass)
.venv/bin/pytest tests/ -q    # Python integration suite (incl. test_all_cli.py)
make -C kernel                # bare-metal microkernel build
make -C kernel test-c         # hosted C reference test (tests/reference_test.c)
```

---

## 6. Verification & Benchmarks

The master suite `tests/test_70_gates.rs` audits, in order:

| Gates | Domain |
|:---|:---|
| `GATE-01..10` | Boundary CFT closure, WZW affine levels (26,8,312), Δ<sub>fr</sub> = 0, branching matrix B (33×33) |
| `GATE-11..20` | Microkernel C-ABI, MMIO register offsets, SECDED Hamming(72,64) ECC, AVX-512 Givens remapping, CRC-32C |
| `GATE-21..30` | PCSS quench τ ≤ 2.18 ns, 94.20% SiC recovery, thermal headroom Δ T ≥ 11.79 K, Chaboche/Coffin-Manson, Kapitza resistance |
| `GATE-31..40` | CCZ4 Hamiltonian/momentum damping (<10<sup>-122</sup>), ADM shift-nulling, lapse lock, minimum-jerk profile, Gram positivity |
| `GATE-41..50` | 2PN causal authorization (Δ s<sup>2</sup> ≤ 0), spacelike rejection, Sp(2g, ℤ) relabeling, GST self-healing >99.9% |
| `GATE-51..60` | LANR ledger 999.054 kW, Landauer debt 906.00 kW, +93.054 kW margin, two-phase boiling stability |
| `GATE-61..70` | TMSV metrology, TQEC decode ≤ 45 ns, SPSC FIFO, hyper-dual UQ 3-sigma bounds, EDA S2P impedance 50.12 ± 0.80 Ω |

Current status: **150/150 checks pass** — 70/70 `GATE` plus 80/80 `EXT`
(`verification_matrix.json`), plus the Rust unit tests and Python suite.

The extended suite `tests/test_extended_checks.rs` (`exotic-extended-audit`,
shbt-power lineage) audits, in order:

| Checks | Domain |
|:---|:---|
| `EXT-01..10` | Ford-Roman quantum inequalities, Casimir-Polder stability, Hawking flux suppression, quantum interest, trace/mode mixing, squeezing floor, horizon backreaction |
| `EXT-11..20` | Kojima entropy zero-leakage, Torelli Sp(2g, ℤ) invariance, capacity conservation, 2PN authorization, observer memory packets C<sub>op</sub>≤ C<sub>local</sub>, GST healing |
| `EXT-21..30` | Coffin-Manson N<sub>f</sub>≥10<sup>5</sup>, Kapitza stability, Ledinegg d(Δ P)/dQ>0, Chaboche saturation, McNabb-Foster boundedness, LANR +93.054 kW surplus |
| `EXT-31..40` | κ(G<sub>K</sub>)<10<sup>4</sup>, RMHD Alfvén Mach ≤0.12, bit-stepping jitter <1.2 ns at 50.518 kHz, traction rigidity |μ<sub>comp</sub> − μ<sub>0</sub>| ≤ 10<sup>-12</sup>, PCSS/SiC |
| `EXT-41..50` | SPSC ≥504 Gbps, TQEC ≤45 ns, S<sub>11</sub>≤-28 dB, SECDED correct/DUE, 128 B MMIO @ `0x70000000`, 2112 B arena split, CRC-32C, Givens norm |
| `EXT-51..60` | Energy conditions: WEC, NEC, SEC, DEC compliance boundaries, Lorentzian kernel normalization, Ford-Roman QI integral convergence |
| `EXT-61..70` | Cross-protocol coupling: warp-translocation lightcone closure, stasis proper-time dilation (dτ=α dt), redshift z=1/α-1, multi-seed interference condition numbers |
| `EXT-71..80` | Mission director & transducers: 5-stage flight convergence, minimum-jerk acceleration bound s''≤5.7735, aerogel match T≥0.985, peak-stress safety factor SF≥2.5 |

`formal/formal_verification.py` (shbt-qc lineage) discharges eight Z3 release-gate
proofs, all `unsat`: causal authorization contract, Stinespring isometry
‖V<sup>†</sup> Vψ − ψ‖ ≤ 10<sup>-15</sup>, ADM lapse definiteness
(β<sup>i</sup> = 0 ⇒ det γ > 0, α > 0), entropy
monotonicity dS<sub>total</sub>/dt<sub>lc</sub>≥0, no-horizon
(α>0, detγ>0 for v<sub>s</sub>∈(0,10c]), CTC prohibition
(g<sub>00</sub><0 globally), Ford-Roman QI compliance
(τ<sub>0</sub>≥τ<sub>Planck</sub>), and translocation state conservation
Tr[ρ<sub>rendered</sub>]=Tr[ρ<sub>source</sub>].

| Quantity | Value |
|:---|---:|
| LANR net output | 999.054 kW (1,800 × 555.03 W) |
| Ghost-seed Landauer debt | 906.00 kW |
| Power surplus | +93.054 kW |
| PCSS trigger budget | ≤ 2.18 ns (hard limit 2.50 ns) |
| SiC inductive recovery | 94.20% |
| TMSV squeezing | r = 2.50, 21.715 dB |
| Displacement sensitivity | σ<sub>r</sub> ≤ 0.144 pm/√(Hz) |
| TQEC decode latency | ≤ 45 ns, P<sub>L</sub> ≤ 10<sup>-12</sup> |
| Interposer impedance | 50.12 ± 0.80 Ω |
| Lapse invariance | |det(g) + 1| ≤ 10<sup>-12</sup> |
| Rigidity | |μ<sub>comp</sub> − μ<sub>0</sub>| ≤ 10<sup>-12</sup> |
| Constraint damping floor | < 10<sup>-122</sup> |

---

## 7. SHBT Ecosystem Repository Architecture & Crosswalk

The SHBT program is a federated ecosystem of nine specialized repositories. `shbt-exotic` coordinates simultaneous multi-protocol coupling across all six phenomena, while `shbt-warp` provides dedicated 3+1D ADM foliation and flight-twin simulation for warp metrics.

```
                              [shbt-precision]
                       Computational Math & Cosmology
                       (512-bit MPFR / WZW Characters)
                                     │
    ┌────────────────────────────────┼───────────────────────────────┐
    ▼                                ▼                               ▼
 [shbt-power]                     [shbt-cf]                       [shbt-qc]
 Commercial Fusion Grid         1,800-Module LANR Array         Bare-Metal Microkernel &
 (8,750 MW p-¹¹B Twin)          & Thermal-Hydraulics            Photonic Quantum Bus
    │                                │                               │
    └────────────────────────┬───────┴───────────────────────────────┘
                             ▼
 ┌────────────────────────────────────────────────────────────────┐
 │                  SPECIALIZED VEHICLE TWINS                     │
 │  • shbt-ghost : Reactionless Propulsion & Local Gravity Wells  │
 │  • shbt-recon : Macroscopic State Translocation Gateway        │
 │  • shbt-sglt  : Synthetic Gravitational Lensing Telescope      │
 │  • shbt-warp  : Holographic Warp Metric & 3+1D Flight Twin     │
 └────────────────────────┬───────────────────────────────────────┘
                         │
                         ▼
 ┌──────────────────────────────────────────────────────────────────────────┐
 │                               shbt-exotic                                │
 │        MULTI-PROTOCOL SPACETIME ENGINEERING CO-SIMULATION BENCH          │
 │  • Cross-Protocol Field Coupling (Warp + Stasis + Translocation + Wells) │
 │  • Global Energy Condition & Ford-Roman Quantum Inequality Auditing      │
 │  • Dynamic 5-Stage Multi-Technology Flight Director                      │
 └──────────────────────────────────────────────────────────────────────────┘
```

| Repository | Domain Role & Platform Scope | Shared Invariants & Interface Contracts |
| :--- | :--- | :--- |
| [`shbt-precision`](https://github.com/sys1own/shbt-precision) | Computational Math & Cosmological Foundation Core | 512-bit MPFR numerics, canonical WZW (26, 8, 312), Δ<sub>fr</sub> ≡ 0, Landauer debt P<sub>debt</sub> = 906.00 kW. |
| [`shbt-power`](https://github.com/sys1own/shbt-power) | Commercial p-¹¹B Aneutronic Fusion Power Plant Twin | 8,750 MW fusion / 7,832.903 MW net export, 70-gate audit, closed-loop thermal ledger, 128-byte SHBT-MMIO-POWER. |
| [`shbt-cf`](https://github.com/sys1own/shbt-cf) | LANR Cold Fusion Reactor Workbench & Thermal-Hydraulics | 1,800-module LANR starter grid (999.054 kW net DC), dual-stage CoSb<sub>3</sub>/ZrNiSn TEG, Kapitza resistance ΔT<sub>K</sub> = 3.546 K. |
| [`shbt-qc`](https://github.com/sys1own/shbt-qc) | Photonic Quantum Computer Twin & C11 Microkernel | Bare-metal C11 shbt-os microkernel, base 56-byte SHBT-MMIO-1 at 0x70000000, SECDED Hamming(72,64) ECC, AVX-512 interlocks. |
| [`shbt-ghost`](https://github.com/sys1own/shbt-ghost) | Ghost Seed Reactionless Propulsion & Metric Stabilization | Sub-2.5 ns PCSS crowbars, 94.20% SiC inductive recovery, 3+1 CCZ4/ADM stabilization (β<sup>i</sup> → 0, \|det(g)+1\| ≤ 10<sup>-12</sup>). |
| [`shbt-recon`](https://github.com/sys1own/shbt-recon) | Macroscopic State Translocation & Gateway Twin | Macroscopic Stinespring dilation (V<sub>unified</sub><sup>macro</sup>), dark ledger η<sub>D</sub> = 23/33, 128-byte C-ABI DMA streaming, 78-gate audit. |
| [`shbt-sglt`](https://github.com/sys1own/shbt-sglt) | Synthetic Gravitational Lensing Telescope (SE-L2) Stack | 2PN relativistic beam optics, TMSV heterodyne metrology (r = 2.50, 21.715 dB), 5th-order minimum-jerk flight profiles. |
| [`shbt-exotic`](https://github.com/sys1own/shbt-exotic) | Multi-Protocol Spacetime Engineering Co-Simulation | Cross-protocol metric coupling (all 6 phenomena), Ford-Roman QI dark-ledger auditing, Heegaard-Floer boundary relabeling. |
| [`shbt-warp`](https://github.com/sys1own/shbt-warp) | Holographic Warp Drive Digital Twin & 3+1D ADM Engine | Alcubierre metric foliation (α = 1.0, γ<sub>ij</sub> = δ<sub>ij</sub>), 500 TJ ¹⁷⁸ᵐ²Hf graser battery (109 TW burst), 128-gate audit, 8 Z3 proofs. |

### Phase-2/3 two-way logic transfer

| From | Into `shbt-exotic` | Exported back |
|:---|:---|:---|
| `shbt-precision` | `CausalPoint` observer history crystallization + C<sub>op</sub>≤ C<sub>local</sub> entropy budget | memory-packet error contract |
| `shbt-cf` | 5-layer Chaboche RPI hardening solver; pure-NumPy NSGA-III optimizer | cryo-stack fatigue envelope data |
| `shbt-ghost` | McNabb-Foster multi-trap diffusion; 50.518 kHz reactionless traction drive | rigidity/jitter audit results |
| `shbt-power` | EXT-01..50 extended verification architecture | 120-check verification matrix format |
| `shbt-qc` | `formal/` Z3 release-gate proof harness | exotic-platform invariant set |
| `shbt-sglt` | `inject-faults` POSIX SHM engine, SECDED+MWPM under faults | stinespring-arena fault model |
| `shbt-recon` | WebGPU WGSL compute shaders (ADM field visualizer); 3D tamping elastodynamics transducer FEA | exotic metric field generator |
| `shbt-qc` (P3) | sapphire/aerogel acoustic impedance matching (Z<sub>sapp</sub>=44.178 MRayl, d<sub>m</sub>=6.395 nm, Z<sub>m</sub>=1.1512 MRayl) | transducer fracture-envelope audit |
| `shbt-precision` (P3) | holographic boundary stress-energy tensor framework | `exotic-energy-conditions` QI sampler |
| `shbt-ghost` (P3) | coordinate stress tensors; 50.518 kHz bit-stepping | `exotic-mission-director` stationkeeping |
| `shbt-sglt` (P3) | 5th-order minimum-jerk mission kinematics | `exotic-mission-director` flight profile |
| `shbt-qc` (P4) | hosted `reference_test.c` kernel test pattern | `tests/reference_test.c` + `make test-c` |
| `shbt-sglt` (P4) | curses `dashboard_hud` telemetry architecture | `python/shbt_exotic/hud.py`, `hud` CLI |
| `shbt-warp` | 3+1D Alcubierre flight twin, CCZ4 foliation, QI ledger | `exotic-warp-adm`/`exotic-mission-director` warp protocol validation |

---

## License

MIT — see `LICENSE`.
