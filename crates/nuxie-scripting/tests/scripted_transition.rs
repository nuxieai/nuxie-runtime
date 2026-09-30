//! Supplemental native boundary regressions derived from d974's
//! lua_transition.cpp and lua_script_backend.cpp (not upstream test copies).
#![cfg(feature = "luau")]

use luaur_rt::{Function, Table};
use nuxie_render_api::{Mat2D, PersistentFactory, RecordingFactory};
use nuxie_runtime::source::{artboard::Artboard, core::CoreArena, factory::RuntimeFactoryHandle};
use nuxie_runtime::{NoopScriptHost, ScriptInstance, ScriptTransitionChildRef};
use nuxie_scripting::vm::ScriptVm;

mod support;
use support::ScriptVmSourceTestExt;

fn absent() -> ScriptTransitionChildRef {
    ScriptTransitionChildRef {
        artboard: None,
        transform: Mat2D::IDENTITY,
    }
}

#[test]
fn manages_to_opts_out_only_for_boolean_false() {
    let vm = ScriptVm::new();
    for (field, expected) in [
        ("", true),
        ("managesTo = nil", true),
        ("managesTo = true", true),
        ("managesTo = false", false),
        ("managesTo = 0", true),
        ("managesTo = ''", true),
        ("managesTo = {}", true),
    ] {
        let table: Table = vm.eval(&format!("return {{ {field} }}")).unwrap();
        assert_eq!(
            vm.script_instance_from_table(table).transition_manages_to(),
            expected,
            "{field}"
        );
    }
}

#[test]
fn changed_receives_dimensions_direction_and_nil_then_expires_children() {
    let vm = ScriptVm::new();
    let arena = CoreArena::default();
    let mut artboard = Artboard::default();
    artboard.set_width(123.0);
    artboard.set_height(45.0);
    let child = ScriptTransitionChildRef {
        artboard: Some(arena.insert(artboard)),
        transform: Mat2D::IDENTITY,
    };
    for fail in [false, true] {
        let table: Table = vm
            .eval(
                r#"return {
            changed = function(self, from, to, direction)
                self.child = from
                self.width = from.width
                self.height = from.height
                self.toWasNil = to == nil
                self.direction = direction
                if self.fail then error('changed failure') end
            end,
        }"#,
            )
            .unwrap();
        table.set("fail", fail).unwrap();
        let mut instance = vm.script_instance_from_table(table.clone());
        instance
            .call_transition_changed(&child, &absent(), -1, &mut NoopScriptHost)
            .unwrap();
        assert_eq!(table.get::<f32>("width").unwrap(), 123.0);
        assert_eq!(table.get::<f32>("height").unwrap(), 45.0);
        assert!(table.get::<bool>("toWasNil").unwrap());
        assert_eq!(table.get::<i32>("direction").unwrap(), -1);
        let inspect: Function = vm
            .eval("return function(self) return self.child.width, self.child.height end")
            .unwrap();
        assert_eq!(inspect.call::<(f32, f32)>(table).unwrap(), (0.0, 0.0));
    }
    let table: Table = vm.eval("return { changed = function(self, from, to) self.bothNil = from == nil and to == nil end }").unwrap();
    vm.script_instance_from_table(table.clone())
        .call_transition_changed(&absent(), &absent(), 0, &mut NoopScriptHost)
        .unwrap();
    assert!(table.get::<bool>("bothNil").unwrap());
}

#[test]
fn draw_preserves_child_transform_and_expires_renderer_on_success_and_error() {
    let vm = ScriptVm::new();
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    vm.install_render_factory(&mut factory).unwrap();
    vm.install_rive_globals().unwrap();
    let arena = CoreArena::default();
    let mut artboard = Artboard::default();
    artboard.set_factory(RuntimeFactoryHandle::from_factory(&mut factory).unwrap());
    artboard.base.base.set_clip(false);
    artboard.set_frame_origin(false);
    artboard.set_width(10.0);
    artboard.set_height(20.0);
    let child = ScriptTransitionChildRef {
        artboard: Some(arena.insert(artboard)),
        transform: Mat2D([1.0, 0.0, 0.0, 1.0, 17.0, 23.0]),
    };
    for fail in [false, true] {
        let table: Table = vm
            .eval(
                r#"return {
            draw = function(self, renderer, from, to)
                self.child = from
                self.renderer = renderer
                self.toWasNil = to == nil
                renderer:save()
                renderer:modulateOpacity(0.25)
                from:draw(renderer)
                self.drew = true
                if self.fail then error('draw failure') end
                -- The host balances this intentionally unmatched save.
            end,
        }"#,
            )
            .unwrap();
        table.set("fail", fail).unwrap();
        let mut instance = vm.script_instance_from_table(table.clone());
        let mut renderer = factory.borrow().make_renderer();
        let before = factory.borrow().stream().len();
        instance
            .call_transition_draw(
                &mut factory,
                &mut renderer,
                &child,
                &absent(),
                &mut NoopScriptHost,
            )
            .unwrap();
        assert!(table.get::<bool>("drew").unwrap());
        assert!(table.get::<bool>("toWasNil").unwrap());
        let stream = factory.borrow().stream();
        let calls = &stream[before..];
        assert!(calls.contains("modulateOpacity opacity=0.25"));
        assert!(calls.contains("transform matrix=[1,0,0,1,17,23]"));
        assert_eq!(
            calls.lines().filter(|line| *line == "save").count(),
            calls.lines().filter(|line| *line == "restore").count()
        );
        let inspect: Function = vm
            .eval(
                r#"return function(self)
            local valid, err = pcall(function() self.renderer:save() end)
            return self.child.width, self.child.height, valid, tostring(err)
        end"#,
            )
            .unwrap();
        let (width, height, valid, error): (f32, f32, bool, String) = inspect.call(table).unwrap();
        assert_eq!((width, height), (0.0, 0.0));
        assert!(!valid);
        assert!(error.contains("Renderer is no longer valid"));
    }
}
