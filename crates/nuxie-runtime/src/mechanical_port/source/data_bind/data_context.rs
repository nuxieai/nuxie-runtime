use std::{cell::RefCell, rc::Rc};

use crate::mechanical_port::source::{
    animation::state_machine_instance::RuntimeStateMachineInstanceWeakHandle,
    assets::manifest_asset::ManifestAsset, core::CoreHandle,
    data_bind::data_bind_path::DataBindPath, data_resolver::DataResolver,
    viewmodel::viewmodel_instance::DataBindContainerDependent,
};

const NO_SLOT: u32 = u32::MAX;

#[derive(Default)]
struct GlobalSlots {
    slot_keys: Vec<u32>,
}

pub struct DataContext {
    parent: Option<RuntimeDataContextHandle>,
    instances: Vec<Option<CoreHandle>>,
    dependent_containers: Vec<DataBindContainerDependent>,
    global_slots: Option<GlobalSlots>,
}

/// One shared mutable occurrence corresponding to upstream `rcp<DataContext>`.
/// Borrows are closure-scoped so references cannot escape the occurrence.
#[derive(Clone)]
pub struct RuntimeDataContextHandle(Rc<RefCell<DataContext>>);

impl RuntimeDataContextHandle {
    pub fn new(context: DataContext) -> Self {
        Self(Rc::new(RefCell::new(context)))
    }

    pub fn with_context<R>(&self, use_context: impl FnOnce(&DataContext) -> R) -> R {
        use_context(&self.0.borrow())
    }

    pub fn with_context_mut<R>(&self, use_context: impl FnOnce(&mut DataContext) -> R) -> R {
        use_context(&mut self.0.borrow_mut())
    }

    /// Mutate the shared occurrence and release its borrow before synchronous
    /// notifications reenter it. Each setter completes its callbacks before the
    /// next setter can run; changes are not coalesced across a closure or frame.
    pub fn set_view_model_instance(&self, value: Option<CoreHandle>) {
        self.mutate_main_instance(|context| context.set_view_model_instance_silently(value));
    }

    pub fn set_main_view_model_instance(&self, value: Option<CoreHandle>) {
        self.mutate_main_instance(|context| context.set_main_view_model_instance_silently(value));
    }

    pub fn remove_main_view_model_instance(&self) {
        self.mutate_main_instance(DataContext::remove_main_view_model_instance_silently);
    }

    fn mutate_main_instance(&self, mutate: impl FnOnce(&mut DataContext)) {
        let containers = {
            let mut context = self.0.borrow_mut();
            mutate(&mut context);
            context.dependent_containers.clone()
        };
        for container in containers {
            container.main_view_model_instance_changed();
        }
    }

    pub fn ptr_eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }

    pub fn debugging_refcnt(&self) -> usize {
        Rc::strong_count(&self.0)
    }
}

impl Drop for DataContext {
    fn drop(&mut self) {
        for instance in self.instances.clone() {
            self.detach_containers(&instance);
        }
    }
}

impl DataContext {
    pub fn new(instance: Option<CoreHandle>) -> Self {
        Self {
            parent: None,
            instances: instance.into_iter().map(Some).collect(),
            dependent_containers: Vec::new(),
            global_slots: None,
        }
    }

    pub fn from_instances(instances: Vec<Option<CoreHandle>>) -> Self {
        Self {
            parent: None,
            instances,
            dependent_containers: Vec::new(),
            global_slots: None,
        }
    }

    fn attach_containers(&self, instance: &Option<CoreHandle>) {
        let Some(instance) = instance else {
            return;
        };
        for container in &self.dependent_containers {
            instance.with_mut(|instance| {
                if let Some(instance) = instance.as_view_model_instance_mut() {
                    match container {
                        DataBindContainerDependent::Authored(container) => {
                            instance.add_dependent(container.clone());
                        }
                        DataBindContainerDependent::StateMachine(container) => {
                            instance.add_state_machine_dependent(container.clone());
                        }
                    }
                }
            });
        }
    }

