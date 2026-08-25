//! Pinned `src/script_input_artboard.cpp` occurrence semantics.

use crate::RuntimeBindableArtboard;
use crate::artboard::RuntimeArtboardAncestorSources;
use crate::data_bind_graph::RuntimeDataBindGraphValue;
use nuxie_binary::{RuntimeFile, RuntimeObject};

/// Exact retained target of C++ `ArtboardReferencer::m_referencedArtboard`.
/// The generated `artboardId` is deliberately not part of this identity.
#[doc(hidden)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScriptArtboardSource {
    File(u64),
    Live(RuntimeBindableArtboard),
}

/// Handwritten state that sits beside generated `ScriptInputArtboard::artboardId`.
///
/// C++ deliberately keeps the generated integer and the resolved Artboard
/// pointer separate. A ViewModel Artboard source replaces only the pointer,
/// while a normal generated-property write changes the integer and then asks
/// the retained File to resolve a new pointer (`script_input_artboard.cpp:
/// 102-135`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct RuntimeScriptInputArtboardOccurrence {
    referenced_artboard: Option<ScriptArtboardSource>,
    file_attached: bool,
    ancestor_sources: Option<RuntimeArtboardAncestorSources>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum RuntimeScriptInputArtboardApply {
    Rejected,
    ChangedWithoutProjection,
    Project(ScriptArtboardSource),
}

impl RuntimeScriptInputArtboardOccurrence {
    pub(crate) fn from_imported(file: &RuntimeFile, input: &RuntimeObject) -> Self {
        let authored_id = input
            .uint_property("artboardId")
            .unwrap_or(u64::from(u32::MAX));
        Self {
            referenced_artboard: file
                .resolved_artboard_for_referencer_object(input)
                .map(|_| ScriptArtboardSource::File(authored_id)),
            // `File::import` installs this before Backboard resolution,
            // including when the authored id cannot resolve.
            file_attached: true,
            ancestor_sources: None,
        }
    }

    #[cfg(test)]
    pub(crate) fn for_test(referenced_artboard_id: Option<u64>, file_attached: bool) -> Self {
        Self {
            referenced_artboard: referenced_artboard_id.map(ScriptArtboardSource::File),
            file_attached,
            ancestor_sources: None,
        }
    }

    pub(crate) fn set_ancestor_sources(&mut self, sources: RuntimeArtboardAncestorSources) {
        self.ancestor_sources = Some(sources);
    }

    /// `ScriptInputArtboard::clone` copies generated Core fields through its
    /// base clone, but copies both `m_referencedArtboard` and `m_file` only
    /// when the source pointer is non-null.
    pub(crate) fn clone_for_scripted_object(&self) -> Self {
        Self {
            referenced_artboard: self.referenced_artboard.clone(),
            file_attached: self.file_attached && self.referenced_artboard.is_some(),
            ancestor_sources: self.ancestor_sources.clone(),
        }
    }

    pub(crate) fn referenced_artboard(&self) -> Option<&ScriptArtboardSource> {
        self.referenced_artboard.as_ref()
    }

