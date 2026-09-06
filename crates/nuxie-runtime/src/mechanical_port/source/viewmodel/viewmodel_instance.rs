use std::{
    collections::{HashMap, hash_map::DefaultHasher},
    hash::{Hash, Hasher},
};

use crate::mechanical_port::source::{
    animation::state_machine_instance::RuntimeStateMachineInstanceWeakHandle,
    core::CoreHandle,
    generated::viewmodel::viewmodel_instance_base::ViewModelInstanceBase,
    importers::{
        artboard_importer::ArtboardImporter, backboard_importer::BackboardImporter,
        import_stack::ImportStack,
    },
    status_code::StatusCode,
};

use super::symbol_type::SymbolType;

#[derive(Clone)]
pub enum DataBindContainerDependent {
    Authored(CoreHandle),
    StateMachine(RuntimeStateMachineInstanceWeakHandle),
}

impl DataBindContainerDependent {
    pub(crate) fn same_identity(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Authored(left), Self::Authored(right)) => left == right,
            (Self::StateMachine(left), Self::StateMachine(right)) => left.ptr_eq(right),
            _ => false,
        }
    }

    pub(crate) fn relink_data_context(&self) {
        let dependent = self.clone();
        if crate::view_model_cell::defer_transaction_notification(move || {
            dependent.relink_data_context()
        }) {
            return;
        }
        match self {
            Self::Authored(dependent) => {
                if dependent.artboard_dirty_handle().is_some() {
                    crate::mechanical_port::source::artboard::Artboard::relink_data_context_handle(
                        dependent,
                    );
                }
            }
            Self::StateMachine(dependent) => {
                dependent.relink_data_context();
            }
        }
    }
}

#[derive(Default)]
pub struct ViewModelInstance {
    pub base: ViewModelInstanceBase,
    property_values: Vec<CoreHandle>,
    // One entry per reference; parents() exposes unique owners.
    parents: Vec<CoreHandle>,
    dependents: Vec<DataBindContainerDependent>,
    property_symbols: HashMap<SymbolType, CoreHandle>,
    view_model: Option<CoreHandle>,
}

impl ViewModelInstance {
    pub(crate) fn handle(&self) -> Option<CoreHandle> {
        self.base.base.base.base.base.base.handle()
    }

    pub fn pointer_key(instance: Option<&CoreHandle>) -> u32 {
        let Some(instance) = instance else {
            return u32::MAX;
        };
        let mut hasher = DefaultHasher::new();
        instance.hash(&mut hasher);
        let value = hasher.finish();
        (value ^ (value >> 32)) as u32
    }

    pub fn add_value(&mut self, value: CoreHandle) {
        if self.property_values.contains(&value) {
            return;
        }
        value
            .with_mut(|value| {
                self.add_value_borrowed(
                    value
                        .as_view_model_instance_value_mut()
                        .expect("ViewModelInstance values derive from ViewModelInstanceValue"),
                );
            })
            .expect("ViewModelInstance values are arena-owned");
    }

    pub(crate) fn add_value_borrowed(
        &mut self,
        value: &mut super::viewmodel_instance_value::ViewModelInstanceValue,
    ) {
        let handle = value
            .handle()
            .expect("ViewModelInstanceValue is arena-owned");
        if self.property_values.contains(&handle) {
            return;
        }
        value.set_view_model_instance_borrowed(self);
        let symbol = value.view_model_property().and_then(|property| {
            property
                .with(|property| {
                    let property = property.as_view_model_property()?;
                    SymbolType::from_i32(property.base.symbol_type_value() as i32)
                })
                .flatten()
        });
        if let Some(symbol) = symbol.filter(|symbol| *symbol != SymbolType::None) {
            self.set_property_symbol(symbol, handle.clone());
        }
        self.property_values.push(handle);
    }

    pub fn remove_value(&mut self, property_id: u32) -> bool {
        let Some(index) = self.property_values.iter().position(|value| {
            value
                .with(|value| {
                    value
                        .as_view_model_instance_value()
                        .is_some_and(|value| value.base.view_model_property_id() == property_id)
                })
                .unwrap_or(false)
        }) else {
            return false;
        };
        let value = self.property_values[index].clone();
        if let Some(referenced) = value
            .with(|value| {
                value
                    .as_view_model_instance_view_model()
                    .and_then(|value| value.reference_view_model_instance())
            })
            .flatten()
            && let Some(this) = self.handle()
        {
            referenced.with_mut(|referenced| {
                if let Some(referenced) = referenced.as_view_model_instance_mut() {
                    referenced.remove_parent(&this);
                }
            });
        }
        self.property_symbols.retain(|_, stored| stored != &value);
        self.property_values.remove(index);
        true
    }

