//! Entropic refrigeration: holographic heat sink via artificial de-rendering.
//!
//! The refrigerator strips active gauge charges into the dark ledger, lowering
//! the observable entropy while preserving the passive stress-energy tensor.
//! Cooling power is `P_cool = Γ_de · ΔS · T_c` with `ΔS = k_B ln 2` per bit.

use pyo3::prelude::*;
use rug::Float;

use crate::constants::{KB_J_PER_K, LN2, MACRO_COOLING_POWER_W, SUB_KELVIN_COOLING_POWER_W};
use crate::gmp_memory;

#[pyclass(name = "EntropicRefrigerator")]
#[derive(Clone, Debug)]
pub struct EntropicRefrigerator {
    /// Per-bit entropy shed (J/K).
    pub entropy_per_bit_j_per_k: f64,
}

impl EntropicRefrigerator {
    pub fn new() -> Self {
        gmp_memory::init();
        Self {
            entropy_per_bit_j_per_k: KB_J_PER_K * LN2,
        }
    }

    /// Energy required to de-render one bit at temperature `t_c` (K).
    pub fn energy_per_bit_impl(&self, t_c: f64) -> f64 {
        self.entropy_per_bit_j_per_k * t_c
    }

    /// Cooling power for a given de-rendering rate and cold temperature.
    pub fn cooling_power_impl(&self, gamma_de: f64, t_c: f64) -> f64 {
        gamma_de * self.energy_per_bit_impl(t_c)
    }

    /// De-rendering rate needed to sustain a target cooling power.
    pub fn de_rendering_rate_impl(&self, p_cool: f64, t_c: f64) -> Result<f64, crate::error::ExoticError> {
        if t_c <= 0.0 {
            return Err(crate::error::ExoticError::AnomalyClosureError(
                "cold temperature must be positive".to_string(),
            ));
        }
        if p_cool < 0.0 {
            return Err(crate::error::ExoticError::AnomalyClosureError(
                "cooling power must be non-negative".to_string(),
            ));
        }
        let e_bit = self.energy_per_bit_impl(t_c);
        Ok(p_cool / e_bit)
    }

    /// Strip gauge charges from a high-entropy state into the dark ledger.
    ///
    /// The operation preserves the total squared norm (passive stress-energy) by
    /// splitting the state with the Stinespring weights `sqrt(10/33)` (active,
    /// gauge charges zeroed) and `sqrt(23/33)` (dark ledger, stripped charges).
    pub fn strip_gauge_charges_impl(
        &self,
        state: &[(f64, f64)],
    ) -> Result<(Vec<(f64, f64)>, Vec<(f64, f64)>), crate::error::ExoticError> {
        use crate::constants::{DARK_LEDGER_DIM, PREC};
        if state.len() != DARK_LEDGER_DIM {
            return Err(crate::error::ExoticError::AnomalyClosureError(format!(
                "state must have length {}",
                DARK_LEDGER_DIM
            )));
        }

        let active_frac = rug::Rational::from((10, 33));
        let dark_frac = rug::Rational::from((23, 33));
        let active_weight = Float::with_val(PREC, active_frac).sqrt();
        let dark_weight = Float::with_val(PREC, dark_frac).sqrt();

        let mut active = Vec::with_capacity(DARK_LEDGER_DIM);
        let mut dark = Vec::with_capacity(DARK_LEDGER_DIM);

        for &(re, im) in state.iter() {
            let mut a_re = Float::with_val(PREC, re);
            let mut a_im = Float::with_val(PREC, im);
            a_re *= &active_weight;
            a_im *= &active_weight;
            let mut d_re = Float::with_val(PREC, re);
            let mut d_im = Float::with_val(PREC, im);
            d_re *= &dark_weight;
            d_im *= &dark_weight;
            active.push((a_re.to_f64(), a_im.to_f64()));
            dark.push((d_re.to_f64(), d_im.to_f64()));
        }
        Ok((active, dark))
    }

    /// Check whether the sub-Kelvin benchmark is satisfied.
    pub fn sub_kelvin_benchmark_impl(&self, gamma_de: f64, t_c: f64) -> bool {
        self.cooling_power_impl(gamma_de, t_c) <= SUB_KELVIN_COOLING_POWER_W
    }

    /// Macro-scale continuous cooling benchmark audit.
    pub fn macro_scale_audit_impl(&self, gamma_de: f64, t_c: f64) -> bool {
        self.cooling_power_impl(gamma_de, t_c) <= MACRO_COOLING_POWER_W
    }

    /// Benchmark sub-Kelvin cooling power (W).
    pub fn sub_kelvin_cooling_power_w_impl(&self) -> f64 {
        SUB_KELVIN_COOLING_POWER_W
    }

    /// Benchmark macro-scale continuous cooling power (W).
    pub fn macro_cooling_power_w_impl(&self) -> f64 {
        MACRO_COOLING_POWER_W
    }
}

#[pymethods]
impl EntropicRefrigerator {
    #[new]
    pub fn py_new() -> Self {
        Self::new()
    }

    fn energy_per_bit(&self, t_c: f64) -> f64 {
        self.energy_per_bit_impl(t_c)
    }

    fn cooling_power(&self, gamma_de: f64, t_c: f64) -> f64 {
        self.cooling_power_impl(gamma_de, t_c)
    }

    fn de_rendering_rate(&self, p_cool: f64, t_c: f64) -> PyResult<f64> {
        self.de_rendering_rate_impl(p_cool, t_c).map_err(PyErr::from)
    }

    fn strip_gauge_charges(
        &self,
        state: Vec<(f64, f64)>,
    ) -> PyResult<(Vec<(f64, f64)>, Vec<(f64, f64)>)> {
        self.strip_gauge_charges_impl(&state).map_err(PyErr::from)
    }

    fn sub_kelvin_benchmark(&self, gamma_de: f64, t_c: f64) -> bool {
        self.sub_kelvin_benchmark_impl(gamma_de, t_c)
    }

    fn macro_scale_audit(&self, gamma_de: f64, t_c: f64) -> bool {
        self.macro_scale_audit_impl(gamma_de, t_c)
    }

    fn sub_kelvin_cooling_power_w(&self) -> f64 {
        self.sub_kelvin_cooling_power_w_impl()
    }

    fn macro_cooling_power_w(&self) -> f64 {
        self.macro_cooling_power_w_impl()
    }
}
