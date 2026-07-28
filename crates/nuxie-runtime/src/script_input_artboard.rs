//! Pinned `src/script_input_artboard.cpp` occurrence semantics.

use crate::data_bind_graph::RuntimeDataBindGraphValue;
use nuxie_binary::{RuntimeFile, RuntimeObject};

/// Handwritten state that sits beside generated `ScriptInputArtboard::artboardId`.
///
/// C++ deliberately keeps the generated integer and the resolved Artboard
/// pointer separate. A ViewModel Artboard source replaces only the pointer,
/// while a normal generated-property write changes the integer and then asks
/// the retained File to resolve a new pointer (`script_input_artboard.cpp:
/// 102-135`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RuntimeScriptInputArtboardOccurrence {
    referenced_artboard_id: Option<u64>,
    file_attached: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RuntimeScriptInputArtboardApply {
    Rejected,
    ChangedWithoutProjection,
    Project(u64),
}

impl RuntimeScriptInputArtboardOccurrence {
    pub(crate) fn from_imported(file: &RuntimeFile, input: &RuntimeObject) -> Self {
        let authored_id = input
            .uint_property("artboardId")
            .unwrap_or(u64::from(u32::MAX));
        Self {
            referenced_artboard_id: file
                .resolved_artboard_for_referencer_object(input)
                .map(|_| authored_id),
            // `File::import` installs this before Backboard resolution,
            // including when the authored id cannot resolve.
            file_attached: true,
        }
    }

    #[cfg(test)]
    pub(crate) fn for_test(referenced_artboard_id: Option<u64>, file_attached: bool) -> Self {
        Self {
            referenced_artboard_id,
            file_attached,
        }
    }

    /// `ScriptInputArtboard::clone` copies generated Core fields through its
    /// base clone, but copies both `m_referencedArtboard` and `m_file` only
    /// when the source pointer is non-null.
    pub(crate) fn clone_for_scripted_object(&self) -> Self {
        Self {
            referenced_artboard_id: self.referenced_artboard_id,
            file_attached: self.file_attached && self.referenced_artboard_id.is_some(),
        }
    }

    pub(crate) fn referenced_artboard_id(&self) -> Option<u64> {
        self.referenced_artboard_id
    }

    #[cfg(test)]
    pub(crate) fn file_attached(&self) -> bool {
        self.file_attached
    }

    /// Apply `DataBindContextValueArtboard` to an ArtboardReferencer.
    ///
    /// A failed lookup preserves the prior reference. A successful lookup
    /// projects fresh userdata even when it resolves to the same artboard as
    /// the prior update (`context_value_artboard.cpp:14-29`;
    /// `script_input_artboard.cpp:123-133`).
    pub(crate) fn apply_artboard_source(
        &mut self,
        file: &RuntimeFile,
        artboard_id: u64,
    ) -> RuntimeScriptInputArtboardApply {
        if !self.file_attached || !file_contains_artboard(file, artboard_id) {
            return RuntimeScriptInputArtboardApply::Rejected;
        }
        self.referenced_artboard_id = Some(artboard_id);
        RuntimeScriptInputArtboardApply::Project(artboard_id)
    }

    /// Apply the generated `artboardIdChanged` callback after the generated
    /// field itself has changed.
    ///
    /// Unlike `updateArtboard`, an attached File replaces the retained
    /// reference with null when the new id cannot resolve. A clone that did
    /// not retain `m_file` changes only its generated integer.
    pub(crate) fn apply_artboard_id_changed(
        &mut self,
        file: &RuntimeFile,
        artboard_id: u64,
    ) -> RuntimeScriptInputArtboardApply {
        if !self.file_attached {
            return RuntimeScriptInputArtboardApply::ChangedWithoutProjection;
        }
        if file_contains_artboard(file, artboard_id) {
            self.referenced_artboard_id = Some(artboard_id);
            RuntimeScriptInputArtboardApply::Project(artboard_id)
        } else {
            self.referenced_artboard_id = None;
            RuntimeScriptInputArtboardApply::ChangedWithoutProjection
        }
    }
}

pub(crate) fn value_property_key() -> Option<u16> {
    crate::properties::property_key_for_name("ScriptInputArtboard", "artboardId")
}

pub(crate) fn authored_target(
    input: &RuntimeObject,
    property_key: u32,
) -> Option<RuntimeDataBindGraphValue> {
    (value_property_key().map(u32::from) == Some(property_key)).then(|| {
        RuntimeDataBindGraphValue::Artboard(
            input
                .uint_property("artboardId")
                .unwrap_or(u64::from(u32::MAX)),
        )
    })
}

fn file_contains_artboard(file: &RuntimeFile, artboard_id: u64) -> bool {
    usize::try_from(artboard_id)
        .ok()
        .and_then(|index| file.artboard(index))
        .is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fresh_clone_copies_file_authority_only_with_a_resolved_reference() {
        let resolved = RuntimeScriptInputArtboardOccurrence::for_test(Some(3), true);
        let resolved_clone = resolved.clone_for_scripted_object();
        assert_eq!(resolved_clone.referenced_artboard_id(), Some(3));
        assert!(resolved_clone.file_attached());

        let unresolved = RuntimeScriptInputArtboardOccurrence::for_test(None, true);
        let unresolved_clone = unresolved.clone_for_scripted_object();
        assert_eq!(unresolved_clone.referenced_artboard_id(), None);
        assert!(!unresolved_clone.file_attached());
    }
}
