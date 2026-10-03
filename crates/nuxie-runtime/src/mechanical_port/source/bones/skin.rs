use crate::mechanical_port::source::{
    component::{Component, ComponentDirt},
    core::CoreHandle,
    core_context::CoreContext,
    generated::{
        bones::skin_base::{SkinBase, SkinBaseCallbacks},
        component_base::ComponentBaseCallbacks,
    },
    math::mat2d::Mat2D,
    status_code::StatusCode,
};

struct SilentSkinCallbacks;
impl ComponentBaseCallbacks for SilentSkinCallbacks {
    fn notify_property_changed(&mut self, _property_key: u16) {}
}
impl SkinBaseCallbacks for SilentSkinCallbacks {
    fn notify_property_changed(&mut self, _property_key: u16) {}
}

pub struct Skin {
    pub base: SkinBase,
    world_transform: Mat2D,
    tendons: Vec<CoreHandle>,
    bone_transforms: Option<Vec<f32>>,
    skinnable: Option<CoreHandle>,
}

impl Default for Skin {
    fn default() -> Self {
        Self {
            base: SkinBase::default(),
            world_transform: Mat2D::identity(),
            tendons: Vec::new(),
            bone_transforms: None,
            skinnable: None,
        }
    }
}

impl Skin {
    /// The skinnable's world transform at bind time.
    pub fn bind_transform(&self) -> &Mat2D {
        &self.world_transform
    }

    fn component(&self) -> &Component {
        &self.base.base.base
    }

    fn component_mut(&mut self) -> &mut Component {
        &mut self.base.base.base
    }

    fn set_matrix_component(
        &mut self,
        value: f32,
        property_key: u16,
        current: impl FnOnce(&SkinBase) -> f32,
        set: impl FnOnce(&mut SkinBase, f32, &mut SilentSkinCallbacks),
    ) {
        if current(&self.base) == value {
            return;
        }
        let mut callbacks = SilentSkinCallbacks;
        set(&mut self.base, value, &mut callbacks);
        self.component_mut()
            .base
            .base
            .notify_property_changed(property_key);
    }

    pub fn xx(&self) -> f32 {
        self.base.xx()
    }
    pub fn set_xx(&mut self, value: f32) {
        self.set_matrix_component(
            value,
            SkinBase::XX_PROPERTY_KEY,
            SkinBase::xx,
            SkinBase::set_xx,
        )
    }
    pub fn yx(&self) -> f32 {
        self.base.yx()
    }
    pub fn set_yx(&mut self, value: f32) {
        self.set_matrix_component(
            value,
            SkinBase::YX_PROPERTY_KEY,
            SkinBase::yx,
            SkinBase::set_yx,
        )
    }
    pub fn xy(&self) -> f32 {
        self.base.xy()
    }
    pub fn set_xy(&mut self, value: f32) {
        self.set_matrix_component(
            value,
            SkinBase::XY_PROPERTY_KEY,
            SkinBase::xy,
            SkinBase::set_xy,
        )
    }
    pub fn yy(&self) -> f32 {
        self.base.yy()
    }
    pub fn set_yy(&mut self, value: f32) {
        self.set_matrix_component(
            value,
            SkinBase::YY_PROPERTY_KEY,
            SkinBase::yy,
            SkinBase::set_yy,
        )
    }
    pub fn tx(&self) -> f32 {
        self.base.tx()
    }
    pub fn set_tx(&mut self, value: f32) {
        self.set_matrix_component(
            value,
            SkinBase::TX_PROPERTY_KEY,
            SkinBase::tx,
            SkinBase::set_tx,
        )
    }
    pub fn ty(&self) -> f32 {
        self.base.ty()
    }
    pub fn set_ty(&mut self, value: f32) {
        self.set_matrix_component(
            value,
            SkinBase::TY_PROPERTY_KEY,
            SkinBase::ty,
            SkinBase::set_ty,
        )
    }

    pub(crate) fn add_tendon(&mut self, tendon: CoreHandle) {
        self.tendons.push(tendon);
    }

    pub fn tendons(&self) -> &[CoreHandle] {
        &self.tendons
    }

    pub fn on_added_dirty(
        &mut self,
        this: CoreHandle,
        context: &mut dyn CoreContext,
    ) -> StatusCode {
        let code = self.component_mut().on_added_dirty(context);
        if code != StatusCode::Ok {
            return code;
        }
        self.world_transform = Mat2D::new(
            self.base.xx(),
            self.base.xy(),
            self.base.yx(),
            self.base.yy(),
            self.base.tx(),
            self.base.ty(),
        );

        let Some(parent) = context.resolve(self.component().base.parent_id()) else {
            return StatusCode::MissingObject;
        };
        let installed = parent
            .with_mut(|parent| {
                parent.as_skinnable_behavior_mut().map(|skinnable| {
                    skinnable.set_skin(this.clone());
                })
            })
            .flatten()
            .is_some();
        if !installed {
            return StatusCode::MissingObject;
        }
        self.skinnable = Some(parent);
        StatusCode::Ok
    }

