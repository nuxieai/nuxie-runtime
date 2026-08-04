#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AstRtti<T> {
    pub(crate) _phantom: core::marker::PhantomData<T>,
}
