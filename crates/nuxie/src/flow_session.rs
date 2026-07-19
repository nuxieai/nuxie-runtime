//! High-level, renderer-neutral execution seam for one remote UI flow.

use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    sync::Arc,
};

use crate::{
    ArtboardRenderCache, Factory, File, LinearAnimationInstance, OwnedArtboardInstance, Renderer,
    StateMachineInstance, ViewModelInstance,
};
use nuxie_runtime::{
    RuntimeEventPropertyValue, RuntimeOwnedViewModelHandle, RuntimeOwnedViewModelInstance,
    StateMachineReportedEvent,
};

/// Maximum UTF-8 byte length accepted for an identifier or property path.
pub const MAX_ID_PATH_BYTES: usize = 4 * 1024;
/// Maximum number of instances retained by one session.
pub const MAX_INSTANCES: usize = 4 * 1024;
pub const MAX_BATCH_ITEMS: usize = 4 * 1024;
pub const MAX_POINTERS_PER_BATCH: usize = 32;
pub const MAX_STRING_BYTES: usize = 1024 * 1024;
pub const MAX_LIST_ITEMS: usize = 4 * 1024;
pub const MAX_VALUE_NODES: usize = 4 * 1024;
pub const MAX_EVENT_PROPERTIES: usize = 256;
pub const MAX_VALUE_EDGES: usize = 16 * 1024;
pub const MAX_VALUE_DEPTH: usize = 32;
pub const MAX_ENCODED_PAYLOAD_BYTES: usize = 4 * 1024 * 1024;

/// Selection requested when creating a flow session.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FlowSessionConfig {
    pub artboard_name: Option<String>,
    /// An explicit player name selects a state machine. Linear animations are
    /// only selected by the deterministic fallback policy.
    pub player_name: Option<String>,
}

/// Machine-readable category for a rejected flow operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowSessionErrorKind {
    NotFound,
    InvalidArgument,
    LimitExceeded,
    Conflict,
    Runtime,
}

/// Typed failure at the flow-session seam.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowSessionError {
    kind: FlowSessionErrorKind,
    message: String,
}

impl FlowSessionError {
    fn new(kind: FlowSessionErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
        }
    }

    pub const fn kind(&self) -> FlowSessionErrorKind {
        self.kind
    }

    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for FlowSessionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for FlowSessionError {}

/// Stable, session-scoped identity exposed to hosts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FlowInstanceId(u64);

impl FlowInstanceId {
    pub const fn new(value: u64) -> Option<Self> {
        if value == 0 { None } else { Some(Self(value)) }
    }

    pub const fn get(self) -> u64 {
        self.0
    }
}

/// Runtime player selected for the session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowPlayerKind {
    StateMachine,
    LinearAnimation,
    Static,
}

