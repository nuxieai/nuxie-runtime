//! Source cases from scripting_property_lifetime_test.cpp at a637bc5e.
use super::super::{LuaScriptInstance, ScriptVm};
use super::*;
use nuxie_runtime::ScriptInstance;
use nuxie_runtime::source::{
    core::{CoreArena, CoreHandle},
    file::{File, RuntimeFileHandle},
    generated::{
        core_registry::CoreRegistry, viewmodel::viewmodel_component_base::ViewModelComponentBase,
    },
    viewmodel::{
        viewmodel::ViewModel, viewmodel_instance::ViewModelInstance,
        viewmodel_instance_boolean::ViewModelInstanceBoolean,
        viewmodel_instance_number::ViewModelInstanceNumber,
        viewmodel_instance_viewmodel::ViewModelInstanceViewModel,
        viewmodel_property_boolean::ViewModelPropertyBoolean,
        viewmodel_property_number::ViewModelPropertyNumber,
        viewmodel_property_viewmodel::ViewModelPropertyViewModel,
    },
};

const CACHING_SCRIPT: &str = r#"
local cached = {}
function init(screen: ViewModel)
  cached.counter = screen:getNumber('DelayCounter')
  cached.enabled = screen:getBoolean('DelayEnabled')
end
function bump()
  cached.enabled.value = true
  cached.counter.value += 1
  return cached.counter.value, cached.enabled.value
end
function probe(name: string) return cached[name] end
"#;

struct Models {
    arena: CoreArena,
    file: RuntimeFileHandle,
    screen_model: CoreHandle,
    counter_property: CoreHandle,
    enabled_property: CoreHandle,
}
struct Screen {
    instance: CoreHandle,
    counter: CoreHandle,
    enabled: CoreHandle,
}

