use crate::components::{ComponentHandle, Mat2D};

#[derive(Debug, Clone)]
pub(crate) struct RuntimeTendonState {
    pub(crate) inverse_bind: Mat2D,
    pub(crate) bone: Option<ComponentHandle>,
}

impl RuntimeTendonState {
    pub(crate) fn for_type(type_name: &str) -> Option<Self> {
        (type_name == "Tendon").then(Self::default)
    }
}

impl Default for RuntimeTendonState {
    fn default() -> Self {
        Self {
            inverse_bind: Mat2D::IDENTITY,
            bone: None,
        }
    }
}
