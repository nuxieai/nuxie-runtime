#[derive(Debug, Clone, Copy)]
struct StaticTextVariationModifier {
    local_id: usize,
    global_id: u32,
    authored_tag: u32,
    authored_value: f32,
}

/// Pinned `TextVariationModifier::modify` computes the axis-value product,
/// then contracts the previous-value product into that add on the supported
/// clang/AArch64 build.
fn text_variation_modifier_interpolate(from_value: f32, axis_value: f32, strength: f32) -> f32 {
    from_value.mul_add(1.0 - strength, axis_value * strength)
}

impl StaticTextVariationModifier {
    fn from_graph(runtime: &RuntimeFile, graph: &ArtboardGraph, local_id: usize) -> Result<Self> {
        let global_id = global_for_local(graph, local_id)?;
        let object = runtime
            .object(global_id as usize)
            .with_context(|| format!("missing TextVariationModifier global {global_id}"))?;
        Ok(Self {
            local_id,
            global_id,
            authored_tag: object.uint_property("axisTag").unwrap_or(0) as u32,
            authored_value: object.double_property("axisValue").unwrap_or(0.0),
        })
    }

    fn tag(self, instance: &ArtboardInstance) -> u32 {
        instance.text_variation_modifier_tag(self.local_id, self.authored_tag)
    }

    fn value(self, instance: &ArtboardInstance) -> f32 {
        property_key_for_name("TextVariationModifier", "axisValue")
            .and_then(|key| instance.double_property(self.local_id, key))
            .unwrap_or(self.authored_value)
    }

    fn modify(
        self,
        instance: &ArtboardInstance,
        font: &SkrifaFontRef<'_>,
        inherited: &BTreeMap<u32, f32>,
        variations: &mut BTreeMap<u32, f32>,
        font_size: f32,
        strength: f32,
    ) -> f32 {
        let tag = self.tag(instance);
        let from = variations
            .get(&tag)
            .or_else(|| inherited.get(&tag))
            .copied()
            .unwrap_or_else(|| {
                font.axes()
                    .iter()
                    .find(|axis| u32::from_be_bytes(axis.tag().to_be_bytes()) == tag)
                    .map(|axis| axis.default_value())
                    .unwrap_or(0.0)
            });
        variations.insert(
            tag,
            text_variation_modifier_interpolate(from, self.value(instance), strength),
        );
        font_size
    }
}

pub(crate) fn text_variation_modifier_double_property_changed(
    instance: &mut ArtboardInstance,
    local_id: usize,
    type_name: Option<&str>,
    property_key: u16,
) -> Option<bool> {
    if type_name != Some("TextVariationModifier")
        || property_key_for_name("TextVariationModifier", "axisValue") != Some(property_key)
    {
        return None;
    }
    let text = instance
        .component_parent_local(local_id)
        .filter(|group| {
            instance
                .component(*group)
                .and_then(|group| nuxie_schema::definition_by_name(group.type_name))
                .is_some_and(|definition| definition.is_a("TextModifierGroup"))
        })
        .and_then(|group| modifier_group_text(instance, group));
    Some(text.is_some_and(|text| crate::text_owner::mark_shape_dirty(instance, text)))
}
