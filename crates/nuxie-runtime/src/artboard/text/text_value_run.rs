use super::super::ArtboardInstance;
use crate::properties::property_key_for_name;
use std::collections::BTreeSet;

/// RAII transaction for exact-name root text-run writes on one occurrence.
/// Payload writes are staged directly in the existing object cells; every
/// invalidation is deferred until commit, while drop restores the original
/// bytes silently and without schema lookup.
pub struct RuntimeTextRunTransaction<'a> {
    artboard: &'a mut ArtboardInstance,
    text_property_key: u16,
    originals: Vec<(usize, Vec<u8>)>,
    notifications: Vec<usize>,
    armed: bool,
}

impl RuntimeTextRunTransaction<'_> {
    pub fn set(&mut self, name: &str, value: Vec<u8>) -> Option<bool> {
        let local_id = self.artboard.root_text_value_run_local_id(name)?;
        if self
            .artboard
            .string_property(local_id, self.text_property_key)
            == Some(value.as_slice())
        {
            return Some(false);
        }
        let changed =
            self.artboard
                .objects
                .set_string_property(local_id, self.text_property_key, value);
        debug_assert!(changed, "validated text-run transaction write");
        self.notifications.push(local_id);
        Some(changed)
    }

    /// All operations below are runtime-owned dirt bookkeeping; host code is
    /// never invoked inline. Publishing them only after the complete batch is
    /// staged makes rollback observationally silent.
    pub fn commit(mut self) {
        for local_id in self.notifications.drain(..) {
            self.artboard
                .apply_string_property_changed(local_id, self.text_property_key);
            self.artboard
                .notify_artboard_data_bind_target_property_changed(
                    local_id,
                    self.text_property_key,
                );
            self.artboard
                .mark_stateful_nested_view_model_contexts_dirty_for_local(local_id);
            self.artboard
                .mark_changed_unless_view_model_instance(local_id);
            self.artboard.mark_text_changed_for_local(local_id);
            self.artboard
                .mark_prepared_changed_for_property(local_id, self.text_property_key);
            self.artboard
                .refresh_retained_focusables_for_property(local_id, self.text_property_key);
        }
        self.originals.clear();
        self.armed = false;
    }
}

impl Drop for RuntimeTextRunTransaction<'_> {
    fn drop(&mut self) {
        if !self.armed {
            return;
        }
        for (local_id, text) in self.originals.drain(..).rev() {
            let _ =
                self.artboard
                    .objects
                    .set_string_property(local_id, self.text_property_key, text);
        }
        self.notifications.clear();
        self.armed = false;
    }
}

impl ArtboardInstance {
    /// Resolve and retain every exact root text-run cell before the first
    /// staged write. Duplicate names share one inverse but retain mutation
    /// order in the transaction's deferred notification list.
    pub fn root_text_value_run_transaction(
        &mut self,
        names: &[String],
    ) -> Option<RuntimeTextRunTransaction<'_>> {
        let text_property_key = property_key_for_name("TextValueRun", "text")?;
        let mut seen = BTreeSet::new();
        let mut originals = Vec::new();
        for name in names {
            if !seen.insert(name.as_str()) {
                continue;
            }
            let local_id = self.root_text_value_run_local_id(name)?;
            originals.push((
                local_id,
                self.string_property(local_id, text_property_key)?.to_vec(),
            ));
        }
        Some(RuntimeTextRunTransaction {
            artboard: self,
            text_property_key,
            originals,
            notifications: Vec::new(),
            armed: true,
        })
    }

    /// Set the first root-artboard `TextValueRun` with the exact authored
    /// component name. Resolution follows component/local order and does not
    /// traverse nested artboards.
    ///
    /// `None` means no matching root text run exists. `Some(false)` means the
    /// existing run already contains `value`; `Some(true)` means it changed.
    pub fn set_root_text_value_run(&mut self, name: &str, value: Vec<u8>) -> Option<bool> {
        let text_property_key = property_key_for_name("TextValueRun", "text")?;
        let local_id = self.root_text_value_run_local_id(name)?;
        if self.string_property(local_id, text_property_key) == Some(value.as_slice()) {
            return Some(false);
        }
        Some(self.set_string_property(local_id, text_property_key, value))
    }

    /// Whether this root artboard contains an exactly named `TextValueRun`.
    /// Nested-artboard occurrences are deliberately outside this lookup.
    pub fn has_root_text_value_run(&self, name: &str) -> bool {
        self.root_text_value_run_local_id(name).is_some()
    }

    /// Read the first root-artboard `TextValueRun` with the exact authored
    /// component name. Like the setter, this deliberately does not traverse
    /// nested-artboard occurrences.
    pub fn root_text_value_run(&self, name: &str) -> Option<&[u8]> {
        let text_property_key = property_key_for_name("TextValueRun", "text")?;
        let local_id = self.root_text_value_run_local_id(name)?;
        self.string_property(local_id, text_property_key)
    }

    fn root_text_value_run_local_id(&self, name: &str) -> Option<usize> {
        self.slots
            .iter()
            .filter(|slot| {
                slot.type_name == Some("TextValueRun") && slot.name.as_deref() == Some(name)
            })
            .min_by_key(|slot| slot.local_id)
            .map(|slot| slot.local_id)
    }
}
