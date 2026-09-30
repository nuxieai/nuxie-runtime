use crate::mechanical_port::source::{
    bones::{
        skin::Skin,
        skinnable::{Skinnable, SkinnableBehavior},
    },
    component::{ComponentDirt, has_dirt},
    generated::shapes::points_path_base::PointsPathBase,
    math::mat2d::Mat2D,
};
static IDENTITY: Mat2D = Mat2D::identity();
impl std::ops::Deref for PointsPath {
    type Target = PointsPathBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for PointsPath {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl PointsPath {
    pub const TYPE_KEY: u16 = PointsPathBase::TYPE_KEY;
}

pub struct PointsPath {
    pub base: PointsPathBase,
    skinnable: Skinnable,
    // Measured winding with the bones' mirroring divided out; zero is unknown.
    winding_reference: i32,
}

impl Default for PointsPath {
    fn default() -> Self {
        Self {
            base: PointsPathBase::default(),
            skinnable: Skinnable::default(),
            winding_reference: 0,
        }
    }
}

impl SkinnableBehavior for PointsPath {
    fn skinnable(&self) -> &Skinnable {
        &self.skinnable
    }

    fn skinnable_mut(&mut self) -> &mut Skinnable {
        &mut self.skinnable
    }

    fn mark_skin_dirty(&mut self) {
        self.base.base.base.mark_path_dirty(true);
    }
}
impl PointsPath {
    pub fn build_dependencies(&mut self) {
        self.base.build_dependencies();
        if let (Some(this), Some(skin)) = (self.base.handle(), self.skin()) {
            skin.with_mut(|skin| {
                if let Some(skin) = skin.as_component_mut() {
                    skin.add_dependent(this);
                }
            });
        }
    }
    pub fn path_transform(&self) -> &Mat2D {
        if self.skin().is_some() {
            &IDENTITY
        } else {
            self.base.world_transform()
        }
    }
    pub(crate) fn update_before_path_super(&mut self, value: ComponentDirt) {
        if has_dirt(value, ComponentDirt::PATH) {
            if let Some(skin) = self.skin() {
                let vertices = self
                    .base
                    .vertices()
                    .iter()
                    .filter_map(|vertex| vertex.authored_handle())
                    .collect::<Vec<_>>();
                skin.with_downcast::<Skin, _>(|skin| skin.deform(&vertices));
            }
        }
    }
    pub fn mark_path_dirty(&mut self, _send_to_layout: bool) {
        // Vertex edits invalidate the measurement; bone deformation does not.
        self.winding_reference = 0;
        if let Some(skin) = self.skin() {
            skin.with_downcast_mut::<Skin, _>(|skin| skin.add_dirt_from_points_path(self))
                .expect("a retained PointsPath skin remains a Skin");
        }
        self.base.base.base.mark_path_dirty(true);
    }
    pub(crate) fn mark_skin_dirty_from_skin(&mut self, _skin: &mut Skin) {
        self.base.base.base.mark_path_dirty(true);
    }
    pub fn mark_skin_dirty(&mut self) {
        self.base.base.base.mark_path_dirty(true);
    }

    /// One when the drawn path winds clockwise, minus one otherwise.
    /// Measure once, then follow bone mirroring. Folds without mirroring keep
    /// the measured answer, just as the authored flag did.
    pub fn winding(&mut self) -> i32 {
        let authored = if self.is_clockwise() { 1 } else { -1 };
        let Some(skin) = self.skin() else {
            return authored;
        };
        let sign = skin
            .with_downcast::<Skin, _>(Skin::winding_sign)
            .expect("a retained PointsPath skin remains a Skin");
        if sign != 0 && self.winding_reference != 0 {
            return self.winding_reference * sign;
        }
        let bounds = self.raw_path().bounds();
        let area = self
            .raw_path()
            .compute_coarse_area_with_origin(bounds.center());
        // std::max in C++ selects the first operand for unordered comparisons.
        let extent = if bounds.width() < bounds.height() {
            bounds.height()
        } else {
            bounds.width()
        };
        if area.abs() <= 1e-5_f32 * extent * extent {
            return authored;
        }
        let measured = if area < 0.0 { -1 } else { 1 };
        if sign != 0 {
            self.winding_reference = measured * sign;
        }
        measured
    }
}
