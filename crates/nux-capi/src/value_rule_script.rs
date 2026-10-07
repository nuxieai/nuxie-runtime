//! Generic synchronous checked writes injected by the embedding host.
use crate::*;

type FileSlot = Rc<RefCell<Option<nuxie::RuntimeFileWeakHandle>>>;
type Operation = Rc<RefCell<nuxie::RuntimeValuePolicyOperation>>;

pub(super) struct Extension {
    inner: Arc<dyn nuxie::ScriptHostExtension>,
    file: FileSlot,
    shares_command_module: bool,
}

impl std::fmt::Debug for Extension {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("ValueRuleScriptExtension")
    }
}

impl Extension {
    pub(super) fn wrap(
        inner: Arc<dyn nuxie::ScriptHostExtension>,
        shares_command_module: bool,
    ) -> (Arc<Self>, FileSlot) {
        let file = Rc::new(RefCell::new(None));
        (
            Arc::new(Self {
                inner,
                file: Rc::clone(&file),
                shares_command_module,
            }),
            file,
        )
    }
}

fn script_error(error: impl std::fmt::Display) -> nuxie::ScriptError {
    nuxie::ScriptError::new(error.to_string())
}

fn same_type<T>(_: &T, value: T) -> T {
    value
}

impl nuxie::ScriptHostExtension for Extension {
    fn install(
        &self,
        vm: &nuxie::ScriptVm,
    ) -> Result<Box<dyn nuxie::ScriptHostExtensionInstance>, nuxie::ScriptError> {
        let inner = self.inner.install(vm)?;
        let lua = vm.lua();
        let module = lua.create_table();
        // Preserve the caller's command function if it chose this exact name.
        let module = if self.shares_command_module {
            vm.registered_module("value_rules")
                .map_err(script_error)?
                .as_table()
                .cloned()
                .ok_or_else(|| script_error("host command module is unavailable"))?
        } else {
            module
        };
        module.set_readonly(false);
        let file = Rc::clone(&self.file);
        let setter = lua
            .create_function(move |lua, (root, path, value): (String, String, _)| {
                let value = same_type(&lua.pack(false)?, value);
                let input = if value.is_nil() {
                    Input::Clear
                } else if let Some(value) = value.as_boolean() {
                    Input::Boolean(value)
                } else if let Some(value) = value.as_number() {
                    Input::Number(value)
                } else if let Some(value) = value.as_string() {
                    Input::Text(value.as_bytes().to_vec())
                } else {
                    return Err("checked value must be a scalar or nil".into());
                };
                checked(&file, &root, &path, input).map_err(Into::into)
            })
            .map_err(script_error)?;
        module.set("set", setter).map_err(script_error)?;
        module.set_readonly(true);
        vm.register_host_module("value_rules", module)
            .map_err(script_error)?;
        Ok(inner)
    }
}

struct Context {
    file: RuntimeFileHandle,
    roots: std::collections::BTreeMap<String, RuntimeOwnedViewModelHandle>,
    policy: Rc<RefCell<Option<nuxie::RuntimeValuePolicy>>>,
    operation: Option<Operation>,
}

thread_local! {
    static ACTIVE: RefCell<Option<Rc<Context>>> = const { RefCell::new(None) };
}

pub(super) struct Guard(Option<Rc<Context>>);
impl Drop for Guard {
    fn drop(&mut self) {
        ACTIVE.with(|slot| *slot.borrow_mut() = self.0.take());
    }
}

pub(super) fn enter(
    player: &NuxPlayer,
    root: Option<&RuntimeOwnedViewModelHandle>,
    operation: Option<Operation>,
) -> Option<Guard> {
    let scripted = player.artboard.scripted.as_ref()?;
    let mut roots = player
        .global_view_models
        .values
        .borrow()
        .iter()
        .filter_map(|(index, value)| {
            player
                .view_model_catalog
                .global_schema_name(*index)
                .map(|name| (name.to_owned(), value.clone()))
        })
        .collect::<std::collections::BTreeMap<_, _>>();
    if let Some(root) = root {
        roots.insert(String::new(), root.clone());
    }
    let context = Rc::new(Context {
        file: scripted.native_file().clone(),
        roots,
        policy: Rc::clone(&player.view_model_catalog.value_policy),
        operation,
    });
    Some(Guard(ACTIVE.with(|slot| slot.replace(Some(context)))))
}

enum Input {
    Clear,
    Number(f64),
    Boolean(bool),
    Text(Vec<u8>),
}

