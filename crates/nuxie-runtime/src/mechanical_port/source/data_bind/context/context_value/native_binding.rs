//! Core occurrence adapter for the translated DataBindContextValue owners.
//! This holds identity, not a second value graph or a generic runtime service.
use super::{ContextApplyBinding, ContextBinding};
use crate::mechanical_port::source::{
    assets::{blob_asset::BlobAsset, font_asset::FontAsset, image_asset::ImageAsset},
    core::CoreHandle,
    data_bind::{
        bindable_property_asset::BindablePropertyAsset,
        bindable_property_viewmodel::BindablePropertyViewModel,
        context::context_target_value::{FieldType, SourceKind, TargetBinding, TargetKind},
        converters::data_converter::DataConverter,
        data_values::{
            data_type::DataType,
            data_value::{DataValue, EmptyDataValue, clone_data_value},
            data_value_artboard::DataValueArtboard,
            data_value_asset_blob::DataValueAssetBlob,
            data_value_asset_font::DataValueAssetFont,
            data_value_asset_image::DataValueAssetImage,
            data_value_boolean::DataValueBoolean,
            data_value_color::DataValueColor,
            data_value_enum::{DataEnumRef, DataValueEnum},
            data_value_integer::{DataValueInteger, integer_value},
            data_value_list::DataValueList,
            data_value_number::DataValueNumber,
            data_value_string::DataValueString,
            data_value_symbol_list_index::DataValueSymbolListIndex,
            data_value_trigger::DataValueTrigger,
            data_value_viewmodel::DataValueViewModel,
        },
    },
    generated::{
        core_registry::{CoreRegistry, data_bind_update_view_model_handle},
        data_bind::bindable_property_id_base::BindablePropertyIdBase,
        solo_base::SoloBase,
    },
    shapes::image::Image,
    solo::Solo,
    text_engine::FontRef,
    viewmodel::{
        viewmodel_instance::ViewModelInstance,
        viewmodel_instance_artboard::ViewModelInstanceArtboard,
        viewmodel_instance_asset_blob::ViewModelInstanceAssetBlob,
        viewmodel_instance_asset_font::ViewModelInstanceAssetFont,
        viewmodel_instance_asset_image::ViewModelInstanceAssetImage,
        viewmodel_instance_boolean::ViewModelInstanceBoolean,
        viewmodel_instance_color::ViewModelInstanceColor,
        viewmodel_instance_enum::ViewModelInstanceEnum,
        viewmodel_instance_list::ViewModelInstanceList,
        viewmodel_instance_number::ViewModelInstanceNumber,
        viewmodel_instance_string::ViewModelInstanceString,
        viewmodel_instance_symbol_list_index::ViewModelInstanceSymbolListIndex,
        viewmodel_instance_trigger::ViewModelInstanceTrigger,
        viewmodel_instance_viewmodel::ViewModelInstanceViewModel,
    },
};
use crate::{RuntimeBlobAsset, view_model_cell::RuntimeHostMutationNotifications};
use nuxie_render_api::RenderImage;
use std::{rc::Rc, sync::Arc};

pub(in super::super) struct CoreBinding {
    bind: CoreHandle,
    target: Option<CoreHandle>,
    property_key: u32,
}

/// The value remains committed immediately; only callbacks crossing the live
/// Core borrow are published after that exact setter has returned.
fn mutate<R>(mutation: impl FnOnce() -> R) -> R {
    let notifications = RuntimeHostMutationNotifications::begin();
    let result = mutation();
    if let Some(notifications) = notifications {
        notifications.commit();
    }
    result
}

