use std::ptr::NonNull;

use crate::mechanical_port::source::{
    component_dirt::ComponentDirt,
    data_bind::data_values::data_value_viewmodel::DataValueViewModel,
    generated::viewmodel::viewmodel_instance_viewmodel_base::ViewModelInstanceViewModelBase,
    importers::{
        artboard_importer::ArtboardImporter, backboard_importer::BackboardImporter,
        import_stack::ImportStack, viewmodel_instance_importer::ViewModelInstanceImporter,
    },
    refcnt::RiveRc,
    status_code::StatusCode,
};

use super::viewmodel_instance::ViewModelInstance;

pub struct ViewModelInstanceViewModel {
    pub base: ViewModelInstanceViewModelBase,
    reference_view_model_instance: Option<RiveRc<ViewModelInstance>>,
    parent_view_model_instance: Option<NonNull<ViewModelInstance>>,
    #[cfg(feature = "rive_tools")]
    changed_callback: Option<fn(&mut Self)>,
}

impl ViewModelInstanceViewModel {
    pub fn set_reference_view_model_instance(&mut self, value: Option<RiveRc<ViewModelInstance>>) {
        if let (Some(instance), Some(parent)) = (
            &mut self.reference_view_model_instance,
            self.parent_view_model_instance,
        ) {
            instance.remove_parent(parent);
        }
        self.reference_view_model_instance = value;
        if let (Some(instance), Some(parent)) = (
            &mut self.reference_view_model_instance,
            self.parent_view_model_instance,
        ) {
            instance.add_parent(parent);
        }
        self.property_value_changed();
    }

    pub fn reference_view_model_instance(&self) -> Option<NonNull<ViewModelInstance>> {
        self.reference_view_model_instance
            .as_ref()
            .and_then(|instance| NonNull::new(instance.as_ptr()))
    }

    pub fn set_parent_view_model_instance(&mut self, parent: Option<NonNull<ViewModelInstance>>) {
        self.parent_view_model_instance = parent;
    }

    pub fn parent_view_model_instance(&self) -> Option<NonNull<ViewModelInstance>> {
        self.parent_view_model_instance
    }

    pub fn property_value_changed(&mut self) {
        self.base.add_dirt(ComponentDirt::BINDINGS);
        #[cfg(feature = "rive_tools")]
        if let Some(callback) = self.changed_callback {
            callback(self);
        }
        self.base.on_value_changed();
    }

    pub fn set_root(&mut self, value: RiveRc<ViewModelInstance>) {
        self.base.set_root(value.clone());
        if let Some(instance) = &mut self.reference_view_model_instance {
            instance.set_root(value);
        }
    }

    pub fn advanced(&mut self) {
        if let Some(instance) = &mut self.reference_view_model_instance {
            instance.advanced();
        }
    }

    pub fn import(&mut self, import_stack: &mut ImportStack) -> StatusCode {
        let status = self.base.import(import_stack);
        if import_stack
            .latest::<ArtboardImporter>(
                crate::mechanical_port::source::generated::artboard_base::ArtboardBase::TYPE_KEY,
            )
            .is_some()
        {
            let Some(backboard) = import_stack.latest::<BackboardImporter>(
                crate::mechanical_port::source::backboard::Backboard::TYPE_KEY,
            ) else {
                return StatusCode::MissingObject;
            };
            if let Some(mut file) = backboard.file() {
                let instance_importer = import_stack.latest::<ViewModelInstanceImporter>(
                    crate::mechanical_port::source::generated::viewmodel::viewmodel_instance_base::ViewModelInstanceBase::TYPE_KEY,
                );
                let instance_importer = instance_importer.expect(
                    "artboard ViewModelInstanceViewModel import requires its instance importer",
                );
                let instance = instance_importer.view_model_instance();
                if let Some(view_model) = unsafe { file.as_mut() }
                    .view_model(unsafe { instance.as_ref() }.base.view_model_id() as usize)
                {
                    if let Some(property) = unsafe { view_model.as_ref() }
                        .property_at(self.base.view_model_property_id() as usize)
                    {
                        if let Some(reference_id) = unsafe { property.as_ref() }
                            .base
                            .as_view_model_reference_id()
                        {
                            if let Some(referenced_view_model) =
                                unsafe { file.as_mut() }.view_model(reference_id as usize)
                            {
                                if let Some(referenced_instance) =
                                    unsafe { referenced_view_model.as_ref() }
                                        .instance_at(self.base.property_value() as usize)
                                {
                                    unsafe { referenced_instance.as_ref() }.base.ref_();
                                    self.set_reference_view_model_instance(Some(unsafe {
                                        RiveRc::from_raw(referenced_instance.as_ptr())
                                    }));
                                }
                            }
                        }
                    }
                }
            }
        }
        status
    }

    pub fn update_view_model(&mut self, value: NonNull<ViewModelInstance>) {
        unsafe { value.as_ref() }.base.ref_();
        let instance = unsafe { RiveRc::from_raw(value.as_ptr()) };
        let mut owner = self.base.view_model_instance().unwrap();
        unsafe { owner.as_mut() }
            .replace_view_model_by_property(NonNull::from(&mut *self), instance);
    }

    pub fn apply_value(&mut self, data_value: &DataValueViewModel) {
        self.update_view_model(data_value.value());
    }

    pub fn clone_value(&self) -> Box<Self> {
        let mut cloned = self.base.clone_view_model_instance_viewmodel();
        if let Some(instance) = &self.reference_view_model_instance {
            let cloned_instance = instance.clone_instance();
            let cloned_instance = unsafe { RiveRc::from_box(cloned_instance) };
            cloned.set_reference_view_model_instance(Some(cloned_instance));
            cloned
                .base
                .set_view_model_instance(self.base.view_model_instance());
        }
        cloned
    }

    #[cfg(feature = "rive_tools")]
    pub fn on_changed(&mut self, callback: Option<fn(&mut Self)>) {
        self.changed_callback = callback;
    }
}
