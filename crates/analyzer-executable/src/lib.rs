//! Executable artifact loading, hashing, WASM inspection, and interface
//! normalization/diffing for the SorobanLabs Analyzer.
//!
//! Responsibilities: canonical executable hashing, WASM metadata parsing,
//! Soroban contract specification extraction, public interface
//! normalization, and current-vs-candidate executable comparison.
//! All outputs must be deterministic.
//!
//! This initial version establishes the crate only.
