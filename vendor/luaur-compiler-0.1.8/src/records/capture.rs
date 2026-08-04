use luaur_common::enums::luau_capture_type::LuauCaptureType;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Capture {
    pub(crate) r#type: LuauCaptureType,
    pub(crate) data: u8,
}
