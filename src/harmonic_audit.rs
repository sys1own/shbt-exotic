//! Frequency-domain harmonic/structural audit for the sapphire waveguide assembly.
//!
//! Audits STEP-exported waveguide geometry against the four critical resonance
//! modes and enforces loss-factor, damping-ratio, and niobium quench limits
//! for megawatt-scale transients.

use pyo3::prelude::*;

use crate::constants::{
    BASELINE_TEMPERATURE_K, INP_DEBYE_A_J_PER_M3_K4, NIOBIUM_TRANSITION_TEMP_K,
};
use crate::error::ExoticError;

/// Critical resonance modes for the sapphire waveguide assembly (Hz).
pub const SHEAR_MODE_HZ: f64 = 4.12e6;
pub const LONGITUDINAL_MODE_HZ: f64 = 8.64e6;
pub const TORSIONAL_MODE_HZ: f64 = 12.35e6;
pub const FLEXURAL_MODE_HZ: f64 = 19.82e6;

/// Minimum damping loss factor for high-order flexural coupling.
pub const MIN_LOSS_FACTOR: f64 = 1.15e-3;

/// Minimum stable damping ratio across active bands.
pub const MIN_DAMPING_RATIO: f64 = 6.0e-4;

/// Frequency tolerance for matching an audit mode to its nominal value.
pub const MODE_FREQUENCY_TOLERANCE_HZ: f64 = 0.05e6;

/// Physical properties for a C-plane sapphire waveguide (room temperature).
pub const SAPPHIRE_DENSITY_KG_M3: f64 = 3980.0;
pub const SAPPHIRE_YOUNG_MODULUS_PA: f64 = 345.0e9;
pub const SAPPHIRE_SHEAR_MODULUS_PA: f64 = 145.0e9;

/// Standard 2.5 ns emergency transient window (s).
pub const TRANSIENT_WINDOW_S: f64 = 2.5e-9;

/// A structural/harmonic resonance mode reported by FEA.
#[derive(Clone, Debug)]
pub struct StructuralMode {
    pub mode_type: String,
    pub frequency_hz: f64,
    pub loss_factor: f64,
    pub damping_ratio: f64,
    pub elastic_energy_j: f64,
}

/// Auditor for frequency-domain structural resonance and thermal hotspot checks.
#[pyclass(name = "HarmonicAuditor")]
#[derive(Clone, Debug)]
pub struct HarmonicAuditor;

impl HarmonicAuditor {
    pub fn new() -> Self {
        Self
    }

    /// Nominal frequency (Hz) for a given mode type.
    pub fn nominal_frequency_hz_impl(&self, mode_type: &str) -> f64 {
        match mode_type.to_lowercase().as_str() {
            "shear" => SHEAR_MODE_HZ,
            "longitudinal" => LONGITUDINAL_MODE_HZ,
            "torsional" => TORSIONAL_MODE_HZ,
            "flexural" => FLEXURAL_MODE_HZ,
            _ => 0.0,
        }
    }

    /// Validate a single mode against the nominal frequency, loss factor, and
    /// damping-ratio requirements.
    fn validate_mode(&self, mode: &StructuralMode) -> Result<(), ExoticError> {
        let nominal = self.nominal_frequency_hz_impl(&mode.mode_type);
        if nominal <= 0.0 {
            return Err(ExoticError::AnomalyClosureError(format!(
                "unknown resonance mode type: {}",
                mode.mode_type
            )));
        }
        if (mode.frequency_hz - nominal).abs() > MODE_FREQUENCY_TOLERANCE_HZ {
            return Err(ExoticError::AnomalyClosureError(format!(
                "{} mode frequency {} Hz deviates from nominal {} Hz by more than {} Hz",
                mode.mode_type,
                mode.frequency_hz,
                nominal,
                MODE_FREQUENCY_TOLERANCE_HZ
            )));
        }
        if mode.loss_factor < MIN_LOSS_FACTOR {
            return Err(ExoticError::AnomalyClosureError(format!(
                "{} mode loss factor {} below minimum {}",
                mode.mode_type, mode.loss_factor, MIN_LOSS_FACTOR
            )));
        }
        if mode.damping_ratio < MIN_DAMPING_RATIO {
            return Err(ExoticError::AnomalyClosureError(format!(
                "{} mode damping ratio {} below minimum {}",
                mode.mode_type, mode.damping_ratio, MIN_DAMPING_RATIO
            )));
        }
        Ok(())
    }

