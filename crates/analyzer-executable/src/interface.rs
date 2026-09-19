//! Deterministic, analyzer-owned normalization of a Soroban contract's
//! public interface from the raw `contractspecv0` entries
//! ([`crate::contract_spec`]).
//!
//! [`ScSpecEntry`] and [`ScSpecTypeDef`] are XDR union/struct types
//! generated to mirror the wire format exactly; they are not a
//! convenient shape for comparison. [`NormalizedInterface`] and its
//! constituent types are this analyzer's own model, built once from the
//! raw entries, so that [`crate::diff`] (interface comparison) never has
//! to reason about XDR framing.
//!
//! Normalization here means: decode every name (`ScSymbol`/`StringM`)
//! to a Rust `String`, recursively rewrite `ScSpecTypeDef` into
//! [`NormalizedType`], and group entries by kind. It does not reorder,
//! deduplicate, or drop anything the specification itself does not
//! already consider redundant: declared order is preserved throughout,
//! because this parser has no independent evidence about which
//! orderings the Soroban wire format or SDK tooling treat as
//! significant, and assuming otherwise would be exactly the kind of
//! invented framing this analyzer must not produce.

use analyzer_core::{AnalysisError, AnalyzerError};
use stellar_xdr::{
    ScSpecEntry, ScSpecEventDataFormat, ScSpecEventParamLocationV0, ScSpecEventParamV0,
    ScSpecEventV0, ScSpecFunctionV0, ScSpecTypeDef, ScSpecUdtEnumV0, ScSpecUdtErrorEnumV0,
    ScSpecUdtStructV0, ScSpecUdtUnionCaseV0, ScSpecUdtUnionV0,
};

/// A normalized, recursively-expanded Soroban value type.
///
/// Mirrors every case of `stellar_xdr::ScSpecTypeDef`, but user-defined
/// types are collapsed to their declared name ([`NormalizedType::Udt`])
/// rather than re-expanded inline: a UDT's own shape is compared
/// separately, as its own struct/union/enum entry.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum NormalizedType {
    Val,
    Bool,
    Void,
    Error,
    U32,
    I32,
    U64,
    I64,
    Timepoint,
    Duration,
    U128,
    I128,
    U256,
    I256,
    Bytes,
    String,
    Symbol,
    Address,
    MuxedAddress,
    Option(Box<NormalizedType>),
    Result {
        ok: Box<NormalizedType>,
        error: Box<NormalizedType>,
    },
    Vec(Box<NormalizedType>),
    Map {
        key: Box<NormalizedType>,
        value: Box<NormalizedType>,
    },
    Tuple(Vec<NormalizedType>),
    BytesN(u32),
    /// A reference to a user-defined type by its declared name.
    Udt(String),
}

/// One function input parameter, in declared order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedParameter {
    pub name: String,
    pub type_: NormalizedType,
}

/// A normalized contract entrypoint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedFunction {
    pub name: String,
    /// Declared input parameters, in call order.
    pub inputs: Vec<NormalizedParameter>,
    /// The declared return type. `ScSpecFunctionV0::outputs` is bounded
    /// to at most one entry in the XDR; this is `None` when the
    /// function declares no return value.
    pub output: Option<NormalizedType>,
}

/// One field of a normalized struct, in declared order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedStructField {
    pub name: String,
    pub type_: NormalizedType,
}

/// A normalized user-defined struct type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedStruct {
    pub name: String,
    pub fields: Vec<NormalizedStructField>,
}

/// One case of a normalized union type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NormalizedUnionCase {
    /// A case with no associated data.
    Void { name: String },
    /// A case carrying a tuple of typed values.
    Tuple {
        name: String,
        types: Vec<NormalizedType>,
    },
}

impl NormalizedUnionCase {
    /// The case's declared name, regardless of which kind it is.
    pub fn name(&self) -> &str {
        match self {
            Self::Void { name } | Self::Tuple { name, .. } => name,
        }
    }
}

/// A normalized user-defined union type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedUnion {
    pub name: String,
    pub cases: Vec<NormalizedUnionCase>,
}

/// One case of a normalized enum type: a name bound to an explicit
/// numeric value (the value, not declaration position, is the case's
/// wire identity).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedEnumCase {
    pub name: String,
    pub value: u32,
}

/// A normalized user-defined enum type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedEnum {
    pub name: String,
    pub cases: Vec<NormalizedEnumCase>,
}

