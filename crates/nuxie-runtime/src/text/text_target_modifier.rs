#[derive(Debug, Clone, Copy)]
struct StaticTextTargetModifier {
    local_id: usize,
    global_id: u32,
}

impl StaticTextTargetModifier {
    fn from_graph(runtime: &RuntimeFile, graph: &ArtboardGraph, local_id: usize) -> Result<Self> {
        let (global_id, _) = text_target_modifier_resolution(runtime, graph, local_id)?;
        Ok(Self {
            local_id,
            global_id,
        })
    }
}

/// Validate the pinned `TextModifier::onAddedDirty` prerequisite before the
/// target modifier can participate in a static Text slice. The actual target
/// pointer counterpart is occurrence-owned and resolved during construction.
fn text_target_modifier_resolution(
    runtime: &RuntimeFile,
    graph: &ArtboardGraph,
    local_id: usize,
) -> Result<(u32, usize)> {
    let group_local = component_for_local(graph, local_id)
        .and_then(|component| component.parent_local)
        .filter(|parent| {
            type_for_local(graph, *parent)
                .and_then(nuxie_schema::definition_by_name)
                .is_some_and(|definition| definition.is_a("TextModifierGroup"))
        })
        .with_context(|| {
            format!(
                "TextTargetModifier local {local_id} requires a direct TextModifierGroup parent"
            )
        })?;
    let global_id = global_for_local(graph, local_id)?;
    runtime
        .object(global_id as usize)
        .with_context(|| format!("missing TextTargetModifier global {global_id}"))?;
    Ok((global_id, group_local))
}

fn text_target_modifier_text_component(
    instance: &ArtboardInstance,
    modifier_local: usize,
) -> Option<usize> {
    let group_local = instance.component_parent_local(modifier_local)?;
    let is_group = instance
        .component(group_local)
        .and_then(|group| nuxie_schema::definition_by_name(group.type_name))
        .is_some_and(|definition| definition.is_a("TextModifierGroup"));
    if !is_group {
        return None;
    }
    modifier_group_text(instance, group_local)
}

fn text_target_modifier_target_id(instance: &ArtboardInstance, modifier_local: usize) -> u32 {
    property_key_for_name("TextTargetModifier", "targetId")
        .and_then(|key| instance.uint_property(modifier_local, key))
        .and_then(|value| u32::try_from(value).ok())
        .unwrap_or(u32::MAX)
}

fn text_target_modifier_target_local(
    instance: &ArtboardInstance,
    modifier_local: usize,
) -> Option<usize> {
    instance
        .component(modifier_local)?
        .concrete
        .text_target
        .as_ref()?
        .target_local()
}