    fn detach_containers(&self, instance: &Option<CoreHandle>) {
        let Some(instance) = instance else {
            return;
        };
        for container in &self.dependent_containers {
            instance.with_mut(|instance| {
                if let Some(instance) = instance.as_view_model_instance_mut() {
                    match container {
                        DataBindContainerDependent::Authored(container) => {
                            instance.remove_dependent(container);
                        }
                        DataBindContainerDependent::StateMachine(container) => {
                            instance.remove_state_machine_dependent(container);
                        }
                    }
                }
            });
        }
    }

    pub fn add_dependent_container(&mut self, container: CoreHandle) {
        self.add_dependent_container_occurrence(DataBindContainerDependent::Authored(container));
    }

    pub fn remove_dependent_container(&mut self, container: &CoreHandle) {
        self.remove_dependent_container_occurrence(&DataBindContainerDependent::Authored(
            container.clone(),
        ));
    }

    pub fn add_state_machine_dependent_container(
        &mut self,
        container: RuntimeStateMachineInstanceWeakHandle,
    ) {
        self.add_dependent_container_occurrence(DataBindContainerDependent::StateMachine(
            container,
        ));
    }

    pub fn remove_state_machine_dependent_container(
        &mut self,
        container: &RuntimeStateMachineInstanceWeakHandle,
    ) {
        self.remove_dependent_container_occurrence(&DataBindContainerDependent::StateMachine(
            container.clone(),
        ));
    }

    fn add_dependent_container_occurrence(&mut self, container: DataBindContainerDependent) {
        if self
            .dependent_containers
            .iter()
            .any(|candidate| candidate.same_identity(&container))
        {
            return;
        }
        self.dependent_containers.push(container.clone());
        for instance in self.instances.iter().flatten() {
            instance.with_mut(|instance| {
                if let Some(instance) = instance.as_view_model_instance_mut() {
                    match &container {
                        DataBindContainerDependent::Authored(container) => {
                            instance.add_dependent(container.clone());
                        }
                        DataBindContainerDependent::StateMachine(container) => {
                            instance.add_state_machine_dependent(container.clone());
                        }
                    }
                }
            });
        }
    }

    fn remove_dependent_container_occurrence(&mut self, container: &DataBindContainerDependent) {
        for instance in self.instances.iter().flatten() {
            instance.with_mut(|instance| {
                if let Some(instance) = instance.as_view_model_instance_mut() {
                    match container {
                        DataBindContainerDependent::Authored(container) => {
                            instance.remove_dependent(container);
                        }
                        DataBindContainerDependent::StateMachine(container) => {
                            instance.remove_state_machine_dependent(container);
                        }
                    }
                }
            });
        }
        self.dependent_containers
            .retain(|candidate| !candidate.same_identity(container));
    }

    fn ensure_global_slots(&mut self) {
        if self.global_slots.is_none() {
            self.global_slots = Some(GlobalSlots {
                slot_keys: vec![NO_SLOT; self.instances.len()],
            });
        }
    }

    fn slot_key_at(&self, index: usize) -> u32 {
        self.global_slots
            .as_ref()
            .and_then(|slots| slots.slot_keys.get(index))
            .copied()
            .unwrap_or(NO_SLOT)
    }

    fn insert_instance_at(&mut self, index: usize, value: CoreHandle, slot_key: u32) {
        self.instances.insert(index, Some(value));
        if let Some(slots) = self.global_slots.as_mut() {
            slots.slot_keys.insert(index, slot_key);
        }
        self.attach_containers(&self.instances[index]);
    }

    fn remove_instance_at(&mut self, index: usize) {
        self.detach_containers(&self.instances[index]);
        self.instances.remove(index);
        if let Some(slots) = self.global_slots.as_mut() {
            slots.slot_keys.remove(index);
        }
    }

    pub fn set_view_model_instance(&mut self, value: Option<CoreHandle>) {
        self.set_view_model_instance_silently(value);
        self.notify_main_view_model_instance_changed();
    }

