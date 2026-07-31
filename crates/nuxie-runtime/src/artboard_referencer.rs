// Direct owner for pinned C++ `src/artboard_referencer.cpp`.

fn resolved_artboard_graph_for_referencer<'a>(
    file: &RuntimeFile,
    artboards: &'a [ArtboardGraph],
    referencer: &nuxie_binary::RuntimeObject,
) -> Option<&'a ArtboardGraph> {
    let referenced = file.resolved_artboard_for_referencer_object(referencer)?;
    artboards
        .iter()
        .find(|artboard| artboard.global_id == referenced.id)
}

fn referencer_data_bind_path(
    file: &RuntimeFile,
    referencer: &nuxie_binary::RuntimeObject,
) -> (Option<Vec<u32>>, bool) {
    let path = file.data_bind_path_for_referencer_object(referencer);
    let is_relative = path
        .as_ref()
        .and_then(|path| path.object)
        .and_then(|path| path.bool_property("isRelative"))
        .unwrap_or(false);
    let ids = path.map(|path| {
        if is_relative {
            path.path_ids
        } else {
            path.resolved_path_ids
        }
    });
    (ids, is_relative)
}