/// Exact selection branch used during deterministic creation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowPlayerSelection {
    ExplicitStateMachine,
    AuthoredDefaultStateMachine,
    FirstStateMachine,
    FirstAnimation,
    Static,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowPlayerMetadata {
    pub kind: FlowPlayerKind,
    pub selection: FlowPlayerSelection,
    pub index: Option<usize>,
    pub name: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FlowArtboardBounds {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowValueType {
    Null,
    String,
    Number,
    Bool,
    Enum,
    Color,
    Image,
    Object,
    ViewModel,
    List,
    /// Semantic trigger property. Its monotonic count is represented by an
    /// `Enum` node because the canonical recursive arena has no trigger node.
    Trigger,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowPropertySchema {
    pub name: String,
    pub value_type: FlowValueType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowSchema {
    pub name: String,
    pub properties: Vec<FlowPropertySchema>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowInstanceMetadata {
    pub id: FlowInstanceId,
    pub schema_name: String,
    pub authored_name: Option<String>,
    pub is_root: bool,
}

/// Authored immutable recipe. Templates are not addressable mutable instances.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowInstanceTemplate {
    pub schema_name: String,
    pub authored_name: Option<String>,
    pub authored_index: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FlowCatalog {
    pub schemas: Vec<FlowSchema>,
    pub templates: Vec<FlowInstanceTemplate>,
    pub instances: Vec<FlowInstanceMetadata>,
    pub root_instance_id: Option<FlowInstanceId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FlowValueId(u32);

impl FlowValueId {
    pub const fn get(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum FlowValue {
    Null,
    String(String),
    Number(f32),
    Bool(bool),
    Enum(u64),
    Color(u32),
    Image(u64),
    Object(Vec<(String, FlowValueId)>),
    ViewModel(Vec<(String, FlowValueId)>),
    List(Vec<FlowValueId>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowValueNode {
    pub id: FlowValueId,
    pub value: FlowValue,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct FlowValueArena {
    pub roots: Vec<(FlowInstanceId, FlowValueId)>,
    pub nodes: Vec<FlowValueNode>,
}

/// Immutable creation result. Building it does not advance a player or drain
/// any runtime queue.
#[derive(Debug, Clone, PartialEq)]
pub struct FlowBootstrap {
    pub artboard_index: usize,
    pub artboard_name: Option<String>,
    pub player: FlowPlayerMetadata,
    pub bounds: FlowArtboardBounds,
    pub catalog: FlowCatalog,
    pub values: FlowValueArena,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FlowOutputPhase {
    /// Reserved for lifecycle callbacks queued by a host-facing session API.
    /// No current operation produces this phase.
    DelayedEventCallbacks,
    ReportedEvents,
    RuntimeAdvance,
    ViewModelChanges,
    HostWork,
    Render,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FlowOutputPayload {
    ReportedEvent {
        name: Option<String>,
        event_type: u32,
        /// Time between the authored event instant and the end of the runtime
        /// advance that produced it. This is overshoot metadata, not a future
        /// delivery deadline.
        delay_seconds: f32,
        properties: Vec<FlowEventProperty>,
    },
    StateChanged {
        instance_id: Option<FlowInstanceId>,
        path: String,
        value: Option<FlowScalarValue>,
        origin_mutation_id: Option<u64>,
    },
    HostCommand {
        name: String,
        payload: Vec<u8>,
    },
    RenderRequested {
        artboard_index: usize,
    },
    Metadata(FlowPlayerMetadata),
    RuntimeAdvanced {
        delta_seconds: f32,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowEventProperty {
    pub name: Option<String>,
    pub value: FlowScalarValue,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowOutput {
    pub sequence: u64,
    pub cycle: u64,
    pub phase: FlowOutputPhase,
    pub payload: FlowOutputPayload,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FlowScalarValue {
    Null,
    String(String),
    Number(f32),
    Bool(bool),
    Enum(u64),
    Color(u32),
    Image(u64),
    Trigger(u64),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FlowInstanceRef {
    Existing(FlowInstanceId),
    New(u32),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowNewInstance {
    pub local_id: u32,
    pub schema_name: String,
    pub authored_instance_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FlowStateMutation {
    SetInputBool {
        name: String,
        value: bool,
    },
    SetInputNumber {
        name: String,
        value: f32,
    },
    FireInputTrigger {
        name: String,
    },
    SetValue {
        instance: FlowInstanceRef,
        path: String,
        value: FlowScalarValue,
    },
    FireTrigger {
        instance: FlowInstanceRef,
        path: String,
    },
    ListInsert {
        instance: FlowInstanceRef,
        path: String,
        index: usize,
        item: FlowInstanceRef,
    },
    ListRemove {
        instance: FlowInstanceRef,
        path: String,
        index: usize,
    },
    ListSwap {
        instance: FlowInstanceRef,
        path: String,
        first: usize,
        second: usize,
    },
    ListMove {
        instance: FlowInstanceRef,
        path: String,
        from: usize,
        to: usize,
    },
    ListSet {
        instance: FlowInstanceRef,
        path: String,
        index: usize,
        item: FlowInstanceRef,
    },
    ListClear {
        instance: FlowInstanceRef,
        path: String,
    },
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct FlowStateBatch {
    pub host_mutation_id: Option<u64>,
    pub mutations: Vec<FlowStateMutation>,
    pub new_instances: Vec<FlowNewInstance>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowPointerKind {
    Down,
    Move,
    Up,
    Cancel,
    Exit,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FlowPointerEvent {
    pub kind: FlowPointerKind,
    pub pointer_id: i32,
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct FlowPointerBatch {
    pub events: Vec<FlowPointerEvent>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct FlowAdvance {
    pub timestamp_seconds: f64,
    pub delta_seconds: f32,
    pub render: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowQuery {
    Bootstrap,
    Values,
    Catalog,
    PlayerInputs,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowInputSnapshot {
    pub name: Option<String>,
    pub kind: crate::StateMachineInputKind,
    pub value: FlowScalarValue,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FlowOperation {
    StateBatch(FlowStateBatch),
    PointerBatch(FlowPointerBatch),
    Advance(FlowAdvance),
    Query(FlowQuery),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlowCreatedInstance {
    pub local_id: u32,
    pub id: FlowInstanceId,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FlowResult {
    pub outputs: Vec<FlowOutput>,
    pub dirty: bool,
    pub settled: bool,
    /// `Some(0)` requests an immediate follow-up operation to drain work
    /// produced by the preceding runtime advance. Future deadlines are not
    /// inferred from reported-event overshoot metadata.
    pub wake_after_seconds: Option<f32>,
    pub snapshot: Option<FlowBootstrap>,
    pub values: Option<FlowValueArena>,
    pub catalog: Option<FlowCatalog>,
    pub player_inputs: Option<Vec<FlowInputSnapshot>>,
    pub created_instances: Vec<FlowCreatedInstance>,
}

impl FlowResult {
    fn idle(settled: bool) -> Self {
        Self {
            outputs: Vec::new(),
            dirty: false,
            settled,
            wake_after_seconds: None,
            snapshot: None,
            values: None,
            catalog: None,
            player_inputs: None,
            created_instances: Vec::new(),
        }
    }
}

enum FlowPlayer {
    StateMachine(Box<StateMachineInstance>),
    Animation(LinearAnimationInstance),
    Static,
}

/// Deep, renderer-neutral module owning one live flow.
pub struct FlowSession {
    artboard: OwnedArtboardInstance,
    player: FlowPlayer,
    instances: BTreeMap<FlowInstanceId, ViewModelInstance>,
    root_instance_id: Option<FlowInstanceId>,
    next_instance_id: u64,
    creation_bootstrap: FlowBootstrap,
    bootstrap: FlowBootstrap,
    next_sequence: u64,
    next_cycle: u64,
    last_timestamp_seconds: Option<f64>,
    pending_animation_events: Vec<StateMachineReportedEvent>,
    active_pointer_ids: BTreeSet<i32>,
    terminal_failure: Option<FlowSessionError>,
}

impl fmt::Debug for FlowSession {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("FlowSession")
            .field("artboard_index", &self.bootstrap.artboard_index)
            .field("player", &self.bootstrap.player)
            .finish_non_exhaustive()
    }
}

impl FlowSession {
    pub fn create(
        file: Arc<File>,
        config: FlowSessionConfig,
    ) -> Result<(Self, FlowBootstrap), FlowSessionError> {
        validate_optional_selector(config.artboard_name.as_deref(), "artboard name")?;
        validate_optional_selector(config.player_name.as_deref(), "player name")?;

        let artboard_index = match config.artboard_name.as_deref() {
            Some(name) => file
                .artboard_named(name)
                .map(|artboard| artboard.index())
                .ok_or_else(|| {
                    FlowSessionError::new(
                        FlowSessionErrorKind::NotFound,
                        format!("artboard '{name}' was not found"),
                    )
                })?,
            None => file
                .default_artboard()
                .map(|artboard| artboard.index())
                .ok_or_else(|| {
                    FlowSessionError::new(
                        FlowSessionErrorKind::NotFound,
                        "file contains no artboards",
                    )
                })?,
        };

        let artboard = file.artboard(artboard_index).ok_or_else(|| {
            FlowSessionError::new(
                FlowSessionErrorKind::Runtime,
                "selected artboard disappeared",
            )
        })?;
        let artboard_name = artboard.name().map(ToOwned::to_owned);
        let (player_metadata, player_index) =
            select_player(artboard, config.player_name.as_deref())?;
        let mut instance = OwnedArtboardInstance::instantiate(Arc::clone(&file), artboard_index)
            .map_err(|error| {
                FlowSessionError::new(FlowSessionErrorKind::Runtime, error.to_string())
            })?;

        let root_view_model_selection = instance.view_model_index().map(|view_model_index| {
            let authored_index = file
                .runtime()
                .view_model_default_instance(view_model_index)
                .map(|default| default.instance_index);
            (view_model_index, authored_index)
        });
        let root_view_model = root_view_model_selection.and_then(|(_, authored_index)| {
            authored_index
                .and_then(|index| instance.instantiate_view_model_instance(index))
                .or_else(|| instance.instantiate_view_model())
        });
        if let Some(view_model) = root_view_model.as_ref() {
            let _ = instance.bind_view_model(view_model);
        }

        let player = match player_metadata.kind {
            FlowPlayerKind::StateMachine => {
                let index = player_index.ok_or_else(|| {
                    FlowSessionError::new(
                        FlowSessionErrorKind::Runtime,
                        "state-machine selection has no index",
                    )
                })?;
                let machine = instance.state_machine_instance(index).ok_or_else(|| {
                    FlowSessionError::new(
                        FlowSessionErrorKind::Runtime,
                        "selected state machine could not be instantiated",
                    )
                })?;
                FlowPlayer::StateMachine(Box::new(machine))
            }
            FlowPlayerKind::LinearAnimation => {
                let index = player_index.ok_or_else(|| {
                    FlowSessionError::new(
                        FlowSessionErrorKind::Runtime,
                        "animation selection has no index",
                    )
                })?;
                let animation =
                    instance
                        .raw()
                        .linear_animation_instance(index)
                        .ok_or_else(|| {
                            FlowSessionError::new(
                                FlowSessionErrorKind::Runtime,
                                "selected animation could not be instantiated",
                            )
                        })?;
                FlowPlayer::Animation(animation)
            }
            FlowPlayerKind::Static => FlowPlayer::Static,
        };

        let (x, y, width, height) = instance.artboard_bounds();
        if !x.is_finite()
            || !y.is_finite()
            || !width.is_finite()
            || !height.is_finite()
            || width <= 0.0
            || height <= 0.0
            || !(x + width).is_finite()
            || !(y + height).is_finite()
        {
            return Err(FlowSessionError::new(
                FlowSessionErrorKind::Runtime,
                "artboard bounds are non-finite",
            ));
        }
        let catalog = build_catalog(&file, root_view_model_selection)?;
        let bootstrap = FlowBootstrap {
            artboard_index,
            artboard_name,
            player: player_metadata,
            bounds: FlowArtboardBounds {
                x,
                y,
                width,
                height,
            },
            catalog,
            values: FlowValueArena::default(),
        };
        let root_instance_id = bootstrap.catalog.root_instance_id;
        let mut instances = BTreeMap::new();
        if let (Some(id), Some(view_model)) = (root_instance_id, root_view_model) {
            instances.insert(id, view_model);
        }
        let mut session = Self {
            artboard: instance,
            player,
            instances,
            root_instance_id,
            next_instance_id: 2,
            creation_bootstrap: bootstrap.clone(),
            bootstrap: bootstrap.clone(),
            next_sequence: 1,
            next_cycle: 1,
            last_timestamp_seconds: None,
            pending_animation_events: Vec::new(),
            active_pointer_ids: BTreeSet::new(),
            terminal_failure: None,
        };
        session.refresh_values()?;
        let bootstrap = session.bootstrap.clone();
        session.creation_bootstrap = bootstrap.clone();
        Ok((session, bootstrap))
    }

    /// Perform one bounded operation against the live flow.
    pub fn perform(&mut self, operation: FlowOperation) -> Result<FlowResult, FlowSessionError> {
        if let Some(failure) = self.terminal_failure.as_ref() {
            return Err(failure.clone());
        }
        match operation {
            FlowOperation::Query(query) => {
                let mut result = FlowResult::idle(self.is_settled());
                match query {
                    FlowQuery::Bootstrap => result.snapshot = Some(self.creation_bootstrap.clone()),
                    FlowQuery::Values => result.values = Some(self.bootstrap.values.clone()),
                    FlowQuery::Catalog => result.catalog = Some(self.bootstrap.catalog.clone()),
                    FlowQuery::PlayerInputs => {
                        let inputs = match &self.player {
                            FlowPlayer::StateMachine(machine) => {
                                if machine.input_count() > MAX_BATCH_ITEMS {
                                    return Err(FlowSessionError::new(
                                        FlowSessionErrorKind::LimitExceeded,
                                        "player input query item limit exceeded",
                                    ));
                                }
                                (0..machine.input_count())
                                    .filter_map(|index| {
                                        let input = machine.input(index)?;
                                        let value = match input.kind() {
                                            crate::StateMachineInputKind::Bool => {
                                                FlowScalarValue::Bool(input.bool_value()?)
                                            }
                                            crate::StateMachineInputKind::Number => {
                                                FlowScalarValue::Number(input.number_value()?)
                                            }
                                            crate::StateMachineInputKind::Trigger => {
                                                FlowScalarValue::Bool(input.trigger_fired()?)
                                            }
                                        };
                                        Some(FlowInputSnapshot {
                                            name: input.name().map(ToOwned::to_owned),
                                            kind: input.kind(),
                                            value,
                                        })
                                    })
                                    .collect()
                            }
                            FlowPlayer::Animation(_) | FlowPlayer::Static => Vec::new(),
                        };
                        validate_player_input_snapshot(&inputs)?;
                        result.player_inputs = Some(inputs);
                    }
                }
                Ok(result)
            }
            FlowOperation::StateBatch(batch) => self.perform_state_batch(batch),
            FlowOperation::PointerBatch(batch) => self.perform_pointer_batch(batch),
            FlowOperation::Advance(advance) => self.perform_advance(advance),
        }
    }

    /// Create renderer resources scoped to this session's artboard.
    pub fn new_render_cache(&self) -> ArtboardRenderCache {
        self.artboard.new_render_cache()
    }

    /// Draw the current settled session state through renderer-neutral traits.
    pub fn draw(
        &mut self,
        factory: &mut dyn Factory,
        renderer: &mut dyn Renderer,
        cache: &mut ArtboardRenderCache,
    ) -> Result<(), FlowSessionError> {
        self.artboard
            .draw_with_render_cache(factory, renderer, cache)
            .map_err(|error| {
                FlowSessionError::new(FlowSessionErrorKind::Runtime, error.to_string())
            })
    }

    pub fn artboard_bounds(&self) -> FlowArtboardBounds {
        let (x, y, width, height) = self.artboard.artboard_bounds();
        FlowArtboardBounds {
            x,
            y,
            width,
            height,
        }
    }

    /// Whether the current player has no queued or continuing work.
    ///
    /// This is a read-only creation/result fact: it never advances the
    /// player or drains reported events.
    pub fn is_settled(&self) -> bool {
        let root_player_settled = match &self.player {
            FlowPlayer::StateMachine(machine) => {
                !machine.needs_advance() && machine.reported_event_count() == 0
            }
            FlowPlayer::Animation(animation) => {
                !self
                    .artboard
                    .raw()
                    .linear_animation_instance_keep_going(animation)
                    && self.pending_animation_events.is_empty()
            }
            FlowPlayer::Static => true,
        };
        root_player_settled && !self.artboard.raw().has_ongoing_nested_work()
    }

    fn has_pending_reports(&self) -> bool {
        match &self.player {
            FlowPlayer::StateMachine(machine) => machine.reported_event_count() > 0,
            FlowPlayer::Animation(_) => !self.pending_animation_events.is_empty(),
            FlowPlayer::Static => false,
        }
    }

    fn perform_state_batch(
        &mut self,
        batch: FlowStateBatch,
    ) -> Result<FlowResult, FlowSessionError> {
        let item_count = batch
            .mutations
            .len()
            .checked_add(batch.new_instances.len())
            .ok_or_else(|| {
                FlowSessionError::new(
                    FlowSessionErrorKind::LimitExceeded,
                    "state batch item count overflow",
                )
            })?;
        if item_count > MAX_BATCH_ITEMS {
            return Err(FlowSessionError::new(
                FlowSessionErrorKind::LimitExceeded,
                "state batch item limit exceeded",
            ));
        }
        if self
            .instances
            .len()
            .saturating_add(batch.new_instances.len())
            > MAX_INSTANCES
        {
            return Err(FlowSessionError::new(
                FlowSessionErrorKind::LimitExceeded,
                "instance limit exceeded",
            ));
        }
        if state_batch_payload_bytes(&batch)? > MAX_ENCODED_PAYLOAD_BYTES {
            return Err(FlowSessionError::new(
                FlowSessionErrorKind::LimitExceeded,
                "encoded state batch exceeds 4 MiB",
            ));
        }

        let mut new_ids = BTreeMap::new();
        let mut prepared_new = BTreeMap::new();
        let mut next_instance_id = self.next_instance_id;
        for new_instance in &batch.new_instances {
            validate_required_id_path(&new_instance.schema_name, "schema name")?;
            validate_optional_selector(
                new_instance.authored_instance_name.as_deref(),
                "authored instance name",
            )?;
            if new_ids.contains_key(&new_instance.local_id) {
                return Err(FlowSessionError::new(
                    FlowSessionErrorKind::Conflict,
                    "duplicate transaction-local instance id",
                ));
            }
            let id = FlowInstanceId(next_instance_id);
            next_instance_id = next_instance_id.checked_add(1).ok_or_else(|| {
                FlowSessionError::new(
                    FlowSessionErrorKind::LimitExceeded,
                    "instance identity overflow",
                )
            })?;
            let view_model = instantiate_named_view_model(
                self.artboard.file(),
                &new_instance.schema_name,
                new_instance.authored_instance_name.as_deref(),
            )?;
            new_ids.insert(new_instance.local_id, id);
            prepared_new.insert(id, view_model);
        }

        let resolved = batch
            .mutations
            .iter()
            .map(|mutation| resolve_mutation(mutation, &new_ids))
            .collect::<Result<Vec<_>, _>>()?;

        for mutation in &resolved {
            for id in mutation_instance_ids(mutation) {
                if !prepared_new.contains_key(&id) && !self.instances.contains_key(&id) {
                    return Err(FlowSessionError::new(
                        FlowSessionErrorKind::NotFound,
                        format!("instance {} was not found", id.get()),
                    ));
                }
            }
        }
        let graph_sources = self
            .instances
            .iter()
            .chain(prepared_new.iter())
            .map(|(id, instance)| (*id, instance.clone()))
            .collect::<Vec<_>>();
        let source_handles = graph_sources
            .iter()
            .map(|(_, instance)| instance.handle().clone())
            .collect::<Vec<_>>();
        let detached_handles = RuntimeOwnedViewModelHandle::detached_graph(&source_handles);
        let candidates = graph_sources
            .iter()
            .zip(detached_handles)
            .map(|((id, _), raw)| (*id, ViewModelInstance { raw }))
            .collect::<BTreeMap<_, _>>();

        let mut machine_candidate = match &self.player {
            FlowPlayer::StateMachine(machine) => Some(machine.clone()),
            FlowPlayer::Animation(_) | FlowPlayer::Static => None,
        };
        for mutation in &resolved {
            prevalidate_and_apply_mutation(
                machine_candidate.as_deref_mut(),
                &candidates,
                mutation,
            )?;
        }
        let cycle = self.next_cycle;
        let next_cycle = self.next_cycle.checked_add(1).ok_or_else(|| {
            FlowSessionError::new(
                FlowSessionErrorKind::LimitExceeded,
                "cycle counter overflow",
            )
        })?;
        let mut result = FlowResult::idle(self.is_settled());
        result.dirty = !resolved.is_empty() || !prepared_new.is_empty();
        result.created_instances = batch
            .new_instances
            .iter()
            .filter_map(|item| {
                new_ids.get(&item.local_id).map(|id| FlowCreatedInstance {
                    local_id: item.local_id,
                    id: *id,
                })
            })
            .collect();
        let mut candidate_catalog = self.bootstrap.catalog.clone();
        for created in &result.created_instances {
            let source = batch
                .new_instances
                .iter()
                .find(|item| item.local_id == created.local_id)
                .ok_or_else(|| {
                    FlowSessionError::new(
                        FlowSessionErrorKind::Runtime,
                        "created instance metadata disappeared",
                    )
                })?;
            candidate_catalog.instances.push(FlowInstanceMetadata {
                id: created.id,
                schema_name: source.schema_name.clone(),
                authored_name: source.authored_instance_name.clone(),
                is_root: false,
            });
        }
        let mut candidate_instances = candidates;
        let candidate_values = prepare_value_snapshot(
            self.artboard.file(),
            &mut candidate_instances,
            &mut candidate_catalog,
            &mut next_instance_id,
        )?;
        let mut candidate_bootstrap = self.bootstrap.clone();
        candidate_bootstrap.catalog = candidate_catalog;
        candidate_bootstrap.values = candidate_values;
        validate_bootstrap_payload(&candidate_bootstrap)?;

        let mut next_sequence = self.next_sequence;
        for mutation in &resolved {
            if let Some((instance_id, path, value)) = mutation_echo(mutation) {
                append_output(
                    &mut result.outputs,
                    &mut next_sequence,
                    cycle,
                    FlowOutputPhase::ViewModelChanges,
                    FlowOutputPayload::StateChanged {
                        instance_id,
                        path,
                        value,
                        origin_mutation_id: batch.host_mutation_id,
                    },
                )?;
            }
        }

        if let Some(root) = self
            .root_instance_id
            .and_then(|id| candidate_instances.get(&id))
        {
            let _ = self.artboard.bind_view_model(root);
            if let Some(machine) = machine_candidate.as_mut() {
                let _ = machine.bind_owned_view_model_handle(root.handle());
            }
        }
        if let (FlowPlayer::StateMachine(machine), Some(candidate)) =
            (&mut self.player, machine_candidate)
        {
            *machine = candidate;
        }
        self.instances = candidate_instances;
        self.bootstrap = candidate_bootstrap;
        self.next_instance_id = next_instance_id;
        self.next_cycle = next_cycle;
        self.next_sequence = next_sequence;
        result.settled = self.is_settled();
        Ok(result)
    }

    fn perform_pointer_batch(
        &mut self,
        batch: FlowPointerBatch,
    ) -> Result<FlowResult, FlowSessionError> {
        if batch.events.len() > MAX_POINTERS_PER_BATCH {
            return Err(FlowSessionError::new(
                FlowSessionErrorKind::LimitExceeded,
                format!("pointer batch exceeds {MAX_POINTERS_PER_BATCH} events"),
            ));
        }
        let mut active_pointer_ids = self.active_pointer_ids.clone();
        for event in &batch.events {
            if event.pointer_id <= 0 {
                return Err(FlowSessionError::new(
                    FlowSessionErrorKind::InvalidArgument,
                    "pointer ids must be positive",
                ));
            }
            if !event.x.is_finite() || !event.y.is_finite() {
                return Err(FlowSessionError::new(
                    FlowSessionErrorKind::InvalidArgument,
                    "pointer coordinates must be finite",
                ));
            }
            match event.kind {
                FlowPointerKind::Down | FlowPointerKind::Move => {
                    active_pointer_ids.insert(event.pointer_id);
                    if active_pointer_ids.len() > MAX_POINTERS_PER_BATCH {
                        return Err(FlowSessionError::new(
                            FlowSessionErrorKind::LimitExceeded,
                            format!("session exceeds {MAX_POINTERS_PER_BATCH} active pointers"),
                        ));
                    }
                }
                FlowPointerKind::Up | FlowPointerKind::Cancel | FlowPointerKind::Exit => {
                    active_pointer_ids.remove(&event.pointer_id);
                }
            }
        }

        let operation_result = (|| {
            let mut result = FlowResult::idle(self.is_settled());
            for event in batch.events {
                let before = self.bootstrap.values.clone();
                let changed = self.apply_pointer_event(event)?;
                result.dirty |= changed;
                match event.kind {
                    FlowPointerKind::Down | FlowPointerKind::Up | FlowPointerKind::Cancel => {
                        let cycle_result = self.run_player_cycle(0.0, false)?;
                        merge_results(&mut result, cycle_result)?;
                    }
                    FlowPointerKind::Move | FlowPointerKind::Exit => {
                        let cycle_result = self.finish_nonadvance_pointer_cycle(before, changed)?;
                        merge_results(&mut result, cycle_result)?;
                    }
                }
            }
            result.settled = self.is_settled();
            Ok(result)
        })();
        match operation_result {
            Ok(result) => {
                self.active_pointer_ids = active_pointer_ids;
                Ok(result)
            }
            Err(error) => Err(self.poison_after_mutation(error)),
        }
    }

    fn perform_advance(&mut self, advance: FlowAdvance) -> Result<FlowResult, FlowSessionError> {
        if !advance.timestamp_seconds.is_finite() || !advance.delta_seconds.is_finite() {
            return Err(FlowSessionError::new(
                FlowSessionErrorKind::InvalidArgument,
                "advance timestamp and delta must be finite",
            ));
        }
        if advance.delta_seconds < 0.0 {
            return Err(FlowSessionError::new(
                FlowSessionErrorKind::InvalidArgument,
                "advance delta must not be negative",
            ));
        }
        if self
            .last_timestamp_seconds
            .is_some_and(|last| advance.timestamp_seconds < last)
        {
            return Err(FlowSessionError::new(
                FlowSessionErrorKind::Conflict,
                "advance timestamp must be nondecreasing",
            ));
        }
        self.last_timestamp_seconds = Some(advance.timestamp_seconds);
        self.run_player_cycle(advance.delta_seconds, advance.render)
            .map_err(|error| self.poison_after_mutation(error))
    }

    fn poison_after_mutation(&mut self, error: FlowSessionError) -> FlowSessionError {
        self.terminal_failure = Some(FlowSessionError::new(
            FlowSessionErrorKind::Runtime,
            format!("flow session is terminal after a failed mutation: {error}"),
        ));
        error
    }

    fn apply_pointer_event(&mut self, event: FlowPointerEvent) -> Result<bool, FlowSessionError> {
        let FlowPlayer::StateMachine(machine) = &mut self.player else {
            return Ok(false);
        };
        let root = self
            .root_instance_id
            .and_then(|id| self.instances.get(&id))
            .cloned();
        let changed = if let Some(root) = root {
            let mut context = root.raw_mut();
            match event.kind {
                FlowPointerKind::Down => machine.pointer_down_with_owned_view_model_context(
                    self.artboard.raw(),
                    event.x,
                    event.y,
                    event.pointer_id,
                    &mut context,
                ),
                FlowPointerKind::Move => machine.pointer_move_with_owned_view_model_context(
                    self.artboard.raw(),
                    event.x,
                    event.y,
                    0.0,
                    event.pointer_id,
                    &mut context,
                ),
                FlowPointerKind::Up | FlowPointerKind::Cancel => {
                    let mut changed = machine.pointer_up_with_owned_view_model_context(
                        self.artboard.raw(),
                        event.x,
                        event.y,
                        event.pointer_id,
                        &mut context,
                    );
                    changed |= machine.pointer_exit_with_owned_view_model_context(
                        self.artboard.raw(),
                        event.x,
                        event.y,
                        event.pointer_id,
                        &mut context,
                    );
                    changed
                }
                FlowPointerKind::Exit => machine.pointer_exit_with_owned_view_model_context(
                    self.artboard.raw(),
                    event.x,
                    event.y,
                    event.pointer_id,
                    &mut context,
                ),
            }
        } else {
            match event.kind {
                FlowPointerKind::Down => {
                    machine.pointer_down(self.artboard.raw(), event.x, event.y, event.pointer_id)
                }
                FlowPointerKind::Move => machine.pointer_move(
                    self.artboard.raw(),
                    event.x,
                    event.y,
                    0.0,
                    event.pointer_id,
                ),
                FlowPointerKind::Up | FlowPointerKind::Cancel => {
                    let mut changed =
                        machine.pointer_up(self.artboard.raw(), event.x, event.y, event.pointer_id);
                    changed |= machine.pointer_exit(
                        self.artboard.raw(),
                        event.x,
                        event.y,
                        event.pointer_id,
                    );
                    changed
                }
                FlowPointerKind::Exit => {
                    machine.pointer_exit(self.artboard.raw(), event.x, event.y, event.pointer_id)
                }
            }
        };
        Ok(changed)
    }

    fn run_player_cycle(
        &mut self,
        delta_seconds: f32,
        render: bool,
    ) -> Result<FlowResult, FlowSessionError> {
        let before_values = self.bootstrap.values.clone();
        let cycle = self.next_cycle;
        self.next_cycle = self.next_cycle.checked_add(1).ok_or_else(|| {
            FlowSessionError::new(
                FlowSessionErrorKind::LimitExceeded,
                "cycle counter overflow",
            )
        })?;

        let pending_events = match &mut self.player {
            FlowPlayer::StateMachine(machine) => machine.take_reported_events(),
            FlowPlayer::Animation(_) => std::mem::take(&mut self.pending_animation_events),
            FlowPlayer::Static => Vec::new(),
        };
        if pending_events.len() > MAX_BATCH_ITEMS {
            return Err(FlowSessionError::new(
                FlowSessionErrorKind::LimitExceeded,
                format!("runtime emitted more than {MAX_BATCH_ITEMS} events"),
            ));
        }

        let mut reported_payloads = Vec::with_capacity(pending_events.len());
        for event in pending_events {
            if !event.seconds_delay().is_finite() || event.seconds_delay() < 0.0 {
                return Err(FlowSessionError::new(
                    FlowSessionErrorKind::Runtime,
                    "runtime emitted an event with an invalid delay",
                ));
            }
            reported_payloads.push(self.event_payload(event)?);
        }
        let changed = match &mut self.player {
            FlowPlayer::StateMachine(machine) => self
                .artboard
                .advance_with_state_machine(machine, delta_seconds),
            FlowPlayer::Animation(animation) => {
                let mut events = Vec::new();
                let mut changed = self
                    .artboard
                    .raw_mut()
                    .advance_linear_animation_instance_with_events(
                        animation,
                        delta_seconds,
                        &mut events,
                    );
                changed |= self
                    .artboard
                    .raw_mut()
                    .apply_linear_animation_instance(animation, 1.0);
                changed |= self.artboard.advance(0.0);
                self.pending_animation_events = events;
                changed
            }
            FlowPlayer::Static => self.artboard.advance(delta_seconds),
        };

        let mut result = FlowResult::idle(false);
        result.dirty = changed;
        for payload in reported_payloads {
            self.push_output(
                &mut result.outputs,
                cycle,
                FlowOutputPhase::ReportedEvents,
                payload,
            )?;
        }
        self.push_output(
            &mut result.outputs,
            cycle,
            FlowOutputPhase::RuntimeAdvance,
            FlowOutputPayload::RuntimeAdvanced { delta_seconds },
        )?;
        self.refresh_values()?;
        let value_changes = diff_value_arenas(&before_values, &self.bootstrap.values)?;
        result.dirty |= !value_changes.is_empty();
        for (instance_id, path, value) in value_changes {
            self.push_output(
                &mut result.outputs,
                cycle,
                FlowOutputPhase::ViewModelChanges,
                FlowOutputPayload::StateChanged {
                    instance_id: Some(instance_id),
                    path,
                    value,
                    origin_mutation_id: None,
                },
            )?;
        }
        if render {
            self.push_output(
                &mut result.outputs,
                cycle,
                FlowOutputPhase::Render,
                FlowOutputPayload::RenderRequested {
                    artboard_index: self.bootstrap.artboard_index,
                },
            )?;
        }
        let has_pending_reports = self.has_pending_reports();
        result.wake_after_seconds = has_pending_reports.then_some(0.0);
        result.settled = self.is_settled();
        Ok(result)
    }

    fn finish_nonadvance_pointer_cycle(
        &mut self,
        before_values: FlowValueArena,
        changed: bool,
    ) -> Result<FlowResult, FlowSessionError> {
        let cycle = self.next_cycle;
        self.next_cycle = self.next_cycle.checked_add(1).ok_or_else(|| {
            FlowSessionError::new(
                FlowSessionErrorKind::LimitExceeded,
                "cycle counter overflow",
            )
        })?;
        self.refresh_values()?;
        let changes = diff_value_arenas(&before_values, &self.bootstrap.values)?;
        let mut result = FlowResult::idle(self.is_settled());
        result.dirty = changed || !changes.is_empty();
        for (instance_id, path, value) in changes {
            self.push_output(
                &mut result.outputs,
                cycle,
                FlowOutputPhase::ViewModelChanges,
                FlowOutputPayload::StateChanged {
                    instance_id: Some(instance_id),
                    path,
                    value,
                    origin_mutation_id: None,
                },
            )?;
        }
        result.wake_after_seconds = self.has_pending_reports().then_some(0.0);
        result.settled = self.is_settled();
        Ok(result)
    }

    fn event_payload(
        &self,
        event: StateMachineReportedEvent,
    ) -> Result<FlowOutputPayload, FlowSessionError> {
        let runtime_properties = self
            .artboard
            .raw()
            .event_properties(event.event_local_index());
        if runtime_properties.len() > MAX_EVENT_PROPERTIES {
            return Err(FlowSessionError::new(
                FlowSessionErrorKind::LimitExceeded,
                "event property limit exceeded",
            ));
        }
        let properties: Vec<FlowEventProperty> = runtime_properties
            .into_iter()
            .map(|property| FlowEventProperty {
                name: property.name,
                value: match property.value {
                    RuntimeEventPropertyValue::Number(value) => FlowScalarValue::Number(value),
                    RuntimeEventPropertyValue::Bool(value) => FlowScalarValue::Bool(value),
                    RuntimeEventPropertyValue::String(value) => {
                        FlowScalarValue::String(String::from_utf8_lossy(&value).into_owned())
                    }
                    RuntimeEventPropertyValue::Color(value) => FlowScalarValue::Color(value),
                    RuntimeEventPropertyValue::Enum(value) => FlowScalarValue::Enum(value),
                    RuntimeEventPropertyValue::Trigger(value) => FlowScalarValue::Trigger(value),
                },
            })
            .collect();
        let name = event.name().map(ToOwned::to_owned);
        let event_type = event.event_core_type();
        let delay_seconds = event.seconds_delay();
        let payload = FlowOutputPayload::ReportedEvent {
            name,
            event_type,
            delay_seconds,
            properties,
        };
        validate_output_payload(&payload)?;
        Ok(payload)
    }

    fn push_output(
        &mut self,
        outputs: &mut Vec<FlowOutput>,
        cycle: u64,
        phase: FlowOutputPhase,
        payload: FlowOutputPayload,
    ) -> Result<(), FlowSessionError> {
        append_output(outputs, &mut self.next_sequence, cycle, phase, payload)
    }

    fn refresh_values(&mut self) -> Result<(), FlowSessionError> {
        self.bootstrap.values = prepare_value_snapshot(
            self.artboard.file(),
            &mut self.instances,
            &mut self.bootstrap.catalog,
            &mut self.next_instance_id,
        )?;
        validate_bootstrap_payload(&self.bootstrap)?;
        Ok(())
    }
}

fn prepare_value_snapshot(
    file: &File,
    instances: &mut BTreeMap<FlowInstanceId, ViewModelInstance>,
    catalog: &mut FlowCatalog,
    next_instance_id: &mut u64,
) -> Result<FlowValueArena, FlowSessionError> {
    let mut discovered = Vec::new();
    let mut traversed_edges = 0_usize;
    for instance in instances.values() {
        collect_reachable_list_instances(
            file,
            instance.handle(),
            "",
            0,
            &mut discovered,
            &mut traversed_edges,
        )?;
    }
    for handle in discovered {
        if instances
            .values()
            .any(|instance| instance.handle().ptr_eq(&handle))
        {
            continue;
        }
        if instances.len() >= MAX_INSTANCES {
            return Err(FlowSessionError::new(
                FlowSessionErrorKind::LimitExceeded,
                "instance limit exceeded",
            ));
        }
        let id = FlowInstanceId(*next_instance_id);
        *next_instance_id = next_instance_id.checked_add(1).ok_or_else(|| {
            FlowSessionError::new(
                FlowSessionErrorKind::LimitExceeded,
                "instance identity overflow",
            )
        })?;
        let view_model_index = handle.borrow().view_model_index();
        let schema_name = file
            .graph()
            .view_models
            .get(view_model_index)
            .and_then(|schema| schema.name.clone())
            .unwrap_or_else(|| format!("viewModel{view_model_index}"));
        catalog.instances.push(FlowInstanceMetadata {
            id,
            schema_name,
            authored_name: None,
            is_root: false,
        });
        instances.insert(id, ViewModelInstance { raw: handle });
    }
    validate_catalog(catalog)?;
    let mut builder = ValueArenaBuilder::new(file);
    for (id, instance) in instances {
        let root = builder.snapshot_handle(instance.handle(), "", 0)?;
        builder.arena.roots.push((*id, root));
    }
    Ok(builder.arena)
}

fn collect_reachable_list_instances(
    file: &File,
    handle: &RuntimeOwnedViewModelHandle,
    prefix: &str,
    depth: usize,
    discovered: &mut Vec<RuntimeOwnedViewModelHandle>,
    traversed_edges: &mut usize,
) -> Result<(), FlowSessionError> {
    if depth > MAX_VALUE_DEPTH {
        return Err(FlowSessionError::new(
            FlowSessionErrorKind::LimitExceeded,
            "view-model graph depth limit exceeded",
        ));
    }
    let view_model_index = if prefix.is_empty() {
        handle.borrow().view_model_index()
    } else {
        let root_index = handle.borrow().view_model_index();
        resolve_nested_schema_index(file, root_index, prefix)?
    };
    let schema = file.runtime().view_model(view_model_index).ok_or_else(|| {
        FlowSessionError::new(
            FlowSessionErrorKind::Runtime,
            "retained view-model schema disappeared",
        )
    })?;
    for property in schema.properties {
        let Some(name) = property.string_property("name") else {
            continue;
        };
        let path = if prefix.is_empty() {
            name.to_owned()
        } else {
            format!("{prefix}/{name}")
        };
        match property.type_name {
            "ViewModelPropertyList" => {
                let item_count = handle
                    .list_item_count_by_property_name_path(&path)
                    .unwrap_or(0);
                if item_count > MAX_LIST_ITEMS {
                    return Err(FlowSessionError::new(
                        FlowSessionErrorKind::LimitExceeded,
                        "reachable list item limit exceeded",
                    ));
                }
                *traversed_edges = traversed_edges.checked_add(item_count).ok_or_else(|| {
                    FlowSessionError::new(
                        FlowSessionErrorKind::LimitExceeded,
                        "reachable value edge counter overflow",
                    )
                })?;
                if *traversed_edges > MAX_VALUE_EDGES {
                    return Err(FlowSessionError::new(
                        FlowSessionErrorKind::LimitExceeded,
                        "reachable value edge limit exceeded",
                    ));
                }
                for item in handle
                    .list_items_by_property_name_path(&path)
                    .unwrap_or_default()
                    .into_iter()
                    .take(item_count)
                {
                    if !discovered.iter().any(|existing| existing.ptr_eq(&item)) {
                        if discovered.len() >= MAX_INSTANCES {
                            return Err(FlowSessionError::new(
                                FlowSessionErrorKind::LimitExceeded,
                                "reachable instance limit exceeded",
                            ));
                        }
                        discovered.push(item.clone());
                        collect_reachable_list_instances(
                            file,
                            &item,
                            "",
                            depth.saturating_add(1),
                            discovered,
                            traversed_edges,
                        )?;
                    }
                }
            }
            "ViewModelPropertyViewModel" => {
                *traversed_edges = traversed_edges.checked_add(1).ok_or_else(|| {
                    FlowSessionError::new(
                        FlowSessionErrorKind::LimitExceeded,
                        "reachable value edge counter overflow",
                    )
                })?;
                if *traversed_edges > MAX_VALUE_EDGES {
                    return Err(FlowSessionError::new(
                        FlowSessionErrorKind::LimitExceeded,
                        "reachable value edge limit exceeded",
                    ));
                }
                collect_reachable_list_instances(
                    file,
                    handle,
                    &path,
                    depth.saturating_add(1),
                    discovered,
                    traversed_edges,
                )?;
            }
            _ => {}
        }
    }
    Ok(())
}

fn resolve_nested_schema_index(
    file: &File,
    root_schema_index: usize,
    prefix: &str,
) -> Result<usize, FlowSessionError> {
    let mut schema_index = root_schema_index;
    for segment in prefix.split('/') {
        let schema = file.runtime().view_model(schema_index).ok_or_else(|| {
            FlowSessionError::new(
                FlowSessionErrorKind::Runtime,
                "nested view-model schema disappeared",
            )
        })?;
        let property = schema
            .properties
            .into_iter()
            .find(|property| property.string_property("name") == Some(segment))
            .ok_or_else(|| {
                FlowSessionError::new(
                    FlowSessionErrorKind::Runtime,
                    "nested view-model path disappeared",
                )
            })?;
        schema_index = usize::try_from(property.uint_property("viewModelId").ok_or_else(|| {
            FlowSessionError::new(
                FlowSessionErrorKind::Runtime,
                "nested view-model property has no schema id",
            )
        })?)
        .map_err(|_| {
            FlowSessionError::new(
                FlowSessionErrorKind::Runtime,
                "nested schema id is out of range",
            )
        })?;
    }
    Ok(schema_index)
}

struct ValueArenaBuilder<'a> {
    file: &'a File,
    arena: FlowValueArena,
    edge_count: usize,
    payload_bytes: usize,
    instance_nodes: Vec<(RuntimeOwnedViewModelHandle, FlowValueId)>,
}

impl<'a> ValueArenaBuilder<'a> {
    fn new(file: &'a File) -> Self {
        Self {
            file,
            arena: FlowValueArena::default(),
            edge_count: 0,
            payload_bytes: 0,
            instance_nodes: Vec::new(),
        }
    }

    fn push(&mut self, value: FlowValue) -> Result<FlowValueId, FlowSessionError> {
        if self.arena.nodes.len() >= MAX_VALUE_NODES {
            return Err(FlowSessionError::new(
                FlowSessionErrorKind::LimitExceeded,
                "value arena node limit exceeded",
            ));
        }
        match &value {
            FlowValue::String(value) if value.len() > MAX_STRING_BYTES => {
                return Err(FlowSessionError::new(
                    FlowSessionErrorKind::LimitExceeded,
                    "string value exceeds 1 MiB",
                ));
            }
            FlowValue::Number(value) if !value.is_finite() => {
                return Err(FlowSessionError::new(
                    FlowSessionErrorKind::Runtime,
                    "runtime produced a non-finite number value",
                ));
            }
            _ => {}
        }
        self.add_payload(16)?;
        let index = u32::try_from(self.arena.nodes.len()).map_err(|_| {
            FlowSessionError::new(
                FlowSessionErrorKind::LimitExceeded,
                "value arena identity overflow",
            )
        })?;
        let id = FlowValueId(index.checked_add(1).ok_or_else(|| {
            FlowSessionError::new(
                FlowSessionErrorKind::LimitExceeded,
                "value arena identity overflow",
            )
        })?);
        self.arena.nodes.push(FlowValueNode { id, value });
        Ok(id)
    }

    fn add_payload(&mut self, count: usize) -> Result<(), FlowSessionError> {
        self.payload_bytes = self.payload_bytes.checked_add(count).ok_or_else(|| {
            FlowSessionError::new(
                FlowSessionErrorKind::LimitExceeded,
                "encoded value arena size overflow",
            )
        })?;
        if self.payload_bytes > MAX_ENCODED_PAYLOAD_BYTES {
            return Err(FlowSessionError::new(
                FlowSessionErrorKind::LimitExceeded,
                "encoded value arena exceeds 4 MiB",
            ));
        }
        Ok(())
    }

    fn add_edges(&mut self, count: usize) -> Result<(), FlowSessionError> {
        self.edge_count = self.edge_count.checked_add(count).ok_or_else(|| {
            FlowSessionError::new(
                FlowSessionErrorKind::LimitExceeded,
                "value edge counter overflow",
            )
        })?;
        if self.edge_count > MAX_VALUE_EDGES {
            return Err(FlowSessionError::new(
                FlowSessionErrorKind::LimitExceeded,
                "value arena edge limit exceeded",
            ));
        }
        Ok(())
    }

    fn snapshot_handle(
        &mut self,
        handle: &RuntimeOwnedViewModelHandle,
        prefix: &str,
        depth: usize,
    ) -> Result<FlowValueId, FlowSessionError> {
        if depth > MAX_VALUE_DEPTH {
            return Err(FlowSessionError::new(
                FlowSessionErrorKind::LimitExceeded,
                "value arena depth limit exceeded",
            ));
        }
        if prefix.is_empty()
            && let Some((_, id)) = self
                .instance_nodes
                .iter()
                .find(|(existing, _)| existing.ptr_eq(handle))
        {
            return Ok(*id);
        }
        let reserved = if prefix.is_empty() {
            let id = self.push(FlowValue::Null)?;
            self.instance_nodes.push((handle.clone(), id));
            Some(id)
        } else {
            None
        };
        let view_model_index = handle.borrow().view_model_index();
        let view_model = self
            .file
            .runtime()
            .view_model(view_model_index)
            .ok_or_else(|| {
                FlowSessionError::new(
                    FlowSessionErrorKind::Runtime,
                    "retained view-model schema disappeared",
                )
            })?;
        let mut edges = Vec::new();
        for property in view_model.properties {
            let Some(name) = property.string_property("name") else {
                continue;
            };
            let path = if prefix.is_empty() {
                name.to_owned()
            } else {
                format!("{prefix}/{name}")
            };
            let child = self.snapshot_property(handle, property, &path, depth.saturating_add(1))?;
            edges.push((name.to_owned(), child));
            self.add_payload(name.len())?;
        }
        self.add_edges(edges.len())?;
        if let Some(id) = reserved {
            let node = self
                .arena
                .nodes
                .iter_mut()
                .find(|node| node.id == id)
                .ok_or_else(|| {
                    FlowSessionError::new(
                        FlowSessionErrorKind::Runtime,
                        "reserved value node disappeared",
                    )
                })?;
            node.value = FlowValue::ViewModel(edges);
            Ok(id)
        } else {
            self.push(FlowValue::ViewModel(edges))
        }
    }

    fn snapshot_property(
        &mut self,
        handle: &RuntimeOwnedViewModelHandle,
        property: &nuxie_binary::RuntimeObject,
        path: &str,
        depth: usize,
    ) -> Result<FlowValueId, FlowSessionError> {
        validate_required_id_path(path, "value path")?;
        if depth > MAX_VALUE_DEPTH {
            return Err(FlowSessionError::new(
                FlowSessionErrorKind::LimitExceeded,
                "value arena depth limit exceeded",
            ));
        }
        let raw = handle.borrow();
        let scalar = match property.type_name {
            "ViewModelPropertyString" => raw
                .string_value_by_property_name_path(path)
                .map(|bytes| {
                    if bytes.len() > MAX_STRING_BYTES {
                        return Err(FlowSessionError::new(
                            FlowSessionErrorKind::LimitExceeded,
                            "string value exceeds 1 MiB",
                        ));
                    }
                    String::from_utf8(bytes.to_vec())
                        .map(FlowValue::String)
                        .map_err(|_| {
                            FlowSessionError::new(
                                FlowSessionErrorKind::Runtime,
                                format!("string property '{path}' is not UTF-8"),
                            )
                        })
                })
                .transpose()?,
            "ViewModelPropertyNumber" | "ViewModelPropertyInteger" => raw
                .number_value_by_property_name_path(path)
                .map(FlowValue::Number),
            "ViewModelPropertyBoolean" => raw
                .boolean_value_by_property_name_path(path)
                .map(FlowValue::Bool),
            "ViewModelPropertyColor" => raw
                .color_value_by_property_name_path(path)
                .map(FlowValue::Color),
            "ViewModelPropertyEnum"
            | "ViewModelPropertyEnumCustom"
            | "ViewModelPropertyEnumSystem" => raw
                .enum_value_by_property_name_path(path)
                .map(FlowValue::Enum),
            "ViewModelPropertySymbolListIndex" => raw
                .symbol_list_index_value_by_property_name_path(path)
                .map(FlowValue::Enum),
            "ViewModelPropertyAsset" | "ViewModelPropertyAssetImage" => raw
                .asset_value_by_property_name_path(path)
                .map(FlowValue::Image),
            "ViewModelPropertyArtboard" => raw
                .artboard_value_by_property_name_path(path)
                .map(FlowValue::Enum),
            "ViewModelPropertyTrigger" => raw
                .trigger_value_by_property_name_path(path)
                .map(FlowValue::Enum),
            _ => None,
        };
        drop(raw);
        if let Some(value) = scalar {
            if let FlowValue::String(value) = &value {
                self.add_payload(value.len())?;
            }
            return self.push(value);
        }

        match property.type_name {
            "ViewModelPropertyList" => {
                let count = handle
                    .list_item_count_by_property_name_path(path)
                    .ok_or_else(|| {
                        FlowSessionError::new(
                            FlowSessionErrorKind::Runtime,
                            format!("list property '{path}' could not be snapshotted"),
                        )
                    })?;
                if count > MAX_LIST_ITEMS {
                    return Err(FlowSessionError::new(
                        FlowSessionErrorKind::LimitExceeded,
                        "list item limit exceeded",
                    ));
                }
                let items = handle
                    .list_items_by_property_name_path(path)
                    .unwrap_or_default();
                let mut item_nodes = Vec::with_capacity(count);
                for item in items.iter().take(count) {
                    item_nodes.push(self.snapshot_handle(item, "", depth.saturating_add(1))?);
                }
                while item_nodes.len() < count {
                    item_nodes.push(self.push(FlowValue::Null)?);
                }
                self.add_edges(item_nodes.len())?;
                self.push(FlowValue::List(item_nodes))
            }
            "ViewModelPropertyViewModel" => {
                let referenced_index =
                    usize::try_from(property.uint_property("viewModelId").ok_or_else(|| {
                        FlowSessionError::new(
                            FlowSessionErrorKind::Runtime,
                            "nested view-model property has no schema id",
                        )
                    })?)
                    .map_err(|_| {
                        FlowSessionError::new(
                            FlowSessionErrorKind::Runtime,
                            "nested view-model schema id is out of range",
                        )
                    })?;
                let nested = self
                    .file
                    .runtime()
                    .view_model(referenced_index)
                    .ok_or_else(|| {
                        FlowSessionError::new(
                            FlowSessionErrorKind::Runtime,
                            "nested view-model schema was not found",
                        )
                    })?;
                let mut edges = Vec::new();
                for child_property in nested.properties {
                    let Some(name) = child_property.string_property("name") else {
                        continue;
                    };
                    let child_path = format!("{path}/{name}");
                    let child = self.snapshot_property(
                        handle,
                        child_property,
                        &child_path,
                        depth.saturating_add(1),
                    )?;
                    edges.push((name.to_owned(), child));
                    self.add_payload(name.len())?;
                }
                self.add_edges(edges.len())?;
                self.push(FlowValue::ViewModel(edges))
            }
            _ => self.push(FlowValue::Null),
        }
    }
}

fn merge_results(target: &mut FlowResult, mut source: FlowResult) -> Result<(), FlowSessionError> {
    if target.outputs.len().saturating_add(source.outputs.len()) > MAX_BATCH_ITEMS {
        return Err(FlowSessionError::new(
            FlowSessionErrorKind::LimitExceeded,
            "output limit exceeded",
        ));
    }
    let encoded_size = target
        .outputs
        .iter()
        .chain(source.outputs.iter())
        .try_fold(0_usize, |total, output| {
            total.checked_add(flow_output_payload_bytes(&output.payload))
        })
        .ok_or_else(|| {
            FlowSessionError::new(
                FlowSessionErrorKind::LimitExceeded,
                "encoded output size overflow",
            )
        })?;
    if encoded_size > MAX_ENCODED_PAYLOAD_BYTES {
        return Err(FlowSessionError::new(
            FlowSessionErrorKind::LimitExceeded,
            "encoded output exceeds 4 MiB",
        ));
    }
    target.outputs.append(&mut source.outputs);
    target.dirty |= source.dirty;
    target.settled = source.settled;
    target.wake_after_seconds = source.wake_after_seconds;
    target
        .created_instances
        .append(&mut source.created_instances);
    Ok(())
}

fn validate_player_input_snapshot(inputs: &[FlowInputSnapshot]) -> Result<(), FlowSessionError> {
    if inputs.len() > MAX_BATCH_ITEMS {
        return Err(FlowSessionError::new(
            FlowSessionErrorKind::LimitExceeded,
            "player input query item limit exceeded",
        ));
    }
    let mut total = inputs.len().saturating_mul(16);
    for input in inputs {
        if let Some(name) = input.name.as_deref() {
            validate_required_id_path(name, "input name")?;
            checked_payload_add(&mut total, name.len())?;
        }
        validate_scalar_value(&input.value, "player input")?;
        if let FlowScalarValue::String(value) = &input.value {
            checked_payload_add(&mut total, value.len())?;
        }
    }
    if total > MAX_ENCODED_PAYLOAD_BYTES {
        return Err(FlowSessionError::new(
            FlowSessionErrorKind::LimitExceeded,
            "encoded player input snapshot exceeds 4 MiB",
        ));
    }
    Ok(())
}

fn checked_payload_add(total: &mut usize, value: usize) -> Result<(), FlowSessionError> {
    *total = total.checked_add(value).ok_or_else(|| {
        FlowSessionError::new(
            FlowSessionErrorKind::LimitExceeded,
            "encoded payload size overflow",
        )
    })?;
    Ok(())
}

fn append_output(
    outputs: &mut Vec<FlowOutput>,
    next_sequence: &mut u64,
    cycle: u64,
    phase: FlowOutputPhase,
    payload: FlowOutputPayload,
) -> Result<(), FlowSessionError> {
    validate_output_payload(&payload)?;
    if outputs.len() >= MAX_BATCH_ITEMS {
        return Err(FlowSessionError::new(
            FlowSessionErrorKind::LimitExceeded,
            "output limit exceeded",
        ));
    }
    let encoded_size = outputs
        .iter()
        .try_fold(flow_output_payload_bytes(&payload), |size, output| {
            size.checked_add(flow_output_payload_bytes(&output.payload))
        })
        .ok_or_else(|| {
            FlowSessionError::new(
                FlowSessionErrorKind::LimitExceeded,
                "encoded output size overflow",
            )
        })?;
    if encoded_size > MAX_ENCODED_PAYLOAD_BYTES {
        return Err(FlowSessionError::new(
            FlowSessionErrorKind::LimitExceeded,
            "encoded output exceeds 4 MiB",
        ));
    }
    let sequence = *next_sequence;
    *next_sequence = next_sequence.checked_add(1).ok_or_else(|| {
        FlowSessionError::new(
            FlowSessionErrorKind::LimitExceeded,
            "output sequence overflow",
        )
    })?;
    outputs.push(FlowOutput {
        sequence,
        cycle,
        phase,
        payload,
    });
    Ok(())
}

fn validate_scalar_value(value: &FlowScalarValue, label: &str) -> Result<(), FlowSessionError> {
    match value {
        FlowScalarValue::String(value) if value.len() > MAX_STRING_BYTES => {
            Err(FlowSessionError::new(
                FlowSessionErrorKind::LimitExceeded,
                format!("{label} string exceeds 1 MiB"),
            ))
        }
        FlowScalarValue::Number(value) if !value.is_finite() => Err(FlowSessionError::new(
            FlowSessionErrorKind::Runtime,
            format!("{label} number is non-finite"),
        )),
        _ => Ok(()),
    }
}

fn validate_output_payload(payload: &FlowOutputPayload) -> Result<(), FlowSessionError> {
    match payload {
        FlowOutputPayload::ReportedEvent {
            name,
            delay_seconds,
            properties,
            ..
        } => {
            if !delay_seconds.is_finite() || *delay_seconds < 0.0 {
                return Err(FlowSessionError::new(
                    FlowSessionErrorKind::Runtime,
                    "reported event delay is invalid",
                ));
            }
            validate_optional_selector(name.as_deref(), "event name")?;
            if properties.len() > MAX_EVENT_PROPERTIES {
                return Err(FlowSessionError::new(
                    FlowSessionErrorKind::LimitExceeded,
                    "event property limit exceeded",
                ));
            }
            for property in properties {
                validate_optional_selector(property.name.as_deref(), "event property name")?;
                validate_scalar_value(&property.value, "event property")?;
            }
        }
        FlowOutputPayload::StateChanged { path, value, .. } => {
            validate_required_id_path(path, "state-change path")?;
            if let Some(value) = value {
                validate_scalar_value(value, "state change")?;
            }
        }
        FlowOutputPayload::HostCommand { name, payload } => {
            validate_required_id_path(name, "host command name")?;
            if payload.len() > MAX_ENCODED_PAYLOAD_BYTES {
                return Err(FlowSessionError::new(
                    FlowSessionErrorKind::LimitExceeded,
                    "host command payload exceeds 4 MiB",
                ));
            }
        }
        FlowOutputPayload::Metadata(metadata) => {
            validate_optional_selector(metadata.name.as_deref(), "player name")?;
        }
        FlowOutputPayload::RuntimeAdvanced { delta_seconds } => {
            if !delta_seconds.is_finite() || *delta_seconds < 0.0 {
                return Err(FlowSessionError::new(
                    FlowSessionErrorKind::Runtime,
                    "runtime advance delta is invalid",
                ));
            }
        }
        FlowOutputPayload::RenderRequested { .. } => {}
    }
    Ok(())
}

fn state_batch_payload_bytes(batch: &FlowStateBatch) -> Result<usize, FlowSessionError> {
    let mut total = 0_usize;
    for item in &batch.new_instances {
        checked_payload_add(&mut total, item.schema_name.len())?;
        checked_payload_add(
            &mut total,
            item.authored_instance_name
                .as_deref()
                .map(str::len)
                .unwrap_or(0),
        )?;
    }
    for mutation in &batch.mutations {
        let (path, value) = match mutation {
            FlowStateMutation::SetInputBool { name, .. }
            | FlowStateMutation::SetInputNumber { name, .. }
            | FlowStateMutation::FireInputTrigger { name } => (name.as_str(), None),
            FlowStateMutation::SetValue { path, value, .. } => (path.as_str(), Some(value)),
            FlowStateMutation::FireTrigger { path, .. }
            | FlowStateMutation::ListInsert { path, .. }
            | FlowStateMutation::ListRemove { path, .. }
            | FlowStateMutation::ListSwap { path, .. }
            | FlowStateMutation::ListMove { path, .. }
            | FlowStateMutation::ListSet { path, .. }
            | FlowStateMutation::ListClear { path, .. } => (path.as_str(), None),
        };
        checked_payload_add(&mut total, path.len())?;
        if let Some(FlowScalarValue::String(value)) = value {
            checked_payload_add(&mut total, value.len())?;
        }
    }
    Ok(total)
}

fn flow_output_payload_bytes(payload: &FlowOutputPayload) -> usize {
    match payload {
        FlowOutputPayload::ReportedEvent {
            name, properties, ..
        } => name.as_deref().map(str::len).unwrap_or(0).saturating_add(
            properties
                .iter()
                .map(|property| {
                    property
                        .name
                        .as_deref()
                        .map(str::len)
                        .unwrap_or(0)
                        .saturating_add(match &property.value {
                            FlowScalarValue::String(value) => value.len(),
                            _ => 16,
                        })
                })
                .sum::<usize>(),
        ),
        FlowOutputPayload::StateChanged { path, value, .. } => {
            path.len().saturating_add(match value {
                Some(FlowScalarValue::String(value)) => value.len(),
                Some(_) => 16,
                None => 0,
            })
        }
        FlowOutputPayload::HostCommand { name, payload } => {
            name.len().saturating_add(payload.len())
        }
        FlowOutputPayload::Metadata(metadata) => {
            metadata.name.as_deref().map(str::len).unwrap_or(0)
        }
        FlowOutputPayload::RenderRequested { .. } | FlowOutputPayload::RuntimeAdvanced { .. } => 16,
    }
}

#[derive(Debug, Clone)]
enum ResolvedMutation {
    SetInputBool {
        name: String,
        value: bool,
    },
    SetInputNumber {
        name: String,
        value: f32,
    },
    FireInputTrigger {
        name: String,
    },
    SetValue {
        instance: FlowInstanceId,
        path: String,
        value: FlowScalarValue,
    },
    FireTrigger {
        instance: FlowInstanceId,
        path: String,
    },
    ListInsert {
        instance: FlowInstanceId,
        path: String,
        index: usize,
        item: FlowInstanceId,
    },
    ListRemove {
        instance: FlowInstanceId,
        path: String,
        index: usize,
    },
    ListSwap {
        instance: FlowInstanceId,
        path: String,
        first: usize,
        second: usize,
    },
    ListMove {
        instance: FlowInstanceId,
        path: String,
        from: usize,
        to: usize,
    },
    ListSet {
        instance: FlowInstanceId,
        path: String,
        index: usize,
        item: FlowInstanceId,
    },
    ListClear {
        instance: FlowInstanceId,
        path: String,
    },
}

fn resolve_instance_ref(
    reference: FlowInstanceRef,
    new_ids: &BTreeMap<u32, FlowInstanceId>,
) -> Result<FlowInstanceId, FlowSessionError> {
    match reference {
        FlowInstanceRef::Existing(id) => Ok(id),
        FlowInstanceRef::New(local_id) => new_ids.get(&local_id).copied().ok_or_else(|| {
            FlowSessionError::new(
                FlowSessionErrorKind::NotFound,
                format!("transaction-local instance {local_id} was not declared"),
            )
        }),
    }
}

fn resolve_mutation(
    mutation: &FlowStateMutation,
    new_ids: &BTreeMap<u32, FlowInstanceId>,
) -> Result<ResolvedMutation, FlowSessionError> {
    Ok(match mutation {
        FlowStateMutation::SetInputBool { name, value } => ResolvedMutation::SetInputBool {
            name: name.clone(),
            value: *value,
        },
        FlowStateMutation::SetInputNumber { name, value } => ResolvedMutation::SetInputNumber {
            name: name.clone(),
            value: *value,
        },
        FlowStateMutation::FireInputTrigger { name } => {
            ResolvedMutation::FireInputTrigger { name: name.clone() }
        }
        FlowStateMutation::SetValue {
            instance,
            path,
            value,
        } => ResolvedMutation::SetValue {
            instance: resolve_instance_ref(*instance, new_ids)?,
            path: path.clone(),
            value: value.clone(),
        },
        FlowStateMutation::FireTrigger { instance, path } => ResolvedMutation::FireTrigger {
            instance: resolve_instance_ref(*instance, new_ids)?,
            path: path.clone(),
        },
        FlowStateMutation::ListInsert {
            instance,
            path,
            index,
            item,
        } => ResolvedMutation::ListInsert {
            instance: resolve_instance_ref(*instance, new_ids)?,
            path: path.clone(),
            index: *index,
            item: resolve_instance_ref(*item, new_ids)?,
        },
        FlowStateMutation::ListRemove {
            instance,
            path,
            index,
        } => ResolvedMutation::ListRemove {
            instance: resolve_instance_ref(*instance, new_ids)?,
            path: path.clone(),
            index: *index,
        },
        FlowStateMutation::ListSwap {
            instance,
            path,
            first,
            second,
        } => ResolvedMutation::ListSwap {
            instance: resolve_instance_ref(*instance, new_ids)?,
            path: path.clone(),
            first: *first,
            second: *second,
        },
        FlowStateMutation::ListMove {
            instance,
            path,
            from,
            to,
        } => ResolvedMutation::ListMove {
            instance: resolve_instance_ref(*instance, new_ids)?,
            path: path.clone(),
            from: *from,
            to: *to,
        },
        FlowStateMutation::ListSet {
            instance,
            path,
            index,
            item,
        } => ResolvedMutation::ListSet {
            instance: resolve_instance_ref(*instance, new_ids)?,
            path: path.clone(),
            index: *index,
            item: resolve_instance_ref(*item, new_ids)?,
        },
        FlowStateMutation::ListClear { instance, path } => ResolvedMutation::ListClear {
            instance: resolve_instance_ref(*instance, new_ids)?,
            path: path.clone(),
        },
    })
}

fn mutation_instance_ids(mutation: &ResolvedMutation) -> Vec<FlowInstanceId> {
    match mutation {
        ResolvedMutation::SetValue { instance, .. }
        | ResolvedMutation::FireTrigger { instance, .. }
        | ResolvedMutation::ListRemove { instance, .. }
        | ResolvedMutation::ListSwap { instance, .. }
        | ResolvedMutation::ListMove { instance, .. }
        | ResolvedMutation::ListClear { instance, .. } => vec![*instance],
        ResolvedMutation::ListInsert { instance, item, .. }
        | ResolvedMutation::ListSet { instance, item, .. } => vec![*instance, *item],
        ResolvedMutation::SetInputBool { .. }
        | ResolvedMutation::SetInputNumber { .. }
        | ResolvedMutation::FireInputTrigger { .. } => Vec::new(),
    }
}

fn instantiate_named_view_model(
    file: &Arc<File>,
    schema_name: &str,
    authored_name: Option<&str>,
) -> Result<ViewModelInstance, FlowSessionError> {
    let view_model_index = file
        .graph()
        .view_models
        .iter()
        .position(|schema| schema.name.as_deref() == Some(schema_name))
        .ok_or_else(|| {
            FlowSessionError::new(
                FlowSessionErrorKind::NotFound,
                format!("view-model schema '{schema_name}' was not found"),
            )
        })?;
    let raw = if let Some(name) = authored_name {
        let reference = file
            .runtime()
            .view_model_instance_named(view_model_index, name)
            .ok_or_else(|| {
                FlowSessionError::new(
                    FlowSessionErrorKind::NotFound,
                    format!("authored instance '{name}' was not found in schema '{schema_name}'"),
                )
            })?;
        RuntimeOwnedViewModelInstance::from_instance(
            file.runtime(),
            view_model_index,
            reference.instance_index,
        )
    } else {
        RuntimeOwnedViewModelInstance::new(file.runtime(), view_model_index)
    }
    .ok_or_else(|| {
        FlowSessionError::new(
            FlowSessionErrorKind::Runtime,
            "view-model instance could not be created",
        )
    })?;
    Ok(ViewModelInstance {
        raw: RuntimeOwnedViewModelHandle::new(raw),
    })
}

fn validate_required_id_path(value: &str, label: &str) -> Result<(), FlowSessionError> {
    if value.is_empty() {
        return Err(FlowSessionError::new(
            FlowSessionErrorKind::InvalidArgument,
            format!("{label} must not be empty"),
        ));
    }
    if value.len() > MAX_ID_PATH_BYTES {
        return Err(FlowSessionError::new(
            FlowSessionErrorKind::LimitExceeded,
            format!("{label} exceeds {MAX_ID_PATH_BYTES} UTF-8 bytes"),
        ));
    }
    if value.split('/').any(str::is_empty) {
        return Err(FlowSessionError::new(
            FlowSessionErrorKind::InvalidArgument,
            format!("{label} contains an empty path segment"),
        ));
    }
    Ok(())
}

fn prevalidate_and_apply_mutation(
    machine: Option<&mut StateMachineInstance>,
    instances: &BTreeMap<FlowInstanceId, ViewModelInstance>,
    mutation: &ResolvedMutation,
) -> Result<(), FlowSessionError> {
    match mutation {
        ResolvedMutation::SetInputBool { name, value } => {
            validate_required_id_path(name, "input name")?;
            let machine = machine.ok_or_else(|| {
                FlowSessionError::new(
                    FlowSessionErrorKind::NotFound,
                    "session has no state-machine player",
                )
            })?;
            let index = machine.input_index_named(name).ok_or_else(|| {
                FlowSessionError::new(
                    FlowSessionErrorKind::NotFound,
                    format!("input '{name}' was not found"),
                )
            })?;
            if machine.input(index).map(|input| input.kind())
                != Some(crate::StateMachineInputKind::Bool)
            {
                return Err(FlowSessionError::new(
                    FlowSessionErrorKind::InvalidArgument,
                    format!("input '{name}' is not boolean"),
                ));
            }
            let _ = machine.set_bool(index, *value);
        }
        ResolvedMutation::SetInputNumber { name, value } => {
            validate_required_id_path(name, "input name")?;
            if !value.is_finite() {
                return Err(FlowSessionError::new(
                    FlowSessionErrorKind::InvalidArgument,
                    "numeric state values must be finite",
                ));
            }
            let machine = machine.ok_or_else(|| {
                FlowSessionError::new(
                    FlowSessionErrorKind::NotFound,
                    "session has no state-machine player",
                )
            })?;
            let index = machine.input_index_named(name).ok_or_else(|| {
                FlowSessionError::new(
                    FlowSessionErrorKind::NotFound,
                    format!("input '{name}' was not found"),
                )
            })?;
            if machine.input(index).map(|input| input.kind())
                != Some(crate::StateMachineInputKind::Number)
            {
                return Err(FlowSessionError::new(
                    FlowSessionErrorKind::InvalidArgument,
                    format!("input '{name}' is not numeric"),
                ));
            }
            let _ = machine.set_number(index, *value);
        }
        ResolvedMutation::FireInputTrigger { name } => {
            validate_required_id_path(name, "input name")?;
            let machine = machine.ok_or_else(|| {
                FlowSessionError::new(
                    FlowSessionErrorKind::NotFound,
                    "session has no state-machine player",
                )
            })?;
            let index = machine.input_index_named(name).ok_or_else(|| {
                FlowSessionError::new(
                    FlowSessionErrorKind::NotFound,
                    format!("input '{name}' was not found"),
                )
            })?;
            if machine.input(index).map(|input| input.kind())
                != Some(crate::StateMachineInputKind::Trigger)
            {
                return Err(FlowSessionError::new(
                    FlowSessionErrorKind::InvalidArgument,
                    format!("input '{name}' is not a trigger"),
                ));
            }
            let _ = machine.fire_trigger(index);
        }
        _ => apply_view_model_mutation(instances, mutation)?,
    }
    Ok(())
}

fn instance(
    instances: &BTreeMap<FlowInstanceId, ViewModelInstance>,
    id: FlowInstanceId,
) -> Result<&ViewModelInstance, FlowSessionError> {
    instances.get(&id).ok_or_else(|| {
        FlowSessionError::new(
            FlowSessionErrorKind::NotFound,
            format!("instance {} was not found", id.get()),
        )
    })
}

fn apply_view_model_mutation(
    instances: &BTreeMap<FlowInstanceId, ViewModelInstance>,
    mutation: &ResolvedMutation,
) -> Result<(), FlowSessionError> {
    match mutation {
        ResolvedMutation::SetValue {
            instance: id,
            path,
            value,
        } => {
            validate_required_id_path(path, "property path")?;
            let view_model = instance(instances, *id)?;
            let mut raw = view_model.raw_mut();
            match value {
                FlowScalarValue::Null => {
                    return Err(FlowSessionError::new(
                        FlowSessionErrorKind::InvalidArgument,
                        "null cannot be assigned to a scalar property",
                    ));
                }
                FlowScalarValue::String(value) => {
                    if value.len() > MAX_STRING_BYTES {
                        return Err(FlowSessionError::new(
                            FlowSessionErrorKind::LimitExceeded,
                            "string value exceeds 1 MiB",
                        ));
                    }
                    if raw
                        .string_source_handle_by_property_name_path(path)
                        .is_none()
                    {
                        return Err(FlowSessionError::new(
                            FlowSessionErrorKind::NotFound,
                            format!("string property '{path}' was not found"),
                        ));
                    }
                    let _ = raw.set_string_by_property_name_path(path, value.as_bytes());
                }
                FlowScalarValue::Number(value) => {
                    if !value.is_finite() {
                        return Err(FlowSessionError::new(
                            FlowSessionErrorKind::InvalidArgument,
                            "numeric state values must be finite",
                        ));
                    }
                    if raw
                        .number_source_handle_by_property_name_path(path)
                        .is_none()
                    {
                        return Err(FlowSessionError::new(
                            FlowSessionErrorKind::NotFound,
                            format!("number property '{path}' was not found"),
                        ));
                    }
                    let _ = raw.set_number_by_property_name_path(path, *value);
                }
                FlowScalarValue::Bool(value) => {
                    if raw
                        .boolean_source_handle_by_property_name_path(path)
                        .is_none()
                    {
                        return Err(FlowSessionError::new(
                            FlowSessionErrorKind::NotFound,
                            format!("boolean property '{path}' was not found"),
                        ));
                    }
                    let _ = raw.set_boolean_by_property_name_path(path, *value);
                }
                FlowScalarValue::Enum(value) => {
                    if raw.enum_source_handle_by_property_name_path(path).is_none() {
                        return Err(FlowSessionError::new(
                            FlowSessionErrorKind::NotFound,
                            format!("enum property '{path}' was not found"),
                        ));
                    }
                    let _ = raw.set_enum_by_property_name_path(path, *value);
                }
                FlowScalarValue::Color(value) => {
                    if raw
                        .color_source_handle_by_property_name_path(path)
                        .is_none()
                    {
                        return Err(FlowSessionError::new(
                            FlowSessionErrorKind::NotFound,
                            format!("color property '{path}' was not found"),
                        ));
                    }
                    let _ = raw.set_color_by_property_name_path(path, *value);
                }
                FlowScalarValue::Image(value) => {
                    if raw
                        .asset_source_handle_by_property_name_path(path)
                        .is_none()
                    {
                        return Err(FlowSessionError::new(
                            FlowSessionErrorKind::NotFound,
                            format!("image property '{path}' was not found"),
                        ));
                    }
                    let _ = raw.set_asset_by_property_name_path(path, *value);
                }
                FlowScalarValue::Trigger(_) => {
                    return Err(FlowSessionError::new(
                        FlowSessionErrorKind::InvalidArgument,
                        "trigger counters are advanced with FireTrigger",
                    ));
                }
            }
        }
        ResolvedMutation::FireTrigger { instance: id, path } => {
            validate_required_id_path(path, "property path")?;
            let view_model = instance(instances, *id)?;
            let mut raw = view_model.raw_mut();
            if raw
                .trigger_source_handle_by_property_name_path(path)
                .is_none()
            {
                return Err(FlowSessionError::new(
                    FlowSessionErrorKind::NotFound,
                    format!("trigger property '{path}' was not found"),
                ));
            }
            let current = raw.trigger_value_by_property_name_path(path).unwrap_or(0);
            let next = current.checked_add(1).ok_or_else(|| {
                FlowSessionError::new(
                    FlowSessionErrorKind::LimitExceeded,
                    "trigger counter overflow",
                )
            })?;
            let _ = raw.set_trigger_by_property_name_path(path, next);
        }
        ResolvedMutation::ListInsert {
            instance: id,
            path,
            index,
            item,
        } => {
            validate_required_id_path(path, "list path")?;
            let owner = instance(instances, *id)?;
            let item = instance(instances, *item)?;
            let count = owner
                .handle()
                .list_item_count_by_property_name_path(path)
                .ok_or_else(|| {
                    FlowSessionError::new(
                        FlowSessionErrorKind::NotFound,
                        format!("list property '{path}' was not found"),
                    )
                })?;
            if count >= MAX_LIST_ITEMS || *index > count {
                return Err(FlowSessionError::new(
                    FlowSessionErrorKind::LimitExceeded,
                    "list insert index or capacity is out of range",
                ));
            }
            if !owner
                .handle()
                .insert_list_item_by_property_name_path(path, *index, item.handle())
            {
                return Err(FlowSessionError::new(
                    FlowSessionErrorKind::Conflict,
                    "list insert would create a cycle",
                ));
            }
        }
        ResolvedMutation::ListRemove {
            instance: id,
            path,
            index,
        } => {
            validate_required_id_path(path, "list path")?;
            let owner = instance(instances, *id)?;
            let count = owner
                .handle()
                .list_item_count_by_property_name_path(path)
                .ok_or_else(|| {
                    FlowSessionError::new(
                        FlowSessionErrorKind::NotFound,
                        format!("list property '{path}' was not found"),
                    )
                })?;
            if *index >= count
                || !owner
                    .handle()
                    .remove_list_item_by_property_name_path(path, *index)
            {
                return Err(FlowSessionError::new(
                    FlowSessionErrorKind::InvalidArgument,
                    "list remove index is out of range",
                ));
            }
        }
        ResolvedMutation::ListSwap {
            instance: id,
            path,
            first,
            second,
        } => {
            validate_required_id_path(path, "list path")?;
            let owner = instance(instances, *id)?;
            let count = owner
                .handle()
                .list_item_count_by_property_name_path(path)
                .ok_or_else(|| {
                    FlowSessionError::new(
                        FlowSessionErrorKind::NotFound,
                        format!("list property '{path}' was not found"),
                    )
                })?;
            if *first >= count || *second >= count {
                return Err(FlowSessionError::new(
                    FlowSessionErrorKind::InvalidArgument,
                    "list swap index is out of range",
                ));
            }
            if first != second {
                let _ = owner
                    .handle()
                    .swap_list_items_by_property_name_path(path, *first, *second);
            }
        }
        ResolvedMutation::ListMove {
            instance: id,
            path,
            from,
            to,
        } => {
            validate_required_id_path(path, "list path")?;
            let owner = instance(instances, *id)?;
            let count = owner
                .handle()
                .list_item_count_by_property_name_path(path)
                .ok_or_else(|| {
                    FlowSessionError::new(
                        FlowSessionErrorKind::NotFound,
                        format!("list property '{path}' was not found"),
                    )
                })?;
            if *from >= count || *to >= count {
                return Err(FlowSessionError::new(
                    FlowSessionErrorKind::InvalidArgument,
                    "list move index is out of range",
                ));
            }
            if from != to {
                let _ = owner
                    .handle()
                    .move_list_item_by_property_name_path(path, *from, *to);
            }
        }
        ResolvedMutation::ListSet {
            instance: id,
            path,
            index,
            item,
        } => {
            validate_required_id_path(path, "list path")?;
            let owner = instance(instances, *id)?;
            let item = instance(instances, *item)?;
            let count = owner
                .handle()
                .list_item_count_by_property_name_path(path)
                .ok_or_else(|| {
                    FlowSessionError::new(
                        FlowSessionErrorKind::NotFound,
                        format!("list property '{path}' was not found"),
                    )
                })?;
            if *index >= count {
                return Err(FlowSessionError::new(
                    FlowSessionErrorKind::InvalidArgument,
                    "list set index is out of range",
                ));
            }
            if !owner
                .handle()
                .set_list_item_by_property_name_path(path, *index, item.handle())
            {
                return Err(FlowSessionError::new(
                    FlowSessionErrorKind::Conflict,
                    "list set would create a cycle or no change",
                ));
            }
        }
        ResolvedMutation::ListClear { instance: id, path } => {
            validate_required_id_path(path, "list path")?;
            let owner = instance(instances, *id)?;
            if owner
                .handle()
                .list_item_count_by_property_name_path(path)
                .is_none()
            {
                return Err(FlowSessionError::new(
                    FlowSessionErrorKind::NotFound,
                    format!("list property '{path}' was not found"),
                ));
            }
            let _ = owner.handle().clear_list_items_by_property_name_path(path);
        }
        ResolvedMutation::SetInputBool { .. }
        | ResolvedMutation::SetInputNumber { .. }
        | ResolvedMutation::FireInputTrigger { .. } => {}
    }
    Ok(())
}

fn mutation_echo(
    mutation: &ResolvedMutation,
) -> Option<(Option<FlowInstanceId>, String, Option<FlowScalarValue>)> {
    match mutation {
        ResolvedMutation::SetInputBool { name, value } => {
            Some((None, name.clone(), Some(FlowScalarValue::Bool(*value))))
        }
        ResolvedMutation::SetInputNumber { name, value } => {
            Some((None, name.clone(), Some(FlowScalarValue::Number(*value))))
        }
        ResolvedMutation::FireInputTrigger { name } => Some((None, name.clone(), None)),
        ResolvedMutation::SetValue {
            instance,
            path,
            value,
        } => Some((Some(*instance), path.clone(), Some(value.clone()))),
        ResolvedMutation::FireTrigger { instance, path } => {
            Some((Some(*instance), path.clone(), None))
        }
        ResolvedMutation::ListInsert { instance, path, .. }
        | ResolvedMutation::ListRemove { instance, path, .. }
        | ResolvedMutation::ListSwap { instance, path, .. }
        | ResolvedMutation::ListMove { instance, path, .. }
        | ResolvedMutation::ListSet { instance, path, .. }
        | ResolvedMutation::ListClear { instance, path } => {
            Some((Some(*instance), path.clone(), None))
        }
    }
}

fn diff_value_arenas(
    before: &FlowValueArena,
    after: &FlowValueArena,
) -> Result<Vec<(FlowInstanceId, String, Option<FlowScalarValue>)>, FlowSessionError> {
    let mut changes = Vec::new();
    for (instance_id, after_root) in &after.roots {
        let before_root = before
            .roots
            .iter()
            .find(|(id, _)| id == instance_id)
            .map(|(_, root)| *root);
        diff_value_node(
            before,
            before_root,
            after,
            Some(*after_root),
            *instance_id,
            "",
            &mut changes,
        )?;
    }
    Ok(changes)
}

fn diff_value_node(
    before: &FlowValueArena,
    before_id: Option<FlowValueId>,
    after: &FlowValueArena,
    after_id: Option<FlowValueId>,
    instance_id: FlowInstanceId,
    path: &str,
    changes: &mut Vec<(FlowInstanceId, String, Option<FlowScalarValue>)>,
) -> Result<(), FlowSessionError> {
    let before_value = before_id
        .and_then(|id| before.nodes.iter().find(|node| node.id == id))
        .map(|node| &node.value);
    let after_value = after_id
        .and_then(|id| after.nodes.iter().find(|node| node.id == id))
        .map(|node| &node.value);
    match (before_value, after_value) {
        (Some(FlowValue::ViewModel(before_edges)), Some(FlowValue::ViewModel(after_edges)))
        | (Some(FlowValue::Object(before_edges)), Some(FlowValue::Object(after_edges))) => {
            for (name, after_child) in after_edges {
                let child_path = if path.is_empty() {
                    name.clone()
                } else {
                    format!("{path}/{name}")
                };
                validate_required_id_path(&child_path, "state-change path")?;
                let before_child = before_edges
                    .iter()
                    .find(|(candidate, _)| candidate == name)
                    .map(|(_, id)| *id);
                diff_value_node(
                    before,
                    before_child,
                    after,
                    Some(*after_child),
                    instance_id,
                    &child_path,
                    changes,
                )?;
            }
        }
        (Some(FlowValue::List(before_items)), Some(FlowValue::List(after_items))) => {
            let before_identity = before_items
                .iter()
                .map(|node| value_instance_id_for_node(before, *node))
                .collect::<Vec<_>>();
            let after_identity = after_items
                .iter()
                .map(|node| value_instance_id_for_node(after, *node))
                .collect::<Vec<_>>();
            if before_items.len() != after_items.len() || before_identity != after_identity {
                validate_required_id_path(path, "state-change path")?;
                changes.push((instance_id, path.to_owned(), None));
            }
            for (index, after_child) in after_items.iter().enumerate() {
                let child_path = format!("{path}/{index}");
                validate_required_id_path(&child_path, "state-change path")?;
                let before_child = before_items.get(index).copied();
                diff_value_node(
                    before,
                    before_child,
                    after,
                    Some(*after_child),
                    instance_id,
                    &child_path,
                    changes,
                )?;
            }
        }
        (_, Some(value)) if before_value != Some(value) => {
            validate_required_id_path(path, "state-change path")?;
            changes.push((
                instance_id,
                path.to_owned(),
                flow_scalar_from_arena_value(value),
            ));
        }
        _ => {}
    }
    Ok(())
}

fn value_instance_id_for_node(arena: &FlowValueArena, node: FlowValueId) -> Option<FlowInstanceId> {
    arena
        .roots
        .iter()
        .find_map(|(instance_id, root)| (*root == node).then_some(*instance_id))
}

fn flow_scalar_from_arena_value(value: &FlowValue) -> Option<FlowScalarValue> {
    match value {
        FlowValue::Null => Some(FlowScalarValue::Null),
        FlowValue::String(value) => Some(FlowScalarValue::String(value.clone())),
        FlowValue::Number(value) => Some(FlowScalarValue::Number(*value)),
        FlowValue::Bool(value) => Some(FlowScalarValue::Bool(*value)),
        FlowValue::Enum(value) => Some(FlowScalarValue::Enum(*value)),
        FlowValue::Color(value) => Some(FlowScalarValue::Color(*value)),
        FlowValue::Image(value) => Some(FlowScalarValue::Image(*value)),
        FlowValue::Object(_) | FlowValue::ViewModel(_) | FlowValue::List(_) => None,
    }
}

fn validate_optional_selector(value: Option<&str>, label: &str) -> Result<(), FlowSessionError> {
    if value.is_some_and(str::is_empty) {
        return Err(FlowSessionError::new(
            FlowSessionErrorKind::InvalidArgument,
            format!("{label} must not be empty"),
        ));
    }
    if value.is_some_and(|value| value.len() > MAX_ID_PATH_BYTES) {
        return Err(FlowSessionError::new(
            FlowSessionErrorKind::LimitExceeded,
            format!("{label} exceeds {MAX_ID_PATH_BYTES} UTF-8 bytes"),
        ));
    }
    Ok(())
}

fn select_player(
    artboard: crate::Artboard<'_>,
    explicit_name: Option<&str>,
) -> Result<(FlowPlayerMetadata, Option<usize>), FlowSessionError> {
    if let Some(name) = explicit_name {
        let index = (0..artboard.state_machine_count())
            .find(|index| artboard.state_machine_name(*index) == Some(name))
            .ok_or_else(|| {
                FlowSessionError::new(
                    FlowSessionErrorKind::NotFound,
                    format!("state machine '{name}' was not found"),
                )
            })?;
        return Ok((
            FlowPlayerMetadata {
                kind: FlowPlayerKind::StateMachine,
                selection: FlowPlayerSelection::ExplicitStateMachine,
                index: Some(index),
                name: Some(name.to_owned()),
            },
            Some(index),
        ));
    }

    if let Some(index) = artboard.default_state_machine_index() {
        return Ok((
            FlowPlayerMetadata {
                kind: FlowPlayerKind::StateMachine,
                selection: FlowPlayerSelection::AuthoredDefaultStateMachine,
                index: Some(index),
                name: artboard.state_machine_name(index).map(ToOwned::to_owned),
            },
            Some(index),
        ));
    }
    if artboard.state_machine_count() > 0 {
        return Ok((
            FlowPlayerMetadata {
                kind: FlowPlayerKind::StateMachine,
                selection: FlowPlayerSelection::FirstStateMachine,
                index: Some(0),
                name: artboard.state_machine_name(0).map(ToOwned::to_owned),
            },
            Some(0),
        ));
    }
    if let Some(animation) = artboard.graph().animations.first() {
        return Ok((
            FlowPlayerMetadata {
                kind: FlowPlayerKind::LinearAnimation,
                selection: FlowPlayerSelection::FirstAnimation,
                index: Some(0),
                name: animation.name.clone(),
            },
            Some(0),
        ));
    }
    Ok((
        FlowPlayerMetadata {
            kind: FlowPlayerKind::Static,
            selection: FlowPlayerSelection::Static,
            index: None,
            name: None,
        },
        None,
    ))
}

fn build_catalog(
    file: &File,
    root_selection: Option<(usize, Option<usize>)>,
) -> Result<FlowCatalog, FlowSessionError> {
    let root_instance_id = root_selection.map(|_| FlowInstanceId(1));
    let mut templates = Vec::new();
    let schemas = file
        .graph()
        .view_models
        .iter()
        .enumerate()
        .map(|(view_model_index, schema)| {
            let schema_name = schema
                .name
                .clone()
                .unwrap_or_else(|| format!("viewModel{view_model_index}"));
            for (authored_index, instance) in schema.instances.iter().enumerate() {
                templates.push(FlowInstanceTemplate {
                    schema_name: schema_name.clone(),
                    authored_name: instance.name.clone(),
                    authored_index,
                });
            }
            FlowSchema {
                name: schema_name,
                properties: schema
                    .properties
                    .iter()
                    .filter_map(|property| {
                        Some(FlowPropertySchema {
                            name: property.name.clone()?,
                            value_type: flow_value_type_for_property(property.type_name),
                        })
                    })
                    .collect(),
            }
        })
        .collect();

    let instances = root_selection
        .map(|(view_model_index, authored_index)| {
            let schema_name = file
                .graph()
                .view_models
                .get(view_model_index)
                .and_then(|schema| schema.name.clone())
                .unwrap_or_else(|| format!("viewModel{view_model_index}"));
            let authored_name = authored_index.and_then(|index| {
                file.graph()
                    .view_models
                    .get(view_model_index)
                    .and_then(|schema| schema.instances.get(index))
                    .and_then(|instance| instance.name.clone())
            });
            vec![FlowInstanceMetadata {
                id: FlowInstanceId(1),
                schema_name,
                authored_name,
                is_root: true,
            }]
        })
        .unwrap_or_default();
    let catalog = FlowCatalog {
        schemas,
        templates,
        instances,
        root_instance_id,
    };
    validate_catalog(&catalog)?;
    Ok(catalog)
}

fn validate_catalog(catalog: &FlowCatalog) -> Result<(), FlowSessionError> {
    if catalog.schemas.len() > MAX_BATCH_ITEMS
        || catalog.templates.len() > MAX_BATCH_ITEMS
        || catalog.instances.len() > MAX_INSTANCES
    {
        return Err(FlowSessionError::new(
            FlowSessionErrorKind::LimitExceeded,
            "catalog collection limit exceeded",
        ));
    }
    let mut property_count = 0_usize;
    for schema in &catalog.schemas {
        validate_required_id_path(&schema.name, "schema name")?;
        property_count = property_count
            .checked_add(schema.properties.len())
            .ok_or_else(|| {
                FlowSessionError::new(
                    FlowSessionErrorKind::LimitExceeded,
                    "catalog property count overflow",
                )
            })?;
        if schema.properties.len() > MAX_BATCH_ITEMS || property_count > MAX_BATCH_ITEMS {
            return Err(FlowSessionError::new(
                FlowSessionErrorKind::LimitExceeded,
                "catalog property limit exceeded",
            ));
        }
        for property in &schema.properties {
            validate_required_id_path(&property.name, "property name")?;
        }
    }
    for template in &catalog.templates {
        validate_required_id_path(&template.schema_name, "schema name")?;
        if template
            .authored_name
            .as_ref()
            .is_some_and(|name| name.len() > MAX_ID_PATH_BYTES)
        {
            return Err(FlowSessionError::new(
                FlowSessionErrorKind::LimitExceeded,
                "authored instance name exceeds identifier limit",
            ));
        }
    }
    for instance in &catalog.instances {
        validate_required_id_path(&instance.schema_name, "schema name")?;
        if instance
            .authored_name
            .as_ref()
            .is_some_and(|name| name.len() > MAX_ID_PATH_BYTES)
        {
            return Err(FlowSessionError::new(
                FlowSessionErrorKind::LimitExceeded,
                "authored instance name exceeds identifier limit",
            ));
        }
    }
    if catalog_payload_bytes(catalog)? > MAX_ENCODED_PAYLOAD_BYTES {
        return Err(FlowSessionError::new(
            FlowSessionErrorKind::LimitExceeded,
            "encoded catalog exceeds 4 MiB",
        ));
    }
    Ok(())
}

fn catalog_payload_bytes(catalog: &FlowCatalog) -> Result<usize, FlowSessionError> {
    let mut total = 32_usize;
    for schema in &catalog.schemas {
        checked_payload_add(&mut total, 16)?;
        checked_payload_add(&mut total, schema.name.len())?;
        for property in &schema.properties {
            checked_payload_add(&mut total, 16)?;
            checked_payload_add(&mut total, property.name.len())?;
        }
    }
    for template in &catalog.templates {
        checked_payload_add(&mut total, 24)?;
        checked_payload_add(&mut total, template.schema_name.len())?;
        checked_payload_add(
            &mut total,
            template.authored_name.as_deref().map(str::len).unwrap_or(0),
        )?;
    }
    for instance in &catalog.instances {
        checked_payload_add(&mut total, 32)?;
        checked_payload_add(&mut total, instance.schema_name.len())?;
        checked_payload_add(
            &mut total,
            instance.authored_name.as_deref().map(str::len).unwrap_or(0),
        )?;
    }
    Ok(total)
}

fn value_arena_payload_bytes(arena: &FlowValueArena) -> Result<usize, FlowSessionError> {
    let mut total = 0_usize;
    checked_payload_add(&mut total, arena.roots.len().saturating_mul(16))?;
    checked_payload_add(&mut total, arena.nodes.len().saturating_mul(16))?;
    for node in &arena.nodes {
        match &node.value {
            FlowValue::String(value) => checked_payload_add(&mut total, value.len())?,
            FlowValue::Object(edges) | FlowValue::ViewModel(edges) => {
                for (name, _) in edges {
                    checked_payload_add(&mut total, 4)?;
                    checked_payload_add(&mut total, name.len())?;
                }
            }
            FlowValue::List(items) => {
                checked_payload_add(&mut total, items.len().saturating_mul(4))?;
            }
            FlowValue::Null
            | FlowValue::Number(_)
            | FlowValue::Bool(_)
            | FlowValue::Enum(_)
            | FlowValue::Color(_)
            | FlowValue::Image(_) => {}
        }
    }
    Ok(total)
}

fn validate_bootstrap_payload(bootstrap: &FlowBootstrap) -> Result<(), FlowSessionError> {
    validate_catalog(&bootstrap.catalog)?;
    validate_optional_selector(bootstrap.artboard_name.as_deref(), "artboard name")?;
    validate_optional_selector(bootstrap.player.name.as_deref(), "player name")?;
    let bounds = bootstrap.bounds;
    if !bounds.x.is_finite()
        || !bounds.y.is_finite()
        || !bounds.width.is_finite()
        || !bounds.height.is_finite()
        || bounds.width <= 0.0
        || bounds.height <= 0.0
        || !(bounds.x + bounds.width).is_finite()
        || !(bounds.y + bounds.height).is_finite()
    {
        return Err(FlowSessionError::new(
            FlowSessionErrorKind::Runtime,
            "artboard bounds are invalid",
        ));
    }
    if bootstrap.values.nodes.len() > MAX_VALUE_NODES {
        return Err(FlowSessionError::new(
            FlowSessionErrorKind::LimitExceeded,
            "value arena node limit exceeded",
        ));
    }
    let mut total = catalog_payload_bytes(&bootstrap.catalog)?;
    checked_payload_add(&mut total, 64)?;
    checked_payload_add(
        &mut total,
        bootstrap
            .artboard_name
            .as_deref()
            .map(str::len)
            .unwrap_or(0),
    )?;
    checked_payload_add(
        &mut total,
        bootstrap.player.name.as_deref().map(str::len).unwrap_or(0),
    )?;
    checked_payload_add(&mut total, value_arena_payload_bytes(&bootstrap.values)?)?;
    if total > MAX_ENCODED_PAYLOAD_BYTES {
        return Err(FlowSessionError::new(
            FlowSessionErrorKind::LimitExceeded,
            "encoded bootstrap exceeds 4 MiB",
        ));
    }
    Ok(())
}

fn flow_value_type_for_property(type_name: &str) -> FlowValueType {
    match type_name {
        "ViewModelPropertyString" => FlowValueType::String,
        "ViewModelPropertyNumber" | "ViewModelPropertyInteger" => FlowValueType::Number,
        "ViewModelPropertyBoolean" => FlowValueType::Bool,
        "ViewModelPropertyEnum"
        | "ViewModelPropertyEnumCustom"
        | "ViewModelPropertyEnumSystem"
        | "ViewModelPropertySymbolListIndex"
        | "ViewModelPropertyArtboard" => FlowValueType::Enum,
        "ViewModelPropertyColor" => FlowValueType::Color,
        "ViewModelPropertyAsset" | "ViewModelPropertyAssetImage" => FlowValueType::Image,
        "ViewModelPropertyList" => FlowValueType::List,
        "ViewModelPropertyViewModel" => FlowValueType::ViewModel,
        "ViewModelPropertyTrigger" => FlowValueType::Trigger,
        _ => FlowValueType::Null,
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use crate::File;

    const FIXTURE: &[u8] = include_bytes!("../../../fixtures/graph/dependency_test.riv");
    const SMI_FIXTURE: &[u8] = include_bytes!("../../../fixtures/animation/smi_test.riv");

    fn smi_session() -> FlowSession {
        let file = Arc::new(File::import(SMI_FIXTURE).expect("import SMI fixture"));
        FlowSession::create(
            file,
            FlowSessionConfig {
                artboard_name: Some("artboard to nest".to_owned()),
                player_name: Some("State Machine 1".to_owned()),
            },
        )
        .expect("create SMI session")
        .0
    }

    fn external_fixture(relative: &str) -> Vec<u8> {
        let root = std::env::var_os("RIVE_RUNTIME_DIR")
            .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
        std::fs::read(
            std::path::PathBuf::from(root)
                .join("tests/unit_tests/assets")
                .join(relative),
        )
        .expect("read external fixture")
    }

    fn arena_number(
        arena: &FlowValueArena,
        instance_id: FlowInstanceId,
        name: &str,
    ) -> Option<f32> {
        let root = arena.roots.iter().find(|(id, _)| *id == instance_id)?.1;
        let root_node = arena.nodes.iter().find(|node| node.id == root)?;
        let FlowValue::ViewModel(edges) = &root_node.value else {
            return None;
        };
        let value_id = edges.iter().find(|(property, _)| property == name)?.1;
        let value = arena.nodes.iter().find(|node| node.id == value_id)?;
        let FlowValue::Number(value) = value.value else {
            return None;
        };
        Some(value)
    }

    fn arena_list_len(
        arena: &FlowValueArena,
        instance_id: FlowInstanceId,
        name: &str,
    ) -> Option<usize> {
        let root = arena.roots.iter().find(|(id, _)| *id == instance_id)?.1;
        let root_node = arena.nodes.iter().find(|node| node.id == root)?;
        let FlowValue::ViewModel(edges) = &root_node.value else {
            return None;
        };
        let value_id = edges.iter().find(|(property, _)| property == name)?.1;
        let value = arena.nodes.iter().find(|node| node.id == value_id)?;
        let FlowValue::List(items) = &value.value else {
            return None;
        };
        Some(items.len())
    }

    fn arena_list_items(
        arena: &FlowValueArena,
        instance_id: FlowInstanceId,
        name: &str,
    ) -> Option<Vec<FlowValueId>> {
        let root = arena.roots.iter().find(|(id, _)| *id == instance_id)?.1;
        let root_node = arena.nodes.iter().find(|node| node.id == root)?;
        let FlowValue::ViewModel(edges) = &root_node.value else {
            return None;
        };
        let value_id = edges.iter().find(|(property, _)| property == name)?.1;
        let value = arena.nodes.iter().find(|node| node.id == value_id)?;
        let FlowValue::List(items) = &value.value else {
            return None;
        };
        Some(items.clone())
    }

    fn input_value(session: &mut FlowSession, name: &str) -> FlowScalarValue {
        session
            .perform(FlowOperation::Query(FlowQuery::PlayerInputs))
            .expect("query inputs")
            .player_inputs
            .expect("input snapshot")
            .into_iter()
            .find(|input| input.name.as_deref() == Some(name))
            .expect("named input")
            .value
    }

    #[test]
    fn explicit_missing_artboard_is_a_typed_not_found_without_fallback() {
        let file = Arc::new(File::import(FIXTURE).expect("import fixture"));

        let error = FlowSession::create(
            file,
            FlowSessionConfig {
                artboard_name: Some("definitely-missing".to_owned()),
                player_name: None,
            },
        )
        .expect_err("an explicit missing artboard must not fall back");

        assert_eq!(error.kind(), FlowSessionErrorKind::NotFound);
    }

    #[test]
    fn bootstrap_query_returns_creation_snapshot_without_ordered_outputs() {
        let file = Arc::new(File::import(FIXTURE).expect("import fixture"));
        let (mut session, bootstrap) =
            FlowSession::create(file, FlowSessionConfig::default()).expect("create session");

        let result = session
            .perform(FlowOperation::Query(FlowQuery::Bootstrap))
            .expect("query bootstrap");

        assert!(result.outputs.is_empty());
        assert_eq!(result.snapshot, Some(bootstrap));
        assert!(!result.dirty);
    }

    #[test]
    fn bootstrap_query_remains_the_immutable_creation_snapshot() {
        let file = Arc::new(
            File::import(&external_fixture("data_binding_test_2.riv"))
                .expect("import data-bind fixture"),
        );
        let (mut session, bootstrap) = FlowSession::create(file, FlowSessionConfig::default())
            .expect("create data-bind session");
        let root = bootstrap.catalog.root_instance_id.expect("root id");

        session
            .perform(FlowOperation::StateBatch(FlowStateBatch {
                host_mutation_id: None,
                mutations: vec![FlowStateMutation::SetValue {
                    instance: FlowInstanceRef::Existing(root),
                    path: "num".to_owned(),
                    value: FlowScalarValue::Number(999.0),
                }],
                new_instances: Vec::new(),
            }))
            .expect("mutate live state");

        let queried = session
            .perform(FlowOperation::Query(FlowQuery::Bootstrap))
            .expect("query bootstrap")
            .snapshot
            .expect("bootstrap snapshot");
        assert_eq!(queried, bootstrap);
        assert_ne!(
            session
                .perform(FlowOperation::Query(FlowQuery::Values))
                .expect("query values")
                .values
                .expect("live values"),
            bootstrap.values,
        );
    }

    #[test]
    fn combined_state_batch_limit_is_atomic() {
        let mut session = smi_session();
        let before_input = input_value(&mut session, "bool");
        let before_catalog = session.bootstrap.catalog.clone();
        let before_values = session.bootstrap.values.clone();
        let before_cycle = session.next_cycle;
        let before_sequence = session.next_sequence;
        let mutation = FlowStateMutation::SetInputBool {
            name: "bool".to_owned(),
            value: true,
        };
        let new_instance = FlowNewInstance {
            local_id: 1,
            schema_name: "unused".to_owned(),
            authored_instance_name: None,
        };

        let error = session
            .perform(FlowOperation::StateBatch(FlowStateBatch {
                host_mutation_id: None,
                mutations: vec![mutation; MAX_BATCH_ITEMS / 2 + 1],
                new_instances: vec![new_instance; MAX_BATCH_ITEMS / 2],
            }))
            .expect_err("combined batch count must be bounded");

        assert_eq!(error.kind(), FlowSessionErrorKind::LimitExceeded);
        assert_eq!(input_value(&mut session, "bool"), before_input);
        assert_eq!(session.bootstrap.catalog, before_catalog);
        assert_eq!(session.bootstrap.values, before_values);
        assert_eq!(session.next_cycle, before_cycle);
        assert_eq!(session.next_sequence, before_sequence);
    }

    #[test]
    fn counter_preflight_failure_leaves_state_and_counters_unchanged() {
        for exhaust_sequence in [false, true] {
            let mut session = smi_session();
            if exhaust_sequence {
                session.next_sequence = u64::MAX;
            } else {
                session.next_cycle = u64::MAX;
            }
            let before_input = input_value(&mut session, "bool");
            let before_catalog = session.bootstrap.catalog.clone();
            let before_values = session.bootstrap.values.clone();
            let before_cycle = session.next_cycle;
            let before_sequence = session.next_sequence;

            let error = session
                .perform(FlowOperation::StateBatch(FlowStateBatch {
                    host_mutation_id: Some(1),
                    mutations: vec![FlowStateMutation::SetInputBool {
                        name: "bool".to_owned(),
                        value: true,
                    }],
                    new_instances: Vec::new(),
                }))
                .expect_err("counter exhaustion must fail before commit");

            assert_eq!(error.kind(), FlowSessionErrorKind::LimitExceeded);
            assert_eq!(input_value(&mut session, "bool"), before_input);
            assert_eq!(session.bootstrap.catalog, before_catalog);
            assert_eq!(session.bootstrap.values, before_values);
            assert_eq!(session.next_cycle, before_cycle);
            assert_eq!(session.next_sequence, before_sequence);
        }
    }

    #[test]
    fn value_arena_preflight_failure_leaves_catalog_ids_and_counters_unchanged() {
        let file = Arc::new(
            File::import(&external_fixture("data_binding_test.riv"))
                .expect("import data-bind fixture"),
        );
        let (mut session, bootstrap) = FlowSession::create(file, FlowSessionConfig::default())
            .expect("create data-bind session");
        let schema = bootstrap
            .catalog
            .schemas
            .iter()
            .filter(|schema| !schema.properties.is_empty())
            .max_by_key(|schema| schema.properties.len())
            .expect("fixture has a nonempty schema");
        let count = MAX_VALUE_EDGES
            .checked_div(schema.properties.len())
            .and_then(|value| value.checked_add(1))
            .expect("bounded instance count");
        assert!(count <= MAX_BATCH_ITEMS);
        assert!(session.instances.len().saturating_add(count) <= MAX_INSTANCES);
        let before_catalog = session.bootstrap.catalog.clone();
        let before_values = session.bootstrap.values.clone();
        let before_next_id = session.next_instance_id;
        let before_cycle = session.next_cycle;
        let before_sequence = session.next_sequence;
        let new_instances = (0..count)
            .map(|local_id| FlowNewInstance {
                local_id: u32::try_from(local_id).expect("local id"),
                schema_name: schema.name.clone(),
                authored_instance_name: None,
            })
            .collect();

        let error = session
            .perform(FlowOperation::StateBatch(FlowStateBatch {
                host_mutation_id: None,
                mutations: Vec::new(),
                new_instances,
            }))
            .expect_err("candidate arena must exceed the edge envelope");

        assert_eq!(error.kind(), FlowSessionErrorKind::LimitExceeded);
        assert_eq!(session.bootstrap.catalog, before_catalog);
        assert_eq!(session.bootstrap.values, before_values);
        assert_eq!(session.next_instance_id, before_next_id);
        assert_eq!(session.next_cycle, before_cycle);
        assert_eq!(session.next_sequence, before_sequence);
    }

    #[test]
    fn advance_uses_required_caller_delta_and_allows_equal_timestamps() {
        let file = Arc::new(File::import(FIXTURE).expect("import fixture"));
        let (mut session, _) =
            FlowSession::create(file, FlowSessionConfig::default()).expect("create session");

        for delta_seconds in [0.25, 0.0] {
            let result = session
                .perform(FlowOperation::Advance(FlowAdvance {
                    timestamp_seconds: 10.0,
                    delta_seconds,
                    render: false,
                }))
                .expect("nondecreasing app clock");
            assert!(result.outputs.iter().any(|output| {
                output.phase == FlowOutputPhase::RuntimeAdvance
                    && output.payload == FlowOutputPayload::RuntimeAdvanced { delta_seconds }
            }));
        }
    }

    #[test]
    fn pointer_batch_limit_is_prevalidated() {
        let file = Arc::new(File::import(FIXTURE).expect("import fixture"));
        let (mut session, _) =
            FlowSession::create(file, FlowSessionConfig::default()).expect("create session");
        let event = FlowPointerEvent {
            kind: FlowPointerKind::Move,
            pointer_id: 1,
            x: 0.0,
            y: 0.0,
        };

        let error = session
            .perform(FlowOperation::PointerBatch(FlowPointerBatch {
                events: vec![event; MAX_POINTERS_PER_BATCH + 1],
            }))
            .expect_err("oversized pointer batch");

        assert_eq!(error.kind(), FlowSessionErrorKind::LimitExceeded);
    }

    #[test]
    fn state_batch_prevalidates_all_input_writes_before_commit() {
        let mut session = smi_session();
        assert_eq!(
            input_value(&mut session, "bool"),
            FlowScalarValue::Bool(false)
        );

        let error = session
            .perform(FlowOperation::StateBatch(FlowStateBatch {
                host_mutation_id: Some(7),
                mutations: vec![
                    FlowStateMutation::SetInputBool {
                        name: "bool".to_owned(),
                        value: true,
                    },
                    FlowStateMutation::SetInputNumber {
                        name: "bool".to_owned(),
                        value: 1.0,
                    },
                ],
                new_instances: Vec::new(),
            }))
            .expect_err("wrong-kind second write rejects whole batch");

        assert_eq!(error.kind(), FlowSessionErrorKind::InvalidArgument);
        assert_eq!(
            input_value(&mut session, "bool"),
            FlowScalarValue::Bool(false)
        );
    }

    #[test]
    fn state_batch_commits_inputs_with_direct_origin_echo_then_advances() {
        let mut session = smi_session();
        let result = session
            .perform(FlowOperation::StateBatch(FlowStateBatch {
                host_mutation_id: Some(42),
                mutations: vec![
                    FlowStateMutation::SetInputBool {
                        name: "bool".to_owned(),
                        value: true,
                    },
                    FlowStateMutation::SetInputNumber {
                        name: "num".to_owned(),
                        value: 12.5,
                    },
                    FlowStateMutation::FireInputTrigger {
                        name: "trig".to_owned(),
                    },
                ],
                new_instances: Vec::new(),
            }))
            .expect("valid state batch");

        assert_eq!(
            input_value(&mut session, "bool"),
            FlowScalarValue::Bool(true)
        );
        assert_eq!(
            input_value(&mut session, "num"),
            FlowScalarValue::Number(12.5)
        );
        assert_eq!(result.outputs.len(), 3);
        assert!(result.outputs.iter().all(|output| {
            output.phase == FlowOutputPhase::ViewModelChanges
                && matches!(
                    &output.payload,
                    FlowOutputPayload::StateChanged {
                        origin_mutation_id: Some(42),
                        ..
                    }
                )
        }));
        assert!(
            result
                .outputs
                .windows(2)
                .all(|pair| pair[0].sequence < pair[1].sequence)
        );

        session
            .perform(FlowOperation::Advance(FlowAdvance {
                timestamp_seconds: 1.0,
                delta_seconds: 0.016,
                render: true,
            }))
            .expect("advance committed state");
    }

    #[test]
    fn view_model_batch_is_atomic_and_refreshes_the_value_arena() {
        let file = Arc::new(
            File::import(&external_fixture("data_binding_test_2.riv"))
                .expect("import data-bind fixture"),
        );
        let (mut session, bootstrap) = FlowSession::create(file, FlowSessionConfig::default())
            .expect("create data-bind session");
        let root = bootstrap
            .catalog
            .root_instance_id
            .expect("retained root id");
        let initial = arena_number(&bootstrap.values, root, "num").expect("initial number");

        let error = session
            .perform(FlowOperation::StateBatch(FlowStateBatch {
                host_mutation_id: Some(90),
                mutations: vec![
                    FlowStateMutation::SetValue {
                        instance: FlowInstanceRef::Existing(root),
                        path: "num".to_owned(),
                        value: FlowScalarValue::Number(initial + 10.0),
                    },
                    FlowStateMutation::SetValue {
                        instance: FlowInstanceRef::Existing(root),
                        path: "num".to_owned(),
                        value: FlowScalarValue::Bool(true),
                    },
                ],
                new_instances: Vec::new(),
            }))
            .expect_err("wrong-kind second mutation rejects whole batch");
        assert_eq!(error.kind(), FlowSessionErrorKind::NotFound);
        let unchanged = session
            .perform(FlowOperation::Query(FlowQuery::Values))
            .expect("query values")
            .values
            .expect("arena");
        assert_eq!(arena_number(&unchanged, root, "num"), Some(initial));

        let result = session
            .perform(FlowOperation::StateBatch(FlowStateBatch {
                host_mutation_id: Some(91),
                mutations: vec![FlowStateMutation::SetValue {
                    instance: FlowInstanceRef::Existing(root),
                    path: "num".to_owned(),
                    value: FlowScalarValue::Number(137.0),
                }],
                new_instances: Vec::new(),
            }))
            .expect("valid VM mutation");
        assert!(matches!(
            result.outputs.as_slice(),
            [FlowOutput {
                payload: FlowOutputPayload::StateChanged {
                    origin_mutation_id: Some(91),
                    ..
                },
                ..
            }]
        ));
        let values = session
            .perform(FlowOperation::Query(FlowQuery::Values))
            .expect("query values")
            .values
            .expect("arena");
        assert_eq!(arena_number(&values, root, "num"), Some(137.0));
    }

    #[test]
    fn list_mutations_are_sequentially_simulated_before_any_commit() {
        let file = Arc::new(
            File::import(&external_fixture("component_list_1.riv")).expect("import list fixture"),
        );
        let (mut session, bootstrap) =
            FlowSession::create(file, FlowSessionConfig::default()).expect("create list session");
        let root = bootstrap
            .catalog
            .root_instance_id
            .expect("retained root id");
        let initial_instance_count = bootstrap.catalog.instances.len();
        let initial_len = arena_list_len(&bootstrap.values, root, "Buttons").expect("initial list");
        assert!(initial_len > 0);
        let new_instance = FlowNewInstance {
            local_id: 10,
            schema_name: "ItemVM".to_owned(),
            authored_instance_name: Some("Instance 3".to_owned()),
        };

        let error = session
            .perform(FlowOperation::StateBatch(FlowStateBatch {
                host_mutation_id: None,
                mutations: vec![
                    FlowStateMutation::ListClear {
                        instance: FlowInstanceRef::Existing(root),
                        path: "Buttons".to_owned(),
                    },
                    FlowStateMutation::ListInsert {
                        instance: FlowInstanceRef::Existing(root),
                        path: "Buttons".to_owned(),
                        index: 0,
                        item: FlowInstanceRef::New(10),
                    },
                    FlowStateMutation::ListRemove {
                        instance: FlowInstanceRef::Existing(root),
                        path: "Buttons".to_owned(),
                        index: 1,
                    },
                ],
                new_instances: vec![new_instance.clone()],
            }))
            .expect_err("post-insert out-of-range remove rejects whole batch");
        assert_eq!(error.kind(), FlowSessionErrorKind::InvalidArgument);
        let unchanged = session
            .perform(FlowOperation::Query(FlowQuery::Values))
            .expect("query values")
            .values
            .expect("arena");
        assert_eq!(
            arena_list_len(&unchanged, root, "Buttons"),
            Some(initial_len)
        );
        assert_eq!(
            session
                .perform(FlowOperation::Query(FlowQuery::Catalog))
                .expect("query catalog")
                .catalog
                .expect("catalog")
                .instances
                .len(),
            initial_instance_count
        );

        let result = session
            .perform(FlowOperation::StateBatch(FlowStateBatch {
                host_mutation_id: Some(101),
                mutations: vec![
                    FlowStateMutation::ListClear {
                        instance: FlowInstanceRef::Existing(root),
                        path: "Buttons".to_owned(),
                    },
                    FlowStateMutation::ListInsert {
                        instance: FlowInstanceRef::Existing(root),
                        path: "Buttons".to_owned(),
                        index: 0,
                        item: FlowInstanceRef::New(10),
                    },
                ],
                new_instances: vec![new_instance],
            }))
            .expect("valid sequential list batch");
        assert_eq!(result.created_instances.len(), 1);
        let values = session
            .perform(FlowOperation::Query(FlowQuery::Values))
            .expect("query values")
            .values
            .expect("arena");
        assert_eq!(arena_list_len(&values, root, "Buttons"), Some(1));
        let catalog = session
            .perform(FlowOperation::Query(FlowQuery::Catalog))
            .expect("query catalog")
            .catalog
            .expect("catalog");
        assert_eq!(catalog.instances.len(), initial_instance_count + 1);
        assert_eq!(
            catalog.instances.last().map(|instance| instance.id),
            Some(result.created_instances[0].id)
        );
    }

    #[test]
    fn list_order_exposes_stable_instance_identity_for_equal_instances() {
        let file = Arc::new(
            File::import(&external_fixture("component_list_1.riv")).expect("import list fixture"),
        );
        let (mut session, bootstrap) =
            FlowSession::create(file, FlowSessionConfig::default()).expect("create list session");
        let root = bootstrap.catalog.root_instance_id.expect("root id");
        let created = session
            .perform(FlowOperation::StateBatch(FlowStateBatch {
                host_mutation_id: None,
                mutations: vec![
                    FlowStateMutation::ListClear {
                        instance: FlowInstanceRef::Existing(root),
                        path: "Buttons".to_owned(),
                    },
                    FlowStateMutation::ListInsert {
                        instance: FlowInstanceRef::Existing(root),
                        path: "Buttons".to_owned(),
                        index: 0,
                        item: FlowInstanceRef::New(1),
                    },
                    FlowStateMutation::ListInsert {
                        instance: FlowInstanceRef::Existing(root),
                        path: "Buttons".to_owned(),
                        index: 1,
                        item: FlowInstanceRef::New(2),
                    },
                ],
                new_instances: vec![
                    FlowNewInstance {
                        local_id: 1,
                        schema_name: "ItemVM".to_owned(),
                        authored_instance_name: None,
                    },
                    FlowNewInstance {
                        local_id: 2,
                        schema_name: "ItemVM".to_owned(),
                        authored_instance_name: None,
                    },
                ],
            }))
            .expect("insert equal default instances")
            .created_instances;
        let first_id = created[0].id;
        let second_id = created[1].id;
        let before = session
            .perform(FlowOperation::Query(FlowQuery::Values))
            .expect("query values")
            .values
            .expect("values");
        let first_node = before
            .roots
            .iter()
            .find(|(id, _)| *id == first_id)
            .expect("first root")
            .1;
        let second_node = before
            .roots
            .iter()
            .find(|(id, _)| *id == second_id)
            .expect("second root")
            .1;
        assert_eq!(
            arena_list_items(&before, root, "Buttons"),
            Some(vec![first_node, second_node])
        );

        session
            .perform(FlowOperation::StateBatch(FlowStateBatch {
                host_mutation_id: None,
                mutations: vec![FlowStateMutation::ListSwap {
                    instance: FlowInstanceRef::Existing(root),
                    path: "Buttons".to_owned(),
                    first: 0,
                    second: 1,
                }],
                new_instances: Vec::new(),
            }))
            .expect("swap equal instances");
        let after = session
            .perform(FlowOperation::Query(FlowQuery::Values))
            .expect("query values")
            .values
            .expect("values");
        let first_node = after
            .roots
            .iter()
            .find(|(id, _)| *id == first_id)
            .expect("first root")
            .1;
        let second_node = after
            .roots
            .iter()
            .find(|(id, _)| *id == second_id)
            .expect("second root")
            .1;
        assert_eq!(
            arena_list_items(&after, root, "Buttons"),
            Some(vec![second_node, first_node])
        );
        assert_eq!(
            diff_value_arenas(&before, &after).expect("diff equal-valued list identities"),
            vec![(root, "Buttons".to_owned(), None)]
        );
    }

    #[test]
    fn bootstrap_preserves_nonzero_authored_artboard_origin() {
        let file = Arc::new(
            File::import(&external_fixture("db_health_tracker.riv"))
                .expect("import origin fixture"),
        );
        let (name, expected) = file
            .artboards()
            .find_map(|artboard| {
                let instance =
                    OwnedArtboardInstance::instantiate(Arc::clone(&file), artboard.index()).ok()?;
                let bounds = instance.artboard_bounds();
                ((bounds.0 != 0.0 || bounds.1 != 0.0) && artboard.name().is_some())
                    .then(|| (artboard.name().unwrap_or_default().to_owned(), bounds))
            })
            .expect("fixture has nonzero-origin artboard");

        let (_, bootstrap) = FlowSession::create(
            file,
            FlowSessionConfig {
                artboard_name: Some(name),
                player_name: None,
            },
        )
        .expect("create nonzero-origin session");

        assert_eq!(
            bootstrap.bounds,
            FlowArtboardBounds {
                x: expected.0,
                y: expected.1,
                width: expected.2,
                height: expected.3
            },
        );
        assert!(bootstrap.bounds.x != 0.0 || bootstrap.bounds.y != 0.0);
    }

    #[test]
    fn atomic_validation_preserves_cross_root_list_aliases_and_does_not_consume_ids() {
        let file = Arc::new(
            File::import(&external_fixture("component_list_1.riv")).expect("import list fixture"),
        );
        let (mut session, _) =
            FlowSession::create(file, FlowSessionConfig::default()).expect("create list session");
        let initial_instance_count = session.bootstrap.catalog.instances.len();
        let created = session
            .perform(FlowOperation::StateBatch(FlowStateBatch {
                host_mutation_id: None,
                mutations: vec![FlowStateMutation::ListInsert {
                    instance: FlowInstanceRef::New(1),
                    path: "Buttons".to_owned(),
                    index: 0,
                    item: FlowInstanceRef::New(2),
                }],
                new_instances: vec![
                    FlowNewInstance {
                        local_id: 1,
                        schema_name: "MainVM".to_owned(),
                        authored_instance_name: None,
                    },
                    FlowNewInstance {
                        local_id: 2,
                        schema_name: "MainVM".to_owned(),
                        authored_instance_name: None,
                    },
                ],
            }))
            .expect("create aliased roots")
            .created_instances;
        let a = created[0].id;
        let b = created[1].id;

        let error = session
            .perform(FlowOperation::StateBatch(FlowStateBatch {
                host_mutation_id: None,
                mutations: vec![
                    FlowStateMutation::ListInsert {
                        instance: FlowInstanceRef::Existing(b),
                        path: "Buttons".to_owned(),
                        index: 0,
                        item: FlowInstanceRef::New(3),
                    },
                    FlowStateMutation::ListInsert {
                        instance: FlowInstanceRef::Existing(b),
                        path: "Buttons".to_owned(),
                        index: 1,
                        item: FlowInstanceRef::Existing(a),
                    },
                ],
                new_instances: vec![FlowNewInstance {
                    local_id: 3,
                    schema_name: "MainVM".to_owned(),
                    authored_instance_name: None,
                }],
            }))
            .expect_err("B to A closes the existing A to B cycle");
        assert_eq!(error.kind(), FlowSessionErrorKind::Conflict);
        let values = session
            .perform(FlowOperation::Query(FlowQuery::Values))
            .expect("query values")
            .values
            .expect("arena");
        assert_eq!(arena_list_len(&values, a, "Buttons"), Some(1));
        assert_eq!(arena_list_len(&values, b, "Buttons"), Some(0));
        assert_eq!(
            session
                .perform(FlowOperation::Query(FlowQuery::Catalog))
                .expect("query catalog")
                .catalog
                .expect("catalog")
                .instances
                .len(),
            initial_instance_count + 2
        );

        let next = session
            .perform(FlowOperation::StateBatch(FlowStateBatch {
                host_mutation_id: None,
                mutations: Vec::new(),
                new_instances: vec![FlowNewInstance {
                    local_id: 4,
                    schema_name: "MainVM".to_owned(),
                    authored_instance_name: None,
                }],
            }))
            .expect("next id remains available")
            .created_instances[0]
            .id;
        assert_eq!(next.get(), b.get() + 1);
    }

    #[test]
    fn events_produced_by_advance_are_drained_before_the_next_advance() {
        let file = Arc::new(
            File::import(&external_fixture("events_on_states.riv")).expect("import event fixture"),
        );
        let (mut session, _) =
            FlowSession::create(file, FlowSessionConfig::default()).expect("create event session");

        let first = session
            .perform(FlowOperation::Advance(FlowAdvance {
                timestamp_seconds: 0.0,
                delta_seconds: 0.0,
                render: false,
            }))
            .expect("first advance");
        assert!(
            !first
                .outputs
                .iter()
                .any(|output| output.phase == FlowOutputPhase::ReportedEvents)
        );
        assert_eq!(first.wake_after_seconds, Some(0.0));
        assert!(!first.settled);

        let second = session
            .perform(FlowOperation::Advance(FlowAdvance {
                timestamp_seconds: 0.0,
                delta_seconds: 0.0,
                render: false,
            }))
            .expect("second advance");
        let event_position = second
            .outputs
            .iter()
            .position(|output| output.phase == FlowOutputPhase::ReportedEvents)
            .expect("event produced by first advance");
        let advance_position = second
            .outputs
            .iter()
            .position(|output| output.phase == FlowOutputPhase::RuntimeAdvance)
            .expect("second runtime advance");
        assert!(event_position < advance_position);
        assert_eq!(
            second.outputs[event_position].cycle,
            second.outputs[advance_position].cycle
        );
        assert!(
            !second
                .outputs
                .iter()
                .any(|output| { output.phase == FlowOutputPhase::DelayedEventCallbacks })
        );
    }

    #[test]
    fn event_seconds_delay_is_immediate_overshoot_metadata_not_a_deadline() {
        let file = Arc::new(
            File::import(&external_fixture("timeline_event_test.riv"))
                .expect("import timeline event fixture"),
        );
        let (mut session, _) = FlowSession::create(file, FlowSessionConfig::default())
            .expect("create timeline event session");
        for (timestamp_seconds, delta_seconds) in [(0.0, 0.0), (0.4, 0.4)] {
            session
                .perform(FlowOperation::Advance(FlowAdvance {
                    timestamp_seconds,
                    delta_seconds,
                    render: false,
                }))
                .expect("prime timeline");
        }
        let produced = session
            .perform(FlowOperation::Advance(FlowAdvance {
                timestamp_seconds: 0.6,
                delta_seconds: 0.2,
                render: false,
            }))
            .expect("cross event keyframe");
        assert_eq!(produced.wake_after_seconds, Some(0.0));
        assert!(!produced.settled);

        let drained = session
            .perform(FlowOperation::Advance(FlowAdvance {
                timestamp_seconds: 0.6,
                delta_seconds: 0.0,
                render: false,
            }))
            .expect("drain report before advancing");
        let reported = drained
            .outputs
            .iter()
            .find_map(|output| match &output.payload {
                FlowOutputPayload::ReportedEvent {
                    name,
                    delay_seconds,
                    ..
                } if name.as_deref() == Some("Half") => Some((*delay_seconds, output.phase)),
                _ => None,
            })
            .expect("half event report");
        assert!((reported.0 - 0.1).abs() < 0.0001);
        assert_eq!(reported.1, FlowOutputPhase::ReportedEvents);
        assert!(
            !drained
                .outputs
                .iter()
                .any(|output| { output.phase == FlowOutputPhase::DelayedEventCallbacks })
        );
        assert_ne!(drained.wake_after_seconds, Some(reported.0));
    }

    #[test]
    fn zero_host_mutation_id_is_preserved_as_present_origin_evidence() {
        let mut session = smi_session();

        let result = session
            .perform(FlowOperation::StateBatch(FlowStateBatch {
                host_mutation_id: Some(0),
                mutations: vec![FlowStateMutation::SetInputBool {
                    name: "bool".to_owned(),
                    value: true,
                }],
                new_instances: Vec::new(),
            }))
            .expect("zero is a valid present mutation identity");

        assert!(matches!(
            result.outputs.as_slice(),
            [FlowOutput {
                payload: FlowOutputPayload::StateChanged {
                    origin_mutation_id: Some(0),
                    ..
                },
                ..
            }]
        ));
    }

    #[test]
    fn value_arena_accepts_exact_node_limit_and_rejects_the_next_node() {
        let file = File::import(FIXTURE).expect("import fixture");
        let mut builder = ValueArenaBuilder::new(&file);

        for _ in 0..MAX_VALUE_NODES {
            builder.push(FlowValue::Null).expect("node within cap");
        }
        let error = builder
            .push(FlowValue::Null)
            .expect_err("node beyond cap must be rejected");

        assert_eq!(error.kind(), FlowSessionErrorKind::LimitExceeded);
        assert_eq!(builder.arena.nodes.len(), MAX_VALUE_NODES);
    }

    #[test]
    fn output_validation_rejects_non_finite_and_oversized_scalar_payloads_before_sequence_use() {
        let mut outputs = Vec::new();
        let mut next_sequence = 1;
        let non_finite = FlowOutputPayload::ReportedEvent {
            name: Some("event".to_owned()),
            event_type: 0,
            delay_seconds: 0.0,
            properties: vec![FlowEventProperty {
                name: Some("number".to_owned()),
                value: FlowScalarValue::Number(f32::NAN),
            }],
        };

        let non_finite_error = append_output(
            &mut outputs,
            &mut next_sequence,
            1,
            FlowOutputPhase::ReportedEvents,
            non_finite,
        )
        .expect_err("non-finite event values must be rejected");
        assert_eq!(non_finite_error.kind(), FlowSessionErrorKind::Runtime);

        let oversized = FlowOutputPayload::StateChanged {
            instance_id: None,
            path: "value".to_owned(),
            value: Some(FlowScalarValue::String("x".repeat(MAX_STRING_BYTES + 1))),
            origin_mutation_id: None,
        };
        let oversized_error = append_output(
            &mut outputs,
            &mut next_sequence,
            1,
            FlowOutputPhase::ViewModelChanges,
            oversized,
        )
        .expect_err("oversized scalar strings must be rejected");

        assert_eq!(oversized_error.kind(), FlowSessionErrorKind::LimitExceeded);
        assert!(outputs.is_empty());
        assert_eq!(next_sequence, 1);
    }

    #[test]
    fn post_mutation_advance_failure_terminally_poisoned_session_rejects_every_later_operation() {
        let mut session = smi_session();
        session.next_sequence = u64::MAX;

        let first_error = session
            .perform(FlowOperation::Advance(FlowAdvance {
                timestamp_seconds: 1.0,
                delta_seconds: 0.016,
                render: false,
            }))
            .expect_err("sequence exhaustion fails after entering the mutation cycle");
        assert_eq!(first_error.kind(), FlowSessionErrorKind::LimitExceeded);

        let terminal_error = session
            .perform(FlowOperation::Query(FlowQuery::Values))
            .expect_err("a terminal session must reject even read operations");
        assert_eq!(terminal_error.kind(), FlowSessionErrorKind::Runtime);
        assert!(
            terminal_error
                .message()
                .contains("flow session is terminal")
        );

        let retry_error = session
            .perform(FlowOperation::Advance(FlowAdvance {
                timestamp_seconds: 1.0,
                delta_seconds: 0.016,
                render: false,
            }))
            .expect_err("a terminal session must never replay the failed frame");
        assert_eq!(retry_error, terminal_error);
    }

    #[test]
    fn active_pointer_limit_spans_batches_and_exit_releases_capacity() {
        let mut session = smi_session();
        let pointer = |kind, pointer_id| {
            FlowOperation::PointerBatch(FlowPointerBatch {
                events: vec![FlowPointerEvent {
                    kind,
                    pointer_id,
                    x: -10_000.0,
                    y: -10_000.0,
                }],
            })
        };

        for pointer_id in 1..=MAX_POINTERS_PER_BATCH as i32 {
            session
                .perform(pointer(FlowPointerKind::Move, pointer_id))
                .expect("pointer within the session-wide cap");
        }
        let error = session
            .perform(pointer(
                FlowPointerKind::Move,
                MAX_POINTERS_PER_BATCH as i32 + 1,
            ))
            .expect_err("the active-pointer cap must span separate batches");
        assert_eq!(error.kind(), FlowSessionErrorKind::LimitExceeded);

        session
            .perform(pointer(FlowPointerKind::Exit, 1))
            .expect("exit releases one active pointer");
        session
            .perform(pointer(
                FlowPointerKind::Move,
                MAX_POINTERS_PER_BATCH as i32 + 1,
            ))
            .expect("released capacity can be reused by another pointer");
    }

    #[test]
    fn schema_mapping_uses_canonical_identity_and_trigger_types() {
        assert_eq!(
            flow_value_type_for_property("ViewModelPropertySymbolListIndex"),
            FlowValueType::Enum
        );
        assert_eq!(
            flow_value_type_for_property("ViewModelPropertyArtboard"),
            FlowValueType::Enum
        );
        assert_eq!(
            flow_value_type_for_property("ViewModelPropertyTrigger"),
            FlowValueType::Trigger
        );
    }
}
