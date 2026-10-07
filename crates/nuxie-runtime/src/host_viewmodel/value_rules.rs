//! Installer-selected checks, independent of writer and display policy.
use super::RuntimeValuePolicyError;

#[derive(Clone, Debug)]
pub enum RuntimeValueRuleKind {
    NumberMinimum(f64),
    NumberMaximum(f64),
    TextMinimum(String),
    TextMaximum(String),
    AllowedValues(Vec<String>),
    ItemCount {
        minimum: usize,
        maximum: usize,
    },
    PickedCount {
        property: String,
        minimum: usize,
        maximum: usize,
    },
    Length {
        minimum: usize,
        maximum: usize,
    },
    Pattern(String),
    Required,
    Url,
    Date,
}

/// A typed observation. Enum labels come from the file's authored enum, never
/// from indexing the installed allowed subset. `None` is an invalid index.
#[derive(Clone, Copy, Debug)]
pub enum RuntimeRuleValue<'a> {
    Number(f32),
    Text(&'a str),
    Enum(Option<&'a str>),
    List { items: usize, picked: Option<usize> },
    Scalar,
}

#[derive(Clone, Debug)]
pub struct RuntimeCompiledValueRule {
    kind: RuntimeValueRuleKind,
    pattern: Option<regress::Regex>,
}

impl RuntimeCompiledValueRule {
    /// Compile once at installation. An invalid pattern is deliberately inert.
    pub fn compile(kind: RuntimeValueRuleKind) -> Result<Self, RuntimeValuePolicyError> {
        use RuntimeValueRuleKind::*;
        match &kind {
            NumberMinimum(bound) | NumberMaximum(bound) if !bound.is_finite() => {
                return Err(RuntimeValuePolicyError::InvalidArgument);
            }
            ItemCount { minimum, maximum }
            | PickedCount {
                minimum, maximum, ..
            }
            | Length { minimum, maximum }
                if minimum > maximum =>
            {
                return Err(RuntimeValuePolicyError::InvalidArgument);
            }
            PickedCount { property, .. } if property.is_empty() => {
                return Err(RuntimeValuePolicyError::InvalidArgument);
            }
            _ => {}
        }
        let pattern = match &kind {
            Pattern(source) => regress::Regex::with_flags(&format!("^(?:{source})$"), "v").ok(),
            _ => None,
        };
        Ok(Self { kind, pattern })
    }

    pub fn kind(&self) -> &RuntimeValueRuleKind {
        &self.kind
    }

    /// `marker` is the installed pair's effective is-set value, when present.
    /// Empty text bypasses every check except required. Type mismatches fail;
    /// the file table additionally rejects them before installing a rule.
    pub fn holds(&self, value: RuntimeRuleValue<'_>, marker: Option<bool>) -> bool {
        use RuntimeValueRuleKind::*;
        if matches!(value, RuntimeRuleValue::Text("")) && !matches!(self.kind, Required) {
            return true;
        }
        match (&self.kind, value) {
            (Required, _) if marker.is_some() => marker == Some(true),
            (Required, RuntimeRuleValue::Text(text)) => !text.is_empty(),
            (Required, RuntimeRuleValue::List { items, picked }) => picked.unwrap_or(items) > 0,
            (Required, _) => true,
            (NumberMinimum(bound), RuntimeRuleValue::Number(number)) => {
                number.is_finite() && f64::from(number) >= *bound
            }
            (NumberMaximum(bound), RuntimeRuleValue::Number(number)) => {
                number.is_finite() && f64::from(number) <= *bound
            }
            (TextMinimum(bound), RuntimeRuleValue::Text(text)) => {
                text.encode_utf16().cmp(bound.encode_utf16()).is_ge()
            }
            (TextMaximum(bound), RuntimeRuleValue::Text(text)) => {
                text.encode_utf16().cmp(bound.encode_utf16()).is_le()
            }
            (AllowedValues(allowed), RuntimeRuleValue::Text(text)) => {
                allowed.iter().any(|item| item == text)
            }
            (AllowedValues(allowed), RuntimeRuleValue::Enum(Some(label))) => {
                allowed.iter().any(|item| item == label)
            }
            (ItemCount { minimum, maximum }, RuntimeRuleValue::List { items, .. }) => {
                (*minimum..=*maximum).contains(&items)
            }
            (
                PickedCount {
                    minimum, maximum, ..
                },
                RuntimeRuleValue::List {
                    picked: Some(count),
                    ..
                },
            ) => (*minimum..=*maximum).contains(&count),
            (Length { minimum, maximum }, RuntimeRuleValue::Text(text)) => {
                (*minimum..=*maximum).contains(&text.encode_utf16().count())
            }
            (Pattern(_), RuntimeRuleValue::Text(text)) => self
                .pattern
                .as_ref()
                .is_none_or(|pattern| pattern.find(text).is_some()),
            (Url, RuntimeRuleValue::Text(text)) => url::Url::parse(text).is_ok(),
            (Date, RuntimeRuleValue::Text(text)) => calendar_day(text),
            _ => false,
        }
    }
}

fn calendar_day(text: &str) -> bool {
    let bytes = text.as_bytes();
    if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-' {
        return false;
    }
    let number = |range: std::ops::Range<usize>| {
        bytes[range].iter().try_fold(0u32, |value, digit| {
            digit
                .is_ascii_digit()
                .then(|| value * 10 + u32::from(*digit - b'0'))
        })
    };
    let (Some(year), Some(month), Some(day)) = (number(0..4), number(5..7), number(8..10)) else {
        return false;
    };
    let maximum = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) => 29,
        2 => 28,
        _ => return false,
    };
    year > 0 && (1..=maximum).contains(&day)
}
