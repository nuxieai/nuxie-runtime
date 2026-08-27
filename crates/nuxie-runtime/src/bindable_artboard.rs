// Mechanical translation of pinned C++ `src/bindable_artboard.cpp` and
// `include/rive/bindable_artboard.hpp`.

/// Retained owner for the source file and concrete Artboard occurrence.
///
/// Pinned C++ stores `rcp<const File>` and `unique_ptr<ArtboardInstance>`.
/// Rust keeps the facade's file lifetime authority opaque and keeps the
/// occurrence behind a `RefCell` because the host refreshes a stable binding
/// identity in place.
pub(crate) struct RuntimeBindableArtboardOwner {
    source_file: Option<Rc<dyn std::any::Any>>,
    artboard: RefCell<Option<ArtboardInstance>>,
}

impl std::fmt::Debug for RuntimeBindableArtboardOwner {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RuntimeBindableArtboardOwner")
            .field("has_source_file", &self.source_file.is_some())
            .field("artboard", &self.artboard)
            .finish()
    }
}

impl RuntimeBindableArtboardOwner {
    /// Mechanical constructor order: retain the file first, then take the
    /// concrete Artboard occurrence.
    pub(crate) fn new(
        source_file: Option<Rc<dyn std::any::Any>>,
        artboard: Option<ArtboardInstance>,
    ) -> Self {
        Self {
            source_file,
            artboard: RefCell::new(artboard),
        }
    }

    /// Safe-Rust equivalent of the primary-header `artboard()` getter.
    pub(crate) fn artboard(&self) -> Option<ArtboardInstance> {
        self.artboard.borrow().clone()
    }

    pub(crate) fn has_artboard(&self) -> bool {
        self.artboard.borrow().is_some()
    }

    pub(crate) fn replace_artboard(&self, artboard: ArtboardInstance) {
        self.artboard.replace(Some(artboard));
    }

    pub(crate) fn source_file<T: 'static>(&self) -> Option<Rc<T>> {
        self.source_file.as_ref().cloned()?.downcast::<T>().ok()
    }
}
