//! ViewModel-property transition comparator ownership.
//!
//! Mirrors pinned C++
//! `src/animation/transition_property_viewmodel_comparator.cpp`.

use super::RuntimeScheduledListenerActionExecutor;
use nuxie_binary::{RuntimeFile, RuntimeObject};

#[derive(Debug, Clone, Copy)]
pub(super) struct RuntimeTransitionPropertyViewModelComparator<'a> {
    comparator: &'a RuntimeObject,
    bindable: &'a RuntimeObject,
}

impl<'a> RuntimeTransitionPropertyViewModelComparator<'a> {
    /// Mechanical counterpart of pinned C++ `import`.
    ///
    /// `latest_bindable_property_for_object` reconstructs the import-stack
    /// lookup and ownership transfer. Rust borrows the imported property from
    /// `RuntimeFile`, so the C++ destructor's delete-and-null body is supplied
    /// by the borrow lifetime rather than a second executable cleanup path.
    pub(super) fn from_object(
        file: &'a RuntimeFile,
        comparator: &'a RuntimeObject,
    ) -> Option<Self> {
        if comparator.type_name != "TransitionPropertyViewModelComparator" {
            return None;
        }
        Some(Self {
            comparator,
            bindable: file.latest_bindable_property_for_object(comparator)?,
        })
    }

    /// Mechanical counterpart of pinned C++ `useInLayer` after Rust's
    /// retained global-id adaptation has replaced bindable/data-bind pointers.
    pub(super) fn use_in_layer(
        bindable_global_id: u32,
        executor: &dyn RuntimeScheduledListenerActionExecutor,
        view_model_trigger_layer_id: u64,
    ) {
        let Some(source) = executor.retained_view_model_source(bindable_global_id) else {
            return;
        };
        source.use_in_layer(view_model_trigger_layer_id);
    }

    /// Mechanical counterpart of pinned C++ `bindableProperty()`.
    pub(super) fn bindable(self) -> &'a RuntimeObject {
        debug_assert_eq!(
            self.comparator.type_name,
            "TransitionPropertyViewModelComparator"
        );
        self.bindable
    }
}

// The header's uncalled `value<T, U>` template is instantiated by Rust at its
// concrete typed evaluation sites: `bindable_*_value` performs the same
// bindable-instance lookup and each caller applies the matching
// `BindableProperty*::defaultValue` when lookup fails. Keeping those live,
// typed paths avoids introducing an unused erased-value facade here.
