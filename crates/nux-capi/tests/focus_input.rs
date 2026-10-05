use nux_capi::*;
use std::{collections::BTreeMap, ptr};

#[path = "../../nuxie-runtime/tests/support/focus_input.rs"]
mod fixture;

fn view(text: &str) -> NuxStringView {
    NuxStringView {
        data: text.as_ptr().cast(),
        len: text.len(),
    }
}
fn string(view: NuxStringView) -> String {
    if view.len == 0 {
        return String::new();
    }
    String::from_utf8(
        unsafe { std::slice::from_raw_parts(view.data.cast::<u8>(), view.len) }.to_vec(),
    )
    .unwrap()
}
fn asset(name: &str) -> Vec<u8> {
    let root = std::env::var_os("RIVE_RUNTIME_DIR").expect("RIVE_RUNTIME_DIR");
    std::fs::read(
        std::path::Path::new(&root)
            .join("tests/unit_tests/assets")
            .join(name),
    )
    .unwrap()
}
struct Scene {
    file: *mut NuxFile,
    artboard: *mut NuxArtboardInstance,
    model: *mut NuxViewModelInstance,
    player: *mut NuxPlayer,
    properties: BTreeMap<usize, String>,
}
impl Scene {
    fn new(bytes: &[u8], name: Option<&str>, bind: bool) -> Self {
        unsafe {
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
            let status = match name {
                Some(name) => nux_artboard_instance_new_named(file, view(name), &mut artboard),
                None => nux_artboard_instance_new(file, 0, &mut artboard),
            };
            assert_eq!(status, NuxStatus::Ok);
            let mut model = ptr::null_mut();
            let mut properties = BTreeMap::new();
            if bind {
                assert_eq!(
                    nux_view_model_instance_new_default(artboard, &mut model),
                    NuxStatus::Ok
                );
                assert_eq!(
                    nux_artboard_instance_bind_view_model(artboard, model),
                    NuxStatus::Ok
                );
                let mut snapshot = ptr::null_mut();
                assert_eq!(
                    nux_view_model_instance_snapshot(model, &mut snapshot),
                    NuxStatus::Ok
                );
                let mut root = NuxViewModelSnapshotInstanceView::default();
                assert_eq!(
                    nux_view_model_snapshot_instance(snapshot, 0, &mut root),
                    NuxStatus::Ok
                );
                assert_eq!(nux_view_model_snapshot_free(snapshot), NuxStatus::Ok);
                let mut catalog = ptr::null_mut();
                assert_eq!(
                    nux_file_view_model_catalog(file, &mut catalog),
                    NuxStatus::Ok
                );
                let mut info = NuxViewModelCatalogInfo::default();
                assert_eq!(
                    nux_view_model_catalog_info(catalog, &mut info),
                    NuxStatus::Ok
                );
                for index in 0..info.property_count {
                    let mut property = NuxViewModelPropertyView::default();
                    assert_eq!(
                        nux_view_model_catalog_property(catalog, index, &mut property),
                        NuxStatus::Ok
                    );
                    if property.schema_index == root.schema_index {
                        properties.insert(property.property_index, string(property.name));
                    }
                }
                assert_eq!(nux_view_model_catalog_free(catalog), NuxStatus::Ok);
            }
            let mut player = ptr::null_mut();
            assert_eq!(nux_player_new_default(artboard, &mut player), NuxStatus::Ok);
            Self {
                file,
                artboard,
                model,
                player,
                properties,
            }
        }
    }
    fn step(&self, focus: &[NuxPlayerFocusInput]) -> ResultView {
        self.operation(
            &NuxPlayerStep {
                focus_inputs: focus.as_ptr(),
                focus_input_count: focus.len(),
                elapsed_seconds: 0.016,
                correlation_id: 42,
                ..NuxPlayerStep::default()
            },
            NuxStatus::Ok,
        )
    }
    fn operation(&self, step: &NuxPlayerStep, expected: NuxStatus) -> ResultView {
        let mut result = ptr::null_mut();
        assert_eq!(
            unsafe { nux_player_step(self.player, step, &mut result) },
            expected
        );
        assert!(!result.is_null());
        ResultView(result)
    }
    fn collect(&self, result: &ResultView, values: &mut BTreeMap<String, NuxViewModelChangeView>) {
        for index in 0..result.info().view_model_change_count {
            let mut change = NuxViewModelChangeView::default();
            assert_eq!(
                unsafe { nux_player_step_result_view_model_change(result.0, index, &mut change) },
                NuxStatus::Ok
            );
            assert_eq!(change.origin, NUX_VIEW_MODEL_CHANGE_ORIGIN_RUNTIME);
            assert_eq!(change.correlation_id, 42);
            if let Some(name) = self.properties.get(&change.property_index) {
                values.insert(name.clone(), change);
            }
        }
    }
    fn focus(&self) -> NuxPlayerFocusState {
        let mut state = NuxPlayerFocusState::default();
        assert_eq!(
            unsafe { nux_player_focus_state(self.player, &mut state) },
            NuxStatus::Ok
        );
        state
    }
}
impl Drop for Scene {
    fn drop(&mut self) {
        unsafe {
            assert_eq!(nux_player_free(self.player), NuxStatus::Ok);
            if !self.model.is_null() {
                assert_eq!(nux_view_model_instance_free(self.model), NuxStatus::Ok);
            }
            assert_eq!(nux_artboard_instance_free(self.artboard), NuxStatus::Ok);
            assert_eq!(nux_file_free(self.file), NuxStatus::Ok);
        }
    }
}
struct ResultView(*mut NuxPlayerStepResult);
impl ResultView {
    fn info(&self) -> NuxPlayerStepInfo {
        let mut info = NuxPlayerStepInfo::default();
        assert_eq!(
            unsafe { nux_player_step_result_info(self.0, &mut info) },
            NuxStatus::Ok
        );
        info
    }
    fn focus(&self, index: usize) -> u32 {
        let mut value = u32::MAX;
        assert_eq!(
            unsafe { nux_player_step_result_focus_input(self.0, index, &mut value) },
            NuxStatus::Ok
        );
        value
    }
    fn events(&self) -> Vec<String> {
        (0..self.info().event_count)
            .map(|index| {
                let mut event = NuxPlayerEventView::default();
                assert_eq!(
                    unsafe { nux_player_step_result_event(self.0, index, &mut event) },
                    NuxStatus::Ok
                );
                string(event.name)
            })
            .collect()
    }
}
impl Drop for ResultView {
    fn drop(&mut self) {
        assert_eq!(
            unsafe { nux_player_step_result_free(self.0) },
            NuxStatus::Ok
        );
    }
}
fn focus(kind: u32) -> NuxPlayerFocusInput {
    NuxPlayerFocusInput {
        kind,
        ..NuxPlayerFocusInput::default()
    }
}
fn key(code: u32, modifiers: u32, pressed: u32, repeat: u32) -> NuxPlayerFocusInput {
    NuxPlayerFocusInput {
        kind: NUX_PLAYER_FOCUS_KIND_KEY,
        key_code: code,
        modifiers,
        pressed,
        repeat,
        ..NuxPlayerFocusInput::default()
    }
}