    /// Audit a list of FEA resonance modes.
    pub fn audit_modes_impl(&self, modes: &[StructuralMode]) -> Result<String, ExoticError> {
        let mut found = [false; 4];
        for mode in modes {
            self.validate_mode(mode)?;
            match mode.mode_type.to_lowercase().as_str() {
                "shear" => found[0] = true,
                "longitudinal" => found[1] = true,
                "torsional" => found[2] = true,
                "flexural" => found[3] = true,
                _ => {}
            }
        }
        let required = ["shear", "longitudinal", "torsional", "flexural"];
        for (i, &present) in found.iter().enumerate() {
            if !present {
                return Err(ExoticError::AnomalyClosureError(format!(
                    "missing required resonance mode: {}",
                    required[i]
                )));
            }
        }
        Ok("STRUCTURAL_RESONANCE_PASS".to_string())
    }

    /// Dissipated power from a single resonance: `P_diss = 2π f_res E_elastic η`.
    pub fn dissipated_power_w_impl(&self, f_res_hz: f64, elastic_energy_j: f64, loss_factor: f64) -> f64 {
        2.0 * std::f64::consts::PI * f_res_hz * elastic_energy_j * loss_factor
    }

    /// Temperature rise for a heat pulse dumped into an InP support volume:
    /// `ΔT = P * t / (C_v * V)` with `C_v = a_InP * T^3`.
    pub fn temperature_rise_k_impl(
        &self,
        p_diss_w: f64,
        support_volume_m3: f64,
        pulse_duration_s: f64,
        baseline_temp_k: f64,
    ) -> f64 {
        if support_volume_m3 <= 0.0 || baseline_temp_k <= 0.0 {
            return f64::INFINITY;
        }
        let heat_capacity = INP_DEBYE_A_J_PER_M3_K4 * baseline_temp_k.powi(3) * support_volume_m3;
        if heat_capacity <= 0.0 {
            return f64::INFINITY;
        }
        p_diss_w * pulse_duration_s / heat_capacity
    }

    /// Thermal audit: ensure transient dissipation does not drive the airbridge
    /// support columns above the niobium quench temperature.
    pub fn audit_thermal_impl(
        &self,
        modes: &[StructuralMode],
        support_volume_m3: f64,
        pulse_duration_s: f64,
    ) -> Result<String, ExoticError> {
        for mode in modes {
            let p = self.dissipated_power_w_impl(mode.frequency_hz, mode.elastic_energy_j, mode.loss_factor);
            let delta_t = self.temperature_rise_k_impl(p, support_volume_m3, pulse_duration_s, BASELINE_TEMPERATURE_K);
            let t_final = BASELINE_TEMPERATURE_K + delta_t;
            if t_final > NIOBIUM_TRANSITION_TEMP_K {
                return Err(ExoticError::AnomalyClosureError(format!(
                    "{} mode heat accumulation drives support columns to {:.3} K, exceeding Nb quench limit {} K (P_diss = {:.6e} W)",
                    mode.mode_type,
                    t_final,
                    NIOBIUM_TRANSITION_TEMP_K,
                    p
                )));
            }
        }
        Ok("THERMAL_HOTSPOT_PASS".to_string())
    }

    /// Simple analytical natural-frequency estimates for a rectangular sapphire
    /// beam.  These are order-of-magnitude FEA sanity checks, not a full 3-D
    /// modal solve.
    pub fn estimate_natural_frequencies_hz_impl(
        &self,
        length_m: f64,
        width_m: f64,
        height_m: f64,
    ) -> Vec<(String, f64)> {
        if length_m <= 0.0 || width_m <= 0.0 || height_m <= 0.0 {
            return Vec::new();
        }
        let area = width_m * height_m;
        let rho = SAPPHIRE_DENSITY_KG_M3;
        let e = SAPPHIRE_YOUNG_MODULUS_PA;
        let g = SAPPHIRE_SHEAR_MODULUS_PA;

        // Longitudinal rod mode: c = sqrt(E/ρ), f = c/(2L).
        let f_long = (e / rho).sqrt() / (2.0 * length_m);

        // Shear wave mode: c = sqrt(G/ρ), f = c/(2L).
        let f_shear = (g / rho).sqrt() / (2.0 * length_m);

        // First flexural mode of a free-free beam (β1 ≈ 4.73).
        let iyy = width_m * height_m.powi(3) / 12.0;
        let beta1: f64 = 4.73;
        let f_flex = beta1.powi(2) / (2.0 * std::f64::consts::PI * length_m.powi(2))
            * (e * iyy / (rho * area)).sqrt();

        // First torsional mode: f = (1/(2L)) sqrt(G * J / (ρ * I_p)).
        // Approximate torsion constant for rectangular cross-section.
        let ratio = width_m.max(height_m) / width_m.min(height_m);
        let alpha = if ratio > 1.0 {
            0.3333 - 0.21 * (1.0 - 1.0 / ratio) / ratio
        } else {
            0.3333
        };
        let j = alpha * width_m * height_m.powi(3);
        let ip = area * (width_m.powi(2) + height_m.powi(2)) / 12.0;
        let f_tors = (g * j / (rho * ip)).sqrt() / (2.0 * length_m);

        vec![
            ("shear".to_string(), f_shear),
            ("longitudinal".to_string(), f_long),
            ("torsional".to_string(), f_tors),
            ("flexural".to_string(), f_flex),
        ]
    }
}