    fn set_view_model_instance_silently(&mut self, value: Option<CoreHandle>) {
        if self.global_slots.is_some() {
            self.set_main_view_model_instance_silently(value);
            return;
        }
        if self.instances.is_empty() {
            self.instances.push(value);
            self.attach_containers(self.instances.last().unwrap());
        } else {
            self.detach_containers(&self.instances[0]);
            self.instances[0] = value;
            self.attach_containers(&self.instances[0]);
        }
    }

    fn notify_main_view_model_instance_changed(&mut self) {
        let containers = self.dependent_containers.clone();
        for container in containers {
            container.main_view_model_instance_changed();
        }
    }

    pub fn set_view_model_instance_for_slot(&mut self, slot_key: u32, value: Option<CoreHandle>) {
        let Some(value) = value else {
            if self.global_slots.is_some()
                && let Some(index) =
                    (0..self.instances.len()).find(|index| self.slot_key_at(*index) == slot_key)
            {
                self.remove_instance_at(index);
            }
            return;
        };
        self.ensure_global_slots();
        if let Some(index) =
            (0..self.instances.len()).find(|index| self.slot_key_at(*index) == slot_key)
        {
            self.detach_containers(&self.instances[index]);
            self.instances[index] = Some(value);
            self.global_slots.as_mut().unwrap().slot_keys[index] = slot_key;
            self.attach_containers(&self.instances[index]);
            return;
        }
        let mut index = 0;
        while index < self.instances.len() && self.slot_key_at(index) == NO_SLOT {
            index += 1;
        }
        while index < self.instances.len()
            && self.slot_key_at(index) != NO_SLOT
            && self.slot_key_at(index) < slot_key
        {
            index += 1;
        }
        self.insert_instance_at(index, value, slot_key);
    }

    pub fn instance_for_slot(&self, slot: u32) -> Option<CoreHandle> {
        (0..self.instances.len())
            .find(|index| self.slot_key_at(*index) == slot)
            .and_then(|index| self.instances[index].clone())
    }

    pub fn resolve_global_view_model(
        &self,
        file: &crate::mechanical_port::source::file::RuntimeFileHandle,
        name: &[u8],
    ) -> Option<CoreHandle> {
        use crate::mechanical_port::source::{
            view_model_type::ViewModelType, viewmodel::viewmodel::ViewModel,
        };
        // Match the upstream const-char name without requiring Lua strings to
        // be UTF-8. C++ name lookup stops at the first NUL.
        let name = name.split(|byte| *byte == 0).next().unwrap_or_default();
        let (slot_key, count, global) = file.with_file(|file| {
            let count = file.view_model_count();
            let slot = (0..count)
                .find(|index| {
                    file.view_model(*index)
                        .and_then(|model| {
                            model.with_downcast::<ViewModel, _>(|model| {
                                model.base.name().as_bytes() == name
                            })
                        })
                        .unwrap_or(false)
                })
                .unwrap_or(count);
            (slot as u32, count, file.view_model(slot))
        });
        if slot_key as usize >= count {
            return None;
        }
        let global = global?;
        if global.with_downcast::<ViewModel, _>(|model| {
            model.base.view_model_type() == ViewModelType::Global as u32
        }) != Some(true)
        {
            return None;
        }

        let mut root = self.parent();
        while let Some(parent) = root
            .as_ref()
            .and_then(|root| root.with_context(DataContext::parent))
        {
            root = Some(parent);
        }
        let slotted = match root {
            Some(root) => root.with_context(|root| root.instance_for_slot(slot_key)),
            None => self.instance_for_slot(slot_key),
        };
        if slotted.is_some() {
            return slotted;
        }

        let find = |instances: &[Option<CoreHandle>]| {
            instances
                .iter()
                .flatten()
                .find(|instance| {
                    instance
                        .with(|instance| {
                            instance
                                .as_view_model_instance()
                                .and_then(|instance| instance.get_view_model())
                                .is_some_and(|model| model == global)
                        })
                        .unwrap_or(false)
                })
                .cloned()
        };
        if let Some(instance) = find(self.view_model_instances()) {
            return Some(instance);
        }
        let mut current = self.parent();
        while let Some(context) = current {
            let (instance, parent) = context
                .with_context(|context| (find(context.view_model_instances()), context.parent()));
            if instance.is_some() {
                return instance;
            }
            current = parent;
        }
        None
    }

