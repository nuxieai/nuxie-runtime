//! The pinned logging factory creates the real VM with the caller's factory.

use nuxie_scripting::make_logging_scripting_context_factory;

#[test]
fn an_empty_sink_does_not_create_a_logging_factory() {
    assert!(make_logging_scripting_context_factory(None).is_none());
}

#[cfg(not(feature = "luau"))]
#[test]
fn scripting_disabled_does_not_create_a_factory_even_with_a_sink() {
    assert!(make_logging_scripting_context_factory(Some(std::sync::Arc::new(|_, _| {}))).is_none());
}

#[cfg(feature = "luau")]
#[test]
fn factory_creates_the_native_vm_with_the_supplied_renderer_and_host_sink() {
    use std::sync::{Arc, Mutex};

    use nuxie_render_api::{PersistentFactory, RecordingFactory};
    use nuxie_runtime::RuntimeFactoryHandle;
    use nuxie_scripting::logging_scripting_context::{ScriptingLogLevel, ScriptingLogSink};

    let lines = Arc::new(Mutex::new(Vec::new()));
    let captured = lines.clone();
    let sink: ScriptingLogSink = Arc::new(move |level, line| {
        captured.lock().unwrap().push((level, line.to_vec()));
    });
    let create_context = make_logging_scripting_context_factory(Some(sink)).unwrap();
    let mut renderer_factory = PersistentFactory::new(RecordingFactory::new());
    let factory = RuntimeFactoryHandle::from_factory(&mut renderer_factory).unwrap();
    let vm = create_context(factory.clone()).expect("the real VM installs its renderer context");
    vm.install_render_factory(&factory)
        .expect("same retained factory identity");

    let mut other_renderer_factory = PersistentFactory::new(RecordingFactory::new());
    let other_factory = RuntimeFactoryHandle::from_factory(&mut other_renderer_factory).unwrap();
    assert!(
        vm.install_render_factory(&other_factory).is_err(),
        "a different factory cannot replace the retained identity"
    );

    let bytecode = support::compile_source("print('native', 7); return {}").unwrap();
    let mut payload = vec![0];
    payload.extend_from_slice(&bytecode);
    vm.with_vm_mut(|vm| {
        vm.install_rive_globals().unwrap();
        vm.register_module("logging-factory", &payload).unwrap();
    });
    assert_eq!(
        lines.lock().unwrap().as_slice(),
        [(ScriptingLogLevel::Info, b"native7".to_vec())]
    );
}

#[cfg(feature = "luau")]
mod support;
