use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, layout::layout_component_style::LayoutComponentStyle,
    layout::layout_sizing_style::LayoutSizingStyle,
};

pub trait LayoutComponentStyleBaseCallbacks: crate::mechanical_port::source::generated::layout::layout_sizing_style_base::LayoutSizingStyleBaseCallbacks {
    fn notify_property_changed(&mut self, property_key: u16);
    fn gap_horizontal_changed(&mut self) {}
    fn gap_vertical_changed(&mut self) {}
    fn border_left_changed(&mut self) {}
    fn border_right_changed(&mut self) {}
    fn border_top_changed(&mut self) {}
    fn border_bottom_changed(&mut self) {}
    fn margin_left_changed(&mut self) {}
    fn margin_right_changed(&mut self) {}
    fn margin_top_changed(&mut self) {}
    fn margin_bottom_changed(&mut self) {}
    fn padding_left_changed(&mut self) {}
    fn padding_right_changed(&mut self) {}
    fn padding_top_changed(&mut self) {}
    fn padding_bottom_changed(&mut self) {}
    fn position_left_changed(&mut self) {}
    fn position_right_changed(&mut self) {}
    fn position_top_changed(&mut self) {}
    fn position_bottom_changed(&mut self) {}
    fn flex_basis_changed(&mut self) {}
    fn aspect_ratio_changed(&mut self) {}
    fn interpolator_id_changed(&mut self) {}
    fn interpolation_time_changed(&mut self) {}
    fn flex_basis_units_value_changed(&mut self) {}
    fn layout_alignment_type_changed(&mut self) {}
    fn animation_style_type_changed(&mut self) {}
    fn interpolation_type_changed(&mut self) {}
    fn position_type_value_changed(&mut self) {}
    fn flex_direction_value_changed(&mut self) {}
    fn direction_value_changed(&mut self) {}
    fn flex_wrap_value_changed(&mut self) {}
    fn overflow_value_changed(&mut self) {}
    fn intrinsically_sized_value_changed(&mut self) {}
    fn border_left_units_value_changed(&mut self) {}
    fn border_right_units_value_changed(&mut self) {}
    fn border_top_units_value_changed(&mut self) {}
    fn border_bottom_units_value_changed(&mut self) {}
    fn margin_left_units_value_changed(&mut self) {}
    fn margin_right_units_value_changed(&mut self) {}
    fn margin_top_units_value_changed(&mut self) {}
    fn margin_bottom_units_value_changed(&mut self) {}
    fn padding_left_units_value_changed(&mut self) {}
    fn padding_right_units_value_changed(&mut self) {}
    fn padding_top_units_value_changed(&mut self) {}
    fn padding_bottom_units_value_changed(&mut self) {}
    fn position_left_units_value_changed(&mut self) {}
    fn position_right_units_value_changed(&mut self) {}
    fn position_top_units_value_changed(&mut self) {}
    fn position_bottom_units_value_changed(&mut self) {}
    fn gap_horizontal_units_value_changed(&mut self) {}
    fn gap_vertical_units_value_changed(&mut self) {}
    fn link_corner_radius_changed(&mut self) {}
    fn justify_items_value_changed(&mut self) {}
    fn layout_type_value_changed(&mut self) {}
    fn corner_radius_tl_changed(&mut self) {}
    fn corner_radius_tr_changed(&mut self) {}
    fn corner_radius_bl_changed(&mut self) {}
    fn corner_radius_br_changed(&mut self) {}
}

pub struct LayoutComponentStyleBase {
    pub base: LayoutSizingStyle,
    gap_horizontal: f32,
    gap_vertical: f32,
    border_left: f32,
    border_right: f32,
    border_top: f32,
    border_bottom: f32,
    margin_left: f32,
    margin_right: f32,
    margin_top: f32,
    margin_bottom: f32,
    padding_left: f32,
    padding_right: f32,
    padding_top: f32,
    padding_bottom: f32,
    position_left: f32,
    position_right: f32,
    position_top: f32,
    position_bottom: f32,
    flex_basis: f32,
    aspect_ratio: f32,
    interpolator_id: u32,
    interpolation_time: f32,
    flex_basis_units_value: u8,
    layout_alignment_type: u8,
    animation_style_type: u8,
    interpolation_type: u8,
    position_type_value: u8,
    flex_direction_value: u8,
    direction_value: u8,
    flex_wrap_value: u8,
    overflow_value: u8,
    intrinsically_sized_value: bool,
    border_left_units_value: u8,
    border_right_units_value: u8,
    border_top_units_value: u8,
    border_bottom_units_value: u8,
    margin_left_units_value: u8,
    margin_right_units_value: u8,
    margin_top_units_value: u8,
    margin_bottom_units_value: u8,
    padding_left_units_value: u8,
    padding_right_units_value: u8,
    padding_top_units_value: u8,
    padding_bottom_units_value: u8,
    position_left_units_value: u8,
    position_right_units_value: u8,
    position_top_units_value: u8,
    position_bottom_units_value: u8,
    gap_horizontal_units_value: u8,
    gap_vertical_units_value: u8,
    link_corner_radius: bool,
    justify_items_value: u8,
    layout_type_value: u8,
    corner_radius_tl: f32,
    corner_radius_tr: f32,
    corner_radius_bl: f32,
    corner_radius_br: f32,
}

impl Default for LayoutComponentStyleBase {
    fn default() -> Self {
        Self {
            base: LayoutSizingStyle::default(),
            gap_horizontal: 0.0,
            gap_vertical: 0.0,
            border_left: 0.0,
            border_right: 0.0,
            border_top: 0.0,
            border_bottom: 0.0,
            margin_left: 0.0,
            margin_right: 0.0,
            margin_top: 0.0,
            margin_bottom: 0.0,
            padding_left: 0.0,
            padding_right: 0.0,
            padding_top: 0.0,
            padding_bottom: 0.0,
            position_left: 0.0,
            position_right: 0.0,
            position_top: 0.0,
            position_bottom: 0.0,
            flex_basis: 0.0,
            aspect_ratio: 0.0,
            interpolator_id: u32::MAX,
            interpolation_time: 0.0,
            flex_basis_units_value: 3,
            layout_alignment_type: 0,
            animation_style_type: 0,
            interpolation_type: 0,
            position_type_value: 1,
            flex_direction_value: 2,
            direction_value: 0,
            flex_wrap_value: 0,
            overflow_value: 0,
            intrinsically_sized_value: false,
            border_left_units_value: 0,
            border_right_units_value: 0,
            border_top_units_value: 0,
            border_bottom_units_value: 0,
            margin_left_units_value: 0,
            margin_right_units_value: 0,
            margin_top_units_value: 0,
            margin_bottom_units_value: 0,
            padding_left_units_value: 0,
            padding_right_units_value: 0,
            padding_top_units_value: 0,
            padding_bottom_units_value: 0,
            position_left_units_value: 0,
            position_right_units_value: 0,
            position_top_units_value: 0,
            position_bottom_units_value: 0,
            gap_horizontal_units_value: 0,
            gap_vertical_units_value: 0,
            link_corner_radius: true,
            justify_items_value: 7,
            layout_type_value: 0,
            corner_radius_tl: 0.0,
            corner_radius_tr: 0.0,
            corner_radius_bl: 0.0,
            corner_radius_br: 0.0,
        }
    }
}

