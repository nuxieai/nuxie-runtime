use std::collections::HashSet;

use crate::mechanical_port::source::component::Component;

#[derive(Default)]
pub struct DependencySorter {
    perm: HashSet<*mut Component>,
    temp: HashSet<*mut Component>,
}

impl DependencySorter {
    pub fn sort(&mut self, root: *mut Component, order: &mut Vec<*mut Component>) {
        order.clear();
        self.visit(root, order);
    }

    pub fn sort_roots(&mut self, roots: Vec<*mut Component>, order: &mut Vec<*mut Component>) {
        order.clear();
        for root in roots {
            self.visit(root, order);
        }
    }

    pub fn visit(&mut self, component: *mut Component, order: &mut Vec<*mut Component>) -> bool {
        if self.perm.contains(&component) {
            return true;
        }
        if self.temp.contains(&component) {
            eprintln!("Dependency cycle!");
            return false;
        }

        self.temp.insert(component);

        let dependents = unsafe { &*component }.dependents().to_vec();
        for dependent in dependents {
            if !self.visit(dependent, order) {
                return false;
            }
        }
        self.perm.insert(component);
        order.insert(0, component);

        true
    }
}