fn checked(
    file: &FileSlot,
    root_name: &str,
    path: &str,
    input: Input,
) -> Result<(bool, Option<String>), String> {
    if root_name.len().saturating_add(path.len()) > MAX_PLAYER_STEP_RESULT_BYTES {
        return Err("checked path exceeds the operation limit".into());
    }
    let file = file
        .borrow()
        .as_ref()
        .and_then(|file| file.upgrade())
        .ok_or("value rule file is unavailable")?;
    let context = ACTIVE
        .with(|slot| slot.borrow().clone())
        .ok_or("checked writes require an active step")?;
    if !context.file.ptr_eq(&file) {
        return Err("checked write belongs to a different file".into());
    }
    let root = context
        .roots
        .get(root_name)
        .ok_or("checked write root was not installed by the host")?;
    let policy = context
        .policy
        .try_borrow()
        .map_err(|_| "value policy is active")?;
    let fallback = nuxie::RuntimeValuePolicy::new(file);
    let policy = policy.as_ref().unwrap_or(&fallback);
    let current = policy
        .property_value(root, path)
        .map_err(|error| format!("value property: {error:?}"))?;
    let marker = matches!(input, Input::Clear).then_some(false);
    let candidate = match (input, current) {
        (Input::Number(value), RuntimeViewModelChangeValue::Number(_)) => {
            RuntimeViewModelChangeValue::Number(value as f32)
        }
        (Input::Number(value), RuntimeViewModelChangeValue::Color(_))
            if value.is_finite()
                && value.fract() == 0.0
                && (0.0..=f64::from(u32::MAX)).contains(&value) =>
        {
            RuntimeViewModelChangeValue::Color(value as u32)
        }
        (Input::Number(value), RuntimeViewModelChangeValue::Enum(_))
            if value.is_finite()
                && value.fract() == 0.0
                && (0.0..=f64::from(u32::MAX)).contains(&value) =>
        {
            RuntimeViewModelChangeValue::Enum(value as u64)
        }
        (Input::Boolean(value), RuntimeViewModelChangeValue::Boolean(_)) => {
            RuntimeViewModelChangeValue::Boolean(value)
        }
        (Input::Text(value), RuntimeViewModelChangeValue::String(_))
            if value.len() <= MAX_PLAYER_STEP_RESULT_BYTES =>
        {
            RuntimeViewModelChangeValue::String(Arc::from(value))
        }
        (Input::Clear, RuntimeViewModelChangeValue::Number(_)) => {
            RuntimeViewModelChangeValue::Number(0.0)
        }
        (Input::Clear, RuntimeViewModelChangeValue::Boolean(_)) => {
            RuntimeViewModelChangeValue::Boolean(false)
        }
        (Input::Clear, RuntimeViewModelChangeValue::Color(_)) => {
            RuntimeViewModelChangeValue::Color(0)
        }
        (Input::Clear, RuntimeViewModelChangeValue::Enum(_)) => {
            RuntimeViewModelChangeValue::Enum(0)
        }
        (Input::Clear, RuntimeViewModelChangeValue::String(_)) => {
            RuntimeViewModelChangeValue::String(Arc::from([]))
        }
        (Input::Clear, RuntimeViewModelChangeValue::List(_)) => {
            RuntimeViewModelChangeValue::List(Vec::new())
        }
        _ => return Err("checked value does not match the property kind".into()),
    };
    let write = || {
        write_native(policy, root, path, &candidate)?;
        if marker == Some(false) {
            let (owner, index) = policy.resolve_property(root, path)?;
            if let Some(index) = policy.marker_property(&owner, index) {
                owner
                    .borrow_mut()
                    .set_boolean_by_property_index(index, false);
            }
        }
        Ok(())
    };
    if let Some(operation) = &context.operation {
        let mut operation = operation
            .try_borrow_mut()
            .map_err(|_| "value rule operation is active")?;
        operation
            .apply_pending_writes(policy, &context.roots.values().cloned().collect::<Vec<_>>())
            .map_err(|error| format!("pending value writes: {error:?}"))?;
        let result = operation
            .checked_write(policy, root, path, candidate.clone(), marker, write)
            .map_err(|error| format!("checked value write: {error:?}"))?;
        let code = result
            .rule_indices
            .first()
            .and_then(|index| policy.rule(*index))
            .map(|rule| rule.code.clone());
        Ok((result.applied, code))
    } else {
        write().map_err(|error| format!("value write: {error:?}"))?;
        Ok((true, None))
    }
}

fn write_native(
    policy: &nuxie::RuntimeValuePolicy,
    root: &RuntimeOwnedViewModelHandle,
    path: &str,
    candidate: &RuntimeViewModelChangeValue,
) -> Result<(), nuxie::RuntimeValuePolicyError> {
    let current = policy.property_value(root, path)?;
    if std::mem::discriminant(&current) != std::mem::discriminant(candidate) {
        return Err(nuxie::RuntimeValuePolicyError::InvalidArgument);
    }
    if matches!(candidate, RuntimeViewModelChangeValue::Enum(value) if u32::try_from(*value).is_err())
    {
        return Err(nuxie::RuntimeValuePolicyError::InvalidArgument);
    }
    let (owner, index) = policy.resolve_property(root, path)?;
    match candidate {
        RuntimeViewModelChangeValue::Number(value) => {
            owner
                .borrow_mut()
                .set_number_by_property_index(index, *value);
        }
        RuntimeViewModelChangeValue::Boolean(value) => {
            owner
                .borrow_mut()
                .set_boolean_by_property_index(index, *value);
        }
        RuntimeViewModelChangeValue::String(value) => {
            owner
                .borrow_mut()
                .set_string_by_property_index(index, value);
        }
        RuntimeViewModelChangeValue::Color(value) => {
            owner
                .borrow_mut()
                .set_color_by_property_index(index, *value);
        }
        RuntimeViewModelChangeValue::Enum(value) => {
            owner.borrow_mut().set_enum_by_property_index(index, *value);
        }
        RuntimeViewModelChangeValue::List(items) if items.is_empty() => {
            root.clear_list_items_by_property_name_path(path);
        }
        _ => return Err(nuxie::RuntimeValuePolicyError::InvalidArgument),
    }
    Ok(())
}

#[cfg(test)]
#[path = "value_rule_script_tests.rs"]
mod tests;
