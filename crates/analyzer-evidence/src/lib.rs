//! Evidence model for the SorobanLabs Analyzer.
//!
//! Every meaningful finding must be traceable to evidence: a WASM
//! section, contract specification, state snapshot, ledger entry,
//! authorization path, migration declaration, controlled execution
//! result, or protocol context. Evidence identity is deterministic and
//! content-based (SHA-256), never a timestamp or local file path.
//!
//! This initial version establishes the crate only.
