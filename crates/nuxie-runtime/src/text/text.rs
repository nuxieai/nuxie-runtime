use crate::{
    artboard::ArtboardInstance,
    components::ComponentDirt,
    layout_node_provider,
    properties::property_key_for_name,
    view_model::{RuntimeCoreObjectListener, RuntimeOwnedViewModelHandle},
};

#[derive(Debug)]
pub(crate) struct RuntimeTextValueRunListener {
    instance: RuntimeOwnedViewModelHandle,
    core: RuntimeCoreObjectListener,
}

impl RuntimeTextValueRunListener {
    pub(crate) fn new(instance: RuntimeOwnedViewModelHandle) -> Self {
        let mut listener = Self {
            instance,
            core: RuntimeCoreObjectListener::default(),
        };
        listener.create_properties();
        listener
    }

    pub(crate) fn remap(&mut self, instance: RuntimeOwnedViewModelHandle) -> bool {
        if self.instance.ptr_eq(&instance) {
            return false;
        }

        // Delete the old property dependents while `self.instance` still owns
        // their source values. Replacing the instance first is the upstream
        // use-after-free ordering this port guards against.
        self.core.delete_properties();
        self.instance = instance;
        self.create_properties();
        true
    }

    fn create_properties(&mut self) {
        let properties = {
            let instance = self.instance.borrow();
            ["textStyle", "textContent"]
                .into_iter()
                .filter_map(|name| instance.string_cell_by_property_name(name))
                .collect::<Vec<_>>()
        };
        // TextValueRunListener::createProperties invokes the inherited base
        // cleanup before installing its text-style and text-content listeners.
        self.core.create_properties(properties);
    }

    pub(crate) fn take_changed(&self) -> bool {
        self.core.take_changed()
    }
}

pub(crate) fn mark_shape_dirty(instance: &mut ArtboardInstance, text_local_id: usize) -> bool {
    mark_shape_dirty_with_layout(instance, text_local_id, true)
}

pub(crate) fn mark_shape_dirty_without_layout(
    instance: &mut ArtboardInstance,
    text_local_id: usize,
) -> bool {
    mark_shape_dirty_with_layout(instance, text_local_id, false)
}

/// Direct pinned `Text::controlSize`: compare the complete retained state,
/// publish every field before callbacks can observe it, then shape-dirty
/// without feeding the new size back into layout.
pub(crate) fn control_size(
    instance: &mut ArtboardInstance,
    text_local_id: usize,
    width: f32,
    height: f32,
    width_scale_type: u64,
    height_scale_type: u64,
    layout_direction: u64,
) -> bool {
    let changed = instance
        .component(text_local_id)
        .and_then(|component| component.concrete.text.as_ref())
        .is_some_and(|text| {
            text.retain_control_size(
                width,
                height,
                width_scale_type,
                height_scale_type,
                layout_direction,
            )
        });
    if !changed {
        return false;
    }
    mark_shape_dirty_without_layout(instance, text_local_id);
    true
}

fn mark_shape_dirty_with_layout(
    instance: &mut ArtboardInstance,
    text_local_id: usize,
    send_to_layout: bool,
) -> bool {
    if !matches!(
        instance
            .component(text_local_id)
            .map(|component| component.type_name),
        Some("Text" | "TextInput")
    ) {
        return false;
    }
    let modifier_group_locals = instance
        .component(text_local_id)
        .and_then(|component| component.concrete.text.as_ref())
        .map(|text| text.modifier_group_locals())
        .unwrap_or_default();
    let modifier_ranges = modifier_group_locals
        .into_iter()
        .map(|group_local| {
            let ranges = instance
                .component(group_local)
                .into_iter()
                .flat_map(|group| group.children.iter())
                .filter_map(|range| instance.component_local_id(*range))
                .filter(|range_local| {
                    instance
                        .component(*range_local)
                        .is_some_and(|range| range.type_name == "TextModifierRange")
                })
                .collect::<Vec<_>>();
            (group_local, ranges)
        })
        .collect::<Vec<_>>();

    // Pinned `Text::markShapeDirty(bool)` publishes Path first, then clears
    // every modifier range map and publishes TextCoverage for each group in
    // authored child order.
    #[cfg(test)]
    if let Some(text) = instance
        .component(text_local_id)
        .and_then(|component| component.concrete.text.as_ref())
    {
        text.begin_modifier_range_map_clear_trace();
    }
    let mut changed = instance.add_dirt(text_local_id, ComponentDirt::PATH, false);
    for (group_local, ranges) in modifier_ranges {
        if let Some(text) = instance
            .component(text_local_id)
            .and_then(|component| component.concrete.text.as_ref())
        {
            for range_local in ranges {
                text.clear_modifier_range_map(range_local);
                #[cfg(test)]
                text.record_modifier_range_clear(range_local);
            }
        }
        changed |= instance.add_dirt(group_local, ComponentDirt::TEXT_COVERAGE, false);
        #[cfg(test)]
        if let Some(text) = instance
            .component(text_local_id)
            .and_then(|component| component.concrete.text.as_ref())
        {
            text.record_modifier_range_group_clear(group_local);
        }
    }
    #[cfg(test)]
    if let Some(text) = instance
        .component(text_local_id)
        .and_then(|component| component.concrete.text.as_ref())
    {
        text.end_modifier_range_map_clear_trace();
    }

    instance.mark_text_shape_changed();
    if let Some(text) = instance
        .component(text_local_id)
        .and_then(|component| component.concrete.text.as_ref())
    {
        text.invalidate_bounds();
    }
    changed |= instance.add_dirt(text_local_id, ComponentDirt::WORLD_TRANSFORM, true);
    if send_to_layout {
        changed |= layout_node_provider::mark_layout_node_dirty(instance, text_local_id);
    }
    changed
}

