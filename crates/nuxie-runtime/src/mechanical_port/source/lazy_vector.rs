pub struct LazyVector<T> {
    values: Option<Vec<T>>,
}

impl<T> Default for LazyVector<T> {
    fn default() -> Self {
        Self { values: None }
    }
}

impl<T: Clone> Clone for LazyVector<T> {
    fn clone(&self) -> Self {
        Self {
            values: self.values.clone(),
        }
    }
}

impl<T: PartialEq> LazyVector<T> {
    pub fn empty(&self) -> bool {
        self.values.as_ref().is_none_or(Vec::is_empty)
    }

    pub fn size(&self) -> usize {
        self.values.as_ref().map_or(0, Vec::len)
    }

    pub fn push_back(&mut self, value: T) {
        self.values.get_or_insert_with(Vec::new).push(value);
    }

    pub fn push_unique(&mut self, value: T) {
        if self
            .values
            .as_ref()
            .is_some_and(|values| values.contains(&value))
        {
            return;
        }
        self.push_back(value);
    }

    pub fn erase_all(&mut self, value: &T) {
        if let Some(values) = self.values.as_mut() {
            values.retain(|candidate| candidate != value);
        }
    }

    pub fn clear(&mut self) {
        if let Some(values) = self.values.as_mut() {
            values.clear();
        }
    }

    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.view().iter()
    }

    pub fn view(&self) -> &[T] {
        self.values.as_deref().unwrap_or(&[])
    }
}
