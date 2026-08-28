use crate::mechanical_port::source::{
    core_context::{CoreContext, StatusCode},
    generated::layout::n_slicer_tile_mode_base::NSlicerTileModeBase,
    layout::n_slicer_details,
};

#[repr(i32)]
#[derive(Clone, Copy)]
pub enum NSlicerTileModeType {
    Stretch = 0,
    Repeat = 1,
    Hidden = 2,
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
            self.base.patch_index(),
            NSlicerTileModeType::from(self.base.style()),
        ) {
            return StatusCode::MissingObject;
        }
        StatusCode::Ok
    }
}
