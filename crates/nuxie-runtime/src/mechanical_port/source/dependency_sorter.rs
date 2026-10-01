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
        dependents: impl IntoIterator<Item = ComponentOccurrenceHandle>,
        order: &mut Vec<ComponentOccurrenceHandle>,
    ) {
        order.clear();
        self.temp.insert(root.clone());
        for dependent in dependents {
            if !self.visit(dependent, order) {
                order.reverse();
                return;
            }
        }
        self.perm.insert(root.clone());
        order.push(root);
        order.reverse();
    }
    pub fn sort(
        &mut self,
        root: ComponentOccurrenceHandle,
        order: &mut Vec<ComponentOccurrenceHandle>,
    ) {
        order.clear();
        self.visit(root, order);
        order.reverse();
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
        order.reverse();
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

        // Borrow the dependent list throughout traversal rather than copying
        // every node's vector. Recursive visits only mutate sorter state.
        let visited = component
            .with_component(|component| {
                for dependent in component.dependents() {
                    if !self.visit(dependent.clone(), order) {
                        return false;
                    }
                }
                true
            })
            .unwrap_or(true);
        if !visited {
            return false;
        }
        self.perm.insert(component.clone());
        // Append in finish order; each sort entrypoint reverses exactly once.
        order.push(component);

        true
    }
}
