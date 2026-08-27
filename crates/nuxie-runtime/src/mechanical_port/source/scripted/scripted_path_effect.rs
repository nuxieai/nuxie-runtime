use crate::mechanical_port::source::scripted::scripted_object::{ScriptProtocol, ScriptedObject};
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ShapePaintPath {
    pub points: Vec<(f32, f32)>,
}
pub struct ScriptedEffectPath {
    path: ShapePaintPath,
}
impl Default for ScriptedEffectPath {
    fn default() -> Self {
        Self {
            path: ShapePaintPath::default(),
        }
    }
}
impl ScriptedEffectPath {
    pub fn invalidate_effect(&mut self) {
        self.path.points.clear()
    }
    pub fn path(&mut self) -> &mut ShapePaintPath {
        &mut self.path
    }
}
#[derive(Default)]
pub struct ScriptedPathEffect {
    pub scripted: ScriptedObject,
    effect: ScriptedEffectPath,
    properties: Vec<usize>,
    advance_active: bool,
    needs_update: bool,
}
impl ScriptedPathEffect {
    pub fn did_hydrate_script_inputs(&mut self) {
        self.advance_active = true;
        self.mark_needs_update()
    }
    pub fn update_effect(&mut self, source: &ShapePaintPath) {
        self.effect.path = source.clone();
        let points = self.effect.path.points.clone();
        self.effect.path.points.clear();
        for (x, y) in points {
            let nx = self.scripted.call_number("effectX", &[x, y]).unwrap_or(x);
            let ny = self.scripted.call_number("effectY", &[x, y]).unwrap_or(y);
            self.effect.path.points.push((nx, ny));
        }
    }
    pub fn advance_component(&mut self, e: f32, animate: bool) -> bool {
        if !animate || !self.advance_active {
            return false;
        }
        self.advance_active = self.scripted.script_advance(e);
        if self.advance_active {
            self.mark_needs_update()
        }
        self.advance_active
    }
    pub fn add_scripted_dirt(&mut self) {
        self.mark_needs_update()
    }
    pub fn add_property(&mut self, p: usize) {
        self.properties.push(p)
    }
    pub fn update(&mut self) {
        self.scripted.script_update();
        self.needs_update = false
    }
    pub fn mark_needs_update(&mut self) {
        self.needs_update = true;
        self.effect.invalidate_effect()
    }
    pub fn create_effect_path(&mut self) -> &mut ScriptedEffectPath {
        &mut self.effect
    }
    pub fn script_protocol(&self) -> ScriptProtocol {
        ScriptProtocol::PathEffect
    }
}
