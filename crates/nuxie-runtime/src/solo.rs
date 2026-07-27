use std::collections::BTreeMap;

#[cfg(test)]
use std::cell::Cell;

use nuxie_binary::RuntimeFile;
use nuxie_graph::ArtboardGraph;
use nuxie_schema::definition_by_name;

use crate::artboard::ArtboardInstance;
use crate::components::ComponentHandle;
use crate::objects::InstanceObjectArena;
use crate::properties::{artboard_index_for_graph, property_key_for_name};

/// Concrete, occurrence-owned members of C++ `Solo`.
///
/// `Solo` inherits its retained `children()` from `ContainerComponent`. The
/// parallel ids below add only the imported Artboard object-table identity
/// needed by generated `activeComponentId`; child identity itself remains
/// solely in the embedded Component base (`src/solo.cpp:8-31,50-81`). There is
/// deliberately no Artboard-side Solo registry or authored-id rediscovery.
#[derive(Debug, Clone)]
pub(crate) struct RuntimeSoloState {
    pub(crate) active_component_property_key: Option<u16>,
    pub(crate) cpp_local_ids: Vec<usize>,
}

impl RuntimeSoloState {
    pub(crate) fn new() -> Self {
        Self {
            active_component_property_key: property_key_for_name("Solo", "activeComponentId"),
            cpp_local_ids: Vec::new(),
        }
    }

    pub(crate) fn clone_for_occurrence(&self) -> Self {
        // Core/generated clone copies activeComponentId, while
        // ContainerComponent::onAddedDirty rebuilds this occurrence's child
        // pointers before Solo::onAddedClean propagates collapse
        // (`src/solo.cpp:38-48`; `src/container_component.cpp:8-37`).
        Self {
            active_component_property_key: self.active_component_property_key,
            cpp_local_ids: Vec::new(),
        }
    }
}

#[cfg(test)]
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SoloMappingWork {
    pub(crate) analyses: usize,
    pub(crate) batch_queries: usize,
    pub(crate) visited_slots: usize,
}

#[cfg(test)]
thread_local! {
    static SOLO_MAPPING_WORK: Cell<SoloMappingWork> = const {
        Cell::new(SoloMappingWork {
            analyses: 0,
            batch_queries: 0,
            visited_slots: 0,
        })
    };
}

#[cfg(test)]
pub(crate) fn reset_solo_mapping_work() {
    SOLO_MAPPING_WORK.set(SoloMappingWork::default());
}

#[cfg(test)]
pub(crate) fn solo_mapping_work() -> SoloMappingWork {
    SOLO_MAPPING_WORK.get()
}

#[cfg(test)]
fn record_solo_mapping_analysis() {
    SOLO_MAPPING_WORK.with(|slot| {
        let mut work = slot.get();
        work.analyses += 1;
        slot.set(work);
    });
}

#[cfg(test)]
fn record_solo_mapping_batch_query(visited_slots: usize) {
    SOLO_MAPPING_WORK.with(|slot| {
        let mut work = slot.get();
        work.batch_queries += 1;
        work.visited_slots += visited_slots;
        slot.set(work);
    });
}

pub(crate) fn retain_runtime_solos(
    file: &RuntimeFile,
    graph: &ArtboardGraph,
    objects: &mut InstanceObjectArena,
) {
    let solo_handles = graph
        .components
        .iter()
        .filter(|component| component.type_name == "Solo")
        .filter_map(|component| objects.component_handle(component.local_id))
        .collect::<Vec<_>>();
    if solo_handles.is_empty() {
        return;
    }

    let runtime_local_by_cpp_local = artboard_index_for_graph(file, graph)
        .map(|artboard_index| runtime_local_by_cpp_artboard_local(file, graph, artboard_index))
        .unwrap_or_default();
    let cpp_local_by_runtime_local = runtime_local_by_cpp_local
        .into_iter()
        .map(|(cpp_local, runtime_local)| (runtime_local, cpp_local))
        .collect::<BTreeMap<_, _>>();

    for solo_handle in solo_handles {
        let cpp_local_ids = objects
            .component(solo_handle)
            .map(|solo| solo.children.clone())
            .unwrap_or_default()
            .into_iter()
            .filter_map(|child| {
                let child_component = objects.component(child)?;
                cpp_local_by_runtime_local
                    .get(&child_component.local_id)
                    .copied()
            })
            .collect();
        if let Some(solo) = objects
            .component_mut(solo_handle)
            .and_then(|component| component.concrete.solo.as_mut())
        {
            solo.cpp_local_ids = cpp_local_ids;
        }
    }
}

fn runtime_local_by_cpp_artboard_local(
    file: &RuntimeFile,
    graph: &ArtboardGraph,
    artboard_index: usize,
) -> BTreeMap<usize, usize> {
    #[cfg(test)]
    record_solo_mapping_analysis();

    let runtime_local_by_global = graph
        .local_objects
        .iter()
        .map(|local_object| (local_object.global_id, local_object.local_id))
        .collect::<BTreeMap<_, _>>();
    let slots = file
        .artboard_local_object_slots(artboard_index)
        .unwrap_or_default();

    #[cfg(test)]
    record_solo_mapping_batch_query(slots.len());

    slots
        .into_iter()
        .enumerate()
        .filter_map(|(cpp_local, object)| {
            object.and_then(|object| {
                runtime_local_by_global
                    .get(&object.id)
                    .copied()
                    .map(|runtime_local| (cpp_local, runtime_local))
            })
        })
        .collect()
}

