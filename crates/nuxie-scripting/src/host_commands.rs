//! Product-neutral, result-based communication from authored scripts to hosts.
//!
//! The installed module has one operation, `command(name, payload)`. It never
//! invokes foreign code synchronously: commands are normalized into owned Rust
//! values and remain in a transaction-scoped FIFO until the caller commits or
//! rolls the transaction back.

use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, HashSet, VecDeque};
use std::rc::{Rc, Weak};

use luaur_rt::{Error, Lua, LuaString, MultiValue, Result, Table, Value};

use crate::vm::{ScriptResourceGuard, ScriptVm};

pub const MAX_HOST_MODULE_NAME_BYTES: usize = 4_096;
pub const MAX_HOST_COMMANDS_PER_CYCLE: usize = 4_096;
pub const MAX_HOST_VALUE_DEPTH: usize = 64;
pub const MAX_HOST_VALUE_NODES: usize = 65_536;
pub const MAX_HOST_IDENTIFIER_BYTES: usize = 4_096;
pub const MAX_HOST_STRING_BYTES: usize = 4 * 1024 * 1024;
pub const MAX_HOST_VALUE_BYTES: usize = 16 * 1024 * 1024;
pub const MAX_HOST_COMMAND_BYTES_PER_CYCLE: usize = 32 * 1024 * 1024;

const COMMANDS_CODE: &str = "script.resource.host_commands";
const IDENTIFIER_CODE: &str = "script.resource.host_identifier";
const STRING_CODE: &str = "script.resource.host_string";
const DEPTH_CODE: &str = "script.resource.host_value_depth";
const NODES_CODE: &str = "script.resource.host_value_nodes";
const VALUE_BYTES_CODE: &str = "script.resource.host_value_bytes";
const COMMAND_BYTES_CODE: &str = "script.resource.host_command_bytes";

/// Caller-selected bounds for one generic host-command module.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HostCommandLimits {
    max_commands_per_cycle: usize,
    max_value_depth: usize,
    max_value_nodes: usize,
    max_identifier_bytes: usize,
    max_string_bytes: usize,
    max_value_bytes: usize,
    max_command_bytes_per_cycle: usize,
}

impl HostCommandLimits {
    pub const fn new() -> Self {
        Self {
            max_commands_per_cycle: 256,
            max_value_depth: 32,
            max_value_nodes: 4_096,
            max_identifier_bytes: 4_096,
            max_string_bytes: 1024 * 1024,
            max_value_bytes: 4 * 1024 * 1024,
            max_command_bytes_per_cycle: 4 * 1024 * 1024,
        }
    }

    pub const fn with_max_commands_per_cycle(mut self, value: usize) -> Self {
        self.max_commands_per_cycle = value;
        self
    }

    pub const fn with_max_value_depth(mut self, value: usize) -> Self {
        self.max_value_depth = value;
        self
    }

    pub const fn with_max_value_nodes(mut self, value: usize) -> Self {
        self.max_value_nodes = value;
        self
    }

    pub const fn with_max_identifier_bytes(mut self, value: usize) -> Self {
        self.max_identifier_bytes = value;
        self
    }

    pub const fn with_max_string_bytes(mut self, value: usize) -> Self {
        self.max_string_bytes = value;
        self
    }

    pub const fn with_max_value_bytes(mut self, value: usize) -> Self {
        self.max_value_bytes = value;
        self
    }

    pub const fn with_max_command_bytes_per_cycle(mut self, value: usize) -> Self {
        self.max_command_bytes_per_cycle = value;
        self
    }

    pub const fn max_commands_per_cycle(self) -> usize {
        self.max_commands_per_cycle
    }

    pub const fn max_value_depth(self) -> usize {
        self.max_value_depth
    }

    pub const fn max_value_nodes(self) -> usize {
        self.max_value_nodes
    }

    pub const fn max_identifier_bytes(self) -> usize {
        self.max_identifier_bytes
    }

    pub const fn max_string_bytes(self) -> usize {
        self.max_string_bytes
    }