    pub fn update(&mut self, _value: ComponentDirt) {
        let bone_transforms = self
            .bone_transforms
            .as_mut()
            .expect("buildDependencies initializes the bone transform buffer");
        let mut transform_index = 6;
        for tendon_handle in self.tendons.iter().cloned() {
            let bone_handle = tendon_handle
                .with_downcast::<crate::mechanical_port::source::bones::tendon::Tendon, _>(
                    |tendon| tendon.bone(),
                )
                .flatten()
                .expect("Tendon::onAddedDirty resolves its Bone before update");
            let (bone_world, inverse_bind) = (
                bone_handle
                    .with(|bone| bone.as_bone().map(|bone| *bone.base.world_transform()))
                    .flatten()
                    .expect("a Tendon Bone handle must remain a Bone"),
                tendon_handle
                    .with_downcast::<crate::mechanical_port::source::bones::tendon::Tendon, _>(
                        |tendon| *tendon.inverse_bind(),
                    )
                    .expect("a retained Tendon must remain a Tendon"),
            );
            let world = bone_world * inverse_bind;
            for coefficient in world.values() {
                bone_transforms[transform_index] = *coefficient;
                transform_index += 1;
            }
        }
    }

    /// 1 when all oriented bones are upright, -1 when all are mirrored,
    /// and 0 when they disagree or none carries an orientation.
    pub fn winding_sign(&self) -> i32 {
        let Some(transforms) = self.bone_transforms.as_ref() else {
            return 0;
        };
        let mut any_mirrored = false;
        let mut any_upright = false;
        let mut transform_index = 0;
        for _tendon in &self.tendons {
            transform_index += 6;
            let transform = &transforms[transform_index..];
            let sign = Self::orientation(transform[0], transform[1], transform[2], transform[3]);
            any_mirrored |= sign < 0;
            any_upright |= sign > 0;
        }
        if any_mirrored == any_upright {
            0
        } else if any_mirrored {
            -1
        } else {
            1
        }
    }

    pub fn orientation(xx: f32, xy: f32, yx: f32, yy: f32) -> i32 {
        let xxyy = xx * yy;
        let xyyx = xy * yx;
        let determinant = xxyy - xyyx;
        if determinant.abs() <= 1e-6_f32 * (xxyy.abs() + xyyx.abs()) {
            return 0;
        }
        if determinant < 0.0 { -1 } else { 1 }
    }

    pub fn build_dependencies(&mut self, this: CoreHandle) {
        for tendon_handle in self.tendons.iter().cloned() {
            let bone_handle = tendon_handle
                .with_downcast::<crate::mechanical_port::source::bones::tendon::Tendon, _>(
                    |tendon| tendon.bone(),
                )
                .flatten()
                .expect("Tendon::onAddedDirty resolves its Bone before dependency building");
            let peer_constraints = bone_handle
                .with(|bone| bone.as_bone().map(|bone| bone.peer_constraints().to_vec()))
                .flatten()
                .expect("a Tendon Bone handle must remain a Bone");
            bone_handle
                .with_mut(|bone| {
                    bone.as_component_mut()
                        .expect("a Tendon Bone handle must remain a Component")
                        .add_dependent(this.clone());
                })
                .expect("a Tendon Bone handle must remain live");
            for constraint_handle in peer_constraints {
                let constraint_parent = constraint_handle
                    .with(|constraint| {
                        constraint
                            .as_component()
                            .expect("a retained Constraint must remain a Component")
                            .parent_handle()
                    })
                    .flatten()
                    .expect("a peer Constraint must have a parent");
                constraint_parent
                    .with_mut(|parent| {
                        parent
                            .as_component_mut()
                            .expect("a Constraint parent must remain a Component")
                            .add_dependent(this.clone());
                    })
                    .expect("a Constraint parent must remain live");
            }
        }

        assert!(self.bone_transforms.is_none());
        let mut bone_transforms = vec![0.0; (self.tendons.len() + 1) * 6];
        bone_transforms[0] = 1.0;
        bone_transforms[1] = 0.0;
        bone_transforms[2] = 0.0;
        bone_transforms[3] = 1.0;
        bone_transforms[4] = 0.0;
        bone_transforms[5] = 0.0;
        self.bone_transforms = Some(bone_transforms);
    }

    pub fn deform(&self, vertices: &[CoreHandle]) {
        let bone_transforms = self
            .bone_transforms
            .as_deref()
            .expect("buildDependencies initializes the bone transform buffer");
        for vertex_handle in vertices.iter().cloned() {
            vertex_handle
                .with_mut(|vertex| {
                    if let Some(cubic) = vertex.as_cubic_vertex_behavior_mut() {
                        crate::mechanical_port::source::shapes::cubic_vertex::CubicVertexBehavior::deform(
                            cubic,
                            &self.world_transform,
                            bone_transforms,
                        );
                    } else {
                        vertex
                            .as_vertex_behavior_mut()
                            .expect("a retained Vertex must remain a Vertex")
                            .deform(&self.world_transform, bone_transforms);
                    }
                })
                .expect("a retained Vertex must remain live");
        }
    }

