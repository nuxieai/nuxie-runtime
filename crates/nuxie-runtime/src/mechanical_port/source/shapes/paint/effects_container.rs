use crate::mechanical_port::source::{
    core::CoreHandle,
    shapes::paint::{
        group_effect::GroupEffect,
        shape_paint_path::ShapePaintPath,
        stroke_effect::{PathProvider, StrokeEffect},
        target_effect::TargetEffect,
    },
};
#[derive(Default)]
pub struct EffectsContainerState {
    pub effects: Vec<CoreHandle>,
}
impl EffectsContainerState {
    pub fn add_stroke_effect(&mut self, effect: CoreHandle) {
        self.effects.push(effect);
    }
    pub fn invalidate_effects(&mut self, invalidating: Option<&CoreHandle>) {
        let mut found = invalidating.is_none();
        for effect in self.effects.iter().cloned() {
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
}

pub(crate) struct ActiveStrokeEffect<'a> {
    identity: CoreHandle,
    effect: &'a mut dyn StrokeEffect,
}

impl<'a> ActiveStrokeEffect<'a> {
    pub(crate) fn new(identity: CoreHandle, effect: &'a mut dyn StrokeEffect) -> Self {
        Self { identity, effect }
    }
}

pub(crate) fn invalidate_effect_handle_with_active(
    effect: &CoreHandle,
    provider: Option<PathProvider>,
    active: &mut Option<ActiveStrokeEffect<'_>>,
) {
    if active
        .as_ref()
        .is_some_and(|active| active.identity == *effect)
    {
        active
            .as_mut()
            .expect("checked active stroke effect")
            .effect
            .invalidate_effect(provider.as_ref());
        return;
    }
    if effect.is_type_of(TargetEffect::TYPE_KEY) {
        TargetEffect::invalidate_effect_handle_with_active(effect, provider, active);
    } else if effect.is_type_of(GroupEffect::TYPE_KEY) {
        GroupEffect::invalidate_effect_handle_with_active(effect, provider, active);
    } else {
        effect.with_mut(|effect| {
            if let Some(effect) = effect.as_stroke_effect_mut() {
                effect.invalidate_effect(provider.as_ref());
            }
        });
    }
}

/// Run upstream's synchronous invalidation order without retaining a Rust
/// owner borrow across callbacks that may legally re-enter the effect graph.
pub(crate) fn invalidate_effects_handle(container: &CoreHandle, invalidating: Option<CoreHandle>) {
    invalidate_effects_handle_with_active(container, invalidating, &mut None);
}

pub(crate) fn invalidate_effects_handle_with_active(
    container: &CoreHandle,
    invalidating: Option<CoreHandle>,
    active: &mut Option<ActiveStrokeEffect<'_>>,
) {
    if container.is_type_of(GroupEffect::TYPE_KEY) {
        let targets = container
            .with_downcast::<GroupEffect, _>(GroupEffect::target_effect_handles)
            .unwrap_or_default();
        for target in targets {
            TargetEffect::invalidate_effect_from_handle_with_active(&target, active);
        }
    }

    let effects = container
        .with_mut(|container| {
            container
                .as_effects_container_mut()
                .map(|container| container.effects_state().effects.clone())
        })
        .flatten()
        .unwrap_or_default();
    let mut found = invalidating.is_none();
    for effect in effects {
        if found {
            invalidate_effect_handle_with_active(&effect, None, active);
        }
        if invalidating.as_ref() == Some(&effect) {
            found = true;
        }
    }

    container.with_mut(|container| {
        if let Some(paint) = container.as_shape_paint_mut() {
            paint.finish_invalidate_effects();
        }
    });
}

pub trait EffectsContainer {
    fn effects_state(&mut self) -> &mut EffectsContainerState;
    // The live effect is already borrowed by its lifecycle callback. Retain
    // its occurrence identity without resolving and reborrowing that owner.
    fn add_stroke_effect(&mut self, identity: CoreHandle, _effect: &mut dyn StrokeEffect) {
        self.effects_state().add_stroke_effect(identity);
    }
    fn invalidate_effects(&mut self, invalidating: Option<&CoreHandle>) {
        self.effects_state().invalidate_effects(invalidating);
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
