use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::{Rc, Weak};

use super::viewmodel_instance_value_runtime::DataType;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PropertyData {
    pub data_type: DataType,
    pub name: String,
    pub enum_name: String,
}

pub trait RuntimeValue {
    fn data_type(&self) -> DataType;
}
pub trait PropertyValueSource {
    fn data_type(&self) -> DataType;
    fn make_runtime(&self) -> Rc<dyn RuntimeValue>;
}
pub trait ViewModelInstanceSource: Sized {
    fn name(&self) -> &str;
    fn view_model_name(&self) -> &str;
    fn property_count(&self) -> usize;
    fn properties(&self) -> Vec<PropertyData>;
    fn property_value(&self, name: &str) -> Option<Rc<dyn PropertyValueSource>>;
    fn referenced_view_model_instance(&self, name: &str) -> Option<Rc<Self>>;
    fn replace_view_model_by_name(&self, name: &str, value: Rc<Self>) -> bool;
}

pub struct ViewModelInstanceRuntime<I: ViewModelInstanceSource> {
    self_weak: Weak<Self>,
    instance: Rc<I>,
    properties: RefCell<HashMap<String, Rc<dyn RuntimeValue>>>,
    view_model_instances: RefCell<HashMap<String, Rc<ViewModelInstanceRuntime<I>>>>,
}

