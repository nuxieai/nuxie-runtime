use crate::focus::RuntimeFocusTree;

#[derive(Debug, Clone)]
pub(crate) struct RuntimeFocusActionClear {
    pub(crate) action_owner: super::RuntimeActionCoreHandle,
}

impl RuntimeFocusActionClear {
    #[cfg(test)]
    pub(crate) fn for_test(flags: u64) -> Self {
        let action_owner = super::RuntimeActionCoreHandle::for_test("FocusActionClear");
        action_owner.set_uint(super::listener_action_owner::LISTENER_FLAGS_KEY, flags);
        Self { action_owner }
    }

    /// Pinned C++ `FocusActionClear::perform` ignores the invocation and
    /// clears only through the current occurrence's focus manager.
    pub(crate) fn perform(&self, focus: &mut RuntimeFocusTree) -> bool {
        focus.clear_focus()
    }
}
