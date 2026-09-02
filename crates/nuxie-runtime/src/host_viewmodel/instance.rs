use super::*;
use crate::mechanical_port::source::data_bind::data_values::data_value_integer::DataValueInteger;
use crate::mechanical_port::source::viewmodel::{
    viewmodel_instance_artboard::ViewModelInstanceArtboard,
    viewmodel_instance_asset_blob::ViewModelInstanceAssetBlob,
    viewmodel_instance_boolean::ViewModelInstanceBoolean,
    viewmodel_instance_color::ViewModelInstanceColor,
    viewmodel_instance_enum::ViewModelInstanceEnum, viewmodel_instance_list::ViewModelInstanceList,
    viewmodel_instance_list_item::ViewModelInstanceListItem,
    viewmodel_instance_number::ViewModelInstanceNumber,
    viewmodel_instance_string::ViewModelInstanceString,
    viewmodel_instance_symbol_list_index::ViewModelInstanceSymbolListIndex,
    viewmodel_instance_trigger::ViewModelInstanceTrigger,
    viewmodel_property_boolean::ViewModelPropertyBoolean,
    viewmodel_property_string::ViewModelPropertyString,
};
use crate::view_model_cell::RuntimeHostMutationNotifications;
impl RuntimeOwnedViewModelInstance {
    pub fn asset_source_handle_by_property_name(
        &self,
        name: &str,
    ) -> Option<RuntimeOwnedViewModelAssetSourceHandle> {
        self.asset_source_handle_by_property_name_path(name)
    }
    pub fn asset_source_handle_by_property_name_path(
        &self,
        name: &str,
    ) -> Option<RuntimeOwnedViewModelAssetSourceHandle> {
        let property_path = self.path_named(name)?;
        self.property_by_path(&property_path)?
            .with(|property| property.as_view_model_instance_asset().map(|_| ()))??;
        Some(RuntimeOwnedViewModelAssetSourceHandle { property_path })
    }
    pub fn asset_value_by_property_name_path(&self, name: &str) -> Option<u64> {
        let path = self.path_named(name)?;
        self.property_by_path(&path)?
            .with(|property| {
                property
                    .as_view_model_instance_asset()
                    .map(|asset| asset.base.property_value() as u64)
            })
            .flatten()
    }
    pub fn artboard_value_by_property_name_path(&self, name: &str) -> Option<u64> {
        let path = self.path_named(name)?;
        self.property_by_path(&path)?
            .with_downcast::<ViewModelInstanceArtboard, _>(|value| {
                value.base.property_value() as u64
            })
    }
    pub fn blob_asset_value_by_property_name_path(
        &self,
        name: &str,
    ) -> Option<RuntimeBlobAssetValue> {
        let path = self.path_named(name)?;
        self.property_by_path(&path)?
            .with_downcast::<ViewModelInstanceAssetBlob, _>(|value| {
                value.asset().map_or_else(
                    || {
                        RuntimeBlobAssetValue::from_file_asset_index(
                            value.base.property_value() as u64
                        )
                    },
                    RuntimeBlobAssetValue::from_live_asset,
                )
            })
    }
    pub fn set_asset_by_property_name_path(&mut self, name: &str, value: u64) -> bool {
        let Some(handle) = self.asset_source_handle_by_property_name_path(name) else {
            return false;
        };
        self.set_asset_by_source_handle(&handle, value)
    }
    pub fn set_asset_by_property_name(&mut self, name: &str, value: u64) -> bool {
        self.set_asset_by_property_name_path(name, value)
    }
    pub fn set_asset_by_source_handle(
        &mut self,
        handle: &RuntimeOwnedViewModelAssetSourceHandle,
        value: u64,
    ) -> bool {
        let Ok(value) = u32::try_from(value) else {
            return false;
        };
        let Some(property) = self.property_by_path(handle.path()) else {
            return false;
        };
        let previous = property
            .with(|property| {
                property
                    .as_view_model_instance_asset()
                    .map(|asset| asset.base.property_value())
            })
            .flatten();
        if previous.is_none() || previous == Some(value) {
            return false;
        }
        mutate(|| {
            crate::mechanical_port::source::generated::core_registry::CoreRegistry::set_uint_handle(&property,crate::mechanical_port::source::generated::viewmodel::viewmodel_instance_asset_base::ViewModelInstanceAssetBase::PROPERTY_VALUE_PROPERTY_KEY as i32,value)
        })
    }
    pub fn view_model_source_handle_by_property_name_path(
        &self,
        name: &str,
    ) -> Option<RuntimeOwnedViewModelViewModelSourceHandle> {
        let property_path = self.path_named(name)?;
        self.property_by_path(&property_path)?
            .with(|property| property.as_view_model_instance_view_model().map(|_| ()))??;
        Some(RuntimeOwnedViewModelViewModelSourceHandle { property_path })
    }
    pub fn list_item_count_by_property_name(&self, name: &str) -> Option<usize> {
        self.list_item_count_by_property_name_path(name)
    }
}
impl RuntimeOwnedViewModelHandle {
    pub fn linked_view_model_by_property_name_path(&self, path: &str) -> Option<Self> {
        if path.is_empty() || path.contains('/') {
            return None;
        }
        let instance = self.borrow();
        let property = instance.property_by_path(&instance.path_named(path)?)?;
        let linked = property
            .with(|property| {
                property
                    .as_view_model_instance_view_model()?
                    .reference_view_model_instance()
            })
            .flatten()?;
        Self::from_native(instance.native_file(), linked)
    }
    pub fn linked_view_model_by_property_path(&self, path: &[usize]) -> Option<Self> {
        let instance = self.borrow();
        let linked = instance
            .property_by_path(path)?
            .with(|property| {
                property
                    .as_view_model_instance_view_model()?
                    .reference_view_model_instance()
            })
            .flatten()?;
        Self::from_native(instance.native_file(), linked)
    }
    fn accepts_child(&self, child: &Self) -> bool {
        child
            .reachable_change_owner_snapshot()
            .is_some_and(|owners| owners.iter().all(|owner| !owner.ptr_eq(self)))
    }
    pub fn link_view_model_by_property_name_path(
        &self,
        path: &str,
        child: &Self,
    ) -> Result<bool, RuntimeViewModelLinkError> {
        if path.is_empty() {
            return Err(RuntimeViewModelLinkError::PropertyNotFound);
        }
        let property_path = {
            let instance = self.borrow();
            instance
                .path_named(path)
                .ok_or(RuntimeViewModelLinkError::PropertyNotFound)?
        };
        let (property_index, owner_path) = property_path
            .split_last()
            .ok_or(RuntimeViewModelLinkError::PropertyNotFound)?;
        let owner = if owner_path.is_empty() {
            self.clone()
        } else {
            self.linked_view_model_by_property_path(owner_path)
                .ok_or(RuntimeViewModelLinkError::PropertyNotFound)?
        };
        let property = {
            let instance = owner.borrow();
            instance
                .property_by_path(&[*property_index])
                .ok_or(RuntimeViewModelLinkError::PropertyNotFound)?
        };
        let definition = property
            .with(|property| {
                property
                    .as_view_model_instance_value()?
                    .view_model_property()
            })
            .flatten()
            .ok_or(RuntimeViewModelLinkError::SchemaMismatch)?;
        let expected=definition.with_downcast::<crate::mechanical_port::source::viewmodel::viewmodel_property_viewmodel::ViewModelPropertyViewModel,_>(|definition|definition.base.view_model_reference_id()).ok_or(RuntimeViewModelLinkError::SchemaMismatch)? as usize;
        if expected != child.borrow().view_model_index() {
            return Err(RuntimeViewModelLinkError::SchemaMismatch);
        }
        if !owner.accepts_child(child) {
            return Err(RuntimeViewModelLinkError::Cycle);
        }
        Ok(mutate(|| {
            ViewModelInstance::replace_view_model_property_occurrence(
                &owner.native_handle(),
                &property,
                Some(child.native_handle()),
            )
        }))
    }
    fn list_property(&self, path: &str) -> Option<CoreHandle> {
        let owner = self.borrow();
        let property = owner.property_by_path(&owner.path_named(path)?)?;
        property.with_downcast::<ViewModelInstanceList, _>(|_| ())?;
        Some(property)
    }
    pub fn testing_list_items_by_property_name(&self, path: &str) -> Option<Vec<Self>> {
        let items = self
            .list_property(path)?
            .with_downcast::<ViewModelInstanceList, _>(|list| list.list_items().to_vec())?;
        items
            .into_iter()
            .filter_map(|item| {
                item.with_downcast::<ViewModelInstanceListItem, _>(
                    ViewModelInstanceListItem::view_model_instance,
                )
                .flatten()
            })
            .map(|instance| Self::from_native(self.native_file(), instance))
            .collect()
    }
    pub fn list_items_by_property_name_path(&self, path: &str) -> Option<Vec<Self>> {
        self.testing_list_items_by_property_name(path)
    }
    pub fn list_item_count_by_property_name_path(&self, path: &str) -> Option<usize> {
        self.borrow().list_item_count_by_property_name_path(path)
    }
    pub fn insert_list_item_by_property_name_path(
        &self,
        path: &str,
        index: usize,
        item: &Self,
    ) -> bool {
        if !self.accepts_child(item) {
            return false;
        }
        let Some(property) = self.list_property(path) else {
            return false;
        };
        let count = property
            .with_downcast::<ViewModelInstanceList, _>(|list| list.list_items().len())
            .unwrap();
        if index > count {
            return false;
        }
        let Some(list_item) = self
            .native_file()
            .with_file_mut(|file| file.view_model_instance_list_item(item.native_handle()))
        else {
            return false;
        };
        mutate(|| {
            property.with_downcast_mut::<ViewModelInstanceList, _>(|list| {
                list.add_item_at(list_item, index as i32)
            })
        })
        .unwrap_or(false)
    }
    pub fn push_list_item_by_property_name_path(&self, path: &str, item: &Self) -> bool {
        let Some(property) = self.list_property(path) else {
            return false;
        };
        let count = property
            .with_downcast::<ViewModelInstanceList, _>(|list| list.list_items().len())
            .unwrap();
        self.insert_list_item_by_property_name_path(path, count, item)
    }
    pub fn remove_list_item_by_property_name_path(&self, path: &str, index: usize) -> bool {
        let Some(property) = self.list_property(path) else {
            return false;
        };
        let count = property
            .with_downcast::<ViewModelInstanceList, _>(|list| list.list_items().len())
            .unwrap();
        if index >= count {
            return false;
        }
        mutate(|| {
            property.with_downcast_mut::<ViewModelInstanceList, _>(|list| {
                list.remove_item_at(index as i32)
            })
        })
        .is_some()
    }
    pub fn swap_list_items_by_property_name_path(&self, path: &str, a: usize, b: usize) -> bool {
        let Some(property) = self.list_property(path) else {
            return false;
        };
        let count = property
            .with_downcast::<ViewModelInstanceList, _>(|list| list.list_items().len())
            .unwrap();
        if a >= count || b >= count {
            return false;
        }
        mutate(|| {
            property
                .with_downcast_mut::<ViewModelInstanceList, _>(|list| list.swap(a as u32, b as u32))
        })
        .is_some()
    }
    pub fn move_list_item_by_property_name_path(&self, path: &str, from: usize, to: usize) -> bool {
        let Some(property) = self.list_property(path) else {
            return false;
        };
        let mut items = property
            .with_downcast::<ViewModelInstanceList, _>(|list| list.list_items().to_vec())
            .unwrap();
        if from >= items.len() || to >= items.len() {
            return false;
        }
        let item = items.remove(from);
        items.insert(to, item);
        mutate(|| {
            property.with_downcast_mut::<ViewModelInstanceList, _>(|list| {
                list.update_list(Some(&items))
            })
        })
        .is_some()
    }
    pub fn set_list_item_by_property_name_path(
        &self,
        path: &str,
        index: usize,
        item: &Self,
    ) -> bool {
        if !self.accepts_child(item) {
            return false;
        }
        let Some(property) = self.list_property(path) else {
            return false;
        };
        mutate(|| {
            property.with_downcast_mut::<ViewModelInstanceList, _>(|list| {
                list.set_host_item_instance(index, item.native_handle())
            })
        })
        .unwrap_or(false)
    }
    pub fn clear_list_items_by_property_name_path(&self, path: &str) -> bool {
        let Some(property) = self.list_property(path) else {
            return false;
        };
        let changed = property
            .with_downcast::<ViewModelInstanceList, _>(|list| !list.list_items().is_empty())
            .unwrap();
        mutate(|| {
            property.with_downcast_mut::<ViewModelInstanceList, _>(
                ViewModelInstanceList::remove_all_items,
            )
        });
        changed
    }
}
thread_local! {
    static HOST_IDENTITIES: RefCell<BTreeMap<(usize,usize,u64),u64>> = RefCell::new(BTreeMap::new());
    static HOST_GENERATIONS: RefCell<BTreeMap<(usize,usize,u64),u64>> = RefCell::new(BTreeMap::new());
}
pub(super) fn identity(owner: &CoreHandle) -> u64 {
    HOST_IDENTITIES.with(|identities| {
        let mut identities = identities.borrow_mut();
        let next = identities.len() as u64 + 1;
        *identities.entry(owner.identity_key()).or_insert(next)
    })
}
pub(super) fn mutate<R>(body: impl FnOnce() -> R) -> R {
    let notifications = RuntimeHostMutationNotifications::begin();
    let result = body();
    if let Some(notifications) = notifications {
        notifications.commit();
    }
    result
}
pub struct RuntimeOwnedViewModelInstance {
    pub(super) file: RuntimeFileHandle,
    pub(super) instance: CoreHandle,
    pub(crate) view_model_index: usize,
}
impl std::fmt::Debug for RuntimeOwnedViewModelInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RuntimeOwnedViewModelInstance")
            .field("instance", &self.instance)
            .field("view_model_index", &self.view_model_index)
            .finish()
    }
}
impl Clone for RuntimeOwnedViewModelInstance {
    fn clone(&self) -> Self {
        Self::from_native(
            self.file.clone(),
            self.instance
                .clone_occurrence()
                .expect("native view model clone"),
        )
        .expect("native view model clone type")
    }
}
impl RuntimeOwnedViewModelInstance {
    pub fn from_native(file: RuntimeFileHandle, instance: CoreHandle) -> Option<Self> {
        let model = instance
            .with_downcast::<ViewModelInstance, _>(ViewModelInstance::get_view_model)
            .flatten()?;
        let view_model_index = file.with_file(|file| {
            (0..file.view_model_count())
                .find(|index| file.view_model(*index).as_ref() == Some(&model))
        })?;
        Some(Self {
            file,
            instance,
            view_model_index,
        })
    }
    pub fn new(file: RuntimeFileHandle, view_model_index: usize) -> Option<Self> {
        let instance = file.with_file_mut(|file| {
            file.create_view_model_instance(file.view_model(view_model_index)?)
        })?;
        Self::from_native(file, instance)
    }
    pub fn from_instance(
        file: RuntimeFileHandle,
        view_model_index: usize,
        instance_index: usize,
    ) -> Option<Self> {
        let instance = file.with_file(|file| {
            file.create_view_model_instance_at(view_model_index, instance_index)
        })?;
        Self::from_native(file, instance)
    }
    pub fn from_instance_name(
        file: RuntimeFileHandle,
        view_model_index: usize,
        name: &str,
    ) -> Option<Self> {
        let instance = file.with_file(|file| {
            let model = file.view_model(view_model_index)?;
            let model_name =
                model.with(|model| model.as_view_model().unwrap().base.name().to_owned())?;
            file.create_view_model_instance_named(&model_name, name)
        })?;
        Self::from_native(file, instance)
    }
    pub fn native_handle(&self) -> CoreHandle {
        self.instance.clone()
    }
    pub fn native_file(&self) -> RuntimeFileHandle {
        self.file.clone()
    }
    pub fn view_model_index(&self) -> usize {
        self.view_model_index
    }
    pub fn instance_identity(&self) -> u64 {
        identity(&self.instance)
    }
    pub fn allocation_identity(&self) -> u64 {
        identity(&self.instance)
    }
    pub fn name(&self) -> String {
        self.instance
            .with_downcast::<ViewModelInstance, _>(|instance| instance.base.name().to_owned())
            .expect("native instance")
    }
    pub fn property_count(&self) -> usize {
        self.instance
            .with_downcast::<ViewModelInstance, _>(|instance| instance.property_values().len())
            .expect("native instance")
    }
    pub(super) fn model(&self) -> CoreHandle {
        self.instance
            .with_downcast::<ViewModelInstance, _>(ViewModelInstance::get_view_model)
            .flatten()
            .expect("native instance definition")
    }
    pub fn property_index_by_name(&self, name: &str) -> Option<usize> {
        let properties = self
            .model()
            .with(|model| model.as_view_model().unwrap().properties())?;
        properties.iter().position(|property| {
            property
                .with(|property| property.as_view_model_property().unwrap().const_name() == name)
                == Some(true)
        })
    }
    fn unique_property_index_by_name(&self, name: &str) -> Option<usize> {
        let properties = self
            .model()
            .with(|model| model.as_view_model().unwrap().properties())?;
        let mut matches = properties
            .iter()
            .enumerate()
            .filter_map(|(index, property)| {
                (property.with(|property| {
                    property
                        .as_view_model_property()
                        .is_some_and(|property| property.base.name() == name)
                }) == Some(true))
                .then_some(index)
            });
        let index = matches.next()?;
        matches.next().is_none().then_some(index)
    }
    fn unique_string_property_index_by_name(&self, name: &str) -> Option<usize> {
        let index = self.unique_property_index_by_name(name)?;
        self.model()
            .with(|model| model.as_view_model().unwrap().property_at(index))??
            .with_downcast::<ViewModelPropertyString, _>(|_| ())?;
        self.property_by_path(&[index])?
            .with_downcast::<ViewModelInstanceString, _>(|_| ())?;
        Some(index)
    }
    fn unique_boolean_property_index_by_name(&self, name: &str) -> Option<usize> {
        let index = self.unique_property_index_by_name(name)?;
        self.model()
            .with(|model| model.as_view_model().unwrap().property_at(index))??
            .with_downcast::<ViewModelPropertyBoolean, _>(|_| ())?;
        self.property_by_path(&[index])?
            .with_downcast::<ViewModelInstanceBoolean, _>(|_| ())?;
        Some(index)
    }
    pub(super) fn property_by_path(&self, path: &[usize]) -> Option<CoreHandle> {
        let mut instance = self.instance.clone();
        for (index, id) in path.iter().enumerate() {
            let value = instance
                .with_downcast::<ViewModelInstance, _>(|instance| {
                    instance.property_value_by_id(u32::try_from(*id).ok()?)
                })
                .flatten()?;
            if index + 1 == path.len() {
                return Some(value);
            }
            instance = value
                .with(|value| {
                    value
                        .as_view_model_instance_view_model()?
                        .reference_view_model_instance()
                })
                .flatten()?;
        }
        None
    }
    pub(super) fn path_named(&self, path: &str) -> Option<Vec<usize>> {
        if path.is_empty() {
            return None;
        }
        let mut instance = self.instance.clone();
        let mut result = Vec::new();
        let names = path.split('/').collect::<Vec<_>>();
        for (index, name) in names.iter().enumerate() {
            let value = instance
                .with_downcast::<ViewModelInstance, _>(|instance| {
                    instance.property_value_named(name)
                })
                .flatten()?;
            let id = value.with(|value| {
                value
                    .as_view_model_instance_value()
                    .unwrap()
                    .base
                    .view_model_property_id()
            })? as usize;
            result.push(id);
            if index + 1 != names.len() {
                instance = value
                    .with(|value| {
                        value
                            .as_view_model_instance_view_model()?
                            .reference_view_model_instance()
                    })
                    .flatten()?;
            }
        }
        Some(result)
    }
    pub fn advanced(&mut self) {
        mutate(|| {
            self.instance
                .with_downcast_mut::<ViewModelInstance, _>(ViewModelInstance::advanced);
        });
    }
    pub fn has_parents(&self) -> bool {
        self.instance
            .with_downcast::<ViewModelInstance, _>(ViewModelInstance::has_parents)
            .expect("native instance")
    }

