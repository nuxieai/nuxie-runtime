//! Complete public lite_rtti_test.cpp at 9eb2a6d9, using the production helper.
use super::*;
use crate::mechanical_port::source::include::rive::refcnt_hpp::{RefCnt, make_rcp};
use std::cell::Cell;
use std::rc::Rc;

struct ATag;
struct BTag;
struct CTag;
struct JTag;
type A = enable_lite_rtti<ATag, { CONST_ID("A") }>;
type B = lite_rtti_override<A, BTag, { CONST_ID("B") }>;
type C = lite_rtti_override<B, CTag, { CONST_ID("C") }>;
type J = lite_rtti_override<A, JTag, { CONST_ID("J") }>;
impl LiteRttiCastFrom<A> for C {
    unsafe fn from_base(base: *mut A) -> *mut Self {
        // Both repr(C) wrapper bases are first fields.
        base.cast()
    }
}

// The three calls preserve mutable, const-result, and const-input source cases.
unsafe fn check_cast<U, T>(base: *mut T, expected: *mut U)
where
    U: LiteRttiCastFrom<T>,
    T: LiteRttiBase,
{
    unsafe {
        assert_eq!(lite_rtti_cast::<U, T>(base), expected);
        assert_eq!(lite_rtti_cast_const::<U, T>(base), expected.cast_const());
        let immutable: *const T = base;
        assert_eq!(
            lite_rtti_cast_const::<U, T>(immutable),
            expected.cast_const()
        );
    }
}

struct DestructorTest(Rc<Cell<u32>>);
impl Drop for DestructorTest {
    fn drop(&mut self) {
        self.0.set(self.0.get() + 1);
    }
}
#[repr(C)]
struct D {
    rtti: enable_lite_rtti<D, { CONST_ID("D") }>,
    x: f32,
    y: i32,
    dt: DestructorTest,
    // Rust spelling of the source virtual destructor dispatched through D*.
    destroy: unsafe fn(*mut D),
}
impl LiteRttiTypeId for D {
    const LITE_RTTI_TYPE_ID: u32 = CONST_ID("D");
    fn can_rtti_cast_to(_: u32) -> bool {
        false
    }
}
impl LiteRttiBase for D {
    fn canRTTICastTo(&self, id: u32) -> bool {
        self.rtti.canRTTICastTo(id)
    }
    fn setRttiPredicate(&mut self, predicate: fn(u32) -> bool) {
        self.rtti.setRttiPredicate(predicate);
    }
}
struct ETag;
type E = lite_rtti_override<D, ETag, { CONST_ID("E") }>;

#[repr(C)]
struct F {
    refcnt: RefCnt<F>,
    rtti: enable_lite_rtti<F, { CONST_ID("F") }>,
    // Rust's explicit equivalent of F's virtual destructor.
    destroy: unsafe fn(*const F),
}
impl LiteRttiTypeId for F {
    const LITE_RTTI_TYPE_ID: u32 = CONST_ID("F");
    fn can_rtti_cast_to(_: u32) -> bool {
        false
    }
}
impl LiteRttiBase for F {
    fn canRTTICastTo(&self, id: u32) -> bool {
        self.rtti.canRTTICastTo(id)
    }
    fn setRttiPredicate(&mut self, predicate: fn(u32) -> bool) {
        self.rtti.setRttiPredicate(predicate);
    }
}
// SAFETY: RefCnt<F> is the first field. The zero hook dispatches to the
// allocation's concrete destructor, like upstream's virtual ~F.
unsafe impl RefCntTarget for F {
    fn r#ref(&self) {
        self.refcnt.r#ref();
    }
    unsafe fn unref(&self) {
        unsafe { self.refcnt.unref() };
    }
    unsafe fn onRefCntReachedZero(ptr: *const Self) {
        unsafe { ((*ptr).destroy)(ptr) };
    }
}
struct GTag;
struct HTag;
type G = lite_rtti_override<F, GTag, { CONST_ID("G") }>;
type H = lite_rtti_override<F, HTag, { CONST_ID("H") }>;
macro_rules! derived_refcnt {
    ($derived:ty) => {
        // SAFETY: wrapper and F each have their base at offset zero.
        unsafe impl RefCntTarget for $derived {
            fn r#ref(&self) {
                self.base().r#ref();
            }
            unsafe fn unref(&self) {
                unsafe { self.base().unref() };
            }
        }
    };
}
derived_refcnt!(G);
derived_refcnt!(H);

