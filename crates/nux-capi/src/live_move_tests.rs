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
        unsafe { Self::open(bytes, None) }
    }
    /// As `from_bytes`, playing the artboard named `name`.
    unsafe fn from_bytes_named(bytes: &[u8], name: &str) -> Self {
        unsafe { Self::open(bytes, Some(name)) }
    }
    unsafe fn open(bytes: &[u8], name: Option<&str>) -> Self {
        unsafe {
            let mut scene = Self {
                file: ptr::null_mut(),
                artboard: ptr::null_mut(),
                vm: ptr::null_mut(),
                player: ptr::null_mut(),
            };
            assert_eq!(import_fixture(bytes, &mut scene.file), NuxStatus::Ok);
            let created = match name {
                Some(name) => {
                    nux_artboard_instance_new_named(scene.file, view(name), &mut scene.artboard)
                }
                None => nux_artboard_instance_new(scene.file, 0, &mut scene.artboard),
            };
            assert_eq!(created, NuxStatus::Ok);
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
    set_flight_progress(root, 2, progress);
}
/// Writes `progress` to Screen B's flight record `flight_key`.
fn set_flight_progress(
    root: &nuxie::RuntimeArtboardInstanceHandle,
    flight_key: u32,
    progress: f32,
) {
    let rows = nuxie::live_move::screens(root).unwrap();
    for record in
        rows["B"].with_artboard(|b| b.find_all_handles::<nuxie::live_move::records::LiveFlight>())
    {
        let is_flight = record
            .with_downcast::<nuxie::live_move::records::LiveFlight, _>(|r| {
                r.values.key == flight_key
            })
            .unwrap();
        if is_flight {
            assert!(
                nuxie::runtime::generated::core_registry::CoreRegistry::set_double_handle(
                    &record, 62033, progress
                )
            );
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
            before + 1,
            "paint list edit left the enclosing index stale"
        );
        step_ok(scene.player);
        let row = list
            .with_downcast::<ArtboardComponentList, _>(|l| l.artboard_instance(0))
            .flatten()
            .unwrap();
        let slot = row.with_artboard(|b| b.find_all_handles::<NestedArtboard>())[0].clone();
        let old = slot
            .with(|o| o.nested_artboard_instance_handle())
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
            .with(|o| o.nested_artboard_instance_handle())
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
        assert_eq!(ids.len(), before + 1);
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
        let bytes = fixture::command_parity_fixture(&payload);
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

#[path = "form_script_tests.rs"]
mod form_script_tests;

/// The id of the one video named `name` in `player`.
unsafe fn video_named(player: *mut NuxPlayer, name: &str) -> usize {
    unsafe extern "C" fn collect(data: *mut c_void, info: *const NuxVideoInfo) {
        let info = unsafe { &*info };
        let name = if info.component_name.len == 0 {
            Vec::new()
        } else {
            unsafe {
                std::slice::from_raw_parts(
                    info.component_name.data.cast::<u8>(),
                    info.component_name.len,
                )
            }
            .to_vec()
        };
        unsafe { &mut *data.cast::<Vec<(usize, Vec<u8>)>>() }.push((info.component_id, name));
    }
    unsafe {
        let mut videos: Vec<(usize, Vec<u8>)> = Vec::new();
        assert_eq!(
            nux_player_visit_videos(player, Some(collect), ptr::from_mut(&mut videos).cast()),
            NuxStatus::Ok
        );
        let named: Vec<_> = videos
            .iter()
            .filter(|(_, video)| video.as_slice() == name.as_bytes())
            .map(|(id, _)| *id)
            .collect();
        assert_eq!(named.len(), 1, "one video named {name}");
        named[0]
    }
}
/// Whether `player` reads the video `id` as visible, with a viewport that
/// holds the whole page.
unsafe fn video_visible(player: *mut NuxPlayer, id: usize) -> u32 {
    let mut visible = u32::MAX;
    assert_eq!(
        unsafe {
            nux_player_video_is_visible(
                player,
                id,
                -10000.0,
                -10000.0,
                10000.0,
                10000.0,
                &mut visible,
            )
        },
        NuxStatus::Ok
    );
    visible
}
/// The page's root as the player holds it.
unsafe fn page_root(scene: &LiveScene) -> nuxie::RuntimeArtboardInstanceHandle {
    unsafe {
        (&(*scene.player).artboard)
            .instance
            .borrow()
            .native_handle()
    }
}
/// Writes `value` to the own opacity of the Screen `name`'s row, as a keyed
/// `ScreenOpacity` does.
fn set_screen_opacity(root: &nuxie::RuntimeArtboardInstanceHandle, name: &str, value: f32) {
    use nuxie::runtime::generated::{
        core_registry::CoreRegistry, world_transform_component_base::WorldTransformComponentBase,
    };
    let rows = nuxie::live_move::screens(root).unwrap();
    assert!(CoreRegistry::set_double_handle(
        &rows[name].core_handle(),
        i32::from(WorldTransformComponentBase::OPACITY_PROPERTY_KEY),
        value
    ));
}
/// Unit c's flight key in `css_flight_video_fixture`.
const UNIT_C_FLIGHT: u32 = 13;
/// Design T21's take-off: `css_flight_video_fixture` played on its page, and
/// unit c taking off alone from A to B by step "t2".
unsafe fn video_take_off() -> (LiveScene, nuxie::RuntimeArtboardInstanceHandle) {
    unsafe {
        let scene = LiveScene::from_bytes_named(&fixture::css_flight_video_fixture(), "Page");
        let root = page_root(&scene);
        let t = view("t2");
        assert_eq!(
            nux_player_step_begin(scene.player, view("B"), &t),
            NuxStatus::Ok
        );
        (scene, root)
    }
}

/// Design T21: a video in a flight draws at its own strength, so it reads
/// visible with its Screen at 0: at take-off, and all through a `step-end`
/// step, which holds the flight at progress 0 until it jumps to 1 at the end.
/// A walk that reads the Screen's 0 as hiding everything on it reads 0.
#[test]
fn a_video_in_a_flight_stays_visible_with_its_screen_at_zero() {
    unsafe {
        let (scene, root) = video_take_off();
        assert_eq!(
            video_visible(scene.player, video_named(scene.player, "clip")),
            1,
            "with Screen B at 1"
        );
        set_screen_opacity(&root, "B", 0.0);
        assert_eq!(
            video_visible(scene.player, video_named(scene.player, "clip")),
            1,
            "at take-off, with Screen B at 0"
        );
        for frame in 0..3 {
            set_flight_progress(&root, UNIT_C_FLIGHT, 0.0);
            step_ok(scene.player);
            assert_eq!(
                video_visible(scene.player, video_named(scene.player, "clip")),
                1,
                "held at progress 0, frame {frame}"
            );
        }
        set_flight_progress(&root, UNIT_C_FLIGHT, 1.0);
        step_ok(scene.player);
        assert_eq!(
            video_visible(scene.player, video_named(scene.player, "clip")),
            1,
            "at the step's end, progress 1"
        );
        let t = view("t2");
        assert_eq!(
            nux_player_step_end(scene.player, view("B"), &t),
            NuxStatus::Ok
        );
        scene.close();
    }
}

/// Design T21, a guard: a plain video on Screen B, in its picture, reads
/// hidden with the Screen at 0, while a flight is in the air (decision 250).
#[test]
fn a_plain_video_on_a_screen_at_zero_reads_hidden() {
    unsafe {
        let (scene, root) = video_take_off();
        let plain = video_named(scene.player, "plain");
        assert_eq!(video_visible(scene.player, plain), 1, "with Screen B at 1");
        set_screen_opacity(&root, "B", 0.0);
        assert_eq!(video_visible(scene.player, plain), 0, "with Screen B at 0");
        scene.close();
    }
}

/// Design T21: an action on a flight stays eligible with its Screen at 0, as
/// the flight draws at its own strength. Unit c's box on A carries a button's
/// semantics, and flies from A to B. A walk that reads the Screen's
/// 0 as hiding everything on it refuses the action.
#[test]
fn an_action_on_a_flight_stays_eligible_with_its_screen_at_zero() {
    use nuxie::runtime::semantic::{semantic_data::SemanticData, semantic_node::SemanticNode};
    unsafe {
        let scene = LiveScene::from_bytes_named(&fixture::css_flight_video_fixture(), "Page");
        let root = page_root(&scene);
        let rows = nuxie::live_move::screens(&root).unwrap();
        let home = rows["A"].clone();
        assert_eq!(nux_player_enable_semantics(scene.player), NuxStatus::Ok);
        let t = view("t2");
        assert_eq!(
            nux_player_step_begin(scene.player, view("B"), &t),
            NuxStatus::Ok
        );
        let button = || {
            home.with_artboard(|b| b.find_all_handles::<SemanticData>())
                .iter()
                .find_map(|h| {
                    h.with_downcast::<SemanticData, _>(|d| d.existing_semantic_node())
                        .flatten()
                })
                .expect("the button's node")
        };
        assert!(
            SemanticNode::is_action_eligible(&button()),
            "with Screen B at 1"
        );
        set_screen_opacity(&root, "B", 0.0);
        assert!(
            SemanticNode::is_action_eligible(&button()),
            "with Screen B at 0"
        );
        scene.close();
    }
}

/// Design T21 below 0, where an easing that overshoots can take a Screen's
/// fade. The draw reads any own opacity at or below 0 as a fade of 0, so the
/// flight still draws at its own strength, and the walk must read the same 0.
/// A walk that asks only at exactly 0 reads the flight's video as hidden.
#[test]
fn a_video_in_a_flight_stays_visible_with_its_screen_below_zero() {
    unsafe {
        let (scene, root) = video_take_off();
        let clip = video_named(scene.player, "clip");
        let plain = video_named(scene.player, "plain");
        for fade in [-0.25, f32::NEG_INFINITY] {
            set_screen_opacity(&root, "B", fade);
            assert_eq!(
                video_visible(scene.player, clip),
                1,
                "the flight's video, with Screen B at {fade}"
            );
            assert_eq!(
                video_visible(scene.player, plain),
                0,
                "the plain video, with Screen B at {fade}"
            );
        }
        scene.close();
    }
}

/// Design T21's marked videos: `PaintCase::MarkedVideos` played on its page,
/// the red taking off alone from A to B by step "go", one frame on so B's
/// entrance holds its marks and the red's flight draws under "cover", then
/// Screen B at 0.
unsafe fn marked_videos_at_zero() -> LiveScene {
    unsafe {
        let scene = LiveScene::from_bytes_named(
            &fixture::css_paint_fixture_in_place(fixture::PaintCase::MarkedVideos).unwrap(),
            "Page",
        );
        let root = page_root(&scene);
        let go = view("go");
        assert_eq!(
            nux_player_step_begin(scene.player, view("B"), &go),
            NuxStatus::Ok
        );
        step_ok(scene.player);
        set_screen_opacity(&root, "B", 0.0);
        scene
    }
}

/// Design T21, the marks: with Screen B at 0, a video marked 2 draws at its
/// own strength wherever it is, so it reads visible, below the red's flight
/// here, and so does one marked 3, which draws above every row
/// (UNIV-6210). One marked 1 above the flight draws at its own strength too,
/// and reads visible. One marked 1 below the flight draws under the Screen's
/// fade, and reads hidden. A walk that drops the split reads 0 on the first
/// two, and one on render opacity reads 1 on the third.
#[test]
fn a_marked_video_reads_as_its_mark_draws_with_its_screen_at_zero() {
    unsafe {
        let scene = marked_videos_at_zero();
        for (name, visible, what) in [
            ("low two", 1, "marked 2, below the flight"),
            ("lifted", 1, "marked 3, above every row"),
            ("high one", 1, "marked 1, above the flight"),
            ("low one", 0, "marked 1, below the flight"),
        ] {
            assert_eq!(
                video_visible(scene.player, video_named(scene.player, name)),
                visible,
                "{name}: {what}, with Screen B at 0"
            );
        }
        scene.close();
    }
}

/// UNIV-6210: the `Above` pass leaves out the box clips above a mark of 3,
/// so the visibility walk does too. "lifted out", marked 3, sits wholly
/// outside "clipper", the box that holds it and clips both axes, and with
/// Screen B at 0 it draws there and reads visible. A walk that keeps the
/// clip reads it hidden, and its decoder would stop while it shows.
#[test]
fn a_video_marked_3_outside_its_clipping_box_reads_visible() {
    unsafe {
        let scene = marked_videos_at_zero();
        assert_eq!(
            video_visible(scene.player, video_named(scene.player, "lifted out")),
            1,
            "lifted out: marked 3, outside the box that clips it, with Screen B at 0"
        );
        scene.close();
    }
}

/// Design T21 in a hand-built file with masks, as the compiler writes none:
/// a mask run stays whole, so a mark inside it doesn't count, and a video in
/// it reads as what the mask sits on and what holds that. With Screen B at
/// 0, one marked 2 inside a run below the red's flight reads hidden, and
/// one marked 1 inside a run above the flight reads hidden. One in a run
/// under a box marked 2 reads visible. A walk on the mark alone reads 1 on
/// the first two, and one that reads everything in a run as mark 0 reads 0
/// on the third.
#[test]
fn a_video_in_a_mask_run_reads_as_the_marks_around_its_run() {
    unsafe {
        let scene = marked_videos_at_zero();
        for (name, visible, what) in [
            (
                "masked low two",
                0,
                "marked 2 inside a run below the flight",
            ),
            (
                "masked high one",
                0,
                "marked 1 inside a run above the flight",
            ),
            ("run video", 1, "in a run under a box marked 2"),
        ] {
            assert_eq!(
                video_visible(scene.player, video_named(scene.player, name)),
                visible,
                "{name}: {what}, with Screen B at 0"
            );
        }
        scene.close();
    }
}

/// `InPlaceFixture::Field` played on its page: a card flying in place from
/// A to B by step "t1", each copy holding a field "edit a" or "edit b", a
/// button and a video "clip a" or "clip b"; B also holds a plain video
/// "plain". B's card rests 100 right and 60 down in its Screen, so its
/// copy moves as it flies; each copy's field and button sit at its card's
/// corner, so half way both copies' are midway between where they rest.
unsafe fn in_place_field_page() -> (LiveScene, nuxie::RuntimeArtboardInstanceHandle) {
    unsafe {
        let scene = LiveScene::from_bytes_named(
            &fixture::in_place_fixture(fixture::InPlaceFixture::Field),
            "Page",
        );
        let root = page_root(&scene);
        (scene, root)
    }
}
/// Writes `progress` to every flight record on the page's Screens.
fn set_in_place_progress(root: &nuxie::RuntimeArtboardInstanceHandle, progress: f32) {
    for row in nuxie::live_move::screens(root).unwrap().values() {
        for record in
            row.with_artboard(|b| b.find_all_handles::<nuxie::live_move::records::LiveFlight>())
        {
            assert!(
                nuxie::runtime::generated::core_registry::CoreRegistry::set_double_handle(
                    &record, 62033, progress
                )
            );
        }
    }
}
/// The semantic node of role `role` on Screen `screen`'s card.
fn in_place_node(
    root: &nuxie::RuntimeArtboardInstanceHandle,
    screen: &str,
    role: u32,
) -> nuxie::runtime::semantic::semantic_node::SemanticNodeRef {
    use nuxie::runtime::semantic::semantic_data::SemanticData;
    let rows = nuxie::live_move::screens(root).unwrap();
    rows[screen]
        .with_artboard(|b| b.find_all_handles::<SemanticData>())
        .iter()
        .find_map(|h| {
            h.with_downcast::<SemanticData, _>(|d| d.existing_semantic_node())
                .flatten()
                .filter(|n| n.borrow().role == role)
        })
        .unwrap_or_else(|| panic!("no node of role {role} on {screen}"))
}
/// The page's semantic snapshot, presented: each node's id, parent, role
/// and state.
unsafe fn in_place_snapshot(
    player: *mut NuxPlayer,
) -> (*mut NuxSemanticSnapshot, Vec<NuxSemanticNodeView>) {
    unsafe {
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
        let nodes = (0..info.node_count)
            .map(|i| {
                let mut node = NuxSemanticNodeView {
                    struct_size: std::mem::size_of::<NuxSemanticNodeView>() as u32,
                    ..Default::default()
                };
                assert_eq!(
                    nux_semantic_snapshot_node(snapshot, i, &mut node),
                    NuxStatus::Ok
                );
                node
            })
            .collect();
        (snapshot, nodes)
    }
}

/// A field in a copy flying in place keeps its native overlay's geometry
/// through a step, as it does on runtime main: the field stays focused
/// and its geometry reads every frame, where it rests, half way and
/// landed. iOS moves its native field by it each frame of a step; where
/// the geometry reads a copy flying in place is a later change.
#[test]
fn in_place_a_focused_fields_geometry_reads_through_a_step() {
    unsafe {
        let (scene, root) = in_place_field_page();
        assert_eq!(nux_player_enable_semantics(scene.player), NuxStatus::Ok);
        step_ok(scene.player);
        let rows = nuxie::live_move::screens(&root).unwrap();
        let text = rows["A"]
            .with_artboard(|b| {
                b.find_handle::<nuxie::runtime::text::text_input::TextInput>("edit a")
            })
            .unwrap();
        let node = rows["A"]
            .with_artboard(|b| b.find_all_handles::<nuxie::runtime::focus_data::FocusData>())[0]
            .with_downcast::<nuxie::runtime::focus_data::FocusData, _>(|f| f.existing_focus_node())
            .flatten()
            .unwrap();
        let manager = node.borrow().manager().unwrap();
        manager.with_focus_manager_mut(|m| m.set_focus(node.clone()));
        step_ok(scene.player);
        let read = |when: &str| {
            assert!(
                text.with_downcast::<nuxie::runtime::text::text_input::TextInput, _>(
                    |t| t.is_focused()
                )
                .unwrap(),
                "{when}: the field is focused"
            );
            let id = in_place_node(&root, "A", 6).borrow().id();
            let (snapshot, _) = in_place_snapshot(scene.player);
            let mut out = NuxTextInputGeometry {
                struct_size: std::mem::size_of::<NuxTextInputGeometry>() as u32,
                ..Default::default()
            };
            assert_eq!(
                nux_player_text_input_geometry(
                    scene.player,
                    snapshot,
                    id,
                    view("edit a"),
                    &mut out
                ),
                NuxStatus::Ok,
                "{when}: the field's geometry"
            );
            assert_eq!(nux_semantic_snapshot_free(snapshot), NuxStatus::Ok);
            [
                out.world_transform[4],
                out.world_transform[5],
                out.max_x - out.min_x,
                out.max_y - out.min_y,
            ]
        };
        let rest = read("at rest");
        let t = view("t1");
        assert_eq!(
            nux_player_step_begin(scene.player, view("B"), &t),
            NuxStatus::Ok
        );
        set_in_place_progress(&root, 0.5);
        step_ok(scene.player);
        assert_eq!(read("half way"), rest, "half way: where it rests");
        set_in_place_progress(&root, 1.0);
        step_ok(scene.player);
        assert_eq!(
            nux_player_step_end(scene.player, view("B"), &t),
            NuxStatus::Ok
        );
        step_ok(scene.player);
        assert_eq!(read("landed"), rest, "landed: where it rests");
        scene.close();
    }
}

// The C API rows of the live-move inventory (15, 17, 23, 31) on an in-place
// page: `in_place_carry_fixture`, whose unit transfers between two keyed
// component placements. Each is the slot test of the same name with
// `in_place_` in front, through the same C calls.

/// An in-place carry page of `kind`, played on its page.
unsafe fn carry_page(kind: fixture::InPlaceCarry) -> LiveScene {
    unsafe { LiveScene::from_bytes_named(&fixture::in_place_carry_fixture(kind), "Page") }
}
/// Screen `screen`'s component placements, in order.
fn placements(root: &nuxie::RuntimeArtboardInstanceHandle, screen: &str) -> Vec<nuxie::CoreHandle> {
    nuxie::live_move::screens(root).unwrap()[screen]
        .with_artboard(|b| b.nested_artboards().to_vec())
}
/// The instance a placement holds, if any.
fn held(host: &nuxie::CoreHandle) -> Option<nuxie::RuntimeArtboardInstanceHandle> {
    host.with(|o| o.as_nested_artboard().unwrap().artboard_instance_default())
        .flatten()
}
/// The in-place component's artboard and its model's schema: "unit x"
/// follows the Screens' artboards, and "Unit" the page's model.
const IN_PLACE_UNIT_ARTBOARD: usize = 2;
const IN_PLACE_UNIT_SCHEMA: usize = 3;

#[test]
fn in_place_live_moves_players_share_the_occurrence_flight_and_history() {
    unsafe {
        let mut scene = carry_page(fixture::InPlaceCarry::TransferVideo);
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
        // One instance flies: B's own was released for it.
        let flying = video_ids(scene.player);
        assert_eq!(flying.len(), 1);
        assert_eq!(video_ids(second), flying);
        step_ok(second);
        assert_eq!(
            nux_player_step_end(scene.player, view("B"), &t),
            NuxStatus::Ok
        );
        // The other player also sees the landed unit and A's own copy.
        assert_eq!(
            nux_player_step_begin(second, view("A"), ptr::null()),
            NuxStatus::Ok
        );
        assert_eq!(
            nux_player_step_end(scene.player, view("A"), ptr::null()),
            NuxStatus::Ok
        );
        assert_eq!(video_ids(second), video_ids(scene.player));
        assert_eq!(video_ids(second).len(), 2);
        assert!(video_ids(second).contains(&flying[0]));
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

/// As the slot test, with a string the two placements bind differently
/// standing for the slot test's completed flight at take-off: in place a
/// step begins at its timeline's first frame, where a number still shows
/// the leaving value, and a string switches at take-off.
#[test]
fn in_place_live_moves_publish_exact_binding_owners_on_takeoff_landing_and_wake() {
    unsafe {
        let scene = carry_page(fixture::InPlaceCarry::TransferValues {
            two_way_text: false,
            reference: false,
        });
        let mut shared = ptr::null_mut();
        assert_eq!(
            nux_view_model_instance_new_schema_default(
                scene.file,
                IN_PLACE_UNIT_SCHEMA,
                &mut shared
            ),
            NuxStatus::Ok
        );
        let root = page_root(&scene);
        let unit = held(&placements(&root, "A")[0]).unwrap();
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
            nux_artboard_instance_new(scene.file, IN_PLACE_UNIT_ARTBOARD, &mut observer),
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
            nux_artboard_instance_new(scene.file, IN_PLACE_UNIT_ARTBOARD, &mut unrelated),
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
        let snapshot_value = |index| {
            let mut snapshot = ptr::null_mut();
            assert_eq!(
                nux_view_model_instance_snapshot(shared, &mut snapshot),
                NuxStatus::Ok
            );
            let mut value = NuxViewModelSnapshotValueView::default();
            assert_eq!(
                nux_view_model_snapshot_value(snapshot, index, &mut value),
                NuxStatus::Ok
            );
            let text = if value.bytes_value.len == 0 {
                String::new()
            } else {
                String::from_utf8_lossy(std::slice::from_raw_parts(
                    value.bytes_value.data,
                    value.bytes_value.len,
                ))
                .into_owned()
            };
            nux_view_model_snapshot_free(snapshot);
            (value.number_value, text)
        };
        let property = || snapshot_value(1).0;
        let label = || snapshot_value(2).1;
        assert_eq!(label(), "outgoing");
        let revision = acknowledge_current(observer_player);
        assert_eq!(
            nux_player_step_begin(scene.player, view("B"), &view("t1")),
            NuxStatus::Ok
        );
        assert_eq!(label(), "incoming");
        assert_eq!(property(), 10.0);
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

/// Through the C API, a transferred unit lands in the arriving Screen's
/// placement and the placement it left holds a fresh copy of its own; a
/// NULL step name moves nothing.
#[test]
fn in_place_step_end_lands_and_refills_through_the_c_api() {
    unsafe {
        let scene = carry_page(fixture::InPlaceCarry::Transfer);
        let player = scene.player;
        let root = page_root(&scene);
        let a = placements(&root, "A")[0].clone();
        let b = placements(&root, "B")[0].clone();
        let original = held(&a).unwrap();
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
        // In place the instance flies in the arriving placement.
        assert!(held(&a).is_none());
        assert_eq!(held(&b).unwrap().core_handle(), original.core_handle());
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
        assert_eq!(held(&b).unwrap().core_handle(), original.core_handle());
        let x =
            nuxie::runtime::generated::core_registry::CoreRegistry::get_double_handle(&keyed, 13)
                .unwrap();
        assert!((x - 50.0).abs() < 0.002, "x={x}");
        // The placement it left draws a copy of its own.
        let own = held(&a).unwrap();
        assert_ne!(own.core_handle(), original.core_handle());
        assert!(
            a.with(|o| o.as_nested_artboard().unwrap().will_draw())
                .unwrap()
        );
        assert_eq!(
            nux_player_step_begin(player, view("A"), ptr::null()),
            NuxStatus::Ok
        );
        assert_eq!(held(&a).unwrap().core_handle(), own.core_handle());
        assert_eq!(held(&b).unwrap().core_handle(), original.core_handle());
        assert_eq!(
            nux_player_step_end(player, view("A"), ptr::null()),
            NuxStatus::Ok
        );
        assert_eq!(held(&a).unwrap().core_handle(), own.core_handle());
        assert_eq!(held(&b).unwrap().core_handle(), original.core_handle());
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
        scene.close();
    }
}

#[test]
fn in_place_live_moves_keep_the_c_api_video_id_reachable() {
    unsafe {
        let scene = carry_page(fixture::InPlaceCarry::TransferVideo);
        let player = scene.player;
        // A's unit's video, and B's own.
        let at_rest = video_ids(player);
        assert_eq!(at_rest.len(), 2);
        let t = view("t1");
        assert_eq!(nux_player_step_begin(player, view("B"), &t), NuxStatus::Ok);
        // The carry puts B's own away, so the one left is A's unit's.
        let original = video_ids(player);
        assert_eq!(original.len(), 1);
        assert!(at_rest.contains(&original[0]));
        assert_eq!(nux_player_step_end(player, view("B"), &t), NuxStatus::Ok);
        // The flown video keeps its id; the copy A gets has a video of its
        // own.
        let landed = video_ids(player);
        assert_eq!(landed.len(), 2);
        assert!(landed.contains(&original[0]));
        assert_eq!(
            nux_player_step_begin(player, view("A"), ptr::null()),
            NuxStatus::Ok
        );
        assert_eq!(video_ids(player), landed);
        assert_eq!(
            nux_player_step_end(player, view("A"), ptr::null()),
            NuxStatus::Ok
        );
        assert_eq!(video_ids(player), landed);
        scene.close();
    }
}

#[test]
fn in_place_live_moves_semantic_snapshot_keeps_one_field_and_focus() {
    use nuxie::runtime::{
        focus_data::FocusData, semantic::semantic_data::SemanticData, text::text_input::TextInput,
    };
    use std::collections::HashSet;
    unsafe {
        for focused in [false, true] {
            let scene = carry_page(fixture::InPlaceCarry::TransferSemantics);
            let player = scene.player;
            let root = page_root(&scene);
            let a = placements(&root, "A");
            let b = placements(&root, "B");
            let unit = held(&a[0]).unwrap();
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
            let role_id = |role| {
                semantic_data
                    .iter()
                    .find_map(|h| {
                        h.with_downcast::<SemanticData, _>(|d| d.existing_semantic_node())
                            .flatten()
                            .filter(|n| n.borrow().role == role)
                    })
                    .unwrap()
                    .borrow()
                    .id()
            };
            let (group_id, field_id) = (role_id(9), role_id(6));
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
            // Each Screen's card has a node of its own, as each slot has on
            // the slot page, so a copy's group sits under its card's.
            let check = |host: &nuxie::CoreHandle, copies: usize| {
                let (snapshot, views) = in_place_snapshot(player);
                let nodes: Vec<_> = views
                    .iter()
                    .map(|n| (n.id, n.parent_id, n.role, n.state_flags))
                    .collect();
                let unique: HashSet<_> = nodes.iter().map(|n| n.0).collect();
                assert_eq!(
                    unique.len(),
                    nodes.len(),
                    "duplicate semantic ids: {nodes:?}"
                );
                assert_eq!(nodes.len(), 2 + copies * 3);
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
                // The same snapshot id still resolves to the live TextInput
                // through the C API.
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
            // One instance flies, in B's placement; A's is empty.
            check(&b[0], 1);
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
            check(&b[0], 1);
            assert_eq!(
                nux_player_step_end(player, view("B"), &transition),
                NuxStatus::Ok
            );
            // A holds a copy of its own, with a field of its own.
            check(&b[0], 2);
            // A cut moves nothing.
            assert_eq!(
                nux_player_step_begin(player, view("A"), ptr::null()),
                NuxStatus::Ok
            );
            check(&b[0], 2);
            assert_eq!(
                nux_player_step_end(player, view("A"), ptr::null()),
                NuxStatus::Ok
            );
            check(&b[0], 2);
            // Back flies the same unit; take-off releases A's never-shown
            // copy, and B gets a copy of its own when it lands.
            let back = view("back");
            assert_eq!(
                nux_player_step_begin(player, view("A"), &back),
                NuxStatus::Ok
            );
            check(&a[0], 1);
            assert_eq!(nux_player_step_end(player, view("A"), &back), NuxStatus::Ok);
            check(&a[0], 2);
            scene.close();
        }
    }
}

#[test]
fn in_place_live_moves_sleep_wake_and_hidden_transfers_keep_sdk_identities_round4() {
    use nuxie::{live_move, runtime::semantic::semantic_data::SemanticData};
    unsafe {
        let scene = carry_page(fixture::InPlaceCarry::TransferVideo);
        let root = page_root(&scene);
        let a = placements(&root, "A");
        let b = placements(&root, "B");
        let unit = held(&a[0]).unwrap();
        // B asleep, so the one video is A's unit's.
        assert_eq!(
            nux_player_set_row_collapsed(scene.player, view("screens"), 1, 1),
            NuxStatus::Ok
        );
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
            let (snapshot, nodes) = in_place_snapshot(scene.player);
            nux_semantic_snapshot_free(snapshot);
            let matches = nodes.iter().filter(|n| n.id == semantic_id).count();
            assert_eq!(matches, usize::from(shown));
            assert_eq!(
                video_ids(scene.player),
                if shown { vec![video_id] } else { vec![] }
            );
            // A sleeping video remains addressable by the same SDK ID.
            let visible = video_visible(scene.player, video_id);
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
        sleep(1, 0);
        let t = view("t1");
        assert_eq!(
            nux_player_step_begin(scene.player, view("B"), &t),
            NuxStatus::Ok
        );
        assert_eq!(
            nux_player_step_end(scene.player, view("B"), &t),
            NuxStatus::Ok
        );
        // Both SDKs retire A in the batch that ends the step; the copy A gets
        // is shown only when A is shown again.
        sleep(0, 1);
        check(true);
        sleep(1, 1);
        check(false);
        // Waking B shows the same occurrence.
        sleep(1, 0);
        check(true);
        // Step incoming Screens must be awake. Exercise the transport seam for
        // an attachment into a sleeping destination instead: A's placement,
        // while A sleeps.
        live_move::carry(&b[0], &a[0], false).unwrap();
        check(false);
        sleep(0, 0);
        check(true);
        sleep(0, 1);
        check(false);
        live_move::carry(&a[0], &b[0], false).unwrap();
        check(true);
        scene.close();
    }
}

/// The slot test's device page has authored Screen phases and completion
/// events, and a timeline for each step. This page has neither the phases
/// nor the events, and a timeline for t1 alone: B's machine plays it. Back's
/// flight record is written frame by frame over the same second, as a
/// timeline would key it. Each step's flight is at 1 before the step ends,
/// as a host ends a step once it plays out.
#[test]
fn in_place_live_moves_remove_outgoing_row_after_back_then_step_round9() {
    use crate::data_binding::*;
    unsafe {
        let scene = carry_page(fixture::InPlaceCarry::TransferPlayed { overshoot: false });
        let root = page_root(&scene);
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
        // Screen B's flight record `key`.
        let record = |key: u32| {
            nuxie::live_move::screens(&root).unwrap()["B"]
                .with_artboard(|b| b.find_all_handles::<nuxie::live_move::records::LiveFlight>())
                .into_iter()
                .find(|r| {
                    r.with_downcast::<nuxie::live_move::records::LiveFlight, _>(|r| {
                        r.values.key == key
                    })
                    .unwrap()
                })
                .unwrap_or_else(|| panic!("no flight record {key}"))
        };
        let progress = |key| {
            nuxie::runtime::generated::core_registry::CoreRegistry::get_double_handle(
                &record(key),
                62033,
            )
            .unwrap()
        };
        let original = held(&placements(&root, "A")[0]).unwrap();
        for (step, incoming, key) in [("t1", "B", 11), ("back", "A", 12)] {
            if step == "back" {
                // The SDK's Back move leaves this two-row order [A, B].
                mutate_list(NUX_VIEW_MODEL_MUTATION_KIND_LIST_MOVE, 0, 0);
                step_ok(scene.player);
                assert_eq!(
                    nux_player_set_row_collapsed(scene.player, view("screens"), 0, 0),
                    NuxStatus::Ok
                );
            }
            let name = view(step);
            assert_eq!(
                nux_player_step_begin(scene.player, view(incoming), &name),
                NuxStatus::Ok
            );
            for frame in 0..=60 {
                if step == "back" {
                    assert!(
                        nuxie::runtime::generated::core_registry::CoreRegistry::set_double_handle(
                            &record(key),
                            62033,
                            frame as f32 / 60.0
                        )
                    );
                }
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
                nux_player_step_result_free(result);
                if frame == 30 {
                    assert!(
                        (progress(key) - 0.5).abs() < 0.02,
                        "{step} is half way at frame 30: {}",
                        progress(key)
                    );
                }
            }
            assert!(
                (progress(key) - 1.0).abs() < 1e-4,
                "{step} has played out before it ends: {}",
                progress(key)
            );
            assert_eq!(
                nux_player_step_end(scene.player, view(incoming), &name),
                NuxStatus::Ok
            );
            if step == "t1" {
                assert_eq!(
                    nux_player_set_row_collapsed(scene.player, view("screens"), 0, 1),
                    NuxStatus::Ok
                );
            }
        }
        // Back landed the unit home, in A's placement.
        assert_eq!(
            held(&placements(&root, "A")[0]).unwrap().core_handle(),
            original.core_handle()
        );
        drop(original);
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
        assert!(!nuxie::live_move::screens(&root).unwrap().contains_key("B"));
    }
}

#[test]
fn in_place_live_moves_blended_midpoint_step_publishes_to_the_shared_owner_observer() {
    unsafe {
        // B's own machine plays t1's flight timeline, as in a published file.
        let scene = carry_page(fixture::InPlaceCarry::TransferPlayed { overshoot: false });
        let root = page_root(&scene);
        let unit = held(&placements(&root, "A")[0]).unwrap();
        let native_model = unit
            .data_context()
            .unwrap()
            .with_context(|c| c.main_view_model_instance())
            .unwrap();
        // As in the exact-owner regression, expose the nested unit's actual
        // model through a public handle and bind another occurrence to it.
        let mut shared = ptr::null_mut();
        assert_eq!(
            nux_view_model_instance_new_schema_default(
                scene.file,
                IN_PLACE_UNIT_SCHEMA,
                &mut shared
            ),
            NuxStatus::Ok
        );
        (*shared).instance =
            RuntimeOwnedViewModelHandle::from_native((*scene.file).file.clone(), native_model)
                .unwrap();
        (*shared).identity = (*shared).instance.instance_identity();
        let mut observer = ptr::null_mut();
        let mut observer_player = ptr::null_mut();
        assert_eq!(
            nux_artboard_instance_new(scene.file, IN_PLACE_UNIT_ARTBOARD, &mut observer),
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

/// UNIV-6769: a Screen with no Exit leaves by playing its Entrance at speed
/// -1, and an Entrance with timed events crosses them backwards. Rive reports
/// those delays signed by direction; the step publishes how long ago each
/// event happened and keeps playing. Driven as the SDK drives two Screen
/// changes: lifecycle writes, step begin, frames, step end.
#[test]
fn second_screen_change_reports_a_reversed_entrance_events_and_keeps_playing() {
    use crate::data_binding::*;
    use nuxie::live_move::device_fixture::reversed_entrance_fixture;
    unsafe {
        let scene = LiveScene::from_bytes(&reversed_entrance_fixture());
        let mut rows = [ptr::null_mut(); 3];
        for (index, row) in rows.iter_mut().enumerate() {
            assert_eq!(
                nux_view_model_instance_list_item_acquire(scene.vm, view("screens"), index, row),
                NuxStatus::Ok
            );
        }
        let [a, b, c] = rows;
        let write = |row, path: &str, value: &str| {
            let path = std::ffi::CString::new(path).unwrap();
            let value = std::ffi::CString::new(value).unwrap();
            assert_eq!(
                nux_view_model_instance_set_string(row, path.as_ptr(), value.as_ptr()),
                NuxStatus::Ok
            );
        };
        write(a, "screen/phase", "active");
        step_ok(scene.player);
        // Frames of 1/8 s from the start of each change: exact in binary, and
        // never landing on an event (0.85, 0.6 and 0.35 s), so each event is
        // crossed inside one frame, 0.025 s (forward from 0) or 0.1 s
        // (backward from 1) before that frame ends.
        const FRAME: f32 = 0.125;
        let mut changes = Vec::new();
        for (incoming, incoming_vm, outgoing_vm) in [("B", b, a), ("C", c, b)] {
            write(incoming_vm, "screen/phase", "entering");
            write(outgoing_vm, "screen/phase", "exiting");
            assert_eq!(
                nux_player_step_begin(scene.player, view(incoming), ptr::null()),
                NuxStatus::Ok
            );
            let mut events = Vec::new();
            for frame in 0..=10 {
                let mut result = ptr::null_mut();
                assert_eq!(
                    step_status(
                        scene.player,
                        &NuxPlayerStep {
                            elapsed_seconds: if frame == 0 { 0.0 } else { FRAME },
                            ..Default::default()
                        },
                        &mut result,
                    ),
                    NuxStatus::Ok,
                    "Screen change to {incoming}, frame {frame}"
                );
                for event in (&(*result).events).iter() {
                    let name = String::from_utf8_lossy(event.name.as_deref().unwrap_or_default());
                    events.push((frame, name.into_owned(), event.seconds_delay));
                }
                nux_player_step_result_free(result);
            }
            assert_eq!(
                nux_player_step_end(scene.player, view(incoming), ptr::null()),
                NuxStatus::Ok
            );
            write(incoming_vm, "screen/phase", "active");
            write(outgoing_vm, "screen/phase", "hidden");
            changes.push(events);
        }
        let expect = |events: &[(i32, String, f32)], want: [(i32, &str, f32); 3], label: &str| {
            assert_eq!(events.len(), want.len(), "{label}: {events:?}");
            for ((frame, name, delay), (want_frame, want_name, want_delay)) in
                events.iter().zip(want)
            {
                assert_eq!(
                    (*frame, name.as_str()),
                    (want_frame, want_name),
                    "{label}: {events:?}"
                );
                assert!(
                    (delay - want_delay).abs() < 1e-4,
                    "{label}: {name} delay {delay}, expected {want_delay}"
                );
            }
        };
        // A to B: B's Entrance plays forward from 0 s.
        expect(
            &changes[0],
            [
                (3, "beat2", 0.025),
                (5, "beat1", 0.025),
                (7, "beat0", 0.025),
            ],
            "B arriving",
        );
        // B to C: B's Entrance plays backward from 1 s.
        expect(
            &changes[1],
            [(2, "beat0", 0.1), (4, "beat1", 0.1), (6, "beat2", 0.1)],
            "B leaving",
        );
        for row in rows {
            nux_view_model_instance_free(row);
        }
    }
}

/// UNIV-6769: a delay that isn't a number is published as the end of the
/// step instead of failing it; a negative one as its distance.
#[test]
fn published_event_delays_are_finite_and_never_negative() {
    use nuxie::page_player::seconds_since_event;
    assert_eq!(seconds_since_event(-0.25), 0.25);
    assert_eq!(seconds_since_event(0.25), 0.25);
    assert_eq!(seconds_since_event(-0.0).to_bits(), 0.0f32.to_bits());
    for delay in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        assert_eq!(seconds_since_event(delay), 0.0);
    }
}

// A file of the retired slot shape, published before flights flew in place
// (docs/live-moves.md, Files of the retired slot shape).

/// It opens when its header lists the slot record's property keys. Nothing
/// empties its flight slots any more: at rest each holds a live copy of its
/// unit, 0 by 0 at the Screen's top left with the unit's own box overflowing
/// it. Its named step is refused with a status and moves nothing, and the
/// page plays on.
#[test]
fn a_retired_slot_file_opens_and_refuses_its_named_step() {
    use nuxie::runtime::layout_component::LayoutComponent;
    unsafe {
        let scene = LiveScene::from_bytes_named(
            &fixture::in_place_fixture(fixture::InPlaceFixture::SlotFile { listed: true }),
            "Page",
        );
        let player = scene.player;
        let root = page_root(&scene);
        let hosts = |screen: &str| placements(&root, screen);
        let (a, b) = (hosts("A"), hosts("B"));
        assert_eq!((a.len(), b.len()), (1, 3), "home on each, two flights on B");
        let copies = || {
            a.iter()
                .chain(&b)
                .map(|host| held(host).map(|unit| unit.core_handle()))
                .collect::<Vec<_>>()
        };
        let at_rest = copies();
        assert!(at_rest.iter().all(Option::is_some), "every slot holds one");
        for flight in &b[1..] {
            let (world, draws) = flight
                .with(|o| {
                    (
                        o.as_world_transform_component().unwrap().world_transform(),
                        o.as_nested_artboard().unwrap().will_draw(),
                    )
                })
                .unwrap();
            assert_eq!((world[4], world[5]), (0.0, 0.0), "the Screen's top left");
            assert!(draws, "the copy draws");
            let unit = held(flight).unwrap();
            let (frame, badge) = unit.with_artboard(|unit| {
                (
                    (unit.layout_width(), unit.layout_height()),
                    unit.find_handle::<LayoutComponent>("badge").unwrap(),
                )
            });
            assert_eq!(frame, (0.0, 0.0), "its frame is the 0 by 0 slot");
            let badge = badge
                .with_downcast::<LayoutComponent, _>(|b| (b.layout_width(), b.layout_height()))
                .unwrap();
            assert_eq!(badge, (40.0, 40.0), "its box overflows the slot");
        }
        for _ in 0..2 {
            assert_eq!(
                nux_player_step_begin(player, view("B"), &view("t1")),
                NuxStatus::InvalidArgument
            );
            assert!(copies() == at_rest, "a refused step moves nothing");
            step_ok(player);
        }
        for screen in ["B", "A"] {
            assert_eq!(
                nux_player_step_begin(player, view(screen), ptr::null()),
                NuxStatus::Ok
            );
            step_ok(player);
            assert_eq!(
                nux_player_step_end(player, view(screen), ptr::null()),
                NuxStatus::Ok
            );
        }
        for collapsed in [1, 0] {
            assert_eq!(
                nux_player_set_row_collapsed(player, view("screens"), 1, collapsed),
                NuxStatus::Ok
            );
            step_ok(player);
        }
        assert!(copies() == at_rest, "a nameless step moves nothing");
        scene.close();
    }
}

/// One whose header lists no slot keys can't be read past its first slot
/// record: it is malformed.
#[test]
fn a_retired_slot_file_without_listed_keys_is_malformed() {
    let mut file = ptr::null_mut();
    let bytes = fixture::in_place_fixture(fixture::InPlaceFixture::SlotFile { listed: false });
    assert_eq!(
        unsafe { import_fixture(&bytes, &mut file) },
        NuxStatus::ImportError
    );
    assert!(file.is_null());
}
