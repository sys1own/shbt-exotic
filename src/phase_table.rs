//! RF phase-modulation table exporter for an 8x8 SHBT emitter array.
//!
//! Maps conformal dimensions `h_ij` and an effective target velocity `v_eff` to
//! discrete microwave phase commands `exp(i theta)` suitable for driving InP/InGaAs
//! SHBT phase shifters.  The gate/base turn-on and collector-drain bias levels
//! are taken into account so that each phase maps to a physical control voltage.

use pyo3::prelude::*;
use serde::{Deserialize, Serialize};

/// Gate/base turn-on voltage (V).
pub const GATE_BASE_TURN_ON_V: f64 = 3.8;

/// Collector drain supply voltage (V).
pub const COLLECTOR_DRAIN_V: f64 = 7.4;

/// Phase-shifter array dimension (8x8 emitters).
pub const ARRAY_DIM: usize = 8;

/// One discrete microwave phase command for a single emitter.
#[pyclass(name = "PhaseCommand", frozen, get_all)]
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct PhaseCommand {
    pub i: usize,
    pub j: usize,
    pub h_ij: f64,
    pub v_eff: f64,
    pub theta_rad: f64,
    pub cos_theta: f64,
    pub sin_theta: f64,
    pub v_phase: f64,
}

impl PhaseCommand {
    fn new(i: usize, j: usize, h_ij: f64, v_eff: f64) -> Self {
        let two_pi = 2.0 * std::f64::consts::PI;
        let theta = (two_pi * h_ij * v_eff).rem_euclid(two_pi);
        let v_phase = GATE_BASE_TURN_ON_V
            + (theta / two_pi) * (COLLECTOR_DRAIN_V - GATE_BASE_TURN_ON_V);
        Self {
            i,
            j,
            h_ij,
            v_eff,
            theta_rad: theta,
            cos_theta: theta.cos(),
            sin_theta: theta.sin(),
            v_phase,
        }
    }
}

/// Exporter that builds JSON/CSV phase-modulation tables for the 8x8 SHBT array.
#[pyclass(name = "ExportPhaseModulationTable")]
#[derive(Clone, Debug)]
pub struct ExportPhaseModulationTable;

impl ExportPhaseModulationTable {
    /// Validate that `h` is an 8x8 matrix.
    fn validate(h: &[Vec<f64>]) -> Result<(), crate::error::ExoticError> {
        if h.len() != ARRAY_DIM {
            return Err(crate::error::ExoticError::AnomalyClosureError(format!(
                "h matrix must have {} rows, got {}",
                ARRAY_DIM,
                h.len()
            )));
        }
        for (i, row) in h.iter().enumerate() {
            if row.len() != ARRAY_DIM {
                return Err(crate::error::ExoticError::AnomalyClosureError(format!(
                    "row {} must have {} columns, got {}",
                    i, ARRAY_DIM, row.len()
                )));
            }
        }
        Ok(())
    }

    /// Build the 64-entry phase-command table.
    pub fn build_table_impl(
        &self,
        h: &[Vec<f64>],
        v_eff: f64,
    ) -> Result<Vec<PhaseCommand>, crate::error::ExoticError> {
        Self::validate(h)?;
        let mut table = Vec::with_capacity(ARRAY_DIM * ARRAY_DIM);
        for (i, row) in h.iter().enumerate() {
            for (j, &h_ij) in row.iter().enumerate() {
                table.push(PhaseCommand::new(i, j, h_ij, v_eff));
            }
        }
        Ok(table)
    }

    /// Export the table as a JSON string.
    pub fn to_json_impl(
        &self,
        h: &[Vec<f64>],
        v_eff: f64,
    ) -> Result<String, crate::error::ExoticError> {
        let table = self.build_table_impl(h, v_eff)?;
        serde_json::to_string_pretty(&table).map_err(|e| {
            crate::error::ExoticError::AnomalyClosureError(format!("JSON error: {}", e))
        })
    }

