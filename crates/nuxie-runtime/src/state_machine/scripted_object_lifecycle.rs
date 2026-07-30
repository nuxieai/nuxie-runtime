//! State-machine ownership of the facade's scripted-object lifecycle token.
//!
//! Pinned C++ receives `StateMachineInstance::internalDataContext` only at an
//! actual bind boundary. Rust's public convenience facade accepts the same
//! root on every advance, so AF-8 requires an identity token rather than
//! translating every wrapper call into a C++ rebind
//! (`state_machine_instance.cpp:2880-2913`).

use super::StateMachineInstance;
use crate::RuntimeOwnedViewModelHandle;
use crate::view_model_cell::RuntimeCellDirt;

impl StateMachineInstance {
    #[doc(hidden)]
    pub fn scripted_object_initialization_complete(&self) -> bool {
        self.scripted_object_initialization_complete
    }

    /// Whether the ordinary public facade crossed a real root-identity
    /// boundary after this concrete occurrence finished its initial scripted
    /// object pass.
    #[doc(hidden)]
    pub fn scripted_facade_root_requires_rebind(
        &self,
        root: Option<&RuntimeOwnedViewModelHandle>,
    ) -> bool {
        if !self.scripted_object_initialization_complete {
            return false;
        }
        match (self.scripted_facade_root_view_model.as_ref(), root) {
            (Some(current), Some(next)) => !current.ptr_eq(next),
            (None, Some(_)) => true,
            // The root-bearing facade is the FL-C4 surface. Clearing or
            // replacing a full DataContext is owned by FL-D.
            (_, None) => false,
        }
    }

    /// Force the next begin-bind call through C++'s clear/install barrier.
    ///
    /// The retained `DataContext` can already contain the same main root while
    /// the facade's root-identity token is still unset (for example, a cold
    /// no-root initialization followed by the first root-bearing call).
    /// `StateMachineInstance::dataContext` still clears listener
    /// registrations and rebinds every fixed ScriptedObject occurrence at
    /// that boundary (`state_machine_instance.cpp:2880-2913,2923-2933`).
    #[doc(hidden)]
    pub fn require_scripted_object_data_context_rebind(&mut self) {
        self.scripted_data_context_bind_complete = false;
    }

    #[doc(hidden)]
    pub fn has_fixed_scripted_data_context_owner(&self) -> bool {
        !self.scripted_object_definitions.is_empty()
            || self.data_bind_graph.has_scripted_converter_occurrence()
    }

    /// Whether Rust's split facade still owes the synchronous C++
    /// `internalDataContext` work for this occurrence.
    ///
    /// C++ never exposes this intermediate state: `dataContext` and
    /// `rebind` clear, bind, hydrate, and initialize every fixed
    /// ScriptedObject/DataConverter before returning. Rust cannot safely run
    /// a low-level callback against the previous context while its owning
    /// facade is between those steps, so direct operations fail closed until
    /// preparation completes (`state_machine_instance.cpp:2880-2933`).
    pub(super) fn scripted_data_context_prepare_pending(&self) -> bool {
        let has_fixed_scripted_owner = self.has_fixed_scripted_data_context_owner();
        if has_fixed_scripted_owner && !self.scripted_object_initialization_complete {
            return true;
        }
        if !has_fixed_scripted_owner || self.owned_data_context.is_none() {
            return false;
        }
        !self.scripted_data_context_bind_complete
            || self
                .owned_view_model_rebind_sink
                .peek_dirt()
                .contains(RuntimeCellDirt::BINDINGS)
    }

    /// Record a successful complete initial mount or live rehydration.
    ///
    /// The handle is identity-only; the mutable ViewModel graph remains owned
    /// by the retained DataContext and is not snapshotted here.
    #[doc(hidden)]
    pub fn mark_scripted_facade_root_hydrated(
        &mut self,
        root: Option<&RuntimeOwnedViewModelHandle>,
    ) {
        self.scripted_facade_root_view_model = root.cloned();
    }

    #[doc(hidden)]
    pub fn mark_scripted_object_initialization_complete(
        &mut self,
        root: Option<&RuntimeOwnedViewModelHandle>,
    ) {
        self.scripted_object_initialization_complete = true;
        self.mark_scripted_facade_root_hydrated(root);
    }
}
