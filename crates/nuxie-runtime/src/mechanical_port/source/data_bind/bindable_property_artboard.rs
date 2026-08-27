use crate::mechanical_port::source::generated::data_bind::bindable_property_artboard_base::BindablePropertyArtboardBase;

#[derive(Default)]
pub struct BindablePropertyArtboard {
    pub base: BindablePropertyArtboardBase,
}
impl BindablePropertyArtboard {
    pub const DEFAULT_VALUE: u32 = u32::MAX;
}