    pub fn remove_main_view_model_instance(&mut self) {
        self.remove_main_view_model_instance_silently();
        self.notify_main_view_model_instance_changed();
    }

    fn remove_main_view_model_instance_silently(&mut self) {
        let mut index = 0;
        while index < self.instances.len() {
            if self.slot_key_at(index) == NO_SLOT {
                self.remove_instance_at(index)
            } else {
                index += 1;
            }
        }
    }

    pub fn set_main_view_model_instance(&mut self, value: Option<CoreHandle>) {
        self.set_main_view_model_instance_silently(value);
        self.notify_main_view_model_instance_changed();
    }

    fn set_main_view_model_instance_silently(&mut self, value: Option<CoreHandle>) {
        self.remove_main_view_model_instance_silently();
        if let Some(value) = value {
            self.insert_instance_at(0, value, NO_SLOT);
        }
    }

    pub fn main_view_model_instance(&self) -> Option<CoreHandle> {
        (0..self.instances.len())
            .find(|index| self.slot_key_at(*index) == NO_SLOT)
            .and_then(|index| self.instances[index].clone())
    }

    pub fn advanced(&self) {
        for instance in &self.instances {
            let instance = instance
                .as_ref()
                .expect("DataContext::advanced requires non-null view model entries");
            crate::source::viewmodel::viewmodel_instance::ViewModelInstance::advanced_handle(
                instance,
            );
        }
    }

    fn instance_view_model_id(instance: &CoreHandle) -> Option<u32> {
        instance
            .with(|instance| {
                instance
                    .as_view_model_instance()
                    .map(|instance| instance.base.view_model_id())
            })
            .flatten()
    }

    fn instance_property_by_id(instance: &CoreHandle, id: u32) -> Option<CoreHandle> {
        instance
            .with(|instance| {
                instance
                    .as_view_model_instance()
                    .and_then(|instance| instance.property_value_by_id(id))
            })
            .flatten()
    }

    fn instance_property_named(instance: &CoreHandle, name: &str) -> Option<CoreHandle> {
        instance
            .with(|instance| {
                instance
                    .as_view_model_instance()
                    .and_then(|instance| instance.property_value_named(name))
            })
            .flatten()
    }

    fn referenced_instance(value: &CoreHandle) -> Option<CoreHandle> {
        value
            .with(|value| {
                value
                    .as_view_model_instance_view_model()
                    .and_then(|value| value.reference_view_model_instance())
            })
            .flatten()
    }

    fn try_property(instance: CoreHandle, path: &[u32]) -> Option<CoreHandle> {
        if Self::instance_view_model_id(&instance)? != path[0] || path.len() == 1 {
            return None;
        }
        let mut current = instance;
        for id in &path[1..path.len() - 1] {
            current = Self::referenced_instance(&Self::instance_property_by_id(&current, *id)?)?;
        }
        Self::instance_property_by_id(&current, *path.last().unwrap())
    }

    fn try_relative_property(
        instance: CoreHandle,
        path: &[u32],
        resolver: &dyn DataResolver,
    ) -> Option<CoreHandle> {
        let mut current = instance;
        if path.len() == 1 {
            return Self::instance_property_named(&current, resolver.resolve_name(path[0] as i32));
        }
        for id in &path[..path.len() - 1] {
            let property =
                Self::instance_property_named(&current, resolver.resolve_name(*id as i32))?;
            current = Self::referenced_instance(&property)?;
        }
        Self::instance_property_named(
            &current,
            resolver.resolve_name(*path.last().unwrap() as i32),
        )
    }

    fn try_instance(instance: CoreHandle, path: &[u32]) -> Option<CoreHandle> {
        if Self::instance_view_model_id(&instance)? != path[0] {
            return None;
        }
        let mut current = instance;
        for id in &path[1..] {
            current = Self::referenced_instance(&Self::instance_property_by_id(&current, *id)?)?;
        }
        Some(current)
    }

