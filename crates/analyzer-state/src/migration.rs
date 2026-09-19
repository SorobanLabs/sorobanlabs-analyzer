//! Author-supplied migration manifest.
//!
//! A [`MigrationManifest`] is a declaration the *contract author*
//! supplies about how state should transition between a current and
//! candidate executable. It is evidence, not proof: this analyzer does
//! not independently verify that a manifest's claims are accurate, that
//! the named migration function actually performs what it describes, or
//! that the declared schema versions match what the executables
//! actually read and write. Every conclusion derived from a manifest
//! must stay labeled as manifest-derived (author-supplied, unverified
//! by this analyzer), never upgraded into an analyzer-established fact
//! by wording alone.

use analyzer_core::{AnalyzerError, SerializationError};
use analyzer_evidence::{Evidence, EvidenceSource};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// One family of related state keys the manifest describes.
///
/// This is author-supplied prose and a claimed type shape; the analyzer
/// does not parse `key_type`/`value_type` as executable schema, it only
/// preserves and reports them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyFamily {
    pub name: String,
    /// What this key family holds and how its keys are formed, in the
    /// author's own words.
    pub description: String,
    /// The author's description of the key type (for example,
    /// `"Address"` or `"(Symbol, u32)"`). Free text, not a parsed type.
    pub key_type: String,
    /// The author's description of the value type.
    pub value_type: String,
}

/// A declared precondition that must hold before migration is safe to
/// run, in the author's own words.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MigrationPrerequisite {
    pub description: String,
}

/// The migration function the author declares handles the state
/// transition, if the manifest names one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MigrationFunction {
    /// The contract entrypoint name the author says performs migration.
    /// This analyzer does not confirm the candidate executable actually
    /// exports a function by this name; that cross-check belongs to
    /// whatever calls this module (it has the normalized interface).
    pub name: String,
    /// Whether the author declares this function is guarded to run at
    /// most once (a run-once migration guard). This is a claim, not a
    /// property this analyzer has checked.
    pub one_time: bool,
}

/// An author-supplied declaration of how a contract's state should
/// transition from the current executable's schema to the candidate's.
///
/// Every field is optional or defaults to empty: a manifest may
/// legitimately declare only part of this information.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MigrationManifest {
    /// The schema version identifier this manifest itself describes.
    pub state_schema_version: Option<String>,
    /// The schema version the author expects the current executable's
    /// existing state to be in.
    pub expected_old_schema: Option<String>,
    /// The schema version the author expects state to be in once the
    /// candidate executable (and, if declared, its migration function)
    /// has run.
    pub expected_new_schema: Option<String>,
    pub key_families: Vec<KeyFamily>,
    pub migration_function: Option<MigrationFunction>,
    pub prerequisites: Vec<MigrationPrerequisite>,
}

impl MigrationManifest {
    /// Parse a manifest from JSON text.
    pub fn parse_json(text: &str) -> Result<Self, AnalyzerError> {
        serde_json::from_str(text).map_err(|source| {
            SerializationError::with_source("failed to parse migration manifest", source).into()
        })
    }

    /// Serialize this manifest to canonical (pretty-printed, trailing
    /// newline) JSON, matching the same canonical style as
    /// `analyzer_report`'s report serialization.
    pub fn to_canonical_json(&self) -> Result<String, AnalyzerError> {
        let mut json = serde_json::to_string_pretty(self).map_err(|source| {
            SerializationError::with_source("failed to serialize migration manifest", source)
        })?;
        json.push('\n');
        Ok(json)
    }

    /// The manifest's deterministic content hash: SHA-256 of its
    /// canonical JSON encoding, hex-encoded. Used as the manifest's
    /// identity in evidence records; never a timestamp or file path.
    pub fn content_hash(&self) -> Result<String, AnalyzerError> {
        let json = self.to_canonical_json()?;
        let digest = Sha256::digest(json.as_bytes());
        Ok(hex::encode(digest))
    }

    /// True if the manifest names a migration function at all. This
    /// reflects only what the author declared; it is not a claim that a
    /// migration is mechanically required, and it is not a claim that
    /// the named function actually exists or works.
    pub fn declares_migration_function(&self) -> bool {
        self.migration_function.is_some()
    }

