use super::*;

#[test]
fn invalid_focus_batch_preserves_the_earlier_bool_input() {
    unsafe {
        let root = std::env::var_os("RIVE_RUNTIME_DIR").expect("RIVE_RUNTIME_DIR");
        let bytes =
            std::fs::read(std::path::Path::new(&root).join("tests/unit_tests/assets/smi_test.riv"))
                .unwrap();
        let mut file = ptr::null_mut();
        assert_eq!(
            nux_file_import(
                bytes.as_ptr(),
                bytes.len(),
                &NuxRenderCallbacks::default(),
                &mut file
            ),
            NuxStatus::Ok
        );
        let mut artboard = ptr::null_mut();
        assert_eq!(
            nux_artboard_instance_new(file, 1, &mut artboard),
            NuxStatus::Ok
        );
        let mut player = ptr::null_mut();
        assert_eq!(
            nux_player_new_state_machine_named(
                artboard,
                NuxStringView::from_static("State Machine 1"),
                &mut player
            ),
            NuxStatus::Ok
        );
        let input = NuxPlayerInputChange {
            kind: NUX_PLAYER_INPUT_KIND_BOOL,
            name: NuxStringView::from_static("bool"),
            bool_value: 1,
            number_value: 0.0,
        };
        let read_bool = || {
            let instance = (*player).instance.borrow();
            let PlayerInstance::StateMachine(machine) = &*instance else {
                panic!("state machine");
            };
            machine.get_bool("bool").unwrap().bool_value().unwrap()
        };
        assert!(!read_bool());
        let invalid_utf8 = [0xffu8];
        let oversized = vec![b'a'; NUX_PLAYER_STEP_MAX_TEXT_BYTES + 1];
        let base = NuxPlayerFocusInput {
            kind: NUX_PLAYER_FOCUS_KIND_KEY,
            ..NuxPlayerFocusInput::default()
        };
        let bad = [
            (
                NuxPlayerFocusInput { kind: 99, ..base },
                NuxStatus::InvalidArgument,
            ),
            (
                NuxPlayerFocusInput {
                    key_code: 0x10000,
                    ..base
                },
                NuxStatus::InvalidArgument,
            ),
            (
                NuxPlayerFocusInput {
                    modifiers: 16,
                    ..base
                },
                NuxStatus::InvalidArgument,
            ),
            (
                NuxPlayerFocusInput { pressed: 2, ..base },
                NuxStatus::InvalidArgument,
            ),
            (
                NuxPlayerFocusInput { repeat: 2, ..base },
                NuxStatus::InvalidArgument,
            ),
            (
                NuxPlayerFocusInput {
                    kind: NUX_PLAYER_FOCUS_KIND_TEXT,
                    text: NuxStringView {
                        data: invalid_utf8.as_ptr().cast(),
                        len: 1,
                    },
                    ..base
                },
                NuxStatus::InvalidArgument,
            ),
            (
                NuxPlayerFocusInput {
                    kind: NUX_PLAYER_FOCUS_KIND_TEXT,
                    text: NuxStringView {
                        data: ptr::null(),
                        len: 1,
                    },
                    ..base
                },
                NuxStatus::NullArgument,
            ),
            (
                NuxPlayerFocusInput {
                    kind: NUX_PLAYER_FOCUS_KIND_TEXT,
                    text: NuxStringView {
                        data: oversized.as_ptr().cast(),
                        len: oversized.len(),
                    },
                    ..base
                },
                NuxStatus::LimitExceeded,
            ),
        ];
        let reject = |focus_inputs, count, expected| {
            let step = NuxPlayerStep {
                inputs: &input,
                input_count: 1,
                focus_inputs,
                focus_input_count: count,
                ..NuxPlayerStep::default()
            };
            let mut result = ptr::null_mut();
            assert_eq!(nux_player_step(player, &step, &mut result), expected);
            assert!(!read_bool(), "earlier bool input must not be applied");
            assert_eq!(nux_player_step_result_free(result), NuxStatus::Ok);
        };
        for (item, expected) in bad {
            reject(&item, 1, expected);
        }
        reject(
            &base,
            NUX_PLAYER_STEP_MAX_FOCUS_INPUTS + 1,
            NuxStatus::LimitExceeded,
        );
        reject(ptr::null(), 1, NuxStatus::NullArgument);
        let text = NuxPlayerFocusInput {
            kind: NUX_PLAYER_FOCUS_KIND_TEXT,
            text: NuxStringView {
                data: oversized.as_ptr().cast(),
                len: NUX_PLAYER_STEP_MAX_TEXT_BYTES,
            },
            ..base
        };
        let aggregate = [text; 5];
        reject(
            aggregate.as_ptr(),
            aggregate.len(),
            NuxStatus::LimitExceeded,
        );
        // An accepted batch proves the observed bool is the input being written.
        let mut result = ptr::null_mut();
        assert_eq!(
            nux_player_step(
                player,
                &NuxPlayerStep {
                    inputs: &input,
                    input_count: 1,
                    ..NuxPlayerStep::default()
                },
                &mut result
            ),
            NuxStatus::Ok
        );
        assert!(read_bool());
        assert_eq!(nux_player_step_result_free(result), NuxStatus::Ok);
        assert_eq!(nux_player_free(player), NuxStatus::Ok);
        assert_eq!(nux_artboard_instance_free(artboard), NuxStatus::Ok);
        assert_eq!(nux_file_free(file), NuxStatus::Ok);
    }
}
