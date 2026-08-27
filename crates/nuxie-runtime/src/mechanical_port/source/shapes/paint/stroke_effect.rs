use crate::mechanical_port::source::shapes::paint::{
    dash_path::DashEffectPath,
    effects_container::EffectsContainer,
    shape_paint::{ShapePaint, ShapePaintPath},
    target_effect::TargetEffectPath,
    trim_path::TrimEffectPath,
};
use std::collections::HashMap;
pub struct PathProvider {
    identity: u8,
}
impl Default for PathProvider {
    fn default() -> Self {
        Self { identity: 0 }
    }
}
pub trait EffectPath {
    fn invalidate_effect(&mut self) {}
    fn path(&mut self) -> Option<&mut ShapePaintPath> {
        None
    }
    fn as_target_mut(&mut self) -> Option<&mut TargetEffectPath> {
        None
    }
    fn as_dash_mut(&mut self) -> Option<&mut DashEffectPath> {
        None
    }
    fn as_trim_mut(&mut self) -> Option<&mut TrimEffectPath> {
        None
    }
}
pub struct EmptyEffectPath;
impl EffectPath for EmptyEffectPath {}
#[derive(Default)]
pub struct StrokeEffectState {
    pub effect_paths: HashMap<*mut PathProvider, Box<dyn EffectPath>>,
}
pub trait StrokeEffect {
    fn stroke_effect_state(&mut self) -> &mut StrokeEffectState;
    fn update_effect(
        &mut self,
        provider: &mut PathProvider,
        source: &ShapePaintPath,
        paint: &ShapePaint,
    );
    fn parent_paint(&mut self) -> Option<&mut dyn EffectsContainer>;
    fn create_effect_path(&mut self) -> Box<dyn EffectPath> {
        Box::new(EmptyEffectPath)
    }
    fn add_path_provider(&mut self, component: &mut PathProvider) {
        let path = self.create_effect_path();
        self.stroke_effect_state()
            .effect_paths
            .insert(component, path);
    }
    fn invalidate_effect(&mut self, provider: Option<&mut PathProvider>) {
        if let Some(provider) = provider {
            if let Some(path) = self
                .stroke_effect_state()
                .effect_paths
                .get_mut(&(provider as *mut _))
            {
                path.invalidate_effect();
            }
        } else {
            for path in self.stroke_effect_state().effect_paths.values_mut() {
                path.invalidate_effect();
            }
        }
    }
    fn effect_path(&mut self, provider: &mut PathProvider) -> Option<&mut ShapePaintPath> {
        self.stroke_effect_state()
            .effect_paths
            .get_mut(&(provider as *mut _))
            .and_then(|path| path.path())
    }
    fn invalidate_effect_from_local(&mut self) {
        for path in self.stroke_effect_state().effect_paths.values_mut() {
            path.invalidate_effect();
        }
        let this = self as *mut Self as *mut dyn StrokeEffect;
        if let Some(parent) = self.parent_paint() {
            parent.invalidate_effects(Some(this));
        }
    }
}
