//! ProjectDO converter semantics, executed directly by the Rust runtime.
//!
//! These types deliberately model the durable ProjectDO vocabulary instead
//! of translating it into Rive-native converter records. A catalog is
//! validated and compiled once when a document is mounted; each binding owns
//! a small [`ProjectDataConverterState`] for stateful interpolation and list
//! materialization.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use serde::{Deserialize, Serialize};

/// A value accepted and produced by ProjectDO converters.
///
/// The scalar/list/object variants are ordinary JavaScript values. The typed
/// variants retain Rive identities that JavaScript cannot represent without
/// loss (notably 64-bit image ordinals and concrete ViewModel pointers). They
/// participate in pass-through conversion and explicit output coercion, but
/// are never approximated as floating-point identities.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ProjectDataValue {
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
    ViewModel(ProjectDataViewModelReference),
}

/// Exact runtime identity for a Project ViewModel value.
///
/// ProjectDO's JavaScript object shape cannot reconstruct these pointers from
/// an arbitrary object. The Rust bridge therefore supports them as opaque,
/// pass-through values and rejects incompatible object coercions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectDataViewModelReference {
    Null,
    DataContextRoot,
    OwnedGenerated {
        view_model_index: usize,
        property_index: usize,
        path_key: u64,
    },
    Imported {
        object_id: u32,
    },
}

/// One durable converter definition in a file-global catalog.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectDataConverterDefinition {
    pub id: String,
    pub spec: ProjectDataConverterSpec,
}

/// A converter and its optional forward-only output coercion.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ProjectDataConverterSpec {
    pub output_type: Option<ProjectDataConverterOutputType>,
    pub kind: ProjectDataConverterKind,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ProjectDataConverterKind {
    Template {
        template: String,
    },
    Validate {
        rule: ProjectDataConverterValidationRule,
        args: BTreeMap<String, ProjectDataValue>,
        invert: bool,
    },
    ListCount,
    Format {
        format: ProjectDataConverterFormat,
        locale: Option<String>,
        time_zone: Option<String>,
        options: BTreeMap<String, ProjectDataValue>,
        trim_zeros: bool,
        commas: bool,
        decimals: Option<u32>,
    },
    Map {
        cases: BTreeMap<String, ProjectDataValue>,
        reverse_map: Option<BTreeMap<String, ProjectDataValue>>,
    },
    Math {
        operation: ProjectDataConverterMathOperation,
        value: Option<f64>,
        value_path: Option<ProjectDataValuePath>,
    },
    RangeMap {
        min_input: f64,
        max_input: f64,
        min_output: f64,
        max_output: f64,
        clamp: ProjectDataConverterRangeClamp,
        reverse: bool,
        modulo: bool,
    },
    ToNumber,
    ToString {
        locale: Option<String>,
        decimals: Option<u32>,
        trim_zeros: bool,
        commas: bool,
    },
    StringTrim {
        mode: ProjectDataConverterStringTrimMode,
    },
    BooleanNot,
    Group {
        items: Vec<String>,
    },
    Formula {
        expression: String,
    },
    Interpolate {
        duration_ms: f64,
        easing: ProjectDataConverterEasing,
    },
    StringPad {
        length: f64,
        text: String,
        side: ProjectDataConverterStringPadSide,
    },
    NumberToList {
        view_model_id: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectDataConverterOutputType {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectDataConverterValidationRule {
    NonEmpty,
    Min,
    Max,
    Range,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectDataConverterFormat {
    Currency,
    Date,
    Number,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectDataConverterMathOperation {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Round,
    Floor,
    Ceil,
    SquareRoot,
    Power,
    Exponential,
    NaturalLog,
    Cosine,
    Sine,
    Tangent,
    ArcCosine,
    ArcSine,
    ArcTangent,
    ArcTangent2,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ProjectDataValuePath {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectDataConverterRangeClamp {
    None,
    Lower,
    Upper,
    Both,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectDataConverterStringTrimMode {
    Start,
    End,
    All,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectDataConverterEasing {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProjectDataConverterStringPadSide {
    Start,
    End,
}

/// A mount-time error. Unsupported `Intl` combinations are errors rather
/// than locale-dependent approximations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectDataConverterCompileError {
    EmptyId,
    DuplicateId {
        id: String,
    },
    UnknownGroupItem {
        converter: String,
        item: String,
    },
    GroupCycle {
        converter: String,
    },
    GroupNestingTooDeep {
        converter: String,
        maximum: usize,
    },
    NonFinite {
        converter: String,
        field: &'static str,
    },
    InvalidFormula {
        converter: String,
        offset: usize,
        message: &'static str,
    },
    UnsupportedFormat {
        converter: String,
        format: ProjectDataConverterFormat,
        reason: String,
    },
}

impl fmt::Display for ProjectDataConverterCompileError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyId => write!(formatter, "project converter id must not be empty"),
            Self::DuplicateId { id } => write!(formatter, "duplicate project converter '{id}'"),
            Self::UnknownGroupItem { converter, item } => write!(
                formatter,
                "project converter '{converter}' references unknown group item '{item}'"
            ),
            Self::GroupCycle { converter } => {
                write!(
                    formatter,
                    "project converter group cycle reaches '{converter}'"
                )
            }
            Self::GroupNestingTooDeep { converter, maximum } => write!(
                formatter,
                "project converter group nesting reaches '{converter}' beyond the runtime limit of {maximum}"
            ),
            Self::NonFinite { converter, field } => write!(
                formatter,
                "project converter '{converter}' has non-finite {field}"
            ),
            Self::InvalidFormula {
                converter,
                offset,
                message,
            } => write!(
                formatter,
                "project converter '{converter}' has invalid formula at byte {offset}: {message}"
            ),
            Self::UnsupportedFormat {
                converter,
                format,
                reason,
            } => write!(
                formatter,
                "project converter '{converter}' uses unsupported {format:?} formatting: {reason}"
            ),
        }
    }
}

impl std::error::Error for ProjectDataConverterCompileError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectDataConverterRuntimeError {
    UnknownConverter {
        id: String,
    },
    GroupCycle {
        id: String,
    },
    ValueTooLarge {
        converter: String,
        maximum_bytes: usize,
    },
    ValueTooComplex {
        converter: String,
    },
}

impl fmt::Display for ProjectDataConverterRuntimeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownConverter { id } => write!(formatter, "unknown project converter '{id}'"),
            Self::GroupCycle { id } => {
                write!(formatter, "project converter group cycle reaches '{id}'")
            }
            Self::ValueTooLarge {
                converter,
                maximum_bytes,
            } => write!(
                formatter,
                "project converter '{converter}' exceeds the {maximum_bytes}-byte runtime value limit"
            ),
            Self::ValueTooComplex { converter } => write!(
                formatter,
                "project converter '{converter}' exceeds the runtime value nesting limit"
            ),
        }
    }
}

impl std::error::Error for ProjectDataConverterRuntimeError {}

/// Runtime services needed by semantic operands and list materialization.
pub trait ProjectDataConverterResolver {
    fn resolve_value(&mut self, path: &ProjectDataValuePath) -> Option<ProjectDataValue>;

    fn create_blank_view_model_instance(&mut self, view_model_id: &str)
    -> Option<ProjectDataValue>;
}

/// Per-application context. Time is deliberately explicit and expressed in
/// milliseconds to match the ProjectDO/React contract.
pub struct ProjectDataConverterContext<'a> {
    pub now_ms: Option<f64>,
    pub resolver: Option<&'a mut dyn ProjectDataConverterResolver>,
}

impl ProjectDataConverterContext<'_> {
    pub const fn new() -> Self {
        Self {
            now_ms: None,
            resolver: None,
        }
    }
}

impl Default for ProjectDataConverterContext<'_> {
    fn default() -> Self {
        Self::new()
    }
}

/// Stateful data owned by one mounted binding occurrence.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ProjectDataConverterState {
    root_id: Option<String>,
    root: ProjectDataConverterStateNode,
}

#[derive(Debug, Clone, Default, PartialEq)]
struct ProjectDataConverterStateNode {
    cache: Option<ProjectDataConverterCache>,
    group_items: Vec<ProjectDataConverterStateNode>,
}

impl ProjectDataConverterState {
    pub fn clear(&mut self) {
        self.root_id = None;
        self.root = ProjectDataConverterStateNode::default();
    }

    pub fn is_interpolating(&self) -> bool {
        self.root.is_interpolating()
    }

    fn root(&mut self, id: &str) -> &mut ProjectDataConverterStateNode {
        if self.root_id.as_deref() != Some(id) {
            self.root_id = Some(id.to_owned());
            self.root = ProjectDataConverterStateNode::default();
        }
        &mut self.root
    }
}

impl ProjectDataConverterStateNode {
    fn group_item(&mut self, position: usize) -> &mut Self {
        if self.group_items.len() <= position {
            self.group_items.resize_with(position + 1, Self::default);
        }
        &mut self.group_items[position]
    }

    fn is_interpolating(&self) -> bool {
        matches!(
            self.cache,
            Some(ProjectDataConverterCache::Interpolate {
                to,
                last_value,
                ..
            }) if to != last_value
        ) || self.group_items.iter().any(Self::is_interpolating)
    }
}

#[derive(Debug, Clone, PartialEq)]
enum ProjectDataConverterCache {
    Interpolate {
        from: f64,
        to: f64,
        start_ms: f64,
        duration_ms: f64,
        last_value: f64,
    },
    NumberToList {
        view_model_id: String,
        items: Vec<ProjectDataValue>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ProjectDataConverterReverseResult {
    pub ok: bool,
    pub value: ProjectDataValue,
}

const PROJECT_DATA_CONVERTER_PROGRAM_MAGIC: &[u8; 8] = b"NUXPCV1\0";
const PROJECT_DATA_CONVERTER_PROGRAM_MAX_BYTES: usize = 4 * 1024 * 1024;
const PROJECT_DATA_CONVERTER_PROGRAM_MAX_DEFINITIONS: usize = 4096;
const PROJECT_DATA_CONVERTER_MAX_DECIMALS: u32 = 100;
const PROJECT_DATA_CONVERTER_MAX_STRING_PAD_UTF16_UNITS: usize = 1_000_000;
pub(crate) const PROJECT_DATA_CONVERTER_MAX_LIST_ITEMS: usize = 10_000;
const PROJECT_DATA_CONVERTER_MAX_FORMULA_NESTING: usize = 64;
const PROJECT_DATA_CONVERTER_MAX_FORMULA_NODES: usize = 256;
const PROJECT_DATA_CONVERTER_MAX_GROUP_NESTING: usize = 64;
const PROJECT_DATA_CONVERTER_MAX_VALUE_BYTES: usize = 4 * 1024 * 1024;
const PROJECT_DATA_CONVERTER_MAX_VALUE_NESTING: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectDataConverterProgramError {
    UnknownRoot { id: String },
    TooLarge { bytes: usize },
    TooManyDefinitions { count: usize },
    InvalidPayload { reason: String },
    InvalidCatalog(ProjectDataConverterCompileError),
}

impl fmt::Display for ProjectDataConverterProgramError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownRoot { id } => write!(formatter, "unknown project converter root '{id}'"),
            Self::TooLarge { bytes } => write!(
                formatter,
                "project converter program is {bytes} bytes, exceeding the runtime limit"
            ),
            Self::TooManyDefinitions { count } => write!(
                formatter,
                "project converter program has {count} definitions, exceeding the runtime limit"
            ),
            Self::InvalidPayload { reason } => {
                write!(formatter, "invalid project converter program: {reason}")
            }
            Self::InvalidCatalog(error) => {
                write!(formatter, "invalid project converter catalog: {error}")
            }
        }
    }
}

