//! Native ScriptedObject lifecycle assertions from pinned
//! scripted_object.cpp::ensureScriptInitialized/hydrateScriptInputs/tryLuaUserInit.
//! Only the approved VM boundary is recorded; prerequisites and input writes
//! run through the real ScriptAsset, ScriptInput, and ScriptedObject owners.

use super::*;
use crate::source::{
    animation::scripted_listener_action::ScriptedListenerAction,
    artboard::Artboard,
    assets::script_asset::ScriptAsset,
    core::{CoreArena, CoreHandle},
    data_bind::data_context::{DataContext, RuntimeDataContextHandle},
    file::RuntimeFileWeakHandle,
    generated::{
        component_base::ComponentBase, core_registry::CoreRegistry,
        custom_property_number_base::CustomPropertyNumberBase,
    },
    lua::scripting_vm::RuntimeScriptingVmHandle,
    script_input_artboard::ScriptInputArtboard,
    script_input_number::ScriptInputNumber,
    script_input_viewmodel_property::ScriptInputViewModelProperty,
    scripted::scripted_object::{INITS_BIT, ScriptUpdateRequestHost, ScriptedObject},
};

#[derive(Debug, PartialEq)]
enum LifecycleEvent {
    Generate { context_present: bool },
    Input(String, ScriptValue),
    Init,
    Invalidate,
}

struct RecordingScript {
    owner: CoreHandle,
    events: Rc<RefCell<Vec<LifecycleEvent>>>,
    init_succeeds: Rc<Cell<bool>>,
    live: bool,
}

impl ScriptInstance for RecordingScript {
    fn has_method(&self, method: ScriptMethod) -> Result<bool, ScriptError> {
        Ok(method == ScriptMethod::Init)
    }

    fn call_method(
        &mut self,
        method: ScriptMethod,
        _args: &[ScriptValue],
        _host: &mut dyn ScriptHost,
    ) -> Result<ScriptValue, ScriptError> {
        assert_eq!(method, ScriptMethod::Init);
        assert!(
            self.owner
                .with(|owner| owner.as_scripted_object().is_some())
                .unwrap()
        );
        self.events.borrow_mut().push(LifecycleEvent::Init);
        Ok(ScriptValue::Bool(self.init_succeeds.get()))
    }

    fn get_input(&self, _name: &str) -> Result<ScriptValue, ScriptError> {
        Ok(ScriptValue::Nil)
    }

    fn set_input(&mut self, name: &str, value: ScriptValue) -> Result<(), ScriptError> {
        assert!(self.live, "disposed script table cannot receive writes");
        assert!(
            self.owner
                .with(|owner| owner.as_scripted_object().is_some())
                .unwrap()
        );
        self.events
            .borrow_mut()
            .push(LifecycleEvent::Input(name.into(), value));
        Ok(())
    }

    fn script_lifetime_valid(&self) -> bool {
        self.live
    }

    fn invalidate_for_init_retry(&mut self) {
        self.events.borrow_mut().push(LifecycleEvent::Invalidate);
        self.live = false;
    }
}

struct RecordingVm {
    owner: CoreHandle,
    events: Rc<RefCell<Vec<LifecycleEvent>>>,
    init_succeeds: Rc<Cell<bool>>,
}

impl ScriptingVm for RecordingVm {
    fn instantiate_program(
        &self,
        program: &RuntimeScriptProgram,
        context_present: bool,
        view_model: Option<ScriptViewModel>,
        parents: Vec<Option<ScriptViewModel>>,
        _host: &mut dyn ScriptHost,
    ) -> Result<Box<dyn ScriptInstance>, ScriptError> {
        assert_eq!(program.backend::<u32>(), Some(&1));
        assert!(view_model.is_none());
        assert!(parents.is_empty());
        assert!(
            self.owner
                .with(|owner| owner.as_scripted_object().is_some())
                .unwrap()
        );
        self.events
            .borrow_mut()
            .push(LifecycleEvent::Generate { context_present });
        Ok(Box::new(RecordingScript {
            owner: self.owner.clone(),
            events: self.events.clone(),
            init_succeeds: self.init_succeeds.clone(),
            live: true,
        }))
    }

