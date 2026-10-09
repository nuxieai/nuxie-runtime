//! Script functions shared by hosts that offer checked batch writes and list
//! reads to Luau. A batch is a list of addressed scalar writes; a list read
//! returns one property from every item. Hosts keep their own step context
//! (roots, policy and rule operation) and call these with it, so every host
//! runs the same checks and raises the same texts.
use nuxie_runtime::{
    RUNTIME_CHECKED_VALUE_BATCH_MAX_ENTRIES, RuntimeCheckedValueBatchEntry,
    RuntimeCheckedValueInput, RuntimeOwnedViewModelHandle, RuntimeValuePolicy,
    RuntimeValuePolicyError, RuntimeValuePolicyOperation, RuntimeViewModelChangeValue,
    runtime_checked_value_write_batch,
};
use nuxie_scripting::{Lua, Value};
use std::cell::RefCell;
use std::collections::BTreeMap;

/// The most writes one script batch carries: the runtime batch's own bound,
/// so a list that reads here is never refused for its length by the batch.
const MAX_SCRIPT_BATCH_WRITES: usize = RUNTIME_CHECKED_VALUE_BATCH_MAX_ENTRIES;

const KEYS: &str = "checked writes must have the keys 1..n";
const ROOT_AND_PATH: &str = "checked write needs a string root and path";

/// Run one script batch inside a host step. `writes` is a Luau list of
/// `{root = <string>, path = <string>, value = <scalar>}` tables with the keys
/// 1..n (at most 4096), where an absent value clears the property and its
/// paired marker. The step's rule operation is required. The writes are
/// checked against the installed rules as one replacement and applied all or
/// none: the result is `(true, None)` when applied, or `(false, Some(code))`
/// with the first refusing rule's code. A malformed list, a missing or
/// borrowed operation and every batch error (an unknown root or path, a
/// mismatched type, a repeated property, a limit) raise, and nothing is
/// written.
pub fn script_checked_value_write_batch(
    policy: &RuntimeValuePolicy,
    operation: Option<&RefCell<RuntimeValuePolicyOperation>>,
    roots: &BTreeMap<String, RuntimeOwnedViewModelHandle>,
    writes: Value,
) -> Result<(bool, Option<String>), String> {
    let entries = script_checked_value_batch_entries(writes)?;
    let mut operation = operation
        .ok_or("checked batches require a value rule operation")?
        .try_borrow_mut()
        .map_err(|_| "value rule operation is active")?;
    let result = runtime_checked_value_write_batch(policy, &mut operation, roots, entries)
        .map_err(|error| format!("checked value batch: {error:?}"))?;
    Ok((result.applied, result.refusal.map(|refusal| refusal.code)))
}

/// Read a Luau list of `{root = <string>, path = <string>, value = <scalar>}`
/// tables into batch entries, in list order. The keys must be exactly 1..n,
/// with n at most 4096. A boolean, number or string value is a candidate; an
/// absent value clears the property and its paired marker. Any other shape
/// fails here, before the batch runs, so nothing is written.
fn script_checked_value_batch_entries(
    writes: Value,
) -> Result<Vec<RuntimeCheckedValueBatchEntry>, String> {
    let list = writes
        .as_table()
        .ok_or("checked writes must be a list of tables")?;
    let mut pairs = Vec::new();
    for pair in list.pairs::<Value, Value>() {
        if pairs.len() >= MAX_SCRIPT_BATCH_WRITES {
            return Err(format!(
                "checked writes exceed {MAX_SCRIPT_BATCH_WRITES} entries"
            ));
        }
        pairs.push(pair.map_err(|error| error.to_string())?);
    }
    let mut entries = Vec::with_capacity(pairs.len());
    entries.resize_with(pairs.len(), || None);
    for (key, write) in pairs {
        let slot = key
            .as_integer()
            .and_then(|key| usize::try_from(key).ok())
            .and_then(|key| key.checked_sub(1))
            .and_then(|index| entries.get_mut(index))
            .ok_or(KEYS)?;
        *slot = Some(script_checked_value_batch_entry(&write)?);
    }
    // Table keys are distinct, so n keys inside 1..=n fill every slot.
    entries
        .into_iter()
        .collect::<Option<Vec<_>>>()
        .ok_or_else(|| KEYS.to_owned())
}

