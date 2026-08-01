// Direct Rust owner for pinned C++ `src/viewmodel/viewmodel_instance_list.cpp`.
// Ordered list occurrences, mutation order, parent registration, and teardown.

/// Pre-resolved typed relation between one string and one boolean on every
/// item of an owned ViewModel list.
///
/// The list path and item schema are validated once while resolving this
/// handle. Applying it later performs no property-name lookup and fails closed
/// if the list is replaced with items from a different or malformed schema.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeOwnedViewModelListStringMatchBooleanHandle {
    list_property_path: Vec<usize>,
    item_view_model_index: usize,
    string_property_index: usize,
    boolean_property_index: usize,
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeOwnedViewModelListHandle {
    pub(super) value: Rc<RefCell<RuntimeOwnedViewModelListValue>>,
    pub(super) cell: RuntimeViewModelCell,
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeOwnedViewModelListItemEntry {
    pub(crate) occurrence_identity: u64,
    pub(crate) instance: RuntimeOwnedViewModelHandle,
}

impl RuntimeOwnedViewModelListHandle {
    fn notify_value_changed(&self) {
        // C++ `ViewModelInstanceList::propertyValueChanged()` dirties the
        // retained property for every successful structural mutation. The
        // list itself, not this DependencyHelper-shaped cell, owns the items.
        self.cell.notify_bindings_value_changed();
    }

    pub(crate) fn items(&self) -> Vec<RuntimeOwnedViewModelHandle> {
        self.value
            .borrow()
            .items
            .iter()
            .map(|item| RuntimeOwnedViewModelHandle::from_shared(Rc::clone(&item.instance)))
            .collect()
    }

    pub(crate) fn item_entries(&self) -> Vec<RuntimeOwnedViewModelListItemEntry> {
        self.value
            .borrow()
            .items
            .iter()
            .map(|item| RuntimeOwnedViewModelListItemEntry {
                occurrence_identity: item.occurrence_identity,
                instance: RuntimeOwnedViewModelHandle::from_shared(Rc::clone(&item.instance)),
            })
            .collect()
    }

    /// Mirrors `ArtboardComponentList::updateList`: immediately before rows
    /// are mounted, C++ writes each wrapper's current logical position into
    /// the synthetic `itemIndex` symbol on its view-model instance. Mutate the
    /// shared list instances before taking the row snapshots so bindings and
    /// public handles observe the same value.
    pub(crate) fn item_entries_with_logical_indices(
        &self,
        file: &RuntimeFile,
    ) -> Vec<RuntimeOwnedViewModelListItemEntry> {
        self.value
            .borrow()
            .items
            .iter()
            .enumerate()
            .map(|(index, item)| {
                let instance = RuntimeOwnedViewModelHandle::from_shared(Rc::clone(&item.instance));
                set_component_list_item_index(file, &mut instance.borrow_mut(), index);
                RuntimeOwnedViewModelListItemEntry {
                    occurrence_identity: item.occurrence_identity,
                    instance,
                }
            })
            .collect()
    }

    pub(crate) fn text_runs(&self) -> Vec<(Vec<u8>, Vec<u8>)> {
        self.value
            .borrow()
            .items
            .iter()
            .filter_map(|item| {
                let item = item.instance.borrow();
                Some((
                    item.string_value_by_property_name("textContent")?.to_vec(),
                    item.string_value_by_property_name("textStyle")
                        .unwrap_or_default()
                        .to_vec(),
                ))
            })
            .collect()
    }
}

#[derive(Debug, Clone)]
struct RuntimeOwnedViewModelList {
    property_index: usize,
    value: Rc<RefCell<RuntimeOwnedViewModelListValue>>,
    cell: RuntimeViewModelCell,
}

#[derive(Debug)]
pub(super) struct RuntimeOwnedViewModelListValue {
    pub(super) parent_relay: Weak<RuntimeOwnedViewModelParentRelay>,
    pub(super) item_count: usize,
    pub(super) items: Vec<RuntimeOwnedViewModelListItem>,
}

impl Default for RuntimeOwnedViewModelListValue {
    fn default() -> Self {
        Self {
            parent_relay: Weak::new(),
            item_count: 0,
            items: Vec::new(),
        }
    }
}

impl RuntimeOwnedViewModelListValue {
    fn bind_parent_relay(&mut self, parent: &Rc<RuntimeOwnedViewModelParentRelay>) {
        self.parent_relay = Rc::downgrade(parent);
    }

    fn attach_item(&self, item: &mut RuntimeOwnedViewModelListItem) {
        if let Some(parent) = self.parent_relay.upgrade() {
            item.attach_parent(&parent);
        }
    }

    fn detach_item(&self, item: &mut RuntimeOwnedViewModelListItem) {
        if let Some(parent) = self.parent_relay.upgrade() {
            item.detach_parent(&parent);
        } else {
            item.parent_registered = false;
        }
    }

    fn set_item_count(&mut self, item_count: usize) -> bool {
        if self.item_count == item_count {
            return false;
        }
        if item_count < self.items.len() {
            let mut removed = self.items.split_off(item_count);
            for item in &mut removed {
                self.detach_item(item);
            }
        }
        self.item_count = item_count;
        true
    }

    fn push_instance(&mut self, instance: Rc<RefCell<RuntimeOwnedViewModelInstance>>) {
        let mut item = RuntimeOwnedViewModelListItem::new(instance);
        self.attach_item(&mut item);
        self.items.push(item);
        self.item_count = self.items.len();
    }

    fn insert_instance(
        &mut self,
        index: usize,
        instance: Rc<RefCell<RuntimeOwnedViewModelInstance>>,
    ) -> bool {
        if index > self.items.len() {
            return false;
        }
        let mut item = RuntimeOwnedViewModelListItem::new(instance);
        self.attach_item(&mut item);
        self.items.insert(index, item);
        self.item_count = self.items.len();
        true
    }

    fn insert_runtime_instance(
        &mut self,
        index: usize,
        instance: Rc<RefCell<RuntimeOwnedViewModelInstance>>,
    ) -> bool {
        if index > self.items.len() {
            return false;
        }
        // The pinned runtime facade calls `addItemAt` while its list item is
        // still empty, then assigns the instance. Consequently this path does
        // not register the inserted instance as a structural parent child.
        // Keep that observable lifetime/propagation asymmetry separate from
        // the authored-list insertion API above.
        self.items.insert(
            index,
            RuntimeOwnedViewModelListItem::new(instance),
        );
        self.item_count = self.items.len();
        true
    }

    fn replace_instance(
        &mut self,
        index: usize,
        instance: Rc<RefCell<RuntimeOwnedViewModelInstance>>,
    ) -> bool {
        let Some(current) = self.items.get(index) else {
            return false;
        };
        if Rc::ptr_eq(&current.instance, &instance) {
            return false;
        }
        let mut replacement = RuntimeOwnedViewModelListItem::new(instance);
        self.attach_item(&mut replacement);
        let mut previous = std::mem::replace(&mut self.items[index], replacement);
        self.detach_item(&mut previous);
        true
    }

    fn pop_instance(&mut self) -> Option<Rc<RefCell<RuntimeOwnedViewModelInstance>>> {
        let mut item = self.items.pop()?;
        // Pinned C++ `ViewModelInstanceList::pop()` omits `removeParent`.
        item.disarm_parent_registration();
        self.item_count = self.items.len();
        Some(item.instance)
    }

    fn remove_instance_at(
        &mut self,
        index: usize,
    ) -> Option<Rc<RefCell<RuntimeOwnedViewModelInstance>>> {
        if index >= self.items.len() {
            return None;
        }
        let mut item = self.items.remove(index);
        self.detach_item(&mut item);
        self.item_count = self.items.len();
        Some(item.instance)
    }

    fn clear_instances(&mut self) -> bool {
        if self.items.is_empty() && self.item_count == 0 {
            return false;
        }
        let mut items = std::mem::take(&mut self.items);
        for item in &mut items {
            self.detach_item(item);
        }
        self.item_count = 0;
        true
    }

    fn remove_instances_by_identity(
        &mut self,
        instance: &Rc<RefCell<RuntimeOwnedViewModelInstance>>,
        remove_all: bool,
    ) -> bool {
        let mut changed = false;
        let mut index = 0;
        while index < self.items.len() {
            if Rc::ptr_eq(&self.items[index].instance, instance) {
                let mut item = self.items.remove(index);
                self.detach_item(&mut item);
                changed = true;
                if !remove_all {
                    break;
                }
            } else {
                index += 1;
            }
        }
        if changed {
            self.item_count = self.items.len();
        }
        changed
    }

    fn replace_instances(&mut self, instances: Vec<RuntimeOwnedViewModelInstance>) -> bool {
        let changed = !instances.is_empty() || !self.items.is_empty() || self.item_count != 0;
        let mut previous = std::mem::take(&mut self.items);
        for item in &mut previous {
            self.detach_item(item);
        }
        for instance in instances {
            self.push_instance(Rc::new(RefCell::new(instance)));
        }
        self.item_count = self.items.len();
        changed
    }
}

impl Drop for RuntimeOwnedViewModelListValue {
    fn drop(&mut self) {
        let Some(parent) = self.parent_relay.upgrade() else {
            return;
        };
        for item in &mut self.items {
            item.detach_parent(&parent);
        }
    }
}

fn reset_runtime_owned_triggers(triggers: &mut [RuntimeOwnedViewModelTrigger]) -> bool {
    let mut changed = false;
    for trigger in triggers {
        if trigger.set_value(0) {
            changed = true;
        }
    }
    changed
}

fn collect_runtime_owned_list_children(
    lists: &[RuntimeOwnedViewModelList],
    children: &mut Vec<Rc<RefCell<RuntimeOwnedViewModelInstance>>>,
) {
    for list in lists {
        children.extend(
            list.value
                .borrow()
                .items
                .iter()
                .map(|item| Rc::clone(&item.instance)),
        );
    }
}

fn advance_runtime_owned_list_children(lists: &[RuntimeOwnedViewModelList]) {
    for list in lists {
        let value = list.value.borrow();
        for item in &value.items {
            item.instance.borrow_mut().advanced_data_context();
        }
    }
}

fn bind_owned_view_model_list_parent_relays(
    lists: &mut [RuntimeOwnedViewModelList],
    parent: &Rc<RuntimeOwnedViewModelParentRelay>,
) {
    for list in lists {
        list.value.borrow_mut().bind_parent_relay(parent);
    }
}

fn bind_owned_view_model_child_parent_relays(
    children: &mut [RuntimeOwnedViewModelViewModel],
    parent: &Rc<RuntimeOwnedViewModelParentRelay>,
) {
    for child in children {
        if let Some(linked) = child.endpoint.linked_instance() {
            let linked_relay = Rc::clone(&linked.borrow().parent_relay);
            RuntimeOwnedViewModelParentRelay::add_parent(&linked_relay, parent);
            // Alias mirrors share the linked instance's list storage. Its
            // owner relay stays with that linked identity; the edge above is
            // the only outer relationship.
            continue;
        }
        bind_owned_view_model_list_parent_relays(&mut child.lists, parent);
        for lists in child.imported_lists.values_mut() {
            bind_owned_view_model_list_parent_relays(lists, parent);
        }
        bind_owned_view_model_child_parent_relays(&mut child.children, parent);
        for children in child.imported_children.values_mut() {
            bind_owned_view_model_child_parent_relays(children, parent);
        }
    }
}

fn runtime_owned_view_model_lists(
    file: &RuntimeFile,
    view_model_index: usize,
) -> Vec<RuntimeOwnedViewModelList> {
    file.view_model(view_model_index)
        .map(|view_model| {
            view_model
                .properties
                .into_iter()
                .enumerate()
                .filter_map(|(property_index, property)| {
                    (property.type_name == "ViewModelPropertyList").then_some(
                        RuntimeOwnedViewModelList {
                            property_index,
                            value: Rc::new(RefCell::new(RuntimeOwnedViewModelListValue::default())),
                            cell: RuntimeViewModelCell::new(RuntimeViewModelCellValue::List),
                        },
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

fn runtime_owned_view_model_lists_for_instance(
    file: &RuntimeFile,
    view_model_index: usize,
    view_model_instance: &RuntimeObject,
) -> Vec<RuntimeOwnedViewModelList> {
    runtime_owned_view_model_instance_value_objects(file, view_model_index, view_model_instance)
        .into_iter()
        .filter_map(|source| {
            let property_index =
                usize::try_from(source.uint_property("viewModelPropertyId")?).ok()?;
            let (item_count, items) =
                match file.view_model_instance_source_data_value_for_object(source)? {
                    RuntimeDataValue::List(items) => {
                        let item_count = items.len();
                        let hydrated = items
                            .into_iter()
                            .filter_map(|item| {
                                let reference = file
                                    .referenced_view_model_instance_for_list_item_object(item)?;
                                runtime_owned_view_model_list_item_instance(file, reference).map(
                                    |(instance, source_object_id)| {
                                        RuntimeOwnedViewModelListItem::from_authored(
                                            Rc::new(RefCell::new(instance)),
                                            source_object_id,
                                        )
                                    },
                                )
                            })
                            .collect::<Vec<_>>();
                        (item_count, hydrated)
                    }
                    _ => return None,
                };
            Some(RuntimeOwnedViewModelList {
                property_index,
                value: Rc::new(RefCell::new(RuntimeOwnedViewModelListValue {
                    parent_relay: Weak::new(),
                    item_count,
                    items,
                })),
                cell: RuntimeViewModelCell::new(RuntimeViewModelCellValue::List),
            })
        })
        .collect()
}

fn runtime_owned_view_model_list_item_instance(
    file: &RuntimeFile,
    reference: RuntimeViewModelInstanceReference<'_>,
) -> Option<(RuntimeOwnedViewModelInstance, u32)> {
    thread_local! {
        static HYDRATING: RefCell<BTreeSet<(usize, usize)>> = RefCell::new(BTreeSet::new());
    }
    let key = (reference.view_model_index, reference.instance_index);
    if !HYDRATING.with(|hydrating| hydrating.borrow_mut().insert(key)) {
        return None;
    }
    let instance = RuntimeOwnedViewModelInstance::from_imported_instance(
        file,
        reference.view_model_index,
        reference.instance_index,
    )
    .map(|instance| (instance, reference.object.id));
    HYDRATING.with(|hydrating| {
        hydrating.borrow_mut().remove(&key);
    });
    instance
}

fn runtime_owned_view_model_imported_lists(
    file: &RuntimeFile,
    view_model_index: usize,
) -> BTreeMap<u32, Vec<RuntimeOwnedViewModelList>> {
    file.view_model(view_model_index)
        .map(|view_model| {
            view_model
                .instances
                .into_iter()
                .map(|instance| {
                    (
                        instance.object.id,
                        runtime_owned_view_model_lists_for_instance(
                            file,
                            view_model_index,
                            instance.object,
                        ),
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

fn runtime_owned_list_matches_string_boolean_handle(
    list: &RuntimeOwnedViewModelListValue,
    handle: &RuntimeOwnedViewModelListStringMatchBooleanHandle,
) -> bool {
    !list.items.is_empty()
        && list.items.iter().all(|item| {
            let item = item.instance.borrow();
            item.view_model_index == handle.item_view_model_index
                && item
                    .strings
                    .iter()
                    .any(|value| value.property_index == handle.string_property_index)
                && item
                    .booleans
                    .iter()
                    .any(|value| value.property_index == handle.boolean_property_index)
        })
}
