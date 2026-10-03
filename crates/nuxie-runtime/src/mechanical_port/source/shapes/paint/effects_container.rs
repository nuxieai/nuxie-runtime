use crate::mechanical_port::source::{
    component::{Component, ComponentOccurrenceHandle},
    component_dirt::ComponentDirt,
    core::CoreHandle,
    shapes::paint::{
        feather::Feather,
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

pub(crate) fn invalidate_rendering_handle(paint: &CoreHandle) {
    invalidate_rendering_handle_with_active(paint, &mut None);
}

fn invalidate_rendering_handle_with_active(
    paint: &CoreHandle,
    active: &mut Option<ActiveStrokeEffect<'_>>,
) {
    let action = paint
        .with(|object| {
            object
                .as_shape_paint_behavior()
                .map(|paint| paint.prepare_rendering_invalidation())
        })
        .flatten();
    let Some(action) = action else {
        return;
    };
    // Stroke calls the actual renderer before Super::invalidateRendering.
    // Release the Core receiver, and then the temporary render-paint identity,
    // before reading any dirt or dependent state after that callback.
    action.before_dirt();
    add_dirt_with_active(
        &ComponentOccurrenceHandle::Authored(paint.clone()),
        ComponentDirt::PATH,
        active,
    );
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
                .unwrap_or_default()
        })
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

    // ShapePaint reads its feather only after every effect callback. A group
    // has no paint tail; its derived target invalidations ran above.
    let feather = container
        .with(|container| {
            container
                .as_shape_paint()
                .and_then(|paint| paint.feather_handle())
        })
        .flatten();
    if let Some(feather) = feather {
        feather.with_downcast_mut::<Feather, _>(|feather| {
            feather.mark_effect_path_dirty();
            if feather.is_inner() {
                feather.base.add_dirt(ComponentDirt::PATH, false);
            }
        });
    }
    invalidate_rendering_handle_with_active(container, active);
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
    use crate::mechanical_port::source::{
        artboard::{Artboard, RuntimeArtboardInstanceHandle},
        core::CoreArena,
        core_context::{CoreContext, StatusCode},
        factory::RuntimeFactoryHandle,
        file::File,
        node::Node,
        shapes::paint::{fill::Fill, paint_image::PaintImage, stroke::Stroke},
    };
    use nuxie_render_api as render;
    use std::{cell::Cell, rc::Rc};

    struct InvalidationPaint {
        calls: Rc<Cell<usize>>,
        callback: Box<dyn FnMut()>,
    }

    impl render::RenderPaint for InvalidationPaint {
        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
        fn style(&mut self, _: render::RenderPaintStyle) {}
        fn color(&mut self, _: render::ColorInt) {}
        fn thickness(&mut self, _: f32) {}
        fn join(&mut self, _: render::StrokeJoin) {}
        fn cap(&mut self, _: render::StrokeCap) {}
        fn feather(&mut self, _: f32) {}
        fn blend_mode(&mut self, _: render::BlendMode) {}
        fn additiveness(&mut self, _: f32) {}
        fn shader(&mut self, _: Option<&dyn render::RenderShader>) {}
        fn invalidate_stroke(&mut self) {
            self.calls.set(self.calls.get() + 1);
            (self.callback)();
        }
    }

    fn loaded_artboard(fixture: &str) -> RuntimeArtboardInstanceHandle {
        let path = std::path::PathBuf::from(
            std::env::var_os("RIVE_RUNTIME_DIR").expect("pinned RIVE_RUNTIME_DIR"),
        )
        .join("tests/unit_tests/assets")
        .join(fixture);
        let bytes = std::fs::read(path).unwrap();
        let mut factory = render::PersistentFactory::new(render::NullFactory);
        let file = File::import(
            &bytes,
            RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
            None,
            None,
            None,
        )
        .unwrap();
        Artboard::instance_from_handle(&file.with_file(File::artboard).unwrap()).unwrap()
    }

    fn install_invalidation_paint(
        paint: &CoreHandle,
        callback: impl FnMut() + 'static,
    ) -> Rc<Cell<usize>> {
        let retained = paint
            .with(|paint| {
                paint
                    .as_shape_paint()
                    .unwrap()
                    .render_paint_handle()
                    .unwrap()
            })
            .unwrap();
        let calls = Rc::new(Cell::new(0));
        *retained.borrow_mut() = Box::new(InvalidationPaint {
            calls: calls.clone(),
            callback: Box::new(callback),
        });
        calls
    }

    fn set_dirt(handle: &CoreHandle, dirt: ComponentDirt) {
        handle
            .with_mut(|object| object.as_component_mut().unwrap().set_dirt(dirt))
            .unwrap();
    }

    fn dirt(handle: &CoreHandle) -> ComponentDirt {
        handle
            .with(|object| object.as_component().unwrap().dirt())
            .unwrap()
    }

    #[test]
    fn stroke_invalidation_releases_owner_and_reads_dirt_and_dependents_after_renderer() {
        // This pinned upstream fixture imports the real white_stroke and its
        // initialized paint mutator, as runtime/stroke_test.cpp does.
        let artboard = loaded_artboard("stroke_name_test.riv");
        let stroke = artboard
            .with_artboard(|root| root.find_handle::<Stroke>("white_stroke"))
            .unwrap();
        let arena = stroke.retain_arena().unwrap();
        let dependent = arena.insert(Node::default());
        set_dirt(&stroke, ComponentDirt::PATH);
        set_dirt(&dependent, ComponentDirt::NONE);
        let callback_stroke = stroke.clone();
        let callback_dependent = dependent.clone();
        let first = Cell::new(true);
        let calls = install_invalidation_paint(&stroke, move || {
            assert!(dirt(&callback_stroke).contains(ComponentDirt::PATH));
            if first.replace(false) {
                callback_stroke
                    .with_mut(|object| {
                        let component = object.as_component_mut().unwrap();
                        component.set_dirt(ComponentDirt::NONE);
                        component.add_dependent(callback_dependent.clone());
                    })
                    .unwrap();
            }
        });
        invalidate_effects_handle(&stroke, None);
        assert_eq!(calls.get(), 1);
        assert!(dirt(&stroke).contains(ComponentDirt::PATH));
        assert!(dirt(&dependent).contains(ComponentDirt::PATH));

        // Renderer invalidation runs even when base addDirt will early-return.
        set_dirt(&dependent, ComponentDirt::NONE);
        invalidate_effects_handle(&stroke, None);
        assert_eq!(calls.get(), 2);
        assert_eq!(dirt(&dependent), ComponentDirt::NONE);
    }

    #[test]
    fn borrowed_derived_and_base_projection_preserve_stroke_virtual_tail() {
        let artboard = loaded_artboard("stroke_name_test.riv");
        let stroke = artboard
            .with_artboard(|root| root.find_handle::<Stroke>("white_stroke"))
            .unwrap();
        let calls = install_invalidation_paint(&stroke, || {});
        set_dirt(&stroke, ComponentDirt::NONE);
        // This is the explicit caller-owned borrowed receiver lane. Its
        // renderer does not try to reborrow that already borrowed receiver.
        stroke
            .with_mut(|object| {
                object
                    .as_effects_container_mut()
                    .unwrap()
                    .invalidate_effects(None);
            })
            .unwrap();
        assert_eq!(calls.get(), 1);
        assert!(dirt(&stroke).contains(ComponentDirt::PATH));
        set_dirt(&stroke, ComponentDirt::NONE);
        stroke
            .with_mut(|object| object.as_shape_paint_mut().unwrap().invalidate_effects())
            .unwrap();
        assert_eq!(calls.get(), 2);
        assert!(dirt(&stroke).contains(ComponentDirt::PATH));
    }

    #[test]
    fn fill_invalidation_uses_base_tail_without_stroke_renderer_callback() {
        let artboard = loaded_artboard("shapetest.riv");
        let fill = artboard
            .with_artboard(|root| root.find_all_handles::<Fill>())
            .into_iter()
            .next()
            .unwrap();
        let calls = install_invalidation_paint(&fill, || panic!("Fill has no Stroke override"));
        set_dirt(&fill, ComponentDirt::NONE);
        invalidate_effects_handle(&fill, None);
        assert_eq!(calls.get(), 0);
        assert!(dirt(&fill).contains(ComponentDirt::PATH));
    }

    #[test]
    fn paint_image_asset_updated_uses_released_parent_virtual_invalidation() {
        struct PaintContext {
            arena: CoreArena,
            root: CoreHandle,
            paint: CoreHandle,
        }
        impl CoreContext for PaintContext {
            fn core_arena(&self) -> &CoreArena {
                &self.arena
            }
            fn resolve_handle(&self, id: u32) -> Option<CoreHandle> {
                match id {
                    0 => Some(self.root.clone()),
                    1 => Some(self.paint.clone()),
                    _ => None,
                }
            }
        }
        let artboard = loaded_artboard("stroke_name_test.riv");
        let stroke = artboard
            .with_artboard(|root| root.find_handle::<Stroke>("white_stroke"))
            .unwrap();
        let arena = stroke.retain_arena().unwrap();
        let dependent = arena.insert(Node::default());
        let mut image = PaintImage::default();
        image.base.base.base.set_parent_id_value(1);
        let image = arena.insert(image);
        let mut context = PaintContext {
            arena,
            root: artboard.core_handle(),
            paint: stroke.clone(),
        };
        assert_eq!(
            image.with_mut(|image| {
                image
                    .as_component_mut()
                    .unwrap()
                    .on_added_dirty(&mut context)
            }),
            Some(StatusCode::Ok)
        );
        set_dirt(&stroke, ComponentDirt::PATH);
        set_dirt(&dependent, ComponentDirt::NONE);
        let callback_stroke = stroke.clone();
        let callback_dependent = dependent.clone();
        let calls = install_invalidation_paint(&stroke, move || {
            callback_stroke
                .with_mut(|object| {
                    let component = object.as_component_mut().unwrap();
                    component.set_dirt(ComponentDirt::NONE);
                    component.add_dependent(callback_dependent.clone());
                })
                .unwrap();
        });
        image
            .with_downcast_mut::<PaintImage, _>(PaintImage::asset_updated)
            .unwrap();
        assert_eq!(calls.get(), 1);
        assert!(dirt(&stroke).contains(ComponentDirt::PATH));
        assert!(dirt(&dependent).contains(ComponentDirt::PATH));
    }

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
            assert!(
                handle
                    .with(|object| object
                        .as_component()
                        .unwrap()
                        .dirt()
                        .contains(ComponentDirt::PATH))
                    .unwrap()
            );
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
