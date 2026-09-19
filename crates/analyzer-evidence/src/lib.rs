//! Evidence model for the SorobanLabs Analyzer.
//!
//! Every meaningful finding must be traceable to evidence: a WASM
//! section, contract specification, state snapshot, ledger entry,
//! authorization path, migration declaration, controlled execution
//! result, or protocol context ([`source::EvidenceSource`]). Evidence
//! identity ([`reference::EvidenceId`]) is deterministic and
//! content-based (SHA-256 over a canonical encoding of the record's
//! fields), never a timestamp or local file path.
//!
//! [`collector::EvidenceCollector`] accumulates evidence in encounter
//! order during one analysis run.

mod collector;
mod reference;
mod source;

pub use collector::EvidenceCollector;
pub use reference::{Evidence, EvidenceId};
pub use source::EvidenceSource;