    pub fn property_value_by_id(&self, id: u32) -> Option<CoreHandle> {
        self.property_values.iter().find_map(|value| {
            value
                .with(|value| {
                    value
                        .as_view_model_instance_value()
                        .is_some_and(|value| value.base.view_model_property_id() == id)
                })
                .unwrap_or(false)
                .then(|| value.clone())
        })
    }

    pub fn property_value_named(&self, name: &str) -> Option<CoreHandle> {
        self.property_values.iter().find_map(|value| {
            let property = value
                .with(|value| {
                    value
                        .as_view_model_instance_value()
                        .and_then(|value| value.view_model_property())
                })
                .flatten()?;
            let matches = property
                .with(|property| {
                    property
                        .as_view_model_property()
                        .is_some_and(|property| property.base.name() == name)
                })
                .unwrap_or(false);
            matches.then(|| value.clone())
        })
    }

    pub fn property_value_for_symbol(&self, symbol_type: SymbolType) -> Option<CoreHandle> {
        self.property_symbols.get(&symbol_type).cloned()
    }

    pub fn set_property_symbol(&mut self, symbol_type: SymbolType, value: CoreHandle) {
        if symbol_type != SymbolType::None {
            self.property_symbols.insert(symbol_type, value);
        }
    }

    pub fn replace_view_model_by_name(owner: &CoreHandle, name: &str, value: CoreHandle) -> bool {
        let Some((view_model, property_values)) = owner
            .with_downcast::<Self, _>(|owner| {
                Some((owner.view_model.clone()?, owner.property_values.clone()))
            })
            .flatten()
        else {
            return false;
        };
        let property = view_model
            .with(|view_model| {
                view_model
                    .as_view_model()
                    .and_then(|view_model| view_model.property_named(name))
            })
            .flatten();
        let Some(property) = property else {
            return false;
        };
        for property_value in &property_values {
            let matches = property_value
                .with(|value| {
                    value
                        .as_view_model_instance_value()
                        .and_then(|value| value.view_model_property())
                        == Some(property.clone())
                })
                .unwrap_or(false);
            if !matches {
                continue;
            }
            let required_id = property.with_downcast::<super::viewmodel_property_viewmodel::ViewModelPropertyViewModel, _>(
                |property| property.base.view_model_reference_id(),
            );
            if required_id
                != value
                    .with(|value| {
                        value
                            .as_view_model_instance()
                            .map(|value| value.base.view_model_id())
                    })
                    .flatten()
            {
                break;
            }
            if Self::replace_view_model_property_occurrence(owner, property_value, Some(value)) {
                return true;
            }
            break;
        }
        false
    }

    pub fn replace_view_model_property_handle(
        &mut self,
        property: CoreHandle,
        value: CoreHandle,
    ) -> bool {
        if !self.property_values.contains(&property) {
            return false;
        }
        let previous = property
            .with(|property| {
                property
                    .as_view_model_instance_view_model()
                    .and_then(|property| property.reference_view_model_instance())
            })
            .flatten();
        property.with_mut(|property| {
            if let Some(property) = property.as_view_model_instance_view_model_mut() {
                property.set_reference_view_model_instance(Some(value));
            }
        });
        property.with_mut(|property| {
            if let Some(property) = property.as_view_model_instance_value_mut() {
                property.relink_dependents();
            }
        });
        self.rebind_dependents();
        if let Some(previous) = previous {
            previous.with_mut(|previous| {
                if let Some(previous) = previous.as_view_model_instance_mut() {
                    previous.rebind_properties();
                }
            });
        }
        true
    }

    pub fn property_values(&self) -> &[CoreHandle] {
        &self.property_values
    }

    pub fn replace_view_model_property_occurrence(
        owner: &CoreHandle,
        property: &CoreHandle,
        value: Option<CoreHandle>,
    ) -> bool {
        if !owner
            .with_downcast::<Self, _>(|owner| owner.property_values.contains(property))
            .unwrap_or(false)
        {
            return false;
        }
        let previous = property
            .with(|property| {
                property
                    .as_view_model_instance_view_model()?
                    .reference_view_model_instance()
            })
            .flatten();
        let notifications = crate::view_model_cell::RuntimeHostMutationNotifications::begin();
        property.with_mut(|property| {
            property
                .as_view_model_instance_view_model_mut()
                .expect("ViewModel property")
                .set_reference_view_model_instance(value)
        });
        if let Some(notifications) = notifications {
            notifications.commit();
        }
        let dependents = property
            .with(|property| {
                property
                    .as_view_model_instance_value()
                    .expect("ViewModel value")
                    .dependents()
            })
            .expect("retained property");
        for dependent in dependents {
            dependent.relink();
        }
        Self::rebind_dependents_occurrence(owner);
        if let Some(previous) = previous {
            Self::rebind_properties_occurrence(&previous);
        }
        true
    }

