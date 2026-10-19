//! The state of a Vertex and its Weight that skin deformation and Path's
//! buildPath read every frame: the authored position and radius, a cubic
//! vertex's cached control points, and the weight's packed bone indices and
//! values with their deformed translations. Held outside the vertex and the
//! weight so a skinned path deforms and rebuilds without borrowing either.

use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use crate::mechanical_port::source::{
    bones::weight::Weight,
    core::CoreHandle,
    math::{mat2d::Mat2D, vec2d::Vec2D},
};

/// Which concrete vertex owns the cells. Only the authored straight and cubic
/// vertices, whose deform and control points are the base behavior, take the
/// cells' fast paths; any other vertex (including a type embedding one of
/// them with its own overrides) is read through its object.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum VertexKind {
    Other,
    Straight,
    Cubic,
}

impl VertexKind {
    /// The kind of the vertex occupying a slot, from its concrete type key.
    pub(crate) fn of(vertex: &CoreHandle) -> Self {
        use crate::mechanical_port::source::shapes::{
            cubic_asymmetric_vertex::CubicAsymmetricVertex,
            cubic_detached_vertex::CubicDetachedVertex, cubic_mirrored_vertex::CubicMirroredVertex,
            straight_vertex::StraightVertex,
        };
        match vertex.occupied_core_type() {
            Some(StraightVertex::TYPE_KEY) => Self::Straight,
            Some(
                CubicMirroredVertex::TYPE_KEY
                | CubicAsymmetricVertex::TYPE_KEY
                | CubicDetachedVertex::TYPE_KEY,
            ) => Self::Cubic,
            _ => Self::Other,
        }
    }
}

pub(crate) struct VertexCells {
    kind: Cell<VertexKind>,
    x: Cell<f32>,
    y: Cell<f32>,
    // StraightVertex's radius; zero for other vertices.
    radius: Cell<f32>,
    // A cubic vertex's control points, valid until a coordinate, rotation or
    // distance changes.
    in_point: Cell<Vec2D>,
    out_point: Cell<Vec2D>,
    in_valid: Cell<bool>,
    out_valid: Cell<bool>,
    weight: RefCell<Option<Rc<WeightCells>>>,
}

impl Default for VertexCells {
    fn default() -> Self {
        Self {
            kind: Cell::new(VertexKind::Other),
            x: Cell::new(0.0),
            y: Cell::new(0.0),
            radius: Cell::new(0.0),
            in_point: Cell::new(Vec2D::default()),
            out_point: Cell::new(Vec2D::default()),
            in_valid: Cell::new(false),
            out_valid: Cell::new(false),
            weight: RefCell::new(None),
        }
    }
}

impl VertexCells {
    #[inline]
    pub(crate) fn kind(&self) -> VertexKind {
        self.kind.get()
    }

    pub(crate) fn set_kind(&self, kind: VertexKind) {
        self.kind.set(kind);
    }

    #[inline]
    pub(crate) fn x(&self) -> f32 {
        self.x.get()
    }

    #[inline]
    pub(crate) fn set_x(&self, value: f32) {
        self.x.set(value);
    }

    #[inline]
    pub(crate) fn y(&self) -> f32 {
        self.y.get()
    }

    #[inline]
    pub(crate) fn set_y(&self, value: f32) {
        self.y.set(value);
    }

    #[inline]
    pub(crate) fn position(&self) -> Vec2D {
        Vec2D::new(self.x.get(), self.y.get())
    }

    #[inline]
    pub(crate) fn radius(&self) -> f32 {
        self.radius.get()
    }

    #[inline]
    pub(crate) fn set_radius(&self, value: f32) {
        self.radius.set(value);
    }

    #[inline]
    pub(crate) fn in_point(&self) -> Option<Vec2D> {
        self.in_valid.get().then(|| self.in_point.get())
    }

    #[inline]
    pub(crate) fn out_point(&self) -> Option<Vec2D> {
        self.out_valid.get().then(|| self.out_point.get())
    }

    #[inline]
    pub(crate) fn cached_in_point(&self) -> Vec2D {
        self.in_point.get()
    }

    #[inline]
    pub(crate) fn cached_out_point(&self) -> Vec2D {
        self.out_point.get()
    }

    /// Store a computed in point without validating it.
    #[inline]
    pub(crate) fn store_in_point(&self, value: Vec2D) {
        self.in_point.set(value);
    }

    #[inline]
    pub(crate) fn store_out_point(&self, value: Vec2D) {
        self.out_point.set(value);
    }

    #[inline]
    pub(crate) fn in_valid(&self) -> bool {
        self.in_valid.get()
    }

    #[inline]
    pub(crate) fn out_valid(&self) -> bool {
        self.out_valid.get()
    }

    #[inline]
    pub(crate) fn set_in_valid(&self, valid: bool) {
        self.in_valid.set(valid);
    }

    #[inline]
    pub(crate) fn set_out_valid(&self, valid: bool) {
        self.out_valid.set(valid);
    }

    pub(crate) fn weight(&self) -> Option<Rc<WeightCells>> {
        self.weight.borrow().clone()
    }

    #[inline]
    pub(crate) fn has_weight(&self) -> bool {
        self.weight.borrow().is_some()
    }

    pub(crate) fn set_weight(&self, weight: Rc<WeightCells>) {
        *self.weight.borrow_mut() = Some(weight);
    }

    /// Vertex::renderTranslation(): the deformed translation when weighted.
    #[inline]
    /// A weighted vertex's renderIn, renderOut and renderTranslation, read
    /// under one weight read; None for an unweighted vertex, whose points may
    /// need computing.
    #[inline]
    pub(crate) fn weighted_render_points(&self) -> Option<(Vec2D, Vec2D, Vec2D)> {
        self.weight.borrow().as_ref().map(|weight| {
            (
                weight.in_translation(),
                weight.out_translation(),
                weight.translation(),
            )
        })
    }

