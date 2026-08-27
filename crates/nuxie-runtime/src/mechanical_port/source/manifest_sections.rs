#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ManifestSections {
    Names = 0,
    Paths = 1,
}
