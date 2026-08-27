use std::{collections::HashMap, ptr::NonNull};

use crate::mechanical_port::source::{
    component::Component,
    data_bind::data_bind_container::DataBindContainer,
    generated::viewmodel::viewmodel_instance_base::ViewModelInstanceBase,
    importers::{
        artboard_importer::ArtboardImporter, backboard_importer::BackboardImporter,
        import_stack::ImportStack,
    },
    refcnt::RiveRc,
    status_code::StatusCode,
};

use super::{
    symbol_type::SymbolType, viewmodel::ViewModel,
    viewmodel_instance_value::ViewModelInstanceValue,
    viewmodel_instance_viewmodel::ViewModelInstanceViewModel,
};

#[derive(Default)]
pub struct ViewModelInstance {
    pub base: ViewModelInstanceBase,
    property_values: Vec<RiveRc<ViewModelInstanceValue>>,
    parents: Vec<NonNull<ViewModelInstance>>,
    dependents: Vec<NonNull<DataBindContainer>>,
    property_symbols: HashMap<SymbolType, NonNull<ViewModelInstanceValue>>,
    view_model: Option<NonNull<ViewModel>>,
}

impl ViewModelInstance {
    pub fn pointer_key(instance: Option<NonNull<Self>>) -> u32 {
        let Some(instance) = instance else {
            return u32::MAX;
        };
        let pointer = instance.as_ptr() as u64;
        (pointer ^ (pointer >> 32)) as u32
    }

    pub fn add_value(&mut self, mut value: NonNull<ViewModelInstanceValue>) {
        if self
            .property_values
            .iter()
            .any(|existing| std::ptr::eq(existing.as_ptr(), value.as_ptr()))
        {
            return;
        }
        unsafe { value.as_mut() }.set_view_model_instance(NonNull::from(&mut *self));
        if let Some(property) = unsafe { value.as_ref() }.view_model_property() {
            if let Some(symbol) =
                SymbolType::from_i32(unsafe { property.as_ref() }.base.symbol_type_value())
            {
                if symbol != SymbolType::None {
                    self.set_property_symbol(symbol, value);
                }
            }
        }
        self.property_values
            .push(unsafe { RiveRc::from_raw(value.as_ptr()) });
    }

    pub fn remove_value(&mut self, property_id: u32) -> bool {
        let Some(index) = self
            .property_values
            .iter()
            .position(|value| value.base.view_model_property_id() == property_id)
        else {
            return false;
        };
        let value = &self.property_values[index];
        if let Some(mut nested) = value.base.as_view_model_instance_viewmodel() {
            if let Some(mut referenced) = unsafe { nested.as_ref() }.reference_view_model_instance()
            {
                unsafe { referenced.as_mut() }.remove_parent(NonNull::from(&mut *self));
            }
        }
        let raw = value.as_ptr();
        self.property_symbols
            .retain(|_, stored| !std::ptr::eq(stored.as_ptr(), raw));
        self.property_values.remove(index);
        true
    }

    pub fn property_value_by_id(&self, id: u32) -> Option<NonNull<ViewModelInstanceValue>> {
        self.property_values
            .iter()
            .find(|value| value.base.view_model_property_id() == id)
            .and_then(|value| NonNull::new(value.as_ptr()))
    }

    pub fn property_value_named(&self, name: &str) -> Option<NonNull<ViewModelInstanceValue>> {
        self.property_values.iter().find_map(|value| {
            let property = value.view_model_property()?;
            (unsafe { property.as_ref() }.base.name() == name)
                .then(|| NonNull::new(value.as_ptr()).unwrap())
        })
    }

    pub fn property_value_for_symbol(
        &self,
        symbol_type: SymbolType,
    ) -> Option<NonNull<ViewModelInstanceValue>> {
        self.property_symbols.get(&symbol_type).copied()
    }

