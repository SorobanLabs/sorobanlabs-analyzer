//! State compatibility analysis: combining what can actually be
//! established about whether existing contract state remains readable
//! and writable under a candidate executable.
//!
//! This deliberately does not attempt to reconstruct a contract's
//! storage layout from raw WASM or from snapshot bytes: that is not
//! reliable, and this analyzer does not fabricate storage-key knowledge
//! from binary patterns (see [`crate::snapshot`]'s module docs). What
//! [`assess_state_compatibility`] can establish from the inputs
//! available at this stage:
//!
//! - what the author's migration manifest ([`crate::migration`]), if
//!   supplied, declares
//! - whether the current and candidate declare the same executable form
//!   ([`crate::snapshot::ExecutableForm`]), and for two WASM
//!   executables, whether their hashes are identical
//!
//! Interface-derived signals (a struct field type changing, for
//! example) and rehearsal-derived signals (execution against
//! representative state) require inputs this crate does not have; they
//! are combined with this module's result in a later, higher-level
//! orchestration stage. Every outcome here carries specific reasons, so
//! a caller never sees a bare status with no explanation.

use crate::migration::MigrationManifest;
use crate::snapshot::ExecutableForm;

/// The outcome of a state compatibility check.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateCompatibility {
    /// No incompatible state condition was established from available
    /// evidence. This is not a guarantee of safety; it means nothing
    /// checked here raised a concern.
    Compatible,
    /// A migration is explicitly required (the manifest names a
    /// migration function) or mechanically implied (the manifest
    /// declares a schema version change with no migration function
    /// named).
    RequiresMigration,
    /// Available evidence indicates that existing state may not be
    /// readable or writable under the candidate executable.
    PotentiallyIncompatible,
    /// The analyzer cannot establish compatibility from the evidence
    /// available.
    NotDetermined,
}

impl StateCompatibility {
    /// The stable, `SCREAMING_SNAKE_CASE` textual form used in reports.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Compatible => "COMPATIBLE",
            Self::RequiresMigration => "REQUIRES_MIGRATION",
            Self::PotentiallyIncompatible => "POTENTIALLY_INCOMPATIBLE",
            Self::NotDetermined => "NOT_DETERMINED",
        }
    }
}

impl std::fmt::Display for StateCompatibility {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

/// The result of a state compatibility check: an outcome plus the
/// specific reasons it was reached.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateCompatibilityResult {
    pub outcome: StateCompatibility,
    /// Specific, human-readable reasons the outcome was reached. Never
    /// empty, and never a single vague "state changed" message.
    pub reasons: Vec<String>,
}

fn result(outcome: StateCompatibility, reason: String) -> StateCompatibilityResult {
    StateCompatibilityResult {
        outcome,
        reasons: vec![reason],
    }
}

fn executable_form_label(form: &ExecutableForm) -> &'static str {
    match form {
        ExecutableForm::Wasm { .. } => "WASM",
        ExecutableForm::StellarAsset => "Stellar Asset",
        ExecutableForm::ExternalRef { .. } => "external reference",
    }
}

