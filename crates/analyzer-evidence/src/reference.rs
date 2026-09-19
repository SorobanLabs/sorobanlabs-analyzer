//! Deterministic, content-addressed evidence records.
//!
//! An [`Evidence`] record answers, for one specific observation: what
//! source was examined ([`crate::source::EvidenceSource`]), what
//! parser or rule produced the observation (`producer`), what location
//! inside the source matters (`location`), and what was concluded
//! (`observation`). Its [`EvidenceId`] is SHA-256 over a canonical
//! serialization of those fields, so identical evidence always gets the
//! identical id, and identity never depends on when the evidence was
//! collected.
//!
//! `observation` is a short, human-readable fact or conclusion, not a
//! dump of raw data: prefer a hash, a bounded excerpt, or a typed
//! summary over embedding large content here. This module does not
//! enforce a length limit; callers are responsible for keeping
//! observations bounded, per the analyzer's evidence design.

use std::fmt;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::source::EvidenceSource;

/// The deterministic, content-addressed identity of an [`Evidence`]
/// record: SHA-256 over a canonical encoding of its fields.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct EvidenceId(#[serde(with = "hex_array")] [u8; 32]);

impl EvidenceId {
    /// The canonical textual representation: 64 lowercase hexadecimal
    /// characters.
    pub fn to_hex(&self) -> String {
        hex::encode(self.0)
    }

    /// The raw 32-byte digest.
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

impl fmt::Display for EvidenceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.to_hex())
    }
}

impl fmt::Debug for EvidenceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("EvidenceId").field(&self.to_hex()).finish()
    }
}

mod hex_array {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(bytes: &[u8; 32], serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&hex::encode(bytes))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<[u8; 32], D::Error> {
        let text = String::deserialize(deserializer)?;
        let decoded = hex::decode(&text).map_err(serde::de::Error::custom)?;
        <[u8; 32]>::try_from(decoded.as_slice())
            .map_err(|_| serde::de::Error::custom("evidence id must be 32 bytes"))
    }
}

/// One deterministic, traceable piece of evidence backing an analyzer
/// finding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Evidence {
    id: EvidenceId,
    source: EvidenceSource,
    producer: String,
    location: Option<String>,
    observation: String,
}

impl Evidence {
    /// Construct an evidence record. The id is derived deterministically
    /// from `source`, `producer`, `location`, and `observation`: two
    /// calls with identical arguments always produce the identical id.
    pub fn new(
        source: EvidenceSource,
        producer: impl Into<String>,
        location: Option<String>,
        observation: impl Into<String>,
    ) -> Self {
        let producer = producer.into();
        let observation = observation.into();
        let id = compute_evidence_id(&source, &producer, location.as_deref(), &observation);
        Self {
            id,
            source,
            producer,
            location,
            observation,
        }
    }

    /// The deterministic, content-addressed identity of this record.
    pub fn id(&self) -> EvidenceId {
        self.id
    }

    /// What source was examined.
    pub fn source(&self) -> &EvidenceSource {
        &self.source
    }

    /// What parser or rule produced this observation (for example,
    /// `"analyzer-executable::validation"` or a rule identifier).
    pub fn producer(&self) -> &str {
        &self.producer
    }

    /// What location inside the source matters, if applicable (a
    /// section name, an XDR field path, a ledger key).
    pub fn location(&self) -> Option<&str> {
        self.location.as_deref()
    }

    /// The bounded, human-readable fact or conclusion this evidence
    /// establishes.
    pub fn observation(&self) -> &str {
        &self.observation
    }
}

fn compute_evidence_id(
    source: &EvidenceSource,
    producer: &str,
    location: Option<&str>,
    observation: &str,
) -> EvidenceId {
    // A simple, explicit, deterministic canonical encoding: each field
    // length-prefixed so no field's content can be confused with a
    // delimiter or with an adjacent field's content.
    let mut buffer = Vec::new();
    write_field(&mut buffer, source.kind().as_bytes());
    let source_json =
        serde_json::to_vec(source).unwrap_or_else(|_| source.kind().as_bytes().to_vec());
    write_field(&mut buffer, &source_json);
    write_field(&mut buffer, producer.as_bytes());
    write_field(&mut buffer, location.unwrap_or("").as_bytes());
    write_field(&mut buffer, observation.as_bytes());

    let digest = Sha256::digest(&buffer);
    let mut out = [0u8; 32];
    out.copy_from_slice(&digest);
    EvidenceId(out)
}

fn write_field(buffer: &mut Vec<u8>, field: &[u8]) {
    buffer.extend_from_slice(&(field.len() as u64).to_le_bytes());
    buffer.extend_from_slice(field);
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    fn sample_source() -> EvidenceSource {
        EvidenceSource::LocalArtifact {
            artifact_hash: "a".repeat(64),
        }
    }

    #[test]
    fn identical_inputs_produce_identical_id() {
        let a = Evidence::new(
            sample_source(),
            "analyzer-executable::validation",
            Some("header".to_string()),
            "structurally valid WASM",
        );
        let b = Evidence::new(
            sample_source(),
            "analyzer-executable::validation",
            Some("header".to_string()),
            "structurally valid WASM",
        );
        assert_eq!(a.id(), b.id());
    }

    #[test]
    fn different_observation_produces_different_id() {
        let a = Evidence::new(sample_source(), "producer", None, "observation A");
        let b = Evidence::new(sample_source(), "producer", None, "observation B");
        assert_ne!(a.id(), b.id());
    }

    #[test]
    fn different_producer_produces_different_id() {
        let a = Evidence::new(sample_source(), "producer-a", None, "same observation");
        let b = Evidence::new(sample_source(), "producer-b", None, "same observation");
        assert_ne!(a.id(), b.id());
    }

    #[test]
    fn different_location_produces_different_id() {
        let a = Evidence::new(
            sample_source(),
            "producer",
            Some("loc-a".to_string()),
            "obs",
        );
        let b = Evidence::new(
            sample_source(),
            "producer",
            Some("loc-b".to_string()),
            "obs",
        );
        assert_ne!(a.id(), b.id());
    }

    #[test]
    fn id_formats_as_lowercase_hex_of_expected_length() {
        let evidence = Evidence::new(sample_source(), "producer", None, "obs");
        let hex = evidence.id().to_hex();
        assert_eq!(hex.len(), 64);
        assert!(hex
            .chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()));
        assert_eq!(evidence.id().to_string(), hex);
    }

    #[test]
    fn id_survives_json_round_trip() {
        let evidence = Evidence::new(sample_source(), "producer", None, "obs");
        let json = serde_json::to_string(&evidence).unwrap();
        let round_tripped: Evidence = serde_json::from_str(&json).unwrap();
        assert_eq!(evidence, round_tripped);
    }
}
