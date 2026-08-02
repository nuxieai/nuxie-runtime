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
    fn new() -> Self {
        Self {
            active_component_property_key: property_key_for_name("Solo", "activeComponentId"),
            cpp_local_ids: Vec::new(),
        }
    }

    fn clone_for_occurrence(&self) -> Self {
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

impl crate::artboard::ArtboardInstance {
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
