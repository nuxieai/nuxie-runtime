use crate::mechanical_port::source::{core::CoreHandle, core_context::CoreContext};

pub const POINTS_PATH_TYPE_KEY: u16 = 16;
pub const MESH_TYPE_KEY: u16 = 109;

#[derive(Default)]
pub struct Skinnable {
    skin: Option<CoreHandle>,
}

impl Skinnable {
    pub fn skin(&self) -> Option<CoreHandle> {
        self.skin.clone()
    }

    pub(crate) fn set_skin(&mut self, skin: CoreHandle) {
        self.skin = Some(skin);
    }
}

pub trait SkinnableBehavior {
    fn skinnable(&self) -> &Skinnable;
    fn skinnable_mut(&mut self) -> &mut Skinnable;
    fn mark_skin_dirty(&mut self);

    fn skin(&self) -> Option<CoreHandle> {
        self.skinnable().skin()
    }

    fn set_skin(&mut self, skin: CoreHandle) {
        self.skinnable_mut().set_skin(skin);
    }
}

pub fn from(component: CoreHandle, _context: &dyn CoreContext) -> Option<CoreHandle> {
    match component.core_type() {
        POINTS_PATH_TYPE_KEY | MESH_TYPE_KEY => Some(component),
        _ => None,
    }
}
