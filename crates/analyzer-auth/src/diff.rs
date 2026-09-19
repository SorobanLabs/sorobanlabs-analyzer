//! Rule-based comparison between two [`AuthorizationSurface`] values.
//!
//! Every difference is a specific, named variant of [`AuthorizationChange`],
//! never a generic "authorization changed" catch-all. Entrypoints are
//! matched between current and candidate by export name; a change that
//! cannot be reliably bound to one specific entrypoint (a module-level
//! fact, such as whether any recognized authorization primitive is
//! imported at all, or whether the custom-account `__check_auth` hook
//! exists) is reported as a global observation instead of an invented
//! per-entrypoint mapping.

use std::collections::BTreeMap;

use crate::extraction::AuthorizationSurface;

/// One detected authorization-surface difference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthorizationChange {
    /// The candidate exports an entrypoint the current executable does
    /// not.
    EntrypointAdded { name: String },
    /// The current executable exports an entrypoint the candidate does
    /// not.
    EntrypointRemoved { name: String },
    /// The entrypoint directly called an authorization primitive in the
    /// current executable, but calls none in the candidate: it appears
    /// no longer protected by a direct call. This does not prove the
    /// entrypoint is actually unprotected (the check may have moved
    /// into a helper function this analyzer does not trace
    /// transitively); see `analyzer_auth::extraction`'s module-level
    /// limitations.
    EntrypointAppearsUnprotected {
        name: String,
        previously_called: Vec<String>,
    },
    /// The entrypoint called no authorization primitive directly in the
    /// current executable, but calls one in the candidate.
    EntrypointGainedAuthorizationCall {
        name: String,
        now_called: Vec<String>,
    },
    /// The entrypoint directly calls an authorization primitive on both
    /// sides, but the specific set of primitives called differs (for
    /// example, `require_auth` replaced by `require_auth_for_args`).
    AuthorizationPathChanged {
        name: String,
        current: Vec<String>,
        candidate: Vec<String>,
    },
    /// Whether the module imports any recognized authorization
    /// primitive at all changed. Reported globally: this is a
    /// module-level fact, not bound to one entrypoint.
    ModuleAuthPrimitiveImportChanged { current: bool, candidate: bool },
    /// Whether the module exports the reserved `__check_auth` custom
    /// account authorization hook changed.
    CustomAuthHookChanged { current: bool, candidate: bool },
}

/// Every detected authorization-surface difference between a current
/// and candidate executable.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AuthorizationDiff {
    pub changes: Vec<AuthorizationChange>,
}

impl AuthorizationDiff {
    pub fn is_empty(&self) -> bool {
        self.changes.is_empty()
    }
}