    pub fn on_dirty(&mut self, dirt: ComponentDirt) {
        // Opacity cannot move bones. Collapsed is retained state, not work.
        if (dirt & !ComponentDirt::COLLAPSED) == ComponentDirt::RENDER_OPACITY {
            return;
        }
        if let Some(skinnable) = self.skinnable.clone() {
            skinnable
                .with_mut(|object| {
                    if let Some(path) = object.as_points_path_mut() {
                        path.mark_skin_dirty_from_skin(self);
                    } else {
                        object
                            .as_skinnable_behavior_mut()
                            .expect("the retained Skinnable must retain its capability")
                            .mark_skin_dirty();
                    }
                })
                .expect("the retained Skinnable must remain live");
        }
    }

    pub(crate) fn on_dirty_occurrence(owner: &CoreHandle, dirt: ComponentDirt) {
        Self::on_dirty_with_path(
            owner,
            dirt,
            crate::mechanical_port::source::shapes::path::Path::mark_path_dirty_base_occurrence,
        );
    }

    pub(crate) fn on_dirty_from_shape(
        owner: &CoreHandle,
        dirt: ComponentDirt,
        active_shape: &mut crate::mechanical_port::source::shapes::shape::Shape,
    ) {
        Self::on_dirty_with_path(owner, dirt, |path| {
            crate::mechanical_port::source::shapes::path::Path::mark_path_dirty_base_from_shape(
                path,
                active_shape,
            );
        });
    }

    pub(crate) fn on_dirty_from_layout(
        owner: &CoreHandle,
        dirt: ComponentDirt,
        active: &mut crate::mechanical_port::source::component::ActiveLayoutOwner<'_>,
        active_handle: &CoreHandle,
    ) {
        Self::on_dirty_with_path(owner, dirt, |path| {
            crate::mechanical_port::source::shapes::path::Path::mark_path_dirty_base_from_layout(
                path,
                active,
                active_handle,
            );
        });
    }

    fn on_dirty_with_path(
        owner: &CoreHandle,
        dirt: ComponentDirt,
        mut mark_path_dirty: impl FnMut(&CoreHandle),
    ) {
        if (dirt & !ComponentDirt::COLLAPSED) == ComponentDirt::RENDER_OPACITY {
            return;
        }
        let skinnable = owner
            .with_downcast::<Self, _>(|skin| skin.skinnable.clone())
            .flatten();
        if let Some(skinnable) = skinnable {
            if skinnable
                .with(|object| object.as_points_path().is_some())
                .unwrap_or(false)
            {
                // PointsPath::markSkinDirty calls Path::markPathDirty directly;
                // its virtual markPathDirty would dirty this Skin again and
                // reset winding. Release both receivers before Path callouts.
                mark_path_dirty(&skinnable);
            } else {
                skinnable.with_mut(|object| {
                    object
                        .as_skinnable_behavior_mut()
                        .expect("the retained Skinnable must retain its capability")
                        .mark_skin_dirty();
                });
            }
        }
    }

    pub(crate) fn add_dirt_from_points_path(
        &mut self,
        path: &mut crate::mechanical_port::source::shapes::points_path::PointsPath,
    ) -> bool {
        if self
            .component_mut()
            .add_dirt_state(ComponentDirt::SKIN)
            .is_none()
        {
            return false;
        }
        assert_eq!(self.skinnable.as_ref(), path.base.handle().as_ref());
        // Skin::onDirty calls this same PointsPath synchronously. Retain the
        // actual borrowed owners across the callback; the already-set Skin
        // bit terminates the recursive addDirt exactly as in Component.cpp.
        path.mark_skin_dirty_from_skin(self);
        let component = self.component();
        if let Some(artboard) = component.artboard_handle()
            && let Some(dirty) = artboard.artboard_dirty_handle()
        {
            dirty.on_component_dirty_at(component.graph_order());
        }
        true
    }

    pub(crate) fn add_dirt_from_points_path_occurrence(owner: &CoreHandle, path: &CoreHandle) {
        let changed = owner
            .with_downcast_mut::<Self, _>(|skin| {
                if skin
                    .component_mut()
                    .add_dirt_state(ComponentDirt::SKIN)
                    .is_none()
                {
                    return false;
                }
                assert_eq!(skin.skinnable.as_ref(), Some(path));
                true
            })
            .expect("live PointsPath Skin");
        if changed {
            // Skin::onDirty calls PointsPath::markSkinDirty, whose super call
            // does not reset winding or dirty the Skin a second time.
            crate::mechanical_port::source::shapes::path::Path::mark_path_dirty_base_occurrence(
                path,
            );
            crate::mechanical_port::source::component::ComponentOccurrenceHandle::Authored(
                owner.clone(),
            )
            .notify_artboard();
        }
    }

    #[cfg(test)]
    pub fn tendons_mut(&mut self) -> &mut Vec<CoreHandle> {
        &mut self.tendons
    }
}
