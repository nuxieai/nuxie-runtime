use std::collections::HashSet;

use crate::mechanical_port::source::core::CoreHandle;

#[derive(Default)]
pub struct DependencySorter {
    perm: HashSet<CoreHandle>,
    temp: HashSet<CoreHandle>,
}

impl DependencySorter {
    pub fn sort(&mut self, root: CoreHandle, order: &mut Vec<CoreHandle>) {
        order.clear();
        self.visit(root, order);
    }

    pub fn sort_roots(&mut self, roots: Vec<CoreHandle>, order: &mut Vec<CoreHandle>) {
        order.clear();
        for root in roots {
            self.visit(root, order);
        }
    }

    pub fn visit(&mut self, component: CoreHandle, order: &mut Vec<CoreHandle>) -> bool {
        if self.perm.contains(&component) {
            return true;
        }
        if self.temp.contains(&component) {
            eprintln!("Dependency cycle!");
            return false;
        }

        self.temp.insert(component.clone());

        let dependents = component
            .with(|component| {
                component
                    .as_component()
                    .map(|component| component.dependents().to_vec())
                    .unwrap_or_default()
            })
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
