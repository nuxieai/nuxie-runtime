#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum SymbolType {
    #[default]
    None = 0,
    VertexX = 1,
    VertexY = 2,
    CubicVertexInPointX = 3,
    CubicVertexInPointY = 4,
    CubicVertexOutPointX = 5,
    CubicVertexOutPointY = 6,
    Rotation = 7,
    InRotation = 8,
    OutRotation = 9,
    Distance = 10,
    InDistance = 11,
    OutDistance = 12,
    TextStyle = 13,
    TextContent = 14,
    ItemIndex = 15,
    DrawIndex = 16,
}

impl SymbolType {
    pub fn from_i32(value: i32) -> Option<Self> {
        Some(match value {
            0 => Self::None,
            1 => Self::VertexX,
            2 => Self::VertexY,
            3 => Self::CubicVertexInPointX,
            4 => Self::CubicVertexInPointY,
            5 => Self::CubicVertexOutPointX,
            6 => Self::CubicVertexOutPointY,
            7 => Self::Rotation,
            8 => Self::InRotation,
            9 => Self::OutRotation,
            10 => Self::Distance,
            11 => Self::InDistance,
            12 => Self::OutDistance,
            13 => Self::TextStyle,
            14 => Self::TextContent,
            15 => Self::ItemIndex,
            16 => Self::DrawIndex,
            _ => return None,
        })
    }
}