/// One case of a normalized error enum type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedErrorEnumCase {
    pub name: String,
    pub value: u32,
}

/// A normalized user-defined error enum type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedErrorEnum {
    pub name: String,
    pub cases: Vec<NormalizedErrorEnumCase>,
}

/// Where an event parameter's value is carried: the event's data
/// payload, or its topic list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventParamLocation {
    Data,
    TopicList,
}

/// How an event's non-topic data is encoded.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventDataFormat {
    SingleValue,
    Vec,
    Map,
}

/// One normalized event parameter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedEventParameter {
    pub name: String,
    pub type_: NormalizedType,
    pub location: EventParamLocation,
}

/// A normalized contract event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedEvent {
    pub name: String,
    pub prefix_topics: Vec<String>,
    pub params: Vec<NormalizedEventParameter>,
    pub data_format: EventDataFormat,
}

/// The full normalized public interface of a contract: every function,
/// user-defined type, and event declared in its `contractspecv0`
/// entries, grouped by kind and decoded to analyzer-owned types.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NormalizedInterface {
    pub functions: Vec<NormalizedFunction>,
    pub structs: Vec<NormalizedStruct>,
    pub unions: Vec<NormalizedUnion>,
    pub enums: Vec<NormalizedEnum>,
    pub error_enums: Vec<NormalizedErrorEnum>,
    pub events: Vec<NormalizedEvent>,
}

/// Normalize a decoded `contractspecv0` entry stream (see
/// [`crate::contract_spec::parse_contract_spec`]) into a
/// [`NormalizedInterface`].
///
/// # Errors
///
/// Returns [`AnalysisError`] if an entry that already decoded
/// successfully as XDR nonetheless contains a name that is not valid
/// UTF-8. This is treated as an analysis failure, not an operational
/// one: the artifact loaded and its spec section decoded, but this
/// specific entry cannot be normalized.
pub fn normalize_interface(entries: &[ScSpecEntry]) -> Result<NormalizedInterface, AnalyzerError> {
    let mut interface = NormalizedInterface::default();

    for entry in entries {
        match entry {
            ScSpecEntry::FunctionV0(function) => {
                interface.functions.push(normalize_function(function)?);
            }
            ScSpecEntry::UdtStructV0(udt) => {
                interface.structs.push(normalize_struct(udt)?);
            }
            ScSpecEntry::UdtUnionV0(udt) => {
                interface.unions.push(normalize_union(udt)?);
            }
            ScSpecEntry::UdtEnumV0(udt) => {
                interface.enums.push(normalize_enum(udt)?);
            }
            ScSpecEntry::UdtErrorEnumV0(udt) => {
                interface.error_enums.push(normalize_error_enum(udt)?);
            }
            ScSpecEntry::EventV0(event) => {
                interface.events.push(normalize_event(event)?);
            }
        }
    }

    Ok(interface)
}

fn utf8_name(bytes: &impl AsRef<[u8]>, what: &str) -> Result<String, AnalyzerError> {
    String::from_utf8(bytes.as_ref().to_vec()).map_err(|source| {
        AnalysisError::with_source(format!("{what} is not valid UTF-8"), source).into()
    })
}

