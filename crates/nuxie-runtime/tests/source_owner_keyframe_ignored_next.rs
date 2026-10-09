use nuxie_runtime::source::{
    animation::{
        keyframe_bool::KeyFrameBool, keyframe_int::KeyFrameInt, keyframe_string::KeyFrameString,
    },
    core::CoreArena,
    generated::core_registry::{CoreCapabilities, CoreRegistry},
};
use std::panic::{AssertUnwindSafe, catch_unwind};
#[test]
fn int_interpolation_does_not_borrow_ignored_next() {
    let arena = CoreArena::default();
    let mut frame = KeyFrameInt::default();
    CoreRegistry::set_int(&mut frame, 1068, 42);
    let target = arena.insert(KeyFrameInt::default());
    let next = arena.insert(KeyFrameInt::default());
    let result = catch_unwind(AssertUnwindSafe(|| {
        next.with_mut(|_| {
            frame.keyframe_interpolate(target.clone(), 1068, 0.5, next.clone(), 1.0, None)
        })
    }));
    assert!(
        result.is_ok(),
        "source ignores nextFrame; its unrelated active loan must not prevent target application"
    );
    assert_eq!(result.unwrap(), Some(true));
    assert_eq!(
        target.with_downcast::<KeyFrameInt, _>(|v| v.base.value()),
        Some(42)
    );
}
#[test]
fn bool_interpolation_does_not_borrow_ignored_next() {
    let arena = CoreArena::default();
    let mut frame = KeyFrameBool::default();
    CoreRegistry::set_bool(&mut frame, 181, true);
    let target = arena.insert(KeyFrameBool::default());
    let next = arena.insert(KeyFrameBool::default());
    let result = catch_unwind(AssertUnwindSafe(|| {
        next.with_mut(|_| {
            frame.keyframe_interpolate(target.clone(), 181, 0.5, next.clone(), 1.0, None)
        })
    }));
    assert!(
        result.is_ok(),
        "source ignores nextFrame; its unrelated active loan must not prevent target application"
    );
    assert_eq!(result.unwrap(), Some(true));
    assert_eq!(
        target.with_downcast::<KeyFrameBool, _>(|v| v.base.value()),
        Some(true)
    );
}
#[test]
fn string_interpolation_does_not_borrow_ignored_next() {
    let arena = CoreArena::default();
    let mut frame = KeyFrameString::default();
    CoreRegistry::set_string(&mut frame, 280, "held".into());
    let target = arena.insert(KeyFrameString::default());
    let next = arena.insert(KeyFrameString::default());
    let result = catch_unwind(AssertUnwindSafe(|| {
        next.with_mut(|_| {
            frame.keyframe_interpolate(target.clone(), 280, 0.5, next.clone(), 1.0, None)
        })
    }));
    assert!(
        result.is_ok(),
        "source ignores nextFrame; its unrelated active loan must not prevent target application"
    );
    assert_eq!(result.unwrap(), Some(true));
    assert_eq!(
        target.with_downcast::<KeyFrameString, _>(|v| v.base.value().to_owned()),
        Some("held".into())
    );
}
