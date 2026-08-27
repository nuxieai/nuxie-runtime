use crate::mechanical_port::source::scripted::scripted_object::{ScriptProtocol, ScriptedObject};
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Vec2 {
    pub x: f32,
    pub y: f32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayoutMeasureMode {
    Undefined,
    Exactly,
    AtMost,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayoutScaleType {
    Fixed,
    Hug,
    Fill,
}
#[derive(Default)]
pub struct ScriptedLayout {
    pub scripted: ScriptedObject,
    size: Vec2,
    pub properties: Vec<usize>,
}
impl ScriptedLayout {
    pub fn did_hydrate_script_inputs(&mut self) {
        self.call_scripted_resize(self.size)
    }
    fn call_scripted_resize(&mut self, size: Vec2) {
        let _ = self.scripted.call_number("resize", &[size.x, size.y]);
    }
    pub fn measure_layout(
        &mut self,
        width: f32,
        width_mode: LayoutMeasureMode,
        height: f32,
        height_mode: LayoutMeasureMode,
    ) -> Vec2 {
        let wm = match width_mode {
            LayoutMeasureMode::Undefined => 0.0,
            LayoutMeasureMode::Exactly => 1.0,
            LayoutMeasureMode::AtMost => 2.0,
        };
        let hm = match height_mode {
            LayoutMeasureMode::Undefined => 0.0,
            LayoutMeasureMode::Exactly => 1.0,
            LayoutMeasureMode::AtMost => 2.0,
        };
        Vec2 {
            x: self
                .scripted
                .call_number("measureWidth", &[width, wm, height, hm])
                .unwrap_or(width),
            y: self
                .scripted
                .call_number("measureHeight", &[width, wm, height, hm])
                .unwrap_or(height),
        }
    }
    pub fn control_size(
        &mut self,
        size: Vec2,
        _w: LayoutScaleType,
        _h: LayoutScaleType,
        _rtl: bool,
    ) {
        if self.size != size {
            self.size = size;
            self.call_scripted_resize(size)
        }
    }
    pub fn add_property(&mut self, p: usize) {
        self.properties.push(p)
    }
    pub fn script_protocol(&self) -> ScriptProtocol {
        ScriptProtocol::Layout
    }
}
