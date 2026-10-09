use std::rc::Rc;

/// Upstream's nullable owning vector. Only read views share backing storage;
/// cloning the owner itself still deep-copies its list.
pub struct LazyVector<T> {
    values: Option<Rc<Vec<T>>>,
}
impl<T> Default for LazyVector<T> {
    fn default() -> Self {
        Self { values: None }
    }
}
impl<T: Clone> Clone for LazyVector<T> {
    fn clone(&self) -> Self {
        Self {
            values: self
                .values
                .as_ref()
                .map(|values| Rc::new((**values).clone())),
        }
    }
}
impl<T> LazyVector<T> {
    pub fn empty(&self) -> bool {
        self.view().is_empty()
    }
    pub fn size(&self) -> usize {
        self.view().len()
    }
    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.view().iter()
    }
    pub fn view(&self) -> &[T] {
        self.values.as_ref().map_or(&[], |values| values.as_slice())
    }
    /// Retain the existing list across a released owner borrow, without copying
    /// its entries. A never-allocated owner remains allocation-free.
    pub fn snapshot(&self) -> LazyVectorSnapshot<T> {
        LazyVectorSnapshot {
            values: self.values.clone(),
            index: 0,
        }
    }
}
impl<T: Clone> LazyVector<T> {
    fn writable(&mut self) -> &mut Vec<T> {
        let values = self.values.get_or_insert_with(|| Rc::new(Vec::new()));
        if Rc::get_mut(values).is_none() {
            // Reentrant mutation must preserve an outstanding traversal's order.
            // Preserve capacity even on this exceptional copy, including clear.
            let mut copy = Vec::with_capacity(values.capacity());
            copy.extend(values.iter().cloned());
            *values = Rc::new(copy);
        }
        Rc::get_mut(values).expect("the writable list is uniquely owned")
    }
    pub fn push_back(&mut self, value: T) {
        self.writable().push(value);
    }
    /// Remove exactly one position, retaining the remaining order. A live
    /// snapshot keeps its prior backing, just as for the other mutations.
    pub fn remove(&mut self, index: usize) -> T {
        self.writable().remove(index)
    }
    pub fn clear(&mut self) {
        if self.values.is_some() {
            self.writable().clear();
        }
    }
}
impl<T: Clone + PartialEq> LazyVector<T> {
    pub fn push_unique(&mut self, value: T) {
        if !self.view().contains(&value) {
            self.push_back(value);
        }
    }
    pub fn erase_all(&mut self, value: &T) {
        if self.values.is_some() {
            self.writable().retain(|candidate| candidate != value);
        }
    }
}

