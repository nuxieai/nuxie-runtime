use std::marker::PhantomData;

use crate::mechanical_port::source::{
    component::ComponentOccurrenceHandle,
    component_dirt::ComponentDirt,
    lazy_vector::{LazyVector, LazyVectorSnapshot},
};

pub trait DirtDependent {
    fn add_dirt(&mut self, value: ComponentDirt, recurse: bool);
}

pub trait DependencyRoot<U> {
    fn on_component_dirty(&mut self, component: &mut U);
}

pub struct DependencyHelper<U> {
    dependents: LazyVector<ComponentOccurrenceHandle>,
    marker: PhantomData<fn() -> U>,
}

impl<U> Default for DependencyHelper<U> {
    fn default() -> Self {
        Self {
            dependents: LazyVector::default(),
            marker: PhantomData,
        }
    }
}

impl<U> Clone for DependencyHelper<U> {
    fn clone(&self) -> Self {
        Self {
            dependents: self.dependents.clone(),
            marker: PhantomData,
        }
    }
}

impl<U: DirtDependent> DependencyHelper<U> {
    pub fn add_dependent(&mut self, component: ComponentOccurrenceHandle) {
        self.dependents.push_unique(component);
    }

    pub fn remove_dependent(&mut self, component: &ComponentOccurrenceHandle) {
        self.dependents.erase_all(component);
    }

    pub fn add_dirt_to_dependents(&mut self, value: ComponentDirt) {
        if self.dependents.empty() {
            return;
        }
        for dependent in self.dependents.iter() {
            dependent.add_dirt(value, true);
        }
    }

    pub fn on_component_dirty<D: DependencyRoot<U>>(&mut self, derived: &mut D, component: &mut U) {
        derived.on_component_dirty(component);
    }

    pub fn dependents(&self) -> &[ComponentOccurrenceHandle] {
        self.dependents.view()
    }

    pub fn dependents_snapshot(&self) -> LazyVectorSnapshot<ComponentOccurrenceHandle> {
        self.dependents.snapshot()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::source::{component::Component, core::CoreArena, node::Node};

    #[test]
    fn helper_is_pointer_sized_and_clone_membership_is_independent() {
        assert_eq!(
            std::mem::size_of::<DependencyHelper<Component>>(),
            std::mem::size_of::<usize>()
        );
        let arena = CoreArena::default();
        let first = ComponentOccurrenceHandle::Authored(arena.insert(Node::default()));
        let second = ComponentOccurrenceHandle::Authored(arena.insert(Node::default()));
        let mut owner = DependencyHelper::<Component>::default();
        owner.add_dependent(first.clone());
        owner.add_dependent(first.clone());
        owner.add_dependent(second.clone());
        assert_eq!(owner.dependents().len(), 2);
        let mut copy = owner.clone();
        copy.remove_dependent(&first);
        assert!(owner.dependents()[0] == first);
        assert!(owner.dependents()[1] == second);
        assert_eq!(copy.dependents().len(), 1);
        assert!(copy.dependents()[0] == second);
    }
}
