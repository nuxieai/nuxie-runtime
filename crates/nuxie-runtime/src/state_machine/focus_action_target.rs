use crate::ArtboardInstance;
use crate::focus::RuntimeFocusTree;

#[derive(Debug, Clone)]
pub(crate) struct RuntimeFocusActionTarget {
    pub(crate) action_owner: super::RuntimeActionCoreHandle,
}

impl RuntimeFocusActionTarget {
    #[cfg(test)]
    pub(crate) fn for_test(flags: u64, target_local_id: Option<usize>) -> Self {
        let action_owner = super::RuntimeActionCoreHandle::for_test("FocusActionTarget");
        action_owner.set_uint(super::listener_action_owner::LISTENER_FLAGS_KEY, flags);
        action_owner.set_uint(
            super::listener_action_owner::FOCUS_TARGET_ID_KEY,
            target_local_id
                .and_then(|value| u64::try_from(value).ok())
                .unwrap_or(u64::from(u32::MAX)),
        );
        Self { action_owner }
    }

    /// Mirrors `FocusActionTarget::perform`: resolve an exact Node, select its
    /// first direct FocusData child in authored order, then focus that
    /// occurrence. The focus tree itself preserves constructor-time lazy
    /// unattached-node creation.
    pub(crate) fn perform(
        &self,
        artboard: &ArtboardInstance,
        focus: &mut RuntimeFocusTree,
    ) -> bool {
        let target_local_id = self
            .action_owner
            .uint(super::listener_action_owner::FOCUS_TARGET_ID_KEY);
        let Ok(target_local_id) = usize::try_from(target_local_id) else {
            return false;
        };
        let Some(target_handle) = artboard.component_handle(target_local_id) else {
            return false;
        };
        let target = artboard.component_at(target_handle);
        if !nuxie_schema::definition_by_name(target.type_name)
            .is_some_and(|definition| definition.is_a("Node"))
        {
            return false;
        }
        let focus_data_local_id =
            (0..artboard.component_child_len(target_handle)).find_map(|index| {
                let child = artboard.component_child_at(target_handle, index)?;
                let child = artboard.component_at(child);
                (child.type_name == "FocusData").then_some(child.local_id)
            });
        let Some(focus_data_local_id) = focus_data_local_id else {
            return false;
        };
        // `FocusManager::setFocus` gates on the same live
        // `isEligibleForFocusTraversal` walk as traversal
        // (`src/input/focus_manager.cpp:118-141`), so retained eligibility is
        // resynchronized from live component state at this query boundary
        // too. Before the first full build (constructor-time entry actions)
        // this occurrence has no mount and the refresh is a no-op, preserving
        // the pinned lazy unattached-node path.
        focus.refresh_visibility_change(artboard);
        focus.set_focus_target_before_topology(artboard, target_local_id, focus_data_local_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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

    fn artboard() -> ArtboardInstance {
        let file = RuntimeFile::from_authoring_records(vec![
            record("Backboard", Vec::new()),
            record("Artboard", Vec::new()),
            record(
                "Node",
                vec![property("Node", "parentId", AuthoringValue::Uint(0))],
            ),
            record(
                "FocusData",
                vec![
                    property("FocusData", "parentId", AuthoringValue::Uint(1)),
                    property("FocusData", "focusFlags", AuthoringValue::Uint(7)),
                ],
            ),
            record(
                "FocusData",
                vec![
                    property("FocusData", "parentId", AuthoringValue::Uint(1)),
                    property("FocusData", "focusFlags", AuthoringValue::Uint(7)),
                ],
            ),
            record(
                "Event",
                vec![property("Event", "parentId", AuthoringValue::Uint(0))],
            ),
        ])
        .expect("focus target records import");
        let graph = GraphFile::from_runtime_file(&file).expect("focus target graph builds");
        ArtboardInstance::from_graph_with_artboards(
            &file,
            graph.artboards.first().expect("focus target artboard"),
            &graph.artboards,
        )
        .expect("focus target artboard instantiates")
    }

    #[test]
    fn action_targets_the_first_direct_focus_data_before_and_after_topology_build() {
        let mut artboard = artboard();
        artboard.update_components();
        let target = artboard.component_handle(1).expect("target node");
        assert_eq!(
            (0..artboard.component_child_len(target))
                .filter_map(|index| artboard.component_child_at(target, index))
                .map(|child| artboard.component_at(child).local_id)
                .collect::<Vec<_>>(),
            vec![2, 3]
        );
        let action = RuntimeFocusActionTarget::for_test(0, Some(1));

        let mut constructor_focus = RuntimeFocusTree::new_unsynchronized(&artboard);
        let constructor_changed = action.perform(&artboard, &mut constructor_focus);
        assert_eq!(
            constructor_focus.focused_listener_chain(),
            vec![(artboard.instance_identity(), 1, 2)]
        );
        assert!(constructor_changed);

        let mut live_focus = RuntimeFocusTree::new_unsynchronized(&artboard);
        live_focus.synchronize_after_layer_initialization(&artboard);
        assert!(action.perform(&artboard, &mut live_focus));
        assert_eq!(
            live_focus.focused_listener_chain(),
            vec![(artboard.instance_identity(), 1, 2)],
            "the later topology build must not replace the first authored FocusData with the duplicate"
        );

        assert!(
            !RuntimeFocusActionTarget::for_test(0, Some(99)).perform(&artboard, &mut live_focus)
        );
        assert!(
            !RuntimeFocusActionTarget::for_test(0, Some(4)).perform(&artboard, &mut live_focus),
            "an existing non-Node target is the same no-op as pinned C++"
        );
    }
}