    pub fn number_source_handle_by_property_name(
        &self,
        name: &str,
    ) -> Option<RuntimeOwnedViewModelNumberSourceHandle> {
        self.number_source_handle_by_property_name_path(name)
    }
    pub fn number_source_handle_by_property_name_path(
        &self,
        path: &str,
    ) -> Option<RuntimeOwnedViewModelNumberSourceHandle> {
        let property_path = self.path_named(path)?;
        self.property_by_path(&property_path)?
            .with_downcast::<ViewModelInstanceNumber, _>(|_| ())?;
        Some(RuntimeOwnedViewModelNumberSourceHandle { property_path })
    }
    pub fn number_value_by_property_name(&self, name: &str) -> Option<f32> {
        self.number_value_by_property_name_path(name)
    }
    pub fn number_value_by_property_name_path(&self, path: &str) -> Option<f32> {
        self.number_value_by_source_handle(&self.number_source_handle_by_property_name_path(path)?)
    }
    pub fn number_value_by_source_handle(
        &self,
        handle: &RuntimeOwnedViewModelNumberSourceHandle,
    ) -> Option<f32> {
        self.property_by_path(handle.path())?
            .with_downcast::<ViewModelInstanceNumber, _>(|value| value.value())
    }
    pub fn set_number_by_property_name(&mut self, name: &str, value: f32) -> bool {
        self.set_number_by_property_name_path(name, value)
    }
    pub fn set_number_by_property_name_path(&mut self, path: &str, value: f32) -> bool {
        let Some(handle) = self.number_source_handle_by_property_name_path(path) else {
            return false;
        };
        self.set_number_by_source_handle(&handle, value)
    }
    pub fn set_number_by_property_index(&mut self, index: usize, value: f32) -> bool {
        let Some(property) = self.property_by_path(std::slice::from_ref(&index)) else {
            return false;
        };
        Self::set_number_property(property, value)
    }
    pub fn set_number_by_source_handle(
        &mut self,
        handle: &RuntimeOwnedViewModelNumberSourceHandle,
        value: f32,
    ) -> bool {
        let Some(property) = self.property_by_path(handle.path()) else {
            return false;
        };
        Self::set_number_property(property, value)
    }
    fn set_number_property(property: CoreHandle, value: f32) -> bool {
        let Some(previous) =
            property.with_downcast::<ViewModelInstanceNumber, _>(|value| value.value())
        else {
            return false;
        };
        if previous == value {
            return false;
        }
        mutate(|| {
            property.with_downcast_mut::<ViewModelInstanceNumber, _>(|property| {
                property.set_value(value);
            })
        })
        .is_some()
    }

