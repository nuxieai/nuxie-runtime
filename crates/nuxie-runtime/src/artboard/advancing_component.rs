use nuxie_graph::AdvancingComponentKind;

use crate::components::ComponentHandle;
use crate::objects::ObjectHandle;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RuntimeAdvancingComponent {
    pub(crate) local_id: usize,
    pub(crate) object: ObjectHandle,
    pub(crate) component: Option<ComponentHandle>,
    pub(crate) kind: AdvancingComponentKind,
}