#[pymethods]
impl HarmonicAuditor {
    #[new]
    fn py_new() -> Self {
        Self::new()
    }

    /// Nominal frequency for a mode type (`"shear"`, `"longitudinal"`, `"torsional"`, `"flexural"`).
    fn nominal_frequency_hz(&self, mode_type: String) -> f64 {
        self.nominal_frequency_hz_impl(&mode_type)
    }

    /// Audit a list of resonance modes supplied as tuples:
    /// `(mode_type, frequency_hz, loss_factor, damping_ratio, elastic_energy_j)`.
    ///
    /// Returns `"STRUCTURAL_RESONANCE_PASS"` if all four required modes are
    /// present, within frequency tolerance, and meet damping thresholds.
    fn audit_modes(&self, modes: Vec<(String, f64, f64, f64, f64)>) -> PyResult<String> {
        let parsed: Vec<StructuralMode> = modes
            .into_iter()
            .map(|(t, f, eta, zeta, e)| StructuralMode {
                mode_type: t,
                frequency_hz: f,
                loss_factor: eta,
                damping_ratio: zeta,
                elastic_energy_j: e,
            })
            .collect();
        self.audit_modes_impl(&parsed).map_err(PyErr::from)
    }

    /// Dissipated power `P_diss = 2π f_res E_elastic η` (W).
    fn dissipated_power_w(&self, f_res_hz: f64, elastic_energy_j: f64, loss_factor: f64) -> f64 {
        self.dissipated_power_w_impl(f_res_hz, elastic_energy_j, loss_factor)
    }

    /// Temperature rise (K) for a pulse dumped into an InP support volume.
    fn temperature_rise_k(
        &self,
        p_diss_w: f64,
        support_volume_m3: f64,
        pulse_duration_s: f64,
        baseline_temp_k: f64,
    ) -> f64 {
        self.temperature_rise_k_impl(p_diss_w, support_volume_m3, pulse_duration_s, baseline_temp_k)
    }

    /// Thermal audit: ensure no mode drives support temperature above 9.3 K.
    ///
    /// `modes` has the same tuple form as `audit_modes`.  `support_volume_m3`
    /// is the total InP support-column volume and `pulse_duration_s` is the
    /// transient width.
    fn audit_thermal(
        &self,
        modes: Vec<(String, f64, f64, f64, f64)>,
        support_volume_m3: f64,
        pulse_duration_s: f64,
    ) -> PyResult<String> {
        let parsed: Vec<StructuralMode> = modes
            .into_iter()
            .map(|(t, f, eta, zeta, e)| StructuralMode {
                mode_type: t,
                frequency_hz: f,
                loss_factor: eta,
                damping_ratio: zeta,
                elastic_energy_j: e,
            })
            .collect();
        self.audit_thermal_impl(&parsed, support_volume_m3, pulse_duration_s)
            .map_err(PyErr::from)
    }

    /// Analytical natural-frequency estimates for a rectangular sapphire beam.
    ///
    /// Returns list of `(mode_type, frequency_hz)` estimates.
    fn estimate_natural_frequencies_hz(
        &self,
        length_m: f64,
        width_m: f64,
        height_m: f64,
    ) -> Vec<(String, f64)> {
        self.estimate_natural_frequencies_hz_impl(length_m, width_m, height_m)
    }

