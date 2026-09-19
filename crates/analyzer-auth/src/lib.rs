//! Authorization-surface extraction and diffing for the SorobanLabs
//! Analyzer.
//!
//! Responsibilities: represent entrypoint to authorization-requirement
//! to principal to check relationships ([`extraction`]), and compare
//! those requirements between a current and candidate executable
//! (added in a later step). Analysis here is structural and
//! evidence-based; it does not judge whether a contract is secure or
//! insecure in general.

mod extraction;

pub use extraction::{
    extract_authorization_surface, AuthorizationSurface, EntrypointAuthorization, AUTH_IMPORT_NAMES,
};
