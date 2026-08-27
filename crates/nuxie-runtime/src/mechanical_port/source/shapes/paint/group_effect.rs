pub use crate::mechanical_port::source::generated::shapes::paint::group_effect_base::GroupEffectBase;
use crate::mechanical_port::source::shapes::paint::{
    effects_container::{EffectsContainer, EffectsContainerState},
    shape_paint::{ShapePaint, ShapePaintPath},
    stroke_effect::{PathProvider, StrokeEffect, StrokeEffectState},
    target_effect::TargetEffect,
};
pub struct GroupEffect {
    pub base: GroupEffectBase,
    effects: EffectsContainerState,
    stroke: StrokeEffectState,
    target_effects: Vec<*mut TargetEffect>,
}
impl GroupEffect {
    pub fn update_effect(
        &mut self,
        provider: &mut PathProvider,
        source: &ShapePaintPath,
        paint: &ShapePaint,
    ) {
        let mut path = source as *const _;
        for effect in self.effects.effects.iter().copied() {
            unsafe {
                (*effect).update_effect(provider, &*path, paint);
                if let Some(next) = (*effect).effect_path(provider) {
                    path = next;
                }
            }
        }
    }
    pub fn invalidate_effects_from(&mut self, effect: Option<*mut dyn StrokeEffect>) {
        for target in self.target_effects.iter().copied() {
            unsafe { (*target).invalidate_effect_from_local() };
        }
        EffectsContainer::invalidate_effects(self, effect);
    }
    pub fn add_target_effect(&mut self, effect: &mut TargetEffect) {
        self.target_effects.push(effect);
    }
    pub fn add_path_provider(&mut self, provider: &mut PathProvider) {
        StrokeEffect::add_path_provider(self, provider);
        for effect in self.effects.effects.iter().copied() {
            unsafe {
                (*effect).add_path_provider(provider);
            }
        }
    }
    pub fn add_stroke_effect_direct(&mut self, effect: *mut dyn StrokeEffect) {
        EffectsContainer::add_stroke_effect(self, effect);
        for provider in self.stroke.effect_paths.keys().copied() {
            unsafe {
                (*effect).add_path_provider(&mut *provider);
            }
        }
    }
    pub fn invalidate_effect(&mut self, provider: Option<&mut PathProvider>) {
        StrokeEffect::invalidate_effect(self, provider);
        for effect in self.effects.effects.iter().copied() {
            unsafe {
                (*effect).invalidate_effect(provider.as_deref_mut());
            }
        }
    }
    pub fn build_dependencies(&mut self) {
        self.base.build_dependencies();
        if let Some(parent) = self.base.parent_mut() {
            parent.add_dependent(self.base.as_component_mut_ptr());
        }
    }
}
impl EffectsContainer for GroupEffect {
    fn effects_state(&mut self) -> &mut EffectsContainerState {
        &mut self.effects
    }
    fn invalidate_effects(&mut self, effect: Option<*mut dyn StrokeEffect>) {
        self.invalidate_effects_from(effect);
    }
    fn add_stroke_effect(&mut self, effect: *mut dyn StrokeEffect) {
        self.add_stroke_effect_direct(effect);
    }
}
impl StrokeEffect for GroupEffect {
    fn stroke_effect_state(&mut self) -> &mut StrokeEffectState {
        &mut self.stroke
    }
    fn update_effect(&mut self, p: &mut PathProvider, s: &ShapePaintPath, paint: &ShapePaint) {
        GroupEffect::update_effect(self, p, s, paint);
    }
    fn parent_paint(&mut self) -> Option<&mut dyn EffectsContainer> {
        None
    }
}
