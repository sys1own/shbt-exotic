//! SHBT Exotic Technologies — unified Rust/Python simulator.
//!
//! Implements the four exotic protocols from the feasibility audit:
//!   1. Non-local holographic communication (Heegaard-Floer relabeling isometry)
//!   2. Temporal stasis (Newton-lock stationarity)
//!   3. Artificial ghost-seed gravity wells (mass-congestion coupling)
//!   4. Entropic refrigeration (holographic heat sink)
//!
//! All state-vector arithmetic is performed at 512-bit precision using the
//! `rug` crate, with a custom GMP/MPFR memory allocator to keep the HIL
//! audit path deterministic.

pub mod constants;
pub mod error;
pub mod gmp_memory;
pub mod ghost_seed;
pub mod hardware;
pub mod heegaard_floer;
pub mod hil_safety;
pub mod newton_lock;
pub mod refrigeration;
pub mod shbt;
pub mod stinespring;

pub use constants::*;
pub use shbt::communication::*;
pub use shbt::hil::*;
pub use shbt::mass_congestion::*;
pub use shbt::safety_monitor::*;
pub use error::*;
pub use ghost_seed::*;
pub use hardware::*;
pub use heegaard_floer::*;
pub use hil_safety::*;
pub use newton_lock::*;
pub use refrigeration::*;
pub use stinespring::*;

use pyo3::prelude::*;

#[pymodule(name = "_core")]
fn _core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    gmp_memory::init();
    m.add_class::<stinespring::UnifiedStinespringMap>()?;
    m.add_class::<heegaard_floer::HeegaardFloerRelabeling>()?;
    m.add_class::<newton_lock::NewtonLockStasis>()?;
    m.add_class::<ghost_seed::GhostSeedSynthesizer>()?;
    m.add_class::<refrigeration::EntropicRefrigerator>()?;
    m.add_class::<hil_safety::HilSafetyMonitor>()?;
    m.add_class::<hardware::HardwareSynthesisAuditor>()?;
    m.add_class::<stinespring::ExoticEngine>()?;
    m.add_class::<shbt::communication::HeegaardMappingTorus>()?;
    m.add_class::<shbt::hil::ThermalHILMonitor>()?;
    m.add_class::<shbt::safety_monitor::GateCycleShunt>()?;
    m.add_class::<shbt::safety_monitor::ThermalShuntAuditor>()?;
    m.add_class::<shbt::safety_monitor::SafetyMonitor>()?;
    m.add("AnomalyClosureError", m.py().get_type::<error::AnomalyClosureError>())?;
    Ok(())
}
