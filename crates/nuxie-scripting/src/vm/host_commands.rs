use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, HashSet, VecDeque};
use std::rc::{Rc, Weak};

use luaur_rt::{Error, Lua, LuaString, MultiValue, Result, Table, Value};

use super::resource_limits::{ResourceLimitTracker, ScriptResourceLimit};

const MAX_HOST_VALUE_DEPTH: usize = 32;
const MAX_HOST_VALUE_NODES: usize = 4_096;
const MAX_HOST_VALUE_EDGES: usize = 16_384;
const MAX_HOST_IDENTIFIER_BYTES: usize = 4_096;
const MAX_HOST_STRING_BYTES: usize = 1024 * 1024;
const MAX_HOST_VALUE_BYTES: usize = 4 * 1024 * 1024;
const MAX_HOST_COMMANDS_PER_CYCLE: usize = 256;
const MAX_HOST_COMMAND_CONTENT_BYTES_PER_CYCLE: usize = 4 * 1024 * 1024;

struct HostValueConversion {
    active_tables: HashSet<usize>,
    nodes: usize,
    edges: usize,
    aggregate_bytes: usize,
    resource_limits: ResourceLimitTracker,
}

impl HostValueConversion {
    fn new(resource_limits: ResourceLimitTracker) -> Self {
        Self {
            active_tables: HashSet::new(),
            nodes: 0,
            edges: 0,
            aggregate_bytes: 0,
            resource_limits,
        }
    }

    fn account_node(&mut self) -> Result<()> {
        self.nodes += 1;
        if self.nodes > MAX_HOST_VALUE_NODES {
            return Err(self.resource_limits.fail(ScriptResourceLimit::HostNodes));
        }
        Ok(())
    }

    fn account_edge(&mut self) -> Result<()> {
        if self.edges >= MAX_HOST_VALUE_EDGES {
            return Err(self.resource_limits.fail(ScriptResourceLimit::HostEdges));
        }
        self.edges += 1;
        Ok(())
    }

    fn account_bytes(&mut self, bytes: usize) -> Result<()> {
        self.aggregate_bytes = self.aggregate_bytes.saturating_add(bytes);
        if self.aggregate_bytes > MAX_HOST_VALUE_BYTES {
            return Err(self
                .resource_limits
                .fail(ScriptResourceLimit::HostValueBytes));
        }
        Ok(())
    }
}

/// A deterministic, VM-independent value emitted by a Nuxie host command.
#[derive(Debug, Clone, PartialEq)]
pub enum HostValue {
    /// A Luau boolean.
    Bool(bool),
    /// A finite Luau number; integers are normalized to this representation.
    Number(f64),
    /// A valid UTF-8 Luau string.
    String(String),
    /// A dense, one-indexed Luau array in index order.
    Array(Vec<HostValue>),
    /// A Luau object keyed and ordered by UTF-8 strings.
    Object(BTreeMap<String, HostValue>),
}

/// One side effect requested through the private `require("nuxie")` module.
#[derive(Debug, Clone, PartialEq)]
pub enum HostCommand {
    /// Deliver a named trigger with normalized object properties.
    Trigger {
        name: String,
        properties: BTreeMap<String, HostValue>,
    },
    /// Assign one response field to a non-nil value.
    ResponseSet { field: String, value: HostValue },
}

#[derive(Clone)]
pub(super) struct HostCommandQueue {
    state: Rc<HostCommandState>,
}

struct HostCommandState {
    commands: RefCell<VecDeque<HostCommand>>,
    commands_this_cycle: Cell<usize>,
    host_nodes_this_cycle: Cell<usize>,
    host_edges_this_cycle: Cell<usize>,
    content_bytes_this_cycle: Cell<usize>,
    resource_limits: ResourceLimitTracker,
}

/// Opaque position used to discard commands from a failed script cycle.
pub struct HostCycleCheckpoint {
    effects: HostEffectCheckpoint,
}

/// Opaque position used to discard a single callback's host effects without
/// refunding any of the enclosing cycle's resource budgets.
pub struct HostEffectCheckpoint {
    state: Weak<HostCommandState>,
    queued_commands: usize,
}

