use crate::mechanical_port::source::{
    component::{Component, ComponentOccurrenceHandle},
    core::CoreHandle,
    generated::{
        container_component_base::ContainerComponentBase, core_registry::CoreCapabilities,
    },
    math::vec2d::Vec2D,
};

pub struct ContainerComponent {
    pub base: ContainerComponentBase,
    children: Vec<CoreHandle>,
    component_children: Vec<ComponentOccurrenceHandle>,
}

impl Default for ContainerComponent {
    fn default() -> Self {
        Self {
            base: ContainerComponentBase::default(),
            children: Vec::new(),
            component_children: Vec::new(),
        }
    }
}

impl ContainerComponent {
    pub fn children(&self) -> &[CoreHandle] {
        &self.children
    }

    pub fn add_child(&mut self, component: CoreHandle) {
        self.component_children.push(component.clone().into());
        self.children.push(component);
    }

    pub(crate) fn add_runtime_child(&mut self, component: ComponentOccurrenceHandle) {
        assert!(component.authored().is_none());
        self.component_children.push(component);
    }

    pub fn component_children(&self) -> &[ComponentOccurrenceHandle] {
        &self.component_children
    }

    pub fn collapse(&mut self, value: bool) -> bool {
        CoreCapabilities::component_collapse(self, value)
    }

    pub(crate) fn collapse_after_component(&mut self, value: bool) {
        for child in self.component_children.iter().cloned() {
            child.collapse(value);
        }
    }

    pub fn for_all(&mut self, mut predicate: impl FnMut(CoreHandle) -> bool) -> bool {
        let Some(this) = self.base.base.base.base.handle() else {
            return false;
        };
        if !predicate(this) {
            return false;
        }
        self.for_each_child(predicate);
        true
    }

    pub fn for_each_child(&mut self, mut predicate: impl FnMut(CoreHandle) -> bool) {
        Self::for_each_child_with(&self.children, &mut predicate);
    }

    fn for_each_child_with(
        children: &[CoreHandle],
        predicate: &mut impl FnMut(CoreHandle) -> bool,
    ) {
        for child in children.iter().cloned() {
            if !predicate(child.clone()) {
                continue;
            }
            child.with_mut(|child| {
                if let Some(container) = child.as_container_component_mut() {
                    Self::for_each_child_with(&container.children, predicate);
                }
            });
        }
    }

    pub fn hit_test_point(
        &self,
        position: &Vec2D,
        skip_on_unclipped: bool,
        is_primary_hit: bool,
    ) -> bool {
        self.base
            .base
            .hit_test_point(position, skip_on_unclipped, is_primary_hit)
    }
}

impl std::ops::Deref for ContainerComponent {
    type Target = ContainerComponentBase;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for ContainerComponent {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
