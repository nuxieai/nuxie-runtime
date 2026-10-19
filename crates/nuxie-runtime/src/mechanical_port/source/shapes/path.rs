use std::{cell::RefCell, rc::Rc};

use crate::mechanical_port::source::{
    component::{ComponentDirt, has_dirt},
    core::CoreHandle,
    core_context::CoreContext,
    generated::shapes::path_base::PathBase,
    math::{aabb::Aabb, mat2d::Mat2D, math_types::PI, raw_path::RawPath, vec2d::Vec2D},
    shapes::{
        cubic_detached_vertex::CubicDetachedVertex,
        cubic_vertex::CubicVertexBehavior,
        path_cells::PathCells,
        path_flags::PathFlags,
        path_vertex::PathVertex,
        shape::Shape,
        straight_vertex::StraightVertex,
        vertex::VertexBehavior,
        vertex_cells::{VertexCells, VertexKind},
    },
    status_code::StatusCode,
};

#[derive(Clone)]
pub enum PathVertexOccurrence {
    // An authored vertex and the cells skin deform and buildPath read.
    Authored(CoreHandle, Rc<VertexCells>),
    RuntimeStraight(Rc<RefCell<StraightVertex>>),
    RuntimeCubicDetached(Rc<RefCell<CubicDetachedVertex>>),
}

impl PathVertexOccurrence {
    pub fn authored_handle(&self) -> Option<CoreHandle> {
        match self {
            Self::Authored(handle, _) => Some(handle.clone()),
            Self::RuntimeStraight(_) | Self::RuntimeCubicDetached(_) => None,
        }
    }
}

#[derive(Clone, Copy)]
enum RenderVertex {
    Cubic {
        translation: Vec2D,
        in_point: Vec2D,
        out_point: Vec2D,
    },
    Straight {
        translation: Vec2D,
        radius: f32,
    },
}

#[derive(Clone, Copy)]
enum VertexRead {
    First,
    Body,
    Incoming,
    Outgoing,
}

fn read_cubic(vertex: &mut dyn CubicVertexBehavior, read: VertexRead) -> RenderVertex {
    let mut translation = Vec2D::default();
    let mut in_point = Vec2D::default();
    let mut out_point = Vec2D::default();
    match read {
        VertexRead::First => {
            in_point = vertex.render_in();
            out_point = vertex.render_out();
            translation = vertex.render_translation();
        }
        VertexRead::Body => {
            in_point = vertex.render_in();
            translation = vertex.render_translation();
        }
        VertexRead::Incoming => in_point = vertex.render_in(),
        VertexRead::Outgoing => out_point = vertex.render_out(),
    }
    RenderVertex::Cubic {
        translation,
        in_point,
        out_point,
    }
}

/// read_cubic, or a straight vertex's read, from an authored straight or
/// cubic vertex's cells.
#[inline]
fn read_vertex_cells(owner: &CoreHandle, cells: &VertexCells, read: VertexRead) -> RenderVertex {
    if cells.kind() == VertexKind::Straight {
        return RenderVertex::Straight {
            translation: cells.render_translation(),
            radius: cells.radius(),
        };
    }
    let mut translation = Vec2D::default();
    let mut in_point = Vec2D::default();
    let mut out_point = Vec2D::default();
    // A weighted vertex reads its deformed points without computing any, so
    // one read of its weight answers all three.
    if let Some((weighted_in, weighted_out, weighted_translation)) = cells.weighted_render_points()
    {
        match read {
            VertexRead::First => {
                in_point = weighted_in;
                out_point = weighted_out;
                translation = weighted_translation;
            }
            VertexRead::Body => {
                in_point = weighted_in;
                translation = weighted_translation;
            }
            VertexRead::Incoming => in_point = weighted_in,
            VertexRead::Outgoing => out_point = weighted_out,
        }
        return RenderVertex::Cubic {
            translation,
            in_point,
            out_point,
        };
    }
    match read {
        VertexRead::First => {
            in_point = cells.render_in_of(owner);
            out_point = cells.render_out_of(owner);
            translation = cells.render_translation();
        }
        VertexRead::Body => {
            in_point = cells.render_in_of(owner);
            translation = cells.render_translation();
        }
        VertexRead::Incoming => in_point = cells.render_in_of(owner),
        VertexRead::Outgoing => out_point = cells.render_out_of(owner),
    }
    RenderVertex::Cubic {
        translation,
        in_point,
        out_point,
    }
}

fn cpp_min(a: f32, b: f32) -> f32 {
    if b < a { b } else { a }
}

impl RenderVertex {
    fn translation(self) -> Vec2D {
        match self {
            Self::Cubic { translation, .. } | Self::Straight { translation, .. } => translation,
        }
    }

    fn incoming(self) -> Vec2D {
        match self {
            Self::Cubic { in_point, .. } => in_point,
            Self::Straight { translation, .. } => translation,
        }
    }

    fn outgoing(self) -> Vec2D {
        match self {
            Self::Cubic { out_point, .. } => out_point,
            Self::Straight { translation, .. } => translation,
        }
    }
}

