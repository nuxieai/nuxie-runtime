//! One-for-one ports of
//! `tests/unit_tests/runtime/scripting/scripting_wake_advance_test.cpp`.
#![cfg(all(
    feature = "luau",
    feature = "compiler",
    feature = "upstream-test-seams"
))]

use nuxie_binary::{FixtureProperty, FixtureRecord, FixtureValue};
use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::{
    NoopScriptHost, RuntimeScriptingVmHandle,
    source::{
        advance_flags::AdvanceFlags,
        animation::state_machine_instance::HitComponent,
        assets::script_asset::ScriptAsset,
        core::{CoreArena, CoreHandle},
        factory::RuntimeFactoryHandle,
        file::{File, RuntimeFileHandle},
        input::focusable::{Key, KeyModifiers},
        listener_type::ListenerType,
        math::vec2d::Vec2D,
        scripted::{
            scripted_drawable::{HitScriptedDrawable, ScriptedDrawable},
            scripted_object::ScriptedObject,
        },
    },
};
use nuxie_schema::definition_by_name;
use nuxie_scripting::vm::{ScriptProgram, ScriptVm};

mod support;
use support::compile_source;

const WAKE_SCRIPT: &str = r#"type MyDrawing = {}
local advanceCount = 0
local pointerDownCount = 0
local keyCount = 0

function init(self: MyDrawing, context: Context): boolean
  return true
end

function advance(self: MyDrawing, seconds: number): boolean
  advanceCount += 1
  return false -- idle immediately
end

function pointerDown(self: MyDrawing, event: PointerEvent)
  pointerDownCount += 1
end

function keyboardEvent(self: MyDrawing, event: KeyboardEvent): boolean
  keyCount += 1
  return false
end

function getAdvanceCount(): number
  return advanceCount
end

function getPointerDownCount(): number
  return pointerDownCount
end

function getKeyCount(): number
  return keyCount
end

return function(): Node<MyDrawing>
  return {
    init = init,
    advance = advance,
    pointerDown = pointerDown,
    keyboardEvent = keyboardEvent,
  }
end
"#;

const ADVANCES: u32 = 1 << 0;
const WANTS_POINTER_DOWN: u32 = 1 << 3;
const WANTS_KEYBOARD_INPUT: u32 = 1 << 16;

struct WakeFixture {
    _vm: RuntimeScriptingVmHandle,
    _file: RuntimeFileHandle,
    _arena: CoreArena,
    program: ScriptProgram,
    drawable: CoreHandle,
    machine: nuxie_runtime::RuntimeStateMachineInstanceHandle,
}

impl WakeFixture {
    fn pointer(implemented_methods: u32) -> Self {
        Self::new(implemented_methods)
    }

    fn keyboard(implemented_methods: u32) -> Self {
        Self::new(implemented_methods)
    }

    fn new(implemented_methods: u32) -> Self {
        let bytecode = compile_source(WAKE_SCRIPT).expect("wake script compiles");
        let mut payload = Vec::with_capacity(bytecode.len() + 1);
        payload.push(0);
        payload.extend(bytecode);

        let vm = ScriptVm::new();
        let mut factory = PersistentFactory::new(RecordingFactory::new());
        let program = vm
            .register_protocol_script_with_factory("wake-advance", &payload, &mut factory)
            .expect("wake script registers");
        let mut instance = vm
            .instantiate_registered_script_with_context(&program, None, Vec::new())
            .expect("wake script instantiates");
        assert!(instance.call_init(&mut NoopScriptHost).unwrap());
        let vm = RuntimeScriptingVmHandle::new(Box::new(vm));

        let file = File::import(
            &scripted_drawable_file(),
            RuntimeFactoryHandle::from_factory(&mut factory).expect("retained recording factory"),
            None,
            None,
            None,
        )
        .expect("scripted-input records import through native File");
        let artboard = file
            .with_file(File::artboard_default)
            .expect("scripted-input artboard instantiates");
        let machine = artboard
            .state_machine_at(0)
            .expect("scripted-input state machine");
        // Upstream deliberately uses a standalone ScriptedDrawable. Keep the
        // translated owner standalone too; the native machine is retained
        // only for HitScriptedDrawable's exact pointer-dispatch seam.
        let arena = CoreArena::default();
        let drawable = arena.insert(ScriptedDrawable::default());
        let asset = drawable
            .insert_sibling(ScriptAsset::default())
            .expect("standalone ScriptAsset owner");
        let drawable_owner = drawable.clone();
        drawable
            .with_mut(|object| {
                let drawable = object
                    .as_scripted_drawable_mut()
                    .expect("scripted drawable owner");
                drawable.scripted.set_asset(drawable_owner, Some(asset));
                drawable
                    .scripted
                    .install_script_instance(instance, vm.clone());
                drawable
                    .scripted
                    .set_implemented_methods(implemented_methods);
            })
            .expect("scripted drawable remains live");
        drawable
            .with(|object| {
                let drawable = object
                    .as_scripted_drawable()
                    .expect("scripted drawable owner");
                assert!(!drawable.base.base.base.base.base.is_collapsed());
                assert!(drawable.scripted.advances());
            })
            .expect("scripted drawable remains live");
        Self {
            _vm: vm,
            _file: file,
            _arena: arena,
            program,
            drawable,
            machine,
        }
    }

    fn counter(&self, getter: &str) -> i32 {
        self.program
            .upstream_test_module_i32_getter(getter)
            .unwrap_or_else(|error| panic!("read {getter}: {error}"))
    }
}

