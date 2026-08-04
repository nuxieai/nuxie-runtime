#[allow(non_camel_case_types)]
pub struct TempVector<'a, T> {
    pub(crate) storage: *mut alloc::vec::Vec<T>,
    pub(crate) offset: usize,
    pub(crate) size_: usize,
    pub(crate) _marker: core::marker::PhantomData<&'a mut T>,
}

impl<'a, T> core::fmt::Debug for TempVector<'a, T>
where
    T: core::fmt::Debug,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("TempVector")
            .field("storage_len", unsafe { &(*self.storage).len() })
            .field("offset", &self.offset)
            .field("size_", &self.size_)
            .finish()
    }
}