    pub(crate) fn render_translation(&self) -> Vec2D {
        match self.weight.borrow().as_ref() {
            Some(weight) => weight.translation(),
            None => self.position(),
        }
    }

    /// CubicVertex::inPoint() for the cubic vertex `owner` these cells
    /// belong to: the cached point, or the vertex's own computeIn.
    #[inline]
    pub(crate) fn in_point_of(&self, owner: &CoreHandle) -> Vec2D {
        if self.in_valid.get() {
            return self.in_point.get();
        }
        Self::compute(owner, true)
    }

    /// CubicVertex::outPoint() for the cubic vertex `owner`.
    #[inline]
    pub(crate) fn out_point_of(&self, owner: &CoreHandle) -> Vec2D {
        if self.out_valid.get() {
            return self.out_point.get();
        }
        Self::compute(owner, false)
    }

    #[cold]
    fn compute(owner: &CoreHandle, incoming: bool) -> Vec2D {
        owner
            .with_mut(|object| {
                let cubic = object
                    .as_cubic_vertex_behavior_mut()
                    .expect("cubic vertex cells belong to a cubic vertex");
                if incoming {
                    cubic.in_point()
                } else {
                    cubic.out_point()
                }
            })
            .expect("live path vertex")
    }

    /// CubicVertex::renderIn(): the deformed in point when weighted.
    #[inline]
    pub(crate) fn render_in_of(&self, owner: &CoreHandle) -> Vec2D {
        if let Some(weight) = self.weight.borrow().as_ref() {
            return weight.in_translation();
        }
        self.in_point_of(owner)
    }

    /// CubicVertex::renderOut(): the deformed out point when weighted.
    #[inline]
    pub(crate) fn render_out_of(&self, owner: &CoreHandle) -> Vec2D {
        if let Some(weight) = self.weight.borrow().as_ref() {
            return weight.out_translation();
        }
        self.out_point_of(owner)
    }

    /// Skin::deform for one straight or cubic path vertex: Vertex::deform,
    /// then for a cubic vertex its control points, in that order.
    pub(crate) fn deform(&self, owner: &CoreHandle, world: &Mat2D, bones: &[f32]) {
        let weight = {
            let held = self.weight.borrow();
            let Some(weight) = held.as_ref() else {
                return;
            };
            weight.deform(self.position(), world, bones);
            if self.kind.get() != VertexKind::Cubic {
                return;
            }
            if self.in_valid.get() && self.out_valid.get() {
                // Both control points are cached, so no callback runs while
                // the weight is borrowed.
                weight.deform_in(self.in_point.get(), world, bones);
                weight.deform_out(self.out_point.get(), world, bones);
                return;
            }
            // Hold the weight selected before the control point callbacks.
            Rc::clone(weight)
        };
        let in_point = self.in_point_of(owner);
        weight.deform_in(in_point, world, bones);
        let out_point = self.out_point_of(owner);
        weight.deform_out(out_point, world, bones);
    }
}

pub(crate) struct WeightCells {
    values: Cell<u32>,
    indices: Cell<u32>,
    // CubicWeight's control point weights; unused by a plain Weight.
    in_values: Cell<u32>,
    in_indices: Cell<u32>,
    out_values: Cell<u32>,
    out_indices: Cell<u32>,
    translation: Cell<Vec2D>,
    in_translation: Cell<Vec2D>,
    out_translation: Cell<Vec2D>,
}

impl Default for WeightCells {
    fn default() -> Self {
        Self {
            values: Cell::new(255),
            indices: Cell::new(1),
            in_values: Cell::new(255),
            in_indices: Cell::new(1),
            out_values: Cell::new(255),
            out_indices: Cell::new(1),
            translation: Cell::new(Vec2D::new(0.0, 0.0)),
            in_translation: Cell::new(Vec2D::new(0.0, 0.0)),
            out_translation: Cell::new(Vec2D::new(0.0, 0.0)),
        }
    }
}

macro_rules! weight_cells {
    ($($get:ident, $set:ident: $ty:ty;)*) => {
        impl WeightCells {
            $(
                #[inline]
                pub(crate) fn $get(&self) -> $ty {
                    self.$get.get()
                }

                #[inline]
                pub(crate) fn $set(&self, value: $ty) {
                    self.$get.set(value);
                }
            )*
        }
    };
}

weight_cells! {
    values, set_values: u32;
    indices, set_indices: u32;
    in_values, set_in_values: u32;
    in_indices, set_in_indices: u32;
    out_values, set_out_values: u32;
    out_indices, set_out_indices: u32;
    translation, set_translation: Vec2D;
    in_translation, set_in_translation: Vec2D;
    out_translation, set_out_translation: Vec2D;
}

impl WeightCells {
    /// Vertex::deform's weight update for `position`.
    #[inline]
    pub(crate) fn deform(&self, position: Vec2D, world: &Mat2D, bones: &[f32]) {
        self.translation.set(Weight::deform(
            position,
            self.indices.get(),
            self.values.get(),
            world,
            bones,
        ));
    }

    #[inline]
    pub(crate) fn deform_in(&self, point: Vec2D, world: &Mat2D, bones: &[f32]) {
        self.in_translation.set(Weight::deform(
            point,
            self.in_indices.get(),
            self.in_values.get(),
            world,
            bones,
        ));
    }

    #[inline]
    pub(crate) fn deform_out(&self, point: Vec2D, world: &Mat2D, bones: &[f32]) {
        self.out_translation.set(Weight::deform(
            point,
            self.out_indices.get(),
            self.out_values.get(),
            world,
            bones,
        ));
    }
}
