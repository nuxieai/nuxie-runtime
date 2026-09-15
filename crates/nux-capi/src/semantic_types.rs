//! Stable portable semantic codes. Unknown roles are treated as NONE by hosts;
//! unknown state/trait bits are ignored. Authored actions remain the authority
//! for interaction, regardless of role.

#[cfg(test)]
use nuxie::runtime::semantic::{
    semantic_role::SemanticRole, semantic_state::SemanticState, semantic_trait::SemanticTrait,
};

pub const NUX_SEMANTIC_ROLE_NONE: u32 = 0;
pub const NUX_SEMANTIC_ROLE_BUTTON: u32 = 1;
pub const NUX_SEMANTIC_ROLE_LINK: u32 = 2;
pub const NUX_SEMANTIC_ROLE_CHECKBOX: u32 = 3;
pub const NUX_SEMANTIC_ROLE_SWITCH_CONTROL: u32 = 4;
pub const NUX_SEMANTIC_ROLE_SLIDER: u32 = 5;
pub const NUX_SEMANTIC_ROLE_TEXT_FIELD: u32 = 6;
pub const NUX_SEMANTIC_ROLE_TEXT: u32 = 7;
pub const NUX_SEMANTIC_ROLE_IMAGE: u32 = 8;
pub const NUX_SEMANTIC_ROLE_GROUP: u32 = 9;
pub const NUX_SEMANTIC_ROLE_LIST: u32 = 10;
pub const NUX_SEMANTIC_ROLE_LIST_ITEM: u32 = 11;
pub const NUX_SEMANTIC_ROLE_TAB: u32 = 12;
pub const NUX_SEMANTIC_ROLE_TAB_LIST: u32 = 13;
pub const NUX_SEMANTIC_ROLE_DIALOG: u32 = 14;
pub const NUX_SEMANTIC_ROLE_ALERT_DIALOG: u32 = 15;
pub const NUX_SEMANTIC_ROLE_RADIO_GROUP: u32 = 16;
pub const NUX_SEMANTIC_ROLE_RADIO_BUTTON: u32 = 17;
pub const NUX_SEMANTIC_STATE_NONE: u32 = 0;
pub const NUX_SEMANTIC_STATE_EXPANDED: u32 = 1 << 0;
pub const NUX_SEMANTIC_STATE_SELECTED: u32 = 1 << 1;
pub const NUX_SEMANTIC_STATE_CHECKED: u32 = 1 << 2;
pub const NUX_SEMANTIC_STATE_MIXED: u32 = 1 << 3;
pub const NUX_SEMANTIC_STATE_TOGGLED: u32 = 1 << 4;
pub const NUX_SEMANTIC_STATE_REQUIRED: u32 = 1 << 5;
pub const NUX_SEMANTIC_STATE_DISABLED: u32 = 1 << 6;
pub const NUX_SEMANTIC_STATE_FOCUSED: u32 = 1 << 7;
pub const NUX_SEMANTIC_STATE_HIDDEN: u32 = 1 << 8;
pub const NUX_SEMANTIC_STATE_LIVE_REGION: u32 = 1 << 9;
pub const NUX_SEMANTIC_STATE_READ_ONLY: u32 = 1 << 10;
pub const NUX_SEMANTIC_STATE_MODAL: u32 = 1 << 11;
pub const NUX_SEMANTIC_STATE_OBSCURED: u32 = 1 << 12;
pub const NUX_SEMANTIC_STATE_MULTILINE: u32 = 1 << 13;
pub const NUX_SEMANTIC_TRAIT_NONE: u32 = 0;
pub const NUX_SEMANTIC_TRAIT_EXPANDABLE: u32 = 1 << 0;
pub const NUX_SEMANTIC_TRAIT_SELECTABLE: u32 = 1 << 1;
pub const NUX_SEMANTIC_TRAIT_CHECKABLE: u32 = 1 << 2;
pub const NUX_SEMANTIC_TRAIT_TOGGLEABLE: u32 = 1 << 3;
pub const NUX_SEMANTIC_TRAIT_REQUIRABLE: u32 = 1 << 4;
pub const NUX_SEMANTIC_TRAIT_ENABLABLE: u32 = 1 << 5;
pub const NUX_SEMANTIC_TRAIT_FOCUSABLE: u32 = 1 << 6;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn public_codes_match_runtime_semantics() {
        assert_eq!(NUX_SEMANTIC_ROLE_NONE, SemanticRole::None as u32);
        assert_eq!(NUX_SEMANTIC_ROLE_BUTTON, SemanticRole::Button as u32);
        assert_eq!(NUX_SEMANTIC_ROLE_LINK, SemanticRole::Link as u32);
        assert_eq!(NUX_SEMANTIC_ROLE_CHECKBOX, SemanticRole::Checkbox as u32);
        assert_eq!(
            NUX_SEMANTIC_ROLE_SWITCH_CONTROL,
            SemanticRole::SwitchControl as u32
        );
        assert_eq!(NUX_SEMANTIC_ROLE_SLIDER, SemanticRole::Slider as u32);
        assert_eq!(NUX_SEMANTIC_ROLE_TEXT_FIELD, SemanticRole::TextField as u32);
        assert_eq!(NUX_SEMANTIC_ROLE_TEXT, SemanticRole::Text as u32);
        assert_eq!(NUX_SEMANTIC_ROLE_IMAGE, SemanticRole::Image as u32);
        assert_eq!(NUX_SEMANTIC_ROLE_GROUP, SemanticRole::Group as u32);
        assert_eq!(NUX_SEMANTIC_ROLE_LIST, SemanticRole::List as u32);
        assert_eq!(NUX_SEMANTIC_ROLE_LIST_ITEM, SemanticRole::ListItem as u32);
        assert_eq!(NUX_SEMANTIC_ROLE_TAB, SemanticRole::Tab as u32);
        assert_eq!(NUX_SEMANTIC_ROLE_TAB_LIST, SemanticRole::TabList as u32);
        assert_eq!(NUX_SEMANTIC_ROLE_DIALOG, SemanticRole::Dialog as u32);
        assert_eq!(
            NUX_SEMANTIC_ROLE_ALERT_DIALOG,
            SemanticRole::AlertDialog as u32
        );
        assert_eq!(
            NUX_SEMANTIC_ROLE_RADIO_GROUP,
            SemanticRole::RadioGroup as u32
        );
        assert_eq!(
            NUX_SEMANTIC_ROLE_RADIO_BUTTON,
            SemanticRole::RadioButton as u32
        );
        assert_eq!(NUX_SEMANTIC_STATE_NONE, SemanticState::NONE.0);
        assert_eq!(NUX_SEMANTIC_STATE_EXPANDED, SemanticState::EXPANDED.0);
        assert_eq!(NUX_SEMANTIC_STATE_SELECTED, SemanticState::SELECTED.0);
        assert_eq!(NUX_SEMANTIC_STATE_CHECKED, SemanticState::CHECKED.0);
        assert_eq!(NUX_SEMANTIC_STATE_MIXED, SemanticState::MIXED.0);
        assert_eq!(NUX_SEMANTIC_STATE_TOGGLED, SemanticState::TOGGLED.0);
        assert_eq!(NUX_SEMANTIC_STATE_REQUIRED, SemanticState::REQUIRED.0);
        assert_eq!(NUX_SEMANTIC_STATE_DISABLED, SemanticState::DISABLED.0);
        assert_eq!(NUX_SEMANTIC_STATE_FOCUSED, SemanticState::FOCUSED.0);
        assert_eq!(NUX_SEMANTIC_STATE_HIDDEN, SemanticState::HIDDEN.0);
        assert_eq!(NUX_SEMANTIC_STATE_LIVE_REGION, SemanticState::LIVE_REGION.0);
        assert_eq!(NUX_SEMANTIC_STATE_READ_ONLY, SemanticState::READ_ONLY.0);
        assert_eq!(NUX_SEMANTIC_STATE_MODAL, SemanticState::MODAL.0);
        assert_eq!(NUX_SEMANTIC_STATE_OBSCURED, SemanticState::OBSCURED.0);
        assert_eq!(NUX_SEMANTIC_STATE_MULTILINE, SemanticState::MULTILINE.0);
        assert_eq!(NUX_SEMANTIC_TRAIT_NONE, SemanticTrait::NONE.0);
        assert_eq!(NUX_SEMANTIC_TRAIT_EXPANDABLE, SemanticTrait::EXPANDABLE.0);
        assert_eq!(NUX_SEMANTIC_TRAIT_SELECTABLE, SemanticTrait::SELECTABLE.0);
        assert_eq!(NUX_SEMANTIC_TRAIT_CHECKABLE, SemanticTrait::CHECKABLE.0);
        assert_eq!(NUX_SEMANTIC_TRAIT_TOGGLEABLE, SemanticTrait::TOGGLEABLE.0);
        assert_eq!(NUX_SEMANTIC_TRAIT_REQUIRABLE, SemanticTrait::REQUIRABLE.0);
        assert_eq!(NUX_SEMANTIC_TRAIT_ENABLABLE, SemanticTrait::ENABLABLE.0);
        assert_eq!(NUX_SEMANTIC_TRAIT_FOCUSABLE, SemanticTrait::FOCUSABLE.0);
    }
}
