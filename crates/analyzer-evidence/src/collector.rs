//! A simple, order-preserving accumulator for [`Evidence`] gathered
//! during one analysis run.
//!
//! `EvidenceCollector` does not deduplicate: if the same observation is
//! recorded twice, both entries are kept, in the order they were added.
//! Deterministic ordering (encounter order) is deliberate; a caller that
//! wants deduplication can key on [`Evidence::id`] itself, since
//! identical evidence always produces identical ids.

use crate::reference::Evidence;

/// Collects [`Evidence`] records in the order they are added.
#[derive(Debug, Clone, Default)]
pub struct EvidenceCollector {
    records: Vec<Evidence>,
}

impl EvidenceCollector {
    /// Create an empty collector.
    pub fn new() -> Self {
        Self::default()
    }

    /// Record one piece of evidence.
    pub fn record(&mut self, evidence: Evidence) {
        self.records.push(evidence);
    }

    /// Every recorded evidence record, in encounter order.
    pub fn records(&self) -> &[Evidence] {
        &self.records
    }

    /// How many records have been collected.
    pub fn len(&self) -> usize {
        self.records.len()
    }

    /// True if no evidence has been recorded.
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    /// Consume the collector, returning its records in encounter order.
    pub fn into_records(self) -> Vec<Evidence> {
        self.records
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::source::EvidenceSource;

    fn evidence(observation: &str) -> Evidence {
        Evidence::new(
            EvidenceSource::LocalArtifact {
                artifact_hash: "a".repeat(64),
            },
            "test-producer",
            None,
            observation,
        )
    }

    #[test]
    fn records_preserve_encounter_order() {
        let mut collector = EvidenceCollector::new();
        collector.record(evidence("first"));
        collector.record(evidence("second"));

        let records = collector.records();
        assert_eq!(records.len(), 2);
        assert_eq!(records[0].observation(), "first");
        assert_eq!(records[1].observation(), "second");
    }

    #[test]
    fn duplicate_observations_are_not_merged() {
        let mut collector = EvidenceCollector::new();
        collector.record(evidence("same"));
        collector.record(evidence("same"));
        assert_eq!(collector.len(), 2);
        assert_eq!(collector.records()[0].id(), collector.records()[1].id());
    }

    #[test]
    fn empty_collector_reports_empty() {
        let collector = EvidenceCollector::new();
        assert!(collector.is_empty());
        assert_eq!(collector.len(), 0);
    }
}
