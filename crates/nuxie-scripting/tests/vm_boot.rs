//! Deliverable: VM boot + script execution on luaur — compile and run Luau
//! source, call functions, read results back, and bind Rust host functions.
#![cfg(feature = "luau")]

use luaur_rt::{Function, Table, Value};
use nuxie_runtime::{
    NoopScriptHost, ScriptDataConverterMethod, ScriptInstance, ScriptListenerActionMethod,
    ScriptListenerInvocation, ScriptPointerEventKind, ScriptValue,
};
use nuxie_scripting::vm::{LuaScriptInstance, ScriptVm};

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
        format!("{err}").contains("require could not find a script named MissingModule"),
        "got: {err}"
    );
}

#[test]
fn rive_vector_math_uses_all_three_components() {
    let vm = ScriptVm::new();
    vm.install_rive_globals().unwrap();

    let values: (f64, f64, f64, f64, f64, f64, f64, f64, f64, f64, f64) = vm
        .eval(
            r#"
            local a = Vector.xyz(1, 2, 3)
            local b = Vector.xyz(4, 5, 6)
            local cross = Vector.cross3(Vector.xyz(1, 0, 0), Vector.xyz(0, 1, 0))
            local normalized = Vector.normalized(Vector.xyz(0, 3, 4))
            local added = Vector.scaleAndAdd(Vector.xyz(0, 0, 1), Vector.xyz(0, 0, 2), 3)
            local subtracted = Vector.scaleAndSub(Vector.xyz(0, 0, 7), Vector.xyz(0, 0, 2), 3)
            return a[3],
                Vector.length(Vector.xyz(1, 2, 2)),
                Vector.lengthSquared(a),
                Vector.dot(a, b),
                Vector.distance(Vector.xyz(1, 1, 1), Vector.xyz(1, 1, 4)),
                Vector.distanceSquared(Vector.origin(), a),
                cross.z,
                normalized.z,
                Vector.lerp(Vector.xyz(0, 0, 10), Vector.xyz(0, 0, 20), 0.5).z,
                added.z,
                subtracted.z
            "#,
        )
        .unwrap();

    assert_eq!(
        values,
        (
            3.0,
            3.0,
            14.0,
            32.0,
            3.0,
            14.0,
            1.0,
            f64::from(0.8_f32),
            15.0,
            7.0,
            1.0,
        )
    );
}

#[test]
fn rive_vector_lerp_binding_returns_the_exact_endpoint() {
    let vm = ScriptVm::new();
    vm.install_rive_globals().unwrap();

    // Load through a local so the source compiler cannot replace this with
    // Luau's built-in vector.lerp fastcall. This pins the Rive binding path.
    let exact: bool = vm
        .eval(
            r#"
            local lerp = Vector.lerp
            local endpoint = Vector.xyz(1, -2, 3)
            return lerp(Vector.xyz(1e20, -1e20, 1e20), endpoint, 1) == endpoint
            "#,
        )
        .unwrap();

    assert!(exact);
}

#[test]
fn rive_math_fround_narrows_to_float32() {
    let vm = ScriptVm::new();
    vm.install_rive_globals().unwrap();

    let rounded: f64 = vm.eval("return math.fround(1.00000006)").unwrap();
    assert_eq!(rounded, f64::from(1.00000006_f64 as f32));
}