#[test]
fn keyboard_changes_are_captured_in_the_step() {
    let scene = Scene::new(&asset("keyboard_listener.riv"), Some("KeyboardInput"), true);
    scene.step(&[]);
    assert_eq!(scene.step(&[focus(NUX_PLAYER_FOCUS_KIND_NEXT)]).focus(0), 1);
    let mut values = BTreeMap::new();
    // The upstream C++ test has one advance for each row. Its two pre-advance
    // checks are observed after the atomic step carrying the corresponding keys.
    for (inputs, expected) in [
        (vec![key(65, 0, 1, 0)], 1.0),
        (vec![key(65, 0, 1, 1)], 1.0),
        (vec![key(65, 0, 0, 0)], 2.0),
        (vec![key(65, 1, 1, 0)], 2.0),
        (
            vec![key(69, 0, 0, 0), key(69, 0, 1, 1), key(69, 0, 1, 0)],
            2.0,
        ),
        (vec![key(66, 0, 1, 0)], 2.0),
        (vec![key(66, 0, 0, 0)], 3.0),
        (vec![key(66, 0, 1, 1)], 4.0),
        (vec![key(68, 0, 1, 0)], 4.0),
        (vec![key(68, 9, 1, 0)], 5.0),
        (vec![key(67, 9, 1, 0)], 5.0),
        (vec![key(67, 1, 1, 0)], 6.0),
        (vec![key(88, 1, 1, 0)], 6.0),
    ] {
        let result = scene.step(&inputs);
        assert_eq!(result.info().focus_input_result_count, inputs.len());
        scene.collect(&result, &mut values);
        assert_eq!(
            values
                .get("keyCount")
                .map(|v| v.number_value)
                .unwrap_or(0.0),
            expected
        );
    }
}

