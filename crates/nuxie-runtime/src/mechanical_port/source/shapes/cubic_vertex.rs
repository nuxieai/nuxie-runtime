use std::rc::Rc;

use crate::mechanical_port::source::{
    generated::shapes::cubic_vertex_base::CubicVertexBase,
    math::{mat2d::Mat2D, vec2d::Vec2D},
    shapes::{vertex::VertexBehavior, vertex_cells::VertexCells},
};

// The control points and their validity live in the vertex cells.
#[derive(Default)]
pub struct CubicVertex {
    pub base: CubicVertexBase,
}

impl CubicVertex {
    pub(crate) fn cells(&self) -> &Rc<VertexCells> {
        self.base.vertex_cells()
    }
}

impl VertexBehavior for CubicVertex {
    fn vertex(&self) -> &crate::mechanical_port::source::shapes::vertex::Vertex {
        &self.base.base.base
    }

    fn vertex_mut(&mut self) -> &mut crate::mechanical_port::source::shapes::vertex::Vertex {
        &mut self.base.base.base
    }

    fn mark_geometry_dirty(&mut self) {
        self.base.mark_geometry_dirty();
    }
}

pub trait CubicVertexBehavior: VertexBehavior {
    fn cubic_vertex(&self) -> &CubicVertex;
    fn cubic_vertex_mut(&mut self) -> &mut CubicVertex;
    fn compute_in(&mut self);
    fn compute_out(&mut self);
    fn in_point(&mut self) -> Vec2D {
        if !self.cubic_vertex().cells().in_valid() {
            self.compute_in();
            self.cubic_vertex().cells().set_in_valid(true);
        }
        self.cubic_vertex().cells().cached_in_point()
    }
    fn out_point(&mut self) -> Vec2D {
        if !self.cubic_vertex().cells().out_valid() {
            self.compute_out();
            self.cubic_vertex().cells().set_out_valid(true);
        }
        self.cubic_vertex().cells().cached_out_point()
    }
    fn set_in_point(&mut self, value: Vec2D) {
        self.cubic_vertex().cells().store_in_point(value);
        self.cubic_vertex().cells().set_in_valid(true);
    }
    fn set_out_point(&mut self, value: Vec2D) {
        self.cubic_vertex().cells().store_out_point(value);
        self.cubic_vertex().cells().set_out_valid(true);
    }
    fn render_in(&mut self) -> Vec2D {
        if let Some(weight) = self.cubic_vertex().cells().weight() {
            return weight.in_translation();
        }
        self.in_point()
    }
    fn render_out(&mut self) -> Vec2D {
        if let Some(weight) = self.cubic_vertex().cells().weight() {
            return weight.out_translation();
        }
        self.out_point()
    }
    fn x_changed(&mut self) {
        self.mark_geometry_dirty();
        self.cubic_vertex().cells().set_in_valid(false);
        self.cubic_vertex().cells().set_out_valid(false);
    }
    fn y_changed(&mut self) {
        self.mark_geometry_dirty();
        self.cubic_vertex().cells().set_in_valid(false);
        self.cubic_vertex().cells().set_out_valid(false);
    }
    fn deform(&mut self, world: &Mat2D, bones: &[f32]) {
        // Qualified Super::deform calls the concrete Vertex base, bypassing
        // any most-derived override of the public deformation operation.
        VertexBehavior::deform(self.vertex_mut(), world, bones);
        // A skinned cubic vertex may have no weight; hold its bind position,
        // just as Vertex::deform does.
        if !self.has_weight() {
            return;
        }
        let weight = self
            .cubic_vertex()
            .cells()
            .weight()
            .expect("a skin-deformed cubic vertex has a CubicWeight");
        // Retain the source local before either virtual point computation;
        // those callbacks can change this vertex's attached weight.
        let in_point = self.in_point();
        weight.deform_in(in_point, world, bones);
        // Source publishes the in translation before calling outPoint(), so
        // computeOut can observe that publication.
        let out_point = self.out_point();
        weight.deform_out(out_point, world, bones);
    }
}

impl CubicVertexBehavior for CubicVertex {
    fn cubic_vertex(&self) -> &CubicVertex {
        self
    }

    fn cubic_vertex_mut(&mut self) -> &mut CubicVertex {
        self
    }

    fn compute_in(&mut self) {
        let point = Vec2D::new(self.base.x(), self.base.y());
        self.cells().store_in_point(point);
    }

    fn compute_out(&mut self) {
        let point = Vec2D::new(self.base.x(), self.base.y());
        self.cells().store_out_point(point);
    }
}

