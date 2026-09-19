//! Soroban contract metadata (`contractmetav0`) parsing.
//!
//! Soroban contracts may carry a custom WASM section named
//! `contractmetav0` containing the serialized bytes of a stream of
//! [`stellar_xdr::ScMetaEntry`] XDR values, back to back with no
//! additional framing, mirroring how [`crate::environment_meta`] reads
//! `contractenvmetav0`. `ScMetaEntry` today has a single case,
//! `SC_META_V0`, wrapping an [`stellar_xdr::ScMetaV0`] key/value string
//! pair. Contract authors and SDKs use this section to record
//! informational metadata such as the Rust compiler version (`rsver`)
//! or the Soroban SDK version (`rssdkver`) used to build the contract.
//!
//! The XDR permits the same key to appear more than once; this module
//! preserves every entry in encounter order rather than assuming keys
//! are unique, and does not invent a registry of "known" keys beyond
//! what the official Stellar documentation and XDR describe.

use std::io::Cursor;

use analyzer_core::{AnalyzerError, UnsupportedArtifactError};
use stellar_xdr::{Limited, Limits, ReadXdr, ScMetaEntry};
use wasmparser::{Parser, Payload};

/// The WASM custom section name Soroban uses for contract metadata.
pub const CONTRACT_META_SECTION: &str = "contractmetav0";

/// The metadata key SDKs conventionally use to record the Rust compiler
/// version used to build the contract, when present.
pub const KNOWN_KEY_RUST_VERSION: &str = "rsver";

/// The metadata key SDKs conventionally use to record the Soroban SDK
/// version used to build the contract, when present.
pub const KNOWN_KEY_SDK_VERSION: &str = "rssdkver";

/// Bound on recursive XDR nesting depth while decoding. Contract
/// metadata entries are flat key/value pairs; this bound exists purely
/// as a defensive limit against a hostile artifact.
const XDR_READ_DEPTH_LIMIT: u32 = 32;

/// A single decoded `SC_META_V0` key/value observation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractMetaEntry {
    /// The metadata key. Not assumed unique across entries.
    pub key: String,
    /// The metadata value associated with this occurrence of `key`.
    pub value: String,
}

/// The result of looking for and decoding `contractmetav0` in a WASM
/// module.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ContractMetaReport {
    /// How many WASM custom sections named `contractmetav0` were found.
    section_occurrences: usize,
    /// Every entry successfully decoded, across all occurrences, in
    /// encounter order. Repeated keys are preserved as repeated entries.
    entries: Vec<ContractMetaEntry>,
    /// One message per structural or encoding problem encountered while
    /// decoding: a malformed XDR stream (fatal to the rest of that
    /// section's stream) or a key/value pair that is not valid UTF-8
    /// (that entry is skipped, decoding continues).
    decode_errors: Vec<String>,
}

impl ContractMetaReport {
    /// True if at least one `contractmetav0` section was found.
    pub fn is_present(&self) -> bool {
        self.section_occurrences > 0
    }

    /// How many occurrences of the section were found.
    pub fn section_occurrences(&self) -> usize {
        self.section_occurrences
    }

    /// True if more than one `contractmetav0` section was found.
    pub fn has_duplicate_sections(&self) -> bool {
        self.section_occurrences > 1
    }

    /// Every entry successfully decoded, in encounter order.
    pub fn entries(&self) -> &[ContractMetaEntry] {
        &self.entries
    }

    /// Every value observed for `key`, in encounter order. Empty if the
    /// key was never observed; more than one value means the key
    /// repeats.
    pub fn values_for<'a>(&'a self, key: &'a str) -> impl Iterator<Item = &'a str> {
        self.entries
            .iter()
            .filter(move |entry| entry.key == key)
            .map(|entry| entry.value.as_str())
    }

    /// The first observed value for `key`, if any.
    pub fn first_value_for<'a>(&'a self, key: &'a str) -> Option<&'a str> {
        self.values_for(key).next()
    }

    /// Diagnostic messages for structural or encoding problems
    /// encountered while decoding.
    pub fn decode_errors(&self) -> &[String] {
        &self.decode_errors
    }

    /// True if the section is present and every occurrence decoded
    /// without any structural or encoding error.
    pub fn decoded_cleanly(&self) -> bool {
        self.is_present() && self.decode_errors.is_empty()
    }
}

