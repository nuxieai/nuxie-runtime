use crate::mechanical_port::source::core::CoreHandle;
pub use crate::mechanical_port::source::generated::shapes::paint::group_effect_base::GroupEffectBase;
use crate::mechanical_port::source::shapes::paint::{
    effects_container::{EffectsContainer, EffectsContainerState},
    shape_paint::ShapePaint,
    shape_paint_path::ShapePaintPath,
    stroke_effect::{PathProvider, StrokeEffect, StrokeEffectState},
    target_effect::TargetEffect,
};
pub struct GroupEffect {
    pub base: GroupEffectBase,
    effects: EffectsContainerState,
    stroke: StrokeEffectState,
    target_effects: Vec<CoreHandle>,
}
impl Default for GroupEffect {
    fn default() -> Self {
        Self {
            base: GroupEffectBase::default(),
            effects: EffectsContainerState::default(),
            stroke: StrokeEffectState::default(),
            target_effects: Vec::new(),
        }
    }
}
impl GroupEffect {
    pub fn update_effect(
        &mut self,
        provider: &PathProvider,
        source: &ShapePaintPath,
        paint: &ShapePaint,
    ) {
        let mut current = None;
        for effect in self.effects.effects.iter().cloned() {
            effect.with_mut(|effect| {
                let Some(effect) = effect.as_stroke_effect_mut() else {
                    return;
                };
                if let Some(current) = current.as_ref() {
                    effect.update_effect(provider, &current.borrow(), paint);
                } else {
                    effect.update_effect(provider, source, paint);
                }
                if let Some(next) = effect.effect_path(provider) {
                    current = Some(next);
                }
            });
        }
    }
    pub fn invalidate_effects_from(&mut self, effect: Option<&CoreHandle>) {
        for target in self.target_effects.iter() {
            target.with_downcast_mut::<TargetEffect, _>(|target| {
                target.invalidate_effect_from_local()
            });
        }
        EffectsContainer::invalidate_effects(self, effect);
    }
    pub fn add_target_effect(&mut self, effect: CoreHandle) {
        self.target_effects.push(effect);
    }
    pub fn add_path_provider(&mut self, provider: &PathProvider) {
        StrokeEffect::add_path_provider(self, provider);
        for effect in self.effects.effects.iter() {
            effect.with_mut(|effect| {
                if let Some(effect) = effect.as_stroke_effect_mut() {
                    effect.add_path_provider(provider);
                }
            });
        }
    }
    pub fn add_stroke_effect_direct(&mut self, effect: CoreHandle) {
        EffectsContainer::add_stroke_effect(self, effect.clone());
        for provider in self.stroke.effect_paths.keys().copied() {
            let provider = PathProvider::with_identity(provider);
            effect.with_mut(|effect| {
                if let Some(effect) = effect.as_stroke_effect_mut() {
                    effect.add_path_provider(&provider);
                }
            });
        }
    }
    pub fn invalidate_effect(&mut self, provider: Option<&PathProvider>) {
        StrokeEffect::invalidate_effect(self, provider);
        for effect in self.effects.effects.iter() {
            effect.with_mut(|effect| {
                if let Some(effect) = effect.as_stroke_effect_mut() {
                    effect.invalidate_effect(provider);
                }
            });
        }
    }
    pub fn build_dependencies(&mut self) {
        self.base.build_dependencies();
        if let (Some(parent), Some(this)) = (self.base.parent_handle(), self.base.handle()) {
            parent.with_mut(|parent| {
                if let Some(parent) = parent.as_component_mut() {
                    parent.add_dependent(this);
                }
            });
        }
    }
}
impl EffectsContainer for GroupEffect {
    fn effects_state(&mut self) -> &mut EffectsContainerState {
        &mut self.effects
    }
    fn invalidate_effects(&mut self, effect: Option<&CoreHandle>) {
        self.invalidate_effects_from(effect);
    }
    fn add_stroke_effect(&mut self, effect: CoreHandle) {
        self.add_stroke_effect_direct(effect);
    }
}
impl StrokeEffect for GroupEffect {
    fn stroke_effect_state(&mut self) -> &mut StrokeEffectState {
        &mut self.stroke
    }
    fn stroke_effect_handle(&self) -> Option<CoreHandle> {
        self.base.handle()
    }
    fn update_effect(&mut self, p: &PathProvider, s: &ShapePaintPath, paint: &ShapePaint) {
        GroupEffect::update_effect(self, p, s, paint);
    }
    fn parent_paint_handle(&self) -> Option<CoreHandle> {
        self.base.parent_handle()
    }
}
