#![allow(clippy::unwrap_used, reason = "handwritten C ABI assertions")]
use nux_capi::*;
use std::ptr;
#[path = "support/value_markers.rs"]
mod fixture;

#[test]
fn plain_step_rejects_scene_growth_on_the_growing_step() {
    assert_scene_growth(None, false, false);
}

#[test]
fn empty_table_preserves_plain_scene_growth_validation() {
    assert_scene_growth(Some(false), false, false);
}

#[test]
fn nonempty_table_defers_quiet_scene_growth_validation_without_poisoning() {
    assert_scene_growth(Some(true), false, true);
}

#[test]
fn removing_last_table_restores_plain_scene_growth_validation() {
    assert_scene_growth(Some(true), true, false);
}

fn view(value: &str) -> NuxStringView {
    NuxStringView {
        data: value.as_ptr().cast(),
        len: value.len(),
    }
}

fn assert_scene_growth(table: Option<bool>, remove: bool, deferred: bool) {
    let bytes = if table == Some(true) {
        fixture::quiet_scene_growth_with_marker()
    } else {
        fixture::quiet_scene_growth()
    };
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
        if let Some(nonempty) = table {
            let marker = NuxValueMarker {
                model: view("Row"),
                value: view("n"),
                marker: view("n_set"),
            };
            let (entries, count) = if nonempty {
                (&marker as *const _, 1)
            } else {
                (ptr::null(), 0)
            };
            assert_eq!(
                nux_file_set_value_markers(file, entries, count),
                NuxStatus::Ok
            );
            if remove {
                assert_eq!(
                    nux_file_set_value_markers(file, ptr::null(), 0),
                    NuxStatus::Ok
                );
            }
        }
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
            if deferred {
                NuxStatus::Ok
            } else {
                NuxStatus::LimitExceeded
            }
        );
        nux_player_step_result_free(result);
        result = ptr::null_mut();
        assert_eq!(
            nux_player_step(player, &NuxPlayerStep::default(), &mut result),
            if deferred {
                NuxStatus::LimitExceeded
            } else {
                NuxStatus::RuntimeError
            }
        );
        nux_player_step_result_free(result);
        if deferred {
            result = ptr::null_mut();
            assert_eq!(
                nux_player_step(player, &NuxPlayerStep::default(), &mut result),
                NuxStatus::LimitExceeded
            );
            nux_player_step_result_free(result);
        }
        nux_player_free(player);
        nux_artboard_instance_free(artboard);
        nux_view_model_instance_free(value);
        nux_file_free(file);
    }
}
