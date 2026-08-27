use crate::mechanical_port::source::{
    bones::bone::Bone,
    constraints::constraint::get_parent_world,
    core_context::{CoreContext, StatusCode},
    generated::constraints::ik_constraint_base::IkConstraintBase,
    math::{mat2d::Mat2D, math_types, transform_components::TransformComponents, vec2d::Vec2D},
    transform_component::TransformComponent,
};

struct BoneChainLink {
    index: i32,
    bone: *mut Bone,
    angle: f32,
    transform_components: TransformComponents,
    parent_world_inverse: Mat2D,
}

pub struct IkConstraint {
    pub base: IkConstraintBase,
    fk_chain: Vec<BoneChainLink>,
}

fn atan2(v: Vec2D) -> f32 { v.y.atan2(v.x) }

impl IkConstraint {
    pub fn build_dependencies(&mut self) {
        self.base.build_dependencies();
        if let Some(target) = self.base.target_mut() {
            target.add_dependent(self.base.as_component_mut_ptr());
        }
    }

    pub fn on_added_clean(&mut self, context: &mut CoreContext) -> StatusCode {
        if !self.base.parent().is::<Bone>() { return StatusCode::InvalidObject; }
        let mut bone_count = self.base.parent_bone_count();
        let mut bone = self.base.parent_mut().as_mut::<Bone>().unwrap() as *mut Bone;
        let mut bones = vec![bone];
        unsafe {
            while (*bone).parent().is::<Bone>() && bone_count > 0 {
                bone_count -= 1;
                bone = (*bone).parent_mut().as_mut::<Bone>().unwrap() as *mut Bone;
                (*bone).add_peer_constraint(self.base.as_component_mut_ptr());
                bones.push(bone);
            }
        }
        let num_bones = bones.len();
        self.fk_chain.clear();
        self.fk_chain.reserve(num_bones);
        for (index, bone) in bones.iter().rev().copied().enumerate() {
            self.fk_chain.push(BoneChainLink {
                index: index as i32,
                bone,
                angle: 0.0,
                transform_components: TransformComponents::default(),
                parent_world_inverse: Mat2D::default(),
            });
        }
        let tip = self.base.parent_mut().as_mut::<Bone>().unwrap() as *mut Bone;
        for index in 1..num_bones {
            unsafe {
                let ancestor = &mut *bones[index];
                let chain_child = bones[index - 1];
                for child in ancestor.children_mut() {
                    if !child.is::<TransformComponent>()
                        || std::ptr::eq(child as *mut _, chain_child.cast())
                    {
                        continue;
                    }
                    (*tip).add_dependent(child.as_mut::<TransformComponent>().unwrap());
                }
            }
        }
        self.base.on_added_clean(context)
    }

    pub fn mark_constraint_dirty(&mut self) {
        self.base.mark_constraint_dirty();
        let length = self.fk_chain.len().saturating_sub(1);
        for link in &mut self.fk_chain[..length] {
            unsafe { (*link.bone).mark_transform_dirty() };
        }
    }

    fn solve1(&mut self, first: usize, world_target_translation: Vec2D) {
        let fk1 = &mut self.fk_chain[first];
        let inverse_world = fk1.parent_world_inverse;
        let p_a = unsafe { (*fk1.bone).world_translation() };
        let to_target = world_target_translation - p_a;
        let to_target_local = Vec2D::transform_dir(to_target, inverse_world);
        let rotation = atan2(to_target_local);
        self.constrain_rotation(first, rotation);
        self.fk_chain[first].angle = rotation;
    }

