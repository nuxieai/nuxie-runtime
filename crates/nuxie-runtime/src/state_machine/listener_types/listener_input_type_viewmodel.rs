use nuxie_binary::RuntimeObject;

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
    view_model_index: Option<usize>,
    property_path: Vec<usize>,
}

impl RuntimeListenerInputTypeViewModel {
    pub(in crate::state_machine) fn from_imported(input_type: &RuntimeObject) -> Self {
        let path = input_type
            .id_list_property("viewModelPathIds")
            .and_then(|encoded| {
                let (view_model_index, property_path) = encoded.split_first()?;
                let view_model_index = usize::try_from(*view_model_index).ok()?;
                let property_path = property_path
                    .iter()
                    .copied()
                    .map(usize::try_from)
                    .collect::<Result<Vec<_>, _>>()
                    .ok()?;
                (!property_path.is_empty()).then_some((view_model_index, property_path))
            });
        let (view_model_index, property_path) = path
            .map(|(view_model_index, property_path)| (Some(view_model_index), property_path))
            .unwrap_or((None, Vec::new()));
        Self {
            global_id: input_type.id,
            view_model_index,
            property_path,
        }
    }

    pub(crate) fn source_path(&self) -> Option<(usize, &[usize])> {
        self.view_model_index
            .map(|view_model_index| (view_model_index, self.property_path.as_slice()))
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

        let input = RuntimeListenerInputTypeViewModel::from_imported(object);

        assert_eq!(input.global_id, object.id);
        assert_eq!(input.source_path(), Some((3, [5, 8].as_slice())));
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