fn script_checked_value_batch_entry(
    write: &Value,
) -> Result<RuntimeCheckedValueBatchEntry, String> {
    let fields = write
        .as_table()
        .ok_or("checked write must be a table of root, path and value")?;
    let (mut root_name, mut path) = (None, None);
    let mut value = RuntimeCheckedValueInput::Clear;
    for pair in fields.pairs::<Value, Value>() {
        let (key, field) = pair.map_err(|error| error.to_string())?;
        match key.as_string().map(|key| key.as_bytes()).as_deref() {
            Some(b"root") => root_name = Some(script_text(&field)?),
            Some(b"path") => path = Some(script_text(&field)?),
            Some(b"value") => value = script_checked_value_input(&field)?,
            _ => return Err("checked write fields are root, path and value".into()),
        }
    }
    match (root_name, path) {
        (Some(root_name), Some(path)) => Ok(RuntimeCheckedValueBatchEntry {
            root_name,
            path,
            value,
        }),
        _ => Err(ROOT_AND_PATH.into()),
    }
}

fn script_text(value: &Value) -> Result<String, String> {
    value
        .as_string()
        .and_then(|text| text.to_str().ok())
        .ok_or_else(|| ROOT_AND_PATH.to_owned())
}

/// Read one script value as a checked write candidate: a boolean, number or
/// string is a candidate and nil clears the property and its paired marker.
/// Any other value raises "checked value must be a scalar or nil". Hosts use
/// it for their single checked write, and the batch uses it for each value.
pub fn script_checked_value_input(value: &Value) -> Result<RuntimeCheckedValueInput, String> {
    if value.is_nil() {
        Ok(RuntimeCheckedValueInput::Clear)
    } else if let Some(value) = value.as_boolean() {
        Ok(RuntimeCheckedValueInput::Boolean(value))
    } else if let Some(value) = value.as_number() {
        Ok(RuntimeCheckedValueInput::Number(value))
    } else if let Some(value) = value.as_string() {
        Ok(RuntimeCheckedValueInput::Text(value.as_bytes()))
    } else {
        Err("checked value must be a scalar or nil".into())
    }
}

/// Read `property` from every item of the list at `path` under the named root,
/// in list order, as a Luau list of booleans, numbers and strings. Items keep
/// their positions: a null item raises rather than being skipped. A path that
/// names any other kind of property gives nil. A missing root, path or item
/// property, or an item property that is not a scalar, raises.
pub fn script_list_property_values(
    lua: &Lua,
    policy: &RuntimeValuePolicy,
    roots: &BTreeMap<String, RuntimeOwnedViewModelHandle>,
    root_name: &str,
    path: &str,
    property: &str,
) -> Result<Value, String> {
    let read_error = |error: RuntimeValuePolicyError| format!("checked value read: {error:?}");
    let root = roots
        .get(root_name)
        .ok_or_else(|| read_error(RuntimeValuePolicyError::NotFound))?;
    policy.resolve_property(root, path).map_err(read_error)?;
    let Some(count) = root.list_item_count_by_property_name_path(path) else {
        return Ok(Value::Nil);
    };
    let values = lua
        .create_table_result()
        .map_err(|error| error.to_string())?;
    for (key, index) in (1_i64..).zip(0..count) {
        let item = root
            .list_item_by_property_name_path(path, index)
            .ok_or("checked list has a null item")?;
        let written = match policy.property_value(&item, property).map_err(read_error)? {
            RuntimeViewModelChangeValue::Boolean(value) => values.raw_set(key, value),
            RuntimeViewModelChangeValue::Number(value) => values.raw_set(key, f64::from(value)),
            RuntimeViewModelChangeValue::Color(value) => values.raw_set(key, value),
            RuntimeViewModelChangeValue::Enum(value) => values.raw_set(key, value),
            RuntimeViewModelChangeValue::String(text) => {
                values.raw_set(key, lua.create_string(&*text))
            }
            _ => return Err("checked list values must be scalars".into()),
        };
        written.map_err(|error| error.to_string())?;
    }
    Ok(Value::Table(values))
}
