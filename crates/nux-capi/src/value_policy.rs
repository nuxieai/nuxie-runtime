use crate::*;

/// One number, boolean, color or enum value and its boolean marker, named in
/// the file's catalog. Both properties belong to the named model.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default)]
pub struct NuxValueMarker {
    pub model: NuxStringView,
    pub value: NuxStringView,
    pub marker: NuxStringView,
}

pub(super) fn status(error: nuxie::RuntimeValuePolicyError) -> NuxStatus {
    match error {
        nuxie::RuntimeValuePolicyError::NotFound => NuxStatus::NotFound,
        nuxie::RuntimeValuePolicyError::InvalidArgument => NuxStatus::InvalidArgument,
        nuxie::RuntimeValuePolicyError::LimitExceeded => NuxStatus::LimitExceeded,
        nuxie::RuntimeValuePolicyError::BorrowConflict => NuxStatus::ReentrantCall,
    }
}

pub(super) fn retain_scope(
    roots: &[RuntimeOwnedViewModelHandle],
) -> Result<Vec<RuntimeOwnedViewModelHandle>, NuxStatus> {
    let mut owners = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    for root in roots {
        for owner in root
            .reachable_change_owner_snapshot()
            .ok_or(NuxStatus::ReentrantCall)?
        {
            if seen.insert(owner.instance_identity()) {
                if owners.len() >= MAX_PLAYER_STEP_STATE_CHANGES {
                    return Err(NuxStatus::LimitExceeded);
                }
                owners.push(owner);
            }
        }
    }
    Ok(owners)
}

/// Replace this file's whole marker table. NULL+0 removes all markers.
/// The next step or mutation applies the table to every matching instance,
/// including native nested/list instances. Value-first, marker-last writes
/// preserve explicit clears. Unchanged host/script writes count; unchanged
/// native listener/binding writes are not observable by the host journal.
/// NOT_FOUND names an absent model/property; INVALID_ARGUMENT rejects kinds,
/// duplicates and marker/value overlap. A refused install changes nothing.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_file_set_value_markers(
    file: *mut NuxFile,
    entries: *const NuxValueMarker,
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
        if count > 4_096 {
            return NuxStatus::LimitExceeded;
        }
        if entries.is_null() && count != 0 {
            return NuxStatus::NullArgument;
        }
        let entries = if count == 0 {
            &[]
        } else {
            unsafe { slice::from_raw_parts(entries, count) }
        };
        let mut owned = Vec::with_capacity(count);
        let mut bytes = 0usize;
        for entry in entries {
            for view in [entry.model, entry.value, entry.marker] {
                let Some(total) = bytes.checked_add(view.len) else {
                    return NuxStatus::LimitExceeded;
                };
                if total > MAX_PLAYER_STEP_RESULT_BYTES {
                    return NuxStatus::LimitExceeded;
                }
                bytes = total;
            }
            let read = |view| with_utf8_view(view, str::to_owned);
            let (model, value, marker) =
                match (read(entry.model), read(entry.value), read(entry.marker)) {
                    (Ok(model), Ok(value), Ok(marker)) => (model, value, marker),
                    (Err(status), _, _) | (_, Err(status), _) | (_, _, Err(status)) => {
                        return status;
                    }
                };
            owned.push(nuxie::RuntimeValueMarker {
                model,
                value,
                marker,
            });
        }
        let Ok(mut slot) = file.view_model_catalog.value_policy.try_borrow_mut() else {
            return NuxStatus::ReentrantCall;
        };
        if let Some(policy) = slot.as_mut() {
            return policy
                .set_markers(&owned)
                .map_or_else(status, |()| NuxStatus::Ok);
        }
        let mut next = nuxie::RuntimeValuePolicy::new(file.file.clone());
        match next.set_markers(&owned) {
            Ok(()) => {
                *slot = Some(next);
                NuxStatus::Ok
            }
            Err(error) => status(error),
        }
    })
}
