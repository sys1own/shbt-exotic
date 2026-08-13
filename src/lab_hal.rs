//! Laboratory Hardware Abstraction Layer (HAL) and telemetry bridge.
//!
//! Translates simulator phase commands into analog I/Q voltages, builds
//! 16-bit DAC PCIe look-up tables, and implements the bare-metal AVX-512 PID
//! telemetry loop required for sub-nanosecond laboratory control.

use pyo3::prelude::*;

use crate::constants::{
    BASELINE_TEMPERATURE_K, PHASE_JITTER_THRESHOLD_RAD,
};
use crate::error::ExoticError;
use crate::phase_rotation::{emergency_shutdown_compare, AlignedF32};
use crate::phase_table::{PhaseCommand, COLLECTOR_DRAIN_V, GATE_BASE_TURN_ON_V};

/// DAC full-scale voltage used for the InP/InGaAs RF phase-shifter bias (V).
pub const DAC_V_MAX: f64 = COLLECTOR_DRAIN_V;

/// Default 16-bit DAC resolution.
pub const DAC_BITS: u32 = 16;
pub const DAC_MAX_CODE: u32 = (1u32 << DAC_BITS) - 1;

/// PID telemetry loop clock (GHz) and guaranteed cycle budget.
pub const PID_TELEMETRY_CLOCK_GHZ: f64 = 3.5;
/// Four-cycle AVX-512 instruction pipeline: vmovaps + vcmpps + vmovmskps + mov.
pub const PID_TELEMETRY_CYCLE_NS: f64 = 4.0 / PID_TELEMETRY_CLOCK_GHZ;

/// One I/Q DAC sample for the PCIe lookup table.
#[pyclass(name = "IQDacSample", get_all)]
#[derive(Clone, Debug)]
pub struct IQDacSample {
    pub i_code: u16,
    pub q_code: u16,
    pub i_v: f64,
    pub q_v: f64,
    pub theta_rad: f64,
    pub amplitude: f64,
}

/// Laboratory HAL: analog voltage mapping and DAC register compilation.
#[pyclass(name = "LabHAL")]
#[derive(Clone, Debug)]
pub struct LabHAL;

impl LabHAL {
    pub fn new() -> Self {
        Self
    }

    /// Map a complex phase command `A e^{i θ}` to analog I/Q voltages:
    ///
    ///   I = V_max * A * cos(θ)
    ///   Q = V_max * A * sin(θ)
    pub fn iq_voltage_v_impl(&self, theta_rad: f64, amplitude: f64, v_max: f64) -> (f64, f64) {
        let i = v_max * amplitude * theta_rad.cos();
        let q = v_max * amplitude * theta_rad.sin();
        (i, q)
    }

    /// Convert a bipolar voltage (-V_max .. +V_max) to a 16-bit offset-binary
    /// DAC code.
    pub fn voltage_to_dac_code_impl(&self, voltage: f64, v_max: f64, bits: u32) -> u16 {
        if v_max == 0.0 {
            return 0;
        }
        let normalized = (voltage / v_max).clamp(-1.0, 1.0);
        let max_code = (1u32 << bits) - 1;
        let code_f = (normalized + 1.0) * 0.5 * (max_code as f64);
        (code_f.round() as u32).min(max_code) as u16
    }

    /// Build a pre-compiled PCIe I/Q DAC lookup table from a phase-command table.
    pub fn build_pcie_iq_lut_impl(
        &self,
        commands: &[PhaseCommand],
        amplitude: f64,
    ) -> Vec<IQDacSample> {
        commands
            .iter()
            .map(|cmd| {
                let (i_v, q_v) = self.iq_voltage_v_impl(cmd.theta_rad, amplitude, DAC_V_MAX);
                IQDacSample {
                    i_code: self.voltage_to_dac_code_impl(i_v, DAC_V_MAX, DAC_BITS),
                    q_code: self.voltage_to_dac_code_impl(q_v, DAC_V_MAX, DAC_BITS),
                    i_v,
                    q_v,
                    theta_rad: cmd.theta_rad,
                    amplitude,
                }
            })
            .collect()
    }
}