impl CoreBinding {
    pub(in super::super) fn new(
        bind: CoreHandle,
        target: Option<CoreHandle>,
        property_key: u32,
    ) -> Self {
        Self {
            bind,
            target,
            property_key,
        }
    }
    pub(in super::super) fn for_bind(bind: CoreHandle) -> Self {
        let (target, property_key) = bind
            .with(|owner| {
                let owner = owner.as_data_bind().expect("DataBind occurrence");
                (owner.target(), owner.property_key())
            })
            .expect("retained DataBind");
        Self::new(bind, target, property_key)
    }
    fn source(&self) -> Option<CoreHandle> {
        self.bind
            .with(|owner| owner.as_data_bind().expect("DataBind occurrence").source())
            .flatten()
    }
    fn source_enum(&self) -> Option<CoreHandle> {
        let property = self
            .source()?
            .with(|owner| owner.as_view_model_instance_value()?.view_model_property())
            .flatten()?;
        property
            .with(|property| property.as_view_model_property_enum()?.data_enum())
            .flatten()
    }
    fn resolved_asset<T: 'static>(&self) -> Option<CoreHandle> {
        let file = self
            .bind
            .with(|owner| owner.as_data_bind().expect("DataBind occurrence").file())?
            .upgrade()?;
        let asset = file.with_file(|file| file.asset(self.source_uint() as usize))?;
        asset.with_downcast::<T, _>(|_| ())?;
        Some(asset)
    }

    fn snapshot_source(&self, initial: bool) -> Option<Box<dyn DataValue>> {
        let source = self.source()?;
        let source_enum = self.source_enum();
        source.with(|source| {
            let source = source.as_any();
            macro_rules! scalar {
                ($vm:ty, $data:ty) => {
                    if let Some(value) = source.downcast_ref::<$vm>() {
                        return Box::new(<$data>::new(value.base.property_value()))
                            as Box<dyn DataValue>;
                    }
                };
            }
            scalar!(ViewModelInstanceNumber, DataValueNumber);
            if let Some(value) = source.downcast_ref::<ViewModelInstanceString>() {
                return Box::new(DataValueString::new(value.base.property_value().to_owned()));
            }
            scalar!(ViewModelInstanceColor, DataValueColor);
            scalar!(ViewModelInstanceBoolean, DataValueBoolean);
            if let Some(value) = source.downcast_ref::<ViewModelInstanceEnum>() {
                let mut data = DataValueEnum::default();
                data.set_value(value.base.property_value());
                if let Some(data_enum) = source_enum {
                    data.set_data_enum(DataEnumRef::Core(data_enum));
                }
                return Box::new(data);
            }
            scalar!(ViewModelInstanceTrigger, DataValueTrigger);
            if let Some(value) = source.downcast_ref::<ViewModelInstanceList>() {
                let mut data = DataValueList::default();
                if !initial {
                    for item in value.list_items() {
                        data.add_item(item.clone());
                    }
                }
                return Box::new(data);
            }
            if let Some(value) = source.downcast_ref::<ViewModelInstanceSymbolListIndex>() {
                return Box::new(DataValueSymbolListIndex::new(if initial {
                    0
                } else {
                    value.base.property_value()
                }));
            }
            scalar!(ViewModelInstanceAssetImage, DataValueAssetImage);
            scalar!(ViewModelInstanceAssetFont, DataValueAssetFont);
            scalar!(ViewModelInstanceAssetBlob, DataValueAssetBlob);
            scalar!(ViewModelInstanceArtboard, DataValueArtboard);
            if let Some(value) = source.downcast_ref::<ViewModelInstanceViewModel>() {
                let mut data = DataValueViewModel::default();
                if !initial {
                    data.set_value(value.reference_view_model_instance());
                }
                return Box::new(data);
            }
            Box::new(EmptyDataValue)
        })
    }
}

