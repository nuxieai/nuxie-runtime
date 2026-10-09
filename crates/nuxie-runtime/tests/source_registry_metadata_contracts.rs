//! Native property schema and capability/validation contracts from pinned160085.
use nuxie_runtime::source::{
    artboard::Artboard,
    assets::script_module_asset::ScriptModuleAsset,
    constraints::scrolling::scroll_constraint::ScrollConstraint,
    core::{CoreArena, CoreHandle},
    core_context::CoreContext,
    generated::{
        assets::script_module_asset_base::ScriptModuleAssetBase, core_registry::CoreRegistry,
        shapes::paint::paint_image_base::PaintImageBase, text::text_input_base::TextInputBase,
    },
    shapes::paint::paint_image::PaintImage,
    text::{text_input::TextInput, text_variation_modifier::TextVariationModifier},
    viewmodel::viewmodel_instance_asset::ViewModelInstanceAsset,
};
#[test]
fn paint_image_transform_properties_report_double_wire_type() {
    let mut failures = Vec::new();
    for key in [
        PaintImageBase::IMAGE_SCALE_X_PROPERTY_KEY,
        PaintImageBase::IMAGE_SCALE_Y_PROPERTY_KEY,
        PaintImageBase::IMAGE_OFFSET_X_PROPERTY_KEY,
        PaintImageBase::IMAGE_OFFSET_Y_PROPERTY_KEY,
        PaintImageBase::IMAGE_ROTATION_PROPERTY_KEY,
    ] {
        let actual = CoreRegistry::property_field_id(key.into());
        if actual != 2 {
            failures.push((key, actual));
        }
    }
    assert!(failures.is_empty(), "wrong double wire types: {failures:?}");
}
#[test]
fn supported_new_native_fields_have_schema_and_owner_metadata() {
    let arena = CoreArena::default();
    let text = arena.insert(TextInput::default());
    let script = arena.insert(ScriptModuleAsset::default());
    let mut failures = Vec::new();
    for (owner, key, field_type) in [
        (&text, TextInputBase::ALIGN_VALUE_PROPERTY_KEY, 0),
        (&text, TextInputBase::VERTICAL_ALIGN_VALUE_PROPERTY_KEY, 0),
        (&text, TextInputBase::OBSCURED_PROPERTY_KEY, 4),
        (&script, ScriptModuleAssetBase::LANGUAGE_PROPERTY_KEY, 0),
    ] {
        let actual = CoreRegistry::property_field_id(key.into());
        let supports = owner.with(|o| CoreRegistry::object_supports_property(o, key.into()));
        if actual != field_type || supports != Some(true) {
            failures.push((key, field_type, actual, supports));
        }
    }
    assert!(failures.is_empty(), "wrong native metadata: {failures:?}");
}
#[test]
fn native_base_projection_is_present_for_asset_and_text_variation() {
    let arena = CoreArena::default();
    let asset = arena.insert(ViewModelInstanceAsset::default());
    let modifier = arena.insert(TextVariationModifier::default());
    let projections = [
        asset.with(|o| o.as_view_model_instance_asset().is_some()),
        asset.with_mut(|o| o.as_view_model_instance_asset_mut().is_some()),
        modifier.with(|o| o.as_text_modifier().is_some()),
        modifier.with_mut(|o| o.as_text_modifier_mut().is_some()),
    ];
    assert_eq!(projections, [Some(true); 4]);
}
struct Context<'a> {
    arena: &'a CoreArena,
    parent: Option<CoreHandle>,
}
impl CoreContext for Context<'_> {
    fn core_arena(&self) -> &CoreArena {
        self.arena
    }
    fn resolve_handle(&self, _id: u32) -> Option<CoreHandle> {
        self.parent.clone()
    }
}
#[test]
fn scroll_validation_uses_component_parent_contract() {
    let arena = CoreArena::default();
    let scroll = arena.insert(ScrollConstraint::default());
    let mut missing = Context {
        arena: &arena,
        parent: None,
    };
    let missing_result = scroll.with_mut(|o| o.validate(&mut missing));
    let wrong = arena.insert(PaintImage::default());
    let mut wrong = Context {
        arena: &arena,
        parent: Some(wrong),
    };
    let wrong_result = scroll.with_mut(|o| o.validate(&mut wrong));
    let valid = arena.insert(Artboard::default());
    let mut valid = Context {
        arena: &arena,
        parent: Some(valid),
    };
    let valid_result = scroll.with_mut(|o| o.validate(&mut valid));
    assert_eq!(
        [missing_result, wrong_result, valid_result],
        [Some(false), Some(false), Some(true)]
    );
}
