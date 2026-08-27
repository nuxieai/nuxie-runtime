use std::ptr::NonNull;

use crate::mechanical_port::source::{
    component_dirt::ComponentDirt,
    generated::viewmodel::viewmodel_instance_list_base::ViewModelInstanceListBase, refcnt::RiveRc,
};

use super::{
    viewmodel_instance::ViewModelInstance, viewmodel_instance_list_item::ViewModelInstanceListItem,
};

#[derive(Default)]
pub struct ViewModelInstanceList {
    pub base: ViewModelInstanceListBase,
    list_items: Vec<RiveRc<ViewModelInstanceListItem>>,
    parent_view_model_instance: Option<NonNull<ViewModelInstance>>,
    #[cfg(feature = "rive_tools")]
    changed_callback: Option<fn(&mut Self)>,
}

impl ViewModelInstanceList {
    fn property_value_changed(&mut self) {
        self.base.add_dirt(ComponentDirt::BINDINGS);
        #[cfg(feature = "rive_tools")]
        if let Some(callback) = self.changed_callback {
            callback(self);
        }
        self.base.on_value_changed();
    }

    pub fn add_item(&mut self, item: RiveRc<ViewModelInstanceListItem>) {
        self.list_items.push(item);
        let instance = self
            .list_items
            .last()
            .and_then(|item| item.view_model_instance());
        if let (Some(mut instance), Some(parent)) = (instance, self.parent_view_model_instance) {
            instance.add_parent(parent);
        }
        self.property_value_changed();
    }

    pub fn add_item_at(&mut self, item: RiveRc<ViewModelInstanceListItem>, index: i32) -> bool {
        if index < 0 || index as usize > self.list_items.len() {
            return false;
        }
        self.list_items.insert(index as usize, item);
        let instance = self.list_items[index as usize].view_model_instance();
        if let (Some(mut instance), Some(parent)) = (instance, self.parent_view_model_instance) {
            instance.add_parent(parent);
        }
        self.property_value_changed();
        true
    }

    pub fn internal_add_item(&mut self, item: RiveRc<ViewModelInstanceListItem>) {
        self.list_items.push(item);
        let instance = self
            .list_items
            .last()
            .and_then(|item| item.view_model_instance());
        if let (Some(mut instance), Some(parent)) = (instance, self.parent_view_model_instance) {
            instance.add_parent(parent);
        }
    }

    pub fn remove_item_at(&mut self, index: i32) {
        if index < 0 || index as usize >= self.list_items.len() {
            return;
        }
        let item = &self.list_items[index as usize];
        if let (Some(mut instance), Some(parent)) =
            (item.view_model_instance(), self.parent_view_model_instance)
        {
            instance.remove_parent(parent);
        }
        self.list_items.remove(index as usize);
        self.property_value_changed();
    }

    pub fn remove_item(&mut self, item: &RiveRc<ViewModelInstanceListItem>) {
        self.list_items
            .retain(|candidate| !RiveRc::ptr_eq(candidate, item));
        if let (Some(mut instance), Some(parent)) =
            (item.view_model_instance(), self.parent_view_model_instance)
        {
            instance.remove_parent(parent);
        }
        self.property_value_changed();
    }

    pub fn list_items(&self) -> &[RiveRc<ViewModelInstanceListItem>] {
        &self.list_items
    }

    pub fn item(&self, index: u32) -> Option<RiveRc<ViewModelInstanceListItem>> {
        self.list_items.get(index as usize).cloned()
    }

    pub fn swap(&mut self, index1: u32, index2: u32) {
        if index1 as usize >= self.list_items.len() || index2 as usize >= self.list_items.len() {
            return;
        }
        self.list_items.swap(index1 as usize, index2 as usize);
        self.property_value_changed();
    }

    pub fn pop(&mut self) -> Option<RiveRc<ViewModelInstanceListItem>> {
        let item = self.list_items.pop()?;
        self.property_value_changed();
        Some(item)
    }

    pub fn shift(&mut self) -> Option<RiveRc<ViewModelInstanceListItem>> {
        let item = self.list_items.first()?.clone();
        self.remove_item_at(0);
        Some(item)
    }

    pub fn remove_all_items(&mut self) {
        if self.list_items.is_empty() {
            return;
        }
        for item in &self.list_items {
            if let (Some(mut instance), Some(parent)) =
                (item.view_model_instance(), self.parent_view_model_instance)
            {
                instance.remove_parent(parent);
            }
        }
        self.list_items.clear();
        self.property_value_changed();
    }

    pub fn remove_all_items_with_view_model_instance(
        &mut self,
        view_model_instance: Option<NonNull<ViewModelInstance>>,
    ) {
        let Some(target) = view_model_instance else {
            return;
        };
        if self.list_items.is_empty() {
            return;
        }
        let mut changed = false;
        let parent = self.parent_view_model_instance;
        self.list_items.retain(|item| {
            let matches = item
                .view_model_instance()
                .is_some_and(|instance| std::ptr::eq(instance.as_ptr(), target.as_ptr()));
            if matches {
                if let (Some(mut instance), Some(parent)) = (item.view_model_instance(), parent) {
                    instance.remove_parent(parent);
                }
                changed = true;
            }
            !matches
        });
        if changed {
            self.property_value_changed();
        }
    }

    pub fn update_list(&mut self, list: Option<&[RiveRc<ViewModelInstanceListItem>]>) {
        let Some(list) = list else {
            return;
        };
        if let Some(parent) = self.parent_view_model_instance {
            for item in &self.list_items {
                if let Some(mut instance) = item.view_model_instance() {
                    instance.remove_parent(parent);
                }
            }
        }
        self.list_items.clear();
        self.list_items.reserve(list.len());
        for item in list {
            self.list_items.push(item.clone());
            if let (Some(parent), Some(mut instance)) =
                (self.parent_view_model_instance, item.view_model_instance())
            {
                instance.add_parent(parent);
            }
        }
        self.property_value_changed();
    }

    pub fn clone_value(&self) -> Box<Self> {
        let mut cloned = Box::new(Self {
            base: self.base.clone_base(),
            list_items: Vec::new(),
            parent_view_model_instance: None,
            #[cfg(feature = "rive_tools")]
            changed_callback: None,
        });
        for item in &self.list_items {
            cloned.internal_add_item(item.clone_core_item());
        }
        cloned
    }

    pub fn advanced(&mut self) {
        for item in &mut self.list_items {
            if let Some(mut instance) = item.view_model_instance() {
                instance.advanced();
            }
        }
        self.base.advanced();
    }

    pub fn set_parent_view_model_instance(&mut self, parent: Option<NonNull<ViewModelInstance>>) {
        self.parent_view_model_instance = parent;
    }

    pub fn parent_view_model_instance(&self) -> Option<NonNull<ViewModelInstance>> {
        self.parent_view_model_instance
    }

    #[cfg(feature = "rive_tools")]
    pub fn on_changed(&mut self, callback: Option<fn(&mut Self)>) {
        self.changed_callback = callback;
    }
}

impl Drop for ViewModelInstanceList {
    fn drop(&mut self) {
        if let Some(parent) = self.parent_view_model_instance {
            for item in &self.list_items {
                if let Some(mut instance) = item.view_model_instance() {
                    instance.remove_parent(parent);
                }
            }
        }
    }
}
