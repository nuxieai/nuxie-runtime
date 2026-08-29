#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AstcFootprint {
    pub width: u8,
    pub height: u8,
}

pub const ASTC_FOOTPRINTS: [AstcFootprint; 14] = [
    AstcFootprint {
        width: 4,
        height: 4,
    },
    AstcFootprint {
        width: 5,
        height: 4,
    },
    AstcFootprint {
        width: 5,
        height: 5,
    },
    AstcFootprint {
        width: 6,
        height: 5,
    },
    AstcFootprint {
        width: 6,
        height: 6,
    },
    AstcFootprint {
        width: 8,
        height: 5,
    },
    AstcFootprint {
        width: 8,
        height: 6,
    },
    AstcFootprint {
        width: 8,
        height: 8,
    },
    AstcFootprint {
        width: 10,
        height: 5,
    },
    AstcFootprint {
        width: 10,
        height: 6,
    },
    AstcFootprint {
        width: 10,
        height: 8,
    },
    AstcFootprint {
        width: 10,
        height: 10,
    },
    AstcFootprint {
        width: 12,
        height: 10,
    },
    AstcFootprint {
        width: 12,
        height: 12,
    },
];

pub fn astc_footprint_index(block_width: u8, block_height: u8) -> i32 {
    ASTC_FOOTPRINTS
        .iter()
        .position(|footprint| footprint.width == block_width && footprint.height == block_height)
        .map_or(-1, |index| index as i32)
}
