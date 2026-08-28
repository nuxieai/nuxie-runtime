use crate::mechanical_port::source::{
    core::CoreHandle,
    layout::{
        layout_enums::{LayoutDirection, LayoutStyleInterpolation},
        layout_participant::LayoutParticipant,
    },
    math::aabb::Aabb,
};

#[derive(Clone, Eq, PartialEq)]
pub struct LayoutNodeKey {
    pub provider: CoreHandle,
    pub index: usize,
}

#[derive(Default)]
pub struct LayoutNodeProviderState {
    layout_constraints: Option<Box<Vec<CoreHandle>>>,
}

impl LayoutNodeProviderState {
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

    fn add_layout_constraint(&mut self, constraint: CoreHandle) {
        let child = self
            .provider_handle()
            .expect("arena-installed layout node provider");
        let state = self.provider_state();
        let constraints = state
            .layout_constraints
            .get_or_insert_with(|| Box::new(Vec::new()));
        assert!(!constraints.contains(&constraint));
        constraints.push(constraint.clone());
        constraint.with_mut(|constraint| {
            constraint.layout_constraint_add_child(child);
        });
    }
}

pub fn with_mut<R>(
    component: &CoreHandle,
    f: impl FnOnce(&mut dyn LayoutNodeProvider) -> R,
) -> Option<R> {
    let provider = component.with(|component| component.layout_provider_handle())??;
    provider
        .with_mut(|provider| provider.as_layout_node_provider_mut().map(f))
        .flatten()
}
