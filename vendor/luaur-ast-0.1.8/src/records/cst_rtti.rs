use core::marker::PhantomData;

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct CstRtti<T> {
    pub(crate) _phantom: PhantomData<T>,
}
