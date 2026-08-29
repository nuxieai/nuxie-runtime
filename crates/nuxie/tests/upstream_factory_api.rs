//! Contract checks for the direct, factory-required upstream API.

use nuxie::{
    Factory, File, FileAssetLoaderRef, ImportResult, PersistentFactory, RecordingFactory,
    RuntimeArtboardInstanceHandle, RuntimeFactoryHandle, RuntimeFileHandle,
    RuntimeScriptingVmHandle,
};

#[test]
fn import_requires_factory_and_reports_native_failure() {
    let import: fn(
        &[u8],
        RuntimeFactoryHandle,
        Option<&mut ImportResult>,
        Option<FileAssetLoaderRef>,
        Option<RuntimeScriptingVmHandle>,
    ) -> Option<RuntimeFileHandle> = File::import;
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let retained = RuntimeFactoryHandle::from_factory(&mut factory).expect("retained factory");
    let mut result = ImportResult::Success;

    assert!(import(b"", retained, Some(&mut result), None, None).is_none());
    assert_eq!(result, ImportResult::Malformed);
}

#[test]
fn factory_handle_retains_the_original_allocation_context() {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let identity = factory
        .persistent_context()
        .expect("factory context")
        .identity();
    let retained = RuntimeFactoryHandle::from_factory(&mut factory).expect("retained factory");
    drop(factory);

    assert_eq!(retained.persistent_context().identity(), identity);
    retained.with_factory_mut(|factory| {
        let _paint = factory.make_render_paint();
    });
}

#[test]
fn draw_accepts_renderer_without_a_late_factory() {
    let _draw: fn(&RuntimeArtboardInstanceHandle, &mut nuxie::runtime::renderer::Renderer) =
        RuntimeArtboardInstanceHandle::draw;
}
