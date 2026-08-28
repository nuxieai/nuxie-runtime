use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

use crate::mechanical_port::source::{component_dirt::ComponentDirt, core::CoreHandle};

use super::viewmodel_instance_value::ValueDependentHandle;

pub trait PropertyWriter {
    fn write_value(&mut self);
}

pub type PropertyWriterHandle = Rc<RefCell<dyn PropertyWriter>>;
pub type CoreObjectListenerHandle = Rc<RefCell<CoreObjectListener>>;

pub struct PropertySymbolDependent {
    core_object: CoreHandle,
    core_object_listener: Weak<RefCell<CoreObjectListener>>,
    instance_value: Option<CoreHandle>,
    writer: PropertyWriterHandle,
    dependent_identity: ValueDependentHandle,
}

impl PropertySymbolDependent {
    pub fn new(
        core_object: CoreHandle,
        core_object_listener: &CoreObjectListenerHandle,
        instance_value: Option<CoreHandle>,
        writer: PropertyWriterHandle,
        dependent_identity: ValueDependentHandle,
    ) -> Self {
        let dependent = Self {
            core_object,
            core_object_listener: Rc::downgrade(core_object_listener),
            instance_value: instance_value.clone(),
            writer,
            dependent_identity: dependent_identity.clone(),
        };
        if let Some(value) = instance_value {
            value.with_mut(|value| {
                if let Some(value) = value.as_view_model_instance_value_mut() {
                    value.add_dependent(dependent_identity);
                }
            });
        }
        dependent
    }

    pub fn add_dirt(&mut self, _value: ComponentDirt, _recurse: bool) {
        self.writer.borrow_mut().write_value();
        if let Some(listener) = self.core_object_listener.upgrade() {
            listener.borrow_mut().mark_dirty();
        }
    }

    pub fn core_object(&self) -> &CoreHandle {
        &self.core_object
    }
}

impl Drop for PropertySymbolDependent {
    fn drop(&mut self) {
        if let Some(value) = self.instance_value.as_ref() {
            value.with_mut(|value| {
                if let Some(value) = value.as_view_model_instance_value_mut() {
                    value.remove_dependent(&self.dependent_identity);
                }
            });
        }
    }
}

pub struct PropertySymbolDependentSingle {
    pub dependent: PropertySymbolDependent,
    property_key: u16,
}

impl PropertySymbolDependentSingle {
    pub fn new(dependent: PropertySymbolDependent, property_key: u16) -> Self {
        Self {
            dependent,
            property_key,
        }
    }

    pub fn property_key(&self) -> u16 {
        self.property_key
    }
}

pub struct PropertySymbolDependentMulti {
    pub dependent: PropertySymbolDependent,
    property_keys: Vec<u16>,
}

impl PropertySymbolDependentMulti {
    pub fn new(dependent: PropertySymbolDependent, property_keys: Vec<u16>) -> Self {
        Self {
            dependent,
            property_keys,
        }
    }

    pub fn property_keys(&self) -> &[u16] {
        &self.property_keys
    }
}

pub trait ListenerCallbacks {
    fn mark_dirty(&mut self) {}
    fn create_properties(&mut self);
}

pub type ListenerCallbacksHandle = Rc<RefCell<dyn ListenerCallbacks>>;

pub struct CoreObjectListener {
    core: Option<CoreHandle>,
    instance: Option<CoreHandle>,
    properties: Vec<Box<PropertySymbolDependent>>,
    callbacks: ListenerCallbacksHandle,
}

impl CoreObjectListener {
    pub fn new(
        core: CoreHandle,
        instance: Option<CoreHandle>,
        callbacks: ListenerCallbacksHandle,
    ) -> Self {
        Self {
            core: Some(core),
            instance,
            properties: Vec::new(),
            callbacks,
        }
    }

    pub fn mark_dirty(&mut self) {
        self.callbacks.borrow_mut().mark_dirty();
    }

    pub fn create_properties(&mut self) {
        self.delete_properties();
        self.callbacks.borrow_mut().create_properties();
    }

    pub fn delete_properties(&mut self) {
        self.properties.clear();
    }

    pub fn remap(&mut self, instance: Option<CoreHandle>) {
        if self.instance != instance {
            self.delete_properties();
            self.instance = instance;
            self.create_properties();
        }
    }

    pub fn add_property(&mut self, property: Box<PropertySymbolDependent>) {
        self.properties.push(property);
    }
}

impl Drop for CoreObjectListener {
    fn drop(&mut self) {
        self.core = None;
        self.delete_properties();
    }
}
