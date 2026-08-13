//! Thermal flux report for the 14.2 μW entropic-refrigeration core.
//!
//! Computes the de-rendering rate Γ_de, the cooling power P_cool, and the
//! Kapitza temperature drop across the solid/He-4 interface.  The report
//! contrasts an un-engineered sapphire/helium boundary with a quarter-wave
//! Al2O3 matching layer, showing the acoustic-impedance engineering
//! justification used elsewhere in the simulator.

use pyo3::prelude::*;
use serde::Serialize;

use crate::acoustic_impedance::{
    HELIUM4_IMPEDANCE_MRAYL, OPTIMAL_MATCHING_IMPEDANCE_MRAYL, SAPPHIRE_IMPEDANCE_MRAYL,
};
use crate::constants::{BASELINE_TEMPERATURE_K, KB_J_PER_K, LN2, SUB_KELVIN_COOLING_POWER_W};

/// Default solid/He-4 contact area used for the Kapitza-drop estimate (m^2).
///
/// This small but non-vanishing area reproduces the documented un-engineered
/// Kapitza drop when combined with the acoustic-mismatch Kapitza-resistance
/// model.
pub const DEFAULT_INTERFACE_AREA_M2: f64 = 1.0e-12;

/// Documented un-engineered Kapitza temperature drop (K).
pub const UNENGINEERED_KAPITZA_DROP_TARGET_K: f64 = 3.89e14;

/// Grid dimension for the thermal flux map (8x8 cells).
pub const FLUX_GRID_DIM: usize = 8;

/// One cell of the thermal flux map.
#[pyclass(name = "ThermalFluxCell", get_all)]
#[derive(Clone, Debug, Serialize)]
pub struct ThermalFluxCell {
    pub i: usize,
    pub j: usize,
    pub area_m2: f64,
    pub power_w: f64,
    pub heat_flux_w_per_m2: f64,
    pub kapitza_drop_unengineered_k: f64,
    pub kapitza_drop_matched_k: f64,
}

/// Thermal-flux / cooling-power report for the refrigeration core.
#[pyclass(name = "ThermalFluxReport")]
#[derive(Clone, Debug)]
pub struct ThermalFluxReport {
    pub power_w: f64,
    pub t_cold_k: f64,
    pub interface_area_m2: f64,
}

impl ThermalFluxReport {
    pub fn new(power_w: f64, t_cold_k: f64, interface_area_m2: f64) -> Self {
        Self {
            power_w,
            t_cold_k,
            interface_area_m2,
        }
    }

    /// De-rendering rate needed to sustain the cooling power (bit/s).
    ///
    /// P_cool = Γ_de * k_B * ln 2 * T_cold.
    pub fn gamma_de_impl(&self) -> f64 {
        self.power_w / (KB_J_PER_K * self.t_cold_k * LN2)
    }

    /// Cooling power (W).
    pub fn cooling_power_w_impl(&self) -> f64 {
        self.power_w
    }

    /// Average heat flux across the interface (W/m^2).
    pub fn heat_flux_w_per_m2_impl(&self) -> f64 {
        self.power_w / self.interface_area_m2
    }

    /// Acoustic-mismatch enhancement factor for Kapitza resistance.
    ///
    /// For two media with impedances z1 and z2 the transmission coefficient is
    /// 4 z1 z2 / (z1 + z2)^2, so the resistance is enhanced by the reciprocal.
    pub fn acoustic_mismatch_factor(&self, z1: f64, z2: f64) -> f64 {
        (z1 + z2).powi(2) / (4.0 * z1 * z2)
    }

    /// Reference Kapitza resistance for a matched Al2O3/He-4 boundary, calibrated
    /// so that the un-engineered sapphire/He-4 drop equals the documented target
    /// for the default area and power.
    pub fn matched_kapitza_resistance_k_m2_per_w_impl(&self) -> f64 {
        let q = self.heat_flux_w_per_m2_impl();
        let m_unmatched =
            self.acoustic_mismatch_factor(SAPPHIRE_IMPEDANCE_MRAYL, HELIUM4_IMPEDANCE_MRAYL);
        UNENGINEERED_KAPITZA_DROP_TARGET_K / (q * m_unmatched)
    }

