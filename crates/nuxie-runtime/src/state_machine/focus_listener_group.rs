use super::listener_types::RuntimeListenerType;
use super::{RuntimeStateMachineListener, ScriptListenerInvocation};
use crate::focus::FocusEventKind;

/// One occurrence of pinned C++ `FocusListenerGroup`.
#[derive(Debug, Clone)]
pub(crate) struct RuntimeFocusListenerGroup {
    pub(crate) listener_index: usize,
    pub(crate) target_local_id: usize,
    pub(crate) focus_data_local_id: usize,
    is_focus_listener: bool,
    is_blur_listener: bool,
}

impl RuntimeFocusListenerGroup {
    pub(crate) fn new(
        listener_index: usize,
        focus_data_local_id: usize,
        listener: &RuntimeStateMachineListener,
    ) -> Option<Self> {
        let is_focus_listener = listener.has_listener(RuntimeListenerType::Focus);
        let is_blur_listener = listener.has_listener(RuntimeListenerType::Blur);
        (is_focus_listener || is_blur_listener).then_some(Self {
            listener_index,
            target_local_id: listener.target_local_id,
            focus_data_local_id,
            is_focus_listener,
            is_blur_listener,
        })
    }

    pub(crate) fn invocation_for(
        &self,
        target_local_id: usize,
        focus_data_local_id: usize,
        kind: FocusEventKind,
    ) -> Option<ScriptListenerInvocation> {
        if target_local_id != self.target_local_id
            || focus_data_local_id != self.focus_data_local_id
        {
            return None;
        }
        let is_focus = match kind {
            FocusEventKind::Focused if self.is_focus_listener => true,
            FocusEventKind::Blurred if self.is_blur_listener => false,
            _ => return None,
        };
        Some(ScriptListenerInvocation::Focus {
            listener_index: self.listener_index,
            is_focus,
        })
    }
}
