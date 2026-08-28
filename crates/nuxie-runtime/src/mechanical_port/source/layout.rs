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

pub mod artboard_component_list_override;
pub mod axis;
pub mod axis_x;
pub mod axis_y;
pub mod grid_item_placement;
pub mod grid_track;
pub mod layout_component_style;
pub mod layout_data;
pub mod layout_enums;
pub mod layout_measure_mode;
pub mod layout_node_provider;
pub mod layout_node_style;
pub mod layout_participant;
pub mod layout_sizing_style;
pub mod layout_style_applier;
pub mod n_sliced_node;
pub mod n_slicer;
pub mod n_slicer_details;
pub mod n_slicer_tile_mode;
pub mod style_overrider;