impl std::error::Error for ProjectDataConverterProgramError {}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SerializedProjectDataConverterProgram {
    root: String,
    definitions: Vec<ProjectDataConverterDefinition>,
    #[serde(default)]
    runtime_view_models: BTreeMap<String, u32>,
}

/// One self-contained converter program stored in a `ScriptAsset` payload.
///
/// The envelope is not executable bytecode. It is decoded, validated, and
/// evaluated by the pure-Rust runtime, including for unsigned imported files.
#[derive(Debug, Clone)]
pub struct ProjectDataConverterProgram {
    root: String,
    catalog: ProjectDataConverterCatalog,
    runtime_view_models: BTreeMap<String, u32>,
}

impl PartialEq for ProjectDataConverterProgram {
    fn eq(&self, other: &Self) -> bool {
        self.root == other.root
            && self.catalog.converters == other.catalog.converters
            && self.runtime_view_models == other.runtime_view_models
    }
}

impl ProjectDataConverterProgram {
    pub fn root(&self) -> &str {
        &self.root
    }

    pub fn is_envelope(bytes: &[u8]) -> bool {
        project_data_converter_envelope(bytes).is_some()
    }

    pub fn is_stateful(&self) -> bool {
        self.catalog.converter_is_stateful(&self.root)
    }

    pub fn is_reversible(&self) -> bool {
        self.catalog.converter_is_reversible(&self.root)
    }

    /// Effective typed output of this root, including the final member of a
    /// group. The runtime bridge uses this metadata to reconstruct Rive's
    /// typed value instead of collapsing every scalar to `Number`.
    pub fn output_type(&self) -> Option<ProjectDataConverterOutputType> {
        self.catalog
            .effective_outputs
            .get(&self.root)
            .copied()
            .flatten()
    }

    pub fn value_paths(&self) -> Vec<ProjectDataValuePath> {
        let mut paths = Vec::new();
        self.catalog
            .collect_value_paths(&self.root, &mut paths, &mut BTreeSet::new());
        paths
    }

    /// Resolve one durable ProjectDO ViewModel identity to the file-local Rive
    /// ViewModel ordinal captured when this exact program was authored.
    pub fn runtime_view_model_index(&self, id: &str) -> Option<usize> {
        self.runtime_view_models
            .get(id)
            .and_then(|index| usize::try_from(*index).ok())
    }

    /// Return the runtime ViewModel used by a root whose effective output is a
    /// NumberToList. Groups follow their final converter, matching conversion.
    pub fn number_to_list_output_view_model_index(&self) -> Option<usize> {
        let id = self
            .catalog
            .number_to_list_output_view_model_id_for(&self.root)?;
        self.runtime_view_model_index(id)
    }

    /// Decode a Project converter envelope, or return `Ok(None)` for ordinary
    /// Luau/script bytes.
    pub fn decode(bytes: &[u8]) -> Result<Option<Self>, ProjectDataConverterProgramError> {
        if !Self::is_envelope(bytes) {
            return Ok(None);
        }
        if bytes.len() > PROJECT_DATA_CONVERTER_PROGRAM_MAX_BYTES {
            return Err(ProjectDataConverterProgramError::TooLarge { bytes: bytes.len() });
        }
        let envelope = project_data_converter_envelope(bytes).ok_or_else(|| {
            ProjectDataConverterProgramError::InvalidPayload {
                reason: "missing payload".to_owned(),
            }
        })?;
        let payload = envelope
            .get(PROJECT_DATA_CONVERTER_PROGRAM_MAGIC.len()..)
            .ok_or_else(|| ProjectDataConverterProgramError::InvalidPayload {
                reason: "missing payload".to_owned(),
            })?;
        let serialized: SerializedProjectDataConverterProgram = serde_json::from_slice(payload)
            .map_err(|error| ProjectDataConverterProgramError::InvalidPayload {
                reason: error.to_string(),
            })?;
        if serialized.definitions.len() > PROJECT_DATA_CONVERTER_PROGRAM_MAX_DEFINITIONS {
            return Err(ProjectDataConverterProgramError::TooManyDefinitions {
                count: serialized.definitions.len(),
            });
        }
        let catalog = ProjectDataConverterCatalog::compile(serialized.definitions)
            .map_err(ProjectDataConverterProgramError::InvalidCatalog)?;
        if !catalog.contains(&serialized.root) {
            return Err(ProjectDataConverterProgramError::UnknownRoot {
                id: serialized.root,
            });
        }
        Ok(Some(Self {
            root: serialized.root,
            catalog,
            runtime_view_models: serialized.runtime_view_models,
        }))
    }

    pub fn convert(
        &self,
        state: &mut ProjectDataConverterState,
        value: ProjectDataValue,
        context: &mut ProjectDataConverterContext<'_>,
    ) -> Result<ProjectDataValue, ProjectDataConverterRuntimeError> {
        self.catalog.convert(&self.root, state, value, context)
    }

    pub fn reverse_convert(
        &self,
        state: &mut ProjectDataConverterState,
        value: ProjectDataValue,
        context: &mut ProjectDataConverterContext<'_>,
    ) -> Result<ProjectDataConverterReverseResult, ProjectDataConverterRuntimeError> {
        self.catalog
            .reverse_convert(&self.root, state, value, context)
    }
}

fn project_data_converter_envelope(bytes: &[u8]) -> Option<&[u8]> {
    if bytes.starts_with(PROJECT_DATA_CONVERTER_PROGRAM_MAGIC) {
        return Some(bytes);
    }
    let unsigned_payload = bytes.strip_prefix(&[0])?;
    unsigned_payload
        .starts_with(PROJECT_DATA_CONVERTER_PROGRAM_MAGIC)
        .then_some(unsigned_payload)
}

/// A validated catalog compiled once per mounted ProjectDO document.
#[derive(Debug, Clone)]
pub struct ProjectDataConverterCatalog {
    converters: BTreeMap<String, ProjectDataConverterSpec>,
    formulas: BTreeMap<String, ProjectFormula>,
    effective_outputs: BTreeMap<String, Option<ProjectDataConverterOutputType>>,
}

impl ProjectDataConverterCatalog {
    pub fn compile(
        definitions: impl IntoIterator<Item = ProjectDataConverterDefinition>,
    ) -> Result<Self, ProjectDataConverterCompileError> {
        let mut converters = BTreeMap::new();
        for definition in definitions {
            if definition.id.is_empty() {
                return Err(ProjectDataConverterCompileError::EmptyId);
            }
            if converters
                .insert(definition.id.clone(), definition.spec)
                .is_some()
            {
                return Err(ProjectDataConverterCompileError::DuplicateId { id: definition.id });
            }
        }

        for (id, spec) in &converters {
            validate_converter(id, spec, &converters)?;
        }
        validate_group_cycles(&converters)?;

        let mut formulas = BTreeMap::new();
        for (id, spec) in &converters {
            if let ProjectDataConverterKind::Formula { expression } = &spec.kind {
                let formula = FormulaParser::parse(expression).map_err(|error| {
                    ProjectDataConverterCompileError::InvalidFormula {
                        converter: id.clone(),
                        offset: error.offset,
                        message: error.message,
                    }
                })?;
                formulas.insert(id.clone(), formula);
            }
        }

        let mut effective_outputs = BTreeMap::new();
        for id in converters.keys() {
            let output = effective_output_for(id, &converters, &mut effective_outputs);
            effective_outputs.insert(id.clone(), output);
        }
        Ok(Self {
            converters,
            formulas,
            effective_outputs,
        })
    }

    pub fn is_empty(&self) -> bool {
        self.converters.is_empty()
    }

    pub fn len(&self) -> usize {
        self.converters.len()
    }

    pub fn contains(&self, id: &str) -> bool {
        self.converters.contains_key(id)
    }

    /// Return the distinct value paths reachable from one converter root.
    ///
    /// Authoring layers use this to prove that every ProjectDO operand path has
    /// a semantic runtime source before an exact artifact is emitted. The
    /// returned paths retain their durable ProjectDO representation; file-local
    /// Rive ordinals are deliberately not part of this API.
    pub fn value_paths(
        &self,
        root: &str,
    ) -> Result<Vec<ProjectDataValuePath>, ProjectDataConverterProgramError> {
        if !self.contains(root) {
            return Err(ProjectDataConverterProgramError::UnknownRoot {
                id: root.to_owned(),
            });
        }
        let mut paths = Vec::new();
        self.collect_value_paths(root, &mut paths, &mut BTreeSet::new());
        Ok(paths)
    }

    /// Return the distinct durable ViewModel identities required by reachable
    /// NumberToList converters under one root.
    pub fn view_model_ids(
        &self,
        root: &str,
    ) -> Result<Vec<String>, ProjectDataConverterProgramError> {
        if !self.contains(root) {
            return Err(ProjectDataConverterProgramError::UnknownRoot {
                id: root.to_owned(),
            });
        }
        let mut ids = Vec::new();
        self.collect_view_model_ids(root, &mut ids, &mut BTreeSet::new());
        Ok(ids)
    }

    /// Whether the selected converter root has a characterized reverse path.
    ///
    /// This is intentionally conservative for operations that can lose
    /// information (clamping, modulo, dynamic zero operands, and formatting).
    pub fn is_reversible(&self, root: &str) -> Result<bool, ProjectDataConverterProgramError> {
        if !self.contains(root) {
            return Err(ProjectDataConverterProgramError::UnknownRoot {
                id: root.to_owned(),
            });
        }
        Ok(self.converter_is_reversible(root))
    }