fn normalize_type(type_def: &ScSpecTypeDef) -> Result<NormalizedType, AnalyzerError> {
    let normalized = match type_def {
        ScSpecTypeDef::Val => NormalizedType::Val,
        ScSpecTypeDef::Bool => NormalizedType::Bool,
        ScSpecTypeDef::Void => NormalizedType::Void,
        ScSpecTypeDef::Error => NormalizedType::Error,
        ScSpecTypeDef::U32 => NormalizedType::U32,
        ScSpecTypeDef::I32 => NormalizedType::I32,
        ScSpecTypeDef::U64 => NormalizedType::U64,
        ScSpecTypeDef::I64 => NormalizedType::I64,
        ScSpecTypeDef::Timepoint => NormalizedType::Timepoint,
        ScSpecTypeDef::Duration => NormalizedType::Duration,
        ScSpecTypeDef::U128 => NormalizedType::U128,
        ScSpecTypeDef::I128 => NormalizedType::I128,
        ScSpecTypeDef::U256 => NormalizedType::U256,
        ScSpecTypeDef::I256 => NormalizedType::I256,
        ScSpecTypeDef::Bytes => NormalizedType::Bytes,
        ScSpecTypeDef::String => NormalizedType::String,
        ScSpecTypeDef::Symbol => NormalizedType::Symbol,
        ScSpecTypeDef::Address => NormalizedType::Address,
        ScSpecTypeDef::MuxedAddress => NormalizedType::MuxedAddress,
        ScSpecTypeDef::Option(option) => {
            NormalizedType::Option(Box::new(normalize_type(&option.value_type)?))
        }
        ScSpecTypeDef::Result(result) => NormalizedType::Result {
            ok: Box::new(normalize_type(&result.ok_type)?),
            error: Box::new(normalize_type(&result.error_type)?),
        },
        ScSpecTypeDef::Vec(vec) => {
            NormalizedType::Vec(Box::new(normalize_type(&vec.element_type)?))
        }
        ScSpecTypeDef::Map(map) => NormalizedType::Map {
            key: Box::new(normalize_type(&map.key_type)?),
            value: Box::new(normalize_type(&map.value_type)?),
        },
        ScSpecTypeDef::Tuple(tuple) => {
            let types = tuple
                .value_types
                .iter()
                .map(normalize_type)
                .collect::<Result<Vec<_>, _>>()?;
            NormalizedType::Tuple(types)
        }
        ScSpecTypeDef::BytesN(bytes_n) => NormalizedType::BytesN(bytes_n.n),
        ScSpecTypeDef::Udt(udt) => NormalizedType::Udt(utf8_name(&udt.name, "udt type name")?),
    };
    Ok(normalized)
}

fn normalize_function(function: &ScSpecFunctionV0) -> Result<NormalizedFunction, AnalyzerError> {
    let name = utf8_name(&function.name, "function name")?;
    let inputs = function
        .inputs
        .iter()
        .map(|input| {
            Ok(NormalizedParameter {
                name: utf8_name(&input.name, "function input name")?,
                type_: normalize_type(&input.type_)?,
            })
        })
        .collect::<Result<Vec<_>, AnalyzerError>>()?;
    let output = match function.outputs.first() {
        Some(output) => Some(normalize_type(output)?),
        None => None,
    };

    Ok(NormalizedFunction {
        name,
        inputs,
        output,
    })
}

fn normalize_struct(udt: &ScSpecUdtStructV0) -> Result<NormalizedStruct, AnalyzerError> {
    let name = utf8_name(&udt.name, "struct name")?;
    let fields = udt
        .fields
        .iter()
        .map(|field| {
            Ok(NormalizedStructField {
                name: utf8_name(&field.name, "struct field name")?,
                type_: normalize_type(&field.type_)?,
            })
        })
        .collect::<Result<Vec<_>, AnalyzerError>>()?;

    Ok(NormalizedStruct { name, fields })
}

fn normalize_union(udt: &ScSpecUdtUnionV0) -> Result<NormalizedUnion, AnalyzerError> {
    let name = utf8_name(&udt.name, "union name")?;
    let cases = udt
        .cases
        .iter()
        .map(|case| match case {
            ScSpecUdtUnionCaseV0::VoidV0(void_case) => Ok(NormalizedUnionCase::Void {
                name: utf8_name(&void_case.name, "union case name")?,
            }),
            ScSpecUdtUnionCaseV0::TupleV0(tuple_case) => {
                let types = tuple_case
                    .type_
                    .iter()
                    .map(normalize_type)
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(NormalizedUnionCase::Tuple {
                    name: utf8_name(&tuple_case.name, "union case name")?,
                    types,
                })
            }
        })
        .collect::<Result<Vec<_>, AnalyzerError>>()?;

    Ok(NormalizedUnion { name, cases })
}

fn normalize_enum(udt: &ScSpecUdtEnumV0) -> Result<NormalizedEnum, AnalyzerError> {
    let name = utf8_name(&udt.name, "enum name")?;
    let cases = udt
        .cases
        .iter()
        .map(|case| {
            Ok(NormalizedEnumCase {
                name: utf8_name(&case.name, "enum case name")?,
                value: case.value,
            })
        })
        .collect::<Result<Vec<_>, AnalyzerError>>()?;

    Ok(NormalizedEnum { name, cases })
}

fn normalize_error_enum(udt: &ScSpecUdtErrorEnumV0) -> Result<NormalizedErrorEnum, AnalyzerError> {
    let name = utf8_name(&udt.name, "error enum name")?;
    let cases = udt
        .cases
        .iter()
        .map(|case| {
            Ok(NormalizedErrorEnumCase {
                name: utf8_name(&case.name, "error enum case name")?,
                value: case.value,
            })
        })
        .collect::<Result<Vec<_>, AnalyzerError>>()?;

    Ok(NormalizedErrorEnum { name, cases })
}