impl ContextBinding for CoreBinding {
    fn to_source(&self) -> bool {
        self.bind
            .with(|owner| {
                owner
                    .as_data_bind()
                    .expect("DataBind occurrence")
                    .to_source()
            })
            .expect("retained DataBind")
    }
    fn initial_source_value(&self) -> Option<Box<dyn DataValue>> {
        self.snapshot_source(true)
    }
    fn sync_source_value(&self, value: &mut dyn DataValue) {
        let Some(next) = self.snapshot_source(false) else {
            return;
        };
        macro_rules! sync { ($($ty:ty),* $(,)?) => { $(if let Some(next) = next.as_any().downcast_ref::<$ty>() {
            value.as_any_mut().downcast_mut::<$ty>().expect("bound source type remains stable").set_value(next.value());
            return;
        })* }; }
        sync!(
            DataValueNumber,
            DataValueColor,
            DataValueBoolean,
            DataValueEnum,
            DataValueTrigger,
            DataValueSymbolListIndex,
            DataValueAssetImage,
            DataValueAssetFont,
            DataValueAssetBlob,
            DataValueArtboard,
            DataValueViewModel
        );
        if let Some(next) = next.as_any().downcast_ref::<DataValueString>() {
            value
                .as_any_mut()
                .downcast_mut::<DataValueString>()
                .expect("String source")
                .set_value(next.value().to_owned());
        } else if let Some(next) = next.as_any().downcast_ref::<DataValueList>() {
            let value = value
                .as_any_mut()
                .downcast_mut::<DataValueList>()
                .expect("List source");
            value.clear();
            for item in next.items() {
                value.add_item(item.clone());
            }
        }
    }
    fn convert(&mut self, input: &dyn DataValue, is_main_direction: bool) -> Box<dyn DataValue> {
        let converter = self
            .bind
            .with(|owner| {
                owner
                    .as_data_bind()
                    .expect("DataBind occurrence")
                    .converter()
            })
            .flatten();
        match converter {
            Some(converter) => {
                DataConverter::convert_handle(&converter, input, &self.bind, !is_main_direction)
            }
            None => clone_data_value(input),
        }
    }
    fn suppress_dirt(&mut self, value: bool) {
        self.bind.with_mut(|owner| {
            owner
                .as_data_bind_mut()
                .expect("DataBind occurrence")
                .suppress_dirt(value)
        });
    }
    fn can_apply_to_source(&self) -> bool {
        self.source().is_some_and(|source| {
            source
                .with(|source| {
                    let source = source.as_any();
                    source.is::<ViewModelInstanceNumber>()
                        || source.is::<ViewModelInstanceString>()
                        || source.is::<ViewModelInstanceColor>()
                        || source.is::<ViewModelInstanceBoolean>()
                        || source.is::<ViewModelInstanceEnum>()
                        || source.is::<ViewModelInstanceTrigger>()
                        || source.is::<ViewModelInstanceSymbolListIndex>()
                        || source.is::<ViewModelInstanceArtboard>()
                        || source.is::<ViewModelInstanceAssetImage>()
                        || source.is::<ViewModelInstanceAssetFont>()
                        || source.is::<ViewModelInstanceAssetBlob>()
                        || source.is::<ViewModelInstanceViewModel>()
                })
                .unwrap_or(false)
        })
    }
    fn apply_source_value(&mut self, value: &dyn DataValue) -> bool {
        let Some(source) = self.source() else {
            return false;
        };
        if source
            .with_downcast::<ViewModelInstanceViewModel, _>(|_| ())
            .is_some()
        {
            return value
                .as_any()
                .downcast_ref::<DataValueViewModel>()
                .is_some_and(|value| {
                    ViewModelInstanceViewModel::update_view_model_occurrence(
                        &source,
                        value.value(),
                    );
                    true
                });
        }
        mutate(|| {
            source.with_mut(|source| {
            macro_rules! typed { ($vm:ty, $data:ty) => {
                if let Some(source) = source.as_any_mut().downcast_mut::<$vm>() {
                    return value.as_any().downcast_ref::<$data>().is_some_and(|value| { source.apply_value(value); true });
                }
            }; }
            typed!(ViewModelInstanceNumber, DataValueNumber);
            typed!(ViewModelInstanceString, DataValueString);
            typed!(ViewModelInstanceColor, DataValueColor);
            typed!(ViewModelInstanceBoolean, DataValueBoolean);
            let Some(integer) = integer_value(value) else { return false; };
            let integer = DataValueInteger::new(integer);
            macro_rules! numeric { ($($vm:ty),* $(,)?) => { $(if let Some(source) = source.as_any_mut().downcast_mut::<$vm>() { source.apply_value(&integer); return true; })* }; }
            numeric!(ViewModelInstanceEnum, ViewModelInstanceTrigger, ViewModelInstanceSymbolListIndex, ViewModelInstanceArtboard);
            if let Some(source) = source.as_any_mut().downcast_mut::<ViewModelInstanceAssetImage>() {
                source.apply_data_value(value);
                return true;
            }
            if let Some(source) = source.as_any_mut().downcast_mut::<ViewModelInstanceAssetFont>() {
                source.apply_data_value(value);
                return true;
            }
            if let Some(source) = source.as_any_mut().downcast_mut::<ViewModelInstanceAssetBlob>() {
                source.apply_data_value(value);
                return true;
            }
            false
        }).unwrap_or(false)
        })
    }
}

