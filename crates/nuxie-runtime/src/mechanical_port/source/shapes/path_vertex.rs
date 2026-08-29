use crate::mechanical_port::source::{
    core_context::{CoreContext, StatusCode},
    generated::shapes::path_vertex_base::PathVertexBase,
    shapes::path::Path,
    shapes::vertex::{Vertex, VertexBehavior},
};
impl std::ops::Deref for PathVertex {
    type Target = PathVertexBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for PathVertex {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl PathVertex {
    pub const TYPE_KEY: u16 = PathVertexBase::TYPE_KEY;
}

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
            .with_mut(|parent| parent.as_path_mut().map(|parent| parent.add_vertex(this)))
            .flatten()
            .is_some();
        if !added {
            return StatusCode::MissingObject;
        }
        StatusCode::Ok
    }
    pub fn mark_geometry_dirty(&mut self) {
        if let Some(parent) = self.base.parent_handle() {
            parent.with_mut(|parent| Path::mark_path_dirty_for(parent, true));
        }
    }
}
