//! Rule-based comparison between two normalized contract interfaces
//! ([`crate::interface::NormalizedInterface`]).
//!
//! Every difference this module can detect is represented by a specific
//! variant of one of the `*Change` enums below, never by a generic
//! "interface changed" catch-all. Entries are matched between the
//! current and candidate interface by their declared name; names that
//! only appear on one side are additions or removals, and names present
//! on both sides are compared structurally. A future finding-synthesis
//! layer (`analyzer-core`) is responsible for mapping each change to a
//! stable analyzer rule identifier and a severity/confidence; this
//! module only establishes *what* differs.

use std::collections::BTreeMap;

use crate::interface::{
    EventDataFormat, NormalizedEnumCase, NormalizedErrorEnumCase, NormalizedEventParameter,
    NormalizedInterface, NormalizedParameter, NormalizedStructField, NormalizedUnionCase,
};

/// A change to a single contract entrypoint (function).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FunctionChange {
    /// The candidate declares a function the current executable does
    /// not.
    Added { name: String },
    /// The current executable declares a function the candidate does
    /// not.
    Removed { name: String },
    /// The function exists on both sides, but with a different number
    /// of input parameters.
    InputCountChanged {
        name: String,
        current_count: usize,
        candidate_count: usize,
    },
    /// The function exists on both sides with the same input count and
    /// the same set of (name, type) pairs, but in a different order.
    InputOrderChanged {
        name: String,
        current_order: Vec<String>,
        candidate_order: Vec<String>,
    },
    /// The function exists on both sides with the same input count, and
    /// the parameter at `index` differs in name and/or type (and this
    /// is not fully explained by a pure reordering).
    InputChanged {
        name: String,
        index: usize,
        current: NormalizedParameter,
        candidate: NormalizedParameter,
    },
    /// The function's declared return type differs.
    OutputChanged {
        name: String,
        current: Option<crate::interface::NormalizedType>,
        candidate: Option<crate::interface::NormalizedType>,
    },
}

/// A change to a user-defined struct type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StructChange {
    Added {
        name: String,
    },
    Removed {
        name: String,
    },
    FieldsChanged {
        name: String,
        current: Vec<NormalizedStructField>,
        candidate: Vec<NormalizedStructField>,
    },
}

/// A change to a user-defined union type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnionChange {
    Added {
        name: String,
    },
    Removed {
        name: String,
    },
    CasesChanged {
        name: String,
        current: Vec<NormalizedUnionCase>,
        candidate: Vec<NormalizedUnionCase>,
    },
}

/// A change to a user-defined enum type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnumChange {
    Added {
        name: String,
    },
    Removed {
        name: String,
    },
    CasesChanged {
        name: String,
        current: Vec<NormalizedEnumCase>,
        candidate: Vec<NormalizedEnumCase>,
    },
}

/// A change to a user-defined error enum type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorEnumChange {
    Added {
        name: String,
    },
    Removed {
        name: String,
    },
    CasesChanged {
        name: String,
        current: Vec<NormalizedErrorEnumCase>,
        candidate: Vec<NormalizedErrorEnumCase>,
    },
}

/// A change to a contract event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventChange {
    Added {
        name: String,
    },
    Removed {
        name: String,
    },
    PrefixTopicsChanged {
        name: String,
        current: Vec<String>,
        candidate: Vec<String>,
    },
    ParametersChanged {
        name: String,
        current: Vec<NormalizedEventParameter>,
        candidate: Vec<NormalizedEventParameter>,
    },
    DataFormatChanged {
        name: String,
        current: EventDataFormat,
        candidate: EventDataFormat,
    },
}

/// Every detected difference between a current and candidate normalized
/// interface, grouped by the kind of entry that changed.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct InterfaceDiff {
    pub function_changes: Vec<FunctionChange>,
    pub struct_changes: Vec<StructChange>,
    pub union_changes: Vec<UnionChange>,
    pub enum_changes: Vec<EnumChange>,
    pub error_enum_changes: Vec<ErrorEnumChange>,
    pub event_changes: Vec<EventChange>,
}

impl InterfaceDiff {
    /// True if no difference was detected in any category.
    pub fn is_empty(&self) -> bool {
        self.function_changes.is_empty()
            && self.struct_changes.is_empty()
            && self.union_changes.is_empty()
            && self.enum_changes.is_empty()
            && self.error_enum_changes.is_empty()
            && self.event_changes.is_empty()
    }
}