impl std::ops::Deref for Path {
    type Target = PathBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for Path {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl Path {
    pub(crate) fn update_occurrence<
        T: crate::mechanical_port::source::transform_component::TransformUpdate,
    >(
        owner: &CoreHandle,
        dirt: ComponentDirt,
    ) {
        Self::update_occurrence_after::<T>(owner, dirt, |_, _| {});
    }

    /// Path::update for an owner whose own update runs `before` and then
    /// Path::update (PointsPath's skin deformation), with both on one loan of
    /// the receiver.
    pub(crate) fn update_occurrence_after<
        T: crate::mechanical_port::source::transform_component::TransformUpdate,
    >(
        owner: &CoreHandle,
        dirt: ComponentDirt,
        before: fn(&mut T, ComponentDirt),
    ) {
        // Every registered Path owner inherits Node's world-update prefix.
        // Keep the actual native receiver through the adjacent pure local work.
        // Without constraints nothing else runs before the rebuild, so the
        // render opacity (from the parent's cells) and the rebuild share that
        // receiver too; constraints release it, as they can revisit this path.
        // A custom owner can project T from as_any_mut; preserve that owner's
        // original projection calls and released update sequence below.
        let prefix = owner
            .with_mut(|object| {
                if let Some(object) = (object as &mut dyn std::any::Any).downcast_mut::<T>() {
                    before(object, dirt);
                    object
                        .as_path_mut()
                        .expect("Path receiver")
                        .update_before_transform_super();
                    if dirt.contains(ComponentDirt::TRANSFORM) {
                        T::update_transform(object);
                    }
                    if dirt.contains(ComponentDirt::WORLD_TRANSFORM) {
                        object
                            .as_node_mut()
                            .expect("Path inherits Node")
                            .update_world_transform_before_super();
                        T::compose_world_transform(object);
                    }
                    let transform = object
                        .as_transform_component_mut()
                        .expect("Path inherits TransformComponent");
                    let unconstrained = !(dirt.contains(ComponentDirt::WORLD_TRANSFORM)
                        && transform.cells().has_constraints());
                    if unconstrained
                        && (!dirt.contains(ComponentDirt::RENDER_OPACITY)
                            || transform.update_render_opacity_from_cells())
                    {
                        Self::update_tail(object, dirt);
                        return Prefix::Done;
                    }
                    return Prefix::Native;
                }
                if let Some(object) = object.as_any_mut().downcast_mut::<T>() {
                    before(object, dirt);
                    object
                        .as_path_mut()
                        .expect("Path receiver")
                        .update_before_transform_super();
                }
                Prefix::Projected
            })
            .unwrap_or(Prefix::Projected);
        enum Prefix {
            Done,
            Native,
            Projected,
        }
        let native_prefix = match prefix {
            Prefix::Done => return,
            Prefix::Native => {
                dirt.contains(ComponentDirt::TRANSFORM)
                    || dirt.contains(ComponentDirt::WORLD_TRANSFORM)
            }
            Prefix::Projected => false,
        };
        if native_prefix {
            if dirt.contains(ComponentDirt::WORLD_TRANSFORM) {
                T::update_constraints(owner);
            }
            crate::mechanical_port::source::transform_component::update_render_opacity_super::<T>(
                owner, dirt,
            );
        } else {
            crate::mechanical_port::source::transform_component::update_transform_super::<T>(
                owner, dirt,
            );
        }
        owner.with_downcast_mut::<T, _>(|object| Self::update_tail(object, dirt));
    }

    /// Path::update after TransformComponent::update: the rebuild.
    fn update_tail<T: crate::mechanical_port::source::transform_component::TransformUpdate>(
        object: &mut T,
        dirt: ComponentDirt,
    ) {
        let closed = Self::is_path_closed_for(object);
        let skin = object.as_points_path().map(|path| {
            crate::mechanical_port::source::bones::skinnable::SkinnableBehavior::skin(path)
                .is_some()
        });
        object
            .as_path_mut()
            .expect("Path receiver")
            .update_after_transform_super(dirt, closed, skin);
    }
    pub const TYPE_KEY: u16 = PathBase::TYPE_KEY;
}

pub struct Path {
    pub base: PathBase,
    // The shape link, notification and deferral gates, flags, built raw path
    // and geometry version live in the base's PathCells.
    vertices: Vec<PathVertexOccurrence>,
}

impl Default for Path {
    fn default() -> Self {
        Self {
            base: PathBase::default(),
            vertices: Vec::new(),
        }
    }
}

impl Path {
    pub(crate) fn cells(&self) -> &Rc<PathCells> {
        self.base.cells()
    }
    pub fn geometry_version(&self) -> u32 {
        self.cells().geometry_version()
    }
    pub fn compute_ideal_control_point_distance(
        to_prev: Vec2D,
        to_next: Vec2D,
        radius: f32,
    ) -> f32 {
        let angle = Vec2D::cross(to_prev, to_next)
            .atan2(Vec2D::dot(to_prev, to_next))
            .abs();
        radius.min(
            (4.0 / 3.0)
                * (PI / (2.0 * ((2.0 * PI) / angle))).tan()
                * radius
                * if angle < PI / 2.0 {
                    1.0 + angle.cos()
                } else {
                    2.0 - angle.sin()
                },
        )
    }

    pub fn on_added_clean(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        self.on_added_clean_with_skin(context, false)
    }
    pub(crate) fn on_added_clean_with_skin(
        &mut self,
        context: &mut dyn CoreContext,
        has_skin: bool,
    ) -> StatusCode {
        let code = self.base.on_added_clean(context);
        if code != StatusCode::Ok {
            return code;
        }
        let Some(this) = self.base.handle() else {
            return StatusCode::MissingObject;
        };
        let mut parent = self.base.parent_handle();
        while let Some(current) = parent {
            if current.is_type_of(
                crate::mechanical_port::source::generated::shapes::shape_base::ShapeBase::TYPE_KEY,
            ) {
                current.with_mut(|shape| {
                    if let Some(shape) = shape.as_shape_mut() {
                        self.cells()
                            .bind_shape(current.clone(), Rc::clone(shape.cells()));
                        shape.add_path_with_skin(this, has_skin, Rc::clone(self.cells()));
                    }
                });
                return StatusCode::Ok;
            }
            parent = current
                .with(|current| current.component_parent_handle())
                .flatten();
        }
        StatusCode::MissingObject
    }

    pub fn build_dependencies(&mut self) {
        self.base.build_dependencies();
    }
    pub fn add_vertex(&mut self, vertex: CoreHandle, cells: Rc<VertexCells>) {
        cells.set_kind(VertexKind::of(&vertex));
        self.vertices
            .push(PathVertexOccurrence::Authored(vertex, cells));
    }
    pub fn add_runtime_straight_vertex(&mut self, vertex: Rc<RefCell<StraightVertex>>) {
        self.vertices
            .push(PathVertexOccurrence::RuntimeStraight(vertex));
    }
    pub fn add_runtime_cubic_vertex(&mut self, vertex: Rc<RefCell<CubicDetachedVertex>>) {
        self.vertices
            .push(PathVertexOccurrence::RuntimeCubicDetached(vertex));
    }
    pub fn clear_vertices(&mut self) {
        self.vertices.clear();
    }
    pub fn vertices(&self) -> &[PathVertexOccurrence] {
        &self.vertices
    }
    pub fn pop_vertex(&mut self) {
        self.vertices.pop();
    }
    pub fn add_flags(&mut self, flags: PathFlags) {
        self.cells().add_flags(flags);
    }
    pub fn is_flagged(&self, flags: PathFlags) -> bool {
        self.cells().is_flagged(flags)
    }
    pub fn can_defer_path_update(&self, _active_points_path_has_skin: Option<bool>) -> bool {
        self.cells().can_defer_path_update()
    }
    pub fn shape_handle(&self) -> Option<CoreHandle> {
        self.cells().shape_handle()
    }
    pub fn path_transform(&self) -> Mat2D {
        self.base.world_transform()
    }
    pub(crate) fn path_transform_for(
        owner: &dyn crate::mechanical_port::source::core::CoreObject,
    ) -> Mat2D {
        if let Some(points) = owner.as_points_path() {
            points.path_transform()
        } else {
            owner
                .as_path()
                .expect("pathTransform is called on a Path owner")
                .path_transform()
        }
    }
    pub fn local_bounds(&self) -> Aabb {
        self.cells().raw_path().precise_bounds()
    }
    pub fn raw_path(&self) -> std::cell::Ref<'_, RawPath> {
        self.cells().raw_path()
    }
    pub fn needs_path_build(&self) -> bool {
        self.base.has_dirt(ComponentDirt::PATH) || self.cells().deferred()
    }
    pub fn try_property_bounds(&self, _result: &mut Aabb) -> bool {
        false
    }
    pub fn is_path_closed(&self) -> bool {
        true
    }
    pub(crate) fn is_path_closed_for(
        owner: &dyn crate::mechanical_port::source::core::CoreObject,
    ) -> bool {
        if let Some(path) = owner.as_points_path() {
            path.is_path_closed()
        } else if let Some(path) = owner
            .as_any()
            .downcast_ref::<crate::mechanical_port::source::shapes::list_path::ListPath>(
        ) {
            path.is_path_closed()
        } else {
            owner
                .as_path()
                .expect("Path virtual receiver")
                .is_path_closed()
        }
    }
    pub(crate) fn mark_path_dirty_for(
        owner: &mut dyn crate::mechanical_port::source::core::CoreObject,
        send_to_layout: bool,
    ) {
        if let Some(path) = owner.as_points_path_mut() {
            path.mark_path_dirty(send_to_layout);
        } else if let Some(path) = owner.as_parametric_path_mut() {
            path.mark_path_dirty(send_to_layout);
        } else {
            owner
                .as_path_mut()
                .expect("Path virtual receiver")
                .mark_path_dirty(send_to_layout);
        }
    }
    pub fn is_hidden(&self) -> bool {
        self.cells().is_hidden()
    }

    fn render_vertex(vertex: &PathVertexOccurrence) -> Option<RenderVertex> {
        Self::read_vertex(vertex, VertexRead::First)
    }

    /// A vertex's render points: an authored straight or cubic vertex's from
    /// its cells inline, any other through its borrow.
    #[inline]
    fn read_vertex(vertex: &PathVertexOccurrence, read: VertexRead) -> Option<RenderVertex> {
        match vertex {
            PathVertexOccurrence::Authored(vertex, cells) if cells.kind() != VertexKind::Other => {
                Some(read_vertex_cells(vertex, cells, read))
            }
            _ => Self::read_vertex_borrowed(vertex, read),
        }
    }

    #[inline(never)]
    fn read_vertex_borrowed(
        vertex: &PathVertexOccurrence,
        read: VertexRead,
    ) -> Option<RenderVertex> {
        match vertex {
            PathVertexOccurrence::Authored(vertex, cells) if cells.kind() != VertexKind::Other => {
                Some(read_vertex_cells(vertex, cells, read))
            }
            PathVertexOccurrence::Authored(vertex, _) => vertex
                .with_mut(|vertex| {
                    if let Some(cubic) = vertex.as_cubic_vertex_behavior_mut() {
                        return Some(read_cubic(cubic, read));
                    }
                    let radius = vertex
                        .as_straight_vertex()
                        .map_or(0.0, |point| point.radius());
                    vertex
                        .as_vertex_behavior()
                        .map(|vertex| RenderVertex::Straight {
                            translation: vertex.render_translation(),
                            radius,
                        })
                })
                .flatten(),
            PathVertexOccurrence::RuntimeStraight(vertex) => {
                let vertex = vertex.borrow();
                Some(RenderVertex::Straight {
                    translation: vertex.render_translation(),
                    radius: vertex.radius(),
                })
            }
            PathVertexOccurrence::RuntimeCubicDetached(vertex) => {
                let mut vertex = vertex.borrow_mut();
                Some(read_cubic(&mut *vertex, read))
            }
        }
    }

    pub fn add_rounded_rect(raw_path: &mut RawPath, bounds: Aabb, radii: [f32; 4]) {
        let corners = [
            (
                Vec2D::new(bounds.left(), bounds.top()),
                Vec2D::new(0.0, 1.0),
                Vec2D::new(1.0, 0.0),
                radii[0],
            ),
            (
                Vec2D::new(bounds.right(), bounds.top()),
                Vec2D::new(-1.0, 0.0),
                Vec2D::new(0.0, 1.0),
                radii[1],
            ),
            (
                Vec2D::new(bounds.right(), bounds.bottom()),
                Vec2D::new(0.0, -1.0),
                Vec2D::new(-1.0, 0.0),
                radii[2],
            ),
            (
                Vec2D::new(bounds.left(), bounds.bottom()),
                Vec2D::new(1.0, 0.0),
                Vec2D::new(0.0, -1.0),
                radii[3],
            ),
        ];
        let max_radius = cpp_min(bounds.width(), bounds.height()) * 0.5;
        let mut start = Vec2D::default();
        for (index, (position, to_prev, to_next, authored_radius)) in
            corners.into_iter().enumerate()
        {
            if authored_radius != 0.0 {
                let radius = cpp_min(authored_radius.abs(), max_radius);
                let ideal = Self::compute_ideal_control_point_distance(to_prev, to_next, radius);
                let enter = Vec2D::scale_and_add(position, to_prev, radius);
                if index == 0 {
                    start = enter;
                    raw_path.move_to_point(enter);
                } else {
                    raw_path.line_to_point(enter);
                }
                let mut out = Vec2D::scale_and_add(position, to_prev, radius - ideal);
                let mut incoming = Vec2D::scale_and_add(position, to_next, radius - ideal);
                let exit = Vec2D::scale_and_add(position, to_next, radius);
                if authored_radius < 0.0 {
                    rotate_points(exit, enter, position, &mut out, &mut incoming);
                }
                raw_path.cubic_to_points(out, incoming, exit);
            } else if index == 0 {
                start = position;
                raw_path.move_to_point(position);
            } else {
                raw_path.line_to_point(position);
            }
        }
        raw_path.line_to_point(start);
        raw_path.close();
    }

    pub fn build_path(&self, raw_path: &mut RawPath, closed: bool, skinned: bool) {
        self.build_path_with_host(raw_path, closed, skinned, None);
    }

    pub(crate) fn build_path_from_shape(
        &self,
        raw_path: &mut RawPath,
        closed: bool,
        skinned: bool,
        shape: &Shape,
    ) {
        self.build_path_with_host(raw_path, closed, skinned, Some(shape));
    }

    fn build_path_with_host(
        &self,
        raw_path: &mut RawPath,
        closed: bool,
        skinned: bool,
        active_shape: Option<&Shape>,
    ) {
        let vertices = &self.vertices;
        let length = vertices.len();
        if length < 2 {
            return;
        }
        let first = Self::read_vertex(&vertices[0], VertexRead::First).expect("live Path vertex");
        let mut out;
        let mut previous_cubic;
        let start;
        let start_in;
        let start_cubic;
        if let RenderVertex::Cubic {
            translation,
            in_point,
            out_point,
        } = first
        {
            start_cubic = true;
            previous_cubic = true;
            start_in = in_point;
            out = out_point;
            start = translation;
            raw_path.move_to_point(start);
        } else {
            start_cubic = false;
            previous_cubic = false;
            let RenderVertex::Straight {
                translation: position,
                radius,
            } = first
            else {
                unreachable!()
            };
            if radius != 0.0 {
                let previous = Self::read_vertex(&vertices[length - 1], VertexRead::Outgoing)
                    .expect("live Path vertex");
                let mut to_prev = previous.outgoing() - position;
                let previous_length = to_prev.normalize_length();
                let next = Self::read_vertex(&vertices[1], VertexRead::Incoming)
                    .expect("live Path vertex");
                let mut to_next = next.incoming() - position;
                let next_length = to_next.normalize_length();
                let render_radius = cpp_min(
                    previous_length / 2.0,
                    cpp_min(
                        next_length / 2.0,
                        if radius > 0.0 { radius } else { -radius },
                    ),
                );
                let ideal =
                    Self::compute_ideal_control_point_distance(to_prev, to_next, render_radius);
                start = Vec2D::scale_and_add(position, to_prev, render_radius);
                start_in = start;
                raw_path.move_to_point(start_in);
                let mut out_point = Vec2D::scale_and_add(position, to_prev, render_radius - ideal);
                let mut in_point = Vec2D::scale_and_add(position, to_next, render_radius - ideal);
                out = Vec2D::scale_and_add(position, to_next, render_radius);
                if radius < 0.0 {
                    rotate_points(out, start_in, position, &mut out_point, &mut in_point);
                }
                raw_path.cubic_to_points(out_point, in_point, out);
            } else {
                out = position;
                start = out;
                start_in = out;
                raw_path.move_to_point(out);
            }
        }
        for index in 1..length {
            let vertex =
                Self::read_vertex(&vertices[index], VertexRead::Body).expect("live Path vertex");
            if let RenderVertex::Cubic {
                translation,
                in_point: incoming,
                out_point: _,
            } = vertex
            {
                raw_path.cubic_to_points(out, incoming, translation);
                previous_cubic = true;
                out = Self::read_vertex(&vertices[index], VertexRead::Outgoing)
                    .expect("live Path vertex")
                    .outgoing();
            } else {
                let RenderVertex::Straight {
                    translation: position,
                    radius,
                } = vertex
                else {
                    unreachable!()
                };
                if radius != 0.0 {
                    let previous = Self::read_vertex(&vertices[index - 1], VertexRead::Outgoing)
                        .expect("live Path vertex");
                    let mut to_prev = previous.outgoing() - position;
                    let previous_length = to_prev.normalize_length();
                    let next =
                        Self::read_vertex(&vertices[(index + 1) % length], VertexRead::Incoming)
                            .expect("live Path vertex");
                    let mut to_next = next.incoming() - position;
                    let next_length = to_next.normalize_length();
                    let render_radius = cpp_min(
                        previous_length / 2.0,
                        cpp_min(
                            next_length / 2.0,
                            if radius > 0.0 { radius } else { -radius },
                        ),
                    );
                    let ideal =
                        Self::compute_ideal_control_point_distance(to_prev, to_next, render_radius);
                    let translation = Vec2D::scale_and_add(position, to_prev, render_radius);
                    if previous_cubic {
                        raw_path.cubic_to_points(out, translation, translation);
                    } else {
                        raw_path.line_to_point(translation);
                    }
                    let mut out_point =
                        Vec2D::scale_and_add(position, to_prev, render_radius - ideal);
                    let mut in_point =
                        Vec2D::scale_and_add(position, to_next, render_radius - ideal);
                    out = Vec2D::scale_and_add(position, to_next, render_radius);
                    if radius < 0.0 {
                        rotate_points(out, translation, position, &mut out_point, &mut in_point);
                    }
                    raw_path.cubic_to_points(out_point, in_point, out);
                    previous_cubic = false;
                } else if previous_cubic {
                    raw_path.cubic_to_points(out, position, position);
                    previous_cubic = false;
                    out = position;
                } else {
                    out = position;
                    raw_path.line_to_point(out);
                }
            }
        }
        if closed {
            if previous_cubic || start_cubic {
                raw_path.cubic_to_points(out, start_in, start);
            } else {
                raw_path.line_to_point(start);
            }
            raw_path.close();
        }
        // Intrinsic measurement can be inside the owning Shape's callback.
        // Read that actual host directly at the source's deformer boundary.
        let deformer = match active_shape {
            Some(shape) => shape.deformer(),
            None => self.deformer(),
        };
        if let Some(deformer) = deformer {
            // PointsPath::pathTransform is identity for skinned geometry.
            // Classify under the caller's existing owner borrow, but read the
            // world matrix only here, after vertex evaluation, as upstream does.
            let transform = if skinned {
                Mat2D::identity()
            } else {
                self.path_transform()
            };
            deformer.with(|deformer| {
                deformer.render_path_deformer_deform_local(
                    raw_path,
                    transform,
                    transform.invert_or_identity(),
                )
            });
        }
    }

    fn deformer(&self) -> Option<CoreHandle> {
        self.cells()
            .shape()
            .filter(|shape| shape.handle.is_occupied())
            .and_then(|shape| shape.cells.deformer())
    }
    pub fn mark_path_dirty(&mut self, _send_to_layout: bool) {
        self.base.add_dirt(ComponentDirt::PATH, false);
        self.shape_path_changed();
    }
    /// Virtual markPathDirty with no arena borrow held across synchronous dirt
    /// callbacks, which can legitimately reach this same path through a constraint.
    pub fn mark_path_dirty_occurrence(owner: &CoreHandle, send_to_layout: bool) {
        let (skin, parametric) = owner
            .with_mut(|object| {
                let parametric = object.as_parametric_path().is_some();
                let skin = object
                    .as_points_path_mut()
                    .and_then(|points| points.prepare_mark_path_dirty());
                (skin, parametric)
            })
            .expect("live Path");
        if let Some(skin) = skin {
            crate::mechanical_port::source::bones::skin::Skin::add_dirt_from_points_path_occurrence(
                &skin, owner,
            );
        }
        Self::mark_path_dirty_base_occurrence(owner);
        if parametric && send_to_layout {
            crate::mechanical_port::source::shapes::parametric_path::ParametricPath::send_layout_dirty_occurrence(owner);
        }
    }

    pub(crate) fn mark_path_dirty_base_occurrence(owner: &CoreHandle) {
        if let Some(cells) = Self::cells_of(owner) {
            cells.mark_path_dirty(owner);
        }
    }

    fn cells_of(owner: &CoreHandle) -> Option<Rc<PathCells>> {
        owner
            .with(|object| object.as_path().map(|path| Rc::clone(path.cells())))
            .flatten()
    }

    pub(crate) fn mark_path_dirty_base_from_shape(owner: &CoreHandle, active_shape: &mut Shape) {
        crate::mechanical_port::source::component::ComponentOccurrenceHandle::Authored(
            owner.clone(),
        )
        .add_dirt_from_shape(active_shape, ComponentDirt::PATH, false);
        Self::send_shape_notification_from_shape(owner, active_shape);
    }

    pub(crate) fn mark_path_dirty_base_from_layout(
        owner: &CoreHandle,
        active: &mut crate::mechanical_port::source::component::ActiveLayoutOwner<'_>,
        active_handle: &CoreHandle,
    ) {
        crate::mechanical_port::source::component::ComponentOccurrenceHandle::Authored(
            owner.clone(),
        )
        .add_dirt_from_layout(active, active_handle, ComponentDirt::PATH, false);
        Self::send_shape_notification_from_layout(owner, active, active_handle);
    }

    pub(crate) fn on_dirty_from_layout(
        owner: &CoreHandle,
        dirt: ComponentDirt,
        active: &mut crate::mechanical_port::source::component::ActiveLayoutOwner<'_>,
        active_handle: &CoreHandle,
    ) {
        if has_dirt(
            dirt,
            ComponentDirt::WORLD_TRANSFORM | ComponentDirt::N_SLICER,
        ) {
            Self::send_shape_notification_from_layout(owner, active, active_handle);
        }
        let deferred = Self::cells_of(owner).is_some_and(|cells| cells.deferred());
        if deferred {
            crate::mechanical_port::source::component::ComponentOccurrenceHandle::Authored(
                owner.clone(),
            )
            .add_dirt_from_layout(active, active_handle, ComponentDirt::PATH, false);
        }
    }

    pub(crate) fn on_dirty_occurrence(owner: &CoreHandle, dirt: ComponentDirt) {
        if let Some(cells) = Self::cells_of(owner) {
            cells.on_dirty(owner, dirt);
        }
    }
    fn send_shape_notification_from_shape(owner: &CoreHandle, active_shape: &mut Shape) {
        let shape = Self::cells_of(owner)
            .and_then(|cells| cells.take_shape_notification())
            .map(|shape| shape.handle);
        if let Some(shape) = shape {
            if active_shape.base.handle().as_ref() == Some(&shape) {
                active_shape.path_changed();
            } else {
                Shape::path_changed_occurrence(&shape);
            }
        }
    }
    fn send_shape_notification_from_layout(
        owner: &CoreHandle,
        active: &mut crate::mechanical_port::source::component::ActiveLayoutOwner<'_>,
        active_handle: &CoreHandle,
    ) {
        let shape = Self::cells_of(owner)
            .and_then(|cells| cells.take_shape_notification())
            .map(|shape| shape.handle);
        if let Some(shape) = shape {
            Shape::path_changed_from_layout_occurrence(&shape, active, active_handle);
        }
    }
    fn shape_path_changed(&mut self) {
        if let Some(shape) = self.cells().take_shape_notification() {
            shape.cells.path_changed(&shape.handle);
        }
    }
    pub fn on_dirty(&mut self, value: ComponentDirt) {
        if has_dirt(
            value,
            ComponentDirt::WORLD_TRANSFORM | ComponentDirt::N_SLICER,
        ) {
            self.shape_path_changed();
        }
        if self.cells().deferred() {
            self.base.add_dirt(ComponentDirt::PATH, false);
        }
    }
    pub(crate) fn on_dirty_from_shape(
        owner: &CoreHandle,
        value: ComponentDirt,
        active_shape: &mut Shape,
    ) {
        if has_dirt(
            value,
            ComponentDirt::WORLD_TRANSFORM | ComponentDirt::N_SLICER,
        ) {
            Self::send_shape_notification_from_shape(owner, active_shape);
        }
        let deferred = Self::cells_of(owner).is_some_and(|cells| cells.deferred());
        if deferred {
            // Source addDirt runs synchronously after pathChanged, including
            // the recursive onDirty call if this adds the Path dirt bit.
            crate::mechanical_port::source::component::ComponentOccurrenceHandle::Authored(
                owner.clone(),
            )
            .add_dirt_from_shape(active_shape, ComponentDirt::PATH, false);
        }
    }
    pub(crate) fn update_before_transform_super(&mut self) {
        self.cells().reset_shape_notification();
    }
    pub(crate) fn update_after_transform_super(
        &mut self,
        value: ComponentDirt,
        closed: bool,
        active_points_path_has_skin: Option<bool>,
    ) {
        let changed = has_dirt(value, ComponentDirt::PATH);
        let world_changed = has_dirt(value, ComponentDirt::WORLD_TRANSFORM);
        let deformer_changed = has_dirt(value, ComponentDirt::N_SLICER);
        if changed || deformer_changed || (self.deformer().is_some() && world_changed) {
            if self.can_defer_path_update(active_points_path_has_skin) {
                self.cells().set_deferred(true);
                return;
            }
            self.cells().set_deferred(false);
            let mut path = self.cells().take_raw_path();
            path.rewind();
            self.build_path(&mut path, closed, active_points_path_has_skin == Some(true));
            self.cells().put_raw_path(path);
            self.cells().bump_geometry_version();
        }
    }
    pub fn is_hole_changed(&mut self) {
        self.mark_path_dirty(true);
    }
    pub fn collapse(&mut self, value: bool) -> bool {
        let changed = self.base.collapse(value);
        if changed {
            self.collapse_after_super();
        }
        changed
    }

    pub(crate) fn collapse_after_super(&mut self) {
        if let Some(shape) = self.cells().shape_handle() {
            shape.with_mut(|shape| {
                if let Some(shape) = shape.as_shape_mut() {
                    shape.path_collapse_changed();
                }
            });
        }
    }

    pub(crate) fn collapse_after_super_occurrence(owner: &CoreHandle) {
        let shape = owner
            .with(|object| object.as_path().and_then(Path::shape_handle))
            .flatten();
        if let Some(shape) = shape {
            Shape::path_collapse_changed_occurrence(&shape);
        }
    }

    pub(crate) fn collapse_after_super_from_layout_occurrence(
        owner: &CoreHandle,
        active: &mut crate::mechanical_port::source::component::ActiveLayoutOwner<'_>,
        active_handle: &CoreHandle,
    ) {
        let shape = owner
            .with(|object| object.as_path().and_then(Path::shape_handle))
            .flatten();
        if let Some(shape) = shape {
            Shape::path_collapse_changed_from_layout_occurrence(&shape, active, active_handle);
        }
    }
}

fn rotate_points(
    next: Vec2D,
    previous: Vec2D,
    point: Vec2D,
    out: &mut Vec2D,
    incoming: &mut Vec2D,
) {
    let v1 = previous - next;
    let v2 = point - next;
    let angle = Vec2D::cross(v1, v2).atan2(Vec2D::dot(v1, v2));
    let s = (angle * 2.0).sin();
    let c = (angle * 2.0).cos();
    *out -= previous;
    *out = Vec2D::new(out.x.mul_add(c, -(out.y * s)), out.x.mul_add(s, out.y * c)) + previous;
    let s = (-angle * 2.0).sin();
    let c = (-angle * 2.0).cos();
    *incoming -= next;
    *incoming = Vec2D::new(
        incoming.x.mul_add(c, -(incoming.y * s)),
        incoming.x.mul_add(s, incoming.y * c),
    ) + next;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rotate_points_matches_pinned_fused_rotation() {
        let next = Vec2D::new(29.2390594, 22.256197);
        let previous = Vec2D::new(36.3585854, -2.2231946);
        let point = Vec2D::new(26.3485889, 8.14053153);
        let mut out = Vec2D::new(26.895485, 7.57431);
        let mut incoming = Vec2D::new(26.50651, 8.91174);

        rotate_points(next, previous, point, &mut out, &mut incoming);

        assert_eq!(out.x.to_bits(), 0x421c_5d52);
        assert_eq!(out.y.to_bits(), 0x4131_f0b0);
        assert_eq!(incoming.x.to_bits(), 0x421a_cf03);
        assert_eq!(incoming.y.to_bits(), 0x4147_56ce);
    }
}

#[cfg(feature = "tools")]
pub struct DisplayCubicVertex {
    pub base: crate::mechanical_port::source::shapes::cubic_vertex::CubicVertex,
}
#[cfg(feature = "tools")]
impl DisplayCubicVertex {
    fn new(in_point: Vec2D, out_point: Vec2D, translation: Vec2D) -> Self {
        let mut vertex = Self {
            base: Default::default(),
        };
        vertex.base.cells().store_in_point(in_point);
        vertex.base.cells().store_out_point(out_point);
        vertex.base.cells().set_in_valid(true);
        vertex.base.cells().set_out_valid(true);
        if vertex.vertex_mut().base.set_x_value(translation.x) {
            CubicVertexBehavior::x_changed(&mut vertex);
            vertex.vertex_mut().notify_property_changed(crate::mechanical_port::source::generated::shapes::vertex_base::VertexBase::X_PROPERTY_KEY);
        }
        if vertex.vertex_mut().base.set_y_value(translation.y) {
            CubicVertexBehavior::y_changed(&mut vertex);
            vertex.vertex_mut().notify_property_changed(crate::mechanical_port::source::generated::shapes::vertex_base::VertexBase::Y_PROPERTY_KEY);
        }
        vertex
    }
}
#[cfg(feature = "tools")]
impl VertexBehavior for DisplayCubicVertex {
    fn vertex(&self) -> &crate::mechanical_port::source::shapes::vertex::Vertex {
        self.base.vertex()
    }
    fn vertex_mut(&mut self) -> &mut crate::mechanical_port::source::shapes::vertex::Vertex {
        self.base.vertex_mut()
    }
    fn mark_geometry_dirty(&mut self) {
        self.base.mark_geometry_dirty();
    }
}
#[cfg(feature = "tools")]
impl CubicVertexBehavior for DisplayCubicVertex {
    fn cubic_vertex(&self) -> &crate::mechanical_port::source::shapes::cubic_vertex::CubicVertex {
        &self.base
    }
    fn cubic_vertex_mut(
        &mut self,
    ) -> &mut crate::mechanical_port::source::shapes::cubic_vertex::CubicVertex {
        &mut self.base
    }
    // The pinned display owner retains the transformed control points.
    fn compute_in(&mut self) {}
    fn compute_out(&mut self) {}
}
#[cfg(feature = "tools")]
pub enum FlattenedPathVertex {
    Cubic(DisplayCubicVertex),
    Straight(PathVertex),
}
#[cfg(feature = "tools")]
impl VertexBehavior for FlattenedPathVertex {
    fn vertex(&self) -> &crate::mechanical_port::source::shapes::vertex::Vertex {
        match self {
            Self::Cubic(vertex) => vertex.vertex(),
            Self::Straight(vertex) => vertex.vertex(),
        }
    }
    fn vertex_mut(&mut self) -> &mut crate::mechanical_port::source::shapes::vertex::Vertex {
        match self {
            Self::Cubic(vertex) => vertex.vertex_mut(),
            Self::Straight(vertex) => vertex.vertex_mut(),
        }
    }
    fn mark_geometry_dirty(&mut self) {
        match self {
            Self::Cubic(vertex) => vertex.mark_geometry_dirty(),
            Self::Straight(vertex) => vertex.mark_geometry_dirty(),
        }
    }
}
#[cfg(feature = "tools")]
pub struct FlattenedPath {
    vertices: Vec<FlattenedPathVertex>,
}
#[cfg(feature = "tools")]
impl FlattenedPath {
    pub fn vertices(&self) -> &[FlattenedPathVertex] {
        &self.vertices
    }
    fn add_vertex(&mut self, vertex: RenderVertex, transform: Mat2D) {
        match vertex {
            RenderVertex::Cubic {
                translation,
                in_point,
                out_point,
            } => self
                .vertices
                .push(FlattenedPathVertex::Cubic(DisplayCubicVertex::new(
                    transform * in_point,
                    transform * out_point,
                    transform * translation,
                ))),
            RenderVertex::Straight { translation, .. } => {
                let translation = transform * translation;
                let mut vertex = PathVertex::default();
                if vertex.vertex_mut().base.set_x_value(translation.x) {
                    VertexBehavior::x_changed(&mut vertex);
                    vertex.vertex_mut().notify_property_changed(crate::mechanical_port::source::generated::shapes::vertex_base::VertexBase::X_PROPERTY_KEY);
                }
                if vertex.vertex_mut().base.set_y_value(translation.y) {
                    VertexBehavior::y_changed(&mut vertex);
                    vertex.vertex_mut().notify_property_changed(crate::mechanical_port::source::generated::shapes::vertex_base::VertexBase::Y_PROPERTY_KEY);
                }
                self.vertices.push(FlattenedPathVertex::Straight(vertex));
            }
        }
    }
}

#[cfg(feature = "tools")]
impl Path {
    pub fn make_flat(
        &self,
        transform_to_parent: bool,
        closed: bool,
        skinned: bool,
    ) -> Option<FlattenedPath> {
        if self.vertices.is_empty() {
            return None;
        }
        let mut transform = if skinned {
            Mat2D::identity()
        } else {
            self.path_transform()
        };
        if transform_to_parent {
            if let Some(world) = self.base.parent_handle().and_then(|parent| {
                parent
                    .with(|object| {
                        object
                            .as_transform_component()
                            .map(|parent| parent.world_transform())
                    })
                    .flatten()
            }) {
                transform = world.invert_or_identity() * transform;
            }
        }
        let mut flat = FlattenedPath {
            vertices: Vec::new(),
        };
        let vertices = &self.vertices;
        let length = vertices.len();
        let mut previous_index = closed.then_some(length - 1);
        let mut previous_generated: Option<RenderVertex> = None;
        for index in 0..length {
            let vertex = Self::render_vertex(&vertices[index])?;
            if let RenderVertex::Straight {
                translation: position,
                radius,
            } = vertex
                && radius > 0.0
                && (closed || (index != 0 && index != length - 1))
            {
                let previous_vertex = match previous_generated {
                    Some(vertex) => vertex,
                    None => Self::read_vertex(
                        &vertices[previous_index.expect("rounded vertex has predecessor")],
                        VertexRead::Outgoing,
                    )?,
                };
                let next =
                    Self::read_vertex(&vertices[(index + 1) % length], VertexRead::Incoming)?;
                let mut to_prev = previous_vertex.outgoing() - position;
                let previous_length = to_prev.normalize_length();
                let mut to_next = next.incoming() - position;
                let next_length = to_next.normalize_length();
                let radius = cpp_min(previous_length / 2.0, cpp_min(next_length / 2.0, radius));
                let ideal = Self::compute_ideal_control_point_distance(to_prev, to_next, radius);
                let translation = Vec2D::scale_and_add(position, to_prev, radius);
                let out = Vec2D::scale_and_add(position, to_prev, radius - ideal);
                flat.add_vertex(
                    RenderVertex::Cubic {
                        translation,
                        in_point: translation,
                        out_point: out,
                    },
                    transform,
                );
                let translation = Vec2D::scale_and_add(position, to_next, radius);
                let incoming = Vec2D::scale_and_add(position, to_next, radius - ideal);
                let generated = RenderVertex::Cubic {
                    translation,
                    in_point: incoming,
                    out_point: translation,
                };
                flat.add_vertex(generated, transform);
                previous_generated = Some(generated);
                previous_index = None;
            } else {
                previous_generated = None;
                previous_index = Some(index);
                flat.add_vertex(vertex, transform);
            }
        }
        Some(flat)
    }
}

#[cfg(test)]
mod path_owner_tests {
    use super::*;

