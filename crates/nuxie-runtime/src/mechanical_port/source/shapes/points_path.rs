use crate::mechanical_port::source::{
    component::{ComponentDirt, has_dirt},
    generated::shapes::points_path_base::PointsPathBase,
    math::mat2d::Mat2D,
};
static IDENTITY: Mat2D = Mat2D::IDENTITY;
pub struct PointsPath {
    pub base: PointsPathBase,
}
impl PointsPath {
    pub fn build_dependencies(&mut self) {
        self.base.build_dependencies();
        let this = self.base.as_component_mut_ptr();
        if let Some(skin) = self.base.skin_mut() {
            skin.add_dependent(this);
        }
    }
    pub fn path_transform(&self) -> &Mat2D {
        if self.base.skin().is_some() {
            &IDENTITY
        } else {
            self.base.world_transform()
        }
    }
    pub fn update(&mut self, value: ComponentDirt) {
        if has_dirt(value, ComponentDirt::PATH) {
            if let Some(skin) = self.base.skin_mut() {
                skin.deform(self.base.vertices_mut());
            }
        }
        self.base.update(value);
    }
    pub fn mark_path_dirty(&mut self, _send_to_layout: bool) {
        if let Some(skin) = self.base.skin_mut() {
            skin.add_dirt(ComponentDirt::SKIN);
        }
        self.base.super_mark_path_dirty();
    }
    pub fn mark_skin_dirty(&mut self) {
        self.mark_path_dirty(true);
    }
}
