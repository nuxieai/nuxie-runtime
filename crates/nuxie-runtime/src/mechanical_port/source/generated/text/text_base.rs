use crate::mechanical_port::source::{
    core::binary_reader::BinaryReader, drawable::Drawable, text::text::Text,
};

pub trait TextBaseCallbacks:
    crate::mechanical_port::source::generated::drawable_base::DrawableBaseCallbacks
{
    fn notify_property_changed(&mut self, property_key: u16);
    fn align_value_changed(&mut self) {}
    fn sizing_value_changed(&mut self) {}
    fn overflow_value_changed(&mut self) {}
    fn width_changed(&mut self) {}
    fn height_changed(&mut self) {}
    fn origin_x_changed(&mut self) {}
    fn origin_y_changed(&mut self) {}
    fn paragraph_spacing_changed(&mut self) {}
    fn origin_value_changed(&mut self) {}
    fn wrap_value_changed(&mut self) {}
    fn vertical_align_value_changed(&mut self) {}
    fn fit_from_baseline_changed(&mut self) {}
    fn text_run_list_source_changed(&mut self) {}
    fn vertical_trim_value_changed(&mut self) {}
}

pub struct TextBase {
    pub base: Drawable,
    align_value: u32,
    sizing_value: u32,
    overflow_value: u32,
    width: f32,
    height: f32,
    origin_x: f32,
    origin_y: f32,
    paragraph_spacing: f32,
    origin_value: u32,
    wrap_value: u32,
    vertical_align_value: u32,
    fit_from_baseline: bool,
    text_run_list_source: u32,
    vertical_trim_value: u32,
}

impl Default for TextBase {
    fn default() -> Self {
        Self {
            base: Drawable::default(),
            align_value: 0,
            sizing_value: 0,
            overflow_value: 0,
            width: 0.0,
            height: 0.0,
            origin_x: 0.0,
            origin_y: 0.0,
            paragraph_spacing: 0.0,
            origin_value: 0,
            wrap_value: 0,
            vertical_align_value: 0,
            fit_from_baseline: true,
            text_run_list_source: u32::MAX,
            vertical_trim_value: 0,
        }
    }
}

impl TextBase {
    pub const TYPE_KEY: u16 = 134;
    pub const ALIGN_VALUE_PROPERTY_KEY: u16 = 281;
    pub const SIZING_VALUE_PROPERTY_KEY: u16 = 284;
    pub const OVERFLOW_VALUE_PROPERTY_KEY: u16 = 287;
    pub const WIDTH_PROPERTY_KEY: u16 = 285;
    pub const HEIGHT_PROPERTY_KEY: u16 = 286;
    pub const ORIGIN_X_PROPERTY_KEY: u16 = 366;
    pub const ORIGIN_Y_PROPERTY_KEY: u16 = 367;
    pub const PARAGRAPH_SPACING_PROPERTY_KEY: u16 = 371;
    pub const ORIGIN_VALUE_PROPERTY_KEY: u16 = 377;
    pub const WRAP_VALUE_PROPERTY_KEY: u16 = 683;
    pub const VERTICAL_ALIGN_VALUE_PROPERTY_KEY: u16 = 685;
    pub const FIT_FROM_BASELINE_PROPERTY_KEY: u16 = 703;
    pub const TEXT_RUN_LIST_SOURCE_PROPERTY_KEY: u16 = 932;
    pub const VERTICAL_TRIM_VALUE_PROPERTY_KEY: u16 = 1026;
    pub const VERTICAL_TRIM_TOP_VALUE_PROPERTY_KEY: u16 = 1027;
    pub const VERTICAL_TRIM_TOP_VALUE_BIT_OFFSET: u32 = 0;
    pub const VERTICAL_TRIM_TOP_VALUE_FIELD_MASK: u32 = 255;
    pub const VERTICAL_TRIM_BOTTOM_VALUE_PROPERTY_KEY: u16 = 1028;
    pub const VERTICAL_TRIM_BOTTOM_VALUE_BIT_OFFSET: u32 = 8;
    pub const VERTICAL_TRIM_BOTTOM_VALUE_FIELD_MASK: u32 = 65280;

