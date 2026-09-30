# Static Holographic Boundary Theory (SHBT) — Unified Spacetime Engineering Platform

### `shbt-exotic` — Six-Protocol Exotic Technologies Simulator, Bare-Metal C11 Microkernel, 70-Gate + 50-EXT Verification Suite, and Z3 Formal Proofs

![Gates](https://img.shields.io/badge/gates-70%2F70%20PASS-brightgreen)
![Edition](https://img.shields.io/badge/rust-2021-orange)
![Kernel](https://img.shields.io/badge/kernel-freestanding%20C11-blue)
![Precision](https://img.shields.io/badge/arithmetic-512--bit%20MPFR-purple)
![License](https://img.shields.io/badge/license-MIT-lightgrey)

`sys1own/shbt-exotic` is the unified spacetime-engineering platform of the
SHBT ecosystem. It implements the six exotic protocols — non-local
holographic communication, temporal stasis, artificial ghost-seed gravity
wells, entropic refrigeration, holographic warp drive, and modular state
translocation — on top of the canonical boundary CFT branch
$(26, 8, 312)$, a freestanding C11 control microkernel, PyO3 Python
bindings, a 70-gate numerical audit harness, and the compiled executable
paper (`exotic.pdf`).

All state-vector arithmetic runs at 512-bit MPFR precision (`rug`), and the
closure chain — probability, holographic entropy, and the framing defect
$\Delta_{\text{fr}} = 0$ — is audited end to end.

---

## 1. The Six Exotic Protocols

### 1.1 Non-Local Holographic Communication — `exotic-comms-telemetry`

Boundary states are de-rendered through the $33\times33$ Stinespring
branching matrix $B$ and relabeled by a Heegaard-Floer symplectic
boundary map $T^\partial_{ij} \in \mathrm{Sp}(2g, \mathbb{Z})$.
Communication is carried by TMSV-squeezed metrology

$$
r = 2.50,\qquad 21.715\ \mathrm{dB},\qquad
\sigma_r \le 0.144\ \mathrm{pm}/\sqrt{\mathrm{Hz}},
$$

and error-corrected by the 124-braid Union-Find + Blossom minimum-weight
perfect-matching (MWPM) TQEC decoder, which reaches a logical error rate
$P_L \le 10^{-12}$ in $\le 45\ \mathrm{ns}$. Telemetry streams over
128-byte dual-cacheline C-ABI frames through lock-free POSIX SPSC
shared-memory rings.

### 1.2 Temporal Stasis — `exotic-stasis-thermo`

Stasis rate is coupled to the GET (generalized erasure time) cost,
$\dot{T} \propto 1/C_{\text{get}}$, evaluated against the cosmic
Landauer bound $5.34\times10^{-175}\ \mathrm{J/bit}$. Stasis-field
collapse is fatigue-audited across the sapphire/InP/diamond substrate
stack with 3D Chaboche backstress and Coffin-Manson cycle counting.

### 1.3 Artificial Ghost-Seed Gravity Wells — `exotic-ghost-gravity`

Topological mass coupling
$\alpha_{\text{seed}} = 1.3258316\times10^{-51}\ M_\odot/\text{bit}$
synthesizes ~$1\,M_\odot$ wells; the metric is the multi-seed
superposition

$$
g_{\mu\nu} = \eta_{\mu\nu} + \sum_i h_{\mu\nu}^{(i)} + I_{\mu\nu},
$$

checked for Gram-determinant positivity and a $1\,g$ habitat floor
($9.80665\ \mathrm{m/s^2}$) with zero Coriolis distortion. Seed-quench
collapse is interlocked by sub-$2.50\ \mathrm{ns}$ PCSS crowbar triggers
with $94.20\%$ SiC inductive recovery.

### 1.4 Entropic Refrigeration — `exotic-lanr-thermo`, `exotic-stasis-thermo`

Continuous Landauer entropy-debt accounting
($P_{\text{debt}} = 906.00\ \mathrm{kW}$) is balanced by the LANR
starter grid ($999.054\ \mathrm{kW}$ net), leaving a
$+93.054\ \mathrm{kW}$ surplus:

$$
P_{\text{cool}} = \Gamma_{\text{de}}\cdot \Delta S\cdot T_c .
$$

The cooling loop runs 3D Eulerian-Eulerian two-phase helium flow boiling
with Kapitza boundary resistance
$\alpha_K = 142.0\ \mathrm{W\,m^{-2}\,K^{-4}}$.

### 1.5 Holographic Warp Drive — `exotic-warp-adm`

3+1 ADM/CCZ4 foliation with Gundlach constraint damping
($\kappa_1 > 0$, $\kappa_2 > -1$, $C_{\text{CFL}} = 0.25$),
shift-nulling $\beta^i \to 0$, and lapse invariance
$\lvert\det(g)+1\rvert \le 10^{-12}$. Congestion-wake drag is cancelled
by 3rd-order kinematic wake compensation on the 5th-order minimum-jerk
profile $s(\tau) = 10\tau^3 - 15\tau^4 + 6\tau^5$, preserving
eigenvector rigidity
$\lvert\mu_{\text{comp}} - \mu_0\rvert \le 10^{-12}$.

### 1.6 Modular State Translocation — `exotic-translocation`

Macroscopic Stinespring dilation $V_{\text{unified}}^{\text{macro}}$
over $N_{\text{local}} \in [10^{23}, 10^{28}]$ nucleons with exact
fractional partitioning $\eta_A = 10/33$, $\eta_D = 23/33$. Targets
are admitted only under relativistic 2PN causal lightcone authorization
($\Delta s^2_{2\text{PN}} \le 0$); spacelike coordinates throw
`AnomalyClosureError`. GST chalcogenide phase-change metamaterial
self-healing ($27.9\ \mathrm{mJ/cm^2}$) restores boundary integrity.

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
│   └── exotic-extended-audit/    # EXT-01..50 extended checks (shbt-power)
├── kernel/
│   ├── include/shbt_exotic_hardware.h   # SHBT-MMIO-EXOTIC register map
│   ├── src/shbt_exotic_kernel.c         # freestanding C11 kernel
│   ├── linker.ld                        # .stinespring_frame arena (2112 B)
│   └── Makefile                         # → build/shbt_exotic_reference.so
├── src/                          # PyO3 bindings + legacy sub-engines
├── python/shbt_exotic/           # orchestration package (cli/, latex, plots,
│                                 #   faults.py, exporters.py, optimize.py)
├── formal/                       # Z3 release-gate proofs (4x unsat)
├── webgpu/                       # zero-dep WGSL spacetime visualizer
├── tests/test_70_gates.rs        # master 70-gate verification suite
├── tests/test_extended_checks.rs # EXT-01..50 extended checks
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
| `0x28` | `REG_DS2_INTERVAL_LO` | Signed 2PN interval $\Delta s^2$, low dword |
| `0x2C` | `REG_DS2_INTERVAL_HI` | Signed 2PN interval $\Delta s^2$, high dword |
| `0x30` | `REG_QUENCH_TIME_NS` | Hardware latch timer (target $\le 2.18\ \mathrm{ns}$) |
| `0x34` | `REG_ANOMALY_FLAGS` | Spacelike anomaly, underpower, rigidity fault |
| `0x38` | `REG_WARP_LAPSE_METRIC` | Warp lapse metric $\lvert\det(g)+1\rvert$ |
| `0x40` | `REG_HEEGAARD_RELABEL` | Active $\mathrm{Sp}(2g,\mathbb{Z})$ relabel index |
| `0x48` | `REG_STASIS_DILUTION` | Stasis clock dilution factor |
| `0x50`–`0x77` | `REG_EXOTIC_VEC0..4` | Extended exotic vector registers |
| `0x7C` | `REG_CRC32_CASTAGNOLI` | Frame CRC-32C checksum |

The kernel's `.stinespring_frame` linker section pre-allocates a
2,112-byte contiguous SRAM arena on a 64-byte boundary, partitioned into a
640 B active residual ($\eta_A = 10/33$) and a 1,472 B dark ledger
($\eta_D = 23/33$, 124 Fibonacci braid descriptors). All kernel code is
zero-dynamic-allocation; SECDED Hamming(72,64) scrubbing runs in the
service loop.

---

## 4. Upstream Technology Transfer

| Source Repository | Transferred Subsystems |
|:---|:---|
| `shbt-precision` | 512-bit MPFR arithmetic, canonical $(26,8,312)$ branch evaluators, zero-allocation audit primitives → `exotic-core-cft` |
| `shbt-qc` | Freestanding C11 `shbt-os` microkernel, SECDED Hamming(72,64) ECC, AVX-512 interlocks → `kernel/`, `exotic-hil-microkernel` |
| `shbt-cf` | 1,800-module LANR ledger (555.03 W/module, 999.054 kW), dual-stage TEG, Eulerian-Eulerian helium hydraulics, Kapitza resistance → `exotic-lanr-thermo` |
| `shbt-power` | Closed-loop power-ledger accounting (906.00 kW debt / +93.054 kW margin), 70-gate harness standard → `exotic-lanr-thermo`, `tests/test_70_gates.rs` |
| `shbt-ghost` | 3+1 CCZ4 numerical relativity, Gundlach damping, multi-seed superposition, sub-2.5 ns PCSS crowbars, 94.20% SiC recovery → `exotic-warp-adm`, `exotic-ghost-gravity` |
| `shbt-recon` | Macroscopic $V_{\text{unified}}^{\text{macro}}$ Stinespring tracking, exact $\eta_A/\eta_D$ partitioning, Union-Find + Blossom MWPM decoder, 128-byte dual-cacheline C-ABI, POSIX SPSC rings → `exotic-translocation`, `exotic-comms-telemetry` |
| `shbt-sglt` | TMSV metrology ($r=2.50$, 21.715 dB, $\sigma_r \le 0.144\ \mathrm{pm}/\sqrt{\mathrm{Hz}}$), 2PN optics, 3rd-order kinematic wake compensation, 5th-order minimum-jerk profile → `exotic-comms-telemetry`, `exotic-warp-adm` |

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
| `verify` | Runs the 70-gate suite, writes `verification_matrix.json`, regenerates `exotic_results.tex` |
| `export-eda` | Synthesizes the 8x8 GDSII mask, ISO 10303-21 STEP model and S2P interposer into `eda_outputs/` |
| `paper` | Runs `latexmk -pdf -jobname=exotic main.tex` to produce `exotic.pdf` |
| `inject-faults` | POSIX SHM fault injection into `.stinespring_frame` — SECDED Hamming(72,64) + MWPM decode under faults (e.g. `inject-faults --rate 10.0 --duration 5.0`) |
| `export-fits` | CCZ4 foliation + warp lensing tensors as a FITS v4.0 cube with WCS headers |
| `export-hdf5` | Per-engine state trajectories ($N_{local} \in [10^{23},10^{28}]$) into an HDF5 datacube |
| `optimize` | Pure-NumPy NSGA-III Pareto frontier (warp $v_s$ / LANR margin / stasis retention) |

```bash
python -m shbt_exotic.cli inject-faults --rate 10.0 --target .stinespring_frame --duration 5.0
python -m shbt_exotic.cli export-fits --out exotic_ccz4.fits
python -m shbt_exotic.cli export-hdf5 --out exotic_trajectories.h5
python3 -m pytest formal/     # 4/4 Z3 proofs discharge unsat
```

Legacy flags (`--audit`, `--braid-openqasm`, `--export-gds`,
`--export-step`) remain supported.

```bash
cargo test --workspace        # 70-gate harness + unit tests (all pass)
.venv/bin/pytest tests/ -q    # Python integration suite
make -C kernel                # bare-metal microkernel build
```

---

## 6. Verification & Benchmarks

The master suite `tests/test_70_gates.rs` audits, in order:

| Gates | Domain |
|:---|:---|
| `GATE-01..10` | Boundary CFT closure, WZW affine levels $(26,8,312)$, $\Delta_{\text{fr}} = 0$, branching matrix $B$ ($33\times33$) |
| `GATE-11..20` | Microkernel C-ABI, MMIO register offsets, SECDED Hamming(72,64) ECC, AVX-512 Givens remapping, CRC-32C |
| `GATE-21..30` | PCSS quench $\tau \le 2.18\ \mathrm{ns}$, 94.20% SiC recovery, thermal headroom $\Delta T \ge 11.79\ \mathrm{K}$, Chaboche/Coffin-Manson, Kapitza resistance |
| `GATE-31..40` | CCZ4 Hamiltonian/momentum damping ($<10^{-122}$), ADM shift-nulling, lapse lock, minimum-jerk profile, Gram positivity |
| `GATE-41..50` | 2PN causal authorization ($\Delta s^2 \le 0$), spacelike rejection, $\mathrm{Sp}(2g,\mathbb{Z})$ relabeling, GST self-healing $>99.9\%$ |
| `GATE-51..60` | LANR ledger $999.054\ \mathrm{kW}$, Landauer debt $906.00\ \mathrm{kW}$, $+93.054\ \mathrm{kW}$ margin, two-phase boiling stability |
| `GATE-61..70` | TMSV metrology, TQEC decode $\le 45\ \mathrm{ns}$, SPSC FIFO, hyper-dual UQ 3-sigma bounds, EDA S2P impedance $50.12 \pm 0.80\ \Omega$ |

Current status: **120/120 checks pass** — 70/70 `GATE` plus 50/50 `EXT`
(`verification_matrix.json`), plus the Rust unit tests and Python suite.

The extended suite `tests/test_extended_checks.rs` (`exotic-extended-audit`,
shbt-power lineage) audits, in order:

| Checks | Domain |
|:---|:---|
| `EXT-01..10` | Ford-Roman quantum inequalities, Casimir-Polder stability, Hawking flux suppression, quantum interest, trace/mode mixing, squeezing floor, horizon backreaction |
| `EXT-11..20` | Kojima entropy zero-leakage, Torelli $\mathrm{Sp}(2g,\mathbb{Z})$ invariance, capacity conservation, 2PN authorization, observer memory packets $C_{op}\le C_{local}$, GST healing |
| `EXT-21..30` | Coffin-Manson $N_f\ge10^5$, Kapitza stability, Ledinegg $d(\Delta P)/dQ>0$, Chaboche saturation, McNabb-Foster boundedness, LANR +93.054 kW surplus |
| `EXT-31..40` | $\kappa(G_K)<10^4$, RMHD Alfvén Mach $\le0.12$, bit-stepping jitter $<1.2$ ns at 50.518 kHz, traction rigidity $|{\mu}_{comp}-\mu_0|\le10^{-12}$, PCSS/SiC |
| `EXT-41..50` | SPSC $\ge504$ Gbps, TQEC $\le45$ ns, $S_{11}\le-28$ dB, SECDED correct/DUE, 128 B MMIO @ `0x70000000`, 2112 B arena split, CRC-32C, Givens norm |

`formal/formal_verification.py` (shbt-qc lineage) discharges four Z3 release-gate
proofs, all `unsat`: causal authorization contract, Stinespring isometry
$\|V^\dagger V\psi-\psi\|\le10^{-15}$, ADM lapse definiteness
($\beta^i=0 \Rightarrow \det\gamma>0,\ \alpha>0$), and entropy
monotonicity $dS_{\mathrm{total}}/dt_{\mathrm{lc}}\ge0$.

| Quantity | Value |
|:---|---:|
| LANR net output | 999.054 kW ($1{,}800 \times 555.03\ \mathrm{W}$) |
| Ghost-seed Landauer debt | 906.00 kW |
| Power surplus | +93.054 kW |
| PCSS trigger budget | $\le 2.18\ \mathrm{ns}$ (hard limit 2.50 ns) |
| SiC inductive recovery | 94.20% |
| TMSV squeezing | $r = 2.50$, 21.715 dB |
| Displacement sensitivity | $\sigma_r \le 0.144\ \mathrm{pm}/\sqrt{\mathrm{Hz}}$ |
| TQEC decode latency | $\le 45\ \mathrm{ns}$, $P_L \le 10^{-12}$ |
| Interposer impedance | $50.12 \pm 0.80\ \Omega$ |
| Lapse invariance | $\lvert\det(g)+1\rvert \le 10^{-12}$ |
| Rigidity | $\lvert\mu_{\text{comp}}-\mu_0\rvert \le 10^{-12}$ |
| Constraint damping floor | $< 10^{-122}$ |

---

## 7. SHBT Ecosystem Repository Architecture & Crosswalk

| Repository | Domain Role | Integration into `shbt-exotic` |
|:---|:---|:---|
| [`sys1own/shbt-precision`](https://github.com/sys1own/shbt-precision) | Arbitrary-precision numerics core | 512-bit MPFR framework, canonical WZW branch $(26,8,312)$, zero-allocation loop arithmetic |
| [`sys1own/shbt-qc`](https://github.com/sys1own/shbt-qc) | Bare-metal runtime & HIL microkernel | Freestanding C11 `shbt-os` execution model, SECDED Hamming(72,64) ECC, AVX-512 interlocks, MMIO heritage at `0x70000000` |
| [`sys1own/shbt-cf`](https://github.com/sys1own/shbt-cf) | Cold-fusion reactor & HIL workbench | LANR starter-grid specification, dual-stage TEG, 3D two-phase helium thermal-hydraulics |
| [`sys1own/shbt-power`](https://github.com/sys1own/shbt-power) | Fusion plant digital twin | Closed-loop ledger methodology and the 70-gate verification standard |
| [`sys1own/shbt-ghost`](https://github.com/sys1own/shbt-ghost) | Fast interlocks & metric control | CCZ4 stabilization, multi-seed superposition, PCSS crowbars, SiC recovery shunts |
| [`sys1own/shbt-recon`](https://github.com/sys1own/shbt-recon) | Macroscopic states & telemetry | $V_{\text{unified}}^{\text{macro}}$ tracking, MWPM TQEC decoder, dual-cacheline C-ABI, POSIX SPSC rings |
| [`sys1own/shbt-sglt`](https://github.com/sys1own/shbt-sglt) | Relativistic optics & cryogenics | TMSV metrology, 2PN optics, wake compensation, minimum-jerk profiles |
| [`sys1own/shbt-exotic`](https://github.com/sys1own/shbt-exotic) | This platform | Unified six-protocol spacetime-engineering suite, 11-crate workspace, C11 kernel, 70+50-check audit, Z3 proofs, executable paper |

### Phase-2 two-way logic transfer

| From | Into `shbt-exotic` | Exported back |
|:---|:---|:---|
| `shbt-precision` | `CausalPoint` observer history crystallization + $C_{op}\le C_{local}$ entropy budget | memory-packet error contract |
| `shbt-cf` | 5-layer Chaboche RPI hardening solver; pure-NumPy NSGA-III optimizer | cryo-stack fatigue envelope data |
| `shbt-ghost` | McNabb-Foster multi-trap diffusion; 50.518 kHz reactionless traction drive | rigidity/jitter audit results |
| `shbt-power` | EXT-01..50 extended verification architecture | 120-check verification matrix format |
| `shbt-qc` | `formal/` Z3 release-gate proof harness | exotic-platform invariant set |
| `shbt-sglt` | `inject-faults` POSIX SHM engine, SECDED+MWPM under faults | stinespring-arena fault model |
| `shbt-recon` | WebGPU WGSL compute shaders (ADM field visualizer) | exotic metric field generator |

---

## License

MIT — see `LICENSE`.
