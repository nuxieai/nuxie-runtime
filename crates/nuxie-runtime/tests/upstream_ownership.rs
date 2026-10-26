//! Safe-Rust adaptations of pinned `refcnt_test.cpp` using the runtime's Rc owner.
//! The complete `lite_rtti_test.cpp` translation lives beside the actual renderer
//! helper in `mechanical_port/source/include/utils/lite_rtti_test.rs`.

use std::any::Any;
use std::rc::Rc;

use nuxie_runtime::source::refcnt::{NativeRefCount, release, reset, safe_ref, safe_unref, swap};

#[derive(Debug, Default)]
struct MyRefCounted {
    integer: i32,
    float: f32,
    boolean: bool,
}

#[test]
fn refcnt_direct_safe_rust_port() {
    let value = Rc::new(MyRefCounted::default());
    assert_eq!(value.debugging_refcnt(), 1);

    let retained = safe_ref(Some(&value));
    assert_eq!(value.debugging_refcnt(), 2);
    safe_unref(retained);
    assert_eq!(value.debugging_refcnt(), 1);

    let nullable: Option<Rc<MyRefCounted>> = None;
    let safe_ref_of_null = safe_ref(nullable.as_ref());
    safe_unref(safe_ref_of_null);
    assert!(nullable.is_none());
}

#[derive(Debug)]
struct Base {
    x: i32,
}

#[derive(Debug)]
struct Derived {
    base: Base,
    y: i32,
}

#[test]
fn rcp_direct_safe_rust_port() {
    let r0: Option<Rc<MyRefCounted>> = None;
    assert!(r0.is_none());

    let mut r1 = Some(Rc::new(MyRefCounted::default()));
    assert!(r1.is_some());
    assert_eq!(r1.as_ref().unwrap().debugging_refcnt(), 1);

    let mut r2 = r1.clone();
    assert!(r1.as_ref().unwrap().ptr_eq(r2.as_ref().unwrap()));
    assert_eq!(r2.as_ref().unwrap().debugging_refcnt(), 2);

    let r3 = Rc::new(MyRefCounted {
        integer: 1,
        float: 0.5,
        boolean: false,
    });
    assert!(!r1.as_ref().unwrap().ptr_eq(&r3));
    assert_eq!(r3.debugging_refcnt(), 1);
    assert_eq!((r3.integer, r3.float, r3.boolean), (1, 0.5, false));

    let released = release(&mut r2).expect("release owned reference");
    assert!(r2.is_none());
    assert!(r1.as_ref().unwrap().ptr_eq(&released));
    assert_eq!(released.debugging_refcnt(), 2);
    drop(released);
    assert_eq!(r1.as_ref().unwrap().debugging_refcnt(), 1);

    reset(&mut r1, None);
    assert!(r1.is_none());

    let mut left = Some(Rc::new(MyRefCounted::default()));
    let mut right = None;
    swap(&mut left, &mut right);
    assert!(left.is_none());
    assert!(right.is_some());

    let derived = Rc::new(Derived {
        base: Base { x: 17 },
        y: 21,
    });
    assert_eq!(derived.y, 21);
    let erased: Rc<dyn Any> = derived.clone();
    assert_eq!(erased.downcast_ref::<Derived>().unwrap().base.x, 17);
    assert_eq!(derived.debugging_refcnt(), 2);
    assert!(erased.downcast_ref::<Base>().is_none());

    let restored = erased.downcast::<Derived>().expect("static Rc cast");
    assert_eq!(restored.y, 21);
    assert_eq!(derived.debugging_refcnt(), 2);
    drop(restored);
    assert_eq!(derived.debugging_refcnt(), 1);

    let erased: Rc<dyn Any> = derived.clone();
    assert_eq!(derived.debugging_refcnt(), 2);
    assert!(std::ptr::eq(
        erased.downcast_ref::<Derived>().unwrap(),
        derived.as_ref()
    ));
    assert_eq!(erased.downcast_ref::<Derived>().unwrap().base.x, 17);
    assert_eq!(erased.downcast_ref::<Derived>().unwrap().y, 21);
}
