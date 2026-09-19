//! Structural authorization-surface extraction for Soroban WASM
//! contracts.
//!
//! Soroban's authorization primitives, `require_auth` and
//! `require_auth_for_args`, are exposed to contract WASM as host
//! function imports. This module locates those imports and determines,
//! per exported entrypoint, whether that entrypoint's own function body
//! contains a *direct* call to one of them.
//!
//! # Evidence basis
//!
//! Unlike environment/contract metadata and the contract specification,
//! `require_auth`/`require_auth_for_args` are **not** declared in the
//! official host's own machine-readable interface definition
//! (`soroban-env-common/env.json` in
//! <https://github.com/stellar/rs-soroban-env>). This was checked
//! directly against both the `main` branch and the released `v28.0.2`
//! tag: neither contains a module or function related to
//! address/authorization anywhere in that file, even though
//! `stellar-protocol`'s CAP-0046-11 describes `require_auth` and
//! `require_auth_for_args` as host functions in prose. The exact
//! `(module, name)` pair a compiled contract imports for them is
//! therefore not confirmed from the primary interface-definition
//! source this analyzer otherwise relies on.
//!
//! What **is** confirmed: a third-party static analysis tool built
//! specifically for Soroban contracts
//! (<https://github.com/Soroban-Static-Analysis-Fuzzing-Toolkit/Soroban-Static-Core>,
//! issue #1, "SOR-101 (wasm) only checks direct calls for
//! require_auth") detects these calls in compiled contract WASM by
//! matching an import whose *name* is (or, in that tool, ends with)
//! the literal string `require_auth`. This module uses the same
//! practical signal: an imported function whose name is exactly
//! `require_auth` or `require_auth_for_args`, in any import module.
//! Treat this as `LOGICALLY COVERED` against that corroborating
//! third-party evidence, not `VERIFIED` against the primary host
//! interface definition, which does not describe these two imports at
//! all.
//!
//! # Limitations
//!
//! - Only **direct** calls from an exported function's own body are
//!   detected (a single level of the call graph). A call that reaches
//!   `require_auth` transitively, through a helper function, is not
//!   detected as protecting the entrypoint that calls the helper. The
//!   third-party tool referenced above has the identical limitation, for
//!   the same underlying reason: resolving it requires a full call-graph
//!   analysis (including through `call_indirect`) that this module does
//!   not attempt, rather than risk an unreliable transitive claim.
//! - This module never infers a principal (which `Address`) from a
//!   variable name or argument position; it only establishes that a
//!   call to the primitive exists, never which address was passed.
//! - The presence of a `require_auth`-family import anywhere in the
//!   module is not, by itself, evidence that any particular entrypoint
//!   is protected; only a direct call from that entrypoint's own body
//!   is treated as such.

use std::collections::{BTreeMap, BTreeSet};

use analyzer_core::{AnalyzerError, UnsupportedArtifactError};
use wasmparser::{ExternalKind, Operator, Parser, Payload, TypeRef};

/// The host import names this analyzer recognizes as Soroban
/// authorization primitives. See the module-level docs for the
/// evidence behind this specific list.
pub const AUTH_IMPORT_NAMES: [&str; 2] = ["require_auth", "require_auth_for_args"];

/// One exported entrypoint's directly-observed authorization calls.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntrypointAuthorization {
    pub export_name: String,
    /// The distinct authorization-primitive names this entrypoint's own
    /// function body directly calls. Empty if none (which is not, by
    /// itself, proof the entrypoint is unprotected: see the module-level
    /// limitations on transitive calls).
    pub direct_calls: Vec<String>,
}

impl EntrypointAuthorization {
    /// True if this entrypoint's own body directly calls a recognized
    /// authorization primitive.
    pub fn has_direct_auth_call(&self) -> bool {
        !self.direct_calls.is_empty()
    }
}

/// The reserved export name Soroban's host calls to delegate
/// authentication/authorization to a custom account contract, every
/// time `require_auth`/`require_auth_for_args` is invoked for that
/// contract's address. Verified against the official Stellar developer
/// documentation
/// (<https://developers.stellar.org/docs/learn/fundamentals/contract-development/authorization>):
/// "`__check_auth` is a reserved function and can only be called by the
/// Soroban environment in response to a call to `require_auth`."
pub const CUSTOM_ACCOUNT_CHECK_AUTH_EXPORT: &str = "__check_auth";

/// The authorization surface extracted from one WASM module.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AuthorizationSurface {
    /// True if the module imports at least one recognized authorization
    /// primitive, regardless of whether any code path reaches it.
    pub imports_auth_primitive: bool,
    /// True if the module exports the reserved `__check_auth` function,
    /// implementing a custom account authorization hook.
    pub has_custom_auth_hook: bool,
    /// Every exported function, each with its direct authorization
    /// calls, sorted by export name for deterministic output.
    pub entrypoints: Vec<EntrypointAuthorization>,
}