// scripting_properties_test.cpp: wrappers created/read from coroutine stacks
// survive collection and dispatch listeners back through the owning VM.
#[test]
fn view_model_properties_work_from_async_bodies() {
    let models = Models::new();
    let screen = models.screen();
    name(&models.counter_property, "n");
    let model = models.facade(&screen.instance);
    assert!(model.set_number("n", 7.0));
    let vm = ScriptVm::new();
    vm.install_rive_globals().unwrap();
    vm.lua
        .load(
            r#"
results = {}
function run(model: ViewModel)
    async(function()
        results.method = model:getNumber("n").value
        results.index = model.n.value
        model:getNumber("n"):addListener(function()
            results.heard = model.n.value
        end)
        results.kept = model:getNumber("n")
        return nil
    end)
    return nil
end
function readKept(): number
    return results.kept.value
end
"#,
        )
        .exec()
        .unwrap();
    vm.lua
        .globals()
        .get::<Function>("run")
        .unwrap()
        .call::<()>(create_scripted_view_model(&vm.lua, model.clone()).unwrap())
        .unwrap();
    vm.lua.gc_collect().unwrap();
    let results: Table = vm.lua.globals().get("results").unwrap();
    assert_eq!(results.get::<f32>("method").unwrap(), 7.0);
    assert_eq!(results.get::<f32>("index").unwrap(), 7.0);
    model.set_number("n", 9.0);
    assert_eq!(results.get::<f32>("heard").unwrap(), 9.0);
    assert_eq!(
        vm.lua
            .globals()
            .get::<Function>("readKept")
            .unwrap()
            .call::<f32>(())
            .unwrap(),
        9.0
    );
}
fn name(handle: &CoreHandle, value: &str) {
    assert!(CoreRegistry::set_string_handle(
        handle,
        ViewModelComponentBase::NAME_PROPERTY_KEY.into(),
        value.into()
    ));
}
impl Models {
    fn new() -> Self {
        let mut factory =
            nuxie_render_api::PersistentFactory::new(nuxie_render_api::RecordingFactory::new());
        let file = RuntimeFileHandle::new(File::new(
            nuxie_runtime::RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
            None,
        ));
        let arena = file.with_file(|file| file.core_arena().clone());
        let screen_model = arena.insert(ViewModel::default());
        let counter_property = arena.insert(ViewModelPropertyNumber::default());
        let enabled_property = arena.insert(ViewModelPropertyBoolean::default());
        name(&screen_model, "Screen");
        name(&counter_property, "DelayCounter");
        name(&enabled_property, "DelayEnabled");
        screen_model
            .with_downcast_mut::<ViewModel, _>(|model| {
                model.add_property(counter_property.clone());
                model.add_property(enabled_property.clone());
            })
            .unwrap();
        Self {
            arena,
            file,
            screen_model,
            counter_property,
            enabled_property,
        }
    }
    fn screen(&self) -> Screen {
        let instance = self.arena.insert(ViewModelInstance::default());
        instance
            .with_downcast_mut::<ViewModelInstance, _>(|v| v.view_model(self.screen_model.clone()))
            .unwrap();
        let counter = self.arena.insert(ViewModelInstanceNumber::default());
        let enabled = self.arena.insert(ViewModelInstanceBoolean::default());
        for (value, property) in [
            (&counter, &self.counter_property),
            (&enabled, &self.enabled_property),
        ] {
            value
                .with_mut(|v| {
                    v.as_view_model_instance_value_mut()
                        .unwrap()
                        .set_view_model_property(property.clone())
                })
                .unwrap();
            instance
                .with_downcast_mut::<ViewModelInstance, _>(|v| v.add_value(value.clone()))
                .unwrap();
        }
        Screen {
            instance,
            counter,
            enabled,
        }
    }
    fn facade(&self, instance: &CoreHandle) -> ScriptViewModel {
        ScriptViewModel::from_native(instance.clone(), self.file.clone()).unwrap()
    }
    fn owner(&self, screen: &Screen) -> (CoreHandle, CoreHandle) {
        let model = self.arena.insert(ViewModel::default());
        name(&model, "Screens");
        let property = self.arena.insert(ViewModelPropertyViewModel::default());
        name(&property, "screen");
        model
            .with_downcast_mut::<ViewModel, _>(|v| v.add_property(property.clone()))
            .unwrap();
        let instance = self.arena.insert(ViewModelInstance::default());
        instance
            .with_downcast_mut::<ViewModelInstance, _>(|v| v.view_model(model))
            .unwrap();
        let value = self.arena.insert(ViewModelInstanceViewModel::default());
        value
            .with_downcast_mut::<ViewModelInstanceViewModel, _>(|v| {
                v.base.set_view_model_property(property);
                v.set_parent_view_model_instance(Some(instance.clone()));
            })
            .unwrap();
        instance
            .with_downcast_mut::<ViewModelInstance, _>(|v| v.add_value(value.clone()))
            .unwrap();
        value
            .with_downcast_mut::<ViewModelInstanceViewModel, _>(|v| {
                v.set_reference_view_model_instance(Some(screen.instance.clone()))
            })
            .unwrap();
        (instance, value)
    }
}
fn counter(screen: &Screen) -> f32 {
    screen
        .counter
        .with_downcast::<ViewModelInstanceNumber, _>(|v| v.value())
        .unwrap()
}
fn enabled(screen: &Screen) -> bool {
    screen
        .enabled
        .with_downcast::<ViewModelInstanceBoolean, _>(|v| v.value())
        .unwrap()
}
fn bump(lua: &Lua) -> (f32, bool) {
    lua.globals()
        .get::<Function>("bump")
        .unwrap()
        .call(())
        .unwrap()
}
fn top(lua: &Lua) -> i32 {
    // Read-only source stack-balance assertion; Lua owns the retained thread.
    unsafe { luaur_vm::functions::lua_gettop::lua_gettop(lua.current_thread().state()) }
}

