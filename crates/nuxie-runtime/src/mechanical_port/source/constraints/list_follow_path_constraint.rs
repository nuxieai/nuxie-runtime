use crate::mechanical_port::source::{
    constraints::{
        constrainable_list::{self, ConstrainableList},
        list_constraint::ListConstraint,
    },
    math::{mat2d::Mat2D, transform_components::TransformComponents},
};

pub use crate::mechanical_port::source::generated::constraints::list_follow_path_constraint_base::ListFollowPathConstraintBase;

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
        let Some(target) = self.base.target_mut() else {
            return TransformComponents::default();
        };
        if target.is_collapsed() {
            return TransformComponents::default();
        }
        let mut transform_b = self.base.target_transform(component_offset);
        self.base
            .constrain_helper(component_transform, &mut transform_b, parent_transform)
    }

    pub fn build_dependencies(&mut self) {
        self.base.build_dependencies();
        if let Some(list) = constrainable_list::from(self.base.parent_mut()) {
            list.add_list_constraint(self as *mut Self as *mut dyn ListConstraint);
        }
    }
}

impl ListConstraint for ListFollowPathConstraint {
    fn constrain_list(&mut self, list: &mut dyn ConstrainableList) {
        let list_transform = *list.list_transform();
        let mut transforms = Vec::new();
        list.list_item_transforms(&mut transforms);
        let count = transforms.len();
        let start_offset = self.base.distance_offset() + self.base.distance();
        let start_to_end_distance = self.base.distance_end() - self.base.distance();
        let offset_distance = if count <= 1 {
            0.0
        } else {
            start_to_end_distance / (count as f32 - 1.0)
        };
        for (index, transform) in transforms.into_iter().enumerate() {
            let components = self.constrain_at_offset(
                transform,
                &list_transform,
                start_offset + index as f32 * offset_distance,
            );
            let transform_b = Mat2D::compose(components);
            transform.set_xx(transform_b.xx());
            transform.set_xy(transform_b.xy());
            transform.set_yx(transform_b.yx());
            transform.set_yy(transform_b.yy());
            transform.set_tx(transform_b.tx());
            transform.set_ty(transform_b.ty());
        }
    }
}
