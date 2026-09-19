//! Soroban contract specification (`contractspecv0`) parsing.
//!
//! Soroban contracts carry a custom WASM section named `contractspecv0`
//! containing the serialized bytes of a stream of
//! [`stellar_xdr::ScSpecEntry`] XDR values, back to back with no
//! additional framing, exactly like [`crate::environment_meta`] and
//! [`crate::contract_meta`]. Each entry describes one function, one
//! user-defined type (struct, union, enum, or error enum), or one
//! event.
//!
//! This module only decodes the raw `ScSpecEntry` stream and exposes it
//! as-is; it does not normalize types or names into an
//! analyzer-specific model. Deterministic normalization (independent of
//! the raw XDR structure) is built on top of this in
//! `crate::interface`.

use std::io::Cursor;

use analyzer_core::{AnalyzerError, UnsupportedArtifactError};
use stellar_xdr::{Limited, Limits, ReadXdr, ScSpecEntry};
use wasmparser::{Parser, Payload};

/// The WASM custom section name Soroban uses for the contract
/// specification.
pub const CONTRACT_SPEC_SECTION: &str = "contractspecv0";

/// Bound on recursive XDR nesting depth while decoding. Contract
/// specifications can nest types (for example `Vec<Option<MyStruct>>`);
/// this bound is generous relative to any specification a real contract
/// would declare, and exists purely as a defensive limit against a
/// hostile artifact.
const XDR_READ_DEPTH_LIMIT: u32 = 64;

/// The result of looking for and decoding `contractspecv0` in a WASM
/// module.
#[derive(Debug, Clone, Default)]
pub struct ContractSpecReport {
    /// How many WASM custom sections named `contractspecv0` were found.
    section_occurrences: usize,
    /// Every entry successfully decoded, across all occurrences, in
    /// encounter order.
    entries: Vec<ScSpecEntry>,
    /// One message per structural decode problem encountered. A
    /// malformed entry stops decoding of the remainder of that
    /// occurrence's stream (XDR framing errors cannot be resynced past).
    decode_errors: Vec<String>,
}

impl ContractSpecReport {
    /// True if at least one `contractspecv0` section was found.
    pub fn is_present(&self) -> bool {
        self.section_occurrences > 0
    }

    /// How many occurrences of the section were found.
    pub fn section_occurrences(&self) -> usize {
        self.section_occurrences
    }

    /// True if more than one `contractspecv0` section was found.
    pub fn has_duplicate_sections(&self) -> bool {
        self.section_occurrences > 1
    }

    /// Every specification entry successfully decoded, in encounter
    /// order.
    pub fn entries(&self) -> &[ScSpecEntry] {
        &self.entries
    }

    /// Diagnostic messages for structural decode problems encountered.
    pub fn decode_errors(&self) -> &[String] {
        &self.decode_errors
    }

    /// True if the section is present and every occurrence decoded
    /// without a structural error.
    pub fn decoded_cleanly(&self) -> bool {
        self.is_present() && self.decode_errors.is_empty()
    }
}

