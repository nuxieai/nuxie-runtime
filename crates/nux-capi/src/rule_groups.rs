use crate::*;

/// A checked member and its computed list. errors_path is slash-separated,
/// relative to the group's model. Each list item has the named string fields.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct NuxRuleGroupMember {
    pub property: NuxStringView,
    pub errors_path: NuxStringView,
    pub item_model: NuxStringView,
    pub code_property: NuxStringView,
    pub message_property: NuxStringView,
}

/// A model's computed boolean and ordered error lists. All strings are copied.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct NuxRuleGroup {
    pub model: NuxStringView,
    pub valid: NuxStringView,
    pub members: *const NuxRuleGroupMember,
    pub member_count: usize,
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

/// Atomically replace computed groups; NULL+0 removes them. Install rules and
/// markers first, before the first step or mutation. Invalid names, kinds,
/// duplicate outputs or output/input conflicts reject the entire table.
/// At most 4096 groups plus members and 8 MiB of copied input are accepted.
/// Each list puts the latest refusal's rules first, then kept-value failures,
/// with installed order within each partition and no duplicates. An accepted
/// write clears that refusal. The computed boolean depends only on kept values.
/// Writes to outputs are corrected in the same operation; quiet outputs emit
/// no rows. Replacing groups or rules clears retained refusal history.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_file_set_rule_groups(
    file: *mut NuxFile,
    entries: *const NuxRuleGroup,
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
        let mut total = count;
        let mut bytes = count.saturating_mul(std::mem::size_of::<NuxRuleGroup>());
        let owned = (|| -> Result<Vec<nuxie::RuntimeRuleGroup>, NuxStatus> {
            let mut owned = Vec::with_capacity(count);
            for entry in entries {
                total = total
                    .checked_add(entry.member_count)
                    .ok_or(NuxStatus::LimitExceeded)?;
                if total > 4096 {
                    return Err(NuxStatus::LimitExceeded);
                }
                if entry.member_count != 0 && entry.members.is_null() {
                    return Err(NuxStatus::NullArgument);
                }
                bytes = bytes.saturating_add(
                    entry
                        .member_count
                        .saturating_mul(std::mem::size_of::<NuxRuleGroupMember>()),
                );
                let members = if entry.member_count == 0 {
                    &[]
                } else {
                    unsafe { slice::from_raw_parts(entry.members, entry.member_count) }
                };
                let members = members
                    .iter()
                    .map(|member| {
                        Ok(nuxie::RuntimeRuleGroupMember {
                            property: copied(member.property, &mut bytes)?,
                            errors_path: copied(member.errors_path, &mut bytes)?,
                            item_model: copied(member.item_model, &mut bytes)?,
                            code_property: copied(member.code_property, &mut bytes)?,
                            message_property: copied(member.message_property, &mut bytes)?,
                        })
                    })
                    .collect::<Result<Vec<_>, NuxStatus>>()?;
                owned.push(nuxie::RuntimeRuleGroup {
                    model: copied(entry.model, &mut bytes)?,
                    valid: copied(entry.valid, &mut bytes)?,
                    members,
                });
            }
            Ok(owned)
        })();
        let owned = match owned {
            Ok(owned) => owned,
            Err(status) => return status,
        };
        let Ok(mut slot) = file.view_model_catalog.value_policy.try_borrow_mut() else {
            return NuxStatus::ReentrantCall;
        };
        if let Some(policy) = slot.as_mut() {
            return policy
                .set_groups(&owned)
                .map_or_else(value_policy::status, |()| NuxStatus::Ok);
        }
        let mut policy = nuxie::RuntimeValuePolicy::new(file.file.clone());
        match policy.set_groups(&owned) {
            Ok(()) => {
                *slot = Some(policy);
                NuxStatus::Ok
            }
            Err(error) => value_policy::status(error),
        }
    })
}
