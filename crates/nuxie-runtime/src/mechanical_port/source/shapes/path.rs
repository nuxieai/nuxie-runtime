use crate::mechanical_port::source::{
    component::{ComponentDirt, has_dirt},
    core::{CoreContext, StatusCode},
    math::{aabb::Aabb, mat2d::Mat2D, math_types::PI, raw_path::RawPath, vec2d::Vec2D},
    shapes::{
        cubic_vertex::CubicVertex, deformer::RenderPathDeformer, path_flags::PathFlags,
        path_vertex::PathVertex, shape::Shape, straight_vertex::StraightVertex,
    },
};

pub struct Path {
    pub base: PathBase,
    shape: Option<*mut Shape>,
    vertices: Vec<PathVertex>,
    deferred_path_dirt: bool,
    path_flags: PathFlags,
    raw_path: RawPath,
}

impl Path {
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

    pub fn on_added_clean(&mut self, context: &mut CoreContext) -> StatusCode {
        let code = self.base.on_added_clean(context);
        if code != StatusCode::Ok {
            return code;
        }
        let mut parent = self.base.parent_mut();
        while let Some(current) = parent {
            if let Some(shape) = current.as_shape_mut() {
                self.shape = Some(shape);
                shape.add_path(self);
                return StatusCode::Ok;
            }
            parent = current.parent_mut();
        }
        StatusCode::MissingObject
    }

    pub fn build_dependencies(&mut self) {
        self.base.build_dependencies();
    }
    pub fn add_vertex(&mut self, vertex: PathVertex) {
        self.vertices.push(vertex);
    }
    pub fn pop_vertex(&mut self) {
        self.vertices.pop();
    }
    pub fn add_flags(&mut self, flags: PathFlags) {
        self.path_flags |= flags;
    }
    pub fn is_flagged(&self, flags: PathFlags) -> bool {
        !(self.path_flags & flags).is_empty()
    }
    pub fn can_defer_path_update(&self) -> bool {
        self.shape.map(|p| unsafe { &*p }).is_some_and(|shape| {
            shape.can_defer_path_update()
                && !shape.is_flagged(PathFlags::FOLLOW_PATH)
                && !self.is_flagged(PathFlags::FOLLOW_PATH | PathFlags::CLIPPING)
        })
    }
    pub fn path_transform(&self) -> Mat2D {
        self.base.world_transform()
    }
    pub fn local_bounds(&self) -> Aabb {
        self.raw_path.precise_bounds()
    }
    pub fn raw_path(&self) -> &RawPath {
        &self.raw_path
    }
    pub fn needs_path_build(&self) -> bool {
        self.base.has_dirt(ComponentDirt::PATH) || self.deferred_path_dirt
    }
    pub fn try_property_bounds(&self, _result: &mut Aabb) -> bool {
        false
    }
    pub fn is_path_closed(&self) -> bool {
        true
    }
    pub fn is_hidden(&self) -> bool {
        self.base.path_flags() & 1 == 1
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
        let max_radius = bounds.width().min(bounds.height()) * 0.5;
        let mut start = Vec2D::default();
        for (index, (position, to_prev, to_next, authored_radius)) in
            corners.into_iter().enumerate()
        {
            if authored_radius != 0.0 {
                let radius = authored_radius.abs().min(max_radius);
                let ideal = Self::compute_ideal_control_point_distance(to_prev, to_next, radius);
                let enter = Vec2D::scale_and_add(position, to_prev, radius);
                if index == 0 {
                    start = enter;
                    raw_path.move_to(enter);
                } else {
                    raw_path.line_to(enter);
                }
                let mut out = Vec2D::scale_and_add(position, to_prev, radius - ideal);
                let mut incoming = Vec2D::scale_and_add(position, to_next, radius - ideal);
                let exit = Vec2D::scale_and_add(position, to_next, radius);
                if authored_radius < 0.0 {
                    rotate_points(exit, enter, position, &mut out, &mut incoming);
                }
                raw_path.cubic_to(out, incoming, exit);
            } else if index == 0 {
                start = position;
                raw_path.move_to(position);
            } else {
                raw_path.line_to(position);
            }
        }
        raw_path.line_to(start);
        raw_path.close();
    }

