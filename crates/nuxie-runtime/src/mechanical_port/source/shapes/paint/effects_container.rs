use crate::mechanical_port::source::{
    component::{Component, ComponentOccurrenceHandle, DependencySnapshot},
    component_dirt::ComponentDirt,
    core::{CoreHandle, CoreObject},
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
    pub fn has_effects(&self) -> bool {
        !self.effects.is_empty()
    }

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

// C++ may recursively dirty the effect whose callback is already on the stack.
// Use that actual Rust owner while preserving onDirty, artboard notification,
// and depth-first dependent order; never reacquire its borrowed Core slot.
fn add_dirt_with_active(
    occurrence: &ComponentOccurrenceHandle,
    value: ComponentDirt,
    active: &mut Option<ActiveStrokeEffect<'_>>,
) {
    let dependents = if let Some(owner) = active
        .as_mut()
        .filter(|owner| occurrence.authored() == Some(&owner.identity))
    {
        if !owner.effect.component_add_dirt(value, false) {
            return;
        }
        owner
            .effect
            .as_component()
            .expect("active stroke effect Component")
            .dependents_snapshot()
    } else {
        if !occurrence.add_dirt(value, false) {
            return;
        }
        occurrence
            .with_component(Component::dependents_snapshot)
            .unwrap_or_default()
    };
    for dependent in dependents {
        add_dirt_with_active(&dependent, value, active);
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

fn invalidate_paint_effects(container: &mut dyn CoreObject) -> Option<DependencySnapshot> {
    let paint = container.as_shape_paint_mut()?;
    paint.invalidate_effect_feather();
    // Finish this paint's synchronous dirty callbacks before releasing
    // it to traverse dependents, including the active source effect.
    if !paint.base.add_dirt(ComponentDirt::PATH, false) {
        return None;
    }
    Some(paint.base.dependents_snapshot())
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

    let (effects, mut dependents) = container
        .with_mut(|container| {
            let effects = container
                .as_effects_container_mut()
                .map(|container| container.effects_state().effects.clone())
                .unwrap_or_default();
            // With no effects there are no intervening callbacks. Finish
            // the paint tail using this already checked owner borrow.
            let dependents = if effects.is_empty() {
                invalidate_paint_effects(container)
            } else {
                None
            };
            (effects, dependents)
        })
        .unwrap_or_default();
    let has_effects = !effects.is_empty();
    let mut found = invalidating.is_none();
    for effect in effects {
        if found {
            invalidate_effect_handle_with_active(&effect, None, active);
        }
        if invalidating.as_ref() == Some(&effect) {
            found = true;
        }
    }

    if has_effects {
        // Effect callbacks may re-enter or change owners. Revalidate and
        // borrow the paint only after all of those callbacks have finished.
        dependents = container.with_mut(invalidate_paint_effects).flatten();
    }
    if let Some(dependents) = dependents {
        for dependent in dependents {
            add_dirt_with_active(&dependent, ComponentDirt::PATH, active);
        }
    }
}

pub trait EffectsContainer {
    fn effects_state(&mut self) -> &mut EffectsContainerState;
    fn has_effects(&mut self) -> bool {
        self.effects_state().has_effects()
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mechanical_port::source::{core::CoreArena, node::Node, shapes::paint::fill::Fill};

    #[test]
    fn empty_effects_still_dirty_paint_and_dependents_after_releasing_owner() {
        let arena = CoreArena::default();
        let paint = arena.insert(Fill::default());
        let dependent = arena.insert(Node::default());
        for handle in [&paint, &dependent] {
            handle.with_mut(|object| {
                object
                    .as_component_mut()
                    .unwrap()
                    .set_dirt(ComponentDirt::NONE)
            });
        }
        paint.with_mut(|object| {
            let component = object.as_component_mut().unwrap();
            // Re-entering the paint as a dependent must happen after its
            // invalidation borrow is released, even on the empty-effects lane.
            component.add_dependent(paint.clone());
            component.add_dependent(dependent.clone());
        });
        invalidate_effects_handle(&paint, None);
        for handle in [&paint, &dependent] {
            assert!(handle
                .with(|object| object
                    .as_component()
                    .unwrap()
                    .dirt()
                    .contains(ComponentDirt::PATH))
                .unwrap());
        }
        // Already-dirty paints must not propagate dirt again.
        dependent.with_mut(|object| {
            object
                .as_component_mut()
                .unwrap()
                .set_dirt(ComponentDirt::NONE)
        });
        invalidate_effects_handle(&paint, Some(dependent.clone()));
        assert_eq!(
            dependent.with(|object| object.as_component().unwrap().dirt()),
            Some(ComponentDirt::NONE)
        );
    }

    #[test]
    fn empty_effects_stale_handle_does_not_invalidate_replacement() {
        let arena = CoreArena::default();
        let stale = arena.insert(Fill::default());
        drop(arena.remove(&stale));
        let replacement = arena.insert(Fill::default());
        replacement.with_mut(|object| {
            object
                .as_component_mut()
                .unwrap()
                .set_dirt(ComponentDirt::NONE)
        });
        invalidate_effects_handle(&stale, None);
        assert_eq!(
            replacement.with(|object| object.as_component().unwrap().dirt()),
            Some(ComponentDirt::NONE)
        );
    }
}
