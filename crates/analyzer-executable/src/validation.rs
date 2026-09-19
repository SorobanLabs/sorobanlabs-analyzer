//! WASM structural validation, in two independent stages.
//!
//! # Stage 1: generic WebAssembly structural validity
//!
//! [`validate_generic_wasm`] confirms that a byte sequence is a
//! well-formed WebAssembly module per the core WebAssembly
//! specification, using [`wasmparser::Validator`]. This says nothing
//! about Soroban: a structurally valid module may still be something
//! Soroban's host will refuse to run (for example, a WASI binary, a
//! component, or a module using memory64).
//!
//! # Stage 2: Soroban structural compatibility
//!
//! [`check_soroban_structural_compatibility`] inspects an
//! already-generic-valid module for the specific structural constructs
//! the official Soroban host rejects. This is not an operational
//! error: the module parses fine, it just is not something the Soroban
//! host executes. The result is a typed report the caller can inspect,
//! never a silent pass/fail.
//!
//! The Soroban-specific rejections implemented here (start section,
//! component-model sections, exception-handling tags, memory64, shared
//! memory) are taken from the official host's own module-parsing logic
//! in `soroban-env-host/src/vm/parsed_module.rs`
//! (<https://github.com/stellar/rs-soroban-env>), which core-Wasm the
//! host will run through Wasmi. This module does not reproduce Soroban's
//! full validation (host-import resolution, function/local/argument
//! limits, and other checks require environment metadata this stage
//! does not have); it only encodes the structural rejections that can
//! be established from the module's shape alone.
//!
//! Both stages treat the input as untrusted: parsing is bounded by
//! [`MAX_WASM_BYTES`] before `wasmparser` is invoked at all, and
//! `wasmparser` itself is responsible for bounded, non-recursive
//! parsing of the module body.

use analyzer_core::{AnalyzerError, UnsupportedArtifactError};
use wasmparser::{Parser, Payload, TypeRef, Validator};

/// Defensive upper bound on artifact size before this analyzer will
/// attempt to parse it as WASM. This is an analyzer-imposed safety
/// bound only; it is not a claim about any Soroban network's actual
/// contract size limit, which is a protocol configuration value this
/// module does not have access to.
pub const MAX_WASM_BYTES: usize = 64 * 1024 * 1024;

/// A specific structural construct the official Soroban host does not
/// support, even in an otherwise structurally valid WebAssembly module.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SorobanStructuralViolation {
    /// The bytes do not parse as a core WebAssembly module (for
    /// example, they parse as a WebAssembly component instead).
    NotCoreWasm,
    /// The module declares a start function. Soroban's host does not
    /// invoke one.
    StartFunctionPresent,
    /// The module contains component-model sections (module, instance,
    /// component, core-type, or related sections).
    ComponentModelSection,
    /// The module declares an exception-handling tag (the
    /// exception-handling proposal's tag section, or a tag import).
    TagPresent,
    /// The module declares or imports 64-bit (`memory64`) memory.
    Memory64Used,
    /// The module declares or imports shared (threads-proposal) memory.
    SharedMemoryUsed,
}

impl std::fmt::Display for SorobanStructuralViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let message = match self {
            Self::NotCoreWasm => "module is not a core WebAssembly module",
            Self::StartFunctionPresent => "module declares a start function",
            Self::ComponentModelSection => "module contains component-model sections",
            Self::TagPresent => "module declares an exception-handling tag",
            Self::Memory64Used => "module uses 64-bit (memory64) memory",
            Self::SharedMemoryUsed => "module uses shared (threads-proposal) memory",
        };
        f.write_str(message)
    }
}

/// The result of checking a structurally valid WASM module against
/// Soroban's known structural restrictions.
///
/// An empty violation list means no *structural* incompatibility was
/// found; it is not a claim that the module is a supported Soroban
/// contract executable (environment metadata, contract spec, and host
/// import checks are separate, later stages).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SorobanStructuralReport {
    violations: Vec<SorobanStructuralViolation>,
}

impl SorobanStructuralReport {
    /// True if no known Soroban structural restriction was violated.
    pub fn is_compatible(&self) -> bool {
        self.violations.is_empty()
    }

    /// The distinct violations found, in the order first observed.
    pub fn violations(&self) -> &[SorobanStructuralViolation] {
        &self.violations
    }

    fn push(&mut self, violation: SorobanStructuralViolation) {
        if !self.violations.contains(&violation) {
            self.violations.push(violation);
        }
    }
}

