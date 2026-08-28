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
        if let (Some(instance), Some(parent)) = (
            Self::item_instance(item),
            self.parent_view_model_instance.as_ref(),
        ) {
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
        self.base.add_dirt(ComponentDirt::BINDINGS);
        #[cfg(feature = "tools")]
        if let Some(callback) = self.changed_callback {
            callback(self);
        }
        self.base.on_value_changed();
    }

    pub fn add_item(&mut self, item: CoreHandle) {
        self.add_parent_to_item(&item);
        self.list_items.push(item);
        self.property_value_changed();
    }

    pub fn add_item_at(&mut self, item: CoreHandle, index: i32) -> bool {
        if index < 0 || index as usize > self.list_items.len() {
            return false;
        }
        self.add_parent_to_item(&item);
        self.list_items.insert(index as usize, item);
        self.property_value_changed();
        true
    }

    pub fn internal_add_item(&mut self, item: CoreHandle) {
        self.add_parent_to_item(&item);
        self.list_items.push(item);
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
        self.remove_parent_from_item(&item);
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
            self.add_parent_to_item(item);
            self.list_items.push(item.clone());
        }
        self.property_value_changed();
    }

    pub fn clone_value(&self) -> Option<CoreHandle> {
        let cloned = self.handle()?.clone_occurrence()?;
        for item in &self.list_items {
            let Some(item) = item.clone_occurrence() else {
                continue;
            };
            cloned.with_mut(|cloned| {
                if let Some(cloned) = cloned.as_view_model_instance_list_mut() {
                    cloned.internal_add_item(item);
                }
            });
        }
        Some(cloned)
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
