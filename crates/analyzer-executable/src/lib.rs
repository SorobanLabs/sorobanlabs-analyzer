//! Executable artifact loading, hashing, WASM inspection, and interface
//! normalization/diffing for the SorobanLabs Analyzer.
//!
//! Responsibilities: canonical executable hashing, WASM metadata parsing,
//! Soroban contract specification extraction, public interface
//! normalization, and current-vs-candidate executable comparison.
//! All outputs must be deterministic.
//!
//! This version establishes local WASM artifact loading and canonical
//! byte-level identity ([`LoadedWasm`], [`ArtifactHash`],
//! [`ArtifactSource`]), generic and Soroban-specific structural WASM
//! validation ([`validation`]), Soroban environment metadata parsing
//! ([`environment_meta`]), and Soroban contract metadata parsing
//! ([`contract_meta`]). Contract specification parsing and interface
//! diffing are added in later steps.

mod artifact;
mod contract_meta;
mod contract_spec;
mod environment_meta;
mod interface;
mod validation;

pub use artifact::{ArtifactHash, ArtifactSource, LoadedWasm};
pub use contract_meta::{
    parse_contract_metadata, ContractMetaEntry, ContractMetaReport, CONTRACT_META_SECTION,
    KNOWN_KEY_RUST_VERSION, KNOWN_KEY_SDK_VERSION,
};
pub use contract_spec::{parse_contract_spec, ContractSpecReport, CONTRACT_SPEC_SECTION};
pub use environment_meta::{
    parse_environment_metadata, EnvironmentInterfaceVersion, EnvironmentMetaReport,
    CONTRACT_ENV_META_SECTION,
};
pub use interface::{
    normalize_interface, EventDataFormat, EventParamLocation, NormalizedEnum, NormalizedEnumCase,
    NormalizedErrorEnum, NormalizedErrorEnumCase, NormalizedEvent, NormalizedEventParameter,
    NormalizedFunction, NormalizedInterface, NormalizedParameter, NormalizedStruct,
    NormalizedStructField, NormalizedType, NormalizedUnion, NormalizedUnionCase,
};
pub use validation::{
    check_soroban_structural_compatibility, validate_generic_wasm, SorobanStructuralReport,
    SorobanStructuralViolation, MAX_WASM_BYTES,
};
