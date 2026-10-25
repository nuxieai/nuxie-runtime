use super::*;
use nuxie::live_move_fixture as fixture;

#[test]
fn page_player_published_emits_initial_presentation() {
    use crate::data_binding::*;
    unsafe {
        let bytes = include_bytes!("../tests/fixtures/emit-values-scene.nux");
        let mut scene = LiveScene {
            file: ptr::null_mut(),
            artboard: ptr::null_mut(),
            vm: ptr::null_mut(),
            player: ptr::null_mut(),
        };
        let commands = NuxHostCommandImportConfig {
            module_name: view("nuxie"),
            ..Default::default()
        };
        let mut imported = ptr::null_mut();
        let status = nux_file_import_trusted_with_host_commands(
            bytes.as_ptr(),
            bytes.len(),
            &NuxRenderCallbacks::default(),
            &commands,
            &mut scene.file,
            &mut imported,
        );
        if !imported.is_null() {
            eprintln!(
                "emits import: {}",
                String::from_utf8_lossy(&(*imported).message)
            );
            nux_capi_result_free(imported);
        }
        assert_eq!(status, NuxStatus::Ok);
        // The portable callback C API has no external-font import hook. Supply
        // the published System slot using the native asset setter used by
        // platform imports, with a deterministic redistributable test face.
        use nuxie::runtime::{assets::font_asset::FontAsset, text::font_hb::HbFont};
        let font_asset = (*scene.file).file.with_file(|file| file.asset(0)).unwrap();
        FontAsset::set_font_occurrence(
            &font_asset,
            Some(
                HbFont::decode(include_bytes!("../../../fixtures/fonts/roboto-input.ttf")).unwrap(),
            ),
        );
        assert_eq!(
            nux_artboard_instance_new_named(scene.file, view("$nuxie.page"), &mut scene.artboard),
            NuxStatus::Ok
        );
        assert_eq!(
            nux_view_model_instance_new_default(scene.artboard, &mut scene.vm),
            NuxStatus::Ok
        );
        assert_eq!(
            nux_artboard_instance_bind_view_model(scene.artboard, scene.vm),
            NuxStatus::Ok
        );
        assert_eq!(
            nux_player_new_state_machine_named(
                scene.artboard,
                view("$nuxie.page"),
                &mut scene.player
            ),
            NuxStatus::Ok
        );
        let mut catalog = ptr::null_mut();
        assert_eq!(
            nux_file_view_model_catalog(scene.file, &mut catalog),
            NuxStatus::Ok
        );
        let mut info = NuxViewModelCatalogInfo::default();
        assert_eq!(
            nux_view_model_catalog_info(catalog, &mut info),
            NuxStatus::Ok
        );
        let row_schema = (0..info.schema_count)
            .find(|&index| {
                let mut schema = NuxViewModelSchemaView::default();
                assert_eq!(
                    nux_view_model_catalog_schema(catalog, index, &mut schema),
                    NuxStatus::Ok
                );
                std::slice::from_raw_parts(schema.name.data.cast::<u8>(), schema.name.len)
                    == b"vm_runtime_root_scr_screens_semits"
            })
            .expect("published emits schema");
        let mut row = ptr::null_mut();
        assert_eq!(
            nux_view_model_instance_new_schema_default(scene.file, row_schema, &mut row),
            NuxStatus::Ok
        );
        for index in 0..3 {
            let mut goal = ptr::null_mut();
            assert_eq!(
                nux_view_model_instance_list_item_acquire(
                    row,
                    view("experience/goals"),
                    index,
                    &mut goal
                ),
                NuxStatus::Ok,
                "published goals defaults must remain populated"
            );
            nux_view_model_instance_free(goal);
        }
        nux_view_model_catalog_free(catalog);
        let mutation = NuxViewModelMutation {
            kind: NUX_VIEW_MODEL_MUTATION_KIND_LIST_INSERT,
            instance: scene.vm,
            path: view("screens"),
            related_instance: row,
            ..Default::default()
        };
        let batch = NuxViewModelMutationBatch {
            mutations: &mutation,
            mutation_count: 1,
            ..Default::default()
        };
        let mut result = ptr::null_mut();
        assert_eq!(nux_view_model_mutate(&batch, &mut result), NuxStatus::Ok);
        nux_view_model_mutation_result_free(result);
        nux_view_model_instance_free(row);
        assert_eq!(nux_player_enable_semantics(scene.player), NuxStatus::Ok);
        step_ok(scene.player);
        acknowledge_current(scene.player);
        let mut snapshot = ptr::null_mut();
        assert_eq!(
            nux_player_semantic_snapshot(scene.player, &mut snapshot),
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
        assert!(info.node_count > 0, "emits must present a semantic tree");
        nux_semantic_snapshot_free(snapshot);
        scene.close();
    }
}

#[test]
fn page_player_minimal_component_list_focus_presentation() {
    unsafe {
        let scene = LiveScene::from_bytes(&fixture::focus_list_fixture());
        use crate::data_binding::*;
        let mut row = ptr::null_mut();
        assert_eq!(
            nux_view_model_instance_new_schema_default(scene.file, 1, &mut row),
            NuxStatus::Ok
        );
        let mutation = NuxViewModelMutation {
            kind: NUX_VIEW_MODEL_MUTATION_KIND_LIST_INSERT,
            instance: scene.vm,
            path: view("screens"),
            related_instance: row,
            ..Default::default()
        };
        let batch = NuxViewModelMutationBatch {
            mutations: &mutation,
            mutation_count: 1,
            ..Default::default()
        };
        let mut result = ptr::null_mut();
        assert_eq!(nux_view_model_mutate(&batch, &mut result), NuxStatus::Ok);
        nux_view_model_mutation_result_free(result);
        nux_view_model_instance_free(row);
        step_ok(scene.player);

        assert_eq!(nux_player_enable_semantics(scene.player), NuxStatus::Ok);
        acknowledge_current(scene.player);
        let mut snapshot = ptr::null_mut();
        assert_eq!(
            nux_player_semantic_snapshot(scene.player, &mut snapshot),
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
        assert!(info.node_count > 0);
        nux_semantic_snapshot_free(snapshot);
        scene.close();
    }
}
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
    if status != NuxStatus::Ok && !result.is_null() {
        eprintln!(
            "fixture import failed: {}",
            String::from_utf8_lossy(unsafe { &(*result).message })
        );
    }
    unsafe {
        nux_capi_result_free(result);
    }
    status
}
// Owns the public handles used by the occurrence regressions below.
struct LiveScene {
    file: *mut NuxFile,
    artboard: *mut NuxArtboardInstance,
    vm: *mut NuxViewModelInstance,
    player: *mut NuxPlayer,
}
impl LiveScene {
    unsafe fn close(mut self) {
        unsafe {
            assert_eq!(nux_player_free(self.player), NuxStatus::Ok);
            self.player = ptr::null_mut();
            assert_eq!(nux_view_model_instance_free(self.vm), NuxStatus::Ok);
            self.vm = ptr::null_mut();
            assert_eq!(nux_artboard_instance_free(self.artboard), NuxStatus::Ok);
            self.artboard = ptr::null_mut();
            assert_eq!(nux_file_free(self.file), NuxStatus::Ok);
            self.file = ptr::null_mut();
        }
    }
    unsafe fn new() -> Self {
        unsafe { Self::from_bytes(&fixture::fixture(2)) }
    }
    unsafe fn from_bytes(bytes: &[u8]) -> Self {
        unsafe {
            let mut scene = Self {
                file: ptr::null_mut(),
                artboard: ptr::null_mut(),
                vm: ptr::null_mut(),
                player: ptr::null_mut(),
            };
            assert_eq!(import_fixture(bytes, &mut scene.file), NuxStatus::Ok);
            assert_eq!(
                nux_artboard_instance_new(scene.file, 0, &mut scene.artboard),
                NuxStatus::Ok
            );
            assert_eq!(
                nux_view_model_instance_new_default(scene.artboard, &mut scene.vm),
                NuxStatus::Ok
            );
            assert_eq!(
                nux_artboard_instance_bind_view_model(scene.artboard, scene.vm),
                NuxStatus::Ok
            );
            assert_eq!(
                nux_player_new_default(scene.artboard, &mut scene.player),
                NuxStatus::Ok
            );
            step_ok(scene.player);
            scene
        }
    }
}
impl Drop for LiveScene {
    fn drop(&mut self) {
        unsafe {
            nux_player_free(self.player);
            nux_view_model_instance_free(self.vm);
            nux_artboard_instance_free(self.artboard);
            nux_file_free(self.file);
        }
    }
}
unsafe fn step_ok(player: *mut NuxPlayer) {
    unsafe {
        let mut result = ptr::null_mut();
        assert_eq!(
            step_status(player, &NuxPlayerStep::default(), &mut result),
            NuxStatus::Ok
        );
        nux_player_step_result_free(result);
    }
}
unsafe fn video_ids(player: *mut NuxPlayer) -> Vec<usize> {
    unsafe extern "C" fn collect(data: *mut c_void, info: *const NuxVideoInfo) {
        unsafe { &mut *data.cast::<Vec<usize>>() }.push(unsafe { &*info }.component_id);
    }
    unsafe {
        let mut ids = Vec::new();
        assert_eq!(
            nux_player_visit_videos(player, Some(collect), ptr::from_mut(&mut ids).cast()),
            NuxStatus::Ok
        );
        ids
    }
}

#[test]
fn live_moves_players_share_the_occurrence_flight_and_history() {
    unsafe {
        let mut scene = LiveScene::new();
        let mut second = ptr::null_mut();
        assert_eq!(
            nux_player_new_default(scene.artboard, &mut second),
            NuxStatus::Ok
        );
        let t = view("t1");
        assert_eq!(
            nux_player_step_begin(scene.player, view("B"), &t),
            NuxStatus::Ok
        );
        let flying = video_ids(scene.player);
        assert_eq!(video_ids(second), flying);
        step_ok(second);
        assert_eq!(
            nux_player_step_end(scene.player, view("B"), &t),
            NuxStatus::Ok
        );
        // The other player also sees the landed unit and its coming-home history.
        assert_eq!(
            nux_player_step_begin(second, view("A"), ptr::null()),
            NuxStatus::Ok
        );
        assert_eq!(
            nux_player_step_end(scene.player, view("A"), ptr::null()),
            NuxStatus::Ok
        );
        assert_eq!(video_ids(second), video_ids(scene.player));
        assert_eq!(
            nux_player_step_begin(scene.player, view("B"), &t),
            NuxStatus::Ok
        );
        nux_player_free(scene.player);
        scene.player = ptr::null_mut();
        step_ok(second);
        assert_eq!(nux_player_step_end(second, view("B"), &t), NuxStatus::Ok);
        nux_player_free(second);
    }
}

#[test]
fn live_moves_sleeping_video_can_reclaim_and_resume_the_same_id() {
    unsafe fn observe(
        player: *mut NuxPlayer,
        id: usize,
        kind: u32,
        generation: u64,
    ) -> Vec<(u32, f64, u64)> {
        unsafe extern "C" fn action(data: *mut c_void, action: *const NuxVideoAction) {
            let a = unsafe { &*action };
            unsafe { &mut *data.cast::<Vec<(u32, f64, u64)>>() }.push((
                a.kind,
                a.value,
                a.generation,
            ));
        }
        let mut actions = Vec::new();
        assert_eq!(
            unsafe {
                nux_player_video_step(
                    player,
                    id,
                    kind,
                    generation,
                    10.0,
                    Some(action),
                    ptr::from_mut(&mut actions).cast(),
                )
            },
            NuxStatus::Ok
        );
        actions
    }
    unsafe {
        let scene = LiveScene::from_bytes(&fixture::playing_video_fixture());
        // Isolate the source's video, retaining the other Screen asleep.
        assert_eq!(
            nux_player_set_row_collapsed(scene.player, view("screens"), 1, 1),
            NuxStatus::Ok
        );
        let ids = video_ids(scene.player);
        assert_eq!(ids.len(), 1);
        let id = ids[0];
        // Drain the authored initial seek before opening its decoder generation.
        assert!(observe(scene.player, id, 0, 0).contains(&(2, 2.5, 1)));
        assert!(observe(scene.player, id, 1, 1).contains(&(0, 0.0, 1)));
        observe(scene.player, id, 2, 1);
        let clock = NuxVideoClockSample {
            generation: 1,
            seconds: 2.5,
            rate: 1.0,
            playing: 1,
            available: 1,
        };
        assert_eq!(
            nux_player_video_report_clock(scene.player, id, 1.0, &clock),
            NuxStatus::Ok
        );
        let mut visible = 0;
        assert_eq!(
            nux_player_video_is_visible(
                scene.player,
                id,
                -1000.0,
                -1000.0,
                1000.0,
                1000.0,
                &mut visible
            ),
            NuxStatus::Ok
        );
        assert_eq!(visible, 1);
        assert_eq!(
            nux_player_set_row_collapsed(scene.player, view("screens"), 0, 1),
            NuxStatus::Ok
        );
        assert!(video_ids(scene.player).is_empty());
        let mut replacement = u64::MAX;
        assert_eq!(
            nux_player_video_reclaim_decoder(scene.player, id, 1, &mut replacement),
            NuxStatus::Ok,
            "sleeping retained video must remain controllable"
        );
        assert_eq!(replacement, 2);
        assert_eq!(
            nux_player_video_is_visible(
                scene.player,
                id,
                -1000.0,
                -1000.0,
                1000.0,
                1000.0,
                &mut visible
            ),
            NuxStatus::Ok
        );
        assert_eq!(visible, 0);
        assert_eq!(
            nux_player_video_command(scene.player, id, 6, 1.0, 1),
            NuxStatus::Ok
        );
        assert!(
            !observe(scene.player, id, 0, replacement)
                .iter()
                .any(|a| a.0 == 0)
        );
        assert_eq!(
            nux_player_set_row_collapsed(scene.player, view("screens"), 0, 0),
            NuxStatus::Ok
        );
        assert_eq!(video_ids(scene.player), ids);
        assert_eq!(
            nux_player_video_command(scene.player, id, 6, 0.0, 1),
            NuxStatus::Ok
        );
        assert_eq!(
            nux_player_video_command(scene.player, id, 6, 0.0, 8),
            NuxStatus::Ok
        );
        assert!(
            observe(scene.player, id, 1, 1).is_empty(),
            "retired decoder ready is ignored"
        );
        let actions = observe(scene.player, id, 1, replacement);
        let seek = actions
            .iter()
            .position(|a| *a == (2, 2.5, replacement))
            .expect("replacement seeks to retained clock");
        let play = actions
            .iter()
            .position(|a| *a == (0, 0.0, replacement))
            .expect("replacement restores play intent");
        assert!(seek < play);
        assert!(
            observe(scene.player, id, 1, replacement).is_empty(),
            "duplicate ready is ignored"
        );
        assert_eq!(video_ids(scene.player), ids);
    }
}

unsafe fn acknowledge_current(player: *mut NuxPlayer) -> u64 {
    unsafe {
        let mut result = ptr::null_mut();
        assert_eq!(
            step_status(player, &NuxPlayerStep::default(), &mut result),
            NuxStatus::Ok
        );
        let mut info = NuxPlayerSchedulingInfo::default();
        assert_eq!(
            nux_player_step_result_scheduling(result, &mut info),
            NuxStatus::Ok
        );
        nux_player_step_result_free(result);
        assert_eq!(
            nux_player_acknowledge_presented(player, info.render_revision),
            NuxStatus::Ok
        );
        info.render_revision
    }
}

fn set_live_flight_progress(root: &nuxie::RuntimeArtboardInstanceHandle, progress: f32) {
    let rows = nuxie::live_move::screens(root).unwrap();
    for record in
        rows["B"].with_artboard(|b| b.find_all_handles::<nuxie::live_move::records::LiveSlot>())
    {
        let is_flight = record
            .with_downcast::<nuxie::live_move::records::LiveSlot, _>(|r| {
                r.values.flight_key == 2 && r.values.role == 1
            })
            .unwrap();
        if is_flight {
            assert!(
                nuxie::runtime::generated::core_registry::CoreRegistry::set_double_handle(
                    &record, 62003, progress
                )
            );
        }
    }
}

#[test]
fn live_moves_publish_exact_binding_owners_on_takeoff_landing_and_wake() {
    unsafe {
        let scene = LiveScene::new();
        let mut shared = ptr::null_mut();
        assert_eq!(
            nux_view_model_instance_new_schema_default(scene.file, 0, &mut shared),
            NuxStatus::Ok
        );
        let root = (&(*scene.player).artboard)
            .instance
            .borrow()
            .native_handle();
        let rows = nuxie::live_move::screens(&root).unwrap();
        let source = rows["A"].with_artboard(|b| b.nested_artboards()[0].clone());
        let unit = source
            .with(|o| o.as_nested_artboard().unwrap().artboard_instance_default())
            .flatten()
            .unwrap();
        // Test setup: expose this nested unit's model through a public shared
        // handle. The C API currently exports root and list models only.
        let native_model = unit
            .data_context()
            .unwrap()
            .with_context(|context| context.main_view_model_instance())
            .unwrap();
        (*shared).instance =
            RuntimeOwnedViewModelHandle::from_native((*scene.file).file.clone(), native_model)
                .unwrap();
        (*shared).identity = (*shared).instance.instance_identity();
        let mut observer = ptr::null_mut();
        let mut observer_player = ptr::null_mut();
        assert_eq!(
            nux_artboard_instance_new(scene.file, 3, &mut observer),
            NuxStatus::Ok
        );
        assert_eq!(
            nux_artboard_instance_bind_view_model(observer, shared),
            NuxStatus::Ok
        );
        assert_eq!(
            nux_player_new_default(observer, &mut observer_player),
            NuxStatus::Ok
        );
        // A second model with the same schema must not receive broad invalidation.
        let mut unrelated = ptr::null_mut();
        let mut unrelated_vm = ptr::null_mut();
        let mut unrelated_player = ptr::null_mut();
        assert_eq!(
            nux_artboard_instance_new(scene.file, 3, &mut unrelated),
            NuxStatus::Ok
        );
        assert_eq!(
            nux_view_model_instance_new_default(unrelated, &mut unrelated_vm),
            NuxStatus::Ok
        );
        assert_eq!(
            nux_artboard_instance_bind_view_model(unrelated, unrelated_vm),
            NuxStatus::Ok
        );
        assert_eq!(
            nux_player_new_default(unrelated, &mut unrelated_player),
            NuxStatus::Ok
        );
        let unrelated_revision = acknowledge_current(unrelated_player);
        let mut screen_b = ptr::null_mut();
        assert_eq!(
            nux_view_model_instance_list_item_acquire(scene.vm, view("screens"), 1, &mut screen_b),
            NuxStatus::Ok
        );
        let property = || {
            let mut snapshot = ptr::null_mut();
            assert_eq!(
                nux_view_model_instance_snapshot(shared, &mut snapshot),
                NuxStatus::Ok
            );
            let mut value = NuxViewModelSnapshotValueView::default();
            assert_eq!(
                nux_view_model_snapshot_value(snapshot, 1, &mut value),
                NuxStatus::Ok
            );
            nux_view_model_snapshot_free(snapshot);
            value.number_value
        };
        // This regression covers immediate value publication at a completed flight.
        set_live_flight_progress(&root, 1.0);
        let revision = acknowledge_current(observer_player);
        assert_eq!(
            nux_player_step_begin(scene.player, view("B"), &view("t1")),
            NuxStatus::Ok
        );
        assert_eq!(property(), 200.0);
        assert_eq!(
            nux_player_acknowledge_presented(observer_player, revision),
            NuxStatus::HandleMismatch,
            "takeoff must invalidate the other holder"
        );
        let revision = acknowledge_current(observer_player);
        assert_eq!(
            nux_view_model_instance_set_number(screen_b, c"offset".as_ptr(), 321.0),
            NuxStatus::Ok
        );
        assert_eq!(
            nux_player_step_end(scene.player, view("B"), &view("t1")),
            NuxStatus::Ok
        );
        assert_eq!(property(), 321.0);
        assert_eq!(
            nux_player_acknowledge_presented(observer_player, revision),
            NuxStatus::HandleMismatch,
            "landing must invalidate the other holder"
        );
        assert_eq!(
            nux_player_set_row_collapsed(scene.player, view("screens"), 1, 1),
            NuxStatus::Ok
        );
        let revision = acknowledge_current(observer_player);
        assert_eq!(
            nux_view_model_instance_set_number(screen_b, c"offset".as_ptr(), 456.0),
            NuxStatus::Ok
        );
        assert_eq!(
            nux_player_acknowledge_presented(observer_player, revision),
            NuxStatus::Ok,
            "a sleeping binding has not written the unit yet"
        );
        assert_eq!(
            nux_player_set_row_collapsed(scene.player, view("screens"), 1, 0),
            NuxStatus::Ok
        );
        assert_eq!(property(), 456.0);
        assert_eq!(
            nux_player_acknowledge_presented(observer_player, revision),
            NuxStatus::HandleMismatch,
            "wake must publish owners that were outside the active index"
        );
        assert_eq!(
            nux_player_acknowledge_presented(unrelated_player, unrelated_revision),
            NuxStatus::Ok,
            "only exact changed owners are published"
        );
        nux_view_model_instance_free(screen_b);
        nux_player_free(unrelated_player);
        nux_view_model_instance_free(unrelated_vm);
        nux_artboard_instance_free(unrelated);
        nux_player_free(observer_player);
        nux_artboard_instance_free(observer);
        nux_view_model_instance_free(shared);
    }
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
            step_status(player, &NuxPlayerStep::default(), &mut result),
            NuxStatus::Ok
        );
        nux_player_step_result_free(result);
        let root = (&(*player).artboard).instance.borrow().native_handle();
        let rows = nuxie::live_move::screens(&root).unwrap();
        let a = rows["A"].with_artboard(|b| b.nested_artboards()[0].clone());
        let b = rows["B"].with_artboard(|b| b.nested_artboards().to_vec());
        let child = |h: &nuxie::CoreHandle| {
            h.with(|o| o.as_nested_artboard().unwrap().artboard_instance_default())
                .flatten()
        };
        let original = child(&a).unwrap();
        let keyed = original
            .with_artboard(|b| b.find_handle::<nuxie::runtime::node::Node>("keyed"))
            .unwrap();
        let transition = view("t1");
        let step = NuxPlayerStep {
            elapsed_seconds: 0.3,
            ..Default::default()
        };
        let mut result = ptr::null_mut();
        assert_eq!(step_status(player, &step, &mut result), NuxStatus::Ok);
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
        assert_eq!(step_status(player, &step, &mut result), NuxStatus::Ok);
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
            nux_player_step_begin(player, view("A"), &view("missing")),
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
            step_status(player, &NuxPlayerStep::default(), &mut result),
            NuxStatus::Ok
        );
        nux_player_step_result_free(result);
        let root = (&(*player).artboard).instance.borrow().native_handle();
        let rows = nuxie::live_move::screens(&root).unwrap();
        for (name, row) in rows {
            for (index, host) in row
                .with_artboard(|b| b.nested_artboards().to_vec())
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
                step_status(player, &NuxPlayerStep::default(), &mut result),
                NuxStatus::Ok
            );
            nux_player_step_result_free(result);
            let root = (&(*player).artboard).instance.borrow().native_handle();
            let rows = nuxie::live_move::screens(&root).unwrap();
            let a = rows["A"].with_artboard(|b| b.nested_artboards().to_vec());
            let b = rows["B"].with_artboard(|b| b.nested_artboards().to_vec());
            nuxie::live_move::empty_slot(&a[1]).unwrap();
            nuxie::live_move::empty_slot(&b[1]).unwrap();
            let unit = a[0]
                .with(|o| o.as_nested_artboard().unwrap().artboard_instance_default())
                .flatten()
                .unwrap();
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
            check(&b[1], 1);
            let mut result = ptr::null_mut();
            assert_eq!(
                step_status(
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
            check(&b[1], 1);
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

unsafe fn step_status(
    player: *mut NuxPlayer,
    input: *const NuxPlayerStep,
    output: *mut *mut NuxPlayerStepResult,
) -> NuxStatus {
    let status = unsafe { nux_player_step(player, input, output) };
    if status != NuxStatus::Ok && !output.is_null() && !unsafe { *output }.is_null() {
        eprintln!(
            "player step failed with {:?}: {}",
            status,
            String::from_utf8_lossy(unsafe { &(**output).message })
        );
    }
    status
}

#[test]
fn live_moves_use_the_player_guards_before_reading_arguments() {
    unsafe fn calls(player: *mut NuxPlayer) -> [NuxStatus; 3] {
        let unreadable = NuxStringView {
            data: std::ptr::dangling(),
            len: 1,
        };
        [
            unsafe { nux_player_step_begin(player, unreadable, std::ptr::dangling()) },
            unsafe { nux_player_step_end(player, unreadable, std::ptr::dangling()) },
            unsafe { nux_player_set_row_collapsed(player, unreadable, 0, 1) },
        ]
    }
    unsafe {
        assert_eq!(calls(ptr::null_mut()), [NuxStatus::NullArgument; 3]);
        assert_eq!(calls(ptr::dangling_mut()), [NuxStatus::HandleMismatch; 3]);
        let (mut file, mut artboard, mut vm, mut player) = (
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
            ptr::null_mut(),
        );
        assert_eq!(
            import_fixture(&fixture::fixture(2), &mut file),
            NuxStatus::Ok
        );
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
        let address = player as usize;
        assert_eq!(
            std::thread::spawn(move || calls(address as *mut NuxPlayer))
                .join()
                .unwrap(),
            [NuxStatus::WrongThread; 3]
        );
        let handle = enter_handle(player, HandleKind::Player).unwrap();
        assert_eq!(calls(player), [NuxStatus::ReentrantCall; 3]);
        drop(handle);
        let occurrence = enter_occurrence(&(*player).artboard).unwrap();
        assert_eq!(calls(player), [NuxStatus::ReentrantCall; 3]);
        drop(occurrence);
        (&(*player).artboard).poisoned.set(true);
        assert_eq!(calls(player), [NuxStatus::RuntimeError; 3]);
        (&(*player).artboard).poisoned.set(false);
        (&(*player).artboard).render_revision.set(u64::MAX);
        assert_eq!(calls(player), [NuxStatus::LimitExceeded; 3]);
        nux_player_free(player);
        nux_view_model_instance_free(vm);
        nux_artboard_instance_free(artboard);
        nux_file_free(file);
    }
}

#[test]
fn live_moves_checkpoint_overflow_poisons_the_occurrence_after_playback() {
    unsafe {
        for properties in [2, 4097] {
            let bytes = fixture::checkpoint_overflow_fixture(properties);
            let (mut file, mut artboard, mut vm, mut player) = (
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null_mut(),
            );
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
            assert_eq!(
                nux_player_new_linear_animation_named(artboard, view("walk"), &mut player),
                NuxStatus::Ok
            );
            let root = (&(*player).artboard).instance.borrow().native_handle();
            let keyed = root
                .with_artboard(|a| a.find_handle::<nuxie::runtime::node::Node>("keyed"))
                .unwrap();
            let mut result = ptr::null_mut();
            let status = step_status(
                player,
                &NuxPlayerStep {
                    elapsed_seconds: 0.5,
                    ..Default::default()
                },
                &mut result,
            );
            if properties == 2 {
                assert_eq!(status, NuxStatus::Ok);
                assert_eq!(nux_artboard_instance_draw(artboard), NuxStatus::Ok);
            } else {
                assert_eq!(status, NuxStatus::LimitExceeded);
                assert_eq!(
                    (*result).message.as_ref(),
                    b"live frame checkpoint exceeds its bound"
                );
                assert!(
                    (nuxie::runtime::generated::core_registry::CoreRegistry::get_double_handle(
                        &keyed, 13
                    )
                    .unwrap()
                        - 50.0)
                        .abs()
                        < 0.002,
                    "playback already changed the scene before overflow was reported"
                );
                assert_eq!(
                    nux_artboard_instance_draw(artboard),
                    NuxStatus::RuntimeError,
                    "a half-applied frame must never remain drawable"
                );
                assert_eq!(
                    nux_player_step_begin(player, view("A"), ptr::null()),
                    NuxStatus::RuntimeError
                );
                let mut retry = ptr::null_mut();
                assert_eq!(
                    step_status(player, &NuxPlayerStep::default(), &mut retry),
                    NuxStatus::RuntimeError
                );
                nux_player_step_result_free(retry);
            }
            nux_player_step_result_free(result);
            nux_player_free(player);
            nux_view_model_instance_free(vm);
            nux_artboard_instance_free(artboard);
            nux_file_free(file);
        }
    }
}

#[test]
fn live_moves_sleep_wake_and_hidden_transfers_keep_sdk_identities_round4() {
    use nuxie::{live_move, runtime::semantic::semantic_data::SemanticData};
    unsafe {
        let scene = LiveScene::from_bytes(&fixture::semantic_fixture());
        let root = (&(*scene.player).artboard)
            .instance
            .borrow()
            .native_handle();
        let rows = live_move::screens(&root).unwrap();
        let a = rows["A"].with_artboard(|b| b.nested_artboards().to_vec());
        let b = rows["B"].with_artboard(|b| b.nested_artboards().to_vec());
        for slot in [&a[1], &b[1], &b[0]] {
            live_move::empty_slot(slot).unwrap();
        }
        let unit = a[0]
            .with(|o| o.as_nested_artboard().unwrap().artboard_instance_default())
            .flatten()
            .unwrap();
        assert_eq!(nux_player_enable_semantics(scene.player), NuxStatus::Ok);
        let field = || {
            unit.with_artboard(|b| b.find_all_handles::<SemanticData>())
                .iter()
                .find_map(|h| {
                    h.with_downcast::<SemanticData, _>(|d| d.existing_semantic_node())
                        .flatten()
                        .filter(|n| n.borrow().role == 6)
                })
                .unwrap()
        };
        let node = field();
        let semantic_id = node.borrow().id();
        let ids = video_ids(scene.player);
        assert_eq!(ids.len(), 1);
        let video_id = ids[0];
        let check = |shown: bool| {
            assert!(std::rc::Rc::ptr_eq(&node, &field()));
            assert_eq!(field().borrow().id(), semantic_id);
            let revision = (&(*scene.player).artboard).render_revision.get();
            assert_eq!(
                nux_player_acknowledge_presented(scene.player, revision),
                NuxStatus::Ok
            );
            let mut snapshot = ptr::null_mut();
            assert_eq!(
                nux_player_semantic_snapshot(scene.player, &mut snapshot),
                NuxStatus::Ok
            );
            let mut info = NuxSemanticSnapshotInfo {
                struct_size: size_of::<NuxSemanticSnapshotInfo>() as u32,
                ..Default::default()
            };
            assert_eq!(
                nux_semantic_snapshot_info(snapshot, &mut info),
                NuxStatus::Ok
            );
            let mut matches = 0;
            for i in 0..info.node_count {
                let mut view = NuxSemanticNodeView {
                    struct_size: size_of::<NuxSemanticNodeView>() as u32,
                    ..Default::default()
                };
                assert_eq!(
                    nux_semantic_snapshot_node(snapshot, i, &mut view),
                    NuxStatus::Ok
                );
                if view.id == semantic_id {
                    matches += 1;
                }
            }
            nux_semantic_snapshot_free(snapshot);
            assert_eq!(matches, usize::from(shown));
            assert_eq!(
                video_ids(scene.player),
                if shown { vec![video_id] } else { vec![] }
            );
            // A sleeping video remains addressable by the same SDK ID.
            let mut visible = 0;
            assert_eq!(
                nux_player_video_is_visible(
                    scene.player,
                    video_id,
                    -10000.0,
                    -10000.0,
                    10000.0,
                    10000.0,
                    &mut visible
                ),
                NuxStatus::Ok
            );
            if !shown {
                assert_eq!(visible, 0);
            }
        };
        let sleep = |row, value| {
            assert_eq!(
                nux_player_set_row_collapsed(scene.player, view("screens"), row, value),
                NuxStatus::Ok
            )
        };
        check(true);
        sleep(0, 1);
        check(false);
        sleep(0, 0);
        check(true);
        let t = view("t1");
        assert_eq!(
            nux_player_step_begin(scene.player, view("B"), &t),
            NuxStatus::Ok
        );
        assert_eq!(
            nux_player_step_end(scene.player, view("B"), &t),
            NuxStatus::Ok
        );
        check(true);
        sleep(1, 1);
        check(false);
        // A cut returns the same occurrence out of a sleeping row.
        assert_eq!(
            nux_player_step_begin(scene.player, view("A"), ptr::null()),
            NuxStatus::Ok
        );
        assert_eq!(
            nux_player_step_end(scene.player, view("A"), ptr::null()),
            NuxStatus::Ok
        );
        check(true);
        // Step incoming Screens must be awake. Exercise the transport seam for
        // an attachment into a sleeping destination instead.
        live_move::move_live(&a[0], &b[0]).unwrap();
        check(false);
        sleep(1, 0);
        check(true);
        sleep(1, 1);
        check(false);
        live_move::move_live(&b[0], &a[0]).unwrap();
        check(true);
        println!(
            "SDK identities through sleep/wake and hidden transfers: semantic={semantic_id} video={video_id}"
        );
    }
}

#[test]
fn live_moves_global_only_boundary_publishes_and_notifies_round5() {
    unsafe {
        let mut scene = LiveScene {
            file: ptr::null_mut(),
            artboard: ptr::null_mut(),
            vm: ptr::null_mut(),
            player: ptr::null_mut(),
        };
        assert_eq!(
            import_fixture(&fixture::global_fixture(), &mut scene.file),
            NuxStatus::Ok
        );
        assert_eq!(
            nux_artboard_instance_new(scene.file, 0, &mut scene.artboard),
            NuxStatus::Ok
        );
        assert_eq!(
            nux_view_model_instance_new_default(scene.artboard, &mut scene.vm),
            NuxStatus::Ok
        );
        assert_eq!(
            nux_player_new_default(scene.artboard, &mut scene.player),
            NuxStatus::Ok
        );
        assert_eq!(
            nux_player_set_global_view_model(scene.player, view("Page"), scene.vm),
            NuxStatus::Ok
        );
        step_ok(scene.player);
        assert!(
            (&(*scene.player).artboard)
                .bound_view_model
                .borrow()
                .is_none()
        );
        let root = (&(*scene.player).artboard)
            .instance
            .borrow()
            .native_handle();
        let rows = nuxie::live_move::screens(&root).unwrap();
        let source = rows["A"].with_artboard(|b| b.nested_artboards()[0].clone());
        let unit = source
            .with(|o| o.as_nested_artboard().unwrap().artboard_instance_default())
            .flatten()
            .unwrap();
        let native_model = unit
            .data_context()
            .unwrap()
            .with_context(|c| c.main_view_model_instance())
            .unwrap();
        let mut shared = ptr::null_mut();
        assert_eq!(
            nux_view_model_instance_new_schema_default(scene.file, 0, &mut shared),
            NuxStatus::Ok
        );
        (*shared).instance =
            RuntimeOwnedViewModelHandle::from_native((*scene.file).file.clone(), native_model)
                .unwrap();
        (*shared).identity = (*shared).instance.instance_identity();
        assert_eq!(
            nux_player_set_global_view_model(scene.player, view("Unit"), shared),
            NuxStatus::Ok
        );
        step_ok(scene.player);
        let mut observer = ptr::null_mut();
        let mut observer_player = ptr::null_mut();
        assert_eq!(
            nux_artboard_instance_new(scene.file, 3, &mut observer),
            NuxStatus::Ok
        );
        assert_eq!(
            nux_player_new_default(observer, &mut observer_player),
            NuxStatus::Ok
        );
        assert_eq!(
            nux_player_set_global_view_model(observer_player, view("Unit"), shared),
            NuxStatus::Ok
        );
        let runtime = nuxie::ViewModelInstanceRuntime::new((*shared).instance.native_handle());
        let number = runtime.property_number("prop").unwrap();
        assert_eq!(
            nux_view_model_instance_set_number(shared, c"prop".as_ptr(), 17.0),
            NuxStatus::Ok
        );
        let mut screen_b = ptr::null_mut();
        assert_eq!(
            nux_view_model_instance_list_item_acquire(scene.vm, view("screens"), 1, &mut screen_b),
            NuxStatus::Ok
        );
        // Without a main bind, the runtime mounts its own default page model.
        // Expose the actual mounted Screen owner, as for the nested unit above;
        // the separately created unbound Page handle need not own these rows.
        let mounted_b = rows["B"]
            .data_context()
            .unwrap()
            .with_context(|c| c.main_view_model_instance())
            .unwrap();
        (*screen_b).instance =
            RuntimeOwnedViewModelHandle::from_native((*scene.file).file.clone(), mounted_b)
                .unwrap();
        (*screen_b).identity = (*screen_b).instance.instance_identity();
        set_live_flight_progress(&root, 1.0);
        let transition = view("t1");
        for (stage, expected) in [("takeoff", 200.0), ("landing", 321.0), ("wake", 456.0)] {
            if stage == "landing" {
                assert_eq!(
                    nux_view_model_instance_set_number(screen_b, c"offset".as_ptr(), expected),
                    NuxStatus::Ok
                );
            } else if stage == "wake" {
                assert_eq!(
                    nux_player_set_row_collapsed(scene.player, view("screens"), 1, 1),
                    NuxStatus::Ok
                );
                assert_eq!(
                    nux_view_model_instance_set_number(screen_b, c"offset".as_ptr(), expected),
                    NuxStatus::Ok
                );
            }
            let revision = acknowledge_current(observer_player);
            number.value_runtime().clear_changes();
            let status = match stage {
                "takeoff" => nux_player_step_begin(scene.player, view("B"), &transition),
                "landing" => nux_player_step_end(scene.player, view("B"), &transition),
                _ => nux_player_set_row_collapsed(scene.player, view("screens"), 1, 0),
            };
            assert_eq!(status, NuxStatus::Ok, "{stage}");
            assert_eq!(
                number.value(),
                expected,
                "{stage} wrote the subscribed global"
            );
            assert_eq!(
                nux_player_acknowledge_presented(observer_player, revision),
                NuxStatus::HandleMismatch,
                "{stage} must invalidate the other global subscriber without a main-model bind"
            );
            assert!(
                number.value_runtime().flush_changes(),
                "{stage} must deliver the shared property's notification after commit"
            );
            assert!(
                !number.value_runtime().flush_changes(),
                "notification is delivered once"
            );
        }
        nux_view_model_instance_free(screen_b);
        nux_player_free(observer_player);
        nux_artboard_instance_free(observer);
        nux_view_model_instance_free(shared);
    }
}

#[test]
fn live_moves_use_step_publication_poisoning_round5() {
    unsafe {
        for stage in ["takeoff", "landing", "wake"] {
            let scene = LiveScene::new();
            let t = view("t1");
            if stage == "landing" {
                assert_eq!(
                    nux_player_step_begin(scene.player, view("B"), &t),
                    NuxStatus::Ok
                );
            }
            if stage == "wake" {
                assert_eq!(
                    nux_player_set_row_collapsed(scene.player, view("screens"), 1, 1),
                    NuxStatus::Ok
                );
            }
            PANIC_BEFORE_STEP_RESULT_REGISTRATION.with(|armed| armed.set(true));
            let status = match stage {
                "takeoff" => nux_player_step_begin(scene.player, view("B"), &t),
                "landing" => nux_player_step_end(scene.player, view("B"), &t),
                _ => nux_player_set_row_collapsed(scene.player, view("screens"), 1, 0),
            };
            PANIC_BEFORE_STEP_RESULT_REGISTRATION.with(|armed| armed.set(false));
            assert_eq!(
                status,
                NuxStatus::RuntimeError,
                "{stage} must share the step's publication boundary"
            );
            assert!((&(*scene.player).artboard).poisoned.get());
            assert_eq!(
                nux_player_set_row_collapsed(scene.player, view("screens"), 0, 0),
                NuxStatus::RuntimeError
            );
        }
    }
}

#[test]
fn live_moves_record_free_media_queries_keep_no_live_cache_round6() {
    unsafe extern "C" fn collect(data: *mut c_void, info: *const NuxVideoInfo) {
        unsafe { &mut *data.cast::<Vec<usize>>() }.push(unsafe { &*info }.component_id);
    }
    unsafe {
        let before = nuxie::hosted_scene::bookkeeping_counts();
        let scene = LiveScene::from_bytes(&fixture::ordinary_fixture(2));
        let mut ids: Vec<usize> = Vec::new();
        assert_eq!(
            nux_player_visit_videos(scene.player, Some(collect), ptr::from_mut(&mut ids).cast()),
            NuxStatus::Ok
        );
        assert!(!ids.is_empty());
        for id in ids {
            let mut visible = 0;
            assert_eq!(
                nux_player_video_is_visible(
                    scene.player,
                    id,
                    0.0,
                    0.0,
                    1000.0,
                    1000.0,
                    &mut visible
                ),
                NuxStatus::Ok
            );
        }
        assert!(
            !(&(*scene.player).artboard)
                .video_occurrences
                .borrow()
                .has_live_bookkeeping()
        );
        assert_eq!(nuxie::hosted_scene::bookkeeping_counts(), before);
        drop(scene);
        assert_eq!(nuxie::hosted_scene::bookkeeping_counts(), before);
    }
}

#[test]
fn live_moves_rebind_default_page_then_step_does_not_poison_round7() {
    unsafe {
        let mut scene = LiveScene::from_bytes(&nuxie::live_move::device_fixture::device_fixture(2));
        for _ in 0..2 {
            let mut replacement = ptr::null_mut();
            assert_eq!(
                nux_view_model_instance_new_default(scene.artboard, &mut replacement),
                NuxStatus::Ok
            );
            assert_eq!(
                nux_artboard_instance_bind_view_model(scene.artboard, replacement),
                NuxStatus::Ok
            );
            nux_view_model_instance_free(scene.vm);
            scene.vm = replacement;
            step_ok(scene.player);
            step_ok(scene.player);
        }
    }
}

#[test]
fn live_moves_paint_content_edits_invalidate_the_enclosing_index_round9() {
    use nuxie_runtime::source::{
        artboard_component_list::ArtboardComponentList, nested_artboard::NestedArtboard,
        shapes::paint::paint_image::PaintImage,
        viewmodel::viewmodel_instance_artboard::ViewModelInstanceArtboard,
    };
    let scenes = nuxie::hosted_scene::bookkeeping_counts().revisions;
    unsafe {
        let scene = LiveScene::from_bytes(&fixture::paint_list_fixture());
        assert_eq!(
            nuxie::hosted_scene::bookkeeping_counts().revisions,
            scenes + 1,
            "only the unhosted Page may own scene bookkeeping"
        );
        let root = (&(*scene.player).artboard)
            .instance
            .borrow()
            .native_handle();
        let painted = root
            .with_artboard(|b| b.find_handle::<PaintImage>("painted"))
            .unwrap();
        let content = painted
            .with_downcast::<PaintImage, _>(|p| p.source.as_ref().unwrap().artboard())
            .unwrap();
        assert!(
            !content
                .with_artboard(|b| b.find_all_handles::<nuxie::live_move::records::LiveStep>())
                .is_empty()
        );
        assert!(!nuxie::hosted_scene::enabled(&content));
        assert!(
            nuxie::live_move::LiveMoves::default()
                .step_begin(&content, "B", Some("t1"))
                .is_err()
        );
        let list = content
            .with_artboard(|b| b.find_handle::<ArtboardComponentList>("inner"))
            .unwrap();
        let screens = root
            .with_artboard(|b| b.find_handle::<ArtboardComponentList>("screens"))
            .unwrap();
        let item = screens
            .with_downcast::<ArtboardComponentList, _>(|l| l.list_item(1))
            .flatten()
            .unwrap();
        let before = video_ids(scene.player).len();
        ArtboardComponentList::update_list_occurrence(&list, &[item.clone()]);
        assert_eq!(
            video_ids(scene.player).len(),
            before + 2,
            "paint list edit left the enclosing index stale"
        );
        step_ok(scene.player);
        let row = list
            .with_downcast::<ArtboardComponentList, _>(|l| l.artboard_instance(0))
            .flatten()
            .unwrap();
        let slot = row.with_artboard(|b| b.find_all_handles::<NestedArtboard>())[0].clone();
        let old = slot
            .with_downcast::<NestedArtboard, _>(NestedArtboard::artboard_instance_default)
            .flatten()
            .unwrap();
        let weak = old.downgrade();
        let old_ids = video_ids(scene.player);
        let value = slot
            .insert_sibling(ViewModelInstanceArtboard::default())
            .unwrap();
        let asset = (&(*scene.file).file)
            .with_file(|f| f.bindable_artboard_named("mascot B"))
            .unwrap();
        value.with_downcast_mut::<ViewModelInstanceArtboard, _>(|v| {
            v.set_asset(Some(asset));
        });
        let revision = nuxie::hosted_scene::membership_revision(&root.core_handle());
        assert_eq!(
            nuxie::hosted_scene::membership_revision(&slot),
            revision,
            "paint descendant must resolve the enclosing scene"
        );
        assert_eq!(
            painted
                .with_downcast::<PaintImage, _>(|p| p
                    .source
                    .as_ref()
                    .unwrap()
                    .artboard()
                    .core_handle())
                .unwrap(),
            content.core_handle(),
            "test must edit the currently mounted paint content"
        );
        NestedArtboard::update_artboard_occurrence(&slot, Some(value));
        let replacement = slot
            .with_downcast::<NestedArtboard, _>(NestedArtboard::artboard_instance_default)
            .flatten()
            .unwrap();
        assert_ne!(
            replacement.core_handle(),
            old.core_handle(),
            "native replacement did not replace the child"
        );
        assert!(
            nuxie::hosted_scene::membership_revision(&root.core_handle()) > revision,
            "paint child replacement must invalidate the enclosing index"
        );
        drop(replacement);
        drop(old);
        let ids = video_ids(scene.player);
        assert_eq!(ids.len(), before + 2);
        assert_eq!(ids.iter().filter(|id| !old_ids.contains(id)).count(), 1);
        step_ok(scene.player);
        assert!(
            weak.upgrade().is_none(),
            "retired paint child retained after refresh"
        );
        assert!(
            !nuxie::hosted_scene::enabled(&content),
            "paint content acquired its own scene"
        );
        ArtboardComponentList::update_list_occurrence(&list, &[]);
        assert_eq!(video_ids(scene.player).len(), before);
    }
}

#[cfg(feature = "scripting")]
#[test]
fn live_moves_script_commands_are_delivered_by_the_next_step_once_round9() {
    unsafe {
        let _flags = luaur_common::ScopedAllFlags::enter(true);
        let source = br#"local bridge=require('bridge')
local function update(self)
 if self.value == 10 then self.atSource=true end
 if self.atSource and self.value >= 200 and self.last ~= self.value then
  self.last=self.value
  bridge.command('moved', {value=self.value})
 end
end
return function() return {value=0,last=0,atSource=false,update=update} end"#;
        let mut size = 0;
        let code = luaur_compiler::functions::luau_compile::luau_compile(
            source.as_ptr().cast(),
            source.len(),
            ptr::null_mut(),
            &mut size,
        );
        assert!(!code.is_null());
        let payload = [
            vec![0],
            std::slice::from_raw_parts(code.cast::<u8>(), size).to_vec(),
        ]
        .concat();
        unsafe extern "C" {
            fn free(p: *mut c_void);
        }
        free(code.cast());
        let bytes = fixture::command_fixture(&payload);
        let mut scene = LiveScene {
            file: ptr::null_mut(),
            artboard: ptr::null_mut(),
            vm: ptr::null_mut(),
            player: ptr::null_mut(),
        };
        let config = NuxHostCommandImportConfig {
            module_name: view("bridge"),
            ..Default::default()
        };
        let mut imported = ptr::null_mut();
        let status = nux_file_import_trusted_with_host_commands(
            bytes.as_ptr(),
            bytes.len(),
            &NuxRenderCallbacks::default(),
            &config,
            &mut scene.file,
            &mut imported,
        );
        if !imported.is_null() {
            eprintln!(
                "script import: {}",
                String::from_utf8_lossy(&(*imported).message)
            );
            nux_capi_result_free(imported);
        }
        assert_eq!(status, NuxStatus::Ok);
        assert_eq!(
            nux_artboard_instance_new(scene.file, 0, &mut scene.artboard),
            NuxStatus::Ok
        );
        assert_eq!(
            nux_view_model_instance_new_default(scene.artboard, &mut scene.vm),
            NuxStatus::Ok
        );
        assert_eq!(
            nux_artboard_instance_bind_view_model(scene.artboard, scene.vm),
            NuxStatus::Ok
        );
        assert_eq!(
            nux_player_new_default(scene.artboard, &mut scene.player),
            NuxStatus::Ok
        );
        step_ok(scene.player);
        // Keep this command-publication regression at the completed value.
        let root = (&(*scene.player).artboard)
            .instance
            .borrow()
            .native_handle();
        set_live_flight_progress(&root, 1.0);
        let t = view("t1");
        assert_eq!(
            nux_player_step_begin(scene.player, view("B"), &t),
            NuxStatus::Ok
        );
        let row = (&(*scene.vm).instance)
            .list_item_by_property_name_path("screens", 1)
            .unwrap();
        assert!(
            row.borrow_mut()
                .set_number_by_property_name("offset", 321.0)
        );
        assert_eq!(
            nux_player_step_end(scene.player, view("B"), &t),
            NuxStatus::Ok
        );
        let mut result = ptr::null_mut();
        assert_eq!(
            step_status(scene.player, &NuxPlayerStep::default(), &mut result),
            NuxStatus::Ok
        );
        let commands = &(*result).host_commands;
        assert_eq!(
            commands.len(),
            2,
            "status-only live operations lost scripted commands"
        );
        let mut numbers = Vec::new();
        let values = &(*result).host_values;
        for command in commands {
            assert_eq!(&*command.name, b"moved");
            let root = &values[command.root_value_index];
            let value = &values[root.children[0].value_index];
            numbers.push(value.number_value);
        }
        assert_eq!(numbers, [200.0, 321.0]);
        nux_player_step_result_free(result);
        result = ptr::null_mut();
        assert_eq!(
            step_status(scene.player, &NuxPlayerStep::default(), &mut result),
            NuxStatus::Ok
        );
        assert!(
            (&(*result).host_commands).is_empty(),
            "queued commands were delivered twice"
        );
        nux_player_step_result_free(result);
    }
}

#[test]
fn live_moves_remove_outgoing_row_after_back_then_step_round9() {
    use crate::data_binding::*;
    unsafe {
        let scene = LiveScene::from_bytes(&nuxie::live_move::device_fixture::device_fixture(2));
        let mut a = ptr::null_mut();
        let mut b = ptr::null_mut();
        for (index, row) in [(0, &mut a), (1, &mut b)] {
            assert_eq!(
                nux_view_model_instance_list_item_acquire(scene.vm, view("screens"), index, row),
                NuxStatus::Ok
            );
        }
        let write = |row, path: &str, value: &str| {
            let path = std::ffi::CString::new(path).unwrap();
            let value = std::ffi::CString::new(value).unwrap();
            assert_eq!(
                nux_view_model_instance_set_string(row, path.as_ptr(), value.as_ptr()),
                NuxStatus::Ok
            );
        };
        let mutate_list = |kind, index, second_index| {
            let mutation = NuxViewModelMutation {
                kind,
                instance: scene.vm,
                path: view("screens"),
                index,
                second_index,
                ..Default::default()
            };
            let batch = NuxViewModelMutationBatch {
                mutations: &mutation,
                mutation_count: 1,
                ..Default::default()
            };
            let mut result = ptr::null_mut();
            assert_eq!(nux_view_model_mutate(&batch, &mut result), NuxStatus::Ok);
            nux_view_model_mutation_result_free(result);
        };
        write(a, "screen/phase", "active");
        for (step, incoming, incoming_vm, outgoing_vm) in [("t1", "B", b, a), ("back", "A", a, b)] {
            if step == "back" {
                // The SDK's Back move leaves this two-row order [A, B].
                mutate_list(NUX_VIEW_MODEL_MUTATION_KIND_LIST_MOVE, 0, 0);
                step_ok(scene.player);
                assert_eq!(
                    nux_player_set_row_collapsed(scene.player, view("screens"), 0, 0),
                    NuxStatus::Ok
                );
            }
            write(incoming_vm, "screen/phase", "entering");
            write(outgoing_vm, "screen/phase", "exiting");
            write(incoming_vm, "screen/transition", step);
            write(outgoing_vm, "screen/transition", step);
            let name = view(step);
            assert_eq!(
                nux_player_step_begin(scene.player, view(incoming), &name),
                NuxStatus::Ok
            );
            let expected = format!("nx_enter_done:{step}");
            let mut completed = false;
            for frame in 0..=90 {
                let mut result = ptr::null_mut();
                assert_eq!(
                    step_status(
                        scene.player,
                        &NuxPlayerStep {
                            elapsed_seconds: if frame == 0 { 0.0 } else { 1.0 / 60.0 },
                            ..Default::default()
                        },
                        &mut result,
                    ),
                    NuxStatus::Ok,
                    "{step} frame {frame}"
                );
                completed = (&(*result).events)
                    .iter()
                    .any(|event| event.name.as_deref() == Some(expected.as_bytes()));
                nux_player_step_result_free(result);
                if completed {
                    break;
                }
            }
            assert!(completed, "missing authored completion for {step}");
            assert_eq!(
                nux_player_step_end(scene.player, view(incoming), &name),
                NuxStatus::Ok
            );
            write(incoming_vm, "screen/phase", "active");
            write(outgoing_vm, "screen/phase", "hidden");
            if step == "t1" {
                assert_eq!(
                    nux_player_set_row_collapsed(scene.player, view("screens"), 0, 1),
                    NuxStatus::Ok
                );
            }
        }
        // Do not keep an SDK row handle alive across removal. This must exercise
        // retirement, rather than accidentally retaining the removed VM storage.
        nux_view_model_instance_free(b);
        nux_view_model_instance_free(a);
        mutate_list(NUX_VIEW_MODEL_MUTATION_KIND_LIST_REMOVE, 1, 0);
        for frame in 1..=2 {
            let mut result = ptr::null_mut();
            assert_eq!(
                step_status(
                    scene.player,
                    &NuxPlayerStep {
                        elapsed_seconds: 1.0 / 60.0,
                        ..Default::default()
                    },
                    &mut result,
                ),
                NuxStatus::Ok,
                "ordinary step {frame} after removing B following Back"
            );
            nux_player_step_result_free(result);
        }
    }
}

unsafe fn detached_row_write_case(remove_before_write: bool, step_after_remove: bool) {
    use crate::data_binding::*;
    unsafe {
        let scene = LiveScene::from_bytes(&nuxie::live_move::device_fixture::device_fixture(2));
        let mut row = ptr::null_mut();
        let mut live = ptr::null_mut();
        assert_eq!(
            nux_view_model_instance_list_item_acquire(scene.vm, view("screens"), 1, &mut row),
            NuxStatus::Ok
        );
        assert_eq!(
            nux_view_model_instance_list_item_acquire(scene.vm, view("screens"), 0, &mut live),
            NuxStatus::Ok
        );
        // Model the weak backlink left by a retired native Screen VM clone.
        // Keep the SDK row alive, but retire the parent arena before publication.
        use nuxie::runtime::{core::CoreArena, viewmodel::viewmodel_instance::ViewModelInstance};
        let arena = CoreArena::default();
        let retired = arena.insert(ViewModelInstance::default());
        (*row)
            .instance
            .native_handle()
            .with_downcast_mut::<ViewModelInstance, _>(|row| row.add_parent(retired.clone()))
            .unwrap();
        drop(arena);
        assert!(
            retired
                .with_downcast::<ViewModelInstance, _>(|_| ())
                .is_none()
        );
        let remove = NuxViewModelMutation {
            kind: NUX_VIEW_MODEL_MUTATION_KIND_LIST_REMOVE,
            instance: scene.vm,
            path: view("screens"),
            index: 1,
            ..Default::default()
        };
        let write = |instance, value: &'static [u8]| NuxViewModelMutation {
            kind: NUX_VIEW_MODEL_MUTATION_KIND_SET_STRING,
            instance,
            path: view("screen/phase"),
            bytes_value: NuxByteView {
                data: value.as_ptr(),
                len: value.len(),
            },
            ..Default::default()
        };
        let mutate = |mutations: &[NuxViewModelMutation]| {
            let batch = NuxViewModelMutationBatch {
                mutations: mutations.as_ptr(),
                mutation_count: mutations.len(),
                ..Default::default()
            };
            let mut result = ptr::null_mut();
            let status = nux_view_model_mutate(&batch, &mut result);
            nux_view_model_mutation_result_free(result);
            assert_eq!(status, NuxStatus::Ok);
        };
        if remove_before_write {
            mutate(&[remove]);
            if step_after_remove {
                step_ok(scene.player);
                // Deliver a late frame's effects using the runtime capture and
                // publication path, after the removal step has completed.
                let roots = [(*row).instance.clone(), (*live).instance.clone()];
                let transaction =
                    nuxie::RuntimeOwnedViewModelGraphTransaction::begin(&roots, 4096).unwrap();
                let capture = nuxie::RuntimeViewModelChangeCapture::begin().unwrap();
                for (owner, value) in [
                    (&roots[0], b"cached".as_slice()),
                    (&roots[1], b"active".as_slice()),
                ] {
                    assert!(
                        owner
                            .borrow_mut()
                            .set_string_by_property_name_path("screen/phase", value)
                    );
                }
                let effects = transaction
                    .resolve_change_capture_with_owners(&roots, capture)
                    .unwrap();
                assert_eq!(
                    effects.len(),
                    2,
                    "both detached and live effects must be delivered"
                );
                transaction.commit();
                let generation = reserve_view_model_mutation_generation().unwrap();
                for (owner, _) in effects {
                    owner.mark_observable_mutation(generation);
                }
            } else {
                let page_generation = (*scene.vm).instance.observable_mutation_generation();
                mutate(&[write(row, b"cached")]);
                assert_eq!(
                    (*scene.vm).instance.observable_mutation_generation(),
                    page_generation,
                    "detached writes must not notify the former Page parent"
                );
                mutate(&[write(live, b"active")]);
            }
        } else {
            mutate(&[write(row, b"cached"), remove, write(live, b"active")]);
        }
        step_ok(scene.player);
        assert_eq!(
            (*row)
                .instance
                .borrow()
                .string_value_by_property_name_path("screen/phase")
                .unwrap()
                .as_ref(),
            b"cached"
        );
        assert_eq!(
            (*live)
                .instance
                .borrow()
                .string_value_by_property_name_path("screen/phase")
                .unwrap()
                .as_ref(),
            b"active"
        );
        assert!((*row).instance.observable_mutation_generation() > 0);
        let live_value_owner = (*live)
            .instance
            .linked_view_model_by_property_name_path("screen")
            .unwrap();
        let generation = live_value_owner.observable_mutation_generation();
        assert!(generation > 0);
        let parents = live_value_owner
            .native_handle()
            .with_downcast::<ViewModelInstance, _>(ViewModelInstance::parents)
            .unwrap();
        let live_parents: Vec<_> = parents
            .into_iter()
            .filter_map(|parent| {
                RuntimeOwnedViewModelHandle::from_native((*live).file.clone(), parent)
            })
            .collect();
        assert!(
            !live_parents.is_empty(),
            "exercise propagation to live parents"
        );
        for parent in live_parents {
            assert_eq!(
                parent.observable_mutation_generation(),
                generation,
                "live parents still receive the changed child's generation"
            );
        }
        nux_view_model_instance_free(row);
        nux_view_model_instance_free(live);
    }
}

#[test]
fn live_moves_detached_row_write_after_remove() {
    unsafe { detached_row_write_case(true, false) }
}
#[test]
fn live_moves_detached_row_write_and_remove_in_one_batch() {
    unsafe { detached_row_write_case(false, false) }
}
#[test]
fn live_moves_detached_row_late_write_after_previous_step() {
    unsafe { detached_row_write_case(true, true) }
}

#[test]
fn live_moves_native_typing_requires_a_fresh_presented_snapshot() {
    use nuxie::runtime::{focus_data::FocusData, semantic::semantic_data::SemanticData};
    unsafe {
        let scene = LiveScene::from_bytes(&nuxie::live_move::device_fixture::device_fixture(2));
        assert_eq!(nux_player_enable_semantics(scene.player), NuxStatus::Ok);
        let root = (&(*scene.player).artboard)
            .instance
            .borrow()
            .native_handle();
        let rows = nuxie::live_move::screens(&root).unwrap();
        let source = rows["A"].with_artboard(|b| b.nested_artboards()[0].clone());
        let unit = source
            .with(|o| o.as_nested_artboard().unwrap().artboard_instance_default())
            .flatten()
            .unwrap();
        let field_id = unit
            .with_artboard(|b| b.find_all_handles::<SemanticData>())
            .into_iter()
            .find_map(|h| {
                h.with_downcast::<SemanticData, _>(|d| d.existing_semantic_node())
                    .flatten()
                    .filter(|node| node.borrow().role == 6)
            })
            .unwrap()
            .borrow()
            .id();
        let focus = unit.with_artboard(|b| b.find_all_handles::<FocusData>())[0]
            .with_downcast::<FocusData, _>(|f| f.existing_focus_node())
            .flatten()
            .unwrap();
        unit.with_artboard(|b| b.focus_manager())
            .unwrap()
            .with_focus_manager_mut(|m| m.set_focus(focus));
        acknowledge_current(scene.player);
        let mut snapshot = ptr::null_mut();
        assert_eq!(
            nux_player_semantic_snapshot(scene.player, &mut snapshot),
            NuxStatus::Ok
        );
        let read = |capture, expected, text: &str| {
            let mut bytes = [0u8; 64];
            let mut length = 999;
            assert_eq!(
                nux_player_field_string_copy(
                    scene.player,
                    capture,
                    field_id,
                    view("edit"),
                    bytes.as_mut_ptr(),
                    bytes.len(),
                    &mut length
                ),
                expected
            );
            if expected == NuxStatus::Ok {
                assert_eq!(&bytes[..length], text.as_bytes());
            } else {
                assert_eq!(length, 0);
            }
        };
        let mut original_owner = ptr::null_mut();
        assert_eq!(
            nux_player_field_view_model_instance(
                scene.player,
                snapshot,
                field_id,
                view("edit"),
                &mut original_owner
            ),
            NuxStatus::Ok
        );
        for text in ["hello", "hello there"] {
            assert_eq!(
                nux_player_validate_semantic_snapshot(scene.player, snapshot),
                NuxStatus::Ok
            );
            // Android nativeWriter selects all, releases the key, then sends
            // the complete EditText draft through zero-time player steps.
            let inputs = [
                NuxPlayerFocusInput {
                    kind: NUX_PLAYER_FOCUS_KIND_KEY,
                    key_code: 65,
                    modifiers: 8,
                    pressed: 1,
                    ..Default::default()
                },
                NuxPlayerFocusInput {
                    kind: NUX_PLAYER_FOCUS_KIND_KEY,
                    key_code: 65,
                    modifiers: 8,
                    pressed: 0,
                    ..Default::default()
                },
                NuxPlayerFocusInput {
                    kind: NUX_PLAYER_FOCUS_KIND_TEXT,
                    text: view(text),
                    ..Default::default()
                },
            ];
            let mut result = ptr::null_mut();
            assert_eq!(
                step_status(
                    scene.player,
                    &NuxPlayerStep {
                        focus_inputs: inputs.as_ptr(),
                        focus_input_count: inputs.len(),
                        ..Default::default()
                    },
                    &mut result
                ),
                NuxStatus::Ok
            );
            let mut scheduling = NuxPlayerSchedulingInfo::default();
            assert_eq!(
                nux_player_step_result_scheduling(result, &mut scheduling),
                NuxStatus::Ok
            );
            nux_player_step_result_free(result);
            assert!(scheduling.render_required);
            read(snapshot, NuxStatus::HandleMismatch, "");
            let mut premature = ptr::null_mut();
            assert_eq!(
                nux_player_semantic_snapshot(scene.player, &mut premature),
                NuxStatus::HandleMismatch
            );
            assert!(premature.is_null());
            // The renderer must actually present this revision before the host
            // acknowledges it. The test models that presentation receipt here.
            assert_eq!(
                nux_player_acknowledge_presented(scene.player, scheduling.render_revision),
                NuxStatus::Ok
            );
            read(snapshot, NuxStatus::HandleMismatch, "");
            nux_semantic_snapshot_free(snapshot);
            snapshot = ptr::null_mut();
            assert_eq!(
                nux_player_semantic_snapshot(scene.player, &mut snapshot),
                NuxStatus::Ok
            );
            read(snapshot, NuxStatus::Ok, text);
            let mut owner = ptr::null_mut();
            assert_eq!(
                nux_player_field_view_model_instance(
                    scene.player,
                    snapshot,
                    field_id,
                    view("edit"),
                    &mut owner
                ),
                NuxStatus::Ok
            );
            assert_eq!(
                (*owner).instance.instance_identity(),
                (*original_owner).instance.instance_identity()
            );
            nux_view_model_instance_free(owner);
        }
        nux_view_model_instance_free(original_owner);
        nux_semantic_snapshot_free(snapshot);
    }
}

unsafe fn page_player_command_scene() -> LiveScene {
    unsafe { page_player_command_scene_payload(0) }
}
unsafe fn page_player_command_scene_payload(payload_bytes: usize) -> LiveScene {
    unsafe {
        let _flags = luaur_common::ScopedAllFlags::enter(true);
        let source = br#"local bridge=require('bridge')
local function update(self)
 if self.value == 10 then self.atSource=true end
 if self.atSource and self.value >= 200 and self.last ~= self.value then
  self.last=self.value
  bridge.command('moved', {value=self.value})
 end
end
return function() return {value=0,last=0,atSource=false,update=update} end"#;
        let source = if payload_bytes == 0 {
            source.to_vec()
        } else {
            String::from_utf8_lossy(source)
                .replace("self.atSource and ", "")
                .replace(
                    "bridge.command('moved', {value=self.value})",
                    &format!("if self.value < 1012 then local payload=string.rep('x', {payload_bytes}); if self.value == 1011 then payload={{}} for i=1,3000 do payload[i]=false end end bridge.command('moved', payload) end"),
                )
                .into_bytes()
        };
        let mut size = 0;
        let code = luaur_compiler::functions::luau_compile::luau_compile(
            source.as_ptr().cast(),
            source.len(),
            ptr::null_mut(),
            &mut size,
        );
        assert!(!code.is_null());
        let payload = [
            vec![0],
            std::slice::from_raw_parts(code.cast::<u8>(), size).to_vec(),
        ]
        .concat();
        unsafe extern "C" {
            fn free(p: *mut c_void);
        }
        free(code.cast());
        let bytes = if payload_bytes == 0 {
            fixture::command_parity_fixture(&payload)
        } else {
            fixture::command_budget_fixture(&payload)
        };
        let mut scene = LiveScene {
            file: ptr::null_mut(),
            artboard: ptr::null_mut(),
            vm: ptr::null_mut(),
            player: ptr::null_mut(),
        };
        let config = NuxHostCommandImportConfig {
            module_name: view("bridge"),
            ..Default::default()
        };
        let mut imported = ptr::null_mut();
        let status = nux_file_import_trusted_with_host_commands(
            bytes.as_ptr(),
            bytes.len(),
            &NuxRenderCallbacks::default(),
            &config,
            &mut scene.file,
            &mut imported,
        );
        if !imported.is_null() {
            eprintln!(
                "script import: {}",
                String::from_utf8_lossy(&(*imported).message)
            );
            nux_capi_result_free(imported);
        }
        assert_eq!(status, NuxStatus::Ok);
        assert_eq!(
            nux_artboard_instance_new(scene.file, 0, &mut scene.artboard),
            NuxStatus::Ok
        );
        assert_eq!(
            nux_view_model_instance_new_default(scene.artboard, &mut scene.vm),
            NuxStatus::Ok
        );
        assert_eq!(
            nux_artboard_instance_bind_view_model(scene.artboard, scene.vm),
            NuxStatus::Ok
        );
        assert_eq!(
            nux_player_new_default(scene.artboard, &mut scene.player),
            NuxStatus::Ok
        );
        step_ok(scene.player);
        scene
    }
}

unsafe fn page_player_parity_step(
    rust: &LiveScene,
    native: &nuxie::page_player::PagePlayer,
    c: &LiveScene,
    elapsed_seconds: f32,
) -> (usize, usize, usize) {
    unsafe {
        let player = &*rust.player;
        let output = native
            .step(
                &mut player.artboard.instance.borrow_mut(),
                &mut player.instance.borrow_mut(),
                nuxie::page_player::Step {
                    elapsed_seconds,
                    ..Default::default()
                },
                std::slice::from_ref(&(&*rust.vm).instance),
                player
                    .artboard
                    .scripted
                    .as_deref()
                    .map(|s| s as &dyn nuxie::page_player::Effects),
            )
            .unwrap();
        let mut result = ptr::null_mut();
        assert_eq!(
            step_status(
                c.player,
                &NuxPlayerStep {
                    elapsed_seconds,
                    ..Default::default()
                },
                &mut result
            ),
            NuxStatus::Ok
        );
        let result_ref = &mut *result;
        let mut events = output.events;
        // Independent imports have different VM and concrete occurrence IDs.
        // Keep the authored path, repetition indices and source address in the
        // comparison; normalize only identities belonging to these two scenes.
        for event in events.iter_mut().chain(result_ref.events.iter_mut()) {
            event.view_model_instance_id = None;
            event.context = event.context.as_ref().map(|context| {
                nuxie_runtime::StateMachineEventContext::new(
                    context.path().to_vec(),
                    context
                        .occurrence()
                        .iter()
                        .map(|item| nuxie_runtime::RuntimeGeometryHitOccurrence {
                            occurrence_identity: 0,
                            ..*item
                        })
                        .collect(),
                )
            });
        }
        assert_eq!(format!("{events:?}"), format!("{:?}", result_ref.events));
        let (commands, values) = flatten_host_commands(output.commands);
        assert_eq!(
            format!("{commands:?}"),
            format!("{:?}", result_ref.host_commands)
        );
        assert_eq!(
            format!("{values:?}"),
            format!("{:?}", result_ref.host_values)
        );
        let changes = output
            .changes
            .iter()
            .map(|change| (change.property_index, &change.value))
            .collect::<Vec<_>>();
        let c_changes = result_ref
            .view_model_changes
            .iter()
            .map(|change| (change.change.property_index, &change.change.value))
            .collect::<Vec<_>>();
        assert_eq!(changes, c_changes);
        let counts = (events.len(), commands.len(), changes.len());
        nux_player_step_result_free(result);
        counts
    }
}

#[test]
fn page_player_rust_and_c_have_identical_events_changes_and_commands() {
    unsafe {
        let _flags = luaur_common::ScopedAllFlags::enter(true);
        let rust = page_player_command_scene();
        let c = page_player_command_scene();
        let native = nuxie::page_player::PagePlayer::default();
        let t = view("t1");
        native
            .step_begin(
                &mut (&*rust.player).artboard.instance.borrow_mut(),
                "B",
                Some("t1"),
                std::slice::from_ref(&(&*rust.vm).instance),
                (&*rust.player)
                    .artboard
                    .scripted
                    .as_deref()
                    .map(|s| s as &dyn nuxie::page_player::Effects),
            )
            .unwrap();
        assert_eq!(
            nux_player_step_begin(c.player, view("B"), &t),
            NuxStatus::Ok
        );
        for scene in [&rust, &c] {
            let row = (&*scene.vm)
                .instance
                .list_item_by_property_name_path("screens", 1)
                .unwrap();
            assert!(
                row.borrow_mut()
                    .set_number_by_property_name("offset", 321.0)
            );
        }
        native
            .step_end(
                &mut (&*rust.player).artboard.instance.borrow_mut(),
                "B",
                Some("t1"),
                std::slice::from_ref(&(&*rust.vm).instance),
                (&*rust.player)
                    .artboard
                    .scripted
                    .as_deref()
                    .map(|s| s as &dyn nuxie::page_player::Effects),
            )
            .unwrap();
        assert_eq!(nux_player_step_end(c.player, view("B"), &t), NuxStatus::Ok);
        for scene in [&rust, &c] {
            let row = (&*scene.vm)
                .instance
                .list_item_by_property_name_path("screens", 1)
                .unwrap();
            assert!(
                row.borrow_mut()
                    .set_number_by_property_name("offset", 456.0)
            );
        }
        let (_, commands, changes) = page_player_parity_step(&rust, &native, &c, 0.1);
        assert!(commands >= 2);
        assert!(changes > 0);
        assert_eq!(page_player_parity_step(&rust, &native, &c, 0.0), (0, 0, 0));

        let bytes = nuxie::live_move::device_fixture::device_fixture(2);
        let rust = LiveScene::from_bytes(&bytes);
        let c = LiveScene::from_bytes(&bytes);
        let native = nuxie::page_player::PagePlayer::default();
        for scene in [&rust, &c] {
            for (index, phase) in [(0, "exiting"), (1, "entering")] {
                let row = (&*scene.vm)
                    .instance
                    .list_item_by_property_name_path("screens", index)
                    .unwrap();
                assert!(
                    row.borrow_mut()
                        .set_string_by_property_name_path("screen/phase", phase.as_bytes())
                );
                assert!(
                    row.borrow_mut()
                        .set_string_by_property_name_path("screen/transition", b"t1")
                );
            }
        }
        native
            .step_begin(
                &mut (&*rust.player).artboard.instance.borrow_mut(),
                "B",
                Some("t1"),
                std::slice::from_ref(&(&*rust.vm).instance),
                None,
            )
            .unwrap();
        assert_eq!(
            nux_player_step_begin(c.player, view("B"), &t),
            NuxStatus::Ok
        );
        let mut events = 0;
        for _ in 0..65 {
            events += page_player_parity_step(&rust, &native, &c, 1.0 / 60.0).0;
        }
        assert!(
            events > 0,
            "parity sequence must include an authored arrival event"
        );
    }
}

#[test]
fn page_player_content_height_c_query_preserves_render_revision() {
    unsafe {
        let scene = LiveScene::from_bytes(&fixture::fit_content_page_fixture());
        let mut height = 0.0;
        let revision = (&*scene.player).artboard.render_revision.get();
        assert_eq!(
            nux_player_content_height(scene.player, &mut height),
            NuxStatus::Ok
        );
        assert!(height > 160.0 && height < 200.0);
        assert_eq!((&*scene.player).artboard.render_revision.get(), revision);
        let first = height;
        assert!(
            (&*scene.vm)
                .instance
                .move_list_item_by_property_name_path("screens", 0, 1)
        );
        step_ok(scene.player);
        assert_eq!(
            nux_player_content_height(scene.player, &mut height),
            NuxStatus::Ok
        );
        assert!((first - height - 80.0).abs() < 0.01);
        let native_height = (&*scene.player)
            .page_player
            .content_height(&(&*scene.player).artboard.instance.borrow())
            .unwrap();
        assert_eq!(native_height, height);
        assert_eq!(
            nux_player_content_height(scene.player, ptr::null_mut()),
            NuxStatus::NullArgument
        );
        assert_eq!(
            nux_player_set_row_collapsed(scene.player, view("screens"), 1, 1),
            NuxStatus::Ok
        );
        step_ok(scene.player);
        assert_eq!(
            nux_player_content_height(scene.player, &mut height),
            NuxStatus::Ok
        );
        assert_eq!(first, height);
    }
}

#[test]
fn live_moves_blended_midpoint_step_publishes_to_the_shared_owner_observer() {
    unsafe {
        let scene = LiveScene::from_bytes(&fixture::keyed_fixture());
        let root = (&(*scene.player).artboard)
            .instance
            .borrow()
            .native_handle();
        let rows = nuxie::live_move::screens(&root).unwrap();
        let source = rows["A"].with_artboard(|b| b.nested_artboards()[0].clone());
        let unit = source
            .with(|o| o.as_nested_artboard().unwrap().artboard_instance_default())
            .flatten()
            .unwrap();
        let native_model = unit
            .data_context()
            .unwrap()
            .with_context(|c| c.main_view_model_instance())
            .unwrap();
        // As in the exact-owner regression, expose the nested unit's actual
        // model through a public handle and bind another occurrence to it.
        let mut shared = ptr::null_mut();
        assert_eq!(
            nux_view_model_instance_new_schema_default(scene.file, 0, &mut shared),
            NuxStatus::Ok
        );
        (*shared).instance =
            RuntimeOwnedViewModelHandle::from_native((*scene.file).file.clone(), native_model)
                .unwrap();
        (*shared).identity = (*shared).instance.instance_identity();
        let mut observer = ptr::null_mut();
        let mut observer_player = ptr::null_mut();
        assert_eq!(
            nux_artboard_instance_new(scene.file, 3, &mut observer),
            NuxStatus::Ok
        );
        assert_eq!(
            nux_artboard_instance_bind_view_model(observer, shared),
            NuxStatus::Ok
        );
        assert_eq!(
            nux_player_new_default(observer, &mut observer_player),
            NuxStatus::Ok
        );
        let runtime = nuxie::ViewModelInstanceRuntime::new((*shared).instance.native_handle());
        let number = runtime.property_number("prop").unwrap();
        let transition = view("t1");
        assert_eq!(
            nux_player_step_begin(scene.player, view("B"), &transition),
            NuxStatus::Ok
        );
        assert_eq!(number.value(), 10.0);
        let revision = acknowledge_current(observer_player);
        number.value_runtime().clear_changes();
        let mut result = ptr::null_mut();
        assert_eq!(
            nux_player_step(
                scene.player,
                &NuxPlayerStep {
                    elapsed_seconds: 0.5,
                    ..Default::default()
                },
                &mut result
            ),
            NuxStatus::Ok
        );
        nux_player_step_result_free(result);
        assert_eq!(number.value(), 105.0);
        assert_eq!(
            nux_player_acknowledge_presented(observer_player, revision),
            NuxStatus::HandleMismatch
        );
        assert!(number.value_runtime().flush_changes());
        assert!(!number.value_runtime().flush_changes());
        let revision = acknowledge_current(observer_player);
        number.value_runtime().clear_changes();
        assert_eq!(
            nux_player_step_end(scene.player, view("B"), &transition),
            NuxStatus::Ok
        );
        assert_eq!(number.value(), 200.0);
        assert_eq!(
            nux_player_acknowledge_presented(observer_player, revision),
            NuxStatus::HandleMismatch
        );
        assert!(number.value_runtime().flush_changes());
        nux_player_free(observer_player);
        nux_artboard_instance_free(observer);
        nux_view_model_instance_free(shared);
    }
}

#[test]
fn row_evaluation_c_api_repeated_arrival_after_cut_without_ordinary_step() {
    use crate::data_binding::*;
    unsafe {
        let scene = LiveScene::from_bytes(&nuxie::live_move::device_fixture::device_fixture(2));
        let mut a = ptr::null_mut();
        let mut b = ptr::null_mut();
        for (index, row) in [(0, &mut a), (1, &mut b)] {
            assert_eq!(
                nux_view_model_instance_list_item_acquire(scene.vm, view("screens"), index, row),
                NuxStatus::Ok
            );
        }
        let phase = |row, phase: &str, transition: &str| {
            for (path, value) in [("screen/phase", phase), ("screen/transition", transition)] {
                let path = std::ffi::CString::new(path).unwrap();
                let value = std::ffi::CString::new(value).unwrap();
                assert_eq!(
                    nux_view_model_instance_set_string(row, path.as_ptr(), value.as_ptr()),
                    NuxStatus::Ok
                );
            }
        };
        let move_first = || {
            let mutation = NuxViewModelMutation {
                kind: NUX_VIEW_MODEL_MUTATION_KIND_LIST_MOVE,
                instance: scene.vm,
                path: view("screens"),
                index: 0,
                second_index: 1,
                ..Default::default()
            };
            let batch = NuxViewModelMutationBatch {
                mutations: &mutation,
                mutation_count: 1,
                ..Default::default()
            };
            let mut result = ptr::null_mut();
            assert_eq!(nux_view_model_mutate(&batch, &mut result), NuxStatus::Ok);
            nux_view_model_mutation_result_free(result);
        };
        let arrival = |label| {
            let mut events = 0;
            for frame in 0..=75 {
                let mut result = ptr::null_mut();
                assert_eq!(
                    step_status(
                        scene.player,
                        &NuxPlayerStep {
                            elapsed_seconds: if frame == 0 { 0.0 } else { 1.0 / 60.0 },
                            ..Default::default()
                        },
                        &mut result
                    ),
                    NuxStatus::Ok
                );
                events += (&(*result).events)
                    .iter()
                    .filter(|event| event.name.as_deref() == Some(b"nx_enter_done:t1"))
                    .count();
                nux_player_step_result_free(result);
            }
            assert_eq!(events, 1, "{label}: completion missing or repeated");
        };
        let t = view("t1");
        phase(a, "exiting", "t1");
        phase(b, "entering", "t1");
        assert_eq!(
            nux_player_step_begin(scene.player, view("B"), &t),
            NuxStatus::Ok
        );
        arrival("first forward");
        assert_eq!(
            nux_player_step_end(scene.player, view("B"), &t),
            NuxStatus::Ok
        );
        assert_eq!(
            nux_player_set_row_collapsed(scene.player, view("screens"), 0, 1),
            NuxStatus::Ok
        );
        phase(a, "hidden", "");
        phase(b, "active", "");

        phase(b, "exiting", "");
        move_first(); // [B, A]
        assert_eq!(
            nux_player_set_row_collapsed(scene.player, view("screens"), 1, 0),
            NuxStatus::Ok
        );
        phase(a, "entering", "");
        assert_eq!(
            nux_player_step_begin(scene.player, view("A"), ptr::null()),
            NuxStatus::Ok
        );
        assert_eq!(
            nux_player_set_row_collapsed(scene.player, view("screens"), 0, 1),
            NuxStatus::Ok
        );
        phase(b, "hidden", "");
        assert_eq!(
            nux_player_step_end(scene.player, view("A"), ptr::null()),
            NuxStatus::Ok
        );
        phase(a, "active", "");

        move_first(); // [A, B], wake before writing entering/t1 again.
        assert_eq!(
            nux_player_set_row_collapsed(scene.player, view("screens"), 1, 0),
            NuxStatus::Ok
        );
        phase(a, "exiting", "t1");
        phase(b, "entering", "t1");
        assert_eq!(
            nux_player_step_begin(scene.player, view("B"), &t),
            NuxStatus::Ok
        );
        arrival("second forward after cut");
        assert_eq!(
            nux_player_step_end(scene.player, view("B"), &t),
            NuxStatus::Ok
        );
        step_ok(scene.player);
        nux_view_model_instance_free(a);
        nux_view_model_instance_free(b);
    }
}

// The published responses.set adapter, copied from scripted-resource-compiler.
const CHECKED_SCREEN_SCRIPT: &[u8] = br#"local module = require('bridge')
local responses = {}
responses.set = function(field, value) if type(field) ~= "string" or #field > 4096 then error("nuxie.responses.set field must be a form.field string of at most 4096 UTF-8 bytes") end; local form, name = string.match(field, "^([^.]+)%.([^.]+)$"); if not form or string.find(field, "[/%z]") then error("nuxie.responses.set field must name a form and field") end; local answer = "experience/responses:" .. form .. "/" .. name; if value == nil or type(value) == "table" then local options = module.listValues("", answer, "value"); if options == nil then if value ~= nil then error("nuxie.responses.set choices need a several-choice answer") end; return module.set("", answer, nil) end; local chosen = {}; if value ~= nil then local count = 0; for key, option in pairs(value) do if type(option) ~= "string" then error("nuxie.responses.set choices must be option names") end; count = count + 1; chosen[option] = true end; for key in pairs(value) do if type(key) ~= "number" or key % 1 ~= 0 or key < 1 or key > count then error("nuxie.responses.set choices must be a list of option names") end end end; local writes = {}; for index, option in ipairs(options) do local hex = string.gsub(option, ".", function(byte) return string.format("%02x", string.byte(byte)) end); writes[index] = {root = "", path = "experience/responses:" .. form .. "/option:" .. name .. ":" .. hex .. "/picked", value = chosen[option] == true}; chosen[option] = nil end; if next(chosen) ~= nil then error("nuxie.responses.set choices must name options of this answer") end; return module.setAll(writes) end; return module.set("", answer, value) end
return function(context)
 return {init = function() return true end, performAction = function(self)
  local mode = context:viewModel().mode.value
  if mode >= 6 then
   local value = if mode == 6 then 'row-a' elseif mode == 8 then 'refused-title' else 'row-b'
   local ok, rule = responses.set('form.title', value)
   if mode == 8 then assert(not ok and rule == 'allowed-title')
   else assert(ok and rule == nil) end
   if mode == 9 then error('abort row') end
   module.command('row-action', {applied=ok})
   return
  end
  local before = module.listValues('', 'experience/responses:form/field', 'picked')
  if mode == 1 or mode == 5 then assert(before[1] and not before[2]) end
  local chosen = {'y'}
  if mode == 2 then chosen = {'x', 'y'} elseif mode == 3 then chosen = nil end
  if mode == 4 or mode == 5 then
   assert(module.set('', 'experience/responses:form/title', 'changed'))
  end
  local ok, rule = responses.set('form.field', chosen)
  if mode == 2 then
   assert(not ok and rule == 'one-choice')
   local after = module.listValues('', 'experience/responses:form/field', 'picked')
   assert(before[1] == after[1] and before[2] == after[2])
  else assert(ok and rule == nil) end
  if mode == 4 then error('abort row') end
  module.command('row-action', {applied=ok})
 end}
end"#;

unsafe fn checked_screen_scene(standalone: bool) -> LiveScene {
    unsafe { checked_screen_scene_with_script(standalone, CHECKED_SCREEN_SCRIPT, false) }
}

unsafe fn checked_screen_scene_with_script(
    standalone: bool,
    script: &[u8],
    listeners: bool,
) -> LiveScene {
    unsafe {
        let _flags = luaur_common::ScopedAllFlags::enter(true);
        let mut size = 0;
        let code = luaur_compiler::functions::luau_compile::luau_compile(
            script.as_ptr().cast(),
            script.len(),
            ptr::null_mut(),
            &mut size,
        );
        assert!(!code.is_null());
        let mut payload = vec![0];
        payload.extend_from_slice(std::slice::from_raw_parts(code.cast::<u8>(), size));
        unsafe extern "C" {
            fn free(p: *mut c_void);
        }
        free(code.cast());
        let bytes = if listeners {
            fixture::checked_screen_listener_fixture(&payload)
        } else {
            fixture::checked_screen_fixture(&payload)
        };
        let mut scene = LiveScene {
            file: ptr::null_mut(),
            artboard: ptr::null_mut(),
            vm: ptr::null_mut(),
            player: ptr::null_mut(),
        };
        let mut result = ptr::null_mut();
        let status = nux_file_import_trusted_with_host_commands(
            bytes.as_ptr(),
            bytes.len(),
            &NuxRenderCallbacks::default(),
            &NuxHostCommandImportConfig {
                module_name: view("bridge"),
                ..Default::default()
            },
            &mut scene.file,
            &mut result,
        );
        if !result.is_null() {
            eprintln!(
                "checked scene: {}",
                String::from_utf8_lossy(&(*result).message)
            );
            nux_capi_result_free(result);
        }
        assert_eq!(status, NuxStatus::Ok);
        let rule = NuxValueRule {
            model: view("Form"),
            property: view("field"),
            kind: NUX_VALUE_RULE_PICKED_COUNT,
            mode: NUX_VALUE_RULE_REFUSE,
            picked_property: view("picked"),
            bound_flags: 3,
            minimum: 0,
            maximum: 1,
            code: view("one-choice"),
            ..Default::default()
        };
        let titles = [
            view("before"),
            view("changed"),
            view("row-a"),
            view("row-b"),
        ];
        let rules = [
            rule,
            NuxValueRule {
                model: view("Form"),
                property: view("title"),
                kind: NUX_VALUE_RULE_ALLOWED_VALUES,
                mode: NUX_VALUE_RULE_REFUSE,
                values: titles.as_ptr(),
                value_count: titles.len(),
                code: view("allowed-title"),
                ..Default::default()
            },
        ];
        assert_eq!(
            nux_file_set_value_rules(scene.file, rules.as_ptr(), rules.len()),
            NuxStatus::Ok
        );
        assert_eq!(
            nux_artboard_instance_new(scene.file, usize::from(standalone), &mut scene.artboard),
            NuxStatus::Ok
        );
        assert_eq!(
            nux_view_model_instance_new_default(scene.artboard, &mut scene.vm),
            NuxStatus::Ok
        );
        assert_eq!(
            nux_artboard_instance_bind_view_model(scene.artboard, scene.vm),
            NuxStatus::Ok
        );
        assert_eq!(
            nux_player_new_default(scene.artboard, &mut scene.player),
            NuxStatus::Ok
        );
        if listeners {
            assert_eq!(
                nux_player_set_global_view_model(scene.player, view("Page"), scene.vm),
                NuxStatus::Ok
            );
        }
        step_ok(scene.player);
        scene
    }
}
fn checked_screen_row(
    scene: &LiveScene,
    index: usize,
    standalone: bool,
) -> RuntimeOwnedViewModelHandle {
    let root = unsafe { &*scene.vm }.instance.clone();
    if standalone {
        root
    } else {
        root.list_item_by_property_name_path("screens", index)
            .unwrap()
    }
}
fn checked_screen_choices(row: &RuntimeOwnedViewModelHandle) -> ([bool; 2], Vec<u64>) {
    let path = "experience/responses:form/field";
    assert_eq!(row.list_item_count_by_property_name_path(path), Some(2));
    let items = (0..2)
        .map(|i| row.list_item_by_property_name_path(path, i).unwrap())
        .collect::<Vec<_>>();
    (
        [
            items[0]
                .borrow()
                .boolean_value_by_property_name("picked")
                .unwrap(),
            items[1]
                .borrow()
                .boolean_value_by_property_name("picked")
                .unwrap(),
        ],
        items.iter().map(|v| v.instance_identity()).collect(),
    )
}
unsafe fn checked_screen_action(scene: &LiveScene, indices: &[usize], expected: NuxStatus) {
    unsafe {
        let pointers = indices
            .iter()
            .map(|&i| NuxPlayerPointerEvent {
                kind: NUX_PLAYER_POINTER_KIND_DOWN,
                x: 100.0 + i as f32 * 300.0,
                y: 50.0,
                pointer_id: i as i32 + 1,
                timestamp_seconds: 0.0,
            })
            .collect::<Vec<_>>();
        let mut result = ptr::null_mut();
        let status = step_status(
            scene.player,
            &NuxPlayerStep {
                pointers: pointers.as_ptr(),
                pointer_count: pointers.len(),
                ..Default::default()
            },
            &mut result,
        );
        assert_eq!(status, expected);
        if expected == NuxStatus::RuntimeError {
            assert!((&(*result).host_commands).is_empty());
            assert!(
                String::from_utf8_lossy(&(*result).message).contains("abort row"),
                "must reach the later callback failure: {}",
                String::from_utf8_lossy(&(*result).message)
            );
        }
        if expected == NuxStatus::Ok {
            assert_eq!(
                (&(*result).host_commands).len(),
                indices.len(),
                "each action must run"
            );
        }
        nux_player_step_result_free(result);
    }
}
#[test]
fn screen_checked_actions_read_and_write_their_own_rows() {
    unsafe {
        let scene = checked_screen_scene(false);
        let a = checked_screen_row(&scene, 0, false);
        let b = checked_screen_row(&scene, 1, false);
        checked_screen_action(&scene, &[0], NuxStatus::Ok);
        assert_eq!(checked_screen_choices(&a).0, [false, true]);
        assert_eq!(checked_screen_choices(&b).0, [true, false]);
        b.borrow_mut().set_number_by_property_name("mode", 5.0);
        checked_screen_action(&scene, &[1], NuxStatus::Ok);
        assert_eq!(checked_screen_choices(&a).0, [false, true]);
        assert_eq!(checked_screen_choices(&b).0, [false, true]);
        for (row, title) in [(&a, b"before".as_slice()), (&b, b"changed".as_slice())] {
            assert_eq!(
                row.borrow()
                    .string_value_by_property_name_path("experience/responses:form/title")
                    .unwrap()
                    .as_ref(),
                title
            );
        }
    }
}
#[test]
fn screen_checked_rejection_is_atomic_and_nil_keeps_records() {
    unsafe {
        let scene = checked_screen_scene(false);
        let a = checked_screen_row(&scene, 0, false);
        let b = checked_screen_row(&scene, 1, false);
        let initial = checked_screen_choices(&a);
        a.borrow_mut().set_number_by_property_name("mode", 2.0);
        checked_screen_action(&scene, &[0], NuxStatus::Ok);
        assert_eq!(checked_screen_choices(&a), initial);
        a.borrow_mut().set_number_by_property_name("mode", 3.0);
        checked_screen_action(&scene, &[0], NuxStatus::Ok);
        assert_eq!(checked_screen_choices(&a), ([false, false], initial.1));
        assert_eq!(checked_screen_choices(&b).0, [true, false]);
    }
}
#[test]
fn screen_checked_later_error_rolls_back_both_rows() {
    unsafe {
        let scene = checked_screen_scene(false);
        let a = checked_screen_row(&scene, 0, false);
        let b = checked_screen_row(&scene, 1, false);
        b.borrow_mut().set_number_by_property_name("mode", 4.0);
        checked_screen_action(&scene, &[0, 1], NuxStatus::RuntimeError);
        assert_eq!(checked_screen_choices(&a).0, [true, false]);
        assert_eq!(checked_screen_choices(&b).0, [true, false]);
        assert_eq!(
            b.borrow()
                .string_value_by_property_name_path("experience/responses:form/title")
                .unwrap()
                .as_ref(),
            b"before"
        );
    }
}
#[test]
fn screen_checked_standalone_keeps_its_bound_root() {
    unsafe {
        let scene = checked_screen_scene(true);
        let root = checked_screen_row(&scene, 0, true);
        root.borrow_mut().set_number_by_property_name("mode", 5.0);
        checked_screen_action(&scene, &[0], NuxStatus::Ok);
        assert_eq!(checked_screen_choices(&root).0, [false, true]);
        assert_eq!(
            root.borrow()
                .string_value_by_property_name_path("experience/responses:form/title")
                .unwrap()
                .as_ref(),
            b"changed"
        );
    }
}

#[test]
fn screen_checked_scalar_responses_stay_in_their_rows_and_reject_atomically() {
    unsafe {
        let scene = checked_screen_scene(false);
        let a = checked_screen_row(&scene, 0, false);
        let b = checked_screen_row(&scene, 1, false);
        let titles = || {
            [&a, &b].map(|row| {
                row.borrow()
                    .string_value_by_property_name_path("experience/responses:form/title")
                    .unwrap()
                    .to_vec()
            })
        };
        assert_eq!(titles(), [b"before".to_vec(), b"before".to_vec()]);
        let choices = [checked_screen_choices(&a), checked_screen_choices(&b)];
        a.borrow_mut().set_number_by_property_name("mode", 6.0);
        checked_screen_action(&scene, &[0], NuxStatus::Ok);
        assert_eq!(titles(), [b"row-a".to_vec(), b"before".to_vec()]);
        b.borrow_mut().set_number_by_property_name("mode", 7.0);
        checked_screen_action(&scene, &[1], NuxStatus::Ok);
        let accepted = [b"row-a".to_vec(), b"row-b".to_vec()];
        assert_eq!(titles(), accepted);
        for (index, row) in [&a, &b].into_iter().enumerate() {
            row.borrow_mut().set_number_by_property_name("mode", 8.0);
            checked_screen_action(&scene, &[index], NuxStatus::Ok);
            assert_eq!(
                titles(),
                accepted,
                "refusal must preserve both scalar answers"
            );
        }
        assert_eq!(
            [checked_screen_choices(&a), checked_screen_choices(&b)],
            choices
        );
    }
}

#[test]
fn screen_checked_scalar_later_error_rolls_back_both_rows() {
    unsafe {
        let scene = checked_screen_scene(false);
        let a = checked_screen_row(&scene, 0, false);
        let b = checked_screen_row(&scene, 1, false);
        a.borrow_mut().set_number_by_property_name("mode", 6.0);
        b.borrow_mut().set_number_by_property_name("mode", 9.0);
        checked_screen_action(&scene, &[0, 1], NuxStatus::RuntimeError);
        for row in [&a, &b] {
            assert_eq!(
                row.borrow()
                    .string_value_by_property_name_path("experience/responses:form/title")
                    .unwrap()
                    .as_ref(),
                b"before",
                "later callback failure must roll back both scalar answers"
            );
        }
    }
}

fn queue_budget_event(scene: &LiveScene, name: &str) {
    use nuxie_runtime::source::{
        animation::nested_state_machine::NestedStateMachine,
        artboard_component_list::ArtboardComponentList,
        event::Event,
        generated::{component_base::ComponentBase, core_registry::CoreRegistry},
        nested_artboard::NestedArtboard,
    };
    let root = unsafe { &*scene.player }
        .artboard
        .instance
        .borrow()
        .native_handle();
    let list = root.with_artboard(|b| b.find_handle::<ArtboardComponentList>("screens").unwrap());
    let row = list
        .with_downcast::<ArtboardComponentList, _>(|l| l.artboard_instance(1))
        .flatten()
        .unwrap();
    for host in row.with_artboard(|b| b.find_all_handles::<NestedArtboard>()) {
        let Some(unit) = host
            .with(|o| o.as_nested_artboard().unwrap().artboard_instance_default())
            .flatten()
        else {
            continue;
        };
        let Some(event) = unit.with_artboard(|b| b.find_all_handles::<Event>().into_iter().next())
        else {
            continue;
        };
        assert!(CoreRegistry::set_string_handle(
            &event,
            i32::from(ComponentBase::NAME_PROPERTY_KEY),
            name.to_owned()
        ));
        let machine = host
            .with(|o| {
                o.as_nested_artboard()
                    .unwrap()
                    .nested_animations()
                    .iter()
                    .find_map(|a| {
                        a.with_downcast::<NestedStateMachine, _>(
                            NestedStateMachine::state_machine_instance,
                        )
                        .flatten()
                    })
            })
            .flatten()
            .unwrap();
        machine.with_instance_mut(|m| m.report_event(event, 0.0));
        return;
    }
    panic!("landed unit must have its authored budget event");
}

#[test]
fn page_player_review_deferred_budget_fault_poisoned() {
    unsafe {
        // String overflow exercises shared Rust admission; node-heavy overflow
        // fits the Rust payload but exceeds the C flattened result budget.
        // The third case exceeds only the combined event/command budget;
        // the fourth exceeds the event projection budget before effects finish.
        for value in [1010.0, 1011.0, 1012.0, 1013.0] {
            let scene = page_player_command_scene_payload(800 * 1024);
            let t = view("t1");
            assert_eq!(
                nux_player_step_begin(scene.player, view("B"), &t),
                NuxStatus::Ok
            );
            assert_eq!(
                nux_player_step_end(scene.player, view("B"), &t),
                NuxStatus::Ok
            );
            step_ok(scene.player);
            queue_budget_event(&scene, "admitted");
            assert_eq!(
                nux_player_set_row_collapsed(scene.player, view("screens"), 0, 1),
                NuxStatus::Ok
            );
            let row = &(&*scene.vm).instance;
            for index in 0..10 {
                row.borrow_mut()
                    .set_number_by_property_name("counter", 1000.0 + index as f32);
                assert_eq!(
                    nux_player_set_row_collapsed(scene.player, view("screens"), 0, 0),
                    NuxStatus::Ok
                );
            }
            row.borrow_mut()
                .set_number_by_property_name("counter", value);
            if value >= 1012.0 {
                let bytes = if value == 1012.0 {
                    200 * 1024
                } else {
                    9 * 1024 * 1024
                };
                queue_budget_event(&scene, &"e".repeat(bytes));
            }
            assert_eq!(
                nux_player_set_row_collapsed(
                    scene.player,
                    view("screens"),
                    0,
                    u8::from(value >= 1012.0)
                ),
                NuxStatus::LimitExceeded,
                "overflow variant {value} must refuse during status admission"
            );
            assert!((&*scene.player).artboard.poisoned.get());
            let root = (&*scene.player).artboard.instance.borrow().native_handle();
            assert!(nuxie::page_player::is_poisoned(&root));
            let before = root.with_artboard(|b| b.change_serial());
            let mut result = ptr::null_mut();
            assert_eq!(
                step_status(scene.player, &NuxPlayerStep::default(), &mut result),
                NuxStatus::RuntimeError,
                "a late budget failure is a poisoned fault"
            );
            if !result.is_null() {
                nux_player_step_result_free(result);
            }
            assert_eq!(
                nux_player_step_begin(scene.player, view("B"), &t),
                NuxStatus::RuntimeError
            );
            assert_eq!(
                root.with_artboard(|b| b.change_serial()),
                before,
                "poisoned calls cannot advance native state"
            );
        }
    }
}

unsafe fn screen_checked_property_listener_scopes(error_in_b: bool) {
    unsafe {
        let script = br#"local module = require('bridge')
local counter
local seen = {}
return function(context)
 -- Share one property userdata as well as one native global property. This
 -- makes both registrations dispatch synchronously through the writer's facade.
 counter = counter or context:globalViewModel('Page').counter
 local mode = context:viewModel().mode.value
 local isA = mode == 6
 local registered = false
 return {init = function() return true end, performAction = function()
  if not registered then
   registered = true
   counter:addListener(function()
   seen[mode] = true
   context:viewModel().mode.value += 10
   local before = module.listValues('', 'experience/responses:form/field', 'picked')
   assert(before[1] == isA and before[2] ~= isA, 'listener must read its own answer')
   assert(module.set('', 'experience/responses:form/title', if isA then 'row-a' else 'row-b'))
   if not isA and ERROR_IN_B then error('listener B failed') end
  end)
  module.command('row-action', {})
  return
  end
  counter.value += 1
  assert(seen[6] and seen[7], "both listeners must fire synchronously")
  -- Both normal return and B's swallowed callback error must restore A.
  assert(module.set('', 'experience/responses:form/title', 'changed'))
  module.command('row-action', {})
 end}
end"#;
        let script = String::from_utf8(script.to_vec())
            .unwrap()
            .replace("ERROR_IN_B", if error_in_b { "true" } else { "false" });
        let scene = checked_screen_scene_with_script(false, script.as_bytes(), true);
        assert_eq!(
            nux_file_set_value_rules(scene.file, ptr::null(), 0),
            NuxStatus::Ok
        );
        let a = checked_screen_row(&scene, 0, false);
        let b = checked_screen_row(&scene, 1, false);
        for (index, picked) in [false, true].into_iter().enumerate() {
            b.list_item_by_property_name_path("experience/responses:form/field", index)
                .unwrap()
                .borrow_mut()
                .set_boolean_by_property_name("picked", picked);
        }
        // Exercise synchronous VM notifications through the shared Rust player.
        // A subscription checkpoint buffers dirt until publication; omit that
        // optional host subscription here so callbacks execute in A's setter.
        // The checked extension still supplies the real page root. Atomic
        // rules/rollback are covered by the ordinary C checked-script tests.
        let action = |indices: &[usize]| {
            let player = &*scene.player;
            let root = (&*scene.vm).instance.clone();
            let effects = player.artboard.scripted.as_deref().unwrap();
            let mut artboard = player.artboard.instance.borrow_mut();
            let mut playback = player.instance.borrow_mut();
            let mut pending = player
                .page_player
                .operation(
                    &mut artboard,
                    &mut playback,
                    nuxie::page_player::Step {
                        pointers: indices
                            .iter()
                            .map(|&i| nuxie::page_player::Pointer {
                                kind: nuxie::page_player::PointerKind::Down,
                                x: 100.0 + i as f32 * 300.0,
                                y: 50.0,
                                pointer_id: i as i32 + 1,
                                timestamp_seconds: 0.0,
                            })
                            .collect(),
                        ..Default::default()
                    },
                    None,
                    &[],
                    None,
                    None,
                    false,
                    Some(effects),
                    |operation| value_rule_script::enter(player, Some(&root), operation),
                )
                .expect("synchronous checked listener step");
            assert_eq!(pending.output.take().unwrap().commands.len(), indices.len());
            pending.commit();
        };
        // Listener actions are instantiated lazily. Register both before firing.
        action(&[0, 1]);
        action(&[0]);
        assert_eq!(
            a.borrow().number_value_by_property_name("mode"),
            Some(16.0),
            "A listener must run"
        );
        assert_eq!(
            b.borrow().number_value_by_property_name("mode"),
            Some(17.0),
            "B listener must run"
        );
        assert_eq!(checked_screen_choices(&a).0, [true, false]);
        assert_eq!(checked_screen_choices(&b).0, [false, true]);
        for (row, title) in [(&a, b"changed".as_slice()), (&b, b"row-b".as_slice())] {
            assert_eq!(
                row.borrow()
                    .string_value_by_property_name_path("experience/responses:form/title")
                    .unwrap()
                    .as_ref(),
                title
            );
        }
    }
}

#[test]
fn screen_checked_property_listeners_use_registering_screen() {
    unsafe { screen_checked_property_listener_scopes(false) }
}

#[test]
fn screen_checked_property_listener_error_restores_writer_scope() {
    unsafe { screen_checked_property_listener_scopes(true) }
}

unsafe fn screen_checked_deferred_scope(kind: &str) {
    unsafe {
        let (register, fire) = match kind {
            "then" => ("p:andThen(finished)", "resolves[i]()"),
            "catch" => ("p:catch(finished)", "rejects[i]('reason')"),
            "finally" => ("p:finally(finished)", "resolves[i]()"),
            "cancel" => ("p:onCancel(finished)", "pending[i]:cancel()"),
            "executor_cancel" => ("", "pending[i]:cancel()"),
            "async" => (
                "async(function() await(p); finished() end)",
                "resolves[i]()",
            ),
            _ => unreachable!(),
        };
        let source = r#"local module = require('bridge')
local pending, resolves, rejects = {}, {}, {}
return function(context)
 local mode = context:viewModel().mode.value
 local index, isA = mode - 5, mode == 6
 local registered = false
 return {init = function() return true end, performAction = function()
  if not registered then
   registered = true
   local function finished()
    context:viewModel().mode.value += 10
    local before = module.listValues('', 'experience/responses:form/field', 'picked')
    assert(before[1] and not before[2], 'deferred callback must read its own answer')
    assert(module.set('', 'experience/responses:form/title', if isA then 'row-a' else 'row-b'))
    assert(module.setAll({
     {root='', path='experience/responses:form/option:field:78/picked', value=false},
     {root='', path='experience/responses:form/option:field:79/picked', value=isA}
    }))
    if not isA then error('deferred B failed') end
   end
   local p = Promise.new(function(resolve, reject, onCancel)
    resolves[index], rejects[index] = resolve, reject
    EXECUTOR_CANCEL
   end)
   pending[index] = p
   REGISTER
   module.command('row-action', {})
   return
  end
  for i=1,2 do FIRE end
  assert(module.set('', 'experience/responses:form/title', 'changed'))
  module.command('row-action', {})
 end}
end"#;
        let source = source
            .replace(
                "EXECUTOR_CANCEL",
                if kind == "executor_cancel" {
                    "onCancel(finished)"
                } else {
                    ""
                },
            )
            .replace("REGISTER", register)
            .replace("FIRE", fire);
        let scene = checked_screen_scene_with_script(false, source.as_bytes(), true);
        let a = checked_screen_row(&scene, 0, false);
        let b = checked_screen_row(&scene, 1, false);
        checked_screen_action(&scene, &[0, 1], NuxStatus::Ok);
        checked_screen_action(&scene, &[0], NuxStatus::Ok);
        assert_eq!(
            a.borrow().number_value_by_property_name("mode"),
            Some(16.0),
            "{kind}: A callback must run"
        );
        assert_eq!(
            b.borrow().number_value_by_property_name("mode"),
            Some(17.0),
            "{kind}: B callback must run"
        );
        assert_eq!(checked_screen_choices(&a).0, [false, true], "{kind}");
        assert_eq!(checked_screen_choices(&b).0, [false, false], "{kind}");
        for (row, title) in [(&a, b"changed".as_slice()), (&b, b"row-b".as_slice())] {
            assert_eq!(
                row.borrow()
                    .string_value_by_property_name_path("experience/responses:form/title")
                    .unwrap()
                    .as_ref(),
                title,
                "{kind}: callback/error must restore A"
            );
        }
    }
}
#[test]
fn screen_checked_deferred_then_scope() {
    unsafe { screen_checked_deferred_scope("then") }
}
#[test]
fn screen_checked_deferred_catch_scope() {
    unsafe { screen_checked_deferred_scope("catch") }
}
#[test]
fn screen_checked_deferred_finally_scope() {
    unsafe { screen_checked_deferred_scope("finally") }
}
#[test]
fn screen_checked_deferred_cancel_scope() {
    unsafe { screen_checked_deferred_scope("cancel") }
}
#[test]
fn screen_checked_deferred_executor_cancel_scope() {
    unsafe { screen_checked_deferred_scope("executor_cancel") }
}
#[test]
fn screen_checked_deferred_async_scope() {
    unsafe { screen_checked_deferred_scope("async") }
}

#[test]
fn screen_checked_method_lookup_uses_own_screen() {
    unsafe {
        let script = br#"local module = require('bridge')
return function(context)
 local mode = context:viewModel().mode.value
 return setmetatable({init = function() return true end}, {
  __index = function(self, key)
   if key == 'performAction' then
    local before = module.listValues('', 'experience/responses:form/field', 'picked')
    assert(before[1] and not before[2])
    assert(module.set('', 'experience/responses:form/title', if mode == 6 then 'row-a' else 'row-b'))
    return function() module.command('row-action', {}) end
   end
  end
 })
end"#;
        let scene = checked_screen_scene_with_script(false, script, true);
        checked_screen_action(&scene, &[0, 1], NuxStatus::Ok);
        for (index, title) in [b"row-a".as_slice(), b"row-b".as_slice()]
            .into_iter()
            .enumerate()
        {
            assert_eq!(
                checked_screen_row(&scene, index, false)
                    .borrow()
                    .string_value_by_property_name_path("experience/responses:form/title")
                    .unwrap()
                    .as_ref(),
                title
            );
        }
    }
}

#[test]
fn page_player_status_capacity_refusals_are_preflight() {
    struct RestoreGeneration(u64);
    impl Drop for RestoreGeneration {
        fn drop(&mut self) {
            VIEW_MODEL_MUTATION_GENERATION.with(|g| g.set(self.0));
        }
    }
    unsafe {
        for generation_limit in [false, true] {
            let scene = LiveScene::new();
            step_ok(scene.player);
            let root = (&*scene.player).artboard.instance.borrow().native_handle();
            let before = root.with_artboard(|b| b.change_serial());
            let revision = (&*scene.player).artboard.render_revision.get();
            let t = view("t1");
            if generation_limit {
                let saved = VIEW_MODEL_MUTATION_GENERATION.with(|g| g.replace(u64::MAX));
                let restore = RestoreGeneration(saved);
                assert_eq!(
                    nux_player_step_begin(scene.player, view("B"), &t),
                    NuxStatus::LimitExceeded
                );
                drop(restore);
            } else {
                (&*scene.player).artboard.render_revision.set(u64::MAX);
                let status = nux_player_step_begin(scene.player, view("B"), &t);
                (&*scene.player).artboard.render_revision.set(revision);
                assert_eq!(status, NuxStatus::LimitExceeded);
            }
            assert!(!(&*scene.player).artboard.poisoned.get());
            assert!(!nuxie::page_player::is_poisoned(&root));
            assert_eq!(root.with_artboard(|b| b.change_serial()), before);
            assert!(
                nuxie::live_move::LiveMoves::default()
                    .flights(&root)
                    .is_empty()
            );
            step_ok(scene.player);
            assert!(
                nuxie::live_move::LiveMoves::default()
                    .flights(&root)
                    .is_empty()
            );
            assert_eq!(
                nux_player_step_begin(scene.player, view("B"), &t),
                NuxStatus::Ok
            );
            assert_eq!(
                nux_player_step_end(scene.player, view("B"), &t),
                NuxStatus::Ok
            );
            scene.close();
        }
    }
}
