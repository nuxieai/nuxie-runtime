use crate::mechanical_port::source::{
    core::CoreHandle, generated::scripted::scripted_layout_base::ScriptedLayoutBase,
    scripted::scripted_object::ScriptProtocol,
};
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayoutMeasureMode {
    Undefined = 0,
    Exactly = 1,
    AtMost = 2,
}
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayoutScaleType {
    Fixed = 0,
    Fill = 1,
    Hug = 2,
}
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayoutDirection {
    Inherit = 0,
    Ltr = 1,
    Rtl = 2,
}
#[derive(Default)]
pub struct ScriptedLayout {
    pub base: ScriptedLayoutBase,
    size: Vec2,
    parent_layout_dirty: bool,
    paint_dirty: bool,
}
impl ScriptedLayout {
    pub fn did_hydrate_script_inputs(&mut self) {
        self.parent_layout_dirty = true;
        self.paint_dirty = true;
    }
    fn call_scripted_resize(&mut self, size: Vec2) {
        if self.base.base.scripted.resizes() && self.base.base.scripted.self_ref() != 0 {
            let _ = self
                .base
                .base
                .scripted
                .call_number("resize", &[size.x, size.y]);
        }
    }
    pub fn measure_layout(
        &mut self,
        width: f32,
        width_mode: LayoutMeasureMode,
        height: f32,
        height_mode: LayoutMeasureMode,
    ) -> Vec2 {
        if !self.base.base.scripted.measures() || self.base.base.scripted.self_ref() == 0 {
            return Vec2::default();
        }
        let Some((measured_width, measured_height)) =
            self.base.base.scripted.call_vec2("measure", &[])
        else {
            return Vec2::default();
        };
        Vec2 {
            x: if width_mode == LayoutMeasureMode::Undefined {
                f32::MAX
            } else {
                width
            }
            .min(measured_width),
            y: if height_mode == LayoutMeasureMode::Undefined {
                f32::MAX
            } else {
                height
            }
            .min(measured_height),
        }
    }
    pub fn control_size(
        &mut self,
        size: Vec2,
        _w: LayoutScaleType,
        _h: LayoutScaleType,
        _direction: LayoutDirection,
    ) {
        self.size = size;
        self.call_scripted_resize(size)
    }
    pub fn add_property(&mut self, p: CoreHandle) {
        self.base.base.add_property(p)
    }
    pub fn remove_property(&mut self, property: &CoreHandle) {
        self.base.base.remove_property(property)
    }
    pub fn script_protocol(&self) -> ScriptProtocol {
        ScriptProtocol::Layout
    }
}