    pub fn rebind_dependents_occurrence(owner: &CoreHandle) {
        let (dependents, parents) = owner
            .with_downcast::<Self, _>(|owner| (owner.dependents.clone(), owner.parents()))
            .expect("ViewModel occurrence");
        for dependent in dependents {
            dependent.relink_data_context();
        }
        for parent in parents {
            Self::rebind_dependents_occurrence(&parent);
        }
    }

    pub fn rebind_properties_occurrence(owner: &CoreHandle) {
        let properties = owner
            .with_downcast::<Self, _>(|owner| owner.property_values.clone())
            .expect("ViewModel occurrence");
        for property in properties {
            let dependents = property
                .with(|property| {
                    property
                        .as_view_model_instance_value()
                        .expect("ViewModel value")
                        .dependents()
                })
                .expect("retained property");
            for dependent in dependents {
                dependent.relink();
            }
            let nested = property
                .with(|property| {
                    property
                        .as_view_model_instance_view_model()?
                        .reference_view_model_instance()
                })
                .flatten();
            if let Some(nested) = nested {
                Self::rebind_properties_occurrence(&nested);
            }
        }
    }

    pub fn property_from_path(&self, path: &[u32], index: usize) -> Option<CoreHandle> {
        let property = self.property_value_by_id(*path.get(index)?)?;
        if index == path.len() - 1 {
            return Some(property);
        }
        let instance = property
            .with(|property| {
                property
                    .as_view_model_instance_view_model()
                    .and_then(|property| property.reference_view_model_instance())
            })
            .flatten()?;
        instance
            .with(|instance| {
                instance
                    .as_view_model_instance()
                    .and_then(|instance| instance.property_from_path(path, index + 1))
            })
            .flatten()
    }

    pub fn view_model(&mut self, value: CoreHandle) {
        self.view_model = Some(value);
    }

    pub fn get_view_model(&self) -> Option<CoreHandle> {
        self.view_model.clone()
    }

    pub fn set_as_root(&mut self, instance: CoreHandle) {
        self.set_root(instance);
    }

