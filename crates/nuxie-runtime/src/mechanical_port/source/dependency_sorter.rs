use std::collections::HashSet;

use crate::mechanical_port::source::component::ComponentOccurrenceHandle;

#[derive(Default)]
pub struct DependencySorter {
    perm: HashSet<ComponentOccurrenceHandle>,
    temp: HashSet<ComponentOccurrenceHandle>,
}

impl DependencySorter {
    pub fn sort_with_root_dependents(
        &mut self,
        root: ComponentOccurrenceHandle,
        dependents: Vec<ComponentOccurrenceHandle>,
        order: &mut Vec<ComponentOccurrenceHandle>,
    ) {
        order.clear();
        self.temp.insert(root.clone());
        for dependent in dependents {
            if !self.visit(dependent, order) {
                return;
            }
        }
        self.perm.insert(root.clone());
        order.insert(0, root);
    }
    pub fn sort(
        &mut self,
        root: ComponentOccurrenceHandle,
        order: &mut Vec<ComponentOccurrenceHandle>,
    ) {
        order.clear();
        self.visit(root, order);
    }

    pub fn sort_roots(
        &mut self,
        roots: Vec<ComponentOccurrenceHandle>,
        order: &mut Vec<ComponentOccurrenceHandle>,
    ) {
        order.clear();
        for root in roots {
            self.visit(root, order);
        }
    }

    pub fn visit(
        &mut self,
        component: ComponentOccurrenceHandle,
        order: &mut Vec<ComponentOccurrenceHandle>,
    ) -> bool {
        if self.perm.contains(&component) {
            return true;
        }
        if self.temp.contains(&component) {
            eprintln!("Dependency cycle!");
            return false;
        }

        self.temp.insert(component.clone());

        let dependents = component
            .with_component(|component| component.dependents().to_vec())
            .unwrap_or_default();
        for dependent in dependents {
            if !self.visit(dependent, order) {
                return false;
            }
        }
        self.perm.insert(component.clone());
        order.insert(0, component);

        true
    }
}
