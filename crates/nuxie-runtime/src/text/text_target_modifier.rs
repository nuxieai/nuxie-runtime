#[derive(Debug, Clone, Copy)]
struct StaticTextTargetModifier {
    local_id: usize,
    global_id: u32,
    target_id: u32,
    resolved_transform_local: Option<usize>,
    group_local: usize,
}

impl StaticTextTargetModifier {
    fn from_graph(
        runtime: &RuntimeFile,
        graph: &ArtboardGraph,
        local_id: usize,
        group_local: usize,
    ) -> Result<Self> {
        let global_id = global_for_local(graph, local_id)?;
        let object = runtime
            .object(global_id as usize)
            .with_context(|| format!("missing TextTargetModifier global {global_id}"))?;
        let target_id = object.uint_property("targetId").unwrap_or(u32::MAX as u64) as u32;
        let resolved_transform_local = usize::try_from(target_id).ok().filter(|target| {
            component_for_local(graph, *target).is_some_and(|component| {
                nuxie_schema::definition_by_name(component.type_name)
                    .is_some_and(|definition| definition.is_a("TransformComponent"))
            })
        });
        Ok(Self {
            local_id,
            global_id,
            target_id,
            resolved_transform_local,
            group_local,
        })
    }
}