    pub fn set_root(&mut self, value: CoreHandle) {
        for property in &self.property_values {
            property.with_mut(|property| {
                if let Some(property) = property.as_view_model_instance_value_mut() {
                    property.set_root(value.clone());
                }
            });
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
        let Some((copy_values, properties, view_model)) =
            source.with_downcast::<Self, _>(|source| {
                (
                    source.base.base.base.base.artboard_handle().is_none(),
                    source.property_values.clone(),
                    source.view_model.clone(),
                )
            })
        else {
            return false;
        };
        // Artboard-owned values are cloned by the artboard's object traversal.
        if copy_values {
            for property in properties {
                let Some(property) = property.clone_occurrence() else {
                    return false;
                };
                if cloned
                    .with_downcast_mut::<Self, _>(|cloned| cloned.add_value(property))
                    .is_none()
                {
                    return false;
                }
            }
        }
        cloned
            .with_downcast_mut::<Self, _>(|cloned| {
                if let Some(view_model) = view_model {
                    cloned.view_model(view_model);
                }
            })
            .is_some()
    }

    pub fn clone_instance(source: &CoreHandle) -> Option<CoreHandle> {
        source.with_downcast::<Self, _>(|_| ())?;
        source.clone_occurrence()
    }

    pub fn import(&mut self, import_stack: &mut ImportStack) -> StatusCode {
        let Some(instance) = self.handle() else {
            return StatusCode::MissingObject;
        };
        let Some(importer) = import_stack.latest::<BackboardImporter>(
            crate::mechanical_port::source::generated::backboard_base::BackboardBase::TYPE_KEY,
        ) else {
            return StatusCode::MissingObject;
        };
        importer.add_view_model_instance(self);
        if import_stack
            .latest::<ArtboardImporter>(
                crate::mechanical_port::source::generated::artboard_base::ArtboardBase::TYPE_KEY,
            )
            .is_some()
        {
            return self.base.import(import_stack);
        }
        import_stack
            .latest::<BackboardImporter>(
                crate::mechanical_port::source::generated::backboard_base::BackboardBase::TYPE_KEY,
            )
            .expect("the BackboardImporter remains on the import stack")
            .add_file_view_model_instance(instance);
        StatusCode::Ok
    }

    pub fn advanced(&mut self) {
        for value in &self.property_values {
            value.with_mut(|value| {
                assert!(
                    value.view_model_instance_value_advanced(),
                    "ViewModel property value advance capability"
                );
            });
        }
    }

    pub fn add_parent(&mut self, parent: CoreHandle) {
        self.parents.push(parent);
    }

    pub fn remove_parent(&mut self, parent: &CoreHandle) {
        if let Some(index) = self.parents.iter().position(|candidate| candidate == parent) {
            self.parents.remove(index);
        }
    }

    pub fn has_parents(&self) -> bool {
        !self.parents.is_empty()
    }

    pub fn add_dependent(&mut self, dependent: CoreHandle) {
        self.add_dependent_occurrence(DataBindContainerDependent::Authored(dependent));
    }

    pub fn remove_dependent(&mut self, dependent: &CoreHandle) {
        self.remove_dependent_occurrence(&DataBindContainerDependent::Authored(dependent.clone()));
    }

    pub fn add_state_machine_dependent(
        &mut self,
        dependent: RuntimeStateMachineInstanceWeakHandle,
    ) {
        self.add_dependent_occurrence(DataBindContainerDependent::StateMachine(dependent));
    }

    pub fn remove_state_machine_dependent(
        &mut self,
        dependent: &RuntimeStateMachineInstanceWeakHandle,
    ) {
        self.remove_dependent_occurrence(&DataBindContainerDependent::StateMachine(
            dependent.clone(),
        ));
    }

    fn add_dependent_occurrence(&mut self, dependent: DataBindContainerDependent) {
        if !self
            .dependents
            .iter()
            .any(|candidate| candidate.same_identity(&dependent))
        {
            self.dependents.push(dependent);
        }
    }

    fn remove_dependent_occurrence(&mut self, dependent: &DataBindContainerDependent) {
        self.dependents
            .retain(|candidate| !candidate.same_identity(dependent));
    }

    #[cfg(any(test, feature = "tools"))]
    pub fn dependents(&self) -> Vec<CoreHandle> {
        self.dependents
            .iter()
            .filter_map(|dependent| match dependent {
                DataBindContainerDependent::Authored(dependent) => Some(dependent.clone()),
                DataBindContainerDependent::StateMachine(_) => None,
            })
            .collect()
    }

    pub fn parents(&self) -> Vec<CoreHandle> {
        let mut owners = Vec::new();
        for parent in &self.parents {
            if !owners.contains(parent) {
                owners.push(parent.clone());
            }
        }
        owners
    }

    pub fn rebind_properties(&mut self) {
        for property in &self.property_values {
            property.with_mut(|property| {
                if let Some(property) = property.as_view_model_instance_value_mut() {
                    property.relink_dependents();
                }
            });
            let nested = property
                .with(|property| {
                    property
                        .as_view_model_instance_view_model()
                        .and_then(|property| property.reference_view_model_instance())
                })
                .flatten();
            if let Some(nested) = nested {
                nested.with_mut(|nested| {
                    if let Some(nested) = nested.as_view_model_instance_mut() {
                        nested.rebind_properties();
                    }
                });
            }
        }
    }

    fn rebind_dependents(&mut self) {
        for dependent in &self.dependents {
            dependent.relink_data_context();
        }
        for parent in self.parents() {
            parent.with_mut(|parent| {
                if let Some(parent) = parent.as_view_model_instance_mut() {
                    parent.rebind_dependents();
                }
            });
        }
    }
}

impl Drop for ViewModelInstance {
    fn drop(&mut self) {
        let this = self.handle();
        for value in &self.property_values {
            let nested = value
                .with(|value| {
                    value
                        .as_view_model_instance_view_model()
                        .and_then(|value| value.reference_view_model_instance())
                })
                .flatten();
            if let (Some(nested), Some(this)) = (nested, this.as_ref()) {
                nested.with_mut(|nested| {
                    if let Some(nested) = nested.as_view_model_instance_mut() {
                        nested.remove_parent(this);
                    }
                });
            }
        }
        self.property_values.clear();
        self.view_model = None;
    }
}
