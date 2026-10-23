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
    /// Refresh a layout boundary without consulting any borrowed ancestor.
    /// Content inherits this mode; nested boxes and participant hosts keep
    /// their own context. Attachment initializes children added later.
    pub(crate) fn refresh_picture_layout_mode(&mut self, css: bool) {
        self.base.set_picture_layout_mode(css);
        self.refresh_attached_picture_layout_mode();
    }

    /// Attachment can move an existing drawing group, not only an empty one.
    pub(crate) fn refresh_attached_picture_layout_mode(&mut self) {
        crate::source::layout::picture_layout_mode::children(
            self.children.clone(),
            self.picture_uses_css_layout(),
            self.base.base.base.handle(),
        );
    }

    pub(crate) fn refresh_picture_children(
        children: &[CoreHandle],
        css: bool,
        parent: Option<&CoreHandle>,
    ) {
        for child in children {
            if Some(child) == parent
                || !child.is_type_of(ContainerComponentBase::TYPE_KEY)
                || child.is_type_of(
                    crate::source::layout::layout_participant::LayoutParticipant::TYPE_KEY,
                )
                || child.is_type_of(
                    crate::source::generated::layout_component_base::LayoutComponentBase::TYPE_KEY,
                )
            {
                continue;
            }
            let descendants = child
                .with_mut(|object| {
                    let component = object.as_component_mut()?;
                    // onAddedDirty can attach an existing occurrence again.
                    // Only its resolved parent supplies the inherited mode.
                    if component.parent_handle().as_ref() != parent
                        || component.owns_picture_layout_mode()
                    {
                        return None;
                    }
                    if component.inherit_picture_layout_mode(css) {
                        if let Some(sizeable) = object.as_intrinsically_sizeable_mut() {
                            sizeable.layout_mode_changed();
                        }
                    }
                    object
                        .as_container_component()
                        .map(|container| container.children().to_vec())
                })
                .flatten();
            if let Some(descendants) = descendants {
                Self::refresh_picture_children(&descendants, css, Some(child));
            }
        }
    }

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
