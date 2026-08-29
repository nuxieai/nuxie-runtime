use crate::mechanical_port::source::{advance_flags::AdvanceFlags, core::CoreHandle};

pub trait AdvancingComponent {
    fn advance_component(&mut self, elapsed_seconds: f32, flags: AdvanceFlags) -> bool;
}

/// Retains the actual owner while allowing its borrow to end during callbacks.
/// This is the Rust representation of upstream's AdvancingComponent pointer.
#[derive(Clone)]
pub struct AdvancingComponentHandle {
    owner: CoreHandle,
    advance: fn(&CoreHandle, f32, AdvanceFlags) -> bool,
}

impl AdvancingComponentHandle {
    pub fn classified(component: &CoreHandle) -> Self {
        use crate::mechanical_port::source::generated::{
            artboard_component_list_base::ArtboardComponentListBase,
            layout::layout_participant_base::LayoutParticipantBase,
            nested_artboard_base::NestedArtboardBase,
        };

        let advance = if component.is_type_of(LayoutParticipantBase::TYPE_KEY) {
            advance_layout_participant
        } else if component.is_type_of(NestedArtboardBase::TYPE_KEY) {
            advance_nested_artboard
        } else if component.is_type_of(ArtboardComponentListBase::TYPE_KEY) {
            advance_artboard_component_list
        } else if component
            .with(|owner| owner.as_scripted_drawable().is_some())
            .unwrap_or(false)
        {
            advance_scripted_drawable
        } else if component
            .with(|owner| owner.as_layout_component().is_some())
            .unwrap_or(false)
        {
            advance_layout_component
        } else {
            advance_dynamic
        };

        Self {
            owner: component.clone(),
            advance,
        }
    }
}

impl AdvancingComponent for AdvancingComponentHandle {
    fn advance_component(&mut self, elapsed_seconds: f32, flags: AdvanceFlags) -> bool {
        (self.advance)(&self.owner, elapsed_seconds, flags)
    }
}

fn advance_layout_participant(
    handle: &CoreHandle,
    elapsed_seconds: f32,
    flags: AdvanceFlags,
) -> bool {
    crate::mechanical_port::source::layout::layout_participant::LayoutParticipant::advance_component_occurrence(
        handle,
        elapsed_seconds,
        flags,
    )
}

fn advance_nested_artboard(handle: &CoreHandle, elapsed_seconds: f32, flags: AdvanceFlags) -> bool {
    crate::mechanical_port::source::nested_artboard::NestedArtboard::advance_component_occurrence(
        handle,
        elapsed_seconds,
        flags,
    )
}

fn advance_artboard_component_list(
    handle: &CoreHandle,
    elapsed_seconds: f32,
    flags: AdvanceFlags,
) -> bool {
    crate::mechanical_port::source::artboard_component_list::ArtboardComponentList::advance_component_occurrence(
        handle,
        elapsed_seconds,
        flags,
    )
}

fn advance_scripted_drawable(
    handle: &CoreHandle,
    elapsed_seconds: f32,
    flags: AdvanceFlags,
) -> bool {
    crate::mechanical_port::source::scripted::scripted_drawable::ScriptedDrawable::advance_occurrence(
        handle,
        elapsed_seconds,
        flags,
    )
}

fn advance_layout_component(
    handle: &CoreHandle,
    elapsed_seconds: f32,
    flags: AdvanceFlags,
) -> bool {
    crate::mechanical_port::source::layout_component::LayoutComponent::advance_component_occurrence(
        handle,
        elapsed_seconds,
        flags,
    )
}

fn advance_dynamic(handle: &CoreHandle, elapsed_seconds: f32, flags: AdvanceFlags) -> bool {
    handle
        .with_mut(|owner| owner.advancing_component_advance(elapsed_seconds, flags))
        .flatten()
        .unwrap_or(false)
}

impl dyn AdvancingComponent {
    pub fn from(component: &CoreHandle) -> Option<AdvancingComponentHandle> {
        use crate::mechanical_port::source::generated::{
            artboard_base::ArtboardBase, artboard_component_list_base::ArtboardComponentListBase,
            constraints::scrolling::scroll_constraint_base::ScrollConstraintBase,
            layout::layout_participant_base::LayoutParticipantBase,
            layout_component_base::LayoutComponentBase, nested_artboard_base::NestedArtboardBase,
            nested_artboard_layout_base::NestedArtboardLayoutBase,
            nested_artboard_leaf_base::NestedArtboardLeafBase,
            scripted::scripted_data_converter_base::ScriptedDataConverterBase,
            scripted::scripted_drawable_base::ScriptedDrawableBase,
            scripted::scripted_layout_base::ScriptedLayoutBase,
            scripted::scripted_path_effect_base::ScriptedPathEffectBase,
            text::text_input_base::TextInputBase,
        };
        match component.core_type()? {
            NestedArtboardLeafBase::TYPE_KEY
            | NestedArtboardLayoutBase::TYPE_KEY
            | NestedArtboardBase::TYPE_KEY
            | LayoutComponentBase::TYPE_KEY
            | LayoutParticipantBase::TYPE_KEY
            | ArtboardBase::TYPE_KEY
            | ArtboardComponentListBase::TYPE_KEY
            | ScrollConstraintBase::TYPE_KEY
            | TextInputBase::TYPE_KEY
            | ScriptedDataConverterBase::TYPE_KEY
            | ScriptedDrawableBase::TYPE_KEY
            | ScriptedLayoutBase::TYPE_KEY
            | ScriptedPathEffectBase::TYPE_KEY => {
                Some(AdvancingComponentHandle::classified(component))
            }
            _ => None,
        }
    }
}
