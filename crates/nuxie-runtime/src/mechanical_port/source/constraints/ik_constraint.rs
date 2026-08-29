use crate::mechanical_port::source::{
    bones::bone::Bone,
    constraints::constraint::get_parent_world,
    core::CoreHandle,
    core_context::{CoreContext, StatusCode},
    generated::{
        constraints::ik_constraint_base::IKConstraintBase, core_registry::CoreCapabilities,
    },
    math::{mat2d::Mat2D, math_types, transform_components::TransformComponents, vec2d::Vec2D},
};

struct BoneChainLink {
    index: i32,
    bone: CoreHandle,
    angle: f32,
    transform_components: TransformComponents,
    parent_world_inverse: Mat2D,
}

/// Pinned C++ `IKConstraint`; `IkConstraint` remains as the generated Rust
/// spelling used by the mechanical registry.
#[derive(Default)]
pub struct IKConstraint {
    pub base: IKConstraintBase,
    fk_chain: Vec<BoneChainLink>,
}

pub type IkConstraint = IKConstraint;

fn atan2(v: Vec2D) -> f32 {
    v.y.atan2(v.x)
}

impl IKConstraint {
    fn this_handle(&self) -> Option<CoreHandle> {
        self.base.handle()
    }

    fn with_bone<R>(bone: &CoreHandle, use_bone: impl FnOnce(&Bone) -> R) -> R {
        bone.with(|bone| {
            use_bone(
                bone.as_bone()
                    .expect("IKConstraint chain handles remain Bone-derived"),
            )
        })
        .expect("IKConstraint chain retains live Bones")
    }

    fn with_bone_mut<R>(bone: &CoreHandle, use_bone: impl FnOnce(&mut Bone) -> R) -> R {
        bone.with_mut(|bone| {
            use_bone(
                bone.as_bone_mut()
                    .expect("IKConstraint chain handles remain Bone-derived"),
            )
        })
        .expect("IKConstraint chain retains live Bones")
    }

    pub fn build_dependencies(&mut self) {
        self.base.build_dependencies();
        if let (Some(target), Some(this)) = (self.base.target(), self.this_handle()) {
            target
                .with_mut(|target| target.component_add_dependent(this))
                .filter(|added| *added)
                .expect("validated IKConstraint target is a TransformComponent");
        }
    }

    pub fn on_added_clean(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        let Some(tip) = self.base.parent_handle() else {
            return StatusCode::InvalidObject;
        };
        if !tip.is_type_of(
            crate::mechanical_port::source::generated::bones::bone_base::BoneBase::TYPE_KEY,
        ) {
            return StatusCode::InvalidObject;
        }
        let this = self
            .this_handle()
            .expect("IKConstraint is arena-owned before onAddedClean");
        let mut bone_count = self.base.parent_bone_count();
        let mut bone = tip.clone();
        let mut bones = vec![bone.clone()];
        loop {
            let parent = bone.with(|bone| bone.component_parent_handle()).flatten();
            let Some(parent) = parent else {
                break;
            };
            if !parent.is_type_of(
                crate::mechanical_port::source::generated::bones::bone_base::BoneBase::TYPE_KEY,
            ) || bone_count == 0
            {
                break;
            }
            bone_count -= 1;
            Self::with_bone_mut(&parent, |bone| bone.add_peer_constraint(this.clone()));
            bone = parent;
            bones.push(bone.clone());
        }

        let num_bones = bones.len();
        self.fk_chain.clear();
        self.fk_chain.reserve(num_bones);
        for (index, bone) in bones.iter().rev().cloned().enumerate() {
            self.fk_chain.push(BoneChainLink {
                index: index as i32,
                bone,
                angle: 0.0,
                transform_components: TransformComponents::default(),
                parent_world_inverse: Mat2D::default(),
            });
        }

        for index in 1..num_bones {
            let ancestor = &bones[index];
            let chain_child = &bones[index - 1];
            let children = ancestor
                .with(|ancestor| {
                    ancestor
                        .as_bone()
                        .expect("IK ancestor remains Bone-derived")
                        .children()
                        .to_vec()
                })
                .expect("IKConstraint chain retains live Bones");
            for child in children {
                let is_transform = child
                    .with(|child| child.as_transform_component().is_some())
                    .unwrap_or(false);
                if !is_transform || child == *chain_child {
                    continue;
                }
                Self::with_bone_mut(&tip, |tip| tip.add_dependent(child));
            }
        }
        self.base.on_added_clean(context)
    }

    pub fn mark_constraint_dirty(&mut self) {
        self.base.mark_constraint_dirty();
        let length = self.fk_chain.len().saturating_sub(1);
        for link in &self.fk_chain[..length] {
            Self::with_bone_mut(&link.bone, |bone| bone.mark_transform_dirty());
        }
    }

    fn solve1(&mut self, first: usize, world_target_translation: Vec2D) {
        let inverse_world = self.fk_chain[first].parent_world_inverse;
        let p_a = Self::with_bone(&self.fk_chain[first].bone, |bone| bone.world_translation());
        let to_target = world_target_translation - p_a;
        let to_target_local = Vec2D::transform_dir(to_target, &inverse_world);
        let rotation = atan2(to_target_local);
        self.constrain_rotation(first, rotation);
        self.fk_chain[first].angle = rotation;
    }