/// Direct pinned `Text::modifierShapeDirty`: Path only. Range-map clearing,
/// WorldTransform dirt, and layout publication belong to `markShapeDirty`.
pub(crate) fn modifier_shape_dirty(instance: &mut ArtboardInstance, text_local_id: usize) -> bool {
    if !matches!(
        instance
            .component(text_local_id)
            .map(|component| component.type_name),
        Some("Text" | "TextInput")
    ) {
        return false;
    }
    instance.add_dirt(text_local_id, ComponentDirt::PATH, false)
}

fn effective_sizing(instance: &ArtboardInstance, text_local_id: usize) -> u64 {
    let authored = property_key_for_name("Text", "sizingValue")
        .and_then(|key| instance.uint_property(text_local_id, key))
        .unwrap_or(0);
    instance
        .component(text_local_id)
        .and_then(|component| component.concrete.text.as_ref())
        .map_or(authored, |text| text.effective_sizing(authored))
}

fn mark_paint_dirty(instance: &mut ArtboardInstance, text_local_id: usize) -> bool {
    instance.add_dirt(text_local_id, ComponentDirt::PAINT, false)
}

fn mark_origin_dirty(instance: &mut ArtboardInstance, text_local_id: usize) -> bool {
    mark_paint_dirty(instance, text_local_id)
        | instance.add_dirt(text_local_id, ComponentDirt::WORLD_TRANSFORM, true)
}

pub(crate) fn double_property_changed(
    instance: &mut ArtboardInstance,
    local_id: usize,
    type_name: Option<&str>,
    property_key: u16,
) -> Option<bool> {
    if type_name != Some("Text") {
        return None;
    }
    if property_key_for_name("Text", "width") == Some(property_key) {
        return Some(if effective_sizing(instance, local_id) != 0 {
            mark_shape_dirty(instance, local_id)
        } else {
            false
        });
    }
    if property_key_for_name("Text", "height") == Some(property_key) {
        return Some(if effective_sizing(instance, local_id) == 2 {
            mark_shape_dirty(instance, local_id)
        } else {
            false
        });
    }
    if property_key_for_name("Text", "paragraphSpacing") == Some(property_key) {
        return Some(mark_paint_dirty(instance, local_id));
    }
    ["originX", "originY"]
        .into_iter()
        .any(|name| property_key_for_name("Text", name) == Some(property_key))
        .then(|| mark_origin_dirty(instance, local_id))
}

pub(crate) fn uint_property_changed(
    instance: &mut ArtboardInstance,
    local_id: usize,
    type_name: Option<&str>,
    property_key: u16,
) -> Option<bool> {
    if type_name != Some("Text") {
        return None;
    }
    if property_key_for_name("Text", "overflowValue") == Some(property_key) {
        return Some(if effective_sizing(instance, local_id) != 0 {
            mark_shape_dirty(instance, local_id)
        } else {
            false
        });
    }
    if [
        "alignValue",
        "sizingValue",
        "verticalTrimValue",
        "verticalTrimTopValue",
        "verticalTrimBottomValue",
    ]
    .into_iter()
    .any(|name| property_key_for_name("Text", name) == Some(property_key))
    {
        return Some(mark_shape_dirty(instance, local_id));
    }
    (property_key_for_name("Text", "originValue") == Some(property_key))
        .then(|| mark_origin_dirty(instance, local_id))
}
