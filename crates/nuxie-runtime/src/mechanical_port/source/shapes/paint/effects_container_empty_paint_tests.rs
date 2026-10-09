use super::tests::{dirt, install_invalidation_paint, loaded_artboard, set_dirt};
use super::*;
use crate::mechanical_port::source::{
    core::{
        Core, CoreArena, CoreObject, PropertySetterCompletion, binary_reader::BinaryReader,
        field_types::core_callback_type::CallbackData,
    },
    generated::{
        core_registry::{CoreCapabilities, CoreField, CoreRegistryObject},
        shapes::paint::fill_base::FillBase,
    },
    math::mat2d::Mat2D,
    shapes::paint::{
        shape_paint::{ShapePaint, ShapePaintBehavior},
        stroke_effect::StrokeEffectState,
    },
};
use std::{
    any::Any,
    cell::{Cell, RefCell},
    rc::Rc,
};

// Projects a real Fill and advertises its generated type predicate, but has
// observable custom accessors. Actual Any identity must retain this fallback.
#[derive(Default)]
struct ProjectedFill {
    fill: Fill,
    log: Rc<RefCell<Vec<&'static str>>>,
    stroke: StrokeEffectState,
    callback: Option<Box<dyn FnMut()>>,
    component_reads: Rc<Cell<usize>>,
    component_mut_reads: Rc<Cell<usize>>,
    dirt_calls: Rc<Cell<usize>>,
    dirt_callback: Option<Box<dyn FnMut()>>,
}
impl CoreCapabilities for ProjectedFill {
    fn as_component(&self) -> Option<&Component> {
        self.component_reads.set(self.component_reads.get() + 1);
        self.fill.as_component()
    }
    fn as_component_mut(&mut self) -> Option<&mut Component> {
        self.component_mut_reads
            .set(self.component_mut_reads.get() + 1);
        self.fill.as_component_mut()
    }
    fn component_on_dirty(&mut self, _: ComponentDirt) -> bool {
        self.dirt_calls.set(self.dirt_calls.get() + 1);
        if let Some(callback) = &mut self.dirt_callback {
            callback();
        }
        true
    }
    fn as_effects_container_mut(&mut self) -> Option<&mut dyn EffectsContainer> {
        self.log.borrow_mut().push("effects");
        Some(&mut self.fill)
    }
    fn as_shape_paint(&self) -> Option<&ShapePaint> {
        self.log.borrow_mut().push("feather");
        Some(&self.fill.base.base)
    }
    fn as_shape_paint_behavior(&self) -> Option<&dyn ShapePaintBehavior> {
        self.log.borrow_mut().push("rendering");
        Some(&self.fill)
    }
    fn as_stroke_effect_mut(&mut self) -> Option<&mut dyn StrokeEffect> {
        Some(self)
    }
}
impl CoreObject for ProjectedFill {
    fn core(&self) -> &Core {
        self.fill.core()
    }
    fn core_mut(&mut self) -> &mut Core {
        self.fill.core_mut()
    }
    fn core_type(&self) -> u16 {
        Fill::TYPE_KEY
    }
    fn is_type_of(&self, key: u16) -> bool {
        FillBase::is_type_of(key)
    }
    fn type_predicate(&self) -> fn(u16) -> bool {
        FillBase::is_type_of
    }
    fn deserialize(&mut self, key: u16, reader: &mut BinaryReader<'_>) -> bool {
        self.fill.deserialize(key, reader)
    }
}
impl CoreRegistryObject for ProjectedFill {
    fn as_registry_any(&self) -> &dyn Any {
        &self.fill
    }
    fn as_registry_any_mut(&mut self) -> &mut dyn Any {
        &mut self.fill
    }
    fn is_type_of(&self, key: u16) -> bool {
        FillBase::is_type_of(key)
    }
    fn set_uint_with_completion(
        &mut self,
        field: CoreField,
        value: u32,
        completion: &mut PropertySetterCompletion,
    ) {
        self.fill.set_uint_with_completion(field, value, completion);
    }
    fn set_string_with_completion(
        &mut self,
        field: CoreField,
        value: String,
        completion: &mut PropertySetterCompletion,
    ) {
        self.fill
            .set_string_with_completion(field, value, completion);
    }
    fn set_color_with_completion(
        &mut self,
        field: CoreField,
        value: i32,
        completion: &mut PropertySetterCompletion,
    ) {
        self.fill
            .set_color_with_completion(field, value, completion);
    }
    fn set_bool_with_completion(
        &mut self,
        field: CoreField,
        value: bool,
        completion: &mut PropertySetterCompletion,
    ) {
        self.fill.set_bool_with_completion(field, value, completion);
    }
    fn set_double_with_completion(
        &mut self,
        field: CoreField,
        value: f32,
        completion: &mut PropertySetterCompletion,
    ) {
        self.fill
            .set_double_with_completion(field, value, completion);
    }
    fn set_callback_with_completion(
        &mut self,
        field: CoreField,
        value: CallbackData<'_>,
        completion: &mut PropertySetterCompletion,
    ) {
        self.fill
            .set_callback_with_completion(field, value, completion);
    }
    fn set_int_with_completion(
        &mut self,
        field: CoreField,
        value: i32,
        completion: &mut PropertySetterCompletion,
    ) {
        self.fill.set_int_with_completion(field, value, completion);
    }
    fn get_uint(&mut self, field: CoreField) -> u32 {
        self.fill.get_uint(field)
    }
    fn get_string(&mut self, field: CoreField) -> String {
        self.fill.get_string(field)
    }
    fn get_color(&mut self, field: CoreField) -> i32 {
        self.fill.get_color(field)
    }
    fn get_bool(&mut self, field: CoreField) -> bool {
        self.fill.get_bool(field)
    }
    fn get_double(&mut self, field: CoreField) -> f32 {
        self.fill.get_double(field)
    }
    fn get_int(&mut self, field: CoreField) -> i32 {
        self.fill.get_int(field)
    }
}
impl StrokeEffect for ProjectedFill {
    fn stroke_effect_state(&mut self) -> &mut StrokeEffectState {
        &mut self.stroke
    }
    fn stroke_effect_handle(&self) -> Option<CoreHandle> {
        self.core().handle()
    }
    fn parent_paint_handle(&self) -> Option<CoreHandle> {
        None
    }
    fn update_effect(&mut self, _: &PathProvider, _: &ShapePaintPath, _: &ShapePaint) {}
    fn invalidate_effect(&mut self, _: Option<&PathProvider>) {
        self.log.borrow_mut().push("invalidate");
        if let Some(callback) = &mut self.callback {
            callback();
        }
    }
}
fn clean_feather(arena: &CoreArena) -> CoreHandle {
    let mut feather = Feather::default();
    feather.rebuild_inner_path(&ShapePaintPath::default(), &Mat2D::default(), false);
    assert!(!feather.effect_path_dirty());
    arena.insert(feather)
}
fn feather_dirty(feather: &CoreHandle) -> bool {
    feather
        .with_downcast::<Feather, _>(Feather::effect_path_dirty)
        .unwrap()
}
#[test]
fn custom_projected_fill_retains_virtual_accessor_order() {
    let arena = CoreArena::default();
    let object = ProjectedFill::default();
    let log = object.log.clone();
    let paint = arena.insert(object);
    log.borrow_mut().clear();
    set_dirt(&paint, ComponentDirt::NONE);
    assert!(paint.is_type_of(Fill::TYPE_KEY));
    assert_eq!(
        paint.with(|object| object.as_any().is::<Fill>()),
        Some(true)
    );
    invalidate_effects_handle(&paint, None);
    assert_eq!(&*log.borrow(), &["effects", "feather", "rendering"]);
    assert!(dirt(&paint).contains(ComponentDirt::PATH));
}
#[test]
fn nonempty_paint_observes_feather_installed_by_effect_callback() {
    let arena = CoreArena::default();
    let paint = arena.insert(Fill::default());
    let feather = clean_feather(&arena);
    let callback_paint = paint.clone();
    let callback_feather = feather.clone();
    let effect = arena.insert(ProjectedFill {
        callback: Some(Box::new(move || {
            callback_paint
                .with_downcast_mut::<Fill, _>(|paint| {
                    paint.base.base.set_feather(callback_feather.clone())
                })
                .unwrap();
        })),
        ..Default::default()
    });
    paint.with_downcast_mut::<Fill, _>(|paint| paint.effects_state().add_stroke_effect(effect));
    invalidate_effects_handle(&paint, None);
    assert!(feather_dirty(&feather));
}
#[test]
fn feathered_paint_does_not_take_empty_paint_shortcut() {
    let arena = CoreArena::default();
    let feather = clean_feather(&arena);
    let mut fill = Fill::default();
    fill.base.base.set_feather(feather.clone());
    let paint = arena.insert(fill);
    invalidate_effects_handle(&paint, None);
    assert!(feather_dirty(&feather));
}
#[test]
fn group_keeps_effect_order_and_invalidating_boundary() {
    let arena = CoreArena::default();
    let a = ProjectedFill::default();
    let a_log = a.log.clone();
    let a = arena.insert(a);
    let b = ProjectedFill::default();
    let b_log = b.log.clone();
    let b = arena.insert(b);
    let mut group = GroupEffect::default();
    group.effects_state().add_stroke_effect(a.clone());
    group.effects_state().add_stroke_effect(b);
    let group = arena.insert(group);
    a_log.borrow_mut().clear();
    b_log.borrow_mut().clear();
    invalidate_effects_handle(&group, Some(a));
    assert!(a_log.borrow().is_empty());
    assert_eq!(&*b_log.borrow(), &["invalidate"]);
}
#[test]
fn renderer_edits_do_not_replay_already_completed_effect_and_feather_phases() {
    let artboard = loaded_artboard("stroke_name_test.riv");
    let stroke = artboard
        .with_artboard(|root| root.find_handle::<Stroke>("white_stroke"))
        .unwrap();
    let arena = stroke.retain_arena().unwrap();
    let feather = clean_feather(&arena);
    let effect = ProjectedFill::default();
    let log = effect.log.clone();
    let effect = arena.insert(effect);
    log.borrow_mut().clear();
    let callback_stroke = stroke.clone();
    let callback_feather = feather.clone();
    let first = std::cell::Cell::new(true);
    let calls = install_invalidation_paint(&stroke, move || {
        if first.replace(false) {
            callback_stroke
                .with_downcast_mut::<Stroke, _>(|paint| {
                    paint.base.base.set_feather(callback_feather.clone());
                    paint.effects_state().add_stroke_effect(effect.clone());
                })
                .unwrap();
        }
    });
    invalidate_effects_handle(&stroke, None);
    assert_eq!(calls.get(), 1);
    assert!(log.borrow().is_empty());
    assert!(!feather_dirty(&feather));
    invalidate_effects_handle(&stroke, None);
    assert_eq!(calls.get(), 2);
    assert_eq!(&*log.borrow(), &["invalidate"]);
    assert!(feather_dirty(&feather));
}
#[test]
fn renderer_can_release_runtime_owner_before_fresh_dirt_read() {
    let artboard = loaded_artboard("stroke_name_test.riv");
    let stroke = artboard
        .with_artboard(|root| root.find_handle::<Stroke>("white_stroke"))
        .unwrap();
    let weak = artboard.downgrade();
    let owner = Rc::new(RefCell::new(Some(artboard)));
    let callback_owner = owner.clone();
    let calls = install_invalidation_paint(&stroke, move || {
        drop(callback_owner.borrow_mut().take());
    });
    invalidate_effects_handle(&stroke, None);
    assert_eq!(calls.get(), 1);
    assert!(weak.upgrade().is_none());
    assert!(!stroke.is_alive());
}