    fn install_native_file_assets(&self, _file: RuntimeFileWeakHandle) -> Result<(), ScriptError> {
        panic!("this owner-local test does not import a File")
    }
    fn initialize_data_global(
        &self,
        _models: BTreeMap<String, ScriptViewModel>,
    ) -> Result<(), ScriptError> {
        panic!("this owner-local test does not install Data globals")
    }
    fn install_render_factory(&self, _factory: &mut dyn RenderFactory) -> Result<(), ScriptError> {
        panic!("this non-rendering script does not install a factory")
    }
    fn install_rive_globals(&self) -> Result<(), ScriptError> {
        panic!("the generator is already registered")
    }
    fn register_module(&self, _name: &str, _payload: &[u8]) -> Result<(), ScriptError> {
        panic!("the generator is already registered")
    }
    fn register_script_assets(
        &self,
        _scripts: &[ScriptAssetRegistration<'_>],
    ) -> Vec<ScriptAssetRegistrationResult> {
        panic!("the generator is already registered")
    }
    fn instantiate_script(
        &self,
        _name: &str,
        _payload: &[u8],
        _host: &mut dyn ScriptHost,
    ) -> Result<Box<dyn ScriptInstance>, ScriptError> {
        panic!("native ScriptAsset must instantiate its retained program")
    }
}

struct NativeHydration {
    arena: CoreArena,
    owner: CoreHandle,
    events: Rc<RefCell<Vec<LifecycleEvent>>>,
    init_succeeds: Rc<Cell<bool>>,
}

impl NativeHydration {
    fn new() -> Self {
        let arena = CoreArena::default();
        let owner = arena.insert(ScriptedListenerAction::default());
        let events = Rc::new(RefCell::new(Vec::new()));
        let init_succeeds = Rc::new(Cell::new(true));
        let vm = RuntimeScriptingVmHandle::new(Box::new(RecordingVm {
            owner: owner.clone(),
            events: events.clone(),
            init_succeeds: init_succeeds.clone(),
        }));
        let mut asset = ScriptAsset::default();
        asset.set_scripting_vm(Some(vm));
        asset.set_serialized_implemented_methods(INITS_BIT);
        asset.registration_complete_native(Some(RuntimeScriptProgram::from_backend(1_u32)));
        let asset = arena.insert(asset);
        owner
            .with_downcast_mut::<ScriptedListenerAction, _>(|action| {
                action.scripted.set_asset(owner.clone(), Some(asset));
                action
                    .scripted
                    .set_data_context(Some(RuntimeDataContextHandle::new(DataContext::new(None))));
            })
            .unwrap();
        Self {
            arena,
            owner,
            events,
            init_succeeds,
        }
    }

    fn add_property(&self, property: CoreHandle) {
        self.owner
            .with_downcast_mut::<ScriptedListenerAction, _>(|action| {
                action.add_property(property);
            })
            .unwrap();
    }

    fn add_number(&self) {
        let input = self.arena.insert(ScriptInputNumber::default());
        assert!(CoreRegistry::set_string_handle(
            &input,
            i32::from(ComponentBase::NAME_PROPERTY_KEY),
            "before".into(),
        ));
        assert!(CoreRegistry::set_double_handle(
            &input,
            i32::from(CustomPropertyNumberBase::PROPERTY_VALUE_PROPERTY_KEY),
            7.0,
        ));
        self.add_property(input);
    }

    fn properties(&self) -> Vec<CoreHandle> {
        ScriptedObject::custom_properties(&self.owner)
    }

    fn initialize(&self) -> bool {
        ScriptedObject::initialize_occurrence(
            &self.owner,
            &self.properties(),
            &mut ScriptUpdateRequestHost::default(),
        )
    }

    fn hydrate(&self) -> bool {
        ScriptedObject::hydrate_occurrence(
            &self.owner,
            &self.properties(),
            &mut ScriptUpdateRequestHost::default(),
        )
    }

    fn reinit(&self) -> bool {
        ScriptedObject::reinit_occurrence(
            &self.owner,
            &self.properties(),
            &mut ScriptUpdateRequestHost::default(),
        )
    }