impl LayoutComponentStyleBase {
    pub const TYPE_KEY: u16 = 420;
    pub const GAP_HORIZONTAL_PROPERTY_KEY: u16 = 498;
    pub const GAP_VERTICAL_PROPERTY_KEY: u16 = 499;
    pub const BORDER_LEFT_PROPERTY_KEY: u16 = 504;
    pub const BORDER_RIGHT_PROPERTY_KEY: u16 = 505;
    pub const BORDER_TOP_PROPERTY_KEY: u16 = 506;
    pub const BORDER_BOTTOM_PROPERTY_KEY: u16 = 507;
    pub const MARGIN_LEFT_PROPERTY_KEY: u16 = 508;
    pub const MARGIN_RIGHT_PROPERTY_KEY: u16 = 509;
    pub const MARGIN_TOP_PROPERTY_KEY: u16 = 510;
    pub const MARGIN_BOTTOM_PROPERTY_KEY: u16 = 511;
    pub const PADDING_LEFT_PROPERTY_KEY: u16 = 512;
    pub const PADDING_RIGHT_PROPERTY_KEY: u16 = 513;
    pub const PADDING_TOP_PROPERTY_KEY: u16 = 514;
    pub const PADDING_BOTTOM_PROPERTY_KEY: u16 = 515;
    pub const POSITION_LEFT_PROPERTY_KEY: u16 = 516;
    pub const POSITION_RIGHT_PROPERTY_KEY: u16 = 517;
    pub const POSITION_TOP_PROPERTY_KEY: u16 = 518;
    pub const POSITION_BOTTOM_PROPERTY_KEY: u16 = 519;
    pub const FLEX_BASIS_PROPERTY_KEY: u16 = 523;
    pub const ASPECT_RATIO_PROPERTY_KEY: u16 = 524;
    pub const INTERPOLATOR_ID_PROPERTY_KEY: u16 = 591;
    pub const INTERPOLATION_TIME_PROPERTY_KEY: u16 = 592;
    pub const FLEX_BASIS_UNITS_VALUE_PROPERTY_KEY: u16 = 705;
    pub const LAYOUT_ALIGNMENT_TYPE_PROPERTY_KEY: u16 = 632;
    pub const ANIMATION_STYLE_TYPE_PROPERTY_KEY: u16 = 589;
    pub const INTERPOLATION_TYPE_PROPERTY_KEY: u16 = 590;
    pub const POSITION_TYPE_VALUE_PROPERTY_KEY: u16 = 597;
    pub const FLEX_DIRECTION_VALUE_PROPERTY_KEY: u16 = 598;
    pub const DIRECTION_VALUE_PROPERTY_KEY: u16 = 599;
    pub const FLEX_WRAP_VALUE_PROPERTY_KEY: u16 = 604;
    pub const OVERFLOW_VALUE_PROPERTY_KEY: u16 = 605;
    pub const INTRINSICALLY_SIZED_VALUE_PROPERTY_KEY: u16 = 606;
    pub const BORDER_LEFT_UNITS_VALUE_PROPERTY_KEY: u16 = 609;
    pub const BORDER_RIGHT_UNITS_VALUE_PROPERTY_KEY: u16 = 610;
    pub const BORDER_TOP_UNITS_VALUE_PROPERTY_KEY: u16 = 611;
    pub const BORDER_BOTTOM_UNITS_VALUE_PROPERTY_KEY: u16 = 612;
    pub const MARGIN_LEFT_UNITS_VALUE_PROPERTY_KEY: u16 = 613;
    pub const MARGIN_RIGHT_UNITS_VALUE_PROPERTY_KEY: u16 = 614;
    pub const MARGIN_TOP_UNITS_VALUE_PROPERTY_KEY: u16 = 615;
    pub const MARGIN_BOTTOM_UNITS_VALUE_PROPERTY_KEY: u16 = 616;
    pub const PADDING_LEFT_UNITS_VALUE_PROPERTY_KEY: u16 = 617;
    pub const PADDING_RIGHT_UNITS_VALUE_PROPERTY_KEY: u16 = 618;
    pub const PADDING_TOP_UNITS_VALUE_PROPERTY_KEY: u16 = 619;
    pub const PADDING_BOTTOM_UNITS_VALUE_PROPERTY_KEY: u16 = 620;
    pub const POSITION_LEFT_UNITS_VALUE_PROPERTY_KEY: u16 = 621;
    pub const POSITION_RIGHT_UNITS_VALUE_PROPERTY_KEY: u16 = 622;
    pub const POSITION_TOP_UNITS_VALUE_PROPERTY_KEY: u16 = 623;
    pub const POSITION_BOTTOM_UNITS_VALUE_PROPERTY_KEY: u16 = 624;
    pub const GAP_HORIZONTAL_UNITS_VALUE_PROPERTY_KEY: u16 = 625;
    pub const GAP_VERTICAL_UNITS_VALUE_PROPERTY_KEY: u16 = 626;
    pub const LINK_CORNER_RADIUS_PROPERTY_KEY: u16 = 639;
    pub const JUSTIFY_ITEMS_VALUE_PROPERTY_KEY: u16 = 1045;
    pub const LAYOUT_TYPE_VALUE_PROPERTY_KEY: u16 = 1059;
    pub const CORNER_RADIUS_TL_PROPERTY_KEY: u16 = 640;
    pub const CORNER_RADIUS_TR_PROPERTY_KEY: u16 = 641;
    pub const CORNER_RADIUS_BL_PROPERTY_KEY: u16 = 642;
    pub const CORNER_RADIUS_BR_PROPERTY_KEY: u16 = 643;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 1056 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn gap_horizontal(&self) -> f32 {
        self.gap_horizontal
    }
    pub fn set_gap_horizontal(
        &mut self,
        value: f32,
        callbacks: &mut impl LayoutComponentStyleBaseCallbacks,
    ) {
        if !self.set_gap_horizontal_value(value) {
            return;
        }
        callbacks.gap_horizontal_changed();
        LayoutComponentStyleBaseCallbacks::notify_property_changed(
            callbacks,
            Self::GAP_HORIZONTAL_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_gap_horizontal_value(&mut self, value: f32) -> bool {
        if self.gap_horizontal == value {
            return false;
        }
        self.gap_horizontal = value;
        true
    }
    pub fn gap_vertical(&self) -> f32 {
        self.gap_vertical
    }
    pub fn set_gap_vertical(
        &mut self,
        value: f32,
        callbacks: &mut impl LayoutComponentStyleBaseCallbacks,
    ) {
        if !self.set_gap_vertical_value(value) {
            return;
        }
        callbacks.gap_vertical_changed();
        LayoutComponentStyleBaseCallbacks::notify_property_changed(
            callbacks,
            Self::GAP_VERTICAL_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_gap_vertical_value(&mut self, value: f32) -> bool {
        if self.gap_vertical == value {
            return false;
        }
        self.gap_vertical = value;
        true
    }
    pub fn border_left(&self) -> f32 {
        self.border_left
    }
    pub fn set_border_left(
        &mut self,
        value: f32,
        callbacks: &mut impl LayoutComponentStyleBaseCallbacks,
    ) {
        if !self.set_border_left_value(value) {
            return;
        }
        callbacks.border_left_changed();
        LayoutComponentStyleBaseCallbacks::notify_property_changed(
            callbacks,
            Self::BORDER_LEFT_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_border_left_value(&mut self, value: f32) -> bool {
        if self.border_left == value {
            return false;
        }
        self.border_left = value;
        true
    }
    pub fn border_right(&self) -> f32 {
        self.border_right
    }
    pub fn set_border_right(
        &mut self,
        value: f32,
        callbacks: &mut impl LayoutComponentStyleBaseCallbacks,
    ) {
        if !self.set_border_right_value(value) {
            return;
        }
        callbacks.border_right_changed();
        LayoutComponentStyleBaseCallbacks::notify_property_changed(
            callbacks,
            Self::BORDER_RIGHT_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_border_right_value(&mut self, value: f32) -> bool {
        if self.border_right == value {
            return false;
        }
        self.border_right = value;
        true
    }
    pub fn border_top(&self) -> f32 {
        self.border_top
    }
    pub fn set_border_top(
        &mut self,
        value: f32,
        callbacks: &mut impl LayoutComponentStyleBaseCallbacks,
    ) {
        if !self.set_border_top_value(value) {
            return;
        }
        callbacks.border_top_changed();
        LayoutComponentStyleBaseCallbacks::notify_property_changed(
            callbacks,
            Self::BORDER_TOP_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_border_top_value(&mut self, value: f32) -> bool {
        if self.border_top == value {
            return false;
        }
        self.border_top = value;
        true
    }
    pub fn border_bottom(&self) -> f32 {
        self.border_bottom
    }
    pub fn set_border_bottom(
        &mut self,
        value: f32,
        callbacks: &mut impl LayoutComponentStyleBaseCallbacks,
    ) {
        if !self.set_border_bottom_value(value) {
            return;
        }
        callbacks.border_bottom_changed();
        LayoutComponentStyleBaseCallbacks::notify_property_changed(
            callbacks,
            Self::BORDER_BOTTOM_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_border_bottom_value(&mut self, value: f32) -> bool {
        if self.border_bottom == value {
            return false;
        }
        self.border_bottom = value;
        true
    }
    pub fn margin_left(&self) -> f32 {
        self.margin_left
    }
    pub fn set_margin_left(
        &mut self,
        value: f32,
        callbacks: &mut impl LayoutComponentStyleBaseCallbacks,
    ) {
        if !self.set_margin_left_value(value) {
            return;
        }
        callbacks.margin_left_changed();
        LayoutComponentStyleBaseCallbacks::notify_property_changed(
            callbacks,
            Self::MARGIN_LEFT_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_margin_left_value(&mut self, value: f32) -> bool {
        if self.margin_left == value {
            return false;
        }
        self.margin_left = value;
        true
    }
    pub fn margin_right(&self) -> f32 {
        self.margin_right
    }
    pub fn set_margin_right(
        &mut self,
        value: f32,
        callbacks: &mut impl LayoutComponentStyleBaseCallbacks,
    ) {
        if !self.set_margin_right_value(value) {
            return;
        }
        callbacks.margin_right_changed();
        LayoutComponentStyleBaseCallbacks::notify_property_changed(
            callbacks,
            Self::MARGIN_RIGHT_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_margin_right_value(&mut self, value: f32) -> bool {
        if self.margin_right == value {
            return false;
        }
        self.margin_right = value;
        true
    }
    pub fn margin_top(&self) -> f32 {
        self.margin_top
    }
    pub fn set_margin_top(
        &mut self,
        value: f32,
        callbacks: &mut impl LayoutComponentStyleBaseCallbacks,
    ) {
        if !self.set_margin_top_value(value) {
            return;
        }
        callbacks.margin_top_changed();
        LayoutComponentStyleBaseCallbacks::notify_property_changed(
            callbacks,
            Self::MARGIN_TOP_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_margin_top_value(&mut self, value: f32) -> bool {
        if self.margin_top == value {
            return false;
        }
        self.margin_top = value;
        true
    }
    pub fn margin_bottom(&self) -> f32 {
        self.margin_bottom
    }
    pub fn set_margin_bottom(
        &mut self,
        value: f32,
        callbacks: &mut impl LayoutComponentStyleBaseCallbacks,
    ) {
        if !self.set_margin_bottom_value(value) {
            return;
        }
        callbacks.margin_bottom_changed();
        LayoutComponentStyleBaseCallbacks::notify_property_changed(
            callbacks,
            Self::MARGIN_BOTTOM_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_margin_bottom_value(&mut self, value: f32) -> bool {
        if self.margin_bottom == value {
            return false;
        }
        self.margin_bottom = value;
        true
    }
    pub fn padding_left(&self) -> f32 {
        self.padding_left
    }
    pub fn set_padding_left(
        &mut self,
        value: f32,
        callbacks: &mut impl LayoutComponentStyleBaseCallbacks,
    ) {
        if !self.set_padding_left_value(value) {
            return;
        }
        callbacks.padding_left_changed();
        LayoutComponentStyleBaseCallbacks::notify_property_changed(
            callbacks,
            Self::PADDING_LEFT_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_padding_left_value(&mut self, value: f32) -> bool {
        if self.padding_left == value {
            return false;
        }
        self.padding_left = value;
        true
    }
    pub fn padding_right(&self) -> f32 {
        self.padding_right
    }
    pub fn set_padding_right(
        &mut self,
        value: f32,
        callbacks: &mut impl LayoutComponentStyleBaseCallbacks,
    ) {
        if !self.set_padding_right_value(value) {
            return;
        }
        callbacks.padding_right_changed();
        LayoutComponentStyleBaseCallbacks::notify_property_changed(
            callbacks,
            Self::PADDING_RIGHT_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_padding_right_value(&mut self, value: f32) -> bool {
        if self.padding_right == value {
            return false;
        }
        self.padding_right = value;
        true
    }
    pub fn padding_top(&self) -> f32 {
        self.padding_top
    }
    pub fn set_padding_top(
        &mut self,
        value: f32,
        callbacks: &mut impl LayoutComponentStyleBaseCallbacks,
    ) {
        if !self.set_padding_top_value(value) {
            return;
        }
        callbacks.padding_top_changed();
        LayoutComponentStyleBaseCallbacks::notify_property_changed(
            callbacks,
            Self::PADDING_TOP_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_padding_top_value(&mut self, value: f32) -> bool {
        if self.padding_top == value {
            return false;
        }
        self.padding_top = value;
        true
    }
    pub fn padding_bottom(&self) -> f32 {
        self.padding_bottom
    }
    pub fn set_padding_bottom(
        &mut self,
        value: f32,
        callbacks: &mut impl LayoutComponentStyleBaseCallbacks,
    ) {
        if !self.set_padding_bottom_value(value) {
            return;
        }
        callbacks.padding_bottom_changed();
        LayoutComponentStyleBaseCallbacks::notify_property_changed(
            callbacks,
            Self::PADDING_BOTTOM_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_padding_bottom_value(&mut self, value: f32) -> bool {
        if self.padding_bottom == value {
            return false;
        }
        self.padding_bottom = value;
        true
    }
    pub fn position_left(&self) -> f32 {
        self.position_left
    }
    pub fn set_position_left(
        &mut self,
        value: f32,
        callbacks: &mut impl LayoutComponentStyleBaseCallbacks,
    ) {
        if !self.set_position_left_value(value) {
            return;
        }
        callbacks.position_left_changed();
        LayoutComponentStyleBaseCallbacks::notify_property_changed(
            callbacks,
            Self::POSITION_LEFT_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_position_left_value(&mut self, value: f32) -> bool {
        if self.position_left == value {
            return false;
        }
        self.position_left = value;
        true
    }
    pub fn position_right(&self) -> f32 {
        self.position_right
    }
    pub fn set_position_right(
        &mut self,
        value: f32,
        callbacks: &mut impl LayoutComponentStyleBaseCallbacks,
    ) {
        if !self.set_position_right_value(value) {
            return;
        }
        callbacks.position_right_changed();
        LayoutComponentStyleBaseCallbacks::notify_property_changed(
            callbacks,
            Self::POSITION_RIGHT_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_position_right_value(&mut self, value: f32) -> bool {
        if self.position_right == value {
            return false;
        }
        self.position_right = value;
        true
    }
    pub fn position_top(&self) -> f32 {
        self.position_top
    }
    pub fn set_position_top(
        &mut self,
        value: f32,
        callbacks: &mut impl LayoutComponentStyleBaseCallbacks,
    ) {
        if !self.set_position_top_value(value) {
            return;
        }
        callbacks.position_top_changed();
        LayoutComponentStyleBaseCallbacks::notify_property_changed(
            callbacks,
            Self::POSITION_TOP_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_position_top_value(&mut self, value: f32) -> bool {
        if self.position_top == value {
            return false;
        }
        self.position_top = value;
        true
    }
    pub fn position_bottom(&self) -> f32 {
        self.position_bottom
    }
    pub fn set_position_bottom(
        &mut self,
        value: f32,
        callbacks: &mut impl LayoutComponentStyleBaseCallbacks,
    ) {
        if !self.set_position_bottom_value(value) {
            return;
        }
        callbacks.position_bottom_changed();
        LayoutComponentStyleBaseCallbacks::notify_property_changed(
            callbacks,
            Self::POSITION_BOTTOM_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_position_bottom_value(&mut self, value: f32) -> bool {
        if self.position_bottom == value {
            return false;
        }
        self.position_bottom = value;
        true
    }
    pub fn flex_basis(&self) -> f32 {
        self.flex_basis
    }
    pub fn set_flex_basis(
        &mut self,
        value: f32,
        callbacks: &mut impl LayoutComponentStyleBaseCallbacks,
    ) {
        if !self.set_flex_basis_value(value) {
            return;
        }
        callbacks.flex_basis_changed();
        LayoutComponentStyleBaseCallbacks::notify_property_changed(
            callbacks,
            Self::FLEX_BASIS_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_flex_basis_value(&mut self, value: f32) -> bool {
        if self.flex_basis == value {
            return false;
        }
        self.flex_basis = value;
        true
    }
    pub fn aspect_ratio(&self) -> f32 {
        self.aspect_ratio
    }
    pub fn set_aspect_ratio(
        &mut self,
        value: f32,
        callbacks: &mut impl LayoutComponentStyleBaseCallbacks,
    ) {
        if !self.set_aspect_ratio_value(value) {
            return;
        }
        callbacks.aspect_ratio_changed();
        LayoutComponentStyleBaseCallbacks::notify_property_changed(
            callbacks,
            Self::ASPECT_RATIO_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_aspect_ratio_value(&mut self, value: f32) -> bool {
        if self.aspect_ratio == value {
            return false;
        }
        self.aspect_ratio = value;
        true
    }
    pub fn interpolator_id(&self) -> u32 {
        self.interpolator_id
    }
    pub fn set_interpolator_id(
        &mut self,
        value: u32,
        callbacks: &mut impl LayoutComponentStyleBaseCallbacks,
    ) {
        if !self.set_interpolator_id_value(value) {
            return;
        }
        callbacks.interpolator_id_changed();
        LayoutComponentStyleBaseCallbacks::notify_property_changed(
            callbacks,
            Self::INTERPOLATOR_ID_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_interpolator_id_value(&mut self, value: u32) -> bool {
        if self.interpolator_id == value {
            return false;
        }
        self.interpolator_id = value;
        true
    }
    pub fn interpolation_time(&self) -> f32 {
        self.interpolation_time
    }
    pub fn set_interpolation_time(
        &mut self,
        value: f32,
        callbacks: &mut impl LayoutComponentStyleBaseCallbacks,
    ) {
        if !self.set_interpolation_time_value(value) {
            return;
        }
        callbacks.interpolation_time_changed();
        LayoutComponentStyleBaseCallbacks::notify_property_changed(
            callbacks,
            Self::INTERPOLATION_TIME_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_interpolation_time_value(&mut self, value: f32) -> bool {
        if self.interpolation_time == value {
            return false;
        }
        self.interpolation_time = value;
        true
    }
    pub fn flex_basis_units_value(&self) -> u8 {
        self.flex_basis_units_value
    }
    pub fn set_flex_basis_units_value(
        &mut self,
        value: u8,
        callbacks: &mut impl LayoutComponentStyleBaseCallbacks,
    ) {
        if !self.set_flex_basis_units_value_value(value) {
            return;
        }
        callbacks.flex_basis_units_value_changed();
        LayoutComponentStyleBaseCallbacks::notify_property_changed(
            callbacks,
            Self::FLEX_BASIS_UNITS_VALUE_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_flex_basis_units_value_value(&mut self, value: u8) -> bool {
        if self.flex_basis_units_value == value {
            return false;
        }
        self.flex_basis_units_value = value;
        true
    }
    pub fn layout_alignment_type(&self) -> u8 {
        self.layout_alignment_type
    }
    pub fn set_layout_alignment_type(
        &mut self,
        value: u8,
        callbacks: &mut impl LayoutComponentStyleBaseCallbacks,
    ) {
        if !self.set_layout_alignment_type_value(value) {
            return;
        }
        callbacks.layout_alignment_type_changed();
        LayoutComponentStyleBaseCallbacks::notify_property_changed(
            callbacks,
            Self::LAYOUT_ALIGNMENT_TYPE_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_layout_alignment_type_value(&mut self, value: u8) -> bool {
        if self.layout_alignment_type == value {
            return false;
        }
        self.layout_alignment_type = value;
        true
    }
    pub fn animation_style_type(&self) -> u8 {
        self.animation_style_type
    }
    pub fn set_animation_style_type(
        &mut self,
        value: u8,
        callbacks: &mut impl LayoutComponentStyleBaseCallbacks,
    ) {
        if !self.set_animation_style_type_value(value) {
            return;
        }
        callbacks.animation_style_type_changed();
        LayoutComponentStyleBaseCallbacks::notify_property_changed(
            callbacks,
            Self::ANIMATION_STYLE_TYPE_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_animation_style_type_value(&mut self, value: u8) -> bool {
        if self.animation_style_type == value {
            return false;
        }
        self.animation_style_type = value;
        true
    }
    pub fn interpolation_type(&self) -> u8 {
        self.interpolation_type
    }
    pub fn set_interpolation_type(
        &mut self,
        value: u8,
        callbacks: &mut impl LayoutComponentStyleBaseCallbacks,
    ) {
        if !self.set_interpolation_type_value(value) {
            return;
        }
        callbacks.interpolation_type_changed();
        LayoutComponentStyleBaseCallbacks::notify_property_changed(
            callbacks,
            Self::INTERPOLATION_TYPE_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_interpolation_type_value(&mut self, value: u8) -> bool {
        if self.interpolation_type == value {
            return false;
        }
        self.interpolation_type = value;
        true
    }
    pub fn position_type_value(&self) -> u8 {
        self.position_type_value
    }
    pub fn set_position_type_value(
        &mut self,
        value: u8,
        callbacks: &mut impl LayoutComponentStyleBaseCallbacks,
    ) {
        if !self.set_position_type_value_value(value) {
            return;
        }
        callbacks.position_type_value_changed();
        LayoutComponentStyleBaseCallbacks::notify_property_changed(
            callbacks,
            Self::POSITION_TYPE_VALUE_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_position_type_value_value(&mut self, value: u8) -> bool {
        if self.position_type_value == value {
            return false;
        }
        self.position_type_value = value;
        true
    }
    pub fn flex_direction_value(&self) -> u8 {
        self.flex_direction_value
    }
    pub fn set_flex_direction_value(
        &mut self,
        value: u8,
        callbacks: &mut impl LayoutComponentStyleBaseCallbacks,
    ) {
        if !self.set_flex_direction_value_value(value) {
            return;
        }
        callbacks.flex_direction_value_changed();
        LayoutComponentStyleBaseCallbacks::notify_property_changed(
            callbacks,
            Self::FLEX_DIRECTION_VALUE_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_flex_direction_value_value(&mut self, value: u8) -> bool {
        if self.flex_direction_value == value {
            return false;
        }
        self.flex_direction_value = value;
        true
    }
    pub fn direction_value(&self) -> u8 {
        self.direction_value
    }
    pub fn set_direction_value(
        &mut self,
        value: u8,
        callbacks: &mut impl LayoutComponentStyleBaseCallbacks,
    ) {
        if !self.set_direction_value_value(value) {
            return;
        }
        callbacks.direction_value_changed();
        LayoutComponentStyleBaseCallbacks::notify_property_changed(
            callbacks,
            Self::DIRECTION_VALUE_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_direction_value_value(&mut self, value: u8) -> bool {
        if self.direction_value == value {
            return false;
        }
        self.direction_value = value;
        true
    }
    pub fn flex_wrap_value(&self) -> u8 {
        self.flex_wrap_value
    }
    pub fn set_flex_wrap_value(
        &mut self,
        value: u8,
        callbacks: &mut impl LayoutComponentStyleBaseCallbacks,
    ) {
        if !self.set_flex_wrap_value_value(value) {
            return;
        }
        callbacks.flex_wrap_value_changed();
        LayoutComponentStyleBaseCallbacks::notify_property_changed(
            callbacks,
            Self::FLEX_WRAP_VALUE_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_flex_wrap_value_value(&mut self, value: u8) -> bool {
        if self.flex_wrap_value == value {
            return false;
        }
        self.flex_wrap_value = value;
        true
    }
    pub fn overflow_value(&self) -> u8 {
        self.overflow_value
    }
    pub fn set_overflow_value(
        &mut self,
        value: u8,
        callbacks: &mut impl LayoutComponentStyleBaseCallbacks,
    ) {
        if !self.set_overflow_value_value(value) {
            return;
        }
        callbacks.overflow_value_changed();
        LayoutComponentStyleBaseCallbacks::notify_property_changed(
            callbacks,
            Self::OVERFLOW_VALUE_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_overflow_value_value(&mut self, value: u8) -> bool {
        if self.overflow_value == value {
            return false;
        }
        self.overflow_value = value;
        true
    }
    pub fn intrinsically_sized_value(&self) -> bool {
        self.intrinsically_sized_value
    }
    pub fn set_intrinsically_sized_value(
        &mut self,
        value: bool,
        callbacks: &mut impl LayoutComponentStyleBaseCallbacks,
    ) {
        if !self.set_intrinsically_sized_value_value(value) {
            return;
        }
        callbacks.intrinsically_sized_value_changed();
        LayoutComponentStyleBaseCallbacks::notify_property_changed(
            callbacks,
            Self::INTRINSICALLY_SIZED_VALUE_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_intrinsically_sized_value_value(&mut self, value: bool) -> bool {
        if self.intrinsically_sized_value == value {
            return false;
        }
        self.intrinsically_sized_value = value;
        true
    }
    pub fn border_left_units_value(&self) -> u8 {
        self.border_left_units_value
    }
    pub fn set_border_left_units_value(
        &mut self,
        value: u8,
        callbacks: &mut impl LayoutComponentStyleBaseCallbacks,
    ) {
        if !self.set_border_left_units_value_value(value) {
            return;
        }
        callbacks.border_left_units_value_changed();
        LayoutComponentStyleBaseCallbacks::notify_property_changed(
            callbacks,
            Self::BORDER_LEFT_UNITS_VALUE_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_border_left_units_value_value(&mut self, value: u8) -> bool {
        if self.border_left_units_value == value {
            return false;
        }
        self.border_left_units_value = value;
        true
    }
    pub fn border_right_units_value(&self) -> u8 {
        self.border_right_units_value
    }
    pub fn set_border_right_units_value(
        &mut self,
        value: u8,
        callbacks: &mut impl LayoutComponentStyleBaseCallbacks,
    ) {
        if !self.set_border_right_units_value_value(value) {
            return;
        }
        callbacks.border_right_units_value_changed();
        LayoutComponentStyleBaseCallbacks::notify_property_changed(
            callbacks,
            Self::BORDER_RIGHT_UNITS_VALUE_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_border_right_units_value_value(&mut self, value: u8) -> bool {
        if self.border_right_units_value == value {
            return false;
        }
        self.border_right_units_value = value;
        true
    }
    pub fn border_top_units_value(&self) -> u8 {
        self.border_top_units_value
    }
    pub fn set_border_top_units_value(
        &mut self,
        value: u8,
        callbacks: &mut impl LayoutComponentStyleBaseCallbacks,
    ) {
        if !self.set_border_top_units_value_value(value) {
            return;
        }
        callbacks.border_top_units_value_changed();
        LayoutComponentStyleBaseCallbacks::notify_property_changed(
            callbacks,
            Self::BORDER_TOP_UNITS_VALUE_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_border_top_units_value_value(&mut self, value: u8) -> bool {
        if self.border_top_units_value == value {
            return false;
        }
        self.border_top_units_value = value;
        true
    }
    pub fn border_bottom_units_value(&self) -> u8 {
        self.border_bottom_units_value
    }
    pub fn set_border_bottom_units_value(
        &mut self,
        value: u8,
        callbacks: &mut impl LayoutComponentStyleBaseCallbacks,
    ) {
        if !self.set_border_bottom_units_value_value(value) {
            return;
        }
        callbacks.border_bottom_units_value_changed();
        LayoutComponentStyleBaseCallbacks::notify_property_changed(
            callbacks,
            Self::BORDER_BOTTOM_UNITS_VALUE_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_border_bottom_units_value_value(&mut self, value: u8) -> bool {
        if self.border_bottom_units_value == value {
            return false;
        }
        self.border_bottom_units_value = value;
        true
    }
    pub fn margin_left_units_value(&self) -> u8 {
        self.margin_left_units_value
    }
    pub fn set_margin_left_units_value(
        &mut self,
        value: u8,
        callbacks: &mut impl LayoutComponentStyleBaseCallbacks,
    ) {
        if !self.set_margin_left_units_value_value(value) {
            return;
        }
        callbacks.margin_left_units_value_changed();
        LayoutComponentStyleBaseCallbacks::notify_property_changed(
            callbacks,
            Self::MARGIN_LEFT_UNITS_VALUE_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_margin_left_units_value_value(&mut self, value: u8) -> bool {
        if self.margin_left_units_value == value {
            return false;
        }
        self.margin_left_units_value = value;
        true
    }
    pub fn margin_right_units_value(&self) -> u8 {
        self.margin_right_units_value
    }
    pub fn set_margin_right_units_value(
        &mut self,
        value: u8,
        callbacks: &mut impl LayoutComponentStyleBaseCallbacks,
    ) {
        if !self.set_margin_right_units_value_value(value) {
            return;
        }
        callbacks.margin_right_units_value_changed();
        LayoutComponentStyleBaseCallbacks::notify_property_changed(
            callbacks,
            Self::MARGIN_RIGHT_UNITS_VALUE_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_margin_right_units_value_value(&mut self, value: u8) -> bool {
        if self.margin_right_units_value == value {
            return false;
        }
        self.margin_right_units_value = value;
        true
    }
    pub fn margin_top_units_value(&self) -> u8 {
        self.margin_top_units_value
    }
    pub fn set_margin_top_units_value(
        &mut self,
        value: u8,
        callbacks: &mut impl LayoutComponentStyleBaseCallbacks,
    ) {
        if !self.set_margin_top_units_value_value(value) {
            return;
        }
        callbacks.margin_top_units_value_changed();
        LayoutComponentStyleBaseCallbacks::notify_property_changed(
            callbacks,
            Self::MARGIN_TOP_UNITS_VALUE_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_margin_top_units_value_value(&mut self, value: u8) -> bool {
        if self.margin_top_units_value == value {
            return false;
        }
        self.margin_top_units_value = value;
        true
    }
    pub fn margin_bottom_units_value(&self) -> u8 {
        self.margin_bottom_units_value
    }
    pub fn set_margin_bottom_units_value(
        &mut self,
        value: u8,
        callbacks: &mut impl LayoutComponentStyleBaseCallbacks,
    ) {
        if !self.set_margin_bottom_units_value_value(value) {
            return;
        }
        callbacks.margin_bottom_units_value_changed();
        LayoutComponentStyleBaseCallbacks::notify_property_changed(
            callbacks,
            Self::MARGIN_BOTTOM_UNITS_VALUE_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_margin_bottom_units_value_value(&mut self, value: u8) -> bool {
        if self.margin_bottom_units_value == value {
            return false;
        }
        self.margin_bottom_units_value = value;
        true
    }
    pub fn padding_left_units_value(&self) -> u8 {
        self.padding_left_units_value
    }
    pub fn set_padding_left_units_value(
        &mut self,
        value: u8,
        callbacks: &mut impl LayoutComponentStyleBaseCallbacks,
    ) {
        if !self.set_padding_left_units_value_value(value) {
            return;
        }
        callbacks.padding_left_units_value_changed();
        LayoutComponentStyleBaseCallbacks::notify_property_changed(
            callbacks,
            Self::PADDING_LEFT_UNITS_VALUE_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_padding_left_units_value_value(&mut self, value: u8) -> bool {
        if self.padding_left_units_value == value {
            return false;
        }
        self.padding_left_units_value = value;
        true
    }
    pub fn padding_right_units_value(&self) -> u8 {
        self.padding_right_units_value
    }
    pub fn set_padding_right_units_value(
        &mut self,
        value: u8,
        callbacks: &mut impl LayoutComponentStyleBaseCallbacks,
    ) {
        if !self.set_padding_right_units_value_value(value) {
            return;
        }
        callbacks.padding_right_units_value_changed();
        LayoutComponentStyleBaseCallbacks::notify_property_changed(
            callbacks,
            Self::PADDING_RIGHT_UNITS_VALUE_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_padding_right_units_value_value(&mut self, value: u8) -> bool {
        if self.padding_right_units_value == value {
            return false;
        }
        self.padding_right_units_value = value;
        true
    }
    pub fn padding_top_units_value(&self) -> u8 {
        self.padding_top_units_value
    }
    pub fn set_padding_top_units_value(
        &mut self,
        value: u8,
        callbacks: &mut impl LayoutComponentStyleBaseCallbacks,
    ) {
        if !self.set_padding_top_units_value_value(value) {
            return;
        }
        callbacks.padding_top_units_value_changed();
        LayoutComponentStyleBaseCallbacks::notify_property_changed(
            callbacks,
            Self::PADDING_TOP_UNITS_VALUE_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_padding_top_units_value_value(&mut self, value: u8) -> bool {
        if self.padding_top_units_value == value {
            return false;
        }
        self.padding_top_units_value = value;
        true
    }
    pub fn padding_bottom_units_value(&self) -> u8 {
        self.padding_bottom_units_value
    }
    pub fn set_padding_bottom_units_value(
        &mut self,
        value: u8,
        callbacks: &mut impl LayoutComponentStyleBaseCallbacks,
    ) {
        if !self.set_padding_bottom_units_value_value(value) {
            return;
        }
        callbacks.padding_bottom_units_value_changed();
        LayoutComponentStyleBaseCallbacks::notify_property_changed(
            callbacks,
            Self::PADDING_BOTTOM_UNITS_VALUE_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_padding_bottom_units_value_value(&mut self, value: u8) -> bool {
        if self.padding_bottom_units_value == value {
            return false;
        }
        self.padding_bottom_units_value = value;
        true
    }
    pub fn position_left_units_value(&self) -> u8 {
        self.position_left_units_value
    }
    pub fn set_position_left_units_value(
        &mut self,
        value: u8,
        callbacks: &mut impl LayoutComponentStyleBaseCallbacks,
    ) {
        if !self.set_position_left_units_value_value(value) {
            return;
        }
        callbacks.position_left_units_value_changed();
        LayoutComponentStyleBaseCallbacks::notify_property_changed(
            callbacks,
            Self::POSITION_LEFT_UNITS_VALUE_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_position_left_units_value_value(&mut self, value: u8) -> bool {
        if self.position_left_units_value == value {
            return false;
        }
        self.position_left_units_value = value;
        true
    }
    pub fn position_right_units_value(&self) -> u8 {
        self.position_right_units_value
    }
    pub fn set_position_right_units_value(
        &mut self,
        value: u8,
        callbacks: &mut impl LayoutComponentStyleBaseCallbacks,
    ) {
        if !self.set_position_right_units_value_value(value) {
            return;
        }
        callbacks.position_right_units_value_changed();
        LayoutComponentStyleBaseCallbacks::notify_property_changed(
            callbacks,
            Self::POSITION_RIGHT_UNITS_VALUE_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_position_right_units_value_value(&mut self, value: u8) -> bool {
        if self.position_right_units_value == value {
            return false;
        }
        self.position_right_units_value = value;
        true
    }
    pub fn position_top_units_value(&self) -> u8 {
        self.position_top_units_value
    }
    pub fn set_position_top_units_value(
        &mut self,
        value: u8,
        callbacks: &mut impl LayoutComponentStyleBaseCallbacks,
    ) {
        if !self.set_position_top_units_value_value(value) {
            return;
        }
        callbacks.position_top_units_value_changed();
        LayoutComponentStyleBaseCallbacks::notify_property_changed(
            callbacks,
            Self::POSITION_TOP_UNITS_VALUE_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_position_top_units_value_value(&mut self, value: u8) -> bool {
        if self.position_top_units_value == value {
            return false;
        }
        self.position_top_units_value = value;
        true
    }
    pub fn position_bottom_units_value(&self) -> u8 {
        self.position_bottom_units_value
    }
    pub fn set_position_bottom_units_value(
        &mut self,
        value: u8,
        callbacks: &mut impl LayoutComponentStyleBaseCallbacks,
    ) {
        if !self.set_position_bottom_units_value_value(value) {
            return;
        }
        callbacks.position_bottom_units_value_changed();
        LayoutComponentStyleBaseCallbacks::notify_property_changed(
            callbacks,
            Self::POSITION_BOTTOM_UNITS_VALUE_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_position_bottom_units_value_value(&mut self, value: u8) -> bool {
        if self.position_bottom_units_value == value {
            return false;
        }
        self.position_bottom_units_value = value;
        true
    }
    pub fn gap_horizontal_units_value(&self) -> u8 {
        self.gap_horizontal_units_value
    }
    pub fn set_gap_horizontal_units_value(
        &mut self,
        value: u8,
        callbacks: &mut impl LayoutComponentStyleBaseCallbacks,
    ) {
        if !self.set_gap_horizontal_units_value_value(value) {
            return;
        }
        callbacks.gap_horizontal_units_value_changed();
        LayoutComponentStyleBaseCallbacks::notify_property_changed(
            callbacks,
            Self::GAP_HORIZONTAL_UNITS_VALUE_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_gap_horizontal_units_value_value(&mut self, value: u8) -> bool {
        if self.gap_horizontal_units_value == value {
            return false;
        }
        self.gap_horizontal_units_value = value;
        true
    }
    pub fn gap_vertical_units_value(&self) -> u8 {
        self.gap_vertical_units_value
    }
    pub fn set_gap_vertical_units_value(
        &mut self,
        value: u8,
        callbacks: &mut impl LayoutComponentStyleBaseCallbacks,
    ) {
        if !self.set_gap_vertical_units_value_value(value) {
            return;
        }
        callbacks.gap_vertical_units_value_changed();
        LayoutComponentStyleBaseCallbacks::notify_property_changed(
            callbacks,
            Self::GAP_VERTICAL_UNITS_VALUE_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_gap_vertical_units_value_value(&mut self, value: u8) -> bool {
        if self.gap_vertical_units_value == value {
            return false;
        }
        self.gap_vertical_units_value = value;
        true
    }
    pub fn link_corner_radius(&self) -> bool {
        self.link_corner_radius
    }
    pub fn set_link_corner_radius(
        &mut self,
        value: bool,
        callbacks: &mut impl LayoutComponentStyleBaseCallbacks,
    ) {
        if !self.set_link_corner_radius_value(value) {
            return;
        }
        callbacks.link_corner_radius_changed();
        LayoutComponentStyleBaseCallbacks::notify_property_changed(
            callbacks,
            Self::LINK_CORNER_RADIUS_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_link_corner_radius_value(&mut self, value: bool) -> bool {
        if self.link_corner_radius == value {
            return false;
        }
        self.link_corner_radius = value;
        true
    }
    pub fn justify_items_value(&self) -> u8 {
        self.justify_items_value
    }
    pub fn set_justify_items_value(
        &mut self,
        value: u8,
        callbacks: &mut impl LayoutComponentStyleBaseCallbacks,
    ) {
        if !self.set_justify_items_value_value(value) {
            return;
        }
        callbacks.justify_items_value_changed();
        LayoutComponentStyleBaseCallbacks::notify_property_changed(
            callbacks,
            Self::JUSTIFY_ITEMS_VALUE_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_justify_items_value_value(&mut self, value: u8) -> bool {
        if self.justify_items_value == value {
            return false;
        }
        self.justify_items_value = value;
        true
    }
    pub fn layout_type_value(&self) -> u8 {
        self.layout_type_value
    }
    pub fn set_layout_type_value(
        &mut self,
        value: u8,
        callbacks: &mut impl LayoutComponentStyleBaseCallbacks,
    ) {
        if !self.set_layout_type_value_value(value) {
            return;
        }
        callbacks.layout_type_value_changed();
        LayoutComponentStyleBaseCallbacks::notify_property_changed(
            callbacks,
            Self::LAYOUT_TYPE_VALUE_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_layout_type_value_value(&mut self, value: u8) -> bool {
        if self.layout_type_value == value {
            return false;
        }
        self.layout_type_value = value;
        true
    }
    pub fn corner_radius_tl(&self) -> f32 {
        self.corner_radius_tl
    }
    pub fn set_corner_radius_tl(
        &mut self,
        value: f32,
        callbacks: &mut impl LayoutComponentStyleBaseCallbacks,
    ) {
        if !self.set_corner_radius_tl_value(value) {
            return;
        }
        callbacks.corner_radius_tl_changed();
        LayoutComponentStyleBaseCallbacks::notify_property_changed(
            callbacks,
            Self::CORNER_RADIUS_TL_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_corner_radius_tl_value(&mut self, value: f32) -> bool {
        if self.corner_radius_tl == value {
            return false;
        }
        self.corner_radius_tl = value;
        true
    }
    pub fn corner_radius_tr(&self) -> f32 {
        self.corner_radius_tr
    }
    pub fn set_corner_radius_tr(
        &mut self,
        value: f32,
        callbacks: &mut impl LayoutComponentStyleBaseCallbacks,
    ) {
        if !self.set_corner_radius_tr_value(value) {
            return;
        }
        callbacks.corner_radius_tr_changed();
        LayoutComponentStyleBaseCallbacks::notify_property_changed(
            callbacks,
            Self::CORNER_RADIUS_TR_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_corner_radius_tr_value(&mut self, value: f32) -> bool {
        if self.corner_radius_tr == value {
            return false;
        }
        self.corner_radius_tr = value;
        true
    }
    pub fn corner_radius_bl(&self) -> f32 {
        self.corner_radius_bl
    }
    pub fn set_corner_radius_bl(
        &mut self,
        value: f32,
        callbacks: &mut impl LayoutComponentStyleBaseCallbacks,
    ) {
        if !self.set_corner_radius_bl_value(value) {
            return;
        }
        callbacks.corner_radius_bl_changed();
        LayoutComponentStyleBaseCallbacks::notify_property_changed(
            callbacks,
            Self::CORNER_RADIUS_BL_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_corner_radius_bl_value(&mut self, value: f32) -> bool {
        if self.corner_radius_bl == value {
            return false;
        }
        self.corner_radius_bl = value;
        true
    }
    pub fn corner_radius_br(&self) -> f32 {
        self.corner_radius_br
    }
    pub fn set_corner_radius_br(
        &mut self,
        value: f32,
        callbacks: &mut impl LayoutComponentStyleBaseCallbacks,
    ) {
        if !self.set_corner_radius_br_value(value) {
            return;
        }
        callbacks.corner_radius_br_changed();
        LayoutComponentStyleBaseCallbacks::notify_property_changed(
            callbacks,
            Self::CORNER_RADIUS_BR_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_corner_radius_br_value(&mut self, value: f32) -> bool {
        if self.corner_radius_br == value {
            return false;
        }
        self.corner_radius_br = value;
        true
    }
    pub fn clone_into(
        &self,
        callbacks: &mut impl LayoutComponentStyleBaseCallbacks,
    ) -> LayoutComponentStyle {
        let mut cloned = LayoutComponentStyle::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl LayoutComponentStyleBaseCallbacks) {
        self.gap_horizontal = object.gap_horizontal;
        self.gap_vertical = object.gap_vertical;
        self.border_left = object.border_left;
        self.border_right = object.border_right;
        self.border_top = object.border_top;
        self.border_bottom = object.border_bottom;
        self.margin_left = object.margin_left;
        self.margin_right = object.margin_right;
        self.margin_top = object.margin_top;
        self.margin_bottom = object.margin_bottom;
        self.padding_left = object.padding_left;
        self.padding_right = object.padding_right;
        self.padding_top = object.padding_top;
        self.padding_bottom = object.padding_bottom;
        self.position_left = object.position_left;
        self.position_right = object.position_right;
        self.position_top = object.position_top;
        self.position_bottom = object.position_bottom;
        self.flex_basis = object.flex_basis;
        self.aspect_ratio = object.aspect_ratio;
        self.interpolator_id = object.interpolator_id;
        self.interpolation_time = object.interpolation_time;
        self.flex_basis_units_value = object.flex_basis_units_value;
        self.layout_alignment_type = object.layout_alignment_type;
        self.animation_style_type = object.animation_style_type;
        self.interpolation_type = object.interpolation_type;
        self.position_type_value = object.position_type_value;
        self.flex_direction_value = object.flex_direction_value;
        self.direction_value = object.direction_value;
        self.flex_wrap_value = object.flex_wrap_value;
        self.overflow_value = object.overflow_value;
        self.intrinsically_sized_value = object.intrinsically_sized_value;
        self.border_left_units_value = object.border_left_units_value;
        self.border_right_units_value = object.border_right_units_value;
        self.border_top_units_value = object.border_top_units_value;
        self.border_bottom_units_value = object.border_bottom_units_value;
        self.margin_left_units_value = object.margin_left_units_value;
        self.margin_right_units_value = object.margin_right_units_value;
        self.margin_top_units_value = object.margin_top_units_value;
        self.margin_bottom_units_value = object.margin_bottom_units_value;
        self.padding_left_units_value = object.padding_left_units_value;
        self.padding_right_units_value = object.padding_right_units_value;
        self.padding_top_units_value = object.padding_top_units_value;
        self.padding_bottom_units_value = object.padding_bottom_units_value;
        self.position_left_units_value = object.position_left_units_value;
        self.position_right_units_value = object.position_right_units_value;
        self.position_top_units_value = object.position_top_units_value;
        self.position_bottom_units_value = object.position_bottom_units_value;
        self.gap_horizontal_units_value = object.gap_horizontal_units_value;
        self.gap_vertical_units_value = object.gap_vertical_units_value;
        self.link_corner_radius = object.link_corner_radius;
        self.justify_items_value = object.justify_items_value;
        self.layout_type_value = object.layout_type_value;
        self.corner_radius_tl = object.corner_radius_tl;
        self.corner_radius_tr = object.corner_radius_tr;
        self.corner_radius_bl = object.corner_radius_bl;
        self.corner_radius_br = object.corner_radius_br;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl LayoutComponentStyleBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::GAP_HORIZONTAL_PROPERTY_KEY => {
                self.gap_horizontal = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::GAP_VERTICAL_PROPERTY_KEY => {
                self.gap_vertical = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::BORDER_LEFT_PROPERTY_KEY => {
                self.border_left = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::BORDER_RIGHT_PROPERTY_KEY => {
                self.border_right = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::BORDER_TOP_PROPERTY_KEY => {
                self.border_top = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::BORDER_BOTTOM_PROPERTY_KEY => {
                self.border_bottom = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::MARGIN_LEFT_PROPERTY_KEY => {
                self.margin_left = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::MARGIN_RIGHT_PROPERTY_KEY => {
                self.margin_right = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::MARGIN_TOP_PROPERTY_KEY => {
                self.margin_top = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::MARGIN_BOTTOM_PROPERTY_KEY => {
                self.margin_bottom = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::PADDING_LEFT_PROPERTY_KEY => {
                self.padding_left = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::PADDING_RIGHT_PROPERTY_KEY => {
                self.padding_right = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::PADDING_TOP_PROPERTY_KEY => {
                self.padding_top = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::PADDING_BOTTOM_PROPERTY_KEY => {
                self.padding_bottom = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::POSITION_LEFT_PROPERTY_KEY => {
                self.position_left = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::POSITION_RIGHT_PROPERTY_KEY => {
                self.position_right = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::POSITION_TOP_PROPERTY_KEY => {
                self.position_top = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::POSITION_BOTTOM_PROPERTY_KEY => {
                self.position_bottom = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::FLEX_BASIS_PROPERTY_KEY => {
                self.flex_basis = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::ASPECT_RATIO_PROPERTY_KEY => {
                self.aspect_ratio = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::INTERPOLATOR_ID_PROPERTY_KEY => {
                self.interpolator_id = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            Self::INTERPOLATION_TIME_PROPERTY_KEY => {
                self.interpolation_time = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::FLEX_BASIS_UNITS_VALUE_PROPERTY_KEY => {
                self.flex_basis_units_value = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader) as u8;
                true
            }
            Self::LAYOUT_ALIGNMENT_TYPE_PROPERTY_KEY => {
                self.layout_alignment_type = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader) as u8;
                true
            }
            Self::ANIMATION_STYLE_TYPE_PROPERTY_KEY => {
                self.animation_style_type = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader) as u8;
                true
            }
            Self::INTERPOLATION_TYPE_PROPERTY_KEY => {
                self.interpolation_type = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader) as u8;
                true
            }
            Self::POSITION_TYPE_VALUE_PROPERTY_KEY => {
                self.position_type_value = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader) as u8;
                true
            }
            Self::FLEX_DIRECTION_VALUE_PROPERTY_KEY => {
                self.flex_direction_value = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader) as u8;
                true
            }
            Self::DIRECTION_VALUE_PROPERTY_KEY => {
                self.direction_value = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader) as u8;
                true
            }
            Self::FLEX_WRAP_VALUE_PROPERTY_KEY => {
                self.flex_wrap_value = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader) as u8;
                true
            }
            Self::OVERFLOW_VALUE_PROPERTY_KEY => {
                self.overflow_value = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader) as u8;
                true
            }
            Self::INTRINSICALLY_SIZED_VALUE_PROPERTY_KEY => {
                self.intrinsically_sized_value = crate::mechanical_port::source::core::field_types::core_bool_type::CoreBoolType::deserialize(reader);
                true
            }
            Self::BORDER_LEFT_UNITS_VALUE_PROPERTY_KEY => {
                self.border_left_units_value = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader) as u8;
                true
            }
            Self::BORDER_RIGHT_UNITS_VALUE_PROPERTY_KEY => {
                self.border_right_units_value = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader) as u8;
                true
            }
            Self::BORDER_TOP_UNITS_VALUE_PROPERTY_KEY => {
                self.border_top_units_value = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader) as u8;
                true
            }
            Self::BORDER_BOTTOM_UNITS_VALUE_PROPERTY_KEY => {
                self.border_bottom_units_value = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader) as u8;
                true
            }
            Self::MARGIN_LEFT_UNITS_VALUE_PROPERTY_KEY => {
                self.margin_left_units_value = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader) as u8;
                true
            }
            Self::MARGIN_RIGHT_UNITS_VALUE_PROPERTY_KEY => {
                self.margin_right_units_value = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader) as u8;
                true
            }
            Self::MARGIN_TOP_UNITS_VALUE_PROPERTY_KEY => {
                self.margin_top_units_value = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader) as u8;
                true
            }
            Self::MARGIN_BOTTOM_UNITS_VALUE_PROPERTY_KEY => {
                self.margin_bottom_units_value = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader) as u8;
                true
            }
            Self::PADDING_LEFT_UNITS_VALUE_PROPERTY_KEY => {
                self.padding_left_units_value = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader) as u8;
                true
            }
            Self::PADDING_RIGHT_UNITS_VALUE_PROPERTY_KEY => {
                self.padding_right_units_value = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader) as u8;
                true
            }
            Self::PADDING_TOP_UNITS_VALUE_PROPERTY_KEY => {
                self.padding_top_units_value = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader) as u8;
                true
            }
            Self::PADDING_BOTTOM_UNITS_VALUE_PROPERTY_KEY => {
                self.padding_bottom_units_value = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader) as u8;
                true
            }
            Self::POSITION_LEFT_UNITS_VALUE_PROPERTY_KEY => {
                self.position_left_units_value = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader) as u8;
                true
            }
            Self::POSITION_RIGHT_UNITS_VALUE_PROPERTY_KEY => {
                self.position_right_units_value = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader) as u8;
                true
            }
            Self::POSITION_TOP_UNITS_VALUE_PROPERTY_KEY => {
                self.position_top_units_value = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader) as u8;
                true
            }
            Self::POSITION_BOTTOM_UNITS_VALUE_PROPERTY_KEY => {
                self.position_bottom_units_value = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader) as u8;
                true
            }
            Self::GAP_HORIZONTAL_UNITS_VALUE_PROPERTY_KEY => {
                self.gap_horizontal_units_value = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader) as u8;
                true
            }
            Self::GAP_VERTICAL_UNITS_VALUE_PROPERTY_KEY => {
                self.gap_vertical_units_value = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader) as u8;
                true
            }
            Self::LINK_CORNER_RADIUS_PROPERTY_KEY => {
                self.link_corner_radius = crate::mechanical_port::source::core::field_types::core_bool_type::CoreBoolType::deserialize(reader);
                true
            }
            Self::JUSTIFY_ITEMS_VALUE_PROPERTY_KEY => {
                self.justify_items_value = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader) as u8;
                true
            }
            Self::LAYOUT_TYPE_VALUE_PROPERTY_KEY => {
                self.layout_type_value = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader) as u8;
                true
            }
            Self::CORNER_RADIUS_TL_PROPERTY_KEY => {
                self.corner_radius_tl = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::CORNER_RADIUS_TR_PROPERTY_KEY => {
                self.corner_radius_tr = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::CORNER_RADIUS_BL_PROPERTY_KEY => {
                self.corner_radius_bl = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::CORNER_RADIUS_BR_PROPERTY_KEY => {
                self.corner_radius_br = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}

impl std::ops::Deref for LayoutComponentStyleBase {
    type Target = LayoutSizingStyle;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for LayoutComponentStyleBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
