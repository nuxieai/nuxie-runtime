use crate::*;

pub const NUX_VALUE_RULE_NUMBER_MINIMUM: u32 = 1;
pub const NUX_VALUE_RULE_NUMBER_MAXIMUM: u32 = 2;
pub const NUX_VALUE_RULE_TEXT_MINIMUM: u32 = 3;
pub const NUX_VALUE_RULE_TEXT_MAXIMUM: u32 = 4;
pub const NUX_VALUE_RULE_ALLOWED_VALUES: u32 = 5;
pub const NUX_VALUE_RULE_ITEM_COUNT: u32 = 6;
pub const NUX_VALUE_RULE_PICKED_COUNT: u32 = 7;
pub const NUX_VALUE_RULE_LENGTH: u32 = 8;
pub const NUX_VALUE_RULE_PATTERN: u32 = 9;
pub const NUX_VALUE_RULE_REQUIRED: u32 = 10;
pub const NUX_VALUE_RULE_URL: u32 = 11;
pub const NUX_VALUE_RULE_DATE: u32 = 12;
pub const NUX_VALUE_RULE_MARK: u32 = 0;
pub const NUX_VALUE_RULE_REFUSE: u32 = 1;
pub const NUX_VALUE_RULE_HAS_MINIMUM: u32 = 1;
pub const NUX_VALUE_RULE_HAS_MAXIMUM: u32 = 2;

/// One ordered rule on a named model property. All strings are copied UTF-8.
/// number_bound is used by numeric minimum/maximum; text by text bounds and
/// pattern; values by allowed-values; picked_property by picked-count.
/// Count and length bounds use bound_flags to select minimum/maximum.
/// Unused operands must be zero or empty. code and message are caller-authored.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct NuxValueRule {
    pub model: NuxStringView,
    pub property: NuxStringView,
    pub kind: u32,
    pub mode: u32,
    pub number_bound: f64,
    pub text: NuxStringView,
    pub values: *const NuxStringView,
    pub value_count: usize,
    pub picked_property: NuxStringView,
    pub bound_flags: u32,
    pub minimum: usize,
    pub maximum: usize,
    pub code: NuxStringView,
    pub message: NuxStringView,
}

/// One rule failure in write order, then installer order. attempted names the
/// written property and carries its attempted value. Refused writes never land
/// in the operation's change rows. Bytes remain borrowed until result free.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct NuxValueRuleReportView {
    pub struct_size: u32,
    pub rule_index: usize,
    pub refused: u32,
    pub attempted: NuxViewModelChangeView,
}
impl Default for NuxValueRuleReportView {
    fn default() -> Self {
        Self {
            struct_size: std::mem::size_of::<Self>() as u32,
            rule_index: 0,
            refused: 0,
            attempted: NuxViewModelChangeView::default(),
        }
    }
}

fn copied(view: NuxStringView, bytes: &mut usize) -> Result<String, NuxStatus> {
    *bytes = bytes
        .checked_add(view.len)
        .ok_or(NuxStatus::LimitExceeded)?;
    if *bytes > MAX_PLAYER_STEP_RESULT_BYTES {
        return Err(NuxStatus::LimitExceeded);
    }
    with_utf8_view(view, str::to_owned)
}