impl HostCommandQueue {
    pub(super) fn new(resource_limits: ResourceLimitTracker) -> Self {
        Self {
            state: Rc::new(HostCommandState {
                commands: RefCell::new(VecDeque::new()),
                commands_this_cycle: Cell::new(0),
                host_nodes_this_cycle: Cell::new(0),
                host_edges_this_cycle: Cell::new(0),
                content_bytes_this_cycle: Cell::new(0),
                resource_limits,
            }),
        }
    }

    fn resource_limits(&self) -> ResourceLimitTracker {
        self.state.resource_limits.clone()
    }

    pub(super) fn begin_cycle(&self) -> HostCycleCheckpoint {
        self.state.commands_this_cycle.set(0);
        self.state.host_nodes_this_cycle.set(0);
        self.state.host_edges_this_cycle.set(0);
        self.state.content_bytes_this_cycle.set(0);
        HostCycleCheckpoint {
            effects: self.checkpoint_effects(),
        }
    }

    pub(super) fn rollback(&self, checkpoint: HostCycleCheckpoint) {
        self.rollback_effects(checkpoint.effects);
        self.state.commands_this_cycle.set(0);
        self.state.host_nodes_this_cycle.set(0);
        self.state.host_edges_this_cycle.set(0);
        self.state.content_bytes_this_cycle.set(0);
    }

    pub(super) fn checkpoint_effects(&self) -> HostEffectCheckpoint {
        HostEffectCheckpoint {
            state: Rc::downgrade(&self.state),
            queued_commands: self.state.commands.borrow().len(),
        }
    }

    pub(super) fn rollback_effects(&self, checkpoint: HostEffectCheckpoint) {
        let Some(checkpoint_state) = checkpoint.state.upgrade() else {
            return;
        };
        if !Rc::ptr_eq(&checkpoint_state, &self.state) {
            return;
        }
        self.state
            .commands
            .borrow_mut()
            .truncate(checkpoint.queued_commands);
    }

    pub(super) fn drain(&self) -> Vec<HostCommand> {
        self.state.commands.borrow_mut().drain(..).collect()
    }

    fn push(&self, command: HostCommand) -> Result<()> {
        self.state.resource_limits.reject_if_tripped()?;
        let emitted = self.state.commands_this_cycle.get();
        if emitted >= MAX_HOST_COMMANDS_PER_CYCLE {
            return Err(self
                .state
                .resource_limits
                .fail(ScriptResourceLimit::Commands));
        }
        let structure = host_command_structure(&command);
        let cycle_nodes = self
            .state
            .host_nodes_this_cycle
            .get()
            .checked_add(structure.nodes)
            .ok_or_else(|| {
                self.state
                    .resource_limits
                    .fail(ScriptResourceLimit::HostNodes)
            })?;
        if cycle_nodes > MAX_HOST_VALUE_NODES {
            return Err(self
                .state
                .resource_limits
                .fail(ScriptResourceLimit::HostNodes));
        }
        let cycle_edges = self
            .state
            .host_edges_this_cycle
            .get()
            .checked_add(structure.edges)
            .ok_or_else(|| {
                self.state
                    .resource_limits
                    .fail(ScriptResourceLimit::HostEdges)
            })?;
        if cycle_edges > MAX_HOST_VALUE_EDGES {
            return Err(self
                .state
                .resource_limits
                .fail(ScriptResourceLimit::HostEdges));
        }
        let content_bytes = host_command_content_bytes(&command);
        let cycle_content_bytes = self
            .state
            .content_bytes_this_cycle
            .get()
            .checked_add(content_bytes)
            .ok_or_else(|| {
                self.state
                    .resource_limits
                    .fail(ScriptResourceLimit::CommandContent)
            })?;
        if cycle_content_bytes > MAX_HOST_COMMAND_CONTENT_BYTES_PER_CYCLE {
            return Err(self
                .state
                .resource_limits
                .fail(ScriptResourceLimit::CommandContent));
        }
        self.state.commands.borrow_mut().push_back(command);
        self.state.commands_this_cycle.set(emitted + 1);
        self.state.host_nodes_this_cycle.set(cycle_nodes);
        self.state.host_edges_this_cycle.set(cycle_edges);
        self.state.content_bytes_this_cycle.set(cycle_content_bytes);
        Ok(())
    }
}

