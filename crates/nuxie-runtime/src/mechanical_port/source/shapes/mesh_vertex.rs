use crate::mechanical_port::source::{
    core_context::{CoreContext, StatusCode},
    generated::shapes::mesh_vertex_base::MeshVertexBase,
    shapes::mesh::Mesh,
    shapes::vertex::{Vertex, VertexBehavior},
};
impl std::ops::Deref for MeshVertex {
    type Target = MeshVertexBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for MeshVertex {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl MeshVertex {
    pub const TYPE_KEY: u16 = MeshVertexBase::TYPE_KEY;
}

#[derive(Default)]
pub struct MeshVertex {
    pub base: MeshVertexBase,
}
impl VertexBehavior for MeshVertex {
    fn vertex(&self) -> &Vertex {
        &self.base.base
    }
    fn vertex_mut(&mut self) -> &mut Vertex {
        &mut self.base.base
    }
    fn mark_geometry_dirty(&mut self) {
        MeshVertex::mark_geometry_dirty(self);
    }
}
impl MeshVertex {
    pub fn mark_geometry_dirty(&mut self) {
        if let Some(parent) = self.base.parent_handle() {
            parent.with_downcast_mut::<Mesh, _>(Mesh::mark_drawable_dirty);
        }
    }
    pub fn on_added_dirty(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        let code = self.base.on_added_dirty(context);
        if code != StatusCode::Ok {
            return code;
        }
        let (Some(parent), Some(this)) = (self.base.parent_handle(), self.base.handle()) else {
            return StatusCode::MissingObject;
        };
        // Preserve the pinned C++ order: test the dynamic parent type before
        // casting/mutating it. Besides being source-exact, this avoids trying
        // to mutably reborrow a self-parented non-Mesh occurrence.
        if !parent.is_type_of(Mesh::TYPE_KEY) {
            return StatusCode::MissingObject;
        }
        if parent
            .with_downcast_mut::<Mesh, _>(|parent| parent.add_vertex(this))
            .is_none()
        {
            return StatusCode::MissingObject;
        }
        StatusCode::Ok
    }
}
