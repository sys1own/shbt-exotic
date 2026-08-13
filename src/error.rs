//! Error types for shbt-exotic.

use pyo3::{PyErr, exceptions::PyException};

#[derive(Debug)]
pub enum ExoticError {
    AnomalyClosureError(String),
    PrecisionLossError(String),
    RigidityViolationError(String),
}

impl std::fmt::Display for ExoticError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExoticError::AnomalyClosureError(msg) => write!(f, "AnomalyClosureError: {}", msg),
            ExoticError::PrecisionLossError(msg) => write!(f, "PrecisionLossError: {}", msg),
            ExoticError::RigidityViolationError(msg) => write!(f, "RigidityViolationError: {}", msg),
        }
    }
}

impl std::error::Error for ExoticError {}

pyo3::create_exception!(shbt_exotic, AnomalyClosureError, PyException);

impl From<ExoticError> for PyErr {
    fn from(err: ExoticError) -> PyErr {
        match err {
            ExoticError::AnomalyClosureError(msg)
            | ExoticError::PrecisionLossError(msg)
            | ExoticError::RigidityViolationError(msg) => PyErr::new::<AnomalyClosureError, _>(msg),
        }
    }
}
