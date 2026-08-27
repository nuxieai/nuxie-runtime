use crate::mechanical_port::source::{
    animation::listener_invocation::ListenerInvocation,
    generated::animation::listener_align_target_base::ListenerAlignTargetBase,
    math::{mat2d::Mat2D, vec2d::Vec2D},
};
pub trait AlignTargetNode {
    fn parent_world(&self) -> Mat2D;
    fn x(&self) -> f32;
    fn y(&self) -> f32;
    fn set_x(&mut self, value: f32);
    fn set_y(&mut self, value: f32);
}
pub trait AlignTargetStateMachine {
    fn resolve_node(&mut self, id: u32) -> Option<&mut dyn AlignTargetNode>;
}
#[derive(Default)]
pub struct ListenerAlignTarget {
    pub base: ListenerAlignTargetBase,
}
impl ListenerAlignTarget {
    pub fn perform(
        &self,
        machine: &mut dyn AlignTargetStateMachine,
        invocation: &ListenerInvocation,
    ) {
        let (position, previous) = invocation
            .as_pointer()
            .map(|p| (p.position, p.previous_position))
            .unwrap_or((Vec2D::new(0.0, 0.0), Vec2D::new(0.0, 0.0)));
        let Some(target) = machine.resolve_node(self.base.target_id()) else {
            return;
        };
        let mut inverse = Mat2D::default();
        if !target.parent_world().invert(&mut inverse) {
            return;
        }
        let local = inverse * position;
        if self.base.preserve_offset() {
            let previous_local = inverse * previous;
            target.set_x(target.x() + local.x - previous_local.x);
            target.set_y(target.y() + local.y - previous_local.y);
        } else {
            target.set_x(local.x);
            target.set_y(local.y);
        }
    }
}
