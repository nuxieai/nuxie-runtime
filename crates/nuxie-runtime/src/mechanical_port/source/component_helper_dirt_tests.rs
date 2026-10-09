use super::*;
use crate::mechanical_port::source::{
    core::CoreArena,
    factory::RuntimeFactoryHandle,
    node::Node,
    shapes::{paint::stroke::Stroke, shape::Shape},
    text::text_variation_helper::RuntimeTextVariationHelperHandle,
};
use nuxie_render_api as render;
use std::cell::Cell;

struct InvalidationPaint(Box<dyn FnMut()>);
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
        (self.0)();
    }
}

fn deferred_composer(arena: &CoreArena) -> (CoreHandle, ComponentOccurrenceHandle) {
    let shape = arena.insert(Shape::default());
    let composer = shape
        .with_downcast_mut::<Shape, _>(|shape| shape.path_composer_mut().clone())
        .unwrap();
    composer.bind_shape(shape.clone());
    composer.with_mut(|helper| {
        helper.component.set_dirt(ComponentDirt::NONE);
        assert!(helper.update(ComponentDirt::PATH).is_none());
    });
    (shape, composer.occurrence())
}

fn add_callback_paint(arena: &CoreArena, shape: &CoreHandle, callback: impl FnMut() + 'static) {
    let mut factory = render::PersistentFactory::new(render::NullFactory);
    let factory = RuntimeFactoryHandle::from_factory(&mut factory).unwrap();
    let mut stroke = Stroke::default();
    assert!(stroke.base.base.init_render_paint(shape.clone(), &factory));
    *stroke.base.base.render_paint_handle().unwrap().borrow_mut() =
        Box::new(InvalidationPaint(Box::new(callback)));
    let stroke = arena.insert(stroke);
    shape.with_downcast_mut::<Shape, _>(|shape| shape.paint_container.add_paint(stroke));
}

fn clean_node(arena: &CoreArena) -> CoreHandle {
    let node = arena.insert(Node::default());
    node.with_mut(|node| {
        node.as_component_mut()
            .unwrap()
            .set_dirt(ComponentDirt::NONE)
    });
    node
}

#[test]
fn deferred_helper_releases_receiver_before_reentry_and_reads_new_dependents_after_callback() {
    let arena = CoreArena::default();
    let (shape, occurrence) = deferred_composer(&arena);
    let dependent = clean_node(&arena);
    let calls = Rc::new(Cell::new(0));
    let callback_calls = calls.clone();
    let callback_occurrence = occurrence.clone();
    let callback_dependent = dependent.clone();
    add_callback_paint(&arena, &shape, move || {
        callback_calls.set(callback_calls.get() + 1);
        assert_eq!(
            callback_occurrence.with_component(Component::dirt),
            Some(ComponentDirt::PATH)
        );
        assert!(callback_occurrence.add_dirt(ComponentDirt::N_SLICER, false));
        callback_occurrence
            .with_component_mut(|helper| helper.add_dependent(callback_dependent.clone()));
    });
    assert!(occurrence.add_dirt(ComponentDirt::PATH, true));
    assert_eq!(calls.get(), 1);
    assert_eq!(
        dependent.with(|node| node.as_component().unwrap().dirt()),
        Some(ComponentDirt::PATH)
    );
    assert!(!occurrence.add_dirt(ComponentDirt::PATH, true));
    assert_eq!(calls.get(), 1);
}

#[test]
fn deferred_helper_releases_last_temporary_owner_before_callback_retirement() {
    let arena = CoreArena::default();
    let (shape, occurrence) = deferred_composer(&arena);
    let root = arena.insert(Artboard::default());
    let dirty = root.artboard_dirty_handle().unwrap();
    dirty.set_component_dirt(ComponentDirt::NONE);
    let dependent = clean_node(&arena);
    occurrence.with_component_mut(|helper| {
        helper.artboard = Some(root);
        helper.add_dependent(dependent.clone());
    });
    let callback_arena = arena.clone();
    let callback_shape = shape.clone();
    let callback_occurrence = occurrence.clone();
    let callback_dirty = dirty.clone();
    let calls = Rc::new(Cell::new(0));
    let callback_calls = calls.clone();
    add_callback_paint(&arena, &shape, move || {
        callback_calls.set(callback_calls.get() + 1);
        assert!(!callback_dirty.has_component_dirt());
        drop(callback_arena.remove(&callback_shape).unwrap());
        assert!(
            callback_occurrence
                .with_component(Component::dirt)
                .is_none()
        );
    });
    assert!(occurrence.add_dirt(ComponentDirt::PATH, true));
    assert_eq!(calls.get(), 1);
    assert!(!dirty.has_component_dirt());
    assert_eq!(
        dependent.with(|node| node.as_component().unwrap().dirt()),
        Some(ComponentDirt::NONE)
    );
    assert!(!occurrence.add_dirt(ComponentDirt::N_SLICER, true));
}

#[test]
fn text_helper_preserves_duplicate_gate_notification_and_recursion_choice() {
    let arena = CoreArena::default();
    let root = arena.insert(Artboard::default());
    let dirty = root.artboard_dirty_handle().unwrap();
    let dependent = clean_node(&arena);
    let helper = RuntimeTextVariationHelperHandle::new(dependent.clone());
    helper.with_mut(|helper| {
        helper.component.artboard = Some(root);
        helper.component.set_dirt(ComponentDirt::NONE);
        helper.component.add_dependent(dependent.clone());
    });
    let occurrence = helper.occurrence();
    dirty.set_component_dirt(ComponentDirt::NONE);
    assert!(occurrence.add_dirt(ComponentDirt::PATH, false));
    assert!(dirty.has_component_dirt());
    assert_eq!(
        dependent.with(|node| node.as_component().unwrap().dirt()),
        Some(ComponentDirt::NONE)
    );
    dirty.set_component_dirt(ComponentDirt::NONE);
    assert!(!occurrence.add_dirt(ComponentDirt::PATH, true));
    assert!(!dirty.has_component_dirt());
    assert!(occurrence.add_dirt(ComponentDirt::N_SLICER, true));
    assert!(dirty.has_component_dirt());
    assert_eq!(
        dependent.with(|node| node.as_component().unwrap().dirt()),
        Some(ComponentDirt::N_SLICER)
    );
    drop(helper);
    assert!(!occurrence.add_dirt(ComponentDirt::WORLD_TRANSFORM, true));
}