/// Compare a current and candidate [`AuthorizationSurface`].
pub fn diff_authorization_surfaces(
    current: &AuthorizationSurface,
    candidate: &AuthorizationSurface,
) -> AuthorizationDiff {
    let mut changes = Vec::new();

    if current.imports_auth_primitive != candidate.imports_auth_primitive {
        changes.push(AuthorizationChange::ModuleAuthPrimitiveImportChanged {
            current: current.imports_auth_primitive,
            candidate: candidate.imports_auth_primitive,
        });
    }

    if current.has_custom_auth_hook != candidate.has_custom_auth_hook {
        changes.push(AuthorizationChange::CustomAuthHookChanged {
            current: current.has_custom_auth_hook,
            candidate: candidate.has_custom_auth_hook,
        });
    }

    let current_by_name: BTreeMap<&str, &Vec<String>> = current
        .entrypoints
        .iter()
        .map(|entry| (entry.export_name.as_str(), &entry.direct_calls))
        .collect();
    let candidate_by_name: BTreeMap<&str, &Vec<String>> = candidate
        .entrypoints
        .iter()
        .map(|entry| (entry.export_name.as_str(), &entry.direct_calls))
        .collect();

    let mut all_names: Vec<&str> = current_by_name
        .keys()
        .chain(candidate_by_name.keys())
        .copied()
        .collect();
    all_names.sort_unstable();
    all_names.dedup();

    for name in all_names {
        match (current_by_name.get(name), candidate_by_name.get(name)) {
            (Some(_), None) => changes.push(AuthorizationChange::EntrypointRemoved {
                name: name.to_string(),
            }),
            (None, Some(_)) => changes.push(AuthorizationChange::EntrypointAdded {
                name: name.to_string(),
            }),
            (Some(current_calls), Some(candidate_calls)) => {
                if current_calls == candidate_calls {
                    continue;
                }
                let change = match (current_calls.is_empty(), candidate_calls.is_empty()) {
                    (false, true) => AuthorizationChange::EntrypointAppearsUnprotected {
                        name: name.to_string(),
                        previously_called: (*current_calls).clone(),
                    },
                    (true, false) => AuthorizationChange::EntrypointGainedAuthorizationCall {
                        name: name.to_string(),
                        now_called: (*candidate_calls).clone(),
                    },
                    _ => AuthorizationChange::AuthorizationPathChanged {
                        name: name.to_string(),
                        current: (*current_calls).clone(),
                        candidate: (*candidate_calls).clone(),
                    },
                };
                changes.push(change);
            }
            (None, None) => unreachable!("name collected from one of the two maps"),
        }
    }

    AuthorizationDiff { changes }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::extraction::EntrypointAuthorization;

    fn surface(
        imports_auth_primitive: bool,
        has_custom_auth_hook: bool,
        entrypoints: Vec<(&str, Vec<&str>)>,
    ) -> AuthorizationSurface {
        AuthorizationSurface {
            imports_auth_primitive,
            has_custom_auth_hook,
            entrypoints: entrypoints
                .into_iter()
                .map(|(name, calls)| EntrypointAuthorization {
                    export_name: name.to_string(),
                    direct_calls: calls.into_iter().map(str::to_string).collect(),
                })
                .collect(),
        }
    }

    #[test]
    fn identical_surfaces_produce_no_changes() {
        let s = surface(true, false, vec![("transfer", vec!["require_auth"])]);
        let diff = diff_authorization_surfaces(&s, &s);
        assert!(diff.is_empty());
    }

    #[test]
    fn detects_entrypoint_added_and_removed() {
        let current = surface(false, false, vec![("burn", vec![])]);
        let candidate = surface(false, false, vec![("mint", vec![])]);
        let diff = diff_authorization_surfaces(&current, &candidate);
        assert!(diff
            .changes
            .contains(&AuthorizationChange::EntrypointRemoved {
                name: "burn".to_string()
            }));
        assert!(diff
            .changes
            .contains(&AuthorizationChange::EntrypointAdded {
                name: "mint".to_string()
            }));
    }

    #[test]
    fn detects_entrypoint_appears_unprotected() {
        let current = surface(true, false, vec![("transfer", vec!["require_auth"])]);
        let candidate = surface(true, false, vec![("transfer", vec![])]);
        let diff = diff_authorization_surfaces(&current, &candidate);
        assert_eq!(
            diff.changes,
            vec![AuthorizationChange::EntrypointAppearsUnprotected {
                name: "transfer".to_string(),
                previously_called: vec!["require_auth".to_string()],
            }]
        );
    }

    #[test]
    fn detects_entrypoint_gained_authorization_call() {
        let current = surface(false, false, vec![("transfer", vec![])]);
        let candidate = surface(true, false, vec![("transfer", vec!["require_auth"])]);
        let diff = diff_authorization_surfaces(&current, &candidate);
        assert!(diff
            .changes
            .contains(&AuthorizationChange::EntrypointGainedAuthorizationCall {
                name: "transfer".to_string(),
                now_called: vec!["require_auth".to_string()],
            }));
    }

    #[test]
    fn detects_authorization_path_changed() {
        let current = surface(true, false, vec![("transfer", vec!["require_auth"])]);
        let candidate = surface(
            true,
            false,
            vec![("transfer", vec!["require_auth_for_args"])],
        );
        let diff = diff_authorization_surfaces(&current, &candidate);
        assert_eq!(
            diff.changes,
            vec![AuthorizationChange::AuthorizationPathChanged {
                name: "transfer".to_string(),
                current: vec!["require_auth".to_string()],
                candidate: vec!["require_auth_for_args".to_string()],
            }]
        );
    }

    #[test]
    fn detects_module_level_import_and_custom_hook_changes() {
        let current = surface(false, false, vec![]);
        let candidate = surface(true, true, vec![]);
        let diff = diff_authorization_surfaces(&current, &candidate);
        assert!(diff
            .changes
            .contains(&AuthorizationChange::ModuleAuthPrimitiveImportChanged {
                current: false,
                candidate: true
            }));
        assert!(diff
            .changes
            .contains(&AuthorizationChange::CustomAuthHookChanged {
                current: false,
                candidate: true
            }));
    }
}
