use std::rc::Rc;

use crate::mechanical_port::source::{
    core::CoreHandle,
    generated::shapes::vertex_base::{VertexBase, VertexBaseCallbacks},
    math::{mat2d::Mat2D, vec2d::Vec2D},
    shapes::vertex_cells::{VertexCells, WeightCells},
};

#[derive(Default)]
pub struct VertexState {
    weight: Option<CoreHandle>,
}

pub struct Vertex {
    pub base: VertexBase,
    state: VertexState,
}

impl Default for Vertex {
    fn default() -> Self {
        Self {
            base: VertexBase::default(),
            state: VertexState::default(),
        }
    }
}

impl VertexBaseCallbacks for Vertex {
    fn notify_property_changed(&mut self, property_key: u16) {
        self.notify_property_changed(property_key);
    }

    fn x_changed(&mut self) {
        VertexBehavior::x_changed(self);
    }

    fn y_changed(&mut self) {
        VertexBehavior::y_changed(self);
    }
}

impl crate::mechanical_port::source::generated::component_base::ComponentBaseCallbacks for Vertex {
    fn notify_property_changed(&mut self, property_key: u16) {
        Vertex::notify_property_changed(self, property_key);
    }
}

impl Vertex {
    /// The cells skin deformation and the owning path read this vertex through.
    pub(crate) fn cells(&self) -> &Rc<VertexCells> {
        self.base.cells()
    }

    pub fn notify_property_changed(&mut self, property_key: u16) {
        self.base
            .base
            .base
            .base
            .notify_property_changed(property_key);
    }
}

pub trait VertexBehavior {
    fn vertex(&self) -> &Vertex;
    fn vertex_mut(&mut self) -> &mut Vertex;
    fn mark_geometry_dirty(&mut self);
    fn set_weight(&mut self, weight: CoreHandle, cells: Rc<WeightCells>) {
        self.vertex_mut().state.weight = Some(weight);
        self.vertex().cells().set_weight(cells);
    }
    fn has_weight(&self) -> bool {
        self.vertex().state.weight.is_some()
    }
    fn weight_handle(&self) -> Option<CoreHandle> {
        self.vertex().state.weight.clone()
    }
    fn render_translation(&self) -> Vec2D {
        self.vertex().cells().render_translation()
    }
    fn x_changed(&mut self) {
        self.mark_geometry_dirty();
    }
    fn y_changed(&mut self) {
        self.mark_geometry_dirty();
    }
    fn deform(&mut self, world: &Mat2D, bone_transforms: &[f32]) {
        // Imported skinned vertices can lack a Weight too. Preserve their
        // bind-space position instead of attempting to deform a missing weight.
        let cells = self.vertex().cells();
        let Some(weight) = cells.weight() else {
            return;
        };
        weight.deform(cells.position(), world, bone_transforms);
    }
}

impl VertexBehavior for Vertex {
    fn vertex(&self) -> &Vertex {
        self
    }

    fn vertex_mut(&mut self) -> &mut Vertex {
        self
    }

    fn mark_geometry_dirty(&mut self) {}
}

impl std::ops::Deref for Vertex {
    type Target = VertexBase;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for Vertex {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
