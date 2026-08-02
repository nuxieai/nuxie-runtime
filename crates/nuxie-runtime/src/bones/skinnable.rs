use crate::components::ComponentHandle;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RuntimeSkinnableKind {
    PointsPath,
    Mesh,
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeSkinnableState {
    pub(crate) kind: RuntimeSkinnableKind,
    pub(crate) skin: Option<ComponentHandle>,
    pub(crate) vertices: Vec<ComponentHandle>,
}

impl RuntimeSkinnableState {
    pub(crate) fn for_type(type_name: &str) -> Option<Self> {
        match type_name {
            "PointsPath" => Some(Self::new(RuntimeSkinnableKind::PointsPath)),
            "Mesh" => Some(Self::new(RuntimeSkinnableKind::Mesh)),
            _ => None,
        }
    }

    fn new(kind: RuntimeSkinnableKind) -> Self {
        Self {
            kind,
            skin: None,
            vertices: Vec::new(),
        }
    }

    pub(crate) fn clone_for_occurrence(&self) -> Self {
        Self::new(self.kind)
    }
}