    fn solve2(&mut self, first: usize, second: usize, world_target_translation: Vec2D) {
        let b1 = self.fk_chain[first].bone;
        let b2 = self.fk_chain[second].bone;
        let first_child_index = self.fk_chain[first].index as usize + 1;
        let first_child = self.fk_chain[first_child_index].bone;
        let inverse_world = self.fk_chain[first].parent_world_inverse;
        let (mut p_a, mut p_c, mut p_b) = unsafe {
            ((*b1).world_translation(), (*first_child).world_translation(), (*b2).tip_world_translation())
        };
        let mut p_bt = world_target_translation;
        p_a = inverse_world * p_a;
        p_c = inverse_world * p_c;
        p_b = inverse_world * p_b;
        p_bt = inverse_world * p_bt;
        let av = p_b - p_c;
        let bv = p_c - p_a;
        let cv = p_bt - p_a;
        let a = av.length();
        let b = bv.length();
        let c = cv.length();
        let angle_a = ((-a * a + b * b + c * c) / (2.0 * b * c)).clamp(-1.0, 1.0).acos();
        let angle_c = ((a * a + b * b - c * c) / (2.0 * a * b)).clamp(-1.0, 1.0).acos();
        let (r1, r2);
        unsafe {
            if (*b2).parent_ptr() != b1.cast() {
                let second_child_index = self.fk_chain[first].index as usize + 2;
                let second_child_inverse = self.fk_chain[second_child_index].parent_world_inverse;
                p_c = (*first_child).world_translation();
                p_b = (*b2).tip_world_translation();
                let av_local = Vec2D::transform_dir(p_b - p_c, second_child_inverse);
                let angle_correction = -atan2(av_local);
                if self.base.invert_direction() {
                    r1 = atan2(cv) - angle_a;
                    r2 = -angle_c + math_types::PI + angle_correction;
                } else {
                    r1 = angle_a + atan2(cv);
                    r2 = angle_c - math_types::PI + angle_correction;
                }
            } else if self.base.invert_direction() {
                r1 = atan2(cv) - angle_a;
                r2 = -angle_c + math_types::PI;
            } else {
                r1 = angle_a + atan2(cv);
                r2 = angle_c - math_types::PI;
            }
        }
        self.constrain_rotation(first, r1);
        self.constrain_rotation(first_child_index, r2);
        if first_child_index != second {
            unsafe {
                *(*b2).mutable_world_transform() = *get_parent_world(&*b2) * *(*b2).transform();
            }
        }
        self.fk_chain[first].angle = r1;
        self.fk_chain[first_child_index].angle = r2;
    }

    pub fn invert_direction_changed(&mut self) { self.mark_constraint_dirty(); }

    fn constrain_rotation(&mut self, index: usize, rotation: f32) {
        let fk = &mut self.fk_chain[index];
        unsafe {
            let bone = &mut *fk.bone;
            let parent_world = *get_parent_world(bone);
            let transform = bone.mutable_transform();
            let components = fk.transform_components;
            *transform = Mat2D::from_rotation(rotation);
            transform[4] = components.x();
            transform[5] = components.y();
            let scale_x = components.scale_x();
            let scale_y = components.scale_y();
            transform[0] *= scale_x;
            transform[1] *= scale_x;
            transform[2] *= scale_y;
            transform[3] *= scale_y;
            let skew = components.skew();
            if skew != 0.0 {
                transform[2] = transform[0] * skew + transform[2];
                transform[3] = transform[1] * skew + transform[3];
            }
            *bone.mutable_world_transform() = parent_world * *transform;
        }
    }

    pub fn constrain(&mut self, _component: &mut TransformComponent) {
        let Some(target) = self.base.target() else { return; };
        if target.is_collapsed() { return; }
        let world_target_translation = target.world_translation();
        for link in &mut self.fk_chain {
            unsafe {
                let bone = &mut *link.bone;
                let parent_world = get_parent_world(bone);
                link.parent_world_inverse = parent_world.invert_or_identity();
                let bone_transform = bone.mutable_transform();
                *bone_transform = link.parent_world_inverse * *bone.world_transform();
                link.transform_components = bone_transform.decompose();
            }
        }
        let count = self.fk_chain.len();
        match count {
            1 => self.solve1(0, world_target_translation),
            2 => self.solve2(0, 1, world_target_translation),
            _ => {
                let last = count - 1;
                for index in 0..last {
                    self.solve2(index, last, world_target_translation);
                    let start = self.fk_chain[index].index as usize + 1;
                    for child in start..self.fk_chain.len() - 1 {
                        let bone = self.fk_chain[child].bone;
                        self.fk_chain[child].parent_world_inverse = unsafe {
                            get_parent_world(&*bone).invert_or_identity()
                        };
                    }
                }
            }
        }
        if self.base.strength() != 1.0 {
            for index in 0..self.fk_chain.len() {
                let from_angle = self.fk_chain[index].transform_components.rotation() % (math_types::PI * 2.0);
                let to_angle = self.fk_chain[index].angle % (math_types::PI * 2.0);
                let mut diff = to_angle - from_angle;
                if diff > math_types::PI { diff -= math_types::PI * 2.0; }
                else if diff < -math_types::PI { diff += math_types::PI * 2.0; }
                let angle = from_angle + diff * self.base.strength();
                self.constrain_rotation(index, angle);
            }
        }
    }
}