#[derive(Default)]
struct HostStructure {
    nodes: usize,
    edges: usize,
}

impl HostStructure {
    fn add(&mut self, other: Self) {
        self.nodes = self.nodes.saturating_add(other.nodes);
        self.edges = self.edges.saturating_add(other.edges);
    }
}

fn host_command_structure(command: &HostCommand) -> HostStructure {
    match command {
        HostCommand::Trigger { properties, .. } => {
            // A trigger's property map is the root of the normalized host value,
            // including the otherwise-empty root for a trigger without payload.
            let mut structure = HostStructure {
                nodes: 1,
                edges: properties.len(),
            };
            for value in properties.values() {
                structure.add(host_value_structure(value));
            }
            structure
        }
        HostCommand::ResponseSet { value, .. } => host_value_structure(value),
    }
}

fn host_value_structure(value: &HostValue) -> HostStructure {
    match value {
        HostValue::Bool(_) | HostValue::Number(_) | HostValue::String(_) => {
            HostStructure { nodes: 1, edges: 0 }
        }
        HostValue::Array(values) => {
            let mut structure = HostStructure {
                nodes: 1,
                edges: values.len(),
            };
            for value in values {
                structure.add(host_value_structure(value));
            }
            structure
        }
        HostValue::Object(values) => {
            let mut structure = HostStructure {
                nodes: 1,
                edges: values.len(),
            };
            for value in values.values() {
                structure.add(host_value_structure(value));
            }
            structure
        }
    }
}

fn host_command_content_bytes(command: &HostCommand) -> usize {
    match command {
        HostCommand::Trigger { name, properties } => {
            name.len()
                .saturating_add(properties.iter().fold(0_usize, |size, (key, value)| {
                    size.saturating_add(key.len())
                        .saturating_add(host_value_content_bytes(value))
                }))
        }
        HostCommand::ResponseSet { field, value } => "$response_set"
            .len()
            .saturating_add("field".len())
            .saturating_add(field.len())
            .saturating_add("value".len())
            .saturating_add(host_value_content_bytes(value)),
    }
}

fn host_value_content_bytes(value: &HostValue) -> usize {
    match value {
        HostValue::Bool(_) | HostValue::Number(_) => 16,
        HostValue::String(value) => value.len(),
        HostValue::Array(values) => values
            .iter()
            .map(host_value_content_bytes)
            .fold(0_usize, usize::saturating_add),
        HostValue::Object(values) => values.iter().fold(0_usize, |size, (key, value)| {
            size.saturating_add(key.len())
                .saturating_add(host_value_content_bytes(value))
        }),
    }
}

pub(super) fn install_nuxie_module(
    lua: &Lua,
    module_cache: &Table,
    queue: HostCommandQueue,
) -> Result<()> {
    let module = lua.create_table();

    let trigger_queue = queue.clone();
    module.set(
        "trigger",
        lua.create_function(move |_, mut args: MultiValue| {
            let resource_limits = trigger_queue.resource_limits();
            resource_limits.reject_if_tripped()?;
            let name =
                required_nonempty_string(args.pop_front(), "nuxie.trigger name", &resource_limits)?;
            let payload = args.pop_front().unwrap_or(Value::Nil);
            let properties = match payload {
                Value::Nil => BTreeMap::new(),
                Value::Table(table) => match host_table_value(table, resource_limits.clone())? {
                    HostValue::Object(properties) => properties,
                    value @ HostValue::Array(_) => {
                        trigger_collection_properties(value, resource_limits.clone())?
                    }
                    _ => unreachable!("a Lua table converts to a host collection"),
                },
                value => trigger_scalar_properties(value, resource_limits.clone())?,
            };
            trigger_queue.push(HostCommand::Trigger { name, properties })?;
            Ok(Value::Nil)
        })?,
    )?;

    let response = lua.create_table();
    response.set(
        "set",
        lua.create_function(move |_, mut args: MultiValue| {
            let resource_limits = queue.resource_limits();
            resource_limits.reject_if_tripped()?;
            let field = required_nonempty_string(
                args.pop_front(),
                "nuxie.response.set field",
                &resource_limits,
            )?;
            let raw_value = args.pop_front().unwrap_or(Value::Nil);
            if matches!(raw_value, Value::Nil) {
                return Ok(Value::Nil);
            }
            let value = host_value(raw_value, resource_limits)?;
            queue.push(HostCommand::ResponseSet { field, value })?;
            Ok(Value::Nil)
        })?,
    )?;
    response.set_readonly(true);
    module.set("response", response)?;
    module.set_readonly(true);
    module_cache.set("nuxie", module)
}

