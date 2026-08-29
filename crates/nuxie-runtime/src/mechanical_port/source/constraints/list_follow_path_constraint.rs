use crate::mechanical_port::source::{
    constraints::{constrainable_list::ConstrainableList, list_constraint::ListConstraint},
    core::CoreObject,
    generated::core_registry::CoreCapabilities,
    math::{mat2d::Mat2D, transform_components::TransformComponents},
};

pub use crate::mechanical_port::source::generated::constraints::list_follow_path_constraint_base::ListFollowPathConstraintBase;

#[derive(Default)]
pub struct ListFollowPathConstraint {
    pub base: ListFollowPathConstraintBase,
}

impl ListFollowPathConstraint {
    pub fn distance_end_changed(&mut self) {
        self.base.mark_constraint_dirty();
    }

    pub fn distance_offset_changed(&mut self) {
        self.base.mark_constraint_dirty();
    }

    fn constrain_at_offset(
        &mut self,
        component_transform: &Mat2D,
        parent_transform: &Mat2D,
        component_offset: f32,
    ) -> TransformComponents {
        let Some(target) = self.base.target() else {
            return TransformComponents::default();
        };
        let target_collapsed = target
            .with(|target| {
                target
                    .as_transform_component()
                    .expect("validated ListFollowPathConstraint target")
                    .is_collapsed()
            })
            .expect("ListFollowPathConstraint retains a live target");
        if target_collapsed {
            return TransformComponents::default();
        }
        let mut transform_b = self.base.target_transform(component_offset);
        self.base
            .constrain_helper(component_transform, &mut transform_b, parent_transform)
    }

    pub fn build_dependencies(&mut self) {
        self.base.build_dependencies();
        let Some(constraint) = self.core().handle() else {
            return;
        };
        let Some(parent) = self.component_parent_handle() else {
            return;
        };
        parent.with_mut(|parent| {
            if let Some(list) = parent.as_constrainable_list_mut() {
                list.add_list_constraint(constraint);
            }
        });
    }
}

impl ListConstraint for ListFollowPathConstraint {
    fn constrain_list(&mut self, list: &mut dyn ConstrainableList) {
        let list_transform = *list.list_transform();
        let mut count = 0usize;
        list.for_each_list_item_transform(&mut |_| count += 1);
        let start_offset = self.base.distance_offset() + self.base.distance();
        let start_to_end_distance = self.base.distance_end() - self.base.distance();
        let offset_distance = if count <= 1 {
            0.0
        } else {
            start_to_end_distance / (count as f32 - 1.0)
        };
        let mut index = 0usize;
        list.for_each_list_item_transform(&mut |transform| {
            let components = self.constrain_at_offset(
                transform,
                &list_transform,
                start_offset + index as f32 * offset_distance,
            );
            let transform_b = Mat2D::compose(&components);
            transform.set_xx(transform_b.xx());
            transform.set_xy(transform_b.xy());
            transform.set_yx(transform_b.yx());
            transform.set_yy(transform_b.yy());
            transform.set_tx(transform_b.tx());
            transform.set_ty(transform_b.ty());
            index += 1;
        });
    }
}
