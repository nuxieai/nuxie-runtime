//! One-for-one ports of
//! `tests/unit_tests/runtime/scripting/scripting_wake_advance_test.cpp`.
#![cfg(all(
    feature = "luau",
    feature = "compiler",
    feature = "upstream-test-seams"
))]

use nuxie_binary::{FixtureProperty, FixtureRecord, FixtureValue, RuntimeFile};
use nuxie_graph::GraphFile;
use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::{ArtboardInstance, NoopScriptHost, StateMachineInstance};
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
    _vm: ScriptVm,
    program: ScriptProgram,
    artboard: ArtboardInstance,
    machine: StateMachineInstance,
}

impl WakeFixture {
    fn pointer(implemented_methods: u32) -> Self {
        Self::new(implemented_methods, scripted_drawable_file(), None)
    }

    fn keyboard(implemented_methods: u32) -> Self {
        Self::new(
            implemented_methods,
            keyboard_scripted_drawable_file(),
            Some(1),
        )
    }

    fn new(implemented_methods: u32, file: RuntimeFile, focus_id: Option<usize>) -> Self {
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

        let graph = GraphFile::from_runtime_file(&file).expect("scripted-input graph builds");
        let mut artboard = ArtboardInstance::from_graph_with_artboards(
            &file,
            graph.artboards.first().expect("scripted-input artboard"),
            &graph.artboards,
        )
        .expect("scripted-input artboard instantiates");
        let global_id = artboard
            .component(1)
            .expect("scripted drawable occurrence")
            .global_id;
        artboard.set_script_instance_for_global_with_implemented_methods(
            global_id,
            instance,
            implemented_methods,
        );
        artboard.update_components();

        let mut machine = artboard
            .state_machine_instance(0)
            .expect("scripted-input state machine");
        if let Some(focus_id) = focus_id {
            machine.set_focus(Some(focus_id));
        }
        machine.mark_scripted_object_initialization_complete(None);

        Self {
            _vm: vm,
            program,
            artboard,
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

fn scripted_drawable_file() -> RuntimeFile {
    RuntimeFile::from_fixture_records(vec![
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
    .expect("scripted-input records import")
}

fn keyboard_scripted_drawable_file() -> RuntimeFile {
    RuntimeFile::from_fixture_records(vec![
        fixture_record("Backboard", Vec::new()),
        fixture_record("Artboard", Vec::new()),
        fixture_record(
            "ScriptedDrawable",
            vec![
                fixture_property("ScriptedDrawable", "parentId", FixtureValue::Uint(0)),
                fixture_property("ScriptedDrawable", "opacity", FixtureValue::Double(1.0)),
            ],
        ),
        fixture_record(
            "FocusData",
            vec![
                fixture_property("FocusData", "parentId", FixtureValue::Uint(1)),
                fixture_property("FocusData", "focusFlags", FixtureValue::Uint(7)),
            ],
        ),
        fixture_record("StateMachine", Vec::new()),
    ])
    .expect("scripted-input records import")
}

fn park_advance_loop(fixture: &mut WakeFixture) {
    let before = fixture.counter("getAdvanceCount");
    fixture
        .artboard
        .advance_script_instances(0.016)
        .expect("first production advance");
    assert_eq!(fixture.counter("getAdvanceCount"), before + 1);
    fixture
        .artboard
        .advance_script_instances(0.016)
        .expect("parked production advance");
    assert_eq!(fixture.counter("getAdvanceCount"), before + 1);
}

#[test]
fn pointer_event_rearms_an_idle_scripted_drawables_advance_loop() {
    let mut drawable = WakeFixture::pointer(ADVANCES | WANTS_POINTER_DOWN);
    park_advance_loop(&mut drawable);

    drawable
        .machine
        .pointer_down(&mut drawable.artboard, 1.0, 1.0, 0);
    assert_eq!(drawable.counter("getPointerDownCount"), 1);

    drawable
        .artboard
        .advance_script_instances(0.016)
        .expect("re-armed production advance");
    assert_eq!(drawable.counter("getAdvanceCount"), 2);
}

#[test]
fn keyboard_event_rearms_an_idle_scripted_drawables_advance_loop() {
    let mut drawable = WakeFixture::keyboard(ADVANCES | WANTS_KEYBOARD_INPUT);
    park_advance_loop(&mut drawable);

    drawable
        .machine
        .key_input(&mut drawable.artboard, 65, 0, true, false);
    assert_eq!(drawable.counter("getKeyCount"), 1);

    drawable
        .artboard
        .advance_script_instances(0.016)
        .expect("re-armed production advance");
    assert_eq!(drawable.counter("getAdvanceCount"), 2);
}
