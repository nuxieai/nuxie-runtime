/// Upstream input listener phase. Unknown serialized values act as normal.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u32)]
pub enum ListenerPhase {
    #[default]
    Normal = 0,
    Capture = 1,
}