    pub fn is_type_of(type_key: u16) -> bool {
        matches!(type_key, Self::TYPE_KEY | 13 | 2 | 38 | 91 | 11 | 10)
    }
    pub fn core_type(&self) -> u16 {
        Self::TYPE_KEY
    }
    pub fn align_value(&self) -> u32 {
        self.align_value
    }
    pub fn set_align_value(&mut self, value: u32, callbacks: &mut impl TextBaseCallbacks) {
        if !self.set_align_value_value(value) {
            return;
        }
        callbacks.align_value_changed();
        TextBaseCallbacks::notify_property_changed(callbacks, Self::ALIGN_VALUE_PROPERTY_KEY);
    }

    pub(crate) fn set_align_value_value(&mut self, value: u32) -> bool {
        if self.align_value == value {
            return false;
        }
        self.align_value = value;
        true
    }
    pub fn sizing_value(&self) -> u32 {
        self.sizing_value
    }
    pub fn set_sizing_value(&mut self, value: u32, callbacks: &mut impl TextBaseCallbacks) {
        if !self.set_sizing_value_value(value) {
            return;
        }
        callbacks.sizing_value_changed();
        TextBaseCallbacks::notify_property_changed(callbacks, Self::SIZING_VALUE_PROPERTY_KEY);
    }

    pub(crate) fn set_sizing_value_value(&mut self, value: u32) -> bool {
        if self.sizing_value == value {
            return false;
        }
        self.sizing_value = value;
        true
    }
    pub fn overflow_value(&self) -> u32 {
        self.overflow_value
    }
    pub fn set_overflow_value(&mut self, value: u32, callbacks: &mut impl TextBaseCallbacks) {
        if !self.set_overflow_value_value(value) {
            return;
        }
        callbacks.overflow_value_changed();
        TextBaseCallbacks::notify_property_changed(callbacks, Self::OVERFLOW_VALUE_PROPERTY_KEY);
    }

    pub(crate) fn set_overflow_value_value(&mut self, value: u32) -> bool {
        if self.overflow_value == value {
            return false;
        }
        self.overflow_value = value;
        true
    }
    pub fn width(&self) -> f32 {
        self.width
    }
    pub fn set_width(&mut self, value: f32, callbacks: &mut impl TextBaseCallbacks) {
        if !self.set_width_value(value) {
            return;
        }
        callbacks.width_changed();
        TextBaseCallbacks::notify_property_changed(callbacks, Self::WIDTH_PROPERTY_KEY);
    }

    pub(crate) fn set_width_value(&mut self, value: f32) -> bool {
        if self.width == value {
            return false;
        }
        self.width = value;
        true
    }
    pub fn height(&self) -> f32 {
        self.height
    }
    pub fn set_height(&mut self, value: f32, callbacks: &mut impl TextBaseCallbacks) {
        if !self.set_height_value(value) {
            return;
        }
        callbacks.height_changed();
        TextBaseCallbacks::notify_property_changed(callbacks, Self::HEIGHT_PROPERTY_KEY);
    }

    pub(crate) fn set_height_value(&mut self, value: f32) -> bool {
        if self.height == value {
            return false;
        }
        self.height = value;
        true
    }
    pub fn origin_x(&self) -> f32 {
        self.origin_x
    }
    pub fn set_origin_x(&mut self, value: f32, callbacks: &mut impl TextBaseCallbacks) {
        if !self.set_origin_x_value(value) {
            return;
        }
        callbacks.origin_x_changed();
        TextBaseCallbacks::notify_property_changed(callbacks, Self::ORIGIN_X_PROPERTY_KEY);
    }

    pub(crate) fn set_origin_x_value(&mut self, value: f32) -> bool {
        if self.origin_x == value {
            return false;
        }
        self.origin_x = value;
        true
    }
    pub fn origin_y(&self) -> f32 {
        self.origin_y
    }
    pub fn set_origin_y(&mut self, value: f32, callbacks: &mut impl TextBaseCallbacks) {
        if !self.set_origin_y_value(value) {
            return;
        }
        callbacks.origin_y_changed();
        TextBaseCallbacks::notify_property_changed(callbacks, Self::ORIGIN_Y_PROPERTY_KEY);
    }