impl<I: ViewModelInstanceSource + 'static> ViewModelInstanceRuntime<I> {
    pub fn new(instance: Rc<I>) -> Rc<Self> {
        Rc::new_cyclic(|self_weak| Self {
            self_weak: self_weak.clone(),
            instance,
            properties: RefCell::new(HashMap::new()),
            view_model_instances: RefCell::new(HashMap::new()),
        })
    }
    pub fn name(&self) -> &str {
        self.instance.name()
    }
    pub fn view_model_name(&self) -> &str {
        self.instance.view_model_name()
    }
    pub fn property_count(&self) -> usize {
        self.instance.property_count()
    }
    pub fn instance(&self) -> Rc<I> {
        self.instance.clone()
    }
    fn get_property_name_from_path<'a>(&self, path: &'a str) -> &'a str {
        if path.is_empty() {
            ""
        } else {
            path.rsplit_once('/').map_or(path, |(_, name)| name)
        }
    }
    fn view_model_instance_at_path(&self, path: &str) -> Option<Rc<Self>> {
        let (first, rest) = path
            .split_once('/')
            .map_or((path, ""), |(first, rest)| (first, rest));
        if first.is_empty() {
            return None;
        }
        let instance = self.instance_runtime(first)?;
        if rest.is_empty() {
            Some(instance)
        } else {
            instance.view_model_instance_at_path(rest)
        }
    }
    fn view_model_instance_from_full_path(&self, path: &str) -> Option<Rc<Self>> {
        path.rsplit_once('/').map_or_else(
            || self.self_weak.upgrade(),
            |(parents, _)| self.view_model_instance_at_path(parents),
        )
    }
    fn get_property_instance(
        &self,
        name: &str,
        data_type: DataType,
    ) -> Option<Rc<dyn RuntimeValue>> {
        if let Some(runtime) = self.properties.borrow().get(name) {
            return (runtime.data_type() == data_type).then(|| runtime.clone());
        }
        let value = self.instance.property_value(name)?;
        if value.data_type() != data_type {
            return None;
        }
        let runtime = value.make_runtime();
        self.properties
            .borrow_mut()
            .insert(name.to_owned(), runtime.clone());
        Some(runtime)
    }
    fn property_of_type(&self, path: &str, data_type: DataType) -> Option<Rc<dyn RuntimeValue>> {
        let name = self.get_property_name_from_path(path);
        let owner = self.view_model_instance_from_full_path(path)?;
        owner.get_property_instance(name, data_type)
    }
    pub fn property_number(&self, path: &str) -> Option<Rc<dyn RuntimeValue>> {
        self.property_of_type(path, DataType::Number)
    }
    pub fn property_string(&self, path: &str) -> Option<Rc<dyn RuntimeValue>> {
        self.property_of_type(path, DataType::String)
    }
    pub fn property_boolean(&self, path: &str) -> Option<Rc<dyn RuntimeValue>> {
        self.property_of_type(path, DataType::Boolean)
    }
    pub fn property_color(&self, path: &str) -> Option<Rc<dyn RuntimeValue>> {
        self.property_of_type(path, DataType::Color)
    }
    pub fn property_enum(&self, path: &str) -> Option<Rc<dyn RuntimeValue>> {
        self.property_of_type(path, DataType::Enum)
    }
    pub fn property_trigger(&self, path: &str) -> Option<Rc<dyn RuntimeValue>> {
        self.property_of_type(path, DataType::Trigger)
    }
    pub fn property_list(&self, path: &str) -> Option<Rc<dyn RuntimeValue>> {
        self.property_of_type(path, DataType::List)
    }
    pub fn property_list_index(&self, path: &str) -> Option<Rc<dyn RuntimeValue>> {
        self.property_of_type(path, DataType::SymbolListIndex)
    }
    pub fn property_image(&self, path: &str) -> Option<Rc<dyn RuntimeValue>> {
        self.property_of_type(path, DataType::AssetImage)
    }
    pub fn property_font(&self, path: &str) -> Option<Rc<dyn RuntimeValue>> {
        self.property_of_type(path, DataType::AssetFont)
    }
    pub fn property_blob(&self, path: &str) -> Option<Rc<dyn RuntimeValue>> {
        self.property_of_type(path, DataType::AssetBlob)
    }
    pub fn property_artboard(&self, path: &str) -> Option<Rc<dyn RuntimeValue>> {
        self.property_of_type(path, DataType::Artboard)
    }
    fn instance_runtime(&self, name: &str) -> Option<Rc<Self>> {
        if let Some(runtime) = self.view_model_instances.borrow().get(name) {
            return Some(runtime.clone());
        }
        let instance = self.instance.referenced_view_model_instance(name)?;
        let runtime = Self::new(instance);
        self.view_model_instances
            .borrow_mut()
            .insert(name.to_owned(), runtime.clone());
        Some(runtime)
    }
    pub fn property_view_model(&self, path: &str) -> Option<Rc<Self>> {
        let name = self.get_property_name_from_path(path);
        self.view_model_instance_from_full_path(path)?
            .instance_runtime(name)
    }
    pub fn property(&self, path: &str) -> Option<Rc<dyn RuntimeValue>> {
        if path.is_empty() {
            return None;
        }
        let name = self.get_property_name_from_path(path);
        let owner = self.view_model_instance_from_full_path(path)?;
        let property = owner
            .properties()
            .into_iter()
            .find(|property| property.name == name)?;
        match property.data_type {
            DataType::String => owner.property_string(name),
            DataType::Number => owner.property_number(name),
            DataType::Boolean => owner.property_boolean(name),
            DataType::Color => owner.property_color(name),
            DataType::AssetImage => owner.property_image(name),
            DataType::AssetFont => owner.property_font(name),
            DataType::AssetBlob => owner.property_blob(name),
            DataType::Artboard => owner.property_artboard(name),
            DataType::List => owner.property_list(name),
            DataType::Enum => owner.property_enum(name),
            DataType::Trigger => owner.property_trigger(name),
            DataType::SymbolListIndex => owner.property_list_index(name),
            _ => None,
        }
    }
    pub fn replace_view_model(&self, path: &str, value: Rc<Self>) -> bool {
        let name = self.get_property_name_from_path(path);
        self.view_model_instance_from_full_path(path)
            .is_some_and(|owner| owner.replace_view_model_by_name(name, value))
    }
    pub fn replace_view_model_by_name(&self, name: &str, value: Rc<Self>) -> bool {
        if !self
            .instance
            .replace_view_model_by_name(name, value.instance())
        {
            return false;
        }
        let is_stored = self
            .view_model_instances
            .borrow()
            .values()
            .any(|stored| Rc::ptr_eq(stored, &value));
        if !is_stored {
            self.view_model_instances
                .borrow_mut()
                .insert(name.to_owned(), value);
        }
        true
    }
    pub fn properties(&self) -> Vec<PropertyData> {
        self.instance.properties()
    }
}
