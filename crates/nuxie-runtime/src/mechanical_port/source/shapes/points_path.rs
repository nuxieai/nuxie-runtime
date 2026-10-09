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

crate::mechanical_port::source::transform_component::impl_transform_update!(
    PointsPath,
    |owner, dirt| {
        if has_dirt(dirt, ComponentDirt::PATH) {
            owner.with_downcast_mut::<Self, _>(|object| object.update_before_path_super(dirt));
        }
        crate::mechanical_port::source::shapes::path::Path::update_occurrence::<Self>(owner, dirt);
    },
    crate::mechanical_port::source::transform_component::update_local_transform::<Self>,
    crate::mechanical_port::source::node::Node::update_world_transform_occurrence::<Self>,
    crate::mechanical_port::source::transform_component::compose_world_transform::<Self>,
    crate::mechanical_port::source::transform_component::update_constraints_super::<Self>
);

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
    pub const TYPE_KEY: u16 = PointsPathBase::TYPE_KEY;

    // Path's import callback needs the current skin state while this concrete
    // occurrence is borrowed. Supply it without resolving this owner again.
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

    // PointsPath::update's prefix. The occurrence adapter invokes Path's
    // superclass update only after releasing this concrete receiver loan.
    pub(crate) fn update_before_path_super(&mut self, value: ComponentDirt) {
        if has_dirt(value, ComponentDirt::PATH) {
            if let Some(skin) = self.skin() {
                let vertices = self
                    .base
                    .vertices()
                    .iter()
                    .filter_map(|vertex| match vertex {
                        PathVertexOccurrence::Authored(vertex) => Some(vertex),
                        PathVertexOccurrence::RuntimeStraight(_)
                        | PathVertexOccurrence::RuntimeCubicDetached(_) => None,
                    });
                skin.with_downcast::<Skin, _>(|skin| skin.deform_borrowed_vertices(vertices));
            }
        }
    }

    pub fn mark_path_dirty(&mut self, _send_to_layout: bool) {
        self.winding_reference = 0;
        if let Some(skin) = self.skin() {
            skin.with_downcast_mut::<Skin, _>(|skin| skin.add_dirt_from_points_path(self))
                .expect("a retained PointsPath skin remains a Skin");
        }
        // C++ deliberately invokes Super::markPathDirty() with its default.
        self.base.base.base.mark_path_dirty(true);
    }

    // Released-owner counterpart of the same prefix. Its caller dirties Skin
    // before executing Path::markPathDirty, preserving synchronous callbacks.
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

        // Measure from the first point; an open path closes on it for free.
        let origin = winding_translation(&points[0], deformed);
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
            let p3 = winding_translation(to, deformed) - origin;
            let p1 = winding_cubic_point(from, false, deformed)
                .map(|point| point - origin)
                .unwrap_or(p0);
            let p2 = winding_cubic_point(to, true, deformed)
                .map(|point| point - origin)
                .unwrap_or(p3);

            // Exact cubic area, including a line with handles at its ends.
            area += 6.0 * Vec2D::cross(p0, p1)
                + 3.0 * Vec2D::cross(p0, p2)
                + Vec2D::cross(p0, p3)
                + 3.0 * Vec2D::cross(p1, p2)
                + 3.0 * Vec2D::cross(p1, p3)
                + 6.0 * Vec2D::cross(p2, p3);
            for point in [p1, p2, p3] {
                // std::min/max retain their first operand if unordered.
                min_x = if point.x < min_x { point.x } else { min_x };
                min_y = if point.y < min_y { point.y } else { min_y };
                max_x = if max_x < point.x { point.x } else { max_x };
                max_y = if max_y < point.y { point.y } else { max_y };
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

    // Keep the bind winding through folds, and follow bones only when their
    // orientation gives a nonzero mirroring sign.
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

// C++'s at(vertex) reads only Vertex's position. In particular, a subtype
// query must not also read the discarded deformed position of the from vertex.
fn winding_translation(vertex: &PathVertexOccurrence, deformed: bool) -> Vec2D {
    match vertex {
        PathVertexOccurrence::Authored(vertex) => vertex
            .with(|object| {
                let vertex = object.as_vertex_behavior().expect("path vertex");
                if deformed {
                    vertex.render_translation()
                } else {
                    Vec2D::new(vertex.vertex().base.x(), vertex.vertex().base.y())
                }
            })
            .expect("live path vertex"),
        PathVertexOccurrence::RuntimeStraight(vertex) => {
            let vertex = vertex.borrow();
            if deformed {
                vertex.render_translation()
            } else {
                Vec2D::new(vertex.x(), vertex.y())
            }
        }
        PathVertexOccurrence::RuntimeCubicDetached(vertex) => {
            let vertex = vertex.borrow();
            if deformed {
                vertex.render_translation()
            } else {
                Vec2D::new(vertex.x(), vertex.y())
            }
        }
    }
}

// A cubic's authored control getter can lazily populate its cache. Retain the
// mutable loan for that one request, separately from the position reads.
fn winding_cubic_point(
    vertex: &PathVertexOccurrence,
    incoming: bool,
    deformed: bool,
) -> Option<Vec2D> {
    fn point(cubic: &mut dyn CubicVertexBehavior, incoming: bool, deformed: bool) -> Vec2D {
        match (incoming, deformed) {
            (true, true) => cubic.render_in(),
            (true, false) => cubic.in_point(),
            (false, true) => cubic.render_out(),
            (false, false) => cubic.out_point(),
        }
    }
    match vertex {
        PathVertexOccurrence::Authored(vertex) => vertex
            .with_mut(|vertex| {
                vertex
                    .as_cubic_vertex_behavior_mut()
                    .map(|cubic| point(cubic, incoming, deformed))
            })
            .expect("live path vertex"),
        PathVertexOccurrence::RuntimeStraight(_) => None,
        PathVertexOccurrence::RuntimeCubicDetached(vertex) => {
            Some(point(&mut *vertex.borrow_mut(), incoming, deformed))
        }
    }
}

#[cfg(test)]
mod hole_callback_tests {
    use super::*;
    use crate::mechanical_port::source::{
        core::{CoreArena, CoreHandle},
        core_context::CoreContext,
        generated::{core_registry::CoreRegistry, shapes::path_base::PathBase},
        status_code::StatusCode,
    };

    struct Context<'a> {
        arena: &'a CoreArena,
        parent: CoreHandle,
    }
    impl CoreContext for Context<'_> {
        fn core_arena(&self) -> &CoreArena {
            self.arena
        }
        fn resolve_handle(&self, id: u32) -> Option<CoreHandle> {
            (id == 1).then(|| self.parent.clone())
        }
    }

    #[test]
    fn registry_hole_change_invalidates_skinned_winding_and_skin() {
        let arena = CoreArena::default();
        let path = arena.insert(PointsPath::default());
        let skin = arena.insert(Skin::default());
        skin.with_mut(|owner| {
            owner
                .as_component_mut()
                .unwrap()
                .base
                .set_parent_id_value(1);
        });
        let mut context = Context {
            arena: &arena,
            parent: path.clone(),
        };
        assert_eq!(
            skin.with_downcast_mut::<Skin, _>(|owner| {
                owner.on_added_dirty(skin.clone(), &mut context)
            }),
            Some(StatusCode::Ok)
        );
        path.with_downcast_mut::<PointsPath, _>(|owner| {
            owner.winding_reference = 1;
            owner.set_dirt(ComponentDirt::NONE);
        });
        skin.with_mut(|owner| {
            owner
                .as_component_mut()
                .unwrap()
                .set_dirt(ComponentDirt::NONE)
        });

        assert!(CoreRegistry::set_bool_handle(
            &path,
            i32::from(PathBase::IS_HOLE_PROPERTY_KEY),
            true
        ));
        assert_eq!(
            path.with_downcast::<PointsPath, _>(|owner| owner.winding_reference),
            Some(0)
        );
        assert!(
            skin.with(|owner| owner.as_component().unwrap().has_dirt(ComponentDirt::SKIN))
                .unwrap()
        );
        assert!(
            path.with(|owner| owner.as_component().unwrap().has_dirt(ComponentDirt::PATH))
                .unwrap()
        );

        // An unchanged generated property must not dispatch the callback.
        path.with_downcast_mut::<PointsPath, _>(|owner| owner.winding_reference = -1);
        skin.with_mut(|owner| {
            owner
                .as_component_mut()
                .unwrap()
                .set_dirt(ComponentDirt::NONE)
        });
        assert!(CoreRegistry::set_bool_handle(
            &path,
            i32::from(PathBase::IS_HOLE_PROPERTY_KEY),
            true
        ));
        assert_eq!(
            path.with_downcast::<PointsPath, _>(|owner| owner.winding_reference),
            Some(-1)
        );
        assert!(
            !skin
                .with(|owner| owner.as_component().unwrap().has_dirt(ComponentDirt::SKIN))
                .unwrap()
        );
    }
}

#[cfg(test)]
#[path = "points_path_deform_tests.rs"]
mod deform_view_tests;
