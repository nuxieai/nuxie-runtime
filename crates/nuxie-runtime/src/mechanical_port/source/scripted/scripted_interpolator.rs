use crate::mechanical_port::source::scripted::scripted_object::{ScriptProtocol, ScriptedObject};
#[derive(Default)]
pub struct ScriptedInterpolator {
    pub scripted: ScriptedObject,
    pub properties: Vec<usize>,
}
impl ScriptedInterpolator {
    pub fn transform(&mut self, factor: f32) -> f32 {
        self.scripted
            .call_number("interpolate", &[factor])
            .unwrap_or(factor)
    }
    pub fn transform_value(&mut self, from: f32, to: f32, factor: f32) -> f32 {
        self.scripted
            .call_number("interpolateValue", &[from, to, factor])
            .unwrap_or(from + (to - from) * factor)
    }
    pub fn add_property(&mut self, p: usize) {
        self.properties.push(p)
    }
    pub fn script_protocol(&self) -> ScriptProtocol {
        ScriptProtocol::Interpolator
    }
}
