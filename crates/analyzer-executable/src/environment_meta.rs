//! Soroban environment metadata (`contractenvmetav0`) parsing.
//!
//! Every Soroban contract WASM module is expected to carry a custom
//! WASM section named `contractenvmetav0` containing the serialized
//! bytes of a stream of [`stellar_xdr::ScEnvMetaEntry`] XDR values, back
//! to back with no additional framing. Today `ScEnvMetaEntry` has a
//! single case, `SC_ENV_META_KIND_INTERFACE_VERSION`, which carries a
//! protocol number and a pre-release number identifying the Soroban
//! environment interface the contract was built against. See the
//! "Build Your Own Contract SDK" section of the official Stellar
//! developer documentation
//! (<https://developers.stellar.org/docs/tools/sdks/build-your-own>)
//! and the `ScEnvMetaEntry`/`ScEnvMetaEntryInterfaceVersion` XDR
//! definitions in `stellar-xdr`.
//!
//! This module only decodes what is present; it never claims
//! environment or protocol *compatibility*. Comparing an observed
//! interface version against a specific analysis protocol context is a
//! judgment the analysis layer makes, not this parser.

use std::io::Cursor;

use analyzer_core::{AnalyzerError, UnsupportedArtifactError};
use stellar_xdr::{Limited, Limits, ReadXdr, ScEnvMetaEntry};
use wasmparser::{Parser, Payload};

/// The WASM custom section name Soroban uses for environment metadata.
pub const CONTRACT_ENV_META_SECTION: &str = "contractenvmetav0";

/// Bound on recursive XDR nesting depth while decoding. Environment
/// metadata is a flat structure; this bound exists purely as a defensive
/// limit against a hostile artifact, not because legitimate metadata
/// nests anywhere near this deep.
const XDR_READ_DEPTH_LIMIT: u32 = 32;

/// A single observed Soroban environment interface version: a protocol
/// number and a pre-release number, as decoded from one
/// `SC_ENV_META_KIND_INTERFACE_VERSION` entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EnvironmentInterfaceVersion {
    /// The protocol component of the interface version.
    pub protocol: u32,
    /// The pre-release component of the interface version. Zero for a
    /// released (non-prerelease) protocol.
    pub pre_release: u32,
}

/// The result of looking for and decoding `contractenvmetav0` in a WASM
/// module.
///
/// This never collapses "absent", "present but malformed", and
/// "present and decoded" into a single boolean: each is recorded
/// explicitly so a caller can tell why a conclusion about environment
/// compatibility can or cannot be drawn.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EnvironmentMetaReport {
    /// How many WASM custom sections named `contractenvmetav0` were
    /// found. Zero means the section is absent. More than one is an
    /// anomaly this parser surfaces rather than silently resolves (by,
    /// for example, concatenating or picking one).
    section_occurrences: usize,
    /// Every `SC_ENV_META_KIND_INTERFACE_VERSION` entry successfully
    /// decoded, across all occurrences of the section, in encounter
    /// order.
    interface_versions: Vec<EnvironmentInterfaceVersion>,
    /// One message per occurrence of the section whose bytes could not
    /// be fully decoded as a stream of `ScEnvMetaEntry` values.
    decode_errors: Vec<String>,
}

impl EnvironmentMetaReport {
    /// True if at least one `contractenvmetav0` section was found.
    pub fn is_present(&self) -> bool {
        self.section_occurrences > 0
    }

    /// How many occurrences of the section were found.
    pub fn section_occurrences(&self) -> usize {
        self.section_occurrences
    }

    /// True if more than one `contractenvmetav0` section was found.
    /// The Soroban toolchain emits exactly one; more than one is
    /// evidence of a malformed or hand-crafted artifact.
    pub fn has_duplicate_sections(&self) -> bool {
        self.section_occurrences > 1
    }

    /// Every interface version successfully decoded, in encounter
    /// order. Empty if the section is absent or every occurrence failed
    /// to decode.
    pub fn interface_versions(&self) -> &[EnvironmentInterfaceVersion] {
        &self.interface_versions
    }

    /// The first successfully decoded interface version, if any.
    pub fn primary_interface_version(&self) -> Option<EnvironmentInterfaceVersion> {
        self.interface_versions.first().copied()
    }

    /// True if more than one *distinct* interface version was observed
    /// across occurrences/entries. A single, internally consistent
    /// contract should never have conflicting observations.
    pub fn has_conflicting_interface_versions(&self) -> bool {
        self.interface_versions
            .windows(2)
            .any(|pair| pair[0] != pair[1])
    }

    /// Diagnostic messages for occurrences of the section that failed
    /// to decode as a stream of `ScEnvMetaEntry` values.
    pub fn decode_errors(&self) -> &[String] {
        &self.decode_errors
    }

    /// True if the section is present and every occurrence decoded
    /// without error. This does not require any interface-version entry
    /// to actually be present: a section that decodes to zero entries
    /// still "decoded cleanly", it is simply empty.
    pub fn decoded_cleanly(&self) -> bool {
        self.is_present() && self.decode_errors.is_empty()
    }
}

