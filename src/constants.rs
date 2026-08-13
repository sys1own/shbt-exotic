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

/// Rational capacities: completed dark capacity 23/33, residual 10/33.
pub const DARK_COMPLETED_NUM: i64 = 23;
pub const DARK_COMPLETED_DEN: i64 = 33;

/// Holographic noise floor ~ 10^-122.
pub const HOLOGRAPHIC_NOISE_FLOOR: f64 = 1.0e-122;

/// Eigenvector rigidity threshold.
pub const EIGENVECTOR_RIGIDITY_THRESHOLD: f64 = 1.0e-12;

/// Cosmic Landauer / GET thermodynamic bound (J/bit).
pub const C_GET_THERMODYNAMIC_BOUND_J: f64 = 5.34e-175;

/// Stefan-Boltzmann constant? Actually used as physical constant placeholder.
pub const ALPHA_SEED_M_SUN_PER_BIT: f64 = 1.67e-51;

/// Solar mass in kilograms.
pub const M_SUN_KG: f64 = 1.988_47e30;

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

/// Maximum tolerable phase jitter for topological edge-state transport (rad).
pub const PHASE_JITTER_THRESHOLD_RAD: f64 = 5.05e-5;