#[test]
fn text_and_key_listener_changes_share_the_step_result() {
    let scene = Scene::new(&asset("text_input_event.riv"), None, true);
    scene.step(&[]);
    let mut values = BTreeMap::new();
    for (input, expected) in [
        (focus(NUX_PLAYER_FOCUS_KIND_NEXT), [1, 0, 0]),
        (key(66, 0, 1, 0), [1, 0, 0]),
        (
            NuxPlayerFocusInput {
                kind: NUX_PLAYER_FOCUS_KIND_TEXT,
                text: view("b"),
                ..NuxPlayerFocusInput::default()
            },
            [1, 0, 1],
        ),
        (key(65, 0, 1, 0), [1, 1, 1]),
    ] {
        let result = scene.step(&[input]);
        scene.collect(&result, &mut values);
        assert_eq!(scene.focus().has_focus, 1);
        assert_eq!(scene.focus().expects_keyboard_input, 1);
        for (name, expected) in ["isFocused", "hasKeyed", "hasTexted"]
            .into_iter()
            .zip(expected)
        {
            assert_eq!(
                values.get(name).map(|v| v.bool_value).unwrap_or(0),
                expected,
                "{name}"
            );
        }
    }
}

#[test]
fn authored_focus_and_blur_require_focus_data() {
    for present in [true, false] {
        let scene = Scene::new(&fixture::focus_events(present), None, false);
        scene.step(&[]);
        assert_eq!(scene.focus().has_focus, 0);
        let result = scene.step(&[focus(NUX_PLAYER_FOCUS_KIND_NEXT)]);
        assert_eq!(result.focus(0), u32::from(present));
        assert_eq!(
            result.events(),
            if present {
                vec!["focus_arrived"]
            } else {
                vec![]
            }
        );
        assert_eq!(scene.focus().has_focus, u32::from(present));
        assert_eq!(scene.focus().expects_keyboard_input, 0);
        let result = scene.step(&[focus(NUX_PLAYER_FOCUS_KIND_CLEAR)]);
        assert_eq!(result.focus(0), 0);
        assert_eq!(
            result.events(),
            if present { vec!["focus_left"] } else { vec![] }
        );
        assert_eq!(scene.focus().has_focus, 0);
        let result = scene.step(&[focus(NUX_PLAYER_FOCUS_KIND_PREVIOUS)]);
        assert_eq!(result.focus(0), u32::from(present));
        assert_eq!(scene.focus().has_focus, u32::from(present));
    }
}

#[test]
fn non_state_machine_players_reject_focus_inputs_and_state_queries() {
    for linear in [false, true] {
        let mut scene = Scene::new(&asset("circle_clips.riv"), None, false);
        unsafe {
            assert_eq!(nux_player_free(scene.player), NuxStatus::Ok);
            let status = if linear {
                let mut name = NuxStringView::default();
                assert_eq!(
                    nux_file_artboard_animation_name(scene.file, 0, 0, &mut name),
                    NuxStatus::Ok
                );
                nux_player_new_linear_animation_named(scene.artboard, name, &mut scene.player)
            } else {
                nux_player_new_static(scene.artboard, &mut scene.player)
            };
            assert_eq!(status, NuxStatus::Ok);
            let mut state = NuxPlayerFocusState::default();
            assert_eq!(
                nux_player_focus_state(scene.player, &mut state),
                NuxStatus::NotFound
            );
        }
        let input = focus(NUX_PLAYER_FOCUS_KIND_NEXT);
        scene.operation(
            &NuxPlayerStep {
                focus_inputs: &input,
                focus_input_count: 1,
                ..NuxPlayerStep::default()
            },
            NuxStatus::NotFound,
        );
    }
}

