use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;
use std::sync::Arc;

use luaur_rt::{Function, Table};
use nuxie_runtime::{RuntimeBlobAsset, ScriptViewModel, ScriptViewModelProperty};

use super::{ScriptedPropertyBlob, create_scripted_view_model};
use crate::vm::{ScriptVm, ScriptingLogLevel};

fn fixture_models(asset: &str) -> BTreeMap<String, ScriptViewModel> {
    let fixture = std::env::var_os("RIVE_RUNTIME_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from("/Users/levi/dev/oss/rive-runtime"))
        .join("tests/unit_tests/assets")
        .join(asset);
    let bytes = std::fs::read(&fixture)
        .unwrap_or_else(|error| panic!("missing fixture {}: {error}", fixture.display()));
    let file = nuxie_binary::read_runtime_file(&bytes).expect("pinned fixture parses");
    nuxie_runtime::script_view_models(&file)
}

fn first_authored_instance(asset: &str, view_model_name: &str) -> ScriptViewModel {
    let fixture = std::env::var_os("RIVE_RUNTIME_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::path::PathBuf::from("/Users/levi/dev/oss/rive-runtime"))
        .join("tests/unit_tests/assets")
        .join(asset);
    let bytes = std::fs::read(&fixture)
        .unwrap_or_else(|error| panic!("missing fixture {}: {error}", fixture.display()));
    let file = nuxie_binary::read_runtime_file(&bytes).expect("pinned fixture parses");
    let instance_name = file
        .view_models()
        .into_iter()
        .find_map(|view_model| {
            (view_model.object.string_property("name") == Some(view_model_name))
                .then(|| {
                    view_model
                        .instances
                        .first()?
                        .object
                        .string_property("name")
                        .map(ToOwned::to_owned)
                })
                .flatten()
        })
        .unwrap_or_else(|| panic!("{asset} has no authored {view_model_name} instance"));
    nuxie_runtime::script_view_models(&file)
        .remove(view_model_name)
        .unwrap_or_else(|| panic!("{asset} has no {view_model_name} view model"))
        .named_instance(Some(&instance_name))
        .expect("authored instance is selectable")
}

fn vm_with_console() -> (ScriptVm, Rc<RefCell<Vec<String>>>) {
    let console = Rc::new(RefCell::new(Vec::new()));
    let output = Rc::clone(&console);
    let vm = ScriptVm::new_with_log_sink(move |level, bytes| {
        assert_eq!(level, ScriptingLogLevel::Info);
        output
            .borrow_mut()
            .push(String::from_utf8(bytes.to_vec()).expect("console line is UTF-8"));
    });
    vm.install_rive_globals().expect("Rive globals install");
    (vm, console)
}

fn call_with_model(vm: &ScriptVm, function: &str, model: &Table) {
    vm.lua()
        .globals()
        .get::<Function>(function)
        .unwrap_or_else(|error| panic!("missing {function}: {error}"))
        .call::<()>(model.clone())
        .unwrap_or_else(|error| panic!("{function} failed: {error}"));
}

#[test]
#[ignore = "expected-red: live ScriptedProperty listeners invoke newest-first, but pinned scripting_properties_test.cpp#2 requires registration order"]
fn wave_c12_scalar_002_scripted_properties_can_be_passed_to_luau() {
    let mut models = fixture_models("data_binding_test.riv");
    let model_definition = models.remove("vm1").expect("pinned vm1 definition");
    let data = model_definition
        .named_instance(None)
        .expect("default vm1 instance");
    assert_eq!(
        data.properties().get("width"),
        Some(&ScriptViewModelProperty::Number)
    );
    assert_eq!(
        data.properties().get("rotation"),
        Some(&ScriptViewModelProperty::Number)
    );
    assert_eq!(
        data.properties().get("color"),
        Some(&ScriptViewModelProperty::Color)
    );
    assert_eq!(
        data.properties().get("text"),
        Some(&ScriptViewModelProperty::String)
    );
    assert_eq!(
        data.properties().get("orient"),
        Some(&ScriptViewModelProperty::Boolean)
    );
    assert!(data.set_number("width", 200.0));
    assert!(data.set_number("rotation", 180.0));
    assert!(data.set_color("color", 0xff00_ff00));
    assert!(data.set_string("text", "New text"));
    assert!(data.set_boolean("orient", true));

    let data_with_trigger = models
        .into_values()
        .find(|model| model.property("trigger-prop") == Some(ScriptViewModelProperty::Trigger))
        .and_then(|model| model.named_instance(None))
        .expect("artboard-2 trigger view-model instance");
    assert_eq!(
        data_with_trigger.property("trigger-prop"),
        Some(ScriptViewModelProperty::Trigger)
    );

    let (vm, console) = vm_with_console();
    vm.lua()
        .load(
            r#"
type ViewModel = {
    getNumber: (self, name:string)->Property<number>?,
    getTrigger: (self, name:string)->PropertyTrigger?
}
type Vm1 = {
    width: Property<number>,
    rotation: Property<rotation>,
    color: Property<color>,
    text: Property<string>,
    orient: Property<boolean>,
    getNumber: (self, name:string)->Property<number>?,
    -- todo: addListener
}
local data:Vm1?
local triggerProp:PropertyTrigger?
local calledChange:boolean = false
local calledChangeWithContext:boolean = false

function provide(vm:Vm1, vm2:ViewModel)
    triggerProp = vm2:getTrigger('trigger-prop')
    if triggerProp then
        print("trigger is good")
        triggerProp:addListener(vm2, triggerTriggered)
    else
        print("bad trigger")
    end
    data = vm
    data2 = vm2
    data.rotation:addListener(changed)
    data.rotation:addListener(vm, changedWithContext)
    print("data provided")
end

function triggerTriggered(context:ViewModel)
    print("trigger was triggered!")
end

function changedWithContext(context:Vm1)
    print("changed with context")
    calledChangeWithContext = true
end

function changed()
    print("changed")
    calledChange = true
end

function getRotation():number
    if data then
        return data.rotation.value
    end
    return 0
end
function getRotationByName():number
    if data then
        local rotation = data:getNumber('rotation')
        if rotation then
            return rotation.value
        end
    end
    return 0
end

function calledBoth():boolean
    return calledChange and calledChangeWithContext
end

function callTriggerIndirectly()
    if triggerProp then
        triggerProp:fire()
    end
end
"#,
        )
        .exec()
        .expect("pinned property script compiles and runs");
    let data_table = create_scripted_view_model(vm.lua(), data.clone()).expect("scripted vm1");
    let trigger_table = create_scripted_view_model(vm.lua(), data_with_trigger.clone())
        .expect("scripted trigger model");
    vm.lua()
        .globals()
        .get::<Function>("provide")
        .expect("provide function")
        .call::<()>((data_table, trigger_table))
        .expect("provide succeeds");
    assert_eq!(
        console.borrow().as_slice(),
        ["trigger is good", "data provided"]
    );

    assert_eq!(
        vm.lua()
            .globals()
            .get::<Function>("getRotation")
            .unwrap()
            .call::<f32>(())
            .unwrap(),
        180.0
    );
    assert_eq!(
        vm.lua()
            .globals()
            .get::<Function>("getRotationByName")
            .unwrap()
            .call::<f32>(())
            .unwrap(),
        180.0
    );
    assert!(
        !vm.lua()
            .globals()
            .get::<Function>("calledBoth")
            .unwrap()
            .call::<bool>(())
            .unwrap()
    );

    assert!(data.set_number("rotation", 360.0));
    assert!(
        vm.lua()
            .globals()
            .get::<Function>("calledBoth")
            .unwrap()
            .call::<bool>(())
            .unwrap()
    );
    assert!(data_with_trigger.fire_trigger("trigger-prop"));
    assert_eq!(
        console.borrow().as_slice(),
        [
            "trigger is good",
            "data provided",
            "changed",
            "changed with context",
            "trigger was triggered!",
        ]
    );
    vm.lua()
        .globals()
        .get::<Function>("callTriggerIndirectly")
        .unwrap()
        .call::<()>(())
        .expect("indirect trigger succeeds");
    assert_eq!(console.borrow().len(), 6);
    assert_eq!(console.borrow()[5], "trigger was triggered!");
}

#[test]
fn wave_c12_scalar_005_scripted_color_can_be_passed_to_luau() {
    let model = first_authored_instance("scripted_color.riv", "colorsVm");
    assert_eq!(model.color("colorProp"), Some(0xff10_1566));
    let (vm, console) = vm_with_console();
    vm.lua()
        .load(
            r#"
function init(vm)
    print(`color init to {vm.colorProp.value}`)
    vm.colorProp:addListener(vm.colorProp, colorChanged)
end

function setRed(vm)
    vm.colorProp.value = Color.rgb(255, 0, 0)
    print(`color is {vm.colorProp.value}`)
end

function colorChanged(color)
    print(`color changed to {color.value}`)
end
"#,
        )
        .exec()
        .expect("pinned color script runs");
    let table = create_scripted_view_model(vm.lua(), model.clone()).expect("scripted colorsVm");
    call_with_model(&vm, "init", &table);
    assert_eq!(model.color("colorProp"), Some(0xff10_1566));
    assert!(model.set_color("colorProp", 0xff10_1567));
    call_with_model(&vm, "setRed", &table);
    assert_eq!(
        console.borrow().as_slice(),
        [
            "color init to 4279244134",
            "color changed to 4279244135",
            "color changed to 4294901760",
            "color is 4294901760",
        ]
    );
}

#[test]
fn wave_c12_scalar_006_scripted_string_can_be_passed_to_luau() {
    let model = first_authored_instance("scripted_string.riv", "stringVm");
    let (vm, console) = vm_with_console();
    vm.lua()
        .load(
            r#"
function init(vm)
    print(`string init to {vm.stringProp.value}`)
    vm.stringProp:addListener(vm.stringProp, stringChanged)
end

function setHello(vm)
    vm.stringProp.value = "Hello World"
    print(`string is {vm.stringProp.value}`)
end

function stringChanged(str)
    print(`string changed to {str.value}`)
end
"#,
        )
        .exec()
        .expect("pinned string script runs");
    let table = create_scripted_view_model(vm.lua(), model.clone()).expect("scripted stringVm");
    call_with_model(&vm, "init", &table);
    assert_eq!(model.string("stringProp").as_deref(), Some("yo"));
    assert!(model.set_string("stringProp", "yoo"));
    call_with_model(&vm, "setHello", &table);
    assert_eq!(
        console.borrow().as_slice(),
        [
            "string init to yo",
            "string changed to yoo",
            "string changed to Hello World",
            "string is Hello World",
        ]
    );
}

#[test]
fn wave_c12_scalar_007_scripted_boolean_can_be_passed_to_luau() {
    let model = first_authored_instance("scripted_boolean.riv", "BoolVM");
    let (vm, console) = vm_with_console();
    vm.lua()
        .load(
            r#"
function init(vm)
    print(`bool init to {vm.BoolProp.value}`)
    vm.BoolProp:addListener(vm.BoolProp, boolChanged)
end

function setTrue(vm)
    vm.BoolProp.value = true
    print(`bool is {vm.BoolProp.value}`)
end

function boolChanged(bool)
    print(`bool changed to {bool.value}`)
end
"#,
        )
        .exec()
        .expect("pinned boolean script runs");
    let table = create_scripted_view_model(vm.lua(), model.clone()).expect("scripted BoolVM");
    call_with_model(&vm, "init", &table);
    assert_eq!(model.boolean("BoolProp"), Some(false));
    call_with_model(&vm, "setTrue", &table);
    assert!(model.set_boolean("BoolProp", false));
    assert_eq!(
        console.borrow().as_slice(),
        [
            "bool init to false",
            "bool changed to true",
            "bool is true",
            "bool changed to false",
        ]
    );
}

#[test]
fn wave_c12_scalar_008_scripted_enum_can_be_passed_to_luau() {
    let model = first_authored_instance("scripted_enum.riv", "EnumVM");
    let (vm, console) = vm_with_console();
    vm.lua()
        .load(
            r#"
function init(vm)
    print(`enum init to {vm.EnumProp.value}`)
    vm.EnumProp:addListener(vm.EnumProp, enumChanged)
end

function setValue(vm, value:string)
    vm.EnumProp.value = value
    print(`enum is {vm.EnumProp.value}`)
end

function enumChanged(e)
    print(`enum changed to {e.value}`)
end
"#,
        )
        .exec()
        .expect("pinned enum script runs");
    let table = create_scripted_view_model(vm.lua(), model.clone()).expect("scripted EnumVM");
    call_with_model(&vm, "init", &table);
    for value in ["blue", "orange", "red"] {
        vm.lua()
            .globals()
            .get::<Function>("setValue")
            .unwrap()
            .call::<()>((table.clone(), value))
            .unwrap_or_else(|error| panic!("setValue({value}) failed: {error}"));
    }
    assert_eq!(
        console.borrow().as_slice(),
        [
            "enum init to white",
            "enum changed to blue",
            "enum is blue",
            "enum changed to orange",
            "enum is orange",
            "enum changed to red",
            "enum is red",
        ]
    );
}

#[test]
fn wave_c12_scalar_017_scripted_blob_property_reads_and_writes_bytes() {
    // Pinned C++ constructs a standalone ViewModelInstanceAssetBlob. Rust's
    // property wrapper is intentionally owner-backed, so use the vendored
    // blob schema only to provide that owner; the value, script, calls, and
    // assertions below remain the exact fixture-free upstream sequence.
    let fixture = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/sync/data_bind_blob_test.riv");
    let bytes = std::fs::read(&fixture)
        .unwrap_or_else(|error| panic!("missing fixture {}: {error}", fixture.display()));
    let file = nuxie_binary::read_runtime_file(&bytes).expect("blob owner fixture parses");
    let mut models = nuxie_runtime::script_view_models(&file);
    let (model, property_name) = models
        .values_mut()
        .find_map(|definition| {
            let property_name = definition.properties().iter().find_map(|(name, kind)| {
                (*kind == ScriptViewModelProperty::Blob).then(|| name.clone())
            })?;
            Some((definition.named_instance(None)?, property_name))
        })
        .expect("fixture has a blob property owner");
    assert!(model.set_blob_asset(
        &property_name,
        Some(Arc::new(RuntimeBlobAsset::new(
            "",
            Arc::<[u8]>::from([10, 20, 30]),
        ))),
    ));

    let vm = ScriptVm::new();
    vm.install_rive_globals().expect("Rive globals install");
    vm.lua()
        .load(
            r#"
function readSize(prop)
    local v = prop.value
    if v then return v.size end
    return -1
end
function readByte(prop, i)
    local v = prop.value
    if v and v.data then
        return buffer.readu8(v.data, i)
    end
    return -1
end
function writeBytes(prop, s)
    prop.value = s
end
"#,
        )
        .exec()
        .expect("pinned blob script runs");
    let property = || {
        vm.lua()
            .create_userdata(ScriptedPropertyBlob::new(
                model.clone(),
                property_name.clone(),
            ))
            .expect("scripted blob property")
    };
    assert_eq!(
        vm.lua()
            .globals()
            .get::<Function>("readSize")
            .unwrap()
            .call::<i64>(property())
            .unwrap(),
        3
    );
    assert_eq!(
        vm.lua()
            .globals()
            .get::<Function>("readByte")
            .unwrap()
            .call::<i64>((property(), 0))
            .unwrap(),
        10
    );
    vm.lua()
        .globals()
        .get::<Function>("writeBytes")
        .unwrap()
        .call::<()>((property(), "abcd"))
        .expect("write four bytes from a string");
    let asset = model
        .blob_asset(&property_name)
        .expect("updated blob asset");
    assert_eq!(asset.bytes().len(), 4);
    assert_eq!(asset.bytes(), b"abcd");
    assert_eq!(
        vm.lua()
            .globals()
            .get::<Function>("readSize")
            .unwrap()
            .call::<i64>(property())
            .unwrap(),
        4
    );
    assert_eq!(
        vm.lua()
            .globals()
            .get::<Function>("readByte")
            .unwrap()
            .call::<i64>((property(), 0))
            .unwrap(),
        i64::from(b'a')
    );
}
