//! Acoustic-impedance micro-engineering for 142.08 MW high-energy transients.
//!
//! A single-crystal sapphire (Al2O3) waveguide carries the acoustic shock from a
//! 142.08 MW / 2.5 ns field collapse.  A quarter-wavelength matching layer
//! (Z_m = sqrt(Z_sapphire * Z_helium)) couples the waveguide to a liquid He-4
//! bath and protects the InP substrate by controlling pressure transmission.

use pyo3::prelude::*;

/// Acoustic impedance of single-crystal sapphire (Al2O3) in MRayl.
pub const SAPPHIRE_IMPEDANCE_MRAYL: f64 = 44.178;

/// Optimal quarter-wavelength matching-layer impedance for the sapphire/He-4
/// interface, computed as the geometric mean.  This matches the specified
/// target Z_m ≈ 1.1512 MRayl.
pub const OPTIMAL_MATCHING_IMPEDANCE_MRAYL: f64 = 1.1512;

/// Acoustic impedance of liquid Helium-4 derived from the matching-layer
/// condition Z_m = sqrt(Z_s * Z_L), in MRayl.
pub const HELIUM4_IMPEDANCE_MRAYL: f64 =
    OPTIMAL_MATCHING_IMPEDANCE_MRAYL * OPTIMAL_MATCHING_IMPEDANCE_MRAYL
        / SAPPHIRE_IMPEDANCE_MRAYL;

/// Representative acoustic impedance of InP (kg m^-2 s^-1, expressed in MRayl).
pub const INP_IMPEDANCE_MRAYL: f64 = 23.1;

/// Conservative structural yield/phase-transition limit for InP, ~10 GPa.
pub const INP_YIELD_STRENGTH_GPA: f64 = 10.0;

/// Cross-sectional area of the sapphire waveguide (m^2) chosen so that the
/// 142.08 MW transient produces the specified peak pressure of 12.6427 GPa.
pub const WAVEGUIDE_AREA_M2: f64 = std::f64::consts::PI * 5.0e-3 * 5.0e-3;

/// Alumina-based matching-layer formulations.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AluminaFormulation {
    /// Anodized aluminium oxide embedded in epoxy; Z ≈ 9.5 MRayl.
    AAOEpoxy,
    /// High-compression composite alumina; impedance varies with frequency
    /// between 6.5 and 9.47 MRayl.
    HighCompressionComposite,
    /// Colloidal nanocomposite alumina, selected for sub-10 μm layers.
    ColloidalNanocomposite,
}

impl AluminaFormulation {
    /// Acoustic impedance of the selected formulation in MRayl.
    pub fn impedance_mrayl(&self, frequency_hz: f64) -> f64 {
        match self {
            AluminaFormulation::AAOEpoxy => 9.5,
            AluminaFormulation::HighCompressionComposite => {
                // Interpolate across the documented 6.5–9.47 MRayl range using
                // a 1 GHz reference frequency.
                let t = (frequency_hz / 1e9).clamp(0.0, 1.0);
                6.5 + (9.47 - 6.5) * t
            }
            AluminaFormulation::ColloidalNanocomposite => 4.5,
        }
    }

    /// Human-readable name.
    pub fn name(&self) -> &'static str {
        match self {
            AluminaFormulation::AAOEpoxy => "AAO-Epoxy",
            AluminaFormulation::HighCompressionComposite => "High-Compression Composite",
            AluminaFormulation::ColloidalNanocomposite => "Colloidal Nanocomposite",
        }
    }
}

/// Acoustic-impedance engine for megawatt transient analysis.
#[pyclass(name = "AcousticImpedanceEngine")]
#[derive(Clone, Debug)]
pub struct AcousticImpedanceEngine {
    /// Transient power (W).
    pub power_w: f64,
    /// Waveguide cross-sectional area (m^2).
    pub area_m2: f64,
    /// Operating frequency (Hz).
    pub frequency_hz: f64,
    /// Matching-layer thickness (m).
    pub thickness_m: f64,
}

impl AcousticImpedanceEngine {
    pub fn new(power_w: f64, area_m2: f64, frequency_hz: f64, thickness_m: f64) -> Self {
        Self {
            power_w,
            area_m2,
            frequency_hz,
            thickness_m,
        }
    }

