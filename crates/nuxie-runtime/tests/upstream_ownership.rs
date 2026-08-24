// Direct safe-Rust adaptations of pinned `refcnt_test.cpp` and
// `lite_rtti_test.cpp`. `Rc` and `Any` are the campaign's approved Rust-native
// owners; this file deliberately does not recreate intrusive C++ ownership.

use std::any::{Any, TypeId};
use std::rc::Rc;

#[derive(Debug, Default)]
struct MyRefCounted {
    integer: i32,
    float: f32,
    boolean: bool,
}

#[test]
fn refcnt_direct_safe_rust_port() {
    let value = Rc::new(MyRefCounted::default());
    assert_eq!(Rc::strong_count(&value), 1);

    let safe_ref = Rc::clone(&value);
    assert_eq!(Rc::strong_count(&value), 2);
    drop(safe_ref);
    assert_eq!(Rc::strong_count(&value), 1);

    let nullable: Option<Rc<MyRefCounted>> = None;
    let safe_ref_of_null = nullable.clone();
    drop(safe_ref_of_null);
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
    assert_eq!(Rc::strong_count(r1.as_ref().unwrap()), 1);

    let mut r2 = r1.clone();
    assert!(Rc::ptr_eq(r1.as_ref().unwrap(), r2.as_ref().unwrap()));
    assert_eq!(Rc::strong_count(r2.as_ref().unwrap()), 2);

    let r3 = Rc::new(MyRefCounted {
        integer: 1,
        float: 0.5,
        boolean: false,
    });
    assert!(!Rc::ptr_eq(r1.as_ref().unwrap(), &r3));
    assert_eq!(Rc::strong_count(&r3), 1);
    assert_eq!((r3.integer, r3.float, r3.boolean), (1, 0.5, false));

    let released = r2.take().expect("release owned reference");
    assert!(r2.is_none());
    assert!(Rc::ptr_eq(r1.as_ref().unwrap(), &released));
    assert_eq!(Rc::strong_count(&released), 2);
    drop(released);
    assert_eq!(Rc::strong_count(r1.as_ref().unwrap()), 1);

    r1.take();
    assert!(r1.is_none());

    let derived = Rc::new(Derived {
        base: Base { x: 17 },
        y: 21,
    });
    assert_eq!(derived.y, 21);
    let erased: Rc<dyn Any> = derived.clone();
    assert_eq!(erased.downcast_ref::<Derived>().unwrap().base.x, 17);
    assert_eq!(Rc::strong_count(&derived), 2);
    assert!(erased.downcast_ref::<Base>().is_none());

    let restored = erased.downcast::<Derived>().expect("static Rc cast");
    assert_eq!(restored.y, 21);
    assert_eq!(Rc::strong_count(&derived), 2);
    drop(restored);
    assert_eq!(Rc::strong_count(&derived), 1);

    let erased: Rc<dyn Any> = derived.clone();
    assert_eq!(Rc::strong_count(&derived), 2);
    assert!(std::ptr::eq(
        erased.downcast_ref::<Derived>().unwrap(),
        derived.as_ref()
    ));
    assert_eq!(erased.downcast_ref::<Derived>().unwrap().base.x, 17);
    assert_eq!(erased.downcast_ref::<Derived>().unwrap().y, 21);
}

#[derive(Debug)]
struct RttiA;
#[derive(Debug)]
struct RttiB;
#[derive(Debug)]
struct RttiC;
#[derive(Debug)]
struct RttiD {
    x: f32,
    y: i32,
}
#[derive(Debug)]
struct RttiE {
    base: RttiD,
}
#[derive(Debug)]
struct RttiF;
#[derive(Debug)]
struct RttiG;
#[derive(Debug)]
struct RttiH;

#[test]
fn lite_rtti_behaves_correctly_direct_safe_rust_port() {
    assert_ne!(TypeId::of::<RttiA>(), TypeId::of::<RttiB>());
    assert_ne!(TypeId::of::<RttiA>(), TypeId::of::<RttiC>());
    assert_ne!(TypeId::of::<RttiB>(), TypeId::of::<RttiC>());

    let a = RttiA;
    let b = RttiB;
    let c = RttiC;
    let a_erased: &dyn Any = &a;
    let b_erased: &dyn Any = &b;
    let c_erased: &dyn Any = &c;
    assert!(a_erased.downcast_ref::<RttiB>().is_none());
    assert!(a_erased.downcast_ref::<RttiC>().is_none());
    assert!(b_erased.downcast_ref::<RttiB>().is_some());
    assert!(b_erased.downcast_ref::<RttiC>().is_none());
    assert!(c_erased.downcast_ref::<RttiB>().is_none());
    assert!(c_erased.downcast_ref::<RttiC>().is_some());

    let nil: Option<&dyn Any> = None;
    assert!(
        nil.and_then(|value| value.downcast_ref::<RttiB>())
            .is_none()
    );
    assert!(
        nil.and_then(|value| value.downcast_ref::<RttiC>())
            .is_none()
    );

    let e = RttiE {
        base: RttiD { x: 4.5, y: 6 },
    };
    let erased: &dyn Any = &e;
    let e = erased.downcast_ref::<RttiE>().expect("derived type");
    assert_eq!(e.base.x, 4.5);
    assert_eq!(e.base.y, 6);

    assert_ne!(TypeId::of::<RttiF>(), TypeId::of::<RttiG>());
    let erased: Rc<dyn Any> = Rc::new(RttiG);
    assert!(erased.clone().downcast::<RttiG>().is_ok());
    assert!(erased.downcast::<RttiH>().is_err());
}
