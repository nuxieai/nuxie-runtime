use crate::mechanical_port::source::{
    bindable_artboard::RuntimeBindableArtboardHandle,
    core::CoreHandle,
    factory::RuntimeFactoryHandle,
    file::{File, ImportResult, RuntimeFileHandle},
    lua::scripting_vm::RuntimeScriptingVmHandle,
};

/// The host supplies a lazy native VM factory through the inverted dependency
/// boundary; decoded content always uses the strict signature policy.
pub fn import_decoded_file(
    bytes: &[u8],
    factory: RuntimeFactoryHandle,
    result: Option<&mut ImportResult>,
    make_vm: Option<&mut dyn FnMut(RuntimeFactoryHandle) -> RuntimeScriptingVmHandle>,
) -> Option<RuntimeFileHandle> {
    File::import_decoded(bytes, factory, result, make_vm)
}

#[derive(Default)]
pub struct DecodedBindable {
    pub artboard: Option<RuntimeBindableArtboardHandle>,
    pub view_model: Option<CoreHandle>,
}

pub fn decoded_bindable(file: &File, name: Option<&str>) -> DecodedBindable {
    let artboard = match name {
        Some(name) => file.bindable_artboard_named(name),
        None => file.bindable_artboard_default(),
    };
    let view_model = artboard.as_ref().and_then(|artboard| {
        file.create_default_view_model_instance_for_artboard(
            artboard.artboard_handle().core_handle(),
        )
    });
    DecodedBindable {
        artboard,
        view_model,
    }
}
