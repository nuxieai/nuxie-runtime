use super::super::ArtboardInstance;
use crate::properties::property_key_for_name;

impl ArtboardInstance {
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
