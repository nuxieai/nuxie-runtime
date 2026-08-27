#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ShapePaintType {
    Stroke = 0,
    Fill = 1,
}
