use super::*;
use nuxie::live_move_fixture as fixture;
fn view(s: &str) -> NuxStringView {
    NuxStringView {
        data: s.as_ptr().cast(),
        len: s.len(),
    }
}
unsafe fn import_fixture(bytes: &[u8], file: &mut *mut NuxFile) -> NuxStatus {
    let caps = NuxVideoPlaybackCapabilities {
        playback_available: 1,
        ..Default::default()
    };
    let mut result = ptr::null_mut();
    let status = unsafe {
        nux_file_import_with_video_capabilities(
            bytes.as_ptr(),
            bytes.len(),
            &NuxRenderCallbacks::default(),
            &caps,
            file,
            &mut result,
        )
    };
    unsafe {
        nux_capi_result_free(result);
    }
    status
}
#[test]
fn live_moves_c_api_takeoff_land_and_home() {
    unsafe {
        let bytes = fixture::fixture(2);
        let mut file = ptr::null_mut();
        assert_eq!(import_fixture(&bytes, &mut file), NuxStatus::Ok);
        let mut artboard = ptr::null_mut();
        assert_eq!(
            nux_artboard_instance_new(file, 0, &mut artboard),
            NuxStatus::Ok
        );
        let mut vm = ptr::null_mut();
        assert_eq!(
            nux_view_model_instance_new_default(artboard, &mut vm),
            NuxStatus::Ok
        );
        assert_eq!(
            nux_artboard_instance_bind_view_model(artboard, vm),
            NuxStatus::Ok
        );
        let mut player = ptr::null_mut();
        assert_eq!(nux_player_new_default(artboard, &mut player), NuxStatus::Ok);
        let mut result = ptr::null_mut();
        assert_eq!(
            nux_player_step(player, &NuxPlayerStep::default(), &mut result),
            NuxStatus::Ok
        );
        nux_player_step_result_free(result);
        let root = (&(*player).artboard).instance.borrow().native_handle();
        let rows = nuxie::live_move::screens(&root).unwrap();
        let a = rows["A"].with_artboard(|b| b.nested_artboards())[0].clone();
        let b = rows["B"].with_artboard(|b| b.nested_artboards());
        let child = |h: &nuxie::CoreHandle| {
            h.with(|o| o.as_nested_artboard().unwrap().artboard_instance_default())
                .flatten()
        };
        let original = child(&a).unwrap();
        let keyed = original
            .with_artboard(|b| b.find_handle::<nuxie::runtime::node::Node>("keyed"))
            .unwrap();
        assert_eq!(
            nux_player_set_moves(player, view(fixture::MOVES)),
            NuxStatus::Ok
        );
        let transition = view("t1");
        let step = NuxPlayerStep {
            elapsed_seconds: 0.3,
            ..Default::default()
        };
        let mut result = ptr::null_mut();
        assert_eq!(nux_player_step(player, &step, &mut result), NuxStatus::Ok);
        nux_player_step_result_free(result);
        assert_eq!(
            nux_player_step_begin(player, view("B"), &transition),
            NuxStatus::Ok
        );
        assert!(child(&a).is_none());
        assert_eq!(child(&b[1]).unwrap().core_handle(), original.core_handle());
        let step = NuxPlayerStep {
            elapsed_seconds: 0.2,
            ..Default::default()
        };
        let mut result = ptr::null_mut();
        assert_eq!(nux_player_step(player, &step, &mut result), NuxStatus::Ok);
        nux_player_step_result_free(result);
        assert_eq!(
            nux_player_step_end(player, view("B"), &transition),
            NuxStatus::Ok
        );
        assert_eq!(child(&b[0]).unwrap().core_handle(), original.core_handle());
        let x =
            nuxie::runtime::generated::core_registry::CoreRegistry::get_double_handle(&keyed, 13)
                .unwrap();
        assert!((x - 50.0).abs() < 0.002, "x={x}");
        assert_eq!(
            nux_player_step_begin(player, view("A"), ptr::null()),
            NuxStatus::Ok
        );
        assert_eq!(child(&a).unwrap().core_handle(), original.core_handle());
        assert_eq!(
            nux_player_step_end(player, view("A"), ptr::null()),
            NuxStatus::Ok
        );
        assert_eq!(
            nux_player_set_row_collapsed(player, view("screens"), 1, 1),
            NuxStatus::Ok
        );
        assert_eq!(
            nux_player_set_row_collapsed(player, view("screens"), 1, 0),
            NuxStatus::Ok
        );
        assert_eq!(
            nux_player_set_moves(player, view("{}")),
            NuxStatus::InvalidArgument
        );
        nux_player_free(player);
        nux_view_model_instance_free(vm);
        nux_artboard_instance_free(artboard);
        nux_file_free(file);
    }
}

