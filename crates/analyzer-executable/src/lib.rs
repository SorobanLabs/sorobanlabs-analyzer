//! Executable artifact loading, hashing, WASM inspection, and interface
//! normalization/diffing for the SorobanLabs Analyzer.
//!
//! Responsibilities: canonical executable hashing, WASM metadata parsing,
//! Soroban contract specification extraction, public interface
//! normalization, and current-vs-candidate executable comparison.
//! All outputs must be deterministic.
//!
//! This version establishes local WASM artifact loading and canonical
//! byte-level identity ([`LoadedWasm`], [`ArtifactHash`],
//! [`ArtifactSource`]), generic and Soroban-specific structural WASM
//! validation ([`validation`]), and Soroban environment metadata
//! parsing ([`environment_meta`]). Contract metadata extraction,
//! contract specification parsing, and interface diffing are added in
//! later steps.

mod artifact;
mod environment_meta;
mod validation;

pub use artifact::{ArtifactHash, ArtifactSource, LoadedWasm};
pub use environment_meta::{
    parse_environment_metadata, EnvironmentInterfaceVersion, EnvironmentMetaReport,
    CONTRACT_ENV_META_SECTION,
};
pub use validation::{
    check_soroban_structural_compatibility, validate_generic_wasm, SorobanStructuralReport,
    SorobanStructuralViolation, MAX_WASM_BYTES,
};
