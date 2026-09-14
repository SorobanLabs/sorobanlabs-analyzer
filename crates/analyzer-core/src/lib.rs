//! Domain model and orchestration for the SorobanLabs Analyzer.
//!
//! This crate owns the concepts shared across the analyzer: analysis
//! inputs, policy, results, findings, confidence, severity, status, and
//! upgrade plans. It orchestrates the other crates rather than performing
//! low-level Stellar/Soroban parsing itself.
//!
//! The domain model is introduced incrementally; this initial version
//! establishes the crate only.