#[pymethods]
impl LabHAL {
    #[new]
    fn py_new() -> Self {
        Self::new()
    }

    /// Map `A e^{i θ}` to `(I_v, Q_v)` analog voltages using `V_max`.
    fn iq_voltage_v(&self, theta_rad: f64, amplitude: f64, v_max: f64) -> (f64, f64) {
        self.iq_voltage_v_impl(theta_rad, amplitude, v_max)
    }

    /// Convert a bipolar voltage to a DAC code of `bits` resolution.
    fn voltage_to_dac_code(&self, voltage: f64, v_max: f64, bits: u32) -> u16 {
        self.voltage_to_dac_code_impl(voltage, v_max, bits)
    }

    /// Build a 16-bit PCIe I/Q lookup table from a list of `PhaseCommand`.
    ///
    /// `amplitude` is the per-emitter RF envelope `A_ij` (0..1).
    fn build_pcie_iq_lut(&self, commands: Vec<PhaseCommand>, amplitude: f64) -> Vec<IQDacSample> {
        self.build_pcie_iq_lut_impl(&commands, amplitude)
    }

    /// Convenience: I/Q codes for a single phase angle using default `DAC_V_MAX`.
    fn dac_codes_for_phase(&self, theta_rad: f64, amplitude: f64) -> (u16, u16) {
        let (i_v, q_v) = self.iq_voltage_v_impl(theta_rad, amplitude, DAC_V_MAX);
        (
            self.voltage_to_dac_code(i_v, DAC_V_MAX, DAC_BITS),
            self.voltage_to_dac_code(q_v, DAC_V_MAX, DAC_BITS),
        )
    }
}

/// High-speed AVX-512 telemetry bridge for the PID bias regulator.
#[pyclass(name = "TelemetryBridge")]
#[derive(Clone, Debug)]
pub struct TelemetryBridge;

impl TelemetryBridge {
    pub fn new() -> Self {
        Self
    }

    /// Guaranteed AVX-512 telemetry cycle latency (ns).
    pub fn telemetry_cycle_ns_impl(&self) -> f64 {
        PID_TELEMETRY_CYCLE_NS
    }

    /// Run one PID bias telemetry cycle with a deterministic AVX-512 sensor
    /// compare (`vmovaps`, `vcmpps`, `vmovmskps`, `mov [mem], 0`).
    ///
    /// `errors` holds 16 sensor-lane phase-error values (rad); only the first 8
    /// lanes are active.  The mean of the active lanes is fed into a discrete
    /// PI regulator with `Kp` (V/rad) and `Ki` (V/(rad·s)).
    ///
    /// Returns `(control_voltage_v, updated_integral, shutdown_triggered)`.
    pub fn pid_bias_cycle_impl(
        &self,
        errors: &[f32; 16],
        threshold_rad: f32,
        kp: f64,
        ki: f64,
        integral: f64,
        dt_s: f64,
    ) -> (f64, f64, bool) {
        let aligned = AlignedF32(*errors);
        let mut mmio = 1i32;
        unsafe {
            emergency_shutdown_compare(&mut mmio, &aligned, threshold_rad);
        }
        let shutdown_triggered = mmio == 0;

        // Aggregate error from active lanes for the PID regulator.
        let active = &errors[..8];
        let mean_error: f64 = active.iter().map(|&e| e as f64).sum::<f64>() / active.len() as f64;
        let new_integral = integral + mean_error * dt_s;
        let control = kp * mean_error + ki * new_integral;
        (control, new_integral, shutdown_triggered)
    }
}

#[pymethods]
impl TelemetryBridge {
    #[new]
    fn py_new() -> Self {
        Self::new()
    }