    pub const fn max_value_bytes(self) -> usize {
        self.max_value_bytes
    }

    pub const fn max_command_bytes_per_cycle(self) -> usize {
        self.max_command_bytes_per_cycle
    }

    pub fn validate(self) -> Result<()> {
        fn bounded(value: usize, maximum: usize, label: &str) -> Result<()> {
            if value == 0 || value > maximum {
                return Err(Error::runtime(format!(
                    "{label} must be between 1 and {maximum}"
                )));
            }
            Ok(())
        }

        bounded(
            self.max_commands_per_cycle,
            MAX_HOST_COMMANDS_PER_CYCLE,
            "host command count limit",
        )?;
        bounded(
            self.max_value_depth,
            MAX_HOST_VALUE_DEPTH,
            "host value depth limit",
        )?;
        bounded(
            self.max_value_nodes,
            MAX_HOST_VALUE_NODES,
            "host value node limit",
        )?;
        bounded(
            self.max_identifier_bytes,
            MAX_HOST_IDENTIFIER_BYTES,
            "host identifier byte limit",
        )?;
        bounded(
            self.max_string_bytes,
            MAX_HOST_STRING_BYTES,
            "host string byte limit",
        )?;
        bounded(
            self.max_value_bytes,
            MAX_HOST_VALUE_BYTES,
            "host value byte limit",
        )?;
        bounded(
            self.max_command_bytes_per_cycle,
            MAX_HOST_COMMAND_BYTES_PER_CYCLE,
            "host command byte limit",
        )
    }
}

impl Default for HostCommandLimits {
    fn default() -> Self {
        Self::new()
    }
}

/// A deterministic value detached from the script VM.
#[derive(Debug, Clone, PartialEq)]
pub enum HostValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    List(Vec<HostValue>),
    Object(BTreeMap<String, HostValue>),
}

/// One product-neutral command emitted by authored script.
#[derive(Debug, Clone, PartialEq)]
pub struct HostCommand {
    pub name: String,
    pub payload: HostValue,
}

#[derive(Clone)]
pub struct HostCommandHost {
    state: Rc<HostCommandState>,
}

impl std::fmt::Debug for HostCommandHost {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("HostCommandHost")
    }
}

struct HostCommandState {
    commands: RefCell<VecDeque<HostCommand>>,
    active: Cell<bool>,
    commands_this_cycle: Cell<usize>,
    nodes_this_cycle: Cell<usize>,
    bytes_this_cycle: Cell<usize>,
    limits: HostCommandLimits,
    resource_guard: ScriptResourceGuard,
}

pub struct HostCycleCheckpoint {
    state: Weak<HostCommandState>,
    queued_commands: usize,
}

pub struct HostEffectCheckpoint {
    state: Weak<HostCommandState>,
    queued_commands: usize,
}

impl HostCommandHost {
    pub fn install(vm: &ScriptVm, module_name: &str, limits: HostCommandLimits) -> Result<Self> {
        limits.validate()?;
        if module_name.is_empty() || module_name.len() > MAX_HOST_MODULE_NAME_BYTES {
            return Err(Error::runtime(format!(
                "host module name must contain 1 to {MAX_HOST_MODULE_NAME_BYTES} UTF-8 bytes"
            )));
        }
        let host = Self {
            state: Rc::new(HostCommandState {
                commands: RefCell::new(VecDeque::new()),
                active: Cell::new(false),
                commands_this_cycle: Cell::new(0),
                nodes_this_cycle: Cell::new(0),
                bytes_this_cycle: Cell::new(0),
                limits,
                resource_guard: vm.resource_guard(),
            }),
        };
        let module = host_module(vm.lua(), host.clone())?;
        vm.register_host_module(module_name, module)?;
        Ok(host)
    }

    pub fn begin_cycle(&self) -> HostCycleCheckpoint {
        self.state.commands.borrow_mut().clear();
        self.state.commands_this_cycle.set(0);
        self.state.nodes_this_cycle.set(0);
        self.state.bytes_this_cycle.set(0);
        self.state.active.set(true);
        HostCycleCheckpoint {
            state: Rc::downgrade(&self.state),
            queued_commands: 0,
        }
    }