#[test]
fn lite_rtti_behaves_correctly() {
    const {
        assert!(CONST_ID("abcd") != CONST_ID("abcc"));
    }
    const {
        assert!(CONST_ID("abcd") != CONST_ID("bbcd"));
    }
    assert_eq!(A::TypeIDForRTTI, CONST_ID("A"));
    assert_eq!(B::TypeIDForRTTI, CONST_ID("B"));
    assert_ne!(B::TypeIDForRTTI, A::TypeIDForRTTI);
    assert_eq!(C::TypeIDForRTTI, CONST_ID("C"));
    assert_ne!(C::TypeIDForRTTI, B::TypeIDForRTTI);
    assert_ne!(C::TypeIDForRTTI, A::TypeIDForRTTI);
    let mut a = A::default();
    let mut b = B::default();
    let mut c = C::default();
    let mut j = J::default();
    let bp = &mut b as *mut B;
    let cp = &mut c as *mut C;
    let jp = &mut j as *mut J;
    let pa = &mut a as *mut A;
    let pab = b.base_mut() as *mut A;
    let pbc = c.base_mut() as *mut B;
    let pac = c.base_mut().base_mut() as *mut A;
    let paj = j.base_mut() as *mut A;
    // SAFETY: live first-field base projections; casts check the dynamic chain.
    unsafe {
        check_cast::<B, _>(pa, core::ptr::null_mut());
        check_cast::<C, _>(pa, core::ptr::null_mut());
        check_cast::<J, _>(pa, core::ptr::null_mut());
        check_cast::<B, _>(pab, bp);
        check_cast::<C, _>(pab, core::ptr::null_mut());
        check_cast::<J, _>(pab, core::ptr::null_mut());
        check_cast::<B, _>(pac, pbc);
        check_cast::<C, _>(pac, cp);
        check_cast::<J, _>(pac, core::ptr::null_mut());
        check_cast::<B, _>(paj, core::ptr::null_mut());
        check_cast::<C, _>(paj, core::ptr::null_mut());
        check_cast::<J, _>(paj, jp);
        check_cast::<C, _>(bp, core::ptr::null_mut());
        check_cast::<C, _>(pbc, cp);
        assert!(lite_rtti_cast::<B, A>(core::ptr::null_mut()).is_null());
        assert!(lite_rtti_cast_const::<C, A>(core::ptr::null()).is_null());
    }

    let destruct_count = Rc::new(Cell::new(0));
    assert_eq!(destruct_count.get(), 0);
    let e = Box::new(E::from_args(|| D {
        rtti: enable_lite_rtti::new(),
        x: 4.5,
        y: 6,
        dt: DestructorTest(destruct_count.clone()),
        destroy: |p| unsafe { drop(Box::from_raw(p.cast::<E>())) },
    }));
    // Transfer ownership through the first-field base, like new E assigned D*.
    let pde = Box::into_raw(e).cast::<D>();
    let pe = unsafe { lite_rtti_cast::<E, D>(pde) };
    assert!(!pe.is_null());
    assert_eq!(unsafe { (*pe).base().x }, 4.5);
    assert_eq!(unsafe { (*pe).base().y }, 6);
    // Delete through the base destructor slot, which destroys the complete E.
    unsafe { ((*pde).destroy)(pde) };
    assert_eq!(destruct_count.get(), 1);

    let mut pg = make_rcp(|| {
        G::from_args(|| F {
            refcnt: RefCnt::new(),
            rtti: enable_lite_rtti::new(),
            destroy: |p| unsafe { drop(Box::from_raw(p.cast_mut().cast::<G>())) },
        })
    });
    // Transfer the same intrusive owner to its first-field F subobject.
    let pfg = unsafe { rcp::<F>::from_ptr(pg.release().cast()) };
    let pg = unsafe { lite_rtti_rcp_cast::<G, F>(pfg.clone()) };
    let ph = unsafe { lite_rtti_rcp_cast::<H, F>(pfg.clone()) };
    assert!(!pg.get().is_null());
    assert!(ph.get().is_null());
}