fn required_nonempty_string(
    value: Option<Value>,
    what: &str,
    resource_limits: &ResourceLimitTracker,
) -> Result<String> {
    match value {
        Some(Value::String(value)) => {
            let value = checked_host_string(value, resource_limits)?;
            if value.is_empty() {
                Err(Error::runtime(format!("{what} must not be empty")))
            } else if value.len() > MAX_HOST_IDENTIFIER_BYTES {
                Err(resource_limits.fail(ScriptResourceLimit::HostIdentifier))
            } else {
                Ok(value)
            }
        }
        Some(value) => Err(Error::runtime(format!(
            "{what} must be a string, got {}",
            value.type_name()
        ))),
        None => Err(Error::runtime(format!("{what} is required"))),
    }
}

fn trigger_scalar_properties(
    value: Value,
    resource_limits: ResourceLimitTracker,
) -> Result<BTreeMap<String, HostValue>> {
    let mut conversion = HostValueConversion::new(resource_limits);
    conversion.account_node()?;
    conversion.edges = 1;
    conversion.account_bytes("value".len())?;
    let value = host_value_at_depth(value, 2, &mut conversion)?;
    Ok(BTreeMap::from([("value".to_owned(), value)]))
}

fn trigger_collection_properties(
    value: HostValue,
    resource_limits: ResourceLimitTracker,
) -> Result<BTreeMap<String, HostValue>> {
    let mut conversion = HostValueConversion::new(resource_limits);
    conversion.account_node()?;
    conversion.edges = 1;
    conversion.account_bytes("value".len())?;
    validate_normalized_host_value(&value, 2, &mut conversion)?;
    Ok(BTreeMap::from([("value".to_owned(), value)]))
}

fn validate_normalized_host_value(
    value: &HostValue,
    depth: usize,
    conversion: &mut HostValueConversion,
) -> Result<()> {
    if depth > MAX_HOST_VALUE_DEPTH {
        return Err(conversion
            .resource_limits
            .fail(ScriptResourceLimit::HostDepth));
    }
    conversion.account_node()?;
    match value {
        HostValue::Bool(_) => conversion.account_bytes(1),
        HostValue::Number(_) => conversion.account_bytes(std::mem::size_of::<f64>()),
        HostValue::String(value) => conversion.account_bytes(value.len()),
        HostValue::Array(values) => {
            account_normalized_edges(conversion, values.len())?;
            for value in values {
                validate_normalized_host_value(value, depth + 1, conversion)?;
            }
            Ok(())
        }
        HostValue::Object(values) => {
            account_normalized_edges(conversion, values.len())?;
            for (key, value) in values {
                conversion.account_bytes(key.len())?;
                validate_normalized_host_value(value, depth + 1, conversion)?;
            }
            Ok(())
        }
    }
}

fn account_normalized_edges(conversion: &mut HostValueConversion, count: usize) -> Result<()> {
    conversion.edges = conversion.edges.saturating_add(count);
    if conversion.edges > MAX_HOST_VALUE_EDGES {
        return Err(conversion
            .resource_limits
            .fail(ScriptResourceLimit::HostEdges));
    }
    Ok(())
}

fn scalar_host_value(value: Value, conversion: &mut HostValueConversion) -> Result<HostValue> {
    match value {
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
        Value::Number(_) => Err(Error::runtime("host numbers must be finite")),
        Value::String(value) => {
            let value = checked_host_string(value, &conversion.resource_limits)?;
            conversion.account_bytes(value.len())?;
            Ok(HostValue::String(value))
        }
        value => Err(Error::runtime(format!(
            "unsupported host value type {}",
            value.type_name()
        ))),
    }
}

