//! Product-neutral extension seam for converter payloads carried by standard
//! Rive `ScriptAsset` records.
//!
//! The baseline owns the bind graph and its runtime-value bridge. Product
//! crates may register a decoder/evaluator without teaching this crate their
//! durable vocabulary, serialization envelope, or error model.

use std::any::Any;
use std::collections::BTreeMap;
use std::fmt;
use std::sync::{Arc, OnceLock, RwLock};

#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeExternalDataValue {
    Null,
    Boolean(bool),
    Number(f64),
    String(String),
    List(Vec<Self>),
    Object(BTreeMap<String, Self>),
    Color(u32),
    Enum(u64),
    ListIndex(u64),
    Trigger(u64),
    Image(u64),
    ViewModel(RuntimeExternalViewModelReference),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeExternalViewModelReference {
    Null,
    DataContextRoot,
    Retained {
        allocation_identity: u64,
    },
    OwnedGenerated {
        view_model_index: usize,
        property_index: usize,
        path_key: u64,
    },
    Imported {
        object_id: u32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeExternalDataOutputType {
    String,
    Number,
    Boolean,
    Color,
    Enum,
    List,
    ListIndex,
    Object,
    Image,
    Trigger,
    ViewModel,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeExternalDataValuePath {
    Ids {
        path_ids: Vec<f64>,
        is_relative: bool,
        name_based: bool,
    },
    Path {
        path: String,
        view_model_name: Option<String>,
        is_relative: bool,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeExternalDataReverseResult {
    pub ok: bool,
    pub value: RuntimeExternalDataValue,
}

pub trait RuntimeExternalDataResolver {
    fn resolve_value(
        &mut self,
        path: &RuntimeExternalDataValuePath,
    ) -> Option<RuntimeExternalDataValue>;

    fn create_blank_view_model_instance(
        &mut self,
        view_model_id: &str,
    ) -> Option<RuntimeExternalDataValue>;
}

pub struct RuntimeExternalDataContext<'a> {
    pub now_ms: Option<f64>,
    pub resolver: Option<&'a mut dyn RuntimeExternalDataResolver>,
}

impl RuntimeExternalDataContext<'_> {
    pub const fn new() -> Self {
        Self {
            now_ms: None,
            resolver: None,
        }
    }

    pub fn resolver(&mut self) -> Option<&mut (dyn RuntimeExternalDataResolver + '_)> {
        match self.resolver.as_mut() {
            Some(resolver) => Some(&mut **resolver),
            None => None,
        }
    }
}

impl Default for RuntimeExternalDataContext<'_> {
    fn default() -> Self {
        Self::new()
    }
}

pub trait RuntimeExternalDataState: fmt::Debug {
    fn clone_box(&self) -> Box<dyn RuntimeExternalDataState>;
    fn as_any_mut(&mut self) -> &mut dyn Any;
    fn clear(&mut self);
    fn is_active(&self) -> bool;
}

impl Clone for Box<dyn RuntimeExternalDataState> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

pub trait RuntimeExternalDataProgram: fmt::Debug + Send + Sync {
    fn as_any(&self) -> &dyn Any;
    fn equals(&self, other: &dyn RuntimeExternalDataProgram) -> bool;
    fn output_type(&self) -> Option<RuntimeExternalDataOutputType>;
    fn is_stateful(&self) -> bool;
    fn is_reversible(&self) -> bool;
    fn value_paths(&self) -> Vec<RuntimeExternalDataValuePath>;
    fn runtime_view_model_index(&self, id: &str) -> Option<usize>;
    fn number_to_list_output_view_model_index(&self) -> Option<usize>;
    fn new_state(&self) -> Box<dyn RuntimeExternalDataState>;
    fn convert(
        &self,
        state: &mut dyn RuntimeExternalDataState,
        value: RuntimeExternalDataValue,
        context: &mut RuntimeExternalDataContext<'_>,
    ) -> Result<RuntimeExternalDataValue, String>;
    fn reverse_convert(
        &self,
        state: &mut dyn RuntimeExternalDataState,
        value: RuntimeExternalDataValue,
        context: &mut RuntimeExternalDataContext<'_>,
    ) -> Result<RuntimeExternalDataReverseResult, String>;
}

#[derive(Clone)]
pub struct RuntimeExternalDataProgramHandle(Arc<dyn RuntimeExternalDataProgram>);

impl RuntimeExternalDataProgramHandle {
    pub fn new(program: Arc<dyn RuntimeExternalDataProgram>) -> Self {
        Self(program)
    }

    pub fn program(&self) -> &(dyn RuntimeExternalDataProgram + 'static) {
        self.0.as_ref()
    }
}

impl std::ops::Deref for RuntimeExternalDataProgramHandle {
    type Target = dyn RuntimeExternalDataProgram + 'static;

    fn deref(&self) -> &Self::Target {
        self.program()
    }
}

impl fmt::Debug for RuntimeExternalDataProgramHandle {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl PartialEq for RuntimeExternalDataProgramHandle {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0) || self.0.equals(other.0.as_ref())
    }
}

pub trait RuntimeExternalDataRegistry: fmt::Debug + Send + Sync {
    fn registry_id(&self) -> &'static str;
    fn recognizes(&self, bytes: &[u8]) -> bool;
    fn decode(&self, bytes: &[u8]) -> Result<Option<RuntimeExternalDataProgramHandle>, String>;
}

fn registries() -> &'static RwLock<Vec<Arc<dyn RuntimeExternalDataRegistry>>> {
    static REGISTRIES: OnceLock<RwLock<Vec<Arc<dyn RuntimeExternalDataRegistry>>>> =
        OnceLock::new();
    REGISTRIES.get_or_init(|| RwLock::new(Vec::new()))
}

/// Install a product converter implementation once per process.
///
/// Re-registering the same stable id is an idempotent no-op. The registry is
/// intentionally neutral and empty in a baseline-only process.
pub fn register_runtime_external_data_registry(registry: Arc<dyn RuntimeExternalDataRegistry>) {
    let mut installed = match registries().write() {
        Ok(installed) => installed,
        Err(poisoned) => poisoned.into_inner(),
    };
    if installed
        .iter()
        .any(|candidate| candidate.registry_id() == registry.registry_id())
    {
        return;
    }
    installed.push(registry);
}

pub fn runtime_external_data_payload_is_claimed(bytes: &[u8]) -> bool {
    let installed = match registries().read() {
        Ok(installed) => installed,
        Err(poisoned) => poisoned.into_inner(),
    };
    installed.iter().any(|registry| registry.recognizes(bytes))
}

pub(crate) fn decode_runtime_external_data_program(
    bytes: &[u8],
) -> Result<Option<RuntimeExternalDataProgramHandle>, String> {
    let installed = match registries().read() {
        Ok(installed) => installed,
        Err(poisoned) => poisoned.into_inner(),
    };
    for registry in installed.iter() {
        if registry.recognizes(bytes) {
            return registry.decode(bytes);
        }
    }
    Ok(None)
}
