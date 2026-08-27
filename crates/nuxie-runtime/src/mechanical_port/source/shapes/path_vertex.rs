use crate::mechanical_port::source::{
    core_context::{CoreContext, StatusCode},
    generated::shapes::path_vertex_base::PathVertexBase,
    shapes::path::Path,
};
pub struct PathVertex {
    pub base: PathVertexBase,
}
impl PathVertex {
    pub fn on_added_dirty(&mut self, context: &mut CoreContext) -> StatusCode {
        let code = self.base.on_added_dirty(context);
        if code != StatusCode::Ok {
            return code;
        }
        if !self.base.parent().is::<Path>() {
            return StatusCode::MissingObject;
        }
        let this = self as *mut _;
        self.base
            .parent_mut()
            .as_mut::<Path>()
            .unwrap()
            .add_vertex(unsafe { &mut *this });
        StatusCode::Ok
    }
    pub fn mark_geometry_dirty(&mut self) {
        if let Some(parent) = self.base.parent_mut() {
            parent.as_mut::<Path>().unwrap().mark_path_dirty(true);
        }
    }
}
