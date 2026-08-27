use std::ptr::NonNull;

use crate::mechanical_port::source::{
    file::File, generated::viewmodel::viewmodel_base::ViewModelBase, refcnt::RiveRc,
};

use super::{
    symbol_type::SymbolType, viewmodel_instance::ViewModelInstance,
    viewmodel_property::ViewModelProperty,
};

pub struct ViewModel {
    pub base: ViewModelBase,
    properties: Vec<NonNull<ViewModelProperty>>,
    instances: Vec<NonNull<ViewModelInstance>>,
    file: Option<NonNull<File>>,
}

impl Default for ViewModel {
    fn default() -> Self {
        Self {
            base: ViewModelBase::default(),
            properties: Vec::new(),
            instances: Vec::new(),
            file: None,
        }
    }
}

impl ViewModel {
    pub fn add_property(&mut self, property: NonNull<ViewModelProperty>) {
        self.properties.push(property);
    }

    pub fn property_at(&self, index: usize) -> Option<NonNull<ViewModelProperty>> {
        self.properties.get(index).copied()
    }

    pub fn property_named(&self, name: &str) -> Option<NonNull<ViewModelProperty>> {
        self.properties
            .iter()
            .copied()
            .find(|property| unsafe { property.as_ref() }.base.name() == name)
    }

    pub fn property_for_symbol(
        &self,
        symbol_type: SymbolType,
    ) -> Option<NonNull<ViewModelProperty>> {
        self.properties.iter().copied().find(|property| {
            unsafe { property.as_ref() }.base.symbol_type_value() == symbol_type as i32
        })
    }

    pub fn add_instance(&mut self, mut value: NonNull<ViewModelInstance>) {
        self.instances.push(value);
        unsafe { value.as_mut() }.view_model(NonNull::from(&mut *self));
    }

    pub fn instance_at(&self, index: usize) -> Option<NonNull<ViewModelInstance>> {
        self.instances.get(index).copied()
    }

    pub fn instance_named(&self, name: &str) -> Option<NonNull<ViewModelInstance>> {
        self.instances
            .iter()
            .copied()
            .find(|instance| unsafe { instance.as_ref() }.base.name() == name)
    }

    pub fn default_instance(&self) -> Option<NonNull<ViewModelInstance>> {
        self.instances.first().copied()
    }

    pub fn instance_count(&self) -> usize {
        self.instances.len()
    }

    pub fn create_instance(&mut self) -> Option<RiveRc<ViewModelInstance>> {
        unsafe { self.file?.as_mut() }.create_view_model_instance(NonNull::from(&mut *self))
    }

    pub fn create_from_instance(
        &mut self,
        instance_name: &str,
    ) -> Option<RiveRc<ViewModelInstance>> {
        let name = self.base.name().to_owned();
        unsafe { self.file?.as_mut() }.create_view_model_instance_named(&name, instance_name)
    }

    pub fn set_file(&mut self, file: NonNull<File>) {
        self.file = Some(file);
    }

    #[cfg(feature = "rive_tools")]
    pub fn file(&self) -> Option<NonNull<File>> {
        self.file
    }

    pub fn properties(&self) -> Vec<NonNull<ViewModelProperty>> {
        self.properties.clone()
    }

    pub fn instances(&self) -> Vec<NonNull<ViewModelInstance>> {
        self.instances.clone()
    }
}

impl Drop for ViewModel {
    fn drop(&mut self) {
        for property in self.properties.drain(..) {
            unsafe { drop(Box::from_raw(property.as_ptr())) };
        }
    }
}
