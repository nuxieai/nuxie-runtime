use crate::mechanical_port::source::{
    bones::{
        skin::Skin,
        skinnable::{Skinnable, SkinnableBehavior},
    },
    component::{ComponentDirt, has_dirt},
    generated::shapes::points_path_base::PointsPathBase,
    math::{mat2d::Mat2D, raw_path::RawPath, vec2d::Vec2D},
    shapes::{cubic_vertex::CubicVertexBehavior, path::PathVertexOccurrence},
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
    // Winding with the bones' mirroring divided out; zero is unknown.
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
    pub fn on_added_clean(
        &mut self,
        context: &mut dyn crate::mechanical_port::source::core_context::CoreContext,
    ) -> crate::mechanical_port::source::status_code::StatusCode {
        let has_skin = self.skin().is_some();
        self.base
            .base
            .base
            .base
            .on_added_clean_with_skin(context, has_skin)
    }
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
    pub(crate) fn prepare_mark_path_dirty(
        &mut self,
    ) -> Option<crate::mechanical_port::source::core::CoreHandle> {
        self.winding_reference = 0;
        self.skin()
    }
    pub(crate) fn mark_skin_dirty_from_skin(&mut self, _skin: &mut Skin) {
        self.base.base.base.mark_path_dirty(true);
    }
    pub fn mark_skin_dirty(&mut self) {
        self.base.base.base.mark_path_dirty(true);
    }

    fn bind_winding(&self) -> i32 {
        let points = self.base.vertices();
        let count = points.len();
        if count < 2 {
            return 0;
        }
        let world = self
            .skin()
            .expect("bound PointsPath skin")
            .with_downcast::<Skin, _>(|skin| *skin.world_transform())
            .expect("live PointsPath skin");
        let mut bound = RawPath::default();
        bound.move_to_point(world * bind_point(&points[0], None).0);
        let segments = if self.is_path_closed() {
            count
        } else {
            count - 1
        };
        for i in 0..segments {
            let from = &points[i];
            let to = &points[(i + 1) % count];
            let (to_point, to_cubic) = bind_point(to, None);
            let (from_point, from_cubic) = bind_point(from, None);
            let end = world * to_point;
            if from_cubic || to_cubic {
                let out = if from_cubic {
                    bind_point(from, Some(false)).0
                } else {
                    from_point
                };
                let incoming = if to_cubic {
                    bind_point(to, Some(true)).0
                } else {
                    to_point
                };
                bound.cubic_to_points(world * out, world * incoming, end);
            } else {
                bound.line_to_point(end);
            }
        }
        bound.close();
        measure_winding(&bound)
    }

    /// Taken from the bind pose, then follows bone mirroring. A fold without
    /// mirroring keeps the reference regardless of the initial animated pose.
    pub fn winding(&mut self) -> i32 {
        let authored = if self.is_clockwise() { 1 } else { -1 };
        let Some(skin) = self.skin() else {
            return authored;
        };
        if self.winding_reference == 0 {
            self.winding_reference = self.bind_winding();
        }
        let sign = skin
            .with_downcast::<Skin, _>(Skin::winding_sign)
            .expect("a retained PointsPath skin remains a Skin");
        if sign != 0 && self.winding_reference != 0 {
            return self.winding_reference * sign;
        }
        let measured = measure_winding(self.raw_path());
        if measured == 0 {
            return authored;
        }
        if sign != 0 {
            self.winding_reference = measured * sign;
        }
        measured
    }
}

fn measure_winding(path: &RawPath) -> i32 {
    let bounds = path.bounds();
    let area = path.compute_coarse_area_with_origin(bounds.center());
    // Match std::max's first-operand behavior for unordered comparisons.
    let extent = if bounds.width() < bounds.height() {
        bounds.height()
    } else {
        bounds.width()
    };
    if area.abs() <= 1e-5_f32 * extent * extent {
        return 0;
    }
    if area < 0.0 { -1 } else { 1 }
}

// Read authored positions, never render translations or deformed tangents.
// A requested cubic control point may lazily populate its source cache.
fn bind_point(vertex: &PathVertexOccurrence, incoming: Option<bool>) -> (Vec2D, bool) {
    match vertex {
        PathVertexOccurrence::Authored(vertex) => vertex
            .with_mut(|vertex| {
                if let Some(cubic) = vertex.as_cubic_vertex_behavior_mut() {
                    let point = match incoming {
                        Some(true) => cubic.in_point(),
                        Some(false) => cubic.out_point(),
                        None => Vec2D::new(cubic.vertex().base.x(), cubic.vertex().base.y()),
                    };
                    (point, true)
                } else {
                    let vertex = vertex.as_vertex_behavior().expect("path vertex").vertex();
                    (Vec2D::new(vertex.base.x(), vertex.base.y()), false)
                }
            })
            .expect("live path vertex"),
        PathVertexOccurrence::RuntimeStraight(vertex) => {
            let vertex = vertex.borrow();
            (Vec2D::new(vertex.x(), vertex.y()), false)
        }
        PathVertexOccurrence::RuntimeCubicDetached(vertex) => {
            let mut vertex = vertex.borrow_mut();
            let point = match incoming {
                Some(true) => vertex.in_point(),
                Some(false) => vertex.out_point(),
                None => Vec2D::new(vertex.x(), vertex.y()),
            };
            (point, true)
        }
    }
}