unsafe fn rule(
    entry: &NuxValueRule,
    bytes: &mut usize,
) -> Result<nuxie::RuntimeValueRule, NuxStatus> {
    use nuxie::RuntimeValueRuleKind as Kind;
    let mode = match entry.mode {
        NUX_VALUE_RULE_MARK => nuxie::RuntimeValueRuleMode::Mark,
        NUX_VALUE_RULE_REFUSE => nuxie::RuntimeValueRuleMode::Refuse,
        _ => return Err(NuxStatus::InvalidArgument),
    };
    if entry.bound_flags & !(NUX_VALUE_RULE_HAS_MINIMUM | NUX_VALUE_RULE_HAS_MAXIMUM) != 0 {
        return Err(NuxStatus::InvalidArgument);
    }
    let counts = matches!(
        entry.kind,
        NUX_VALUE_RULE_ITEM_COUNT | NUX_VALUE_RULE_PICKED_COUNT | NUX_VALUE_RULE_LENGTH
    );
    let numeric = matches!(
        entry.kind,
        NUX_VALUE_RULE_NUMBER_MINIMUM | NUX_VALUE_RULE_NUMBER_MAXIMUM
    );
    let text_operand = matches!(
        entry.kind,
        NUX_VALUE_RULE_TEXT_MINIMUM | NUX_VALUE_RULE_TEXT_MAXIMUM | NUX_VALUE_RULE_PATTERN
    );
    if (!counts && (entry.bound_flags != 0 || entry.minimum != 0 || entry.maximum != 0))
        || (!numeric && entry.number_bound != 0.0)
        || (!text_operand && entry.text.len != 0)
        || (entry.kind != NUX_VALUE_RULE_ALLOWED_VALUES && entry.value_count != 0)
        || (entry.kind != NUX_VALUE_RULE_PICKED_COUNT && entry.picked_property.len != 0)
        || (entry.bound_flags & NUX_VALUE_RULE_HAS_MINIMUM == 0 && entry.minimum != 0)
        || (entry.bound_flags & NUX_VALUE_RULE_HAS_MAXIMUM == 0 && entry.maximum != 0)
    {
        return Err(NuxStatus::InvalidArgument);
    }
    let minimum = if entry.bound_flags & NUX_VALUE_RULE_HAS_MINIMUM != 0 {
        entry.minimum
    } else {
        0
    };
    let maximum = if entry.bound_flags & NUX_VALUE_RULE_HAS_MAXIMUM != 0 {
        entry.maximum
    } else {
        usize::MAX
    };
    let kind = match entry.kind {
        NUX_VALUE_RULE_NUMBER_MINIMUM => Kind::NumberMinimum(entry.number_bound),
        NUX_VALUE_RULE_NUMBER_MAXIMUM => Kind::NumberMaximum(entry.number_bound),
        NUX_VALUE_RULE_TEXT_MINIMUM => Kind::TextMinimum(copied(entry.text, bytes)?),
        NUX_VALUE_RULE_TEXT_MAXIMUM => Kind::TextMaximum(copied(entry.text, bytes)?),
        NUX_VALUE_RULE_PATTERN => Kind::Pattern(copied(entry.text, bytes)?),
        NUX_VALUE_RULE_ALLOWED_VALUES => {
            if entry.value_count > 4096 {
                return Err(NuxStatus::LimitExceeded);
            }
            if entry.value_count != 0 && entry.values.is_null() {
                return Err(NuxStatus::NullArgument);
            }
            *bytes = bytes
                .checked_add(
                    entry
                        .value_count
                        .saturating_mul(std::mem::size_of::<NuxStringView>()),
                )
                .ok_or(NuxStatus::LimitExceeded)?;
            if *bytes > MAX_PLAYER_STEP_RESULT_BYTES {
                return Err(NuxStatus::LimitExceeded);
            }
            let values = if entry.value_count == 0 {
                &[]
            } else {
                unsafe { slice::from_raw_parts(entry.values, entry.value_count) }
            };
            Kind::AllowedValues(
                values
                    .iter()
                    .map(|value| copied(*value, bytes))
                    .collect::<Result<_, _>>()?,
            )
        }
        NUX_VALUE_RULE_ITEM_COUNT => Kind::ItemCount { minimum, maximum },
        NUX_VALUE_RULE_PICKED_COUNT => Kind::PickedCount {
            property: copied(entry.picked_property, bytes)?,
            minimum,
            maximum,
        },
        NUX_VALUE_RULE_LENGTH => Kind::Length { minimum, maximum },
        NUX_VALUE_RULE_REQUIRED => Kind::Required,
        NUX_VALUE_RULE_URL => Kind::Url,
        NUX_VALUE_RULE_DATE => Kind::Date,
        _ => return Err(NuxStatus::InvalidArgument),
    };
    Ok(nuxie::RuntimeValueRule {
        model: copied(entry.model, bytes)?,
        property: copied(entry.property, bytes)?,
        kind,
        mode,
        code: copied(entry.code, bytes)?,
        message: copied(entry.message, bytes)?,
    })
}

