use crate::mechanical_port::source::{
    component_dirt::ComponentDirt,
    core::CoreHandle,
    data_bind::data_values::data_value_viewmodel::DataValueViewModel,
    generated::viewmodel::viewmodel_instance_viewmodel_base::ViewModelInstanceViewModelBase,
    importers::{
        artboard_importer::ArtboardImporter, backboard_importer::BackboardImporter,
        import_stack::ImportStack, viewmodel_instance_importer::ViewModelInstanceImporter,
    },
    status_code::StatusCode,
};

use super::viewmodel_instance::ViewModelInstance;

#[derive(Default)]
pub struct ViewModelInstanceViewModel {
    pub base: ViewModelInstanceViewModelBase,
    reference_view_model_instance: Option<CoreHandle>,
    parent_view_model_instance: Option<CoreHandle>,
    #[cfg(feature = "tools")]
    changed_callback: Option<fn(&mut Self)>,
}

impl ViewModelInstanceViewModel {
    pub fn set_reference_view_model_instance(&mut self, value: Option<CoreHandle>) {
        if let (Some(instance), Some(parent)) = (
            self.reference_view_model_instance.as_ref(),
            self.parent_view_model_instance.as_ref(),
        ) {
            instance.with_mut(|instance| {
                if let Some(instance) = instance.as_view_model_instance_mut() {
                    instance.remove_parent(parent);
                }
            });
        }
        self.reference_view_model_instance = value;
        if let (Some(instance), Some(parent)) = (
            self.reference_view_model_instance.as_ref(),
            self.parent_view_model_instance.as_ref(),
        ) {
            instance.with_mut(|instance| {
                if let Some(instance) = instance.as_view_model_instance_mut() {
                    instance.add_parent(parent.clone());
                }
            });
        }
        self.property_value_changed();
    }

    pub fn reference_view_model_instance(&self) -> Option<CoreHandle> {
        self.reference_view_model_instance.clone()
    }

    pub fn set_parent_view_model_instance(&mut self, parent: Option<CoreHandle>) {
        self.parent_view_model_instance = parent;
    }

    pub fn parent_view_model_instance(&self) -> Option<CoreHandle> {
        self.parent_view_model_instance.clone()
    }

    pub fn property_value_changed(&mut self) {
        self.base.add_dirt(ComponentDirt::BINDINGS);
        #[cfg(feature = "tools")]
        if let Some(callback) = self.changed_callback {
            callback(self);
        }
        self.base.on_value_changed();
    }

    pub fn set_root(&mut self, value: CoreHandle) {
        self.base.set_root(value.clone());
        if let Some(instance) = &self.reference_view_model_instance {
            instance.with_mut(|instance| {
                if let Some(instance) = instance.as_view_model_instance_mut() {
                    instance.set_root(value);
                }
            });
        }
    }

    pub fn advanced(&mut self) {
        if let Some(instance) = &self.reference_view_model_instance {
            instance.with_mut(|instance| {
                if let Some(instance) = instance.as_view_model_instance_mut() {
                    instance.advanced();
                }
            });
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
            if let Some(file) = backboard.file() {
                let instance_importer = import_stack.latest::<ViewModelInstanceImporter>(
                    crate::mechanical_port::source::generated::viewmodel::viewmodel_instance_base::ViewModelInstanceBase::TYPE_KEY,
                );
                let instance_importer = instance_importer.expect(
                    "artboard ViewModelInstanceViewModel import requires its instance importer",
                );
                let instance = instance_importer.view_model_instance();
                let view_model_id = instance
                    .with(|instance| {
                        instance
                            .as_view_model_instance()
                            .map(|instance| instance.base.view_model_id())
                    })
                    .flatten();
                let referenced_instance = view_model_id.and_then(|view_model_id| {
                    file.with_file_mut(|file| {
                        let view_model = file.view_model(view_model_id as usize)?;
                        let property = view_model
                            .with(|view_model| {
                                view_model.as_view_model().and_then(|view_model| {
                                    view_model
                                        .property_at(self.base.view_model_property_id() as usize)
                                })
                            })
                            .flatten()?;
                        let reference_id = property
                            .with(|property| {
                                property
                                    .as_view_model_property()
                                    .and_then(|property| property.base.as_view_model_reference_id())
                            })
                            .flatten()?;
                        let referenced_view_model = file.view_model(reference_id as usize)?;
                        referenced_view_model
                            .with(|view_model| {
                                view_model.as_view_model().and_then(|view_model| {
                                    view_model.instance_at(self.base.property_value() as usize)
                                })
                            })
                            .flatten()
                    })
                    .flatten()
                });
                if let Some(referenced_instance) = referenced_instance {
                    self.set_reference_view_model_instance(Some(referenced_instance));
                }
            }
        }
        status
    }

    pub fn update_view_model(&mut self, value: CoreHandle) {
        let Some(property) = self.base.base.base.base.base.handle() else {
            return;
        };
        let Some(owner) = self.base.view_model_instance() else {
            return;
        };
        owner.with_mut(|owner| {
            if let Some(owner) = owner.as_view_model_instance_mut() {
                owner.replace_view_model_property_handle(property, value);
            }
        });
    }

    pub fn update_view_model_occurrence(property: &CoreHandle, value: Option<CoreHandle>) -> bool {
        let Some(owner) = property
            .with(|property| {
                property
                    .as_view_model_instance_value()?
                    .view_model_instance()
            })
            .flatten()
        else {
            return false;
        };
        ViewModelInstance::replace_view_model_property_occurrence(&owner, property, value)
    }

    pub fn apply_value(&mut self, data_value: &DataValueViewModel) {
        if let Some(value) = data_value.value() {
            self.update_view_model(value);
        }
    }

    pub fn clone_definition(&self) -> Self {
        let mut clone = Self::default();
        let mut base = std::mem::take(&mut clone.base);
        base.copy(&self.base, &mut clone);
        clone.base = base;
        clone
    }

    pub fn complete_clone(source: &CoreHandle, cloned: &CoreHandle) -> bool {
        let Some((reference, owner)) = source.with_downcast::<Self, _>(|source| {
            (
                source.reference_view_model_instance.clone(),
                source.base.view_model_instance(),
            )
        }) else {
            return false;
        };
        if let Some(instance) = reference {
            let Some(cloned_instance) = ViewModelInstance::clone_instance(&instance) else {
                return false;
            };
            return cloned
                .with_downcast_mut::<Self, _>(|cloned| {
                    cloned.set_reference_view_model_instance(Some(cloned_instance));
                    if let Some(owner) = owner {
                        cloned.base.set_view_model_instance(owner);
                    }
                })
                .is_some();
        }
        true
    }

    pub fn clone_value(source: &CoreHandle) -> Option<CoreHandle> {
        source.with_downcast::<Self, _>(|_| ())?;
        source.clone_occurrence()
    }

    #[cfg(feature = "tools")]
    pub fn on_changed(&mut self, callback: Option<fn(&mut Self)>) {
        self.changed_callback = callback;
    }
}
