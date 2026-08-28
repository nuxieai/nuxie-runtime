use super::*;
use crate::mechanical_port::source::{
    bindable_artboard::RuntimeBindableArtboardHandle,
    text_engine::FontRef,
    viewmodel::{
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
        viewmodel_instance_value::HostValueState,
        viewmodel_instance_viewmodel::ViewModelInstanceViewModel,
    },
};
use crate::view_model_cell::{RuntimeHostMutationNotifications, RuntimeHostTransactionPublication};
#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeViewModelChange {
    pub owner_instance_identity: u64,
    pub property_index: usize,
    pub value: RuntimeViewModelChangeValue,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeViewModelGraphTransactionError {
    Reentrant,
    BorrowConflict,
    LimitExceeded,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeViewModelLinkError {
    PropertyNotFound,
    NestedPathUnsupported,
    SchemaMismatch,
    Cycle,
    BorrowConflict,
}
enum Payload {
    Number(f32),
    Boolean(bool),
    String(String),
    Color(i32),
    Enum(u32),
    Index(u32),
    Trigger(u32),
    Image(u32, Option<Rc<dyn nuxie_render_api::RenderImage>>),
    Font(u32, Option<FontRef>),
    Blob(u32, Option<Arc<RuntimeBlobAsset>>),
    Artboard(
        u32,
        Option<RuntimeBindableArtboardHandle>,
        Option<CoreHandle>,
    ),
    List(Vec<(CoreHandle, Option<CoreHandle>)>),
    ViewModel(Option<CoreHandle>),
}
struct Snapshot {
    owner: CoreHandle,
    state: HostValueState,
    payload: Payload,
}
impl Snapshot {
    fn capture(owner: CoreHandle) -> Option<Self> {
        let (state, payload) = owner
            .with(|owner| {
                let state = owner.as_view_model_instance_value()?.host_snapshot();
                let value = owner.as_any();
                let payload = if let Some(v) = value.downcast_ref::<ViewModelInstanceNumber>() {
                    Payload::Number(v.value())
                } else if let Some(v) = value.downcast_ref::<ViewModelInstanceBoolean>() {
                    Payload::Boolean(v.value())
                } else if let Some(v) = value.downcast_ref::<ViewModelInstanceString>() {
                    Payload::String(v.value())
                } else if let Some(v) = value.downcast_ref::<ViewModelInstanceColor>() {
                    Payload::Color(v.value())
                } else if let Some(v) = value.downcast_ref::<ViewModelInstanceEnum>() {
                    Payload::Enum(v.base.property_value())
                } else if let Some(v) = value.downcast_ref::<ViewModelInstanceSymbolListIndex>() {
                    Payload::Index(v.base.property_value())
                } else if let Some(v) = value.downcast_ref::<ViewModelInstanceTrigger>() {
                    Payload::Trigger(v.base.property_value())
                } else if let Some(v) = value.downcast_ref::<ViewModelInstanceAssetImage>() {
                    Payload::Image(v.base.property_value(), v.asset().render_image())
                } else if let Some(v) = value.downcast_ref::<ViewModelInstanceAssetFont>() {
                    Payload::Font(v.base.property_value(), v.asset().font())
                } else if let Some(v) = value.downcast_ref::<ViewModelInstanceAssetBlob>() {
                    Payload::Blob(v.base.property_value(), v.asset())
                } else if let Some(v) = value.downcast_ref::<ViewModelInstanceArtboard>() {
                    Payload::Artboard(
                        v.base.property_value(),
                        v.asset(),
                        v.bound_view_model_instance(),
                    )
                } else if let Some(v) = value.downcast_ref::<ViewModelInstanceList>() {
                    Payload::List(
                        v.list_items()
                            .iter()
                            .map(|item| {
                                (
                                    item.clone(),
                                    item.with(|item| {
                                        item.as_view_model_instance_list_item()
                                            .unwrap()
                                            .view_model_instance()
                                    })
                                    .flatten(),
                                )
                            })
                            .collect(),
                    )
                } else if let Some(v) = value.downcast_ref::<ViewModelInstanceViewModel>() {
                    Payload::ViewModel(v.reference_view_model_instance())
                } else {
                    return None;
                };
                Some((state, payload))
            })
            .flatten()?;
        Some(Self {
            owner,
            state,
            payload,
        })
    }
    fn restore(self) {
        self.owner
            .with_mut(|owner| {
                let value = owner.as_any_mut();
                match self.payload {
                    Payload::Number(v) => {
                        value
                            .downcast_mut::<ViewModelInstanceNumber>()
                            .unwrap()
                            .base
                            .set_property_value_value(v);
                    }
                    Payload::Boolean(v) => {
                        value
                            .downcast_mut::<ViewModelInstanceBoolean>()
                            .unwrap()
                            .base
                            .set_property_value_value(v);
                    }
                    Payload::String(v) => {
                        value
                            .downcast_mut::<ViewModelInstanceString>()
                            .unwrap()
                            .base
                            .set_property_value_value(v);
                    }
                    Payload::Color(v) => {
                        value
                            .downcast_mut::<ViewModelInstanceColor>()
                            .unwrap()
                            .base
                            .set_property_value_value(v);
                    }
                    Payload::Enum(v) => {
                        value
                            .downcast_mut::<ViewModelInstanceEnum>()
                            .unwrap()
                            .base
                            .set_property_value_value(v);
                    }
                    Payload::Index(v) => {
                        value
                            .downcast_mut::<ViewModelInstanceSymbolListIndex>()
                            .unwrap()
                            .base
                            .set_property_value_value(v);
                    }
                    Payload::Trigger(v) => {
                        value
                            .downcast_mut::<ViewModelInstanceTrigger>()
                            .unwrap()
                            .base
                            .set_property_value_value(v);
                    }
                    Payload::Image(id, image) => {
                        let v = value.downcast_mut::<ViewModelInstanceAssetImage>().unwrap();
                        v.base.set_property_value_value(id);
                        v.asset().restore_host_image(image);
                    }
                    Payload::Font(id, font) => {
                        let v = value.downcast_mut::<ViewModelInstanceAssetFont>().unwrap();
                        v.base.set_property_value_value(id);
                        v.asset().restore_host_font(font);
                    }
                    Payload::Blob(id, asset) => {
                        let v = value.downcast_mut::<ViewModelInstanceAssetBlob>().unwrap();
                        v.base.set_property_value_value(id);
                        v.restore_host_asset(asset);
                    }
                    Payload::Artboard(id, asset, instance) => {
                        let v = value.downcast_mut::<ViewModelInstanceArtboard>().unwrap();
                        v.base.set_property_value_value(id);
                        v.restore_host_asset(asset, instance);
                    }
                    Payload::List(items) => {
                        let list = value.downcast_mut::<ViewModelInstanceList>().unwrap();
                        list.restore_host_items(Vec::new());
                        let items = items
                            .into_iter()
                            .map(|(item, instance)| {
                                item.with_mut(|item| {
                                    item.as_view_model_instance_list_item_mut()
                                        .unwrap()
                                        .set_view_model_instance(instance)
                                });
                                item
                            })
                            .collect();
                        list.restore_host_items(items);
                    }
                    Payload::ViewModel(instance) => {
                        value
                            .downcast_mut::<ViewModelInstanceViewModel>()
                            .unwrap()
                            .restore_host_reference(instance);
                    }
                }
                owner
                    .as_view_model_instance_value_mut()
                    .unwrap()
                    .restore_host_snapshot(self.state);
            })
            .expect("transaction retains native property owner");
    }
}
struct Transaction {
    snapshots: Vec<Snapshot>,
    notifications: Option<RuntimeHostMutationNotifications>,
    publication: Option<RuntimeHostTransactionPublication>,
    armed: bool,
    files: Vec<RuntimeFileHandle>,
}
impl Transaction {
    fn begin() -> Result<Self, RuntimeViewModelGraphTransactionError> {
        let notifications = RuntimeHostMutationNotifications::begin()
            .ok_or(RuntimeViewModelGraphTransactionError::Reentrant)?;
        let publication = RuntimeHostTransactionPublication::begin()
            .ok_or(RuntimeViewModelGraphTransactionError::Reentrant)?;
        Ok(Self {
            snapshots: Vec::new(),
            notifications: Some(notifications),
            publication: Some(publication),
            armed: true,
            files: Vec::new(),
        })
    }
    fn capture(&mut self, owner: &RuntimeOwnedViewModelHandle, path: &str) -> bool {
        let property = {
            let instance = owner.borrow();
            let Some(path) = instance.path_named(path) else {
                return false;
            };
            instance.property_by_path(&path)
        };
        let Some(snapshot) = property.and_then(Snapshot::capture) else {
            return false;
        };
        self.files.push(owner.native_file());
        self.snapshots.push(snapshot);
        true
    }
    fn commit(&mut self) {
        self.publication.take();
        self.notifications
            .take()
            .expect("transaction publication")
            .commit();
        self.snapshots.clear();
        self.armed = false;
    }
    fn rollback(&mut self) {
        if !self.armed {
            return;
        }
        while let Some(snapshot) = self.snapshots.pop() {
            snapshot.restore();
        }
        self.publication.take();
        if let Some(notifications) = self.notifications.take() {
            notifications.discard();
        }
        self.armed = false;
    }
}
impl Drop for Transaction {
    fn drop(&mut self) {
        self.rollback();
    }
}
pub struct RuntimeOwnedViewModelTransaction(Transaction);
pub struct RuntimeOwnedViewModelGraphTransaction(Transaction);
impl std::fmt::Debug for RuntimeOwnedViewModelTransaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RuntimeOwnedViewModelTransaction")
            .field("undo_entries", &self.0.snapshots.len())
            .finish()
    }
}
impl std::fmt::Debug for RuntimeOwnedViewModelGraphTransaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RuntimeOwnedViewModelGraphTransaction")
            .field("undo_entries", &self.0.snapshots.len())
            .finish()
    }
}
impl RuntimeOwnedViewModelTransaction {
    pub fn begin() -> Option<Self> {
        Transaction::begin().ok().map(Self)
    }
    pub fn commit(mut self) {
        self.0.commit();
    }
    pub fn set_number(
        &mut self,
        owner: &RuntimeOwnedViewModelHandle,
        path: &str,
        value: f32,
    ) -> bool {
        self.0.capture(owner, path)
            && owner
                .borrow_mut()
                .set_number_by_property_name_path(path, value)
    }
    pub fn set_boolean(
        &mut self,
        owner: &RuntimeOwnedViewModelHandle,
        path: &str,
        value: bool,
    ) -> bool {
        self.0.capture(owner, path)
            && owner
                .borrow_mut()
                .set_boolean_by_property_name_path(path, value)
    }
    pub fn set_string(
        &mut self,
        owner: &RuntimeOwnedViewModelHandle,
        path: &str,
        value: &[u8],
    ) -> bool {
        self.0.capture(owner, path)
            && owner
                .borrow_mut()
                .set_string_by_property_name_path(path, value)
    }
    pub fn set_color(
        &mut self,
        owner: &RuntimeOwnedViewModelHandle,
        path: &str,
        value: u32,
    ) -> bool {
        self.0.capture(owner, path)
            && owner
                .borrow_mut()
                .set_color_by_property_name_path(path, value)
    }
    pub fn set_enum(
        &mut self,
        owner: &RuntimeOwnedViewModelHandle,
        path: &str,
        value: u64,
    ) -> bool {
        self.0.capture(owner, path)
            && owner
                .borrow_mut()
                .set_enum_by_property_name_path(path, value)
    }
    pub fn fire_trigger(&mut self, owner: &RuntimeOwnedViewModelHandle, path: &str) -> bool {
        self.0.capture(owner, path) && owner.borrow_mut().fire_trigger_by_property_name_path(path)
    }
    pub fn set_list_index(
        &mut self,
        owner: &RuntimeOwnedViewModelHandle,
        path: &str,
        value: u64,
    ) -> bool {
        self.0.capture(owner, path)
            && owner
                .borrow_mut()
                .set_symbol_list_index_by_property_name_path(path, value)
    }
    pub fn set_asset(
        &mut self,
        owner: &RuntimeOwnedViewModelHandle,
        path: &str,
        value: u64,
    ) -> bool {
        self.0.capture(owner, path)
            && owner
                .borrow_mut()
                .set_asset_by_property_name_path(path, value)
    }
    pub fn link_view_model(
        &mut self,
        owner: &RuntimeOwnedViewModelHandle,
        path: &str,
        value: &RuntimeOwnedViewModelHandle,
    ) -> Result<bool, RuntimeViewModelLinkError> {
        if !self.0.capture(owner, path) {
            return Err(RuntimeViewModelLinkError::PropertyNotFound);
        }
        owner.link_view_model_by_property_name_path(path, value)
    }
    pub fn list_insert(
        &mut self,
        owner: &RuntimeOwnedViewModelHandle,
        path: &str,
        index: usize,
        item: &RuntimeOwnedViewModelHandle,
    ) -> bool {
        self.0.capture(owner, path)
            && owner.insert_list_item_by_property_name_path(path, index, item)
    }
    pub fn list_remove(
        &mut self,
        owner: &RuntimeOwnedViewModelHandle,
        path: &str,
        index: usize,
    ) -> bool {
        self.0.capture(owner, path) && owner.remove_list_item_by_property_name_path(path, index)
    }
    pub fn list_swap(
        &mut self,
        owner: &RuntimeOwnedViewModelHandle,
        path: &str,
        a: usize,
        b: usize,
    ) -> bool {
        if a == b {
            return true;
        }
        self.0.capture(owner, path) && owner.swap_list_items_by_property_name_path(path, a, b)
    }
    pub fn list_move(
        &mut self,
        owner: &RuntimeOwnedViewModelHandle,
        path: &str,
        a: usize,
        b: usize,
    ) -> bool {
        if a == b {
            return true;
        }
        self.0.capture(owner, path) && owner.move_list_item_by_property_name_path(path, a, b)
    }
    pub fn list_set(
        &mut self,
        owner: &RuntimeOwnedViewModelHandle,
        path: &str,
        index: usize,
        item: &RuntimeOwnedViewModelHandle,
    ) -> bool {
        self.0.capture(owner, path) && owner.set_list_item_by_property_name_path(path, index, item)
    }
    pub fn list_clear(&mut self, owner: &RuntimeOwnedViewModelHandle, path: &str) -> bool {
        self.0.capture(owner, path) && {
            owner.clear_list_items_by_property_name_path(path);
            true
        }
    }
}
impl RuntimeOwnedViewModelGraphTransaction {
    pub fn begin(
        roots: &[RuntimeOwnedViewModelHandle],
        maximum_entries: usize,
    ) -> Result<Self, RuntimeViewModelGraphTransactionError> {
        let mut transaction = Transaction::begin()?;
        let mut visited = BTreeSet::new();
        for root in roots {
            let owners = root
                .reachable_change_owner_snapshot()
                .ok_or(RuntimeViewModelGraphTransactionError::BorrowConflict)?;
            transaction.files.push(root.native_file());
            for owner in owners {
                let properties = owner
                    .native_handle()
                    .with_downcast::<ViewModelInstance, _>(|owner| owner.property_values().to_vec())
                    .ok_or(RuntimeViewModelGraphTransactionError::BorrowConflict)?;
                for property in properties {
                    if !visited.insert(property.identity_key()) {
                        continue;
                    }
                    if transaction.snapshots.len() >= maximum_entries {
                        return Err(RuntimeViewModelGraphTransactionError::LimitExceeded);
                    }
                    transaction.snapshots.push(
                        Snapshot::capture(property)
                            .ok_or(RuntimeViewModelGraphTransactionError::BorrowConflict)?,
                    );
                }
            }
        }
        Ok(Self(transaction))
    }
    pub fn commit(mut self) {
        self.0.commit();
    }
}
pub(crate) fn capture_native_change(owner: CoreHandle, value: RuntimeViewModelChangeValue) {
    if !crate::view_model_cell::is_capturing_view_model_changes() {
        return;
    }
    crate::view_model_cell::capture_view_model_change(instance::identity(&owner) as usize, value);
}
pub(crate) fn capture_native_list_change(owner: CoreHandle, items: &[CoreHandle]) {
    if !crate::view_model_cell::is_capturing_view_model_changes() {
        return;
    }
    let values = items
        .iter()
        .filter_map(|item| {
            item.with(|item| {
                item.as_view_model_instance_list_item()
                    .unwrap()
                    .view_model_instance()
            })
            .flatten()
        })
        .map(|instance| instance::identity(&instance))
        .collect();
    capture_native_change(owner, RuntimeViewModelChangeValue::List(values));
}
pub(crate) fn capture_native_view_model_change(owner: CoreHandle, value: Option<&CoreHandle>) {
    if !crate::view_model_cell::is_capturing_view_model_changes() {
        return;
    }
    capture_native_change(
        owner,
        RuntimeViewModelChangeValue::ViewModel(value.map(instance::identity)),
    );
}
impl RuntimeOwnedViewModelHandle {
    pub fn resolve_change_capture(
        &self,
        capture: RuntimeViewModelChangeCapture,
    ) -> Option<Vec<RuntimeViewModelChange>> {
        Self::resolve_change_capture_across(std::slice::from_ref(self), capture)
    }
    pub fn resolve_change_capture_across(
        roots: &[Self],
        capture: RuntimeViewModelChangeCapture,
    ) -> Option<Vec<RuntimeViewModelChange>> {
        Some(
            Self::resolve_change_capture_across_with_owners(roots, capture)?
                .into_iter()
                .map(|(_, change)| change)
                .collect(),
        )
    }
    pub fn resolve_change_capture_with_owners(
        &self,
        capture: RuntimeViewModelChangeCapture,
    ) -> Option<Vec<(Self, RuntimeViewModelChange)>> {
        Self::resolve_change_capture_across_with_owners(std::slice::from_ref(self), capture)
    }
    pub fn resolve_change_capture_across_with_owners(
        roots: &[Self],
        capture: RuntimeViewModelChangeCapture,
    ) -> Option<Vec<(Self, RuntimeViewModelChange)>> {
        let captured = capture.finish().ok()?;
        let mut owners = BTreeMap::new();
        for root in roots {
            for owner in root.reachable_change_owner_snapshot()? {
                let properties = owner
                    .native_handle()
                    .with_downcast::<ViewModelInstance, _>(|owner| {
                        owner.property_values().to_vec()
                    })?;
                for property in properties {
                    let index = property.with(|property| {
                        property
                            .as_view_model_instance_value()
                            .unwrap()
                            .base
                            .view_model_property_id()
                    })? as usize;
                    owners.insert(
                        instance::identity(&property) as usize,
                        (owner.clone(), index),
                    );
                }
            }
        }
        captured
            .into_iter()
            .map(|change| {
                let (owner, index) = owners.get(&change.cell_identity)?;
                Some((
                    owner.clone(),
                    RuntimeViewModelChange {
                        owner_instance_identity: owner.instance_identity(),
                        property_index: *index,
                        value: change.value,
                    },
                ))
            })
            .collect()
    }
}
