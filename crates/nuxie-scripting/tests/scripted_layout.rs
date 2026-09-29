#![cfg(feature = "luau")]

use luaur_rt::{Function, Table, Value, Vector};
use nuxie_render_api::{NullFactory, PersistentFactory};
use nuxie_runtime::source::{
    core::CoreArena,
    lua::scripting_vm::RuntimeScriptingVmHandle,
    scripted::scripted_layout::{LayoutDirection, LayoutScaleType, ScriptedLayout, Vec2},
};
use nuxie_runtime::{
    NoopScriptHost, ScriptInstance, ScriptMethod, ScriptOptionalMethodResult, ScriptValue,
};
use nuxie_scripting::vm::{LuaScriptInstance, ScriptVm};
use std::rc::Rc;

// Upstream scripting_layout_test.cpp's selfTable seam. The safe Lua API owns
// stack restoration for chunk/getter calls; typed results also assert the
// upstream LUA_TVECTOR checks without reaching into the VM's private stack.
fn resize_dispatch_instance(source: &str) -> (Rc<ScriptVm>, LuaScriptInstance) {
    let vm = Rc::new(ScriptVm::new());
    vm.install_rive_globals().expect("install Rive globals");
    vm.load("layout-resize-dispatch", source)
        .expect("load resize script")
        .call::<Value>(())
        .expect("execute resize script");
    let table: Table = vm.lua().globals().get("selfTable").unwrap();
    let instance = vm.script_instance_from_table(table);
    (vm, instance)
}

fn resize_getter<T: luaur_rt::FromLuaMulti>(vm: &ScriptVm, name: &str) -> T {
    vm.lua()
        .globals()
        .get::<Function>(name)
        .unwrap()
        .call(())
        .unwrap()
}

#[test]
fn layout_resize_dispatch_carries_the_display_scale() {
    let (vm, mut instance) = resize_dispatch_instance(
        r#"
type MyLayout = {}
local lastSize: Vector?
local lastScale: number?
function resize(self: MyLayout, size: Vector, scale: number)
  lastSize = size
  lastScale = scale
end
selfTable = { resize = resize }
function getLastSize(): Vector?
  return lastSize
end
function getLastScale(): number?
  return lastScale
end
return function(): Layout<MyLayout>
  return { resize = resize }
end
"#,
    );
    vm.set_display_scale(2.5);
    vm.call_layout_resize(&mut instance, Vec2::new(300.0, 200.0), &mut NoopScriptHost)
        .expect("layout resize");
    let size: Vector = resize_getter(&vm, "getLastSize");
    assert_eq!(size.x(), 300.0);
    assert_eq!(size.y(), 200.0);
    assert_eq!(resize_getter::<f64>(&vm, "getLastScale"), 2.5);
}

#[test]
fn display_scale_changes_reannounce_the_layout_size() {
    let (vm, instance) = resize_dispatch_instance(
        r#"
type MyLayout = {}
local resizeCallCount = 0
local lastSize: Vector?
local lastScale: number?
function resize(self: MyLayout, size: Vector, scale: number)
  resizeCallCount = resizeCallCount + 1
  lastSize = size
  lastScale = scale
end
selfTable = { resize = resize }
function getResizeCallCount(): number
  return resizeCallCount
end
function getLastScale(): number?
  return lastScale
end
function getLastSize(): Vector?
  return lastSize
end
"#,
    );
    let backend = RuntimeScriptingVmHandle::new(Box::new(vm.clone()));
    let arena = CoreArena::default();
    let layout = arena.insert(ScriptedLayout::default());
    layout
        .with_mut(|object| {
            let layout = object.as_scripted_layout_mut().unwrap();
            layout
                .base
                .base
                .scripted
                .install_script_instance(Box::new(instance), backend.clone());
            layout.base.base.scripted.set_implemented_methods(1 << 12);
        })
        .unwrap();
    backend.register_scripted_object(layout.clone());

    vm.set_display_scale(3.0);
    assert_eq!(resize_getter::<f64>(&vm, "getResizeCallCount"), 0.0);
    layout
        .with_mut(|object| {
            object.as_scripted_layout_mut().unwrap().control_size(
                Vec2::new(240.0, 120.0),
                LayoutScaleType::Fixed,
                LayoutScaleType::Fixed,
                LayoutDirection::Inherit,
            );
        })
        .unwrap();
    assert_eq!(resize_getter::<f64>(&vm, "getResizeCallCount"), 1.0);

    vm.set_display_scale(4.0);
    assert_eq!(resize_getter::<f64>(&vm, "getResizeCallCount"), 2.0);
    assert_eq!(resize_getter::<f64>(&vm, "getLastScale"), 4.0);
    let size: Vector = resize_getter(&vm, "getLastSize");
    assert_eq!(size.x(), 240.0);
    assert_eq!(size.y(), 120.0);

    vm.set_display_scale(4.0);
    assert_eq!(resize_getter::<f64>(&vm, "getResizeCallCount"), 2.0);
    backend.unregister_scripted_object(&layout);
}