    /// Effective output metadata for one converter root.
    pub fn output_type(
        &self,
        root: &str,
    ) -> Result<Option<ProjectDataConverterOutputType>, ProjectDataConverterProgramError> {
        if !self.contains(root) {
            return Err(ProjectDataConverterProgramError::UnknownRoot {
                id: root.to_owned(),
            });
        }
        Ok(self.effective_outputs.get(root).copied().flatten())
    }

    /// Clone this catalog while replacing exact durable value paths.
    ///
    /// This is the narrow lowering seam used by document authoring. It avoids
    /// exposing converter definitions merely so another layer can reconstruct
    /// and mutate the catalog. Replacements that are not present in a given
    /// root are harmless, which lets a caller provide bindings for a complete
    /// catalog while materializing one root at a time.
    pub fn replace_value_paths(
        &self,
        replacements: &[(ProjectDataValuePath, ProjectDataValuePath)],
    ) -> Result<Self, ProjectDataConverterCompileError> {
        let definitions = self
            .converters
            .iter()
            .map(|(id, spec)| {
                let mut spec = spec.clone();
                if let ProjectDataConverterKind::Math {
                    value_path: Some(path),
                    ..
                } = &mut spec.kind
                    && let Some((_, replacement)) =
                        replacements.iter().find(|(candidate, _)| candidate == path)
                {
                    path.clone_from(replacement);
                }
                ProjectDataConverterDefinition {
                    id: id.clone(),
                    spec,
                }
            })
            .collect::<Vec<_>>();
        Self::compile(definitions)
    }

    fn converter_is_stateful(&self, id: &str) -> bool {
        match self.converters.get(id).map(|spec| &spec.kind) {
            Some(ProjectDataConverterKind::Interpolate { .. }) => true,
            Some(ProjectDataConverterKind::Group { items }) => {
                items.iter().any(|item| self.converter_is_stateful(item))
            }
            _ => false,
        }
    }

    fn converter_is_reversible(&self, id: &str) -> bool {
        match self.converters.get(id).map(|spec| &spec.kind) {
            Some(ProjectDataConverterKind::BooleanNot) => true,
            Some(ProjectDataConverterKind::Math {
                operation,
                value,
                value_path,
            }) => match operation {
                ProjectDataConverterMathOperation::Add
                | ProjectDataConverterMathOperation::Subtract => true,
                ProjectDataConverterMathOperation::Multiply
                | ProjectDataConverterMathOperation::Divide => {
                    value_path.is_none() && value.is_none_or(|value| value != 0.0)
                }
                _ => false,
            },
            Some(ProjectDataConverterKind::RangeMap {
                min_input,
                max_input,
                min_output,
                max_output,
                clamp,
                modulo,
                ..
            }) => {
                min_input != max_input
                    && min_output != max_output
                    && *clamp == ProjectDataConverterRangeClamp::None
                    && !modulo
            }
            Some(ProjectDataConverterKind::Map { cases, reverse_map }) => match reverse_map {
                Some(reverse) => !reverse.is_empty(),
                None => !cases.is_empty() && build_reverse_map(cases).is_some(),
            },
            Some(ProjectDataConverterKind::Group { items }) => {
                items.iter().all(|item| self.converter_is_reversible(item))
            }
            _ => false,
        }
    }

    fn collect_value_paths(
        &self,
        id: &str,
        paths: &mut Vec<ProjectDataValuePath>,
        visiting: &mut BTreeSet<String>,
    ) {
        if !visiting.insert(id.to_owned()) {
            return;
        }
        match self.converters.get(id).map(|spec| &spec.kind) {
            Some(ProjectDataConverterKind::Math {
                value_path: Some(path),
                ..
            }) => {
                if !paths.contains(path) {
                    paths.push(path.clone());
                }
            }
            Some(ProjectDataConverterKind::Group { items }) => {
                for item in items {
                    self.collect_value_paths(item, paths, visiting);
                }
            }
            _ => {}
        }
        visiting.remove(id);
    }

    fn collect_view_model_ids(
        &self,
        id: &str,
        view_model_ids: &mut Vec<String>,
        visiting: &mut BTreeSet<String>,
    ) {
        if !visiting.insert(id.to_owned()) {
            return;
        }
        match self.converters.get(id).map(|spec| &spec.kind) {
            Some(ProjectDataConverterKind::NumberToList { view_model_id }) => {
                if !view_model_ids.contains(view_model_id) {
                    view_model_ids.push(view_model_id.clone());
                }
            }
            Some(ProjectDataConverterKind::Group { items }) => {
                for item in items {
                    self.collect_view_model_ids(item, view_model_ids, visiting);
                }
            }
            _ => {}
        }
        visiting.remove(id);
    }

    fn number_to_list_output_view_model_id_for(&self, id: &str) -> Option<&str> {
        if self.effective_outputs.get(id).copied().flatten()
            != Some(ProjectDataConverterOutputType::List)
        {
            return None;
        }
        match self.converters.get(id).map(|spec| &spec.kind)? {
            ProjectDataConverterKind::NumberToList { view_model_id } => Some(view_model_id),
            ProjectDataConverterKind::Group { items } => {
                let output_item = items.iter().rev().find(|item| {
                    self.effective_outputs
                        .get(item.as_str())
                        .copied()
                        .flatten()
                        .is_some()
                })?;
                self.number_to_list_output_view_model_id_for(output_item)
            }
            _ => None,
        }
    }

    /// Encode a self-contained program for one root converter. The complete
    /// catalog is retained so group references keep their durable identities.
    pub fn encode_program(&self, root: &str) -> Result<Vec<u8>, ProjectDataConverterProgramError> {
        self.encode_program_with_runtime_view_models(root, BTreeMap::new())
    }

    /// Encode a program after the authoring layer has lowered durable
    /// ViewModel identities to this file's exact runtime ordinals.
    pub fn encode_program_with_runtime_view_models(
        &self,
        root: &str,
        runtime_view_models: BTreeMap<String, u32>,
    ) -> Result<Vec<u8>, ProjectDataConverterProgramError> {
        if !self.contains(root) {
            return Err(ProjectDataConverterProgramError::UnknownRoot {
                id: root.to_owned(),
            });
        }
        let serialized = SerializedProjectDataConverterProgram {
            root: root.to_owned(),
            definitions: self
                .converters
                .iter()
                .map(|(id, spec)| ProjectDataConverterDefinition {
                    id: id.clone(),
                    spec: spec.clone(),
                })
                .collect(),
            runtime_view_models,
        };
        let payload = serde_json::to_vec(&serialized).map_err(|error| {
            ProjectDataConverterProgramError::InvalidPayload {
                reason: error.to_string(),
            }
        })?;
        let byte_count = PROJECT_DATA_CONVERTER_PROGRAM_MAGIC
            .len()
            .checked_add(payload.len())
            .ok_or(ProjectDataConverterProgramError::TooLarge { bytes: usize::MAX })?;
        if byte_count > PROJECT_DATA_CONVERTER_PROGRAM_MAX_BYTES {
            return Err(ProjectDataConverterProgramError::TooLarge { bytes: byte_count });
        }
        let mut output = Vec::with_capacity(byte_count);
        output.extend_from_slice(PROJECT_DATA_CONVERTER_PROGRAM_MAGIC);
        output.extend_from_slice(&payload);
        Ok(output)
    }

    pub fn convert(
        &self,
        id: &str,
        state: &mut ProjectDataConverterState,
        value: ProjectDataValue,
        context: &mut ProjectDataConverterContext<'_>,
    ) -> Result<ProjectDataValue, ProjectDataConverterRuntimeError> {
        self.convert_inner(id, state.root(id), value, context, &mut BTreeSet::new())
    }

    pub fn reverse_convert(
        &self,
        id: &str,
        state: &mut ProjectDataConverterState,
        value: ProjectDataValue,
        context: &mut ProjectDataConverterContext<'_>,
    ) -> Result<ProjectDataConverterReverseResult, ProjectDataConverterRuntimeError> {
        self.reverse_inner(id, state.root(id), value, context, &mut BTreeSet::new())
    }

    fn convert_inner(
        &self,
        id: &str,
        state: &mut ProjectDataConverterStateNode,
        value: ProjectDataValue,
        context: &mut ProjectDataConverterContext<'_>,
        visiting: &mut BTreeSet<String>,
    ) -> Result<ProjectDataValue, ProjectDataConverterRuntimeError> {
        let spec = self.converter(id)?;
        if !visiting.insert(id.to_owned()) {
            return Err(ProjectDataConverterRuntimeError::GroupCycle { id: id.to_owned() });
        }
        let result = self.apply_forward(id, spec, state, value, context, visiting)?;
        visiting.remove(id);
        Ok(coerce_value(
            result,
            self.effective_outputs.get(id).copied().flatten(),
        ))
    }

    fn reverse_inner(
        &self,
        id: &str,
        state: &mut ProjectDataConverterStateNode,
        value: ProjectDataValue,
        context: &mut ProjectDataConverterContext<'_>,
        visiting: &mut BTreeSet<String>,
    ) -> Result<ProjectDataConverterReverseResult, ProjectDataConverterRuntimeError> {
        let spec = self.converter(id)?;
        if !visiting.insert(id.to_owned()) {
            return Err(ProjectDataConverterRuntimeError::GroupCycle { id: id.to_owned() });
        }
        let result = self.apply_reverse(spec, state, value, context, visiting)?;
        visiting.remove(id);
        Ok(result)
    }

    fn converter(
        &self,
        id: &str,
    ) -> Result<&ProjectDataConverterSpec, ProjectDataConverterRuntimeError> {
        self.converters
            .get(id)
            .ok_or_else(|| ProjectDataConverterRuntimeError::UnknownConverter { id: id.to_owned() })
    }
}

