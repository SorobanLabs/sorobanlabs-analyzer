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
//! [`ArtifactSource`]). WASM semantic inspection, Soroban contract
//! specification extraction, and interface diffing are added in later
//! steps.

mod artifact;

pub use artifact::{ArtifactHash, ArtifactSource, LoadedWasm};