    /// Convenience: full structural + thermal audit of the STEP waveguide.
    ///
    /// `support_volume_m3` is the total InP support-column volume.
    fn audit_waveguide(
        &self,
        modes: Vec<(String, f64, f64, f64, f64)>,
        support_volume_m3: f64,
    ) -> PyResult<String> {
        let parsed: Vec<StructuralMode> = modes
            .into_iter()
            .map(|(t, f, eta, zeta, e)| StructuralMode {
                mode_type: t,
                frequency_hz: f,
                loss_factor: eta,
                damping_ratio: zeta,
                elastic_energy_j: e,
            })
            .collect();
        self.audit_modes_impl(&parsed)?;
        self.audit_thermal_impl(&parsed, support_volume_m3, TRANSIENT_WINDOW_S)?;
        Ok("HARMONIC_AUDIT_PASS".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn nominal_modes() -> Vec<StructuralMode> {
        vec![
            StructuralMode {
                mode_type: "shear".to_string(),
                frequency_hz: SHEAR_MODE_HZ,
                loss_factor: MIN_LOSS_FACTOR,
                damping_ratio: MIN_DAMPING_RATIO,
                elastic_energy_j: 1.0e-9,
            },
            StructuralMode {
                mode_type: "longitudinal".to_string(),
                frequency_hz: LONGITUDINAL_MODE_HZ,
                loss_factor: MIN_LOSS_FACTOR,
                damping_ratio: MIN_DAMPING_RATIO,
                elastic_energy_j: 1.0e-9,
            },
            StructuralMode {
                mode_type: "torsional".to_string(),
                frequency_hz: TORSIONAL_MODE_HZ,
                loss_factor: MIN_LOSS_FACTOR,
                damping_ratio: MIN_DAMPING_RATIO,
                elastic_energy_j: 1.0e-9,
            },
            StructuralMode {
                mode_type: "flexural".to_string(),
                frequency_hz: FLEXURAL_MODE_HZ,
                loss_factor: MIN_LOSS_FACTOR,
                damping_ratio: MIN_DAMPING_RATIO,
                elastic_energy_j: 1.0e-9,
            },
        ]
    }

    #[test]
    fn audit_passes_for_nominal_modes() {
        let auditor = HarmonicAuditor::new();
        let modes = nominal_modes();
        assert_eq!(auditor.audit_modes_impl(&modes).unwrap(), "STRUCTURAL_RESONANCE_PASS");
    }

    #[test]
    fn audit_fails_for_low_damping() {
        let auditor = HarmonicAuditor::new();
        let mut modes = nominal_modes();
        modes[0].damping_ratio = 1.0e-5;
        assert!(auditor.audit_modes_impl(&modes).is_err());
    }

    #[test]
    fn audit_fails_for_low_loss_factor() {
        let auditor = HarmonicAuditor::new();
        let mut modes = nominal_modes();
        modes[3].loss_factor = 1.0e-4;
        assert!(auditor.audit_modes_impl(&modes).is_err());
    }

    #[test]
    fn audit_fails_for_frequency_deviation() {
        let auditor = HarmonicAuditor::new();
        let mut modes = nominal_modes();
        modes[1].frequency_hz += 1.0e6;
        assert!(auditor.audit_modes_impl(&modes).is_err());
    }

    #[test]
    fn thermal_passes_for_small_elastic_energy() {
        let auditor = HarmonicAuditor::new();
        let modes = nominal_modes();
        // With 1e-9 J elastic energy and support volume 1 cm^3, rise is tiny.
        assert_eq!(
            auditor.audit_thermal_impl(&modes, 1.0e-6, TRANSIENT_WINDOW_S).unwrap(),
            "THERMAL_HOTSPOT_PASS"
        );
    }

    #[test]
    fn thermal_fails_for_excessive_energy() {
        let auditor = HarmonicAuditor::new();
        let mut modes = nominal_modes();
        modes[3].elastic_energy_j = 1.0e3; // Flexural energy large enough to quench.
        assert!(auditor.audit_thermal_impl(&modes, 1.0e-6, TRANSIENT_WINDOW_S).is_err());
    }

    #[test]
    fn dissipated_power_formula_matches_spec() {
        let auditor = HarmonicAuditor::new();
        let f = 10.0e6;
        let e = 1.0e-6;
        let eta = 1.0e-3;
        let p = auditor.dissipated_power_w(f, e, eta);
        let expected = 2.0 * std::f64::consts::PI * f * e * eta;
        assert!((p - expected).abs() < 1e-20);
    }

    #[test]
    fn natural_frequency_estimates_are_finite_for_reasonable_beam() {
        let auditor = HarmonicAuditor::new();
        // 1 cm long, 1 mm x 0.5 mm sapphire beam.
        let freqs = auditor.estimate_natural_frequencies_hz(0.01, 1.0e-3, 0.5e-3);
        assert_eq!(freqs.len(), 4);
        for (_, f) in freqs {
            assert!(f.is_finite());
            assert!(f > 0.0);
        }
    }
}
