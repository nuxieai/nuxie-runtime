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

/// Current values written into one dynamic `TextValueRun` by the retained
/// list listener. `None` is observably different from an empty string: a
/// missing property creates no C++ property listener, while an empty property
/// creates a listener and performs its initial write.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RuntimeTextListRun {
    pub(crate) text: Option<Vec<u8>>,
    pub(crate) style: Option<Vec<u8>>,
}

impl RuntimeOwnedViewModelListHandle {
    pub(crate) fn ptr_eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.value, &other.value)
    }

    fn notify_value_changed(&self) {
        // C++ `ViewModelInstanceList::propertyValueChanged()` dirties the
        // retained property for every successful structural mutation. The
        // list itself, not this DependencyHelper-shaped cell, owns the items.
        let identities = self
            .value
            .borrow()
            .items
            .iter()
            .filter_map(|item| {
                item.instance
                    .as_ref()
                    .map(|instance| instance.borrow().instance_identity())
            })
            .collect();
        self.cell
            .notify_structural_value_changed(RuntimeViewModelChangeValue::List(identities));
    }

    pub(crate) fn items(&self) -> Vec<RuntimeOwnedViewModelHandle> {
        self.value
            .borrow()
            .items
            .iter()
            .filter_map(|item| {
                item.instance
                    .as_ref()
                    .map(|instance| RuntimeOwnedViewModelHandle::from_shared(Rc::clone(instance)))
            })
            .collect()
    }

    pub(crate) fn item_count(&self) -> usize {
        self.value.borrow().items.len()
    }

    pub(crate) fn item_at(&self, index: usize) -> Option<RuntimeOwnedViewModelHandle> {
        let value = self.value.borrow();
        let instance = value.items.get(index)?.instance.as_ref()?;
        Some(RuntimeOwnedViewModelHandle::from_shared(Rc::clone(
            instance,
        )))
    }

    pub(crate) fn item_entry_at(&self, index: usize) -> Option<RuntimeOwnedViewModelListItemEntry> {
        let value = self.value.borrow();
        let item = value.items.get(index)?;
        Some(RuntimeOwnedViewModelListItemEntry {
            occurrence_identity: item.occurrence_identity,
            instance: RuntimeOwnedViewModelHandle::from_shared(Rc::clone(item.instance.as_ref()?)),
        })
    }

    pub(crate) fn occurrence_identity_at(&self, index: usize) -> Option<u64> {
        self.value
            .borrow()
            .items
            .get(index)
            .map(|item| item.occurrence_identity)
    }

    fn transaction_snapshot_items(&self) -> Vec<RuntimeOwnedViewModelListItem> {
        self.value
            .borrow()
            .items
            .iter()
            .map(|item| {
                let mut snapshot = RuntimeOwnedViewModelListItem::copy_identity_from(
                    item,
                    item.instance.as_ref().map(Rc::clone),
                );
                snapshot.parent_registered = item.parent_registered;
                snapshot
            })
            .collect()
    }

    fn transaction_restore_items(&self, mut items: Vec<RuntimeOwnedViewModelListItem>) {
        let mut value = self.value.borrow_mut();
        let mut current = std::mem::take(&mut value.items);
        for item in &mut current {
            value.detach_item(item);
        }
        for item in &mut items {
            if item.parent_registered {
                // `attach_item` also records the local registration bit. The
                // snapshot already has that exact bit, so add only the relay
                // edge that was removed with the staged topology.
                if let Some(parent) = value.parent_relay.upgrade() {
                    if let Some(child_relay) = item.child_relay.as_ref() {
                        RuntimeOwnedViewModelParentRelay::add_parent(child_relay, &parent);
                    }
                }
            }
        }
        value.item_count = items.len();
        value.items = items;
    }

    pub(crate) fn item_entries(&self) -> Vec<RuntimeOwnedViewModelListItemEntry> {
        self.value
            .borrow()
            .items
            .iter()
            .filter_map(|item| {
                Some(RuntimeOwnedViewModelListItemEntry {
                    occurrence_identity: item.occurrence_identity,
                    instance: RuntimeOwnedViewModelHandle::from_shared(Rc::clone(
                        item.instance.as_ref()?,
                    )),
                })
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
            .filter_map(|(index, item)| {
                let instance =
                    RuntimeOwnedViewModelHandle::from_shared(Rc::clone(item.instance.as_ref()?));
                set_component_list_item_index(file, &mut instance.borrow_mut(), index);
                Some(RuntimeOwnedViewModelListItemEntry {
                    occurrence_identity: item.occurrence_identity,
                    instance,
                })
            })
            .collect()
    }

    pub(crate) fn text_runs(&self) -> Vec<RuntimeTextListRun> {
        self.value
            .borrow()
            .items
            .iter()
            .filter_map(|item| {
                let item = item.instance.as_ref()?.borrow();
                // `TextValueRunListener::createProperties` creates and
                // initially writes style before content. Read in the same
                // order while retaining absent properties as absent.
                let style = item
                    .string_value_by_property_name("textStyle")
                    .map(|value| value.to_vec());
                let text = item
                    .string_value_by_property_name("textContent")
                    .map(|value| value.to_vec());
                Some(RuntimeTextListRun { text, style })
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
        } else {
            self.items
                .resize_with(item_count, RuntimeOwnedViewModelListItem::empty);
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
        self.items
            .insert(index, RuntimeOwnedViewModelListItem::new(instance));
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
        if current
            .instance
            .as_ref()
            .is_some_and(|current| Rc::ptr_eq(current, &instance))
        {
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
        item.instance
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
        item.instance
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
            if self.items[index]
                .instance
                .as_ref()
                .is_some_and(|item| Rc::ptr_eq(item, instance))
            {
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
        changed |= trigger.value() != 0;
        // C++ calls ViewModelInstanceTrigger::advanced(), which resets the
        // counter through propertyValue(0) under SuppressDelegation. Calling
        // the ordinary setter here replayed script delegates for the internal
        // acknowledgment edge.
        trigger.advanced();
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
                .filter_map(|item| item.instance.as_ref().map(Rc::clone)),
        );
    }
}

fn advance_runtime_owned_list_children(lists: &[RuntimeOwnedViewModelList]) {
    for list in lists {
        let value = list.value.borrow();
        for item in &value.items {
            if let Some(instance) = item.instance.as_ref() {
                instance.borrow_mut().advanced_data_context();
            }
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
                            .map(|item| {
                                let view_model_id = item
                                    .uint_property("viewModelId")
                                    .and_then(|value| u32::try_from(value).ok())
                                    .unwrap_or(u32::MAX);
                                let view_model_instance_id = item
                                    .uint_property("viewModelInstanceId")
                                    .and_then(|value| u32::try_from(value).ok())
                                    .unwrap_or(u32::MAX);
                                let resolved = file
                                    .referenced_view_model_instance_for_list_item_object(item)
                                    .and_then(|reference| {
                                        runtime_owned_view_model_list_item_instance(file, reference)
                                    });
                                let source_object_id = resolved
                                    .as_ref()
                                    .map(|(_, source_object_id)| *source_object_id);
                                let instance =
                                    resolved.map(|(instance, _)| Rc::new(RefCell::new(instance)));
                                RuntimeOwnedViewModelListItem::from_authored(
                                    view_model_id,
                                    view_model_instance_id,
                                    instance,
                                    source_object_id,
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
            let Some(instance) = item.instance.as_ref() else {
                return false;
            };
            let item = instance.borrow();
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

#[cfg(test)]
mod dynamic_text_run_projection_tests {
    use super::*;
    use nuxie_binary::{FixtureProperty, FixtureRecord, FixtureValue};
    use nuxie_schema::definition_by_name;

    fn record(type_name: &str, properties: Vec<FixtureProperty>) -> FixtureRecord {
        FixtureRecord {
            type_key: definition_by_name(type_name)
                .unwrap_or_else(|| panic!("missing schema definition {type_name}"))
                .type_key
                .int,
            properties,
        }
    }

    fn property(type_name: &str, name: &str, value: FixtureValue) -> FixtureProperty {
        FixtureProperty {
            key: crate::properties::property_key_for_name(type_name, name)
                .unwrap_or_else(|| panic!("missing property {type_name}.{name}")),
            value,
        }
    }

    fn view_model(name: &str) -> FixtureRecord {
        record(
            "ViewModel",
            vec![property(
                "ViewModel",
                "name",
                FixtureValue::String(name.to_owned()),
            )],
        )
    }

    fn string_property(name: &str) -> FixtureRecord {
        record(
            "ViewModelPropertyString",
            vec![property(
                "ViewModelPropertyString",
                "name",
                FixtureValue::String(name.to_owned()),
            )],
        )
    }

    #[test]
    fn cxx_text_run_projection_preserves_missing_properties_and_item_order() {
        let file = RuntimeFile::from_fixture_records(vec![
            record("Backboard", Vec::new()),
            view_model("StyleOnly"),
            string_property("textStyle"),
            view_model("ContentOnly"),
            string_property("textContent"),
            view_model("Complete"),
            string_property("textStyle"),
            string_property("textContent"),
        ])
        .expect("dynamic TextValueRun view-model fixture");

        let mut style_only = RuntimeOwnedViewModelInstance::new(&file, 0).expect("style-only");
        assert!(style_only.set_string_by_property_name("textStyle", b"title"));
        let mut content_only = RuntimeOwnedViewModelInstance::new(&file, 1).expect("content-only");
        assert!(content_only.set_string_by_property_name("textContent", b"visible"));
        let mut complete = RuntimeOwnedViewModelInstance::new(&file, 2).expect("complete");
        assert!(complete.set_string_by_property_name("textStyle", b"body"));
        assert!(complete.set_string_by_property_name("textContent", b"complete"));

        let items = vec![style_only, content_only, complete]
            .into_iter()
            .map(|instance| RuntimeOwnedViewModelListItem::new(Rc::new(RefCell::new(instance))))
            .collect::<Vec<_>>();
        let handle = RuntimeOwnedViewModelListHandle {
            value: Rc::new(RefCell::new(RuntimeOwnedViewModelListValue {
                parent_relay: Weak::new(),
                item_count: items.len(),
                items,
            })),
            cell: RuntimeViewModelCell::new(RuntimeViewModelCellValue::List),
        };

        assert_eq!(
            handle.text_runs(),
            vec![
                RuntimeTextListRun {
                    text: None,
                    style: Some(b"title".to_vec()),
                },
                RuntimeTextListRun {
                    text: Some(b"visible".to_vec()),
                    style: None,
                },
                RuntimeTextListRun {
                    text: Some(b"complete".to_vec()),
                    style: Some(b"body".to_vec()),
                },
            ]
        );
    }
}
