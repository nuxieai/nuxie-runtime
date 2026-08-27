//! Direct owner for pinned C++ `DataConverterNumberToList`.
//!
//! C++ retains its generated `ViewModelInstanceListItem`s and output
//! `DataValueList` on the converter. Rust retains the generated instances on
//! the occurrence-local list binding and carries the list value by value.

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum RuntimeDataConverterNumberToListInput {
    List { item_count: usize },
    Number(f32),
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum RuntimeDataConverterNumberToListOutput {
    /// Pinned C++ returns an input `DataValueList` unchanged.
    PassthroughList { item_count: usize },
    /// Pinned C++ returns its converter-owned output list for numeric input,
    /// including when the selected view model does not exist and that list is
    /// therefore empty.
    GeneratedList {
        item_count: usize,
        view_model_id: u64,
    },
}

/// Mechanical translation of `convert` for every input kind.
///
/// The C++ float-to-`int` cast is undefined for non-finite values and values
/// outside the signed-int range. `None` leaves precisely those inputs
/// undefined instead of choosing a Rust-only result. All C++-defined numbers
/// preserve `floor`, the negative clamp, and the absence of an item-count cap.
pub(crate) fn convert(
    input: RuntimeDataConverterNumberToListInput,
    view_model_id: u64,
    view_model_count: usize,
) -> Option<RuntimeDataConverterNumberToListOutput> {
    match input {
        RuntimeDataConverterNumberToListInput::List { item_count } => {
            Some(RuntimeDataConverterNumberToListOutput::PassthroughList { item_count })
        }
        RuntimeDataConverterNumberToListInput::Number(value) => {
            let count = cpp_defined_item_count(value)?;
            let item_count = if resolved_view_model_index(view_model_id, view_model_count).is_some()
            {
                count
            } else {
                0
            };
            Some(RuntimeDataConverterNumberToListOutput::GeneratedList {
                item_count,
                view_model_id,
            })
        }
        RuntimeDataConverterNumberToListInput::Other => None,
    }
}

/// Rust projection of C++ `File::viewModel(viewModelId())`.
pub(crate) fn resolved_view_model_index(
    view_model_id: u64,
    view_model_count: usize,
) -> Option<usize> {
    usize::try_from(view_model_id)
        .ok()
        .filter(|&index| index < view_model_count)
}

fn cpp_defined_item_count(value: f32) -> Option<usize> {
    let floored = value.floor();
    // `i32::MAX as f32` rounds up to 2^31, so use the exclusive upper
    // boundary explicitly rather than accepting that unrepresentable int.
    if !floored.is_finite() || floored < -2_147_483_648.0 || floored >= 2_147_483_648.0 {
        return None;
    }
    Some((floored as i32).max(0) as usize)
}

/// Mechanical translation of `clearItems`; dropping Rust handles performs the
/// C++ `unref` side effect.
pub(crate) fn clear_items<T>(items: &mut Vec<T>) -> bool {
    let changed = !items.is_empty();
    items.clear();
    changed
}

/// Mechanical translation of the generated `viewModelId` setter followed by
/// `viewModelIdChanged()`. The caller clears its occurrence-local item cache
/// and propagates the returned converter-dirty signal.
pub(crate) fn set_view_model_id(current: &mut u64, value: u64) -> bool {
    if *current == value {
        return false;
    }
    *current = value;
    true
}

/// Mechanical translation of the primary-header `outputType()` inline.
pub(crate) fn output_type() -> nuxie_binary::RuntimeDataType {
    nuxie_binary::RuntimeDataType::List
}
