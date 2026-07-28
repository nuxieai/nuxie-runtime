use crate::focus::RuntimeFocusTree;

#[derive(Debug, Clone)]
pub(crate) struct RuntimeFocusActionTraversal {
    pub(crate) action_owner: super::RuntimeActionCoreHandle,
}

impl RuntimeFocusActionTraversal {
    #[cfg(test)]
    pub(crate) fn for_test(flags: u64, traversal_kind: u64) -> Self {
        let action_owner = super::RuntimeActionCoreHandle::for_test("FocusActionTraversal");
        action_owner.set_uint(super::listener_action_owner::LISTENER_FLAGS_KEY, flags);
        action_owner.set_uint(
            super::listener_action_owner::FOCUS_TRAVERSAL_KIND_KEY,
            traversal_kind,
        );
        Self { action_owner }
    }

    /// Values 0–5 map to next/previous/up/down/left/right; C++ defaults every
    /// other authored value to next.
    pub(crate) fn perform(
        &self,
        _artboard: &crate::ArtboardInstance,
        focus: &mut RuntimeFocusTree,
    ) -> bool {
        focus.traverse(
            self.action_owner
                .uint(super::listener_action_owner::FOCUS_TRAVERSAL_KIND_KEY),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ArtboardInstance;
    use crate::state_machine::focus_action_clear::RuntimeFocusActionClear;
    use nuxie_binary::{AuthoringProperty, AuthoringRecord, AuthoringValue, RuntimeFile};
    use nuxie_graph::GraphFile;

    fn record(type_name: &str, properties: Vec<AuthoringProperty>) -> AuthoringRecord {
        AuthoringRecord {
            type_key: nuxie_schema::definition_by_name(type_name)
                .unwrap_or_else(|| panic!("missing {type_name}"))
                .type_key
                .int,
            properties,
        }
    }

    fn property(type_name: &str, name: &str, value: AuthoringValue) -> AuthoringProperty {
        AuthoringProperty {
            key: crate::properties::property_key_for_name(type_name, name)
                .unwrap_or_else(|| panic!("missing {type_name}.{name}")),
            value,
        }
    }

    fn focus_artboard() -> ArtboardInstance {
        let mut records = vec![
            record("Backboard", Vec::new()),
            record("Artboard", Vec::new()),
        ];
        for (x, y) in [
            (0.0, 0.0),
            (10.0, 0.0),
            (-10.0, 0.0),
            (0.0, -10.0),
            (0.0, 10.0),
        ] {
            let node_local_id = records.len() - 1;
            records.push(record(
                "Node",
                vec![
                    property("Node", "parentId", AuthoringValue::Uint(0)),
                    property("Node", "x", AuthoringValue::Double(x)),
                    property("Node", "y", AuthoringValue::Double(y)),
                ],
            ));
            records.push(record(
                "FocusData",
                vec![
                    property(
                        "FocusData",
                        "parentId",
                        AuthoringValue::Uint(node_local_id as u64),
                    ),
                    property("FocusData", "focusFlags", AuthoringValue::Uint(7)),
                ],
            ));
        }
        let file = RuntimeFile::from_authoring_records(records).expect("focus action records");
        let graph = GraphFile::from_runtime_file(&file).expect("focus action graph");
        let mut artboard = ArtboardInstance::from_graph_with_artboards(
            &file,
            graph.artboards.first().expect("focus action artboard"),
            &graph.artboards,
        )
        .expect("focus action instance");
        artboard.update_components();
        artboard
    }

    fn focused_target(focus: &RuntimeFocusTree) -> Option<usize> {
        focus
            .focused_listener_chain()
            .first()
            .map(|(_, target, _)| *target)
    }

    #[test]
    fn traversal_maps_all_cpp_values_invalid_to_next_and_clear_is_idempotent() {
        let artboard = focus_artboard();
        let mut focus = RuntimeFocusTree::new_unsynchronized(&artboard);
        focus.synchronize_after_layer_initialization(&artboard);

        for (kind, start, expected) in [
            (0, 1, 3),
            (1, 3, 1),
            (2, 1, 7),
            (3, 1, 9),
            (4, 1, 5),
            (5, 1, 3),
            (99, 1, 3),
        ] {
            focus.clear_focus();
            assert!(focus.set_focus_target(start));
            assert!(
                RuntimeFocusActionTraversal::for_test(0, kind).perform(&artboard, &mut focus),
                "traversal kind {kind}"
            );
            assert_eq!(focused_target(&focus), Some(expected), "kind {kind}");
        }

        let clear = RuntimeFocusActionClear::for_test(0);
        assert!(clear.perform(&mut focus));
        assert_eq!(focused_target(&focus), None);
        assert!(!clear.perform(&mut focus));
    }
}