/// Safe lifetime adaptation for callbacks that re-enter their list owner.
/// Retains vector storage, not the Components represented by its weak entries.
pub struct LazyVectorSnapshot<T> {
    values: Option<Rc<Vec<T>>>,
    index: usize,
}
impl<T> Default for LazyVectorSnapshot<T> {
    fn default() -> Self {
        Self {
            values: None,
            index: 0,
        }
    }
}
impl<T> LazyVectorSnapshot<T> {
    /// Borrow the unconsumed entries from the retained backing. Reentrant owner
    /// edits copy that backing, so these references keep the traversal's order
    /// without cloning each weak occurrence on the way to its callback.
    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.values
            .as_ref()
            .map_or(&[][..], |values| &values[self.index..])
            .iter()
    }
}
impl<T: Clone> Iterator for LazyVectorSnapshot<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        let value = self.values.as_ref()?.get(self.index)?.clone();
        self.index += 1;
        Some(value)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self
            .values
            .as_ref()
            .map_or(0, |values| values.len() - self.index);
        (len, Some(len))
    }
}
impl<T: Clone> ExactSizeIterator for LazyVectorSnapshot<T> {}
impl<T: Clone> std::iter::FusedIterator for LazyVectorSnapshot<T> {}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn nullable_owner_is_pointer_sized_and_reading_does_not_allocate() {
        assert_eq!(
            std::mem::size_of::<LazyVector<u32>>(),
            std::mem::size_of::<usize>()
        );
        let mut values = LazyVector::<u32>::default();
        assert!(values.empty());
        assert_eq!(values.size(), 0);
        assert_eq!(values.iter().count(), 0);
        assert_eq!(values.snapshot().len(), 0);
        values.clear();
        values.erase_all(&1);
        assert!(values.values.is_none());
        assert!(values.clone().values.is_none());
    }
    #[test]
    fn duplicates_order_erase_all_and_clear_preserve_source_semantics() {
        let mut values = LazyVector::default();
        for value in [3, 1, 3, 2] {
            values.push_back(value);
        }
        values.push_unique(3);
        values.push_unique(4);
        assert_eq!(values.view(), &[3, 1, 3, 2, 4]);
        let capacity = values.values.as_ref().unwrap().capacity();
        values.erase_all(&3);
        assert_eq!(values.view(), &[1, 2, 4]);
        assert_eq!(values.values.as_ref().unwrap().capacity(), capacity);
        values.clear();
        assert!(values.empty());
        assert_eq!(values.values.as_ref().unwrap().capacity(), capacity);
        values.push_back(9);
        assert_eq!(values.view(), &[9]);
    }
    #[test]
    fn owner_clone_is_deep_but_read_view_retains_the_same_backing() {
        let mut values = LazyVector::default();
        values.push_back(1);
        values.push_back(2);
        let copy = values.clone();
        assert!(!Rc::ptr_eq(
            values.values.as_ref().unwrap(),
            copy.values.as_ref().unwrap()
        ));
        let mut snapshot = values.snapshot();
        assert!(Rc::ptr_eq(
            values.values.as_ref().unwrap(),
            snapshot.values.as_ref().unwrap()
        ));
        let capacity = values.values.as_ref().unwrap().capacity();
        values.erase_all(&1);
        values.push_back(3);
        assert_eq!(values.view(), &[2, 3]);
        assert_eq!(copy.view(), &[1, 2]);
        assert_eq!(snapshot.next(), Some(1));
        values.clear();
        assert_eq!(values.values.as_ref().unwrap().capacity(), capacity);
        assert_eq!(snapshot.next(), Some(2));
        assert_eq!(snapshot.len(), 0);
        assert_eq!(snapshot.next(), None);
        assert_eq!(snapshot.next(), None);
    }
    #[test]
    fn moving_owner_transfers_backing_and_leaves_taken_owner_null() {
        let mut values = LazyVector::default();
        values.push_back(7);
        let snapshot = values.snapshot();
        let moved = std::mem::take(&mut values);
        assert!(values.values.is_none());
        assert!(Rc::ptr_eq(
            moved.values.as_ref().unwrap(),
            snapshot.values.as_ref().unwrap()
        ));
        assert_eq!(moved.view(), &[7]);
    }

    #[test]
    fn clear_with_outstanding_view_retains_capacity_and_original_items() {
        let mut values = LazyVector::default();
        for value in 0..7 {
            values.push_back(value);
        }
        let capacity = values.values.as_ref().unwrap().capacity();
        let snapshot = values.snapshot();
        values.clear();
        assert!(values.empty());
        assert_eq!(values.values.as_ref().unwrap().capacity(), capacity);
        assert_eq!(snapshot.collect::<Vec<_>>(), (0..7).collect::<Vec<_>>());
        assert!(values.clone().values.is_some());
    }
    #[test]
    fn remove_keeps_duplicate_positions_and_outstanding_snapshot_order() {
        let mut values = LazyVector::default();
        for value in [7, 3, 7, 5] {
            values.push_back(value);
        }
        let snapshot = values.snapshot();
        assert_eq!(values.remove(0), 7);
        assert_eq!(values.view(), &[3, 7, 5]);
        assert_eq!(snapshot.iter().copied().collect::<Vec<_>>(), [7, 3, 7, 5]);
        assert_eq!(values.remove(1), 7);
        assert_eq!(values.view(), &[3, 5]);
    }
}
