use nuxie_binary::RuntimeObject;

/// Authored semantic-action constraint owned by ListenerInputTypeSemantic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RuntimeSemanticInput {
    pub(crate) global_id: u32,
    pub(super) action_type: u32,
}

impl RuntimeSemanticInput {
    pub(super) fn from_imported(object: &RuntimeObject) -> Self {
        Self {
            global_id: object.id,
            action_type: object.uint_property("actionType").unwrap_or(0) as u32,
        }
    }
}