/// Parse the `contractmetav0` custom section(s), if any, out of a WASM
/// module.
///
/// This assumes `bytes` is already-known-valid WASM (see
/// [`crate::validation::validate_generic_wasm`]); it re-parses the
/// module's payload stream independently and reports a parse failure as
/// an [`UnsupportedArtifactError`] rather than panicking.
pub fn parse_contract_metadata(bytes: &[u8]) -> Result<ContractMetaReport, AnalyzerError> {
    let mut report = ContractMetaReport::default();

    for payload in Parser::new(0).parse_all(bytes) {
        let payload = payload.map_err(|source| {
            UnsupportedArtifactError::new(format!("failed to parse wasm module: {source}"))
        })?;

        let Payload::CustomSection(section) = payload else {
            continue;
        };

        if section.name() != CONTRACT_META_SECTION {
            continue;
        }

        report.section_occurrences += 1;

        let limits = Limits {
            depth: XDR_READ_DEPTH_LIMIT,
            len: section.data().len(),
        };
        let mut reader = Limited::new(Cursor::new(section.data()), limits);

        for entry in ScMetaEntry::read_xdr_iter(&mut reader) {
            match entry {
                Ok(ScMetaEntry::ScMetaV0(v0)) => {
                    match (v0.key.to_utf8_string(), v0.val.to_utf8_string()) {
                        (Ok(key), Ok(value)) => {
                            report.entries.push(ContractMetaEntry { key, value })
                        }
                        _ => report.decode_errors.push(
                            "contractmetav0 entry key or value is not valid UTF-8".to_string(),
                        ),
                    }
                }
                Err(source) => {
                    report
                        .decode_errors
                        .push(format!("malformed contractmetav0 entry: {source}"));
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
    use stellar_xdr::{ScMetaV0, StringM, WriteXdr};

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

    fn meta_entry(key: &str, val: &str) -> ScMetaEntry {
        ScMetaEntry::ScMetaV0(ScMetaV0 {
            key: StringM::try_from(key).unwrap(),
            val: StringM::try_from(val).unwrap(),
        })
    }

    fn module_with_contract_meta(entries: &[ScMetaEntry]) -> Vec<u8> {
        let mut data = Vec::new();
        for entry in entries {
            data.extend_from_slice(&entry.to_xdr(Limits::none()).unwrap());
        }
        let mut module = MINIMAL_HEADER.to_vec();
        module.extend_from_slice(&custom_section(CONTRACT_META_SECTION, &data));
        module
    }

    #[test]
    fn absent_section_is_reported_as_absent() {
        let report = parse_contract_metadata(MINIMAL_HEADER).unwrap();
        assert!(!report.is_present());
        assert!(report.entries().is_empty());
        assert!(!report.decoded_cleanly());
    }

    #[test]
    fn known_keys_decode() {
        let module = module_with_contract_meta(&[
            meta_entry(KNOWN_KEY_RUST_VERSION, "1.84.0"),
            meta_entry(KNOWN_KEY_SDK_VERSION, "27.0.0"),
        ]);
        let report = parse_contract_metadata(&module).unwrap();

        assert!(report.decoded_cleanly());
        assert_eq!(report.entries().len(), 2);
        assert_eq!(
            report.first_value_for(KNOWN_KEY_RUST_VERSION),
            Some("1.84.0")
        );
        assert_eq!(
            report.first_value_for(KNOWN_KEY_SDK_VERSION),
            Some("27.0.0")
        );
    }

    #[test]
    fn unknown_keys_remain_inspectable() {
        let module = module_with_contract_meta(&[meta_entry("custom-key", "custom-value")]);
        let report = parse_contract_metadata(&module).unwrap();

        assert_eq!(report.first_value_for("custom-key"), Some("custom-value"));
        assert_eq!(report.first_value_for(KNOWN_KEY_RUST_VERSION), None);
    }

    #[test]
    fn repeated_keys_are_preserved_as_repeated_entries() {
        let module = module_with_contract_meta(&[
            meta_entry("binding", "python"),
            meta_entry("binding", "javascript"),
        ]);
        let report = parse_contract_metadata(&module).unwrap();

        let values: Vec<&str> = report.values_for("binding").collect();
        assert_eq!(values, vec!["python", "javascript"]);
    }

    #[test]
    fn malformed_section_bytes_are_reported_not_panicked() {
        let mut module = MINIMAL_HEADER.to_vec();
        module.extend_from_slice(&custom_section(CONTRACT_META_SECTION, &[0xff, 0xff, 0xff]));
        let report = parse_contract_metadata(&module).unwrap();

        assert!(report.is_present());
        assert!(!report.decoded_cleanly());
        assert!(report.entries().is_empty());
    }

    #[test]
    fn unrelated_custom_sections_are_ignored() {
        let mut module = MINIMAL_HEADER.to_vec();
        module.extend_from_slice(&custom_section("producers", b"irrelevant"));
        let report = parse_contract_metadata(&module).unwrap();
        assert!(!report.is_present());
    }
}
