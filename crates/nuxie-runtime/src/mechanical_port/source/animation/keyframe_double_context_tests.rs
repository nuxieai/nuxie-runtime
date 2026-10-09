use super::*;
use crate::mechanical_port::source::{
    animation::blend_accumulator::BlendAccumulator,
    core::{CoreArena, CoreObject},
    custom_property_number::CustomPropertyNumber,
    generated::custom_property_number_base::CustomPropertyNumberBase,
};
use std::{
    cell::{Cell, RefCell},
    panic::{AssertUnwindSafe, catch_unwind},
    rc::Rc,
};

struct Context<'a> {
    number: Box<dyn Fn(&CoreHandle) -> Option<f32> + 'a>,
    accumulator: Option<Rc<RefCell<BlendAccumulator>>>,
}
impl KeyFrameValueContext for Context<'_> {
    fn blend_accumulator(&self) -> Option<Rc<RefCell<BlendAccumulator>>> {
        self.accumulator.clone()
    }
    fn number_value(&self, keyframe: &CoreHandle) -> Option<f32> {
        (self.number)(keyframe)
    }
    fn bool_value(&self, _: &CoreHandle) -> Option<bool> {
        None
    }
    fn color_value(&self, _: &CoreHandle) -> Option<i32> {
        None
    }
    fn string_value(&self, _: &CoreHandle) -> Option<String> {
        None
    }
    fn stateful_interpolator_transform_value(
        &self,
        _: &CoreHandle,
        _: &CoreHandle,
        _: f32,
        _: f32,
        _: f32,
    ) -> Option<f32> {
        None
    }
    fn stateful_interpolator_transform(
        &self,
        _: &CoreHandle,
        _: &CoreHandle,
        _: f32,
    ) -> Option<f32> {
        None
    }
}
fn frame(value: f32, frame: u32) -> KeyFrameDouble {
    let mut keyframe = KeyFrameDouble::default();
    keyframe.base.set_value_value(value);
    keyframe.base.set_frame_value(frame);
    keyframe.base.compute_seconds(60);
    keyframe
}

#[test]
fn absent_identity_skips_open_context_and_replaced_identity_is_read_fresh() {
    let calls = RefCell::new(Vec::new());
    let context = Context {
        number: Box::new(|key| {
            calls.borrow_mut().push(key.identity_key());
            None
        }),
        accumulator: None,
    };
    let unregistered = frame(7.0, 0);
    assert_eq!(unregistered.effective_value(Some(&context)), 7.0);
    assert!(calls.borrow().is_empty());
    let arena = CoreArena::default();
    let first = arena.insert(frame(11.0, 0));
    let second = arena.insert(frame(19.0, 0));
    let mut detached = arena.remove(&first).unwrap();
    assert_eq!(
        detached
            .as_any()
            .downcast_ref::<KeyFrameDouble>()
            .unwrap()
            .effective_value(Some(&context)),
        11.0
    );
    detached.core_mut().set_handle(second.clone());
    assert_eq!(
        detached
            .as_any()
            .downcast_ref::<KeyFrameDouble>()
            .unwrap()
            .effective_value(Some(&context)),
        11.0
    );
    assert_eq!(
        *calls.borrow(),
        [first.identity_key(), second.identity_key()]
    );
}

#[test]
fn open_numeric_callback_can_drop_arena_and_retain_identity_until_receiver_releases() {
    let arena = CoreArena::default();
    let key = arena.insert(frame(7.0, 0));
    let owner = RefCell::new(Some(arena));
    let retained = RefCell::new(None);
    let context = Context {
        number: Box::new(|key| {
            drop(owner.borrow_mut().take());
            assert!(key.is_alive());
            assert_eq!(
                key.with_downcast::<KeyFrameDouble, _>(|v| v.base.value()),
                Some(7.0)
            );
            *retained.borrow_mut() = Some(key.clone());
            Some(23.0)
        }),
        accumulator: None,
    };
    assert_eq!(
        key.with_downcast::<KeyFrameDouble, _>(|v| v.effective_value(Some(&context))),
        Some(23.0)
    );
    assert!(!key.is_alive());
    assert!(!retained.borrow().as_ref().unwrap().is_alive());
}

