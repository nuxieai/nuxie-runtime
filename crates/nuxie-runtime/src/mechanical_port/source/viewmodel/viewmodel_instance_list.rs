use crate::mechanical_port::source::{
    component_dirt::ComponentDirt, core::CoreHandle,
    generated::viewmodel::viewmodel_instance_list_base::ViewModelInstanceListBase,
};

#[derive(Default)]
pub struct ViewModelInstanceList {
    pub base: ViewModelInstanceListBase,
    list_items: Vec<CoreHandle>,
    parent_view_model_instance: Option<CoreHandle>,
    #[cfg(feature = "tools")]
    changed_callback: Option<fn(&mut Self)>,
}

impl ViewModelInstanceList {
    pub(crate) fn set_host_item_instance(&mut self, index: usize, instance: CoreHandle) -> bool {
        let Some(item) = self.list_items.get(index).cloned() else {
            return false;
        };
        self.remove_parent_from_item(&item);
        item.with_mut(|item| {
            item.as_view_model_instance_list_item_mut()
                .unwrap()
                .set_view_model_instance(Some(instance))
        });
        self.add_parent_to_item(&item);
        self.property_value_changed();
        true
    }
    pub(crate) fn restore_host_items(&mut self, items: Vec<CoreHandle>) {
        for item in &self.list_items {
            self.remove_parent_from_item(item);
        }
        for item in &items {
            self.add_parent_to_item(item);
        }
        self.list_items = items;
    }
    fn handle(&self) -> Option<CoreHandle> {
        self.base.base.base.base.base.base.handle()
    }

    fn item_instance(item: &CoreHandle) -> Option<CoreHandle> {
        item.with(|item| {
            item.as_view_model_instance_list_item()
                .and_then(|item| item.view_model_instance())
        })
        .flatten()
    }

    fn add_parent_to_item(&self, item: &CoreHandle) {
        self.add_parent_to_instance(Self::item_instance(item));
    }

    fn add_parent_to_instance(&self, instance: Option<CoreHandle>) {
        if let (Some(instance), Some(parent)) = (instance, self.parent_view_model_instance.as_ref())
        {
            instance.with_mut(|instance| {
                if let Some(instance) = instance.as_view_model_instance_mut() {
                    instance.add_parent(parent.clone());
                }
            });
        }
    }

    fn remove_parent_from_item(&self, item: &CoreHandle) {
        if let (Some(instance), Some(parent)) = (
            Self::item_instance(item),
            self.parent_view_model_instance.as_ref(),
        ) {
            instance.with_mut(|instance| {
                if let Some(instance) = instance.as_view_model_instance_mut() {
                    instance.remove_parent(parent);
                }
            });
        }
    }

    fn property_value_changed(&mut self) {
        if let Some(owner) = crate::mechanical_port::source::core::CoreObject::core(self).handle() {
            crate::host_viewmodel::capture_native_list_change(owner, &self.list_items);
        }
        self.base.add_dirt(ComponentDirt::BINDINGS);
        #[cfg(feature = "tools")]
        if let Some(callback) = self.changed_callback {
            if !crate::view_model_cell::defer_transaction_tools_callback(self, callback) {
                callback(self);
            }
        }
        self.base.on_value_changed();
    }

    pub fn add_item(&mut self, item: CoreHandle) {
        self.list_items.push(item.clone());
        self.add_parent_to_item(&item);
        self.property_value_changed();
    }

    pub fn add_item_at(&mut self, item: CoreHandle, index: i32) -> bool {
        if index < 0 || index as usize > self.list_items.len() {
            return false;
        }
        self.list_items.insert(index as usize, item.clone());
        self.add_parent_to_item(&item);
        self.property_value_changed();
        true
    }

    pub fn internal_add_item(&mut self, item: CoreHandle) {
        item.with_downcast::<super::viewmodel_instance_list_item::ViewModelInstanceListItem, _>(
            |item| self.internal_add_item_borrowed(item),
        )
        .expect("native list item");
    }

    pub(crate) fn internal_add_item_borrowed(
        &mut self,
        item: &super::viewmodel_instance_list_item::ViewModelInstanceListItem,
    ) {
        self.list_items
            .push(item.base.base.handle().expect("arena-owned list item"));
        self.add_parent_to_instance(item.view_model_instance());
    }

    pub fn remove_item_at(&mut self, index: i32) {
        if index < 0 || index as usize >= self.list_items.len() {
            return;
        }
        let item = self.list_items[index as usize].clone();
        self.remove_parent_from_item(&item);
        self.list_items.remove(index as usize);
        self.property_value_changed();
    }