fn initialized(vm: &ScriptVm, model: ScriptViewModel, both: bool) -> LuaScriptInstance {
    let source = if both {
        r#"
function init(self, context: Context): boolean
  local vm = context:viewModel()
  if vm == nil then return false end
  self.counter = vm:getNumber('DelayCounter')
  self.enabled = vm:getBoolean('DelayEnabled')
  return true
end
return function() return { init = init, counter = nil, enabled = nil } end
"#
    } else {
        r#"
function init(self, context: Context): boolean
  local vm = context:viewModel()
  if vm == nil then return false end
  self.counter = vm:getNumber('DelayCounter')
  return true
end
return function() return { init = init, counter = nil } end
"#
    };
    let generator: Function = vm.lua.load(source).eval().unwrap();
    let table: Table = generator.call(()).unwrap();
    let model = Rc::new(RefCell::new(Some(model)));
    let missing = Rc::new(Cell::new(false));
    let context = ScriptedContext::new(&vm.lua, model.clone(), Vec::new(), missing.clone(), None);
    let owner = context.listener_owner();
    let mut instance = LuaScriptInstance::new(table);
    instance.context = Some(vm.lua.create_userdata(context).unwrap());
    instance.context_view_model = model;
    instance.context_present.set(true);
    instance.context_missing_requested_data = missing;
    instance.property_listener_owner = owner;
    assert!(
        instance
            .call_init(&mut nuxie_runtime::NoopScriptHost)
            .unwrap()
    );
    instance
}

#[test]
fn cached_property_keeps_its_own_view_model_across_swap() {
    let models = Models::new();
    let first = models.screen();
    let second = models.screen();
    let (owner, value) = models.owner(&first);
    let lua = Lua::new();
    lua.load(
        r#"
local cached = {}
function init(screens: ViewModel)
  local screen = screens:getViewModel('screen').value
  cached.counter = screen:getNumber('DelayCounter')
  cached.enabled = screen:getBoolean('DelayEnabled')
end
function bump()
  cached.enabled.value = true
  cached.counter.value += 1
  return cached.counter.value, cached.enabled.value
end
function reresolve(screens: ViewModel) init(screens) end
function probe(name: string) return cached[name] end
"#,
    )
    .exec()
    .unwrap();
    let initial_top = top(&lua);
    let wrapper = create_scripted_view_model(&lua, models.facade(&owner)).unwrap();
    let wrapper_ref = lua.create_registry_value(wrapper).unwrap();
    lua.globals()
        .get::<Function>("init")
        .unwrap()
        .call::<()>(lua.registry_value::<Table>(&wrapper_ref).unwrap())
        .unwrap();
    bump(&lua);
    assert_eq!(counter(&first), 1.0);
    assert!(ViewModelInstance::replace_view_model_property_occurrence(
        &owner,
        &value,
        Some(second.instance.clone())
    ));
    lua.gc_collect().unwrap();
    assert_eq!(bump(&lua), (2.0, true));
    assert_eq!(counter(&first), 2.0);
    // CoreHandle is a weak arena identity, unlike an owning C++ rcp. The
    // wrapper's own retained instance is checked directly after the swap.
    let cached_counter: AnyUserData = lua
        .globals()
        .get::<Function>("probe")
        .unwrap()
        .call("counter")
        .unwrap();
    assert_eq!(
        scripted_property_state(&cached_counter)
            .unwrap()
            .owning_instance(),
        Some(first.instance.clone())
    );
    drop(cached_counter);
    assert_eq!(
        first
            .counter
            .with(|v| v
                .as_view_model_instance_value()
                .unwrap()
                .view_model_instance())
            .flatten(),
        Some(first.instance.clone())
    );
    assert_eq!(counter(&second), 0.0);
    lua.globals()
        .get::<Function>("reresolve")
        .unwrap()
        .call::<()>(lua.registry_value::<Table>(&wrapper_ref).unwrap())
        .unwrap();
    bump(&lua);
    assert_eq!(counter(&second), 1.0);
    assert!(enabled(&second));
    assert_eq!(counter(&first), 2.0);
    lua.remove_registry_value(wrapper_ref).unwrap();
    assert_eq!(top(&lua), initial_top);
}

