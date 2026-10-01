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
    // The angle before our write, and the local transform we left behind.
    base_rotation: f32,
    solved_local: Mat2D,
    solved: bool,
    ours: bool,
}

impl BoneChainLink {
    fn holds_our_solve(&self) -> bool {
        self.solved
            && IKConstraint::with_bone(&self.bone, |bone| {
                get_parent_world(bone) * self.solved_local == *bone.world_transform()
            })
    }

    fn record_solve(&mut self) {
        self.solved_local = IKConstraint::with_bone(&self.bone, |bone| *bone.transform());
        self.solved = true;
    }
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
        self.fk_chain.truncate(num_bones);
        for (index, bone) in bones.iter().rev().cloned().enumerate() {
            // C++ resize preserves existing links and their solve history.
            if let Some(link) = self.fk_chain.get_mut(index) {
                link.index = index as i32;
                link.bone = bone;
                link.angle = 0.0;
                continue;
            }
            self.fk_chain.push(BoneChainLink {
                index: index as i32,
                bone,
                angle: 0.0,
                transform_components: TransformComponents::default(),
                parent_world_inverse: Mat2D::default(),
                base_rotation: 0.0,
                solved_local: Mat2D::default(),
                solved: false,
                ours: false,
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

    pub(crate) fn ancestor_bones(&self) -> Vec<CoreHandle> {
        self.fk_chain[..self.fk_chain.len().saturating_sub(1)]
            .iter()
            .map(|link| link.bone.clone())
            .collect()
    }

    pub fn on_dirty(
        &mut self,
        dirt: crate::mechanical_port::source::component_dirt::ComponentDirt,
    ) {
        use crate::mechanical_port::source::component_dirt::ComponentDirt;
        if (dirt & !ComponentDirt::COLLAPSED) != ComponentDirt::RENDER_OPACITY {
            self.mark_constraint_dirty();
        }
    }

    pub fn strength_changed(&mut self) {
        self.mark_constraint_dirty();
    }

    pub(crate) fn set_strength_occurrence(owner: &CoreHandle, value: f32) -> bool {
        use crate::mechanical_port::source::generated::constraints::constraint_base::ConstraintBase;
        let Some(changed) = owner.with_downcast_mut::<Self, _>(|constraint| {
            constraint
                .base
                .base
                .base
                .base
                .base
                .set_strength_value(value)
        }) else {
            return false;
        };
        if changed {
            super::constraint::Constraint::mark_constraint_dirty_occurrence(owner);
            owner.with_mut(|owner| {
                owner
                    .core_mut()
                    .notify_property_changed(ConstraintBase::STRENGTH_PROPERTY_KEY)
            });
        }
        true
    }

    pub(crate) fn set_invert_direction_occurrence(owner: &CoreHandle, value: bool) -> bool {
        let Some(changed) = owner.with_downcast_mut::<Self, _>(|constraint| {
            constraint.base.set_invert_direction_value(value)
        }) else {
            return false;
        };
        if changed {
            super::constraint::Constraint::mark_constraint_dirty_occurrence(owner);
            owner.with_mut(|owner| {
                owner
                    .core_mut()
                    .notify_property_changed(IKConstraintBase::INVERT_DIRECTION_PROPERTY_KEY)
            });
        }
        true
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
        // Pinned production C++ rounds b*b, then contracts each remaining
        // numerator product in source order; denominators stay separate.
        let angle_a = (c.mul_add(c, (-a).mul_add(a, b * b)) / (2.0 * b * c))
            .min(1.0)
            .max(-1.0)
            .acos();
        let angle_c = ((-c).mul_add(c, a.mul_add(a, b * b)) / (2.0 * a * b))
            .min(1.0)
            .max(-1.0)
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
            Self::with_bone_mut(&b2, |bone| bone.compose_world_transform());
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
                transform[2] = transform[0].mul_add(skew, transform[2]);
                transform[3] = transform[1].mul_add(skew, transform[3]);
            }
            bone.compose_world_transform();
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
        // The pose to blend from: whatever ran before us leaves its work here.
        for link in &mut self.fk_chain {
            let (parent_world_inverse, transform_components) =
                Self::with_bone(&link.bone, |bone| {
                    let parent_world_inverse = get_parent_world(bone).invert_or_identity();
                    (
                        parent_world_inverse,
                        (parent_world_inverse * *bone.world_transform()).decompose(),
                    )
                });
            link.parent_world_inverse = parent_world_inverse;
            link.transform_components = transform_components;
            link.ours = link.holds_our_solve();
            if link.ours {
                // Blending from our own angle would creep toward a full solve.
                link.transform_components.set_rotation(link.base_rotation);
            }
            link.base_rotation = link.transform_components.rotation();
        }

        // Longer chains solve from the pose they stand in, so put ours back first.
        let mut rebuilt = false;
        for index in 0..self.fk_chain.len() {
            rebuilt = rebuilt || self.fk_chain[index].ours;
            if rebuilt {
                self.fk_chain[index].parent_world_inverse =
                    Self::with_bone(&self.fk_chain[index].bone, |bone| {
                        get_parent_world(bone).invert_or_identity()
                    });
                self.constrain_rotation(
                    index,
                    self.fk_chain[index].transform_components.rotation(),
                );
            }
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
                let angle = diff.mul_add(self.base.strength(), from_angle);
                self.constrain_rotation(index, angle);
            }
        }
        // Distinguish our output from a rebuilt pose on the next solve.
        for link in &mut self.fk_chain {
            link.record_solve();
        }
    }
}