#[test]
fn native_paint_keeps_custom_descendant_projections_and_reentrant_list_order() {
    let arena = CoreArena::default();
    let paint = arena.insert(Fill::default());
    let later = arena.insert(Fill::default());
    set_dirt(&paint, ComponentDirt::NONE);
    set_dirt(&later, ComponentDirt::NONE);
    let log = Rc::new(RefCell::new(Vec::new()));
    let callback_paint = paint.clone();
    let callback_later = later.clone();
    let callback_log = log.clone();
    let first = ProjectedFill {
        dirt_callback: Some(Box::new(move || {
            callback_log.borrow_mut().push("first");
            callback_paint
                .with_mut(|object| {
                    let component = object.as_component_mut().unwrap();
                    component.set_dirt(ComponentDirt::NONE);
                    component.add_dependent(callback_later.clone());
                })
                .unwrap();
        })),
        ..Default::default()
    };
    let shared_reads = first.component_reads.clone();
    let mutable_reads = first.component_mut_reads.clone();
    let dirty_calls = first.dirt_calls.clone();
    let first = arena.insert(first);
    let callback_log = log.clone();
    let second = arena.insert(ProjectedFill {
        dirt_callback: Some(Box::new(move || callback_log.borrow_mut().push("second"))),
        ..Default::default()
    });
    set_dirt(&first, ComponentDirt::NONE);
    set_dirt(&second, ComponentDirt::NONE);
    shared_reads.set(0);
    mutable_reads.set(0);
    paint
        .with_mut(|object| {
            let component = object.as_component_mut().unwrap();
            component.add_dependent(first.clone());
            component.add_dependent(second);
        })
        .unwrap();
    invalidate_effects_handle(&paint, None);
    assert_eq!(&*log.borrow(), &["first", "second"]);
    // The original custom path projects once for notification and once for
    // the separately released dependent snapshot, after its onDirty call.
    assert_eq!(shared_reads.get(), 2);
    assert_eq!(mutable_reads.get(), 1);
    assert_eq!(dirty_calls.get(), 1);
    assert_eq!(dirt(&later), ComponentDirt::NONE);
    assert_eq!(dirt(&paint), ComponentDirt::NONE);
    invalidate_effects_handle(&paint, None);
    assert!(dirt(&later).contains(ComponentDirt::PATH));
    assert_eq!(&*log.borrow(), &["first", "second"]);
}

