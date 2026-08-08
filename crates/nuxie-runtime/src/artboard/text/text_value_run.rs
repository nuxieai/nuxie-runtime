use super::super::ArtboardInstance;
use crate::properties::property_key_for_name;
use std::collections::BTreeSet;

/// Undo journal for exact-name root text-run writes on one occurrence.
///
/// The journal owns the original bytes and restores through the same runtime
/// setter, so a host can contain a late error or panic without replacing the
/// artboard occurrence or any of its observable identities.
pub struct RuntimeTextRunUndoLog {
    entries: Vec<(String, Vec<u8>)>,
}

impl RuntimeTextRunUndoLog {
    pub fn rollback(self, artboard: &mut ArtboardInstance) -> bool {
        let mut restored = true;
        for (name, text) in self.entries {
            restored &= artboard.set_root_text_value_run(&name, text).is_some();
        }
        restored
    }
}

impl ArtboardInstance {
    /// Capture the original value of each unique exact-name root text run.
    /// Returns `None` before any write when one of the names does not exist.
    pub fn root_text_value_run_undo_log(&self, names: &[String]) -> Option<RuntimeTextRunUndoLog> {
        let mut seen = BTreeSet::new();
        let mut entries = Vec::new();
        for name in names {
            if !seen.insert(name.as_str()) {
                continue;
            }
            entries.push((name.clone(), self.root_text_value_run(name)?.to_vec()));
        }
        Some(RuntimeTextRunUndoLog { entries })
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
