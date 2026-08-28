use crate::mechanical_port::source::{
    core_context::{CoreContext, StatusCode},
    generated::shapes::path_vertex_base::PathVertexBase,
    shapes::path::Path,
    shapes::vertex::{Vertex, VertexBehavior},
};
#[derive(Default)]
pub struct PathVertex {
    pub base: PathVertexBase,
}

impl VertexBehavior for PathVertex {
    fn vertex(&self) -> &Vertex {
        &self.base.base
    }

    fn vertex_mut(&mut self) -> &mut Vertex {
        &mut self.base.base
    }

    fn mark_geometry_dirty(&mut self) {
        PathVertex::mark_geometry_dirty(self);
    }
}

impl PathVertex {
    pub fn on_added_dirty(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        let code = self.base.on_added_dirty(context);
        if code != StatusCode::Ok {
            return code;
        }
        let (Some(parent), Some(this)) = (self.base.parent_handle(), self.base.handle()) else {
            return StatusCode::MissingObject;
        };
        let added = parent
            .with_downcast_mut::<Path, _>(|parent| parent.add_vertex(this))
            .is_some();
        if !added {
            return StatusCode::MissingObject;
        }
        StatusCode::Ok
    }
    pub fn mark_geometry_dirty(&mut self) {
        if let Some(parent) = self.base.parent_handle() {
            parent.with_downcast_mut::<Path, _>(|parent| parent.mark_path_dirty(true));
        }
    }
}