    pub(crate) fn referenced_artboard_id(&self) -> Option<u64> {
        match self.referenced_artboard.as_ref()? {
            ScriptArtboardSource::File(id) => Some(*id),
            ScriptArtboardSource::Live(_) => None,
        }
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
        runtime_artboard: Option<RuntimeBindableArtboard>,
    ) -> RuntimeScriptInputArtboardApply {
        if let Some(runtime_artboard) = runtime_artboard {
            let Some(candidate) = runtime_artboard.artboard_instance() else {
                return RuntimeScriptInputArtboardApply::Rejected;
            };
            if self
                .ancestor_sources
                .as_ref()
                .is_some_and(|ancestors| ancestors.rejects(&candidate))
            {
                return RuntimeScriptInputArtboardApply::Rejected;
            }
            let source = ScriptArtboardSource::Live(runtime_artboard);
            self.referenced_artboard = Some(source.clone());
            return RuntimeScriptInputArtboardApply::Project(source);
        }
        let Some(candidate) = file_artboard(file, artboard_id) else {
            return RuntimeScriptInputArtboardApply::Rejected;
        };
        if !self.file_attached
            || self
                .ancestor_sources
                .as_ref()
                .is_some_and(|ancestors| ancestors.rejects_file_artboard_global(candidate.id))
        {
            return RuntimeScriptInputArtboardApply::Rejected;
        }
        let source = ScriptArtboardSource::File(artboard_id);
        self.referenced_artboard = Some(source.clone());
        RuntimeScriptInputArtboardApply::Project(source)
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
            let source = ScriptArtboardSource::File(artboard_id);
            self.referenced_artboard = Some(source.clone());
            RuntimeScriptInputArtboardApply::Project(source)
        } else {
            self.referenced_artboard = None;
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
    file_artboard(file, artboard_id).is_some()
}

fn file_artboard(file: &RuntimeFile, artboard_id: u64) -> Option<&RuntimeObject> {
    usize::try_from(artboard_id)
        .ok()
        .and_then(|index| file.artboard(index))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ArtboardInstance;
    use nuxie_binary::{FixtureProperty, FixtureRecord, FixtureValue};

    fn record(type_name: &str, properties: Vec<FixtureProperty>) -> FixtureRecord {
        FixtureRecord {
            type_key: nuxie_schema::definition_by_name(type_name)
                .expect("fixture type")
                .type_key
                .int,
            properties,
        }
    }

    fn property(type_name: &str, name: &str, value: FixtureValue) -> FixtureProperty {
        FixtureProperty {
            key: crate::properties::property_key_for_name(type_name, name)
                .expect("fixture property"),
            value,
        }
    }

    fn artboard_fixture() -> (RuntimeFile, nuxie_graph::GraphFile) {
        let file = RuntimeFile::from_fixture_records(vec![
            record("Backboard", Vec::new()),
            record(
                "Artboard",
                vec![
                    property("Artboard", "width", FixtureValue::Double(100.0)),
                    property("Artboard", "height", FixtureValue::Double(100.0)),
                ],
            ),
            record(
                "Artboard",
                vec![
                    property("Artboard", "width", FixtureValue::Double(50.0)),
                    property("Artboard", "height", FixtureValue::Double(50.0)),
                ],
            ),
            record(
                "Artboard",
                vec![
                    property("Artboard", "width", FixtureValue::Double(25.0)),
                    property("Artboard", "height", FixtureValue::Double(25.0)),
                ],
            ),
        ])
        .expect("artboard fixture imports");
        let graphs = nuxie_graph::GraphFile::from_runtime_file(&file)
            .expect("artboard fixture graph builds");
        (file, graphs)
    }

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

    #[test]
    fn live_asset_is_preferred_and_preserves_generated_artboard_id() {
        let (file, graphs) = artboard_fixture();
        let parent = ArtboardInstance::from_graph_with_artboards(
            &file,
            &graphs.artboards[0],
            &graphs.artboards,
        )
        .expect("parent builds");
        let mut live = ArtboardInstance::from_graph_with_artboards(
            &file,
            &graphs.artboards[2],
            &graphs.artboards,
        )
        .expect("live source builds");
        assert!(live.set_artboard_dimensions(73.0, 37.0));
        let bindable = RuntimeBindableArtboard::new_with_artboard_instance("live", &live);
        let mut occurrence = RuntimeScriptInputArtboardOccurrence::for_test(Some(1), true);
        occurrence.set_ancestor_sources(parent.artboard_referencer_ancestor_sources());

        assert_eq!(
            occurrence.apply_artboard_source(&file, 1, Some(bindable.clone())),
            RuntimeScriptInputArtboardApply::Project(ScriptArtboardSource::Live(bindable.clone()))
        );
        let ScriptArtboardSource::Live(retained) = occurrence
            .referenced_artboard()
            .expect("live reference retained")
        else {
            panic!("file id incorrectly replaced the live source");
        };
        assert!(retained.ptr_eq(&bindable));
        assert_eq!(
            retained
                .artboard_instance()
                .expect("live occurrence")
                .artboard_dimensions(),
            (73.0, 37.0)
        );

        let mut properties =
            crate::scripted_object::RuntimeScriptInputProperties::for_test_artboard(
                "panel",
                u32::MAX,
                77,
                Some(1),
                true,
            );
        properties.set_artboard_ancestor_sources(parent.artboard_referencer_ancestor_sources());
        assert_eq!(
            properties.apply_artboard_source(&file, 1, Some(bindable)),
            crate::scripted_object::RuntimeScriptInputTargetApply::ChangedWithTableProjection
        );
        assert_eq!(
            properties.value(),
            Some(&RuntimeDataBindGraphValue::Artboard(77)),
            "referencedArtboardId() remains the generated artboardId; updateArtboard changes only the retained pointer"
        );
    }

    #[test]
    fn ancestor_live_asset_is_rejected_then_absent_asset_uses_file_index() {
        let (file, graphs) = artboard_fixture();
        let parent = ArtboardInstance::from_graph_with_artboards(
            &file,
            &graphs.artboards[0],
            &graphs.artboards,
        )
        .expect("parent builds");
        let ancestor = RuntimeBindableArtboard::new_with_artboard_instance("ancestor", &parent);
        let mut occurrence = RuntimeScriptInputArtboardOccurrence::for_test(Some(1), true);
        occurrence.set_ancestor_sources(parent.artboard_referencer_ancestor_sources());

        assert_eq!(
            occurrence.apply_artboard_source(&file, 2, Some(ancestor)),
            RuntimeScriptInputArtboardApply::Rejected
        );
        assert_eq!(
            occurrence.referenced_artboard(),
            Some(&ScriptArtboardSource::File(1)),
            "a rejected ancestor preserves the old retained pointer"
        );
        assert_eq!(
            occurrence.apply_artboard_source(&file, 2, None),
            RuntimeScriptInputArtboardApply::Project(ScriptArtboardSource::File(2)),
            "numeric File lookup is the fallback only when asset() is null"
        );
    }

    #[test]
    fn fresh_clone_preserves_the_exact_live_bindable_identity() {
        let (file, graphs) = artboard_fixture();
        let parent = ArtboardInstance::from_graph_with_artboards(
            &file,
            &graphs.artboards[0],
            &graphs.artboards,
        )
        .expect("parent builds");
        let live = ArtboardInstance::from_graph_with_artboards(
            &file,
            &graphs.artboards[2],
            &graphs.artboards,
        )
        .expect("live source builds");
        let bindable = RuntimeBindableArtboard::new_with_artboard_instance("live", &live);
        let mut occurrence = RuntimeScriptInputArtboardOccurrence::for_test(Some(1), true);
        occurrence.set_ancestor_sources(parent.artboard_referencer_ancestor_sources());
        assert!(matches!(
            occurrence.apply_artboard_source(&file, 1, Some(bindable.clone())),
            RuntimeScriptInputArtboardApply::Project(ScriptArtboardSource::Live(_))
        ));

        let cloned = occurrence.clone_for_scripted_object();
        let ScriptArtboardSource::Live(cloned_bindable) = cloned
            .referenced_artboard()
            .expect("clone retains live pointer")
        else {
            panic!("clone collapsed the live pointer to an id");
        };
        assert!(cloned_bindable.ptr_eq(&bindable));
        assert!(cloned.file_attached());
    }
}
