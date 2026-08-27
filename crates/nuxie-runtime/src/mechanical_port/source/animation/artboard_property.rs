#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArtboardProperty {
    Width = 0,
    Height = 1,
    Ratio = 2,
}
