use nuxie_binary::{RuntimeDataBindPath, RuntimeFile, RuntimeObject};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum RuntimeListenerViewModelPath {
    Absolute {
        view_model_index: usize,
        property_path: Vec<usize>,
    },
    Relative {
        resolved_name_ids: Vec<u32>,
        absolute_fallback: Option<(usize, Vec<usize>)>,
    },
}

impl RuntimeListenerViewModelPath {
    pub(crate) fn from_data_bind_path(path: RuntimeDataBindPath<'_>) -> Option<Self> {
        if path.is_relative {
            return (!path.resolved_path_ids.is_empty()).then_some(Self::Relative {
                resolved_name_ids: path.resolved_path_ids,
                absolute_fallback: Self::absolute_components(&path.path_ids),
            });
        }

        let (view_model_index, property_path) = Self::absolute_components(&path.path_ids)?;
        Some(Self::Absolute {
            view_model_index,
            property_path,
        })
    }

    fn absolute_components(path_ids: &[u32]) -> Option<(usize, Vec<usize>)> {
        let (view_model_index, property_path) = path_ids.split_first()?;
        let view_model_index = usize::try_from(*view_model_index).ok()?;
        let property_path = property_path
            .iter()
            .copied()
            .map(usize::try_from)
            .collect::<Result<Vec<_>, _>>()
            .ok()?;
        (!property_path.is_empty()).then_some((view_model_index, property_path))
    }

    #[cfg(test)]
    pub(crate) fn absolute_source_path(&self) -> Option<(usize, &[usize])> {
        match self {
            Self::Absolute {
                view_model_index,
                property_path,
            } => Some((*view_model_index, property_path)),
            Self::Relative { .. } => None,
        }
    }
}

/// Authored `ListenerInputTypeViewModel` definition shared by state-machine
/// occurrences.
///
/// C++ retains the `DataBindPath` on this definition and each
/// `ListenerViewModelPropertyBindingInput` keeps a pointer back to the
/// definition when it relinks
/// (`listener_input_type_viewmodel.cpp`;
/// `state_machine_instance.cpp:1349-1372,1454-1478`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RuntimeListenerInputTypeViewModel {
    pub(crate) global_id: u32,
    path: Option<RuntimeListenerViewModelPath>,
}

impl RuntimeListenerInputTypeViewModel {
    pub(in crate::state_machine) fn from_imported(
        file: &RuntimeFile,
        input_type: &RuntimeObject,
    ) -> Self {
        Self {
            global_id: input_type.id,
            path: file
                .data_bind_path_for_referencer_object(input_type)
                .and_then(RuntimeListenerViewModelPath::from_data_bind_path),
        }
    }

    pub(crate) fn path(&self) -> Option<&RuntimeListenerViewModelPath> {
        self.path.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nuxie_binary::{AuthoringProperty, AuthoringRecord, AuthoringValue, RuntimeFile};

    #[test]
    fn imported_definition_retains_exact_authored_path() {
        let file = RuntimeFile::from_authoring_records(vec![
            record("Backboard", Vec::new()),
            record("Artboard", Vec::new()),
            record("StateMachine", Vec::new()),
            record(
                "StateMachineListener",
                vec![property(
                    "StateMachineListener",
                    "targetId",
                    AuthoringValue::Uint(0),
                )],
            ),
            record(
                "ListenerInputTypeViewModel",
                vec![property(
                    "ListenerInputTypeViewModel",
                    "viewModelPathIds",
                    AuthoringValue::Bytes(vec![3, 5, 8]),
                )],
            ),
        ])
        .expect("view-model listener input imports");
        let object = file
            .objects
            .iter()
            .flatten()
            .find(|object| object.type_name == "ListenerInputTypeViewModel")
            .expect("imported definition");

        let input = RuntimeListenerInputTypeViewModel::from_imported(&file, object);

        assert_eq!(input.global_id, object.id);
        assert_eq!(
            input.path().and_then(|path| path.absolute_source_path()),
            Some((3, [5, 8].as_slice()))
        );
    }

    fn record(type_name: &str, properties: Vec<AuthoringProperty>) -> AuthoringRecord {
        AuthoringRecord {
            type_key: nuxie_schema::definition_by_name(type_name)
                .unwrap_or_else(|| panic!("missing schema definition {type_name}"))
                .type_key
                .int,
            properties,
        }
    }

    fn property(type_name: &str, name: &str, value: AuthoringValue) -> AuthoringProperty {
        let definition = nuxie_schema::definition_by_name(type_name)
            .unwrap_or_else(|| panic!("missing schema definition {type_name}"));
        AuthoringProperty {
            key: definition
                .properties
                .iter()
                .find(|property| property.name == name)
                .unwrap_or_else(|| panic!("missing property {type_name}.{name}"))
                .key
                .int,
            value,
        }
    }
}