    pub fn boolean_source_handle_by_property_name(
        &self,
        name: &str,
    ) -> Option<RuntimeOwnedViewModelBooleanSourceHandle> {
        self.boolean_source_handle_by_property_name_path(name)
    }
    pub fn boolean_source_handle_by_property_name_path(
        &self,
        path: &str,
    ) -> Option<RuntimeOwnedViewModelBooleanSourceHandle> {
        let property_path = self.path_named(path)?;
        self.property_by_path(&property_path)?
            .with_downcast::<ViewModelInstanceBoolean, _>(|_| ())?;
        Some(RuntimeOwnedViewModelBooleanSourceHandle { property_path })
    }
    pub fn boolean_value_by_property_name(&self, name: &str) -> Option<bool> {
        self.boolean_value_by_property_name_path(name)
    }
    pub fn boolean_value_by_property_name_path(&self, path: &str) -> Option<bool> {
        self.boolean_value_by_source_handle(
            &self.boolean_source_handle_by_property_name_path(path)?,
        )
    }
    pub fn boolean_value_by_source_handle(
        &self,
        handle: &RuntimeOwnedViewModelBooleanSourceHandle,
    ) -> Option<bool> {
        self.property_by_path(handle.path())?
            .with_downcast::<ViewModelInstanceBoolean, _>(|value| value.value())
    }
    pub fn set_boolean_by_property_name(&mut self, name: &str, value: bool) -> bool {
        self.set_boolean_by_property_name_path(name, value)
    }
    pub fn set_boolean_by_property_name_path(&mut self, path: &str, value: bool) -> bool {
        let Some(handle) = self.boolean_source_handle_by_property_name_path(path) else {
            return false;
        };
        self.set_boolean_by_source_handle(&handle, value)
    }
    pub fn set_boolean_by_property_index(&mut self, index: usize, value: bool) -> bool {
        self.set_boolean_by_source_handle(
            &RuntimeOwnedViewModelBooleanSourceHandle {
                property_path: vec![index],
            },
            value,
        )
    }
    pub fn set_boolean_by_source_handle(
        &mut self,
        handle: &RuntimeOwnedViewModelBooleanSourceHandle,
        value: bool,
    ) -> bool {
        let Some(property) = self.property_by_path(handle.path()) else {
            return false;
        };
        let Some(previous) =
            property.with_downcast::<ViewModelInstanceBoolean, _>(|value| value.value())
        else {
            return false;
        };
        if previous == value {
            return false;
        }
        mutate(|| {
            property.with_downcast_mut::<ViewModelInstanceBoolean, _>(|property| {
                property.set_value(value);
            })
        })
        .is_some()
    }