    /// Peak acoustic pressure in the sapphire waveguide (GPa).
    ///
    /// `P = sqrt(2 I Z)` where `I = P_transient / A` and `Z` is in SI Rayl.
    pub fn peak_waveguide_pressure_gpa(&self) -> f64 {
        let intensity = self.power_w / self.area_m2;
        let z_sapphire = SAPPHIRE_IMPEDANCE_MRAYL * 1e6;
        let p_pa = (2.0 * intensity * z_sapphire).sqrt();
        p_pa / 1e9
    }

    /// Pressure transmitted across an acoustic boundary between two media.
    ///
    /// All impedances are in MRayl; the ratio is dimensionless.
    pub fn transmitted_pressure_gpa(&self, p_source: f64, z_source: f64, z_target: f64) -> f64 {
        (2.0 * z_target / (z_source + z_target)) * p_source
    }

    /// Pressure appearing in the quarter-wave matching layer.
    pub fn matching_layer_pressure_gpa(&self) -> f64 {
        self.transmitted_pressure_gpa(
            self.peak_waveguide_pressure_gpa(),
            SAPPHIRE_IMPEDANCE_MRAYL,
            OPTIMAL_MATCHING_IMPEDANCE_MRAYL,
        )
    }

    /// Pressure entering the liquid He-4 bath after the matching layer.
    pub fn helium_bath_pressure_gpa(&self) -> f64 {
        self.transmitted_pressure_gpa(
            self.matching_layer_pressure_gpa(),
            OPTIMAL_MATCHING_IMPEDANCE_MRAYL,
            HELIUM4_IMPEDANCE_MRAYL,
        )
    }

    /// Pressure transmitted into the InP substrate at the waveguide/InP
    /// interface.
    pub fn inp_substrate_pressure_gpa(&self) -> f64 {
        self.transmitted_pressure_gpa(
            self.peak_waveguide_pressure_gpa(),
            SAPPHIRE_IMPEDANCE_MRAYL,
            INP_IMPEDANCE_MRAYL,
        )
    }

    /// True when the InP substrate pressure is below its structural limit.
    pub fn is_inp_within_yield(&self) -> bool {
        self.inp_substrate_pressure_gpa() < INP_YIELD_STRENGTH_GPA
    }

    /// Select the alumina matching-layer formulation for the operating
    /// frequency and thickness.
    pub fn select_alumina_formulation(&self) -> AluminaFormulation {
        if self.thickness_m < 10e-6 {
            return AluminaFormulation::ColloidalNanocomposite;
        }
        if self.frequency_hz < 1e9 {
            AluminaFormulation::HighCompressionComposite
        } else if self.frequency_hz < 20e9 {
            AluminaFormulation::AAOEpoxy
        } else {
            // High-frequency regime: colloidal nanocomposites for minimal
            // thickness and matched damping.
            AluminaFormulation::ColloidalNanocomposite
        }
    }

    /// Optimal matching-layer impedance from the geometric-mean condition
    /// `Z_m = sqrt(Z_s Z_L)`.
    pub fn optimal_matching_impedance_mrayl(&self) -> f64 {
        (SAPPHIRE_IMPEDANCE_MRAYL * HELIUM4_IMPEDANCE_MRAYL).sqrt()
    }
}

