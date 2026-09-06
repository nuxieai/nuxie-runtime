//! renderer/include/rive/renderer/stack_vector.hpp at f7c22102.
#![allow(dead_code)]
use std::ops::{Index, IndexMut};

pub(crate) struct StackVector<T: Copy, const N: usize> {
    data: [T; N],
    size: usize,
}
impl<T: Copy, const N: usize> StackVector<T, N> {
    // Initialize inactive storage explicitly without constraining element defaults.
    pub(crate) fn new(fill: T) -> Self {
        Self {
            data: [fill; N],
            size: 0,
        }
    }
}
impl<T: Copy, const N: usize> StackVector<T, N> {
    pub(crate) fn clear(&mut self) {
        self.size = 0;
    }
    pub(crate) fn size(&self) -> usize {
        self.size
    }
    pub(crate) fn as_slice(&self) -> &[T] {
        &self.data[..self.size]
    }
    pub(crate) fn as_mut_slice(&mut self) -> &mut [T] {
        &mut self.data[..self.size]
    }
    pub(crate) fn front(&self) -> &T {
        &self[0]
    }
    pub(crate) fn back(&self) -> &T {
        &self[self.size - 1]
    }
    fn push(&mut self, count: usize) -> &mut [T] {
        assert!(count <= N - self.size);
        let start = self.size;
        self.size += count;
        &mut self.data[start..self.size]
    }
    pub(crate) fn push_back(&mut self, value: T) -> &mut T {
        let result = &mut self.push(1)[0];
        *result = value;
        result
    }
    pub(crate) fn push_back_n(&mut self, count: usize, source: Option<&[T]>) -> &mut [T] {
        let result = self.push(count);
        if let Some(source) = source {
            result.copy_from_slice(&source[..count]);
        }
        result
    }
    pub(crate) fn push_back_repeated(&mut self, count: usize, value: T) -> &mut [T] {
        let result = self.push(count);
        result.fill(value);
        result
    }
    pub(crate) fn insert(&mut self, index: usize, value: T) -> &mut T {
        assert!(index <= self.size);
        self.push(1);
        self.data.copy_within(index..self.size - 1, index + 1);
        self.data[index] = value;
        &mut self.data[index]
    }
}
impl<T: Copy, const N: usize> Index<usize> for StackVector<T, N> {
    type Output = T;
    fn index(&self, index: usize) -> &T {
        &self.as_slice()[index]
    }
}
impl<T: Copy, const N: usize> IndexMut<usize> for StackVector<T, N> {
    fn index_mut(&mut self, index: usize) -> &mut T {
        &mut self.as_mut_slice()[index]
    }
}

#[cfg(test)]
mod tests {
    use super::StackVector;
    const VALUES: [u32; 8] = [99999, 12345, 0, 1, 2468, 1358, 777777, u32::MAX];
    #[test]
    fn insert_into_empty() {
        let mut vec = StackVector::<u32, 8>::new(0);
        assert_eq!(VALUES[0], *vec.insert(0, VALUES[0]));
        let inserted = vec.insert(1, VALUES[1]) as *const u32;
        assert_eq!(inserted, &vec[1] as *const u32);
        assert_eq!(vec.size(), 2);
        assert_eq!(vec[0], VALUES[0]);
        assert_eq!(vec[1], VALUES[1]);
    }
    #[test]
    fn insert_at_end_matches_push_back() {
        let mut vec = StackVector::<u32, 8>::new(0);
        for i in 0..VALUES.len() {
            assert_eq!(VALUES[i], *vec.insert(vec.size(), VALUES[i]));
            assert!(std::ptr::eq(vec.back(), &vec[i]));
            assert_eq!(vec.size(), i + 1);
        }
        for i in 0..VALUES.len() {
            assert_eq!(vec[i], VALUES[i]);
        }
    }
    #[test]
    fn insert_at_front_shifts_up() {
        let mut vec = StackVector::<u32, 8>::new(0);
        for i in 0..VALUES.len() {
            assert_eq!(VALUES[i], *vec.insert(0, VALUES[i]));
            assert_eq!(*vec.front(), VALUES[i]);
            assert_eq!(vec.size(), i + 1);
        }
        for i in 0..VALUES.len() {
            assert_eq!(vec[i], VALUES[VALUES.len() - 1 - i]);
        }
    }
    #[test]
    fn insert_in_middle() {
        let mut vec = StackVector::<u32, 8>::new(0);
        vec.push_back(VALUES[0]);
        vec.push_back(VALUES[1]);
        vec.push_back(VALUES[2]);
        assert_eq!(VALUES[3], *vec.insert(2, VALUES[3]));
        assert_eq!(vec.size(), 4);
        assert_eq!(
            vec.as_slice(),
            &[VALUES[0], VALUES[1], VALUES[3], VALUES[2]]
        );
        assert_eq!(VALUES[4], *vec.insert(1, VALUES[4]));
        assert_eq!(vec.size(), 5);
        assert_eq!(
            vec.as_slice(),
            &[VALUES[0], VALUES[4], VALUES[1], VALUES[3], VALUES[2]]
        );
    }
}
