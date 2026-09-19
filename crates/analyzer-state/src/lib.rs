//! Contract state representation and compatibility analysis for the
//! SorobanLabs Analyzer.
//!
//! Responsibilities: local state snapshot loading
//! ([`snapshot`], [`source`]), ledger entry normalization, and
//! comparison of known state requirements between a current and
//! candidate executable. Compatibility outcomes must distinguish
//! COMPATIBLE, REQUIRES_MIGRATION, POTENTIALLY_INCOMPATIBLE, and
//! NOT_DETERMINED; the analyzer must never claim compatibility when
//! evidence is missing. Those compatibility rules, the read-only RPC
//! source, and the migration manifest model are added in later steps.

mod snapshot;
mod source;

pub use snapshot::{ContractDataEntrySnapshot, Durability, ExecutableForm, StateSnapshot};
pub use source::{LocalSnapshotSource, StateSource};