    pub fn build_path(&self, raw_path: &mut RawPath) {
        let closed = self.is_path_closed();
        let length = self.vertices.len();
        if length < 2 {
            return;
        }
        let first = &self.vertices[0];
        let mut out;
        let mut previous_cubic;
        let start;
        let start_in;
        let start_cubic;
        if let Some(cubic) = first.as_cubic_vertex() {
            start_cubic = true;
            previous_cubic = true;
            start_in = cubic.render_in();
            out = cubic.render_out();
            start = cubic.render_translation();
            raw_path.move_to(start);
        } else {
            start_cubic = false;
            previous_cubic = false;
            let point = first.as_straight_vertex().unwrap();
            let radius = point.radius();
            if radius != 0.0 {
                let previous = &self.vertices[length - 1];
                let position = point.render_translation();
                let mut to_prev = previous
                    .as_cubic_vertex()
                    .map(CubicVertex::render_out)
                    .unwrap_or_else(|| previous.render_translation())
                    - position;
                let previous_length = to_prev.normalize_length();
                let next = &self.vertices[1];
                let mut to_next = next
                    .as_cubic_vertex()
                    .map(CubicVertex::render_in)
                    .unwrap_or_else(|| next.render_translation())
                    - position;
                let next_length = to_next.normalize_length();
                let render_radius =
                    (previous_length / 2.0).min((next_length / 2.0).min(radius.abs()));
                let ideal =
                    Self::compute_ideal_control_point_distance(to_prev, to_next, render_radius);
                start = Vec2D::scale_and_add(position, to_prev, render_radius);
                start_in = start;
                raw_path.move_to(start_in);
                let mut out_point = Vec2D::scale_and_add(position, to_prev, render_radius - ideal);
                let mut in_point = Vec2D::scale_and_add(position, to_next, render_radius - ideal);
                out = Vec2D::scale_and_add(position, to_next, render_radius);
                if radius < 0.0 {
                    rotate_points(out, start_in, position, &mut out_point, &mut in_point);
                }
                raw_path.cubic_to(out_point, in_point, out);
            } else {
                out = point.render_translation();
                start = out;
                start_in = out;
                raw_path.move_to(out);
            }
        }
        for index in 1..length {
            let vertex = &self.vertices[index];
            if let Some(cubic) = vertex.as_cubic_vertex() {
                let incoming = cubic.render_in();
                let translation = cubic.render_translation();
                raw_path.cubic_to(out, incoming, translation);
                previous_cubic = true;
                out = cubic.render_out();
            } else {
                let point = vertex.as_straight_vertex().unwrap();
                let position = point.render_translation();
                let radius = point.radius();
                if radius != 0.0 {
                    let previous = &self.vertices[index - 1];
                    let mut to_prev = previous
                        .as_cubic_vertex()
                        .map(CubicVertex::render_out)
                        .unwrap_or_else(|| previous.render_translation())
                        - position;
                    let previous_length = to_prev.normalize_length();
                    let next = &self.vertices[(index + 1) % length];
                    let mut to_next = next
                        .as_cubic_vertex()
                        .map(CubicVertex::render_in)
                        .unwrap_or_else(|| next.render_translation())
                        - position;
                    let next_length = to_next.normalize_length();
                    let render_radius =
                        (previous_length / 2.0).min((next_length / 2.0).min(radius.abs()));
                    let ideal =
                        Self::compute_ideal_control_point_distance(to_prev, to_next, render_radius);
                    let translation = Vec2D::scale_and_add(position, to_prev, render_radius);
                    if previous_cubic {
                        raw_path.cubic_to(out, translation, translation);
                    } else {
                        raw_path.line_to(translation);
                    }
                    let mut out_point =
                        Vec2D::scale_and_add(position, to_prev, render_radius - ideal);
                    let mut in_point =
                        Vec2D::scale_and_add(position, to_next, render_radius - ideal);
                    out = Vec2D::scale_and_add(position, to_next, render_radius);
                    if radius < 0.0 {
                        rotate_points(out, translation, position, &mut out_point, &mut in_point);
                    }
                    raw_path.cubic_to(out_point, in_point, out);
                    previous_cubic = false;
                } else if previous_cubic {
                    raw_path.cubic_to(out, position, position);
                    previous_cubic = false;
                    out = position;
                } else {
                    out = position;
                    raw_path.line_to(out);
                }
            }
        }
        if closed {
            if previous_cubic || start_cubic {
                raw_path.cubic_to(out, start_in, start);
            } else {
                raw_path.line_to(start);
            }
            raw_path.close();
        }
        if let Some(deformer) = self.deformer() {
            let transform = self.path_transform();
            deformer.deform_local_render_path(raw_path, transform, transform.invert_or_identity());
        }
    }

