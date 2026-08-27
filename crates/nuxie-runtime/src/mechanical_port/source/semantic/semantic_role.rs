#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum SemanticRole {
    #[default]
    None = 0,
    Button = 1,
    Link = 2,
    Checkbox = 3,
    SwitchControl = 4,
    Slider = 5,
    TextField = 6,
    Text = 7,
    Image = 8,
    Group = 9,
    List = 10,
    ListItem = 11,
    Tab = 12,
    TabList = 13,
    Dialog = 14,
    AlertDialog = 15,
    RadioGroup = 16,
    RadioButton = 17,
}

pub fn is_interactive_role(role: SemanticRole) -> bool {
    matches!(
        role,
        SemanticRole::Button
            | SemanticRole::Link
            | SemanticRole::Checkbox
            | SemanticRole::SwitchControl
            | SemanticRole::Slider
            | SemanticRole::Tab
            | SemanticRole::ListItem
            | SemanticRole::RadioButton
    )
}

pub fn is_interactive_role_value(value: u32) -> bool {
    matches!(value, 1 | 2 | 3 | 4 | 5 | 12 | 11 | 17)
}
