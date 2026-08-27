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

pub struct NSlicerTileMode {
    pub base: NSlicerTileModeBase,
}
impl NSlicerTileMode {
    pub fn on_added_dirty(&mut self, context: &mut CoreContext) -> StatusCode {
        let code = self.base.on_added_dirty(context);
        if code != StatusCode::Ok {
            return code;
        }
        let Some(container) = n_slicer_details::from(self.base.parent_mut()) else {
            return StatusCode::MissingObject;
        };
        container.add_tile_mode(
            self.base.patch_index(),
            NSlicerTileModeType::from(self.base.style()),
        );
        StatusCode::Ok
    }
}
