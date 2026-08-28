use crate::mechanical_port::source::data_bind::data_values::{
    data_type::DataType, data_value::DataValue, data_value_asset_blob::DataValueAssetBlob,
    data_value_asset_font::DataValueAssetFont, data_value_asset_image::DataValueAssetImage,
    data_value_boolean::DataValueBoolean, data_value_color::DataValueColor,
    data_value_integer::DataValueInteger, data_value_number::DataValueNumber,
    data_value_string::DataValueString, data_value_viewmodel::DataValueViewModel,
};
use crate::mechanical_port::source::text_engine::FontRef;
use crate::{RuntimeBlobAsset, mechanical_port::source::core::CoreHandle};
use nuxie_render_api::RenderImage;
use std::rc::Rc;
use std::sync::Arc;
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FieldType {
    Uint,
    Color,
    Double,
    String,
    Bool,
    Other,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SourceKind {
    Artboard,
    AssetImage,
    AssetFont,
    AssetBlob,
    ViewModel,
    Other,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TargetKind {
    Solo,
    BindableAsset,
    BindableViewModel,
    ViewModelInstanceViewModel,
    ArtboardReferencer,
    TextStyle,
    Image,
    ViewModelAssetImage,
    ViewModelAssetFont,
    ViewModelAssetBlob,
    Other,
}
pub trait TargetBinding {
    fn has_target(&self) -> bool;
    fn field_type(&self) -> FieldType;
    fn is_solo_active_property(&self) -> bool;
    fn source_output_type(&self) -> DataType;
    fn source_kind(&self) -> SourceKind;
    fn target_kind(&self) -> TargetKind;
    fn uint_value(&self) -> u32;
    fn color_value(&self) -> i32;
    fn double_value(&self) -> f32;
    fn string_value(&self) -> String;
    fn bool_value(&self) -> bool;
    fn active_child_name(&self) -> String;
    fn active_child_index(&self) -> u32;
    fn enum_index_for_name(&self, name: &str) -> Option<u32>;
    fn image_value(&self) -> Option<Rc<dyn RenderImage>>;
    fn font_value(&self) -> Option<FontRef>;
    fn blob_value(&self) -> Option<Arc<RuntimeBlobAsset>>;
    fn view_model_value(&self) -> Option<CoreHandle>;
}
#[derive(Default)]
pub struct DataBindContextTargetValue {
    target_value: Option<Box<dyn DataValue>>,
}
impl DataBindContextTargetValue {
    pub fn initialize(&mut self, binding: &dyn TargetBinding) {
        self.target_value = match binding.field_type() {
            FieldType::Uint => {
                if binding.is_solo_active_property() {
                    match binding.source_output_type() {
                        DataType::String => Some(Box::new(DataValueString::default())),
                        DataType::Number => Some(Box::new(DataValueNumber::default())),
                        DataType::Enum => Some(Box::new(DataValueInteger::default())),
                        _ => None,
                    }
                } else {
                    Some(match binding.source_kind() {
                        SourceKind::AssetImage => {
                            Box::new(DataValueAssetImage::default()) as Box<dyn DataValue>
                        }
                        SourceKind::AssetFont => Box::new(DataValueAssetFont::default()),
                        SourceKind::AssetBlob => Box::new(DataValueAssetBlob::default()),
                        SourceKind::ViewModel => Box::new(DataValueViewModel::default()),
                        SourceKind::Artboard | SourceKind::Other => {
                            Box::new(DataValueInteger::default())
                        }
                    })
                }
            }
            FieldType::Color => Some(Box::new(DataValueColor::default())),
            FieldType::Double => Some(Box::new(DataValueNumber::default())),
            FieldType::String => Some(Box::new(DataValueString::default())),
            FieldType::Bool => Some(Box::new(DataValueBoolean::default())),
            FieldType::Other => None,
        };
    }
    fn update_integer(&mut self, value: u32) -> bool {
        let target = self.target_value.as_mut().unwrap();
        if let Some(target) = target.as_any_mut().downcast_mut::<DataValueInteger>() {
            if target.value() != value {
                target.set_value(value);
                return true;
            }
        } else if let Some(target) = target.as_any_mut().downcast_mut::<DataValueAssetImage>() {
            if target.value() != value {
                target.set_value(value);
                return true;
            }
        } else if let Some(target) = target.as_any_mut().downcast_mut::<DataValueAssetFont>() {
            if target.value() != value {
                target.set_value(value);
                return true;
            }
        } else if let Some(target) = target.as_any_mut().downcast_mut::<DataValueAssetBlob>() {
            if target.value() != value {
                target.set_value(value);
                return true;
            }
        }
        false
    }
    fn update_number(&mut self, value: f32) -> bool {
        let target = self
            .target_value
            .as_mut()
            .unwrap()
            .as_any_mut()
            .downcast_mut::<DataValueNumber>()
            .unwrap();
        if target.value() != value {
            target.set_value(value);
            true
        } else {
            false
        }
    }
    fn update_string(&mut self, value: String) -> bool {
        let target = self
            .target_value
            .as_mut()
            .unwrap()
            .as_any_mut()
            .downcast_mut::<DataValueString>()
            .unwrap();
        if target.value() != value {
            target.set_value(value);
            true
        } else {
            false
        }
    }
    fn update_color(&mut self, value: i32) -> bool {
        let target = self
            .target_value
            .as_mut()
            .unwrap()
            .as_any_mut()
            .downcast_mut::<DataValueColor>()
            .unwrap();
        if target.value() != value {
            target.set_value(value);
            true
        } else {
            false
        }
    }
    fn update_boolean(&mut self, value: bool) -> bool {
        let target = self
            .target_value
            .as_mut()
            .unwrap()
            .as_any_mut()
            .downcast_mut::<DataValueBoolean>()
            .unwrap();
        if target.value() != value {
            target.set_value(value);
            true
        } else {
            false
        }
    }
    pub fn sync_target_value(&mut self, binding: &dyn TargetBinding) -> bool {
        if !binding.has_target() {
            return false;
        }
        match binding.field_type() {
            FieldType::Uint => {
                if binding.is_solo_active_property() && binding.target_kind() == TargetKind::Solo {
                    match binding.source_output_type() {
                        DataType::String => self.update_string(binding.active_child_name()),
                        DataType::Number => self.update_number(binding.active_child_index() as f32),
                        DataType::Integer => self.update_integer(binding.active_child_index()),
                        DataType::Enum => binding
                            .enum_index_for_name(&binding.active_child_name())
                            .is_some_and(|index| self.update_integer(index)),
                        _ => false,
                    }
                } else if binding.target_kind() == TargetKind::BindableAsset {
                    let mut changed = self.update_integer(binding.uint_value());
                    if let Some(value) = self
                        .target_value
                        .as_mut()
                        .and_then(|v| v.as_any_mut().downcast_mut::<DataValueAssetImage>())
                    {
                        let next = binding.image_value();
                        if !same_rc(&value.image_value(), &next) {
                            value.set_image_value(next);
                            changed = true;
                        }
                    } else if let Some(value) = self
                        .target_value
                        .as_mut()
                        .and_then(|v| v.as_any_mut().downcast_mut::<DataValueAssetFont>())
                    {
                        let next = binding.font_value();
                        if !same_rc(&value.font_value(), &next) {
                            value.set_font_value(next);
                            changed = true;
                        }
                    } else if let Some(value) = self
                        .target_value
                        .as_mut()
                        .and_then(|v| v.as_any_mut().downcast_mut::<DataValueAssetBlob>())
                    {
                        let next = binding.blob_value();
                        if !same_rc(&value.file_asset(), &next) {
                            value.set_blob_value(next);
                            changed = true;
                        }
                    }
                    changed
                } else if matches!(
                    binding.target_kind(),
                    TargetKind::BindableViewModel | TargetKind::ViewModelInstanceViewModel
                ) {
                    let next = binding.view_model_value();
                    let value = self
                        .target_value
                        .as_mut()
                        .unwrap()
                        .as_any_mut()
                        .downcast_mut::<DataValueViewModel>()
                        .unwrap();
                    if value.value() != next {
                        value.set_value(next);
                        true
                    } else {
                        false
                    }
                } else {
                    self.update_integer(binding.uint_value())
                }
            }
            FieldType::Color => self.update_color(binding.color_value()),
            FieldType::Double => self.update_number(binding.double_value()),
            FieldType::String => self.update_string(binding.string_value()),
            FieldType::Bool => self.update_boolean(binding.bool_value()),
            FieldType::Other => false,
        }
    }
    pub fn data_value(&self) -> Option<&dyn DataValue> {
        self.target_value.as_deref()
    }
}
fn same_rc<T: ?Sized>(a: &Option<Rc<T>>, b: &Option<Rc<T>>) -> bool {
    match (a, b) {
        (Some(a), Some(b)) => Rc::ptr_eq(a, b),
        (None, None) => true,
        _ => false,
    }
}
