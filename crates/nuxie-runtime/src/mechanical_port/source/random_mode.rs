#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum RandomMode {
    Once = 0,
    Always = 1,
    SourceChange = 2,
}
