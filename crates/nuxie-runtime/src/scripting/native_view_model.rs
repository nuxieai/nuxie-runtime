//! Approved Lua-facing projection of the translated ViewModel owners.
//! Values remain in their CoreArena; this module never makes a legacy graph.
use super::*;
use crate::mechanical_port::source::{
    assets::{blob_asset::BlobAsset, font_asset::FontAsset, image_asset::ImageAsset},
    core::CoreHandle,
    file::{File, RuntimeFileHandle, RuntimeFileWeakHandle},
    generated::viewmodel as generated,
    text::font_hb::HbFont,
    viewmodel::{
        data_enum::DataEnum,
        symbol_type::SymbolType,
        viewmodel_instance::ViewModelInstance,
        viewmodel_instance_asset_blob::ViewModelInstanceAssetBlob,
        viewmodel_instance_asset_font::ViewModelInstanceAssetFont,
        viewmodel_instance_asset_image::ViewModelInstanceAssetImage,
        viewmodel_instance_boolean::ViewModelInstanceBoolean,
        viewmodel_instance_color::ViewModelInstanceColor,
        viewmodel_instance_enum::ViewModelInstanceEnum,
        viewmodel_instance_list::ViewModelInstanceList,
        viewmodel_instance_list_item::ViewModelInstanceListItem,
        viewmodel_instance_number::ViewModelInstanceNumber,
        viewmodel_instance_string::ViewModelInstanceString,
        viewmodel_instance_symbol_list_index::ViewModelInstanceSymbolListIndex,
        viewmodel_instance_trigger::ViewModelInstanceTrigger,
        viewmodel_instance_value::{
            SuppressDelegation, ViewModelInstanceValueDelegate,
            ViewModelInstanceValueDelegateHandle,
        },
        viewmodel_property_viewmodel::ViewModelPropertyViewModel,
    },
};

#[derive(Clone)]
pub(super) struct NativeScriptViewModel {
    pub instance: Option<CoreHandle>,
    pub model: CoreHandle,
    pub file: NativeScriptFile,
}

/// File's Data constructor globals must not own File back through its VM.
/// Materialized view-model userdata acquire an owning lease when constructed.
#[derive(Clone)]
pub(super) struct NativeScriptFile {
    weak: RuntimeFileWeakHandle,
    _lease: Option<RuntimeFileHandle>,
}
impl NativeScriptFile {
    pub fn owning(file: RuntimeFileHandle) -> Self {
        Self {
            weak: file.downgrade(),
            _lease: Some(file),
        }
    }
    pub fn definition(file: &RuntimeFileHandle) -> Self {
        Self {
            weak: file.downgrade(),
            _lease: None,
        }
    }
    fn upgrade(&self) -> Option<RuntimeFileHandle> {
        self.weak.upgrade()
    }
    fn handle(&self) -> RuntimeFileHandle {
        self.upgrade()
            .expect("instantiated native view model retains File")
    }
    fn with_file<R>(&self, callback: impl FnOnce(&File) -> R) -> R {
        self.handle().with_file(callback)
    }
}

impl std::fmt::Debug for NativeScriptViewModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NativeScriptViewModel")
            .field("instance", &self.instance)
            .field("model", &self.model)
            .finish()
    }
}

struct NativePropertyDelegate {
    value: CoreHandle,
    dependent: crate::view_model_cell::RuntimeCellDependent,
}
impl ViewModelInstanceValueDelegate for NativePropertyDelegate {
    fn value_changed(&mut self) {
        let dependent = self.dependent.clone();
        let value = self.value.clone();
        if !crate::view_model_cell::defer_host_mutation_notification(move || {
            // Keep the source's Delegating guard during the released callback,
            // so a listener writing this value cannot recursively notify it.
            let _delegating = SuppressDelegation::new(value);
            dependent.add_dirt(crate::view_model_cell::RuntimeCellDirt::BINDINGS);
        }) {
            self.dependent
                .add_dirt(crate::view_model_cell::RuntimeCellDirt::BINDINGS);
        }
    }
}

