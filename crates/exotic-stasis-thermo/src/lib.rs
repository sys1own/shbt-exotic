//! Temporal stasis and entropic-refrigeration thermodynamics.
//!
//! Stasis GET cost modulation against the cosmic Landauer bound
//! (5.34e-175 J/bit), entropic cooling P_cool = Gamma_de * dS * T_c,
//! 3D Chaboche backstress and Coffin-Manson thermal fatigue over the
//! sapphire / InP / diamond substrate stack.

/// Boltzmann constant (J/K, exact SI).
pub const KB: f64 = 1.380_649e-23;
/// Cosmic Landauer bound per erased bit (J/bit) — holographic floor set by
/// the de-render horizon temperature.
pub const LANDAUER_COSMIC: f64 = 5.34e-175;

/// Local GET (generalized erasure time) cost per bit for stasis bias `b`.
/// Scales the cosmic Landauer bound by the bias dilution 1/b.
pub fn local_c_get(bias: f64) -> f64 {
    LANDAUER_COSMIC / bias.abs().max(f64::MIN_POSITIVE)
}

/// Stasis dilation gamma ~ T_rate: slower clocks as C_get grows,
/// dot(T) proportional to 1/C_get.
pub fn stasis_rate(bias: f64) -> f64 {
    let c = local_c_get(bias);
    LANDAUER_COSMIC / c
}

/// Newton-lock gamma for the stasis field (Lorentz-style contraction of the
/// de-render cadence).
pub fn gamma_stasis(bias: f64) -> f64 {
    1.0 / (1.0 - stasis_rate(bias).min(1.0 - f64::EPSILON)).max(f64::EPSILON).sqrt()
}

/// De-rendering rate (bits/s) needed to sustain entropy flow `q_dot` (W)
/// into a bath at `t_c` (K).
pub fn de_rendering_rate(q_dot: f64, t_c: f64) -> f64 {
    q_dot / (KB * t_c * std::f64::consts::LN_2)
}

/// Entropic cooling power P_cool = Gamma_de * dS * T_c.
pub fn cooling_power(gamma_de: f64, t_c: f64) -> f64 {
    gamma_de * KB * std::f64::consts::LN_2 * t_c
}

/// Coffin-Manson fatigue: cycles to failure under plastic strain amplitude
/// `d_eps` with ductility coefficient c ~ -0.6 for InP-bonded stacks.
pub fn coffin_manson_cycles(d_eps: f64, eps_f: f64) -> f64 {
    (d_eps / (2.0 * eps_f)).powf(1.0 / -0.6)
}

/// Chaboche combined hardening state on one substrate interface.
#[derive(Clone, Copy, Debug)]
pub struct ChabocheBackstress {
    /// Kinematic backstress alpha (Pa).
    pub alpha: f64,
    /// Isotropic drag stress R (Pa).
    pub drag: f64,
}

impl ChabocheBackstress {
    pub fn new() -> Self {
        Self { alpha: 0.0, drag: 50.0e6 }
    }

    /// Armstrong-Frederick update: d_alpha = C*(d_eps_p) - gamma*alpha*|d_eps_p|;
    /// isotropic recovery toward R_inf with rate b.
    pub fn step(&mut self, d_eps_p: f64, c: f64, gamma: f64, r_inf: f64, b: f64) {
        self.alpha += c * d_eps_p - gamma * self.alpha * d_eps_p.abs();
        self.drag += b * (r_inf - self.drag) * d_eps_p.abs();
    }

    /// Von-Mises-equivalent yield surface radius |sigma - alpha| <= R.
    pub fn yield_margin(&self, sigma_eq: f64) -> f64 {
        self.drag - (sigma_eq - self.alpha).abs()
    }
}

impl Default for ChabocheBackstress {
    fn default() -> Self {
        Self::new()
    }
}

/// Thermal-fatigue audit across the substrate stack during a stasis field
/// collapse of amplitude `d_t` (K): the fatigue reserve is positive when the
/// imposed plastic strain leaves >10^6 cycles of margin.
pub fn fatigue_margin_cycles(d_t: f64, cte_mismatch: f64, eps_f: f64) -> f64 {
    let d_eps = cte_mismatch * d_t;
    coffin_manson_cycles(d_eps.max(1e-12), eps_f)
}