impl std::ops::Deref for CubicVertex {
    type Target = CubicVertexBase;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for CubicVertex {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mechanical_port::source::{
        bones::cubic_weight::CubicWeight,
        core::{CoreArena, CoreHandle},
        shapes::{vertex::Vertex, vertex_cells::WeightCells},
    };

    struct CallbackVertex {
        base: CubicVertex,
        original_weight: CoreHandle,
        replacement_weight: Option<(CoreHandle, Rc<WeightCells>)>,
        observed_in: Option<Vec2D>,
        geometry_changes: usize,
    }

    impl VertexBehavior for CallbackVertex {
        fn vertex(&self) -> &Vertex {
            self.base.vertex()
        }
        fn vertex_mut(&mut self) -> &mut Vertex {
            self.base.vertex_mut()
        }
        fn mark_geometry_dirty(&mut self) {
            assert!(self.base.cells().in_valid() && self.base.cells().out_valid());
            self.geometry_changes += 1;
        }
        fn x_changed(&mut self) {
            panic!("qualified Vertex::xChanged bypasses this override");
        }
        fn y_changed(&mut self) {
            panic!("qualified Vertex::yChanged bypasses this override");
        }
        fn deform(&mut self, _: &Mat2D, _: &[f32]) {
            panic!("qualified Vertex::deform bypasses this override");
        }
    }
    impl CubicVertexBehavior for CallbackVertex {
        fn cubic_vertex(&self) -> &CubicVertex {
            &self.base
        }
        fn cubic_vertex_mut(&mut self) -> &mut CubicVertex {
            &mut self.base
        }
        fn compute_in(&mut self) {
            self.base.cells().store_in_point(Vec2D::new(2.0, 3.0));
            if let Some((replacement, cells)) = self.replacement_weight.take() {
                self.set_weight(replacement, cells);
            }
        }
        fn compute_out(&mut self) {
            self.observed_in = self
                .original_weight
                .with_downcast_mut::<CubicWeight, _>(|weight| weight.in_translation());
            self.base.cells().store_out_point(Vec2D::new(5.0, 7.0));
        }
    }

    fn weight(arena: &CoreArena) -> CoreHandle {
        let mut weight = CubicWeight::default();
        weight.base.base.set_values(255);
        weight.base.base.set_indices(0);
        weight.set_in_indices(0);
        weight.set_out_indices(0);
        weight.set_in_values(255);
        weight.set_out_values(255);
        arena.insert(weight)
    }

    fn weight_cells(weight: &CoreHandle) -> Rc<WeightCells> {
        weight
            .with_downcast::<CubicWeight, _>(|weight| Rc::clone(weight.base.base.cells()))
            .expect("a CubicWeight")
    }

    #[test]
    fn deform_keeps_weight_selected_before_virtual_point_callbacks() {
        let arena = CoreArena::default();
        let original = weight(&arena);
        let replacement = weight(&arena);
        let mut vertex = CallbackVertex {
            base: CubicVertex::default(),
            original_weight: original.clone(),
            replacement_weight: Some((replacement.clone(), weight_cells(&replacement))),
            observed_in: None,
            geometry_changes: 0,
        };
        vertex.set_weight(original.clone(), weight_cells(&original));
        CubicVertexBehavior::deform(
            &mut vertex,
            &Mat2D::default(),
            &[1.0, 0.0, 0.0, 1.0, 10.0, 20.0],
        );
        assert_eq!(
            original.with_downcast_mut::<CubicWeight, _>(|weight| weight.in_translation()),
            Some(Vec2D::new(12.0, 23.0))
        );
        assert_eq!(
            original.with_downcast_mut::<CubicWeight, _>(|weight| weight.out_translation()),
            Some(Vec2D::new(15.0, 27.0))
        );
        assert_eq!(
            replacement.with_downcast_mut::<CubicWeight, _>(|weight| weight.in_translation()),
            Some(Vec2D::new(0.0, 0.0))
        );
    }

    #[test]
    fn deform_stores_in_translation_before_virtual_out_point() {
        let arena = CoreArena::default();
        let original = weight(&arena);
        let mut vertex = CallbackVertex {
            base: CubicVertex::default(),
            original_weight: original.clone(),
            replacement_weight: None,
            observed_in: None,
            geometry_changes: 0,
        };
        let cells = weight_cells(&original);
        vertex.set_weight(original, cells);
        CubicVertexBehavior::deform(
            &mut vertex,
            &Mat2D::default(),
            &[1.0, 0.0, 0.0, 1.0, 10.0, 20.0],
        );
        assert_eq!(vertex.observed_in, Some(Vec2D::new(12.0, 23.0)));
    }
    #[test]
    fn coordinate_changes_call_the_base_and_dispatch_geometry_before_invalidating() {
        let arena = CoreArena::default();
        let original = weight(&arena);
        let mut vertex = CallbackVertex {
            base: CubicVertex::default(),
            original_weight: original,
            replacement_weight: None,
            observed_in: None,
            geometry_changes: 0,
        };
        vertex.base.cells().set_in_valid(true);
        vertex.base.cells().set_out_valid(true);
        CubicVertexBehavior::x_changed(&mut vertex);
        assert_eq!(vertex.geometry_changes, 1);
        assert!(!vertex.base.cells().in_valid() && !vertex.base.cells().out_valid());
        vertex.base.cells().set_in_valid(true);
        vertex.base.cells().set_out_valid(true);
        CubicVertexBehavior::y_changed(&mut vertex);
        assert_eq!(vertex.geometry_changes, 2);
        assert!(!vertex.base.cells().in_valid() && !vertex.base.cells().out_valid());
    }
}
