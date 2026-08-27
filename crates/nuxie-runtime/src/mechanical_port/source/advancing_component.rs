use crate::mechanical_port::source::{advance_flags::AdvanceFlags, core::Core};

pub trait AdvancingComponent {
    fn advance_component(&mut self, elapsed_seconds: f32, flags: AdvanceFlags) -> bool;
}

impl dyn AdvancingComponent {
    pub fn from(component: &mut Core) -> Option<&mut dyn AdvancingComponent> {
        use crate::mechanical_port::source::generated::{
            artboard_base::ArtboardBase, artboard_component_list_base::ArtboardComponentListBase,
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
        use crate::mechanical_port::source::{
            constraints::scrolling::scroll_constraint::ScrollConstraint,
            generated::constraints::scrolling::scroll_constraint_base::ScrollConstraintBase,
        };

        match component.core_type() {
            NestedArtboardLeafBase::TYPE_KEY
            | NestedArtboardLayoutBase::TYPE_KEY
            | NestedArtboardBase::TYPE_KEY => component
                .as_nested_artboard_mut()
                .map(|value| value as &mut dyn AdvancingComponent),
            LayoutComponentBase::TYPE_KEY => component
                .as_layout_component_mut()
                .map(|value| value as &mut dyn AdvancingComponent),
            LayoutParticipantBase::TYPE_KEY => component
                .as_layout_participant_mut()
                .map(|value| value as &mut dyn AdvancingComponent),
            ArtboardBase::TYPE_KEY => component
                .as_artboard_mut()
                .map(|value| value as &mut dyn AdvancingComponent),
            ArtboardComponentListBase::TYPE_KEY => component
                .as_artboard_component_list_mut()
                .map(|value| value as &mut dyn AdvancingComponent),
            ScrollConstraintBase::TYPE_KEY => component
                .as_scroll_constraint_mut()
                .map(|value: &mut ScrollConstraint| value as &mut dyn AdvancingComponent),
            TextInputBase::TYPE_KEY => component
                .as_text_input_mut()
                .map(|value| value as &mut dyn AdvancingComponent),
            ScriptedDataConverterBase::TYPE_KEY => component
                .as_scripted_data_converter_mut()
                .map(|value| value as &mut dyn AdvancingComponent),
            ScriptedDrawableBase::TYPE_KEY => component
                .as_scripted_drawable_mut()
                .map(|value| value as &mut dyn AdvancingComponent),
            ScriptedLayoutBase::TYPE_KEY => component
                .as_scripted_layout_mut()
                .map(|value| value as &mut dyn AdvancingComponent),
            ScriptedPathEffectBase::TYPE_KEY => component
                .as_scripted_path_effect_mut()
                .map(|value| value as &mut dyn AdvancingComponent),
            _ => None,
        }
    }
}