impl AuthorizationSurface {
    /// Look up one exported entrypoint's authorization observation by
    /// name.
    pub fn entrypoint(&self, name: &str) -> Option<&EntrypointAuthorization> {
        self.entrypoints
            .iter()
            .find(|entry| entry.export_name == name)
    }
}

/// Extract the [`AuthorizationSurface`] from a WASM module's bytes.
///
/// This assumes `bytes` is already-known-valid WASM (see
/// `analyzer_executable::validate_generic_wasm`); it re-parses the
/// module's payload stream independently and reports a parse failure as
/// an [`UnsupportedArtifactError`] rather than panicking.
pub fn extract_authorization_surface(bytes: &[u8]) -> Result<AuthorizationSurface, AnalyzerError> {
    let mut imported_function_count: u32 = 0;
    let mut auth_import_names: BTreeMap<u32, String> = BTreeMap::new();
    let mut export_names_by_function_index: BTreeMap<u32, Vec<String>> = BTreeMap::new();
    let mut local_function_position: u32 = 0;
    let mut direct_calls_by_function_index: BTreeMap<u32, BTreeSet<String>> = BTreeMap::new();

    for payload in Parser::new(0).parse_all(bytes) {
        let payload = payload.map_err(|source| {
            UnsupportedArtifactError::new(format!("failed to parse wasm module: {source}"))
        })?;

        match payload {
            Payload::ImportSection(reader) => {
                for import in reader {
                    let import = import.map_err(|source| {
                        UnsupportedArtifactError::new(format!(
                            "failed to parse wasm import section: {source}"
                        ))
                    })?;
                    if let TypeRef::Func(_) = import.ty {
                        let index = imported_function_count;
                        if AUTH_IMPORT_NAMES.contains(&import.name) {
                            auth_import_names.insert(index, import.name.to_string());
                        }
                        imported_function_count += 1;
                    }
                }
            }
            Payload::ExportSection(reader) => {
                for export in reader {
                    let export = export.map_err(|source| {
                        UnsupportedArtifactError::new(format!(
                            "failed to parse wasm export section: {source}"
                        ))
                    })?;
                    if export.kind == ExternalKind::Func {
                        export_names_by_function_index
                            .entry(export.index)
                            .or_default()
                            .push(export.name.to_string());
                    }
                }
            }
            Payload::CodeSectionEntry(body) => {
                let function_index = imported_function_count + local_function_position;
                local_function_position += 1;

                if !auth_import_names.is_empty() {
                    let operators = body.get_operators_reader().map_err(|source| {
                        UnsupportedArtifactError::new(format!(
                            "failed to read wasm function body: {source}"
                        ))
                    })?;
                    let mut called = BTreeSet::new();
                    for operator in operators {
                        let operator = operator.map_err(|source| {
                            UnsupportedArtifactError::new(format!(
                                "failed to decode wasm function body instructions: {source}"
                            ))
                        })?;
                        if let Operator::Call {
                            function_index: callee,
                        } = operator
                        {
                            if let Some(name) = auth_import_names.get(&callee) {
                                called.insert(name.clone());
                            }
                        }
                    }
                    if !called.is_empty() {
                        direct_calls_by_function_index.insert(function_index, called);
                    }
                }
            }
            _ => {}
        }
    }

    let mut entrypoints = Vec::new();
    for (function_index, names) in &export_names_by_function_index {
        let direct_calls: Vec<String> = direct_calls_by_function_index
            .get(function_index)
            .map(|set| set.iter().cloned().collect())
            .unwrap_or_default();
        for export_name in names {
            entrypoints.push(EntrypointAuthorization {
                export_name: export_name.clone(),
                direct_calls: direct_calls.clone(),
            });
        }
    }
    entrypoints.sort_by(|a, b| a.export_name.cmp(&b.export_name));
    let has_custom_auth_hook = entrypoints
        .iter()
        .any(|entry| entry.export_name == CUSTOM_ACCOUNT_CHECK_AUTH_EXPORT);

    Ok(AuthorizationSurface {
        imports_auth_primitive: !auth_import_names.is_empty(),
        has_custom_auth_hook,
        entrypoints,
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    const MINIMAL_HEADER: &[u8] = &[0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00];

    fn leb128(out: &mut Vec<u8>, mut value: u64) {
        loop {
            let byte = (value & 0x7f) as u8;
            value >>= 7;
            if value == 0 {
                out.push(byte);
                break;
            }
            out.push(byte | 0x80);
        }
    }

    /// Build a minimal module with:
    /// - one imported function (module "a", name `import_name`), type () -> ()
    /// - one local function, type () -> (), exported as `export_name`
    /// - the local function's body optionally calls the import (function
    ///   index 0) before returning, when `calls_import` is true.
    fn module_with_import_and_export(
        import_name: &str,
        export_name: &str,
        calls_import: bool,
    ) -> Vec<u8> {
        let mut module = MINIMAL_HEADER.to_vec();

        // Type section: 1 type, () -> ()
        module.extend_from_slice(&[0x01, 0x04, 0x01, 0x60, 0x00, 0x00]);

        // Import section: "a".import_name : type 0
        let mut import_body = Vec::new();
        leb128(&mut import_body, 1); // 1 import
        leb128(&mut import_body, 1); // module name len
        import_body.push(b'a');
        leb128(&mut import_body, import_name.len() as u64);
        import_body.extend_from_slice(import_name.as_bytes());
        import_body.push(0x00); // import kind: func
        leb128(&mut import_body, 0); // type index
        module.push(0x02); // import section id
        leb128(&mut module, import_body.len() as u64);
        module.extend_from_slice(&import_body);

        // Function section: 1 local function of type 0
        module.extend_from_slice(&[0x03, 0x02, 0x01, 0x00]);

        // Export section: export local function (index 1: 0 is the import) as export_name
        let mut export_body = Vec::new();
        leb128(&mut export_body, 1); // 1 export
        leb128(&mut export_body, export_name.len() as u64);
        export_body.extend_from_slice(export_name.as_bytes());
        export_body.push(0x00); // export kind: func
        leb128(&mut export_body, 1); // function index 1 (local)
        module.push(0x07); // export section id
        leb128(&mut module, export_body.len() as u64);
        module.extend_from_slice(&export_body);

        // Code section: 1 function body, no locals, optional call to
        // function index 0 (the import), then end.
        let mut function_body = vec![0x00]; // 0 local declarations
        if calls_import {
            function_body.push(0x10); // call opcode
            leb128(&mut function_body, 0); // callee function index 0
        }
        function_body.push(0x0b); // end

        let mut code_body = Vec::new();
        leb128(&mut code_body, 1); // 1 function body
        leb128(&mut code_body, function_body.len() as u64);
        code_body.extend_from_slice(&function_body);

        module.push(0x0a); // code section id
        leb128(&mut module, code_body.len() as u64);
        module.extend_from_slice(&code_body);

        module
    }

    #[test]
    fn module_with_no_auth_import_reports_no_surface() {
        let module = module_with_import_and_export("unrelated_import", "hello", false);
        let surface = extract_authorization_surface(&module).unwrap();
        assert!(!surface.imports_auth_primitive);
        assert!(!surface.entrypoint("hello").unwrap().has_direct_auth_call());
    }

    #[test]
    fn entrypoint_directly_calling_require_auth_is_detected() {
        let module = module_with_import_and_export("require_auth", "transfer", true);
        let surface = extract_authorization_surface(&module).unwrap();

        assert!(surface.imports_auth_primitive);
        let entry = surface.entrypoint("transfer").unwrap();
        assert!(entry.has_direct_auth_call());
        assert_eq!(entry.direct_calls, vec!["require_auth".to_string()]);
    }

    #[test]
    fn imported_auth_primitive_not_called_is_not_attributed_to_entrypoint() {
        // The module imports require_auth but the exported function
        // never calls it: presence of the import alone must not be
        // treated as protecting the entrypoint.
        let module = module_with_import_and_export("require_auth", "unprotected", false);
        let surface = extract_authorization_surface(&module).unwrap();

        assert!(surface.imports_auth_primitive);
        assert!(!surface
            .entrypoint("unprotected")
            .unwrap()
            .has_direct_auth_call());
    }

    #[test]
    fn require_auth_for_args_is_also_recognized() {
        let module = module_with_import_and_export("require_auth_for_args", "swap", true);
        let surface = extract_authorization_surface(&module).unwrap();
        let entry = surface.entrypoint("swap").unwrap();
        assert_eq!(
            entry.direct_calls,
            vec!["require_auth_for_args".to_string()]
        );
    }

    #[test]
    fn malformed_bytes_return_structured_error_not_panic() {
        let result = extract_authorization_surface(b"not wasm at all");
        assert!(matches!(result, Err(AnalyzerError::UnsupportedArtifact(_))));
    }

    #[test]
    fn module_with_no_exports_has_empty_entrypoint_list() {
        let mut module = MINIMAL_HEADER.to_vec();
        module.extend_from_slice(&[0x01, 0x04, 0x01, 0x60, 0x00, 0x00]);
        let surface = extract_authorization_surface(&module).unwrap();
        assert!(surface.entrypoints.is_empty());
        assert!(!surface.imports_auth_primitive);
    }

    #[test]
    fn exported_check_auth_is_detected_as_custom_auth_hook() {
        let module =
            module_with_import_and_export("unrelated", CUSTOM_ACCOUNT_CHECK_AUTH_EXPORT, false);
        let surface = extract_authorization_surface(&module).unwrap();
        assert!(surface.has_custom_auth_hook);
    }

    #[test]
    fn module_without_check_auth_export_has_no_custom_auth_hook() {
        let module = module_with_import_and_export("unrelated", "transfer", false);
        let surface = extract_authorization_surface(&module).unwrap();
        assert!(!surface.has_custom_auth_hook);
    }
}