/// Confirm that `bytes` are a structurally well-formed WebAssembly
/// module (core module or component) per the core/component WebAssembly
/// specifications, using `wasmparser`'s default feature set.
///
/// # Errors
///
/// Returns [`UnsupportedArtifactError`] if `bytes` exceed
/// [`MAX_WASM_BYTES`], or if `wasmparser` rejects the bytes as
/// malformed.
pub fn validate_generic_wasm(bytes: &[u8]) -> Result<(), AnalyzerError> {
    if bytes.len() > MAX_WASM_BYTES {
        return Err(UnsupportedArtifactError::new(format!(
            "artifact exceeds the analyzer's {MAX_WASM_BYTES}-byte parsing bound"
        ))
        .into());
    }

    Validator::new()
        .validate_all(bytes)
        .map(|_types| ())
        .map_err(|source| {
            UnsupportedArtifactError::new(format!(
                "artifact is not a structurally valid WASM module: {source}"
            ))
        })?;

    Ok(())
}

/// Check a structurally valid WASM module (see [`validate_generic_wasm`])
/// against the structural constructs the official Soroban host rejects.
///
/// This performs its own bounded, non-validating walk of the module's
/// payloads; it does not assume the module has already been confirmed
/// valid, but callers should normally call [`validate_generic_wasm`]
/// first so parse errors are reported as generic WASM problems rather
/// than surfacing here.
///
/// # Errors
///
/// Returns [`UnsupportedArtifactError`] if `bytes` exceed
/// [`MAX_WASM_BYTES`], or if the module cannot be parsed at all.
pub fn check_soroban_structural_compatibility(
    bytes: &[u8],
) -> Result<SorobanStructuralReport, AnalyzerError> {
    if bytes.len() > MAX_WASM_BYTES {
        return Err(UnsupportedArtifactError::new(format!(
            "artifact exceeds the analyzer's {MAX_WASM_BYTES}-byte parsing bound"
        ))
        .into());
    }

    let mut report = SorobanStructuralReport::default();

    if !Parser::is_core_wasm(bytes) {
        report.push(SorobanStructuralViolation::NotCoreWasm);
        return Ok(report);
    }

    for payload in Parser::new(0).parse_all(bytes) {
        let payload = payload.map_err(|source| {
            UnsupportedArtifactError::new(format!("failed to parse wasm module: {source}"))
        })?;

        match payload {
            Payload::StartSection { .. } => {
                report.push(SorobanStructuralViolation::StartFunctionPresent);
            }
            Payload::TagSection(_) => {
                report.push(SorobanStructuralViolation::TagPresent);
            }
            Payload::ModuleSection { .. }
            | Payload::InstanceSection(_)
            | Payload::CoreTypeSection(_)
            | Payload::ComponentSection { .. }
            | Payload::ComponentInstanceSection(_)
            | Payload::ComponentAliasSection(_)
            | Payload::ComponentTypeSection(_)
            | Payload::ComponentCanonicalSection(_)
            | Payload::ComponentStartSection { .. }
            | Payload::ComponentImportSection(_)
            | Payload::ComponentExportSection(_) => {
                report.push(SorobanStructuralViolation::ComponentModelSection);
            }
            Payload::MemorySection(reader) => {
                for memory in reader {
                    let memory = memory.map_err(|source| {
                        UnsupportedArtifactError::new(format!(
                            "failed to parse wasm memory section: {source}"
                        ))
                    })?;
                    if memory.memory64 {
                        report.push(SorobanStructuralViolation::Memory64Used);
                    }
                    if memory.shared {
                        report.push(SorobanStructuralViolation::SharedMemoryUsed);
                    }
                }
            }
            Payload::ImportSection(reader) => {
                for import in reader {
                    let import = import.map_err(|source| {
                        UnsupportedArtifactError::new(format!(
                            "failed to parse wasm import section: {source}"
                        ))
                    })?;
                    match import.ty {
                        TypeRef::Memory(memory) => {
                            if memory.memory64 {
                                report.push(SorobanStructuralViolation::Memory64Used);
                            }
                            if memory.shared {
                                report.push(SorobanStructuralViolation::SharedMemoryUsed);
                            }
                        }
                        TypeRef::Tag(_) => {
                            report.push(SorobanStructuralViolation::TagPresent);
                        }
                        TypeRef::Func(_) | TypeRef::Table(_) | TypeRef::Global(_) => {}
                    }
                }
            }
            _ => {}
        }
    }

    Ok(report)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    const MINIMAL_VALID: &[u8] = &[0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00];

    #[test]
    fn minimal_module_is_generically_valid() {
        validate_generic_wasm(MINIMAL_VALID).unwrap();
    }

    #[test]
    fn minimal_module_is_soroban_structurally_compatible() {
        let report = check_soroban_structural_compatibility(MINIMAL_VALID).unwrap();
        assert!(report.is_compatible());
        assert!(report.violations().is_empty());
    }

    #[test]
    fn truncated_bytes_are_not_generically_valid() {
        let result = validate_generic_wasm(&[0x00, 0x61, 0x73, 0x6d]);
        assert!(matches!(result, Err(AnalyzerError::UnsupportedArtifact(_))));
    }

    #[test]
    fn arbitrary_bytes_are_not_generically_valid() {
        let result = validate_generic_wasm(b"this is not wasm at all");
        assert!(matches!(result, Err(AnalyzerError::UnsupportedArtifact(_))));
    }

    #[test]
    fn oversized_input_is_rejected_before_parsing() {
        let oversized = vec![0u8; MAX_WASM_BYTES + 1];
        let result = validate_generic_wasm(&oversized);
        assert!(matches!(result, Err(AnalyzerError::UnsupportedArtifact(_))));
    }

    // Module with a start section: (module (func $f) (start $f))
    // encoded by hand: type section (empty func type), function
    // section (1 function of that type), start section (function 0),
    // code section (1 empty function body).
    fn module_with_start_section() -> Vec<u8> {
        let mut bytes = MINIMAL_VALID.to_vec();
        // Type section: 1 type, () -> ()
        bytes.extend_from_slice(&[0x01, 0x04, 0x01, 0x60, 0x00, 0x00]);
        // Function section: 1 function, type index 0
        bytes.extend_from_slice(&[0x03, 0x02, 0x01, 0x00]);
        // Start section: function index 0
        bytes.extend_from_slice(&[0x08, 0x01, 0x00]);
        // Code section: 1 function body, empty locals, single `end`
        bytes.extend_from_slice(&[0x0a, 0x04, 0x01, 0x02, 0x00, 0x0b]);
        bytes
    }

    #[test]
    fn start_section_is_generically_valid_but_soroban_incompatible() {
        let bytes = module_with_start_section();
        validate_generic_wasm(&bytes).unwrap();

        let report = check_soroban_structural_compatibility(&bytes).unwrap();
        assert!(!report.is_compatible());
        assert_eq!(
            report.violations(),
            &[SorobanStructuralViolation::StartFunctionPresent]
        );
    }

    // Module declaring one 32-bit, non-shared memory (0 initial pages,
    // no maximum): a compatible baseline used to isolate the memory64
    // and shared-memory checks from each other.
    fn module_with_memory(flags: u8, initial: u8, maximum: Option<u8>) -> Vec<u8> {
        let mut bytes = MINIMAL_VALID.to_vec();
        let mut memory_section_body = vec![0x01]; // 1 memory
        memory_section_body.push(flags);
        memory_section_body.push(initial);
        if let Some(max) = maximum {
            memory_section_body.push(max);
        }
        bytes.push(0x05); // memory section id
        bytes.push(u8::try_from(memory_section_body.len()).unwrap());
        bytes.extend_from_slice(&memory_section_body);
        bytes
    }

    #[test]
    fn plain_memory_is_soroban_compatible() {
        // flags = 0x00: no maximum, not shared, 32-bit.
        let bytes = module_with_memory(0x00, 0x01, None);
        validate_generic_wasm(&bytes).unwrap();
        let report = check_soroban_structural_compatibility(&bytes).unwrap();
        assert!(report.is_compatible());
    }

    #[test]
    fn shared_memory_is_soroban_incompatible() {
        // flags = 0x03: has maximum (required for shared) and shared.
        let bytes = module_with_memory(0x03, 0x01, Some(0x02));
        validate_generic_wasm(&bytes).unwrap();
        let report = check_soroban_structural_compatibility(&bytes).unwrap();
        assert!(!report.is_compatible());
        assert_eq!(
            report.violations(),
            &[SorobanStructuralViolation::SharedMemoryUsed]
        );
    }

    #[test]
    fn component_bytes_are_not_core_wasm() {
        // Component binary: magic + version 0x0a 0x00, layer 0x01 0x00
        // (the second half of the 8-byte header distinguishes a
        // component from a core module in the binary format).
        let mut bytes = vec![0x00, 0x61, 0x73, 0x6d, 0x0a, 0x00, 0x01, 0x00];
        // No further sections; this is enough for `is_core_wasm` to
        // classify the bytes as non-core. Padding is irrelevant because
        // we only exercise the Soroban structural check, not the
        // generic validator, for this case.
        bytes.truncate(8);
        let report = check_soroban_structural_compatibility(&bytes).unwrap();
        assert!(!report.is_compatible());
        assert_eq!(
            report.violations(),
            &[SorobanStructuralViolation::NotCoreWasm]
        );
    }
}
