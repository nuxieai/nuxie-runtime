use crate::mechanical_port::source::{
    constraints::layout_constraint::LayoutConstraint,
    core::CoreHandle,
    generated::{
        artboard_component_list_base::ArtboardComponentListBase,
        layout_component_base::LayoutComponentBase,
        nested_artboard_layout_base::NestedArtboardLayoutBase,
        shapes::{image_base::ImageBase, shape_base::ShapeBase},
        text::text_base::TextBase,
    },
    layout::layout_enums::{LayoutDirection, LayoutStyleInterpolation},
    math::aabb::Aabb,
};
use std::{cell::RefCell, rc::Rc};

#[derive(Clone)]
pub struct LayoutNodeKey {
    pub provider: CoreHandle,
    pub index: usize,
    pub owner: Rc<RefCell<Option<CoreHandle>>>,
}

impl PartialEq for LayoutNodeKey {
    fn eq(&self, other: &Self) -> bool {
        self.provider == other.provider && self.index == other.index
    }
}
impl Eq for LayoutNodeKey {}

pub fn layout_node_for(provider: &CoreHandle, index: usize) -> Option<LayoutNodeKey> {
    provider
        .with(|object| {
            if let Some(layout) = object.as_layout_component() {
                layout.layout_node_key(index)
            } else if let Some(nested) = object
                .as_any()
                .downcast_ref::<crate::mechanical_port::source::nested_artboard_layout::NestedArtboardLayout>()
            {
                nested.layout_node(index as i32)
            } else if let Some(list) = object
                .as_any()
                .downcast_ref::<crate::mechanical_port::source::artboard_component_list::ArtboardComponentList>()
            {
                list.layout_node(index as i32)
            } else {
                let participant = object
                    .as_any()
                    .downcast_ref::<crate::mechanical_port::source::layout::layout_participant::LayoutParticipant>()
                    .expect("a native layout provider owns a layout node");
                participant.layout_node_key(index)
            }
        })
        .flatten()
}

/// Resolve the native node represented by a provider occurrence. Hosted
/// artboards expose their root layout node through the host provider rather
/// than through the root LayoutComponent itself.
pub fn layout_node_owner_for(key: &LayoutNodeKey) -> Option<CoreHandle> {
    key.provider
        .with(|object| {
            if object.as_layout_component().is_some()
                || object.as_any().is::<
                    crate::mechanical_port::source::layout::layout_participant::LayoutParticipant,
                >()
            {
                assert_eq!(key.index, 0);
                Some(key.provider.clone())
            } else if let Some(nested) = object.as_any().downcast_ref::<
                crate::mechanical_port::source::nested_artboard_layout::NestedArtboardLayout,
            >() {
                nested
                    .base
                    .base
                    .artboard_instance_handle(key.index as i32)
                    .map(|instance| instance.core_handle())
            } else if let Some(list) = object.as_any().downcast_ref::<
                crate::mechanical_port::source::artboard_component_list::ArtboardComponentList,
            >() {
                list.item(key.index as i32)
                    .map(|instance| instance.core_handle())
            } else {
                None
            }
        })
        .flatten()
}

#[derive(Default)]
pub struct LayoutNodeProviderState {
    layout_constraints: Option<Box<Vec<CoreHandle>>>,
    node_owners: RefCell<Vec<Rc<RefCell<Option<CoreHandle>>>>>,
}

impl LayoutNodeProviderState {
    pub fn node_key(&self, provider: CoreHandle, index: usize) -> LayoutNodeKey {
        let mut owners = self.node_owners.borrow_mut();
        while owners.len() <= index {
            owners.push(Rc::new(RefCell::new(None)));
        }
        LayoutNodeKey {
            provider,
            index,
            owner: owners[index].clone(),
        }
    }
    pub fn layout_constraints(&self) -> &[CoreHandle] {
        self.layout_constraints
            .as_deref()
            .map_or(&[], Vec::as_slice)
    }
}

pub trait LayoutNodeProvider {
    fn provider_state(&mut self) -> &mut LayoutNodeProviderState;
    /// Identity of this exact provider occurrence. This is distinct from the
    /// authored component represented by a separate LayoutParticipant child.
    fn provider_handle(&self) -> Option<CoreHandle>;
    fn owner_handle(&self) -> Option<CoreHandle>;
    fn layout_bounds(&self) -> Aabb;
    fn layout_bounds_for_node(&self, _index: usize) -> Aabb {
        self.layout_bounds()
    }
    fn sync_style_changes(&mut self) -> bool {
        false
    }
    fn update_layout_bounds(&mut self, _animate: bool) {}
    fn mark_layout_node_dirty(&mut self, _should_force_update_layout_bounds: bool) {}
    fn add_layout_style_applier(&mut self, _applier: CoreHandle) {}
    fn num_layout_nodes(&self) -> usize;
    fn cascade_layout_style(
        &mut self,
        _interpolation: LayoutStyleInterpolation,
        _interpolator: Option<CoreHandle>,
        _time: f32,
        _direction: LayoutDirection,
    ) -> bool {
        false
    }

    fn add_layout_constraint(&mut self, constraint: &mut dyn LayoutConstraint) {
        let child = self
            .provider_handle()
            .expect("arena-installed layout node provider");
        let state = self.provider_state();
        let constraints = state
            .layout_constraints
            .get_or_insert_with(|| Box::new(Vec::new()));
        let constraint_handle = constraint.constraint_handle();
        assert!(!constraints.contains(&constraint_handle));
        constraints.push(constraint_handle);
        constraint.add_layout_child(child);
    }
}

pub fn from_component(component: &CoreHandle) -> Option<CoreHandle> {
    match component.core_type()? {
        LayoutComponentBase::TYPE_KEY
        | NestedArtboardLayoutBase::TYPE_KEY
        | ArtboardComponentListBase::TYPE_KEY => Some(component.clone()),
        TextBase::TYPE_KEY | ImageBase::TYPE_KEY | ShapeBase::TYPE_KEY => {
            component.with(|component| component.layout_provider_handle())?
        }
        _ => None,
    }
}

pub fn with_mut<R>(
    component: &CoreHandle,
    f: impl FnOnce(&mut dyn LayoutNodeProvider) -> R,
) -> Option<R> {
    let provider = from_component(component)?;
    provider
        .with_mut(|provider| provider.as_layout_node_provider_mut().map(f))
        .flatten()
}