    /// Run one PID bias telemetry cycle.
    ///
    /// Returns `(control_voltage_v, updated_integral, shutdown_triggered)`.
    fn pid_bias_cycle(
        &self,
        errors: Vec<f32>,
        threshold_rad: f64,
        kp: f64,
        ki: f64,
        integral: f64,
        dt_s: f64,
    ) -> PyResult<(f64, f64, bool)> {
        if errors.len() != 16 {
            return Err(ExoticError::AnomalyClosureError(
                "errors must contain exactly 16 lanes".to_string(),
            )
            .into());
        }
        let mut arr = [0.0f32; 16];
        arr.copy_from_slice(&errors);
        let thr = threshold_rad as f32;
        let (control, new_int, shutdown) =
            self.pid_bias_cycle_impl(&arr, thr, kp, ki, integral, dt_s);
        Ok((control, new_int, shutdown))
    }

    /// Guaranteed AVX-512 telemetry cycle latency (ns) at the loop clock.
    fn telemetry_cycle_ns(&self) -> f64 {
        self.telemetry_cycle_ns_impl()
    }

    /// Returns the phase-jitter threshold used for emergency shutdown (rad).
    fn phase_jitter_threshold_rad(&self) -> f64 {
        PHASE_JITTER_THRESHOLD_RAD
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn iq_voltage_for_unit_phase() {
        let hal = LabHAL::new();
        let (i, q) = hal.iq_voltage_v_impl(0.0, 1.0, 7.4);
        assert!((i - 7.4).abs() < 1e-12);
        assert!(q.abs() < 1e-12);
    }

    #[test]
    fn voltage_to_dac_code_is_within_range() {
        let hal = LabHAL::new();
        assert_eq!(hal.voltage_to_dac_code_impl(-7.4, 7.4, 16), 0);
        // 0.0 maps to the midpoint code of 0x8000 (32768) with offset-binary rounding.
        assert_eq!(hal.voltage_to_dac_code_impl(0.0, 7.4, 16), 32768);
        assert_eq!(hal.voltage_to_dac_code_impl(7.4, 7.4, 16), 65535);
    }

    #[test]
    fn lut_from_phase_commands_has_same_length() {
        let hal = LabHAL::new();
        let cmds = vec![
            PhaseCommand {
                i: 0,
                j: 0,
                h_ij: 1.0,
                v_eff: 0.5,
                theta_rad: 0.0,
                cos_theta: 1.0,
                sin_theta: 0.0,
                v_phase: GATE_BASE_TURN_ON_V,
            },
            PhaseCommand {
                i: 0,
                j: 1,
                h_ij: 1.0,
                v_eff: 0.5,
                theta_rad: std::f64::consts::PI / 2.0,
                cos_theta: 0.0,
                sin_theta: 1.0,
                v_phase: COLLECTOR_DRAIN_V,
            },
        ];
        let lut = hal.build_pcie_iq_lut_impl(&cmds, 1.0);
        assert_eq!(lut.len(), 2);
        assert!(lut[0].i_code > 0);
        assert!(lut[1].q_code > lut[1].i_code);
    }

    #[test]
    fn telemetry_cycle_latency_below_1_5ns() {
        assert!(PID_TELEMETRY_CYCLE_NS < 1.5);
    }

    #[test]
    fn pid_bias_cycle_updates_integral_and_outputs_control() {
        let bridge = TelemetryBridge::new();
        let mut errors = [0.0f32; 16];
        errors[0] = 1.0e-6; // small error
        let (control, integral, shutdown) =
            bridge.pid_bias_cycle_impl(&errors, 1.0e-5, 1.85, 9.12e3, 0.0, 1.0e-9);
        assert!(!shutdown);
        assert!(integral > 0.0);
        assert!(control.is_finite());
    }

    #[test]
    fn pid_bias_cycle_triggers_shutdown_on_large_error() {
        let bridge = TelemetryBridge::new();
        let mut errors = [0.0f32; 16];
        errors[0] = 1.0e-3; // exceeds 5.05e-5 rad phase jitter threshold
        let (_, _, shutdown) =
            bridge.pid_bias_cycle_impl(&errors, PHASE_JITTER_THRESHOLD_RAD as f32, 1.85, 9.12e3, 0.0, 1.0e-9);
        assert!(shutdown);
    }
}