impl NativeScriptViewModel {
    pub fn properties(&self) -> BTreeMap<String, ScriptViewModelProperty> {
        let properties = self
            .model
            .with(|model| model.as_view_model().map(|model| model.properties()))
            .flatten()
            .expect("native ViewModel definition");
        properties.into_iter().filter_map(|property| {
            let name = property.with(|property| property.as_view_model_property().map(|property| property.const_name().to_owned())).flatten()?;
            use ScriptViewModelProperty as Kind;
            let kind = match property.core_type()? {
                generated::viewmodel_property_number_base::ViewModelPropertyNumberBase::TYPE_KEY => Kind::Number,
                generated::viewmodel_property_color_base::ViewModelPropertyColorBase::TYPE_KEY => Kind::Color,
                generated::viewmodel_property_string_base::ViewModelPropertyStringBase::TYPE_KEY => Kind::String,
                generated::viewmodel_property_boolean_base::ViewModelPropertyBooleanBase::TYPE_KEY => Kind::Boolean,
                generated::viewmodel_property_trigger_base::ViewModelPropertyTriggerBase::TYPE_KEY => Kind::Trigger,
                generated::viewmodel_property_asset_image_base::ViewModelPropertyAssetImageBase::TYPE_KEY => Kind::Image,
                generated::viewmodel_property_asset_font_base::ViewModelPropertyAssetFontBase::TYPE_KEY => Kind::Font,
                generated::viewmodel_property_asset_blob_base::ViewModelPropertyAssetBlobBase::TYPE_KEY => Kind::Blob,
                generated::viewmodel_property_list_base::ViewModelPropertyListBase::TYPE_KEY => Kind::List,
                generated::viewmodel_property_viewmodel_base::ViewModelPropertyViewModelBase::TYPE_KEY => Kind::ViewModel,
                generated::viewmodel_property_symbol_list_index_base::ViewModelPropertySymbolListIndexBase::TYPE_KEY => Kind::SymbolListIndex,
                _ if property.is_type_of(generated::viewmodel_property_enum_base::ViewModelPropertyEnumBase::TYPE_KEY) => Kind::Enum,
                _ => return None,
            };
            Some((name, kind))
        }).collect()
    }

    pub fn property(&self, name: &str) -> Option<CoreHandle> {
        self.instance
            .as_ref()?
            .with_downcast::<ViewModelInstance, _>(|instance| instance.property_value_named(name))
            .flatten()
    }

    pub fn property_path(&self, name: &str) -> Option<Vec<usize>> {
        let properties = self
            .model
            .with(|model| model.as_view_model().map(|model| model.properties()))
            .flatten()?;
        properties
            .iter()
            .position(|property| {
                property
                    .with(|property| {
                        property
                            .as_view_model_property()
                            .is_some_and(|property| property.const_name() == name)
                    })
                    .unwrap_or(false)
            })
            .map(|index| vec![index])
    }

    pub fn has_parents(&self) -> bool {
        self.instance
            .as_ref()
            .and_then(|instance| {
                instance.with_downcast::<ViewModelInstance, _>(ViewModelInstance::has_parents)
            })
            .unwrap_or(false)
    }

    pub fn property_dirt_sink(&self, name: &str) -> Option<RuntimeCellDirtSink> {
        let property = self.property(name)?;
        let mut sink = RuntimeCellDirtSink::new();
        let delegate = Rc::new(RefCell::new(NativePropertyDelegate {
            value: property.clone(),
            dependent: sink.downgrade(),
        }));
        let erased: ViewModelInstanceValueDelegateHandle = delegate.clone();
        property.with_mut(|property| {
            property
                .as_view_model_instance_value_mut()
                .expect("native value owner")
                .add_delegate(&erased)
        })?;
        sink.retain_owner(delegate);
        Some(sink)
    }

