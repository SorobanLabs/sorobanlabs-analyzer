//! Contract state representation and compatibility analysis for the
//! SorobanLabs Analyzer.
//!
//! Responsibilities: local and RPC state snapshot loading ([`snapshot`],
//! [`source`], [`rpc`]), author-supplied migration manifests
//! ([`migration`]), and comparison of known state requirements between
//! a current and candidate executable. Compatibility outcomes must
//! distinguish COMPATIBLE, REQUIRES_MIGRATION, POTENTIALLY_INCOMPATIBLE,
//! and NOT_DETERMINED; the analyzer must never claim compatibility when
//! evidence is missing. Those compatibility rules are added in a later
//! step.

mod migration;
mod rpc;
mod snapshot;
mod source;

pub use migration::{KeyFamily, MigrationFunction, MigrationManifest, MigrationPrerequisite};
pub use rpc::{DataKeyRequest, RpcStateSource, Transport, UreqTransport};
pub use snapshot::{ContractDataEntrySnapshot, Durability, ExecutableForm, StateSnapshot};
pub use source::{LocalSnapshotSource, StateSource};
