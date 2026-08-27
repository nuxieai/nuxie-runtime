/// Mechanical owner for pinned `TextStyleFeature::onAddedDirty` after the
/// Component Super has linked the retained direct parent. A non-`Ok` Super
/// never enters this function; the caller propagates that failure unchanged.
/// The successful branch validates the direct is-a `TextStyle` parent before
/// appending to its occurrence-owned feature list in authored traversal order.
pub(crate) fn text_style_feature_on_added_dirty_after_super(
    local_id: usize,
    parent_type: &str,
    add_feature: impl FnOnce(usize),
) -> Result<()> {
    if !definition_by_name(parent_type)
        .is_some_and(|definition| definition.is_a("TextStyle"))
    {
        bail!("TextStyleFeature local {local_id} requires a direct TextStyle parent");
    }
    add_feature(local_id);
    Ok(())
}

/// Retained authored option snapshot for a type-164 TextStyleFeature.
/// Live key 356/357 writes intentionally do not mutate this snapshot or dirty
/// text; C++'s generated callbacks are empty.
#[derive(Debug, Clone, Copy)]
struct StaticTextStyleFeature {
    local_id: usize,
    global_id: u32,
    authored_tag: u32,
    authored_value: u32,
}

impl StaticTextStyleFeature {
    fn from_graph(runtime: &RuntimeFile, graph: &ArtboardGraph, local_id: usize) -> Result<Self> {
        Self::from_graph_with_occurrence(runtime, graph, None, local_id)
    }

    fn from_graph_with_occurrence(
        runtime: &RuntimeFile,
        graph: &ArtboardGraph,
        instance: Option<&ArtboardInstance>,
        local_id: usize,
    ) -> Result<Self> {
        let global_id = global_for_local(graph, local_id)?;
        let object = runtime
            .object(global_id as usize)
            .with_context(|| format!("missing TextStyleFeature global {global_id}"))?;
        if type_for_local(graph, local_id) != Some("TextStyleFeature") {
            bail!("TextStyleFeature local {local_id} has the wrong runtime type");
        }
        let parent_type = match instance {
            Some(instance) => instance
                .component_parent_local(local_id)
                .and_then(|parent| instance.component(parent))
                .map(|parent| parent.type_name),
            None => component_for_local(graph, local_id)
                .and_then(|value| value.parent_local)
                .and_then(|parent| type_for_local(graph, parent)),
        };
        if !parent_type.is_some_and(|type_name| {
            definition_by_name(type_name).is_some_and(|definition| definition.is_a("TextStyle"))
        }) {
            bail!("TextStyleFeature local {local_id} requires a direct TextStyle parent");
        }
        Ok(Self {
            local_id,
            global_id,
            authored_tag: object.uint_property("tag").unwrap_or(0) as u32,
            authored_value: object.uint_property("featureValue").unwrap_or(1) as u32,
        })
    }

    fn option(self, instance: &ArtboardInstance) -> (u32, u32) {
        instance.text_style_feature_option(self.local_id, self.authored_tag, self.authored_value)
    }

    fn live_option(self, instance: &ArtboardInstance) -> (u32, u32) {
        let tag = property_key_for_name("TextStyleFeature", "tag")
            .and_then(|key| instance.uint_property(self.local_id, key))
            .and_then(|value| u32::try_from(value).ok())
            .unwrap_or(self.authored_tag);
        let value = property_key_for_name("TextStyleFeature", "featureValue")
            .and_then(|key| instance.uint_property(self.local_id, key))
            .and_then(|value| u32::try_from(value).ok())
            .unwrap_or(self.authored_value);
        (tag, value)
    }

    fn harf_feature(self, instance: &ArtboardInstance) -> Feature {
        let (tag, value) = self.option(instance);
        Feature::new(HarfTag::from_u32(tag), value, ..)
    }
}

/// Generated `TextStyleFeatureBase::{tag,featureValue}` stores first, invokes
/// an empty callback, then notifies observers. Return `Some(false)` for those
/// two inherited keys so the common setter retains notification order while
/// suppressing the invented generic prepared/Text invalidation tail.
pub(crate) fn text_style_feature_uint_property_changed(
    _instance: &mut ArtboardInstance,
    _local_id: usize,
    type_name: Option<&str>,
    property_key: u16,
) -> Option<bool> {
    if !type_name.is_some_and(|type_name| {
        definition_by_name(type_name).is_some_and(|definition| definition.is_a("TextStyleFeature"))
    }) {
        return None;
    }
    ["tag", "featureValue"]
        .into_iter()
        .any(|name| property_key_for_name("TextStyleFeature", name) == Some(property_key))
        .then_some(false)
}
