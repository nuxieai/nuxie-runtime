//! Public focus API facade over the direct retained focus owners.

pub(crate) use crate::focus_data::RuntimeFocusTree;
pub use crate::input::{
    FocusBounds, FocusDirection, FocusEdgeBehavior, FocusEvent, FocusEventKind, FocusManager,
    FocusNode, FocusNodeId, FocusPoint,
};