    fn deformer(&self) -> Option<&dyn RenderPathDeformer> {
        self.shape.map(|p| unsafe { &*p }).and_then(Shape::deformer)
    }
    pub fn mark_path_dirty(&mut self, _send_to_layout: bool) {
        self.base.add_dirt(ComponentDirt::PATH);
        if let Some(shape) = self.shape {
            unsafe { &mut *shape }.path_changed();
        }
    }
    pub fn on_dirty(&mut self, value: ComponentDirt) {
        if has_dirt(
            value,
            ComponentDirt::WORLD_TRANSFORM | ComponentDirt::N_SLICER,
        ) {
            if let Some(shape) = self.shape {
                unsafe { &mut *shape }.path_changed();
            }
        }
        if self.deferred_path_dirt {
            self.base.add_dirt(ComponentDirt::PATH);
        }
    }
    pub fn update(&mut self, value: ComponentDirt) {
        self.base.update(value);
        let changed = has_dirt(value, ComponentDirt::PATH);
        let world_changed = has_dirt(value, ComponentDirt::WORLD_TRANSFORM);
        let deformer_changed = has_dirt(value, ComponentDirt::N_SLICER);
        if changed || deformer_changed || (self.deformer().is_some() && world_changed) {
            if self.can_defer_path_update() {
                self.deferred_path_dirt = true;
                return;
            }
            self.deferred_path_dirt = false;
            let mut path = std::mem::take(&mut self.raw_path);
            path.rewind();
            self.build_path(&mut path);
            self.raw_path = path;
        }
    }
    pub fn is_hole_changed(&mut self) {
        self.mark_path_dirty(true);
    }
    pub fn collapse(&mut self, value: bool) -> bool {
        let changed = self.base.collapse(value);
        if changed {
            if let Some(shape) = self.shape {
                unsafe { &mut *shape }.path_collapse_changed();
            }
        }
        changed
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
    let (s, c) = (angle * 2.0).sin_cos();
    *out -= previous;
    *out = Vec2D::new(out.x * c - out.y * s, out.x * s + out.y * c) + previous;
    let (s, c) = (-angle * 2.0).sin_cos();
    *incoming -= next;
    *incoming = Vec2D::new(
        incoming.x * c - incoming.y * s,
        incoming.x * s + incoming.y * c,
    ) + next;
}

#[cfg(feature = "enable_query_flat_vertices")]
pub struct FlattenedPath {
    vertices: Vec<PathVertex>,
}
#[cfg(feature = "enable_query_flat_vertices")]
impl FlattenedPath {
    pub fn vertices(&self) -> &[PathVertex] {
        &self.vertices
    }
    pub fn add_vertex(&mut self, vertex: &PathVertex, transform: Mat2D) {
        if let Some(cubic) = vertex.as_cubic_vertex() {
            self.vertices.push(PathVertex::display_cubic(
                transform * cubic.render_in(),
                transform * cubic.render_out(),
                transform * cubic.render_translation(),
            ));
        } else {
            let translation = transform * vertex.render_translation();
            self.vertices
                .push(PathVertex::new(translation.x, translation.y));
        }
    }
}

#[cfg(feature = "enable_query_flat_vertices")]
impl Path {
    pub fn make_flat(&self, transform_to_parent: bool) -> Option<FlattenedPath> {
        if self.vertices.is_empty() {
            return None;
        }
        let mut transform = self.path_transform();
        if transform_to_parent {
            if let Some(parent) = self.base.parent().as_transform_component() {
                transform = parent.world_transform().invert_or_identity() * transform;
            }
        }
        let mut flat = FlattenedPath {
            vertices: Vec::new(),
        };
        let length = self.vertices.len();
        let mut previous = self
            .is_path_closed()
            .then(|| self.vertices[length - 1].clone());
        for (index, vertex) in self.vertices.iter().enumerate() {
            if let Some(point) = vertex.as_straight_vertex().filter(|point| {
                point.radius() > 0.0
                    && (self.is_path_closed() || (index != 0 && index != length - 1))
            }) {
                let next = &self.vertices[(index + 1) % length];
                let previous_vertex = previous.as_ref().unwrap();
                let previous_point = previous_vertex
                    .as_cubic_vertex()
                    .map(CubicVertex::render_out)
                    .unwrap_or_else(|| previous_vertex.render_translation());
                let next_point = next
                    .as_cubic_vertex()
                    .map(CubicVertex::render_in)
                    .unwrap_or_else(|| next.render_translation());
                let position = point.render_translation();
                let mut to_prev = previous_point - position;
                let previous_length = to_prev.normalize_length();
                let mut to_next = next_point - position;
                let next_length = to_next.normalize_length();
                let radius = (previous_length / 2.0).min((next_length / 2.0).min(point.radius()));
                let ideal = Self::compute_ideal_control_point_distance(to_prev, to_next, radius);
                let translation = Vec2D::scale_and_add(position, to_prev, radius);
                let out = Vec2D::scale_and_add(position, to_prev, radius - ideal);
                flat.add_vertex(
                    &PathVertex::display_cubic(translation, out, translation),
                    transform,
                );
                let translation = Vec2D::scale_and_add(position, to_next, radius);
                let incoming = Vec2D::scale_and_add(position, to_next, radius - ideal);
                let generated = PathVertex::display_cubic(incoming, translation, translation);
                flat.add_vertex(&generated, transform);
                previous = Some(generated);
            } else {
                previous = Some(vertex.clone());
                flat.add_vertex(vertex, transform);
            }
        }
        Some(flat)
    }
}
