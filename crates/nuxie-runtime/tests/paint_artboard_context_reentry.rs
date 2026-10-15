//! Supplemental Rust ownership regression; not an upstream test replacement.
#[path = "support/paint_artboard.rs"]
mod support;
use nuxie_runtime::source::{
    animation::scripted_listener_action::ScriptedListenerAction,
    artboard::RuntimeArtboardInstanceWeakHandle,
    assets::script_asset::ScriptAsset,
    data_bind::data_context::{DataContext, RuntimeDataContextHandle},
    file::RuntimeFileWeakHandle,
    lua::scripting_vm::RuntimeScriptingVmHandle,
    scripted::{script_backend::ScriptBackend, scripted_object::INITS_BIT},
};
use nuxie_runtime::{Artboard, CoreHandle, RuntimeFactoryHandle};
use nuxie_runtime::{
    RuntimeScriptProgram, ScriptAssetRegistration, ScriptAssetRegistrationResult, ScriptError,
    ScriptHost, ScriptInstance, ScriptMethod, ScriptValue, ScriptViewModel, ScriptedContextSource,
    ScriptingVm,
};
use std::{cell::Cell, collections::BTreeMap, rc::Rc};

// The existing VM-neutral ScriptInstance hook invokes real runtime layout
// invalidation. Only script execution is supplied here; context propagation,
// ScriptAsset initialization and host callbacks are the production owners.
struct DirtyLayoutOnInit {
    child: RuntimeArtboardInstanceWeakHandle,
    calls: Rc<Cell<u32>>,
}
impl ScriptInstance for DirtyLayoutOnInit {
    fn has_method(&self, method: ScriptMethod) -> Result<bool, ScriptError> {
        Ok(method == ScriptMethod::Init)
    }
    fn call_method(
        &mut self,
        method: ScriptMethod,
        _: &[ScriptValue],
        _: &mut dyn ScriptHost,
    ) -> Result<ScriptValue, ScriptError> {
        assert_eq!(method, ScriptMethod::Init);
        let child = self
            .child
            .upgrade()
            .expect("paint source remains alive during init");
        let root = child.core_handle();
        child.with_artboard_mut(|a| a.mark_layout_dirty(root));
        self.calls.set(self.calls.get() + 1);
        Ok(ScriptValue::Bool(true))
    }
    fn get_input(&self, _: &str) -> Result<ScriptValue, ScriptError> {
        Ok(ScriptValue::Nil)
    }
    fn set_input(&mut self, _: &str, _: ScriptValue) -> Result<(), ScriptError> {
        Ok(())
    }
}
struct InitVm {
    backend: ScriptBackend,
    child: RuntimeArtboardInstanceWeakHandle,
    calls: Rc<Cell<u32>>,
}
impl ScriptingVm for InitVm {
    fn script_backend(&self) -> &ScriptBackend {
        &self.backend
    }
    fn instantiate_program(
        &self,
        _: &RuntimeScriptProgram,
        present: bool,
        _: Option<ScriptedContextSource>,
        _: Option<ScriptViewModel>,
        _: Vec<Option<ScriptViewModel>>,
        _: &mut dyn ScriptHost,
    ) -> Result<Box<dyn ScriptInstance>, ScriptError> {
        assert!(
            present,
            "binding must install context before initializing the script"
        );
        Ok(Box::new(DirtyLayoutOnInit {
            child: self.child.clone(),
            calls: self.calls.clone(),
        }))
    }
    fn install_native_file_assets(&self, _: RuntimeFileWeakHandle) -> Result<(), ScriptError> {
        unreachable!()
    }
    fn initialize_data_global(
        &self,
        _: BTreeMap<String, ScriptViewModel>,
    ) -> Result<(), ScriptError> {
        unreachable!()
    }
    fn install_render_factory(
        &self,
        _: &mut dyn nuxie_render_api::Factory,
    ) -> Result<(), ScriptError> {
        unreachable!()
    }
    fn install_rive_globals(&self) -> Result<(), ScriptError> {
        unreachable!()
    }
    fn register_module(&self, _: &str, _: &[u8]) -> Result<(), ScriptError> {
        unreachable!()
    }
    fn register_script_assets(
        &self,
        _: &[ScriptAssetRegistration<'_>],
    ) -> Vec<ScriptAssetRegistrationResult> {
        unreachable!()
    }
    fn instantiate_script(
        &self,
        _: &str,
        _: &[u8],
        _: &mut dyn ScriptHost,
    ) -> Result<Box<dyn ScriptInstance>, ScriptError> {
        unreachable!()
    }
}

#[test]
fn binding_paint_source_context_allows_script_init_to_dirty_child_layout() {
    let factory = RuntimeFactoryHandle::from_factory(
        &mut nuxie_render_api::PersistentFactory::new(nuxie_render_api::NullFactory),
    )
    .unwrap();
    let source = support::Scene::source(factory.clone(), 100.0, 50.0);
    let host = support::Scene::host(factory, Some(source.artboard.clone()), 10.0, 10.0, 0);
    let instance = host.instance();
    let paint = support::paint_of(&instance);
    let child = support::inner(&paint);
    let arena = child.core_handle().retain_arena().unwrap();
    let calls = Rc::new(Cell::new(0));
    let vm = RuntimeScriptingVmHandle::new(Box::new(InitVm {
        backend: ScriptBackend::default(),
        child: child.downgrade(),
        calls: calls.clone(),
    }));
    let mut asset = ScriptAsset::default();
    asset.set_scripting_vm(Some(vm));
    asset.set_serialized_implemented_methods(INITS_BIT);
    asset.registration_complete_native(Some(RuntimeScriptProgram::from_backend(1_u32)));
    let asset = arena.insert(asset);
    let script: CoreHandle = arena.insert(ScriptedListenerAction::default());
    script
        .with_downcast_mut::<ScriptedListenerAction, _>(|a| {
            a.scripted.set_asset(script.clone(), Some(asset))
        })
        .unwrap();
    child.with_artboard_mut(|a| a.add_scripted_object(script));
    assert_eq!(calls.get(), 0);
    // Before the occurrence fix, this path held PaintImage's mutable borrow
    // while child script initialization synchronously called its host again.
    let context = RuntimeDataContextHandle::new(DataContext::new(None));
    Artboard::internal_data_context_handle(&instance.core_handle(), context);
    assert_eq!(calls.get(), 1);
    assert_eq!(support::inner(&paint).core_handle(), child.core_handle());
}
