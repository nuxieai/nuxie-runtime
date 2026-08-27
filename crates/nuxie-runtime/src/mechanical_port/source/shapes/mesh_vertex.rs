use crate::mechanical_port::source::{
    core_context::{CoreContext, StatusCode},
    generated::shapes::mesh_vertex_base::MeshVertexBase,
    shapes::mesh::Mesh,
};
pub struct MeshVertex {
    pub base: MeshVertexBase,
}
impl MeshVertex {
    pub fn mark_geometry_dirty(&mut self) {
        self.base
            .parent_mut()
            .as_mut::<Mesh>()
            .unwrap()
            .mark_drawable_dirty();
    }
    pub fn on_added_dirty(&mut self, context: &mut CoreContext) -> StatusCode {
        let code = self.base.on_added_dirty(context);
        if code != StatusCode::Ok {
            return code;
        }
        if !self.base.parent().is::<Mesh>() {
            return StatusCode::MissingObject;
        }
        let this = self as *mut _;
        self.base
            .parent_mut()
            .as_mut::<Mesh>()
            .unwrap()
            .add_vertex(unsafe { &mut *this });
        StatusCode::Ok
    }
}
