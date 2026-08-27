use crate::mechanical_port::source::{
    component::Component,
    shapes::paint::{
        fill::{Fill, FillBase},
        group_effect::{GroupEffect, GroupEffectBase},
        shape_paint::{ShapePaint, ShapePaintBase, ShapePaintPath},
        stroke::{Stroke, StrokeBase},
        stroke_effect::{PathProvider, StrokeEffect},
    },
};
#[derive(Default)]
pub struct EffectsContainerState {
    pub effects: Vec<*mut dyn StrokeEffect>,
}
pub trait EffectsContainer {
    fn effects_state(&mut self) -> &mut EffectsContainerState;
    fn add_stroke_effect(&mut self, effect: *mut dyn StrokeEffect) {
        self.effects_state().effects.push(effect);
    }
    fn invalidate_effects(&mut self, invalidating: Option<*mut dyn StrokeEffect>) {
        let mut found = invalidating.is_none();
        for effect in self.effects_state().effects.iter().copied() {
            if found {
                unsafe { (*effect).invalidate_effect(None) };
            }
            if invalidating.is_some_and(|value| std::ptr::addr_eq(value, effect)) {
                found = true;
            }
        }
    }
    fn last_effect_path(&mut self, provider: &mut PathProvider) -> Option<&mut ShapePaintPath> {
        for effect in self.effects_state().effects.iter().rev().copied() {
            if let Some(path) = unsafe { (*effect).effect_path(provider) } {
                return Some(path);
            }
        }
        None
    }
}
pub fn from(component: &mut Component) -> Option<&mut dyn EffectsContainer> {
    match component.core_type() {
        ShapePaintBase::TYPE_KEY | FillBase::TYPE_KEY | StrokeBase::TYPE_KEY => component
            .as_mut::<ShapePaint>()
            .map(|v| v as &mut dyn EffectsContainer),
        GroupEffectBase::TYPE_KEY => component
            .as_mut::<GroupEffect>()
            .map(|v| v as &mut dyn EffectsContainer),
        _ => None,
    }
}
