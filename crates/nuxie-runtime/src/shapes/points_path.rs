//! PointsPath owns authored vertices and optional Skin deformation. Its
//! generated property callbacks are inherited by Path/ParametricPath; vertex
//! callbacks route here through `path_vertex`.

use crate::components::Mat2D;

pub(crate) fn path_transform(has_skin: bool, world_transform: Mat2D) -> Mat2D {
    if has_skin {
        Mat2D::IDENTITY
    } else {
        world_transform
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeSet, path::PathBuf};

    use nuxie_binary::read_runtime_file;
    use nuxie_graph::GraphFile;

    use crate::{ArtboardInstance, ComponentDirt};

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum DirtCall {
        Skin(usize),
        Path(usize),
    }

    /// Test-only invocation of the concrete pinned owner. Keeping it in the
    /// PointsPath module makes the inheritance order explicit without adding
    /// a new public runtime operation: PointsPath dirties its retained Skin,
    /// then delegates to Path::markPathDirty.
    fn mark_path_dirty(artboard: &mut ArtboardInstance, path_local: usize) -> Vec<DirtCall> {
        let mut calls = Vec::with_capacity(2);
        if let Some(skin_local) = artboard.points_path_skin_local_for_test(path_local) {
            calls.push(DirtCall::Skin(skin_local));
            assert!(artboard.mark_points_path_skin_dirty(path_local));
        }
        calls.push(DirtCall::Path(path_local));
        super::super::mark_path_dirty(artboard, path_local);
        calls
    }

    #[test]
    fn upstream_bad_skin_without_parent_skinnable_does_not_crash() {
        let fixture = PathBuf::from(
            std::env::var_os("RIVE_RUNTIME_DIR")
                .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into()),
        )
        .join("tests/unit_tests/assets/bad_skin.riv");
        let runtime = read_runtime_file(&std::fs::read(fixture).expect("read bad_skin.riv"))
            .expect("import bad_skin.riv");
        let graphs = GraphFile::from_runtime_file(&runtime).expect("graph bad_skin.riv");
        let graph = &graphs.artboards[0];
        assert_eq!(graph.name.as_deref(), Some("Illustration WOman.svg"));
        let mut artboard =
            ArtboardInstance::from_graph_with_artboards(&runtime, graph, &graphs.artboards)
                .expect("instantiate bad_skin.riv");

        artboard.update_components();
        let point_paths = graph
            .components
            .iter()
            .filter(|component| component.type_name == "PointsPath")
            .map(|component| component.local_id)
            .collect::<Vec<_>>();
        assert_eq!(point_paths.len(), 77);

        let skins = graph
            .components
            .iter()
            .filter(|component| component.type_name == "Skin")
            .map(|component| component.local_id)
            .collect::<Vec<_>>();
        assert_eq!(skins.len(), 8);
        let attached_skin_owners = skins
            .iter()
            .filter_map(|&skin_local| {
                artboard
                    .skin_skinnable_local_for_test(skin_local)
                    .map(|path_local| (skin_local, path_local))
            })
            .collect::<Vec<_>>();
        assert_eq!(attached_skin_owners.len(), 7);
        assert_eq!(
            skins
                .iter()
                .filter(|&&skin_local| artboard.skin_skinnable_local_for_test(skin_local).is_none())
                .count(),
            1,
            "the malformed Skin remains an orphan instead of crashing instantiation",
        );

        let mut dirt_calls = Vec::new();
        for &path_local in &point_paths {
            let expected_skin = artboard.points_path_skin_local_for_test(path_local);
            let calls = mark_path_dirty(&mut artboard, path_local);
            match expected_skin {
                Some(skin_local) => assert_eq!(
                    calls,
                    [DirtCall::Skin(skin_local), DirtCall::Path(path_local)],
                    "PointsPath local {path_local} must dirty Skin before Path",
                ),
                None => assert_eq!(calls, [DirtCall::Path(path_local)]),
            }
            dirt_calls.extend(calls);
            assert!(
                artboard
                    .debug_component_dirt(path_local)
                    .is_some_and(|dirt| dirt.contains(ComponentDirt::PATH)),
                "PointsPath local {path_local} carries Path dirt",
            );
        }

        let attached_skins_from_paths = point_paths
            .iter()
            .filter_map(|&path_local| artboard.points_path_skin_local_for_test(path_local))
            .collect::<BTreeSet<_>>();
        assert_eq!(attached_skins_from_paths.len(), 7);
        assert_eq!(
            attached_skins_from_paths,
            attached_skin_owners
                .iter()
                .map(|&(skin_local, _)| skin_local)
                .collect(),
        );
        assert_eq!(
            dirt_calls
                .iter()
                .filter(|call| matches!(call, DirtCall::Skin(_)))
                .count(),
            7,
        );
        assert_eq!(
            dirt_calls
                .iter()
                .filter(|call| matches!(call, DirtCall::Path(_)))
                .count(),
            77,
        );
        artboard.update_components();
    }
}
