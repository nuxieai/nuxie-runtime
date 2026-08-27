use crate::mechanical_port::source::{
    artboard_component_list::{ArtboardComponentList, ArtboardComponentListBase},
    component::Component,
    constraints::layout_constraint::LayoutConstraint,
    layout::{
        layout_enums::{LayoutDirection, LayoutStyleInterpolation},
        layout_participant::LayoutParticipant,
        layout_style_applier::LayoutStyleApplier,
    },
    layout_component::LayoutComponent,
    math::aabb::Aabb,
    nested_artboard_layout::NestedArtboardLayout,
    shapes::{
        image::{Image, ImageBase},
        shape::{Shape, ShapeBase},
    },
    text::text::{Text, TextBase},
    transform_component::TransformComponent,
};

#[derive(Default)]
pub struct LayoutNodeProviderState {
    layout_constraints: Option<Box<Vec<*mut dyn LayoutConstraint>>>,
}

pub trait LayoutNodeProvider {
    fn provider_state(&mut self) -> &mut LayoutNodeProviderState;
    #[cfg(feature = "rive-layout")]
    fn layout_node(&mut self, index: i32) -> *mut core::ffi::c_void;
    fn transform_component_mut(&mut self) -> Option<&mut TransformComponent>;
    fn transform_component(&self) -> Option<&TransformComponent>;
    fn layout_bounds(&self) -> Aabb;
    fn layout_bounds_for_node(&self, _index: usize) -> Aabb {
        self.layout_bounds()
    }
    fn sync_style_changes(&mut self) -> bool {
        false
    }
    fn update_layout_bounds(&mut self, _animate: bool) {}
    fn mark_layout_node_dirty(&mut self, _should_force_update_layout_bounds: bool) {}
    #[cfg(feature = "rive-layout")]
    fn add_layout_style_applier(&mut self, _applier: &mut dyn LayoutStyleApplier) {}
    fn num_layout_nodes(&self) -> usize;
    #[cfg(feature = "rive-layout")]
    fn cascade_layout_style(
        &mut self,
        _interpolation: LayoutStyleInterpolation,
        _interpolator: Option<&mut KeyFrameInterpolator>,
        _time: f32,
        _direction: LayoutDirection,
    ) -> bool {
        false
    }

    fn add_layout_constraint(&mut self, constraint: *mut dyn LayoutConstraint) {
        let state = self.provider_state();
        let constraints = state
            .layout_constraints
            .get_or_insert_with(|| Box::new(Vec::new()));
        assert!(!constraints.contains(&constraint));
        constraints.push(constraint);
        unsafe { (*constraint).add_layout_child(self) };
    }
}

pub fn from(component: &mut Component) -> Option<&mut dyn LayoutNodeProvider> {
    match component.core_type() {
        LayoutComponent::TYPE_KEY => component
            .as_mut::<LayoutComponent>()
            .map(|v| v as &mut dyn LayoutNodeProvider),
        NestedArtboardLayout::TYPE_KEY => component
            .as_mut::<NestedArtboardLayout>()
            .map(|v| v as &mut dyn LayoutNodeProvider),
        ArtboardComponentListBase::TYPE_KEY => component
            .as_mut::<ArtboardComponentList>()
            .map(|v| v as &mut dyn LayoutNodeProvider),
        TextBase::TYPE_KEY => component
            .as_mut::<Text>()?
            .layout_participant_mut()
            .map(|v| v as &mut dyn LayoutNodeProvider),
        ImageBase::TYPE_KEY => component
            .as_mut::<Image>()?
            .layout_participant_mut()
            .map(|v| v as &mut dyn LayoutNodeProvider),
        ShapeBase::TYPE_KEY => component
            .as_mut::<Shape>()?
            .layout_participant_mut()
            .map(|v| v as &mut dyn LayoutNodeProvider),
        _ => None,
    }
}