#[test]
fn active_effect_keeps_its_borrowed_receiver_through_native_paint_recursion() {
    let arena = CoreArena::default();
    let paint = arena.insert(Fill::default());
    let effect = ProjectedFill::default();
    let dirty_calls = effect.dirt_calls.clone();
    let effect = arena.insert(effect);
    set_dirt(&paint, ComponentDirt::NONE);
    set_dirt(&effect, ComponentDirt::NONE);
    paint.with_mut(|object| {
        object
            .as_component_mut()
            .unwrap()
            .add_dependent(effect.clone())
    });
    effect
        .with_mut(|object| {
            let mut active = Some(ActiveStrokeEffect::new(
                effect.clone(),
                object.as_stroke_effect_mut().unwrap(),
            ));
            invalidate_effects_handle_with_active(&paint, None, &mut active);
        })
        .unwrap();
    assert_eq!(dirty_calls.get(), 1);
    assert!(dirt(&effect).contains(ComponentDirt::PATH));
    assert!(dirt(&paint).contains(ComponentDirt::PATH));
}

#[test]
fn duplicate_native_paint_still_attempts_the_checked_mutable_receiver_loan() {
    use std::panic::{AssertUnwindSafe, catch_unwind};
    let arena = CoreArena::default();
    let paint = arena.insert(Fill::default());
    set_dirt(&paint, ComponentDirt::PATH);
    let result = catch_unwind(AssertUnwindSafe(|| {
        paint.with(|_| invalidate_effects_handle(&paint, None));
    }));
    assert!(result.is_err());
    assert_eq!(dirt(&paint), ComponentDirt::PATH);
    set_dirt(&paint, ComponentDirt::NONE);
    invalidate_effects_handle(&paint, None);
    assert!(dirt(&paint).contains(ComponentDirt::PATH));
}

#[test]
fn stroke_renderer_retirement_and_slot_reuse_precede_the_native_dirt_loan() {
    let artboard = loaded_artboard("stroke_name_test.riv");
    let stroke = artboard
        .with_artboard(|root| root.find_handle::<Stroke>("white_stroke"))
        .unwrap();
    let arena = stroke.retain_arena().unwrap();
    let replacement = Rc::new(RefCell::new(None));
    let callback_stroke = stroke.clone();
    let callback_replacement = replacement.clone();
    let calls = install_invalidation_paint(&stroke, move || {
        drop(arena.remove(&callback_stroke).unwrap());
        let new_paint = arena.insert(Fill::default());
        assert_eq!(new_paint.identity_key().1, callback_stroke.identity_key().1);
        assert_ne!(new_paint.identity_key().2, callback_stroke.identity_key().2);
        set_dirt(&new_paint, ComponentDirt::NONE);
        *callback_replacement.borrow_mut() = Some(new_paint);
    });
    invalidate_effects_handle(&stroke, None);
    assert_eq!(calls.get(), 1);
    assert!(!stroke.is_alive());
    assert_eq!(
        dirt(replacement.borrow().as_ref().unwrap()),
        ComponentDirt::NONE
    );
}