/// Assess state compatibility between a current and candidate
/// executable, optionally informed by an author-supplied migration
/// manifest.
///
/// Manifest-derived signals take priority: an author who supplies a
/// manifest is telling this analyzer directly what they expect, and
/// that declaration (itself `UNVERIFIED (author-supplied)`, see
/// [`MigrationManifest::to_evidence`]) is stronger evidence than the
/// structural fallback checks below it.
pub fn assess_state_compatibility(
    current_executable: &ExecutableForm,
    candidate_executable: &ExecutableForm,
    manifest: Option<&MigrationManifest>,
) -> StateCompatibilityResult {
    if let Some(manifest) = manifest {
        if manifest.declares_migration_function() {
            return result(
                StateCompatibility::RequiresMigration,
                "UNVERIFIED (author-supplied): migration manifest declares a migration function"
                    .to_string(),
            );
        }
        if manifest.declares_schema_change() {
            return result(
                StateCompatibility::RequiresMigration,
                "UNVERIFIED (author-supplied): migration manifest declares a schema version change with no migration function named"
                    .to_string(),
            );
        }
    }

    if std::mem::discriminant(current_executable) != std::mem::discriminant(candidate_executable) {
        return result(
            StateCompatibility::PotentiallyIncompatible,
            format!(
                "executable form changed ({} -> {}); this analyzer cannot compare storage access across different executable forms",
                executable_form_label(current_executable),
                executable_form_label(candidate_executable),
            ),
        );
    }

    match (current_executable, candidate_executable) {
        (ExecutableForm::StellarAsset, ExecutableForm::StellarAsset) => result(
            StateCompatibility::Compatible,
            "both executables are the native Stellar Asset Contract; there is no WASM storage-access change to evaluate".to_string(),
        ),
        (ExecutableForm::ExternalRef { .. }, ExecutableForm::ExternalRef { .. }) => result(
            StateCompatibility::NotDetermined,
            "both executables are external references; this analyzer cannot resolve or compare externally-owned executables".to_string(),
        ),
        (ExecutableForm::Wasm { hash: current_hash }, ExecutableForm::Wasm { hash: candidate_hash }) => {
            if current_hash == candidate_hash {
                result(
                    StateCompatibility::Compatible,
                    "candidate WASM hash is identical to the current executable; no executable change to evaluate".to_string(),
                )
            } else if manifest.is_none() {
                result(
                    StateCompatibility::NotDetermined,
                    "no migration manifest was supplied, and this analyzer does not reconstruct a contract's storage layout from raw WASM; static storage-access compatibility cannot be established from this evidence alone".to_string(),
                )
            } else {
                // A manifest that declares neither a migration function
                // nor a schema change is silence, not a positive
                // compatibility claim: the manifest is author-supplied
                // and UNVERIFIED (see MigrationManifest::to_evidence).
                // "the author did not describe a migration" must not be
                // read as "the analyzer established compatibility".
                result(
                    StateCompatibility::NotDetermined,
                    "the executable changed and a migration manifest was supplied, but it declares neither a migration function nor a schema change; this is an unverified author declaration of silence, not analyzer-established evidence of compatibility, so compatibility still cannot be established from this evidence alone".to_string(),
                )
            }
        }
        _ => unreachable!("executable form discriminants were already confirmed equal above"),
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::migration::MigrationFunction;

    fn wasm(hash: &str) -> ExecutableForm {
        ExecutableForm::Wasm {
            hash: hash.to_string(),
        }
    }

    #[test]
    fn identical_wasm_hash_is_compatible() {
        let result = assess_state_compatibility(&wasm("aa"), &wasm("aa"), None);
        assert_eq!(result.outcome, StateCompatibility::Compatible);
    }

    #[test]
    fn changed_wasm_hash_with_no_manifest_is_not_determined() {
        let result = assess_state_compatibility(&wasm("aa"), &wasm("bb"), None);
        assert_eq!(result.outcome, StateCompatibility::NotDetermined);
        assert!(result.reasons[0].contains("does not reconstruct"));
    }

    #[test]
    fn manifest_with_migration_function_requires_migration() {
        let manifest = MigrationManifest {
            migration_function: Some(MigrationFunction {
                name: "migrate".to_string(),
                one_time: true,
            }),
            ..Default::default()
        };
        let result = assess_state_compatibility(&wasm("aa"), &wasm("bb"), Some(&manifest));
        assert_eq!(result.outcome, StateCompatibility::RequiresMigration);
        assert!(result.reasons[0].contains("migration function"));
    }

    #[test]
    fn manifest_with_declared_schema_change_requires_migration() {
        let manifest = MigrationManifest {
            expected_old_schema: Some("1".to_string()),
            expected_new_schema: Some("2".to_string()),
            ..Default::default()
        };
        let result = assess_state_compatibility(&wasm("aa"), &wasm("bb"), Some(&manifest));
        assert_eq!(result.outcome, StateCompatibility::RequiresMigration);
        assert!(result.reasons[0].contains("schema version change"));
    }

    #[test]
    fn changed_hash_with_empty_manifest_is_not_determined_not_compatible() {
        // A manifest that declares nothing is an unverified author
        // declaration of silence, not proof of compatibility.
        let manifest = MigrationManifest::default();
        let result = assess_state_compatibility(&wasm("aa"), &wasm("bb"), Some(&manifest));
        assert_eq!(result.outcome, StateCompatibility::NotDetermined);
        assert!(result.reasons[0].contains("unverified author declaration"));
    }

    #[test]
    fn identical_hash_with_empty_manifest_is_still_compatible() {
        // No executable change at all; the manifest's silence is moot
        // because there is nothing for it to have declared about.
        let manifest = MigrationManifest::default();
        let result = assess_state_compatibility(&wasm("aa"), &wasm("aa"), Some(&manifest));
        assert_eq!(result.outcome, StateCompatibility::Compatible);
    }

    #[test]
    fn executable_form_change_is_potentially_incompatible() {
        let result = assess_state_compatibility(&wasm("aa"), &ExecutableForm::StellarAsset, None);
        assert_eq!(result.outcome, StateCompatibility::PotentiallyIncompatible);
        assert!(result.reasons[0].contains("WASM -> Stellar Asset"));
    }

    #[test]
    fn both_stellar_asset_is_compatible() {
        let result = assess_state_compatibility(
            &ExecutableForm::StellarAsset,
            &ExecutableForm::StellarAsset,
            None,
        );
        assert_eq!(result.outcome, StateCompatibility::Compatible);
    }

    #[test]
    fn both_external_ref_is_not_determined() {
        let external = ExecutableForm::ExternalRef {
            executable_owner_xdr_hex: "aa".to_string(),
            tag: "v1".to_string(),
        };
        let result = assess_state_compatibility(&external, &external, None);
        assert_eq!(result.outcome, StateCompatibility::NotDetermined);
    }

    #[test]
    fn manifest_signal_takes_priority_over_identical_hash() {
        // Even if the hash is unchanged, an author who explicitly
        // declares a migration function is telling us something the
        // hash comparison alone cannot see (e.g. a deliberate re-run).
        let manifest = MigrationManifest {
            migration_function: Some(MigrationFunction {
                name: "migrate".to_string(),
                one_time: false,
            }),
            ..Default::default()
        };
        let result = assess_state_compatibility(&wasm("aa"), &wasm("aa"), Some(&manifest));
        assert_eq!(result.outcome, StateCompatibility::RequiresMigration);
    }

    #[test]
    fn status_strings_match_the_documented_constants() {
        assert_eq!(StateCompatibility::Compatible.to_string(), "COMPATIBLE");
        assert_eq!(
            StateCompatibility::RequiresMigration.to_string(),
            "REQUIRES_MIGRATION"
        );
        assert_eq!(
            StateCompatibility::PotentiallyIncompatible.to_string(),
            "POTENTIALLY_INCOMPATIBLE"
        );
        assert_eq!(
            StateCompatibility::NotDetermined.to_string(),
            "NOT_DETERMINED"
        );
    }
}
