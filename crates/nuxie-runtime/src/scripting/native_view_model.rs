//! Approved Lua-facing projection of the translated ViewModel owners.
//! Values remain in their CoreArena; this module never makes a legacy graph.
use super::*;
use crate::host_viewmodel::CopyShape;
use crate::mechanical_port::source::{
    assets::{blob_asset::BlobAsset, font_asset::FontAsset, image_asset::ImageAsset},
    core::CoreHandle,
    file::{File, RuntimeFileHandle, RuntimeFileWeakHandle},
    generated::viewmodel as generated,
    text::font_hb::HbFont,
    viewmodel::{
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
    pub model: Option<CoreHandle>,
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
    pub(super) fn release_lease(&mut self) {
        self._lease = None;
    }
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
        let Some(model) = self.model.as_ref() else {
            return BTreeMap::new();
        };
        properties_of(model)
    }
}

/// The properties a script sees on an instance of `model`.
fn properties_of(model: &CoreHandle) -> BTreeMap<String, ScriptViewModelProperty> {
        let properties = model
            .with(|model| {
                model
                    .as_view_model()
                    .map(|model| model.properties().to_vec())
            })
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
                generated::viewmodel_property_artboard_base::ViewModelPropertyArtboardBase::TYPE_KEY => Kind::Artboard,
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

impl NativeScriptViewModel {
    pub fn property(&self, name: &str) -> Option<CoreHandle> {
        self.instance
            .as_ref()?
            .with_downcast::<ViewModelInstance, _>(|instance| instance.property_value_named(name))
            .flatten()
    }

    pub fn property_path(&self, name: &str) -> Option<Vec<usize>> {
        let properties = self
            .model
            .as_ref()?
            .with(|model| {
                model
                    .as_view_model()
                    .map(|model| model.properties().to_vec())
            })
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
            .as_ref()?
            .with(|model| {
                model
                    .as_view_model()
                    .map(|model| model.base.name().to_owned())
            })
            .flatten()?;
        let file = self.file.upgrade()?;
        let instance = file.with_file(|file| {
            name.and_then(|name| file.create_view_model_instance_named(&model_name, name))
                .or_else(|| file.create_view_model_instance(self.model.clone()?))
        })?;
        ScriptViewModel::from_native(instance, file)
    }

    /// What copying the authored instance `name` (`named_instance(Some(name))`)
    /// would make, measured on the authored instances before anything is
    /// copied. An authored instance names its nested instances and list
    /// items by index, which the copy resolves (`File::copy_view_model_instance`);
    /// this resolves them the same way. Counts as many instances as the copy
    /// visits, shared sources once per reference, stopping past
    /// `max_instances`.
    pub fn named_copy_shape(&self, name: &str, max_instances: usize) -> CopyShape {
        let Some(model_name) = self
            .model
            .as_ref()
            .and_then(|model| {
                model.with(|model| {
                    model
                        .as_view_model()
                        .map(|model| model.base.name().to_owned())
                })
            })
            .flatten()
        else {
            return CopyShape::NoTemplate;
        };
        let Some(file) = self.file.upgrade() else {
            return CopyShape::NoTemplate;
        };
        file.with_file(|file| {
            let Some(source) = file.view_model_named(&model_name).and_then(|model| {
                model
                    .with(|model| model.as_view_model()?.instance_named(name))
                    .flatten()
            }) else {
                return CopyShape::NoTemplate;
            };
            copy_shape_of(file, source, max_instances)
        })
    }

    fn mutate<T: 'static>(&self, name: &str, mutation: impl FnOnce(&mut T) -> bool) -> bool {
        let Some(property) = self.property(name) else {
            return false;
        };
        let notifications = RuntimeHostMutationNotifications::begin();
        let changed = property.with_downcast_mut::<T, _>(mutation);
        if changed == Some(false) {
            crate::host_viewmodel::capture_unchanged_native_write(&property);
        }
        if let Some(notifications) = notifications {
            notifications.commit();
        }
        changed.unwrap_or(false)
    }

    pub fn number(&self, name: &str) -> Option<f32> {
        self.property(name)?
            .with_downcast::<ViewModelInstanceNumber, _>(ViewModelInstanceNumber::value)
    }
    pub fn set_number(&self, name: &str, value: f32) -> bool {
        let Some(property) = self.property(name) else {
            return false;
        };
        let Some(previous) =
            property.with_downcast::<ViewModelInstanceNumber, _>(ViewModelInstanceNumber::value)
        else {
            return false;
        };
        let notifications = RuntimeHostMutationNotifications::begin();
        ViewModelInstanceNumber::set_value_handle(&property, value);
        if previous == value {
            crate::host_viewmodel::capture_unchanged_native_write(&property);
        }
        if let Some(notifications) = notifications {
            notifications.commit();
        }
        previous != value
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
    fn set_string_owned(
        &self,
        name: &str,
        value: &str,
        host_owner: crate::host_viewmodel::HostOwnerToken,
    ) -> bool {
        self.mutate::<ViewModelInstanceString>(name, |owner| {
            let changed = owner.value() != value;
            owner.set_value_owned(value, host_owner);
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
        let Some(property) = self.property(name) else {
            return false;
        };
        let notifications = RuntimeHostMutationNotifications::begin();
        let changed = ViewModelInstanceTrigger::trigger_handle(&property);
        if let Some(notifications) = notifications {
            notifications.commit();
        }
        changed
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
        self.data_enum(name)?.with(|owner| {
            let owner = owner.as_data_enum().expect("authored enum");
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
            .with(|owner| owner.as_data_enum().expect("authored enum").key_at(index))
    }
    pub fn set_enum_value(&self, name: &str, value: &str) -> bool {
        let Some(index) = self
            .data_enum(name)
            .and_then(|owner| {
                owner.with(|owner| {
                    owner
                        .as_data_enum()
                        .expect("authored enum")
                        .value_index_by_name(value)
                })
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
    #[cfg(feature = "scriptnet")]
    pub fn set_artboard(
        &self,
        name: &str,
        artboard: Option<crate::source::bindable_artboard::RuntimeBindableArtboardHandle>,
        view_model: Option<CoreHandle>,
    ) -> bool {
        use crate::source::viewmodel::viewmodel_instance_artboard::ViewModelInstanceArtboard;
        self.mutate::<ViewModelInstanceArtboard>(name, |owner| {
            owner.set_bound_view_model_instance(view_model);
            owner.set_asset(artboard)
        })
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
                    (Some(previous), Some(native)) => Arc::ptr_eq(previous, native),
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
            let current = owner.asset();
            let changed = owner.base.property_value() != u32::MAX
                || match (&current, &asset) {
                    (Some(current), Some(asset)) => !Arc::ptr_eq(current, asset),
                    (None, None) => false,
                    _ => true,
                };
            owner.set_value(asset);
            changed
        })
    }
    pub fn set_blob(&self, name: &str, bytes: Option<Arc<[u8]>>) -> bool {
        let current = self.blob_asset(name);
        self.set_blob_asset(
            name,
            bytes.map(|bytes| {
                current
                    .filter(|asset| asset.bytes_arc_ptr_eq(&bytes))
                    .unwrap_or_else(|| Arc::new(RuntimeBlobAsset::new("", bytes)))
            }),
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
            .as_ref()?
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
    pub fn referenced_view_model_value(
        &self,
        name: &str,
        creation_time_model: Option<CoreHandle>,
    ) -> ScriptViewModel {
        let instance = self.property(name).and_then(|property| {
            property
                .with(|property| {
                    property
                        .as_view_model_instance_view_model()?
                        .reference_view_model_instance()
                })
                .flatten()
        });
        let model = if let Some(instance) = instance.as_ref() {
            instance
                .with(|instance| instance.as_view_model_instance()?.get_view_model())
                .flatten()
        } else {
            creation_time_model
        };
        ScriptViewModel::from_native_parts(model, instance, self.file.handle())
    }
    pub fn set_view_model(&self, name: &str, value: &ScriptViewModel) -> bool {
        let (Some(instance), Some(property)) = (&self.instance, self.property(name)) else {
            return false;
        };
        ViewModelInstance::replace_view_model_property_occurrence(
            instance,
            &property,
            value.native_instance(),
        )
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
        self.insert_owned_list_item(name, index, value, None)
    }
    fn insert_owned_list_item(
        &self,
        name: &str,
        index: Option<usize>,
        value: &ScriptViewModel,
        owner: Option<crate::host_viewmodel::HostOwnerToken>,
    ) -> bool {
        let (Some(list), Some(instance)) = (self.property(name), value.native_instance()) else {
            return false;
        };
        let mut item = ViewModelInstanceListItem::default();
        item.set_view_model_instance(Some(instance));
        // The item lives in the list's arena from here, whether or not the
        // list takes it, so it keeps the owner from here.
        item.set_host_owner(owner);
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
}

/// Host-owned values: what a host made the runtime hold, kept alive (through
/// a [`crate::host_viewmodel::HostOwnerToken`]) for exactly as long as the
/// runtime keeps the value.
impl ScriptViewModel {
    /// Keeps `owner` with this view model's instance for as long as the
    /// instance lives (an instance lives as long as its File's arena).
    /// `false` when there is no instance.
    pub fn set_host_owner(&self, owner: crate::host_viewmodel::HostOwnerToken) -> bool {
        self.native()
            .instance
            .as_ref()
            .and_then(|instance| {
                instance.with_downcast_mut::<ViewModelInstance, _>(|instance| {
                    instance.set_host_owner(owner)
                })
            })
            .is_some()
    }

    /// What `named_instance(Some(name))` would copy, measured before the
    /// copy is made.
    pub fn named_copy_shape(&self, name: &str, max_instances: usize) -> CopyShape {
        self.native().named_copy_shape(name, max_instances)
    }

    /// The owner [`Self::set_host_owner`] gave the instance.
    pub fn host_owner(&self) -> Option<crate::host_viewmodel::HostOwnerToken> {
        self.native()
            .instance
            .as_ref()?
            .with_downcast::<ViewModelInstance, _>(|instance| instance.host_owner().cloned())
            .flatten()
    }

    /// `set_string`, keeping `owner` for as long as the property holds this
    /// value: any later change of value, whoever makes it, drops it.
    pub fn set_string_owned(
        &self,
        name: &str,
        value: &str,
        owner: crate::host_viewmodel::HostOwnerToken,
    ) -> bool {
        let native = self.native();
        let changed = native.set_string_owned(name, value, owner);
        self.finish_property_change(&native.property_path(name).unwrap_or_default(), changed)
    }

    /// The owner the property's current value keeps.
    pub fn string_host_owner(&self, name: &str) -> Option<crate::host_viewmodel::HostOwnerToken> {
        self.native()
            .property(name)?
            .with_downcast::<ViewModelInstanceString, _>(|value| value.host_owner().cloned())
            .flatten()
    }

    /// `push_list_item` (no `index`) or `insert_list_item`, with `owner` kept
    /// by the new list item for as long as it lives: a removed item stays in
    /// the list's arena, so it keeps the owner.
    pub fn insert_list_item_owned(
        &self,
        name: &str,
        index: Option<usize>,
        item: &ScriptViewModel,
        owner: crate::host_viewmodel::HostOwnerToken,
    ) -> bool {
        let native = self.native();
        let changed = native.insert_owned_list_item(name, index, item, Some(owner));
        self.finish_property_change(&native.property_path(name).unwrap_or_default(), changed)
    }

    /// The owners the list's items keep, in order.
    pub fn list_item_host_owners(
        &self,
        name: &str,
    ) -> Vec<Option<crate::host_viewmodel::HostOwnerToken>> {
        let Some(list) = self.native().property(name) else {
            return Vec::new();
        };
        list.with_downcast::<ViewModelInstanceList, _>(|list| list.list_items().to_vec())
            .unwrap_or_default()
            .iter()
            .map(|item| {
                item.with_downcast::<ViewModelInstanceListItem, _>(|item| item.host_owner().cloned())
                    .flatten()
            })
            .collect()
    }
}

fn copy_shape_of(file: &File, source: CoreHandle, max_instances: usize) -> CopyShape {
    use crate::host_viewmodel::ViewModelCopyShape;
    use crate::mechanical_port::source::viewmodel::viewmodel::ViewModel;
    // As `File::copy_view_model_instance` copies: each source once (a source
    // reached again, or a back-edge to one being copied, shares the copy),
    // nested ones only while the depth is under 256; in its order, so the
    // depth a source is first reached at is the copy's.
    const MAX_DEPTH: usize = 256;
    let mut shape = ViewModelCopyShape::default();
    let mut seen = std::collections::HashSet::new();
    let mut pending = vec![(source, 1_usize)];
    while let Some((instance, depth)) = pending.pop() {
        if !seen.insert(instance.clone()) {
            continue;
        }
        if shape.instances >= max_instances {
            return CopyShape::Unbounded;
        }
        shape.instances = shape.instances.saturating_add(1);
        let nest = depth < MAX_DEPTH;
        let mut children = Vec::new();
        let Some((model_id, values)) = instance
            .with(|instance| {
                let instance = instance.as_view_model_instance()?;
                Some((
                    instance.base.view_model_id() as usize,
                    instance.property_values().to_vec(),
                ))
            })
            .flatten()
        else {
            continue;
        };
        let Some(model) = file.view_model_handle(model_id) else {
            continue;
        };
        // The properties a script sees on the copy, as `properties()` lists them.
        for name in properties_of(&model).into_keys() {
            shape.properties = shape.properties.saturating_add(1);
            shape.name_bytes = shape.name_bytes.saturating_add(name.len());
        }
        for value in values {
            if let Some(len) = value.with_downcast::<ViewModelInstanceString, _>(|value| {
                value.base.property_value().len()
            }) {
                shape.string_bytes = shape.string_bytes.saturating_add(len);
                continue;
            }
            let nested = value
                .with(|value| {
                    let index = value.as_view_model_instance_view_model()?.base.property_value();
                    let property = value.as_view_model_instance_value()?.base.view_model_property_id();
                    Some((property as usize, index as usize))
                })
                .flatten();
            if let Some((property_id, index)) = nested {
                let source = model
                    .with_downcast::<ViewModel, _>(|model| model.property_at(property_id))
                    .flatten()
                    .and_then(|property| {
                        property.with_downcast::<ViewModelPropertyViewModel, _>(|property| {
                            property.base.view_model_reference_id() as usize
                        })
                    })
                    .and_then(|reference| file.view_model_handle(reference))
                    .and_then(|model| {
                        model
                            .with_downcast::<ViewModel, _>(|model| model.instance_at(index))
                            .flatten()
                    });
                children.extend(source.filter(|_| nest));
                continue;
            }
            let items = value
                .with_downcast::<ViewModelInstanceList, _>(|list| list.list_items().to_vec())
                .unwrap_or_default();
            for item in items {
                shape.list_items = shape.list_items.saturating_add(1);
                let source = item
                    .with_downcast::<ViewModelInstanceListItem, _>(|item| {
                        (
                            item.base.view_model_id() as usize,
                            item.base.view_model_instance_id() as usize,
                        )
                    })
                    .and_then(|(model_id, instance_id)| {
                        file.view_model_handle(model_id)?
                            .with_downcast::<ViewModel, _>(|model| model.instance_at(instance_id))
                            .flatten()
                    });
                children.extend(source.filter(|_| nest));
            }
        }
        // Visited in order: the first child is popped first.
        pending.extend(children.into_iter().rev().map(|child| (child, depth + 1)));
    }
    CopyShape::Shape(shape)
}

#[cfg(test)]
mod copy_shape_tests {
    use super::*;
    use crate::mechanical_port::source::generated::{
        core_registry::CoreRegistry,
        viewmodel::{
            viewmodel_instance_viewmodel_base::ViewModelInstanceViewModelBase,
            viewmodel_property_viewmodel_base::ViewModelPropertyViewModelBase,
        },
    };
    use crate::mechanical_port::source::viewmodel::{
        viewmodel::ViewModel, viewmodel_instance_viewmodel::ViewModelInstanceViewModel,
    };
    use nuxie_render_api::{PersistentFactory, RecordingFactory};
    use std::collections::HashSet;

    fn import(asset: &str) -> RuntimeFileHandle {
        let root = std::path::PathBuf::from(
            std::env::var_os("RIVE_RUNTIME_DIR").expect("pinned upstream checkout"),
        );
        let bytes = std::fs::read(root.join("tests/unit_tests/assets").join(asset)).unwrap();
        let mut factory = PersistentFactory::new(RecordingFactory::new());
        let factory = crate::RuntimeFactoryHandle::from_factory(&mut factory).unwrap();
        File::import(&bytes, factory, None, None, None).unwrap()
    }

    fn authored(file: &File) -> Vec<CoreHandle> {
        (0..file.view_model_count())
            .filter_map(|i| file.view_model_handle(i))
            .flat_map(|model| {
                let count = model
                    .with_downcast::<ViewModel, _>(ViewModel::instance_count)
                    .unwrap_or(0);
                (0..count)
                    .filter_map(|j| {
                        model
                            .with_downcast::<ViewModel, _>(|model| model.instance_at(j))
                            .flatten()
                    })
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    /// The instances a copy made: the copied one and each nested one it
    /// reaches, once each.
    fn made(copy: &CoreHandle) -> usize {
        let mut seen = HashSet::new();
        let mut pending = vec![copy.clone()];
        while let Some(instance) = pending.pop() {
            if !seen.insert(instance.clone()) {
                continue;
            }
            let values = instance
                .with_downcast::<ViewModelInstance, _>(|i| i.property_values().to_vec())
                .unwrap_or_default();
            for value in values {
                pending.extend(
                    value
                        .with_downcast::<ViewModelInstanceViewModel, _>(
                            ViewModelInstanceViewModel::reference_view_model_instance,
                        )
                        .flatten(),
                );
                let items = value
                    .with_downcast::<ViewModelInstanceList, _>(|l| l.list_items().to_vec())
                    .unwrap_or_default();
                pending.extend(items.iter().filter_map(|item| {
                    item.with_downcast::<ViewModelInstanceListItem, _>(
                        ViewModelInstanceListItem::view_model_instance,
                    )
                    .flatten()
                }));
            }
        }
        seen.len()
    }

    fn assert_measured(file: &RuntimeFileHandle, asset: &str) {
        file.with_file(|file| {
            for source in authored(file) {
                let shape = copy_shape_of(file, source.clone(), 4096);
                let copy = file.copy_view_model_instance(Some(&source)).expect("copy");
                match shape {
                    CopyShape::Shape(shape) => {
                        assert_eq!(shape.instances, made(&copy), "{asset}")
                    }
                    other => panic!("{asset}: {other:?}"),
                }
            }
        });
    }

    // Lane H2 (codex review): the measure walked a cycle the copy cuts until
    // it gave up, so a named copy the runtime makes was refused.
    #[test]
    fn a_measured_copy_is_what_the_copy_makes() {
        for asset in [
            "rebind_with_nested_viewmodel.riv",
            "component_list_virtualized.riv",
            "viewmodel_deep_chain.riv",
        ] {
            assert_measured(&import(asset), asset);
        }
        // A self-referential authored instance, made as the upstream cycle
        // test makes one.
        let file = import("rebind_with_nested_viewmodel.riv");
        let found = file.with_file(|file| {
            for i in 0..file.view_model_count() {
                let model = file.view_model_handle(i)?;
                let count = model.with_downcast::<ViewModel, _>(ViewModel::instance_count)?;
                for j in 0..count {
                    let instance = model
                        .with_downcast::<ViewModel, _>(|model| model.instance_at(j))
                        .flatten()?;
                    let values = instance
                        .with_downcast::<ViewModelInstance, _>(|i| i.property_values().to_vec())?;
                    for value in values {
                        let Some(id) = value.with_downcast::<ViewModelInstanceViewModel, _>(
                            |value| value.base.view_model_property_id(),
                        ) else {
                            continue;
                        };
                        let property = model
                            .with_downcast::<ViewModel, _>(|m| m.property_at(id as usize))
                            .flatten()?;
                        return Some((i, j, property, value));
                    }
                }
            }
            None
        });
        let (model, instance, property, value) = found.expect("a nested reference");
        assert!(CoreRegistry::set_uint_handle(
            &property,
            i32::from(ViewModelPropertyViewModelBase::VIEW_MODEL_REFERENCE_ID_PROPERTY_KEY),
            model as u32,
        ));
        assert!(CoreRegistry::set_uint_handle(
            &value,
            i32::from(ViewModelInstanceViewModelBase::PROPERTY_VALUE_PROPERTY_KEY),
            instance as u32,
        ));
        assert_measured(&file, "self-referential");
    }
}
