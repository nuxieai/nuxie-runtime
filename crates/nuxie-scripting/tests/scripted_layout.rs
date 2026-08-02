#![cfg(feature = "luau")]

use luaur_rt::{Function, Table, Value, Vector};
use nuxie_render_api::{NullFactory, PersistentFactory};
use nuxie_runtime::{
    NoopScriptHost, ScriptInstance, ScriptMethod, ScriptOptionalMethodResult, ScriptValue,
};
use nuxie_scripting::vm::{LuaScriptInstance, ScriptVm};

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