    fn try_relative_instance(
        instance: CoreHandle,
        path: &[u32],
        resolver: &dyn DataResolver,
    ) -> Option<CoreHandle> {
        let mut current = instance;
        for id in path {
            let property =
                Self::instance_property_named(&current, resolver.resolve_name(*id as i32))?;
            current = Self::referenced_instance(&property)?;
        }
        Some(current)
    }

    pub fn get_view_model_property(&self, path: &[u32]) -> Option<CoreHandle> {
        if path.is_empty() {
            return None;
        }
        for instance in self.instances.iter().flatten() {
            if let Some(value) = Self::try_property(instance.clone(), path) {
                return Some(value);
            }
        }
        self.parent
            .as_ref()?
            .with_context(|parent| parent.get_view_model_property(path))
    }

    pub fn get_relative_view_model_property(
        &self,
        path: &[u32],
        resolver: Option<&dyn DataResolver>,
    ) -> Option<CoreHandle> {
        let resolver = resolver?;
        if path.is_empty() {
            return None;
        }
        for instance in self.instances.iter().flatten() {
            if let Some(value) = Self::try_relative_property(instance.clone(), path, resolver) {
                return Some(value);
            }
        }
        self.parent
            .as_ref()?
            .with_context(|parent| parent.get_relative_view_model_property(path, Some(resolver)))
    }

    pub fn get_view_model_instance(&self, path: &[u32]) -> Option<CoreHandle> {
        if path.is_empty() {
            return None;
        }
        for instance in self.instances.iter().flatten() {
            if let Some(value) = Self::try_instance(instance.clone(), path) {
                return Some(value);
            }
        }
        self.parent
            .as_ref()?
            .with_context(|parent| parent.get_view_model_instance(path))
    }

    pub fn get_relative_view_model_instance(
        &self,
        path: &[u32],
        resolver: Option<&dyn DataResolver>,
    ) -> Option<CoreHandle> {
        let resolver = resolver?;
        if path.is_empty() {
            return None;
        }
        for instance in self.instances.iter().flatten() {
            if let Some(value) = Self::try_relative_instance(instance.clone(), path, resolver) {
                return Some(value);
            }
        }
        self.parent
            .as_ref()?
            .with_context(|parent| parent.get_relative_view_model_instance(path, Some(resolver)))
    }

    pub fn get_property_from_path(&self, path: &mut DataBindPath) -> Option<CoreHandle> {
        if path.is_relative() {
            let resolved = path.resolved_path().to_vec();
            path.file()
                .with_file(|file| {
                    file.manifest()?
                        .with_downcast::<ManifestAsset, _>(|resolver| {
                            self.get_relative_view_model_property(&resolved, Some(resolver))
                        })
                })
                .flatten()
                .flatten()
        } else {
            self.get_view_model_property(path.path())
        }
    }

    pub fn get_instance_from_path(&self, path: Option<&mut DataBindPath>) -> Option<CoreHandle> {
        let path = path?;
        if path.is_relative() {
            let resolved = path.resolved_path().to_vec();
            path.file()
                .with_file(|file| {
                    file.manifest()?
                        .with_downcast::<ManifestAsset, _>(|resolver| {
                            self.get_relative_view_model_instance(&resolved, Some(resolver))
                        })
                })
                .flatten()
                .flatten()
        } else {
            self.get_view_model_instance(path.resolved_path())
        }
    }

    pub fn set_parent(&mut self, value: Option<RuntimeDataContextHandle>) {
        self.parent = value
    }

    pub fn parent(&self) -> Option<RuntimeDataContextHandle> {
        self.parent.clone()
    }

    pub fn view_model_instances(&self) -> &[Option<CoreHandle>] {
        &self.instances
    }

    pub fn root_view_model_instance(&self) -> Option<CoreHandle> {
        self.parent.as_ref().map_or_else(
            || self.main_view_model_instance(),
            |parent| parent.with_context(DataContext::root_view_model_instance),
        )
    }

    pub fn view_model_value(&self) -> Option<CoreHandle> {
        self.parent
            .as_ref()
            .and_then(|parent| parent.with_context(DataContext::view_model_value))
    }
}