/// Compare a current and candidate normalized interface, producing an
/// [`InterfaceDiff`] that names every detected difference specifically.
///
/// Entries are matched by declared name, in deterministic (sorted)
/// name order, so the result does not depend on the order entries
/// happened to appear in either `contractspecv0` section.
pub fn diff_interfaces(
    current: &NormalizedInterface,
    candidate: &NormalizedInterface,
) -> InterfaceDiff {
    InterfaceDiff {
        function_changes: diff_functions(current, candidate),
        struct_changes: diff_by_name(
            &current.structs,
            &candidate.structs,
            |s| s.name.as_str(),
            |name| StructChange::Added { name },
            |name| StructChange::Removed { name },
            |name, current, candidate| {
                (current.fields != candidate.fields).then(|| StructChange::FieldsChanged {
                    name,
                    current: current.fields.clone(),
                    candidate: candidate.fields.clone(),
                })
            },
        ),
        union_changes: diff_by_name(
            &current.unions,
            &candidate.unions,
            |u| u.name.as_str(),
            |name| UnionChange::Added { name },
            |name| UnionChange::Removed { name },
            |name, current, candidate| {
                (current.cases != candidate.cases).then(|| UnionChange::CasesChanged {
                    name,
                    current: current.cases.clone(),
                    candidate: candidate.cases.clone(),
                })
            },
        ),
        enum_changes: diff_by_name(
            &current.enums,
            &candidate.enums,
            |e| e.name.as_str(),
            |name| EnumChange::Added { name },
            |name| EnumChange::Removed { name },
            |name, current, candidate| {
                (current.cases != candidate.cases).then(|| EnumChange::CasesChanged {
                    name,
                    current: current.cases.clone(),
                    candidate: candidate.cases.clone(),
                })
            },
        ),
        error_enum_changes: diff_by_name(
            &current.error_enums,
            &candidate.error_enums,
            |e| e.name.as_str(),
            |name| ErrorEnumChange::Added { name },
            |name| ErrorEnumChange::Removed { name },
            |name, current, candidate| {
                (current.cases != candidate.cases).then(|| ErrorEnumChange::CasesChanged {
                    name,
                    current: current.cases.clone(),
                    candidate: candidate.cases.clone(),
                })
            },
        ),
        event_changes: diff_events(current, candidate),
    }
}

/// Match two slices of named entries by name (deterministic, sorted
/// order) and apply `on_added`/`on_removed`/`on_both` to build the
/// change list. `on_both` returns `None` when the matched pair is
/// identical.
fn diff_by_name<'a, T, C>(
    current: &'a [T],
    candidate: &'a [T],
    name_of: impl Fn(&T) -> &str,
    on_added: impl Fn(String) -> C,
    on_removed: impl Fn(String) -> C,
    on_both: impl Fn(String, &T, &T) -> Option<C>,
) -> Vec<C> {
    let current_by_name: BTreeMap<&str, &T> =
        current.iter().map(|item| (name_of(item), item)).collect();
    let candidate_by_name: BTreeMap<&str, &T> =
        candidate.iter().map(|item| (name_of(item), item)).collect();

    let mut all_names: Vec<&str> = current_by_name
        .keys()
        .chain(candidate_by_name.keys())
        .copied()
        .collect();
    all_names.sort_unstable();
    all_names.dedup();

    let mut changes = Vec::new();
    for name in all_names {
        match (current_by_name.get(name), candidate_by_name.get(name)) {
            (Some(_), None) => changes.push(on_removed(name.to_string())),
            (None, Some(_)) => changes.push(on_added(name.to_string())),
            (Some(current_item), Some(candidate_item)) => {
                if let Some(change) = on_both(name.to_string(), current_item, candidate_item) {
                    changes.push(change);
                }
            }
            (None, None) => unreachable!("name collected from one of the two maps"),
        }
    }
    changes
}

fn diff_functions(
    current: &NormalizedInterface,
    candidate: &NormalizedInterface,
) -> Vec<FunctionChange> {
    diff_by_name(
        &current.functions,
        &candidate.functions,
        |f| f.name.as_str(),
        |name| FunctionChange::Added { name },
        |name| FunctionChange::Removed { name },
        diff_function_signature,
    )
}

fn diff_function_signature(
    name: String,
    current: &crate::interface::NormalizedFunction,
    candidate: &crate::interface::NormalizedFunction,
) -> Option<FunctionChange> {
    if current.inputs.len() != candidate.inputs.len() {
        return Some(FunctionChange::InputCountChanged {
            name,
            current_count: current.inputs.len(),
            candidate_count: candidate.inputs.len(),
        });
    }

    if current.inputs != candidate.inputs {
        let is_reorder = {
            let mut current_sorted = current.inputs.clone();
            let mut candidate_sorted = candidate.inputs.clone();
            current_sorted.sort_by(|a, b| a.name.cmp(&b.name));
            candidate_sorted.sort_by(|a, b| a.name.cmp(&b.name));
            current_sorted == candidate_sorted
        };

        if is_reorder {
            return Some(FunctionChange::InputOrderChanged {
                name,
                current_order: current.inputs.iter().map(|p| p.name.clone()).collect(),
                candidate_order: candidate.inputs.iter().map(|p| p.name.clone()).collect(),
            });
        }

        for (index, (current_input, candidate_input)) in current
            .inputs
            .iter()
            .zip(candidate.inputs.iter())
            .enumerate()
        {
            if current_input != candidate_input {
                return Some(FunctionChange::InputChanged {
                    name,
                    index,
                    current: current_input.clone(),
                    candidate: candidate_input.clone(),
                });
            }
        }
    }

    if current.output != candidate.output {
        return Some(FunctionChange::OutputChanged {
            name,
            current: current.output.clone(),
            candidate: candidate.output.clone(),
        });
    }

    None
}

