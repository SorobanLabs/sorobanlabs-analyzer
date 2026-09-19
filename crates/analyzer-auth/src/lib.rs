//! Authorization-surface extraction and diffing for the SorobanLabs
//! Analyzer.
//!
//! Responsibilities: represent entrypoint to authorization-requirement
//! to principal to check relationships ([`extraction`]), and compare
//! those requirements between a current and candidate executable
//! ([`diff`]). Analysis here is structural and evidence-based; it does
//! not judge whether a contract is secure or insecure in general.

mod diff;
mod extraction;

pub use diff::{diff_authorization_surfaces, AuthorizationChange, AuthorizationDiff};
pub use extraction::{
    extract_authorization_surface, AuthorizationSurface, EntrypointAuthorization,
    AUTH_IMPORT_NAMES, CUSTOM_ACCOUNT_CHECK_AUTH_EXPORT,
};