    pub fn set_property_symbol(
        &mut self,
        symbol_type: SymbolType,
        value: NonNull<ViewModelInstanceValue>,
    ) {
        if symbol_type != SymbolType::None {
            self.property_symbols.insert(symbol_type, value);
        }
    }

    pub fn replace_view_model_by_name(
        &mut self,
        name: &str,
        value: RiveRc<ViewModelInstance>,
    ) -> bool {
        let view_model = self
            .view_model
            .expect("replaceViewModelByName requires the instance's ViewModel");
        let Some(property) = (unsafe { view_model.as_ref() }).property_named(name) else {
            return false;
        };
        for property_value in &mut self.property_values {
            if property_value.view_model_property() != Some(property) {
                continue;
            }
            let Some(mut nested) = property_value.base.as_view_model_instance_viewmodel() else {
                break;
            };
            if value.base.view_model_id()
                != unsafe { property.as_ref() }.base.view_model_reference_id()
            {
                break;
            }
            let previous = unsafe { nested.as_ref() }.reference_view_model_instance();
            unsafe { nested.as_mut() }.set_reference_view_model_instance(Some(value));
            let snapshot = property_value.dependents().to_vec();
            for mut dependent in snapshot {
                unsafe { dependent.as_mut() }.relink_data_bind();
            }
            self.rebind_dependents();
            if let Some(mut previous) = previous {
                unsafe { previous.as_mut() }.rebind_properties();
            }
            return true;
        }
        false
    }

    pub fn replace_view_model_by_property(
        &mut self,
        property: NonNull<ViewModelInstanceViewModel>,
        value: RiveRc<ViewModelInstance>,
    ) -> bool {
        for property_value in &mut self.property_values {
            if property_value.as_ptr().cast() != property.as_ptr() {
                continue;
            }
            let mut nested = property;
            let previous = unsafe { nested.as_ref() }.reference_view_model_instance();
            unsafe { nested.as_mut() }.set_reference_view_model_instance(Some(value));
            let snapshot = property_value.dependents().to_vec();
            for mut dependent in snapshot {
                unsafe { dependent.as_mut() }.relink_data_bind();
            }
            self.rebind_dependents();
            if let Some(mut previous) = previous {
                unsafe { previous.as_mut() }.rebind_properties();
            }
            return true;
        }
        false
    }

    pub fn property_values(&self) -> &[RiveRc<ViewModelInstanceValue>] {
        &self.property_values
    }

    pub fn property_from_path(
        &self,
        path: &[u32],
        index: usize,
    ) -> Option<NonNull<ViewModelInstanceValue>> {
        let property = self.property_value_by_id(*path.get(index)?)?;
        if index == path.len() - 1 {
            return Some(property);
        }
        let nested = unsafe { property.as_ref() }
            .base
            .as_view_model_instance_viewmodel()?;
        let instance = unsafe { nested.as_ref() }
            .reference_view_model_instance()
            .expect("a nested property path requires a referenced ViewModelInstance");
        unsafe { instance.as_ref() }.property_from_path(path, index + 1)
    }

    pub fn view_model(&mut self, mut value: NonNull<ViewModel>) {
        if let Some(mut old) = self.view_model {
            unsafe { old.as_mut() }.base.unref();
        }
        unsafe { value.as_mut() }.base.ref_();
        self.view_model = Some(value);
    }

    pub fn get_view_model(&self) -> Option<NonNull<ViewModel>> {
        self.view_model
    }

    pub fn on_component_dirty(&mut self, _component: NonNull<Component>) {}

    pub fn set_as_root(&mut self, instance: RiveRc<ViewModelInstance>) {
        self.set_root(instance);
    }

    pub fn set_root(&mut self, value: RiveRc<ViewModelInstance>) {
        for property in &mut self.property_values {
            property.set_root(value.clone());
        }
    }

