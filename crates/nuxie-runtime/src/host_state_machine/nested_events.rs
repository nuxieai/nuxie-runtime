//! Observe the live occurrence tree without consuming native listener queues.
use super::{StateMachineEventContext, StateMachineReportedEvent};
use crate::host_semantics::{retained_occurrence_identity, source_artboard_global_id};
use crate::mechanical_port::source::{
    animation::{
        nested_state_machine::NestedStateMachine,
        state_machine_instance::RuntimeStateMachineInstanceHandle,
    },
    artboard::RuntimeArtboardInstanceHandle,
    artboard_component_list::ArtboardComponentList,
};
use crate::{RuntimeGeometryHitOccurrence, RuntimeGeometryHitPathSegment};
use std::collections::HashSet;

type Child = (
    RuntimeArtboardInstanceHandle,
    Vec<RuntimeStateMachineInstanceHandle>,
    Vec<RuntimeGeometryHitPathSegment>,
    Vec<RuntimeGeometryHitOccurrence>,
);

pub(super) fn collect(
    root: &RuntimeArtboardInstanceHandle,
    after: u64,
) -> Vec<StateMachineReportedEvent> {
    let mut events = Vec::new();
    let mut pending: Vec<Child> = vec![(root.clone(), Vec::new(), Vec::new(), Vec::new())];
    let mut visited = HashSet::new();
    while let Some((artboard, machines, path, occurrence)) = pending.pop() {
        if !visited.insert(artboard.core_handle().identity_key()) {
            continue;
        }
        for machine in machines {
            let reports = machine.with_instance(|machine| {
                (0..machine.reported_event_count())
                    .map(|index| machine.reported_event_at(index))
                    .collect::<Vec<_>>()
            });
            for report in reports {
                if report.host_sequence <= after {
                    continue;
                }
                let view_model_instance_id = report.host_view_model_instance_id;
                if let Some(event) = StateMachineReportedEvent::from_native(
                    report,
                    &artboard,
                    Some(StateMachineEventContext {
                        path: path.clone(),
                        occurrence: occurrence.clone(),
                        view_model_instance_id,
                    }),
                ) {
                    events.push(event);
                }
            }
        }
        let Some(artboard_global_id) = source_artboard_global_id(&artboard) else {
            continue;
        };
        let objects = artboard.with_artboard(|artboard| artboard.base.objects().to_vec());
        for (local_id, object) in objects.into_iter().enumerate() {
            let Some(object) = object else {
                continue;
            };
            let mut child_path = path.clone();
            child_path.push(RuntimeGeometryHitPathSegment {
                artboard_global_id,
                local_id,
            });
            if let Some((child, machines)) = object
                .with(|object| {
                    let nested = object.as_nested_artboard()?;
                    let child = nested.artboard_instance_default()?;
                    let machines = nested
                        .nested_animations()
                        .iter()
                        .filter_map(|animation| {
                            animation
                                .with_downcast::<NestedStateMachine, _>(
                                    NestedStateMachine::state_machine_instance,
                                )
                                .flatten()
                        })
                        .collect();
                    Some((child, machines))
                })
                .flatten()
            {
                pending.push((child, machines, child_path, occurrence.clone()));
                continue;
            }
            object.with_downcast::<ArtboardComponentList, _>(|list| {
                for index in 0..list.artboard_count() {
                    let Ok(item_index) = i32::try_from(index) else {
                        continue;
                    };
                    let (Some(child), Some(item)) = (
                        list.artboard_instance(item_index),
                        list.list_item(item_index),
                    ) else {
                        continue;
                    };
                    let mut child_occurrence = occurrence.clone();
                    child_occurrence.push(RuntimeGeometryHitOccurrence {
                        artboard_global_id,
                        host_local_id: local_id,
                        item_index: index,
                        occurrence_identity: retained_occurrence_identity(&item),
                    });
                    pending.push((
                        child,
                        list.state_machine_instance(item_index)
                            .into_iter()
                            .collect(),
                        child_path.clone(),
                        child_occurrence,
                    ));
                }
            });
        }
    }
    events
}
