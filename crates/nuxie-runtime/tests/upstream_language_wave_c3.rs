//! Exact Rust-ownership adaptation of pinned `lite_rtti_test.cpp`.

use std::any::{Any, TypeId};
use std::rc::Rc;

struct Abcd;
struct Abcc;
struct Bbcd;
struct RttiA;
struct RttiB;
struct RttiC;
struct RttiD {
    x: f32,
    y: i32,
}
struct RttiE {
    base: RttiD,
}
struct RttiF;
struct RttiG;
struct RttiH;

#[test]
fn wave_c3_lite_rtti_001_behaves_correctly() {
    // `TypeId` plus `Any` is the approved Rust ownership adaptation for the
    // upstream compile-time string id and exact dynamic-type checks.
    assert_ne!(TypeId::of::<Abcd>(), TypeId::of::<Abcc>());
    assert_ne!(TypeId::of::<Abcd>(), TypeId::of::<Bbcd>());

    let a = RttiA;
    assert_eq!(Any::type_id(&a), TypeId::of::<RttiA>());

    let b = RttiB;
    assert_ne!(TypeId::of::<RttiB>(), TypeId::of::<RttiA>());
    assert_ne!(Any::type_id(&b), Any::type_id(&a));
    assert_eq!(Any::type_id(&b), TypeId::of::<RttiB>());

    let c = RttiC;
    assert_ne!(TypeId::of::<RttiC>(), TypeId::of::<RttiA>());
    assert_ne!(TypeId::of::<RttiC>(), TypeId::of::<RttiB>());
    assert_ne!(Any::type_id(&c), Any::type_id(&a));
    assert_ne!(Any::type_id(&c), Any::type_id(&b));
    assert_eq!(Any::type_id(&c), TypeId::of::<RttiC>());

    let erased_a: &dyn Any = &a;
    let erased_b: &dyn Any = &b;
    let erased_c: &dyn Any = &c;

    assert!(erased_a.downcast_ref::<RttiB>().is_none());
    assert!(erased_a.downcast_ref::<RttiC>().is_none());

    assert!(std::ptr::eq(
        erased_b.downcast_ref::<RttiB>().expect("B exact cast"),
        &b
    ));
    assert!(erased_b.downcast_ref::<RttiC>().is_none());

    assert!(erased_c.downcast_ref::<RttiB>().is_none());
    assert!(std::ptr::eq(
        erased_c.downcast_ref::<RttiC>().expect("C exact cast"),
        &c
    ));

    let erased_c_from_b_contract: &dyn Any = &c;
    assert!(erased_c_from_b_contract.downcast_ref::<RttiB>().is_none());
    assert!(std::ptr::eq(
        erased_c_from_b_contract
            .downcast_ref::<RttiC>()
            .expect("C exact cast through erased base contract"),
        &c
    ));

    let nil: Option<&dyn Any> = None;
    assert!(
        nil.and_then(|value| value.downcast_ref::<RttiB>())
            .is_none()
    );
    assert!(
        nil.and_then(|value| value.downcast_ref::<RttiC>())
            .is_none()
    );

    let erased_d: Box<dyn Any> = Box::new(RttiE {
        base: RttiD { x: 4.5, y: 6 },
    });
    assert_eq!(Any::type_id(erased_d.as_ref()), TypeId::of::<RttiE>());
    let e = erased_d.downcast::<RttiE>().expect("E exact cast");
    assert_eq!(e.base.x, 4.5);
    assert_eq!(e.base.y, 6);

    assert_ne!(TypeId::of::<RttiF>(), TypeId::of::<RttiG>());
    let erased_f: Rc<dyn Any> = Rc::new(RttiG);
    let g = Rc::clone(&erased_f).downcast::<RttiG>();
    let h = erased_f.downcast::<RttiH>();
    assert!(g.is_ok());
    assert!(h.is_err());
}