    pub fn clone_instance(&self) -> Box<ViewModelInstance> {
        let mut cloned = Box::new(Self {
            base: self.base.clone_base(),
            property_values: Vec::new(),
            parents: Vec::new(),
            dependents: Vec::new(),
            property_symbols: HashMap::new(),
            view_model: None,
        });
        if self.base.artboard().is_none() {
            for property in &self.property_values {
                cloned.add_value(property.clone_core_value());
            }
        }
        if let Some(view_model) = self.view_model {
            cloned.view_model(view_model);
        }
        cloned
    }

    pub fn import(&mut self, import_stack: &mut ImportStack) -> StatusCode {
        let Some(importer) = import_stack.latest::<BackboardImporter>(
            crate::mechanical_port::source::backboard::Backboard::TYPE_KEY,
        ) else {
            return StatusCode::MissingObject;
        };
        importer.add_view_model_instance(NonNull::from(&mut *self));
        if import_stack
            .latest::<ArtboardImporter>(
                crate::mechanical_port::source::generated::artboard_base::ArtboardBase::TYPE_KEY,
            )
            .is_some()
        {
            return self.base.import(import_stack);
        }
        if let Some(mut file) = importer.file() {
            unsafe { file.as_mut() }.add_file_view_model_instance(NonNull::from(&mut *self));
        }
        StatusCode::Ok
    }

    pub fn advanced(&mut self) {
        for value in &mut self.property_values {
            value.advanced();
        }
    }

    pub fn add_parent(&mut self, parent: NonNull<ViewModelInstance>) {
        if !self.parents.contains(&parent) {
            self.parents.push(parent);
        }
    }

    pub fn remove_parent(&mut self, parent: NonNull<ViewModelInstance>) {
        self.parents.retain(|candidate| *candidate != parent);
    }

    pub fn has_parents(&self) -> bool {
        !self.parents.is_empty()
    }

    pub fn add_dependent(&mut self, dependent: NonNull<DataBindContainer>) {
        if !self.dependents.contains(&dependent) {
            self.dependents.push(dependent);
        }
    }

    pub fn remove_dependent(&mut self, dependent: NonNull<DataBindContainer>) {
        self.dependents.retain(|candidate| *candidate != dependent);
    }

    #[cfg(feature = "testing")]
    pub fn dependents(&self) -> Vec<NonNull<DataBindContainer>> {
        self.dependents.clone()
    }

    #[cfg(feature = "testing")]
    pub fn parents(&self) -> Vec<NonNull<ViewModelInstance>> {
        self.parents.clone()
    }

    fn rebind_properties(&mut self) {
        for property in &mut self.property_values {
            let snapshot = property.dependents().to_vec();
            for mut dependent in snapshot {
                unsafe { dependent.as_mut() }.relink_data_bind();
            }
            if let Some(nested) = property.base.as_view_model_instance_viewmodel() {
                if let Some(mut instance) =
                    unsafe { nested.as_ref() }.reference_view_model_instance()
                {
                    unsafe { instance.as_mut() }.rebind_properties();
                }
            }
        }
    }

    fn rebind_dependents(&mut self) {
        for dependent in &mut self.dependents {
            unsafe { dependent.as_mut() }.relink_data_context();
        }
        for parent in self.parents.clone() {
            unsafe { parent.as_ptr().as_mut().unwrap() }.rebind_dependents();
        }
    }
}

impl Drop for ViewModelInstance {
    fn drop(&mut self) {
        let self_pointer = NonNull::from(&mut *self);
        for value in &mut self.property_values {
            if let Some(nested) = value.base.as_view_model_instance_viewmodel() {
                if let Some(mut instance) =
                    unsafe { nested.as_ref() }.reference_view_model_instance()
                {
                    unsafe { instance.as_mut() }.remove_parent(self_pointer);
                }
            }
        }
        self.property_values.clear();
        if let Some(mut view_model) = self.view_model {
            unsafe { view_model.as_mut() }.base.unref();
        }
    }
}
