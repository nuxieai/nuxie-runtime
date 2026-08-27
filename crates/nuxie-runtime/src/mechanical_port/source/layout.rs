#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Fit {
    Fill,
    Contain,
    Cover,
    FitWidth,
    FitHeight,
    None,
    ScaleDown,
    Layout,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Alignment {
    x: f32,
    y: f32,
}

impl Alignment {
    pub const TOP_LEFT: Self = Self::new(-1.0, -1.0);
    pub const TOP_CENTER: Self = Self::new(0.0, -1.0);
    pub const TOP_RIGHT: Self = Self::new(1.0, -1.0);
    pub const CENTER_LEFT: Self = Self::new(-1.0, 0.0);
    pub const CENTER: Self = Self::new(0.0, 0.0);
    pub const CENTER_RIGHT: Self = Self::new(1.0, 0.0);
    pub const BOTTOM_LEFT: Self = Self::new(-1.0, 1.0);
    pub const BOTTOM_CENTER: Self = Self::new(0.0, 1.0);
    pub const BOTTOM_RIGHT: Self = Self::new(1.0, 1.0);

    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    pub const fn x(&self) -> f32 {
        self.x
    }

    pub const fn y(&self) -> f32 {
        self.y
    }
}

impl Default for Alignment {
    fn default() -> Self {
        Self::new(0.0, 0.0)
    }
}