    fn straight(x: f32, y: f32) -> Rc<RefCell<StraightVertex>> {
        let mut vertex = StraightVertex::default();
        vertex.vertex_mut().base.set_x_value(x);
        vertex.vertex_mut().base.set_y_value(y);
        Rc::new(RefCell::new(vertex))
    }

    #[test]
    fn single_vertex_returns_before_reading_live_vertex() {
        let vertex = straight(1.0, 2.0);
        let mut path = Path::default();
        path.add_runtime_straight_vertex(vertex.clone());
        let mut output = RawPath::default();
        output.move_to(10.0, 20.0);
        let before = output.clone();
        // Upstream checks length before invoking any vertex getter. Holding
        // this actual borrow detects accidental eager materialization.
        let _borrow = vertex.borrow_mut();
        path.build_path(&mut output, true, false);
        assert_eq!(output, before);
    }

    #[test]
    fn build_appends_open_and_closed_geometry_without_snapshots() {
        let mut path = Path::default();
        path.add_runtime_straight_vertex(straight(1.0, 2.0));
        path.add_runtime_straight_vertex(straight(3.0, 4.0));
        for closed in [false, true] {
            let mut output = RawPath::default();
            output.move_to(10.0, 20.0);
            output.close();
            path.build_path(&mut output, closed, false);
            let mut expected = RawPath::default();
            expected.move_to(10.0, 20.0);
            expected.close();
            expected.move_to(1.0, 2.0);
            expected.line_to(3.0, 4.0);
            if closed {
                expected.line_to(1.0, 2.0);
                expected.close();
            }
            assert_eq!(output, expected);
        }
    }

