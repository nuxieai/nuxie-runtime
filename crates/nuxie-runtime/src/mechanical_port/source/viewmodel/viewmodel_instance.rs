use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use crate::mechanical_port::source::{
    animation::state_machine_instance::RuntimeStateMachineInstanceWeakHandle,
    core::CoreHandle,
    data_bind::data_bind::{DataBind, SOURCE_TO_TARGET_FIRST},
    data_bind::data_bind_container::DataBindContainerOwner,
    generated::viewmodel::viewmodel_instance_base::ViewModelInstanceBase,
    importers::{
        artboard_importer::ArtboardImporter, backboard_importer::BackboardImporter,
        import_stack::ImportStack,
    },
    lazy_vector::LazyVector,
    status_code::StatusCode,
};

use super::symbol_type::SymbolType;

#[derive(Clone)]
pub enum DataBindContainerDependent {
    Authored(CoreHandle),
    StateMachine(RuntimeStateMachineInstanceWeakHandle),
}
impl PartialEq for DataBindContainerDependent {
    fn eq(&self, other: &Self) -> bool {
        self.same_identity(other)
    }
}

impl DataBindContainerDependent {
    pub(crate) fn main_view_model_instance_changed(&self) {
        match self {
            Self::Authored(owner) => {
                DataBindContainerOwner::Authored(owner.clone()).main_view_model_instance_changed()
            }
            Self::StateMachine(owner) => DataBindContainerOwner::StateMachine(owner.clone())
                .main_view_model_instance_changed(),
        }
    }
    pub(crate) fn drop_instance_value_binds_targeting(&self, target: &CoreHandle) {
        match self {
            Self::Authored(owner) => DataBindContainerOwner::Authored(owner.clone())
                .drop_instance_value_binds_targeting(target),
            Self::StateMachine(owner) => DataBindContainerOwner::StateMachine(owner.clone())
                .drop_instance_value_binds_targeting(target),
        }
    }
    pub(crate) fn same_identity(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Authored(left), Self::Authored(right)) => left == right,
            (Self::StateMachine(left), Self::StateMachine(right)) => left.ptr_eq(right),
            _ => false,
        }
    }

    pub(crate) fn relink_data_context(&self) {
        let dependent = self.clone();
        if crate::view_model_cell::defer_transaction_dependency_notification(move || {
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
    value_data_binds: LazyVector<CoreHandle>,
    parent: Option<CoreHandle>,
    more_parents: LazyVector<CoreHandle>,
    dependents: LazyVector<DataBindContainerDependent>,
    property_symbols: Vec<(SymbolType, CoreHandle)>,
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
        self.append_value(value);
    }
    pub fn reserve_values(&mut self, count: usize) {
        self.property_values
            .reserve(count.saturating_sub(self.property_values.len()));
    }
    pub fn append_value(&mut self, value: CoreHandle) {
        value
            .with_mut(|value| {
                self.append_value_borrowed(
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
        self.append_value_borrowed(value);
    }
    fn append_value_borrowed(
        &mut self,
        value: &mut super::viewmodel_instance_value::ViewModelInstanceValue,
    ) {
        let handle = value
            .handle()
            .expect("ViewModelInstanceValue is arena-owned");
        value.set_view_model_instance_borrowed(self);
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
        for bind in self.value_data_binds.view().to_vec() {
            if bind
                .with(|bind| bind.as_data_bind().and_then(DataBind::target))
                .flatten()
                .as_ref()
                == Some(&value)
            {
                self.value_data_binds.erase_all(&bind);
                DataBind::unbind_handle(&bind);
                bind.remove_occurrence();
            }
        }
        for dependent in self.dependents.view().to_vec() {
            dependent.drop_instance_value_binds_targeting(&value);
        }
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
        self.property_symbols.retain(|(_, stored)| stored != &value);
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
        self.property_symbols
            .iter()
            .find(|(symbol, _)| *symbol == symbol_type)
            .map(|(_, value)| value.clone())
    }

    pub fn set_property_symbol(&mut self, symbol_type: SymbolType, value: CoreHandle) {
        if symbol_type != SymbolType::None {
            if let Some((_, stored)) = self
                .property_symbols
                .iter_mut()
                .find(|(symbol, _)| *symbol == symbol_type)
            {
                *stored = value;
            } else {
                self.property_symbols.push((symbol_type, value));
            }
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
            return Self::replace_view_model_property_occurrence(
                owner,
                property_value,
                Some(value),
            );
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
        if previous.as_ref() == Some(&value) {
            return true;
        }
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
        // CoreHandle equality is instance identity. No swap means no mutation,
        // callback, or dependent invalidation, including the None/None case.
        if previous == value {
            return true;
        }
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
        let dependents = owner
            .with_downcast::<Self, _>(|owner| owner.dependents.snapshot())
            .expect("ViewModel occurrence");
        for dependent in dependents {
            dependent.relink_data_context();
        }
        if let Some(parent) = owner
            .with_downcast::<Self, _>(|owner| owner.parent.clone())
            .flatten()
        {
            Self::rebind_dependents_occurrence(&parent);
        }
        let parents = owner
            .with_downcast::<Self, _>(|owner| owner.more_parents.snapshot())
            .expect("ViewModel occurrence");
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
        let Some((copy_values, properties)) = source.with_downcast::<Self, _>(|source| {
            (
                source.base.base.base.base.artboard_handle().is_none(),
                source.property_values.clone(),
            )
        }) else {
            return false;
        };
        // Artboard-owned values are cloned by the artboard's object traversal.
        if copy_values {
            for property in &properties {
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
            // Source starts this distinct collection after property clone
            // callbacks; those callbacks may add owned value bindings.
            let Some(binds) = source.with_downcast::<Self, _>(|source| {
                source.value_data_binds.view().to_vec()
            }) else {
                return false;
            };
            for bind in binds {
                let target = bind
                    .with(|bind| bind.as_data_bind().and_then(DataBind::target))
                    .flatten();
                let Some(index) = source.with_downcast::<Self, _>(|source| {
                    source.property_values.iter()
                        .position(|property| Some(property) == target.as_ref())
                }) else {
                    return false;
                };
                if let Some(index) = index {
                    // An earlier binding clone may change the source index or
                    // the clone's current property. Read both at this use.
                    let Some(property) = cloned.with_downcast::<Self, _>(|cloned| {
                        cloned.property_values.get(index).cloned()
                    }).flatten() else {
                        return false;
                    };
                    let Some(bind) = DataBind::clone_with_target_handle(
                        &bind,
                        Some(property),
                    ) else {
                        return false;
                    };
                    if cloned.with_downcast_mut::<Self, _>(|cloned| {
                        cloned.add_value_data_bind(bind)
                    }).is_none() {
                        return false;
                    }
                }
            }
        }
        // ViewModelInstance::clone reads viewModel() after every open clone.
        let Some(view_model) = source.with_downcast::<Self, _>(Self::get_view_model) else {
            return false;
        };
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

    pub fn value_data_binds(&self) -> &[CoreHandle] {
        self.value_data_binds.view()
    }

    pub fn add_value_data_bind(&mut self, bind: CoreHandle) {
        let flags = bind
            .with(|owner| {
                let bind = owner.as_data_bind().unwrap();
                (bind.to_source() && bind.to_target())
                    .then(|| bind.base.flags() | SOURCE_TO_TARGET_FIRST)
            })
            .flatten();
        if let Some(flags) = flags {
            // The source setter notifies before insertion. Release the bind
            // first: its FLAGS observer may legally be the bind itself.
            DataBind::set_uint_handle(
                &bind,
                crate::source::generated::data_bind::data_bind_base::DataBindBase::FLAGS_PROPERTY_KEY,
                flags,
            );
        }
        self.value_data_binds.push_back(bind);
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
        let mut index = 0;
        while index < self.property_values.len() {
            #[cfg(feature = "tools")]
            let value = self.property_values[index].clone();
            #[cfg(not(feature = "tools"))]
            let value = &self.property_values[index];
            value.with_mut(|value| {
                assert!(
                    value.view_model_instance_value_advanced(),
                    "ViewModel property value advance capability"
                );
            });
            index += 1;
        }
    }

    /// Release the owner borrow before tools callbacks can remove entries.
    /// The index advances even after removal, just as the upstream loop does.
    pub fn advanced_handle(owner: &CoreHandle) {
        #[cfg(not(feature = "tools"))]
        owner.with_mut(|object| {
            if let Some(instance) = object.as_view_model_instance_mut() {
                instance.advanced();
            }
        });
        #[cfg(feature = "tools")]
        {
            let _retained = owner.retain_arena();
            let mut index = 0;
            while let Some(value) = owner
                .with(|object| {
                    object
                        .as_view_model_instance()
                        .and_then(|instance| instance.property_values.get(index).cloned())
                })
                .flatten()
            {
                Self::advanced_value_handle(&value);
                index += 1;
            }
        }
    }

    pub(crate) fn advanced_value_handle(value: &CoreHandle) {
        // Keep the arena-owned occurrence alive through the complete call,
        // including a leaf's synchronous tools callback.
        #[cfg(feature = "tools")]
        let _retained = value.retain_arena();
        #[cfg(feature = "tools")]
        {
            if value
                .with(|object| object.as_view_model_instance_trigger().is_some())
                .unwrap_or(false)
            {
                super::viewmodel_instance_trigger::ViewModelInstanceTrigger::advanced_handle(value);
                return;
            }
            let bound_artboard_instance = value
                .with(|object| {
                    object
                        .as_view_model_instance_artboard()
                        .map(|value| value.bound_view_model_instance())
                })
                .flatten();
            if let Some(instance) = bound_artboard_instance {
                if let Some(instance) = instance {
                    Self::advanced_handle(&instance);
                }
                value.with_mut(|object| {
                    object
                        .as_view_model_instance_artboard_mut()
                        .expect("artboard value")
                        .base
                        .advanced()
                });
                return;
            }
            let nested = value.with(|object| {
                if object.as_view_model_instance_list().is_some() {
                    return (true, None);
                }
                (
                    false,
                    object
                        .as_view_model_instance_view_model()
                        .map(|value| value.reference_view_model_instance()),
                )
            });
            match nested {
                Some((true, _)) => {
                    super::viewmodel_instance_list::ViewModelInstanceList::advanced_handle(value);
                    return;
                }
                Some((false, Some(instance))) => {
                    if let Some(instance) = instance {
                        Self::advanced_handle(&instance);
                    }
                    return;
                }
                _ => {}
            }
        }
        value.with_mut(|object| {
            assert!(
                object.view_model_instance_value_advanced(),
                "ViewModelInstance property values implement advanced"
            )
        });
    }

    pub fn add_parent(&mut self, parent: CoreHandle) {
        if self.parent.as_ref() == Some(&parent) {
            return;
        }
        if self.parent.is_none() {
            self.parent = Some(parent);
            return;
        }
        self.more_parents.push_unique(parent);
    }

    pub fn remove_parent(&mut self, parent: &CoreHandle) {
        self.more_parents.erase_all(parent);
        if self.parent.as_ref() == Some(parent) {
            self.parent = self.more_parents.view().last().cloned();
            if let Some(parent) = &self.parent {
                self.more_parents.erase_all(parent);
            }
        }
    }

    pub fn has_parents(&self) -> bool {
        self.parent.is_some()
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
        self.dependents.push_unique(dependent);
    }

    fn remove_dependent_occurrence(&mut self, dependent: &DataBindContainerDependent) {
        self.dependents.erase_all(dependent);
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
        let mut parents = self.more_parents.view().to_vec();
        if let Some(parent) = &self.parent {
            parents.insert(0, parent.clone());
        }
        parents
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
        for dependent in self.dependents.iter() {
            dependent.relink_data_context();
        }
        if let Some(parent) = self.parent.clone() {
            parent.with_mut(|parent| {
                if let Some(parent) = parent.as_view_model_instance_mut() {
                    parent.rebind_dependents();
                }
            });
        }
        for parent in self.more_parents.snapshot() {
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
        for bind in self.value_data_binds.view().to_vec() {
            DataBind::unbind_handle(&bind);
            bind.remove_occurrence();
        }
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

#[cfg(test)]
mod parent_storage_tests {
    use super::*;
    use crate::source::core::CoreArena;

    #[test]
    fn removing_inline_parent_promotes_last_extra_parent() {
        let arena = CoreArena::default();
        let parents: Vec<_> = (0..4)
            .map(|_| arena.insert(ViewModelInstance::default()))
            .collect();
        let mut instance = ViewModelInstance::default();
        for parent in &parents {
            instance.add_parent(parent.clone());
        }
        instance.add_parent(parents[0].clone());
        instance.add_parent(parents[2].clone());
        assert_eq!(instance.parents(), parents);
        instance.remove_parent(&parents[0]);
        assert_eq!(
            instance.parents(),
            vec![parents[3].clone(), parents[1].clone(), parents[2].clone()]
        );
        instance.remove_parent(&parents[1]);
        instance.remove_parent(&parents[3]);
        assert_eq!(instance.parents(), vec![parents[2].clone()]);
        instance.remove_parent(&parents[2]);
        assert!(!instance.has_parents());
    }
}