impl ProjectDataConverterCatalog {
    fn apply_forward(
        &self,
        id: &str,
        spec: &ProjectDataConverterSpec,
        state: &mut ProjectDataConverterStateNode,
        value: ProjectDataValue,
        context: &mut ProjectDataConverterContext<'_>,
        visiting: &mut BTreeSet<String>,
    ) -> Result<ProjectDataValue, ProjectDataConverterRuntimeError> {
        let result = match &spec.kind {
            ProjectDataConverterKind::Template { template } => {
                ProjectDataValue::String(apply_template(id, template, &value)?)
            }
            ProjectDataConverterKind::Validate { rule, args, invert } => {
                let valid = apply_validation(&value, *rule, args);
                ProjectDataValue::Boolean(if *invert { !valid } else { valid })
            }
            ProjectDataConverterKind::ListCount => ProjectDataValue::Number(match value {
                ProjectDataValue::List(items) => items.len() as f64,
                _ => 0.0,
            }),
            ProjectDataConverterKind::Format {
                format,
                locale: _,
                time_zone: _,
                options: _,
                trim_zeros,
                commas,
                decimals,
            } => apply_format(&value, *format, *decimals, *trim_zeros, *commas),
            ProjectDataConverterKind::Map { cases, .. } => normalize_map_key(&value)
                .and_then(|key| cases.get(&key).cloned())
                .unwrap_or(value),
            ProjectDataConverterKind::Math {
                operation,
                value: operand,
                value_path,
            } => {
                let input = strict_number(&value).unwrap_or(0.0);
                let operand = resolve_operand(*operand, value_path.as_ref(), context);
                ProjectDataValue::Number(apply_math(input, *operation, operand))
            }
            ProjectDataConverterKind::RangeMap {
                min_input,
                max_input,
                min_output,
                max_output,
                clamp,
                reverse,
                modulo,
            } => ProjectDataValue::Number(apply_range_map(
                strict_number(&value).unwrap_or(0.0),
                *min_input,
                *max_input,
                *min_output,
                *max_output,
                *clamp,
                *reverse,
                *modulo,
            )),
            ProjectDataConverterKind::ToNumber => ProjectDataValue::Number(match value {
                ProjectDataValue::Boolean(value) => {
                    if value {
                        1.0
                    } else {
                        0.0
                    }
                }
                value => strict_number(&value).unwrap_or(0.0),
            }),
            ProjectDataConverterKind::ToString {
                locale: _,
                decimals,
                trim_zeros,
                commas,
            } => ProjectDataValue::String(apply_to_string(&value, *decimals, *trim_zeros, *commas)),
            ProjectDataConverterKind::StringTrim { mode } => match value {
                ProjectDataValue::String(value) => ProjectDataValue::String(match mode {
                    ProjectDataConverterStringTrimMode::Start => value.trim_start().to_owned(),
                    ProjectDataConverterStringTrimMode::End => value.trim_end().to_owned(),
                    ProjectDataConverterStringTrimMode::All => value.trim().to_owned(),
                }),
                value => value,
            },
            ProjectDataConverterKind::BooleanNot => {
                ProjectDataValue::Boolean(!javascript_truthy(&value))
            }
            ProjectDataConverterKind::Group { items } => {
                let mut current = value;
                for (position, item) in items.iter().enumerate() {
                    current = self.convert_inner(
                        item,
                        state.group_item(position),
                        current,
                        context,
                        visiting,
                    )?;
                }
                current
            }
            ProjectDataConverterKind::Formula { .. } => {
                let input = strict_number(&value).unwrap_or(0.0);
                let evaluated = self
                    .formulas
                    .get(id)
                    .and_then(|formula| evaluate_formula(formula, input))
                    .unwrap_or(input);
                ProjectDataValue::Number(evaluated)
            }
            ProjectDataConverterKind::Interpolate {
                duration_ms,
                easing,
            } => apply_interpolator(state, value, context.now_ms, *duration_ms, *easing),
            ProjectDataConverterKind::StringPad { length, text, side } => {
                ProjectDataValue::String(apply_string_pad(&value, *length, text, *side))
            }
            ProjectDataConverterKind::NumberToList { view_model_id } => {
                apply_number_to_list(state, value, view_model_id, context)
            }
        };
        Ok(result)
    }

    fn apply_reverse(
        &self,
        spec: &ProjectDataConverterSpec,
        state: &mut ProjectDataConverterStateNode,
        value: ProjectDataValue,
        context: &mut ProjectDataConverterContext<'_>,
        visiting: &mut BTreeSet<String>,
    ) -> Result<ProjectDataConverterReverseResult, ProjectDataConverterRuntimeError> {
        let original = value.clone();
        let converted = match &spec.kind {
            ProjectDataConverterKind::BooleanNot => {
                Some(ProjectDataValue::Boolean(!javascript_truthy(&value)))
            }
            ProjectDataConverterKind::Math {
                operation,
                value: operand,
                value_path,
            } => {
                let Some(input) = strict_number(&value) else {
                    return Ok(reverse_unsupported(original));
                };
                let operand = resolve_operand(*operand, value_path.as_ref(), context);
                match operation {
                    ProjectDataConverterMathOperation::Add => {
                        Some(ProjectDataValue::Number(input - operand.unwrap_or(0.0)))
                    }
                    ProjectDataConverterMathOperation::Subtract => {
                        Some(ProjectDataValue::Number(input + operand.unwrap_or(0.0)))
                    }
                    ProjectDataConverterMathOperation::Multiply => match operand {
                        None => Some(ProjectDataValue::Number(input)),
                        Some(0.0) => None,
                        Some(operand) => Some(ProjectDataValue::Number(input / operand)),
                    },
                    ProjectDataConverterMathOperation::Divide => match operand {
                        None => Some(ProjectDataValue::Number(input)),
                        Some(0.0) => None,
                        Some(operand) => Some(ProjectDataValue::Number(input * operand)),
                    },
                    _ => None,
                }
            }
            ProjectDataConverterKind::RangeMap {
                min_input,
                max_input,
                min_output,
                max_output,
                clamp,
                reverse,
                modulo,
            } => {
                let Some(input) = strict_number(&value) else {
                    return Ok(reverse_unsupported(original));
                };
                if min_input == max_input || min_output == max_output {
                    None
                } else {
                    Some(ProjectDataValue::Number(apply_range_map(
                        input,
                        *min_output,
                        *max_output,
                        *min_input,
                        *max_input,
                        *clamp,
                        *reverse,
                        *modulo,
                    )))
                }
            }
            ProjectDataConverterKind::Map { cases, reverse_map } => {
                let reverse = reverse_map.clone().or_else(|| build_reverse_map(cases));
                normalize_map_key(&value)
                    .and_then(|key| reverse.and_then(|map| map.get(&key).cloned()))
            }
            ProjectDataConverterKind::Group { items } => {
                let mut current = value;
                for (position, item) in items.iter().enumerate().rev() {
                    let result = self.reverse_inner(
                        item,
                        state.group_item(position),
                        current,
                        context,
                        visiting,
                    )?;
                    if !result.ok {
                        return Ok(reverse_unsupported(original));
                    }
                    current = result.value;
                }
                return Ok(ProjectDataConverterReverseResult {
                    ok: true,
                    value: current,
                });
            }
            _ => None,
        };
        Ok(match converted {
            Some(value) => ProjectDataConverterReverseResult { ok: true, value },
            None => reverse_unsupported(original),
        })
    }
}

fn reverse_unsupported(value: ProjectDataValue) -> ProjectDataConverterReverseResult {
    ProjectDataConverterReverseResult { ok: false, value }
}

fn validate_converter(
    id: &str,
    spec: &ProjectDataConverterSpec,
    converters: &BTreeMap<String, ProjectDataConverterSpec>,
) -> Result<(), ProjectDataConverterCompileError> {
    match &spec.kind {
        ProjectDataConverterKind::Group { items } => {
            for item in items {
                if !converters.contains_key(item) {
                    return Err(ProjectDataConverterCompileError::UnknownGroupItem {
                        converter: id.to_owned(),
                        item: item.clone(),
                    });
                }
            }
        }
        ProjectDataConverterKind::Math {
            value, value_path, ..
        } => {
            if value.is_some_and(|value| !value.is_finite()) {
                return Err(non_finite(id, "math value"));
            }
            if let Some(ProjectDataValuePath::Ids { path_ids, .. }) = value_path
                && path_ids.iter().any(|value| !value.is_finite())
            {
                return Err(non_finite(id, "valuePath.pathIds"));
            }
        }
        ProjectDataConverterKind::RangeMap {
            min_input,
            max_input,
            min_output,
            max_output,
            ..
        } => {
            for (field, value) in [
                ("range minInput", min_input),
                ("range maxInput", max_input),
                ("range minOutput", min_output),
                ("range maxOutput", max_output),
            ] {
                if !value.is_finite() {
                    return Err(non_finite(id, field));
                }
            }
        }
        ProjectDataConverterKind::Interpolate { duration_ms, .. } => {
            if !duration_ms.is_finite() {
                return Err(non_finite(id, "interpolate durationMs"));
            }
        }
        ProjectDataConverterKind::StringPad { length, .. } => {
            if !length.is_finite() {
                return Err(non_finite(id, "string_pad length"));
            }
        }
        ProjectDataConverterKind::Format {
            format,
            locale,
            time_zone,
            options,
            ..
        } => validate_format(id, *format, locale, time_zone, options)?,
        ProjectDataConverterKind::ToString {
            locale,
            decimals,
            commas,
            ..
        } => {
            let uses_intl = locale
                .as_ref()
                .is_some_and(|value| !value.trim().is_empty())
                || decimals.is_some()
                || *commas;
            if uses_intl && normalize_optional(locale) != Some("en-US") {
                return Err(ProjectDataConverterCompileError::UnsupportedFormat {
                    converter: id.to_owned(),
                    format: ProjectDataConverterFormat::Number,
                    reason: "to_string Intl settings require locale en-US".to_owned(),
                });
            }
        }
        _ => {}
    }
    Ok(())
}

fn non_finite(id: &str, field: &'static str) -> ProjectDataConverterCompileError {
    ProjectDataConverterCompileError::NonFinite {
        converter: id.to_owned(),
        field,
    }
}