    pub fn rollback_cycle(&self, checkpoint: HostCycleCheckpoint) {
        self.truncate(checkpoint.state, checkpoint.queued_commands);
        self.state.active.set(false);
    }

    pub fn checkpoint_effects(&self) -> HostEffectCheckpoint {
        HostEffectCheckpoint {
            state: Rc::downgrade(&self.state),
            queued_commands: self.state.commands.borrow().len(),
        }
    }

    pub fn rollback_effects(&self, checkpoint: HostEffectCheckpoint) {
        self.truncate(checkpoint.state, checkpoint.queued_commands);
    }

    pub fn drain(&self, checkpoint: HostCycleCheckpoint) -> Vec<HostCommand> {
        if !checkpoint
            .state
            .upgrade()
            .is_some_and(|state| Rc::ptr_eq(&state, &self.state))
        {
            return Vec::new();
        }
        self.state.active.set(false);
        self.state.commands.borrow_mut().drain(..).collect()
    }

    pub fn drain_effects(&self) -> Vec<HostCommand> {
        self.state.active.set(false);
        self.state.commands.borrow_mut().drain(..).collect()
    }

    /// Return the first authored callback failure observed by this VM during
    /// the active transaction. The runtime may preserve C++'s protected-call
    /// behavior internally, but a result-based host must reject the whole
    /// effect batch rather than publishing commands queued before that error.
    pub fn callback_failure(&self) -> Option<String> {
        self.state.resource_guard.callback_failure()
    }

    fn truncate(&self, state: Weak<HostCommandState>, queued_commands: usize) {
        if state
            .upgrade()
            .is_some_and(|state| Rc::ptr_eq(&state, &self.state))
        {
            self.state.commands.borrow_mut().truncate(queued_commands);
        }
    }

    fn push(&self, command: HostCommand) -> Result<()> {
        self.state.resource_guard.reject_if_tripped()?;
        if !self.state.active.get() {
            return Err(Error::runtime(
                "host commands may only be emitted during an active runtime transaction",
            ));
        }
        let count = self.state.commands_this_cycle.get();
        if count >= self.state.limits.max_commands_per_cycle {
            return Err(self.state.resource_guard.fail(
                COMMANDS_CODE,
                format!(
                    "script cycle exceeds {} host commands",
                    self.state.limits.max_commands_per_cycle
                ),
            ));
        }
        let bytes = host_command_bytes(&command);
        let nodes = self
            .state
            .nodes_this_cycle
            .get()
            .checked_add(host_value_nodes(&command.payload))
            .ok_or_else(|| {
                self.state
                    .resource_guard
                    .fail(NODES_CODE, "host value node count overflowed")
            })?;
        if nodes > self.state.limits.max_value_nodes {
            return Err(self.state.resource_guard.fail(
                NODES_CODE,
                format!(
                    "script cycle exceeds {} host value nodes",
                    self.state.limits.max_value_nodes
                ),
            ));
        }
        let total = self
            .state
            .bytes_this_cycle
            .get()
            .checked_add(bytes)
            .ok_or_else(|| {
                self.state
                    .resource_guard
                    .fail(COMMAND_BYTES_CODE, "host command byte count overflowed")
            })?;
        if total > self.state.limits.max_command_bytes_per_cycle {
            return Err(self.state.resource_guard.fail(
                COMMAND_BYTES_CODE,
                format!(
                    "script cycle exceeds {} bytes of host commands",
                    self.state.limits.max_command_bytes_per_cycle
                ),
            ));
        }
        self.state.commands.borrow_mut().push_back(command);
        self.state.commands_this_cycle.set(count + 1);
        self.state.nodes_this_cycle.set(nodes);
        self.state.bytes_this_cycle.set(total);
        Ok(())
    }
}

