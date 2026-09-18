//! Local WASM artifact loading and canonical byte-level identity.
//!
//! This module establishes the analyzer's *Level 1* artifact model: a
//! byte sequence that has been confirmed to be structurally a WASM
//! binary. It does not attempt to determine whether that WASM binary is
//! a supported Soroban executable (Level 2), or whether it is a
//! semantically analyzable Soroban contract (Level 3). Those
//! distinctions belong to later steps.
//!
//! # Identity
//!
//! An artifact's identity, [`ArtifactHash`], is `SHA256` of the exact
//! bytes loaded from disk. It intentionally does not account for:
//!
//! - the source path or filename the bytes were loaded from
//! - any parsed or normalized representation of the WASM
//! - the semantic behavior of the contract
//!
//! Byte identity is not semantic identity: two artifacts with different
//! `ArtifactHash` values may later be found to behave identically, and
//! this module makes no attempt to detect that. Conversely, identical
//! bytes always produce the identical hash regardless of where they
//! were loaded from.
//!
//! # WASM validation
//!
//! Structural WASM identification here is limited to checking the
//! standard 8-byte WASM binary header: the `\0asm` magic number followed
//! by the little-endian version `1`. This is deliberately minimal: a
//! full parser or validator is not pulled in as a dependency merely to
//! check eight header bytes. Host import validation, custom section
//! interpretation, and Soroban contract specification checks are left to
//! later steps that need them.

use std::fmt;
use std::path::{Path, PathBuf};

use analyzer_core::{AnalyzerError, BackendError, InvalidInputError, UnsupportedArtifactError};
use sha2::{Digest, Sha256};

/// The WASM binary magic number: the ASCII/byte sequence `\0asm`.
const WASM_MAGIC: [u8; 4] = [0x00, 0x61, 0x73, 0x6d];

/// The WASM binary format version this analyzer recognizes (version 1,
/// little-endian).
const WASM_VERSION: [u8; 4] = [0x01, 0x00, 0x00, 0x00];

/// The canonical SHA-256 identity of an artifact's exact bytes.
///
/// Two artifacts share an [`ArtifactHash`] if and only if their raw byte
/// content is identical. The hash is computed solely from
/// [`LoadedWasm::bytes`]; it never depends on [`ArtifactSource`], so
/// loading identical bytes from two different paths yields the same
/// hash.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ArtifactHash([u8; 32]);

impl ArtifactHash {
    /// Compute the canonical identity of `bytes`: `SHA256(bytes)`.
    pub fn of(bytes: &[u8]) -> Self {
        let digest = Sha256::digest(bytes);
        let mut out = [0u8; 32];
        out.copy_from_slice(&digest);
        Self(out)
    }

    /// The raw 32-byte SHA-256 digest.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    /// The canonical textual representation: 64 lowercase hexadecimal
    /// characters.
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }
}

impl fmt::Display for ArtifactHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_hex())
    }
}

impl fmt::Debug for ArtifactHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("ArtifactHash").field(&self.to_hex()).finish()
    }
}

/// Where a [`LoadedWasm`]'s bytes came from.
///
/// This is diagnostic metadata, not artifact identity: it is never
/// consulted when computing an [`ArtifactHash`]. Only local filesystem
/// sources are represented for now; remote (RPC) sources are a later
/// step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArtifactSource {
    /// Loaded from a local filesystem path.
    LocalFile(PathBuf),
}

/// A WASM artifact loaded from local storage.
///
/// `LoadedWasm` preserves the exact bytes read from disk alongside their
/// canonical [`ArtifactHash`] and their [`ArtifactSource`]. The type
/// establishes the invariant `hash == ArtifactHash::of(&bytes)`: the
/// hash is computed once at load time and cannot drift out of sync,
/// because the bytes and hash are only ever exposed as immutable views.
///
/// Loading only confirms that the bytes carry a valid WASM binary
/// header (Level 1). It does not confirm that the artifact is a
/// supported Soroban executable (Level 2) or that it is semantically
/// analyzable (Level 3).
#[derive(Debug, Clone)]
pub struct LoadedWasm {
    bytes: Vec<u8>,
    hash: ArtifactHash,
    source: ArtifactSource,
}

