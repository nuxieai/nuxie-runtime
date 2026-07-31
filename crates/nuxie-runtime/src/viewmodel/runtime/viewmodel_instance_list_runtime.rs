// Direct Rust owner for pinned C++
// `src/viewmodel/runtime/viewmodel_instance_list_runtime.cpp`.
// List mutation delegates to the retained authored list; wrapper identity is
// cached per list-item occurrence, not merely per child instance.

#[derive(Debug, Clone)]
pub struct ViewModelInstanceListRuntime {
    value: ViewModelInstanceValueRuntime,
    file: Rc<RuntimeFile>,
    owner: RuntimeOwnedViewModelHandle,
    property_path: Vec<usize>,
    items: Rc<RefCell<BTreeMap<u64, ViewModelInstanceRuntime>>>,
}

impl ViewModelInstanceListRuntime {
    fn new(
        name: impl Into<String>,
        cell: RuntimeViewModelCell,
        file: Rc<RuntimeFile>,
        owner: RuntimeOwnedViewModelHandle,
        property_path: Vec<usize>,
    ) -> Self {
        Self {
            value: ViewModelInstanceValueRuntime::new(
                name,
                ViewModelRuntimeDataType::List,
                cell,
            ),
            file,
            owner,
            property_path,
            items: Rc::new(RefCell::new(BTreeMap::new())),
        }
    }

    fn entries(&self) -> Option<Vec<RuntimeOwnedViewModelListItemEntry>> {
        self.owner
            .borrow()
            .list_handle_by_property_path(&self.property_path)
            .map(|list| list.item_entries())
    }

    pub fn instance_at(&self, index: isize) -> Option<ViewModelInstanceRuntime> {
        let index = usize::try_from(index).ok()?;
        let entry = self.entries()?.into_iter().nth(index)?;
        if let Some(runtime) = self.items.borrow().get(&entry.occurrence_identity) {
            return Some(runtime.clone());
        }
        let runtime =
            ViewModelInstanceRuntime::from_handle(Rc::clone(&self.file), entry.instance);
        self.items
            .borrow_mut()
            .insert(entry.occurrence_identity, runtime.clone());
        Some(runtime)
    }

    pub fn add_instance(&self, instance: &ViewModelInstanceRuntime) -> bool {
        if !self
            .owner
            .push_list_item_by_property_path(&self.property_path, instance.handle().shared())
        {
            return false;
        }
        let Some(entry) = self.entries().and_then(|entries| entries.into_iter().last()) else {
            return false;
        };
        self.items
            .borrow_mut()
            .insert(entry.occurrence_identity, instance.clone());
        true
    }

    pub fn add_instance_at(
        &self,
        instance: &ViewModelInstanceRuntime,
        index: isize,
    ) -> bool {
        let Ok(index) = usize::try_from(index) else {
            return false;
        };
        if index > self.size() {
            return false;
        }
        if !self.owner.insert_runtime_list_item_by_property_path(
            &self.property_path,
            index,
            instance.handle().shared(),
        ) {
            return false;
        }
        let Some(entry) = self.entries().and_then(|entries| entries.into_iter().nth(index)) else {
            return false;
        };
        self.items
            .borrow_mut()
            .insert(entry.occurrence_identity, instance.clone());
        true
    }

    pub fn remove_instance(&self, instance: &ViewModelInstanceRuntime) -> bool {
        let occurrence_ids = self
            .entries()
            .unwrap_or_default()
            .into_iter()
            .filter_map(|entry| {
                entry
                    .instance
                    .ptr_eq(instance.handle())
                    .then_some(entry.occurrence_identity)
            })
            .collect::<Vec<_>>();
        if occurrence_ids.is_empty()
            || !self
                .owner
                .borrow_mut()
                .remove_list_items_by_identity_at_property_path(
                    &self.property_path,
                    &instance.handle().shared(),
                    true,
                )
        {
            return false;
        }
        let mut items = self.items.borrow_mut();
        for occurrence_id in occurrence_ids {
            items.remove(&occurrence_id);
        }
        true
    }

    pub fn remove_instance_at(&self, index: isize) -> bool {
        let Ok(index) = usize::try_from(index) else {
            return false;
        };
        let Some(occurrence_identity) = self
            .entries()
            .and_then(|entries| entries.into_iter().nth(index))
            .map(|entry| entry.occurrence_identity)
        else {
            return false;
        };
        if !self
            .owner
            .borrow_mut()
            .remove_list_item_at_by_property_path(&self.property_path, index)
        {
            return false;
        }
        self.items.borrow_mut().remove(&occurrence_identity);
        true
    }

    pub fn swap(&self, first: usize, second: usize) -> bool {
        self.owner
            .borrow_mut()
            .swap_list_items_by_property_path(&self.property_path, first, second)
    }

    pub fn remove_all_instances(&self) -> bool {
        let changed = self
            .owner
            .borrow_mut()
            .clear_list_items_by_property_path(&self.property_path);
        self.items.borrow_mut().clear();
        changed
    }

    pub fn size(&self) -> usize {
        self.owner
            .borrow()
            .list_item_count_by_property_path(&self.property_path)
            .unwrap_or(0)
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        self.value.ptr_eq(&other.value)
    }

    pub fn value_runtime(&self) -> &ViewModelInstanceValueRuntime {
        &self.value
    }
}
