//! Cross-checks a real generated [`AnalysisReport`] against
//! `schemas/analysis-result.schema.json`.
//!
//! This is not a full JSON Schema conformance validator (no
//! `jsonschema`-family dependency is used, deliberately: the newest
//! available release declares a higher MSRV than this workspace's
//! pinned 1.84.0, and adding one only for this test was not judged
//! worth the compatibility cost). Instead, it reads the schema file's
//! own `required` and `enum` declarations at test time and asserts a
//! real generated report satisfies them, so the test stays tied to the
//! schema document itself rather than to a second, hand-copied list of
//! expectations that could drift out of sync with it.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use analyzer_core::{AnalysisStatus, Confidence, Finding, FindingCategory, Rule, Severity};
use analyzer_report::{to_canonical_json, AnalysisReport, ReportExecutableIdentity};
use serde_json::Value;

fn schema() -> Value {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../schemas/analysis-result.schema.json"
    );
    let text = std::fs::read_to_string(path).expect("schema file must be readable");
    serde_json::from_str(&text).expect("schema file must be valid JSON")
}

fn sample_report() -> AnalysisReport {
    let finding = Finding::new(
        FindingCategory::Interface,
        Rule::ContractSignatureChanged,
        "transfer",
        "input type changed",
        "transfer's amount parameter changed from u32 to i128",
        Severity::Medium,
        Confidence::Detected,
        vec![],
    )
    .with_remediation("review callers of transfer");

    AnalysisReport::new(
        "0.1.0",
        Some(28),
        ReportExecutableIdentity {
            hash: "a".repeat(64),
            byte_length: 1024,
        },
        ReportExecutableIdentity {
            hash: "b".repeat(64),
            byte_length: 1040,
        },
        AnalysisStatus::ReviewRequired,
        std::slice::from_ref(&finding),
    )
}

fn required_keys(schema: &Value, pointer: &str) -> Vec<String> {
    schema
        .pointer(pointer)
        .and_then(|node| node.get("required"))
        .and_then(Value::as_array)
        .expect("schema node must declare required keys")
        .iter()
        .map(|v| {
            v.as_str()
                .expect("required entries are strings")
                .to_string()
        })
        .collect()
}

fn enum_values(schema: &Value, pointer: &str) -> Vec<String> {
    schema
        .pointer(pointer)
        .and_then(Value::as_array)
        .expect("schema node must be an enum array")
        .iter()
        .map(|v| v.as_str().expect("enum entries are strings").to_string())
        .collect()
}

#[test]
fn generated_report_has_every_schema_required_top_level_key() {
    let schema = schema();
    let report = sample_report();
    let json = to_canonical_json(&report).expect("report must serialize");
    let value: Value = serde_json::from_str(&json).expect("report json must parse");
    let object = value
        .as_object()
        .expect("report must serialize as a JSON object");

    for key in required_keys(&schema, "") {
        assert!(
            object.contains_key(&key),
            "generated report is missing required key '{key}'"
        );
    }
}

#[test]
fn generated_finding_has_every_schema_required_key() {
    let schema = schema();
    let report = sample_report();
    let json = to_canonical_json(&report).expect("report must serialize");
    let value: Value = serde_json::from_str(&json).expect("report json must parse");
    let findings = value["findings"]
        .as_array()
        .expect("findings must be an array");
    assert!(
        !findings.is_empty(),
        "sample report must include at least one finding"
    );

    let required = required_keys(&schema, "/$defs/finding");
    for finding in findings {
        let object = finding
            .as_object()
            .expect("each finding must be a JSON object");
        for key in &required {
            assert!(
                object.contains_key(key),
                "finding is missing required key '{key}'"
            );
        }
    }
}

#[test]
fn status_value_is_within_the_schema_enum() {
    let schema = schema();
    let allowed = enum_values(&schema, "/properties/status/enum");
    let report = sample_report();
    assert!(
        allowed.contains(&report.status),
        "status '{}' is not in the schema enum",
        report.status
    );
}

#[test]
fn finding_category_rule_severity_confidence_are_within_schema_enums() {
    let schema = schema();
    let allowed_categories = enum_values(&schema, "/$defs/finding/properties/category/enum");
    let allowed_rules = enum_values(&schema, "/$defs/finding/properties/rule/enum");
    let allowed_severities = enum_values(&schema, "/$defs/finding/properties/severity/enum");
    let allowed_confidences = enum_values(&schema, "/$defs/finding/properties/confidence/enum");

    let report = sample_report();
    for finding in &report.findings {
        assert!(allowed_categories.contains(&finding.category));
        assert!(allowed_rules.contains(&finding.rule));
        assert!(allowed_severities.contains(&finding.severity));
        assert!(allowed_confidences.contains(&finding.confidence));
    }
}

#[test]
fn executable_hash_matches_the_schema_hex64_pattern() {
    let report = sample_report();
    for hash in [
        &report.current_executable.hash,
        &report.candidate_executable.hash,
    ] {
        assert_eq!(hash.len(), 64, "hash must be 64 hex characters");
        assert!(
            hash.chars()
                .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()),
            "hash must be lowercase hexadecimal"
        );
    }
}

#[test]
fn report_schema_version_matches_the_schema_const() {
    let schema = schema();
    let expected = schema
        .pointer("/properties/schema_version/const")
        .and_then(Value::as_str)
        .expect("schema must declare a const schema_version");
    let report = sample_report();
    assert_eq!(report.schema_version, expected);
}
