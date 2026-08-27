use crate::mechanical_port::source::{component::ComponentHandle, core_context::CoreContext};

pub const POINTS_PATH_TYPE_KEY: u16 = 16;
pub const MESH_TYPE_KEY: u16 = 109;

#[derive(Default)]
pub struct SkinnableState {
    skin: Option<ComponentHandle>,
}

impl SkinnableState {
    pub fn skin(&self) -> Option<ComponentHandle> {
        self.skin
    }

    pub(crate) fn set_skin(&mut self, skin: ComponentHandle) {
        self.skin = Some(skin);
    }
}

pub trait Skinnable {
    fn skinnable_state(&self) -> &SkinnableState;
    fn skinnable_state_mut(&mut self) -> &mut SkinnableState;
    fn mark_skin_dirty(&mut self);

    fn skin(&self) -> Option<ComponentHandle> {
        self.skinnable_state().skin()
    }

    fn set_skin(&mut self, skin: ComponentHandle) {
        self.skinnable_state_mut().set_skin(skin);
    }
}

pub fn from(component: ComponentHandle, context: &CoreContext) -> Option<ComponentHandle> {
    match context.core_type(component) {
        POINTS_PATH_TYPE_KEY | MESH_TYPE_KEY => Some(component),
        _ => None,
    }
}
