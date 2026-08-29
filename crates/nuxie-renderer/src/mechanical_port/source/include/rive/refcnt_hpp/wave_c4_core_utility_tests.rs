use super::{
    RefCnt, RefCntTarget, make_rcp, operator_eq_rcp, operator_ne_rcp, rcp, ref_rcp, safe_ref,
    safe_unref, static_rcp_cast,
};

#[repr(C)]
struct MyRefCnt {
    base: RefCnt<MyRefCnt>,
}

impl MyRefCnt {
    fn new() -> Self {
        Self {
            base: RefCnt::new(),
        }
    }

    fn new_with_args(_: i32, _: f32, _: bool) -> Self {
        Self::new()
    }

    fn debugging_refcnt(&self) -> i32 {
        self.base.debugging_refcnt()
    }
}

unsafe impl RefCntTarget for MyRefCnt {
    fn r#ref(&self) {
        self.base.r#ref();
    }

    unsafe fn unref(&self) {
        unsafe { self.base.unref() };
    }
}

#[test]
fn wave_c4_refcnt_001_refcnt() {
    let my = MyRefCnt::new();
    assert_eq!(my.debugging_refcnt(), 1);
    my.r#ref();
    assert_eq!(my.debugging_refcnt(), 2);
    unsafe { my.unref() };
    assert_eq!(my.debugging_refcnt(), 1);

    unsafe { safe_ref(&my as *const MyRefCnt as *mut MyRefCnt) };
    assert_eq!(my.debugging_refcnt(), 2);
    unsafe { safe_unref(&my as *const MyRefCnt as *mut MyRefCnt) };
    assert_eq!(my.debugging_refcnt(), 1);

    unsafe { safe_ref(core::ptr::null_mut::<MyRefCnt>()) };
    unsafe { safe_unref(core::ptr::null_mut::<MyRefCnt>()) };
}

#[repr(C)]
struct A {
    base: RefCnt<A>,
    x: i32,
}

unsafe impl RefCntTarget for A {
    fn r#ref(&self) {
        self.base.r#ref();
    }

    unsafe fn unref(&self) {
        unsafe { self.base.unref() };
    }
}

#[repr(C)]
struct B {
    base: RefCnt<B>,
    x: i32,
    y: i32,
}

unsafe impl RefCntTarget for B {
    fn r#ref(&self) {
        self.base.r#ref();
    }

    unsafe fn unref(&self) {
        unsafe { self.base.unref() };
    }
}

#[test]
fn wave_c4_refcnt_002_rcp() {
    let r0 = rcp::<MyRefCnt>::from_null(None);

    assert!(r0.get().is_null());
    assert!(!r0.operator_bool());

    let mut r1 = make_rcp(MyRefCnt::new);
    assert!(!r1.get().is_null());
    assert!(r1.operator_bool());
    assert!(operator_ne_rcp(&r1, &r0));
    assert_eq!(unsafe { (*r1.get()).debugging_refcnt() }, 1);

    let mut r2 = r1.clone();
    assert_eq!(r1.get(), r2.get());
    assert!(operator_eq_rcp(&r1, &r2));
    assert_eq!(unsafe { (*r2.get()).debugging_refcnt() }, 2);

    let r3 = make_rcp(|| MyRefCnt::new_with_args(1, 0.5, false));
    assert!(!r3.get().is_null());
    assert!(r3.operator_bool());
    assert!(operator_ne_rcp(&r3, &r1));
    assert_eq!(unsafe { (*r3.get()).debugging_refcnt() }, 1);

    let ptr = r2.release();
    assert!(r2.get().is_null());
    assert_eq!(r1.get(), ptr);

    assert_eq!(unsafe { (*r1.get()).debugging_refcnt() }, 2);
    unsafe { (*ptr).unref() };
    assert_eq!(unsafe { (*r1.get()).debugging_refcnt() }, 1);

    unsafe { r1.reset(core::ptr::null_mut()) };
    assert!(r1.get().is_null());

    let b = make_rcp(|| B {
        base: RefCnt::new(),
        x: 17,
        y: 21,
    });
    assert_eq!(unsafe { (*b.get()).y }, 21);
    let mut a = unsafe { rcp::<A>::converting_copy_ctor(&b) };
    assert_eq!(unsafe { (*a.get()).x }, 17);
    assert_eq!(unsafe { (*static_rcp_cast::<B, A>(a.clone()).get()).y }, 21);
    assert_eq!(unsafe { (*rcp::<A>::converting_copy_ctor(&b).get()).x }, 17);
    assert_eq!(unsafe { (*a.get()).base.debugging_refcnt() }, 2);
    assert_eq!(unsafe { (*b.get()).base.debugging_refcnt() }, 2);
    assert_eq!(
        unsafe { (*static_rcp_cast::<B, A>(rcp::move_ctor(&mut a)).get()).y },
        21
    );
    assert!(a.get().is_null());
    assert_eq!(unsafe { (*b.get()).base.debugging_refcnt() }, 1);
    let mut retained = unsafe { ref_rcp(b.get()) };
    let mut retained_as_a = unsafe { rcp::<A>::converting_move_ctor(&mut retained) };
    a.operator_assign_move(&mut retained_as_a);
    assert!(operator_eq_rcp(&b, &a));
    assert_eq!(unsafe { (*a.get()).base.debugging_refcnt() }, 2);
    assert_eq!(unsafe { (*a.get()).x }, 17);
    assert_eq!(unsafe { (*static_rcp_cast::<B, A>(a.clone()).get()).y }, 21);
}
