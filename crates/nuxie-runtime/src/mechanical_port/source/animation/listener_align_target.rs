use crate::mechanical_port::source::{
    animation::{
        listener_invocation::ListenerInvocation, state_machine_instance::StateMachineInstance,
    },
    generated::{
        animation::listener_align_target_base::ListenerAlignTargetBase,
        core_registry::CoreRegistry, node_base::NodeBase,
    },
    math::{mat2d::Mat2D, vec2d::Vec2D},
};
#[derive(Default)]
pub struct ListenerAlignTarget {
    pub base: ListenerAlignTargetBase,
}
impl ListenerAlignTarget {
    pub fn perform(&self, machine: &mut StateMachineInstance, invocation: &ListenerInvocation) {
        let (position, previous) = invocation
            .as_pointer()
            .map(|p| (p.position, p.previous_position))
            .unwrap_or((Vec2D::new(0.0, 0.0), Vec2D::new(0.0, 0.0)));
        let Some(target) = machine.resolve_artboard_object(self.base.target_id()) else {
            return;
        };
        let Some(parent_world) = target
            .with(|target| {
                target.as_node()?;
                target
                    .as_transform_component()
                    .map(crate::mechanical_port::source::constraints::constraint::get_parent_world)
            })
            .flatten()
        else {
            return;
        };
        let mut inverse = Mat2D::default();
        if !parent_world.invert(&mut inverse) {
            return;
        }
        let local = inverse * position;
        if self.base.preserve_offset() {
            let previous_local = inverse * previous;
            let x = target
                .with(|target| {
                    target
                        .as_layout_component()
                        .map(|layout| layout.layout_x())
                        .or_else(|| target.as_node().map(|node| node.base.x()))
                })
                .flatten()
                .expect("an align target remains a Node");
            CoreRegistry::set_double_handle(
                &target,
                NodeBase::X_PROPERTY_KEY as i32,
                x + local.x - previous_local.x,
            );
            let y = target
                .with(|target| {
                    target
                        .as_layout_component()
                        .map(|layout| layout.layout_y())
                        .or_else(|| target.as_node().map(|node| node.base.y()))
                })
                .flatten()
                .expect("an align target remains a Node");
            CoreRegistry::set_double_handle(
                &target,
                NodeBase::Y_PROPERTY_KEY as i32,
                y + local.y - previous_local.y,
            );
        } else {
            CoreRegistry::set_double_handle(&target, NodeBase::X_PROPERTY_KEY as i32, local.x);
            CoreRegistry::set_double_handle(&target, NodeBase::Y_PROPERTY_KEY as i32, local.y);
        }
    }
}
