use crate::mechanical_port::source::{
    core::CoreHandle,
    shapes::paint::{shape_paint_path::ShapePaintPath, stroke_effect::PathProvider},
};
#[derive(Default)]
pub struct EffectsContainerState {
    pub effects: Vec<CoreHandle>,
}
pub trait EffectsContainer {
    fn effects_state(&mut self) -> &mut EffectsContainerState;
    fn add_stroke_effect(&mut self, effect: CoreHandle) {
        self.effects_state().effects.push(effect);
    }
    fn invalidate_effects(&mut self, invalidating: Option<&CoreHandle>) {
        let mut found = invalidating.is_none();
        for effect in self.effects_state().effects.iter().cloned() {
            if found {
                effect.with_mut(|effect| {
                    if let Some(effect) = effect.as_stroke_effect_mut() {
                        effect.invalidate_effect(None);
                    }
                });
            }
            if invalidating == Some(&effect) {
                found = true;
            }
        }
    }
    fn last_effect_path(
        &mut self,
        provider: &PathProvider,
    ) -> Option<std::rc::Rc<std::cell::RefCell<ShapePaintPath>>> {
        for effect in self.effects_state().effects.iter().rev() {
            if let Some(path) = effect
                .with_mut(|effect| {
                    effect
                        .as_stroke_effect_mut()
                        .and_then(|effect| effect.effect_path(provider))
                })
                .flatten()
            {
                return Some(path);
            }
        }
        None
    }
}
