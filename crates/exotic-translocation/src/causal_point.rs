//! Observer history crystallization, adapted from
//! `shbt-precision/src/shbt/causal_point.rs`.
//!
//! A `CausalPoint` is a local observer address whose history is a chain of
//! crystallized memory packets. Retrieval of a packet costs
//! `C_op` bits; it may only crystallize when the cost fits the local
//! holographic entropy budget
//!
//! ```text
//! C_op <= C_local = A / (4 L_P^2 ln 2)
//! ```
//!
//! Violating the budget during de-rendering or re-rendering raises
//! [`AnomalyClosureError`] (entropy-anomaly subclass).

use super::AnomalyClosureError;

/// Planck length in metres.
pub const L_PLANCK_M: f64 = 1.616_255e-35;

/// Local holographic bit budget on an area `a_m2` boundary:
/// C_local = A / (4 L_P^2 ln 2).
pub fn local_entropy_budget_bits(a_m2: f64) -> f64 {
    a_m2 / (4.0 * L_PLANCK_M * L_PLANCK_M * std::f64::consts::LN_2)
}

/// One crystallized observer memory packet.
#[derive(Clone, Debug)]
pub struct MemoryPacket {
    /// Tick index of the packet within the observer history.
    pub tick: u64,
    /// Payload bits.
    pub bits: u64,
    /// Retrieval/de-render cost charged to the entropy budget (bits).
    pub c_op: f64,
}

/// A local observer address with a crystallized packet history.
#[derive(Clone, Debug)]
pub struct CausalPoint {
    /// Boundary area available to this observer (m^2).
    pub area_m2: f64,
    history: Vec<MemoryPacket>,
    pub bit_total: u64,
}

impl CausalPoint {
    pub fn new(area_m2: f64) -> Self {
        Self {
            area_m2,
            history: Vec::new(),
            bit_total: 0,
        }
    }

    /// The observer's entropy budget in bits.
    pub fn entropy_budget_bits(&self) -> f64 {
        local_entropy_budget_bits(self.area_m2)
    }

    /// Attempt to crystallize a memory packet carrying `bits` payload bits
    /// with retrieval cost `c_op`. Enforces `c_op <= C_local` and cumulative
    /// budget accounting: the running total of committed bits must also fit.
    pub fn crystallize(&mut self, tick: u64, bits: u64, c_op: f64) -> Result<(), AnomalyClosureError> {
        let c_local = self.entropy_budget_bits();
        let projected = self.bit_total as f64 + c_op;
        if c_op > c_local || projected > c_local {
            return Err(AnomalyClosureError {
                ds2: 0.0,
                reason: "entropy",
            });
        }
        self.history.push(MemoryPacket { tick, bits, c_op });
        self.bit_total += bits;
        Ok(())
    }

    /// Number of crystallized packets.
    pub fn depth(&self) -> usize {
        self.history.len()
    }
}