impl ArtboardInstance {
    /// C++ `Solo::getActiveChildIndex`, adapted to `Option` so the existing
    /// Rust data-bind path keeps its behavior for unresolved authored ids.
    pub(crate) fn solo_active_child_index(&self, solo_local_id: usize) -> Option<usize> {
        let solo = self.component_handle(solo_local_id)?;
        let component = self.objects.component(solo)?;
        let solo_state = component.concrete.solo.as_ref()?;
        let active_component_id = usize::try_from(
            self.uint_property(solo_local_id, solo_state.active_component_property_key?)?,
        )
        .ok()?;
        solo_state
            .cpp_local_ids
            .iter()
            .position(|cpp_local_id| *cpp_local_id == active_component_id)
    }

    /// C++ `Solo::getActiveChildName`, borrowing the retained child name from
    /// this Artboard occurrence instead of rediscovering authored ownership.
    pub(crate) fn solo_active_child_name(&self, solo_local_id: usize) -> Option<&str> {
        let active_child_index = self.solo_active_child_index(solo_local_id)?;
        let solo = self.component_handle(solo_local_id)?;
        let component = self.objects.component(solo)?;
        let active_local_id = self
            .objects
            .component_local_id(*component.children.get(active_child_index)?)?;
        self.slot(active_local_id)?.name.as_deref()
    }

    pub(crate) fn apply_component_collapse_changed(&mut self, local_id: usize) -> bool {
        let Some(solo) = self.component_handle(local_id) else {
            return false;
        };
        self.propagate_solo_collapse(solo)
    }

    pub(crate) fn set_solo_active_child_by_index(
        &mut self,
        solo_local_id: usize,
        value: f32,
    ) -> bool {
        let rounded = value.round();
        if rounded < 0.0 || !rounded.is_finite() {
            return false;
        }
        let Some(solo) = self.component_handle(solo_local_id) else {
            return false;
        };
        let child_index = rounded as usize;
        let is_child = self
            .objects
            .component(solo)
            .and_then(|component| component.concrete.solo.as_ref().map(|_| component))
            .is_some_and(|component| child_index < component.children.len());
        if !is_child {
            return false;
        }
        self.set_solo_active_child(solo, child_index)
    }

    pub(crate) fn set_solo_active_child_by_name(
        &mut self,
        solo_local_id: usize,
        value: &[u8],
    ) -> bool {
        let Some(solo) = self.component_handle(solo_local_id) else {
            return false;
        };
        let child_count = self
            .objects
            .component(solo)
            .and_then(|component| {
                component
                    .concrete
                    .solo
                    .as_ref()
                    .map(|_| component.children.len())
            })
            .unwrap_or(0);
        for child_index in 0..child_count {
            let child_local_id = self
                .objects
                .component(solo)
                .and_then(|component| component.children.get(child_index).copied())
                .and_then(|child| self.objects.component_local_id(child));
            if child_local_id
                .and_then(|local_id| self.slot(local_id))
                .and_then(|slot| slot.name.as_deref())
                .is_some_and(|name| name.as_bytes() == value)
            {
                return self.set_solo_active_child(solo, child_index);
            }
        }
        false
    }

    pub(crate) fn set_solo_active_child(
        &mut self,
        solo: ComponentHandle,
        child_index: usize,
    ) -> bool {
        let Some((solo_local_id, active_component_property_key, cpp_local_id)) =
            self.objects.component(solo).and_then(|component| {
                let solo = component.concrete.solo.as_ref()?;
                Some((
                    component.local_id,
                    solo.active_component_property_key?,
                    *solo.cpp_local_ids.get(child_index)?,
                ))
            })
        else {
            return false;
        };
        let Ok(cpp_local_id) = u64::try_from(cpp_local_id) else {
            return false;
        };
        // C++ `Solo::updateByIndex`/`updateByName` writes the generated
        // Artboard object-table id; `activeComponentIdChanged` then invokes
        // `propagateCollapse` on this same occurrence (`src/solo.cpp:42-81`).
        self.set_uint_property(solo_local_id, active_component_property_key, cpp_local_id)
    }

    pub(crate) fn propagate_solo_collapse(&mut self, solo: ComponentHandle) -> bool {
        let Some((solo_local_id, solo_collapsed, active_component_property_key, child_count)) =
            self.objects.component(solo).and_then(|component| {
                let state = component.concrete.solo.as_ref()?;
                Some((
                    component.local_id,
                    component.is_collapsed(),
                    state.active_component_property_key?,
                    component.children.len().min(state.cpp_local_ids.len()),
                ))
            })
        else {
            return false;
        };

        let active_cpp_local = self
            .uint_property(solo_local_id, active_component_property_key)
            .and_then(|id| usize::try_from(id).ok());

        let mut changed = false;
        for child_index in 0..child_count {
            let Some((child, cpp_local_id, participates)) =
                self.objects.component(solo).and_then(|component| {
                    let child = *component.children.get(child_index)?;
                    let cpp_local_id = *component
                        .concrete
                        .solo
                        .as_ref()?
                        .cpp_local_ids
                        .get(child_index)?;
                    let child_type = self.objects.component(child)?.type_name;
                    let participates = definition_by_name(child_type).is_none_or(|definition| {
                        !definition.is_a("Constraint") && !definition.is_a("ClippingShape")
                    });
                    Some((child, cpp_local_id, participates))
                })
            else {
                continue;
            };
            let collapsed = if participates {
                solo_collapsed || Some(cpp_local_id) != active_cpp_local
            } else {
                solo_collapsed
            };
            let Some(child_local_id) = self.objects.component_local_id(child) else {
                continue;
            };
            changed |= self.collapse_component_tree(child_local_id, collapsed);
        }
        changed
    }
}
