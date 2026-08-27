use std::{any::Any, marker::PhantomData};

pub trait TypedCore: Any {
    fn as_any(&self) -> &dyn Any;
}

pub struct TypedChild<'a, T: Any> {
    children: &'a [Option<&'a dyn TypedCore>],
    index: usize,
    marker: PhantomData<T>,
}

impl<'a, T: Any> TypedChild<'a, T> {
    fn new(children: &'a [Option<&'a dyn TypedCore>], index: usize) -> Self {
        Self {
            children,
            index,
            marker: PhantomData,
        }
    }

    fn advance_to_typed(&mut self) {
        while self.index < self.children.len()
            && self.children[self.index].is_none_or(|child| !child.as_any().is::<T>())
        {
            self.index += 1;
        }
    }
}

impl<'a, T: Any> Iterator for TypedChild<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.advance_to_typed();
        if self.index == self.children.len() {
            return None;
        }
        let child = self.children[self.index]
            .expect("typed-child filtering rejects null children")
            .as_any()
            .downcast_ref::<T>()
            .expect("typed-child filtering accepts only the requested type");
        self.index += 1;
        Some(child)
    }
}

pub struct TypedChildren<'a, T: Any> {
    children: &'a [Option<&'a dyn TypedCore>],
    marker: PhantomData<T>,
}

impl<'a, T: Any> TypedChildren<'a, T> {
    pub fn new(children: &'a [Option<&'a dyn TypedCore>]) -> Self {
        Self {
            children,
            marker: PhantomData,
        }
    }

    pub fn iter(&self) -> TypedChild<'a, T> {
        let mut child = TypedChild::new(self.children, 0);
        child.advance_to_typed();
        child
    }

    pub fn first(&self) -> Option<&'a T> {
        self.iter().next()
    }

    pub fn size(&self) -> usize {
        self.iter().count()
    }
}
