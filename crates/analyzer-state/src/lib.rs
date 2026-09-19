//! Contract state representation and compatibility analysis for the
//! SorobanLabs Analyzer.
//!
//! Responsibilities: local and RPC state snapshot loading ([`snapshot`],
//! [`source`], [`rpc`]), author-supplied migration manifests
//! ([`migration`]), and comparison of known state requirements between
//! a current and candidate executable ([`compatibility`]). Compatibility
//! outcomes distinguish COMPATIBLE, REQUIRES_MIGRATION,
//! POTENTIALLY_INCOMPATIBLE, and NOT_DETERMINED; the analyzer never
//! claims compatibility when evidence is missing.

mod compatibility;
mod migration;
mod rpc;
mod snapshot;
mod source;

pub use compatibility::{assess_state_compatibility, StateCompatibility, StateCompatibilityResult};
pub use migration::{KeyFamily, MigrationFunction, MigrationManifest, MigrationPrerequisite};
pub use rpc::{DataKeyRequest, RpcStateSource, Transport, UreqTransport};
pub use snapshot::{ContractDataEntrySnapshot, Durability, ExecutableForm, StateSnapshot};
pub use source::{LocalSnapshotSource, StateSource};
