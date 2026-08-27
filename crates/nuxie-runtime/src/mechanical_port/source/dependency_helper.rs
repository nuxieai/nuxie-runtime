use crate::mechanical_port::source::component_dirt::ComponentDirt;

pub trait DirtDependent {
    fn add_dirt(&mut self, value: ComponentDirt, recurse: bool);
}

pub trait DependencyRoot<U> {
    fn on_component_dirty(&mut self, component: &mut U);
}

pub struct DependencyHelper<U> {
    dependents: Vec<*mut U>,
}

impl<U> Default for DependencyHelper<U> {
    fn default() -> Self {
        Self {
            dependents: Vec::new(),
        }
    }
}

impl<U: DirtDependent> DependencyHelper<U> {
    pub fn add_dependent(&mut self, component: *mut U) {
        if !self.dependents.contains(&component) {
            self.dependents.push(component);
        }
    }

    pub fn remove_dependent(&mut self, component: *mut U) {
        self.dependents.retain(|candidate| *candidate != component);
    }

    pub fn add_dirt_to_dependents(&mut self, value: ComponentDirt) {
        if self.dependents.is_empty() {
            return;
        }
        for dependent in self.dependents.iter().copied() {
            unsafe {
                dependent
                    .as_mut()
                    .expect("registered dependent must remain live")
            }
            .add_dirt(value, true);
        }
    }

    pub fn on_component_dirty<D: DependencyRoot<U>>(&mut self, derived: &mut D, component: &mut U) {
        derived.on_component_dirty(component);
    }

    pub fn dependents(&self) -> &[*mut U] {
        &self.dependents
    }
}