impl TargetBinding for CoreBinding {
    fn has_target(&self) -> bool {
        self.target.is_some()
    }
    fn field_type(&self) -> FieldType {
        match CoreRegistry::property_field_id(self.property_key as i32) {
            0 => FieldType::Uint,
            1 => FieldType::String,
            2 => FieldType::Double,
            3 => FieldType::Color,
            4 => FieldType::Bool,
            _ => FieldType::Other,
        }
    }
    fn is_solo_active_property(&self) -> bool {
        self.property_key == u32::from(SoloBase::ACTIVE_COMPONENT_ID_PROPERTY_KEY)
    }
    fn source_output_type(&self) -> DataType {
        self.bind
            .with(|owner| {
                owner
                    .as_data_bind()
                    .expect("DataBind occurrence")
                    .source_output_type()
            })
            .expect("retained DataBind")
    }
    fn source_kind(&self) -> SourceKind {
        self.source()
            .and_then(|source| {
                source.with(|source| {
                    let source = source.as_any();
                    if source.is::<ViewModelInstanceArtboard>() {
                        SourceKind::Artboard
                    } else if source.is::<ViewModelInstanceAssetImage>() {
                        SourceKind::AssetImage
                    } else if source.is::<ViewModelInstanceAssetFont>() {
                        SourceKind::AssetFont
                    } else if source.is::<ViewModelInstanceAssetBlob>() {
                        SourceKind::AssetBlob
                    } else if source.is::<ViewModelInstanceViewModel>() {
                        SourceKind::ViewModel
                    } else {
                        SourceKind::Other
                    }
                })
            })
            .unwrap_or(SourceKind::Other)
    }
    fn target_kind(&self) -> TargetKind {
        self.target
            .as_ref()
            .and_then(|target| {
                target.with(|target| {
                    let any = target.as_any();
                    if any.is::<Solo>() {
                        TargetKind::Solo
                    } else if any.is::<BindablePropertyAsset>() {
                        TargetKind::BindableAsset
                    } else if any.is::<BindablePropertyViewModel>() {
                        TargetKind::BindableViewModel
                    } else if any.is::<ViewModelInstanceViewModel>() {
                        TargetKind::ViewModelInstanceViewModel
                    } else if target
                        .artboard_referencer_referenced_artboard_id()
                        .is_some()
                    {
                        TargetKind::ArtboardReferencer
                    } else if target.as_text_style().is_some() {
                        TargetKind::TextStyle
                    } else if any.is::<Image>() {
                        TargetKind::Image
                    } else if any.is::<ViewModelInstanceAssetImage>() {
                        TargetKind::ViewModelAssetImage
                    } else if any.is::<ViewModelInstanceAssetFont>() {
                        TargetKind::ViewModelAssetFont
                    } else if any.is::<ViewModelInstanceAssetBlob>() {
                        TargetKind::ViewModelAssetBlob
                    } else {
                        TargetKind::Other
                    }
                })
            })
            .unwrap_or(TargetKind::Other)
    }
    fn uint_value(&self) -> u32 {
        CoreRegistry::get_uint_handle(
            self.target.as_ref().expect("binding target"),
            self.property_key as i32,
        )
        .expect("retained target")
    }
    fn color_value(&self) -> i32 {
        CoreRegistry::get_color_handle(
            self.target.as_ref().expect("binding target"),
            self.property_key as i32,
        )
        .expect("retained target")
    }
    fn double_value(&self) -> f32 {
        CoreRegistry::get_double_handle(
            self.target.as_ref().expect("binding target"),
            self.property_key as i32,
        )
        .expect("retained target")
    }
    fn string_value(&self) -> String {
        CoreRegistry::get_string_handle(
            self.target.as_ref().expect("binding target"),
            self.property_key as i32,
        )
        .expect("retained target")
    }
    fn bool_value(&self) -> bool {
        CoreRegistry::get_bool_handle(
            self.target.as_ref().expect("binding target"),
            self.property_key as i32,
        )
        .expect("retained target")
    }
    fn active_child_name(&self) -> String {
        self.target
            .as_ref()
            .expect("Solo target")
            .with_downcast::<Solo, _>(Solo::get_active_child_name)
            .expect("Solo")
    }
    fn active_child_index(&self) -> i32 {
        self.target
            .as_ref()
            .expect("Solo target")
            .with_downcast_mut::<Solo, _>(Solo::get_active_child_index)
            .expect("Solo")
    }
    fn enum_index_for_name(&self, name: &str) -> Option<u32> {
        let index = self.source_enum()?.with(|data_enum| {
            data_enum
                .as_data_enum()
                .expect("authored enum")
                .value_index_by_name(name)
        })?;
        Some(index as u32)
    }
    fn image_value(&self) -> Option<Rc<dyn RenderImage>> {
        self.target
            .as_ref()?
            .with_downcast::<BindablePropertyAsset, _>(BindablePropertyAsset::image_value)
            .flatten()
    }
    fn font_value(&self) -> Option<FontRef> {
        self.target
            .as_ref()?
            .with_downcast::<BindablePropertyAsset, _>(BindablePropertyAsset::font_value)
            .flatten()
    }
    fn blob_value(&self) -> Option<Arc<RuntimeBlobAsset>> {
        self.target
            .as_ref()?
            .with_downcast::<BindablePropertyAsset, _>(BindablePropertyAsset::blob_value)
            .flatten()
    }
    fn view_model_value(&self) -> Option<CoreHandle> {
        self.target
            .as_ref()?
            .with(|target| {
                if let Some(target) = target.as_any().downcast_ref::<BindablePropertyViewModel>() {
                    target.view_model_instance_value()
                } else {
                    target
                        .as_view_model_instance_view_model()?
                        .reference_view_model_instance()
                }
            })
            .flatten()
    }
}

