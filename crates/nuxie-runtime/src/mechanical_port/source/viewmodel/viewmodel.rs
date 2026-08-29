use crate::mechanical_port::source::{
    core::CoreHandle, file::RuntimeFileWeakHandle,
    generated::viewmodel::viewmodel_base::ViewModelBase,
};

use super::symbol_type::SymbolType;

pub struct ViewModel {
    pub base: ViewModelBase,
    properties: Vec<CoreHandle>,
    instances: Vec<CoreHandle>,
    file: RuntimeFileWeakHandle,
}

impl Default for ViewModel {
    fn default() -> Self {
        Self {
            base: ViewModelBase::default(),
            properties: Vec::new(),
            instances: Vec::new(),
            file: RuntimeFileWeakHandle::default(),
        }
    }
}

impl ViewModel {
    fn handle(&self) -> Option<CoreHandle> {
        self.base.base.base.base.handle()
    }

    pub fn add_property(&mut self, property: CoreHandle) {
        self.properties.push(property);
    }

    pub fn property_at(&self, index: usize) -> Option<CoreHandle> {
        self.properties.get(index).cloned()
    }

    pub fn property_named(&self, name: &str) -> Option<CoreHandle> {
        self.properties
            .iter()
            .find(|property| {
                property
                    .with(|property| {
                        property
                            .as_view_model_property()
                            .is_some_and(|property| property.base.name() == name)
                    })
                    .unwrap_or(false)
            })
            .cloned()
    }

    pub fn property_for_symbol(&self, symbol_type: SymbolType) -> Option<CoreHandle> {
        self.properties
            .iter()
            .find(|property| {
                property
                    .with(|property| {
                        property.as_view_model_property().is_some_and(|property| {
                            property.base.symbol_type_value() == symbol_type as u32
                        })
                    })
                    .unwrap_or(false)
            })
            .cloned()
    }

    pub fn add_instance(&mut self, value: &mut super::viewmodel_instance::ViewModelInstance) {
        self.instances
            .push(value.handle().expect("ViewModel instances are arena-owned"));
        value.view_model(self.handle().expect("ViewModel is arena-owned"));
    }

    pub fn instance_at(&self, index: usize) -> Option<CoreHandle> {
        self.instances.get(index).cloned()
    }

    pub fn instance_named(&self, name: &str) -> Option<CoreHandle> {
        self.instances
            .iter()
            .find(|instance| {
                instance
                    .with(|instance| {
                        instance
                            .as_view_model_instance()
                            .is_some_and(|instance| instance.base.name() == name)
                    })
                    .unwrap_or(false)
            })
            .cloned()
    }

    pub fn default_instance(&self) -> Option<CoreHandle> {
        self.instances.first().cloned()
    }

    pub fn instance_count(&self) -> usize {
        self.instances.len()
    }

    pub fn create_instance(&mut self) -> Option<CoreHandle> {
        let view_model = self.handle()?;
        self.file
            .with_file_mut(|file| file.create_view_model_instance(view_model))
            .flatten()
    }

    pub fn create_instance_handle(owner: &CoreHandle) -> Option<CoreHandle> {
        let file = owner.with_downcast::<Self, _>(Self::file_handle)?;
        file.with_file_mut(|file| file.create_view_model_instance(owner.clone()))
            .flatten()
    }

    pub fn create_from_instance_handle(owner: &CoreHandle, name: &str) -> Option<CoreHandle> {
        let (file, model_name) = owner
            .with_downcast::<Self, _>(|model| (model.file.clone(), model.base.name().to_owned()))?;
        file.with_file(|file| file.create_view_model_instance_named(&model_name, name))
            .flatten()
    }

    pub fn create_from_instance(&mut self, instance_name: &str) -> Option<CoreHandle> {
        let name = self.base.name().to_owned();
        self.file
            .with_file_mut(|file| file.create_view_model_instance_named(&name, instance_name))
            .flatten()
    }

    pub fn set_file(&mut self, file: RuntimeFileWeakHandle) {
        self.file = file;
    }

    pub(crate) fn file_handle(&self) -> RuntimeFileWeakHandle {
        self.file.clone()
    }

    #[cfg(feature = "tools")]
    pub fn file(&self) -> RuntimeFileWeakHandle {
        self.file.clone()
    }

    pub fn properties(&self) -> Vec<CoreHandle> {
        self.properties.clone()
    }

    pub fn instances(&self) -> Vec<CoreHandle> {
        self.instances.clone()
    }
}

impl Drop for ViewModel {
    fn drop(&mut self) {
        // Properties are arena-owned Core occurrences. The ViewModel only
        // retains their identities and must not reconstruct owning Boxes.
        self.properties.clear();
    }
}