    pub fn color_source_handle_by_property_name(
        &self,
        name: &str,
    ) -> Option<RuntimeOwnedViewModelColorSourceHandle> {
        self.color_source_handle_by_property_name_path(name)
    }
    pub fn color_source_handle_by_property_name_path(
        &self,
        path: &str,
    ) -> Option<RuntimeOwnedViewModelColorSourceHandle> {
        let property_path = self.path_named(path)?;
        self.property_by_path(&property_path)?
            .with_downcast::<ViewModelInstanceColor, _>(|_| ())?;
        Some(RuntimeOwnedViewModelColorSourceHandle { property_path })
    }
    pub fn color_value_by_property_name(&self, name: &str) -> Option<u32> {
        self.color_value_by_property_name_path(name)
    }
    pub fn color_value_by_property_name_path(&self, path: &str) -> Option<u32> {
        self.color_value_by_source_handle(&self.color_source_handle_by_property_name_path(path)?)
    }
    pub fn color_value_by_source_handle(
        &self,
        handle: &RuntimeOwnedViewModelColorSourceHandle,
    ) -> Option<u32> {
        self.property_by_path(handle.path())?
            .with_downcast::<ViewModelInstanceColor, _>(|value| value.value() as u32)
    }
    pub fn set_color_by_property_name(&mut self, name: &str, value: u32) -> bool {
        self.set_color_by_property_name_path(name, value)
    }
    pub fn set_color_by_property_name_path(&mut self, path: &str, value: u32) -> bool {
        let Some(handle) = self.color_source_handle_by_property_name_path(path) else {
            return false;
        };
        self.set_color_by_source_handle(&handle, value)
    }
    pub fn set_color_by_property_index(&mut self, index: usize, value: u32) -> bool {
        self.set_color_by_source_handle(
            &RuntimeOwnedViewModelColorSourceHandle {
                property_path: vec![index],
            },
            value,
        )
    }
    pub fn set_color_by_source_handle(
        &mut self,
        handle: &RuntimeOwnedViewModelColorSourceHandle,
        value: u32,
    ) -> bool {
        let Some(property) = self.property_by_path(handle.path()) else {
            return false;
        };
        let Some(previous) =
            property.with_downcast::<ViewModelInstanceColor, _>(|value| value.value() as u32)
        else {
            return false;
        };
        if previous == value {
            return false;
        }
        mutate(|| {
            property.with_downcast_mut::<ViewModelInstanceColor, _>(|property| {
                property.set_value(value as i32);
            })
        })
        .is_some()
    }