    pub fn named_instance(&self, name: Option<&str>) -> Option<ScriptViewModel> {
        let model_name = self
            .model
            .with(|model| {
                model
                    .as_view_model()
                    .map(|model| model.base.name().to_owned())
            })
            .flatten()?;
        let file = self.file.upgrade()?;
        let instance = file.with_file_mut(|file| {
            name.and_then(|name| file.create_view_model_instance_named(&model_name, name))
                .or_else(|| file.create_view_model_instance(self.model.clone()))
        })?;
        ScriptViewModel::from_native(instance, file)
    }

    fn mutate<T: 'static>(&self, name: &str, mutation: impl FnOnce(&mut T) -> bool) -> bool {
        let Some(property) = self.property(name) else {
            return false;
        };
        let notifications = RuntimeHostMutationNotifications::begin();
        let changed = property
            .with_downcast_mut::<T, _>(mutation)
            .unwrap_or(false);
        if let Some(notifications) = notifications {
            notifications.commit();
        }
        changed
    }

    pub fn number(&self, name: &str) -> Option<f32> {
        self.property(name)?
            .with_downcast::<ViewModelInstanceNumber, _>(ViewModelInstanceNumber::value)
    }
    pub fn set_number(&self, name: &str, value: f32) -> bool {
        self.mutate::<ViewModelInstanceNumber>(name, |owner| {
            let changed = owner.value() != value;
            owner.set_value(value);
            changed
        })
    }
    pub fn color(&self, name: &str) -> Option<u32> {
        self.property(name)?
            .with_downcast::<ViewModelInstanceColor, _>(|owner| owner.value() as u32)
    }
    pub fn set_color(&self, name: &str, value: u32) -> bool {
        self.mutate::<ViewModelInstanceColor>(name, |owner| {
            let changed = owner.value() as u32 != value;
            owner.set_value(value as i32);
            changed
        })
    }
    pub fn string(&self, name: &str) -> Option<String> {
        self.property(name)?
            .with_downcast::<ViewModelInstanceString, _>(ViewModelInstanceString::value)
    }
    pub fn set_string(&self, name: &str, value: &str) -> bool {
        self.mutate::<ViewModelInstanceString>(name, |owner| {
            let changed = owner.value() != value;
            owner.set_value(value);
            changed
        })
    }
    pub fn boolean(&self, name: &str) -> Option<bool> {
        self.property(name)?
            .with_downcast::<ViewModelInstanceBoolean, _>(ViewModelInstanceBoolean::value)
    }
    pub fn set_boolean(&self, name: &str, value: bool) -> bool {
        self.mutate::<ViewModelInstanceBoolean>(name, |owner| {
            let changed = owner.value() != value;
            owner.set_value(value);
            changed
        })
    }
    pub fn trigger(&self, name: &str) -> Option<u64> {
        self.property(name)?
            .with_downcast::<ViewModelInstanceTrigger, _>(|owner| {
                u64::from(owner.base.property_value())
            })
    }
    pub fn fire_trigger(&self, name: &str) -> bool {
        self.mutate::<ViewModelInstanceTrigger>(name, |owner| {
            owner.trigger();
            true
        })
    }

    fn data_enum(&self, name: &str) -> Option<CoreHandle> {
        let property = self
            .property(name)?
            .with(|value| value.as_view_model_instance_value()?.view_model_property())
            .flatten()?;
        property
            .with(|property| property.as_view_model_property_enum()?.data_enum())
            .flatten()
    }
    pub fn enum_values(&self, name: &str) -> Option<Vec<String>> {
        self.data_enum(name)?.with_downcast::<DataEnum, _>(|owner| {
            (0..owner.values().len())
                .map(|index| owner.key_at(index as u32))
                .collect()
        })
    }
    pub fn enum_value(&self, name: &str) -> Option<String> {
        let index = self
            .property(name)?
            .with_downcast::<ViewModelInstanceEnum, _>(|owner| owner.base.property_value())?;
        self.data_enum(name)?
            .with_downcast::<DataEnum, _>(|owner| owner.key_at(index))
    }
    pub fn set_enum_value(&self, name: &str, value: &str) -> bool {
        let Some(index) = self
            .data_enum(name)
            .and_then(|owner| {
                owner.with_downcast::<DataEnum, _>(|owner| owner.value_index_by_name(value))
            })
            .filter(|index| *index >= 0)
        else {
            return false;
        };
        self.mutate::<ViewModelInstanceEnum>(name, |owner| {
            let changed = owner.base.property_value() != index as u32;
            owner.apply_value(&crate::mechanical_port::source::data_bind::data_values::data_value_integer::DataValueInteger::new(index as u32));
            changed
        })
    }

    fn file_asset_for_value(&self, name: &str) -> Option<(usize, CoreHandle)> {
        let property = self.property(name)?;
        let index = property
            .with(|owner| {
                if let Some(owner) = owner.as_any().downcast_ref::<ViewModelInstanceAssetImage>() {
                    return Some(owner.base.property_value() as usize);
                }
                if let Some(owner) = owner.as_any().downcast_ref::<ViewModelInstanceAssetFont>() {
                    return Some(owner.base.property_value() as usize);
                }
                owner
                    .as_any()
                    .downcast_ref::<ViewModelInstanceAssetBlob>()
                    .map(|owner| owner.base.property_value() as usize)
            })
            .flatten()?;
        self.file
            .with_file(|file| file.asset(index))
            .map(|asset| (index, asset))
    }
    pub fn image(&self, name: &str) -> Option<ScriptImage> {
        let (index, asset) = self.file_asset_for_value(name)?;
        asset.with_downcast::<ImageAsset, _>(|asset| ScriptImage {
            file_asset_index: index as u64,
            asset_global_id: asset.base.asset_id(),
        })
    }
    pub fn render_image(&self, name: &str) -> Option<Rc<dyn nuxie_render_api::RenderImage>> {
        let live = self
            .property(name)?
            .with_downcast::<ViewModelInstanceAssetImage, _>(|owner| owner.asset().render_image())
            .flatten();
        live.or_else(|| {
            self.file_asset_for_value(name)?
                .1
                .with_downcast::<ImageAsset, _>(|asset| asset.render_image().cloned())
                .flatten()
        })
    }
    pub fn image_asset_named(&self, name: &str) -> Option<ScriptImage> {
        self.file
            .with_file(|file| file.assets().to_vec())
            .iter()
            .enumerate()
            .find_map(|(index, asset)| {
                asset
                    .with_downcast::<ImageAsset, _>(|asset| {
                        (asset.base.name() == name).then(|| ScriptImage {
                            file_asset_index: index as u64,
                            asset_global_id: asset.base.asset_id(),
                        })
                    })
                    .flatten()
            })
    }
    pub fn set_image(&self, name: &str, image: Option<ScriptImage>) -> bool {
        let image = image
            .and_then(|image| {
                self.file
                    .with_file(|file| file.asset(image.file_asset_index as usize))
            })
            .and_then(|asset| {
                asset.with_downcast::<ImageAsset, _>(|asset| asset.render_image().cloned())
            })
            .flatten();
        self.set_render_image(name, image)
    }
    pub fn set_render_image(
        &self,
        name: &str,
        image: Option<Rc<dyn nuxie_render_api::RenderImage>>,
    ) -> bool {
        self.mutate::<ViewModelInstanceAssetImage>(name, |owner| {
            owner.set_value(image);
            true
        })
    }
    pub fn font(&self, name: &str) -> Option<ScriptFont> {
        let live = self
            .property(name)?
            .with_downcast::<ViewModelInstanceAssetFont, _>(|owner| owner.asset().font())
            .flatten();
        let font = live.or_else(|| {
            self.file_asset_for_value(name)?
                .1
                .with_downcast::<FontAsset, _>(FontAsset::font)
                .flatten()
        })?;
        let bytes = font
            .as_any()
            .downcast_ref::<HbFont>()
            .expect("approved Rust font owner")
            .source_bytes();
        Some(ScriptFont {
            asset_global_id: None,
            live_font_bytes: Some(bytes),
            native_font: Some(font),
        })
    }
    pub fn set_font(&self, name: &str, font: Option<&ScriptFont>) -> bool {
        let native = match font {
            Some(font) => match &font.native_font {
                Some(font) => Some(font.clone()),
                None => return self.set_font_bytes(name, font.live_font_bytes.clone()),
            },
            None => None,
        };
        self.mutate::<ViewModelInstanceAssetFont>(name, |owner| {
            let previous = owner.asset().font();
            let changed = owner.base.property_value() != u32::MAX
                || !match (&previous, &native) {
                    (Some(previous), Some(native)) => Rc::ptr_eq(previous, native),
                    (None, None) => true,
                    _ => false,
                };
            owner.set_value(native);
            changed
        })
    }
    pub fn set_font_bytes(&self, name: &str, bytes: Option<Arc<[u8]>>) -> bool {
        let font = match bytes {
            Some(bytes) => {
                let Some(font) = HbFont::decode(&bytes) else {
                    return false;
                };
                Some(font)
            }
            None => None,
        };
        self.mutate::<ViewModelInstanceAssetFont>(name, |owner| {
            owner.set_value(font);
            true
        })
    }
    pub fn blob_asset(&self, name: &str) -> Option<Arc<RuntimeBlobAsset>> {
        let live = self
            .property(name)?
            .with_downcast::<ViewModelInstanceAssetBlob, _>(ViewModelInstanceAssetBlob::asset)
            .flatten();
        live.or_else(|| {
            self.file_asset_for_value(name)?
                .1
                .with_downcast::<BlobAsset, _>(BlobAsset::script_asset)
        })
    }
    pub fn set_blob_asset(&self, name: &str, asset: Option<Arc<RuntimeBlobAsset>>) -> bool {
        self.mutate::<ViewModelInstanceAssetBlob>(name, |owner| {
            owner.set_value(asset);
            true
        })
    }
    pub fn set_blob(&self, name: &str, bytes: Option<Arc<[u8]>>) -> bool {
        self.set_blob_asset(
            name,
            bytes.map(|bytes| Arc::new(RuntimeBlobAsset::from_decoded("", bytes.to_vec()))),
        )
    }
    pub fn component_list_item_index(&self) -> Option<u64> {
        self.instance
            .as_ref()?
            .with_downcast::<ViewModelInstance, _>(|owner| {
                owner.property_value_for_symbol(SymbolType::ItemIndex)
            })
            .flatten()?
            .with_downcast::<ViewModelInstanceSymbolListIndex, _>(|owner| {
                u64::from(owner.base.property_value())
            })
    }
    pub fn view_model(&self, name: &str, active_only: bool) -> Option<ScriptViewModel> {
        let property = self.property(name);
        let instance = property.as_ref().and_then(|property| {
            property
                .with(|owner| {
                    owner
                        .as_view_model_instance_view_model()?
                        .reference_view_model_instance()
                })
                .flatten()
        });
        if let Some(instance) = instance {
            return ScriptViewModel::from_native(instance, self.file.handle());
        }
        if active_only {
            return None;
        }
        let definition = self
            .model
            .with(|owner| owner.as_view_model()?.property_named(name))
            .flatten()?;
        let id = definition.with_downcast::<ViewModelPropertyViewModel, _>(|owner| {
            owner.base.view_model_reference_id()
        })?;
        let model = self.file.with_file(|file| file.view_model(id as usize))?;
        Some(ScriptViewModel::from_native_definition(
            model,
            None,
            self.file.handle(),
        ))
    }
    pub fn set_view_model(&self, name: &str, value: &ScriptViewModel) -> bool {
        let (Some(instance), Some(property), Some(value)) =
            (&self.instance, self.property(name), value.native_instance())
        else {
            return false;
        };
        ViewModelInstance::replace_view_model_property_occurrence(instance, &property, Some(value))
    }
    pub fn list_len(&self, name: &str) -> Option<usize> {
        self.property(name)?
            .with_downcast::<ViewModelInstanceList, _>(|owner| owner.list_items().len())
    }
    pub fn list_item(&self, name: &str, index: usize) -> Option<ScriptViewModel> {
        let item = self
            .property(name)?
            .with_downcast::<ViewModelInstanceList, _>(|owner| owner.item(index as u32))
            .flatten()?;
        let instance = item
            .with_downcast::<ViewModelInstanceListItem, _>(
                ViewModelInstanceListItem::view_model_instance,
            )
            .flatten()?;
        ScriptViewModel::from_native(instance, self.file.handle())
    }
    pub fn insert_list_item(
        &self,
        name: &str,
        index: Option<usize>,
        value: &ScriptViewModel,
    ) -> bool {
        let (Some(list), Some(instance)) = (self.property(name), value.native_instance()) else {
            return false;
        };
        let mut item = ViewModelInstanceListItem::default();
        item.set_view_model_instance(Some(instance));
        let Some(item) = list.insert_sibling(item) else {
            return false;
        };
        self.mutate::<ViewModelInstanceList>(name, |owner| {
            if let Some(index) = index {
                owner.add_item_at(item, index as i32)
            } else {
                owner.add_item(item);
                true
            }
        })
    }
    pub fn pop_list_item(&self, name: &str, shift: bool) -> Option<ScriptViewModel> {
        let property = self.property(name)?;
        let notifications = RuntimeHostMutationNotifications::begin();
        let item = property
            .with_downcast_mut::<ViewModelInstanceList, _>(|owner| {
                if shift { owner.shift() } else { owner.pop() }
            })
            .flatten();
        if let Some(notifications) = notifications {
            notifications.commit();
        }
        let instance = item?
            .with_downcast::<ViewModelInstanceListItem, _>(
                ViewModelInstanceListItem::view_model_instance,
            )
            .flatten()?;
        ScriptViewModel::from_native(instance, self.file.handle())
    }
    pub fn swap_list_items(&self, name: &str, first: usize, second: usize) -> bool {
        self.mutate::<ViewModelInstanceList>(name, |owner| {
            if first >= owner.list_items().len() || second >= owner.list_items().len() {
                return false;
            }
            owner.swap(first as u32, second as u32);
            true
        })
    }
    pub fn clear_list_items(&self, name: &str) -> bool {
        self.mutate::<ViewModelInstanceList>(name, |owner| {
            let changed = !owner.list_items().is_empty();
            owner.remove_all_items();
            changed
        })
    }
    pub fn remove_list_item_at(&self, name: &str, index: usize) -> bool {
        self.mutate::<ViewModelInstanceList>(name, |owner| {
            if index >= owner.list_items().len() {
                return false;
            }
            owner.remove_item_at(index as i32);
            true
        })
    }
    pub fn remove_list_item(&self, name: &str, value: &ScriptViewModel, all: bool) -> bool {
        let Some(instance) = value.native_instance() else {
            return false;
        };
        self.mutate::<ViewModelInstanceList>(name, |owner| {
            let matching = owner
                .list_items()
                .iter()
                .find(|item| {
                    item.with_downcast::<ViewModelInstanceListItem, _>(
                        ViewModelInstanceListItem::view_model_instance,
                    )
                    .flatten()
                    .as_ref()
                        == Some(&instance)
                })
                .cloned();
            let Some(item) = matching else {
                return false;
            };
            if all {
                owner.remove_all_items_with_view_model_instance(Some(instance));
            } else {
                owner.remove_item(&item);
            }
            true
        })
    }
    pub fn advance(&self) -> bool {
        let Some(instance) = &self.instance else {
            return false;
        };
        let changed = instance
            .with_downcast::<ViewModelInstance, _>(|owner| {
                owner.property_values().iter().any(|value| {
                    value
                        .with(|value| {
                            value
                                .as_view_model_instance_value()
                                .is_some_and(|value| value.has_changed())
                        })
                        .unwrap_or(false)
                })
            })
            .unwrap_or(false);
        instance.with_downcast_mut::<ViewModelInstance, _>(ViewModelInstance::advanced);
        changed
    }
}