#[test]
fn init_time_properties_belong_to_scripted_object() {
    let models = Models::new();
    let screen = models.screen();
    let vm = ScriptVm::new();
    let instance = initialized(&vm, models.facade(&screen.instance), true);
    assert_eq!(instance.property_listener_owner.tracked_property_count(), 2);
    for name in ["counter", "enabled"] {
        let property: AnyUserData = instance.table().get(name).unwrap();
        let state = scripted_property_state(&property).unwrap();
        assert!(
            state
                .owner()
                .unwrap()
                .ptr_eq(&instance.property_listener_owner)
        );
    }
}

#[cfg(feature = "tools")]
#[test]
fn untagged_orphan_sweep_spares_claimed_properties() {
    let models = Models::new();
    let screen = models.screen();
    let lua = Lua::new();
    lua.load(CACHING_SCRIPT).exec().unwrap();
    let context = ScriptViewModelFrameContext::for_lua(&lua);
    let initial_top = top(&lua);
    let wrapper = create_scripted_view_model(&lua, models.facade(&screen.instance)).unwrap();
    let wrapper_ref = lua.create_registry_value(wrapper).unwrap();
    context.set_orphan_owner_tag(7);
    lua.globals()
        .get::<Function>("init")
        .unwrap()
        .call::<()>(lua.registry_value::<Table>(&wrapper_ref).unwrap())
        .unwrap();
    context.set_orphan_owner_tag(0);
    assert_eq!(bump(&lua).0, 1.0);
    context.dispose_orphan_scripted_properties(false);
    assert_eq!(bump(&lua), (2.0, true));
    assert_eq!(counter(&screen), 2.0);
    context.dispose_orphan_scripted_properties_for_tag(7);
    assert_eq!(bump(&lua).0, 0.0);
    lua.remove_registry_value(wrapper_ref).unwrap();
    assert_eq!(top(&lua), initial_top);
}

#[test]
fn scripted_objects_do_not_share_property_wrappers() {
    let models = Models::new();
    let screen = models.screen();
    let vm = ScriptVm::new();
    let first = initialized(&vm, models.facade(&screen.instance), false);
    assert_eq!(first.property_listener_owner.tracked_property_count(), 1);
    let first_counter: AnyUserData = first.table().get("counter").unwrap();
    let second = initialized(&vm, models.facade(&screen.instance), false);
    assert_eq!(second.property_listener_owner.tracked_property_count(), 1);
    let second_counter: AnyUserData = second.table().get("counter").unwrap();
    assert_ne!(first_counter.to_pointer(), second_counter.to_pointer());
    let first_property = first_counter.borrow::<ScriptedPropertyNumber>().unwrap();
    let second_property = second_counter.borrow::<ScriptedPropertyNumber>().unwrap();
    assert_eq!(
        first_property.model.native_instance(),
        second_property.model.native_instance()
    );
    assert_eq!(first_property.name, second_property.name);
    assert_eq!(
        first_property.model.native_instance(),
        Some(screen.instance.clone())
    );
    drop(first_property);
    drop(second_property);
    drop(first);
    let state = scripted_property_state(&second_counter).unwrap();
    assert!(!state.disposed());
    assert_eq!(state.owning_instance(), Some(screen.instance.clone()));
    vm.lua
        .globals()
        .set("secondCounter", second_counter)
        .unwrap();
    vm.lua.load("secondCounter.value = 4").exec().unwrap();
    assert_eq!(counter(&screen), 4.0);
}

