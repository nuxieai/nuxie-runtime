use crate::mechanical_port::source::scripted::scripted_object::{ScriptProtocol, ScriptedObject};
use std::collections::HashMap;
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ShapePaintPath {
    pub points: Vec<(f32, f32)>,
}
pub struct ScriptedEffectPath {
    path: ShapePaintPath,
    valid: bool,
}
impl Default for ScriptedEffectPath {
    fn default() -> Self {
        Self {
            path: ShapePaintPath::default(),
            valid: false,
        }
    }
}
impl ScriptedEffectPath {
    pub fn invalidate_effect(&mut self) {
        self.path.points.clear();
        self.valid = false;
    }
    pub fn path(&mut self) -> &mut ShapePaintPath {
        &mut self.path
    }
}
pub struct ScriptedPathEffect {
    pub scripted: ScriptedObject,
    effect_paths: HashMap<usize, ScriptedEffectPath>,
    properties: Vec<usize>,
    advance_active: bool,
    needs_update: bool,
    paint_dirty: bool,
}
impl Default for ScriptedPathEffect {
    fn default() -> Self {
        Self {
            scripted: ScriptedObject::default(),
            effect_paths: HashMap::new(),
            properties: Vec::new(),
            advance_active: true,
            needs_update: false,
            paint_dirty: false,
        }
    }
}
impl ScriptedPathEffect {
    pub fn did_hydrate_script_inputs(&mut self) {
        self.advance_active = true;
        self.paint_dirty = true;
    }
    pub fn update_effect(&mut self, path_provider: usize, source: &ShapePaintPath) {
        if !self.scripted.updates() {
            return;
        }
        let effect = self.effect_paths.entry(path_provider).or_default();
        if effect.valid {
            return;
        }
        self.scripted.set_in_update_phase(true);
        if let Some(points) = self.scripted.call_path("update", &source.points) {
            effect.path.points = points;
        }
        effect.valid = true;
        self.scripted.set_in_update_phase(false);
    }
    pub fn advance_component(&mut self, mut e: f32, advance_nested: bool) -> bool {
        if e == 0.0 || !self.advance_active {
            return false;
        }
        self.advance_active = false;
        if !advance_nested {
            e = 0.0;
        }
        let advanced = self.scripted.script_advance(e);
        if advanced {
            self.advance_active = true;
        }
        advanced
    }
    pub fn add_scripted_dirt(&mut self) {
        self.mark_needs_update()
    }
    pub fn add_property(&mut self, p: usize) {
        self.properties.push(p)
    }
    pub fn update(&mut self, script_update_dirty: bool) {
        if script_update_dirty {
            for effect in self.effect_paths.values_mut() {
                effect.invalidate_effect();
            }
            self.advance_active = true;
        }
        self.needs_update = false
    }
    pub fn mark_needs_update(&mut self) {
        if self.scripted.in_update_phase() {
            return;
        }
        self.needs_update = true;
    }
    pub fn create_effect_path(&self) -> ScriptedEffectPath {
        ScriptedEffectPath::default()
    }
    pub fn script_protocol(&self) -> ScriptProtocol {
        ScriptProtocol::PathEffect
    }
}