fn host_value(value: Value, resource_limits: ResourceLimitTracker) -> Result<HostValue> {
    host_value_at_depth(value, 1, &mut HostValueConversion::new(resource_limits))
}

fn host_table_value(table: Table, resource_limits: ResourceLimitTracker) -> Result<HostValue> {
    let mut conversion = HostValueConversion::new(resource_limits);
    conversion.account_node()?;
    table_host_value(table, 1, &mut conversion)
}

fn host_value_at_depth(
    value: Value,
    depth: usize,
    conversion: &mut HostValueConversion,
) -> Result<HostValue> {
    if depth > MAX_HOST_VALUE_DEPTH {
        return Err(conversion
            .resource_limits
            .fail(ScriptResourceLimit::HostDepth));
    }
    conversion.account_node()?;
    match value {
        Value::Table(table) => table_host_value(table, depth, conversion),
        value => scalar_host_value(value, conversion),
    }
}

fn table_host_value(
    table: Table,
    depth: usize,
    conversion: &mut HostValueConversion,
) -> Result<HostValue> {
    let identity = table.to_pointer() as usize;
    if !conversion.active_tables.insert(identity) {
        return Err(Error::runtime("cyclic host value is not supported"));
    }

    let result = (|| {
        // Preflight the edge budget incrementally. This preserves edge-limit
        // precedence over child-node validation without retaining every Lua
        // key and value in a temporary Vec.
        let mut entry_count = 0_usize;
        for entry in table.pairs::<Value, Value>() {
            conversion.account_edge()?;
            let _ = entry?;
            entry_count += 1;
        }
        if entry_count == 0 {
            return Ok(HostValue::Object(BTreeMap::new()));
        }

        let mut object = BTreeMap::new();
        let mut array = Vec::new();
        for entry in table.pairs::<Value, Value>() {
            let (key, value) = entry?;
            match key {
                Value::String(key) if array.is_empty() => {
                    let key = checked_host_string(key, &conversion.resource_limits)?;
                    if key.is_empty() {
                        return Err(Error::runtime("host object keys must not be empty"));
                    }
                    if key.len() > MAX_HOST_IDENTIFIER_BYTES {
                        return Err(conversion
                            .resource_limits
                            .fail(ScriptResourceLimit::HostIdentifier));
                    }
                    conversion.account_bytes(key.len())?;
                    object.insert(key, host_value_at_depth(value, depth + 1, conversion)?);
                }
                Value::Integer(index) if object.is_empty() && index > 0 => {
                    array.push((
                        index as usize,
                        host_value_at_depth(value, depth + 1, conversion)?,
                    ));
                }
                Value::Number(index)
                    if object.is_empty()
                        && index.is_finite()
                        && index > 0.0
                        && index.fract() == 0.0
                        && index <= usize::MAX as f64 =>
                {
                    array.push((
                        index as usize,
                        host_value_at_depth(value, depth + 1, conversion)?,
                    ));
                }
                Value::String(_) | Value::Integer(_) | Value::Number(_) => {
                    return Err(Error::runtime("mixed or invalid host table keys"));
                }
                key => {
                    return Err(Error::runtime(format!(
                        "unsupported host table key type {}",
                        key.type_name()
                    )));
                }
            }
        }

        if !object.is_empty() {
            return Ok(HostValue::Object(object));
        }

        array.sort_by_key(|(index, _)| *index);
        for (expected, (actual, _)) in (1..=array.len()).zip(&array) {
            if actual != &expected {
                return Err(Error::runtime("sparse host arrays are not supported"));
            }
        }
        Ok(HostValue::Array(
            array.into_iter().map(|(_, value)| value).collect(),
        ))
    })();

    conversion.active_tables.remove(&identity);
    result
}

fn checked_host_string(value: LuaString, resource_limits: &ResourceLimitTracker) -> Result<String> {
    let value = value.to_str()?;
    if value.len() > MAX_HOST_STRING_BYTES {
        return Err(resource_limits.fail(ScriptResourceLimit::HostString));
    }
    Ok(value)
}