#[test]
fn caller_ending_at_correlation_id_retains_pointer_results() {
    #[repr(C)]
    struct OlderStep {
        struct_size: u32,
        inputs: *const NuxPlayerInputChange,
        input_count: usize,
        pointers: *const NuxPlayerPointerEvent,
        pointer_count: usize,
        elapsed_seconds: f32,
        correlation_id: u64,
    }
    assert_eq!(
        std::mem::size_of::<OlderStep>(),
        std::mem::offset_of!(NuxPlayerStep, focus_inputs)
    );
    let old = Scene::new(&asset("click_event.riv"), Some("art-1"), false);
    let full = Scene::new(&asset("click_event.riv"), Some("art-1"), false);
    let pointers = [NuxPlayerPointerEvent {
        kind: NUX_PLAYER_POINTER_KIND_DOWN,
        x: 50.0,
        y: 50.0,
        pointer_id: 1,
        timestamp_seconds: 0.0,
    }];
    let legacy = OlderStep {
        struct_size: std::mem::size_of::<OlderStep>() as u32,
        inputs: ptr::null(),
        input_count: 0,
        pointers: pointers.as_ptr(),
        pointer_count: pointers.len(),
        elapsed_seconds: 0.016,
        correlation_id: 42,
    };
    let mut result = ptr::null_mut();
    assert_eq!(
        unsafe {
            nux_player_step(
                old.player,
                (&legacy as *const OlderStep).cast(),
                &mut result,
            )
        },
        NuxStatus::Ok
    );
    let old_result = ResultView(result);
    let full_result = full.operation(
        &NuxPlayerStep {
            pointers: pointers.as_ptr(),
            pointer_count: pointers.len(),
            elapsed_seconds: 0.016,
            correlation_id: 42,
            ..NuxPlayerStep::default()
        },
        NuxStatus::Ok,
    );
    assert_eq!(old_result.info().focus_input_result_count, 0);
    assert_eq!(full_result.info().focus_input_result_count, 0);
    assert_eq!(old_result.info().pointer_result_count, 1);
    for index in 0..pointers.len() {
        let mut old_hit = u32::MAX;
        let mut full_hit = u32::MAX;
        assert_eq!(
            unsafe { nux_player_step_result_pointer(old_result.0, index, &mut old_hit) },
            NuxStatus::Ok
        );
        assert_eq!(
            unsafe { nux_player_step_result_pointer(full_result.0, index, &mut full_hit) },
            NuxStatus::Ok
        );
        assert_eq!(old_hit, full_hit);
    }
    assert_eq!(old_result.events(), full_result.events());
    assert_eq!(old_result.info().keep_going, full_result.info().keep_going);
}

#[test]
fn focus_state_and_result_accessors_validate_caller_arguments() {
    let scene = Scene::new(&fixture::focus_events(true), None, false);
    scene.step(&[]);
    let mut state = NuxPlayerFocusState {
        struct_size: 4,
        ..NuxPlayerFocusState::default()
    };
    assert_eq!(
        unsafe { nux_player_focus_state(scene.player, &mut state) },
        NuxStatus::InvalidStructSize
    );
    assert_eq!(
        unsafe { nux_player_focus_state(scene.player, ptr::null_mut()) },
        NuxStatus::NullArgument
    );
    let result = scene.step(&[
        focus(NUX_PLAYER_FOCUS_KIND_NEXT),
        focus(NUX_PLAYER_FOCUS_KIND_CLEAR),
    ]);
    assert_eq!(result.events(), vec!["focus_arrived", "focus_left"]);
    assert_eq!(result.focus(0), 1);
    assert_eq!(result.focus(1), 0);
    let mut value = 0;
    assert_eq!(
        unsafe { nux_player_step_result_focus_input(result.0, 2, &mut value) },
        NuxStatus::NotFound
    );
    assert_eq!(
        unsafe { nux_player_step_result_focus_input(result.0, 0, ptr::null_mut()) },
        NuxStatus::NullArgument
    );
}