fn host_module(lua: &Lua, host: HostCommandHost) -> Result<Table> {
    let module = lua.create_table();
    module.set(
        "command",
        lua.create_function(move |_, mut args: MultiValue| {
            let name = required_identifier(
                args.pop_front(),
                "host command name",
                host.state.limits,
                &host.state.resource_guard,
            )?;
            let payload = host_value(
                args.pop_front().unwrap_or(Value::Nil),
                host.state.limits,
                host.state.resource_guard.clone(),
            )?;
            host.push(HostCommand { name, payload })?;
            Ok(Value::Nil)
        })?,
    )?;
    module.set_readonly(true);
    Ok(module)
}

struct Conversion {
    active_tables: HashSet<usize>,
    nodes: usize,
    bytes: usize,
    limits: HostCommandLimits,
    resource_guard: ScriptResourceGuard,
}

impl Conversion {
    fn account_node(&mut self) -> Result<()> {
        self.nodes = self.nodes.saturating_add(1);
        if self.nodes > self.limits.max_value_nodes {
            return Err(self.resource_guard.fail(
                NODES_CODE,
                format!("host value exceeds {} nodes", self.limits.max_value_nodes),
            ));
        }
        Ok(())
    }

    fn account_bytes(&mut self, bytes: usize) -> Result<()> {
        self.bytes = self.bytes.saturating_add(bytes);
        if self.bytes > self.limits.max_value_bytes {
            return Err(self.resource_guard.fail(
                VALUE_BYTES_CODE,
                format!(
                    "host value exceeds {} aggregate bytes",
                    self.limits.max_value_bytes
                ),
            ));
        }
        Ok(())
    }
}

fn host_value(
    value: Value,
    limits: HostCommandLimits,
    resource_guard: ScriptResourceGuard,
) -> Result<HostValue> {
    value_at_depth(
        value,
        1,
        &mut Conversion {
            active_tables: HashSet::new(),
            nodes: 0,
            bytes: 0,
            limits,
            resource_guard,
        },
    )
}

fn value_at_depth(value: Value, depth: usize, conversion: &mut Conversion) -> Result<HostValue> {
    if depth > conversion.limits.max_value_depth {
        return Err(conversion.resource_guard.fail(
            DEPTH_CODE,
            format!(
                "host value exceeds depth {}",
                conversion.limits.max_value_depth
            ),
        ));
    }
    conversion.account_node()?;
    match value {
        Value::Nil => Ok(HostValue::Null),
        Value::Boolean(value) => {
            conversion.account_bytes(1)?;
            Ok(HostValue::Bool(value))
        }
        Value::Integer(value) => {
            conversion.account_bytes(std::mem::size_of::<f64>())?;
            Ok(HostValue::Number(value as f64))
        }
        Value::Number(value) if value.is_finite() => {
            conversion.account_bytes(std::mem::size_of::<f64>())?;
            Ok(HostValue::Number(value))
        }
        Value::Number(_) => Err(Error::runtime("host command numbers must be finite")),
        Value::String(value) => {
            let value = checked_string(value, conversion.limits, &conversion.resource_guard)?;
            conversion.account_bytes(value.len())?;
            Ok(HostValue::String(value))
        }
        Value::Table(table) => table_value(table, depth, conversion),
        value => Err(Error::runtime(format!(
            "unsupported host command value type {}",
            value.type_name()
        ))),
    }
}