/// Parse the `contractenvmetav0` custom section(s), if any, out of a
/// WASM module.
///
/// This assumes `bytes` is already-known-valid WASM (see
/// [`crate::validation::validate_generic_wasm`]); it re-parses the
/// module's payload stream independently and reports a parse failure as
/// an [`UnsupportedArtifactError`] rather than panicking.
pub fn parse_environment_metadata(bytes: &[u8]) -> Result<EnvironmentMetaReport, AnalyzerError> {
    let mut report = EnvironmentMetaReport::default();

    for payload in Parser::new(0).parse_all(bytes) {
        let payload = payload.map_err(|source| {
            UnsupportedArtifactError::new(format!("failed to parse wasm module: {source}"))
        })?;

        let Payload::CustomSection(section) = payload else {
            continue;
        };

        if section.name() != CONTRACT_ENV_META_SECTION {
            continue;
        }

        report.section_occurrences += 1;

        let limits = Limits {
            depth: XDR_READ_DEPTH_LIMIT,
            len: section.data().len(),
        };
        let mut reader = Limited::new(Cursor::new(section.data()), limits);

        for entry in ScEnvMetaEntry::read_xdr_iter(&mut reader) {
            match entry {
                Ok(ScEnvMetaEntry::ScEnvMetaKindInterfaceVersion(version)) => {
                    report.interface_versions.push(EnvironmentInterfaceVersion {
                        protocol: version.protocol,
                        pre_release: version.pre_release,
                    });
                }
                Err(source) => {
                    report
                        .decode_errors
                        .push(format!("malformed contractenvmetav0 entry: {source}"));
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
    use stellar_xdr::{ScEnvMetaEntryInterfaceVersion, WriteXdr};

    const MINIMAL_HEADER: &[u8] = &[0x00, 0x61, 0x73, 0x6d, 0x01, 0x00, 0x00, 0x00];

    fn custom_section(name: &str, data: &[u8]) -> Vec<u8> {
        let mut name_bytes = Vec::new();
        write_leb128(&mut name_bytes, name.len() as u64);
        name_bytes.extend_from_slice(name.as_bytes());

        let mut content = name_bytes;
        content.extend_from_slice(data);

        let mut section = vec![0x00]; // custom section id
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

    fn module_with_env_meta(entries: &[ScEnvMetaEntry]) -> Vec<u8> {
        let mut data = Vec::new();
        for entry in entries {
            data.extend_from_slice(&entry.to_xdr(Limits::none()).unwrap());
        }
        let mut module = MINIMAL_HEADER.to_vec();
        module.extend_from_slice(&custom_section(CONTRACT_ENV_META_SECTION, &data));
        module
    }

    fn interface_version(protocol: u32, pre_release: u32) -> ScEnvMetaEntry {
        ScEnvMetaEntry::ScEnvMetaKindInterfaceVersion(ScEnvMetaEntryInterfaceVersion {
            protocol,
            pre_release,
        })
    }

    #[test]
    fn absent_section_is_reported_as_absent() {
        let report = parse_environment_metadata(MINIMAL_HEADER).unwrap();
        assert!(!report.is_present());
        assert_eq!(report.section_occurrences(), 0);
        assert!(report.interface_versions().is_empty());
        assert!(!report.decoded_cleanly());
    }

    #[test]
    fn single_interface_version_decodes() {
        let module = module_with_env_meta(&[interface_version(28, 0)]);
        let report = parse_environment_metadata(&module).unwrap();

        assert!(report.is_present());
        assert_eq!(report.section_occurrences(), 1);
        assert!(report.decoded_cleanly());
        assert_eq!(
            report.primary_interface_version(),
            Some(EnvironmentInterfaceVersion {
                protocol: 28,
                pre_release: 0,
            })
        );
        assert!(!report.has_conflicting_interface_versions());
    }

    #[test]
    fn conflicting_interface_versions_are_detected() {
        let module = module_with_env_meta(&[interface_version(28, 0), interface_version(27, 0)]);
        let report = parse_environment_metadata(&module).unwrap();

        assert_eq!(report.interface_versions().len(), 2);
        assert!(report.has_conflicting_interface_versions());
    }

    #[test]
    fn malformed_section_bytes_are_reported_not_panicked() {
        let mut module = MINIMAL_HEADER.to_vec();
        module.extend_from_slice(&custom_section(
            CONTRACT_ENV_META_SECTION,
            &[0xff, 0xff, 0xff],
        ));
        let report = parse_environment_metadata(&module).unwrap();

        assert!(report.is_present());
        assert!(!report.decoded_cleanly());
        assert_eq!(report.decode_errors().len(), 1);
        assert!(report.interface_versions().is_empty());
    }

    #[test]
    fn duplicate_sections_are_counted_not_merged_silently() {
        let mut module = MINIMAL_HEADER.to_vec();
        let single = {
            let mut data = Vec::new();
            data.extend_from_slice(&interface_version(28, 0).to_xdr(Limits::none()).unwrap());
            data
        };
        module.extend_from_slice(&custom_section(CONTRACT_ENV_META_SECTION, &single));
        module.extend_from_slice(&custom_section(CONTRACT_ENV_META_SECTION, &single));

        let report = parse_environment_metadata(&module).unwrap();
        assert!(report.has_duplicate_sections());
        assert_eq!(report.section_occurrences(), 2);
        assert_eq!(report.interface_versions().len(), 2);
    }

    #[test]
    fn unrelated_custom_sections_are_ignored() {
        let mut module = MINIMAL_HEADER.to_vec();
        module.extend_from_slice(&custom_section("producers", b"irrelevant"));
        let report = parse_environment_metadata(&module).unwrap();
        assert!(!report.is_present());
    }
}