fn fixture_property(type_name: &str, property_name: &str, value: FixtureValue) -> FixtureProperty {
    let definition = definition_by_name(type_name).expect("fixture type exists");
    let property = std::iter::once(definition.name)
        .chain(definition.ancestors.iter().copied())
        .filter_map(definition_by_name)
        .flat_map(|owner| owner.properties)
        .find(|property| property.name == property_name)
        .unwrap_or_else(|| panic!("missing {type_name}.{property_name}"));
    FixtureProperty {
        key: property.key.int,
        value,
    }
}

fn fixture_record(type_name: &str, properties: Vec<FixtureProperty>) -> FixtureRecord {
    FixtureRecord {
        type_key: definition_by_name(type_name)
            .expect("fixture type exists")
            .type_key
            .int,
        properties,
    }
}

fn push_var_uint(bytes: &mut Vec<u8>, mut value: u64) {
    loop {
        let mut byte = (value & 0x7f) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        bytes.push(byte);
        if value == 0 {
            break;
        }
    }
}

fn encode_fixture_records(records: &[FixtureRecord]) -> Vec<u8> {
    let mut bytes = b"RIVE".to_vec();
    push_var_uint(&mut bytes, 7);
    push_var_uint(&mut bytes, 0);
    push_var_uint(&mut bytes, 0x4e55_5849);
    push_var_uint(&mut bytes, 0);
    for record in records {
        push_var_uint(&mut bytes, u64::from(record.type_key));
        for property in &record.properties {
            push_var_uint(&mut bytes, u64::from(property.key));
            match &property.value {
                FixtureValue::Bool(value) => bytes.push(u8::from(*value)),
                FixtureValue::Bytes(value) => {
                    push_var_uint(&mut bytes, value.len() as u64);
                    bytes.extend_from_slice(value);
                }
                FixtureValue::Color(value) => bytes.extend_from_slice(&value.to_le_bytes()),
                FixtureValue::Double(value) => bytes.extend_from_slice(&value.to_le_bytes()),
                FixtureValue::Int(value) => {
                    let encoded = ((*value as u32) << 1) ^ ((*value >> 31) as u32);
                    push_var_uint(&mut bytes, u64::from(encoded));
                }
                FixtureValue::String(value) => {
                    push_var_uint(&mut bytes, value.len() as u64);
                    bytes.extend_from_slice(value.as_bytes());
                }
                FixtureValue::Uint(value) => push_var_uint(&mut bytes, *value),
            }
        }
        push_var_uint(&mut bytes, 0);
    }
    bytes
}

fn scripted_drawable_file() -> Vec<u8> {
    encode_fixture_records(&[
        fixture_record("Backboard", Vec::new()),
        fixture_record("Artboard", Vec::new()),
        fixture_record(
            "ScriptedDrawable",
            vec![
                fixture_property("ScriptedDrawable", "parentId", FixtureValue::Uint(0)),
                fixture_property("ScriptedDrawable", "opacity", FixtureValue::Double(1.0)),
            ],
        ),
        fixture_record("StateMachine", Vec::new()),
    ])
}

fn park_advance_loop(fixture: &mut WakeFixture) {
    let before = fixture.counter("getAdvanceCount");
    ScriptedDrawable::advance_occurrence(
        &fixture.drawable,
        0.016,
        AdvanceFlags(
            AdvanceFlags::ANIMATE.0 | AdvanceFlags::NEW_FRAME.0 | AdvanceFlags::ADVANCE_NESTED.0,
        ),
    );
    assert_eq!(fixture.counter("getAdvanceCount"), before + 1);
    ScriptedDrawable::advance_occurrence(
        &fixture.drawable,
        0.016,
        AdvanceFlags(
            AdvanceFlags::ANIMATE.0 | AdvanceFlags::NEW_FRAME.0 | AdvanceFlags::ADVANCE_NESTED.0,
        ),
    );
    assert_eq!(fixture.counter("getAdvanceCount"), before + 1);
}

#[test]
fn pointer_event_rearms_an_idle_scripted_drawables_advance_loop() {
    let mut drawable = WakeFixture::pointer(ADVANCES | WANTS_POINTER_DOWN);
    park_advance_loop(&mut drawable);

    let hit = HitScriptedDrawable::new(drawable.drawable.clone());
    drawable.machine.with_instance_mut(|machine| {
        hit.process_event(
            machine,
            Vec2D::new(1.0, 1.0),
            ListenerType::Down,
            true,
            0.0,
            0,
        );
    });
    assert_eq!(drawable.counter("getPointerDownCount"), 1);

    ScriptedDrawable::advance_occurrence(
        &drawable.drawable,
        0.016,
        AdvanceFlags(
            AdvanceFlags::ANIMATE.0 | AdvanceFlags::NEW_FRAME.0 | AdvanceFlags::ADVANCE_NESTED.0,
        ),
    );
    assert_eq!(drawable.counter("getAdvanceCount"), 2);
}

#[test]
fn keyboard_event_rearms_an_idle_scripted_drawables_advance_loop() {
    let mut drawable = WakeFixture::keyboard(ADVANCES | WANTS_KEYBOARD_INPUT);
    park_advance_loop(&mut drawable);

    ScriptedDrawable::key_input_occurrence(
        &drawable.drawable,
        Key::A,
        KeyModifiers::NONE,
        true,
        false,
    );
    assert_eq!(drawable.counter("getKeyCount"), 1);

    ScriptedDrawable::advance_occurrence(
        &drawable.drawable,
        0.016,
        AdvanceFlags(
            AdvanceFlags::ANIMATE.0 | AdvanceFlags::NEW_FRAME.0 | AdvanceFlags::ADVANCE_NESTED.0,
        ),
    );
    assert_eq!(drawable.counter("getAdvanceCount"), 2);
}
