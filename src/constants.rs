//! Canonical constants for the (26, 8, 312) SHBT exotic-technology simulator.

/// Holographic precision in bits.
pub const PREC: u32 = 512;

/// Canonical boundary kernel (SU(2)_26 x SU(3)_8 with K=312).
pub const SU2_LEVEL: usize = 26;
pub const SU3_LEVEL: usize = 8;
pub const BOUNDARY_KERNEL_K: usize = 312;

/// Visible state dimension and dark-ledger block size.
pub const VISIBLE_STATE_DIM: usize = 16;
pub const DARK_LEDGER_DIM: usize = 8;

/// Full Stinespring branching-matrix dimension and 11x11 block size.
///
/// The 33-dimensional local register decomposes into three 11x11 blocks:
/// one active-visible block (10 active + 1 shared singlet) and two dark blocks.
pub const STINESPRING_BRANCH_DIM: usize = 33;
pub const STINESPRING_BLOCK_DIM: usize = 11;

/// Rational capacities: completed dark capacity 23/33, residual 10/33.
pub const DARK_COMPLETED_NUM: i64 = 23;
pub const DARK_COMPLETED_DEN: i64 = 33;

/// Holographic noise floor ~ 10^-122.
pub const HOLOGRAPHIC_NOISE_FLOOR: f64 = 1.0e-122;

/// Eigenvector rigidity threshold.
pub const EIGENVECTOR_RIGIDITY_THRESHOLD: f64 = 1.0e-12;

/// Cosmic Landauer / GET thermodynamic bound (J/bit).
pub const C_GET_THERMODYNAMIC_BOUND_J: f64 = 5.34e-175;

/// Solar mass in kilograms.
pub const M_SUN_KG: f64 = 1.988_47e30;

/// Planck mass in kilograms (effective branch value that reproduces the
/// topological residue alpha_seed = 1.325812080894556e-51 M_sun/bit).
pub const PLANCK_MASS_KG: f64 = 2.176_434_342_051_127e-8;

/// Natural logarithm of the total holographic bit ceiling used in the
/// mass-congestion residue, N_total = e^{33}.
pub const TOTAL_BITS_NATURAL_LN: f64 = 33.0;

/// Boltzmann constant (J/K).
pub const KB_J_PER_K: f64 = 1.380_649e-23;

/// Natural logarithm of 2.
pub const LN2: f64 = std::f64::consts::LN_2;

/// Benchmark sub-Kelvin cooling power (W).
pub const SUB_KELVIN_COOLING_POWER_W: f64 = 14.2e-6;

/// Boundary state routing bandwidth (bits/s).
pub const ROUTING_BANDWIDTH_BPS: f64 = 40.0e9;

/// InP/InGaAs SHBT maximum oscillation frequency (Hz).
pub const F_MAX_HZ: f64 = 72.0e9;

/// Ghost-seed high-energy transient benchmark (W).
pub const GHOST_SEED_TRANSIENT_W: f64 = 142.08e6;

/// Macro-scale continuous cooling power (W).
pub const MACRO_COOLING_POWER_W: f64 = 906.0e9;

/// Dilution refrigerator baseline temperature for HIL operation (K).
pub const BASELINE_TEMPERATURE_K: f64 = 15.4e-3;

/// Alias for the cryogenic base temperature used in thermal audits.
pub const TEMPERATURE_K: f64 = BASELINE_TEMPERATURE_K;

/// Local holographic bit budget used for thermal heat-capacity estimates.
pub const N_LOCAL_BITS: f64 = 1.0e65;

/// Maximum tolerable phase jitter for topological edge-state transport (rad).
pub const PHASE_JITTER_THRESHOLD_RAD: f64 = 5.05e-5;

/// Debye T^3 proportionality constant for InP acoustic phonons (J/(m^3 K^4)).
pub const INP_DEBYE_A_J_PER_M3_K4: f64 = 3.877_594_83;

/// Niobium superconducting transition temperature (K).
pub const NIOBIUM_TRANSITION_TEMP_K: f64 = 9.3;

/// Minimum InP interconnect dissipation volume to avoid a thermal quench (cm^3).
/// Exact value derived from the Debye T^3 model for T_c = 9.3 K, rounded to 48.98 cm^3.
pub const MIN_DISSIPATION_VOLUME_CM3: f64 = 48.982_249_547_178_32;

/// Speed of light in vacuum (m/s).
pub const SPEED_OF_LIGHT_M_S: f64 = 299_792_458.0;

/// Lindblad charge-dephasing rate for 72 GHz microwave emission (s^-1).
pub const GAMMA_CHARGE_HZ: f64 = 8.42e-5;

/// Lindblad phonon-coupling rate for sapphire waveguide vibrations (s^-1).
pub const GAMMA_PHONON_HZ: f64 = 3.58e-5;

/// Combined environmental decoherence rate (s^-1).
pub const GAMMA_DEC_HZ: f64 = 1.2e-4;

/// Solovay-Kitaev constant used for gate-error scaling.
pub const C_SK: f64 = 1.3418e-4;

/// Physical gate error used in the SK logical-error calculation.
pub const EPSILON_0: f64 = 1.74e-10;

/// Solovay-Kitaev recursion depth for the braid compiler.
pub const SK_DEPTH: usize = 9;