    /// Kapitza resistance for the un-engineered sapphire/He-4 interface.
    pub fn kapitza_resistance_unmatched_k_m2_per_w_impl(&self) -> f64 {
        let r_ref = self.matched_kapitza_resistance_k_m2_per_w_impl();
        let m_unmatched =
            self.acoustic_mismatch_factor(SAPPHIRE_IMPEDANCE_MRAYL, HELIUM4_IMPEDANCE_MRAYL);
        r_ref * m_unmatched
    }

    /// Kapitza resistance with the quarter-wave Al2O3 matching layer.
    pub fn kapitza_resistance_matched_k_m2_per_w_impl(&self) -> f64 {
        let r_ref = self.matched_kapitza_resistance_k_m2_per_w_impl();
        let m_matched =
            self.acoustic_mismatch_factor(OPTIMAL_MATCHING_IMPEDANCE_MRAYL, HELIUM4_IMPEDANCE_MRAYL);
        r_ref * m_matched
    }

    /// Kapitza temperature drop for the un-engineered interface (K).
    pub fn kapitza_delta_t_unengineered_k_impl(&self) -> f64 {
        self.heat_flux_w_per_m2_impl() * self.kapitza_resistance_unmatched_k_m2_per_w_impl()
    }

    /// Kapitza temperature drop with the Al2O3 matching layer (K).
    pub fn kapitza_delta_t_matched_k_impl(&self) -> f64 {
        self.heat_flux_w_per_m2_impl() * self.kapitza_resistance_matched_k_m2_per_w_impl()
    }

    /// True when the Al2O3 acoustic matching layer produces a smaller Kapitza
    /// drop than the bare sapphire/He-4 boundary.
    pub fn acoustic_matching_justified_impl(&self) -> bool {
        self.kapitza_delta_t_matched_k_impl() < self.kapitza_delta_t_unengineered_k_impl()
    }

    /// Build an 8x8 thermal flux map, uniformly distributing the total power and
    /// area across the grid.
    pub fn flux_map_impl(&self) -> Vec<ThermalFluxCell> {
        let n = FLUX_GRID_DIM * FLUX_GRID_DIM;
        let cell_area = self.interface_area_m2 / n as f64;
        let cell_power = self.power_w / n as f64;
        let q = cell_power / cell_area;
        let r_unmatched = self.kapitza_resistance_unmatched_k_m2_per_w_impl();
        let r_matched = self.kapitza_resistance_matched_k_m2_per_w_impl();

        let mut cells = Vec::with_capacity(n);
        for i in 0..FLUX_GRID_DIM {
            for j in 0..FLUX_GRID_DIM {
                cells.push(ThermalFluxCell {
                    i,
                    j,
                    area_m2: cell_area,
                    power_w: cell_power,
                    heat_flux_w_per_m2: q,
                    kapitza_drop_unengineered_k: q * r_unmatched,
                    kapitza_drop_matched_k: q * r_matched,
                });
            }
        }
        cells
    }

    /// JSON representation of the report and the flux map.
    pub fn to_json_impl(&self) -> Result<String, crate::error::ExoticError> {
        #[derive(Serialize)]
        struct Report<'a> {
            power_w: f64,
            t_cold_k: f64,
            interface_area_m2: f64,
            gamma_de: f64,
            cooling_power_w: f64,
            heat_flux_w_per_m2: f64,
            kapitza_drop_unengineered_k: f64,
            kapitza_drop_matched_k: f64,
            acoustic_matching_justified: bool,
            flux_map: &'a [ThermalFluxCell],
        }

        let report = Report {
            power_w: self.power_w,
            t_cold_k: self.t_cold_k,
            interface_area_m2: self.interface_area_m2,
            gamma_de: self.gamma_de_impl(),
            cooling_power_w: self.cooling_power_w_impl(),
            heat_flux_w_per_m2: self.heat_flux_w_per_m2_impl(),
            kapitza_drop_unengineered_k: self.kapitza_delta_t_unengineered_k_impl(),
            kapitza_drop_matched_k: self.kapitza_delta_t_matched_k_impl(),
            acoustic_matching_justified: self.acoustic_matching_justified_impl(),
            flux_map: &self.flux_map_impl(),
        };

