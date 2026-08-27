//! Mechanical owner for pinned `src/dependency_sorter.cpp`.

use std::collections::BTreeSet;

use crate::{components::ComponentHandle, objects::InstanceObjectArena};

#[derive(Default)]
pub(crate) struct DependencySorter {
    permanent: BTreeSet<ComponentHandle>,
    temporary: BTreeSet<ComponentHandle>,
}

impl DependencySorter {
    pub(crate) fn sort(
        &mut self,
        root: ComponentHandle,
        objects: &InstanceObjectArena,
    ) -> (Vec<ComponentHandle>, bool) {
        let mut order = Vec::new();
        let complete = self.visit(root, objects, &mut order);
        (order, complete)
    }

    #[allow(dead_code)]
    pub(crate) fn sort_roots(
        &mut self,
        roots: impl IntoIterator<Item = ComponentHandle>,
        objects: &InstanceObjectArena,
    ) -> Vec<ComponentHandle> {
        let mut order = Vec::new();
        for root in roots {
            let _ = self.visit(root, objects, &mut order);
        }
        order
    }

    fn visit(
        &mut self,
        component: ComponentHandle,
        objects: &InstanceObjectArena,
        order: &mut Vec<ComponentHandle>,
    ) -> bool {
        if self.permanent.contains(&component) {
            return true;
        }
        if self.temporary.contains(&component) {
            eprintln!("Dependency cycle!");
            return false;
        }

        self.temporary.insert(component);

        let dependent_count = objects.dependent_len(component);
        for index in 0..dependent_count {
            let Some(dependent) = objects.dependent_at(component, index) else {
                continue;
            };
            if !self.visit(dependent, objects, order) {
                return false;
            }
        }
        self.permanent.insert(component);
        order.insert(0, component);

        true
    }
}
