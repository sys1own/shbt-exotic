//! Causal coordinate and future-cone authorization for modular translocation.

use pyo3::prelude::*;

use crate::error::ExoticError;

/// Spacetime coordinate used for causal-cone authorization.
#[pyclass(name = "CausalCoordinate", get_all)]
#[derive(Clone, Copy, Debug)]
pub struct CausalCoordinate {
    pub t: f64,
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl CausalCoordinate {
    pub fn new(t: f64, x: f64, y: f64, z: f64) -> Self {
        Self { t, x, y, z }
    }

    /// Return true when `other` lies inside or on the future causal cone of `self`
    /// in natural units (c = 1).
    pub fn is_causally_authorized(&self, other: &Self) -> bool {
        let dt = other.t - self.t;
        if dt < 0.0 {
            return false;
        }
        let dx = other.x - self.x;
        let dy = other.y - self.y;
        let dz = other.z - self.z;
        let spatial = dx * dx + dy * dy + dz * dz;
        spatial <= dt * dt
    }
}

#[pymethods]
impl CausalCoordinate {
    #[new]
    pub fn py_new(t: f64, x: f64, y: f64, z: f64) -> Self {
        Self::new(t, x, y, z)
    }

    /// Check whether `other` is in the future causal cone of `self`.
    fn is_in_future_cone(&self, other: CausalCoordinate) -> bool {
        self.is_causally_authorized(&other)
    }
}

/// Verify that `tar` lies in the future causal cone of `src`; otherwise raise
/// `AnomalyClosureError`.
pub fn verify_future_cone_fatal(src: &CausalCoordinate, tar: &CausalCoordinate) -> Result<(), ExoticError> {
    if !src.is_causally_authorized(tar) {
        return Err(ExoticError::AnomalyClosureError(
            "Target coordinate is outside the future causal cone of the source".to_string(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn future_point_is_authorized() {
        let src = CausalCoordinate::new(0.0, 0.0, 0.0, 0.0);
        let tar = CausalCoordinate::new(1.0, 0.0, 0.0, 0.0);
        assert!(src.is_causally_authorized(&tar));
    }

    #[test]
    fn spacelike_point_is_not_authorized() {
        let src = CausalCoordinate::new(0.0, 0.0, 0.0, 0.0);
        let tar = CausalCoordinate::new(0.0, 1.0, 1.0, 0.0);
        assert!(!src.is_causally_authorized(&tar));
    }

    #[test]
    fn fatal_check_raises_on_spacelike() {
        let src = CausalCoordinate::new(0.0, 0.0, 0.0, 0.0);
        let tar = CausalCoordinate::new(0.0, 2.0, 0.0, 0.0);
        assert!(verify_future_cone_fatal(&src, &tar).is_err());
    }
}
