//! Exercise File::register_scripts, recording only its external VM boundary.
use super::*;
use crate::scripting::*;
use crate::source::scripted::script_backend::ScriptBackend;
use nuxie_render_api::{Factory, PersistentFactory};
use std::collections::BTreeMap;

struct RecordingVm {
    backend: ScriptBackend,
    received: Rc<RefCell<BTreeMap<String, ScriptViewModel>>>,
}

impl ScriptingVm for RecordingVm {
    fn script_backend(&self) -> &ScriptBackend {
        &self.backend
    }
    fn install_native_file_assets(&self, _: RuntimeFileWeakHandle) -> Result<(), ScriptError> {
        Ok(())
    }
    fn initialize_data_global(
        &self,
        models: BTreeMap<String, ScriptViewModel>,
    ) -> Result<(), ScriptError> {
        *self.received.borrow_mut() = models;
        Ok(())
    }
    fn install_render_factory(&self, _: &mut dyn Factory) -> Result<(), ScriptError> {
        Ok(())
    }
    fn install_rive_globals(&self) -> Result<(), ScriptError> {
        Ok(())
    }
    fn register_module(&self, _: &str, _: &[u8]) -> Result<(), ScriptError> {
        Ok(())
    }
    fn register_script_assets(
        &self,
        scripts: &[ScriptAssetRegistration<'_>],
    ) -> Vec<ScriptAssetRegistrationResult> {
        scripts
            .iter()
            .map(|_| ScriptAssetRegistrationResult::default())
            .collect()
    }
    fn instantiate_program(
        &self,
        _: &RuntimeScriptProgram,
        _: bool,
        _: Option<ScriptedContextSource>,
        _: Option<ScriptViewModel>,
        _: Vec<Option<ScriptViewModel>>,
        _: &mut dyn ScriptHost,
    ) -> Result<Box<dyn ScriptInstance>, ScriptError> {
        panic!("registration must not instantiate a script")
    }
    fn instantiate_script(
        &self,
        _: &str,
        _: &[u8],
        _: &mut dyn ScriptHost,
    ) -> Result<Box<dyn ScriptInstance>, ScriptError> {
        panic!("registration must not instantiate a script")
    }
}

#[test]
fn native_file_registration_keeps_first_c_string_named_definition() {
    for first_name in ["Dup", "Dup\0first"] {
        let mut factory = PersistentFactory::new(crate::utils::svg_factory::SVGFactory::new(None));
        let mut file = File::new(
            RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
            None,
        );
        let mut first = ViewModel::default();
        first.base.base.base.set_name_value(first_name.to_owned());
        let first = file.core_arena.insert(first);
        let mut second = ViewModel::default();
        second.base.base.base.set_name_value("Dup".to_owned());
        let second = file.core_arena.insert(second);
        file.view_models
            .borrow_mut()
            .extend([first.clone(), second.clone()]);
        // A real ScriptAsset admits the actual native registration path. Only
        // the optional VM engine is replaced by an argument-recording boundary.
        let script = file.core_arena.insert(ScriptAsset::default());
        file.file_assets.push(script);
        let received = Rc::new(RefCell::new(BTreeMap::new()));
        file.set_scripting_vm(Some(RuntimeScriptingVmHandle::new(Box::new(RecordingVm {
            backend: ScriptBackend::default(),
            received: received.clone(),
        }))));
        let file = RuntimeFileHandle::new(file);
        File::register_scripts(&file, None);
        let received = received.borrow();
        assert_eq!(received.len(), 1);
        assert_eq!(received["Dup"].native_model().unwrap(), first);
        assert_ne!(received["Dup"].native_model().unwrap(), second);
    }
}