#[test]
fn live_moves_keep_the_c_api_video_id_reachable() {
    unsafe extern "C" fn collect(data: *mut c_void, info: *const NuxVideoInfo) {
        unsafe { &mut *data.cast::<Vec<usize>>() }.push(unsafe { &*info }.component_id);
    }
    unsafe fn ids(player: *mut NuxPlayer) -> Vec<usize> {
        let mut ids = Vec::new();
        assert_eq!(
            unsafe {
                nux_player_visit_videos(player, Some(collect), ptr::from_mut(&mut ids).cast())
            },
            NuxStatus::Ok
        );
        ids
    }
    unsafe {
        let bytes = fixture::fixture(2);
        let mut file = ptr::null_mut();
        let mut artboard = ptr::null_mut();
        let mut vm = ptr::null_mut();
        let mut player = ptr::null_mut();
        assert_eq!(import_fixture(&bytes, &mut file), NuxStatus::Ok);
        assert_eq!(
            nux_artboard_instance_new(file, 0, &mut artboard),
            NuxStatus::Ok
        );
        assert_eq!(
            nux_view_model_instance_new_default(artboard, &mut vm),
            NuxStatus::Ok
        );
        assert_eq!(
            nux_artboard_instance_bind_view_model(artboard, vm),
            NuxStatus::Ok
        );
        assert_eq!(nux_player_new_default(artboard, &mut player), NuxStatus::Ok);
        let mut result = ptr::null_mut();
        assert_eq!(
            nux_player_step(player, &NuxPlayerStep::default(), &mut result),
            NuxStatus::Ok
        );
        nux_player_step_result_free(result);
        let root = (&(*player).artboard).instance.borrow().native_handle();
        let rows = nuxie::live_move::screens(&root).unwrap();
        for (name, row) in rows {
            for (index, host) in row
                .with_artboard(|b| b.nested_artboards())
                .into_iter()
                .enumerate()
            {
                if name != "A" || index != 0 {
                    nuxie::live_move::empty_slot(&host).unwrap();
                }
            }
        }
        let original = ids(player);
        assert_eq!(original.len(), 1);
        assert_eq!(
            nux_player_set_moves(player, view(fixture::MOVES)),
            NuxStatus::Ok
        );
        let t = view("t1");
        assert_eq!(nux_player_step_begin(player, view("B"), &t), NuxStatus::Ok);
        assert_eq!(ids(player), original);
        assert_eq!(nux_player_step_end(player, view("B"), &t), NuxStatus::Ok);
        assert_eq!(ids(player), original);
        assert_eq!(
            nux_player_step_begin(player, view("A"), ptr::null()),
            NuxStatus::Ok
        );
        assert_eq!(ids(player), original);
        assert_eq!(
            nux_player_step_end(player, view("A"), ptr::null()),
            NuxStatus::Ok
        );
        println!("video id across S/F/D/S: {}", original[0]);
        nux_player_free(player);
        nux_view_model_instance_free(vm);
        nux_artboard_instance_free(artboard);
        nux_file_free(file);
    }
}