#[test]
fn two_parameter_resize_scripts_ignore_the_scale_argument() {
    let (vm, mut instance) = resize_dispatch_instance(
        r#"
type MyLayout = {}
local resizeCallCount = 0
function resize(self: MyLayout, size: Vector)
  resizeCallCount = resizeCallCount + 1
end
selfTable = { resize = resize }
function getResizeCallCount(): number
  return resizeCallCount
end
return function(): Layout<MyLayout>
  return { resize = resize }
end
"#,
    );
    vm.call_layout_resize(&mut instance, Vec2::new(120.0, 80.0), &mut NoopScriptHost)
        .expect("layout resize");
    assert_eq!(resize_getter::<f64>(&vm, "getResizeCallCount"), 1.0);
}

fn layout_instance(source: &str) -> (ScriptVm, LuaScriptInstance, Table) {
    let vm = ScriptVm::new();
    vm.install_rive_globals().expect("install Rive globals");
    let chunk = vm
        .load("scripted-layout-test", source)
        .expect("load layout");
    let generator: Function = chunk.call(()).expect("execute layout chunk");
    let table: Table = generator.call(Value::Nil).expect("create layout table");
    let instance = vm.script_instance_from_table(table.clone());
    (vm, instance, table)
}

#[test]
fn scripted_layout_measure_function_can_be_called() {
    let (_vm, mut instance, table) = layout_instance(
        r#"
        return function()
            return {
                measureCallCount = 0,
                measure = function(self)
                    self.measureCallCount += 1
                    return Vector.xy(200, 150)
                end,
            }
        end
        "#,
    );

    assert_eq!(
        instance
            .call_method(ScriptMethod::Measure, &[], &mut NoopScriptHost)
            .expect("measure"),
        ScriptValue::Vec3 {
            x: 200.0,
            y: 150.0,
            z: 0.0,
        }
    );
    assert_eq!(table.get::<i64>("measureCallCount").unwrap(), 1);
}

#[test]
fn scripted_layout_resize_function_can_be_called() {
    let (_vm, mut instance, table) = layout_instance(
        r#"
        return function()
            return {
                resizeCallCount = 0,
                resize = function(self, size)
                    self.resizeCallCount += 1
                    self.lastResizeSize = size
                end,
            }
        end
        "#,
    );

    instance
        .call_method(
            ScriptMethod::Resize,
            &[ScriptValue::Vec2 { x: 300.0, y: 200.0 }],
            &mut NoopScriptHost,
        )
        .expect("resize");
    assert_eq!(table.get::<i64>("resizeCallCount").unwrap(), 1);
    assert_eq!(
        table.get::<Vector>("lastResizeSize").unwrap(),
        Vector::new(300.0, 200.0, 0.0)
    );
}

#[test]
fn scripted_layout_advance_function_can_be_called() {
    let (_vm, mut instance, table) = layout_instance(
        r#"
        return function()
            return {
                advanceCallCount = 0,
                totalElapsed = 0,
                advance = function(self, seconds)
                    self.advanceCallCount += 1
                    self.totalElapsed += seconds
                    return true
                end,
            }
        end
        "#,
    );

    assert!(
        instance
            .call_advance_truthy(0.033, &mut NoopScriptHost)
            .expect("advance")
    );
    assert_eq!(table.get::<i64>("advanceCallCount").unwrap(), 1);
    assert!((table.get::<f64>("totalElapsed").unwrap() - 0.033).abs() < 1e-9);
}

#[test]
fn scripted_layout_update_function_can_be_called() {
    let (_vm, mut instance, table) = layout_instance(
        r#"
        return function()
            return {
                updateCallCount = 0,
                update = function(self)
                    self.updateCallCount += 1
                end,
            }
        end
        "#,
    );

    instance
        .call_method(ScriptMethod::Update, &[], &mut NoopScriptHost)
        .expect("update");
    assert_eq!(table.get::<i64>("updateCallCount").unwrap(), 1);
}

#[test]
fn scripted_layout_draw_function_can_be_called() {
    let (vm, mut instance, table) = layout_instance(
        r#"
        return function()
            return {
                drawCallCount = 0,
                draw = function(self, _renderer)
                    self.drawCallCount += 1
                end,
            }
        end
        "#,
    );
    let mut factory = PersistentFactory::new(NullFactory::new());
    vm.install_render_factory(&mut factory)
        .expect("install render factory");
    let mut renderer = factory.borrow().make_renderer();

    instance
        .call_draw(&mut factory, &mut renderer, &mut NoopScriptHost)
        .expect("draw");
    assert_eq!(table.get::<i64>("drawCallCount").unwrap(), 1);
}

#[test]
fn optional_layout_callback_is_resolved_once() {
    let (_vm, mut instance, table) = layout_instance(
        r#"
        return function()
            local layout = { lookups = 0 }
            return setmetatable(layout, {
                __index = function(self, key)
                    if key == "measure" then
                        rawset(self, "lookups", self.lookups + 1)
                        if self.lookups == 1 then
                            return function(_self)
                                return Vector.xy(12, 34)
                            end
                        end
                    end
                    return nil
                end,
            })
        end
        "#,
    );

    assert_eq!(
        instance
            .call_optional_method(ScriptMethod::Measure, &[], &mut NoopScriptHost)
            .expect("optional measure"),
        ScriptOptionalMethodResult::Returned(ScriptValue::Vec3 {
            x: 12.0,
            y: 34.0,
            z: 0.0,
        })
    );
    assert_eq!(table.get::<i64>("lookups").unwrap(), 1);
}
mod support;
use support::ScriptVmSourceTestExt as _;
