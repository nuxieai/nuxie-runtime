//! Port of a `First<T, ...>` template metafunction whose member `using type = T`
//! exposes the first type of a pack.
//!
//! **Deviation (documented):** C++'s `First<T>::type` is an inherent member type
//! alias; Rust inherent associated types are unstable, so the `type` member is
//! re-expressed as the stable associated type of the [`FirstType`] trait. The
//! relationship `First<T> -> T` is preserved; access it as
//! `<First<T> as FirstType>::Type`.

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct First<T> {
    pub(crate) _marker: core::marker::PhantomData<T>,
}

/// Carries the `First<T>::type == T` relationship as a stable associated type.
pub trait FirstType {
    type Type;
}

impl<T> FirstType for First<T> {
    type Type = T;
}