    pub fn remove_item(&mut self, item: &CoreHandle) {
        self.remove_parent_from_item(item);
        self.list_items.retain(|candidate| candidate != item);
        self.property_value_changed();
    }

    pub fn list_items(&self) -> &[CoreHandle] {
        &self.list_items
    }

    pub fn item(&self, index: u32) -> Option<CoreHandle> {
        self.list_items.get(index as usize).cloned()
    }

    pub fn swap(&mut self, index1: u32, index2: u32) {
        if index1 as usize >= self.list_items.len() || index2 as usize >= self.list_items.len() {
            return;
        }
        self.list_items.swap(index1 as usize, index2 as usize);
        self.property_value_changed();
    }

    pub fn pop(&mut self) -> Option<CoreHandle> {
        let item = self.list_items.pop()?;
        self.property_value_changed();
        Some(item)
    }

    pub fn shift(&mut self) -> Option<CoreHandle> {
        let item = self.list_items.first()?.clone();
        self.remove_item_at(0);
        Some(item)
    }

    pub fn remove_all_items(&mut self) {
        if self.list_items.is_empty() {
            return;
        }
        for item in &self.list_items {
            self.remove_parent_from_item(item);
        }
        self.list_items.clear();
        self.property_value_changed();
    }

    pub fn remove_all_items_with_view_model_instance(
        &mut self,
        view_model_instance: Option<CoreHandle>,
    ) {
        let Some(target) = view_model_instance else {
            return;
        };
        if self.list_items.is_empty() {
            return;
        }
        let parent = self.parent_view_model_instance.clone();
        let mut changed = false;
        self.list_items.retain(|item| {
            let instance = Self::item_instance(item);
            let matches = instance.as_ref() == Some(&target);
            if matches {
                if let (Some(instance), Some(parent)) = (instance, parent.as_ref()) {
                    instance.with_mut(|instance| {
                        if let Some(instance) = instance.as_view_model_instance_mut() {
                            instance.remove_parent(parent);
                        }
                    });
                }
                changed = true;
            }
            !matches
        });
        if changed {
            self.property_value_changed();
        }
    }

    pub fn update_list(&mut self, list: Option<&[CoreHandle]>) {
        let Some(list) = list else {
            return;
        };
        for item in &self.list_items {
            self.remove_parent_from_item(item);
        }
        self.list_items.clear();
        self.list_items.reserve(list.len());
        for item in list {
            self.list_items.push(item.clone());
            self.add_parent_to_item(item);
        }
        self.property_value_changed();
    }

    pub fn clone_definition(&self) -> Self {
        let mut clone = Self::default();
        let mut base = std::mem::take(&mut clone.base);
        base.copy(&self.base, &mut clone);
        clone.base = base;
        clone
    }

    pub fn complete_clone(source: &CoreHandle, cloned: &CoreHandle) -> bool {
        let Some(items) = source.with_downcast::<Self, _>(|source| source.list_items.clone())
        else {
            return false;
        };
        for item in items {
            let Some(item) = item.clone_occurrence() else {
                return false;
            };
            if cloned
                .with_downcast_mut::<Self, _>(|cloned| cloned.internal_add_item(item))
                .is_none()
            {
                return false;
            }
        }
        true
    }

    pub fn clone_value(source: &CoreHandle) -> Option<CoreHandle> {
        source.with_downcast::<Self, _>(|_| ())?;
        source.clone_occurrence()
    }

    pub fn advanced(&mut self) {
        for item in &self.list_items {
            if let Some(instance) = Self::item_instance(item) {
                instance.with_mut(|instance| {
                    if let Some(instance) = instance.as_view_model_instance_mut() {
                        instance.advanced();
                    }
                });
            }
        }
        self.base.advanced();
    }

    pub fn set_parent_view_model_instance(&mut self, parent: Option<CoreHandle>) {
        self.parent_view_model_instance = parent;
    }

    pub fn parent_view_model_instance(&self) -> Option<CoreHandle> {
        self.parent_view_model_instance.clone()
    }

    #[cfg(feature = "tools")]
    pub fn on_changed(&mut self, callback: Option<fn(&mut Self)>) {
        self.changed_callback = callback;
    }
}

impl Drop for ViewModelInstanceList {
    fn drop(&mut self) {
        for item in &self.list_items {
            self.remove_parent_from_item(item);
        }
    }
}
