use std::marker::PhantomData;

use crate::mechanical_port::source::{
    component::ComponentOccurrenceHandle, component_dirt::ComponentDirt,
};

pub trait DirtDependent {
    fn add_dirt(&mut self, value: ComponentDirt, recurse: bool);
}

pub trait DependencyRoot<U> {
    fn on_component_dirty(&mut self, component: &mut U);
}

pub struct DependencyHelper<U> {
    dependents: Vec<ComponentOccurrenceHandle>,
    marker: PhantomData<fn() -> U>,
}

impl<U> Default for DependencyHelper<U> {
    fn default() -> Self {
        Self {
            dependents: Vec::new(),
            marker: PhantomData,
        }
    }
}

impl<U: DirtDependent> DependencyHelper<U> {
    pub fn add_dependent(&mut self, component: ComponentOccurrenceHandle) {
        if !self.dependents.contains(&component) {
            self.dependents.push(component);
        }
    }

    pub fn remove_dependent(&mut self, component: &ComponentOccurrenceHandle) {
        self.dependents.retain(|candidate| candidate != component);
    }

    pub fn add_dirt_to_dependents(&mut self, value: ComponentDirt) {
        if self.dependents.is_empty() {
            return;
        }
        for dependent in self.dependents.iter().cloned() {
            dependent.add_dirt(value, true);
        }
    }

    pub fn on_component_dirty<D: DependencyRoot<U>>(&mut self, derived: &mut D, component: &mut U) {
        derived.on_component_dirty(component);
    }

    pub fn dependents(&self) -> &[ComponentOccurrenceHandle] {
        &self.dependents
    }
}