    pub fn enum_source_handle_by_property_name(
        &self,
        name: &str,
    ) -> Option<RuntimeOwnedViewModelEnumSourceHandle> {
        self.enum_source_handle_by_property_name_path(name)
    }
    pub fn enum_source_handle_by_property_name_path(
        &self,
        path: &str,
    ) -> Option<RuntimeOwnedViewModelEnumSourceHandle> {
        let property_path = self.path_named(path)?;
        self.property_by_path(&property_path)?
            .with_downcast::<ViewModelInstanceEnum, _>(|_| ())?;
        Some(RuntimeOwnedViewModelEnumSourceHandle { property_path })
    }
    pub fn enum_value_by_property_name(&self, name: &str) -> Option<u64> {
        self.enum_value_by_property_name_path(name)
    }
    pub fn enum_value_by_property_name_path(&self, path: &str) -> Option<u64> {
        self.enum_value_by_source_handle(&self.enum_source_handle_by_property_name_path(path)?)
    }
    pub fn enum_value_by_source_handle(
        &self,
        handle: &RuntimeOwnedViewModelEnumSourceHandle,
    ) -> Option<u64> {
        self.property_by_path(handle.path())?
            .with_downcast::<ViewModelInstanceEnum, _>(|value| value.base.property_value() as u64)
    }
    pub fn set_enum_by_property_name(&mut self, name: &str, value: u64) -> bool {
        self.set_enum_by_property_name_path(name, value)
    }
    pub fn set_enum_by_property_name_path(&mut self, path: &str, value: u64) -> bool {
        let Some(handle) = self.enum_source_handle_by_property_name_path(path) else {
            return false;
        };
        self.set_enum_by_source_handle(&handle, value)
    }
    pub fn set_enum_by_property_index(&mut self, index: usize, value: u64) -> bool {
        self.set_enum_by_source_handle(
            &RuntimeOwnedViewModelEnumSourceHandle {
                property_path: vec![index],
            },
            value,
        )
    }
    pub fn set_enum_by_source_handle(
        &mut self,
        handle: &RuntimeOwnedViewModelEnumSourceHandle,
        value: u64,
    ) -> bool {
        let Some(property) = self.property_by_path(handle.path()) else {
            return false;
        };
        let Some(previous) = property
            .with_downcast::<ViewModelInstanceEnum, _>(|value| value.base.property_value() as u64)
        else {
            return false;
        };
        if previous == value {
            return false;
        }
        mutate(|| {
            property.with_downcast_mut::<ViewModelInstanceEnum, _>(|property| {
                property.apply_value(&DataValueInteger::new(value as u32));
            })
        })
        .is_some()
    }