fn table_value(table: Table, depth: usize, conversion: &mut Conversion) -> Result<HostValue> {
    let identity = table.to_pointer() as usize;
    if !conversion.active_tables.insert(identity) {
        return Err(Error::runtime(
            "cyclic host command values are not supported",
        ));
    }
    let result = (|| {
        let mut object = BTreeMap::new();
        let mut list = Vec::new();
        let mut entry_count = 0_usize;
        for entry in table.pairs::<Value, Value>() {
            let (key, value) = entry?;
            entry_count = entry_count.saturating_add(1);
            match key {
                Value::String(key) if list.is_empty() => {
                    let key = checked_identifier(
                        key,
                        "host object key",
                        conversion.limits,
                        &conversion.resource_guard,
                    )?;
                    if key.is_empty() {
                        return Err(Error::runtime("host object keys must not be empty"));
                    }
                    conversion.account_bytes(key.len())?;
                    object.insert(key, value_at_depth(value, depth + 1, conversion)?);
                }
                Value::Integer(index) if object.is_empty() && index > 0 => {
                    let index = usize::try_from(index)
                        .map_err(|_| Error::runtime("mixed or invalid host table keys"))?;
                    list.push((index, value_at_depth(value, depth + 1, conversion)?));
                }
                Value::Number(index)
                    if object.is_empty()
                        && index.is_finite()
                        && index > 0.0
                        && index.fract() == 0.0
                        && index <= usize::MAX as f64 =>
                {
                    list.push((
                        index as usize,
                        value_at_depth(value, depth + 1, conversion)?,
                    ));
                }
                _ => return Err(Error::runtime("mixed or invalid host table keys")),
            }
        }
        if entry_count == 0 {
            return Ok(HostValue::Object(BTreeMap::new()));
        }
        if !object.is_empty() {
            return Ok(HostValue::Object(object));
        }
        list.sort_by_key(|(index, _)| *index);
        for (expected, (actual, _)) in (1..=list.len()).zip(&list) {
            if expected != *actual {
                return Err(Error::runtime("sparse host lists are not supported"));
            }
        }
        Ok(HostValue::List(
            list.into_iter().map(|(_, value)| value).collect(),
        ))
    })();
    conversion.active_tables.remove(&identity);
    result
}

fn required_identifier(
    value: Option<Value>,
    label: &str,
    limits: HostCommandLimits,
    resource_guard: &ScriptResourceGuard,
) -> Result<String> {
    let Some(Value::String(value)) = value else {
        return Err(Error::runtime(format!("{label} must be a string")));
    };
    let value = checked_identifier(value, label, limits, resource_guard)?;
    if value.is_empty() {
        return Err(Error::runtime(format!("{label} must not be empty")));
    }
    resource_guard.reject_if_tripped()?;
    Ok(value)
}

fn checked_identifier(
    value: LuaString,
    label: &str,
    limits: HostCommandLimits,
    resource_guard: &ScriptResourceGuard,
) -> Result<String> {
    let value = value.to_str()?;
    if value.len() > limits.max_identifier_bytes {
        return Err(resource_guard.fail(
            IDENTIFIER_CODE,
            format!(
                "{label} exceeds {} UTF-8 bytes",
                limits.max_identifier_bytes
            ),
        ));
    }
    Ok(value)
}

fn checked_string(
    value: LuaString,
    limits: HostCommandLimits,
    resource_guard: &ScriptResourceGuard,
) -> Result<String> {
    let value = value.to_str()?;
    if value.len() > limits.max_string_bytes {
        return Err(resource_guard.fail(
            STRING_CODE,
            format!(
                "host string exceeds {} UTF-8 bytes",
                limits.max_string_bytes
            ),
        ));
    }
    Ok(value)
}

fn host_command_bytes(command: &HostCommand) -> usize {
    command
        .name
        .len()
        .saturating_add(host_value_bytes(&command.payload))
}

fn host_value_bytes(value: &HostValue) -> usize {
    match value {
        HostValue::Null => 0,
        HostValue::Bool(_) => 1,
        HostValue::Number(_) => std::mem::size_of::<f64>(),
        HostValue::String(value) => value.len(),
        HostValue::List(values) => values
            .iter()
            .map(host_value_bytes)
            .fold(0, usize::saturating_add),
        HostValue::Object(values) => values.iter().fold(0, |bytes, (key, value)| {
            bytes
                .saturating_add(key.len())
                .saturating_add(host_value_bytes(value))
        }),
    }
}

fn host_value_nodes(value: &HostValue) -> usize {
    match value {
        HostValue::Null | HostValue::Bool(_) | HostValue::Number(_) | HostValue::String(_) => 1,
        HostValue::List(values) => values
            .iter()
            .map(host_value_nodes)
            .fold(1, usize::saturating_add),
        HostValue::Object(values) => values
            .values()
            .map(host_value_nodes)
            .fold(1, usize::saturating_add),
    }
}