/// Atomically replace this file's ordered rule table; NULL+0 removes it.
/// Bad names/kinds/bounds refuse the whole table. Invalid patterns always hold.
/// Existing values never cause install failure or emit operation reports.
/// Marker tables remain installed. Only explicitly named properties have rules.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_file_set_value_rules(
    file: *mut NuxFile,
    entries: *const NuxValueRule,
    count: usize,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        let _call = match enter_handle(file, HandleKind::File) {
            Ok(guard) => guard,
            Err(status) => return status,
        };
        let Some(file) = (unsafe { file.as_ref() }) else {
            return NuxStatus::NullArgument;
        };
        if count > 4096 {
            return NuxStatus::LimitExceeded;
        }
        if count != 0 && entries.is_null() {
            return NuxStatus::NullArgument;
        }
        let entries = if count == 0 {
            &[]
        } else {
            unsafe { slice::from_raw_parts(entries, count) }
        };
        let mut bytes = count.saturating_mul(std::mem::size_of::<NuxValueRule>());
        let owned = match entries
            .iter()
            .map(|entry| unsafe { rule(entry, &mut bytes) })
            .collect::<Result<Vec<_>, _>>()
        {
            Ok(owned) => owned,
            Err(status) => return status,
        };
        let Ok(mut slot) = file.view_model_catalog.value_policy.try_borrow_mut() else {
            return NuxStatus::ReentrantCall;
        };
        if let Some(policy) = slot.as_mut() {
            return policy
                .set_rules(&owned)
                .map_or_else(value_policy::status, |()| NuxStatus::Ok);
        }
        let mut policy = nuxie::RuntimeValuePolicy::new(file.file.clone());
        match policy.set_rules(&owned) {
            Ok(()) => {
                *slot = Some(policy);
                NuxStatus::Ok
            }
            Err(error) => value_policy::status(error),
        }
    })
}

pub(super) unsafe fn report_view(
    report: &nuxie::RuntimeValueRuleReport,
    origin: u32,
    correlation_id: u64,
    out: *mut NuxValueRuleReportView,
) -> NuxStatus {
    let change = data_binding::OwnedViewModelChange {
        origin,
        correlation_id,
        change: nuxie::RuntimeViewModelChange {
            owner_instance_identity: report.owner_instance_identity,
            property_index: report.property_index,
            value: report.attempted.clone(),
        },
    };
    let value = NuxValueRuleReportView {
        rule_index: report.rule_index,
        refused: u32::from(report.refused),
        attempted: data_binding::view_model_change_view(&change),
        ..Default::default()
    };
    unsafe { write_caller_struct(out, &value, std::mem::size_of::<NuxValueRuleReportView>()) }
        .map_or_else(|status| status, |()| NuxStatus::Ok)
}

/// Read a rule report from a successful step, in attempted-write order.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_player_step_result_rule_report(
    result: *const NuxPlayerStepResult,
    index: usize,
    out: *mut NuxValueRuleReportView,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        let _call = match enter_handle(result, HandleKind::PlayerStepResult) {
            Ok(guard) => guard,
            Err(status) => return status,
        };
        let Some(result) = (unsafe { result.as_ref() }) else {
            return NuxStatus::NullArgument;
        };
        if result.status != NuxStatus::Ok {
            return result.status;
        }
        let Some(report) = result.rule_reports.get(index) else {
            return NuxStatus::NotFound;
        };
        unsafe { report_view(report, NUX_VIEW_MODEL_CHANGE_ORIGIN_RUNTIME, 0, out) }
    })
}

/// Read one attempted list member's identity from a rule report.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_player_step_result_rule_report_list_item(
    result: *const NuxPlayerStepResult,
    report_index: usize,
    item_index: usize,
    out_identity: *mut u64,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if out_identity.is_null() {
            return NuxStatus::NullArgument;
        }
        unsafe { *out_identity = 0 };
        let _call = match enter_handle(result, HandleKind::PlayerStepResult) {
            Ok(guard) => guard,
            Err(status) => return status,
        };
        let Some(result) = (unsafe { result.as_ref() }) else {
            return NuxStatus::NullArgument;
        };
        if result.status != NuxStatus::Ok {
            return result.status;
        }
        let Some(report) = result.rule_reports.get(report_index) else {
            return NuxStatus::NotFound;
        };
        let RuntimeViewModelChangeValue::List(items) = &report.attempted else {
            return NuxStatus::InvalidArgument;
        };
        let Some(identity) = items.get(item_index) else {
            return NuxStatus::NotFound;
        };
        unsafe { *out_identity = *identity };
        NuxStatus::Ok
    })
}