    pub(crate) fn set_origin_y_value(&mut self, value: f32) -> bool {
        if self.origin_y == value {
            return false;
        }
        self.origin_y = value;
        true
    }
    pub fn paragraph_spacing(&self) -> f32 {
        self.paragraph_spacing
    }
    pub fn set_paragraph_spacing(&mut self, value: f32, callbacks: &mut impl TextBaseCallbacks) {
        if !self.set_paragraph_spacing_value(value) {
            return;
        }
        callbacks.paragraph_spacing_changed();
        TextBaseCallbacks::notify_property_changed(callbacks, Self::PARAGRAPH_SPACING_PROPERTY_KEY);
    }

    pub(crate) fn set_paragraph_spacing_value(&mut self, value: f32) -> bool {
        if self.paragraph_spacing == value {
            return false;
        }
        self.paragraph_spacing = value;
        true
    }
    pub fn origin_value(&self) -> u32 {
        self.origin_value
    }
    pub fn set_origin_value(&mut self, value: u32, callbacks: &mut impl TextBaseCallbacks) {
        if !self.set_origin_value_value(value) {
            return;
        }
        callbacks.origin_value_changed();
        TextBaseCallbacks::notify_property_changed(callbacks, Self::ORIGIN_VALUE_PROPERTY_KEY);
    }

    pub(crate) fn set_origin_value_value(&mut self, value: u32) -> bool {
        if self.origin_value == value {
            return false;
        }
        self.origin_value = value;
        true
    }
    pub fn wrap_value(&self) -> u32 {
        self.wrap_value
    }
    pub fn set_wrap_value(&mut self, value: u32, callbacks: &mut impl TextBaseCallbacks) {
        if !self.set_wrap_value_value(value) {
            return;
        }
        callbacks.wrap_value_changed();
        TextBaseCallbacks::notify_property_changed(callbacks, Self::WRAP_VALUE_PROPERTY_KEY);
    }

