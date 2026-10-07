#![allow(clippy::unwrap_used, reason = "handwritten C ABI assertions")]
use nux_capi::*;
use std::ptr;
#[path = "support/value_markers.rs"]
mod fixture;

#[test]
fn plain_step_rejects_scene_growth_on_the_growing_step() {
    let bytes = fixture::quiet_scene_growth();
    let mut file = ptr::null_mut();
    let mut value = ptr::null_mut();
    let mut artboard = ptr::null_mut();
    let mut player = ptr::null_mut();
    unsafe {
        assert_eq!(
            nux_file_import(
                bytes.as_ptr(),
                bytes.len(),
                &NuxRenderCallbacks::default(),
                &mut file,
            ),
            NuxStatus::Ok
        );
        assert_eq!(
            nux_view_model_instance_new_authored(file, 1, 0, &mut value),
            NuxStatus::Ok
        );
        assert_eq!(
            nux_artboard_instance_new(file, 2, &mut artboard),
            NuxStatus::Ok
        );
        assert_eq!(
            nux_artboard_instance_bind_view_model(artboard, value),
            NuxStatus::Ok
        );
        assert_eq!(nux_player_new_default(artboard, &mut player), NuxStatus::Ok);
        let mut result = ptr::null_mut();
        assert_eq!(
            nux_player_step(player, &NuxPlayerStep::default(), &mut result),
            NuxStatus::LimitExceeded
        );
        nux_player_step_result_free(result);
        result = ptr::null_mut();
        assert_eq!(
            nux_player_step(player, &NuxPlayerStep::default(), &mut result),
            NuxStatus::RuntimeError
        );
        nux_player_step_result_free(result);
        nux_player_free(player);
        nux_artboard_instance_free(artboard);
        nux_view_model_instance_free(value);
        nux_file_free(file);
    }
}
