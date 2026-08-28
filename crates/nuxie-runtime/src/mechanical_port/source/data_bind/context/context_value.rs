use crate::RuntimeBlobAsset;
use crate::mechanical_port::source::data_bind::context::context_target_value::{
    DataBindContextTargetValue, TargetBinding,
};
use crate::mechanical_port::source::{
    core::CoreHandle, data_bind::data_values::data_value::DataValue, text_engine::FontRef,
};
use nuxie_render_api::RenderImage;
use std::rc::Rc;
use std::sync::Arc;
pub trait ContextBinding: TargetBinding {
    fn to_source(&self) -> bool;
    fn initial_source_value(&self) -> Option<Box<dyn DataValue>>;
    fn sync_source_value(&self, value: &mut dyn DataValue);
    fn convert(&mut self, input: &dyn DataValue, is_main_direction: bool) -> Box<dyn DataValue>;
    fn suppress_dirt(&mut self, value: bool);
    fn apply_source_value(&mut self, value: &dyn DataValue);
}
pub trait ContextApplyBinding: ContextBinding {
    fn set_bool(&mut self, property_key: u32, value: bool);
    fn set_color(&mut self, property_key: u32, value: i32);
    fn set_double(&mut self, property_key: u32, value: f32);
    fn set_uint(&mut self, property_key: u32, value: u32);
    fn set_string(&mut self, property_key: u32, value: String);
    fn target_is_solo(&self) -> bool;
    fn solo_update_by_index(&mut self, index: usize);
    fn solo_update_by_name(&mut self, name: String);
    fn update_list(&mut self, items: &[CoreHandle]);
    fn update_view_model(&mut self, value: Option<CoreHandle>);
    fn target_is_bindable_view_model(&self) -> bool;
    fn set_bindable_view_model(&mut self, value: Option<CoreHandle>);
    fn bindable_view_model_property_key(&self) -> u32;
    fn pointer_key(&self, value: &CoreHandle) -> u32;
    fn source_uint(&self) -> u32;
    fn source_artboard(&self) -> Option<CoreHandle>;
    fn update_artboard(&mut self, source: Option<CoreHandle>);
    fn resolved_image_asset(&self) -> Option<CoreHandle>;
    fn resolved_font_asset(&self) -> Option<CoreHandle>;
    fn resolved_blob_asset(&self) -> Option<CoreHandle>;
    fn source_image_asset(&self) -> Option<CoreHandle>;
    fn source_font_asset(&self) -> Option<CoreHandle>;
    fn source_image(&self) -> Option<Rc<dyn RenderImage>>;
    fn source_font(&self) -> Option<FontRef>;
    fn source_blob(&self) -> Option<Arc<RuntimeBlobAsset>>;
    fn set_target_image_asset(&mut self, asset: CoreHandle);
    fn set_target_font_asset(&mut self, asset: CoreHandle);
    fn set_bindable_image(&mut self, image: Option<Rc<dyn RenderImage>>);
    fn set_bindable_font(&mut self, font: Option<FontRef>);
    fn set_bindable_blob(&mut self, blob: Option<Arc<RuntimeBlobAsset>>);
    fn set_view_model_image(&mut self, image: Option<Rc<dyn RenderImage>>);
    fn set_view_model_font(&mut self, font: Option<FontRef>);
    fn set_view_model_blob(&mut self, blob: Option<Arc<RuntimeBlobAsset>>);
}
pub struct DataBindContextValue {
    data_value: Option<Box<dyn DataValue>>,
    target_value: DataBindContextTargetValue,
    is_valid: bool,
}
impl DataBindContextValue {
    pub fn new(data_bind: &mut dyn ContextBinding) -> Self {
        let mut target_value = DataBindContextTargetValue::default();
        if data_bind.to_source() {
            target_value.initialize(data_bind);
        }
        let data_value = data_bind.initial_source_value();
        Self {
            data_value,
            target_value,
            is_valid: false,
        }
    }
    pub fn invalidate(&mut self) {
        self.is_valid = false
    }
    pub fn refresh_target_value(&mut self, data_bind: &dyn ContextBinding) {
        if data_bind.to_source() {
            self.target_value.sync_target_value(data_bind);
        }
    }
    pub fn sync_target_value(&mut self, _target: Option<CoreHandle>, _property_key: u32) -> bool {
        false
    }
    pub fn sync_source_value(&mut self, data_bind: &dyn ContextBinding) {
        if let Some(value) = self.data_value.as_deref_mut() {
            data_bind.sync_source_value(value);
        }
    }
    pub fn calculate_untyped_data_value(
        &mut self,
        input: &dyn DataValue,
        is_main_direction: bool,
        data_bind: &mut dyn ContextBinding,
    ) -> Box<dyn DataValue> {
        data_bind.convert(input, is_main_direction)
    }
    pub fn apply_to_source(
        &mut self,
        _component: Option<CoreHandle>,
        _property_key: u32,
        is_main_direction: bool,
        data_bind: &mut dyn ContextBinding,
    ) {
        if self.target_value.sync_target_value(data_bind) || !self.is_valid {
            if let Some(target_value) = self.target_value.data_value() {
                let converted = data_bind.convert(target_value, is_main_direction);
                data_bind.suppress_dirt(true);
                data_bind.apply_source_value(converted.as_ref());
                data_bind.suppress_dirt(false);
                self.is_valid = true;
            }
        }
    }
    pub fn data_value(&self) -> Option<&dyn DataValue> {
        self.data_value.as_deref()
    }
    pub fn data_value_mut(&mut self) -> Option<&mut dyn DataValue> {
        self.data_value.as_deref_mut()
    }
}
