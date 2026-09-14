//! Authorization-surface extraction and diffing for the SorobanLabs
//! Analyzer.
//!
//! Responsibilities: represent entrypoint to authorization-requirement to
//! principal to check relationships, and compare those requirements
//! between a current and candidate executable. Analysis here is
//! structural and evidence-based; it does not judge whether a contract is
//! secure or insecure in general.
//!
//! This initial version establishes the crate only.