fn normalize_event(event: &ScSpecEventV0) -> Result<NormalizedEvent, AnalyzerError> {
    let name = utf8_name(&event.name, "event name")?;
    let prefix_topics = event
        .prefix_topics
        .iter()
        .map(|topic| utf8_name(topic, "event prefix topic"))
        .collect::<Result<Vec<_>, AnalyzerError>>()?;
    let params = event
        .params
        .iter()
        .map(normalize_event_param)
        .collect::<Result<Vec<_>, AnalyzerError>>()?;
    let data_format = match event.data_format {
        ScSpecEventDataFormat::SingleValue => EventDataFormat::SingleValue,
        ScSpecEventDataFormat::Vec => EventDataFormat::Vec,
        ScSpecEventDataFormat::Map => EventDataFormat::Map,
    };

    Ok(NormalizedEvent {
        name,
        prefix_topics,
        params,
        data_format,
    })
}

fn normalize_event_param(
    param: &ScSpecEventParamV0,
) -> Result<NormalizedEventParameter, AnalyzerError> {
    let location = match param.location {
        ScSpecEventParamLocationV0::Data => EventParamLocation::Data,
        ScSpecEventParamLocationV0::TopicList => EventParamLocation::TopicList,
    };

    Ok(NormalizedEventParameter {
        name: utf8_name(&param.name, "event parameter name")?,
        type_: normalize_type(&param.type_)?,
        location,
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use stellar_xdr::{
        ScSpecFunctionInputV0, ScSpecTypeBytesN, ScSpecTypeOption, ScSpecTypeTuple, ScSpecTypeUdt,
        ScSpecUdtEnumCaseV0, ScSpecUdtStructFieldV0, ScSpecUdtUnionCaseTupleV0,
        ScSpecUdtUnionCaseVoidV0, ScSymbol, StringM, VecM,
    };

    fn symbol(s: &str) -> ScSymbol {
        ScSymbol(StringM::try_from(s).unwrap())
    }

    fn name30(s: &str) -> StringM<30> {
        StringM::try_from(s).unwrap()
    }

    fn name60(s: &str) -> StringM<60> {
        StringM::try_from(s).unwrap()
    }

    #[test]
    fn normalizes_scalar_types() {
        assert_eq!(
            normalize_type(&ScSpecTypeDef::U32).unwrap(),
            NormalizedType::U32
        );
        assert_eq!(
            normalize_type(&ScSpecTypeDef::Address).unwrap(),
            NormalizedType::Address
        );
    }

    #[test]
    fn normalizes_nested_option_of_tuple() {
        let type_def = ScSpecTypeDef::Option(Box::new(ScSpecTypeOption {
            value_type: Box::new(ScSpecTypeDef::Tuple(Box::new(ScSpecTypeTuple {
                value_types: VecM::try_from(vec![ScSpecTypeDef::U32, ScSpecTypeDef::Bool]).unwrap(),
            }))),
        }));

        let normalized = normalize_type(&type_def).unwrap();
        assert_eq!(
            normalized,
            NormalizedType::Option(Box::new(NormalizedType::Tuple(vec![
                NormalizedType::U32,
                NormalizedType::Bool,
            ])))
        );
    }

    #[test]
    fn normalizes_bytes_n_and_udt() {
        assert_eq!(
            normalize_type(&ScSpecTypeDef::BytesN(ScSpecTypeBytesN { n: 32 })).unwrap(),
            NormalizedType::BytesN(32)
        );
        assert_eq!(
            normalize_type(&ScSpecTypeDef::Udt(ScSpecTypeUdt {
                name: name60("Balance")
            }))
            .unwrap(),
            NormalizedType::Udt("Balance".to_string())
        );
    }

    #[test]
    fn normalizes_function_with_inputs_and_output() {
        let function = ScSpecFunctionV0 {
            doc: StringM::default(),
            name: symbol("transfer"),
            inputs: VecM::try_from(vec![
                ScSpecFunctionInputV0 {
                    doc: StringM::default(),
                    name: name30("to"),
                    type_: ScSpecTypeDef::Address,
                },
                ScSpecFunctionInputV0 {
                    doc: StringM::default(),
                    name: name30("amount"),
                    type_: ScSpecTypeDef::I128,
                },
            ])
            .unwrap(),
            outputs: VecM::try_from(vec![ScSpecTypeDef::Bool]).unwrap(),
        };

        let normalized = normalize_function(&function).unwrap();
        assert_eq!(normalized.name, "transfer");
        assert_eq!(normalized.inputs.len(), 2);
        assert_eq!(normalized.inputs[0].name, "to");
        assert_eq!(normalized.inputs[1].type_, NormalizedType::I128);
        assert_eq!(normalized.output, Some(NormalizedType::Bool));
    }

    #[test]
    fn function_with_no_output_normalizes_to_none() {
        let function = ScSpecFunctionV0 {
            doc: StringM::default(),
            name: symbol("initialize"),
            inputs: VecM::default(),
            outputs: VecM::default(),
        };
        let normalized = normalize_function(&function).unwrap();
        assert_eq!(normalized.output, None);
    }

    #[test]
    fn normalizes_struct_fields_in_order() {
        let udt = ScSpecUdtStructV0 {
            doc: StringM::default(),
            lib: StringM::default(),
            name: name60("Balance"),
            fields: VecM::try_from(vec![
                ScSpecUdtStructFieldV0 {
                    doc: StringM::default(),
                    name: name30("amount"),
                    type_: ScSpecTypeDef::I128,
                },
                ScSpecUdtStructFieldV0 {
                    doc: StringM::default(),
                    name: name30("authorized"),
                    type_: ScSpecTypeDef::Bool,
                },
            ])
            .unwrap(),
        };

        let normalized = normalize_struct(&udt).unwrap();
        assert_eq!(normalized.name, "Balance");
        assert_eq!(
            normalized.fields,
            vec![
                NormalizedStructField {
                    name: "amount".to_string(),
                    type_: NormalizedType::I128,
                },
                NormalizedStructField {
                    name: "authorized".to_string(),
                    type_: NormalizedType::Bool,
                },
            ]
        );
    }

    #[test]
    fn normalizes_union_void_and_tuple_cases() {
        let udt = ScSpecUdtUnionV0 {
            doc: StringM::default(),
            lib: StringM::default(),
            name: name60("Status"),
            cases: VecM::try_from(vec![
                ScSpecUdtUnionCaseV0::VoidV0(ScSpecUdtUnionCaseVoidV0 {
                    doc: StringM::default(),
                    name: name60("Active"),
                }),
                ScSpecUdtUnionCaseV0::TupleV0(ScSpecUdtUnionCaseTupleV0 {
                    doc: StringM::default(),
                    name: name60("Frozen"),
                    type_: VecM::try_from(vec![ScSpecTypeDef::U64]).unwrap(),
                }),
            ])
            .unwrap(),
        };

        let normalized = normalize_union(&udt).unwrap();
        assert_eq!(
            normalized.cases,
            vec![
                NormalizedUnionCase::Void {
                    name: "Active".to_string()
                },
                NormalizedUnionCase::Tuple {
                    name: "Frozen".to_string(),
                    types: vec![NormalizedType::U64],
                },
            ]
        );
    }

    #[test]
    fn normalizes_enum_cases_with_explicit_values() {
        let udt = ScSpecUdtEnumV0 {
            doc: StringM::default(),
            lib: StringM::default(),
            name: name60("Role"),
            cases: VecM::try_from(vec![
                ScSpecUdtEnumCaseV0 {
                    doc: StringM::default(),
                    name: name60("Admin"),
                    value: 0,
                },
                ScSpecUdtEnumCaseV0 {
                    doc: StringM::default(),
                    name: name60("Member"),
                    value: 1,
                },
            ])
            .unwrap(),
        };

        let normalized = normalize_enum(&udt).unwrap();
        assert_eq!(
            normalized.cases,
            vec![
                NormalizedEnumCase {
                    name: "Admin".to_string(),
                    value: 0
                },
                NormalizedEnumCase {
                    name: "Member".to_string(),
                    value: 1
                },
            ]
        );
    }

    #[test]
    fn full_entry_stream_normalizes_and_groups_by_kind() {
        let function = ScSpecEntry::FunctionV0(ScSpecFunctionV0 {
            doc: StringM::default(),
            name: symbol("noop"),
            inputs: VecM::default(),
            outputs: VecM::default(),
        });
        let interface = normalize_interface(&[function]).unwrap();
        assert_eq!(interface.functions.len(), 1);
        assert!(interface.structs.is_empty());
        assert_eq!(interface.functions[0].name, "noop");
    }
}