#[test]
fn numeric_callback_unwind_releases_pending_receiver_retirement() {
    let arena = CoreArena::default();
    let key = arena.insert(frame(7.0, 0));
    let owner = RefCell::new(Some(arena));
    let context = Context {
        number: Box::new(|key| {
            drop(owner.borrow_mut().take());
            assert!(key.is_alive());
            panic!("numeric callback sentinel")
        }),
        accumulator: None,
    };
    assert!(
        catch_unwind(AssertUnwindSafe(|| key.with_downcast::<KeyFrameDouble, _>(
            |v| v.effective_value(Some(&context))
        )))
        .is_err()
    );
    assert!(!key.is_alive());
}

#[test]
fn numeric_callback_cannot_replace_handle_through_active_shared_keyframe() {
    let arena = CoreArena::default();
    let key = arena.insert(frame(7.0, 0));
    let replacement = arena.insert(frame(8.0, 0));
    let context = Context {
        number: Box::new(|key| {
            let attempted = catch_unwind(AssertUnwindSafe(|| {
                key.with_mut(|v| v.core_mut().set_handle(replacement.clone()))
            }));
            assert!(attempted.is_err());
            None
        }),
        accumulator: None,
    };
    assert_eq!(
        key.with_downcast::<KeyFrameDouble, _>(|v| v.effective_value(Some(&context))),
        Some(7.0)
    );
    assert_eq!(key.with(|v| v.core().handle()).flatten(), Some(key.clone()));
}

#[test]
fn interpolation_resolves_from_then_fresh_to_through_open_context() {
    let arena = CoreArena::default();
    let from = arena.insert(frame(1.0, 0));
    let to = arena.insert(frame(1.0, 60));
    let target = arena.insert(CustomPropertyNumber::default());
    let property = i32::from(CustomPropertyNumberBase::PROPERTY_VALUE_PROPERTY_KEY);
    let next_value = Cell::new(0.0);
    let calls = RefCell::new(Vec::new());
    let context = Context {
        number: Box::new(|key| {
            calls.borrow_mut().push(key.identity_key());
            if key == &from {
                next_value.set(10.0);
                Some(2.0)
            } else {
                Some(next_value.get())
            }
        }),
        accumulator: None,
    };
    from.with_downcast::<KeyFrameDouble, _>(|a| {
        to.with_downcast::<KeyFrameDouble, _>(|b| {
            assert!(a.apply_interpolation(&target, property, 0.5, b, 1.0, Some(&context)));
        })
    })
    .unwrap()
    .unwrap();
    assert_eq!(*calls.borrow(), [from.identity_key(), to.identity_key()]);
    assert_eq!(
        CoreRegistry::get_double_handle(&target, property),
        Some(6.0)
    );
}

#[test]
fn numeric_callback_can_seed_accumulator_before_numeric_apply_borrows_it() {
    let arena = CoreArena::default();
    let key = arena.insert(frame(1.0, 0));
    let target = arena.insert(CustomPropertyNumber::default());
    let property = i32::from(CustomPropertyNumberBase::PROPERTY_VALUE_PROPERTY_KEY);
    let accumulator = Rc::new(RefCell::new(BlendAccumulator::default()));
    let context = Context {
        number: Box::new(|_| {
            accumulator
                .borrow_mut()
                .seed_double(&target, property, 10.0);
            Some(18.0)
        }),
        accumulator: Some(accumulator.clone()),
    };
    assert_eq!(
        key.with_downcast::<KeyFrameDouble, _>(|v| v.apply(&target, property, 0.5, Some(&context))),
        Some(true)
    );
    accumulator.borrow_mut().flush();
    assert_eq!(
        CoreRegistry::get_double_handle(&target, property),
        Some(14.0)
    );
}