/// Parse the `contractspecv0` custom section(s), if any, out of a WASM
/// module.
///
/// This assumes `bytes` is already-known-valid WASM (see
/// [`crate::validation::validate_generic_wasm`]); it re-parses the
/// module's payload stream independently and reports a parse failure as
/// an [`UnsupportedArtifactError`] rather than panicking.
pub fn parse_contract_spec(bytes: &[u8]) -> Result<ContractSpecReport, AnalyzerError> {
    let mut report = ContractSpecReport::default();

    for payload in Parser::new(0).parse_all(bytes) {
        let payload = payload.map_err(|source| {
            UnsupportedArtifactError::new(format!("failed to parse wasm module: {source}"))
        })?;

        let Payload::CustomSection(section) = payload else {
            continue;
        };

        if section.name() != CONTRACT_SPEC_SECTION {
            continue;
        }

        report.section_occurrences += 1;

        let limits = Limits {
            depth: XDR_READ_DEPTH_LIMIT,
            len: section.data().len(),
        };
        let mut reader = Limited::new(Cursor::new(section.data()), limits);

        for entry in ScSpecEntry::read_xdr_iter(&mut reader) {
            match entry {
                Ok(entry) => report.entries.push(entry),
                Err(source) => {
                    report
                        .decode_errors
                        .push(format!("malformed contractspecv0 entry: {source}"));
                    break;
                }
            }
        }
    }

    Ok(report)
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use stellar_xdr::{
        ScSpecFunctionInputV0, ScSpecFunctionV0, ScSpecTypeDef, ScSymbol, StringM, VecM, WriteXdr,
    };

    const MINIMAL_HEADER: &[u8] = &[0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00];

    fn custom_section(name: &str, data: &[u8]) -> Vec<u8> {
        let mut name_bytes = Vec::new();
        write_leb128(&mut name_bytes, name.len() as u64);
        name_bytes.extend_from_slice(name.as_bytes());

        let mut content = name_bytes;
        content.extend_from_slice(data);

        let mut section = vec![0x00];
        write_leb128(&mut section, content.len() as u64);
        section.extend_from_slice(&content);
        section
    }

    fn write_leb128(out: &mut Vec<u8>, mut value: u64) {
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

    fn symbol(s: &str) -> ScSymbol {
        ScSymbol(StringM::try_from(s).unwrap())
    }

    fn simple_function(name: &str) -> ScSpecEntry {
        ScSpecEntry::FunctionV0(ScSpecFunctionV0 {
            doc: StringM::default(),
            name: symbol(name),
            inputs: VecM::try_from(vec![ScSpecFunctionInputV0 {
                doc: StringM::default(),
                name: StringM::try_from("amount").unwrap(),
                type_: ScSpecTypeDef::I128,
            }])
            .unwrap(),
            outputs: VecM::default(),
        })
    }

    fn module_with_spec(entries: &[ScSpecEntry]) -> Vec<u8> {
        let mut data = Vec::new();
        for entry in entries {
            data.extend_from_slice(&entry.to_xdr(Limits::none()).unwrap());
        }
        let mut module = MINIMAL_HEADER.to_vec();
        module.extend_from_slice(&custom_section(CONTRACT_SPEC_SECTION, &data));
        module
    }

    #[test]
    fn absent_section_is_reported_as_absent() {
        let report = parse_contract_spec(MINIMAL_HEADER).unwrap();
        assert!(!report.is_present());
        assert!(report.entries().is_empty());
    }

    #[test]
    fn function_entry_decodes() {
        let module = module_with_spec(&[simple_function("transfer")]);
        let report = parse_contract_spec(&module).unwrap();

        assert!(report.decoded_cleanly());
        assert_eq!(report.entries().len(), 1);
        match &report.entries()[0] {
            ScSpecEntry::FunctionV0(function) => {
                assert_eq!(function.name.to_utf8_string().unwrap(), "transfer");
                assert_eq!(function.inputs.len(), 1);
            }
            other => panic!("expected FunctionV0, got {other:?}"),
        }
    }

    #[test]
    fn multiple_entries_decode_in_order() {
        let module = module_with_spec(&[simple_function("mint"), simple_function("burn")]);
        let report = parse_contract_spec(&module).unwrap();

        assert_eq!(report.entries().len(), 2);
        let names: Vec<String> = report
            .entries()
            .iter()
            .map(|entry| match entry {
                ScSpecEntry::FunctionV0(function) => function.name.to_utf8_string().unwrap(),
                _ => unreachable!(),
            })
            .collect();
        assert_eq!(names, vec!["mint".to_string(), "burn".to_string()]);
    }

    #[test]
    fn malformed_section_bytes_are_reported_not_panicked() {
        let mut module = MINIMAL_HEADER.to_vec();
        module.extend_from_slice(&custom_section(CONTRACT_SPEC_SECTION, &[0xff, 0xff, 0xff]));
        let report = parse_contract_spec(&module).unwrap();

        assert!(report.is_present());
        assert!(!report.decoded_cleanly());
        assert!(report.entries().is_empty());
    }

    #[test]
    fn unrelated_custom_sections_are_ignored() {
        let mut module = MINIMAL_HEADER.to_vec();
        module.extend_from_slice(&custom_section("producers", b"irrelevant"));
        let report = parse_contract_spec(&module).unwrap();
        assert!(!report.is_present());
    }
}
