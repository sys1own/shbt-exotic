//! Closed-loop hardware calibration and PID bias regulator for the InP/InGaAs
//! SHBT emitter array.
//!
//! Synthesises the on-chip calibration waveform
//!     V_cal(t) = 3.3 V + 50 mV * sin(2π * 10 MHz * t + δφ(t))
//! and runs a discrete PID regulator on the 3.3 V base bias to hold phase
//! jitter |δφ| below the 5.05×10⁻⁵ rad HIL limit.

use pyo3::prelude::*;

/// Base calibration bias (V).
pub const CAL_BASE_VOLTAGE_V: f64 = 3.3;
/// AC amplitude of the calibration tone (V).
pub const CAL_AMPLITUDE_V: f64 = 0.05;
/// Calibration tone frequency (Hz).
pub const CAL_FREQUENCY_HZ: f64 = 10.0e6;
/// HIL phase-jitter threshold (rad).
pub const PHASE_JITTER_LIMIT_RAD: f64 = 5.05e-5;

/// PID proportional gain (V/rad).
pub const PID_KP: f64 = 1.85;
/// PID integral gain (V/(rad·s)).
pub const PID_KI: f64 = 9.12e3;
/// PID derivative gain (V·s/rad).
pub const PID_KD: f64 = 3.45e-7;

/// Maximum control voltage correction (V).
const OUTPUT_CLAMP_V: f64 = 0.5;

/// Calibration engine with a discrete PID regulator.
#[pyclass(name = "CalibrationEngine")]
#[derive(Clone, Debug)]
pub struct CalibrationEngine {
    integral: f64,
    prev_error: f64,
    first_step: bool,
    setpoint: f64,
    kp: f64,
    ki: f64,
    kd: f64,
    jitter_limit: f64,
}

impl CalibrationEngine {
    pub fn new() -> Self {
        Self {
            integral: 0.0,
            prev_error: 0.0,
            first_step: true,
            setpoint: 0.0,
            kp: PID_KP,
            ki: PID_KI,
            kd: PID_KD,
            jitter_limit: PHASE_JITTER_LIMIT_RAD,
        }
    }

    /// Calibration tone V_cal(t) for a given residual phase jitter.
    pub fn calibration_waveform_impl(&self, t: f64, phase_jitter_rad: f64) -> f64 {
        let argument = 2.0 * std::f64::consts::PI * CAL_FREQUENCY_HZ * t + phase_jitter_rad;
        CAL_BASE_VOLTAGE_V + CAL_AMPLITUDE_V * argument.sin()
    }

    /// PID output voltage for a single control cycle.
    pub fn pid_output_impl(&mut self, dt: f64, measured_phase_rad: f64) -> f64 {
        let error = measured_phase_rad - self.setpoint;
        self.integral += error * dt;

        let derivative = if self.first_step {
            0.0
        } else {
            (error - self.prev_error) / dt
        };

        self.prev_error = error;
        self.first_step = false;

        let mut output = self.kp * error + self.ki * self.integral + self.kd * derivative;
        if output > OUTPUT_CLAMP_V {
            output = OUTPUT_CLAMP_V;
        } else if output < -OUTPUT_CLAMP_V {
            output = -OUTPUT_CLAMP_V;
        }
        output
    }

    /// One closed-loop calibration step.
    ///
    /// Returns `(bias_voltage_v, corrected_phase_rad, status)` where
    /// `status` is `STATUS_NOMINAL_PASS` when the corrected jitter lies below
    /// the HIL limit and `STATUS_EMERGENCY_SHUTDOWN` otherwise.
    pub fn step_impl(&mut self, dt: f64, measured_phase_rad: f64) -> (f64, f64, String) {
        let control_v = self.pid_output_impl(dt, measured_phase_rad);
        // Convert control voltage back to an equivalent phase correction (rad)
        // using the proportional gain as the plant conversion factor.
        let phase_correction_rad = if self.kp != 0.0 {
            control_v / self.kp
        } else {
            0.0
        };
        let corrected_phase = measured_phase_rad - phase_correction_rad;
        let bias_voltage = CAL_BASE_VOLTAGE_V + control_v;
        let status = if corrected_phase.abs() <= self.jitter_limit {
            "STATUS_NOMINAL_PASS".to_string()
        } else {
            "STATUS_EMERGENCY_SHUTDOWN".to_string()
        };
        (bias_voltage, corrected_phase, status)
    }

    /// Reset the PID state.
    pub fn reset_impl(&mut self) {
        self.integral = 0.0;
        self.prev_error = 0.0;
        self.first_step = true;
    }
}

#[pymethods]
impl CalibrationEngine {
    #[new]
    fn py_new() -> Self {
        Self::new()
    }

    /// Compute the calibration waveform voltage at time `t` (s).
    fn calibration_waveform(&self, t: f64, phase_jitter_rad: f64) -> f64 {
        self.calibration_waveform_impl(t, phase_jitter_rad)
    }

    /// Run one PID control step and return `(bias_voltage, corrected_phase, status)`.
    fn step(&mut self, dt: f64, measured_phase_rad: f64) -> (f64, f64, String) {
        self.step_impl(dt, measured_phase_rad)
    }

    /// Return the raw PID control voltage for the latest step.
    fn pid_output(&mut self, dt: f64, measured_phase_rad: f64) -> f64 {
        self.pid_output_impl(dt, measured_phase_rad)
    }

    /// Reset integrator and previous error.
    fn reset(&mut self) {
        self.reset_impl();
    }

    /// Expose the PID gains as `(kp, ki, kd)`.
    fn pid_gains(&self) -> (f64, f64, f64) {
        (self.kp, self.ki, self.kd)
    }

    /// HIL phase-jitter limit (rad).
    fn phase_jitter_limit_rad(&self) -> f64 {
        self.jitter_limit
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn waveform_at_zero_phase_is_base_voltage() {
        let engine = CalibrationEngine::new();
        let v = engine.calibration_waveform_impl(0.0, 0.0);
        assert!((v - CAL_BASE_VOLTAGE_V).abs() < 1e-9);
    }

    #[test]
    fn waveform_frequency_is_10mhz() {
        let engine = CalibrationEngine::new();
        let period = 1.0 / CAL_FREQUENCY_HZ;
        let v0 = engine.calibration_waveform_impl(0.0, 0.0);
        let v1 = engine.calibration_waveform_impl(period, 0.0);
        assert!((v0 - v1).abs() < 1e-9);
    }

    #[test]
    fn pid_corrects_small_jitter_to_nominal() {
        let mut engine = CalibrationEngine::new();
        let dt = 1.0e-6;
        let jitter = 4.0e-5; // below the 5.05e-5 limit
        let (_, corrected, status) = engine.step_impl(dt, jitter);
        assert_eq!(status, "STATUS_NOMINAL_PASS");
        assert!(corrected.abs() <= PHASE_JITTER_LIMIT_RAD);
    }

    #[test]
    fn pid_shuts_down_for_excessive_jitter() {
        let mut engine = CalibrationEngine::new();
        let dt = 1.0e-6;
        let jitter = 10.0; // far above the limit
        let (_, corrected, status) = engine.step_impl(dt, jitter);
        assert_eq!(status, "STATUS_EMERGENCY_SHUTDOWN");
        assert!(corrected.abs() > PHASE_JITTER_LIMIT_RAD);
    }

    #[test]
    fn pid_gains_match_spec() {
        let engine = CalibrationEngine::new();
        let (kp, ki, kd) = engine.pid_gains();
        assert_eq!(kp, 1.85);
        assert_eq!(ki, 9.12e3);
        assert_eq!(kd, 3.45e-7);
    }
}
