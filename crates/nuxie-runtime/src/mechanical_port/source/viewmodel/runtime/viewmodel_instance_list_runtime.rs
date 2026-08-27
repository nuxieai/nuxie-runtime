use super::viewmodel_instance_runtime::{ViewModelInstanceRuntime, ViewModelInstanceSource};
use super::viewmodel_instance_value_runtime::{
    DataType, ViewModelInstanceValue, ViewModelInstanceValueRuntime,
};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
pub trait ListItem<I> {
    fn identity(&self) -> usize;
    fn view_model_instance(&self) -> Option<Rc<I>>;
    fn set_view_model_instance(&self, value: Rc<I>);
}
pub trait ListValue<I, Item: ListItem<I>>: ViewModelInstanceValue {
    fn list_items(&self) -> Vec<Rc<Item>>;
    fn make_item(&self) -> Rc<Item>;
    fn add_item(&self, item: Rc<Item>);
    fn add_item_at(&self, item: Rc<Item>, index: i32) -> bool;
    fn remove_item(&self, item: &Rc<Item>);
    fn swap(&self, a: u32, b: u32);
    fn remove_all_items(&self);
}
pub struct ViewModelInstanceListRuntime<L, I, Item>
where
    L: ListValue<I, Item>,
    I: ViewModelInstanceSource,
    Item: ListItem<I>,
{
    base: ViewModelInstanceValueRuntime<L>,
    items_map: RefCell<HashMap<usize, Rc<ViewModelInstanceRuntime<I>>>>,
}
impl<L, I, Item> ViewModelInstanceListRuntime<L, I, Item>
where
    L: ListValue<I, Item>,
    I: ViewModelInstanceSource + 'static,
    Item: ListItem<I>,
{
    pub fn new(value: Rc<L>) -> Self {
        Self {
            base: ViewModelInstanceValueRuntime::new(value),
            items_map: RefCell::new(HashMap::new()),
        }
    }
    pub fn instance_at(&self, index: i32) -> Option<Rc<ViewModelInstanceRuntime<I>>> {
        let items = self.base.value().list_items();
        if index < 0 || index as usize >= items.len() {
            return None;
        }
        let item = &items[index as usize];
        let instance = item.view_model_instance()?;
        if let Some(runtime) = self.items_map.borrow().get(&item.identity()) {
            return Some(runtime.clone());
        }
        let runtime = ViewModelInstanceRuntime::new(instance);
        self.items_map
            .borrow_mut()
            .insert(item.identity(), runtime.clone());
        Some(runtime)
    }
    pub fn add_instance(&self, runtime: Rc<ViewModelInstanceRuntime<I>>) {
        let item = self.base.value().make_item();
        item.set_view_model_instance(runtime.instance());
        self.items_map.borrow_mut().insert(item.identity(), runtime);
        self.base.value().add_item(item)
    }
    pub fn add_instance_at(&self, runtime: Rc<ViewModelInstanceRuntime<I>>, index: i32) -> bool {
        let item = self.base.value().make_item();
        if self.base.value().add_item_at(item.clone(), index) {
            item.set_view_model_instance(runtime.instance());
            self.items_map.borrow_mut().insert(item.identity(), runtime);
            true
        } else {
            false
        }
    }
    pub fn remove_instance(&self, runtime: &Rc<ViewModelInstanceRuntime<I>>) {
        let items: Vec<_> = self
            .base
            .value()
            .list_items()
            .into_iter()
            .filter(|item| {
                item.view_model_instance()
                    .is_some_and(|instance| Rc::ptr_eq(&instance, &runtime.instance()))
            })
            .collect();
        for item in items {
            self.items_map.borrow_mut().remove(&item.identity());
            self.base.value().remove_item(&item);
        }
    }
    pub fn remove_instance_at(&self, index: i32) {
        let items = self.base.value().list_items();
        if index >= 0 && (index as usize) < items.len() {
            let item = &items[index as usize];
            self.base.value().remove_item(item);
            self.items_map.borrow_mut().remove(&item.identity());
        }
    }
    pub fn swap(&self, a: u32, b: u32) {
        self.base.value().swap(a, b)
    }
    pub fn remove_all_instances(&self) {
        self.base.value().remove_all_items();
        self.items_map.borrow_mut().clear()
    }
    pub fn size(&self) -> usize {
        self.base.value().list_items().len()
    }
    pub fn data_type(&self) -> DataType {
        DataType::List
    }
}