fn validate_format(
    id: &str,
    format: ProjectDataConverterFormat,
    locale: &Option<String>,
    time_zone: &Option<String>,
    options: &BTreeMap<String, ProjectDataValue>,
) -> Result<(), ProjectDataConverterCompileError> {
    let unsupported = |reason: &str| ProjectDataConverterCompileError::UnsupportedFormat {
        converter: id.to_owned(),
        format,
        reason: reason.to_owned(),
    };
    match format {
        ProjectDataConverterFormat::Number => {
            if normalize_optional(locale) != Some("en-US") {
                return Err(unsupported("number formatting requires locale en-US"));
            }
            if normalize_optional(time_zone).is_some() {
                return Err(unsupported("number formatting does not accept timeZone"));
            }
            if !options.is_empty() {
                return Err(unsupported(
                    "number Intl options outside decimals/commas are not characterized",
                ));
            }
        }
        ProjectDataConverterFormat::Currency => {
            if normalize_optional(locale) != Some("en-US") {
                return Err(unsupported("currency formatting requires locale en-US"));
            }
            if normalize_optional(time_zone).is_some() {
                return Err(unsupported("currency formatting does not accept timeZone"));
            }
            let expected = BTreeMap::from([(
                "currency".to_owned(),
                ProjectDataValue::String("USD".to_owned()),
            )]);
            if options != &expected {
                return Err(unsupported(
                    "only the characterized options { currency: \"USD\" } are supported",
                ));
            }
        }
        ProjectDataConverterFormat::Date => {
            if normalize_optional(locale) != Some("en-GB") {
                return Err(unsupported("date formatting requires locale en-GB"));
            }
            if normalize_optional(time_zone) != Some("UTC") {
                return Err(unsupported("date formatting requires timeZone UTC"));
            }
            let expected = BTreeMap::from([
                (
                    "day".to_owned(),
                    ProjectDataValue::String("2-digit".to_owned()),
                ),
                (
                    "month".to_owned(),
                    ProjectDataValue::String("2-digit".to_owned()),
                ),
                (
                    "year".to_owned(),
                    ProjectDataValue::String("numeric".to_owned()),
                ),
            ]);
            if options != &expected {
                return Err(unsupported(
                    "only numeric year and two-digit month/day are characterized",
                ));
            }
        }
    }
    Ok(())
}

fn normalize_optional(value: &Option<String>) -> Option<&str> {
    value
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

fn validate_group_cycles(
    converters: &BTreeMap<String, ProjectDataConverterSpec>,
) -> Result<(), ProjectDataConverterCompileError> {
    fn visit(
        id: &str,
        nesting: usize,
        converters: &BTreeMap<String, ProjectDataConverterSpec>,
        visiting: &mut BTreeSet<String>,
        heights: &mut BTreeMap<String, usize>,
    ) -> Result<usize, ProjectDataConverterCompileError> {
        if let Some(height) = heights.get(id).copied() {
            if nesting.saturating_add(height) > PROJECT_DATA_CONVERTER_MAX_GROUP_NESTING {
                return Err(ProjectDataConverterCompileError::GroupNestingTooDeep {
                    converter: id.to_owned(),
                    maximum: PROJECT_DATA_CONVERTER_MAX_GROUP_NESTING,
                });
            }
            return Ok(height);
        }
        if !visiting.insert(id.to_owned()) {
            return Err(ProjectDataConverterCompileError::GroupCycle {
                converter: id.to_owned(),
            });
        }
        let height = if let Some(ProjectDataConverterSpec {
            kind: ProjectDataConverterKind::Group { items },
            ..
        }) = converters.get(id)
        {
            if !items.is_empty() && nesting >= PROJECT_DATA_CONVERTER_MAX_GROUP_NESTING {
                return Err(ProjectDataConverterCompileError::GroupNestingTooDeep {
                    converter: id.to_owned(),
                    maximum: PROJECT_DATA_CONVERTER_MAX_GROUP_NESTING,
                });
            }
            let mut height = 0usize;
            for item in items {
                let child_height = visit(
                    item,
                    nesting.saturating_add(1),
                    converters,
                    visiting,
                    heights,
                )?;
                height = height.max(child_height.saturating_add(1));
            }
            height
        } else {
            0
        };
        visiting.remove(id);
        heights.insert(id.to_owned(), height);
        Ok(height)
    }

    let mut heights = BTreeMap::new();
    for id in converters.keys() {
        visit(id, 0, converters, &mut BTreeSet::new(), &mut heights)?;
    }
    Ok(())
}

fn effective_output_for(
    id: &str,
    converters: &BTreeMap<String, ProjectDataConverterSpec>,
    known: &mut BTreeMap<String, Option<ProjectDataConverterOutputType>>,
) -> Option<ProjectDataConverterOutputType> {
    if let Some(output) = known.get(id) {
        return *output;
    }
    let spec = converters.get(id)?;
    if let Some(output) = spec.output_type {
        return Some(output);
    }
    match &spec.kind {
        ProjectDataConverterKind::Template { .. }
        | ProjectDataConverterKind::Format { .. }
        | ProjectDataConverterKind::ToString { .. }
        | ProjectDataConverterKind::StringTrim { .. }
        | ProjectDataConverterKind::StringPad { .. } => {
            Some(ProjectDataConverterOutputType::String)
        }
        ProjectDataConverterKind::Validate { .. } | ProjectDataConverterKind::BooleanNot => {
            Some(ProjectDataConverterOutputType::Boolean)
        }
        ProjectDataConverterKind::ListCount
        | ProjectDataConverterKind::Math { .. }
        | ProjectDataConverterKind::RangeMap { .. }
        | ProjectDataConverterKind::ToNumber
        | ProjectDataConverterKind::Formula { .. } => Some(ProjectDataConverterOutputType::Number),
        ProjectDataConverterKind::NumberToList { .. } => Some(ProjectDataConverterOutputType::List),
        ProjectDataConverterKind::Map { .. } | ProjectDataConverterKind::Interpolate { .. } => None,
        ProjectDataConverterKind::Group { items } => items
            .iter()
            .rev()
            .find_map(|item| effective_output_for(item, converters, known)),
    }
}

fn coerce_value(
    value: ProjectDataValue,
    output: Option<ProjectDataConverterOutputType>,
) -> ProjectDataValue {
    match output {
        None
        | Some(
            ProjectDataConverterOutputType::Color
            | ProjectDataConverterOutputType::Enum
            | ProjectDataConverterOutputType::ListIndex
            | ProjectDataConverterOutputType::Image
            | ProjectDataConverterOutputType::Trigger
            | ProjectDataConverterOutputType::ViewModel,
        ) => value,
        Some(ProjectDataConverterOutputType::String) => ProjectDataValue::String(match &value {
            ProjectDataValue::Null => String::new(),
            value => javascript_string(value),
        }),
        Some(ProjectDataConverterOutputType::Number) => {
            ProjectDataValue::Number(strict_number(&value).unwrap_or(0.0))
        }
        Some(ProjectDataConverterOutputType::Boolean) => {
            ProjectDataValue::Boolean(javascript_truthy(&value))
        }
        Some(ProjectDataConverterOutputType::List) => match value {
            value @ ProjectDataValue::List(_) => value,
            _ => ProjectDataValue::List(Vec::new()),
        },
        Some(ProjectDataConverterOutputType::Object) => match value {
            value @ ProjectDataValue::Object(_) => value,
            _ => ProjectDataValue::Object(BTreeMap::new()),
        },
    }
}

fn strict_number(value: &ProjectDataValue) -> Option<f64> {
    match value {
        ProjectDataValue::Number(value) if value.is_finite() => Some(*value),
        ProjectDataValue::String(value) => parse_javascript_number(value),
        ProjectDataValue::Color(value) => Some(f64::from(*value)),
        ProjectDataValue::Enum(value)
        | ProjectDataValue::ListIndex(value)
        | ProjectDataValue::Trigger(value) => Some(*value as f64),
        _ => None,
    }
}

fn parse_javascript_number(value: &str) -> Option<f64> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }
    let parsed = if let Some(hex) = value
        .strip_prefix("0x")
        .or_else(|| value.strip_prefix("0X"))
    {
        u64::from_str_radix(hex, 16).ok().map(|value| value as f64)
    } else if let Some(binary) = value
        .strip_prefix("0b")
        .or_else(|| value.strip_prefix("0B"))
    {
        u64::from_str_radix(binary, 2)
            .ok()
            .map(|value| value as f64)
    } else if let Some(octal) = value
        .strip_prefix("0o")
        .or_else(|| value.strip_prefix("0O"))
    {
        u64::from_str_radix(octal, 8).ok().map(|value| value as f64)
    } else {
        value.parse::<f64>().ok()
    };
    parsed.filter(|value| value.is_finite())
}

fn javascript_truthy(value: &ProjectDataValue) -> bool {
    match value {
        ProjectDataValue::Null => false,
        ProjectDataValue::Boolean(value) => *value,
        ProjectDataValue::Number(value) => *value != 0.0 && !value.is_nan(),
        ProjectDataValue::String(value) => !value.is_empty(),
        ProjectDataValue::Color(value) => *value != 0,
        ProjectDataValue::Enum(value)
        | ProjectDataValue::ListIndex(value)
        | ProjectDataValue::Trigger(value) => *value != 0,
        ProjectDataValue::List(_)
        | ProjectDataValue::Object(_)
        | ProjectDataValue::Image(_)
        | ProjectDataValue::ViewModel(_) => true,
    }
}

fn javascript_string(value: &ProjectDataValue) -> String {
    match value {
        ProjectDataValue::Null => "null".to_owned(),
        ProjectDataValue::Boolean(value) => value.to_string(),
        ProjectDataValue::Number(value) => javascript_number_string(*value),
        ProjectDataValue::String(value) => value.clone(),
        ProjectDataValue::Color(value) => value.to_string(),
        ProjectDataValue::Enum(value)
        | ProjectDataValue::ListIndex(value)
        | ProjectDataValue::Trigger(value) => value.to_string(),
        ProjectDataValue::List(values) => values
            .iter()
            .map(|value| match value {
                ProjectDataValue::Null => String::new(),
                value => javascript_string(value),
            })
            .collect::<Vec<_>>()
            .join(","),
        ProjectDataValue::Object(_)
        | ProjectDataValue::Image(_)
        | ProjectDataValue::ViewModel(_) => "[object Object]".to_owned(),
    }
}

fn javascript_number_string(value: f64) -> String {
    if value == 0.0 {
        return "0".to_owned();
    }
    if value.is_nan() {
        return "NaN".to_owned();
    }
    if value == f64::INFINITY {
        return "Infinity".to_owned();
    }
    if value == f64::NEG_INFINITY {
        return "-Infinity".to_owned();
    }
    let mut rendered = value.to_string();
    if let Some(exponent) = rendered.find('e') {
        let sign_index = exponent.saturating_add(1);
        if rendered.as_bytes().get(sign_index) != Some(&b'-')
            && rendered.as_bytes().get(sign_index) != Some(&b'+')
        {
            rendered.insert(sign_index, '+');
        }
    }
    rendered
}