    pub fn symbol_list_index_source_handle_by_property_name(
        &self,
        name: &str,
    ) -> Option<RuntimeOwnedViewModelSymbolListIndexSourceHandle> {
        self.symbol_list_index_source_handle_by_property_name_path(name)
    }
    pub fn symbol_list_index_source_handle_by_property_name_path(
        &self,
        path: &str,
    ) -> Option<RuntimeOwnedViewModelSymbolListIndexSourceHandle> {
        let property_path = self.path_named(path)?;
        self.property_by_path(&property_path)?
            .with_downcast::<ViewModelInstanceSymbolListIndex, _>(|_| ())?;
        Some(RuntimeOwnedViewModelSymbolListIndexSourceHandle { property_path })
    }
    pub fn symbol_list_index_value_by_property_name(&self, name: &str) -> Option<u64> {
        self.symbol_list_index_value_by_property_name_path(name)
    }
    pub fn symbol_list_index_value_by_property_name_path(&self, path: &str) -> Option<u64> {
        self.symbol_list_index_value_by_source_handle(
            &self.symbol_list_index_source_handle_by_property_name_path(path)?,
        )
    }
    pub fn symbol_list_index_value_by_source_handle(
        &self,
        handle: &RuntimeOwnedViewModelSymbolListIndexSourceHandle,
    ) -> Option<u64> {
        self.property_by_path(handle.path())?
            .with_downcast::<ViewModelInstanceSymbolListIndex, _>(|value| {
                value.base.property_value() as u64
            })
    }
    pub fn set_symbol_list_index_by_property_name(&mut self, name: &str, value: u64) -> bool {
        self.set_symbol_list_index_by_property_name_path(name, value)
    }
    pub fn set_symbol_list_index_by_property_name_path(&mut self, path: &str, value: u64) -> bool {
        let Some(handle) = self.symbol_list_index_source_handle_by_property_name_path(path) else {
            return false;
        };
        self.set_symbol_list_index_by_source_handle(&handle, value)
    }
    pub fn set_symbol_list_index_by_property_index(&mut self, index: usize, value: u64) -> bool {
        self.set_symbol_list_index_by_source_handle(
            &RuntimeOwnedViewModelSymbolListIndexSourceHandle {
                property_path: vec![index],
            },
            value,
        )
    }
    pub fn set_symbol_list_index_by_source_handle(
        &mut self,
        handle: &RuntimeOwnedViewModelSymbolListIndexSourceHandle,
        value: u64,
    ) -> bool {
        let Some(property) = self.property_by_path(handle.path()) else {
            return false;
        };
        let Some(previous) =
            property.with_downcast::<ViewModelInstanceSymbolListIndex, _>(|value| {
                value.base.property_value() as u64
            })
        else {
            return false;
        };
        if previous == value {
            return false;
        }
        mutate(|| {
            property.with_downcast_mut::<ViewModelInstanceSymbolListIndex, _>(|property| {
                property.apply_value(&DataValueInteger::new(value as u32));
            })
        })
        .is_some()
    }

