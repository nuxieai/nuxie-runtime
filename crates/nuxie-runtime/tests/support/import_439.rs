#![allow(dead_code)]
use nuxie_render_api::{NullFactory, PersistentFactory};
pub use nuxie_runtime::{
    Artboard, CoreHandle, File, ImportResult, RuntimeFactoryHandle, RuntimeFileHandle,
};
pub fn import(bytes: &[u8]) -> RuntimeFileHandle {
    let mut factory = PersistentFactory::new(NullFactory::default());
    let mut result = ImportResult::Malformed;
    let file = File::import(
        bytes,
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        Some(&mut result),
        None,
        None,
    );
    assert_eq!(result, ImportResult::Success);
    file.expect("file")
}
pub fn artboard(file: &RuntimeFileHandle) -> CoreHandle {
    file.with_file(File::artboard).expect("artboard")
}
pub fn objects(file: &RuntimeFileHandle) -> Vec<Option<CoreHandle>> {
    artboard(file)
        .with_downcast::<Artboard, _>(|a| a.objects().to_vec())
        .unwrap()
}