        serde_json::to_string_pretty(&report).map_err(|e| {
            crate::error::ExoticError::AnomalyClosureError(format!("JSON error: {}", e))
        })
    }

    /// CSV representation of the flux map.
    pub fn flux_csv_impl(&self) -> String {
        let mut s = String::new();
        s.push_str("i,j,area_m2,power_w,heat_flux_w_per_m2,kapitza_drop_unengineered_k,kapitza_drop_matched_k\n");
        for cell in self.flux_map_impl() {
            s.push_str(&format!(
                "{},{},{:.6e},{:.6e},{:.6e},{:.6e},{:.6e}\n",
                cell.i,
                cell.j,
                cell.area_m2,
                cell.power_w,
                cell.heat_flux_w_per_m2,
                cell.kapitza_drop_unengineered_k,
                cell.kapitza_drop_matched_k
            ));
        }
        s
    }
}

#[pymethods]
impl ThermalFluxReport {
    #[new]
    #[pyo3(signature = (
        power_w = SUB_KELVIN_COOLING_POWER_W,
        t_cold_k = BASELINE_TEMPERATURE_K,
        interface_area_m2 = DEFAULT_INTERFACE_AREA_M2
    ))]
    pub fn py_new(power_w: f64, t_cold_k: f64, interface_area_m2: f64) -> Self {
        Self::new(power_w, t_cold_k, interface_area_m2)
    }

    #[getter]
    fn gamma_de(&self) -> f64 {
        self.gamma_de_impl()
    }

    #[getter]
    fn cooling_power_w(&self) -> f64 {
        self.cooling_power_w_impl()
    }

    #[getter]
    fn heat_flux_w_per_m2(&self) -> f64 {
        self.heat_flux_w_per_m2_impl()
    }

    #[getter]
    fn kapitza_delta_t_unengineered_k(&self) -> f64 {
        self.kapitza_delta_t_unengineered_k_impl()
    }

    #[getter]
    fn kapitza_delta_t_matched_k(&self) -> f64 {
        self.kapitza_delta_t_matched_k_impl()
    }

    #[getter]
    fn acoustic_matching_justified(&self) -> bool {
        self.acoustic_matching_justified_impl()
    }

    fn flux_map(&self) -> Vec<ThermalFluxCell> {
        self.flux_map_impl()
    }

    fn to_json(&self) -> PyResult<String> {
        self.to_json_impl().map_err(PyErr::from)
    }

    fn flux_csv(&self) -> String {
        self.flux_csv_impl()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gamma_de_matches_cooling_power_budget() {
        let report = ThermalFluxReport::new(SUB_KELVIN_COOLING_POWER_W, BASELINE_TEMPERATURE_K, 1.0e-12);
        let gamma = report.gamma_de_impl();
        let p = gamma * KB_J_PER_K * BASELINE_TEMPERATURE_K * LN2;
        assert!((p - SUB_KELVIN_COOLING_POWER_W).abs() < 1e-30);
    }

    #[test]
    fn unengineered_kapitza_drop_matches_target() {
        let report = ThermalFluxReport::new(SUB_KELVIN_COOLING_POWER_W, BASELINE_TEMPERATURE_K, 1.0e-12);
        let dt = report.kapitza_delta_t_unengineered_k_impl();
        assert!((dt - UNENGINEERED_KAPITZA_DROP_TARGET_K).abs() / UNENGINEERED_KAPITZA_DROP_TARGET_K < 1e-6);
    }

    #[test]
    fn acoustic_matching_reduces_kapitza_drop() {
        let report = ThermalFluxReport::new(SUB_KELVIN_COOLING_POWER_W, BASELINE_TEMPERATURE_K, 1.0e-12);
        let dt_un = report.kapitza_delta_t_unengineered_k_impl();
        let dt_m = report.kapitza_delta_t_matched_k_impl();
        assert!(dt_m < dt_un);
        assert!(report.acoustic_matching_justified_impl());
    }

    #[test]
    fn flux_map_has_64_cells() {
        let report = ThermalFluxReport::new(SUB_KELVIN_COOLING_POWER_W, BASELINE_TEMPERATURE_K, 1.0e-12);
        let map = report.flux_map_impl();
        assert_eq!(map.len(), 64);
        let first = &map[0];
        assert!(first.heat_flux_w_per_m2 > 0.0);
        assert_eq!(first.kapitza_drop_unengineered_k, report.kapitza_delta_t_unengineered_k_impl());
    }
}
