use crate::mechanical_port::source::{
    core_context::{CoreContext, StatusCode},
    generated::layout::n_slicer_tile_mode_base::NSlicerTileModeBase,
    layout::n_slicer_details,
};

#[repr(transparent)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct NSlicerTileModeType(i32);
#[allow(non_upper_case_globals)]
impl NSlicerTileModeType {
    pub const Stretch: Self = Self(0);
    pub const Repeat: Self = Self(1);
    pub const Hidden: Self = Self(2);
}
impl From<u32> for NSlicerTileModeType {
    fn from(value: u32) -> Self {
        Self(value as i32)
    }
}

impl std::ops::Deref for NSlicerTileMode {
    type Target = NSlicerTileModeBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for NSlicerTileMode {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl NSlicerTileMode {
    pub const TYPE_KEY: u16 = NSlicerTileModeBase::TYPE_KEY;
}

#[derive(Default)]
pub struct NSlicerTileMode {
    pub base: NSlicerTileModeBase,
}
impl NSlicerTileMode {
    pub fn on_added_dirty(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        let code = self.base.on_added_dirty(context);
        if code != StatusCode::Ok {
            return code;
        }
        let Some(parent) = self.base.parent_handle() else {
            return StatusCode::MissingObject;
        };
        if !n_slicer_details::add_tile_mode(
            &parent,
            self.base.patch_index() as i32,
            NSlicerTileModeType::from(self.base.style()),
        ) {
            return StatusCode::MissingObject;
        }
        StatusCode::Ok
    }
}
