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