    pub fn number_slot_by_property_index(&self, index: usize) -> Option<usize> {
        self.property_by_path(&[index])?
            .with_downcast::<ViewModelInstanceNumber, _>(|_| index)
    }
    pub fn number_value_by_slot(&self, index: usize) -> Option<f32> {
        self.property_by_path(std::slice::from_ref(&index))?
            .with_downcast::<ViewModelInstanceNumber, _>(|value| value.value())
    }
    pub fn set_number_by_slot(&mut self, index: usize, value: f32) -> bool {
        self.set_number_by_property_index(index, value)
    }
    pub fn string_source_handle_by_property_name(
        &self,
        name: &str,
    ) -> Option<RuntimeOwnedViewModelStringSourceHandle> {
        self.string_source_handle_by_property_name_path(name)
    }
    pub fn string_source_handle_by_property_name_path(
        &self,
        name: &str,
    ) -> Option<RuntimeOwnedViewModelStringSourceHandle> {
        let property_path = self.path_named(name)?;
        self.property_by_path(&property_path)?
            .with_downcast::<ViewModelInstanceString, _>(|_| ())?;
        Some(RuntimeOwnedViewModelStringSourceHandle { property_path })
    }
    pub fn string_value_by_property_name(&self, name: &str) -> Option<Arc<[u8]>> {
        self.string_value_by_property_name_path(name)
    }
    pub fn string_value_by_property_name_path(&self, name: &str) -> Option<Arc<[u8]>> {
        self.string_value_by_source_handle(&self.string_source_handle_by_property_name_path(name)?)
    }
    pub fn string_value_by_source_handle(
        &self,
        handle: &RuntimeOwnedViewModelStringSourceHandle,
    ) -> Option<Arc<[u8]>> {
        self.property_by_path(handle.path())?
            .with_downcast::<ViewModelInstanceString, _>(|value| {
                Arc::from(value.value().into_bytes())
            })
    }
    pub fn can_set_string_by_source_handle(
        &self,
        handle: &RuntimeOwnedViewModelStringSourceHandle,
    ) -> bool {
        self.string_value_by_source_handle(handle).is_some()
    }
    pub fn can_set_string_by_property_index(&self, index: usize) -> bool {
        self.property_by_path(&[index])
            .and_then(|property| property.with_downcast::<ViewModelInstanceString, _>(|_| ()))
            .is_some()
    }
    pub fn set_string_by_property_name(&mut self, name: &str, value: &[u8]) -> bool {
        self.set_string_by_property_name_path(name, value)
    }
    pub fn set_string_by_property_name_path(&mut self, name: &str, value: &[u8]) -> bool {
        let Some(handle) = self.string_source_handle_by_property_name_path(name) else {
            return false;
        };
        self.set_string_by_source_handle(&handle, value)
    }
    pub fn set_string_by_property_index(&mut self, index: usize, value: &[u8]) -> bool {
        self.set_string_by_source_handle(
            &RuntimeOwnedViewModelStringSourceHandle {
                property_path: vec![index],
            },
            value,
        )
    }
    pub fn set_string_by_source_handle(
        &mut self,
        handle: &RuntimeOwnedViewModelStringSourceHandle,
        value: &[u8],
    ) -> bool {
        let Ok(value) = std::str::from_utf8(value) else {
            return false;
        };
        let Some(property) = self.property_by_path(handle.path()) else {
            return false;
        };
        if property
            .with_downcast::<ViewModelInstanceString, _>(|property| property.value() == value)
            != Some(false)
        {
            return false;
        }
        mutate(|| {
            property.with_downcast_mut::<ViewModelInstanceString, _>(|property| {
                property.set_value(value)
            })
        })
        .is_some()
    }
    pub fn trigger_source_handle_by_property_name(
        &self,
        name: &str,
    ) -> Option<RuntimeOwnedViewModelTriggerSourceHandle> {
        self.trigger_source_handle_by_property_name_path(name)
    }
    pub fn trigger_source_handle_by_property_name_path(
        &self,
        name: &str,
    ) -> Option<RuntimeOwnedViewModelTriggerSourceHandle> {
        let property_path = self.path_named(name)?;
        self.property_by_path(&property_path)?
            .with_downcast::<ViewModelInstanceTrigger, _>(|_| ())?;
        Some(RuntimeOwnedViewModelTriggerSourceHandle { property_path })
    }
    pub fn trigger_value_by_property_name(&self, name: &str) -> Option<u64> {
        self.trigger_value_by_property_name_path(name)
    }
    pub fn trigger_value_by_property_name_path(&self, name: &str) -> Option<u64> {
        let path = self.path_named(name)?;
        self.property_by_path(&path)?
            .with_downcast::<ViewModelInstanceTrigger, _>(|v| v.base.property_value() as u64)
    }
    pub fn fire_trigger_by_property_name(&mut self, name: &str) -> bool {
        self.fire_trigger_by_property_name_path(name)
    }
    pub fn fire_trigger_by_property_name_path(&mut self, name: &str) -> bool {
        let Some(handle) = self.trigger_source_handle_by_property_name_path(name) else {
            return false;
        };
        self.fire_trigger_by_source_handle(&handle)
    }
    pub fn set_trigger_by_property_name_path(&mut self, name: &str, value: u64) -> bool {
        let Ok(value) = u32::try_from(value) else {
            return false;
        };
        let Some(handle) = self.trigger_source_handle_by_property_name_path(name) else {
            return false;
        };
        let Some(property) = self.property_by_path(handle.path()) else {
            return false;
        };
        let previous = property.with_downcast::<ViewModelInstanceTrigger, _>(|property| {
            property.base.property_value()
        });
        if previous.is_none() || previous == Some(value) {
            return false;
        }
        mutate(|| {
            crate::mechanical_port::source::generated::core_registry::CoreRegistry::set_uint_handle(
                &property,
                crate::mechanical_port::source::generated::viewmodel::viewmodel_instance_trigger_base::ViewModelInstanceTriggerBase::PROPERTY_VALUE_PROPERTY_KEY as i32,
                value,
            )
        })
    }
    pub fn fire_trigger_by_source_handle(
        &mut self,
        handle: &RuntimeOwnedViewModelTriggerSourceHandle,
    ) -> bool {
        let Some(property) = self.property_by_path(handle.path()) else {
            return false;
        };
        mutate(|| {
            property
                .with_downcast_mut::<ViewModelInstanceTrigger, _>(ViewModelInstanceTrigger::trigger)
        })
        .is_some()
    }
    pub fn list_source_handle_by_property_name(
        &self,
        name: &str,
    ) -> Option<RuntimeOwnedViewModelListSourceHandle> {
        self.list_source_handle_by_property_name_path(name)
    }
    pub fn list_source_handle_by_property_name_path(
        &self,
        name: &str,
    ) -> Option<RuntimeOwnedViewModelListSourceHandle> {
        let property_path = self.path_named(name)?;
        self.property_by_path(&property_path)?
            .with_downcast::<ViewModelInstanceList, _>(|_| ())?;
        Some(RuntimeOwnedViewModelListSourceHandle { property_path })
    }
    pub fn list_item_count_by_property_name_path(&self, name: &str) -> Option<usize> {
        let path = self.path_named(name)?;
        self.property_by_path(&path)?
            .with_downcast::<ViewModelInstanceList, _>(|list| list.list_items().len())
    }
    fn list_string_match_boolean_items(
        &self,
        handle: &RuntimeOwnedViewModelListStringMatchBooleanHandle,
    ) -> Option<Vec<RuntimeOwnedViewModelInstance>> {
        let items = self
            .property_by_path(&handle.list_property_path)?
            .with_downcast::<ViewModelInstanceList, _>(|list| list.list_items().to_vec())?;
        if items.is_empty() {
            return None;
        }
        let mut instances = Vec::with_capacity(items.len());
        for item in items {
            let instance = item
                .with_downcast::<ViewModelInstanceListItem, _>(
                    ViewModelInstanceListItem::view_model_instance,
                )
                .flatten()?;
            let instance = RuntimeOwnedViewModelInstance::from_native(self.file.clone(), instance)?;
            if instance.view_model_index != handle.item_view_model_index
                || instance
                    .property_by_path(&[handle.string_property_index])?
                    .with_downcast::<ViewModelInstanceString, _>(|_| ())
                    .is_none()
                || instance
                    .property_by_path(&[handle.boolean_property_index])?
                    .with_downcast::<ViewModelInstanceBoolean, _>(|_| ())
                    .is_none()
            {
                return None;
            }
            instances.push(instance);
        }
        Some(instances)
    }
    pub fn list_string_match_boolean_handle_by_property_name_path(
        &self,
        list_path: &str,
        string_property_name: &str,
        boolean_property_name: &str,
    ) -> Option<RuntimeOwnedViewModelListStringMatchBooleanHandle> {
        if string_property_name.is_empty()
            || string_property_name.contains('/')
            || boolean_property_name.is_empty()
            || boolean_property_name.contains('/')
        {
            return None;
        }
        let list_property_path = self.path_named(list_path)?;
        let items = self
            .property_by_path(&list_property_path)?
            .with_downcast::<ViewModelInstanceList, _>(|list| list.list_items().to_vec())?;
        let first = items
            .first()?
            .with_downcast::<ViewModelInstanceListItem, _>(
                ViewModelInstanceListItem::view_model_instance,
            )??;
        let first = RuntimeOwnedViewModelInstance::from_native(self.file.clone(), first)?;
        let handle = RuntimeOwnedViewModelListStringMatchBooleanHandle {
            list_property_path,
            item_view_model_index: first.view_model_index,
            string_property_index: first
                .unique_string_property_index_by_name(string_property_name)?,
            boolean_property_index: first
                .unique_boolean_property_index_by_name(boolean_property_name)?,
        };
        self.list_string_match_boolean_items(&handle)?;
        Some(handle)
    }
    pub fn can_apply_list_string_match_boolean(
        &self,
        handle: &RuntimeOwnedViewModelListStringMatchBooleanHandle,
    ) -> bool {
        self.list_string_match_boolean_items(handle).is_some()
    }
    pub fn apply_list_string_match_boolean(
        &mut self,
        handle: &RuntimeOwnedViewModelListStringMatchBooleanHandle,
        selected: &[u8],
    ) -> Option<bool> {
        let mut items = self.list_string_match_boolean_items(handle)?;
        let mut changed = false;
        for item in &mut items {
            let matches = item
                .string_value_by_source_handle(&RuntimeOwnedViewModelStringSourceHandle {
                    property_path: vec![handle.string_property_index],
                })
                .is_some_and(|value| value.as_ref() == selected);
            changed |= item.set_boolean_by_property_index(handle.boolean_property_index, matches);
        }
        Some(changed)
    }
}
#[derive(Clone, Debug)]
pub struct RuntimeOwnedViewModelHandle {
    instance: Rc<RefCell<RuntimeOwnedViewModelInstance>>,
}
impl RuntimeOwnedViewModelHandle {
    pub fn new(instance: RuntimeOwnedViewModelInstance) -> Self {
        Self {
            instance: Rc::new(RefCell::new(instance)),
        }
    }
    pub fn from_native(file: RuntimeFileHandle, instance: CoreHandle) -> Option<Self> {
        Some(Self::new(RuntimeOwnedViewModelInstance::from_native(
            file, instance,
        )?))
    }
    pub fn borrow(&self) -> Ref<'_, RuntimeOwnedViewModelInstance> {
        self.instance.borrow()
    }
    pub fn try_borrow(
        &self,
    ) -> Result<Ref<'_, RuntimeOwnedViewModelInstance>, std::cell::BorrowError> {
        self.instance.try_borrow()
    }
    pub fn borrow_mut(&self) -> RefMut<'_, RuntimeOwnedViewModelInstance> {
        self.instance.borrow_mut()
    }
    pub fn try_borrow_mut(
        &self,
    ) -> Result<RefMut<'_, RuntimeOwnedViewModelInstance>, std::cell::BorrowMutError> {
        self.instance.try_borrow_mut()
    }
    pub fn native_handle(&self) -> CoreHandle {
        self.instance.borrow().native_handle()
    }
    pub fn native_file(&self) -> RuntimeFileHandle {
        self.instance.borrow().native_file()
    }
    pub fn ptr_eq(&self, other: &Self) -> bool {
        self.native_handle() == other.native_handle()
    }
    pub fn instance_identity(&self) -> u64 {
        identity(&self.native_handle())
    }
    pub fn observable_mutation_generation(&self) -> u64 {
        HOST_GENERATIONS.with(|values| {
            values
                .borrow()
                .get(&self.native_handle().identity_key())
                .copied()
                .unwrap_or(0)
        })
    }
    pub fn mark_observable_mutation(&self, generation: u64) {
        fn visit(owner: CoreHandle, generation: u64, seen: &mut BTreeSet<(usize, usize, u64)>) {
            if !seen.insert(owner.identity_key()) {
                return;
            }
            HOST_GENERATIONS.with(|values| {
                values.borrow_mut().insert(owner.identity_key(), generation);
            });
            let parents = owner
                .with_downcast::<ViewModelInstance, _>(ViewModelInstance::parents)
                .expect("native instance");
            for parent in parents {
                visit(parent, generation, seen);
            }
        }
        visit(self.native_handle(), generation, &mut BTreeSet::new());
    }
    pub fn reachable_change_owner_snapshot(&self) -> Option<Vec<Self>> {
        fn visit(
            file: &RuntimeFileHandle,
            owner: CoreHandle,
            seen: &mut BTreeSet<(usize, usize, u64)>,
            out: &mut Vec<RuntimeOwnedViewModelHandle>,
        ) -> Option<()> {
            if !seen.insert(owner.identity_key()) {
                return Some(());
            }
            out.push(RuntimeOwnedViewModelHandle::from_native(
                file.clone(),
                owner.clone(),
            )?);
            let properties = owner
                .with_downcast::<ViewModelInstance, _>(|owner| owner.property_values().to_vec())?;
            for property in properties {
                if let Some(child) = property
                    .with(|property| {
                        property
                            .as_view_model_instance_view_model()?
                            .reference_view_model_instance()
                    })
                    .flatten()
                {
                    visit(file, child, seen, out)?;
                }
                if let Some(items) = property
                    .with_downcast::<ViewModelInstanceList, _>(|list| list.list_items().to_vec())
                {
                    for item in items {
                        if let Some(child) = item
                            .with_downcast::<ViewModelInstanceListItem, _>(
                                ViewModelInstanceListItem::view_model_instance,
                            )
                            .flatten()
                        {
                            visit(file, child, seen, out)?;
                        }
                    }
                }
            }
            Some(())
        }
        let mut out = Vec::new();
        visit(
            &self.native_file(),
            self.native_handle(),
            &mut BTreeSet::new(),
            &mut out,
        )?;
        Some(out)
    }
}