#[cfg(feature = "tools")]
#[test]
fn disposed_nested_view_model_property_is_reminted() {
    let models = Models::new();
    let screen = models.screen();
    let (owner, value) = models.owner(&screen);
    let lua = Lua::new();
    lua.load("function resolve(owner: ViewModel) return owner:getViewModel('screen') end")
        .exec()
        .unwrap();
    let initial_top = top(&lua);
    let wrapper = create_scripted_view_model(&lua, models.facade(&owner)).unwrap();
    let owner_ref = lua.create_registry_value(wrapper).unwrap();
    let resolve: Function = lua.globals().get("resolve").unwrap();
    let first: AnyUserData = resolve
        .call(lua.registry_value::<Table>(&owner_ref).unwrap())
        .unwrap();
    let state = scripted_property_state(&first).unwrap();
    state.dispose();
    assert!(state.disposed());
    let second: AnyUserData = resolve
        .call(lua.registry_value::<Table>(&owner_ref).unwrap())
        .unwrap();
    assert!(!scripted_property_state(&second).unwrap().disposed());
    let property = second.borrow::<ScriptedPropertyViewModel>().unwrap();
    let actual = property
        .parent
        .native_instance()
        .unwrap()
        .with_downcast::<ViewModelInstance, _>(|v| v.property_value_named(&property.name))
        .flatten();
    assert_eq!(actual, Some(value));
    drop(property);
    lua.remove_registry_value(owner_ref).unwrap();
    assert_eq!(top(&lua), initial_top);
}

#[test]
fn failed_generator_properties_remain_owned_until_occurrence_teardown() {
    use nuxie_runtime::source::scripted::scripted_drawable::ScriptedDrawable;

    for fail_with_error in [false, true] {
        for install_successfully in [false, true] {
            let models = Models::new();
            let screen = models.screen();
            let occurrence = models.arena.insert(ScriptedDrawable::default());
            let vm = Rc::new(ScriptVm::new());
            let wrapper =
                create_scripted_view_model(&vm.lua, models.facade(&screen.instance)).unwrap();
            vm.lua.globals().set("screen", wrapper).unwrap();
            let source = if fail_with_error {
                "return function(context) escaped = screen:getNumber('DelayCounter'); error('failed generator') end"
            } else {
                "return function(context) escaped = screen:getNumber('DelayCounter'); return false end"
            };
            let generator: Function = vm.lua.load(source).eval().unwrap();
            let program = super::super::ScriptProgram { generator };
            let instantiate = |program: &super::super::ScriptProgram| {
                vm.instantiate_registered_script_with_optional_factory_and_context(
                    program,
                    None,
                    None,
                    Vec::new(),
                    Some(nuxie_runtime::ScriptedContextSource::for_occurrence(
                        occurrence.clone(),
                    )),
                    None,
                    None,
                )
            };
            occurrence
                .with_mut(|owner| {
                    owner
                        .as_scripted_object_mut()
                        .unwrap()
                        .prepare_cold_script_init()
                })
                .unwrap();
            let result = instantiate(&program);
            assert!(result.is_err());
            let escaped: AnyUserData = vm.lua.globals().get("escaped").unwrap();
            let state = scripted_property_state(&escaped).unwrap();
            assert!(
                !state.disposed(),
                "failed generator does not eagerly dispose the object's properties"
            );
            vm.lua.load("escaped.value = 3").exec().unwrap();
            assert_eq!(counter(&screen), 3.0);
            // A failed instantiate never installed m_vm. Another cold retry
            // therefore does not dispose properties created by that attempt.
            occurrence
                .with_mut(|owner| {
                    owner
                        .as_scripted_object_mut()
                        .unwrap()
                        .prepare_cold_script_init()
                })
                .unwrap();
            assert!(!state.disposed());
            assert!(instantiate(&program).is_err());
            assert!(!state.disposed());
            if install_successfully {
                occurrence
                    .with_mut(|owner| {
                        owner
                            .as_scripted_object_mut()
                            .unwrap()
                            .prepare_cold_script_init()
                    })
                    .unwrap();
                let generator: Function = vm
                    .lua
                    .load("return function(context) return {} end")
                    .eval()
                    .unwrap();
                let instance = instantiate(&super::super::ScriptProgram { generator }).unwrap();
                let vm_handle =
                    nuxie_runtime::source::lua::scripting_vm::RuntimeScriptingVmHandle::new(
                        Box::new(vm.clone()),
                    );
                occurrence
                    .with_mut(|owner| {
                        owner
                            .as_scripted_object_mut()
                            .unwrap()
                            .install_script_instance(instance, vm_handle)
                    })
                    .unwrap();
                assert!(
                    !state.disposed(),
                    "successful first install retains earlier failed-attempt properties"
                );
            }
            occurrence
                .with_mut(|owner| owner.as_scripted_object_mut().unwrap().script_dispose())
                .unwrap();
            assert!(state.disposed());
            let value: f32 = vm
                .lua
                .load("escaped.value = 4; return escaped.value")
                .eval()
                .unwrap();
            assert_eq!(value, 0.0);
            assert_eq!(counter(&screen), 3.0);
        }
    }
}

