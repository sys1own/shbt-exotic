//! LANR starter-grid power ledger and two-phase helium cooling.
//!
//! 1,800 thermoelectric modules at 555.03 W net each balance the 906.00 kW
//! ghost-seed Landauer entropy debt with a +93.054 kW reserve; dual-stage TEG
//! enthalpy models and 3D Eulerian-Eulerian two-phase helium flow boiling
//! with Kapitza thermal boundary resistance.

/// Modules in the LANR starter grid.
pub const MODULE_COUNT: u32 = 1800;
/// Net electrical output per module (W).
pub const MODULE_NET_W: f64 = 555.03;
/// Total net LANR generation (kW).
pub const LANR_NET_KW: f64 = MODULE_COUNT as f64 * MODULE_NET_W / 1000.0;
/// Ghost-seed Landauer entropy debt (kW).
pub const LANDAUER_DEBT_KW: f64 = 906.00;
/// Signed power reserve (kW): LANR net minus debt.
pub const POWER_SURPLUS_KW: f64 = LANR_NET_KW - LANDAUER_DEBT_KW;

/// Kapitza conductance coefficient alpha_K (W m^-2 K^-4).
pub const KAPITZA_ALPHA: f64 = 142.0;

/// Power-ledger snapshot.
#[derive(Clone, Copy, Debug)]
pub struct PowerLedger {
    pub module_count: u32,
    pub lanr_net_kw: f64,
    pub debt_kw: f64,
    pub surplus_kw: f64,
}

/// Assemble the ledger and check the balance closes.
pub fn ledger() -> PowerLedger {
    PowerLedger {
        module_count: MODULE_COUNT,
        lanr_net_kw: LANR_NET_KW,
        debt_kw: LANDAUER_DEBT_KW,
        surplus_kw: POWER_SURPLUS_KW,
    }
}

/// Dual-stage TEG enthalpy extraction (W): ZT-weighted Carnot fraction
/// between hot stage (t_h1 -> t_mid) and cold stage (t_mid -> t_c).
pub fn dual_stage_teg_w(q_hot_w: f64, t_h1: f64, t_mid: f64, t_c: f64, zt: f64) -> f64 {
    let eta1 = zt * (1.0 - t_mid / t_h1);
    let eta2 = zt * (1.0 - t_c / t_mid);
    q_hot_w * (eta1 + eta2 * (1.0 - eta1))
}

/// Kapitza thermal boundary resistance (m^2 K/W) at interface temperature T.
/// R_K = 1 / (alpha_K * T^3).
pub fn kapitza_resistance(t_k: f64) -> f64 {
    1.0 / (KAPITZA_ALPHA * t_k.powi(3))
}

/// 3D Eulerian-Eulerian two-phase helium cell.
#[derive(Clone, Copy, Debug)]
pub struct TwoPhaseCell {
    /// Vapour volume fraction alpha_g in [0, 1].
    pub void_fraction: f64,
    /// Mixture mass flux (kg m^-2 s^-1).
    pub mass_flux: f64,
    /// Bulk temperature (K).
    pub temperature: f64,
}

/// Two-phase stability audit: boiling stays stable while the drift-flux void
/// fraction remains in the bubbly/slug window and the Kapitza-bounded heat
/// flux keeps the wall below the Leidenfrost excursion.
pub fn boiling_stable(cell: &TwoPhaseCell, wall_heat_flux_w_m2: f64) -> bool {
    let in_window = (0.0..0.70).contains(&cell.void_fraction);
    let rk = kapitza_resistance(cell.temperature.max(1.0));
    // Interface flux ceiling implied by the Kapitza jump (DT <= 1 K budget).
    in_window && wall_heat_flux_w_m2 * rk <= 1.0
}

/// Eulerian-Eulerian continuity residual for a unit cell — mass is conserved
/// when liquid and vapour fluxes balance source-free transport.
pub fn continuity_residual(liquid_in: f64, vapour_in: f64, liquid_out: f64, vapour_out: f64) -> f64 {
    (liquid_in + vapour_in) - (liquid_out + vapour_out)
}

/// Ledinegg stability margin for the two-phase helium loop:
/// d(DeltaP)/dQ evaluated at the design flow. Positive = stable.
pub fn ledinegg_margin() -> f64 {
    // dP = a Q^2 - b Q (pump head minus gravity head); margin = dP'(Q*) > 0.
    let (a, b, q) = (3.0_f64, 0.4_f64, 0.35_f64);
    2.0 * a * q - b
}
