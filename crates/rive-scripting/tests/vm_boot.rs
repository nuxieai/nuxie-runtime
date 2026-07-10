//! Deliverable: VM boot + script execution on luaur — compile and run Luau
//! source, call functions, read results back, and bind Rust host functions.
#![cfg(feature = "luau")]

use luaur_rt::{Function, Table, Value};
use rive_runtime::{ScriptDataConverterMethod, ScriptInstance, ScriptValue};
use rive_scripting::vm::{LuaScriptInstance, ScriptVm};

#[test]
fn boots_and_evaluates_source() {
    let vm = ScriptVm::new();
    let n: f64 = vm.eval("return 2 + 3 * 4").unwrap();
    assert_eq!(n, 14.0);

    let (a, b): (String, bool) = vm.eval("return ('ri' .. 've'), 1 < 2").unwrap();
    assert_eq!(a, "rive");
    assert!(b);
}

#[test]
fn defines_function_in_luau_and_calls_it_from_rust() {
    let vm = ScriptVm::new();
    vm.eval::<()>(
        "function lerp(a: number, b: number, t: number): number\n\
             return a + (b - a) * t\n\
         end",
    )
    .unwrap();

    let mid: f64 = vm.call_global("lerp", (10.0, 20.0, 0.25)).unwrap();
    assert_eq!(mid, 12.5);
}

#[test]
fn compiles_a_chunk_into_a_function_and_reads_table_results() {
    let vm = ScriptVm::new();
    let generator = vm
        .load(
            "generator",
            "return function(context)\n\
                 return { frames = 0, advance = function(self, dt) self.frames += 1 end }\n\
             end",
        )
        .unwrap();
    // Chunk -> generator function -> instance table, mirroring how Rive's
    // protocol scripts produce scripted-object instances.
    let make: Function = generator.call(()).unwrap();
    let instance: Table = make.call(Value::Nil).unwrap();
    let advance: Function = instance.get("advance").unwrap();
    advance.call::<()>((instance.clone(), 0.016)).unwrap();
    advance.call::<()>((instance.clone(), 0.016)).unwrap();
    assert_eq!(instance.get::<f64>("frames").unwrap(), 2.0);
}

#[test]
fn binds_rust_host_functions_into_luau() {
    let vm = ScriptVm::new();
    let host_scale = vm
        .lua()
        .create_function(|_, (value, factor): (f64, f64)| Ok(value * factor))
        .unwrap();
    vm.lua().globals().set("hostScale", host_scale).unwrap();

    let out: f64 = vm.eval("return hostScale(21, 2)").unwrap();
    assert_eq!(out, 42.0);
}

#[test]
fn luau_runtime_errors_surface_as_rust_errors() {
    let vm = ScriptVm::new();
    let err = vm.eval::<Value>("error('boom')").unwrap_err();
    assert!(format!("{err}").contains("boom"), "got: {err}");
}

#[test]
fn rive_globals_are_installed_before_sandboxing() {
    let vm = ScriptVm::new();
    vm.install_rive_globals().unwrap();

    let vector: Value = vm.eval("return Vector(1, 2)").unwrap();
    assert!(matches!(vector, Value::Vector(_)), "got: {vector:?}");

    let late: Value = vm.eval("return late('deferred')").unwrap();
    assert!(matches!(late, Value::Nil), "got: {late:?}");

    let err = vm
        .eval::<Value>("return require('MissingModule')")
        .unwrap_err();
    assert!(
        format!("{err}").contains("module 'MissingModule' not found"),
        "got: {err}"
    );
}

#[test]
fn rive_sandbox_marks_libraries_readonly() {
    let vm = ScriptVm::new();
    vm.install_rive_globals().unwrap();

    let math: Table = vm.lua().globals().get("math").unwrap();
    assert!(math.is_readonly());

    let err = vm
        .eval::<()>("math.abs = function(value) return value end")
        .unwrap_err();
    assert!(format!("{err}").contains("readonly"), "got: {err}");
}

#[test]
fn installing_rive_globals_is_idempotent_after_sandboxing() {
    let vm = ScriptVm::new();
    vm.install_rive_globals().unwrap();
    vm.install_rive_globals().unwrap();

    let vector: Value = vm.eval("return Vector(3, 4)").unwrap();
    assert!(matches!(vector, Value::Vector(_)), "got: {vector:?}");

    let err = vm
        .eval::<Value>("return require('StillMissing')")
        .unwrap_err();
    assert!(
        format!("{err}").contains("module 'StillMissing' not found"),
        "got: {err}"
    );
}

#[test]
fn scripted_data_values_round_trip_converter_types_and_color_channels() {
    let vm = ScriptVm::new();
    vm.install_rive_globals().unwrap();
    let table: Table = vm
        .eval(
            r#"
            return {
                convert = function(self, input)
                    if input:isNumber() then
                        local output = DataValue.number()
                        output.value = input.value + 2
                        return output
                    elseif input:isString() then
                        local output = DataValue.string()
                        output.value = input.value .. "!"
                        return output
                    elseif input:isBoolean() then
                        local output = DataValue.boolean()
                        output.value = not input.value
                        return output
                    end
                    local output = DataValue.color()
                    output.value = input.value
                    output.red = input.red + 1
                    output.green = input.green + 2
                    output.blue = input.blue + 3
                    output.alpha = input.alpha - 1
                    return output
                end,
            }
            "#,
        )
        .unwrap();
    let mut instance = LuaScriptInstance::new(table);

    assert_eq!(
        instance
            .call_data_converter(ScriptDataConverterMethod::Convert, ScriptValue::Number(3.0),)
            .unwrap(),
        ScriptValue::Number(5.0)
    );
    assert_eq!(
        instance
            .call_data_converter(
                ScriptDataConverterMethod::Convert,
                ScriptValue::String("rive".to_owned()),
            )
            .unwrap(),
        ScriptValue::String("rive!".to_owned())
    );
    assert_eq!(
        instance
            .call_data_converter(ScriptDataConverterMethod::Convert, ScriptValue::Bool(true),)
            .unwrap(),
        ScriptValue::Bool(false)
    );
    assert_eq!(
        instance
            .call_data_converter(
                ScriptDataConverterMethod::Convert,
                ScriptValue::Color(0xff102030),
            )
            .unwrap(),
        ScriptValue::Color(0xfe112233)
    );
}

#[test]
fn rejects_garbage_bytecode_with_an_error_not_a_crash() {
    let vm = ScriptVm::new();
    let err = vm
        .load_bytecode("garbage", &[0xff, 0x00, 0x13, 0x37])
        .unwrap_err();
    let message = format!("{err}");
    assert!(
        message.contains("bytecode") || message.contains("version"),
        "unexpected error: {message}"
    );
}
