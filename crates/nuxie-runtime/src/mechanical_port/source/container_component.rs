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

    pub fn children_typed<T: crate::mechanical_port::source::core::CoreType>(
        &self,
    ) -> crate::mechanical_port::source::typed_children::TypedChildren<'_, T> {
        crate::mechanical_port::source::typed_children::TypedChildren::from_handles(&self.children)
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

    pub fn for_all(this: &CoreHandle, mut predicate: impl FnMut(CoreHandle) -> bool) -> bool {
        if !predicate(this.clone()) {
            return false;
        }
        Self::for_each_child(this, predicate);
        true
    }

    pub fn for_each_child(this: &CoreHandle, mut predicate: impl FnMut(CoreHandle) -> bool) {
        Self::for_each_child_with(this, &mut predicate);
    }

    fn for_each_child_with(this: &CoreHandle, predicate: &mut impl FnMut(CoreHandle) -> bool) {
        // C++ callbacks may mutate the visited object. Release its arena borrow
        // before invoking them, retaining only the ordered child identities.
        let children = this
            .with(|object| {
                object
                    .as_container_component()
                    .expect("ContainerComponent traversal owner")
                    .children
                    .clone()
            })
            .expect("live ContainerComponent traversal owner");
        for child in children {
            if !predicate(child.clone()) {
                continue;
            }
            if child.is_type_of(ContainerComponentBase::TYPE_KEY) {
                Self::for_each_child_with(&child, predicate);
            }
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
