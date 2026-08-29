use std::marker::PhantomData;

use crate::mechanical_port::source::core::{CoreHandle, CoreType};

#[derive(Clone, Copy)]
enum Children<'a> {
    Nullable(&'a [Option<CoreHandle>]),
    NonNull(&'a [CoreHandle]),
}

impl<'a> Children<'a> {
    fn len(self) -> usize {
        match self {
            Self::Nullable(children) => children.len(),
            Self::NonNull(children) => children.len(),
        }
    }

    fn get(self, index: usize) -> Option<&'a CoreHandle> {
        match self {
            Self::Nullable(children) => children[index].as_ref(),
            Self::NonNull(children) => Some(&children[index]),
        }
    }
}

pub struct TypedChild<'a, T: CoreType> {
    children: Children<'a>,
    index: usize,
    marker: PhantomData<T>,
}

impl<T: CoreType> Iterator for TypedChild<'_, T> {
    type Item = CoreHandle;

    fn next(&mut self) -> Option<Self::Item> {
        while self.index < self.children.len() {
            let child = self.children.get(self.index);
            self.index += 1;
            if let Some(child) = child.filter(|child| child.is_type_of(T::TYPE_KEY)) {
                return Some(child.clone());
            }
        }
        None
    }
}

pub struct TypedChildren<'a, T: CoreType> {
    children: Children<'a>,
    marker: PhantomData<T>,
}

impl<'a, T: CoreType> TypedChildren<'a, T> {
    pub fn new(children: &'a [Option<CoreHandle>]) -> Self {
        Self {
            children: Children::Nullable(children),
            marker: PhantomData,
        }
    }

    pub fn from_handles(children: &'a [CoreHandle]) -> Self {
        Self {
            children: Children::NonNull(children),
            marker: PhantomData,
        }
    }

    pub fn iter(&self) -> TypedChild<'a, T> {
        TypedChild {
            children: self.children,
            index: 0,
            marker: PhantomData,
        }
    }

    pub fn first(&self) -> Option<CoreHandle> {
        self.iter().next()
    }

    pub fn size(&self) -> usize {
        self.iter().count()
    }
}
