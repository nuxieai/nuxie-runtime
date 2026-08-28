use std::rc::Rc;

use crate::mechanical_port::source::{
    assets::manifest_asset::ManifestAsset, core::CoreHandle,
    data_bind::data_bind_path::DataBindPath, data_resolver::DataResolver,
};

const NO_SLOT: u32 = u32::MAX;

#[derive(Default)]
struct GlobalSlots {
    slot_keys: Vec<u32>,
}

pub struct DataContext {
    parent: Option<Rc<DataContext>>,
    instances: Vec<CoreHandle>,
    dependent_containers: Vec<CoreHandle>,
    global_slots: Option<GlobalSlots>,
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
            instances: instance.into_iter().collect(),
            dependent_containers: Vec::new(),
            global_slots: None,
        }
    }

    pub fn from_instances(instances: Vec<CoreHandle>) -> Self {
        Self {
            parent: None,
            instances,
            dependent_containers: Vec::new(),
            global_slots: None,
        }
    }

    fn attach_containers(&self, instance: &CoreHandle) {
        for container in &self.dependent_containers {
            instance.with_mut(|instance| {
                if let Some(instance) = instance.as_view_model_instance_mut() {
                    instance.add_dependent(container.clone());
                }
            });
        }
    }

    fn detach_containers(&self, instance: &CoreHandle) {
        for container in &self.dependent_containers {
            instance.with_mut(|instance| {
                if let Some(instance) = instance.as_view_model_instance_mut() {
                    instance.remove_dependent(container);
                }
            });
        }
    }

    pub fn add_dependent_container(&mut self, container: CoreHandle) {
        if self.dependent_containers.contains(&container) {
            return;
        }
        self.dependent_containers.push(container.clone());
        for instance in &self.instances {
            instance.with_mut(|instance| {
                if let Some(instance) = instance.as_view_model_instance_mut() {
                    instance.add_dependent(container.clone());
                }
            });
        }
    }

    pub fn remove_dependent_container(&mut self, container: &CoreHandle) {
        for instance in &self.instances {
            instance.with_mut(|instance| {
                if let Some(instance) = instance.as_view_model_instance_mut() {
                    instance.remove_dependent(container);
                }
            });
        }
        self.dependent_containers.retain(|item| item != container);
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
        self.instances.insert(index, value);
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

    pub fn set_view_model_instance(&mut self, value: CoreHandle) {
        if self.global_slots.is_some() {
            self.set_main_view_model_instance(Some(value));
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
            self.instances[index] = value;
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
            .map(|index| self.instances[index].clone())
    }

    pub fn remove_main_view_model_instance(&mut self) {
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
        self.remove_main_view_model_instance();
        if let Some(value) = value {
            self.insert_instance_at(0, value, NO_SLOT);
        }
    }

    pub fn main_view_model_instance(&self) -> Option<CoreHandle> {
        (0..self.instances.len())
            .find(|index| self.slot_key_at(*index) == NO_SLOT)
            .map(|index| self.instances[index].clone())
    }

    pub fn advanced(&self) {
        for instance in &self.instances {
            instance.with_mut(|instance| {
                if let Some(instance) = instance.as_view_model_instance_mut() {
                    instance.advanced();
                }
            });
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
        for instance in &self.instances {
            if let Some(value) = Self::try_property(instance.clone(), path) {
                return Some(value);
            }
        }
        self.parent.as_ref()?.get_view_model_property(path)
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
        for instance in &self.instances {
            if let Some(value) = Self::try_relative_property(instance.clone(), path, resolver) {
                return Some(value);
            }
        }
        self.parent
            .as_ref()?
            .get_relative_view_model_property(path, Some(resolver))
    }

    pub fn get_view_model_instance(&self, path: &[u32]) -> Option<CoreHandle> {
        if path.is_empty() {
            return None;
        }
        for instance in &self.instances {
            if let Some(value) = Self::try_instance(instance.clone(), path) {
                return Some(value);
            }
        }
        self.parent.as_ref()?.get_view_model_instance(path)
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
        for instance in &self.instances {
            if let Some(value) = Self::try_relative_instance(instance.clone(), path, resolver) {
                return Some(value);
            }
        }
        self.parent
            .as_ref()?
            .get_relative_view_model_instance(path, Some(resolver))
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

    pub fn set_parent(&mut self, value: Option<Rc<DataContext>>) {
        self.parent = value
    }

    pub fn parent(&self) -> Option<Rc<DataContext>> {
        self.parent.clone()
    }

    pub fn view_model_instances(&self) -> &[CoreHandle] {
        &self.instances
    }

    pub fn root_view_model_instance(&self) -> Option<CoreHandle> {
        self.parent.as_ref().map_or_else(
            || self.main_view_model_instance(),
            |parent| parent.root_view_model_instance(),
        )
    }

    pub fn view_model_value(&self) -> Option<CoreHandle> {
        self.parent
            .as_ref()
            .and_then(|parent| parent.view_model_value())
    }
}
