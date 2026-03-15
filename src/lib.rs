//! # qala-shared
//!
//! Shared types, event contracts, error definitions, and domain models for the
//! qala Solution Factory Operating System.
//!
//! All Rust services in the qala platform depend on this crate for:
//! - Canonical domain types (Solution, SDE, SolutionFactory, …)
//! - Platform-wide event envelopes and topic taxonomy
//! - Standardised error types
//! - Common primitive aliases and utility types

pub mod errors;
pub mod events;
pub mod models;
pub mod types;

// Convenience re-exports
pub use errors::{QalaError, QalaResult};
pub use events::{EventEnvelope, EventTopic};
pub use types::{DataType, Maturity, SolutionType};