#[test]
fn live_moves_semantic_snapshot_keeps_one_field_and_focus() {
    use nuxie::runtime::{
        focus_data::FocusData, semantic::semantic_data::SemanticData, text::text_input::TextInput,
    };
    use std::collections::HashSet;
    unsafe {
        for focused in [false, true] {
            let bytes = fixture::semantic_fixture();
            let mut file = ptr::null_mut();
            assert_eq!(import_fixture(&bytes, &mut file), NuxStatus::Ok);
            let mut artboard = ptr::null_mut();
            assert_eq!(
                nux_artboard_instance_new(file, 0, &mut artboard),
                NuxStatus::Ok
            );
            let mut vm = ptr::null_mut();
            assert_eq!(
                nux_view_model_instance_new_default(artboard, &mut vm),
                NuxStatus::Ok
            );
            assert_eq!(
                nux_artboard_instance_bind_view_model(artboard, vm),
                NuxStatus::Ok
            );
            let mut player = ptr::null_mut();
            assert_eq!(nux_player_new_default(artboard, &mut player), NuxStatus::Ok);
            let mut result = ptr::null_mut();
            assert_eq!(
                nux_player_step(player, &NuxPlayerStep::default(), &mut result),
                NuxStatus::Ok
            );
            nux_player_step_result_free(result);
            let root = (&(*player).artboard).instance.borrow().native_handle();
            let rows = nuxie::live_move::screens(&root).unwrap();
            let a = rows["A"].with_artboard(|b| b.nested_artboards());
            let b = rows["B"].with_artboard(|b| b.nested_artboards());
            nuxie::live_move::empty_slot(&a[1]).unwrap();
            nuxie::live_move::empty_slot(&b[1]).unwrap();
            let unit = a[0]
                .with(|o| o.as_nested_artboard().unwrap().artboard_instance_default())
                .flatten()
                .unwrap();
            assert_eq!(
                nux_player_set_moves(player, view(fixture::MOVES)),
                NuxStatus::Ok
            );
            assert_eq!(nux_player_enable_semantics(player), NuxStatus::Ok);
            let semantic_data = unit.with_artboard(|b| b.find_all_handles::<SemanticData>());
            let ids: HashSet<_> = semantic_data
                .iter()
                .map(|h| {
                    h.with_downcast::<SemanticData, _>(|d| {
                        d.existing_semantic_node().unwrap().borrow().id()
                    })
                    .unwrap()
                })
                .collect();
            assert_eq!(ids.len(), 3);
            let group_id = semantic_data
                .iter()
                .find_map(|h| {
                    h.with_downcast::<SemanticData, _>(|d| d.existing_semantic_node())
                        .flatten()
                        .filter(|n| n.borrow().role == 9)
                })
                .unwrap()
                .borrow()
                .id();
            let field_id = semantic_data
                .iter()
                .find_map(|h| {
                    h.with_downcast::<SemanticData, _>(|d| d.existing_semantic_node())
                        .flatten()
                        .filter(|n| n.borrow().role == 6)
                })
                .unwrap()
                .borrow()
                .id();
            let text = unit
                .with_artboard(|b| b.find_handle::<TextInput>("edit"))
                .unwrap();
            text.with_downcast_mut::<TextInput, _>(|t| t.text_input("hello mascot"));
            let focus_node = unit.with_artboard(|b| b.find_all_handles::<FocusData>())[0]
                .with_downcast::<FocusData, _>(|f| f.existing_focus_node())
                .flatten()
                .unwrap();
            let focus_manager = unit.with_artboard(|b| b.focus_manager()).unwrap();
            if focused {
                focus_manager.with_focus_manager_mut(|m| m.set_focus(focus_node.clone()));
            }
            let check = |host: &nuxie::CoreHandle, copies: usize| {
                let revision = (&(*player).artboard).render_revision.get();
                assert_eq!(
                    nux_player_acknowledge_presented(player, revision),
                    NuxStatus::Ok
                );
                let mut snapshot = ptr::null_mut();
                assert_eq!(
                    nux_player_semantic_snapshot(player, &mut snapshot),
                    NuxStatus::Ok
                );
                let mut info = NuxSemanticSnapshotInfo {
                    struct_size: std::mem::size_of::<NuxSemanticSnapshotInfo>() as u32,
                    ..Default::default()
                };
                assert_eq!(
                    nux_semantic_snapshot_info(snapshot, &mut info),
                    NuxStatus::Ok
                );
                let nodes: Vec<_> = (0..info.node_count)
                    .map(|i| {
                        let mut node = NuxSemanticNodeView {
                            struct_size: std::mem::size_of::<NuxSemanticNodeView>() as u32,
                            ..Default::default()
                        };
                        assert_eq!(
                            nux_semantic_snapshot_node(snapshot, i, &mut node),
                            NuxStatus::Ok
                        );
                        (node.id, node.parent_id, node.role, node.state_flags)
                    })
                    .collect();
                let unique: HashSet<_> = nodes.iter().map(|n| n.0).collect();
                assert_eq!(
                    unique.len(),
                    nodes.len(),
                    "duplicate semantic ids: {nodes:?}"
                );
                assert_eq!(nodes.len(), 4 + copies * 3);
                assert_eq!(
                    nodes.iter().find(|n| n.0 == field_id).unwrap().3 & NUX_SEMANTIC_STATE_FOCUSED
                        != 0,
                    focused
                );
                for id in &ids {
                    assert_eq!(nodes.iter().filter(|n| n.0 == *id).count(), 1);
                }
                let mut descendants = HashSet::from([group_id]);
                loop {
                    let old_len = descendants.len();
                    for n in &nodes {
                        if descendants.contains(&(n.1 as u32)) {
                            descendants.insert(n.0);
                        }
                    }
                    if descendants.len() == old_len {
                        break;
                    }
                }
                assert_eq!(descendants, ids);
                let parent =
                    SemanticData::find_closest_semantic_node_handle(Some(host.clone())).unwrap();
                assert_eq!(
                    nodes.iter().find(|n| n.0 == group_id).unwrap().1,
                    parent.borrow().id() as i32
                );
                assert_eq!(
                    nodes
                        .iter()
                        .filter(|n| n.2 == NUX_SEMANTIC_ROLE_TEXT_FIELD)
                        .count(),
                    copies
                );
                assert_eq!(
                    nodes.iter().find(|n| n.0 == field_id).unwrap().2,
                    NUX_SEMANTIC_ROLE_TEXT_FIELD
                );
                // The same snapshot id still resolves to the live TextInput through the C API.
                let mut length = 0;
                assert_eq!(
                    nux_player_field_string_copy(
                        player,
                        snapshot,
                        field_id,
                        view("edit"),
                        ptr::null_mut(),
                        0,
                        &mut length
                    ),
                    NuxStatus::Ok
                );
                let mut value = vec![0; length];
                assert_eq!(
                    nux_player_field_string_copy(
                        player,
                        snapshot,
                        field_id,
                        view("edit"),
                        value.as_mut_ptr(),
                        value.len(),
                        &mut length
                    ),
                    NuxStatus::Ok
                );
                assert_eq!(value, b"hello mascot");
                assert_eq!(
                    text.with_downcast::<TextInput, _>(|t| t.is_focused())
                        .unwrap(),
                    focused
                );
                assert_eq!(
                    focus_manager.with_focus_manager(|m| m.has_primary_focus(&focus_node)),
                    focused
                );
                nux_semantic_snapshot_free(snapshot);
            };
            check(&a[0], 2);
            let transition = view("t1");
            assert_eq!(
                nux_player_step_begin(player, view("B"), &transition),
                NuxStatus::Ok
            );
            check(&b[1], 2);
            let mut result = ptr::null_mut();
            assert_eq!(
                nux_player_step(
                    player,
                    &NuxPlayerStep {
                        elapsed_seconds: 0.1,
                        ..Default::default()
                    },
                    &mut result
                ),
                NuxStatus::Ok
            );
            nux_player_step_result_free(result);
            check(&b[1], 2);
            assert_eq!(
                nux_player_step_end(player, view("B"), &transition),
                NuxStatus::Ok
            );
            check(&b[0], 1);
            assert_eq!(
                nux_player_step_begin(player, view("A"), ptr::null()),
                NuxStatus::Ok
            );
            check(&a[0], 1);
            assert_eq!(
                nux_player_step_end(player, view("A"), ptr::null()),
                NuxStatus::Ok
            );
            check(&a[0], 1);
            assert_eq!(
                nux_player_step_begin(player, view("B"), &transition),
                NuxStatus::Ok
            );
            check(&b[1], 1);
            assert_eq!(
                nux_player_step_end(player, view("B"), &transition),
                NuxStatus::Ok
            );
            check(&b[0], 1);
            let back = view("back");
            assert_eq!(
                nux_player_step_begin(player, view("A"), &back),
                NuxStatus::Ok
            );
            check(&b[1], 1);
            assert_eq!(nux_player_step_end(player, view("A"), &back), NuxStatus::Ok);
            check(&a[0], 1);
            nux_player_free(player);
            nux_view_model_instance_free(vm);
            nux_artboard_instance_free(artboard);
            nux_file_free(file);
        }
    }
}