    /// Export the table as a CSV string.
    pub fn to_csv_impl(
        &self,
        h: &[Vec<f64>],
        v_eff: f64,
    ) -> Result<String, crate::error::ExoticError> {
        let table = self.build_table_impl(h, v_eff)?;
        let mut s = String::new();
        s.push_str("i,j,h_ij,v_eff,theta_rad,cos_theta,sin_theta,v_phase\n");
        for cmd in &table {
            s.push_str(&format!(
                "{},{},{:.12},{:.12},{:.12},{:.12},{:.12},{:.12}\n",
                cmd.i,
                cmd.j,
                cmd.h_ij,
                cmd.v_eff,
                cmd.theta_rad,
                cmd.cos_theta,
                cmd.sin_theta,
                cmd.v_phase
            ));
        }
        Ok(s)
    }

    /// Return the 64 phase commands as a plain Rust vector.
    pub fn table_entries_impl(
        &self,
        h: &[Vec<f64>],
        v_eff: f64,
    ) -> Result<Vec<PhaseCommand>, crate::error::ExoticError> {
        self.build_table_impl(h, v_eff)
    }
}

#[pymethods]
impl ExportPhaseModulationTable {
    #[new]
    pub fn py_new() -> Self {
        Self
    }

    /// Export an 8x8 phase-modulation table as JSON.
    pub fn export_json(&self, h: Vec<Vec<f64>>, v_eff: f64) -> PyResult<String> {
        self.to_json_impl(&h, v_eff).map_err(PyErr::from)
    }

    /// Export an 8x8 phase-modulation table as CSV.
    pub fn export_csv(&self, h: Vec<Vec<f64>>, v_eff: f64) -> PyResult<String> {
        self.to_csv_impl(&h, v_eff).map_err(PyErr::from)
    }

    /// Return the table as a list of `PhaseCommand` objects.
    pub fn table_entries(&self, h: Vec<Vec<f64>>, v_eff: f64) -> PyResult<Vec<PhaseCommand>> {
        self.table_entries_impl(&h, v_eff).map_err(PyErr::from)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_matrix() -> Vec<Vec<f64>> {
        (0..ARRAY_DIM)
            .map(|i| {
                (0..ARRAY_DIM)
                    .map(|j| (i as f64 + 1.0) / (j as f64 + 2.0))
                    .collect()
            })
            .collect()
    }

    #[test]
    fn produces_64_commands() {
        let exporter = ExportPhaseModulationTable;
        let table = exporter.build_table_impl(&test_matrix(), 0.5).unwrap();
        assert_eq!(table.len(), 64);
    }

    #[test]
    fn voltages_stay_within_bias_window() {
        let exporter = ExportPhaseModulationTable;
        let table = exporter.build_table_impl(&test_matrix(), 0.75).unwrap();
        for cmd in &table {
            assert!(cmd.v_phase >= GATE_BASE_TURN_ON_V);
            assert!(cmd.v_phase <= COLLECTOR_DRAIN_V);
        }
    }

    #[test]
    fn unit_complex_command_has_unit_norm() {
        let exporter = ExportPhaseModulationTable;
        let table = exporter.build_table_impl(&test_matrix(), 1.0).unwrap();
        for cmd in &table {
            let norm = (cmd.cos_theta * cmd.cos_theta + cmd.sin_theta * cmd.sin_theta).sqrt();
            assert!((norm - 1.0).abs() < 1e-12, "norm = {}", norm);
        }
    }

    #[test]
    fn rejects_non_8x8_matrix() {
        let exporter = ExportPhaseModulationTable;
        let bad = vec![vec![1.0; 3]; 3];
        assert!(exporter.build_table_impl(&bad, 1.0).is_err());
    }

    #[test]
    fn json_export_is_valid_and_non_empty() {
        let exporter = ExportPhaseModulationTable;
        let json = exporter.to_json_impl(&test_matrix(), 0.5).unwrap();
        assert!(json.starts_with('['));
        let parsed: Vec<PhaseCommand> = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.len(), 64);
    }
}
