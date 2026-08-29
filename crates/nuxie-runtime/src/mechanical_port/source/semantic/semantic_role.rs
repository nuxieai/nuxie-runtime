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

impl SemanticRole {
    pub const fn from_raw(value: u32) -> Option<Self> {
        Some(match value {
            0 => Self::None,
            1 => Self::Button,
            2 => Self::Link,
            3 => Self::Checkbox,
            4 => Self::SwitchControl,
            5 => Self::Slider,
            6 => Self::TextField,
            7 => Self::Text,
            8 => Self::Image,
            9 => Self::Group,
            10 => Self::List,
            11 => Self::ListItem,
            12 => Self::Tab,
            13 => Self::TabList,
            14 => Self::Dialog,
            15 => Self::AlertDialog,
            16 => Self::RadioGroup,
            17 => Self::RadioButton,
            _ => return None,
        })
    }

    pub const fn is_interactive(self) -> bool {
        matches!(
            self,
            Self::Button
                | Self::Link
                | Self::Checkbox
                | Self::SwitchControl
                | Self::Slider
                | Self::ListItem
                | Self::Tab
                | Self::RadioButton
        )
    }
}

pub fn is_interactive_role(role: SemanticRole) -> bool {
    role.is_interactive()
}

pub fn is_interactive_role_value(value: u32) -> bool {
    SemanticRole::from_raw(value).is_some_and(SemanticRole::is_interactive)
}