#[test]
fn failed_user_init_properties_survive_cold_retries_until_occurrence_teardown() {
    use nuxie_runtime::NoopScriptHost;
    use nuxie_runtime::source::{
        lua::scripting_vm::RuntimeScriptingVmHandle,
        scripted::{
            scripted_drawable::ScriptedDrawable,
            scripted_object::{INITS_BIT, ScriptedObject},
        },
    };

    // ScriptedObject::tryUserInit releases self/context and unregisters the
    // VM on false/error, but leaves tracked properties for actual teardown.
    for fail_with_error in [false, true] {
        let models = Models::new();
        let screen = models.screen();
        let occurrence = models.arena.insert(ScriptedDrawable::default());
        let vm = Rc::new(ScriptVm::new());
        let source = if fail_with_error {
            "return function() return { init = function(self, context) escaped = screen:getNumber('DelayCounter'); error('failed user init') end } end"
        } else {
            "return function() return { init = function(self, context) escaped = screen:getNumber('DelayCounter'); return false end } end"
        };
        let generator: Function = vm.lua.load(source).eval().unwrap();
        let program = super::super::ScriptProgram { generator };
        let vm_handle = RuntimeScriptingVmHandle::new(Box::new(vm.clone()));
        let mut escaped_states = Vec::new();

        for attempt in 1..=2 {
            occurrence
                .with_mut(|owner| {
                    owner
                        .as_scripted_object_mut()
                        .unwrap()
                        .prepare_cold_script_init();
                })
                .unwrap();
            // Each init resolves a distinct wrapper, just as separate Context
            // lifetimes do, while both properties refer to the same native value.
            let wrapper =
                create_scripted_view_model(&vm.lua, models.facade(&screen.instance)).unwrap();
            vm.lua.globals().set("screen", wrapper).unwrap();
            let instance = vm
                .instantiate_registered_script_with_optional_factory_and_context(
                    &program,
                    None,
                    None,
                    Vec::new(),
                    Some(nuxie_runtime::ScriptedContextSource::for_occurrence(
                        occurrence.clone(),
                    )),
                    None,
                    None,
                )
                .unwrap();
            occurrence
                .with_mut(|owner| {
                    let owner = owner.as_scripted_object_mut().unwrap();
                    owner.install_script_instance(instance, vm_handle.clone());
                    owner.set_implemented_methods(INITS_BIT);
                })
                .unwrap();
            assert!(!ScriptedObject::hydrate_occurrence(
                &occurrence,
                &[],
                &mut NoopScriptHost
            ));
            let escaped: AnyUserData = vm.lua.globals().get("escaped").unwrap();
            let state = scripted_property_state(&escaped).unwrap();
            assert!(!state.disposed());
            escaped_states.push(state);
            vm.lua
                .globals()
                .set(format!("escaped{attempt}"), escaped)
                .unwrap();
            vm.lua.load("escaped.value += 1").exec().unwrap();
            assert_eq!(counter(&screen), attempt as f32);
            for state in &escaped_states {
                assert!(
                    !state.disposed(),
                    "cold retry must retain earlier failed-init properties"
                );
            }
        }

        let values: (f32, f32) = vm
            .lua
            .load("escaped1.value = 7; escaped2.value += 1; return escaped1.value, escaped2.value")
            .eval()
            .unwrap();
        assert_eq!(values, (8.0, 8.0));
        assert_eq!(counter(&screen), 8.0);
        occurrence
            .with_mut(|owner| owner.as_scripted_object_mut().unwrap().script_dispose())
            .unwrap();
        for state in escaped_states {
            assert!(state.disposed());
        }
        let values: (f32, f32) = vm
            .lua
            .load("escaped1.value = 9; escaped2.value = 10; return escaped1.value, escaped2.value")
            .eval()
            .unwrap();
        assert_eq!(values, (0.0, 0.0));
        assert_eq!(counter(&screen), 8.0);
    }
}