impl LoadedWasm {
    /// Load a WASM artifact from a local filesystem path.
    ///
    /// This reads the file's exact bytes, verifies the standard WASM
    /// binary header, and computes the canonical [`ArtifactHash`] over
    /// those bytes.
    ///
    /// # Errors
    ///
    /// Returns [`InvalidInputError`] if `path` is empty. Returns a
    /// [`BackendError`] if the path cannot be read (missing file,
    /// permission denied, or the path names a directory). Returns an
    /// [`UnsupportedArtifactError`] if the bytes read do not begin with
    /// a valid WASM binary header.
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, AnalyzerError> {
        let path = path.as_ref();

        if path.as_os_str().is_empty() {
            return Err(InvalidInputError::new("artifact path must not be empty").into());
        }

        let bytes = std::fs::read(path).map_err(|source| {
            BackendError::with_source(
                format!("failed to read artifact at '{}'", path.display()),
                source,
            )
        })?;

        validate_wasm_header(&bytes)?;

        let hash = ArtifactHash::of(&bytes);
        Ok(Self {
            bytes,
            hash,
            source: ArtifactSource::LocalFile(path.to_path_buf()),
        })
    }

    /// The exact bytes read from the artifact's source, unmodified.
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    /// The canonical SHA-256 identity of [`Self::bytes`].
    pub fn hash(&self) -> ArtifactHash {
        self.hash
    }

    /// Where this artifact's bytes were loaded from.
    pub fn source(&self) -> &ArtifactSource {
        &self.source
    }
}

/// Confirm that `bytes` begin with the standard WASM binary header: the
/// `\0asm` magic number followed by version 1.
///
/// This is a purely structural check. It does not parse sections, and
/// it makes no claim about whether the artifact is a supported Soroban
/// executable.
fn validate_wasm_header(bytes: &[u8]) -> Result<(), AnalyzerError> {
    let header_ok = bytes.len() >= 8 && bytes[0..4] == WASM_MAGIC && bytes[4..8] == WASM_VERSION;

    if !header_ok {
        return Err(UnsupportedArtifactError::new(
            "artifact is not a valid WASM binary: missing or unrecognized WASM header",
        )
        .into());
    }

    Ok(())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    fn fixture(name: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("fixtures")
            .join("executable")
            .join(name)
    }

    #[test]
    fn loads_valid_wasm_artifact() {
        let loaded = LoadedWasm::from_path(fixture("minimal-valid.wasm")).unwrap();
        assert_eq!(
            loaded.bytes(),
            &[0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00]
        );
    }

    #[test]
    fn loaded_bytes_exactly_match_file_bytes() {
        let path = fixture("minimal-valid.wasm");
        let expected = std::fs::read(&path).unwrap();
        let loaded = LoadedWasm::from_path(&path).unwrap();
        assert_eq!(loaded.bytes(), expected.as_slice());
    }

    #[test]
    fn hash_is_deterministic_across_loads() {
        let path = fixture("minimal-valid.wasm");
        let first = LoadedWasm::from_path(&path).unwrap();
        let second = LoadedWasm::from_path(&path).unwrap();
        assert_eq!(first.hash(), second.hash());
    }

    #[test]
    fn hash_matches_known_sha256_vector() {
        // SHA-256("abc") is a well-known test vector.
        let hash = ArtifactHash::of(b"abc");
        assert_eq!(
            hash.to_hex(),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn same_bytes_from_different_paths_produce_same_hash() {
        let a = LoadedWasm::from_path(fixture("minimal-valid.wasm")).unwrap();
        let b = LoadedWasm::from_path(fixture("minimal-valid-copy.wasm")).unwrap();
        assert_eq!(a.hash(), b.hash());
        assert_eq!(a.bytes(), b.bytes());
    }

    #[test]
    fn different_bytes_produce_different_hashes() {
        let a = LoadedWasm::from_path(fixture("minimal-valid.wasm")).unwrap();
        let b = LoadedWasm::from_path(fixture("second-valid.wasm")).unwrap();
        assert_ne!(a.hash(), b.hash());
    }

    #[test]
    fn missing_path_returns_structured_error_without_panicking() {
        let result = LoadedWasm::from_path(fixture("does-not-exist.wasm"));
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AnalyzerError::Backend(_)));
    }

    #[test]
    fn directory_path_returns_structured_error_without_panicking() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR"));
        let result = LoadedWasm::from_path(dir);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), AnalyzerError::Backend(_)));
    }

    #[test]
    fn empty_path_returns_invalid_input_error() {
        let result = LoadedWasm::from_path("");
        assert!(matches!(
            result.unwrap_err(),
            AnalyzerError::InvalidInput(_)
        ));
    }

    #[test]
    fn invalid_wasm_bytes_return_unsupported_artifact_error() {
        let result = LoadedWasm::from_path(fixture("invalid.wasm"));
        assert!(matches!(
            result.unwrap_err(),
            AnalyzerError::UnsupportedArtifact(_)
        ));
    }

    #[test]
    fn hash_formats_as_lowercase_hex_of_expected_length() {
        let hash = ArtifactHash::of(b"soroban");
        let hex = hash.to_hex();
        assert_eq!(hex.len(), 64);
        assert!(hex
            .chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()));
        assert_eq!(hash.to_string(), hex);
    }
}
