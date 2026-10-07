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
    number_bound: Option<f32>,
}

impl RuntimeCompiledValueRule {
    /// Compile once at installation. Invalid patterns reject the rule.
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
            Pattern(source) => Some(
                regress::Regex::with_flags(&format!("^(?:{source})$"), "v")
                    .map_err(|_| RuntimeValuePolicyError::InvalidArgument)?,
            ),
            _ => None,
        };
        let number_bound = match &kind {
            NumberMinimum(bound) | NumberMaximum(bound) => Some(*bound as f32),
            _ => None,
        };
        Ok(Self {
            kind,
            pattern,
            number_bound,
        })
    }

    pub fn kind(&self) -> &RuntimeValueRuleKind {
        &self.kind
    }

    /// `marker` is the installed pair's effective is-set value, when present.
    /// Empty text and unanswered lists bypass every check except required. Type mismatches fail;
    /// the file table additionally rejects them before installing a rule.
    pub fn holds(&self, value: RuntimeRuleValue<'_>, marker: Option<bool>) -> bool {
        use RuntimeValueRuleKind::*;
        let empty = match value {
            RuntimeRuleValue::Text("") => true,
            RuntimeRuleValue::List { items, picked } => {
                picked == Some(0) || (matches!(self.kind, ItemCount { .. }) && items == 0)
            }
            _ => false,
        };
        if empty && !matches!(self.kind, Required) {
            return true;
        }
        match (&self.kind, value) {
            (Required, _) if marker.is_some() => marker == Some(true),
            (Required, RuntimeRuleValue::Text(text)) => !text.is_empty(),
            (Required, RuntimeRuleValue::List { items, picked }) => picked.unwrap_or(items) > 0,
            (Required, _) => true,
            (NumberMinimum(_), RuntimeRuleValue::Number(number)) => {
                number.is_finite() && self.number_bound.is_some_and(|bound| number >= bound)
            }
            (NumberMaximum(_), RuntimeRuleValue::Number(number)) => {
                number.is_finite() && self.number_bound.is_some_and(|bound| number <= bound)
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
                .is_some_and(|pattern| pattern.find(text).is_some()),
            (Url, RuntimeRuleValue::Text(text)) => url::Url::parse(text).is_ok(),
            (Date, RuntimeRuleValue::Text(text)) => calendar_day(text),
            _ => false,
        }
    }
}

fn calendar_day(text: &str) -> bool {
    let bytes = text.as_bytes();
    if bytes.len() != 10 || bytes.get(4) != Some(&b'-') || bytes.get(7) != Some(&b'-') {
        return false;
    }
    let number = |range: std::ops::Range<usize>| {
        bytes.get(range)?.iter().try_fold(0u32, |value, digit| {
            let digit = digit.checked_sub(b'0').filter(|digit| *digit <= 9)?;
            value.checked_mul(10)?.checked_add(u32::from(digit))
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
