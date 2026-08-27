// Direct owner for pinned C++ `src/artboard_referencer.cpp`.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RuntimeArtboardReferencerKind {
    NestedArtboard,
    ScriptInputArtboard,
}

/// Mechanical `ArtboardReferencer::from(Core*)` dispatch. All three generated
/// nested-artboard types share the handwritten `NestedArtboard` referencer.
pub(crate) fn artboard_referencer_from_type(
    type_name: &str,
) -> Option<RuntimeArtboardReferencerKind> {
    match type_name {
        "NestedArtboard" | "NestedArtboardLeaf" | "NestedArtboardLayout" => {
            Some(RuntimeArtboardReferencerKind::NestedArtboard)
        }
        "ScriptInputArtboard" => Some(RuntimeArtboardReferencerKind::ScriptInputArtboard),
        _ => None,
    }
}

pub(crate) struct RuntimeViewModelInstanceArtboardReference<'a> {
    pub(crate) asset: Option<&'a crate::RuntimeBindableArtboard>,
    pub(crate) property_value: u64,
}

pub(crate) enum RuntimeResolvedArtboardReference {
    Live {
        source: crate::RuntimeBindableArtboard,
    },
    File {
        artboard_id: u64,
    },
}

/// Mechanical `ArtboardReferencer::findArtboard` translation. The retained
/// live asset has priority over the generated file index and never falls back
/// to that index when its artboard is absent. Self/ancestor references are
/// rejected in both branches.
pub(crate) fn find_artboard(
    view_model_instance_artboard: Option<RuntimeViewModelInstanceArtboardReference<'_>>,
    parent_artboard_ancestors: Option<&RuntimeArtboardAncestorSources>,
    file: Option<&RuntimeFile>,
) -> Option<RuntimeResolvedArtboardReference> {
    let view_model_instance_artboard = view_model_instance_artboard?;
    if let Some(asset) = view_model_instance_artboard.asset {
        let artboard = asset.artboard_instance()?;
        if parent_artboard_ancestors.is_some_and(|ancestors| ancestors.rejects(&artboard)) {
            return None;
        }
        return Some(RuntimeResolvedArtboardReference::Live {
            source: asset.clone(),
        });
    }
    if let Some(file) = file {
        let artboard_id = view_model_instance_artboard.property_value;
        let artboard = usize::try_from(artboard_id)
            .ok()
            .and_then(|index| file.artboard(index))?;
        if parent_artboard_ancestors
            .is_some_and(|ancestors| ancestors.rejects_file_artboard_global(artboard.id))
        {
            return None;
        }
        return Some(RuntimeResolvedArtboardReference::File { artboard_id });
    }
    None
}

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