    #[test]
    fn neighbour_read_does_not_compute_unused_cubic_control() {
        let vertex = Rc::new(RefCell::new(CubicDetachedVertex::default()));
        let occurrence = PathVertexOccurrence::RuntimeCubicDetached(vertex.clone());
        assert!(!vertex.borrow().cubic_vertex().cells().in_valid());
        assert!(!vertex.borrow().cubic_vertex().cells().out_valid());
        Path::read_vertex(&occurrence, VertexRead::Incoming).unwrap();
        assert!(vertex.borrow().cubic_vertex().cells().in_valid());
        assert!(!vertex.borrow().cubic_vertex().cells().out_valid());
        Path::read_vertex(&occurrence, VertexRead::Outgoing).unwrap();
        assert!(vertex.borrow().cubic_vertex().cells().out_valid());
    }

    #[test]
    fn source_min_keeps_first_operand_for_nan_and_equal_zero() {
        let nan = f32::from_bits(0x7fc0_1234);
        assert_eq!(cpp_min(nan, 1.0).to_bits(), nan.to_bits());
        assert_eq!(cpp_min(1.0, nan), 1.0);
        assert_eq!(cpp_min(-0.0, 0.0).to_bits(), (-0.0f32).to_bits());
        assert_eq!(cpp_min(0.0, -0.0).to_bits(), 0.0f32.to_bits());
    }

    #[cfg(feature = "tools")]
    #[test]
    fn flat_path_uses_explicit_open_virtual_context() {
        let mut path = Path::default();
        path.add_runtime_straight_vertex(straight(1.0, 2.0));
        path.add_runtime_straight_vertex(straight(3.0, 4.0));
        let flat = path.make_flat(false, false, false).unwrap();
        assert_eq!(flat.vertices().len(), 2);
        assert_eq!(
            flat.vertices()[0].render_translation(),
            Vec2D::new(1.0, 2.0)
        );
        assert_eq!(
            flat.vertices()[1].render_translation(),
            Vec2D::new(3.0, 4.0)
        );
    }
}

#[cfg(test)]
#[path = "legacy_node_alias_tests.rs"]
mod legacy_node_alias_tests;
