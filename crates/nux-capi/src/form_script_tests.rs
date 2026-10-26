//! Form scripts name their own Screen, `context:viewModel()`, as the checked
//! root (UNIV-5587). On a page, the step's bound root is the page, so a form
//! script can't use the unnamed root to reach its Screen's answers.
use super::*;

// The helper the scripted-resource compiler emits for
// `nuxie.responses.set(context, field, value)`, kept here as its contract with
// the checked-write module. One line, as the compiler emits it.
const RESPONSES_SET: &str = r#"local responses = {}; responses.set = function(context, field, value) local screen = context:viewModel(); if screen == nil then error("nuxie.responses.set needs the script's context") end; if type(field) ~= "string" or #field > 4096 then error("nuxie.responses.set field must be a form.field string of at most 4096 UTF-8 bytes") end; local form, name = string.match(field, "^([^.]+)%.([^.]+)$"); if not form or string.find(field, "[/%z]") then error("nuxie.responses.set field must name a form and field") end; local answer = "experience/responses:" .. form .. "/" .. name; if value == nil or type(value) == "table" then local options = module.listValues(screen, answer, "value"); if options == nil then if value ~= nil then error("nuxie.responses.set choices need a several-choice answer") end; return module.set(screen, answer, nil) end; local chosen = {}; if value ~= nil then local count = 0; for key, option in pairs(value) do if type(option) ~= "string" then error("nuxie.responses.set choices must be option names") end; count = count + 1; chosen[option] = true end; for key in pairs(value) do if type(key) ~= "number" or key % 1 ~= 0 or key < 1 or key > count then error("nuxie.responses.set choices must be a list of option names") end end end; local writes = {}; for index, option in ipairs(options) do local hex = string.gsub(option, ".", function(byte) return string.format("%02x", string.byte(byte)) end); writes[index] = {root = screen, path = "experience/responses:" .. form .. "/option:" .. name .. ":" .. hex .. "/picked", value = chosen[option] == true}; chosen[option] = nil end; if next(chosen) ~= nil then error("nuxie.responses.set choices must name options of this answer") end; return module.setAll(writes) end; return module.set(screen, answer, value) end"#;

// Each Screen's `mode` picks what its tap does.
const FORM_SCRIPT: &str = r#"
return function(context)
 return {init = function() return true end, performAction = function(self)
  local mode = context:viewModel().mode.value
  if mode == 1 then
   local ok, rule = responses.set(context, 'form.field', {'y'})
   assert(ok and rule == nil)
  elseif mode == 2 then
   local ok, rule = responses.set(context, 'form.field', {'x', 'y'})
   assert(not ok and rule == 'one-choice')
  elseif mode == 3 then
   assert(responses.set(context, 'form.field', nil))
  elseif mode == 4 then
   assert(responses.set(context, 'form.title', 'changed'))
   assert(responses.set(context, 'form.field', {'y'}))
   error('abort row')
  elseif mode == 5 then
   local ok, rule = responses.set(context, 'form.title', 'changed')
   assert(ok and rule == nil)
  elseif mode == 6 then
   local loose = Data.Form.new()
   local before = loose.title.value
   local ok, message = pcall(module.set, loose, 'title', 'row-a')
   assert(not ok and string.find(tostring(message), 'NotFound'), tostring(message))
   assert(loose.title.value == before)
  elseif mode == 7 then
   local ok, message = pcall(module.set, {}, 'experience/responses:form/title', 'row-a')
   assert(not ok and string.find(tostring(message), 'checked root must be a name or a view model'), tostring(message))
  end
  module.command('row-action', {mode = mode})
 end}
end"#;

unsafe fn form_scene(rows: usize, standalone: bool) -> LiveScene {
    unsafe {
        let script = format!("local module = require('bridge')\n{RESPONSES_SET}\n{FORM_SCRIPT}");
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
        let bytes = fixture::form_script_page_fixture(&payload, rows);
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
                "form scene: {}",
                String::from_utf8_lossy(&(*result).message)
            );
            nux_capi_result_free(result);
        }
        assert_eq!(status, NuxStatus::Ok);
        let titles = [
            view("before"),
            view("changed"),
            view("row-a"),
            view("row-b"),
        ];
        let rules = [
            NuxValueRule {
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
            },
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
        step_ok(scene.player);
        scene
    }
}

