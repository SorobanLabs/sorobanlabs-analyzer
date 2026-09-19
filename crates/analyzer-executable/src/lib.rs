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
//! [`ArtifactSource`]), plus generic and Soroban-specific structural
//! WASM validation ([`validation`]). Soroban environment/contract
//! metadata extraction, contract specification parsing, and interface
//! diffing are added in later steps.

mod artifact;
mod validation;

pub use artifact::{ArtifactHash, ArtifactSource, LoadedWasm};
pub use validation::{
    check_soroban_structural_compatibility, validate_generic_wasm, SorobanStructuralReport,
    SorobanStructuralViolation, MAX_WASM_BYTES,
};
