//! Public contract after the explicit upstream-shaped API decision.
//!
//! The former test was a symbol census of the removed facade, not a behavioral
//! test. That compatibility contract is intentionally superseded; the native
//! lifecycle and behavioral tests exercise the owners exposed here.

use crate::{
    File, FileAssetLoaderRef, ImportResult, RuntimeArtboardInstanceHandle, RuntimeFactoryHandle,
    RuntimeFileHandle, RuntimeScriptingVmHandle, RuntimeStateMachineInstanceHandle,
    source::{
        animation::state_machine_instance::StateMachineInstance, renderer::Renderer,
        viewmodel::runtime::viewmodel_instance_runtime::ViewModelInstanceRuntime,
    },
};

#[test]
fn import_requires_factory_and_draw_uses_retained_resources() {
    let _: fn(
        &[u8],
        RuntimeFactoryHandle,
        Option<&mut ImportResult>,
        Option<FileAssetLoaderRef>,
        Option<RuntimeScriptingVmHandle>,
    ) -> Option<RuntimeFileHandle> = File::import;
    let _: fn(&RuntimeArtboardInstanceHandle, &mut Renderer<'_>) =
        RuntimeArtboardInstanceHandle::draw;
}

#[test]
fn native_owners_are_public() {
    macro_rules! methods_are_reachable {
        ($owner:ty; $($method:ident),+ $(,)?) => {
            $(let _ = <$owner>::$method;)+
        };
    }
    methods_are_reachable!(File;
        artboard_count, artboard_default, artboard_at, artboard_named,
        view_model_by_name, view_model_by_index,
    );
    methods_are_reachable!(StateMachineInstance;
        pointer_down, pointer_up, pointer_move, pointer_exit,
        reset, advance,
        focus_manager, focus_state, enable_semantics, semantic_manager, set_focus,
        input_count, input, bool_input, number_input, trigger_input,
        name, state_machine, artboard, needs_advance,
    );
    methods_are_reachable!(RuntimeStateMachineInstanceHandle;
        advance_and_apply, advance_and_apply_view_models,
    );
    methods_are_reachable!(ViewModelInstanceRuntime;
        property_number, property_string, property_boolean, property_color,
        property_enum, property_trigger, property_list, property_list_index,
        property_image, property_font, property_blob, property_artboard,
        property_view_model, property, replace_view_model, properties,
    );
}
