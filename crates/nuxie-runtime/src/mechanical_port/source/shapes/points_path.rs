use crate::mechanical_port::source::{
    bones::{
        skin::Skin,
        skinnable::{Skinnable, SkinnableBehavior},
    },
    component::{ComponentDirt, has_dirt},
    generated::shapes::points_path_base::PointsPathBase,
    math::{mat2d::Mat2D, vec2d::Vec2D},
    shapes::{
        cubic_vertex::CubicVertexBehavior, path::PathVertexOccurrence, vertex::VertexBehavior,
    },
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

    fn measure_winding(&self, deformed: bool) -> i32 {
        let points = self.base.vertices();
        let count = points.len();
        if count < 2 {
            return 0;
        }
        let origin = winding_point(&points[0], None, deformed).0;
        let mut p0 = Vec2D::new(0.0, 0.0);
        let (mut area, mut min_x, mut min_y, mut max_x, mut max_y) =
            (0.0_f32, 0.0_f32, 0.0_f32, 0.0_f32, 0.0_f32);
        let segments = if self.is_path_closed() {
            count
        } else {
            count - 1
        };
        for i in 0..segments {
            let from = &points[i];
            let to = &points[(i + 1) % count];
            let (to_point, to_cubic) = winding_point(to, None, deformed);
            let (_, from_cubic) = winding_point(from, None, deformed);
            let p3 = to_point - origin;
            let p1 = if from_cubic {
                winding_point(from, Some(false), deformed).0 - origin
            } else {
                p0
            };
            let p2 = if to_cubic {
                winding_point(to, Some(true), deformed).0 - origin
            } else {
                p3
            };
            area += 6.0 * Vec2D::cross(p0, p1)
                + 3.0 * Vec2D::cross(p0, p2)
                + Vec2D::cross(p0, p3)
                + 3.0 * Vec2D::cross(p1, p2)
                + 3.0 * Vec2D::cross(p1, p3)
                + 6.0 * Vec2D::cross(p2, p3);
            for p in [p1, p2, p3] {
                // std::min/max retain their first operand for unordered comparisons.
                min_x = if p.x < min_x { p.x } else { min_x };
                min_y = if p.y < min_y { p.y } else { min_y };
                max_x = if max_x < p.x { p.x } else { max_x };
                max_y = if max_y < p.y { p.y } else { max_y };
            }
            p0 = p3;
        }
        area /= 20.0;
        let extent = if max_x - min_x < max_y - min_y {
            max_y - min_y
        } else {
            max_x - min_x
        };
        if area.abs() <= 1e-5_f32 * extent * extent {
            return 0;
        }
        if area < 0.0 { -1 } else { 1 }
    }

    /// Taken from the bind pose, then follows bone mirroring. A fold without
    /// mirroring keeps the reference regardless of the initial animated pose.
    pub fn winding(&mut self) -> i32 {
        let authored = if self.is_clockwise() { 1 } else { -1 };
        let Some(skin) = self.skin() else {
            return authored;
        };
        if self.winding_reference == 0 {
            let bind = skin
                .with_downcast::<Skin, _>(|skin| *skin.bind_transform())
                .expect("live PointsPath skin");
            self.winding_reference = self.measure_winding(false)
                * Skin::orientation(bind.xx(), bind.xy(), bind.yx(), bind.yy());
        }
        let sign = skin
            .with_downcast::<Skin, _>(Skin::winding_sign)
            .expect("a retained PointsPath skin remains a Skin");
        if sign != 0 && self.winding_reference != 0 {
            return self.winding_reference * sign;
        }
        let measured = self.measure_winding(true);
        if measured == 0 {
            return authored;
        }
        if sign != 0 {
            self.winding_reference = measured * sign;
        }
        measured
    }
}

// Select authored positions or where the bones put the vertices and controls.
// A requested cubic control point may lazily populate its source cache.
fn winding_point(
    vertex: &PathVertexOccurrence,
    incoming: Option<bool>,
    deformed: bool,
) -> (Vec2D, bool) {
    match vertex {
        PathVertexOccurrence::Authored(vertex) => vertex
            .with_mut(|vertex| {
                if let Some(cubic) = vertex.as_cubic_vertex_behavior_mut() {
                    let point = match incoming {
                        Some(true) => {
                            if deformed {
                                cubic.render_in()
                            } else {
                                cubic.in_point()
                            }
                        }
                        Some(false) => {
                            if deformed {
                                cubic.render_out()
                            } else {
                                cubic.out_point()
                            }
                        }
                        None if deformed => cubic.render_translation(),
                        None => Vec2D::new(cubic.vertex().base.x(), cubic.vertex().base.y()),
                    };
                    (point, true)
                } else {
                    let vertex = vertex.as_vertex_behavior().expect("path vertex");
                    (
                        if deformed {
                            vertex.render_translation()
                        } else {
                            Vec2D::new(vertex.vertex().base.x(), vertex.vertex().base.y())
                        },
                        false,
                    )
                }
            })
            .expect("live path vertex"),
        PathVertexOccurrence::RuntimeStraight(vertex) => {
            let vertex = vertex.borrow();
            (
                if deformed {
                    vertex.render_translation()
                } else {
                    Vec2D::new(vertex.x(), vertex.y())
                },
                false,
            )
        }
        PathVertexOccurrence::RuntimeCubicDetached(vertex) => {
            let mut vertex = vertex.borrow_mut();
            let point = match incoming {
                Some(true) => {
                    if deformed {
                        vertex.render_in()
                    } else {
                        vertex.in_point()
                    }
                }
                Some(false) => {
                    if deformed {
                        vertex.render_out()
                    } else {
                        vertex.out_point()
                    }
                }
                None if deformed => vertex.render_translation(),
                None => Vec2D::new(vertex.x(), vertex.y()),
            };
            (point, true)
        }
    }
}
