//! brain-core: the Neuroform brain engine.
//!
//! M0 scope (DESIGN.md §20): tick loop, global latent state, neuromodulatory
//! system, NF1 format (write/read/verify, encryption, capacity ledger),
//! deterministic replay. M1 adds memory stores and the LLM boundary.

pub mod audit;
pub mod body;
pub mod boundary;
pub mod brain;
pub mod capacity;
pub mod drawing;
pub mod embodiment;
pub mod events;
pub mod format;
pub mod memory;
pub mod modulators;
pub mod network;
pub mod physics;
pub mod rng;
pub mod semantic;
pub mod sleep;
pub mod state;
pub mod voice;
pub mod writing;

pub use brain::{Brain, SIM_TICK_SECS, SNAPSHOT_EVERY_TICKS};
pub use capacity::{Admission, CapacityLedger, Tier, TierName};
pub use format::FormatError;

pub const TICKS_PER_SECOND: u64 = 10;
