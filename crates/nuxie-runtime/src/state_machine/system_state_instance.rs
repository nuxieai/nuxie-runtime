/// Mutable occurrence for Any, Entry, Exit, and generic `LayerState`
/// definitions.
///
/// Pinned C++ `SystemStateInstance` retains only its definition identity.
/// Advance and apply are deliberate no-ops and never keep the frame alive.
#[derive(Debug, Clone)]
pub(super) struct RuntimeSystemStateInstance;

impl RuntimeSystemStateInstance {
    pub(super) fn advance(&mut self) -> bool {
        false
    }

    pub(super) fn apply(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_state_occurrence_is_a_complete_noop() {
        let mut occurrence = RuntimeSystemStateInstance;
        assert!(!occurrence.advance());
        assert!(!occurrence.apply());
    }
}