fn apply_template(
    converter: &str,
    template: &str,
    value: &ProjectDataValue,
) -> Result<String, ProjectDataConverterRuntimeError> {
    let mut replacement = String::new();
    if !matches!(value, ProjectDataValue::Null) {
        append_bounded_javascript_string(converter, value, &mut replacement, 0)?;
    }
    let mut rendered = String::new();
    let mut rest = template;
    while let Some(open) = rest.find("{{") {
        push_bounded_template_part(converter, &mut rendered, &rest[..open])?;
        let after_open = &rest[open.saturating_add(2)..];
        let Some(close) = after_open.find("}}") else {
            push_bounded_template_part(converter, &mut rendered, &rest[open..])?;
            return Ok(rendered);
        };
        let token = &after_open[..close];
        if token.trim() == "value" {
            push_bounded_template_part(converter, &mut rendered, &replacement)?;
        } else {
            push_bounded_template_part(converter, &mut rendered, "{{")?;
            push_bounded_template_part(converter, &mut rendered, token)?;
            push_bounded_template_part(converter, &mut rendered, "}}")?;
        }
        rest = &after_open[close.saturating_add(2)..];
    }
    push_bounded_template_part(converter, &mut rendered, rest)?;
    Ok(rendered)
}

fn push_bounded_template_part(
    converter: &str,
    rendered: &mut String,
    part: &str,
) -> Result<(), ProjectDataConverterRuntimeError> {
    if rendered
        .len()
        .checked_add(part.len())
        .is_none_or(|length| length > PROJECT_DATA_CONVERTER_MAX_VALUE_BYTES)
    {
        return Err(value_too_large(converter));
    }
    rendered.push_str(part);
    Ok(())
}

fn append_bounded_javascript_string(
    converter: &str,
    value: &ProjectDataValue,
    rendered: &mut String,
    nesting: usize,
) -> Result<(), ProjectDataConverterRuntimeError> {
    match value {
        ProjectDataValue::Null => push_bounded_template_part(converter, rendered, "null"),
        ProjectDataValue::Boolean(value) => {
            push_bounded_template_part(converter, rendered, if *value { "true" } else { "false" })
        }
        ProjectDataValue::Number(value) => push_bounded_template_part(
            converter,
            rendered,
            javascript_number_string(*value).as_str(),
        ),
        ProjectDataValue::String(value) => push_bounded_template_part(converter, rendered, value),
        ProjectDataValue::Color(value) => {
            push_bounded_template_part(converter, rendered, value.to_string().as_str())
        }
        ProjectDataValue::Enum(value)
        | ProjectDataValue::ListIndex(value)
        | ProjectDataValue::Trigger(value) => {
            push_bounded_template_part(converter, rendered, value.to_string().as_str())
        }
        ProjectDataValue::List(values) => {
            if nesting >= PROJECT_DATA_CONVERTER_MAX_VALUE_NESTING {
                return Err(ProjectDataConverterRuntimeError::ValueTooComplex {
                    converter: converter.to_owned(),
                });
            }
            for (index, value) in values.iter().enumerate() {
                if index != 0 {
                    push_bounded_template_part(converter, rendered, ",")?;
                }
                if !matches!(value, ProjectDataValue::Null) {
                    append_bounded_javascript_string(converter, value, rendered, nesting + 1)?;
                }
            }
            Ok(())
        }
        ProjectDataValue::Object(_)
        | ProjectDataValue::Image(_)
        | ProjectDataValue::ViewModel(_) => {
            push_bounded_template_part(converter, rendered, "[object Object]")
        }
    }
}

fn value_too_large(converter: &str) -> ProjectDataConverterRuntimeError {
    ProjectDataConverterRuntimeError::ValueTooLarge {
        converter: converter.to_owned(),
        maximum_bytes: PROJECT_DATA_CONVERTER_MAX_VALUE_BYTES,
    }
}

fn collection_length(value: &ProjectDataValue) -> Option<f64> {
    match value {
        ProjectDataValue::String(value) => Some(value.encode_utf16().count() as f64),
        ProjectDataValue::List(value) => Some(value.len() as f64),
        ProjectDataValue::Object(value) => Some(value.len() as f64),
        value => strict_number(value),
    }
}

fn apply_validation(
    value: &ProjectDataValue,
    rule: ProjectDataConverterValidationRule,
    args: &BTreeMap<String, ProjectDataValue>,
) -> bool {
    match rule {
        ProjectDataConverterValidationRule::NonEmpty => match value {
            ProjectDataValue::Null => false,
            ProjectDataValue::String(value) => !value.is_empty(),
            ProjectDataValue::List(value) => !value.is_empty(),
            ProjectDataValue::Object(value) => !value.is_empty(),
            _ => true,
        },
        ProjectDataConverterValidationRule::Min => {
            let bound = ["min", "value", "length"]
                .into_iter()
                .find_map(|key| args.get(key).and_then(strict_number));
            bound
                .zip(collection_length(value))
                .is_some_and(|(bound, length)| length >= bound)
        }
        ProjectDataConverterValidationRule::Max => {
            let bound = ["max", "value", "length"]
                .into_iter()
                .find_map(|key| args.get(key).and_then(strict_number));
            bound
                .zip(collection_length(value))
                .is_some_and(|(bound, length)| length <= bound)
        }
        ProjectDataConverterValidationRule::Range => {
            let min = args.get("min").and_then(strict_number);
            let max = args.get("max").and_then(strict_number);
            min.zip(max)
                .zip(collection_length(value))
                .is_some_and(|((min, max), length)| length >= min && length <= max)
        }
    }
}

fn resolve_operand(
    constant: Option<f64>,
    path: Option<&ProjectDataValuePath>,
    context: &mut ProjectDataConverterContext<'_>,
) -> Option<f64> {
    if let Some(path) = path
        && let Some(resolver) = context.resolver.as_deref_mut()
        && let Some(value) = resolver.resolve_value(path)
        && let Some(value) = strict_number(&value)
    {
        return Some(value);
    }
    constant.filter(|value| value.is_finite())
}

fn javascript_round(value: f64) -> f64 {
    if !value.is_finite() || value.fract() == 0.0 {
        return value;
    }
    let rounded = (value + 0.5).floor();
    if rounded == 0.0 && value < 0.0 {
        -0.0
    } else {
        rounded
    }
}

