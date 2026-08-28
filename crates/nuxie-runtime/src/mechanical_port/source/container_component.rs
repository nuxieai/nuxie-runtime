use crate::mechanical_port::source::{
    component::Component,
    core::CoreHandle,
    generated::{
        container_component_base::ContainerComponentBase, core_registry::CoreCapabilities,
    },
    math::vec2d::Vec2D,
};

pub struct ContainerComponent {
    pub base: ContainerComponentBase,
    children: Vec<CoreHandle>,
}

impl Default for ContainerComponent {
    fn default() -> Self {
        Self {
            base: ContainerComponentBase::default(),
            children: Vec::new(),
        }
    }
}

impl ContainerComponent {
    pub fn children(&self) -> &[CoreHandle] {
        &self.children
    }

    pub fn add_child(&mut self, component: CoreHandle) {
        self.children.push(component);
    }

    pub fn collapse(&mut self, value: bool) -> bool {
        CoreCapabilities::component_collapse(self, value)
    }

    pub(crate) fn collapse_after_component(&mut self, value: bool) {
        for child in self.children.iter().cloned() {
            child.with_mut(|child| {
                child.component_collapse(value);
            });
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