    fn user_init_done(&self) -> bool {
        self.owner
            .with(|owner| owner.as_scripted_object().unwrap().user_lua_init_done())
            .unwrap()
    }
}

#[test]
fn all_artboard_prerequisites_precede_every_native_input_write() {
    let fixture = NativeHydration::new();
    fixture.add_number();
    fixture.add_property(fixture.arena.insert(ScriptInputArtboard::default()));

    // Upstream artboard cold-init validation permits table creation; the
    // missing artboard is rejected by the complete hydration preflight.
    assert!(fixture.initialize());
    assert!(!fixture.hydrate());
    assert_eq!(
        &*fixture.events.borrow(),
        &[LifecycleEvent::Generate {
            context_present: true
        }]
    );
    assert!(!fixture.user_init_done());
    assert!(
        fixture
            .owner
            .with(|owner| owner.as_scripted_object().unwrap().self_ref() != 0)
            .unwrap()
    );
}

#[test]
fn unresolved_view_model_prerequisite_precedes_the_earlier_scalar_setter() {
    let fixture = NativeHydration::new();
    fixture.add_number();
    fixture.add_property(
        fixture
            .arena
            .insert(ScriptInputViewModelProperty::default()),
    );

    assert!(fixture.initialize());
    assert!(!fixture.hydrate());
    assert_eq!(
        &*fixture.events.borrow(),
        &[LifecycleEvent::Generate {
            context_present: true
        }]
    );
    assert!(!fixture.user_init_done());
}

#[test]
fn absent_native_script_lifetime_rejects_hydration_without_writes() {
    let fixture = NativeHydration::new();
    fixture.add_number();
    fixture.add_property(fixture.arena.insert(ScriptInputArtboard::default()));

    assert!(!fixture.hydrate());
    assert!(fixture.events.borrow().is_empty());
    assert!(!fixture.user_init_done());
}

#[test]
fn native_hydration_writes_inputs_before_one_time_user_init() {
    let fixture = NativeHydration::new();
    fixture.add_number();

    assert!(fixture.reinit());
    assert!(fixture.user_init_done());
    assert_eq!(
        &*fixture.events.borrow(),
        &[
            LifecycleEvent::Generate {
                context_present: true
            },
            LifecycleEvent::Input("before".into(), ScriptValue::Number(7.0)),
            LifecycleEvent::Init,
        ]
    );
    fixture.events.borrow_mut().clear();
    assert!(fixture.reinit());
    assert_eq!(
        &*fixture.events.borrow(),
        &[LifecycleEvent::Input(
            "before".into(),
            ScriptValue::Number(7.0)
        ),]
    );
}

#[test]
fn failed_native_init_disposes_before_retry_recreates_and_rehydrates() {
    let fixture = NativeHydration::new();
    fixture.add_number();
    fixture.init_succeeds.set(false);

    assert!(!fixture.reinit());
    assert!(!fixture.user_init_done());
    assert_eq!(
        &*fixture.events.borrow(),
        &[
            LifecycleEvent::Generate {
                context_present: true
            },
            LifecycleEvent::Input("before".into(), ScriptValue::Number(7.0)),
            LifecycleEvent::Init,
            LifecycleEvent::Invalidate,
        ]
    );
    assert_eq!(
        fixture
            .owner
            .with(|owner| owner.as_scripted_object().unwrap().self_ref()),
        Some(0)
    );

    fixture.events.borrow_mut().clear();
    fixture.init_succeeds.set(true);
    assert!(fixture.reinit());
    assert!(fixture.user_init_done());
    assert_eq!(
        &*fixture.events.borrow(),
        &[
            LifecycleEvent::Generate {
                context_present: true
            },
            LifecycleEvent::Input("before".into(), ScriptValue::Number(7.0)),
            LifecycleEvent::Init,
        ]
    );
}

#[test]
fn disposed_native_lifetime_returns_before_rich_artboard_construction() {
    let fixture = NativeHydration::new();
    assert!(fixture.initialize());
    fixture
        .owner
        .with_mut(|owner| owner.as_scripted_object_mut().unwrap().script_dispose())
        .unwrap();
    fixture.events.borrow_mut().clear();

    let source = fixture.arena.insert(Artboard::default());
    assert!(source.remove_occurrence());
    // The source is deliberately stale and this ScriptAsset has no File.
    // Reaching construction would fail; the source m_self guard precedes it.
    ScriptedObject::set_artboard_input_occurrence(&fixture.owner, "panel".into(), source);
    assert!(!fixture.hydrate());
    assert!(fixture.events.borrow().is_empty());
}
