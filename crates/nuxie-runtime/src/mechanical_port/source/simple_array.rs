#[cfg(feature = "testing")]
pub mod testing {
    use std::sync::atomic::{AtomicI32, Ordering};

    pub static MALLOC_COUNT: AtomicI32 = AtomicI32::new(0);
    pub static REALLOC_COUNT: AtomicI32 = AtomicI32::new(0);
    pub static FREE_COUNT: AtomicI32 = AtomicI32::new(0);

    pub fn reset_counters() {
        MALLOC_COUNT.store(0, Ordering::Relaxed);
        REALLOC_COUNT.store(0, Ordering::Relaxed);
        FREE_COUNT.store(0, Ordering::Relaxed);
    }
}

pub struct SimpleArray<T> {
    values: Box<[T]>,
}

impl<T> Default for SimpleArray<T> {
    fn default() -> Self {
        Self {
            values: Box::new([]),
        }
    }
}

impl<T: Default> SimpleArray<T> {
    pub fn new(size: usize) -> Self {
        let mut values = Vec::new();
        if values.try_reserve_exact(size).is_ok() {
            values.resize_with(size, T::default);
            #[cfg(feature = "testing")]
            testing::MALLOC_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
        Self {
            values: values.into_boxed_slice(),
        }
    }
}

impl<T: Clone + Default> SimpleArray<T> {
    pub fn from_slice(slice: &[T]) -> Self {
        let mut array = Self::new(slice.len());
        if !array.is_empty() {
            array.values.clone_from_slice(slice);
        }
        array
    }
}

impl<T> SimpleArray<T> {
    pub fn data(&self) -> *const T {
        self.values.as_ptr()
    }

    pub fn data_mut(&mut self) -> *mut T {
        self.values.as_mut_ptr()
    }

    pub fn size(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn front(&self) -> &T {
        &self[0]
    }

    pub fn back(&self) -> &T {
        &self[self.size() - 1]
    }

    pub fn size_bytes(&self) -> usize {
        self.size() * std::mem::size_of::<T>()
    }

    pub fn as_slice(&self) -> &[T] {
        &self.values
    }

    pub fn as_mut_slice(&mut self) -> &mut [T] {
        &mut self.values
    }
}

impl<T: Clone> Clone for SimpleArray<T> {
    fn clone(&self) -> Self {
        Self {
            values: self.values.clone(),
        }
    }
}

impl<T> std::ops::Index<usize> for SimpleArray<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.values[index]
    }
}

impl<T> std::ops::IndexMut<usize> for SimpleArray<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.values[index]
    }
}

impl<T> Drop for SimpleArray<T> {
    fn drop(&mut self) {
        #[cfg(feature = "testing")]
        testing::FREE_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }
}

pub struct SimpleArrayBuilder<T> {
    values: Vec<T>,
}

impl<T> SimpleArrayBuilder<T> {
    pub fn with_reserve(reserve: usize) -> Self {
        Self {
            values: Vec::with_capacity(reserve),
        }
    }

    pub fn new() -> Self {
        Self::with_reserve(0)
    }

    pub fn add(&mut self, value: T) {
        if self.values.len() == self.values.capacity() {
            let target = std::cmp::max(1, self.values.capacity() * 2);
            self.values.reserve_exact(target - self.values.len());
            #[cfg(feature = "testing")]
            testing::REALLOC_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
        self.values.push(value);
    }

    pub fn capacity(&self) -> usize {
        self.values.capacity()
    }

    pub fn size(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn front(&self) -> &T {
        &self.values[0]
    }

    pub fn back(&self) -> &T {
        &self.values[self.values.len() - 1]
    }

    pub fn as_slice(&self) -> &[T] {
        &self.values
    }

    pub fn into_simple_array(mut self) -> SimpleArray<T> {
        self.values.shrink_to_fit();
        SimpleArray {
            values: self.values.into_boxed_slice(),
        }
    }
}

impl<T> Default for SimpleArrayBuilder<T> {
    fn default() -> Self {
        Self::new()
    }
}