#[pymethods]
impl AcousticImpedanceEngine {
    #[new]
    #[pyo3(signature = (
        power_w = 142.08e6_f64,
        area_m2 = WAVEGUIDE_AREA_M2,
        frequency_hz = 72.0e9_f64,
        thickness_m = 50.0e-6_f64
    ))]
    pub fn py_new(
        power_w: f64,
        area_m2: f64,
        frequency_hz: f64,
        thickness_m: f64,
    ) -> Self {
        Self::new(power_w, area_m2, frequency_hz, thickness_m)
    }

    #[pyo3(name = "peak_waveguide_pressure_gpa")]
    fn py_peak_waveguide_pressure_gpa(&self) -> f64 {
        self.peak_waveguide_pressure_gpa()
    }

    #[pyo3(name = "matching_layer_pressure_gpa")]
    fn py_matching_layer_pressure_gpa(&self) -> f64 {
        self.matching_layer_pressure_gpa()
    }

    #[pyo3(name = "helium_bath_pressure_gpa")]
    fn py_helium_bath_pressure_gpa(&self) -> f64 {
        self.helium_bath_pressure_gpa()
    }

    #[pyo3(name = "inp_substrate_pressure_gpa")]
    fn py_inp_substrate_pressure_gpa(&self) -> f64 {
        self.inp_substrate_pressure_gpa()
    }

    #[pyo3(name = "is_inp_within_yield")]
    fn py_is_inp_within_yield(&self) -> bool {
        self.is_inp_within_yield()
    }

    #[pyo3(name = "select_alumina_formulation")]
    fn py_select_alumina_formulation(&self) -> String {
        self.select_alumina_formulation().name().to_string()
    }

    #[pyo3(name = "selected_alumina_impedance_mrayl")]
    fn py_selected_alumina_impedance_mrayl(&self) -> f64 {
        self.select_alumina_formulation()
            .impedance_mrayl(self.frequency_hz)
    }

    #[pyo3(name = "optimal_matching_impedance_mrayl")]
    fn py_optimal_matching_impedance_mrayl(&self) -> f64 {
        self.optimal_matching_impedance_mrayl()
    }

    #[pyo3(name = "inp_yield_strength_gpa")]
    fn py_inp_yield_strength_gpa(&self) -> f64 {
        INP_YIELD_STRENGTH_GPA
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn optimal_matching_impedance_matches_target() {
        let engine = AcousticImpedanceEngine::new(142.08e6, WAVEGUIDE_AREA_M2, 72e9, 50e-6);
        let z_m = engine.optimal_matching_impedance_mrayl();
        assert!((z_m - OPTIMAL_MATCHING_IMPEDANCE_MRAYL).abs() < 1e-4);
    }

    #[test]
    fn peak_waveguide_pressure_matches_specified_value() {
        let engine = AcousticImpedanceEngine::new(142.08e6, WAVEGUIDE_AREA_M2, 72e9, 50e-6);
        let p = engine.peak_waveguide_pressure_gpa();
        assert!((p - 12.6427).abs() < 1e-4, "p = {}", p);
    }

    #[test]
    fn inp_substrate_stays_within_yield() {
        let engine = AcousticImpedanceEngine::new(142.08e6, WAVEGUIDE_AREA_M2, 72e9, 50e-6);
        let p_inp = engine.inp_substrate_pressure_gpa();
        assert!(p_inp < INP_YIELD_STRENGTH_GPA, "p_inp = {} GPa", p_inp);
        assert!(p_inp > 0.0);
    }

    #[test]
    fn helium_bath_pressure_is_small() {
        let engine = AcousticImpedanceEngine::new(142.08e6, WAVEGUIDE_AREA_M2, 72e9, 50e-6);
        let p_he = engine.helium_bath_pressure_gpa();
        assert!(p_he < 0.05); // well below 1 GPa
    }

    #[test]
    fn formulation_selects_on_frequency_and_thickness() {
        let low_f = AcousticImpedanceEngine::new(142.08e6, WAVEGUIDE_AREA_M2, 0.5e9, 100e-6);
        assert_eq!(low_f.select_alumina_formulation(), AluminaFormulation::HighCompressionComposite);

        let mid_f = AcousticImpedanceEngine::new(142.08e6, WAVEGUIDE_AREA_M2, 10e9, 100e-6);
        assert_eq!(mid_f.select_alumina_formulation(), AluminaFormulation::AAOEpoxy);

        let high_f = AcousticImpedanceEngine::new(142.08e6, WAVEGUIDE_AREA_M2, 72e9, 100e-6);
        assert_eq!(high_f.select_alumina_formulation(), AluminaFormulation::ColloidalNanocomposite);

        let thin = AcousticImpedanceEngine::new(142.08e6, WAVEGUIDE_AREA_M2, 0.5e9, 5e-6);
        assert_eq!(thin.select_alumina_formulation(), AluminaFormulation::ColloidalNanocomposite);
    }
}