    /// True if the manifest declares different expected old/new schema
    /// versions. A declared schema *change* is evidence worth surfacing
    /// even when no migration function is named.
    pub fn declares_schema_change(&self) -> bool {
        match (&self.expected_old_schema, &self.expected_new_schema) {
            (Some(old), Some(new)) => old != new,
            _ => false,
        }
    }

    /// Build an [`Evidence`] record describing this manifest as a
    /// source. The observation text is explicitly prefixed
    /// `UNVERIFIED (author-supplied)` so it cannot be mistaken for an
    /// analyzer-established fact when read out of context.
    pub fn to_evidence(&self) -> Result<Evidence, AnalyzerError> {
        let hash = self.content_hash()?;
        let observation =
            format!(
            "UNVERIFIED (author-supplied): manifest declares {} key famil{}, {}, {} prerequisite{}",
            self.key_families.len(),
            if self.key_families.len() == 1 { "y" } else { "ies" },
            match &self.migration_function {
                Some(function) if function.one_time => {
                    format!("a one-time migration function '{}'", function.name)
                }
                Some(function) => format!("a migration function '{}'", function.name),
                None => "no migration function".to_string(),
            },
            self.prerequisites.len(),
            if self.prerequisites.len() == 1 { "" } else { "s" },
        );

        Ok(Evidence::new(
            EvidenceSource::MigrationManifest {
                manifest_hash: hash,
            },
            "analyzer-state::migration",
            None,
            observation,
        ))
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    fn sample_manifest() -> MigrationManifest {
        MigrationManifest {
            state_schema_version: Some("2".to_string()),
            expected_old_schema: Some("1".to_string()),
            expected_new_schema: Some("2".to_string()),
            key_families: vec![KeyFamily {
                name: "balances".to_string(),
                description: "per-account balance entries".to_string(),
                key_type: "Address".to_string(),
                value_type: "i128".to_string(),
            }],
            migration_function: Some(MigrationFunction {
                name: "migrate".to_string(),
                one_time: true,
            }),
            prerequisites: vec![MigrationPrerequisite {
                description: "admin must call migrate before any other entrypoint".to_string(),
            }],
        }
    }

    #[test]
    fn parses_and_round_trips_through_json() {
        let manifest = sample_manifest();
        let json = manifest.to_canonical_json().unwrap();
        let parsed = MigrationManifest::parse_json(&json).unwrap();
        assert_eq!(manifest, parsed);
    }

    #[test]
    fn malformed_json_returns_structured_error_not_panic() {
        let result = MigrationManifest::parse_json("not json");
        assert!(matches!(result, Err(AnalyzerError::Serialization(_))));
    }

    #[test]
    fn empty_manifest_declares_nothing() {
        let manifest = MigrationManifest::default();
        assert!(!manifest.declares_migration_function());
        assert!(!manifest.declares_schema_change());
    }

    #[test]
    fn declares_migration_function_reflects_manifest_content() {
        assert!(sample_manifest().declares_migration_function());
    }

    #[test]
    fn declares_schema_change_requires_both_versions_to_differ() {
        assert!(sample_manifest().declares_schema_change());

        let mut same_schema = sample_manifest();
        same_schema.expected_new_schema = same_schema.expected_old_schema.clone();
        assert!(!same_schema.declares_schema_change());

        let mut missing_old = sample_manifest();
        missing_old.expected_old_schema = None;
        assert!(!missing_old.declares_schema_change());
    }

    #[test]
    fn content_hash_is_deterministic_and_ignores_nothing_external() {
        let manifest = sample_manifest();
        let first = manifest.content_hash().unwrap();
        let second = manifest.content_hash().unwrap();
        assert_eq!(first, second);
        assert_eq!(first.len(), 64);
    }

    #[test]
    fn different_manifests_produce_different_hashes() {
        let a = sample_manifest();
        let mut b = sample_manifest();
        b.state_schema_version = Some("3".to_string());
        assert_ne!(a.content_hash().unwrap(), b.content_hash().unwrap());
    }

    #[test]
    fn evidence_observation_is_explicitly_labeled_unverified() {
        let manifest = sample_manifest();
        let evidence = manifest.to_evidence().unwrap();
        assert!(evidence
            .observation()
            .starts_with("UNVERIFIED (author-supplied)"));
        assert!(matches!(
            evidence.source(),
            EvidenceSource::MigrationManifest { .. }
        ));
    }
}
