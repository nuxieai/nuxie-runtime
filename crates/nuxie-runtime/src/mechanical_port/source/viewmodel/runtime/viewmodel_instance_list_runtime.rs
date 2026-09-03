use std::{cell::RefCell, collections::HashMap};

use crate::mechanical_port::source::{
    core::CoreHandle, viewmodel::viewmodel_instance_list_item::ViewModelInstanceListItem,
};

use super::{
    viewmodel_instance_runtime::{RuntimeViewModelInstanceHandle, ViewModelInstanceRuntime},
    viewmodel_instance_value_runtime::{DataType, ViewModelInstanceValueRuntime},
};

#[derive(Clone)]
pub struct ViewModelInstanceListRuntime {
    base: ViewModelInstanceValueRuntime,
    items: std::rc::Rc<RefCell<HashMap<CoreHandle, RuntimeViewModelInstanceHandle>>>,
}

impl ViewModelInstanceListRuntime {
    pub fn new(base: ViewModelInstanceValueRuntime) -> Option<Self> {
        (base.data_type() == DataType::List).then_some(Self {
            base,
            items: Default::default(),
        })
    }

    fn list_items(&self) -> Vec<CoreHandle> {
        self.base
            .handle()
            .with(|list| {
                list.as_view_model_instance_list()
                    .map(|list| list.list_items().to_vec())
            })
            .flatten()
            .unwrap_or_default()
    }

    pub fn instance_at(&self, index: i32) -> Option<RuntimeViewModelInstanceHandle> {
        let item = self.list_items().get(usize::try_from(index).ok()?)?.clone();
        if let Some(runtime) = self.items.borrow().get(&item) {
            return Some(runtime.clone());
        }
        let instance = item
            .with(|item| {
                item.as_view_model_instance_list_item()?
                    .view_model_instance()
            })
            .flatten()?;
        let runtime = ViewModelInstanceRuntime::new(instance).into_handle();
        self.items.borrow_mut().insert(item, runtime.clone());
        Some(runtime)
    }

    fn make_item(&self, runtime: &RuntimeViewModelInstanceHandle) -> Option<CoreHandle> {
        let item = self
            .base
            .handle()
            .insert_sibling(ViewModelInstanceListItem::default())?;
        let initialized = item
            .with_mut(|item| {
                item.as_view_model_instance_list_item_mut()
                    .map(|item| item.set_view_model_instance(Some(runtime.instance())))
            })
            .flatten()
            .is_some();
        if !initialized {
            item.remove_occurrence();
            return None;
        }
        self.items
            .borrow_mut()
            .insert(item.clone(), runtime.clone());
        Some(item)
    }

    pub fn add_instance(&self, runtime: RuntimeViewModelInstanceHandle) {
        let Some(item) = self.make_item(&runtime) else {
            return;
        };
        self.base.handle().with_mut(|list| {
            if let Some(list) = list.as_view_model_instance_list_mut() {
                list.add_item(item);
            }
        });
    }

    pub fn add_instance_at(&self, runtime: RuntimeViewModelInstanceHandle, index: i32) -> bool {
        let Some(item) = self
            .base
            .handle()
            .insert_sibling(ViewModelInstanceListItem::default())
        else {
            return false;
        };
        item.with_mut(|item| {
            item.as_view_model_instance_list_item_mut()
                .expect("new list item")
                .set_view_model_instance(Some(runtime.instance()));
        });
        let inserted = self
            .base
            .handle()
            .with_mut(|list| {
                list.as_view_model_instance_list_mut()
                    .is_some_and(|list| list.add_item_at(item.clone(), index))
            })
            .unwrap_or(false);
        if inserted {
            self.items.borrow_mut().insert(item, runtime);
        } else {
            item.remove_occurrence();
        }
        inserted
    }

    pub fn remove_instance(&self, runtime: &RuntimeViewModelInstanceHandle) {
        let items = self.list_items();
        for item in items {
            let matches = item
                .with(|item| {
                    item.as_view_model_instance_list_item()
                        .and_then(|item| item.view_model_instance())
                        .is_some_and(|instance| instance == runtime.instance())
                })
                .unwrap_or(false);
            if matches {
                self.base.handle().with_mut(|list| {
                    if let Some(list) = list.as_view_model_instance_list_mut() {
                        list.remove_item(&item);
                    }
                });
                self.items.borrow_mut().remove(&item);
                item.remove_occurrence();
            }
        }
    }

    pub fn remove_instance_at(&self, index: i32) {
        let item = usize::try_from(index)
            .ok()
            .and_then(|index| self.list_items().get(index).cloned());
        self.base.handle().with_mut(|list| {
            if let Some(list) = list.as_view_model_instance_list_mut() {
                list.remove_item_at(index);
            }
        });
        if let Some(item) = item {
            self.items.borrow_mut().remove(&item);
            item.remove_occurrence();
        }
    }

    pub fn swap(&self, a: u32, b: u32) {
        self.base.handle().with_mut(|list| {
            if let Some(list) = list.as_view_model_instance_list_mut() {
                list.swap(a, b);
            }
        });
    }

    pub fn remove_all_instances(&self) {
        let items = self.list_items();
        self.base.handle().with_mut(|list| {
            if let Some(list) = list.as_view_model_instance_list_mut() {
                list.remove_all_items();
            }
        });
        self.items.borrow_mut().clear();
        for item in items {
            item.remove_occurrence();
        }
    }

    pub fn size(&self) -> usize {
        self.list_items().len()
    }
    pub fn data_type(&self) -> DataType {
        DataType::List
    }
    pub fn value_runtime(&self) -> &ViewModelInstanceValueRuntime {
        &self.base
    }
}