/// The page's bound view model, or the Screen's in a single-Screen file.
fn form_root(scene: &LiveScene) -> RuntimeOwnedViewModelHandle {
    unsafe { &*scene.vm }.instance.clone()
}

fn form_screen(scene: &LiveScene, index: usize) -> RuntimeOwnedViewModelHandle {
    form_root(scene)
        .list_item_by_property_name_path("screens", index)
        .unwrap()
}

fn form_picks(screen: &RuntimeOwnedViewModelHandle) -> [bool; 2] {
    let path = "experience/responses:form/field";
    [0, 1].map(|index| {
        screen
            .list_item_by_property_name_path(path, index)
            .unwrap()
            .borrow()
            .boolean_value_by_property_name("picked")
            .unwrap()
    })
}

fn form_title(screen: &RuntimeOwnedViewModelHandle) -> Vec<u8> {
    screen
        .borrow()
        .string_value_by_property_name_path("experience/responses:form/title")
        .unwrap()
        .to_vec()
}

fn set_mode(screen: &RuntimeOwnedViewModelHandle, mode: f32) {
    screen
        .borrow_mut()
        .set_number_by_property_name("mode", mode);
}

/// Tap the given Screens in one step. Every tap's action must run when the step
/// is accepted; a rejected step publishes no commands.
unsafe fn form_tap(scene: &LiveScene, screens: &[usize], expected: NuxStatus) {
    unsafe {
        let pointers = screens
            .iter()
            .map(|&index| NuxPlayerPointerEvent {
                kind: NUX_PLAYER_POINTER_KIND_DOWN,
                x: 100.0 + index as f32 * 300.0,
                y: 50.0,
                pointer_id: index as i32 + 1,
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
        let message = if result.is_null() {
            "missing result".into()
        } else {
            String::from_utf8_lossy(&(*result).message).into_owned()
        };
        assert_eq!(status, expected, "{message}");
        if expected == NuxStatus::RuntimeError {
            assert!((&(*result).host_commands).is_empty());
            assert!(message.contains("abort row"), "{message}");
        } else {
            assert_eq!((&(*result).host_commands).len(), screens.len());
        }
        nux_player_step_result_free(result);
    }
}

#[test]
fn form_scripts_write_their_own_screen_on_a_page() {
    unsafe {
        let scene = form_scene(2, false);
        let (a, b) = (form_screen(&scene, 0), form_screen(&scene, 1));
        form_tap(&scene, &[0], NuxStatus::Ok);
        assert_eq!(form_picks(&a), [false, true]);
        assert_eq!(form_picks(&b), [true, false]);
        set_mode(&b, 5.0);
        form_tap(&scene, &[1], NuxStatus::Ok);
        assert_eq!(form_title(&a), b"before");
        assert_eq!(form_title(&b), b"changed");
    }
}

#[test]
fn form_scripts_refuse_and_clear_only_their_own_screen() {
    unsafe {
        let scene = form_scene(2, false);
        let (a, b) = (form_screen(&scene, 0), form_screen(&scene, 1));
        set_mode(&a, 2.0);
        form_tap(&scene, &[0], NuxStatus::Ok);
        assert_eq!(form_picks(&a), [true, false]);
        set_mode(&a, 3.0);
        form_tap(&scene, &[0], NuxStatus::Ok);
        assert_eq!(form_picks(&a), [false, false]);
        assert_eq!(form_picks(&b), [true, false]);
    }
}

#[test]
fn form_script_later_error_undoes_both_screens() {
    unsafe {
        let scene = form_scene(2, false);
        let (a, b) = (form_screen(&scene, 0), form_screen(&scene, 1));
        set_mode(&b, 4.0);
        form_tap(&scene, &[0, 1], NuxStatus::RuntimeError);
        assert_eq!(form_picks(&a), [true, false]);
        assert_eq!(form_picks(&b), [true, false]);
        assert_eq!(form_title(&b), b"before");
    }
}

#[test]
fn form_script_on_a_single_screen_writes_its_bound_root() {
    unsafe {
        let scene = form_scene(2, true);
        let screen = form_root(&scene);
        form_tap(&scene, &[0], NuxStatus::Ok);
        assert_eq!(form_picks(&screen), [false, true]);
        set_mode(&screen, 5.0);
        form_tap(&scene, &[0], NuxStatus::Ok);
        assert_eq!(form_title(&screen), b"changed");
    }
}

#[test]
fn form_script_is_accepted_beside_sleeping_screens() {
    unsafe {
        let scene = form_scene(3, false);
        for index in [1, 2] {
            assert_eq!(
                nux_player_set_row_collapsed(scene.player, view("screens"), index as u32, 1),
                NuxStatus::Ok
            );
        }
        form_tap(&scene, &[0], NuxStatus::Ok);
        assert_eq!(form_picks(&form_screen(&scene, 0)), [false, true]);
        for index in [1, 2] {
            assert_eq!(form_picks(&form_screen(&scene, index)), [true, false]);
        }
    }
}

#[test]
fn form_script_root_must_be_reached_from_the_step() {
    unsafe {
        let scene = form_scene(2, false);
        let a = form_screen(&scene, 0);
        // A loose view model of the same file is not found. The script catches
        // the error and its step is accepted, so the player keeps working.
        set_mode(&a, 6.0);
        form_tap(&scene, &[0], NuxStatus::Ok);
        // Anything other than a name or a view model is refused by the module.
        set_mode(&a, 7.0);
        form_tap(&scene, &[0], NuxStatus::Ok);
        set_mode(&a, 1.0);
        form_tap(&scene, &[0], NuxStatus::Ok);
        assert_eq!(form_picks(&a), [false, true]);
    }
}

/// Every Screen of a 40-Screen page resolves as a root, asleep or awake, and
/// a write through the last one still lands. Prints what one root lookup
/// costs against the graph walk a step makes when it starts.
#[test]
fn form_script_roots_resolve_on_a_forty_screen_page() {
    const SCREENS: usize = 40;
    const ROUNDS: u32 = 200;
    unsafe {
        let scene = form_scene(SCREENS, false);
        for index in 1..SCREENS / 2 {
            assert_eq!(
                nux_player_set_row_collapsed(scene.player, view("screens"), index as u32, 1),
                NuxStatus::Ok
            );
        }
        let page = form_root(&scene);
        let roots = std::collections::BTreeMap::from([(String::new(), page.clone())]);
        let owners = page.reachable_change_owner_snapshot().unwrap().len();
        let mut slowest = std::time::Duration::ZERO;
        for index in 0..SCREENS {
            let screen = form_screen(&scene, index);
            let root = nuxie::RuntimeCheckedValueRoot::ViewModel(
                nuxie::ScriptViewModel::from_native(screen.native_handle(), screen.native_file())
                    .unwrap(),
            );
            let started = std::time::Instant::now();
            for _ in 0..ROUNDS {
                let found = nuxie::runtime_checked_value_root(&roots, &root).unwrap();
                assert!(found.ptr_eq(&screen));
            }
            slowest = slowest.max(started.elapsed() / ROUNDS);
        }
        let started = std::time::Instant::now();
        for _ in 0..ROUNDS {
            assert_eq!(
                page.reachable_change_owner_snapshot().unwrap().len(),
                owners
            );
        }
        let step_walk = started.elapsed() / ROUNDS;
        eprintln!(
            "forty-screen page: {owners} view models; slowest root lookup {slowest:?}; \
             the step's own graph walk {step_walk:?}"
        );
        let last = form_screen(&scene, SCREENS - 1);
        form_tap(&scene, &[SCREENS - 1], NuxStatus::Ok);
        assert_eq!(form_picks(&last), [false, true]);
    }
}
