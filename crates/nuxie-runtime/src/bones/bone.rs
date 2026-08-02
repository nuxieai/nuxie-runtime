use nuxie_schema::definition_by_name;

use crate::components::ComponentHandle;

#[derive(Debug, Clone)]
pub(crate) struct RuntimeBoneState {
    /// Concrete C++ subtype identity used by the `Bone::x/y` versus
    /// `RootBoneBase::x/y` virtual dispatch.
    pub(crate) is_root: bool,
    pub(crate) child_bones: Vec<ComponentHandle>,
    pub(crate) peer_constraints: Vec<ComponentHandle>,
}

impl RuntimeBoneState {
    pub(crate) fn for_type(type_name: &'static str) -> Option<Self> {
        (type_name == "Bone"
            || definition_by_name(type_name).is_some_and(|definition| definition.is_a("Bone")))
        .then(|| Self::new(super::root_bone::is_root_bone(type_name)))
    }

    fn new(is_root: bool) -> Self {
        Self {
            is_root,
            child_bones: Vec::new(),
            peer_constraints: Vec::new(),
        }
    }

    pub(crate) fn clone_for_occurrence(&self) -> Self {
        Self::new(self.is_root)
    }
}
