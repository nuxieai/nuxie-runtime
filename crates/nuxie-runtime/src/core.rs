// Direct Rust owner for pinned C++ `src/core.cpp` property observers.

use std::collections::BTreeMap;

/// Property-key observers retained by arena-owned Core occurrences.
///
/// C++ stores an intrusive observer list on each `Core`. Rust's imported Core
/// objects live in one dense arena, so `(local_id, property_key)` is the
/// structure-preserving owner key. Registration is construction/lifecycle
/// work; generated setters only take the empty fast path and visit the exact
/// property's retained observers.
#[derive(Debug, Clone)]
pub(crate) struct RuntimeCorePropertyObservers<T> {
    by_core_property: BTreeMap<(usize, u16), Vec<T>>,
}

impl<T> Default for RuntimeCorePropertyObservers<T> {
    fn default() -> Self {
        Self {
            by_core_property: BTreeMap::new(),
        }
    }
}

impl<T: PartialEq> RuntimeCorePropertyObservers<T> {
    pub(crate) fn add_property_observer(
        &mut self,
        local_id: usize,
        property_key: u16,
        observer: T,
    ) {
        let observers = self
            .by_core_property
            .entry((local_id, property_key))
            .or_default();
        debug_assert!(
            !observers.contains(&observer),
            "Core property observer must be removed before it is added again"
        );
        if observers.contains(&observer) {
            return;
        }
        observers.push(observer);
    }

    pub(crate) fn observes_property(&self, local_id: usize, property_key: u16) -> bool {
        self.by_core_property
            .contains_key(&(local_id, property_key))
    }

    #[inline]
    pub(crate) fn notify_property_changed(
        &self,
        local_id: usize,
        property_key: u16,
        mut notify: impl FnMut(&T),
    ) -> bool {
        if self.by_core_property.is_empty() {
            return false;
        }
        let Some(observers) = self.by_core_property.get(&(local_id, property_key)) else {
            return false;
        };
        for observer in observers {
            notify(observer);
        }
        true
    }

    pub(crate) fn remove_property_observer(
        &mut self,
        local_id: usize,
        property_key: u16,
        observer: &T,
    ) -> bool {
        let key = (local_id, property_key);
        let Some(observers) = self.by_core_property.get_mut(&key) else {
            return false;
        };
        let Some(index) = observers.iter().position(|candidate| candidate == observer) else {
            return false;
        };
        observers.remove(index);
        if observers.is_empty() {
            self.by_core_property.remove(&key);
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::RuntimeCorePropertyObservers;

    #[test]
    fn property_observers_use_an_empty_fast_path_and_exact_property_fanout() {
        let mut observers = RuntimeCorePropertyObservers::default();
        let mut notified = Vec::new();

        assert!(!observers.notify_property_changed(7, 11, |observer| notified.push(*observer)));
        observers.add_property_observer(7, 11, 1_u8);
        observers.add_property_observer(7, 11, 2_u8);
        observers.add_property_observer(7, 12, 3_u8);
        observers.add_property_observer(8, 11, 4_u8);

        assert!(observers.notify_property_changed(7, 11, |observer| notified.push(*observer)));
        assert_eq!(notified, vec![1, 2]);
        assert!(observers.observes_property(7, 11));
        assert!(!observers.observes_property(9, 11));
    }

    #[test]
    fn removing_an_observer_prunes_the_core_property_entry() {
        let mut observers = RuntimeCorePropertyObservers::default();
        observers.add_property_observer(7, 11, 1_u8);
        observers.add_property_observer(7, 11, 2_u8);

        assert!(observers.remove_property_observer(7, 11, &1));
        assert!(observers.observes_property(7, 11));
        assert!(observers.remove_property_observer(7, 11, &2));
        assert!(!observers.observes_property(7, 11));
    }
}
