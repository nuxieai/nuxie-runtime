use std::ptr::NonNull;

use crate::mechanical_port::source::{component_dirt::ComponentDirt, core::Core, refcnt::RiveRc};

use super::{
    viewmodel_instance::ViewModelInstance, viewmodel_instance_value::ViewModelInstanceValue,
};

pub trait PropertyWriter {
    fn write_value(&mut self);
}

pub struct PropertySymbolDependent {
    core_object: NonNull<Core>,
    core_object_listener: NonNull<CoreObjectListener>,
    instance_value: Option<NonNull<ViewModelInstanceValue>>,
    writer: NonNull<dyn PropertyWriter>,
    dependent_identity: NonNull<dyn super::viewmodel_value_dependent::ViewModelValueDependent>,
}

impl PropertySymbolDependent {
    pub fn new(
        core_object: NonNull<Core>,
        core_object_listener: NonNull<CoreObjectListener>,
        mut instance_value: Option<NonNull<ViewModelInstanceValue>>,
        writer: NonNull<dyn PropertyWriter>,
        dependent_identity: NonNull<dyn super::viewmodel_value_dependent::ViewModelValueDependent>,
    ) -> Self {
        let dependent = Self {
            core_object,
            core_object_listener,
            instance_value,
            writer,
            dependent_identity,
        };
        if let Some(value) = &mut instance_value {
            unsafe { value.as_mut() }.add_dependent(dependent.dependent_identity);
        }
        dependent
    }

    pub fn add_dirt(&mut self, _value: ComponentDirt, _recurse: bool) {
        unsafe { self.writer.as_mut() }.write_value();
        unsafe { self.core_object_listener.as_mut() }.mark_dirty();
    }
}

impl Drop for PropertySymbolDependent {
    fn drop(&mut self) {
        if let Some(mut value) = self.instance_value {
            unsafe { value.as_mut() }.remove_dependent(self.dependent_identity);
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
}

pub trait ListenerCallbacks {
    fn mark_dirty(&mut self) {}
    fn create_properties(&mut self);
}

pub struct CoreObjectListener {
    core: Option<Box<Core>>,
    instance: Option<RiveRc<ViewModelInstance>>,
    properties: Vec<NonNull<PropertySymbolDependent>>,
    callbacks: NonNull<dyn ListenerCallbacks>,
}

impl CoreObjectListener {
    pub fn new(
        core: Box<Core>,
        instance: Option<RiveRc<ViewModelInstance>>,
        callbacks: NonNull<dyn ListenerCallbacks>,
    ) -> Self {
        Self {
            core: Some(core),
            instance,
            properties: Vec::new(),
            callbacks,
        }
    }

    pub fn mark_dirty(&mut self) {
        unsafe { self.callbacks.as_mut() }.mark_dirty();
    }

    pub fn create_properties(&mut self) {
        self.delete_properties();
        unsafe { self.callbacks.as_mut() }.create_properties();
    }

    pub fn delete_properties(&mut self) {
        for property in self.properties.drain(..) {
            unsafe { drop(Box::from_raw(property.as_ptr())) };
        }
    }

    pub fn remap(&mut self, instance: Option<RiveRc<ViewModelInstance>>) {
        let changed = match (&self.instance, &instance) {
            (Some(left), Some(right)) => !RiveRc::ptr_eq(left, right),
            (None, None) => false,
            _ => true,
        };
        if changed {
            self.delete_properties();
            self.instance = instance;
            self.create_properties();
        }
    }
}

impl Drop for CoreObjectListener {
    fn drop(&mut self) {
        self.core.take();
        self.delete_properties();
    }
}