    fn solve2(&mut self, first: usize, second: usize, world_target_translation: Vec2D) {
        let b1 = self.fk_chain[first].bone.clone();
        let b2 = self.fk_chain[second].bone.clone();
        let first_child_index = self.fk_chain[first].index as usize + 1;
        let first_child = self.fk_chain[first_child_index].bone.clone();
        let inverse_world = self.fk_chain[first].parent_world_inverse;
        let mut p_a = Self::with_bone(&b1, |bone| bone.world_translation());
        let mut p_c = Self::with_bone(&first_child, |bone| bone.world_translation());
        let mut p_b = Self::with_bone(&b2, Bone::tip_world_translation);
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
        let angle_a = ((-a * a + b * b + c * c) / (2.0 * b * c))
            .clamp(-1.0, 1.0)
            .acos();
        let angle_c = ((a * a + b * b - c * c) / (2.0 * a * b))
            .clamp(-1.0, 1.0)
            .acos();
        let b2_parent = b2.with(|bone| bone.component_parent_handle()).flatten();
        let (r1, r2) = if b2_parent.as_ref() != Some(&b1) {
            let second_child_index = self.fk_chain[first].index as usize + 2;
            let second_child_inverse = self.fk_chain[second_child_index].parent_world_inverse;
            p_c = Self::with_bone(&first_child, |bone| bone.world_translation());
            p_b = Self::with_bone(&b2, Bone::tip_world_translation);
            let av_local = Vec2D::transform_dir(p_b - p_c, &second_child_inverse);
            let angle_correction = -atan2(av_local);
            if self.base.invert_direction() {
                (
                    atan2(cv) - angle_a,
                    -angle_c + math_types::PI + angle_correction,
                )
            } else {
                (
                    angle_a + atan2(cv),
                    angle_c - math_types::PI + angle_correction,
                )
            }
        } else if self.base.invert_direction() {
            (atan2(cv) - angle_a, -angle_c + math_types::PI)
        } else {
            (angle_a + atan2(cv), angle_c - math_types::PI)
        };
        self.constrain_rotation(first, r1);
        self.constrain_rotation(first_child_index, r2);
        if first_child_index != second {
            Self::with_bone_mut(&b2, |bone| {
                *bone.mutable_world_transform() = get_parent_world(bone) * *bone.transform();
            });
        }
        self.fk_chain[first].angle = r1;
        self.fk_chain[first_child_index].angle = r2;
    }

    pub fn invert_direction_changed(&mut self) {
        self.mark_constraint_dirty();
    }

    fn constrain_rotation(&mut self, index: usize, rotation: f32) {
        let bone = self.fk_chain[index].bone.clone();
        let components = self.fk_chain[index].transform_components;
        Self::with_bone_mut(&bone, |bone| {
            let parent_world = get_parent_world(bone);
            let transform = bone.mutable_transform();
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
        });
    }

    // Upstream receives but does not dereference this component. Retaining its
    // identity avoids borrowing the tip Bone across the in-place chain solve.
    pub fn constrain(&mut self, _component: &CoreHandle) {
        let Some(target) = self.base.target() else {
            return;
        };
        let (target_collapsed, world_target_translation) = target
            .with(|target| {
                let target = target
                    .as_transform_component()
                    .expect("validated IKConstraint target");
                (target.is_collapsed(), target.world_translation())
            })
            .expect("IKConstraint retains a live target");
        if target_collapsed {
            return;
        }
        for link in &mut self.fk_chain {
            let (parent_world_inverse, transform_components) =
                Self::with_bone_mut(&link.bone, |bone| {
                    let parent_world_inverse = get_parent_world(bone).invert_or_identity();
                    let world_transform = *bone.world_transform();
                    let bone_transform = bone.mutable_transform();
                    *bone_transform = parent_world_inverse * world_transform;
                    (parent_world_inverse, bone_transform.decompose())
                });
            link.parent_world_inverse = parent_world_inverse;
            link.transform_components = transform_components;
        }
        let count = self.fk_chain.len();
        assert!(
            count > 0,
            "IKConstraint onAddedClean establishes a non-empty FK chain"
        );
        match count {
            1 => self.solve1(0, world_target_translation),
            2 => self.solve2(0, 1, world_target_translation),
            _ => {
                let last = count - 1;
                for index in 0..last {
                    self.solve2(index, last, world_target_translation);
                    let start = self.fk_chain[index].index as usize + 1;
                    for child in start..self.fk_chain.len() - 1 {
                        self.fk_chain[child].parent_world_inverse =
                            Self::with_bone(&self.fk_chain[child].bone, |bone| {
                                get_parent_world(bone).invert_or_identity()
                            });
                    }
                }
            }
        }
        if self.base.strength() != 1.0 {
            for index in 0..self.fk_chain.len() {
                let from_angle =
                    self.fk_chain[index].transform_components.rotation() % (math_types::PI * 2.0);
                let to_angle = self.fk_chain[index].angle % (math_types::PI * 2.0);
                let mut diff = to_angle - from_angle;
                if diff > math_types::PI {
                    diff -= math_types::PI * 2.0;
                } else if diff < -math_types::PI {
                    diff += math_types::PI * 2.0;
                }
                let angle = from_angle + diff * self.base.strength();
                self.constrain_rotation(index, angle);
            }
        }
    }
}
