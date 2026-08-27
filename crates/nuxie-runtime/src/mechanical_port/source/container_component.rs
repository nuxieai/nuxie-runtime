use std::any::Any;

use crate::mechanical_port::source::{
    component::Component, generated::container_component_base::ContainerComponentBase,
    math::vec2d::Vec2D,
};

pub struct ContainerComponent {
    pub base: ContainerComponentBase,
    children: Vec<*mut Component>,
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
    pub fn children_of<T: Any>(&self) -> impl Iterator<Item = &T> {
        self.children.iter().filter_map(|child| {
            let core = unsafe { &**child }.base.base.as_any();
            core.downcast_ref::<T>()
        })
    }

    pub fn first_child<T: Any>(&self) -> Option<&T> {
        self.children_of::<T>().next()
    }

    pub fn children(&self) -> &[*mut Component] {
        &self.children
    }

    pub fn add_child(&mut self, component: *mut Component) {
        self.children.push(component);
    }

    pub fn collapse(&mut self, value: bool) -> bool {
        if !self.base.base.collapse(value) {
            return false;
        }
        for child in self.children.iter().copied() {
            unsafe { &mut *child }.collapse(value);
        }
        true
    }

    pub fn for_all(&mut self, mut predicate: impl FnMut(*mut Component) -> bool) -> bool {
        if !predicate((&mut self.base.base) as *mut Component) {
            return false;
        }
        self.for_each_child(predicate);
        true
    }

    pub fn for_each_child(&mut self, mut predicate: impl FnMut(*mut Component) -> bool) {
        Self::for_each_child_with(&self.children, &mut predicate);
    }

    fn for_each_child_with(
        children: &[*mut Component],
        predicate: &mut impl FnMut(*mut Component) -> bool,
    ) {
        for child in children.iter().copied() {
            if !predicate(child) {
                continue;
            }
            if unsafe { &*child }
                .base
                .base
                .is_type_of(ContainerComponentBase::TYPE_KEY)
            {
                let container = unsafe { &mut *child.cast::<ContainerComponent>() };
                Self::for_each_child_with(&container.children, predicate);
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
