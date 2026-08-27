use std::rc::Rc;
const NO_SLOT: u32 = u32::MAX;
pub trait DependentContainer {}
pub trait ViewModelValue {
    fn referenced_instance(&self) -> Option<Rc<dyn ViewModelInstance>>;
}
pub trait ViewModelInstance {
    fn view_model_id(&self) -> u32;
    fn property_value(&self, id: u32) -> Option<Rc<dyn ViewModelValue>>;
    fn property_value_named(&self, name: &str) -> Option<Rc<dyn ViewModelValue>>;
    fn add_dependent(&self, container: *mut dyn DependentContainer);
    fn remove_dependent(&self, container: *mut dyn DependentContainer);
    fn advanced(&self);
}
pub trait DataResolver {
    fn resolve_name(&self, id: u32) -> String;
}
pub trait DataBindPathLookup {
    fn is_relative(&self) -> bool;
    fn resolved_path(&mut self) -> Vec<u32>;
    fn path(&self) -> Vec<u32>;
    fn resolver(&self) -> Option<&dyn DataResolver>;
}
#[derive(Default)]
struct GlobalSlots {
    slot_keys: Vec<u32>,
}
pub struct DataContext {
    parent: Option<Rc<DataContext>>,
    instances: Vec<Rc<dyn ViewModelInstance>>,
    dependent_containers: Vec<*mut dyn DependentContainer>,
    global_slots: Option<GlobalSlots>,
}
impl Drop for DataContext {
    fn drop(&mut self) {
        let instances = self.instances.clone();
        for instance in instances {
            self.detach_containers(instance.as_ref());
        }
    }
}
impl DataContext {
    pub fn new(instance: Option<Rc<dyn ViewModelInstance>>) -> Self {
        Self {
            parent: None,
            instances: instance.into_iter().collect(),
            dependent_containers: Vec::new(),
            global_slots: None,
        }
    }
    pub fn from_instances(instances: Vec<Rc<dyn ViewModelInstance>>) -> Self {
        Self {
            parent: None,
            instances,
            dependent_containers: Vec::new(),
            global_slots: None,
        }
    }
    fn attach_containers(&self, instance: &dyn ViewModelInstance) {
        for container in &self.dependent_containers {
            instance.add_dependent(*container);
        }
    }
    fn detach_containers(&self, instance: &dyn ViewModelInstance) {
        for container in &self.dependent_containers {
            instance.remove_dependent(*container);
        }
    }
    pub fn add_dependent_container(&mut self, container: *mut dyn DependentContainer) {
        if self
            .dependent_containers
            .iter()
            .any(|item| core::ptr::addr_eq(*item, container))
        {
            return;
        }
        self.dependent_containers.push(container);
        for instance in &self.instances {
            instance.add_dependent(container);
        }
    }
    pub fn remove_dependent_container(&mut self, container: *mut dyn DependentContainer) {
        for instance in &self.instances {
            instance.remove_dependent(container);
        }
        self.dependent_containers
            .retain(|item| !core::ptr::addr_eq(*item, container));
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
    fn insert_instance_at(
        &mut self,
        index: usize,
        value: Rc<dyn ViewModelInstance>,
        slot_key: u32,
    ) {
        self.instances.insert(index, value);
        if let Some(slots) = self.global_slots.as_mut() {
            slots.slot_keys.insert(index, slot_key);
        }
        self.attach_containers(self.instances[index].as_ref());
    }
    fn remove_instance_at(&mut self, index: usize) {
        self.detach_containers(self.instances[index].as_ref());
        self.instances.remove(index);
        if let Some(slots) = self.global_slots.as_mut() {
            slots.slot_keys.remove(index);
        }
    }
    pub fn set_view_model_instance(&mut self, value: Rc<dyn ViewModelInstance>) {
        if self.global_slots.is_some() {
            self.set_main_view_model_instance(Some(value));
            return;
        }
        if self.instances.is_empty() {
            self.instances.push(value);
            self.attach_containers(self.instances.last().unwrap().as_ref());
        } else {
            self.detach_containers(self.instances[0].as_ref());
            self.instances[0] = value;
            self.attach_containers(self.instances[0].as_ref());
        }
    }
    pub fn set_view_model_instance_for_slot(
        &mut self,
        slot_key: u32,
        value: Option<Rc<dyn ViewModelInstance>>,
    ) {
        let Some(value) = value else {
            if self.global_slots.is_some() {
                if let Some(index) =
                    (0..self.instances.len()).find(|index| self.slot_key_at(*index) == slot_key)
                {
                    self.remove_instance_at(index);
                }
            }
            return;
        };
        self.ensure_global_slots();
        if let Some(index) =
            (0..self.instances.len()).find(|index| self.slot_key_at(*index) == slot_key)
        {
            self.detach_containers(self.instances[index].as_ref());
            self.instances[index] = value;
            self.global_slots.as_mut().unwrap().slot_keys[index] = slot_key;
            self.attach_containers(self.instances[index].as_ref());
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
    pub fn instance_for_slot(&self, slot: u32) -> Option<Rc<dyn ViewModelInstance>> {
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
    pub fn set_main_view_model_instance(&mut self, value: Option<Rc<dyn ViewModelInstance>>) {
        self.remove_main_view_model_instance();
        if let Some(value) = value {
            self.insert_instance_at(0, value, NO_SLOT);
        }
    }
    pub fn main_view_model_instance(&self) -> Option<Rc<dyn ViewModelInstance>> {
        (0..self.instances.len())
            .find(|index| self.slot_key_at(*index) == NO_SLOT)
            .map(|index| self.instances[index].clone())
    }
    pub fn advanced(&self) {
        for instance in &self.instances {
            instance.advanced();
        }
    }
    fn try_property(
        instance: Rc<dyn ViewModelInstance>,
        path: &[u32],
    ) -> Option<Rc<dyn ViewModelValue>> {
        if instance.view_model_id() != path[0] || path.len() == 1 {
            return None;
        }
        let mut current = instance;
        for id in &path[1..path.len() - 1] {
            current = current.property_value(*id)?.referenced_instance()?;
        }
        current.property_value(*path.last().unwrap())
    }
    fn try_relative_property(
        instance: Rc<dyn ViewModelInstance>,
        path: &[u32],
        resolver: &dyn DataResolver,
    ) -> Option<Rc<dyn ViewModelValue>> {
        let mut current = instance;
        if path.len() == 1 {
            return current.property_value_named(&resolver.resolve_name(path[0]));
        }
        for id in &path[..path.len() - 1] {
            current = current
                .property_value_named(&resolver.resolve_name(*id))?
                .referenced_instance()?;
        }
        current.property_value_named(&resolver.resolve_name(*path.last().unwrap()))
    }
    fn try_instance(
        instance: Rc<dyn ViewModelInstance>,
        path: &[u32],
    ) -> Option<Rc<dyn ViewModelInstance>> {
        if instance.view_model_id() != path[0] {
            return None;
        }
        let mut current = instance;
        for id in &path[1..] {
            current = current.property_value(*id)?.referenced_instance()?;
        }
        Some(current)
    }
    fn try_relative_instance(
        instance: Rc<dyn ViewModelInstance>,
        path: &[u32],
        resolver: &dyn DataResolver,
    ) -> Option<Rc<dyn ViewModelInstance>> {
        let mut current = instance;
        for id in path {
            current = current
                .property_value_named(&resolver.resolve_name(*id))?
                .referenced_instance()?;
        }
        Some(current)
    }
    pub fn get_view_model_property(&self, path: &[u32]) -> Option<Rc<dyn ViewModelValue>> {
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
    ) -> Option<Rc<dyn ViewModelValue>> {
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
    pub fn get_view_model_instance(&self, path: &[u32]) -> Option<Rc<dyn ViewModelInstance>> {
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
    ) -> Option<Rc<dyn ViewModelInstance>> {
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
    pub fn get_property_from_path(
        &self,
        path: &mut dyn DataBindPathLookup,
    ) -> Option<Rc<dyn ViewModelValue>> {
        if path.is_relative() {
            let resolver = path.resolver()?;
            self.get_relative_view_model_property(&path.resolved_path(), Some(resolver))
        } else {
            self.get_view_model_property(&path.path())
        }
    }
    pub fn get_instance_from_path(
        &self,
        path: Option<&mut dyn DataBindPathLookup>,
    ) -> Option<Rc<dyn ViewModelInstance>> {
        let path = path?;
        if path.is_relative() {
            let resolver = path.resolver()?;
            self.get_relative_view_model_instance(&path.resolved_path(), Some(resolver))
        } else {
            self.get_view_model_instance(&path.resolved_path())
        }
    }
    pub fn set_parent(&mut self, value: Option<Rc<DataContext>>) {
        self.parent = value
    }
    pub fn parent(&self) -> Option<Rc<DataContext>> {
        self.parent.clone()
    }
    pub fn view_model_instances(&self) -> &[Rc<dyn ViewModelInstance>] {
        &self.instances
    }
    pub fn root_view_model_instance(&self) -> Option<Rc<dyn ViewModelInstance>> {
        self.parent.as_ref().map_or_else(
            || self.main_view_model_instance(),
            |parent| parent.root_view_model_instance(),
        )
    }
    pub fn view_model_value(&self) -> Option<Rc<dyn ViewModelValue>> {
        self.parent
            .as_ref()
            .and_then(|parent| parent.view_model_value())
    }
}
