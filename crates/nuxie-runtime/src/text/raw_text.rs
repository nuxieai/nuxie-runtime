/// Return the exact settled text rendered by one Text object.
///
/// This follows the same resolved-run path as shaping, including live string
/// property writes and dynamically projected list runs. Callers opt into this
/// allocation only for semantic text observation; ordinary geometry queries
/// do not materialize text values.
pub(crate) fn static_text_value(
    runtime: &RuntimeFile,
    graph: &ArtboardGraph,
    instance: &ArtboardInstance,
    text_local: usize,
) -> Option<String> {
    let slice = StaticTextSlice::from_graph(runtime, graph, text_local).ok()?;
    let runs = slice.resolved_runs(runtime, instance).ok()?;
    Some(runs.into_iter().map(|run| run.text).collect())
}
