#[derive(Clone, Copy)]
pub struct Span<'a, T> {
    slice: &'a [T],
}

impl<'a, T> Default for Span<'a, T> {
    fn default() -> Self {
        Self { slice: &[] }
    }
}

impl<'a, T> Span<'a, T> {
    pub const fn new(slice: &'a [T]) -> Self {
        Self { slice }
    }

    pub unsafe fn from_raw_parts(pointer: *const T, size: usize) -> Self {
        assert!(pointer.addr() <= pointer.wrapping_add(size).addr());
        Self {
            slice: unsafe { std::slice::from_raw_parts(pointer, size) },
        }
    }

    pub const fn data(&self) -> *const T {
        self.slice.as_ptr()
    }

    pub const fn size(&self) -> usize {
        self.slice.len()
    }

    pub const fn empty(&self) -> bool {
        self.slice.is_empty()
    }

    pub fn front(&self) -> &T {
        &self[0]
    }

    pub fn back(&self) -> &T {
        &self[self.size() - 1]
    }

    pub const fn size_bytes(&self) -> usize {
        self.size() * std::mem::size_of::<T>()
    }

    pub const fn count(&self) -> usize {
        self.size()
    }

    pub fn subset(&self, offset: usize, size: usize) -> Self {
        assert!(offset <= self.size());
        assert!(size <= self.size() - offset);
        Self::new(&self.slice[offset..offset + size])
    }

    pub const fn as_slice(&self) -> &'a [T] {
        self.slice
    }

    pub fn iter(&self) -> std::slice::Iter<'a, T> {
        self.slice.iter()
    }
}

impl<T> std::ops::Index<usize> for Span<'_, T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        assert!(index < self.size());
        &self.slice[index]
    }
}

impl<T> PartialEq for Span<'_, T> {
    fn eq(&self, other: &Self) -> bool {
        self.data() == other.data() && self.size() == other.size()
    }
}

impl<T> Eq for Span<'_, T> {}

impl<'a, T> From<&'a [T]> for Span<'a, T> {
    fn from(value: &'a [T]) -> Self {
        Self::new(value)
    }
}

impl<'a, T, const N: usize> From<&'a [T; N]> for Span<'a, T> {
    fn from(value: &'a [T; N]) -> Self {
        Self::new(value)
    }
}

pub fn make_span<T>(slice: &[T]) -> Span<'_, T> {
    Span::new(slice)
}