impl ContextApplyBinding for CoreBinding {
    fn set_bool(&mut self, key: u32, value: bool) {
        if let Some(target) = &self.target {
            mutate(|| CoreRegistry::set_bool_handle(target, key as i32, value));
        }
    }
    fn set_color(&mut self, key: u32, value: i32) {
        if let Some(target) = &self.target {
            mutate(|| CoreRegistry::set_color_handle(target, key as i32, value));
        }
    }
    fn set_double(&mut self, key: u32, value: f32) {
        if let Some(target) = &self.target {
            mutate(|| CoreRegistry::set_double_handle(target, key as i32, value));
        }
    }
    fn set_uint(&mut self, key: u32, value: u32) {
        if let Some(target) = &self.target {
            mutate(|| CoreRegistry::set_uint_handle(target, key as i32, value));
        }
    }
    fn set_string(&mut self, key: u32, value: String) {
        if let Some(target) = &self.target {
            mutate(|| CoreRegistry::set_string_handle(target, key as i32, value));
        }
    }
    fn target_is_solo(&self) -> bool {
        self.target_kind() == TargetKind::Solo
    }
    fn solo_update_by_index(&mut self, index: usize) {
        Solo::update_by_index_occurrence(self.target.as_ref().expect("Solo target"), index);
    }
    fn solo_update_by_name(&mut self, name: String) {
        Solo::update_by_name_occurrence(self.target.as_ref().expect("Solo target"), &name);
    }
    fn update_list(&mut self, items: &[CoreHandle]) {
        if let Some(target) = &self.target {
            mutate(|| {
                crate::mechanical_port::source::generated::core_registry::data_bind_update_list_handle(target, items)
            });
        }
    }
    fn update_view_model(&mut self, value: Option<CoreHandle>) {
        if let Some(target) = &self.target {
            data_bind_update_view_model_handle(target, value);
        }
    }
    fn target_is_bindable_view_model(&self) -> bool {
        self.target_kind() == TargetKind::BindableViewModel
    }
    fn set_bindable_view_model(&mut self, value: Option<CoreHandle>) {
        self.target
            .as_ref()
            .expect("bindable ViewModel")
            .with_downcast_mut::<BindablePropertyViewModel, _>(|target| {
                target.set_view_model_instance_value(value)
            });
    }
    fn bindable_view_model_property_key(&self) -> u32 {
        u32::from(BindablePropertyIdBase::PROPERTY_VALUE_PROPERTY_KEY)
    }
    fn pointer_key(&self, value: &CoreHandle) -> u32 {
        ViewModelInstance::pointer_key(Some(value))
    }
    fn source_uint(&self) -> u32 {
        self.source()
            .and_then(|source| {
                source.with(|source| {
                    let source = source.as_any();
                    if let Some(source) = source.downcast_ref::<ViewModelInstanceAssetImage>() {
                        source.base.property_value()
                    } else if let Some(source) = source.downcast_ref::<ViewModelInstanceAssetFont>()
                    {
                        source.base.property_value()
                    } else if let Some(source) = source.downcast_ref::<ViewModelInstanceAssetBlob>()
                    {
                        source.base.property_value()
                    } else if let Some(source) = source.downcast_ref::<ViewModelInstanceArtboard>()
                    {
                        source.base.property_value()
                    } else {
                        panic!("asset context requires its source owner")
                    }
                })
            })
            .expect("asset context source")
    }
    fn source_artboard(&self) -> Option<CoreHandle> {
        self.source().filter(|source| {
            source
                .with_downcast::<ViewModelInstanceArtboard, _>(|_| ())
                .is_some()
        })
    }
    fn update_artboard(&mut self, source: Option<CoreHandle>) {
        if let Some(target) = &self.target {
            crate::mechanical_port::source::generated::core_registry::artboard_referencer_update_artboard_handle(target, source);
        }
    }
    fn resolved_image_asset(&self) -> Option<CoreHandle> {
        self.resolved_asset::<ImageAsset>()
    }
    fn resolved_font_asset(&self) -> Option<CoreHandle> {
        self.resolved_asset::<FontAsset>()
    }
    fn resolved_blob_asset(&self) -> Option<CoreHandle> {
        self.resolved_asset::<BlobAsset>()
    }
    fn source_image_asset(&self) -> Option<CoreHandle> {
        let source = self.source()?;
        let asset = source
            .with_downcast::<ViewModelInstanceAssetImage, _>(ViewModelInstanceAssetImage::asset)?;
        Some(asset.core_asset(&source))
    }
    fn source_font_asset(&self) -> Option<CoreHandle> {
        let source = self.source()?;
        let asset = source
            .with_downcast::<ViewModelInstanceAssetFont, _>(ViewModelInstanceAssetFont::asset)?;
        Some(asset.core_asset(&source))
    }
    fn source_image(&self) -> Option<Rc<dyn RenderImage>> {
        self.source()?
            .with_downcast::<ViewModelInstanceAssetImage, _>(|source| source.asset().render_image())
            .flatten()
    }
    fn source_font(&self) -> Option<FontRef> {
        self.source()?
            .with_downcast::<ViewModelInstanceAssetFont, _>(|source| source.asset().font())
            .flatten()
    }
    fn source_blob(&self) -> Option<Arc<RuntimeBlobAsset>> {
        self.source()?
            .with_downcast::<ViewModelInstanceAssetBlob, _>(ViewModelInstanceAssetBlob::asset)
            .flatten()
    }
    fn set_target_image_asset(&mut self, asset: CoreHandle) {
        Image::set_asset_occurrence(self.target.as_ref().expect("Image target"), Some(asset));
    }
    fn set_target_font_asset(&mut self, asset: CoreHandle) {
        crate::mechanical_port::source::text::text_style::TextStyle::set_asset_occurrence(
            self.target.as_ref().expect("TextStyle target"),
            Some(asset),
        );
    }
    fn set_bindable_image(&mut self, image: Option<Rc<dyn RenderImage>>) {
        self.target
            .as_ref()
            .expect("bindable asset")
            .with_downcast_mut::<BindablePropertyAsset, _>(|target| target.set_image_value(image));
    }
    fn set_bindable_font(&mut self, font: Option<FontRef>) {
        self.target
            .as_ref()
            .expect("bindable asset")
            .with_downcast_mut::<BindablePropertyAsset, _>(|target| target.set_font_value(font));
    }
    fn set_bindable_blob(&mut self, blob: Option<Arc<RuntimeBlobAsset>>) {
        self.target
            .as_ref()
            .expect("bindable asset")
            .with_downcast_mut::<BindablePropertyAsset, _>(|target| target.set_blob_value(blob));
    }
    fn set_view_model_image(&mut self, image: Option<Rc<dyn RenderImage>>) {
        mutate(|| {
            self.target
                .as_ref()
                .expect("ViewModel image")
                .with_downcast_mut::<ViewModelInstanceAssetImage, _>(|target| {
                    target.set_value(image)
                })
        });
    }
    fn set_view_model_font(&mut self, font: Option<FontRef>) {
        mutate(|| {
            self.target
                .as_ref()
                .expect("ViewModel font")
                .with_downcast_mut::<ViewModelInstanceAssetFont, _>(|target| target.set_value(font))
        });
    }
    fn set_view_model_blob(&mut self, blob: Option<Arc<RuntimeBlobAsset>>) {
        mutate(|| {
            self.target
                .as_ref()
                .expect("ViewModel blob")
                .with_downcast_mut::<ViewModelInstanceAssetBlob, _>(|target| target.set_value(blob))
        });
    }
}
