use nuxie_graph::AdvancingComponentKind;

use crate::components::ComponentHandle;
use crate::objects::ObjectHandle;

/// Rust's retained interface record for pinned C++ `AdvancingComponent`.
///
/// `ArtboardInstance` owns these in authored object order and dispatches each
/// `advanceComponent(elapsedSeconds, flags)` through `kind`. The Rust advance
/// loop carries `Animate` as its normal path, `NewFrame` as the explicit
/// `new_frame` argument, and `AdvanceNested` at the nested-artboard call site.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct RuntimeAdvancingComponent {
    pub(crate) local_id: usize,
    pub(crate) object: ObjectHandle,
    pub(crate) component: Option<ComponentHandle>,
    pub(crate) kind: AdvancingComponentKind,
}

impl RuntimeAdvancingComponent {
    /// Mechanical translation of pinned `AdvancingComponent::from(Core*)`.
    ///
    /// The `ObjectHandle` is the Rust `Core*` identity. Most concrete owners
    /// also have a `ComponentHandle`; `ScriptedDataConverter` deliberately
    /// does not because the upstream switch accepts `Core`, not `Component`.
    pub(crate) fn from(
        local_id: usize,
        type_name: &str,
        object: ObjectHandle,
        component: Option<ComponentHandle>,
    ) -> Option<Self> {
        let kind = match type_name {
            "NestedArtboardLeaf" => AdvancingComponentKind::NestedArtboard,
            "NestedArtboardLayout" => AdvancingComponentKind::NestedArtboard,
            "NestedArtboard" => AdvancingComponentKind::NestedArtboard,
            "LayoutComponent" => AdvancingComponentKind::LayoutComponent,
            "LayoutParticipant" => AdvancingComponentKind::LayoutParticipant,
            "Artboard" => AdvancingComponentKind::Artboard,
            "ArtboardComponentList" => AdvancingComponentKind::ArtboardComponentList,
            "ScrollConstraint" => AdvancingComponentKind::ScrollConstraint,
            "TextInput" => AdvancingComponentKind::TextInput,
            "ScriptedDataConverter" => AdvancingComponentKind::ScriptedDataConverter,
            "ScriptedDrawable" => AdvancingComponentKind::ScriptedDrawable,
            "ScriptedLayout" => AdvancingComponentKind::ScriptedLayout,
            "ScriptedPathEffect" => AdvancingComponentKind::ScriptedPathEffect,
            _ => return None,
        };
        Some(Self {
            local_id,
            object,
            component,
            kind,
        })
    }
}