    pub(crate) fn set_wrap_value_value(&mut self, value: u32) -> bool {
        if self.wrap_value == value {
            return false;
        }
        self.wrap_value = value;
        true
    }
    pub fn vertical_align_value(&self) -> u32 {
        self.vertical_align_value
    }
    pub fn set_vertical_align_value(&mut self, value: u32, callbacks: &mut impl TextBaseCallbacks) {
        if !self.set_vertical_align_value_value(value) {
            return;
        }
        callbacks.vertical_align_value_changed();
        TextBaseCallbacks::notify_property_changed(
            callbacks,
            Self::VERTICAL_ALIGN_VALUE_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_vertical_align_value_value(&mut self, value: u32) -> bool {
        if self.vertical_align_value == value {
            return false;
        }
        self.vertical_align_value = value;
        true
    }
    pub fn fit_from_baseline(&self) -> bool {
        self.fit_from_baseline
    }
    pub fn set_fit_from_baseline(&mut self, value: bool, callbacks: &mut impl TextBaseCallbacks) {
        if !self.set_fit_from_baseline_value(value) {
            return;
        }
        callbacks.fit_from_baseline_changed();
        TextBaseCallbacks::notify_property_changed(callbacks, Self::FIT_FROM_BASELINE_PROPERTY_KEY);
    }

    pub(crate) fn set_fit_from_baseline_value(&mut self, value: bool) -> bool {
        if self.fit_from_baseline == value {
            return false;
        }
        self.fit_from_baseline = value;
        true
    }
    pub fn text_run_list_source(&self) -> u32 {
        self.text_run_list_source
    }
    pub fn set_text_run_list_source(&mut self, value: u32, callbacks: &mut impl TextBaseCallbacks) {
        if !self.set_text_run_list_source_value(value) {
            return;
        }
        callbacks.text_run_list_source_changed();
        TextBaseCallbacks::notify_property_changed(
            callbacks,
            Self::TEXT_RUN_LIST_SOURCE_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_text_run_list_source_value(&mut self, value: u32) -> bool {
        if self.text_run_list_source == value {
            return false;
        }
        self.text_run_list_source = value;
        true
    }
    pub fn vertical_trim_value(&self) -> u32 {
        self.vertical_trim_value
    }
    pub fn set_vertical_trim_value(&mut self, value: u32, callbacks: &mut impl TextBaseCallbacks) {
        if !self.set_vertical_trim_value_value(value) {
            return;
        }
        callbacks.vertical_trim_value_changed();
        TextBaseCallbacks::notify_property_changed(
            callbacks,
            Self::VERTICAL_TRIM_VALUE_PROPERTY_KEY,
        );
    }

    pub(crate) fn set_vertical_trim_value_value(&mut self, value: u32) -> bool {
        if self.vertical_trim_value == value {
            return false;
        }
        self.vertical_trim_value = value;
        true
    }
    pub fn clone_into(&self, callbacks: &mut impl TextBaseCallbacks) -> Text {
        let mut cloned = Text::default();
        cloned.base.copy(self, callbacks);
        cloned
    }
    pub fn copy(&mut self, object: &Self, callbacks: &mut impl TextBaseCallbacks) {
        self.align_value = object.align_value;
        self.sizing_value = object.sizing_value;
        self.overflow_value = object.overflow_value;
        self.width = object.width;
        self.height = object.height;
        self.origin_x = object.origin_x;
        self.origin_y = object.origin_y;
        self.paragraph_spacing = object.paragraph_spacing;
        self.origin_value = object.origin_value;
        self.wrap_value = object.wrap_value;
        self.vertical_align_value = object.vertical_align_value;
        self.fit_from_baseline = object.fit_from_baseline;
        self.text_run_list_source = object.text_run_list_source;
        self.vertical_trim_value = object.vertical_trim_value;
        self.base.copy(&object.base, callbacks);
    }
    pub fn deserialize(
        &mut self,
        property_key: u16,
        reader: &mut BinaryReader<'_>,
        callbacks: &mut impl TextBaseCallbacks,
    ) -> bool {
        match property_key {
            Self::ALIGN_VALUE_PROPERTY_KEY => {
                self.align_value = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            Self::SIZING_VALUE_PROPERTY_KEY => {
                self.sizing_value = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            Self::OVERFLOW_VALUE_PROPERTY_KEY => {
                self.overflow_value = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            Self::WIDTH_PROPERTY_KEY => {
                self.width = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::HEIGHT_PROPERTY_KEY => {
                self.height = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::ORIGIN_X_PROPERTY_KEY => {
                self.origin_x = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::ORIGIN_Y_PROPERTY_KEY => {
                self.origin_y = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::PARAGRAPH_SPACING_PROPERTY_KEY => {
                self.paragraph_spacing = crate::mechanical_port::source::core::field_types::core_double_type::CoreDoubleType::deserialize(reader);
                true
            }
            Self::ORIGIN_VALUE_PROPERTY_KEY => {
                self.origin_value = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            Self::WRAP_VALUE_PROPERTY_KEY => {
                self.wrap_value = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            Self::VERTICAL_ALIGN_VALUE_PROPERTY_KEY => {
                self.vertical_align_value = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            Self::FIT_FROM_BASELINE_PROPERTY_KEY => {
                self.fit_from_baseline = crate::mechanical_port::source::core::field_types::core_bool_type::CoreBoolType::deserialize(reader);
                true
            }
            Self::TEXT_RUN_LIST_SOURCE_PROPERTY_KEY => {
                self.text_run_list_source = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            Self::VERTICAL_TRIM_VALUE_PROPERTY_KEY => {
                self.vertical_trim_value = crate::mechanical_port::source::core::field_types::core_uint_type::CoreUintType::deserialize(reader);
                true
            }
            _ => self.base.deserialize(property_key, reader, callbacks),
        }
    }
}

impl std::ops::Deref for TextBase {
    type Target = Drawable;

    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for TextBase {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