#[test]
fn stale_property_cleanup_does_not_dispose_a_reused_weak_registry_slot() {
    let models = Models::new();
    let screen = models.screen();
    let lua = Lua::new();
    let first_wrapper = create_scripted_view_model(&lua, models.facade(&screen.instance)).unwrap();
    let first: AnyUserData = first_wrapper.get("DelayCounter").unwrap();
    let old_state = scripted_property_state(&first).unwrap();
    let old_slot = property_weak_key(first.to_pointer());
    drop(first);
    drop(first_wrapper);
    lua.gc_collect().unwrap();

    let second_wrapper = create_scripted_view_model(&lua, models.facade(&screen.instance)).unwrap();
    let second: AnyUserData = second_wrapper.get("DelayCounter").unwrap();
    let second_state = scripted_property_state(&second).unwrap();
    assert!(!Rc::ptr_eq(&old_state, &second_state));
    // Deterministically model a later Lua allocation reusing the old address.
    // A lingering property watch must not tear down the new occupant.
    let weak_properties: Table = lua
        .named_registry_value("rive_weak_scripted_properties")
        .unwrap();
    weak_properties.raw_set(old_slot, second.clone()).unwrap();
    old_state.dispose();
    assert!(!second_state.disposed());
    assert_eq!(
        second_state.owning_instance(),
        Some(screen.instance.clone())
    );
    lua.globals().set("second", second).unwrap();
    lua.load("second.value = 6").exec().unwrap();
    assert_eq!(counter(&screen), 6.0);
}

#[test]
#[cfg(target_pointer_width = "64")]
fn tagged_property_pointer_round_trips_through_weak_table() {
    let lua = Lua::new();
    let weak_properties = lua.create_table();
    let meta = lua.create_table();
    meta.set("__mode", "v").unwrap();
    weak_properties.set_metatable(Some(meta)).unwrap();
    let tagged = 0xB400_0071_2345_6780_u64 as usize as *const std::ffi::c_void;
    let untagged = 0x0000_0071_2345_6780_u64 as usize as *const std::ffi::c_void;
    let tagged_value = lua.create_table();
    let untagged_value = lua.create_table();
    let adjacent = 0xB400_0071_2345_6790_u64 as usize as *const std::ffi::c_void;
    let adjacent_value = lua.create_table();
    weak_properties
        .raw_set(property_weak_key(tagged), tagged_value.clone())
        .unwrap();
    weak_properties
        .raw_set(property_weak_key(untagged), untagged_value.clone())
        .unwrap();
    weak_properties
        .raw_set(property_weak_key(adjacent), adjacent_value.clone())
        .unwrap();
    let actual: Table = weak_properties
        .raw_get(property_weak_key(adjacent))
        .unwrap();
    assert_eq!(actual.to_pointer(), adjacent_value.to_pointer());
    let actual: Table = weak_properties.raw_get(property_weak_key(tagged)).unwrap();
    assert_eq!(actual.to_pointer(), tagged_value.to_pointer());
    let actual: Table = weak_properties
        .raw_get(property_weak_key(untagged))
        .unwrap();
    assert_eq!(actual.to_pointer(), untagged_value.to_pointer());
    assert_eq!(
        property_weak_key(tagged).0 as usize as u64,
        0xB400_0071_2345_6780
    );
}