fn diff_events(current: &NormalizedInterface, candidate: &NormalizedInterface) -> Vec<EventChange> {
    diff_by_name(
        &current.events,
        &candidate.events,
        |e| e.name.as_str(),
        |name| EventChange::Added { name },
        |name| EventChange::Removed { name },
        |name, current, candidate| {
            if current.prefix_topics != candidate.prefix_topics {
                return Some(EventChange::PrefixTopicsChanged {
                    name,
                    current: current.prefix_topics.clone(),
                    candidate: candidate.prefix_topics.clone(),
                });
            }
            if current.params != candidate.params {
                return Some(EventChange::ParametersChanged {
                    name,
                    current: current.params.clone(),
                    candidate: candidate.params.clone(),
                });
            }
            if current.data_format != candidate.data_format {
                return Some(EventChange::DataFormatChanged {
                    name,
                    current: current.data_format,
                    candidate: candidate.data_format,
                });
            }
            None
        },
    )
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::interface::{
        EventParamLocation, NormalizedEvent, NormalizedFunction, NormalizedType,
    };

    fn function(
        name: &str,
        inputs: Vec<(&str, NormalizedType)>,
        output: Option<NormalizedType>,
    ) -> NormalizedFunction {
        NormalizedFunction {
            name: name.to_string(),
            inputs: inputs
                .into_iter()
                .map(|(name, type_)| NormalizedParameter {
                    name: name.to_string(),
                    type_,
                })
                .collect(),
            output,
        }
    }

    fn interface_with_function(function: NormalizedFunction) -> NormalizedInterface {
        NormalizedInterface {
            functions: vec![function],
            ..Default::default()
        }
    }

    #[test]
    fn identical_interfaces_produce_no_changes() {
        let interface = interface_with_function(function(
            "transfer",
            vec![("to", NormalizedType::Address)],
            None,
        ));
        let diff = diff_interfaces(&interface, &interface);
        assert!(diff.is_empty());
    }

    #[test]
    fn detects_added_entrypoint() {
        let current = NormalizedInterface::default();
        let candidate = interface_with_function(function("mint", vec![], None));
        let diff = diff_interfaces(&current, &candidate);
        assert_eq!(
            diff.function_changes,
            vec![FunctionChange::Added {
                name: "mint".to_string()
            }]
        );
    }

    #[test]
    fn detects_removed_entrypoint() {
        let current = interface_with_function(function("burn", vec![], None));
        let candidate = NormalizedInterface::default();
        let diff = diff_interfaces(&current, &candidate);
        assert_eq!(
            diff.function_changes,
            vec![FunctionChange::Removed {
                name: "burn".to_string()
            }]
        );
    }

    #[test]
    fn detects_input_count_changed() {
        let current = interface_with_function(function(
            "transfer",
            vec![("to", NormalizedType::Address)],
            None,
        ));
        let candidate = interface_with_function(function(
            "transfer",
            vec![
                ("to", NormalizedType::Address),
                ("amount", NormalizedType::I128),
            ],
            None,
        ));
        let diff = diff_interfaces(&current, &candidate);
        assert_eq!(
            diff.function_changes,
            vec![FunctionChange::InputCountChanged {
                name: "transfer".to_string(),
                current_count: 1,
                candidate_count: 2,
            }]
        );
    }

    #[test]
    fn detects_input_type_changed() {
        let current = interface_with_function(function(
            "set_amount",
            vec![("amount", NormalizedType::U32)],
            None,
        ));
        let candidate = interface_with_function(function(
            "set_amount",
            vec![("amount", NormalizedType::U64)],
            None,
        ));
        let diff = diff_interfaces(&current, &candidate);
        assert_eq!(
            diff.function_changes,
            vec![FunctionChange::InputChanged {
                name: "set_amount".to_string(),
                index: 0,
                current: NormalizedParameter {
                    name: "amount".to_string(),
                    type_: NormalizedType::U32
                },
                candidate: NormalizedParameter {
                    name: "amount".to_string(),
                    type_: NormalizedType::U64
                },
            }]
        );
    }

    #[test]
    fn detects_input_order_changed() {
        let current = interface_with_function(function(
            "swap",
            vec![
                ("from", NormalizedType::Address),
                ("to", NormalizedType::Address),
            ],
            None,
        ));
        let candidate = interface_with_function(function(
            "swap",
            vec![
                ("to", NormalizedType::Address),
                ("from", NormalizedType::Address),
            ],
            None,
        ));
        let diff = diff_interfaces(&current, &candidate);
        assert_eq!(
            diff.function_changes,
            vec![FunctionChange::InputOrderChanged {
                name: "swap".to_string(),
                current_order: vec!["from".to_string(), "to".to_string()],
                candidate_order: vec!["to".to_string(), "from".to_string()],
            }]
        );
    }

    #[test]
    fn detects_output_type_changed() {
        let current =
            interface_with_function(function("balance", vec![], Some(NormalizedType::U32)));
        let candidate =
            interface_with_function(function("balance", vec![], Some(NormalizedType::I128)));
        let diff = diff_interfaces(&current, &candidate);
        assert_eq!(
            diff.function_changes,
            vec![FunctionChange::OutputChanged {
                name: "balance".to_string(),
                current: Some(NormalizedType::U32),
                candidate: Some(NormalizedType::I128),
            }]
        );
    }

    fn event(name: &str, params: Vec<NormalizedEventParameter>) -> NormalizedEvent {
        NormalizedEvent {
            name: name.to_string(),
            prefix_topics: vec![],
            params,
            data_format: EventDataFormat::SingleValue,
        }
    }

    #[test]
    fn detects_added_and_changed_event() {
        let current = NormalizedInterface {
            events: vec![event(
                "transfer",
                vec![NormalizedEventParameter {
                    name: "amount".to_string(),
                    type_: NormalizedType::I128,
                    location: EventParamLocation::Data,
                }],
            )],
            ..Default::default()
        };
        let candidate = NormalizedInterface {
            events: vec![
                event(
                    "transfer",
                    vec![NormalizedEventParameter {
                        name: "amount".to_string(),
                        type_: NormalizedType::U64,
                        location: EventParamLocation::Data,
                    }],
                ),
                event("mint", vec![]),
            ],
            ..Default::default()
        };

        let diff = diff_interfaces(&current, &candidate);
        assert_eq!(diff.event_changes.len(), 2);
        assert!(diff.event_changes.contains(&EventChange::Added {
            name: "mint".to_string()
        }));
        assert!(diff.event_changes.iter().any(|change| matches!(
            change,
            EventChange::ParametersChanged { name, .. } if name == "transfer"
        )));
    }

    #[test]
    fn detects_changed_user_defined_struct() {
        use crate::interface::NormalizedStruct;

        let current = NormalizedInterface {
            structs: vec![NormalizedStruct {
                name: "Balance".to_string(),
                fields: vec![NormalizedStructField {
                    name: "amount".to_string(),
                    type_: NormalizedType::I128,
                }],
            }],
            ..Default::default()
        };
        let candidate = NormalizedInterface {
            structs: vec![NormalizedStruct {
                name: "Balance".to_string(),
                fields: vec![
                    NormalizedStructField {
                        name: "amount".to_string(),
                        type_: NormalizedType::I128,
                    },
                    NormalizedStructField {
                        name: "authorized".to_string(),
                        type_: NormalizedType::Bool,
                    },
                ],
            }],
            ..Default::default()
        };

        let diff = diff_interfaces(&current, &candidate);
        assert_eq!(diff.struct_changes.len(), 1);
        assert!(matches!(
            &diff.struct_changes[0],
            StructChange::FieldsChanged { name, .. } if name == "Balance"
        ));
    }

    #[test]
    fn detects_changed_error_enum() {
        use crate::interface::NormalizedErrorEnum;

        let current = NormalizedInterface {
            error_enums: vec![NormalizedErrorEnum {
                name: "ContractError".to_string(),
                cases: vec![NormalizedErrorEnumCase {
                    name: "NotFound".to_string(),
                    value: 1,
                }],
            }],
            ..Default::default()
        };
        let candidate = NormalizedInterface {
            error_enums: vec![NormalizedErrorEnum {
                name: "ContractError".to_string(),
                cases: vec![
                    NormalizedErrorEnumCase {
                        name: "NotFound".to_string(),
                        value: 1,
                    },
                    NormalizedErrorEnumCase {
                        name: "Unauthorized".to_string(),
                        value: 2,
                    },
                ],
            }],
            ..Default::default()
        };

        let diff = diff_interfaces(&current, &candidate);
        assert_eq!(diff.error_enum_changes.len(), 1);
        assert!(matches!(
            &diff.error_enum_changes[0],
            ErrorEnumChange::CasesChanged { name, .. } if name == "ContractError"
        ));
    }
}
