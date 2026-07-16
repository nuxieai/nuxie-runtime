//! Coarse Rust port of `renderer/src/gr_triangulator.cpp` and
//! `renderer/src/gr_inner_fan_triangulator.hpp`.
//!
//! The upstream implementation stores a mutable planar mesh in pointer-linked
//! arena objects. This port uses stable integer handles into owned arenas so
//! topology rewrites remain explicit and memory-safe.

#![allow(dead_code)]

use crate::gpu::TriangleVertex;
use nuxie_render_api::{FillRule, Mat2D, PathVerb, RawPath, Vec2D};

type VertexId = usize;
type EdgeId = usize;
type MonotoneId = usize;
type PolyId = usize;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SweepDirection {
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum WindingFaces {
    Negative,
    Positive,
    All,
}

impl WindingFaces {
    fn includes(self, weight: i16) -> bool {
        match self {
            Self::Negative => weight < 0,
            Self::Positive => weight >= 0,
            Self::All => true,
        }
    }
}

impl SweepDirection {
    fn less(self, a: Vec2D, b: Vec2D) -> bool {
        match self {
            // A horizontal sweep is a 90-degree counterclockwise rotation,
            // not a transpose, so its secondary Y ordering is descending.
            Self::Horizontal => a.x < b.x || (a.x == b.x && a.y > b.y),
            Self::Vertical => a.y < b.y || (a.y == b.y && a.x < b.x),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Side {
    Left,
    Right,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EdgeType {
    Inner,
    Outer,
    Connector,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CheckResult {
    NoIntersection,
    FoundIntersection,
    Failed,
}

#[derive(Clone, Copy, Debug)]
struct Vertex {
    point: Vec2D,
    prev: Option<VertexId>,
    next: Option<VertexId>,
    first_edge_above: Option<EdgeId>,
    last_edge_above: Option<EdgeId>,
    first_edge_below: Option<EdgeId>,
    last_edge_below: Option<EdgeId>,
    left_enclosing_edge: Option<EdgeId>,
    right_enclosing_edge: Option<EdgeId>,
    partner: Option<VertexId>,
    alpha: u8,
    synthetic: bool,
}

impl Vertex {
    fn new(point: Vec2D, alpha: u8) -> Self {
        Self {
            point,
            prev: None,
            next: None,
            first_edge_above: None,
            last_edge_above: None,
            first_edge_below: None,
            last_edge_below: None,
            left_enclosing_edge: None,
            right_enclosing_edge: None,
            partner: None,
            alpha,
            synthetic: false,
        }
    }

    fn is_connected(self) -> bool {
        self.first_edge_above.is_some() || self.first_edge_below.is_some()
    }
}

#[derive(Clone, Copy, Debug)]
struct Line {
    a: f64,
    b: f64,
    c: f64,
}

impl Line {
    fn new(p: Vec2D, q: Vec2D) -> Self {
        Self {
            a: f64::from(q.y) - f64::from(p.y),
            b: f64::from(p.x) - f64::from(q.x),
            c: f64::from(p.y) * f64::from(q.x) - f64::from(p.x) * f64::from(q.y),
        }
    }

    fn distance(self, point: Vec2D) -> f64 {
        self.a * f64::from(point.x) + self.b * f64::from(point.y) + self.c
    }
}

#[derive(Clone, Copy, Debug)]
struct Edge {
    alive: bool,
    winding: i32,
    top: VertexId,
    bottom: VertexId,
    edge_type: EdgeType,
    left: Option<EdgeId>,
    right: Option<EdgeId>,
    prev_edge_above: Option<EdgeId>,
    next_edge_above: Option<EdgeId>,
    prev_edge_below: Option<EdgeId>,
    next_edge_below: Option<EdgeId>,
    left_poly: Option<PolyId>,
    right_poly: Option<PolyId>,
    left_poly_prev: Option<EdgeId>,
    left_poly_next: Option<EdgeId>,
    right_poly_prev: Option<EdgeId>,
    right_poly_next: Option<EdgeId>,
    used_in_left_poly: bool,
    used_in_right_poly: bool,
    line: Line,
}

impl Edge {
    fn new(
        top: VertexId,
        bottom: VertexId,
        winding: i32,
        edge_type: EdgeType,
        vertices: &[Vertex],
    ) -> Self {
        Self {
            alive: true,
            winding,
            top,
            bottom,
            edge_type,
            left: None,
            right: None,
            prev_edge_above: None,
            next_edge_above: None,
            prev_edge_below: None,
            next_edge_below: None,
            left_poly: None,
            right_poly: None,
            left_poly_prev: None,
            left_poly_next: None,
            right_poly_prev: None,
            right_poly_next: None,
            used_in_left_poly: false,
            used_in_right_poly: false,
            line: Line::new(vertices[top].point, vertices[bottom].point),
        }
    }

    fn distance(self, point: Vec2D, vertices: &[Vertex]) -> f64 {
        if point == vertices[self.top].point || point == vertices[self.bottom].point {
            0.0
        } else {
            self.line.distance(point)
        }
    }

    fn intersect(self, other: Self, vertices: &[Vertex]) -> Option<(Vec2D, Option<u8>)> {
        if self.top == other.top
            || self.bottom == other.bottom
            || self.top == other.bottom
            || self.bottom == other.top
        {
            return None;
        }
        let (point, s, t) = recursive_edge_intersection(
            self.line,
            vertices[self.top].point,
            vertices[self.bottom].point,
            other.line,
            vertices[other.top].point,
            vertices[other.bottom].point,
        )?;
        let alpha = match (self.edge_type, other.edge_type) {
            (EdgeType::Inner, _) | (_, EdgeType::Inner) => Some(255),
            (EdgeType::Outer, EdgeType::Outer) => Some(0),
            _ => {
                let interpolate = |top: u8, bottom: u8, amount: f64| {
                    (1.0 - amount) * f64::from(top) + amount * f64::from(bottom)
                };
                Some(
                    interpolate(vertices[self.top].alpha, vertices[self.bottom].alpha, s).max(
                        interpolate(vertices[other.top].alpha, vertices[other.bottom].alpha, t),
                    ) as u8,
                )
            }
        };
        Some((point, alpha))
    }
}

#[derive(Clone, Copy, Debug)]
struct MonotonePoly {
    side: Side,
    first_edge: Option<EdgeId>,
    last_edge: Option<EdgeId>,
    prev: Option<MonotoneId>,
    next: Option<MonotoneId>,
    winding: i32,
}

#[derive(Clone, Copy, Debug)]
struct Poly {
    first_vertex: VertexId,
    winding: i32,
    head: Option<MonotoneId>,
    tail: Option<MonotoneId>,
    next: Option<PolyId>,
    partner: Option<PolyId>,
    count: usize,
}

#[derive(Default)]
struct Mesh {
    vertices: Vec<Vertex>,
    edges: Vec<Edge>,
    monotones: Vec<MonotonePoly>,
    polys: Vec<Poly>,
    breadcrumbs: Vec<[Vec2D; 3]>,
    sorted_head: Option<VertexId>,
    sorted_tail: Option<VertexId>,
}

pub(crate) struct InnerFanTriangulator {
    mesh: Mesh,
    poly_head: Option<PolyId>,
    fill_rule: FillRule,
    reverse_triangles: bool,
    negate_winding: bool,
}

impl InnerFanTriangulator {
    pub(crate) fn new(
        path: &RawPath,
        view: Mat2D,
        direction: SweepDirection,
        fill_rule: FillRule,
    ) -> Self {
        let determinant = view.0[0] * view.0[3] - view.0[2] * view.0[1];
        let fill_rule = if fill_rule == FillRule::EvenOdd {
            FillRule::EvenOdd
        } else {
            FillRule::NonZero
        };
        let mut mesh = Mesh::from_path(path, direction);
        let poly_head = mesh
            .simplify(direction)
            .and_then(|_| mesh.tessellate())
            .ok()
            .flatten();
        Self {
            mesh,
            poly_head,
            fill_rule,
            reverse_triangles: determinant < 0.0,
            negate_winding: false,
        }
    }

    pub(crate) fn negate_winding(&mut self) {
        self.negate_winding = !self.negate_winding;
    }

    pub(crate) fn max_vertex_count(&self) -> usize {
        self.triangles(0, WindingFaces::All).len()
    }

    pub(crate) fn triangles(&self, path_id: u16, faces: WindingFaces) -> Vec<TriangleVertex> {
        self.mesh.emit_triangles(
            self.poly_head,
            self.fill_rule,
            path_id,
            self.reverse_triangles,
            self.negate_winding,
            faces,
        )
    }

    pub(crate) fn grout_triangles(&self) -> &[[Vec2D; 3]] {
        &self.mesh.breadcrumbs
    }
}

#[derive(Clone, Copy, Debug, Default)]
struct ActiveEdgeList {
    head: Option<EdgeId>,
    tail: Option<EdgeId>,
}

impl ActiveEdgeList {
    fn contains(self, mesh: &Mesh, edge: EdgeId) -> bool {
        mesh.edges[edge].left.is_some()
            || mesh.edges[edge].right.is_some()
            || self.head == Some(edge)
    }

    fn insert_after(&mut self, mesh: &mut Mesh, edge: EdgeId, previous: Option<EdgeId>) -> bool {
        if self.contains(mesh, edge) {
            return false;
        }
        let next = previous
            .map(|previous| mesh.edges[previous].right)
            .unwrap_or(self.head);
        mesh.edges[edge].left = previous;
        mesh.edges[edge].right = next;
        if let Some(previous) = previous {
            mesh.edges[previous].right = Some(edge);
        } else {
            self.head = Some(edge);
        }
        if let Some(next) = next {
            mesh.edges[next].left = Some(edge);
        } else {
            self.tail = Some(edge);
        }
        true
    }

    fn remove(&mut self, mesh: &mut Mesh, edge: EdgeId) -> bool {
        if !self.contains(mesh, edge) {
            return false;
        }
        let Edge { left, right, .. } = mesh.edges[edge];
        if let Some(left) = left {
            mesh.edges[left].right = right;
        } else {
            self.head = right;
        }
        if let Some(right) = right {
            mesh.edges[right].left = left;
        } else {
            self.tail = left;
        }
        mesh.edges[edge].left = None;
        mesh.edges[edge].right = None;
        true
    }

    fn iter<'a>(self, mesh: &'a Mesh) -> impl Iterator<Item = EdgeId> + 'a {
        std::iter::successors(self.head, |&edge| mesh.edges[edge].right)
    }
}

impl Mesh {
    fn from_path(path: &RawPath, direction: SweepDirection) -> Self {
        let contours = path_to_contours(path);
        let mut mesh = Self::default();
        for contour in contours {
            let first = mesh.vertices.len();
            mesh.vertices
                .extend(contour.iter().copied().map(|point| Vertex::new(point, 255)));
            let count = contour.len();
            for offset in 0..count {
                let vertex = first + offset;
                mesh.vertices[vertex].prev = Some(first + (offset + count - 1) % count);
                mesh.vertices[vertex].next = Some(first + (offset + 1) % count);
            }
            for offset in 0..count {
                mesh.connect(first + offset, first + (offset + 1) % count, direction);
            }
        }
        mesh.sort_vertices(direction);
        mesh
    }

    fn connect(&mut self, previous: VertexId, next: VertexId, direction: SweepDirection) {
        if self.vertices[previous].point == self.vertices[next].point {
            return;
        }
        let winding = if direction.less(self.vertices[previous].point, self.vertices[next].point) {
            1
        } else {
            -1
        };
        let (top, bottom) = if winding < 0 {
            (next, previous)
        } else {
            (previous, next)
        };
        let edge_id = self.edges.len();
        self.edges.push(Edge::new(
            top,
            bottom,
            winding,
            EdgeType::Inner,
            &self.vertices,
        ));
        self.insert_edge_below(edge_id, top, direction);
        self.insert_edge_above(edge_id, bottom, direction);
        self.merge_collinear_edges(edge_id, direction);
    }

    fn edge_is_right_of(&self, edge: EdgeId, vertex: VertexId) -> bool {
        self.edges[edge].distance(self.vertices[vertex].point, &self.vertices) < 0.0
    }

    fn remove_edge_above(&mut self, edge: EdgeId) {
        let record = self.edges[edge];
        let vertex = record.bottom;
        if let Some(previous) = record.prev_edge_above {
            self.edges[previous].next_edge_above = record.next_edge_above;
        } else {
            self.vertices[vertex].first_edge_above = record.next_edge_above;
        }
        if let Some(next) = record.next_edge_above {
            self.edges[next].prev_edge_above = record.prev_edge_above;
        } else {
            self.vertices[vertex].last_edge_above = record.prev_edge_above;
        }
        self.edges[edge].prev_edge_above = None;
        self.edges[edge].next_edge_above = None;
    }

    fn remove_edge_below(&mut self, edge: EdgeId) {
        let record = self.edges[edge];
        let vertex = record.top;
        if let Some(previous) = record.prev_edge_below {
            self.edges[previous].next_edge_below = record.next_edge_below;
        } else {
            self.vertices[vertex].first_edge_below = record.next_edge_below;
        }
        if let Some(next) = record.next_edge_below {
            self.edges[next].prev_edge_below = record.prev_edge_below;
        } else {
            self.vertices[vertex].last_edge_below = record.prev_edge_below;
        }
        self.edges[edge].prev_edge_below = None;
        self.edges[edge].next_edge_below = None;
    }

    fn disconnect_edge(&mut self, edge: EdgeId) {
        self.remove_edge_above(edge);
        self.remove_edge_below(edge);
    }

    fn append_breadcrumb(&mut self, edge: EdgeId, split: VertexId) {
        let record = self.edges[edge];
        let mut a = self.vertices[record.top].point;
        let mut b = self.vertices[record.bottom].point;
        let c = self.vertices[split].point;
        if a == b || a == c || b == c || record.winding == 0 {
            return;
        }
        let mut winding = record.winding;
        if winding < 0 {
            std::mem::swap(&mut a, &mut b);
            winding = -winding;
        }
        self.breadcrumbs
            .extend(std::iter::repeat_n([a, b, c], winding as usize));
    }

    fn set_top(&mut self, edge: EdgeId, vertex: VertexId, direction: SweepDirection) {
        self.remove_edge_below(edge);
        self.append_breadcrumb(edge, vertex);
        self.edges[edge].top = vertex;
        if self.vertices[vertex].point == self.vertices[self.edges[edge].bottom].point {
            self.remove_edge_above(edge);
            self.edges[edge].alive = false;
            return;
        }
        self.edges[edge].line = Line::new(
            self.vertices[vertex].point,
            self.vertices[self.edges[edge].bottom].point,
        );
        self.insert_edge_below(edge, vertex, direction);
    }

    fn set_bottom(&mut self, edge: EdgeId, vertex: VertexId, direction: SweepDirection) {
        self.remove_edge_above(edge);
        self.append_breadcrumb(edge, vertex);
        self.edges[edge].bottom = vertex;
        if self.vertices[self.edges[edge].top].point == self.vertices[vertex].point {
            self.remove_edge_below(edge);
            self.edges[edge].alive = false;
            return;
        }
        self.edges[edge].line = Line::new(
            self.vertices[self.edges[edge].top].point,
            self.vertices[vertex].point,
        );
        self.insert_edge_above(edge, vertex, direction);
    }

    fn split_edge(
        &mut self,
        edge: EdgeId,
        vertex: VertexId,
        direction: SweepDirection,
    ) -> Option<EdgeId> {
        let record = self.edges[edge];
        if !record.alive || vertex == record.top || vertex == record.bottom {
            return None;
        }
        let mut winding = record.winding;
        let (top, bottom) =
            if direction.less(self.vertices[vertex].point, self.vertices[record.top].point) {
                winding = -winding;
                self.set_top(edge, vertex, direction);
                (vertex, record.top)
            } else if direction.less(
                self.vertices[record.bottom].point,
                self.vertices[vertex].point,
            ) {
                winding = -winding;
                self.set_bottom(edge, vertex, direction);
                (record.bottom, vertex)
            } else {
                self.set_bottom(edge, vertex, direction);
                (vertex, record.bottom)
            };
        let new_edge = self.edges.len();
        self.edges.push(Edge::new(
            top,
            bottom,
            winding,
            record.edge_type,
            &self.vertices,
        ));
        self.insert_edge_below(new_edge, top, direction);
        self.insert_edge_above(new_edge, bottom, direction);
        self.merge_collinear_edges(new_edge, direction);
        Some(new_edge)
    }

    fn top_collinear(&self, left: EdgeId, right: EdgeId) -> bool {
        let (left, right) = (self.edges[left], self.edges[right]);
        left.alive
            && right.alive
            && (self.vertices[left.top].point == self.vertices[right.top].point
                || left.distance(self.vertices[right.top].point, &self.vertices) <= 0.0
                || right.distance(self.vertices[left.top].point, &self.vertices) >= 0.0)
    }

    fn bottom_collinear(&self, left: EdgeId, right: EdgeId) -> bool {
        let (left, right) = (self.edges[left], self.edges[right]);
        left.alive
            && right.alive
            && (self.vertices[left.bottom].point == self.vertices[right.bottom].point
                || left.distance(self.vertices[right.bottom].point, &self.vertices) <= 0.0
                || right.distance(self.vertices[left.bottom].point, &self.vertices) >= 0.0)
    }

    fn merge_edges_above(&mut self, edge: EdgeId, other: EdgeId, direction: SweepDirection) {
        let (edge_record, other_record) = (self.edges[edge], self.edges[other]);
        if self.vertices[edge_record.top].point == self.vertices[other_record.top].point {
            self.edges[other].winding += edge_record.winding;
            self.disconnect_edge(edge);
            self.edges[edge].alive = false;
        } else if direction.less(
            self.vertices[edge_record.top].point,
            self.vertices[other_record.top].point,
        ) {
            self.edges[other].winding += edge_record.winding;
            self.set_bottom(edge, other_record.top, direction);
        } else {
            self.edges[edge].winding += other_record.winding;
            self.set_bottom(other, edge_record.top, direction);
        }
    }

    fn merge_edges_below(&mut self, edge: EdgeId, other: EdgeId, direction: SweepDirection) {
        let (edge_record, other_record) = (self.edges[edge], self.edges[other]);
        if self.vertices[edge_record.bottom].point == self.vertices[other_record.bottom].point {
            self.edges[other].winding += edge_record.winding;
            self.disconnect_edge(edge);
            self.edges[edge].alive = false;
        } else if direction.less(
            self.vertices[edge_record.bottom].point,
            self.vertices[other_record.bottom].point,
        ) {
            self.edges[edge].winding += other_record.winding;
            self.set_top(other, edge_record.bottom, direction);
        } else {
            self.edges[other].winding += edge_record.winding;
            self.set_top(edge, other_record.bottom, direction);
        }
    }

    fn merge_collinear_edges(&mut self, edge: EdgeId, direction: SweepDirection) {
        while self.edges[edge].alive {
            let record = self.edges[edge];
            if let Some(previous) = record.prev_edge_above {
                if self.top_collinear(previous, edge) {
                    self.merge_edges_above(previous, edge, direction);
                    continue;
                }
            }
            if let Some(next) = record.next_edge_above {
                if self.top_collinear(edge, next) {
                    self.merge_edges_above(next, edge, direction);
                    continue;
                }
            }
            if let Some(previous) = record.prev_edge_below {
                if self.bottom_collinear(previous, edge) {
                    self.merge_edges_below(previous, edge, direction);
                    continue;
                }
            }
            if let Some(next) = record.next_edge_below {
                if self.bottom_collinear(edge, next) {
                    self.merge_edges_below(next, edge, direction);
                    continue;
                }
            }
            break;
        }
    }

    fn merge_coincident_vertices(&mut self, direction: SweepDirection) -> bool {
        let mut merged = false;
        let mut source = self.sorted_head.and_then(|head| self.vertices[head].next);
        while let Some(vertex) = source {
            let next = self.vertices[vertex].next;
            let destination = self.vertices[vertex].prev.unwrap();
            if direction.less(
                self.vertices[vertex].point,
                self.vertices[destination].point,
            ) {
                self.vertices[vertex].point = self.vertices[destination].point;
            }
            if self.vertices[vertex].point == self.vertices[destination].point {
                self.merge_vertex_into(vertex, destination, direction);
                merged = true;
            }
            source = next;
        }
        merged
    }

    fn merge_vertex_into(
        &mut self,
        source: VertexId,
        destination: VertexId,
        direction: SweepDirection,
    ) {
        self.vertices[destination].alpha = self.vertices[destination]
            .alpha
            .max(self.vertices[source].alpha);
        if let Some(partner) = self.vertices[source].partner {
            self.vertices[partner].partner = Some(destination);
        }
        while let Some(edge) = self.vertices[source].first_edge_above {
            self.set_bottom(edge, destination, direction);
            if self.edges[edge].alive {
                self.merge_collinear_edges(edge, direction);
            }
        }
        while let Some(edge) = self.vertices[source].first_edge_below {
            self.set_top(edge, destination, direction);
            if self.edges[edge].alive {
                self.merge_collinear_edges(edge, direction);
            }
        }
        let (previous, next) = (self.vertices[source].prev, self.vertices[source].next);
        if let Some(previous) = previous {
            self.vertices[previous].next = next;
        } else {
            self.sorted_head = next;
        }
        if let Some(next) = next {
            self.vertices[next].prev = previous;
        } else {
            self.sorted_tail = previous;
        }
        self.vertices[source].prev = None;
        self.vertices[source].next = None;
        self.vertices[destination].synthetic = true;
    }

    fn make_sorted_vertex(
        &mut self,
        point: Vec2D,
        alpha: u8,
        reference: Option<VertexId>,
        direction: SweepDirection,
    ) -> VertexId {
        let mut previous = reference;
        while let Some(vertex) = previous {
            if !direction.less(point, self.vertices[vertex].point) {
                break;
            }
            previous = self.vertices[vertex].prev;
        }
        let mut next = previous
            .and_then(|vertex| self.vertices[vertex].next)
            .or(self.sorted_head);
        while let Some(vertex) = next {
            if !direction.less(self.vertices[vertex].point, point) {
                break;
            }
            previous = next;
            next = self.vertices[vertex].next;
        }
        if let Some(vertex) = previous.filter(|&vertex| self.vertices[vertex].point == point) {
            self.vertices[vertex].alpha = self.vertices[vertex].alpha.max(alpha);
            return vertex;
        }
        if let Some(vertex) = next.filter(|&vertex| self.vertices[vertex].point == point) {
            self.vertices[vertex].alpha = self.vertices[vertex].alpha.max(alpha);
            return vertex;
        }
        let vertex = self.vertices.len();
        let mut record = Vertex::new(point, alpha);
        record.prev = previous;
        record.next = next;
        self.vertices.push(record);
        if let Some(previous) = previous {
            self.vertices[previous].next = Some(vertex);
        } else {
            self.sorted_head = Some(vertex);
        }
        if let Some(next) = next {
            self.vertices[next].prev = Some(vertex);
        } else {
            self.sorted_tail = Some(vertex);
        }
        vertex
    }

    fn check_and_split_intersection(
        &mut self,
        left: EdgeId,
        right: EdgeId,
        current: VertexId,
        direction: SweepDirection,
    ) -> Option<VertexId> {
        let (mut point, alpha) = self.edges[left].intersect(self.edges[right], &self.vertices)?;
        if !point.x.is_finite() || !point.y.is_finite() {
            return None;
        }
        let mut reference = Some(current);
        while let Some(vertex) = reference {
            if !direction.less(point, self.vertices[vertex].point) {
                break;
            }
            reference = self.vertices[vertex].prev;
        }
        let left_record = self.edges[left];
        let right_record = self.edges[right];
        point = clamp_to_edge_box(
            point,
            self.vertices[left_record.top].point,
            self.vertices[left_record.bottom].point,
            direction,
        );
        point = clamp_to_edge_box(
            point,
            self.vertices[right_record.top].point,
            self.vertices[right_record.bottom].point,
            direction,
        );
        let vertex = [
            left_record.top,
            left_record.bottom,
            right_record.top,
            right_record.bottom,
        ]
        .into_iter()
        .find(|&vertex| self.vertices[vertex].point == point)
        .unwrap_or_else(|| {
            self.make_sorted_vertex(point, alpha.unwrap_or(255), reference, direction)
        });
        self.split_edge(left, vertex, direction);
        self.split_edge(right, vertex, direction);
        self.vertices[vertex].alpha = self.vertices[vertex].alpha.max(alpha.unwrap_or(255));
        Some(vertex)
    }

    fn rewind(
        &mut self,
        active: &mut ActiveEdgeList,
        current: &mut VertexId,
        mut destination: VertexId,
        direction: SweepDirection,
    ) -> bool {
        if *current == destination
            || direction.less(
                self.vertices[*current].point,
                self.vertices[destination].point,
            )
        {
            return true;
        }
        let mut vertex = *current;
        while vertex != destination {
            let Some(previous) = self.vertices[vertex].prev else {
                return false;
            };
            vertex = previous;
            let below = self.edges_below(vertex);
            for edge in below {
                if self.edges[edge].alive && !active.remove(self, edge) {
                    return false;
                }
            }
            let mut left_edge = self.vertices[vertex].left_enclosing_edge;
            let above = self.edges_above(vertex);
            for edge in above {
                if !self.edges[edge].alive || !active.insert_after(self, edge, left_edge) {
                    return false;
                }
                left_edge = Some(edge);
                let top = self.edges[edge].top;
                if direction.less(self.vertices[top].point, self.vertices[destination].point) {
                    let left_bad = self.vertices[top].left_enclosing_edge.is_some_and(|left| {
                        self.edges[left].distance(self.vertices[top].point, &self.vertices) <= 0.0
                    });
                    let right_bad = self.vertices[top]
                        .right_enclosing_edge
                        .is_some_and(|right| {
                            self.edges[right].distance(self.vertices[top].point, &self.vertices)
                                >= 0.0
                        });
                    if left_bad || right_bad {
                        destination = top;
                    }
                }
            }
        }
        *current = vertex;
        true
    }

    fn rewind_if_necessary(
        &mut self,
        edge: EdgeId,
        active: &mut ActiveEdgeList,
        current: &mut VertexId,
        direction: SweepDirection,
    ) -> bool {
        if !self.edges[edge].alive {
            return false;
        }
        let record = self.edges[edge];
        if let Some(left) = record.left {
            let neighbor = self.edges[left];
            if neighbor.alive {
                let destination = if direction.less(
                    self.vertices[neighbor.top].point,
                    self.vertices[record.top].point,
                ) && neighbor
                    .distance(self.vertices[record.top].point, &self.vertices)
                    <= 0.0
                {
                    Some(neighbor.top)
                } else if direction.less(
                    self.vertices[record.top].point,
                    self.vertices[neighbor.top].point,
                ) && record.distance(self.vertices[neighbor.top].point, &self.vertices)
                    >= 0.0
                {
                    Some(record.top)
                } else if direction.less(
                    self.vertices[record.bottom].point,
                    self.vertices[neighbor.bottom].point,
                ) && neighbor.distance(self.vertices[record.bottom].point, &self.vertices)
                    <= 0.0
                {
                    Some(neighbor.top)
                } else if direction.less(
                    self.vertices[neighbor.bottom].point,
                    self.vertices[record.bottom].point,
                ) && record.distance(self.vertices[neighbor.bottom].point, &self.vertices)
                    >= 0.0
                {
                    Some(record.top)
                } else {
                    None
                };
                if destination.is_some_and(|destination| {
                    !self.rewind(active, current, destination, direction)
                }) {
                    return false;
                }
            }
        }
        if let Some(right) = self.edges[edge].right {
            let record = self.edges[edge];
            let neighbor = self.edges[right];
            if neighbor.alive {
                let destination = if direction.less(
                    self.vertices[neighbor.top].point,
                    self.vertices[record.top].point,
                ) && neighbor
                    .distance(self.vertices[record.top].point, &self.vertices)
                    >= 0.0
                {
                    Some(neighbor.top)
                } else if direction.less(
                    self.vertices[record.top].point,
                    self.vertices[neighbor.top].point,
                ) && record.distance(self.vertices[neighbor.top].point, &self.vertices)
                    <= 0.0
                {
                    Some(record.top)
                } else if direction.less(
                    self.vertices[record.bottom].point,
                    self.vertices[neighbor.bottom].point,
                ) && neighbor.distance(self.vertices[record.bottom].point, &self.vertices)
                    >= 0.0
                {
                    Some(neighbor.top)
                } else if direction.less(
                    self.vertices[neighbor.bottom].point,
                    self.vertices[record.bottom].point,
                ) && record.distance(self.vertices[neighbor.bottom].point, &self.vertices)
                    <= 0.0
                {
                    Some(record.top)
                } else {
                    None
                };
                if destination.is_some_and(|destination| {
                    !self.rewind(active, current, destination, direction)
                }) {
                    return false;
                }
            }
        }
        true
    }

    fn set_top_sweep(
        &mut self,
        edge: EdgeId,
        vertex: VertexId,
        active: &mut ActiveEdgeList,
        current: &mut VertexId,
        direction: SweepDirection,
    ) -> bool {
        self.remove_edge_below(edge);
        self.append_breadcrumb(edge, vertex);
        self.edges[edge].top = vertex;
        if self.vertices[vertex].point == self.vertices[self.edges[edge].bottom].point {
            self.remove_edge_above(edge);
            self.edges[edge].alive = false;
            return true;
        }
        self.edges[edge].line = Line::new(
            self.vertices[vertex].point,
            self.vertices[self.edges[edge].bottom].point,
        );
        self.insert_edge_below(edge, vertex, direction);
        self.rewind_if_necessary(edge, active, current, direction)
            && self.merge_collinear_edges_sweep(edge, active, current, direction)
    }

    fn set_bottom_sweep(
        &mut self,
        edge: EdgeId,
        vertex: VertexId,
        active: &mut ActiveEdgeList,
        current: &mut VertexId,
        direction: SweepDirection,
    ) -> bool {
        self.remove_edge_above(edge);
        self.append_breadcrumb(edge, vertex);
        self.edges[edge].bottom = vertex;
        if self.vertices[self.edges[edge].top].point == self.vertices[vertex].point {
            self.remove_edge_below(edge);
            self.edges[edge].alive = false;
            return true;
        }
        self.edges[edge].line = Line::new(
            self.vertices[self.edges[edge].top].point,
            self.vertices[vertex].point,
        );
        self.insert_edge_above(edge, vertex, direction);
        self.rewind_if_necessary(edge, active, current, direction)
            && self.merge_collinear_edges_sweep(edge, active, current, direction)
    }

    fn merge_edges_above_sweep(
        &mut self,
        edge: EdgeId,
        other: EdgeId,
        active: &mut ActiveEdgeList,
        current: &mut VertexId,
        direction: SweepDirection,
    ) -> bool {
        let (edge_record, other_record) = (self.edges[edge], self.edges[other]);
        if self.vertices[edge_record.top].point == self.vertices[other_record.top].point {
            if !self.rewind(active, current, edge_record.top, direction) {
                return false;
            }
            self.edges[other].winding += edge_record.winding;
            self.disconnect_edge(edge);
            self.edges[edge].alive = false;
        } else if direction.less(
            self.vertices[edge_record.top].point,
            self.vertices[other_record.top].point,
        ) {
            if !self.rewind(active, current, edge_record.top, direction) {
                return false;
            }
            self.edges[other].winding += edge_record.winding;
            return self.set_bottom_sweep(edge, other_record.top, active, current, direction);
        } else {
            if !self.rewind(active, current, other_record.top, direction) {
                return false;
            }
            self.edges[edge].winding += other_record.winding;
            return self.set_bottom_sweep(other, edge_record.top, active, current, direction);
        }
        true
    }

    fn merge_edges_below_sweep(
        &mut self,
        edge: EdgeId,
        other: EdgeId,
        active: &mut ActiveEdgeList,
        current: &mut VertexId,
        direction: SweepDirection,
    ) -> bool {
        let (edge_record, other_record) = (self.edges[edge], self.edges[other]);
        if self.vertices[edge_record.bottom].point == self.vertices[other_record.bottom].point {
            if !self.rewind(active, current, edge_record.top, direction) {
                return false;
            }
            self.edges[other].winding += edge_record.winding;
            self.disconnect_edge(edge);
            self.edges[edge].alive = false;
        } else if direction.less(
            self.vertices[edge_record.bottom].point,
            self.vertices[other_record.bottom].point,
        ) {
            if !self.rewind(active, current, other_record.top, direction) {
                return false;
            }
            self.edges[edge].winding += other_record.winding;
            return self.set_top_sweep(other, edge_record.bottom, active, current, direction);
        } else {
            if !self.rewind(active, current, edge_record.top, direction) {
                return false;
            }
            self.edges[other].winding += edge_record.winding;
            return self.set_top_sweep(edge, other_record.bottom, active, current, direction);
        }
        true
    }

    fn merge_collinear_edges_sweep(
        &mut self,
        edge: EdgeId,
        active: &mut ActiveEdgeList,
        current: &mut VertexId,
        direction: SweepDirection,
    ) -> bool {
        while self.edges[edge].alive {
            let record = self.edges[edge];
            if let Some(previous) = record.prev_edge_above {
                if self.top_collinear(previous, edge) {
                    if !self.merge_edges_above_sweep(previous, edge, active, current, direction) {
                        return false;
                    }
                    continue;
                }
            }
            if let Some(next) = record.next_edge_above {
                if self.top_collinear(edge, next) {
                    if !self.merge_edges_above_sweep(next, edge, active, current, direction) {
                        return false;
                    }
                    continue;
                }
            }
            if let Some(previous) = record.prev_edge_below {
                if self.bottom_collinear(previous, edge) {
                    if !self.merge_edges_below_sweep(previous, edge, active, current, direction) {
                        return false;
                    }
                    continue;
                }
            }
            if let Some(next) = record.next_edge_below {
                if self.bottom_collinear(edge, next) {
                    if !self.merge_edges_below_sweep(next, edge, active, current, direction) {
                        return false;
                    }
                    continue;
                }
            }
            break;
        }
        true
    }

    fn split_edge_sweep(
        &mut self,
        edge: EdgeId,
        vertex: VertexId,
        active: &mut ActiveEdgeList,
        current: &mut VertexId,
        direction: SweepDirection,
    ) -> CheckResult {
        let record = self.edges[edge];
        if !record.alive || vertex == record.top || vertex == record.bottom {
            return CheckResult::NoIntersection;
        }
        let mut winding = record.winding;
        let (top, bottom, success) =
            if direction.less(self.vertices[vertex].point, self.vertices[record.top].point) {
                winding = -winding;
                (
                    vertex,
                    record.top,
                    self.set_top_sweep(edge, vertex, active, current, direction),
                )
            } else if direction.less(
                self.vertices[record.bottom].point,
                self.vertices[vertex].point,
            ) {
                winding = -winding;
                (
                    record.bottom,
                    vertex,
                    self.set_bottom_sweep(edge, vertex, active, current, direction),
                )
            } else {
                (
                    vertex,
                    record.bottom,
                    self.set_bottom_sweep(edge, vertex, active, current, direction),
                )
            };
        if !success {
            return CheckResult::Failed;
        }
        let new_edge = self.allocate_edge(top, bottom, winding, record.edge_type);
        self.insert_edge_below(new_edge, top, direction);
        self.insert_edge_above(new_edge, bottom, direction);
        if !self.merge_collinear_edges_sweep(new_edge, active, current, direction) {
            return CheckResult::Failed;
        }
        CheckResult::FoundIntersection
    }

    fn edges_above(&self, vertex: VertexId) -> Vec<EdgeId> {
        std::iter::successors(self.vertices[vertex].first_edge_above, |&edge| {
            self.edges[edge].next_edge_above
        })
        .collect()
    }

    fn edges_below(&self, vertex: VertexId) -> Vec<EdgeId> {
        std::iter::successors(self.vertices[vertex].first_edge_below, |&edge| {
            self.edges[edge].next_edge_below
        })
        .collect()
    }

    fn check_for_intersection(
        &mut self,
        left: Option<EdgeId>,
        right: Option<EdgeId>,
        active: &mut ActiveEdgeList,
        current: &mut VertexId,
        direction: SweepDirection,
    ) -> CheckResult {
        let (Some(left), Some(right)) = (left, right) else {
            return CheckResult::NoIntersection;
        };
        if !self.edges[left].alive || !self.edges[right].alive {
            return CheckResult::Failed;
        }
        if let Some((mut point, alpha)) =
            self.edges[left].intersect(self.edges[right], &self.vertices)
        {
            if !point.x.is_finite() || !point.y.is_finite() {
                return self.intersect_edge_pair(left, right, active, current, direction);
            }
            let mut reference = Some(*current);
            while let Some(vertex) = reference {
                if !direction.less(point, self.vertices[vertex].point) {
                    break;
                }
                reference = self.vertices[vertex].prev;
            }
            let left_record = self.edges[left];
            let right_record = self.edges[right];
            point = clamp_to_edge_box(
                point,
                self.vertices[left_record.top].point,
                self.vertices[left_record.bottom].point,
                direction,
            );
            point = clamp_to_edge_box(
                point,
                self.vertices[right_record.top].point,
                self.vertices[right_record.bottom].point,
                direction,
            );
            let vertex = [
                left_record.top,
                left_record.bottom,
                right_record.top,
                right_record.bottom,
            ]
            .into_iter()
            .find(|&vertex| self.vertices[vertex].point == point)
            .unwrap_or_else(|| {
                self.make_sorted_vertex(point, alpha.unwrap_or(255), reference, direction)
            });
            let rewind_to = reference.unwrap_or(vertex);
            if !self.rewind(active, current, rewind_to, direction) {
                return CheckResult::Failed;
            }
            if self.split_edge_sweep(left, vertex, active, current, direction)
                == CheckResult::Failed
                || self.split_edge_sweep(right, vertex, active, current, direction)
                    == CheckResult::Failed
            {
                return CheckResult::Failed;
            }
            self.vertices[vertex].alpha = self.vertices[vertex].alpha.max(alpha.unwrap_or(255));
            return CheckResult::FoundIntersection;
        }
        self.intersect_edge_pair(left, right, active, current, direction)
    }

    fn intersect_edge_pair(
        &mut self,
        left: EdgeId,
        right: EdgeId,
        active: &mut ActiveEdgeList,
        current: &mut VertexId,
        direction: SweepDirection,
    ) -> CheckResult {
        let (left_record, right_record) = (self.edges[left], self.edges[right]);
        if !left_record.alive
            || !right_record.alive
            || left_record.top == right_record.top
            || left_record.bottom == right_record.bottom
        {
            return CheckResult::NoIntersection;
        }
        let mut split = None;
        if direction.less(
            self.vertices[left_record.top].point,
            self.vertices[right_record.top].point,
        ) {
            if left_record.distance(self.vertices[right_record.top].point, &self.vertices) <= 0.0 {
                split = Some((left, right_record.top));
            }
        } else if right_record.distance(self.vertices[left_record.top].point, &self.vertices) >= 0.0
        {
            split = Some((right, left_record.top));
        }
        if direction.less(
            self.vertices[right_record.bottom].point,
            self.vertices[left_record.bottom].point,
        ) {
            if left_record.distance(self.vertices[right_record.bottom].point, &self.vertices) <= 0.0
            {
                split = Some((left, right_record.bottom));
            }
        } else if right_record.distance(self.vertices[left_record.bottom].point, &self.vertices)
            >= 0.0
        {
            split = Some((right, left_record.bottom));
        }
        let Some((edge, vertex)) = split else {
            return CheckResult::NoIntersection;
        };
        let top = self.edges[edge].top;
        if !self.rewind(active, current, top, direction) {
            return CheckResult::Failed;
        }
        self.split_edge_sweep(edge, vertex, active, current, direction)
    }

    fn simplify(&mut self, direction: SweepDirection) -> Result<bool, ()> {
        self.merge_coincident_vertices(direction);
        let initial_edges = self.edges.len().max(1);
        let mut intersection_count = 0usize;
        let mut found_intersection = false;
        let mut active = ActiveEdgeList::default();
        let mut current = self.sorted_head;
        while let Some(mut vertex) = current {
            if !self.vertices[vertex].is_connected() {
                current = self.vertices[vertex].next;
                continue;
            }
            if self.edges.len() > 170 * initial_edges || intersection_count > 500_000 {
                return Err(());
            }
            let (left_enclosing, right_enclosing) = loop {
                let (left_enclosing, right_enclosing) = self.enclosing_edges(vertex, active);
                self.vertices[vertex].left_enclosing_edge = left_enclosing;
                self.vertices[vertex].right_enclosing_edge = right_enclosing;
                let mut result = CheckResult::NoIntersection;
                let below = self.edges_below(vertex);
                if below.is_empty() {
                    result = self.check_for_intersection(
                        left_enclosing,
                        right_enclosing,
                        &mut active,
                        &mut vertex,
                        direction,
                    );
                } else {
                    for edge in below {
                        result = self.check_for_intersection(
                            left_enclosing,
                            Some(edge),
                            &mut active,
                            &mut vertex,
                            direction,
                        );
                        if result == CheckResult::NoIntersection {
                            result = self.check_for_intersection(
                                Some(edge),
                                right_enclosing,
                                &mut active,
                                &mut vertex,
                                direction,
                            );
                        }
                        if result != CheckResult::NoIntersection {
                            break;
                        }
                    }
                }
                match result {
                    CheckResult::Failed => return Err(()),
                    CheckResult::FoundIntersection => {
                        found_intersection = true;
                        intersection_count += 1;
                        continue;
                    }
                    CheckResult::NoIntersection => break (left_enclosing, right_enclosing),
                }
            };

            for edge in self.edges_above(vertex) {
                if self.edges[edge].alive && !active.remove(self, edge) {
                    return Err(());
                }
            }
            let mut left_edge = left_enclosing;
            for edge in self.edges_below(vertex) {
                if self.edges[edge].alive {
                    if !active.insert_after(self, edge, left_edge) {
                        return Err(());
                    }
                    left_edge = Some(edge);
                }
            }
            let _ = right_enclosing;
            current = self.vertices[vertex].next;
        }
        if active.head.is_some() || active.tail.is_some() {
            return Err(());
        }
        Ok(found_intersection)
    }

    fn allocate_edge(
        &mut self,
        top: VertexId,
        bottom: VertexId,
        winding: i32,
        edge_type: EdgeType,
    ) -> EdgeId {
        let edge = self.edges.len();
        self.edges
            .push(Edge::new(top, bottom, winding, edge_type, &self.vertices));
        edge
    }

    fn make_poly(&mut self, vertex: VertexId, winding: i32, head: &mut Option<PolyId>) -> PolyId {
        let poly = self.polys.len();
        self.polys.push(Poly {
            first_vertex: vertex,
            winding,
            head: None,
            tail: None,
            next: *head,
            partner: None,
            count: 0,
        });
        *head = Some(poly);
        poly
    }

    fn allocate_monotone(&mut self, edge: EdgeId, side: Side, winding: i32) -> MonotoneId {
        let monotone = self.monotones.len();
        self.monotones.push(MonotonePoly {
            side,
            first_edge: Some(edge),
            last_edge: Some(edge),
            prev: None,
            next: None,
            winding,
        });
        self.link_edge_to_monotone(monotone, edge, false);
        monotone
    }

    fn link_edge_to_monotone(&mut self, monotone: MonotoneId, edge: EdgeId, append: bool) {
        let side = self.monotones[monotone].side;
        let previous = append
            .then_some(self.monotones[monotone].last_edge)
            .flatten();
        match side {
            Side::Right => {
                assert!(!self.edges[edge].used_in_right_poly);
                self.edges[edge].right_poly_prev = previous;
                self.edges[edge].right_poly_next = None;
                if let Some(previous) = previous {
                    self.edges[previous].right_poly_next = Some(edge);
                }
                self.edges[edge].used_in_right_poly = true;
            }
            Side::Left => {
                assert!(!self.edges[edge].used_in_left_poly);
                self.edges[edge].left_poly_prev = previous;
                self.edges[edge].left_poly_next = None;
                if let Some(previous) = previous {
                    self.edges[previous].left_poly_next = Some(edge);
                }
                self.edges[edge].used_in_left_poly = true;
            }
        }
        if !append {
            self.monotones[monotone].first_edge = Some(edge);
        }
        self.monotones[monotone].last_edge = Some(edge);
    }

    fn poly_last_vertex(&self, poly: PolyId) -> VertexId {
        self.polys[poly]
            .tail
            .and_then(|tail| self.monotones[tail].last_edge)
            .map(|edge| self.edges[edge].bottom)
            .unwrap_or(self.polys[poly].first_vertex)
    }

    fn poly_add_edge(&mut self, poly: PolyId, edge: EdgeId, side: Side) -> PolyId {
        if match side {
            Side::Right => self.edges[edge].used_in_right_poly,
            Side::Left => self.edges[edge].used_in_left_poly,
        } {
            return poly;
        }
        let partner = self.polys[poly].partner;
        if let Some(partner) = partner {
            self.polys[poly].partner = None;
            self.polys[partner].partner = None;
        }
        let Some(tail) = self.polys[poly].tail else {
            let monotone = self.allocate_monotone(edge, side, self.polys[poly].winding);
            self.polys[poly].head = Some(monotone);
            self.polys[poly].tail = Some(monotone);
            self.polys[poly].count += 2;
            return poly;
        };
        let tail_edge = self.monotones[tail].last_edge.unwrap();
        if self.edges[edge].bottom == self.edges[tail_edge].bottom {
            return poly;
        }
        if side == self.monotones[tail].side {
            self.link_edge_to_monotone(tail, edge, true);
            self.polys[poly].count += 1;
            return poly;
        }

        let connector = self.allocate_edge(
            self.edges[tail_edge].bottom,
            self.edges[edge].bottom,
            1,
            EdgeType::Inner,
        );
        self.link_edge_to_monotone(tail, connector, true);
        self.polys[poly].count += 1;
        if let Some(partner) = partner {
            self.poly_add_edge(partner, connector, side)
        } else {
            let monotone = self.allocate_monotone(connector, side, self.polys[poly].winding);
            self.monotones[monotone].prev = Some(tail);
            self.monotones[tail].next = Some(monotone);
            self.polys[poly].tail = Some(monotone);
            poly
        }
    }

    fn tessellate(&mut self) -> Result<Option<PolyId>, ()> {
        let mut active = ActiveEdgeList::default();
        let mut poly_head = None;
        let vertices = self.sorted_vertices().collect::<Vec<_>>();
        for vertex in vertices {
            if !self.vertices[vertex].is_connected() {
                continue;
            }
            let (left_enclosing, right_enclosing) = self.enclosing_edges(vertex, active);
            let first_above = self.vertices[vertex].first_edge_above;
            let last_above = self.vertices[vertex].last_edge_above;
            let first_below = self.vertices[vertex].first_edge_below;
            let (mut left_poly, mut right_poly) =
                if let (Some(first), Some(last)) = (first_above, last_above) {
                    (self.edges[first].left_poly, self.edges[last].right_poly)
                } else {
                    (
                        left_enclosing.and_then(|edge| self.edges[edge].right_poly),
                        right_enclosing.and_then(|edge| self.edges[edge].left_poly),
                    )
                };

            if let (Some(first), Some(last)) = (first_above, last_above) {
                if let Some(poly) = left_poly {
                    left_poly = Some(self.poly_add_edge(poly, first, Side::Right));
                }
                if let Some(poly) = right_poly {
                    right_poly = Some(self.poly_add_edge(poly, last, Side::Left));
                }
                let mut edge = first;
                while edge != last {
                    let Some(right_edge) = self.edges[edge].next_edge_above else {
                        return Err(());
                    };
                    if !active.remove(self, edge) {
                        return Err(());
                    }
                    if let Some(poly) = self.edges[edge].right_poly {
                        self.poly_add_edge(poly, edge, Side::Left);
                    }
                    if let Some(poly) = self.edges[right_edge].left_poly {
                        if Some(poly) != self.edges[edge].right_poly {
                            self.poly_add_edge(poly, edge, Side::Right);
                        }
                    }
                    edge = right_edge;
                }
                if !active.remove(self, last) {
                    return Err(());
                }
                if first_below.is_none() {
                    if let (Some(left), Some(right)) = (left_poly, right_poly) {
                        if left != right {
                            self.polys[left].partner = Some(right);
                            self.polys[right].partner = Some(left);
                        }
                    }
                }
            }

            if let Some(first_below) = first_below {
                if first_above.is_none() {
                    if let (Some(mut left), Some(mut right)) = (left_poly, right_poly) {
                        if left == right {
                            let split_left = self.polys[left]
                                .tail
                                .is_some_and(|tail| self.monotones[tail].side == Side::Left);
                            if split_left {
                                left = self.make_poly(
                                    self.poly_last_vertex(left),
                                    self.polys[left].winding,
                                    &mut poly_head,
                                );
                                let enclosing = left_enclosing.ok_or(())?;
                                self.edges[enclosing].right_poly = Some(left);
                            } else {
                                right = self.make_poly(
                                    self.poly_last_vertex(right),
                                    self.polys[right].winding,
                                    &mut poly_head,
                                );
                                let enclosing = right_enclosing.ok_or(())?;
                                self.edges[enclosing].left_poly = Some(right);
                            }
                        }
                        let join = self.allocate_edge(
                            self.poly_last_vertex(left),
                            vertex,
                            1,
                            EdgeType::Inner,
                        );
                        left_poly = Some(self.poly_add_edge(left, join, Side::Right));
                        right_poly = Some(self.poly_add_edge(right, join, Side::Left));
                    }
                }
                self.edges[first_below].left_poly = left_poly;
                if !active.insert_after(self, first_below, left_enclosing) {
                    return Err(());
                }
                let mut left_edge = first_below;
                let mut right_edge = self.edges[left_edge].next_edge_below;
                while let Some(edge) = right_edge {
                    if !active.insert_after(self, edge, Some(left_edge)) {
                        return Err(());
                    }
                    let winding = self.edges[left_edge]
                        .left_poly
                        .map(|poly| self.polys[poly].winding)
                        .unwrap_or(0)
                        + self.edges[left_edge].winding;
                    if winding != 0 {
                        let poly = self.make_poly(vertex, winding, &mut poly_head);
                        self.edges[left_edge].right_poly = Some(poly);
                        self.edges[edge].left_poly = Some(poly);
                    }
                    left_edge = edge;
                    right_edge = self.edges[edge].next_edge_below;
                }
                let last = self.vertices[vertex].last_edge_below.unwrap();
                self.edges[last].right_poly = right_poly;
            }
        }
        if active.head.is_some() || active.tail.is_some() {
            return Err(());
        }
        Ok(poly_head)
    }

    fn emit_monotone(
        &self,
        monotone: MonotoneId,
        path_id: u16,
        reverse: bool,
        negate_winding: bool,
        faces: WindingFaces,
        output: &mut Vec<TriangleVertex>,
    ) {
        let record = self.monotones[monotone];
        let mut weight = i16::try_from(-record.winding).expect("triangulator winding fits i16");
        if negate_winding {
            weight = -weight;
        }
        if !faces.includes(weight) {
            return;
        }
        let mut vertices = Vec::new();
        let first = record.first_edge.unwrap();
        vertices.push(self.edges[first].top);
        let mut edge = Some(first);
        while let Some(current) = edge {
            match record.side {
                Side::Right => {
                    vertices.push(self.edges[current].bottom);
                    edge = self.edges[current].right_poly_next;
                }
                Side::Left => {
                    vertices.insert(0, self.edges[current].bottom);
                    edge = self.edges[current].left_poly_next;
                }
            }
        }
        let mut index = 1;
        while vertices.len() >= 3 && index + 1 < vertices.len() {
            let (previous, current, next) =
                (vertices[index - 1], vertices[index], vertices[index + 1]);
            let a = self.vertices[current].point;
            let b = self.vertices[previous].point;
            let c = self.vertices[next].point;
            let cross = f64::from(a.x - b.x) * f64::from(c.y - a.y)
                - f64::from(a.y - b.y) * f64::from(c.x - a.x);
            if vertices.len() == 3 || cross >= 0.0 {
                let mut triangle = [previous, current, next];
                if reverse {
                    triangle.swap(0, 2);
                }
                output.extend(triangle.map(|vertex| {
                    let point = self.vertices[vertex].point;
                    TriangleVertex::new([point.x, point.y], weight, path_id)
                }));
                if vertices.len() == 3 {
                    break;
                }
                vertices.remove(index);
                if index > 1 {
                    index -= 1;
                }
            } else {
                index += 1;
            }
        }
    }

    fn emit_triangles(
        &self,
        mut poly: Option<PolyId>,
        fill_rule: FillRule,
        path_id: u16,
        reverse: bool,
        negate_winding: bool,
        faces: WindingFaces,
    ) -> Vec<TriangleVertex> {
        let mut output = Vec::new();
        while let Some(poly_id) = poly {
            let record = self.polys[poly_id];
            let filled = match fill_rule {
                FillRule::EvenOdd => record.winding & 1 != 0,
                FillRule::NonZero | FillRule::Clockwise => record.winding != 0,
            };
            if filled && record.count >= 3 {
                let mut monotone = record.head;
                while let Some(monotone_id) = monotone {
                    self.emit_monotone(
                        monotone_id,
                        path_id,
                        reverse,
                        negate_winding,
                        faces,
                        &mut output,
                    );
                    monotone = self.monotones[monotone_id].next;
                }
            }
            poly = record.next;
        }
        output
    }

    fn insert_edge_above(&mut self, edge: EdgeId, vertex: VertexId, direction: SweepDirection) {
        let record = self.edges[edge];
        if self.vertices[record.top].point == self.vertices[record.bottom].point
            || direction.less(
                self.vertices[record.bottom].point,
                self.vertices[record.top].point,
            )
        {
            return;
        }
        let mut previous = None;
        let mut next = self.vertices[vertex].first_edge_above;
        while let Some(candidate) = next {
            if self.edge_is_right_of(candidate, record.top) {
                break;
            }
            previous = next;
            next = self.edges[candidate].next_edge_above;
        }
        self.edges[edge].prev_edge_above = previous;
        self.edges[edge].next_edge_above = next;
        if let Some(previous) = previous {
            self.edges[previous].next_edge_above = Some(edge);
        } else {
            self.vertices[vertex].first_edge_above = Some(edge);
        }
        if let Some(next) = next {
            self.edges[next].prev_edge_above = Some(edge);
        } else {
            self.vertices[vertex].last_edge_above = Some(edge);
        }
    }

    fn insert_edge_below(&mut self, edge: EdgeId, vertex: VertexId, direction: SweepDirection) {
        let record = self.edges[edge];
        if self.vertices[record.top].point == self.vertices[record.bottom].point
            || direction.less(
                self.vertices[record.bottom].point,
                self.vertices[record.top].point,
            )
        {
            return;
        }
        let mut previous = None;
        let mut next = self.vertices[vertex].first_edge_below;
        while let Some(candidate) = next {
            if self.edge_is_right_of(candidate, record.bottom) {
                break;
            }
            previous = next;
            next = self.edges[candidate].next_edge_below;
        }
        self.edges[edge].prev_edge_below = previous;
        self.edges[edge].next_edge_below = next;
        if let Some(previous) = previous {
            self.edges[previous].next_edge_below = Some(edge);
        } else {
            self.vertices[vertex].first_edge_below = Some(edge);
        }
        if let Some(next) = next {
            self.edges[next].prev_edge_below = Some(edge);
        } else {
            self.vertices[vertex].last_edge_below = Some(edge);
        }
    }

    fn sort_vertices(&mut self, direction: SweepDirection) {
        let mut order = (0..self.vertices.len()).collect::<Vec<_>>();
        order.sort_by(|&a, &b| {
            let a_point = self.vertices[a].point;
            let b_point = self.vertices[b].point;
            if direction.less(a_point, b_point) {
                std::cmp::Ordering::Less
            } else if direction.less(b_point, a_point) {
                std::cmp::Ordering::Greater
            } else {
                // C++ merge sort pulls from the back half on ties.
                b.cmp(&a)
            }
        });
        for (index, &vertex) in order.iter().enumerate() {
            self.vertices[vertex].prev = index.checked_sub(1).map(|i| order[i]);
            self.vertices[vertex].next = order.get(index + 1).copied();
        }
        self.sorted_head = order.first().copied();
        self.sorted_tail = order.last().copied();
    }

    fn sorted_vertices(&self) -> impl Iterator<Item = VertexId> + '_ {
        std::iter::successors(self.sorted_head, |&vertex| self.vertices[vertex].next)
    }

    fn enclosing_edges(
        &self,
        vertex: VertexId,
        active: ActiveEdgeList,
    ) -> (Option<EdgeId>, Option<EdgeId>) {
        let record = self.vertices[vertex];
        if let (Some(first), Some(last)) = (record.first_edge_above, record.last_edge_above) {
            return (self.edges[first].left, self.edges[last].right);
        }
        let mut next = None;
        let mut previous = active.tail;
        while let Some(edge) = previous {
            if self.edges[edge].distance(record.point, &self.vertices) > 0.0 {
                break;
            }
            next = previous;
            previous = self.edges[edge].left;
        }
        (previous, next)
    }
}

fn path_to_contours(path: &RawPath) -> Vec<Vec<Vec2D>> {
    let mut contours = Vec::<Vec<Vec2D>>::new();
    let mut contour = Vec::new();
    let mut point_index = 0;
    for verb in path.verbs() {
        let point_count = match verb {
            PathVerb::Move | PathVerb::Line => 1,
            PathVerb::Quad => 2,
            PathVerb::Cubic => 3,
            PathVerb::Close => 0,
        };
        if *verb == PathVerb::Move && !contour.is_empty() {
            sanitize_contour(&mut contour);
            if contour.len() >= 2 {
                contours.push(std::mem::take(&mut contour));
            } else {
                contour.clear();
            }
        }
        if point_count != 0 {
            // Interior triangulation supplies an already-linear scratch path.
            // Taking the endpoint for curves also matches upstream's
            // tolerance==0 path-to-contours behavior.
            let point = path.points()[point_index + point_count - 1];
            if point.x.is_finite() && point.y.is_finite() {
                contour.push(point);
            }
            point_index += point_count;
        }
    }
    sanitize_contour(&mut contour);
    if contour.len() >= 2 {
        contours.push(contour);
    }
    contours
}

fn sanitize_contour(contour: &mut Vec<Vec2D>) {
    if contour.len() > 1 && contour.first() == contour.last() {
        // Upstream walks from tail to head and removes the head on a cyclic
        // duplicate, preserving the authored tail as the contour endpoint.
        contour.remove(0);
    }
    let mut index = 1;
    while index < contour.len() {
        if contour[index - 1] == contour[index]
            || !contour[index].x.is_finite()
            || !contour[index].y.is_finite()
        {
            contour.remove(index);
        } else {
            index += 1;
        }
    }
}

fn clamp_to_edge_box(point: Vec2D, min: Vec2D, max: Vec2D, direction: SweepDirection) -> Vec2D {
    match direction {
        SweepDirection::Horizontal => Vec2D::new(
            point.x.clamp(min.x, max.x),
            point.y.clamp(min.y.min(max.y), min.y.max(max.y)),
        ),
        SweepDirection::Vertical => Vec2D::new(
            point.x.clamp(min.x.min(max.x), min.x.max(max.x)),
            point.y.clamp(min.y, max.y),
        ),
    }
}

fn clamped_f32(value: f64) -> f32 {
    const NEAR_ZERO: f64 = 16.0 * f32::MIN_POSITIVE as f64;
    if value.abs() < NEAR_ZERO {
        0.0
    } else {
        value.clamp(-(f32::MAX as f64), f32::MAX as f64) as f32
    }
}

fn exponent_for_recursion(value: f32) -> i32 {
    let value = value.abs();
    if value < 1.0 {
        0
    } else {
        ((value.to_bits() >> 23) & 0xff) as i32 - 127
    }
}

fn edge_line_needs_recursion(p0: Vec2D, p1: Vec2D) -> bool {
    (exponent_for_recursion(p0.x) - exponent_for_recursion(p1.x)).abs() > 20
        || (exponent_for_recursion(p0.y) - exponent_for_recursion(p1.y)).abs() > 20
}

fn recursive_edge_intersection(
    u: Line,
    mut u0: Vec2D,
    mut u1: Vec2D,
    v: Line,
    mut v0: Vec2D,
    mut v1: Vec2D,
) -> Option<(Vec2D, f64, f64)> {
    if u0.x.min(u1.x) > v0.x.max(v1.x)
        || u0.x.max(u1.x) < v0.x.min(v1.x)
        || u0.y.min(u1.y) > v0.y.max(v1.y)
        || u0.y.max(u1.y) < v0.y.min(v1.y)
    {
        return None;
    }

    let denominator = u.a * v.b - u.b * v.a;
    if denominator == 0.0 {
        return None;
    }
    let dx = f64::from(v0.x) - f64::from(u0.x);
    let dy = f64::from(v0.y) - f64::from(u0.y);
    let s_numerator = dy * v.b + dx * v.a;
    let t_numerator = dy * u.b + dx * u.a;
    let outside = if denominator > 0.0 {
        s_numerator < 0.0
            || s_numerator > denominator
            || t_numerator < 0.0
            || t_numerator > denominator
    } else {
        s_numerator > 0.0
            || s_numerator < denominator
            || t_numerator > 0.0
            || t_numerator < denominator
    };
    if outside {
        return None;
    }

    let s = s_numerator / denominator;
    let t = t_numerator / denominator;
    let split_u = edge_line_needs_recursion(u0, u1);
    let split_v = edge_line_needs_recursion(v0, v1);
    if !split_u && !split_v {
        return Some((
            Vec2D::new(
                clamped_f32(f64::from(u0.x) - s * u.b),
                clamped_f32(f64::from(u0.y) + s * u.a),
            ),
            s,
            t,
        ));
    }

    let (mut s_scale, mut s_shift) = (1.0, 0.0);
    let (mut t_scale, mut t_shift) = (1.0, 0.0);
    if split_u {
        let midpoint = Vec2D::new(0.5 * u0.x + 0.5 * u1.x, 0.5 * u0.y + 0.5 * u1.y);
        s_scale = 0.5;
        if s >= 0.5 {
            u0 = midpoint;
            s_shift = 0.5;
        } else {
            u1 = midpoint;
        }
    }
    if split_v {
        let midpoint = Vec2D::new(0.5 * v0.x + 0.5 * v1.x, 0.5 * v0.y + 0.5 * v1.y);
        t_scale = 0.5;
        if t >= 0.5 {
            v0 = midpoint;
            t_shift = 0.5;
        } else {
            v1 = midpoint;
        }
    }

    let (point, child_s, child_t) =
        recursive_edge_intersection(Line::new(u0, u1), u0, u1, Line::new(v0, v1), v0, v1)?;
    Some((
        point,
        s_scale * child_s + s_shift,
        t_scale * child_t + t_shift,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::work_metrics::{CountedDeviceExt, CountedQueueExt};
    use bytemuck::Zeroable;

    fn direct_grid_path() -> RawPath {
        let mut path = RawPath::new();
        for index in 0..50 {
            let position = index as f32 * 20.0;
            path.move_to(0.0, position);
            if index % 2 == 0 {
                path.line_to(0.0, position + 20.0);
                path.line_to(1000.0, position + 20.0);
                path.line_to(1000.0, position);
            } else {
                path.line_to(1000.0, position);
                path.line_to(1000.0, position + 20.0);
                path.line_to(0.0, position + 20.0);
            }
            path.close();
        }
        for index in 0..50 {
            let position = index as f32 * 20.0;
            path.move_to(position, 0.0);
            if index % 2 == 0 {
                path.line_to(position, 1000.0);
                path.line_to(position + 20.0, 1000.0);
                path.line_to(position + 20.0, 0.0);
            } else {
                path.line_to(position + 20.0, 0.0);
                path.line_to(position + 20.0, 1000.0);
                path.line_to(position, 1000.0);
            }
            path.close();
        }
        path
    }

    fn direct_flower_path() -> RawPath {
        let mut path = RawPath::new();
        path.move_to(833.333374, 500.0);
        path.cubic_to(
            1035.17468, 626.838745, 991.497986, 746.839539, 755.348145, 714.262573,
        );
        path.cubic_to(
            828.437256, 941.167725, 717.843933, 1005.01886, 557.88269, 828.269287,
        );
        path.cubic_to(
            468.020355, 1049.06946, 342.258209, 1026.89429, 333.333313, 788.67511,
        );
        path.cubic_to(
            122.567162, 900.055542, 40.4816399, 802.229858, 186.769104, 614.006653,
        );
        path.cubic_to(
            -46.2811317,
            563.851013,
            -46.2810974,
            436.148895,
            186.769135,
            385.993195,
        );
        path.cubic_to(
            40.4817047, 197.77002, 122.567123, 99.9444427, 333.333344, 211.324844,
        );
        path.cubic_to(
            342.258423,
            -26.8943138,
            468.020172,
            -49.0694847,
            557.882874,
            171.730774,
        );
        path.cubic_to(
            717.843994, -5.0188098, 828.437256, 58.8322334, 755.348206, 285.737518,
        );
        path.cubic_to(
            991.497986, 253.160507, 1035.17468, 373.161469, 833.333374, 500.000061,
        );
        path.close();
        path.move_to(750.0, 500.0);
        path.cubic_to(750.0, 637.97876, 637.97876, 750.0, 500.0, 750.0);
        path.cubic_to(362.02124, 750.0, 250.0, 637.97876, 250.0, 500.0);
        path.cubic_to(250.0, 362.02124, 362.02124, 250.0, 500.0, 250.0);
        path.cubic_to(637.97876, 250.0, 750.0, 362.02124, 750.0, 500.0);
        path.close();
        path
    }

    #[test]
    fn sweep_order_matches_upstream_rotation() {
        assert!(SweepDirection::Vertical.less(Vec2D::new(0.0, 1.0), Vec2D::new(0.0, 2.0)));
        assert!(SweepDirection::Vertical.less(Vec2D::new(1.0, 2.0), Vec2D::new(2.0, 2.0)));
        assert!(SweepDirection::Horizontal.less(Vec2D::new(1.0, 2.0), Vec2D::new(2.0, 0.0)));
        assert!(SweepDirection::Horizontal.less(Vec2D::new(2.0, 3.0), Vec2D::new(2.0, 1.0)));
    }

    #[test]
    fn crossing_segments_report_point_and_parameters() {
        let u0 = Vec2D::new(0.0, 0.0);
        let u1 = Vec2D::new(10.0, 10.0);
        let v0 = Vec2D::new(0.0, 10.0);
        let v1 = Vec2D::new(10.0, 0.0);
        let (point, s, t) =
            recursive_edge_intersection(Line::new(u0, u1), u0, u1, Line::new(v0, v1), v0, v1)
                .unwrap();
        assert_eq!(point, Vec2D::new(5.0, 5.0));
        assert_eq!(s, 0.5);
        assert_eq!(t, 0.5);
    }

    #[test]
    fn disjoint_and_parallel_segments_do_not_intersect() {
        let intersect = |u0, u1, v0, v1| {
            recursive_edge_intersection(Line::new(u0, u1), u0, u1, Line::new(v0, v1), v0, v1)
        };
        assert!(intersect(
            Vec2D::new(0.0, 0.0),
            Vec2D::new(1.0, 0.0),
            Vec2D::new(0.0, 1.0),
            Vec2D::new(1.0, 1.0),
        )
        .is_none());
        assert!(intersect(
            Vec2D::new(0.0, 0.0),
            Vec2D::new(1.0, 1.0),
            Vec2D::new(2.0, 0.0),
            Vec2D::new(3.0, 1.0),
        )
        .is_none());
    }

    #[test]
    fn large_magnitude_intersection_recurses_to_stable_segments() {
        let u0 = Vec2D::new(0.0, 0.0);
        let u1 = Vec2D::new(1.0e30, 1.0);
        let v0 = Vec2D::new(0.0, 1.0);
        let v1 = Vec2D::new(1.0e30, 0.0);
        let result =
            recursive_edge_intersection(Line::new(u0, u1), u0, u1, Line::new(v0, v1), v0, v1);
        let (point, s, t) = result.unwrap();
        assert!(point.x.is_finite());
        assert_eq!(point.y, 0.5);
        assert_eq!(s, 0.5);
        assert_eq!(t, 0.5);
    }

    #[test]
    fn path_mesh_closes_contours_and_preserves_authored_collinear_vertices() {
        let mut path = RawPath::new();
        path.move_to(0.0, 0.0);
        path.line_to(1.0, 0.0);
        path.line_to(2.0, 0.0);
        path.line_to(2.0, 1.0);
        path.close();
        let mesh = Mesh::from_path(&path, SweepDirection::Vertical);
        assert_eq!(mesh.vertices.len(), 4);
        assert_eq!(mesh.edges.len(), 4);
        assert_eq!(mesh.edges.iter().map(|edge| edge.winding).sum::<i32>(), 2);
        assert!(mesh.edges.iter().all(|edge| SweepDirection::Vertical.less(
            mesh.vertices[edge.top].point,
            mesh.vertices[edge.bottom].point
        )));
        let sorted = mesh
            .sorted_vertices()
            .map(|vertex| mesh.vertices[vertex].point)
            .collect::<Vec<_>>();
        assert_eq!(
            sorted,
            vec![
                Vec2D::new(0.0, 0.0),
                Vec2D::new(1.0, 0.0),
                Vec2D::new(2.0, 0.0),
                Vec2D::new(2.0, 1.0),
            ]
        );
    }

    #[test]
    fn checkerboard_mesh_is_global_across_all_contours() {
        let path = direct_grid_path();
        let mesh = Mesh::from_path(&path, SweepDirection::Vertical);
        assert_eq!(mesh.vertices.len(), 400);
        assert_eq!(mesh.edges.len(), 400);
        assert_eq!(mesh.sorted_vertices().count(), 400);

        let mut mesh = mesh;
        assert!(mesh.merge_coincident_vertices(SweepDirection::Vertical));
        let sorted = mesh.sorted_vertices().collect::<Vec<_>>();
        assert_eq!(sorted.len(), 200);
        assert!(sorted.windows(2).all(|vertices| {
            mesh.vertices[vertices[0]].point != mesh.vertices[vertices[1]].point
        }));
        assert_eq!(mesh.simplify(SweepDirection::Vertical), Ok(true));
        assert_eq!(mesh.sorted_vertices().count(), 2_601);
        let poly_head = mesh.tessellate().unwrap();
        let triangles = mesh.emit_triangles(
            poly_head,
            FillRule::NonZero,
            1,
            false,
            false,
            WindingFaces::All,
        );
        assert_eq!(triangles.len(), 7_500);
        let (mut negative, mut positive) = (0, 0);
        for triangle in &triangles {
            match triangle.weight_path_id >> 16 {
                -2 => negative += 1,
                2 => positive += 1,
                weight => panic!("unexpected checkerboard weight {weight}"),
            }
        }
        assert_eq!((negative, positive), (3_750, 3_750));
    }

    #[test]
    fn active_edge_list_has_stable_insert_remove_links() {
        let mut path = RawPath::new();
        path.move_to(0.0, 0.0);
        path.line_to(10.0, 0.0);
        path.line_to(10.0, 10.0);
        path.line_to(0.0, 10.0);
        path.close();
        let mut mesh = Mesh::from_path(&path, SweepDirection::Vertical);
        let mut active = ActiveEdgeList::default();
        assert!(active.insert_after(&mut mesh, 0, None));
        assert!(active.insert_after(&mut mesh, 1, Some(0)));
        assert!(active.insert_after(&mut mesh, 2, Some(0)));
        assert_eq!(active.iter(&mesh).collect::<Vec<_>>(), vec![0, 2, 1]);
        assert!(!active.insert_after(&mut mesh, 2, None));
        assert!(active.remove(&mut mesh, 2));
        assert_eq!(active.iter(&mesh).collect::<Vec<_>>(), vec![0, 1]);
        assert!(!active.remove(&mut mesh, 2));
        assert!(active.remove(&mut mesh, 0));
        assert!(active.remove(&mut mesh, 1));
        assert_eq!(active.head, None);
        assert_eq!(active.tail, None);
    }

    #[test]
    fn splitting_an_edge_preserves_winding_and_collects_grout() {
        let mut path = RawPath::new();
        path.move_to(0.0, 0.0);
        path.line_to(10.0, 10.0);
        path.line_to(0.0, 10.0);
        path.close();
        let mut mesh = Mesh::from_path(&path, SweepDirection::Vertical);
        let split = mesh.vertices.len();
        mesh.vertices.push(Vertex::new(Vec2D::new(5.0, 5.0), 255));
        let original = mesh.edges[0];
        let new_edge = mesh.split_edge(0, split, SweepDirection::Vertical).unwrap();
        assert_eq!(mesh.edges[0].top, original.top);
        assert_eq!(mesh.edges[0].bottom, split);
        assert_eq!(mesh.edges[new_edge].top, split);
        assert_eq!(mesh.edges[new_edge].bottom, original.bottom);
        assert_eq!(mesh.edges[new_edge].winding, original.winding);
        assert_eq!(mesh.breadcrumbs.len(), 1);
        assert_eq!(mesh.breadcrumbs[0][2], Vec2D::new(5.0, 5.0));
    }

    #[test]
    fn bowtie_intersection_is_inserted_once_and_splits_both_edges() {
        let mut path = RawPath::new();
        path.move_to(0.0, 0.0);
        path.line_to(10.0, 10.0);
        path.line_to(0.0, 10.0);
        path.line_to(10.0, 0.0);
        path.close();
        let mut mesh = Mesh::from_path(&path, SweepDirection::Vertical);
        let current = mesh
            .sorted_vertices()
            .find(|&vertex| mesh.vertices[vertex].point == Vec2D::new(0.0, 10.0))
            .unwrap();
        let intersection = mesh
            .check_and_split_intersection(0, 2, current, SweepDirection::Vertical)
            .unwrap();
        assert_eq!(mesh.vertices[intersection].point, Vec2D::new(5.0, 5.0));
        assert_eq!(mesh.vertices.len(), 5);
        assert_eq!(mesh.edges.len(), 6);
        assert_eq!(mesh.breadcrumbs.len(), 2);
        assert_eq!(
            mesh.sorted_vertices()
                .filter(|&vertex| mesh.vertices[vertex].point == Vec2D::new(5.0, 5.0))
                .count(),
            1
        );
    }

    #[test]
    fn sweep_simplifies_bowtie_into_a_global_planar_mesh() {
        let mut path = RawPath::new();
        path.move_to(0.0, 0.0);
        path.line_to(10.0, 10.0);
        path.line_to(0.0, 10.0);
        path.line_to(10.0, 0.0);
        path.close();
        let mut mesh = Mesh::from_path(&path, SweepDirection::Vertical);
        assert_eq!(mesh.simplify(SweepDirection::Vertical), Ok(true));
        assert_eq!(mesh.sorted_vertices().count(), 5);
        assert_eq!(
            mesh.sorted_vertices()
                .filter(|&vertex| mesh.vertices[vertex].point == Vec2D::new(5.0, 5.0))
                .count(),
            1
        );
        assert_eq!(mesh.breadcrumbs.len(), 2);
    }

    #[test]
    fn configured_cpp_grid_triangles_match_record_for_record() {
        let Ok(path) = std::env::var("RIVE_CPP_DIRECT_GRID_INPUTS") else {
            return;
        };
        let capture =
            crate::direct_grid_oracle::DirectGridInputs::parse(&std::fs::read(path).unwrap())
                .unwrap();
        let mut mesh = Mesh::from_path(&direct_grid_path(), SweepDirection::Vertical);
        mesh.simplify(SweepDirection::Vertical).unwrap();
        let poly_head = mesh.tessellate().unwrap();
        let rust = mesh.emit_triangles(
            poly_head,
            FillRule::NonZero,
            1,
            false,
            false,
            WindingFaces::All,
        );
        let rust = rust
            .iter()
            .map(|vertex| crate::direct_grid_oracle::TriangleRecord {
                x_bits: vertex.point[0].to_bits(),
                y_bits: vertex.point[1].to_bits(),
                weight_path_id: vertex.weight_path_id as u32,
            })
            .collect::<Vec<_>>();
        assert_eq!(rust, capture.triangles);
    }

    #[test]
    fn configured_cpp_flower_triangles_match_record_for_record() {
        let Ok(path) = std::env::var("RIVE_CPP_DIRECT_FLOWER_INPUTS") else {
            return;
        };
        let capture = crate::direct_grid_oracle::DirectGridInputs::parse_flower(
            &std::fs::read(path).unwrap(),
        )
        .unwrap();
        let tessellation = crate::draw::build_interior_tessellation(
            &direct_flower_path(),
            Mat2D([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]),
            FillRule::Clockwise,
            false,
        )
        .unwrap();
        let rust_contours = tessellation
            .contours
            .iter()
            .map(|contour| crate::direct_grid_oracle::ContourRecord {
                midpoint_x_bits: contour.midpoint[0].to_bits(),
                midpoint_y_bits: contour.midpoint[1].to_bits(),
                path_id: contour.path_id,
                vertex_index0: contour.vertex_index0,
            })
            .collect::<Vec<_>>();
        assert_eq!(rust_contours, capture.contours);
        let factory = crate::WgpuFactory::new(1000, 1000).unwrap();
        let height = crate::draw::tessellation_texture_height(&tessellation.spans);
        assert_eq!([capture.tess_width, capture.tess_height], [2048, height]);
        let uniforms = crate::analytic_uniforms(1000, 1000, height);
        let paths = [crate::gpu::PathData::zeroed(), tessellation.path];
        let mut encoder = factory.context.device.create_counted_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("nuxie-direct-flower-tessellation-encoder"),
            },
        );
        let mut tessellation_uploads = factory
            .context
            .tessellator
            .begin_frame_uploads(&factory.context.device);
        let texture = factory.context.tessellator.encode(
            &factory.context.device,
            &mut tessellation_uploads,
            &mut encoder,
            &factory.context.feather_lut.view,
            &tessellation.spans,
            &uniforms,
            &paths,
            &tessellation.contours,
            height,
        );
        let bytes_per_row = capture.tess_width * 16;
        let readback = factory
            .context
            .device
            .create_buffer(&wgpu::BufferDescriptor {
                label: Some("nuxie-direct-flower-tessellation-readback"),
                size: u64::from(bytes_per_row) * u64::from(height),
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            });
        encoder.copy_texture_to_buffer(
            texture.as_image_copy(),
            wgpu::TexelCopyBufferInfo {
                buffer: &readback,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(bytes_per_row),
                    rows_per_image: Some(height),
                },
            },
            texture.size(),
        );
        tessellation_uploads.flush(&factory.context.queue);
        factory.context.queue.submit_counted(Some(encoder.finish()));
        let slice = readback.slice(..);
        let (sender, receiver) = std::sync::mpsc::channel();
        slice.map_async(wgpu::MapMode::Read, move |result| {
            sender.send(result).unwrap();
        });
        factory
            .context
            .device
            .poll(wgpu::PollType::wait_indefinitely())
            .unwrap();
        receiver.recv().unwrap().unwrap();
        let mapped = slice.get_mapped_range().unwrap();
        let rust_texels = mapped
            .chunks_exact(16)
            .map(|texel| {
                [
                    u32::from_le_bytes(texel[0..4].try_into().unwrap()),
                    u32::from_le_bytes(texel[4..8].try_into().unwrap()),
                    u32::from_le_bytes(texel[8..12].try_into().unwrap()),
                    u32::from_le_bytes(texel[12..16].try_into().unwrap()),
                ]
            })
            .collect::<Vec<_>>();
        assert_eq!(rust_texels, capture.texels);
        drop(mapped);
        readback.unmap();
        let rust = tessellation
            .triangles
            .iter()
            .map(|vertex| crate::direct_grid_oracle::TriangleRecord {
                x_bits: vertex.point[0].to_bits(),
                y_bits: vertex.point[1].to_bits(),
                weight_path_id: vertex.weight_path_id as u32,
            })
            .collect::<Vec<_>>();
        assert_eq!(rust, capture.triangles);
    }
}
