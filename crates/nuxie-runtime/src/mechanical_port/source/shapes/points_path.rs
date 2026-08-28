use crate::mechanical_port::source::{
    bones::{
        skin::Skin,
        skinnable::{Skinnable, SkinnableBehavior},
    },
    component::{ComponentDirt, has_dirt},
    generated::shapes::points_path_base::PointsPathBase,
    math::mat2d::Mat2D,
};
static IDENTITY: Mat2D = Mat2D::IDENTITY;
pub struct PointsPath {
    pub base: PointsPathBase,
    skinnable: Skinnable,
}

impl Default for PointsPath {
    fn default() -> Self {
        Self {
            base: PointsPathBase::default(),
            skinnable: Skinnable::default(),
        }
    }
}

impl SkinnableBehavior for PointsPath {
    fn skinnable(&self) -> &Skinnable {
        &self.skinnable
    }

    fn skinnable_mut(&mut self) -> &mut Skinnable {
        &mut self.skinnable
    }

    fn mark_skin_dirty(&mut self) {
        self.mark_path_dirty(true);
    }
}
impl PointsPath {
    pub fn build_dependencies(&mut self) {
        self.base.build_dependencies();
        if let (Some(this), Some(skin)) = (self.base.handle(), self.skin()) {
            skin.with_mut(|skin| {
                if let Some(skin) = skin.as_component_mut() {
                    skin.add_dependent(this);
                }
            });
        }
    }
    pub fn path_transform(&self) -> &Mat2D {
        if self.skin().is_some() {
            &IDENTITY
        } else {
            self.base.world_transform()
        }
    }
    pub fn update(&mut self, value: ComponentDirt) {
        if has_dirt(value, ComponentDirt::PATH) {
            if let Some(skin) = self.skin() {
                let vertices = self
                    .base
                    .vertices()
                    .iter()
                    .filter_map(|vertex| vertex.authored_handle())
                    .collect::<Vec<_>>();
                skin.with_downcast::<Skin, _>(|skin| skin.deform(&vertices));
            }
        }
        self.base.update(value);
    }
    pub fn mark_path_dirty(&mut self, _send_to_layout: bool) {
        if let Some(skin) = self.skin() {
            skin.with_mut(|skin| {
                if let Some(skin) = skin.as_component_mut() {
                    skin.add_dirt(ComponentDirt::SKIN, true);
                }
            });
        }
        self.base.super_mark_path_dirty();
    }
    pub fn mark_skin_dirty(&mut self) {
        self.mark_path_dirty(true);
    }
}
