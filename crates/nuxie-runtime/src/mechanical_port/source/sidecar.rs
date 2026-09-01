/// An owning pointer to a heap-allocated `T`, allocated lazily on first
/// authored write.
///
/// Generated Core base classes use this to hoist clusters of rarely authored
/// ("cold") properties out of the inline object. Reads do not allocate;
/// generated getters instead fall back to the property's default value.
pub struct Sidecar<T> {
    value: Option<Box<T>>,
}

impl<T> Default for Sidecar<T> {
    fn default() -> Self {
        Self { value: None }
    }
}

impl<T: Clone> Clone for Sidecar<T> {
    fn clone(&self) -> Self {
        Self {
            value: self.value.clone(),
        }
    }
}

impl<T> Sidecar<T> {
    /// Returns the backing object, or `None` when it has never been allocated.
    pub fn get(&self) -> Option<&T> {
        self.value.as_deref()
    }

    /// Returns the writable backing object, or `None` when it has never been
    /// allocated.
    pub fn get_mut(&mut self) -> Option<&mut T> {
        self.value.as_deref_mut()
    }

    /// Releases the backing object and returns to the never-allocated state.
    pub fn reset(&mut self) {
        self.value = None;
    }
}

impl<T: Default> Sidecar<T> {
    /// Allocates the backing object on first use, then returns it.
    pub fn ensure(&mut self) -> &mut T {
        self.value.get_or_insert_with(|| Box::new(T::default()))
    }
}

#[cfg(test)]
mod tests {
    use super::Sidecar;

    #[derive(Clone, Debug, Default, PartialEq)]
    struct Payload {
        value: u32,
    }

    #[test]
    fn storage_is_pointer_sized_and_reads_do_not_allocate() {
        let sidecar = Sidecar::<Payload>::default();

        assert_eq!(
            std::mem::size_of::<Sidecar<Payload>>(),
            std::mem::size_of::<usize>()
        );
        assert!(sidecar.get().is_none());
    }

    #[test]
    fn ensure_reuses_storage_and_reset_releases_it() {
        let mut sidecar = Sidecar::<Payload>::default();
        sidecar.ensure().value = 7;
        let allocated = sidecar.get().unwrap() as *const Payload;

        assert_eq!(sidecar.ensure().value, 7);
        assert_eq!(sidecar.get().unwrap() as *const Payload, allocated);

        sidecar.reset();
        assert!(sidecar.get().is_none());
    }

    #[test]
    fn cloning_empty_storage_stays_empty() {
        let sidecar = Sidecar::<Payload>::default();
        let cloned = sidecar.clone();

        assert!(sidecar.get().is_none());
        assert!(cloned.get().is_none());
    }

    #[test]
    fn cloning_allocated_storage_is_deep() {
        let mut sidecar = Sidecar::<Payload>::default();
        sidecar.ensure().value = 7;
        let mut cloned = sidecar.clone();

        assert_eq!(cloned.get(), sidecar.get());
        assert!(!std::ptr::eq(sidecar.get().unwrap(), cloned.get().unwrap()));

        cloned.get_mut().unwrap().value = 9;
        assert_eq!(sidecar.get().unwrap().value, 7);
        assert_eq!(cloned.get().unwrap().value, 9);
    }
}