fn apply_math(
    input: f64,
    operation: ProjectDataConverterMathOperation,
    operand: Option<f64>,
) -> f64 {
    match operation {
        ProjectDataConverterMathOperation::Add => input + operand.unwrap_or(0.0),
        ProjectDataConverterMathOperation::Subtract => input - operand.unwrap_or(0.0),
        ProjectDataConverterMathOperation::Multiply => {
            operand.map_or(input, |operand| input * operand)
        }
        ProjectDataConverterMathOperation::Divide => {
            operand.map_or(input, |operand| input / operand)
        }
        ProjectDataConverterMathOperation::Modulo => {
            operand.map_or(input, |operand| ((input % operand) + operand) % operand)
        }
        ProjectDataConverterMathOperation::Round => javascript_round(input),
        ProjectDataConverterMathOperation::Floor => input.floor(),
        ProjectDataConverterMathOperation::Ceil => input.ceil(),
        ProjectDataConverterMathOperation::SquareRoot => input.sqrt(),
        ProjectDataConverterMathOperation::Power => {
            operand.map_or(input, |operand| input.powf(operand))
        }
        ProjectDataConverterMathOperation::Exponential => input.exp(),
        ProjectDataConverterMathOperation::NaturalLog => input.ln(),
        ProjectDataConverterMathOperation::Cosine => input.cos(),
        ProjectDataConverterMathOperation::Sine => input.sin(),
        ProjectDataConverterMathOperation::Tangent => input.tan(),
        ProjectDataConverterMathOperation::ArcCosine => input.acos(),
        ProjectDataConverterMathOperation::ArcSine => input.asin(),
        ProjectDataConverterMathOperation::ArcTangent => input.atan(),
        ProjectDataConverterMathOperation::ArcTangent2 => {
            operand.map_or(input, |operand| input.atan2(operand))
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn apply_range_map(
    input: f64,
    min_input: f64,
    max_input: f64,
    min_output: f64,
    max_output: f64,
    clamp: ProjectDataConverterRangeClamp,
    reverse: bool,
    modulo: bool,
) -> f64 {
    if max_input == min_input {
        return min_output;
    }
    let mut value = input;
    if modulo {
        let span = max_input - min_input;
        value = ((((value - min_input) % span) + span) % span) + min_input;
    }
    if matches!(
        clamp,
        ProjectDataConverterRangeClamp::Lower | ProjectDataConverterRangeClamp::Both
    ) && value < min_input
    {
        value = min_input;
    }
    if matches!(
        clamp,
        ProjectDataConverterRangeClamp::Upper | ProjectDataConverterRangeClamp::Both
    ) && value > max_input
    {
        value = max_input;
    }
    let mut amount = (value - min_input) / (max_input - min_input);
    if reverse {
        amount = 1.0 - amount;
    }
    amount * max_output + (1.0 - amount) * min_output
}

fn normalize_map_key(value: &ProjectDataValue) -> Option<String> {
    match value {
        ProjectDataValue::String(value) => Some(value.clone()),
        ProjectDataValue::Number(value) => Some(javascript_number_string(*value)),
        ProjectDataValue::Boolean(value) => Some(value.to_string()),
        ProjectDataValue::Color(value) => Some(value.to_string()),
        ProjectDataValue::Enum(value)
        | ProjectDataValue::ListIndex(value)
        | ProjectDataValue::Trigger(value) => Some(value.to_string()),
        _ => None,
    }
}

fn build_reverse_map(
    cases: &BTreeMap<String, ProjectDataValue>,
) -> Option<BTreeMap<String, ProjectDataValue>> {
    let mut reverse = BTreeMap::new();
    for (key, value) in cases {
        let map_key = normalize_map_key(value)?;
        if reverse.contains_key(&map_key) {
            return None;
        }
        reverse.insert(map_key, ProjectDataValue::String(key.clone()));
    }
    Some(reverse)
}

fn apply_format(
    value: &ProjectDataValue,
    format: ProjectDataConverterFormat,
    decimals: Option<u32>,
    trim_zeros: bool,
    commas: bool,
) -> ProjectDataValue {
    match format {
        ProjectDataConverterFormat::Number => strict_number(value)
            .map(|value| {
                ProjectDataValue::String(format_en_us_number(value, decimals, trim_zeros, commas))
            })
            .unwrap_or_else(|| value.clone()),
        ProjectDataConverterFormat::Currency => strict_number(value)
            .map(|value| {
                let rendered = format_en_us_number(value.abs(), Some(2), false, true);
                ProjectDataValue::String(if value.is_sign_negative() {
                    format!("-${rendered}")
                } else {
                    format!("${rendered}")
                })
            })
            .unwrap_or_else(|| value.clone()),
        ProjectDataConverterFormat::Date => strict_number(value)
            .and_then(format_en_gb_utc_date)
            .map(ProjectDataValue::String)
            .unwrap_or_else(|| value.clone()),
    }
}

fn format_en_us_number(
    value: f64,
    decimals: Option<u32>,
    trim_zeros: bool,
    commas: bool,
) -> String {
    if decimals.is_some_and(|decimals| decimals > PROJECT_DATA_CONVERTER_MAX_DECIMALS) {
        let mut rendered = javascript_number_string(value);
        if trim_zeros {
            rendered = trim_trailing_zeros(rendered);
        }
        return rendered;
    }
    let precision = decimals.and_then(|value| usize::try_from(value).ok());
    let mut rendered = match precision {
        Some(precision) => {
            let factor = 10.0_f64.powi(i32::try_from(precision).unwrap_or(i32::MAX));
            let scaled = value * factor;
            let rounded = if scaled.is_sign_negative() {
                (scaled - 0.5).ceil()
            } else {
                (scaled + 0.5).floor()
            } / factor;
            format!("{rounded:.precision$}")
        }
        None => javascript_number_string(value),
    };
    if trim_zeros {
        rendered = trim_trailing_zeros(rendered);
    }
    if commas {
        rendered = add_ascii_grouping(&rendered);
    }
    rendered
}

fn trim_trailing_zeros(mut value: String) -> String {
    if let Some(decimal) = value.find('.') {
        while value.ends_with('0') {
            value.pop();
        }
        if value.len() == decimal.saturating_add(1) {
            value.pop();
        }
    }
    value
}

fn add_ascii_grouping(value: &str) -> String {
    let (sign, unsigned) = value
        .strip_prefix('-')
        .map_or(("", value), |value| ("-", value));
    let (integer, fraction) = unsigned
        .split_once('.')
        .map_or((unsigned, None), |(integer, fraction)| {
            (integer, Some(fraction))
        });
    if !integer.bytes().all(|byte| byte.is_ascii_digit()) {
        return value.to_owned();
    }
    let mut grouped = String::with_capacity(value.len().saturating_add(integer.len() / 3));
    grouped.push_str(sign);
    for (index, character) in integer.chars().enumerate() {
        let remaining = integer.len().saturating_sub(index);
        if index > 0 && remaining % 3 == 0 {
            grouped.push(',');
        }
        grouped.push(character);
    }
    if let Some(fraction) = fraction {
        grouped.push('.');
        grouped.push_str(fraction);
    }
    grouped
}

fn format_en_gb_utc_date(milliseconds: f64) -> Option<String> {
    if !milliseconds.is_finite() || milliseconds.abs() > 8_640_000_000_000_000.0 {
        return None;
    }
    let days = (milliseconds.trunc() / 86_400_000.0).floor();
    if days < i64::MIN as f64 || days > i64::MAX as f64 {
        return None;
    }
    let (year, month, day) = civil_from_unix_days(days as i64)?;
    Some(format!("{day:02}/{month:02}/{year:04}"))
}

// Howard Hinnant's civil-from-days algorithm, with day zero at 1970-01-01.
fn civil_from_unix_days(days: i64) -> Option<(i64, u32, u32)> {
    let shifted = days.checked_add(719_468)?;
    let era = if shifted >= 0 {
        shifted
    } else {
        shifted.checked_sub(146_096)?
    } / 146_097;
    let day_of_era = shifted.checked_sub(era.checked_mul(146_097)?)?;
    let year_of_era = (day_of_era
        .checked_sub(day_of_era / 1_460)?
        .checked_add(day_of_era / 36_524)?
        .checked_sub(day_of_era / 146_096)?)
        / 365;
    let mut year = year_of_era.checked_add(era.checked_mul(400)?)?;
    let day_of_year = day_of_era.checked_sub(
        365_i64
            .checked_mul(year_of_era)?
            .checked_add(year_of_era / 4)?
            .checked_sub(year_of_era / 100)?,
    )?;
    let month_prime = 5_i64.checked_mul(day_of_year)?.checked_add(2)? / 153;
    let day = day_of_year
        .checked_sub(153_i64.checked_mul(month_prime)?.checked_add(2)? / 5)?
        .checked_add(1)?;
    let month = month_prime.checked_add(if month_prime < 10 { 3 } else { -9 })?;
    if month <= 2 {
        year = year.checked_add(1)?;
    }
    Some((year, u32::try_from(month).ok()?, u32::try_from(day).ok()?))
}

fn apply_to_string(
    value: &ProjectDataValue,
    decimals: Option<u32>,
    trim_zeros: bool,
    commas: bool,
) -> String {
    match value {
        ProjectDataValue::Null => String::new(),
        ProjectDataValue::Number(value) if value.is_finite() => {
            format_en_us_number(*value, decimals, trim_zeros, commas)
        }
        ProjectDataValue::Color(value) => {
            format_en_us_number(f64::from(*value), decimals, trim_zeros, commas)
        }
        ProjectDataValue::Enum(value)
        | ProjectDataValue::ListIndex(value)
        | ProjectDataValue::Trigger(value) => {
            format_en_us_number(*value as f64, decimals, trim_zeros, commas)
        }
        ProjectDataValue::Boolean(value) => {
            if *value {
                "1".to_owned()
            } else {
                "0".to_owned()
            }
        }
        value => javascript_string(value),
    }
}

fn apply_string_pad(
    value: &ProjectDataValue,
    length: f64,
    text: &str,
    side: ProjectDataConverterStringPadSide,
) -> String {
    let input = match value {
        ProjectDataValue::Null => String::new(),
        value => javascript_string(value),
    };
    let target =
        normalized_bounded_length(length, PROJECT_DATA_CONVERTER_MAX_STRING_PAD_UTF16_UNITS);
    let input_len = input.encode_utf16().count();
    if text.is_empty() || input_len >= target {
        return input;
    }
    let needed = target.saturating_sub(input_len);
    let units = text.encode_utf16().collect::<Vec<_>>();
    if units.is_empty() {
        return input;
    }
    let filler = units
        .iter()
        .copied()
        .cycle()
        .take(needed)
        .collect::<Vec<_>>();
    let filler = String::from_utf16_lossy(&filler);
    match side {
        ProjectDataConverterStringPadSide::Start => filler + &input,
        ProjectDataConverterStringPadSide::End => input + &filler,
    }
}

fn apply_interpolator(
    state: &mut ProjectDataConverterStateNode,
    value: ProjectDataValue,
    now_ms: Option<f64>,
    duration_ms: f64,
    easing: ProjectDataConverterEasing,
) -> ProjectDataValue {
    let Some(numeric) = strict_number(&value) else {
        state.cache = None;
        return value;
    };
    let Some(now_ms) = now_ms.filter(|value| value.is_finite()) else {
        state.cache = None;
        return value;
    };
    let duration_ms = duration_ms.max(0.0);
    if duration_ms == 0.0 {
        state.cache = None;
        return ProjectDataValue::Number(numeric);
    }
    if !matches!(
        state.cache,
        Some(ProjectDataConverterCache::Interpolate { .. })
    ) {
        state.cache = Some(ProjectDataConverterCache::Interpolate {
            from: numeric,
            to: numeric,
            start_ms: now_ms,
            duration_ms,
            last_value: numeric,
        });
        return ProjectDataValue::Number(numeric);
    }
    let Some(ProjectDataConverterCache::Interpolate {
        from,
        to,
        start_ms,
        duration_ms: cached_duration,
        last_value,
    }) = state.cache.as_mut()
    else {
        return ProjectDataValue::Number(numeric);
    };
    if *to != numeric {
        *from = *last_value;
        *to = numeric;
        *start_ms = now_ms;
        *cached_duration = duration_ms;
    }
    let elapsed = now_ms - *start_ms;
    let amount = (elapsed / *cached_duration).clamp(0.0, 1.0);
    if amount >= 1.0 {
        *last_value = *to;
        return ProjectDataValue::Number(*to);
    }
    let eased = match easing {
        ProjectDataConverterEasing::Linear => amount,
        ProjectDataConverterEasing::EaseIn => amount * amount,
        ProjectDataConverterEasing::EaseOut => amount * (2.0 - amount),
        ProjectDataConverterEasing::EaseInOut if amount < 0.5 => 2.0 * amount * amount,
        ProjectDataConverterEasing::EaseInOut => -1.0 + (4.0 - 2.0 * amount) * amount,
    };
    let output = *from + (*to - *from) * eased;
    *last_value = output;
    ProjectDataValue::Number(output)
}

fn apply_number_to_list(
    state: &mut ProjectDataConverterStateNode,
    value: ProjectDataValue,
    view_model_id: &str,
    context: &mut ProjectDataConverterContext<'_>,
) -> ProjectDataValue {
    if matches!(value, ProjectDataValue::List(_)) {
        return value;
    }
    let Some(resolver) = context.resolver.as_deref_mut() else {
        return ProjectDataValue::List(Vec::new());
    };
    let count = project_data_converter_bounded_list_length(strict_number(&value).unwrap_or(0.0));
    if !matches!(
        &state.cache,
        Some(ProjectDataConverterCache::NumberToList {
            view_model_id: cached,
            ..
        }) if cached == view_model_id
    ) {
        state.cache = Some(ProjectDataConverterCache::NumberToList {
            view_model_id: view_model_id.to_owned(),
            items: Vec::new(),
        });
    }
    let Some(ProjectDataConverterCache::NumberToList { items, .. }) = state.cache.as_mut() else {
        return ProjectDataValue::List(Vec::new());
    };
    while items.len() < count {
        let Some(item) = resolver.create_blank_view_model_instance(view_model_id) else {
            items.clear();
            return ProjectDataValue::List(Vec::new());
        };
        items.push(item);
    }
    items.truncate(count);
    ProjectDataValue::List(items.clone())
}

fn normalized_bounded_length(value: f64, maximum: usize) -> usize {
    if !value.is_finite() {
        return 0;
    }
    value.floor().max(0.0).min(maximum as f64) as usize
}

pub(crate) fn project_data_converter_bounded_list_length(value: f64) -> usize {
    normalized_bounded_length(value, PROJECT_DATA_CONVERTER_MAX_LIST_ITEMS)
}

#[derive(Debug, Clone)]
enum ProjectFormula {
    Input,
    Value(f64),
    Negate(Box<Self>),
    Binary {
        left: Box<Self>,
        operation: FormulaOperation,
        right: Box<Self>,
    },
    Function {
        function: FormulaFunction,
        arguments: Vec<Self>,
    },
}

#[derive(Debug, Clone, Copy)]
enum FormulaOperation {
    Add,
    Subtract,
    Multiply,
    Divide,
    Remainder,
}

#[derive(Debug, Clone, Copy)]
enum FormulaFunction {
    Min,
    Max,
    Abs,
    Round,
    Floor,
    Ceil,
    SquareRoot,
    Power,
    Exponential,
    NaturalLog,
    Sine,
    Cosine,
    Tangent,
}

fn evaluate_formula(formula: &ProjectFormula, input: f64) -> Option<f64> {
    fn evaluate(formula: &ProjectFormula, input: f64) -> Option<f64> {
        match formula {
            ProjectFormula::Input => Some(input),
            ProjectFormula::Value(value) => Some(*value),
            ProjectFormula::Negate(value) => Some(-evaluate(value, input)?),
            ProjectFormula::Binary {
                left,
                operation,
                right,
            } => {
                let left = evaluate(left, input)?;
                let right = evaluate(right, input)?;
                Some(match operation {
                    FormulaOperation::Add => left + right,
                    FormulaOperation::Subtract => left - right,
                    FormulaOperation::Multiply => left * right,
                    FormulaOperation::Divide => left / right,
                    FormulaOperation::Remainder => left % right,
                })
            }
            ProjectFormula::Function {
                function,
                arguments,
            } => {
                let arguments = arguments
                    .iter()
                    .map(|argument| evaluate(argument, input))
                    .collect::<Option<Vec<_>>>()?;
                match function {
                    FormulaFunction::Min if arguments.iter().any(|value| value.is_nan()) => {
                        Some(f64::NAN)
                    }
                    FormulaFunction::Min => arguments.into_iter().reduce(f64::min),
                    FormulaFunction::Max if arguments.iter().any(|value| value.is_nan()) => {
                        Some(f64::NAN)
                    }
                    FormulaFunction::Max => arguments.into_iter().reduce(f64::max),
                    FormulaFunction::Abs => arguments.first().map(|value| value.abs()),
                    FormulaFunction::Round => {
                        arguments.first().map(|value| javascript_round(*value))
                    }
                    FormulaFunction::Floor => arguments.first().map(|value| value.floor()),
                    FormulaFunction::Ceil => arguments.first().map(|value| value.ceil()),
                    FormulaFunction::SquareRoot => arguments.first().map(|value| value.sqrt()),
                    FormulaFunction::Power => arguments
                        .first()
                        .zip(arguments.get(1))
                        .map(|(base, exponent)| base.powf(*exponent)),
                    FormulaFunction::Exponential => arguments.first().map(|value| value.exp()),
                    FormulaFunction::NaturalLog => arguments.first().map(|value| value.ln()),
                    FormulaFunction::Sine => arguments.first().map(|value| value.sin()),
                    FormulaFunction::Cosine => arguments.first().map(|value| value.cos()),
                    FormulaFunction::Tangent => arguments.first().map(|value| value.tan()),
                }
            }
        }
    }
    evaluate(formula, input).filter(|value| value.is_finite())
}

#[derive(Debug, Clone, Copy)]
struct FormulaParseError {
    offset: usize,
    message: &'static str,
}

struct FormulaParser<'a> {
    source: &'a str,
    offset: usize,
    nesting: usize,
    nodes: usize,
}

impl<'a> FormulaParser<'a> {
    fn parse(source: &'a str) -> Result<ProjectFormula, FormulaParseError> {
        let mut parser = Self {
            source,
            offset: 0,
            nesting: 0,
            nodes: 0,
        };
        let formula = parser.parse_expression()?;
        parser.skip_whitespace();
        if parser.offset != source.len() {
            return Err(parser.error("unexpected formula token"));
        }
        Ok(formula)
    }

    fn error(&self, message: &'static str) -> FormulaParseError {
        FormulaParseError {
            offset: self.offset,
            message,
        }
    }

    fn remaining(&self) -> &'a str {
        self.source.get(self.offset..).unwrap_or_default()
    }

    fn peek(&self) -> Option<char> {
        self.remaining().chars().next()
    }

    fn bump(&mut self) -> Option<char> {
        let character = self.peek()?;
        self.offset = self.offset.saturating_add(character.len_utf8());
        Some(character)
    }

    fn skip_whitespace(&mut self) {
        while self.peek().is_some_and(char::is_whitespace) {
            self.bump();
        }
    }

    fn consume(&mut self, expected: char) -> bool {
        self.skip_whitespace();
        if self.peek() != Some(expected) {
            return false;
        }
        self.bump();
        true
    }

    fn enter_nesting(&mut self) -> Result<(), FormulaParseError> {
        if self.nesting >= PROJECT_DATA_CONVERTER_MAX_FORMULA_NESTING {
            return Err(self.error("formula nesting limit exceeded"));
        }
        self.nesting += 1;
        Ok(())
    }

    fn node(&mut self, formula: ProjectFormula) -> Result<ProjectFormula, FormulaParseError> {
        self.nodes = self.nodes.saturating_add(1);
        if self.nodes > PROJECT_DATA_CONVERTER_MAX_FORMULA_NODES {
            return Err(self.error("formula node limit exceeded"));
        }
        Ok(formula)
    }

    fn parse_nested_expression(&mut self) -> Result<ProjectFormula, FormulaParseError> {
        self.enter_nesting()?;
        let result = self.parse_expression();
        self.nesting -= 1;
        result
    }

    fn parse_nested_factor(&mut self) -> Result<ProjectFormula, FormulaParseError> {
        self.enter_nesting()?;
        let result = self.parse_factor();
        self.nesting -= 1;
        result
    }

    fn parse_expression(&mut self) -> Result<ProjectFormula, FormulaParseError> {
        let mut expression = self.parse_term()?;
        loop {
            self.skip_whitespace();
            let operation = match self.peek() {
                Some('+') => FormulaOperation::Add,
                Some('-') => FormulaOperation::Subtract,
                _ => return Ok(expression),
            };
            self.bump();
            let right = self.parse_term()?;
            expression = self.node(ProjectFormula::Binary {
                left: Box::new(expression),
                operation,
                right: Box::new(right),
            })?;
        }
    }

    fn parse_term(&mut self) -> Result<ProjectFormula, FormulaParseError> {
        let mut expression = self.parse_factor()?;
        loop {
            self.skip_whitespace();
            let operation = match self.peek() {
                Some('*') => FormulaOperation::Multiply,
                Some('/') => FormulaOperation::Divide,
                Some('%') => FormulaOperation::Remainder,
                _ => return Ok(expression),
            };
            self.bump();
            let right = self.parse_factor()?;
            expression = self.node(ProjectFormula::Binary {
                left: Box::new(expression),
                operation,
                right: Box::new(right),
            })?;
        }
    }

    fn parse_factor(&mut self) -> Result<ProjectFormula, FormulaParseError> {
        self.skip_whitespace();
        match self.peek() {
            Some('+') => {
                self.bump();
                self.parse_nested_factor()
            }
            Some('-') => {
                self.bump();
                let value = self.parse_nested_factor()?;
                self.node(ProjectFormula::Negate(Box::new(value)))
            }
            Some('(') => {
                self.bump();
                let expression = self.parse_nested_expression()?;
                if !self.consume(')') {
                    return Err(self.error("unbalanced formula parenthesis"));
                }
                Ok(expression)
            }
            Some(character) if character.is_ascii_digit() || character == '.' => {
                self.parse_number()
            }
            Some(character) if character.is_ascii_alphabetic() || character == '_' => {
                self.parse_identifier_or_function()
            }
            _ => Err(self.error("expected a formula value")),
        }
    }

    fn parse_number(&mut self) -> Result<ProjectFormula, FormulaParseError> {
        let start = self.offset;
        let mut saw_dot = false;
        while let Some(character) = self.peek() {
            if character.is_ascii_digit() {
                self.bump();
            } else if character == '.' && !saw_dot {
                saw_dot = true;
                self.bump();
            } else {
                break;
            }
        }
        let value = self
            .source
            .get(start..self.offset)
            .and_then(|value| value.parse::<f64>().ok())
            .filter(|value| value.is_finite())
            .ok_or_else(|| self.error("formula number must be finite"))?;
        self.node(ProjectFormula::Value(value))
    }

    fn parse_identifier_or_function(&mut self) -> Result<ProjectFormula, FormulaParseError> {
        let start = self.offset;
        while self
            .peek()
            .is_some_and(|character| character.is_ascii_alphanumeric() || character == '_')
        {
            self.bump();
        }
        let name = self.source.get(start..self.offset).unwrap_or_default();
        self.skip_whitespace();
        if self.peek() != Some('(') {
            return match name {
                "value" => self.node(ProjectFormula::Input),
                "pi" => self.node(ProjectFormula::Value(std::f64::consts::PI)),
                "e" => self.node(ProjectFormula::Value(std::f64::consts::E)),
                _ => Err(self.error("unknown formula identifier")),
            };
        }

        self.bump();
        let mut arguments = Vec::new();
        self.skip_whitespace();
        if self.peek() != Some(')') {
            loop {
                arguments.push(self.parse_nested_expression()?);
                self.skip_whitespace();
                if self.peek() != Some(',') {
                    break;
                }
                self.bump();
            }
        }
        if !self.consume(')') {
            return Err(self.error("invalid formula function arguments"));
        }
        let function = match name {
            "min" => FormulaFunction::Min,
            "max" => FormulaFunction::Max,
            "abs" => FormulaFunction::Abs,
            "round" => FormulaFunction::Round,
            "floor" => FormulaFunction::Floor,
            "ceil" => FormulaFunction::Ceil,
            "sqrt" => FormulaFunction::SquareRoot,
            "pow" => FormulaFunction::Power,
            "exp" => FormulaFunction::Exponential,
            "log" => FormulaFunction::NaturalLog,
            "sin" => FormulaFunction::Sine,
            "cos" => FormulaFunction::Cosine,
            "tan" => FormulaFunction::Tangent,
            _ => return Err(self.error("unknown formula function")),
        };
        let valid_arity = match function {
            FormulaFunction::Min | FormulaFunction::Max => !arguments.is_empty(),
            FormulaFunction::Power => arguments.len() == 2,
            FormulaFunction::Abs
            | FormulaFunction::Round
            | FormulaFunction::Floor
            | FormulaFunction::Ceil
            | FormulaFunction::SquareRoot
            | FormulaFunction::Exponential
            | FormulaFunction::NaturalLog
            | FormulaFunction::Sine
            | FormulaFunction::Cosine
            | FormulaFunction::Tangent => arguments.len() == 1,
        };
        if !valid_arity {
            return Err(self.error("invalid formula function arity"));
        }
        self.node(ProjectFormula::Function {
            function,
            arguments,
        })
    }
}