#[test]
fn rive_vector_instance_methods_and_buffer_writes_match_cpp() {
    let vm = ScriptVm::new();
    vm.install_rive_globals().unwrap();

    let values: (f64, f64, f64, f64, f64) = vm
        .eval(
            r#"
            local value = Vector.xyz(1, 2, 3)
            local bytes = buffer.create(28)
            value:writeToBuffer(bytes, 4)
            value:writeVec4(bytes, 12, 4)
            return value:length(),
                buffer.readf32(bytes, 4),
                buffer.readf32(bytes, 8),
                buffer.readf32(bytes, 12),
                buffer.readf32(bytes, 24)
            "#,
        )
        .unwrap();

    assert_eq!(values, (f64::from(14.0_f32.sqrt()), 1.0, 2.0, 1.0, 4.0));

    let in_bounds: bool = vm
        .eval(
            r#"
            local bytes = buffer.create(12)
            return pcall(function()
                Vector.origin():writeToBuffer(bytes, 4)
            end)
            "#,
        )
        .unwrap();
    assert!(!in_bounds);
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
        format!("{err}").contains("require could not find a script named StillMissing"),
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

#[test]
fn scripted_listener_prefers_perform_action_and_preserves_pointer_payload() {
    let vm = ScriptVm::new();
    vm.install_rive_globals().unwrap();
    let table: Table = vm
        .eval(
            r#"
            return {
                legacyCalled = false,
                actionCalled = false,
                perform = function(self, event)
                    self.legacyCalled = true
                end,
                performAction = function(self, invocation)
                    self.actionCalled = invocation:isPointerEvent()
                    local event = invocation:asPointerEvent()
                    self.pointerId = event.id
                    self.pointerType = event.type
                    self.positionX = event.position.x
                    self.previousX = event.previousPosition.x
                    self.timeStamp = event.timeStamp
                end,
            }
            "#,
        )
        .unwrap();
    let mut instance = LuaScriptInstance::new(table);
    instance
        .call_listener_action(
            ScriptListenerActionMethod::PerformAction,
            &ScriptListenerInvocation::Pointer {
                pointer_id: 9,
                x: 20.0,
                y: 30.0,
                previous_x: 10.0,
                previous_y: 15.0,
                event: ScriptPointerEventKind::Click,
                timestamp_seconds: 0.25,
            },
            &mut NoopScriptHost,
        )
        .unwrap();

    assert!(instance.table().get::<bool>("actionCalled").unwrap());
    assert!(!instance.table().get::<bool>("legacyCalled").unwrap());
    assert_eq!(instance.table().get::<i64>("pointerId").unwrap(), 9);
    assert_eq!(
        instance.table().get::<String>("pointerType").unwrap(),
        "click"
    );
    assert_eq!(instance.table().get::<f64>("positionX").unwrap(), 20.0);
    assert_eq!(instance.table().get::<f64>("previousX").unwrap(), 10.0);
    assert_eq!(instance.table().get::<f64>("timeStamp").unwrap(), 0.25);
}

#[test]
fn protocol_generator_tables_are_fresh_per_state_machine_instance() {
    let vm = ScriptVm::new();
    let generator: Function = vm
        .eval(
            r#"
            return function(context)
                return {
                    input = 0,
                    init = function(self, initContext)
                        self.initializedWith = self.input
                        return true
                    end,
                    evaluate = function(self)
                        return self.input == 7
                    end,
                }
            end
            "#,
        )
        .unwrap();
    let context = vm.lua().create_table();
    let first: Table = generator.call(context.clone()).unwrap();
    let second: Table = generator.call(context).unwrap();
    let mut first = LuaScriptInstance::new(first);
    let mut second = LuaScriptInstance::new(second);

    first.set_input("input", ScriptValue::Number(7.0)).unwrap();
    first
        .call_method(nuxie_runtime::ScriptMethod::Init, &[], &mut NoopScriptHost)
        .unwrap();

    assert_eq!(first.get_input("input").unwrap(), ScriptValue::Number(7.0));
    assert_eq!(second.get_input("input").unwrap(), ScriptValue::Number(0.0));
    assert_eq!(
        second
            .call_method(
                nuxie_runtime::ScriptMethod::Evaluate,
                &[],
                &mut NoopScriptHost,
            )
            .unwrap(),
        ScriptValue::Bool(false)
    );
}

#[test]
fn setting_file_models_refreshes_data_on_an_already_initialized_vm() {
    let fixture = std::env::var_os("RIVE_RUNTIME_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from("/Users/levi/dev/oss/rive-runtime"))
        .join("tests/unit_tests/assets/script_create_viewmodel_instance.riv");
    let bytes = std::fs::read(&fixture)
        .unwrap_or_else(|error| panic!("missing fixture {}: {error}", fixture.display()));
    let file = nuxie_binary::read_runtime_file(&bytes).expect("fixture parses");
    let models = nuxie_runtime::script_view_models(&file);
    let model_name = models
        .keys()
        .next()
        .cloned()
        .expect("fixture contains a view-model definition");

    // Mirrors an externally supplied VM: globals already exist before this
    // file's view-model definitions are registered.
    let mut vm = ScriptVm::new();
    vm.install_rive_globals().expect("globals install");
    vm.set_view_models(models);

    let data: Table = vm.lua().globals().get("Data").expect("Data global");
    let definition: Table = data
        .get(model_name.as_str())
        .expect("late-registered model is visible through Data");
    assert!(matches!(
        definition
            .get::<Value>("new")
            .expect("Data model constructor"),
        Value::Function(_)
    ));
}
