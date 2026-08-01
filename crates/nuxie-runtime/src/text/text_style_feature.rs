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
        let global_id = global_for_local(graph, local_id)?;
        let object = runtime
            .object(global_id as usize)
            .with_context(|| format!("missing TextStyleFeature global {global_id}"))?;
        if type_for_local(graph, local_id) != Some("TextStyleFeature") {
            bail!("TextStyleFeature local {local_id} has the wrong runtime type");
        }
        let parent = component_for_local(graph, local_id).and_then(|value| value.parent_local);
        if parent.is_none_or(|parent| {
            !matches!(type_for_local(graph, parent), Some("TextStyle" | "TextStylePaint"))
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
        instance.text_style_feature_option(
            self.local_id,
            self.authored_tag,
            self.authored_value,
        )
    }

    fn harf_feature(self, instance: &ArtboardInstance) -> Feature {
        let (tag, value) = self.option(instance);
        Feature::new(HarfTag::from_u32(tag), value, ..)
    }
}
