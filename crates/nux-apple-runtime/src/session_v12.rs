//! ABI 1.2's bounded, coarse flow-session protocol.
//!
//! The C layouts in this module are deliberately independent from the Rust
//! session model. Every caller-owned view is validated and copied before the
//! private adapter seam is entered, and every returned view borrows storage
//! owned by one opaque result handle. This keeps the public ABI stable while
//! `nuxie::flow_session` is completed behind the seam.

use super::*;
use std::{collections::HashSet, ffi::c_void, ptr, slice};

pub const NUX_FLOW_SESSION_ABI_MINOR: u16 = 2;

pub const NUX_FLOW_MAX_ID_BYTE_LENGTH: u64 = 4_096;
pub const NUX_FLOW_MAX_PATH_BYTE_LENGTH: u64 = 4_096;
pub const NUX_FLOW_MAX_STRING_BYTE_LENGTH: u64 = 1_048_576;
pub const NUX_FLOW_MAX_BATCH_ITEM_COUNT: u64 = 4_096;
pub const NUX_FLOW_MAX_QUERY_COUNT: u64 = 4_096;
pub const NUX_FLOW_MAX_OUTPUT_COUNT: u64 = 4_096;
pub const NUX_FLOW_MAX_INSTANCE_COUNT: u64 = 4_096;
pub const NUX_FLOW_MAX_LIST_ITEM_COUNT: u64 = 4_096;
pub const NUX_FLOW_MAX_VALUE_EDGE_COUNT: u64 = 16_384;
pub const NUX_FLOW_MAX_VALUE_DEPTH: u32 = 32;
pub const NUX_FLOW_MAX_EVENT_PROPERTY_COUNT: u64 = 256;
pub const NUX_FLOW_MAX_OPERATION_PAYLOAD_BYTE_LENGTH: u64 = 4_194_304;
pub const NUX_FLOW_MAX_POINTER_COUNT: u64 = 32;

const NO_VALUE_ROOT: u32 = u32::MAX;

/// Stable-width selected-player kind.
pub type NuxFlowPlayerKind = u32;

pub const NUX_FLOW_PLAYER_KIND_STATE_MACHINE: NuxFlowPlayerKind = 1;
pub const NUX_FLOW_PLAYER_KIND_LINEAR_ANIMATION: NuxFlowPlayerKind = 2;
pub const NUX_FLOW_PLAYER_KIND_STATIC: NuxFlowPlayerKind = 3;

/// Stable-width branch used by deterministic player selection.
pub type NuxFlowPlayerSelection = u32;

pub const NUX_FLOW_PLAYER_SELECTION_EXPLICIT_STATE_MACHINE: NuxFlowPlayerSelection = 1;
pub const NUX_FLOW_PLAYER_SELECTION_AUTHORED_DEFAULT_STATE_MACHINE: NuxFlowPlayerSelection = 2;
pub const NUX_FLOW_PLAYER_SELECTION_FIRST_STATE_MACHINE: NuxFlowPlayerSelection = 3;
pub const NUX_FLOW_PLAYER_SELECTION_FIRST_ANIMATION: NuxFlowPlayerSelection = 4;
pub const NUX_FLOW_PLAYER_SELECTION_STATIC: NuxFlowPlayerSelection = 5;

/// Stable-width state-machine input kind returned by a player-input query.
pub type NuxFlowPlayerInputKind = u32;

pub const NUX_FLOW_PLAYER_INPUT_KIND_BOOL: NuxFlowPlayerInputKind = 1;
pub const NUX_FLOW_PLAYER_INPUT_KIND_NUMBER: NuxFlowPlayerInputKind = 2;
pub const NUX_FLOW_PLAYER_INPUT_KIND_TRIGGER: NuxFlowPlayerInputKind = 3;

/// Stable-width generic session-operation kind.
pub type NuxFlowSessionOperationKind = u32;

pub const NUX_FLOW_SESSION_OPERATION_KIND_STATE_BATCH: NuxFlowSessionOperationKind = 1;
pub const NUX_FLOW_SESSION_OPERATION_KIND_POINTER_BATCH: NuxFlowSessionOperationKind = 2;
pub const NUX_FLOW_SESSION_OPERATION_KIND_ADVANCE: NuxFlowSessionOperationKind = 3;
pub const NUX_FLOW_SESSION_OPERATION_KIND_QUERY: NuxFlowSessionOperationKind = 4;

/// Stable-width canonical-state mutation kind.
pub type NuxFlowStateMutationKind = u32;

pub const NUX_FLOW_STATE_MUTATION_KIND_SET: NuxFlowStateMutationKind = 1;
pub const NUX_FLOW_STATE_MUTATION_KIND_TRIGGER: NuxFlowStateMutationKind = 2;
pub const NUX_FLOW_STATE_MUTATION_KIND_LIST_INSERT: NuxFlowStateMutationKind = 3;
pub const NUX_FLOW_STATE_MUTATION_KIND_LIST_REMOVE: NuxFlowStateMutationKind = 4;
pub const NUX_FLOW_STATE_MUTATION_KIND_LIST_SWAP: NuxFlowStateMutationKind = 5;
pub const NUX_FLOW_STATE_MUTATION_KIND_LIST_MOVE: NuxFlowStateMutationKind = 6;
pub const NUX_FLOW_STATE_MUTATION_KIND_LIST_SET: NuxFlowStateMutationKind = 7;
pub const NUX_FLOW_STATE_MUTATION_KIND_LIST_CLEAR: NuxFlowStateMutationKind = 8;
pub const NUX_FLOW_STATE_MUTATION_KIND_SET_INPUT_BOOL: NuxFlowStateMutationKind = 9;
pub const NUX_FLOW_STATE_MUTATION_KIND_SET_INPUT_NUMBER: NuxFlowStateMutationKind = 10;
pub const NUX_FLOW_STATE_MUTATION_KIND_FIRE_INPUT_TRIGGER: NuxFlowStateMutationKind = 11;

/// Stable-width instance reference used by state mutations.
pub type NuxFlowInstanceReferenceKind = u32;

pub const NUX_FLOW_INSTANCE_REFERENCE_KIND_EXISTING: NuxFlowInstanceReferenceKind = 1;
pub const NUX_FLOW_INSTANCE_REFERENCE_KIND_NEW: NuxFlowInstanceReferenceKind = 2;

/// Stable-width pointer command. Coordinates are already inverse-mapped into
/// artboard space and are never clamped by the runtime.
pub type NuxFlowPointerEventKind = u32;

pub const NUX_FLOW_POINTER_EVENT_KIND_DOWN: NuxFlowPointerEventKind = 1;
pub const NUX_FLOW_POINTER_EVENT_KIND_MOVE: NuxFlowPointerEventKind = 2;
pub const NUX_FLOW_POINTER_EVENT_KIND_UP: NuxFlowPointerEventKind = 3;
pub const NUX_FLOW_POINTER_EVENT_KIND_CANCEL: NuxFlowPointerEventKind = 4;
pub const NUX_FLOW_POINTER_EVENT_KIND_EXIT: NuxFlowPointerEventKind = 5;

/// Stable-width query kind. Query results populate the result's borrowed
/// bootstrap, value, catalog, and player-input views; queries do not emit
/// ordered output records.
pub type NuxFlowQueryKind = u32;

pub const NUX_FLOW_QUERY_KIND_BOOTSTRAP: NuxFlowQueryKind = 1;
pub const NUX_FLOW_QUERY_KIND_VALUES: NuxFlowQueryKind = 2;
pub const NUX_FLOW_QUERY_KIND_CATALOG: NuxFlowQueryKind = 3;
pub const NUX_FLOW_QUERY_KIND_PLAYER_INPUTS: NuxFlowQueryKind = 4;

/// Stable-width recursive value kind.
pub type NuxFlowValueKind = u32;

pub const NUX_FLOW_VALUE_KIND_NULL: NuxFlowValueKind = 0;
pub const NUX_FLOW_VALUE_KIND_STRING: NuxFlowValueKind = 1;
pub const NUX_FLOW_VALUE_KIND_NUMBER: NuxFlowValueKind = 2;
pub const NUX_FLOW_VALUE_KIND_BOOL: NuxFlowValueKind = 3;
pub const NUX_FLOW_VALUE_KIND_ENUM: NuxFlowValueKind = 4;
pub const NUX_FLOW_VALUE_KIND_COLOR: NuxFlowValueKind = 5;
pub const NUX_FLOW_VALUE_KIND_IMAGE: NuxFlowValueKind = 6;
pub const NUX_FLOW_VALUE_KIND_OBJECT: NuxFlowValueKind = 7;
pub const NUX_FLOW_VALUE_KIND_VIEW_MODEL: NuxFlowValueKind = 8;
pub const NUX_FLOW_VALUE_KIND_LIST: NuxFlowValueKind = 9;

/// Stable-width observable output phase. Phases are monotonic inside one
/// cycle, and may restart when a pointer batch starts another immediate cycle.
pub type NuxFlowOutputPhase = u32;

/// Reserved for the runtime's ordering contract; current Rive event delays are
/// overshoot metadata and do not schedule callbacks into this phase.
pub const NUX_FLOW_OUTPUT_PHASE_DELAYED_EVENT_CALLBACKS: NuxFlowOutputPhase = 0;
pub const NUX_FLOW_OUTPUT_PHASE_REPORTED_EVENTS: NuxFlowOutputPhase = 1;
pub const NUX_FLOW_OUTPUT_PHASE_RUNTIME_ADVANCE: NuxFlowOutputPhase = 2;
pub const NUX_FLOW_OUTPUT_PHASE_VIEW_MODEL_CHANGES: NuxFlowOutputPhase = 3;
pub const NUX_FLOW_OUTPUT_PHASE_HOST_WORK: NuxFlowOutputPhase = 4;
pub const NUX_FLOW_OUTPUT_PHASE_RENDER: NuxFlowOutputPhase = 5;

/// Stable-width output payload family.
pub type NuxFlowOutputKind = u32;

pub const NUX_FLOW_OUTPUT_KIND_REPORTED_EVENT: NuxFlowOutputKind = 2;
pub const NUX_FLOW_OUTPUT_KIND_STATE_CHANGE: NuxFlowOutputKind = 3;
pub const NUX_FLOW_OUTPUT_KIND_VIEW_MODEL_CHANGE: NuxFlowOutputKind = 4;
pub const NUX_FLOW_OUTPUT_KIND_HOST_COMMAND: NuxFlowOutputKind = 5;
pub const NUX_FLOW_OUTPUT_KIND_RENDER_REQUEST: NuxFlowOutputKind = 6;
pub const NUX_FLOW_OUTPUT_KIND_QUERY_RESULT: NuxFlowOutputKind = 7;
pub const NUX_FLOW_OUTPUT_KIND_METADATA: NuxFlowOutputKind = 8;
pub const NUX_FLOW_OUTPUT_KIND_RUNTIME_ADVANCED: NuxFlowOutputKind = 9;

/// Stable-width schema property kind. Values intentionally share the recursive
/// value-kind vocabulary where the property is directly representable.
pub type NuxFlowSchemaPropertyKind = u32;

pub const NUX_FLOW_SCHEMA_PROPERTY_KIND_STRING: NuxFlowSchemaPropertyKind = 1;
pub const NUX_FLOW_SCHEMA_PROPERTY_KIND_NUMBER: NuxFlowSchemaPropertyKind = 2;
pub const NUX_FLOW_SCHEMA_PROPERTY_KIND_BOOL: NuxFlowSchemaPropertyKind = 3;
pub const NUX_FLOW_SCHEMA_PROPERTY_KIND_TRIGGER: NuxFlowSchemaPropertyKind = 4;
pub const NUX_FLOW_SCHEMA_PROPERTY_KIND_ENUM: NuxFlowSchemaPropertyKind = 5;
pub const NUX_FLOW_SCHEMA_PROPERTY_KIND_COLOR: NuxFlowSchemaPropertyKind = 6;
pub const NUX_FLOW_SCHEMA_PROPERTY_KIND_IMAGE: NuxFlowSchemaPropertyKind = 7;
pub const NUX_FLOW_SCHEMA_PROPERTY_KIND_VIEW_MODEL: NuxFlowSchemaPropertyKind = 8;
pub const NUX_FLOW_SCHEMA_PROPERTY_KIND_LIST: NuxFlowSchemaPropertyKind = 9;
pub const NUX_FLOW_SCHEMA_PROPERTY_KIND_OBJECT: NuxFlowSchemaPropertyKind = 10;
pub const NUX_FLOW_SCHEMA_PROPERTY_KIND_NULL: NuxFlowSchemaPropertyKind = 11;

/// ABI 1.2 configured-session descriptor. `minimum_abi_minor` must be 2 for
/// this surface. A null `artboard_name` selects the default artboard. A null
/// `player_name` uses the authored fallback policy; a nonempty UTF-8 name
/// explicitly selects a state machine. Linear animations are fallback-only.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NuxFlowConfiguredSessionDescriptor {
    pub struct_size: u32,
    pub required_abi_major: u16,
    pub minimum_abi_minor: u16,
    pub artboard_name: NuxByteView,
    pub player_name: NuxByteView,
}

/// One node in a caller-owned recursive value arena. Array elements require
/// the exact published size. `identity_value` carries enum/image identity;
/// caller-supplied object/view-model nodes use `schema_id`, and view-model
/// nodes additionally use `instance_id`. Result view-model nodes always carry
/// stable `instance_id`; `schema_id` is populated when catalog metadata is in
/// the same result. Composite children occupy `first_edge..edge_count`.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NuxFlowValueNode {
    pub struct_size: u32,
    pub kind: NuxFlowValueKind,
    pub number_value: f64,
    pub color_value: u32,
    /// Canonical false/true values are exactly 0 and 1.
    pub bool_value: u32,
    pub first_edge: u32,
    pub edge_count: u32,
    /// Canonical 0/1 presence flag for `instance_id`.
    pub has_instance_id: u32,
    pub instance_id: u64,
    pub identity_value: u64,
    pub string_value: NuxByteView,
    pub schema_id: NuxByteView,
}

/// One edge in a caller-owned recursive value arena. Object and view-model
/// edges require a nonempty UTF-8 key; list edges require an empty key.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NuxFlowValueEdge {
    pub struct_size: u32,
    pub node_index: u32,
    pub key: NuxByteView,
}

/// One root binding from a stable external instance to a value-arena node.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NuxFlowValueRootView {
    pub struct_size: u32,
    pub value_root_index: u32,
    pub instance_id: u64,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NuxFlowValueArena {
    pub struct_size: u32,
    pub nodes: *const NuxFlowValueNode,
    pub node_count: u64,
    pub edges: *const NuxFlowValueEdge,
    pub edge_count: u64,
}

/// One host-created view-model instance available to all mutations in the
/// same atomic batch. `local_id` is referenced by `NEW` instance references
/// and is resolved to a stable runtime ID only if the entire batch commits.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NuxFlowNewInstance {
    pub struct_size: u32,
    pub local_id: u32,
    pub schema_name: NuxByteView,
    /// Null selects schema defaults; a name selects an authored template.
    pub authored_instance_name: NuxByteView,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NuxFlowInstanceReference {
    pub kind: NuxFlowInstanceReferenceKind,
    pub local_id: u32,
    pub instance_id: u64,
}

/// One canonical-state mutation. `index` and `other_index` are interpreted by
/// list operations. A value root is required by scalar set and must be
/// `UINT32_MAX` for mutations without a scalar value. List insert/set select
/// their view-model item through `item` instead. Player-input operations use
/// `input_name` and require `instance`, `item`, and `path` to be zero/absent.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NuxFlowStateMutation {
    pub struct_size: u32,
    pub kind: NuxFlowStateMutationKind,
    pub instance: NuxFlowInstanceReference,
    /// Used by list insert/set and zeroed for other mutation kinds.
    pub item: NuxFlowInstanceReference,
    pub path: NuxByteView,
    pub input_name: NuxByteView,
    pub value_root_index: u32,
    pub index: u32,
    pub other_index: u32,
}

/// One all-or-nothing canonical-state batch. Rust prevalidates the complete
/// batch, including sequential list effects, before applying any mutation.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NuxFlowStateBatch {
    pub struct_size: u32,
    /// Canonical 0/1 presence flag for `host_mutation_id`.
    pub has_host_mutation_id: u32,
    pub host_mutation_id: u64,
    pub value_arena: *const NuxFlowValueArena,
    pub new_instances: *const NuxFlowNewInstance,
    pub new_instance_count: u64,
    pub mutations: *const NuxFlowStateMutation,
    pub mutation_count: u64,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NuxFlowPointerEvent {
    pub struct_size: u32,
    pub kind: NuxFlowPointerEventKind,
    /// Positive session-scoped pointer identity, passed losslessly to runtime.
    pub pointer_id: i32,
    pub x: f32,
    pub y: f32,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NuxFlowPointerBatch {
    pub struct_size: u32,
    pub events: *const NuxFlowPointerEvent,
    pub event_count: u64,
}

/// One app-clock advance. The first delta after create/resume is zero. A live
/// Apple drawable and completion pair are borrowed only for the synchronous
/// perform call and follow the same exactly-once completion contract as ABI
/// 1.1's frame operation.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NuxFlowAdvanceOperation {
    pub struct_size: u32,
    pub timestamp_seconds: f64,
    pub delta_seconds: f32,
    /// Canonical false/true values are exactly 0 and 1.
    pub render: u32,
    pub apple_drawable: *mut c_void,
    pub completion_context: *mut c_void,
    pub completion_callback: Option<unsafe extern "C" fn(context: *mut c_void)>,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NuxFlowQuery {
    pub struct_size: u32,
    pub kind: NuxFlowQueryKind,
}

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NuxFlowQueryBatch {
    pub struct_size: u32,
    pub queries: *const NuxFlowQuery,
    pub query_count: u64,
}

/// Tagged generic operation. Exactly the pointer selected by `kind` must be
/// non-null and the other payload pointers must be null.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NuxFlowSessionOperation {
    pub struct_size: u32,
    pub required_abi_major: u16,
    pub minimum_abi_minor: u16,
    pub kind: NuxFlowSessionOperationKind,
    pub state_batch: *const NuxFlowStateBatch,
    pub pointer_batch: *const NuxFlowPointerBatch,
    pub advance: *const NuxFlowAdvanceOperation,
    pub query_batch: *const NuxFlowQueryBatch,
}

/// Borrowed selected-player metadata owned by a session result.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NuxFlowPlayerMetadataView {
    pub struct_size: u32,
    pub kind: NuxFlowPlayerKind,
    pub selection: NuxFlowPlayerSelection,
    /// Authored player index, or `UINT32_MAX` for a static artboard.
    pub player_index: u32,
    pub artboard_name: NuxByteView,
    pub player_name: NuxByteView,
    pub min_x: f32,
    pub min_y: f32,
    pub max_x: f32,
    pub max_y: f32,
}

/// Borrowed state-machine input snapshot. `name` is null only for an unnamed
/// authored input. The value root is owned by the same session result.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NuxFlowPlayerInputView {
    pub struct_size: u32,
    pub kind: NuxFlowPlayerInputKind,
    pub value_root_index: u32,
    pub name: NuxByteView,
}

/// Borrowed view-model schema record owned by a session result.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NuxFlowSchemaView {
    pub struct_size: u32,
    pub first_property: u32,
    pub property_count: u32,
    pub schema_id: NuxByteView,
    pub name: NuxByteView,
}

/// Borrowed schema-property record owned by a session result.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NuxFlowSchemaPropertyView {
    pub struct_size: u32,
    pub kind: NuxFlowSchemaPropertyKind,
    pub schema_id: NuxByteView,
    pub property_id: NuxByteView,
    pub name: NuxByteView,
}

/// Borrowed authored instance template. Templates are immutable creation
/// recipes and are not addressable live instances.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NuxFlowInstanceTemplateView {
    pub struct_size: u32,
    pub authored_index: u32,
    pub schema_id: NuxByteView,
    pub authored_name: NuxByteView,
}

/// Borrowed stable external instance record owned by a session result.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NuxFlowInstanceView {
    pub struct_size: u32,
    pub value_root_index: u32,
    pub is_root: u32,
    pub instance_id: u64,
    pub schema_id: NuxByteView,
    pub name: NuxByteView,
}

/// Mapping returned after an atomic batch commits host-created instances.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NuxFlowCreatedInstanceView {
    pub struct_size: u32,
    pub local_id: u32,
    pub instance_id: u64,
}

/// Borrowed exact-order output owned by a session result. `payload_root_index`
/// is `UINT32_MAX` when the item has no typed arena payload.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NuxFlowOutputView {
    pub struct_size: u32,
    pub phase: NuxFlowOutputPhase,
    pub kind: NuxFlowOutputKind,
    pub payload_root_index: u32,
    /// Canonical 0/1 presence flag for `origin_mutation_id`.
    pub has_origin_mutation_id: u32,
    /// Canonical 0/1 presence flag for `instance_id`.
    pub has_instance_id: u32,
    pub sequence: u64,
    pub cycle: u64,
    pub origin_mutation_id: u64,
    pub instance_id: u64,
    pub event_type: u32,
    pub first_event_property: u32,
    pub event_property_count: u32,
    pub delay_seconds: f32,
    pub name: NuxByteView,
    pub path: NuxByteView,
    pub payload: NuxByteView,
}

/// Borrowed typed property of a reported event.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NuxFlowEventPropertyView {
    pub struct_size: u32,
    pub value_root_index: u32,
    /// Canonical 0/1 presence flag for `trigger_count`.
    pub has_trigger_count: u32,
    pub trigger_count: u64,
    /// Null when the authored event property has no name.
    pub name: NuxByteView,
}

/// Opaque owned ABI 1.2 result. Every borrowed view returned by an accessor
/// remains valid until this handle is freed.
pub struct NuxFlowSessionResult {
    _private: [u8; 0],
}

#[derive(Debug, Clone, PartialEq)]
struct OwnedValueNode {
    kind: NuxFlowValueKind,
    number_value: f64,
    color_value: u32,
    bool_value: bool,
    first_edge: u32,
    edge_count: u32,
    instance_id: Option<u64>,
    identity_value: u64,
    string_value: Vec<u8>,
    schema_id: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct OwnedValueEdge {
    node_index: u32,
    key: Vec<u8>,
}

#[derive(Debug, Clone, Default, PartialEq)]
struct OwnedValueArena {
    nodes: Vec<OwnedValueNode>,
    edges: Vec<OwnedValueEdge>,
}

#[derive(Debug, Clone)]
struct OwnedConfiguredSessionDescriptor {
    artboard_name: Option<String>,
    player_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct OwnedNewInstance {
    local_id: u32,
    schema_name: Vec<u8>,
    authored_instance_name: Option<Vec<u8>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OwnedInstanceReference {
    Existing(u64),
    New(u32),
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct OwnedStateMutation {
    kind: NuxFlowStateMutationKind,
    instance: Option<OwnedInstanceReference>,
    item: Option<OwnedInstanceReference>,
    path: Option<Vec<u8>>,
    input_name: Option<Vec<u8>>,
    value_root_index: Option<u32>,
    index: u32,
    other_index: u32,
}

#[derive(Debug, Clone, PartialEq)]
struct OwnedStateBatch {
    host_mutation_id: Option<u64>,
    value_arena: OwnedValueArena,
    new_instances: Vec<OwnedNewInstance>,
    mutations: Vec<OwnedStateMutation>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct OwnedPointerEvent {
    kind: NuxFlowPointerEventKind,
    pointer_id: i32,
    x: f32,
    y: f32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct OwnedQuery {
    kind: NuxFlowQueryKind,
}

#[derive(Debug)]
struct OwnedAdvanceOperation {
    timestamp_seconds: f64,
    delta_seconds: f32,
    render: bool,
    drawable_identity: usize,
    completion_context_identity: usize,
    completion_callback: Option<unsafe extern "C" fn(context: *mut c_void)>,
}

impl Drop for OwnedAdvanceOperation {
    fn drop(&mut self) {
        if let Some(callback) = self.completion_callback.take() {
            unsafe {
                callback(ptr::with_exposed_provenance_mut(
                    self.completion_context_identity,
                ));
            }
        }
    }
}

#[derive(Debug)]
enum OwnedSessionOperation {
    StateBatch(OwnedStateBatch),
    PointerBatch(Vec<OwnedPointerEvent>),
    Advance(OwnedAdvanceOperation),
    Query(Vec<OwnedQuery>),
}

#[derive(Debug, Default)]
struct PayloadBudget {
    bytes: usize,
}

impl PayloadBudget {
    fn charge(&mut self, bytes: usize) -> Result<(), NuxStatus> {
        self.bytes = self
            .bytes
            .checked_add(bytes)
            .ok_or(NuxStatus::InvalidArgument)?;
        if self.bytes > NUX_FLOW_MAX_OPERATION_PAYLOAD_BYTE_LENGTH as usize {
            return Err(NuxStatus::InvalidArgument);
        }
        Ok(())
    }
}

fn validate_v12_version(required_major: u16, minimum_minor: u16) -> Result<(), NuxStatus> {
    if required_major == NUX_RUNTIME_ABI_MAJOR && minimum_minor == NUX_FLOW_SESSION_ABI_MINOR {
        Ok(())
    } else {
        Err(NuxStatus::AbiMismatch)
    }
}

fn checked_count(count: u64, maximum: u64) -> Result<usize, NuxStatus> {
    if count > maximum {
        return Err(NuxStatus::InvalidArgument);
    }
    usize::try_from(count).map_err(|_| NuxStatus::InvalidArgument)
}

unsafe fn copy_array<T: Copy>(
    values: *const T,
    count: u64,
    maximum: u64,
) -> Result<Vec<T>, NuxStatus> {
    let count = checked_count(count, maximum)?;
    if count != 0 && values.is_null() {
        return Err(NuxStatus::NullArgument);
    }
    let byte_length = count
        .checked_mul(std::mem::size_of::<T>())
        .ok_or(NuxStatus::InvalidArgument)?;
    if byte_length > isize::MAX as usize {
        return Err(NuxStatus::InvalidArgument);
    }
    if count == 0 {
        return Ok(Vec::new());
    }
    // SAFETY: the FFI caller promises a readable array of the declared count
    // for this synchronous call. Every element and nested view is copied before
    // the private session seam can retain the request.
    Ok(unsafe { slice::from_raw_parts(values, count) }.to_vec())
}

fn copy_bytes(
    view: NuxByteView,
    maximum: u64,
    budget: &mut PayloadBudget,
) -> Result<Vec<u8>, NuxStatus> {
    let maximum = usize::try_from(maximum).map_err(|_| NuxStatus::InvalidArgument)?;
    let bytes = byte_vec(view, maximum)?;
    budget.charge(bytes.len())?;
    Ok(bytes)
}

fn copy_required_utf8(
    view: NuxByteView,
    maximum: u64,
    budget: &mut PayloadBudget,
) -> Result<Vec<u8>, NuxStatus> {
    let bytes = copy_bytes(view, maximum, budget)?;
    if bytes.is_empty() || std::str::from_utf8(&bytes).is_err() {
        return Err(NuxStatus::InvalidArgument);
    }
    Ok(bytes)
}

fn copy_optional_utf8(
    view: NuxByteView,
    maximum: u64,
    budget: &mut PayloadBudget,
) -> Result<Option<Vec<u8>>, NuxStatus> {
    if view.data.is_null() && view.len == 0 {
        return Ok(None);
    }
    copy_required_utf8(view, maximum, budget).map(Some)
}

unsafe fn copy_configured_session_descriptor(
    descriptor: *const NuxFlowConfiguredSessionDescriptor,
) -> Result<OwnedConfiguredSessionDescriptor, NuxStatus> {
    if descriptor.is_null() {
        return Err(NuxStatus::NullArgument);
    }
    let struct_size = unsafe { read_struct_size(descriptor) };
    if struct_size < size_u32::<NuxFlowConfiguredSessionDescriptor>() {
        return Err(NuxStatus::InvalidArgument);
    }
    let descriptor = unsafe { descriptor.read() };
    validate_v12_version(descriptor.required_abi_major, descriptor.minimum_abi_minor)?;
    let mut budget = PayloadBudget::default();
    let artboard_name = copy_optional_utf8(
        descriptor.artboard_name,
        NUX_FLOW_MAX_ID_BYTE_LENGTH,
        &mut budget,
    )?
    .map(String::from_utf8)
    .transpose()
    .map_err(|_| NuxStatus::InvalidArgument)?;
    let player_name = copy_optional_utf8(
        descriptor.player_name,
        NUX_FLOW_MAX_ID_BYTE_LENGTH,
        &mut budget,
    )?
    .map(String::from_utf8)
    .transpose()
    .map_err(|_| NuxStatus::InvalidArgument)?;
    Ok(OwnedConfiguredSessionDescriptor {
        artboard_name,
        player_name,
    })
}

unsafe fn copy_value_arena(
    arena: *const NuxFlowValueArena,
    budget: &mut PayloadBudget,
) -> Result<OwnedValueArena, NuxStatus> {
    if arena.is_null() {
        return Ok(OwnedValueArena::default());
    }
    if unsafe { read_struct_size(arena) } < size_u32::<NuxFlowValueArena>() {
        return Err(NuxStatus::InvalidArgument);
    }
    let arena = unsafe { arena.read() };
    let raw_nodes =
        unsafe { copy_array(arena.nodes, arena.node_count, NUX_FLOW_MAX_BATCH_ITEM_COUNT)? };
    let raw_edges =
        unsafe { copy_array(arena.edges, arena.edge_count, NUX_FLOW_MAX_VALUE_EDGE_COUNT)? };
    let mut nodes = Vec::with_capacity(raw_nodes.len());
    for node in raw_nodes {
        if node.struct_size != size_u32::<NuxFlowValueNode>() {
            return Err(NuxStatus::InvalidArgument);
        }
        let string_value = copy_bytes(node.string_value, NUX_FLOW_MAX_STRING_BYTE_LENGTH, budget)?;
        let schema_id = copy_bytes(node.schema_id, NUX_FLOW_MAX_ID_BYTE_LENGTH, budget)?;
        let instance_id = match node.has_instance_id {
            0 if node.instance_id == 0 => None,
            1 if node.instance_id != 0 => Some(node.instance_id),
            _ => return Err(NuxStatus::InvalidArgument),
        };
        if (!string_value.is_empty() && std::str::from_utf8(&string_value).is_err())
            || (!schema_id.is_empty() && std::str::from_utf8(&schema_id).is_err())
        {
            return Err(NuxStatus::InvalidArgument);
        }
        if node.bool_value > 1 {
            return Err(NuxStatus::InvalidArgument);
        }
        let first_edge =
            usize::try_from(node.first_edge).map_err(|_| NuxStatus::InvalidArgument)?;
        let edge_count =
            usize::try_from(node.edge_count).map_err(|_| NuxStatus::InvalidArgument)?;
        let edge_end = first_edge
            .checked_add(edge_count)
            .ok_or(NuxStatus::InvalidArgument)?;
        if edge_count > NUX_FLOW_MAX_LIST_ITEM_COUNT as usize || edge_end > raw_edges.len() {
            return Err(NuxStatus::InvalidArgument);
        }
        let has_edges = edge_count != 0;
        let has_canonical_edge_start = has_edges || node.first_edge == 0;
        let number_is_zero = node.number_value.to_bits() == 0;
        let fields_are_canonical = match node.kind {
            NUX_FLOW_VALUE_KIND_NULL => {
                number_is_zero
                    && node.color_value == 0
                    && node.bool_value == 0
                    && node.identity_value == 0
                    && string_value.is_empty()
                    && instance_id.is_none()
                    && schema_id.is_empty()
                    && !has_edges
                    && has_canonical_edge_start
            }
            NUX_FLOW_VALUE_KIND_STRING => {
                number_is_zero
                    && node.color_value == 0
                    && node.bool_value == 0
                    && node.identity_value == 0
                    && instance_id.is_none()
                    && schema_id.is_empty()
                    && !has_edges
                    && has_canonical_edge_start
            }
            NUX_FLOW_VALUE_KIND_NUMBER => {
                node.number_value.is_finite()
                    && node.number_value.abs() <= f64::from(f32::MAX)
                    && node.color_value == 0
                    && node.bool_value == 0
                    && node.identity_value == 0
                    && string_value.is_empty()
                    && instance_id.is_none()
                    && schema_id.is_empty()
                    && !has_edges
                    && has_canonical_edge_start
            }
            NUX_FLOW_VALUE_KIND_BOOL => {
                number_is_zero
                    && node.color_value == 0
                    && node.identity_value == 0
                    && string_value.is_empty()
                    && instance_id.is_none()
                    && schema_id.is_empty()
                    && !has_edges
                    && has_canonical_edge_start
            }
            NUX_FLOW_VALUE_KIND_COLOR => {
                number_is_zero
                    && node.bool_value == 0
                    && node.identity_value == 0
                    && string_value.is_empty()
                    && instance_id.is_none()
                    && schema_id.is_empty()
                    && !has_edges
                    && has_canonical_edge_start
            }
            NUX_FLOW_VALUE_KIND_ENUM | NUX_FLOW_VALUE_KIND_IMAGE => {
                number_is_zero
                    && node.color_value == 0
                    && node.bool_value == 0
                    && string_value.is_empty()
                    && instance_id.is_none()
                    && schema_id.is_empty()
                    && !has_edges
                    && has_canonical_edge_start
            }
            NUX_FLOW_VALUE_KIND_OBJECT => {
                number_is_zero
                    && node.color_value == 0
                    && node.bool_value == 0
                    && node.identity_value == 0
                    && string_value.is_empty()
                    && instance_id.is_none()
                    && !schema_id.is_empty()
                    && has_canonical_edge_start
            }
            NUX_FLOW_VALUE_KIND_VIEW_MODEL => {
                number_is_zero
                    && node.color_value == 0
                    && node.bool_value == 0
                    && node.identity_value == 0
                    && string_value.is_empty()
                    && instance_id.is_some()
                    && !schema_id.is_empty()
                    && has_canonical_edge_start
            }
            NUX_FLOW_VALUE_KIND_LIST => {
                number_is_zero
                    && node.color_value == 0
                    && node.bool_value == 0
                    && node.identity_value == 0
                    && string_value.is_empty()
                    && instance_id.is_none()
                    && schema_id.is_empty()
                    && has_canonical_edge_start
            }
            _ => false,
        };
        if !fields_are_canonical {
            return Err(NuxStatus::InvalidArgument);
        }
        nodes.push(OwnedValueNode {
            kind: node.kind,
            number_value: node.number_value,
            color_value: node.color_value,
            bool_value: node.bool_value == 1,
            first_edge: node.first_edge,
            edge_count: node.edge_count,
            instance_id,
            identity_value: node.identity_value,
            string_value,
            schema_id,
        });
    }
    let mut edges = Vec::with_capacity(raw_edges.len());
    for edge in raw_edges {
        if edge.struct_size != size_u32::<NuxFlowValueEdge>() {
            return Err(NuxStatus::InvalidArgument);
        }
        let node_index =
            usize::try_from(edge.node_index).map_err(|_| NuxStatus::InvalidArgument)?;
        if node_index >= nodes.len() {
            return Err(NuxStatus::InvalidArgument);
        }
        let key = copy_bytes(edge.key, NUX_FLOW_MAX_PATH_BYTE_LENGTH, budget)?;
        if !key.is_empty() && std::str::from_utf8(&key).is_err() {
            return Err(NuxStatus::InvalidArgument);
        }
        edges.push(OwnedValueEdge {
            node_index: edge.node_index,
            key,
        });
    }
    validate_value_graph(&nodes, &edges)?;
    Ok(OwnedValueArena { nodes, edges })
}

fn validate_value_graph(
    nodes: &[OwnedValueNode],
    edges: &[OwnedValueEdge],
) -> Result<(), NuxStatus> {
    for node in nodes {
        let start = node.first_edge as usize;
        let end = start
            .checked_add(node.edge_count as usize)
            .ok_or(NuxStatus::InvalidArgument)?;
        let node_edges = edges.get(start..end).ok_or(NuxStatus::InvalidArgument)?;
        match node.kind {
            NUX_FLOW_VALUE_KIND_OBJECT | NUX_FLOW_VALUE_KIND_VIEW_MODEL => {
                let mut keys = HashSet::with_capacity(node_edges.len());
                for edge in node_edges {
                    if edge.key.is_empty() || !keys.insert(edge.key.as_slice()) {
                        return Err(NuxStatus::InvalidArgument);
                    }
                }
            }
            NUX_FLOW_VALUE_KIND_LIST => {
                if node_edges.iter().any(|edge| !edge.key.is_empty()) {
                    return Err(NuxStatus::InvalidArgument);
                }
            }
            _ if !node_edges.is_empty() => return Err(NuxStatus::InvalidArgument),
            _ => {}
        }
    }
    let mut states = vec![0_u8; nodes.len()];
    let mut heights = vec![0_u32; nodes.len()];
    for node_index in 0..nodes.len() {
        validate_value_height(node_index, 0, nodes, edges, &mut states, &mut heights)?;
    }
    Ok(())
}

fn validate_value_height(
    node_index: usize,
    depth: u32,
    nodes: &[OwnedValueNode],
    edges: &[OwnedValueEdge],
    states: &mut [u8],
    heights: &mut [u32],
) -> Result<u32, NuxStatus> {
    if depth > NUX_FLOW_MAX_VALUE_DEPTH {
        return Err(NuxStatus::InvalidArgument);
    }
    match states
        .get(node_index)
        .copied()
        .ok_or(NuxStatus::InvalidArgument)?
    {
        1 => return Err(NuxStatus::InvalidArgument),
        2 => {
            return heights
                .get(node_index)
                .copied()
                .ok_or(NuxStatus::InvalidArgument);
        }
        _ => {}
    }
    *states
        .get_mut(node_index)
        .ok_or(NuxStatus::InvalidArgument)? = 1;
    let node = nodes.get(node_index).ok_or(NuxStatus::InvalidArgument)?;
    let start = node.first_edge as usize;
    let end = start
        .checked_add(node.edge_count as usize)
        .ok_or(NuxStatus::InvalidArgument)?;
    let mut height = 0_u32;
    for edge in edges.get(start..end).ok_or(NuxStatus::InvalidArgument)? {
        let child_height = validate_value_height(
            edge.node_index as usize,
            depth.checked_add(1).ok_or(NuxStatus::InvalidArgument)?,
            nodes,
            edges,
            states,
            heights,
        )?;
        height = height.max(
            child_height
                .checked_add(1)
                .ok_or(NuxStatus::InvalidArgument)?,
        );
        if height > NUX_FLOW_MAX_VALUE_DEPTH {
            return Err(NuxStatus::InvalidArgument);
        }
    }
    *states
        .get_mut(node_index)
        .ok_or(NuxStatus::InvalidArgument)? = 2;
    *heights
        .get_mut(node_index)
        .ok_or(NuxStatus::InvalidArgument)? = height;
    Ok(height)
}

unsafe fn copy_state_batch(batch: *const NuxFlowStateBatch) -> Result<OwnedStateBatch, NuxStatus> {
    if batch.is_null() {
        return Err(NuxStatus::NullArgument);
    }
    if unsafe { read_struct_size(batch) } < size_u32::<NuxFlowStateBatch>() {
        return Err(NuxStatus::InvalidArgument);
    }
    let batch = unsafe { batch.read() };
    let mut budget = PayloadBudget::default();
    let host_mutation_id = match batch.has_host_mutation_id {
        0 if batch.host_mutation_id == 0 => None,
        1 => Some(batch.host_mutation_id),
        _ => return Err(NuxStatus::InvalidArgument),
    };
    let value_arena = unsafe { copy_value_arena(batch.value_arena, &mut budget)? };
    let raw_instances = unsafe {
        copy_array(
            batch.new_instances,
            batch.new_instance_count,
            NUX_FLOW_MAX_INSTANCE_COUNT,
        )?
    };
    let raw_mutations = unsafe {
        copy_array(
            batch.mutations,
            batch.mutation_count,
            NUX_FLOW_MAX_BATCH_ITEM_COUNT,
        )?
    };
    if raw_instances
        .len()
        .checked_add(raw_mutations.len())
        .is_none_or(|item_count| item_count > NUX_FLOW_MAX_BATCH_ITEM_COUNT as usize)
    {
        return Err(NuxStatus::InvalidArgument);
    }
    if raw_instances.is_empty() && raw_mutations.is_empty() {
        return Err(NuxStatus::InvalidArgument);
    }
    let mut new_instances = Vec::with_capacity(raw_instances.len());
    let mut local_ids = HashSet::with_capacity(raw_instances.len());
    for instance in raw_instances {
        if instance.struct_size != size_u32::<NuxFlowNewInstance>() {
            return Err(NuxStatus::InvalidArgument);
        }
        if !local_ids.insert(instance.local_id) {
            return Err(NuxStatus::InvalidArgument);
        }
        let schema_name = copy_required_utf8(
            instance.schema_name,
            NUX_FLOW_MAX_ID_BYTE_LENGTH,
            &mut budget,
        )?;
        let authored_instance_name = copy_optional_utf8(
            instance.authored_instance_name,
            NUX_FLOW_MAX_ID_BYTE_LENGTH,
            &mut budget,
        )?;
        new_instances.push(OwnedNewInstance {
            local_id: instance.local_id,
            schema_name,
            authored_instance_name,
        });
    }
    let mut mutations = Vec::with_capacity(raw_mutations.len());
    for mutation in raw_mutations {
        if mutation.struct_size != size_u32::<NuxFlowStateMutation>() {
            return Err(NuxStatus::InvalidArgument);
        }
        let path = copy_optional_utf8(mutation.path, NUX_FLOW_MAX_PATH_BYTE_LENGTH, &mut budget)?;
        let input_name = copy_optional_utf8(
            mutation.input_name,
            NUX_FLOW_MAX_ID_BYTE_LENGTH,
            &mut budget,
        )?;
        let value_root_index = optional_value_root(mutation.value_root_index, &value_arena)?;
        let (instance, item, valid_shape) = match mutation.kind {
            NUX_FLOW_STATE_MUTATION_KIND_SET => (
                Some(copy_instance_reference(mutation.instance)?),
                None,
                value_root_index.is_some()
                    && path.is_some()
                    && input_name.is_none()
                    && instance_reference_is_zero(mutation.item)
                    && mutation.index == 0
                    && mutation.other_index == 0,
            ),
            NUX_FLOW_STATE_MUTATION_KIND_TRIGGER => (
                Some(copy_instance_reference(mutation.instance)?),
                None,
                value_root_index.is_none()
                    && path.is_some()
                    && input_name.is_none()
                    && instance_reference_is_zero(mutation.item)
                    && mutation.index == 0
                    && mutation.other_index == 0,
            ),
            NUX_FLOW_STATE_MUTATION_KIND_LIST_INSERT | NUX_FLOW_STATE_MUTATION_KIND_LIST_SET => (
                Some(copy_instance_reference(mutation.instance)?),
                Some(copy_instance_reference(mutation.item)?),
                value_root_index.is_none()
                    && path.is_some()
                    && input_name.is_none()
                    && mutation.other_index == 0,
            ),
            NUX_FLOW_STATE_MUTATION_KIND_LIST_REMOVE => (
                Some(copy_instance_reference(mutation.instance)?),
                None,
                value_root_index.is_none()
                    && path.is_some()
                    && input_name.is_none()
                    && instance_reference_is_zero(mutation.item)
                    && mutation.other_index == 0,
            ),
            NUX_FLOW_STATE_MUTATION_KIND_LIST_SWAP | NUX_FLOW_STATE_MUTATION_KIND_LIST_MOVE => (
                Some(copy_instance_reference(mutation.instance)?),
                None,
                value_root_index.is_none()
                    && path.is_some()
                    && input_name.is_none()
                    && instance_reference_is_zero(mutation.item),
            ),
            NUX_FLOW_STATE_MUTATION_KIND_LIST_CLEAR => (
                Some(copy_instance_reference(mutation.instance)?),
                None,
                value_root_index.is_none()
                    && path.is_some()
                    && input_name.is_none()
                    && instance_reference_is_zero(mutation.item)
                    && mutation.index == 0
                    && mutation.other_index == 0,
            ),
            NUX_FLOW_STATE_MUTATION_KIND_SET_INPUT_BOOL => (
                None,
                None,
                value_root_has_kind(value_root_index, &value_arena, NUX_FLOW_VALUE_KIND_BOOL)
                    && input_name.is_some()
                    && path.is_none()
                    && instance_reference_is_zero(mutation.instance)
                    && instance_reference_is_zero(mutation.item)
                    && mutation.index == 0
                    && mutation.other_index == 0,
            ),
            NUX_FLOW_STATE_MUTATION_KIND_SET_INPUT_NUMBER => (
                None,
                None,
                value_root_has_kind(value_root_index, &value_arena, NUX_FLOW_VALUE_KIND_NUMBER)
                    && input_name.is_some()
                    && path.is_none()
                    && instance_reference_is_zero(mutation.instance)
                    && instance_reference_is_zero(mutation.item)
                    && mutation.index == 0
                    && mutation.other_index == 0,
            ),
            NUX_FLOW_STATE_MUTATION_KIND_FIRE_INPUT_TRIGGER => (
                None,
                None,
                value_root_index.is_none()
                    && input_name.is_some()
                    && path.is_none()
                    && instance_reference_is_zero(mutation.instance)
                    && instance_reference_is_zero(mutation.item)
                    && mutation.index == 0
                    && mutation.other_index == 0,
            ),
            _ => (None, None, false),
        };
        if !valid_shape {
            return Err(NuxStatus::InvalidArgument);
        }
        if matches!(instance, Some(OwnedInstanceReference::New(local_id)) if !local_ids.contains(&local_id))
        {
            return Err(NuxStatus::InvalidArgument);
        }
        if matches!(item, Some(OwnedInstanceReference::New(local_id)) if !local_ids.contains(&local_id))
        {
            return Err(NuxStatus::InvalidArgument);
        }
        mutations.push(OwnedStateMutation {
            kind: mutation.kind,
            instance,
            item,
            path,
            input_name,
            value_root_index,
            index: mutation.index,
            other_index: mutation.other_index,
        });
    }
    Ok(OwnedStateBatch {
        host_mutation_id,
        value_arena,
        new_instances,
        mutations,
    })
}

fn instance_reference_is_zero(reference: NuxFlowInstanceReference) -> bool {
    reference.kind == 0 && reference.local_id == 0 && reference.instance_id == 0
}

fn copy_instance_reference(
    reference: NuxFlowInstanceReference,
) -> Result<OwnedInstanceReference, NuxStatus> {
    match reference.kind {
        NUX_FLOW_INSTANCE_REFERENCE_KIND_EXISTING
            if reference.instance_id != 0 && reference.local_id == 0 =>
        {
            Ok(OwnedInstanceReference::Existing(reference.instance_id))
        }
        NUX_FLOW_INSTANCE_REFERENCE_KIND_NEW if reference.instance_id == 0 => {
            Ok(OwnedInstanceReference::New(reference.local_id))
        }
        _ => Err(NuxStatus::InvalidArgument),
    }
}

fn optional_value_root(root_index: u32, arena: &OwnedValueArena) -> Result<Option<u32>, NuxStatus> {
    if root_index == NO_VALUE_ROOT {
        return Ok(None);
    }
    let index = usize::try_from(root_index).map_err(|_| NuxStatus::InvalidArgument)?;
    if index >= arena.nodes.len() {
        return Err(NuxStatus::InvalidArgument);
    }
    Ok(Some(root_index))
}

fn value_root_has_kind(
    root_index: Option<u32>,
    arena: &OwnedValueArena,
    expected_kind: NuxFlowValueKind,
) -> bool {
    root_index
        .and_then(|index| arena.nodes.get(index as usize))
        .is_some_and(|node| node.kind == expected_kind)
}

unsafe fn copy_pointer_batch(
    batch: *const NuxFlowPointerBatch,
) -> Result<Vec<OwnedPointerEvent>, NuxStatus> {
    if batch.is_null() {
        return Err(NuxStatus::NullArgument);
    }
    if unsafe { read_struct_size(batch) } < size_u32::<NuxFlowPointerBatch>() {
        return Err(NuxStatus::InvalidArgument);
    }
    let batch = unsafe { batch.read() };
    let raw_events =
        unsafe { copy_array(batch.events, batch.event_count, NUX_FLOW_MAX_POINTER_COUNT)? };
    if raw_events.is_empty() {
        return Err(NuxStatus::InvalidArgument);
    }
    let mut events = Vec::with_capacity(raw_events.len());
    for event in raw_events {
        if event.struct_size != size_u32::<NuxFlowPointerEvent>()
            || !event.x.is_finite()
            || !event.y.is_finite()
            || event.pointer_id <= 0
            || !matches!(
                event.kind,
                NUX_FLOW_POINTER_EVENT_KIND_DOWN
                    | NUX_FLOW_POINTER_EVENT_KIND_MOVE
                    | NUX_FLOW_POINTER_EVENT_KIND_UP
                    | NUX_FLOW_POINTER_EVENT_KIND_CANCEL
                    | NUX_FLOW_POINTER_EVENT_KIND_EXIT
            )
        {
            return Err(NuxStatus::InvalidArgument);
        }
        events.push(OwnedPointerEvent {
            kind: event.kind,
            pointer_id: event.pointer_id,
            x: event.x,
            y: event.y,
        });
    }
    Ok(events)
}

unsafe fn copy_query_batch(batch: *const NuxFlowQueryBatch) -> Result<Vec<OwnedQuery>, NuxStatus> {
    if batch.is_null() {
        return Err(NuxStatus::NullArgument);
    }
    if unsafe { read_struct_size(batch) } < size_u32::<NuxFlowQueryBatch>() {
        return Err(NuxStatus::InvalidArgument);
    }
    let batch = unsafe { batch.read() };
    let raw_queries =
        unsafe { copy_array(batch.queries, batch.query_count, NUX_FLOW_MAX_QUERY_COUNT)? };
    if raw_queries.is_empty() {
        return Err(NuxStatus::InvalidArgument);
    }
    let mut queries = Vec::with_capacity(raw_queries.len());
    for query in raw_queries {
        if query.struct_size != size_u32::<NuxFlowQuery>() {
            return Err(NuxStatus::InvalidArgument);
        }
        if !matches!(
            query.kind,
            NUX_FLOW_QUERY_KIND_BOOTSTRAP
                | NUX_FLOW_QUERY_KIND_VALUES
                | NUX_FLOW_QUERY_KIND_CATALOG
                | NUX_FLOW_QUERY_KIND_PLAYER_INPUTS
        ) {
            return Err(NuxStatus::InvalidArgument);
        }
        queries.push(OwnedQuery { kind: query.kind });
    }
    Ok(queries)
}

struct PendingAdvanceCompletion {
    callback: Option<unsafe extern "C" fn(context: *mut c_void)>,
    context_identity: usize,
}

impl PendingAdvanceCompletion {
    fn from_operation(operation: &NuxFlowAdvanceOperation) -> Result<Self, NuxStatus> {
        if operation.completion_callback.is_some() == operation.completion_context.is_null() {
            return Err(NuxStatus::InvalidArgument);
        }
        Ok(Self {
            callback: operation.completion_callback,
            context_identity: operation.completion_context.expose_provenance(),
        })
    }

    fn disarm(mut self) {
        self.callback = None;
    }
}

impl Drop for PendingAdvanceCompletion {
    fn drop(&mut self) {
        if let Some(callback) = self.callback.take() {
            unsafe {
                callback(ptr::with_exposed_provenance_mut(self.context_identity));
            }
        }
    }
}

unsafe fn copy_advance_operation(
    operation: *const NuxFlowAdvanceOperation,
) -> Result<OwnedAdvanceOperation, NuxStatus> {
    if operation.is_null() {
        return Err(NuxStatus::NullArgument);
    }
    if unsafe { read_struct_size(operation) } < size_u32::<NuxFlowAdvanceOperation>() {
        return Err(NuxStatus::InvalidArgument);
    }
    let operation = unsafe { operation.read() };
    let completion = PendingAdvanceCompletion::from_operation(&operation)?;
    let render = match operation.render {
        0 => false,
        1 => true,
        _ => return Err(NuxStatus::InvalidArgument),
    };
    if !operation.timestamp_seconds.is_finite()
        || operation.timestamp_seconds < 0.0
        || !operation.delta_seconds.is_finite()
        || operation.delta_seconds < 0.0
        || (!render && !operation.apple_drawable.is_null())
        || (operation.completion_callback.is_some() && operation.apple_drawable.is_null())
    {
        return Err(NuxStatus::InvalidArgument);
    }
    let owned = OwnedAdvanceOperation {
        timestamp_seconds: operation.timestamp_seconds,
        delta_seconds: operation.delta_seconds,
        render,
        drawable_identity: operation.apple_drawable.expose_provenance(),
        completion_context_identity: operation.completion_context.expose_provenance(),
        completion_callback: operation.completion_callback,
    };
    completion.disarm();
    Ok(owned)
}

unsafe fn copy_session_operation(
    operation: *const NuxFlowSessionOperation,
) -> Result<OwnedSessionOperation, NuxStatus> {
    if operation.is_null() {
        return Err(NuxStatus::NullArgument);
    }
    if unsafe { read_struct_size(operation) } < size_u32::<NuxFlowSessionOperation>() {
        return Err(NuxStatus::InvalidArgument);
    }
    let operation = unsafe { operation.read() };
    validate_v12_version(operation.required_abi_major, operation.minimum_abi_minor)?;
    let selected_payload_count = [
        !operation.state_batch.is_null(),
        !operation.pointer_batch.is_null(),
        !operation.advance.is_null(),
        !operation.query_batch.is_null(),
    ]
    .into_iter()
    .filter(|selected| *selected)
    .count();
    if selected_payload_count != 1 {
        return Err(NuxStatus::InvalidArgument);
    }
    match operation.kind {
        NUX_FLOW_SESSION_OPERATION_KIND_STATE_BATCH
            if !operation.state_batch.is_null()
                && operation.pointer_batch.is_null()
                && operation.advance.is_null()
                && operation.query_batch.is_null() =>
        {
            unsafe { copy_state_batch(operation.state_batch) }
                .map(OwnedSessionOperation::StateBatch)
        }
        NUX_FLOW_SESSION_OPERATION_KIND_POINTER_BATCH
            if operation.state_batch.is_null()
                && !operation.pointer_batch.is_null()
                && operation.advance.is_null()
                && operation.query_batch.is_null() =>
        {
            unsafe { copy_pointer_batch(operation.pointer_batch) }
                .map(OwnedSessionOperation::PointerBatch)
        }
        NUX_FLOW_SESSION_OPERATION_KIND_ADVANCE
            if operation.state_batch.is_null()
                && operation.pointer_batch.is_null()
                && !operation.advance.is_null()
                && operation.query_batch.is_null() =>
        {
            unsafe { copy_advance_operation(operation.advance) }.map(OwnedSessionOperation::Advance)
        }
        NUX_FLOW_SESSION_OPERATION_KIND_QUERY
            if operation.state_batch.is_null()
                && operation.pointer_batch.is_null()
                && operation.advance.is_null()
                && !operation.query_batch.is_null() =>
        {
            unsafe { copy_query_batch(operation.query_batch) }.map(OwnedSessionOperation::Query)
        }
        _ => Err(NuxStatus::InvalidArgument),
    }
}

#[derive(Debug, Clone)]
struct OwnedPlayerMetadata {
    kind: NuxFlowPlayerKind,
    selection: NuxFlowPlayerSelection,
    player_index: Option<u32>,
    artboard_name: Vec<u8>,
    player_name: Vec<u8>,
    min_x: f32,
    min_y: f32,
    max_x: f32,
    max_y: f32,
}

#[derive(Debug, Clone)]
struct OwnedPlayerInput {
    kind: NuxFlowPlayerInputKind,
    value_root_index: u32,
    name: Vec<u8>,
}

#[derive(Debug, Clone)]
struct OwnedSchema {
    first_property: u32,
    property_count: u32,
    schema_id: Vec<u8>,
    name: Vec<u8>,
}

#[derive(Debug, Clone)]
struct OwnedSchemaProperty {
    kind: NuxFlowSchemaPropertyKind,
    schema_id: Vec<u8>,
    property_id: Vec<u8>,
    name: Vec<u8>,
}

#[derive(Debug, Clone)]
struct OwnedInstanceTemplate {
    authored_index: u32,
    schema_id: Vec<u8>,
    authored_name: Vec<u8>,
}

#[derive(Debug, Clone)]
struct OwnedInstance {
    value_root_index: Option<u32>,
    instance_id: u64,
    is_root: bool,
    schema_id: Vec<u8>,
    name: Vec<u8>,
}

#[derive(Debug, Clone, Copy)]
struct OwnedValueRoot {
    value_root_index: u32,
    instance_id: u64,
}

#[derive(Debug, Clone, Copy)]
struct OwnedCreatedInstance {
    local_id: u32,
    instance_id: u64,
}

#[derive(Debug, Clone)]
struct OwnedOutput {
    phase: NuxFlowOutputPhase,
    kind: NuxFlowOutputKind,
    payload_root_index: Option<u32>,
    sequence: u64,
    cycle: u64,
    origin_mutation_id: Option<u64>,
    instance_id: Option<u64>,
    event_type: u32,
    first_event_property: u32,
    event_property_count: u32,
    delay_seconds: f32,
    name: Vec<u8>,
    path: Vec<u8>,
    payload: Vec<u8>,
}

#[derive(Debug, Clone)]
struct OwnedEventProperty {
    value_root_index: Option<u32>,
    trigger_count: Option<u64>,
    name: Vec<u8>,
}

struct FlowSessionResultHandle {
    status: NuxStatus,
    surface_disposition: NuxSurfaceDisposition,
    is_dirty: bool,
    is_settled: bool,
    wake_after: Option<f64>,
    player_metadata: Option<OwnedPlayerMetadata>,
    player_inputs: Vec<OwnedPlayerInput>,
    schemas: Vec<OwnedSchema>,
    schema_properties: Vec<OwnedSchemaProperty>,
    instance_templates: Vec<OwnedInstanceTemplate>,
    instances: Vec<OwnedInstance>,
    value_arena: OwnedValueArena,
    value_roots: Vec<OwnedValueRoot>,
    created_instances: Vec<OwnedCreatedInstance>,
    outputs: Vec<OwnedOutput>,
    event_properties: Vec<OwnedEventProperty>,
    diagnostics: Vec<OwnedDiagnostic>,
}

impl FlowSessionResultHandle {
    fn empty_success() -> Self {
        Self {
            status: NuxStatus::Ok,
            surface_disposition: NuxSurfaceDisposition::None,
            is_dirty: false,
            is_settled: false,
            wake_after: None,
            player_metadata: None,
            player_inputs: Vec::new(),
            schemas: Vec::new(),
            schema_properties: Vec::new(),
            instance_templates: Vec::new(),
            instances: Vec::new(),
            value_arena: OwnedValueArena::default(),
            value_roots: Vec::new(),
            created_instances: Vec::new(),
            outputs: Vec::new(),
            event_properties: Vec::new(),
            diagnostics: Vec::new(),
        }
    }

    fn failure(status: NuxStatus, diagnostic: impl Into<Vec<u8>>) -> Self {
        let message = diagnostic.into();
        Self {
            status,
            surface_disposition: NuxSurfaceDisposition::Fatal,
            is_dirty: false,
            is_settled: false,
            wake_after: None,
            player_metadata: None,
            player_inputs: Vec::new(),
            schemas: Vec::new(),
            schema_properties: Vec::new(),
            instance_templates: Vec::new(),
            instances: Vec::new(),
            value_arena: OwnedValueArena::default(),
            value_roots: Vec::new(),
            created_instances: Vec::new(),
            outputs: Vec::new(),
            event_properties: Vec::new(),
            diagnostics: vec![OwnedDiagnostic {
                severity: NUX_DIAGNOSTIC_SEVERITY_FATAL,
                code: diagnostic_code_for_status(status).to_vec(),
                message,
            }],
        }
    }

    fn validate(&self) -> Result<(), NuxStatus> {
        if let Some(wake_after) = self.wake_after
            && (!wake_after.is_finite() || wake_after < 0.0)
        {
            return Err(NuxStatus::RuntimeError);
        }
        if self.schemas.len() > NUX_FLOW_MAX_INSTANCE_COUNT as usize
            || self.player_inputs.len() > NUX_FLOW_MAX_BATCH_ITEM_COUNT as usize
            || self.schema_properties.len() > NUX_FLOW_MAX_BATCH_ITEM_COUNT as usize
            || self.instance_templates.len() > NUX_FLOW_MAX_INSTANCE_COUNT as usize
            || self.instances.len() > NUX_FLOW_MAX_INSTANCE_COUNT as usize
            || self.value_roots.len() > NUX_FLOW_MAX_INSTANCE_COUNT as usize
            || self.created_instances.len() > NUX_FLOW_MAX_INSTANCE_COUNT as usize
            || self.outputs.len() > NUX_FLOW_MAX_OUTPUT_COUNT as usize
            || self.event_properties.len()
                > (NUX_FLOW_MAX_OUTPUT_COUNT * NUX_FLOW_MAX_EVENT_PROPERTY_COUNT) as usize
            || self.value_arena.nodes.len() > NUX_FLOW_MAX_BATCH_ITEM_COUNT as usize
            || self.value_arena.edges.len() > NUX_FLOW_MAX_VALUE_EDGE_COUNT as usize
        {
            return Err(NuxStatus::RuntimeError);
        }
        for node in &self.value_arena.nodes {
            validate_result_value_node(node)?;
        }
        validate_value_graph(&self.value_arena.nodes, &self.value_arena.edges)
            .map_err(|_| NuxStatus::RuntimeError)?;
        let mut payload_bytes = 0usize;
        if let Some(metadata) = self.player_metadata.as_ref() {
            let selection_is_consistent = match metadata.selection {
                NUX_FLOW_PLAYER_SELECTION_EXPLICIT_STATE_MACHINE
                | NUX_FLOW_PLAYER_SELECTION_AUTHORED_DEFAULT_STATE_MACHINE
                | NUX_FLOW_PLAYER_SELECTION_FIRST_STATE_MACHINE => {
                    metadata.kind == NUX_FLOW_PLAYER_KIND_STATE_MACHINE
                        && metadata.player_index.is_some()
                }
                NUX_FLOW_PLAYER_SELECTION_FIRST_ANIMATION => {
                    metadata.kind == NUX_FLOW_PLAYER_KIND_LINEAR_ANIMATION
                        && metadata.player_index.is_some()
                }
                NUX_FLOW_PLAYER_SELECTION_STATIC => {
                    metadata.kind == NUX_FLOW_PLAYER_KIND_STATIC && metadata.player_index.is_none()
                }
                _ => false,
            };
            if !selection_is_consistent
                || !metadata.min_x.is_finite()
                || !metadata.min_y.is_finite()
                || !metadata.max_x.is_finite()
                || !metadata.max_y.is_finite()
                || metadata.max_x < metadata.min_x
                || metadata.max_y < metadata.min_y
            {
                return Err(NuxStatus::RuntimeError);
            }
            charge_result_utf8(
                &mut payload_bytes,
                &metadata.artboard_name,
                NUX_FLOW_MAX_ID_BYTE_LENGTH,
            )?;
            charge_result_utf8(
                &mut payload_bytes,
                &metadata.player_name,
                NUX_FLOW_MAX_ID_BYTE_LENGTH,
            )?;
        }
        for input in &self.player_inputs {
            let expected_value_kind = match input.kind {
                NUX_FLOW_PLAYER_INPUT_KIND_BOOL | NUX_FLOW_PLAYER_INPUT_KIND_TRIGGER => {
                    NUX_FLOW_VALUE_KIND_BOOL
                }
                NUX_FLOW_PLAYER_INPUT_KIND_NUMBER => NUX_FLOW_VALUE_KIND_NUMBER,
                _ => return Err(NuxStatus::RuntimeError),
            };
            if self
                .value_arena
                .nodes
                .get(input.value_root_index as usize)
                .is_none_or(|node| node.kind != expected_value_kind)
            {
                return Err(NuxStatus::RuntimeError);
            }
            charge_result_utf8(&mut payload_bytes, &input.name, NUX_FLOW_MAX_ID_BYTE_LENGTH)?;
        }
        for schema in &self.schemas {
            let first = schema.first_property as usize;
            let count = schema.property_count as usize;
            let end = first.checked_add(count).ok_or(NuxStatus::RuntimeError)?;
            if end > self.schema_properties.len() || schema.schema_id.is_empty() {
                return Err(NuxStatus::RuntimeError);
            }
            charge_result_utf8(
                &mut payload_bytes,
                &schema.schema_id,
                NUX_FLOW_MAX_ID_BYTE_LENGTH,
            )?;
            charge_result_utf8(
                &mut payload_bytes,
                &schema.name,
                NUX_FLOW_MAX_ID_BYTE_LENGTH,
            )?;
        }
        for property in &self.schema_properties {
            if !matches!(
                property.kind,
                NUX_FLOW_SCHEMA_PROPERTY_KIND_STRING
                    | NUX_FLOW_SCHEMA_PROPERTY_KIND_NUMBER
                    | NUX_FLOW_SCHEMA_PROPERTY_KIND_BOOL
                    | NUX_FLOW_SCHEMA_PROPERTY_KIND_TRIGGER
                    | NUX_FLOW_SCHEMA_PROPERTY_KIND_ENUM
                    | NUX_FLOW_SCHEMA_PROPERTY_KIND_COLOR
                    | NUX_FLOW_SCHEMA_PROPERTY_KIND_IMAGE
                    | NUX_FLOW_SCHEMA_PROPERTY_KIND_VIEW_MODEL
                    | NUX_FLOW_SCHEMA_PROPERTY_KIND_LIST
                    | NUX_FLOW_SCHEMA_PROPERTY_KIND_OBJECT
                    | NUX_FLOW_SCHEMA_PROPERTY_KIND_NULL
            ) || property.schema_id.is_empty()
                || property.property_id.is_empty()
            {
                return Err(NuxStatus::RuntimeError);
            }
            charge_result_utf8(
                &mut payload_bytes,
                &property.schema_id,
                NUX_FLOW_MAX_ID_BYTE_LENGTH,
            )?;
            charge_result_utf8(
                &mut payload_bytes,
                &property.property_id,
                NUX_FLOW_MAX_ID_BYTE_LENGTH,
            )?;
            charge_result_utf8(
                &mut payload_bytes,
                &property.name,
                NUX_FLOW_MAX_ID_BYTE_LENGTH,
            )?;
        }
        for template in &self.instance_templates {
            if template.schema_id.is_empty() {
                return Err(NuxStatus::RuntimeError);
            }
            charge_result_utf8(
                &mut payload_bytes,
                &template.schema_id,
                NUX_FLOW_MAX_ID_BYTE_LENGTH,
            )?;
            charge_result_utf8(
                &mut payload_bytes,
                &template.authored_name,
                NUX_FLOW_MAX_ID_BYTE_LENGTH,
            )?;
        }
        let mut instance_ids = HashSet::with_capacity(self.instances.len());
        for instance in &self.instances {
            if instance.instance_id == 0
                || instance.schema_id.is_empty()
                || !instance_ids.insert(instance.instance_id)
            {
                return Err(NuxStatus::RuntimeError);
            }
            if let Some(root) = instance.value_root_index
                && root as usize >= self.value_arena.nodes.len()
            {
                return Err(NuxStatus::RuntimeError);
            }
            charge_result_utf8(
                &mut payload_bytes,
                &instance.schema_id,
                NUX_FLOW_MAX_ID_BYTE_LENGTH,
            )?;
            charge_result_utf8(
                &mut payload_bytes,
                &instance.name,
                NUX_FLOW_MAX_ID_BYTE_LENGTH,
            )?;
        }
        let mut root_instance_ids = HashSet::with_capacity(self.value_roots.len());
        for root in &self.value_roots {
            if root.instance_id == 0
                || !root_instance_ids.insert(root.instance_id)
                || root.value_root_index as usize >= self.value_arena.nodes.len()
            {
                return Err(NuxStatus::RuntimeError);
            }
        }
        let mut created_local_ids = HashSet::with_capacity(self.created_instances.len());
        let mut created_instance_ids = HashSet::with_capacity(self.created_instances.len());
        for created in &self.created_instances {
            if created.instance_id == 0
                || !created_local_ids.insert(created.local_id)
                || !created_instance_ids.insert(created.instance_id)
            {
                return Err(NuxStatus::RuntimeError);
            }
        }
        for node in &self.value_arena.nodes {
            charge_result_utf8(
                &mut payload_bytes,
                &node.string_value,
                NUX_FLOW_MAX_STRING_BYTE_LENGTH,
            )?;
            charge_result_utf8(
                &mut payload_bytes,
                &node.schema_id,
                NUX_FLOW_MAX_ID_BYTE_LENGTH,
            )?;
        }
        for edge in &self.value_arena.edges {
            charge_result_utf8(&mut payload_bytes, &edge.key, NUX_FLOW_MAX_PATH_BYTE_LENGTH)?;
        }
        let mut prior_sequence = None;
        let mut prior_cycle_phase = None;
        for output in &self.outputs {
            if output.instance_id == Some(0) {
                return Err(NuxStatus::RuntimeError);
            }
            if !matches!(
                output.phase,
                NUX_FLOW_OUTPUT_PHASE_DELAYED_EVENT_CALLBACKS
                    | NUX_FLOW_OUTPUT_PHASE_REPORTED_EVENTS
                    | NUX_FLOW_OUTPUT_PHASE_RUNTIME_ADVANCE
                    | NUX_FLOW_OUTPUT_PHASE_VIEW_MODEL_CHANGES
                    | NUX_FLOW_OUTPUT_PHASE_HOST_WORK
                    | NUX_FLOW_OUTPUT_PHASE_RENDER
            ) || !matches!(
                output.kind,
                NUX_FLOW_OUTPUT_KIND_REPORTED_EVENT
                    | NUX_FLOW_OUTPUT_KIND_STATE_CHANGE
                    | NUX_FLOW_OUTPUT_KIND_VIEW_MODEL_CHANGE
                    | NUX_FLOW_OUTPUT_KIND_HOST_COMMAND
                    | NUX_FLOW_OUTPUT_KIND_RENDER_REQUEST
                    | NUX_FLOW_OUTPUT_KIND_QUERY_RESULT
                    | NUX_FLOW_OUTPUT_KIND_METADATA
                    | NUX_FLOW_OUTPUT_KIND_RUNTIME_ADVANCED
            ) {
                return Err(NuxStatus::RuntimeError);
            }
            if prior_sequence.is_some_and(|sequence| output.sequence <= sequence) {
                return Err(NuxStatus::RuntimeError);
            }
            if let Some((cycle, phase)) = prior_cycle_phase
                && (output.cycle < cycle || (output.cycle == cycle && output.phase < phase))
            {
                return Err(NuxStatus::RuntimeError);
            }
            if let Some(root) = output.payload_root_index {
                let node = self
                    .value_arena
                    .nodes
                    .get(root as usize)
                    .ok_or(NuxStatus::RuntimeError)?;
                if output.kind == NUX_FLOW_OUTPUT_KIND_REPORTED_EVENT
                    && node.edge_count as u64 > NUX_FLOW_MAX_EVENT_PROPERTY_COUNT
                {
                    return Err(NuxStatus::RuntimeError);
                }
            }
            if !output.delay_seconds.is_finite() || output.delay_seconds < 0.0 {
                return Err(NuxStatus::RuntimeError);
            }
            let property_start = output.first_event_property as usize;
            let property_count = output.event_property_count as usize;
            let property_end = property_start
                .checked_add(property_count)
                .ok_or(NuxStatus::RuntimeError)?;
            if property_count > NUX_FLOW_MAX_EVENT_PROPERTY_COUNT as usize
                || property_end > self.event_properties.len()
                || (output.kind != NUX_FLOW_OUTPUT_KIND_REPORTED_EVENT && property_count != 0)
            {
                return Err(NuxStatus::RuntimeError);
            }
            charge_result_utf8(
                &mut payload_bytes,
                &output.name,
                NUX_FLOW_MAX_ID_BYTE_LENGTH,
            )?;
            charge_result_utf8(
                &mut payload_bytes,
                &output.path,
                NUX_FLOW_MAX_PATH_BYTE_LENGTH,
            )?;
            charge_result_bytes(
                &mut payload_bytes,
                &output.payload,
                NUX_FLOW_MAX_OPERATION_PAYLOAD_BYTE_LENGTH,
            )?;
            prior_sequence = Some(output.sequence);
            prior_cycle_phase = Some((output.cycle, output.phase));
        }
        for property in &self.event_properties {
            if (property.value_root_index.is_some() == property.trigger_count.is_some())
                || property
                    .value_root_index
                    .is_some_and(|root| root as usize >= self.value_arena.nodes.len())
            {
                return Err(NuxStatus::RuntimeError);
            }
            charge_result_utf8(
                &mut payload_bytes,
                &property.name,
                NUX_FLOW_MAX_ID_BYTE_LENGTH,
            )?;
        }
        for diagnostic in &self.diagnostics {
            charge_result_utf8(
                &mut payload_bytes,
                &diagnostic.code,
                NUX_FLOW_MAX_ID_BYTE_LENGTH,
            )?;
            charge_result_utf8(
                &mut payload_bytes,
                &diagnostic.message,
                NUX_FLOW_MAX_STRING_BYTE_LENGTH,
            )?;
        }
        Ok(())
    }
}

fn validate_result_value_node(node: &OwnedValueNode) -> Result<(), NuxStatus> {
    let has_edges = node.edge_count != 0;
    let canonical_edge_start = has_edges || node.first_edge == 0;
    let number_is_zero = node.number_value.to_bits() == 0;
    let common_composite = number_is_zero
        && node.color_value == 0
        && !node.bool_value
        && node.identity_value == 0
        && node.string_value.is_empty()
        && canonical_edge_start;
    let valid = match node.kind {
        NUX_FLOW_VALUE_KIND_NULL => {
            common_composite
                && !has_edges
                && node.instance_id.is_none()
                && node.schema_id.is_empty()
        }
        NUX_FLOW_VALUE_KIND_STRING => {
            number_is_zero
                && node.color_value == 0
                && !node.bool_value
                && node.identity_value == 0
                && node.instance_id.is_none()
                && node.schema_id.is_empty()
                && !has_edges
                && canonical_edge_start
        }
        NUX_FLOW_VALUE_KIND_NUMBER => {
            node.number_value.is_finite()
                && node.number_value.abs() <= f64::from(f32::MAX)
                && node.color_value == 0
                && !node.bool_value
                && node.identity_value == 0
                && node.string_value.is_empty()
                && node.instance_id.is_none()
                && node.schema_id.is_empty()
                && !has_edges
                && canonical_edge_start
        }
        NUX_FLOW_VALUE_KIND_BOOL => {
            number_is_zero
                && node.color_value == 0
                && node.identity_value == 0
                && node.string_value.is_empty()
                && node.instance_id.is_none()
                && node.schema_id.is_empty()
                && !has_edges
                && canonical_edge_start
        }
        NUX_FLOW_VALUE_KIND_COLOR => {
            number_is_zero
                && !node.bool_value
                && node.identity_value == 0
                && node.string_value.is_empty()
                && node.instance_id.is_none()
                && node.schema_id.is_empty()
                && !has_edges
                && canonical_edge_start
        }
        NUX_FLOW_VALUE_KIND_ENUM | NUX_FLOW_VALUE_KIND_IMAGE => {
            number_is_zero
                && node.color_value == 0
                && !node.bool_value
                && node.string_value.is_empty()
                && node.instance_id.is_none()
                && node.schema_id.is_empty()
                && !has_edges
                && canonical_edge_start
        }
        NUX_FLOW_VALUE_KIND_OBJECT => common_composite && node.instance_id.is_none(),
        NUX_FLOW_VALUE_KIND_VIEW_MODEL => {
            common_composite && node.instance_id.is_none_or(|instance_id| instance_id != 0)
        }
        NUX_FLOW_VALUE_KIND_LIST => {
            common_composite && node.instance_id.is_none() && node.schema_id.is_empty()
        }
        _ => false,
    };
    if valid {
        Ok(())
    } else {
        Err(NuxStatus::RuntimeError)
    }
}

fn charge_result_utf8(total: &mut usize, bytes: &[u8], maximum: u64) -> Result<(), NuxStatus> {
    if std::str::from_utf8(bytes).is_err() {
        return Err(NuxStatus::RuntimeError);
    }
    charge_result_bytes(total, bytes, maximum)
}

fn charge_result_bytes(total: &mut usize, bytes: &[u8], maximum: u64) -> Result<(), NuxStatus> {
    if bytes.len() > maximum as usize {
        return Err(NuxStatus::RuntimeError);
    }
    *total = total
        .checked_add(bytes.len())
        .ok_or(NuxStatus::RuntimeError)?;
    if *total > NUX_FLOW_MAX_OPERATION_PAYLOAD_BYTE_LENGTH as usize {
        return Err(NuxStatus::RuntimeError);
    }
    Ok(())
}

fn borrowed_view(bytes: &[u8]) -> NuxByteView {
    if bytes.is_empty() {
        return NuxByteView::default();
    }
    NuxByteView {
        data: bytes.as_ptr(),
        len: u64::try_from(bytes.len()).unwrap_or(u64::MAX),
    }
}

fn replace_session_result(
    out_result: *mut *mut NuxFlowSessionResult,
    mut result: FlowSessionResultHandle,
) -> NuxStatus {
    if out_result.is_null() {
        return result.status;
    }
    if result.validate().is_err() {
        result = FlowSessionResultHandle::failure(
            NuxStatus::RuntimeError,
            "runtime produced an invalid or oversized ABI 1.2 result",
        );
    }
    let status = result.status;
    unsafe {
        *out_result = Box::into_raw(Box::new(result)).cast();
    }
    status
}

fn write_session_failure(
    out_result: *mut *mut NuxFlowSessionResult,
    status: NuxStatus,
    diagnostic: impl Into<Vec<u8>>,
) -> NuxStatus {
    replace_session_result(
        out_result,
        FlowSessionResultHandle::failure(status, diagnostic),
    )
}

fn reset_session_result(out_result: *mut *mut NuxFlowSessionResult) {
    if !out_result.is_null() {
        unsafe {
            *out_result = ptr::null_mut();
        }
    }
}

fn ffi_guard_with_session_result(
    out_result: *mut *mut NuxFlowSessionResult,
    on_panic: impl FnOnce(),
    body: impl FnOnce() -> NuxStatus,
) -> NuxStatus {
    match panic::catch_unwind(AssertUnwindSafe(body)) {
        Ok(status) => status,
        Err(_) => {
            let _ = panic::catch_unwind(AssertUnwindSafe(on_panic));
            reset_session_result(out_result);
            write_session_failure(out_result, NuxStatus::RuntimeError, PANIC_DIAGNOSTIC)
        }
    }
}

#[cfg(feature = "apple-product")]
mod configured_session_seam {
    use super::*;
    use nuxie::flow_session as core;

    pub(super) fn create(
        context: &FlowRuntimeContextHandle,
        descriptor: OwnedConfiguredSessionDescriptor,
    ) -> Result<(Box<FlowRenderSessionHandle>, FlowSessionResultHandle), RuntimeFailure> {
        let worker = Arc::clone(&context.worker);
        let creation = worker.call(None, move |state| create_on_worker(state, descriptor));
        let (session_id, result) = match creation {
            Ok(result) => result?,
            Err(WorkerCallError::Panicked) => {
                return Err(RuntimeFailure::runtime(PANIC_DIAGNOSTIC));
            }
            Err(WorkerCallError::Unavailable) => {
                return Err(RuntimeFailure::runtime("runtime worker is unavailable"));
            }
        };
        let handle = Box::new(FlowRenderSessionHandle {
            token: Arc::new(SessionToken {
                worker,
                id: session_id,
            }),
        });
        Ok((handle, result))
    }

    fn create_on_worker(
        state: &mut WorkerState,
        descriptor: OwnedConfiguredSessionDescriptor,
    ) -> Result<(SessionId, FlowSessionResultHandle), RuntimeFailure> {
        let config = core::FlowSessionConfig {
            artboard_name: descriptor.artboard_name,
            player_name: descriptor.player_name,
        };
        let (session, bootstrap) = core::FlowSession::create(Arc::clone(&state.file), config)
            .map_err(runtime_failure_from_core)?;
        let mut result = result_from_bootstrap(&bootstrap)?;
        result.is_settled = session.is_settled();
        result
            .validate()
            .map_err(|_| RuntimeFailure::runtime("bootstrap exceeds ABI 1.2 bounds"))?;
        let session_id = state.allocate_session_id()?;
        state.sessions.insert(
            session_id,
            SessionState {
                is_fatal: false,
                flow_session: session,
                render_cache: None,
                legacy_timestamp_seconds: 0.0,
                #[cfg(test)]
                render_attempts: 0,
                attachment: None,
            },
        );
        Ok((session_id, result))
    }

    pub(super) fn perform(
        session: *const NuxFlowRenderSession,
        operation: OwnedSessionOperation,
    ) -> (NuxStatus, FlowSessionResultHandle) {
        let handle = unsafe { &*session.cast::<FlowRenderSessionHandle>() };
        let session_id = handle.token.id;
        match handle.token.worker.call(Some(session_id), move |state| {
            perform_on_worker(state, session_id, operation)
        }) {
            Ok(Ok(result)) => (NuxStatus::Ok, result),
            Ok(Err(failure)) => (
                failure.status,
                FlowSessionResultHandle::failure(failure.status, failure.diagnostic),
            ),
            Err(WorkerCallError::Panicked) => (
                NuxStatus::RuntimeError,
                FlowSessionResultHandle::failure(NuxStatus::RuntimeError, PANIC_DIAGNOSTIC),
            ),
            Err(WorkerCallError::Unavailable) => (
                NuxStatus::RuntimeError,
                FlowSessionResultHandle::failure(
                    NuxStatus::RuntimeError,
                    "runtime worker is unavailable",
                ),
            ),
        }
    }

    fn perform_on_worker(
        state: &mut WorkerState,
        session_id: SessionId,
        operation: OwnedSessionOperation,
    ) -> Result<FlowSessionResultHandle, RuntimeFailure> {
        state.require_live_session(session_id)?;
        if let OwnedSessionOperation::Advance(advance) = operation {
            return perform_advance_on_worker(state, session_id, advance);
        }
        let session = state.session_mut(session_id)?;
        let session = &mut session.flow_session;
        let core_result = match operation {
            OwnedSessionOperation::Query(queries) => {
                let mut combined = FlowSessionResultHandle::empty_success();
                let mut deferred_player_inputs = None;
                for query in queries {
                    let query = match query.kind {
                        NUX_FLOW_QUERY_KIND_BOOTSTRAP => core::FlowQuery::Bootstrap,
                        NUX_FLOW_QUERY_KIND_VALUES => core::FlowQuery::Values,
                        NUX_FLOW_QUERY_KIND_CATALOG => core::FlowQuery::Catalog,
                        NUX_FLOW_QUERY_KIND_PLAYER_INPUTS => core::FlowQuery::PlayerInputs,
                        _ => {
                            return Err(RuntimeFailure::new(
                                NuxStatus::InvalidArgument,
                                "unknown query kind",
                            ));
                        }
                    };
                    let mut result = session
                        .perform(core::FlowOperation::Query(query))
                        .map_err(runtime_failure_from_core)?;
                    if result.player_inputs.is_some() {
                        deferred_player_inputs = result.player_inputs.take();
                    }
                    merge_core_result(&mut combined, result)?;
                }
                if let Some(inputs) = deferred_player_inputs {
                    replace_player_inputs(&mut combined, inputs)?;
                    combined.validate().map_err(|_| {
                        RuntimeFailure::runtime("query result exceeds ABI 1.2 bounds")
                    })?;
                }
                return Ok(combined);
            }
            OwnedSessionOperation::StateBatch(batch) => {
                core::FlowOperation::StateBatch(state_batch_to_core(batch)?)
            }
            OwnedSessionOperation::PointerBatch(events) => {
                core::FlowOperation::PointerBatch(core::FlowPointerBatch {
                    events: events
                        .into_iter()
                        .map(|event| {
                            Ok(core::FlowPointerEvent {
                                kind: pointer_kind_to_core(event.kind)?,
                                pointer_id: event.pointer_id,
                                x: event.x,
                                y: event.y,
                            })
                        })
                        .collect::<Result<Vec<_>, RuntimeFailure>>()?,
                })
            }
            OwnedSessionOperation::Advance(_) => {
                return Err(RuntimeFailure::runtime("advance dispatch is inconsistent"));
            }
        };
        let result = session
            .perform(core_result)
            .map_err(runtime_failure_from_core)?;
        result_from_core(result)
    }

    fn perform_advance_on_worker(
        state: &mut WorkerState,
        session_id: SessionId,
        mut advance: OwnedAdvanceOperation,
    ) -> Result<FlowSessionResultHandle, RuntimeFailure> {
        let completion = PendingFrameCompletion {
            callback: advance.completion_callback.take(),
            context_identity: advance.completion_context_identity,
        };
        let session = state.session_mut(session_id)?;
        let flow_session = &mut session.flow_session;
        let core_result = flow_session
            .perform(core::FlowOperation::Advance(core::FlowAdvance {
                timestamp_seconds: advance.timestamp_seconds,
                delta_seconds: advance.delta_seconds,
                render: advance.render,
            }))
            .map_err(runtime_failure_from_core)?;
        session.legacy_timestamp_seconds = advance.timestamp_seconds;
        let mut result = result_from_core(core_result)?;
        if !advance.render {
            return Ok(result);
        }
        let attachment = session
            .attachment
            .as_mut()
            .ok_or_else(|| RuntimeFailure::surface("surface is not attached"))?;
        if let Some(disposition) = attachment
            .surface
            .preflight_present(&attachment.factory, advance.drawable_identity != 0)
            .map_err(|error| RuntimeFailure::surface(format!("{error:#}")))?
        {
            result.surface_disposition = surface_disposition(disposition);
            return Ok(result);
        }
        let (viewport_width, viewport_height) = attachment.surface.dimensions();
        let bounds = flow_session.artboard_bounds();
        let presentation_transform = centered_contain_transform(
            bounds.x,
            bounds.y,
            bounds.width,
            bounds.height,
            viewport_width,
            viewport_height,
        )?;
        let mut frame = attachment.factory.begin_frame(0x0000_0000);
        frame.transform(presentation_transform);
        #[cfg(test)]
        {
            session.render_attempts = session.render_attempts.saturating_add(1);
        }
        let render_cache = session
            .render_cache
            .get_or_insert_with(|| flow_session.new_render_cache());
        flow_session
            .draw(&mut attachment.factory, &mut frame, render_cache)
            .map_err(runtime_failure_from_core)?;
        let drawable = ptr::with_exposed_provenance_mut::<c_void>(advance.drawable_identity);
        let completion = completion.into_renderer_completion();
        let (disposition, _metrics) = unsafe {
            attachment
                .surface
                .present(&mut attachment.factory, frame, drawable, completion)
        }
        .map_err(|error| RuntimeFailure::surface(format!("{error:#}")))?;
        result.surface_disposition = surface_disposition(disposition);
        Ok(result)
    }

    fn runtime_failure_from_core(error: core::FlowSessionError) -> RuntimeFailure {
        let status = match error.kind() {
            core::FlowSessionErrorKind::NotFound => NuxStatus::NotFound,
            core::FlowSessionErrorKind::InvalidArgument
            | core::FlowSessionErrorKind::LimitExceeded
            | core::FlowSessionErrorKind::Conflict => NuxStatus::InvalidArgument,
            core::FlowSessionErrorKind::Runtime => NuxStatus::RuntimeError,
        };
        RuntimeFailure::new(status, error.message())
    }

    fn state_batch_to_core(batch: OwnedStateBatch) -> Result<core::FlowStateBatch, RuntimeFailure> {
        let OwnedStateBatch {
            host_mutation_id,
            value_arena,
            new_instances,
            mutations,
        } = batch;
        let new_instances = new_instances
            .into_iter()
            .map(|instance| {
                let schema_name = String::from_utf8(instance.schema_name).map_err(|_| {
                    RuntimeFailure::new(
                        NuxStatus::InvalidArgument,
                        "new-instance schema name is not UTF-8",
                    )
                })?;
                let authored_instance_name = instance
                    .authored_instance_name
                    .map(String::from_utf8)
                    .transpose()
                    .map_err(|_| {
                        RuntimeFailure::new(
                            NuxStatus::InvalidArgument,
                            "new-instance authored name is not UTF-8",
                        )
                    })?;
                Ok(core::FlowNewInstance {
                    local_id: instance.local_id,
                    schema_name,
                    authored_instance_name,
                })
            })
            .collect::<Result<Vec<_>, RuntimeFailure>>()?;
        let mutations = mutations
            .into_iter()
            .map(|mutation| mutation_to_core(mutation, &value_arena))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(core::FlowStateBatch {
            host_mutation_id,
            mutations,
            new_instances,
        })
    }

    fn pointer_kind_to_core(
        kind: NuxFlowPointerEventKind,
    ) -> Result<core::FlowPointerKind, RuntimeFailure> {
        match kind {
            NUX_FLOW_POINTER_EVENT_KIND_DOWN => Ok(core::FlowPointerKind::Down),
            NUX_FLOW_POINTER_EVENT_KIND_MOVE => Ok(core::FlowPointerKind::Move),
            NUX_FLOW_POINTER_EVENT_KIND_UP => Ok(core::FlowPointerKind::Up),
            NUX_FLOW_POINTER_EVENT_KIND_CANCEL => Ok(core::FlowPointerKind::Cancel),
            NUX_FLOW_POINTER_EVENT_KIND_EXIT => Ok(core::FlowPointerKind::Exit),
            _ => Err(RuntimeFailure::new(
                NuxStatus::InvalidArgument,
                "unknown pointer event kind",
            )),
        }
    }

    fn mutation_to_core(
        mutation: OwnedStateMutation,
        arena: &OwnedValueArena,
    ) -> Result<core::FlowStateMutation, RuntimeFailure> {
        let OwnedStateMutation {
            kind,
            instance,
            item,
            path,
            input_name,
            value_root_index,
            index,
            other_index,
        } = mutation;
        if matches!(
            kind,
            NUX_FLOW_STATE_MUTATION_KIND_SET_INPUT_BOOL
                | NUX_FLOW_STATE_MUTATION_KIND_SET_INPUT_NUMBER
                | NUX_FLOW_STATE_MUTATION_KIND_FIRE_INPUT_TRIGGER
        ) {
            let name = String::from_utf8(input_name.ok_or_else(|| {
                RuntimeFailure::new(
                    NuxStatus::InvalidArgument,
                    "player-input mutation has no input name",
                )
            })?)
            .map_err(|_| {
                RuntimeFailure::new(NuxStatus::InvalidArgument, "player-input name is not UTF-8")
            })?;
            return match kind {
                NUX_FLOW_STATE_MUTATION_KIND_SET_INPUT_BOOL => {
                    let core::FlowScalarValue::Bool(value) =
                        scalar_value_at(arena, value_root_index)?
                    else {
                        return Err(RuntimeFailure::new(
                            NuxStatus::InvalidArgument,
                            "bool input mutation requires a bool value",
                        ));
                    };
                    Ok(core::FlowStateMutation::SetInputBool { name, value })
                }
                NUX_FLOW_STATE_MUTATION_KIND_SET_INPUT_NUMBER => {
                    let core::FlowScalarValue::Number(value) =
                        scalar_value_at(arena, value_root_index)?
                    else {
                        return Err(RuntimeFailure::new(
                            NuxStatus::InvalidArgument,
                            "number input mutation requires a number value",
                        ));
                    };
                    Ok(core::FlowStateMutation::SetInputNumber { name, value })
                }
                NUX_FLOW_STATE_MUTATION_KIND_FIRE_INPUT_TRIGGER => {
                    Ok(core::FlowStateMutation::FireInputTrigger { name })
                }
                _ => Err(RuntimeFailure::new(
                    NuxStatus::InvalidArgument,
                    "unknown player-input mutation kind",
                )),
            };
        }
        let instance = instance_reference_to_core(instance.ok_or_else(|| {
            RuntimeFailure::new(
                NuxStatus::InvalidArgument,
                "view-model mutation has no instance",
            )
        })?)?;
        let path = String::from_utf8(path.ok_or_else(|| {
            RuntimeFailure::new(
                NuxStatus::InvalidArgument,
                "view-model mutation has no path",
            )
        })?)
        .map_err(|_| {
            RuntimeFailure::new(NuxStatus::InvalidArgument, "mutation path is not UTF-8")
        })?;
        match kind {
            NUX_FLOW_STATE_MUTATION_KIND_SET => Ok(core::FlowStateMutation::SetValue {
                instance,
                path,
                value: scalar_value_at(arena, value_root_index)?,
            }),
            NUX_FLOW_STATE_MUTATION_KIND_TRIGGER => {
                Ok(core::FlowStateMutation::FireTrigger { instance, path })
            }
            NUX_FLOW_STATE_MUTATION_KIND_LIST_INSERT => Ok(core::FlowStateMutation::ListInsert {
                instance,
                path,
                index: index as usize,
                item: instance_reference_to_core(item.ok_or_else(|| {
                    RuntimeFailure::new(NuxStatus::InvalidArgument, "list insert item is missing")
                })?)?,
            }),
            NUX_FLOW_STATE_MUTATION_KIND_LIST_REMOVE => Ok(core::FlowStateMutation::ListRemove {
                instance,
                path,
                index: index as usize,
            }),
            NUX_FLOW_STATE_MUTATION_KIND_LIST_SWAP => Ok(core::FlowStateMutation::ListSwap {
                instance,
                path,
                first: index as usize,
                second: other_index as usize,
            }),
            NUX_FLOW_STATE_MUTATION_KIND_LIST_MOVE => Ok(core::FlowStateMutation::ListMove {
                instance,
                path,
                from: index as usize,
                to: other_index as usize,
            }),
            NUX_FLOW_STATE_MUTATION_KIND_LIST_SET => Ok(core::FlowStateMutation::ListSet {
                instance,
                path,
                index: index as usize,
                item: instance_reference_to_core(item.ok_or_else(|| {
                    RuntimeFailure::new(NuxStatus::InvalidArgument, "list set item is missing")
                })?)?,
            }),
            NUX_FLOW_STATE_MUTATION_KIND_LIST_CLEAR => {
                Ok(core::FlowStateMutation::ListClear { instance, path })
            }
            _ => Err(RuntimeFailure::new(
                NuxStatus::InvalidArgument,
                "unknown mutation kind",
            )),
        }
    }

    fn instance_reference_to_core(
        reference: OwnedInstanceReference,
    ) -> Result<core::FlowInstanceRef, RuntimeFailure> {
        match reference {
            OwnedInstanceReference::Existing(id) => core::FlowInstanceId::new(id)
                .map(core::FlowInstanceRef::Existing)
                .ok_or_else(|| {
                    RuntimeFailure::new(NuxStatus::InvalidArgument, "instance ID zero is reserved")
                }),
            OwnedInstanceReference::New(local_id) => Ok(core::FlowInstanceRef::New(local_id)),
        }
    }

    fn scalar_value_at(
        arena: &OwnedValueArena,
        root: Option<u32>,
    ) -> Result<core::FlowScalarValue, RuntimeFailure> {
        let root = root.ok_or_else(|| {
            RuntimeFailure::new(NuxStatus::InvalidArgument, "set mutation has no value")
        })?;
        let node = arena.nodes.get(root as usize).ok_or_else(|| {
            RuntimeFailure::new(NuxStatus::InvalidArgument, "set value root is out of range")
        })?;
        match node.kind {
            NUX_FLOW_VALUE_KIND_NULL => Ok(core::FlowScalarValue::Null),
            NUX_FLOW_VALUE_KIND_STRING => String::from_utf8(node.string_value.clone())
                .map(core::FlowScalarValue::String)
                .map_err(|_| {
                    RuntimeFailure::new(NuxStatus::InvalidArgument, "string value is not UTF-8")
                }),
            NUX_FLOW_VALUE_KIND_NUMBER => {
                Ok(core::FlowScalarValue::Number(node.number_value as f32))
            }
            NUX_FLOW_VALUE_KIND_BOOL => Ok(core::FlowScalarValue::Bool(node.bool_value)),
            NUX_FLOW_VALUE_KIND_ENUM => Ok(core::FlowScalarValue::Enum(node.identity_value)),
            NUX_FLOW_VALUE_KIND_COLOR => Ok(core::FlowScalarValue::Color(node.color_value)),
            NUX_FLOW_VALUE_KIND_IMAGE => Ok(core::FlowScalarValue::Image(node.identity_value)),
            _ => Err(RuntimeFailure::new(
                NuxStatus::InvalidArgument,
                "set values must be scalar",
            )),
        }
    }

    fn result_from_bootstrap(
        bootstrap: &core::FlowBootstrap,
    ) -> Result<FlowSessionResultHandle, RuntimeFailure> {
        let max_x = bootstrap.bounds.x + bootstrap.bounds.width;
        let max_y = bootstrap.bounds.y + bootstrap.bounds.height;
        if !max_x.is_finite() || !max_y.is_finite() {
            return Err(RuntimeFailure::runtime("artboard bounds overflowed"));
        }
        let mut result = FlowSessionResultHandle::empty_success();
        result.player_metadata = Some(OwnedPlayerMetadata {
            kind: player_kind_from_core(bootstrap.player.kind),
            selection: player_selection_from_core(bootstrap.player.selection),
            player_index: bootstrap
                .player
                .index
                .map(u32::try_from)
                .transpose()
                .map_err(|_| RuntimeFailure::runtime("player index overflowed"))?,
            artboard_name: bootstrap
                .artboard_name
                .as_deref()
                .unwrap_or_default()
                .as_bytes()
                .to_vec(),
            player_name: bootstrap
                .player
                .name
                .as_deref()
                .unwrap_or_default()
                .as_bytes()
                .to_vec(),
            min_x: bootstrap.bounds.x,
            min_y: bootstrap.bounds.y,
            max_x,
            max_y,
        });
        append_value_arena(&mut result, &bootstrap.values)?;
        replace_catalog(&mut result, &bootstrap.catalog)?;
        synchronize_instance_roots(&mut result);
        Ok(result)
    }

    fn player_kind_from_core(kind: core::FlowPlayerKind) -> NuxFlowPlayerKind {
        match kind {
            core::FlowPlayerKind::StateMachine => NUX_FLOW_PLAYER_KIND_STATE_MACHINE,
            core::FlowPlayerKind::LinearAnimation => NUX_FLOW_PLAYER_KIND_LINEAR_ANIMATION,
            core::FlowPlayerKind::Static => NUX_FLOW_PLAYER_KIND_STATIC,
        }
    }

    fn player_selection_from_core(selection: core::FlowPlayerSelection) -> NuxFlowPlayerSelection {
        match selection {
            core::FlowPlayerSelection::ExplicitStateMachine => {
                NUX_FLOW_PLAYER_SELECTION_EXPLICIT_STATE_MACHINE
            }
            core::FlowPlayerSelection::AuthoredDefaultStateMachine => {
                NUX_FLOW_PLAYER_SELECTION_AUTHORED_DEFAULT_STATE_MACHINE
            }
            core::FlowPlayerSelection::FirstStateMachine => {
                NUX_FLOW_PLAYER_SELECTION_FIRST_STATE_MACHINE
            }
            core::FlowPlayerSelection::FirstAnimation => NUX_FLOW_PLAYER_SELECTION_FIRST_ANIMATION,
            core::FlowPlayerSelection::Static => NUX_FLOW_PLAYER_SELECTION_STATIC,
        }
    }

    fn replace_catalog(
        result: &mut FlowSessionResultHandle,
        catalog: &core::FlowCatalog,
    ) -> Result<(), RuntimeFailure> {
        result.schemas.clear();
        result.schema_properties.clear();
        result.instance_templates.clear();
        result.instances.clear();
        for schema in &catalog.schemas {
            let first_property = u32::try_from(result.schema_properties.len())
                .map_err(|_| RuntimeFailure::runtime("schema property index overflowed"))?;
            for property in &schema.properties {
                result.schema_properties.push(OwnedSchemaProperty {
                    kind: schema_property_kind_from_core(property.value_type),
                    schema_id: schema.name.as_bytes().to_vec(),
                    property_id: property.name.as_bytes().to_vec(),
                    name: property.name.as_bytes().to_vec(),
                });
            }
            let property_count = u32::try_from(schema.properties.len())
                .map_err(|_| RuntimeFailure::runtime("schema property count overflowed"))?;
            result.schemas.push(OwnedSchema {
                first_property,
                property_count,
                schema_id: schema.name.as_bytes().to_vec(),
                name: schema.name.as_bytes().to_vec(),
            });
        }
        for template in &catalog.templates {
            result.instance_templates.push(OwnedInstanceTemplate {
                authored_index: u32::try_from(template.authored_index)
                    .map_err(|_| RuntimeFailure::runtime("authored instance index overflowed"))?,
                schema_id: template.schema_name.as_bytes().to_vec(),
                authored_name: template
                    .authored_name
                    .as_deref()
                    .unwrap_or_default()
                    .as_bytes()
                    .to_vec(),
            });
        }
        for instance in &catalog.instances {
            result.instances.push(OwnedInstance {
                value_root_index: None,
                instance_id: instance.id.get(),
                is_root: instance.is_root,
                schema_id: instance.schema_name.as_bytes().to_vec(),
                name: instance
                    .authored_name
                    .as_deref()
                    .unwrap_or_default()
                    .as_bytes()
                    .to_vec(),
            });
        }
        Ok(())
    }

    fn schema_property_kind_from_core(kind: core::FlowValueType) -> NuxFlowSchemaPropertyKind {
        match kind {
            core::FlowValueType::Null => NUX_FLOW_SCHEMA_PROPERTY_KIND_NULL,
            core::FlowValueType::String => NUX_FLOW_SCHEMA_PROPERTY_KIND_STRING,
            core::FlowValueType::Number => NUX_FLOW_SCHEMA_PROPERTY_KIND_NUMBER,
            core::FlowValueType::Bool => NUX_FLOW_SCHEMA_PROPERTY_KIND_BOOL,
            core::FlowValueType::Enum => NUX_FLOW_SCHEMA_PROPERTY_KIND_ENUM,
            core::FlowValueType::Color => NUX_FLOW_SCHEMA_PROPERTY_KIND_COLOR,
            core::FlowValueType::Image => NUX_FLOW_SCHEMA_PROPERTY_KIND_IMAGE,
            core::FlowValueType::Object => NUX_FLOW_SCHEMA_PROPERTY_KIND_OBJECT,
            core::FlowValueType::ViewModel => NUX_FLOW_SCHEMA_PROPERTY_KIND_VIEW_MODEL,
            core::FlowValueType::List => NUX_FLOW_SCHEMA_PROPERTY_KIND_LIST,
            core::FlowValueType::Trigger => NUX_FLOW_SCHEMA_PROPERTY_KIND_TRIGGER,
        }
    }

    fn append_value_arena(
        result: &mut FlowSessionResultHandle,
        arena: &core::FlowValueArena,
    ) -> Result<(), RuntimeFailure> {
        let mut indexes = HashMap::with_capacity(arena.nodes.len());
        for (index, node) in arena.nodes.iter().enumerate() {
            let index = result
                .value_arena
                .nodes
                .len()
                .checked_add(index)
                .and_then(|index| u32::try_from(index).ok())
                .ok_or_else(|| RuntimeFailure::runtime("value node index overflowed"))?;
            if indexes.insert(node.id.get(), index).is_some() {
                return Err(RuntimeFailure::runtime(
                    "value arena contains duplicate node IDs",
                ));
            }
        }
        for node in &arena.nodes {
            let first_edge = u32::try_from(result.value_arena.edges.len())
                .map_err(|_| RuntimeFailure::runtime("value edge index overflowed"))?;
            let (kind, number_value, color_value, bool_value, identity_value, string_value) =
                match &node.value {
                    core::FlowValue::Null => {
                        (NUX_FLOW_VALUE_KIND_NULL, 0.0, 0, false, 0, Vec::new())
                    }
                    core::FlowValue::String(value) => (
                        NUX_FLOW_VALUE_KIND_STRING,
                        0.0,
                        0,
                        false,
                        0,
                        value.as_bytes().to_vec(),
                    ),
                    core::FlowValue::Number(value) => (
                        NUX_FLOW_VALUE_KIND_NUMBER,
                        f64::from(*value),
                        0,
                        false,
                        0,
                        Vec::new(),
                    ),
                    core::FlowValue::Bool(value) => {
                        (NUX_FLOW_VALUE_KIND_BOOL, 0.0, 0, *value, 0, Vec::new())
                    }
                    core::FlowValue::Enum(value) => {
                        (NUX_FLOW_VALUE_KIND_ENUM, 0.0, 0, false, *value, Vec::new())
                    }
                    core::FlowValue::Color(value) => {
                        (NUX_FLOW_VALUE_KIND_COLOR, 0.0, *value, false, 0, Vec::new())
                    }
                    core::FlowValue::Image(value) => {
                        (NUX_FLOW_VALUE_KIND_IMAGE, 0.0, 0, false, *value, Vec::new())
                    }
                    core::FlowValue::Object(children) => {
                        append_named_edges(&mut result.value_arena.edges, children, &indexes)?;
                        (NUX_FLOW_VALUE_KIND_OBJECT, 0.0, 0, false, 0, Vec::new())
                    }
                    core::FlowValue::ViewModel(children) => {
                        append_named_edges(&mut result.value_arena.edges, children, &indexes)?;
                        (NUX_FLOW_VALUE_KIND_VIEW_MODEL, 0.0, 0, false, 0, Vec::new())
                    }
                    core::FlowValue::List(children) => {
                        for child in children {
                            let node_index = *indexes.get(&child.get()).ok_or_else(|| {
                                RuntimeFailure::runtime("value edge references a missing node")
                            })?;
                            result.value_arena.edges.push(OwnedValueEdge {
                                node_index,
                                key: Vec::new(),
                            });
                        }
                        (NUX_FLOW_VALUE_KIND_LIST, 0.0, 0, false, 0, Vec::new())
                    }
                };
            let edge_count = u32::try_from(result.value_arena.edges.len())
                .ok()
                .and_then(|end| end.checked_sub(first_edge))
                .ok_or_else(|| RuntimeFailure::runtime("value edge count overflowed"))?;
            let first_edge = if edge_count == 0 { 0 } else { first_edge };
            result.value_arena.nodes.push(OwnedValueNode {
                kind,
                number_value,
                color_value,
                bool_value,
                first_edge,
                edge_count,
                instance_id: None,
                identity_value,
                string_value,
                schema_id: Vec::new(),
            });
        }
        for (instance, root) in &arena.roots {
            let value_root_index = *indexes
                .get(&root.get())
                .ok_or_else(|| RuntimeFailure::runtime("value root references a missing node"))?;
            if let Some(node) = result.value_arena.nodes.get_mut(value_root_index as usize)
                && node.kind == NUX_FLOW_VALUE_KIND_VIEW_MODEL
            {
                node.instance_id = Some(instance.get());
            }
            result.value_roots.push(OwnedValueRoot {
                value_root_index,
                instance_id: instance.get(),
            });
        }
        Ok(())
    }

    fn append_named_edges(
        output: &mut Vec<OwnedValueEdge>,
        children: &[(String, core::FlowValueId)],
        indexes: &HashMap<u32, u32>,
    ) -> Result<(), RuntimeFailure> {
        for (key, child) in children {
            let node_index = *indexes
                .get(&child.get())
                .ok_or_else(|| RuntimeFailure::runtime("value edge references a missing node"))?;
            output.push(OwnedValueEdge {
                node_index,
                key: key.as_bytes().to_vec(),
            });
        }
        Ok(())
    }

    fn synchronize_instance_roots(result: &mut FlowSessionResultHandle) {
        for instance in &mut result.instances {
            instance.value_root_index = result
                .value_roots
                .iter()
                .find(|root| root.instance_id == instance.instance_id)
                .map(|root| root.value_root_index);
        }
        for root in &result.value_roots {
            let schema_id = result
                .instances
                .iter()
                .find(|instance| instance.instance_id == root.instance_id)
                .map(|instance| instance.schema_id.clone())
                .unwrap_or_default();
            if let Some(node) = result
                .value_arena
                .nodes
                .get_mut(root.value_root_index as usize)
                && node.kind == NUX_FLOW_VALUE_KIND_VIEW_MODEL
            {
                node.instance_id = Some(root.instance_id);
                node.schema_id = schema_id;
            }
        }
    }

    fn result_from_core(
        result: core::FlowResult,
    ) -> Result<FlowSessionResultHandle, RuntimeFailure> {
        let mut translated = FlowSessionResultHandle::empty_success();
        merge_core_result(&mut translated, result)?;
        Ok(translated)
    }

    fn merge_core_result(
        translated: &mut FlowSessionResultHandle,
        result: core::FlowResult,
    ) -> Result<(), RuntimeFailure> {
        translated.is_dirty = result.dirty;
        translated.is_settled = result.settled;
        translated.wake_after = result.wake_after_seconds.map(f64::from);
        if let Some(snapshot) = result.snapshot.as_ref() {
            let bootstrap = result_from_bootstrap(snapshot)?;
            translated.player_metadata = bootstrap.player_metadata;
            translated.schemas = bootstrap.schemas;
            translated.schema_properties = bootstrap.schema_properties;
            translated.instance_templates = bootstrap.instance_templates;
            translated.instances = bootstrap.instances;
            translated.value_arena = bootstrap.value_arena;
            translated.value_roots = bootstrap.value_roots;
        }
        if let Some(values) = result.values.as_ref() {
            translated.value_arena = OwnedValueArena::default();
            translated.value_roots.clear();
            append_value_arena(translated, values)?;
        }
        if let Some(catalog) = result.catalog.as_ref() {
            replace_catalog(translated, catalog)?;
        }
        if let Some(inputs) = result.player_inputs {
            replace_player_inputs(translated, inputs)?;
        }
        synchronize_instance_roots(translated);
        translated
            .created_instances
            .extend(
                result
                    .created_instances
                    .into_iter()
                    .map(|created| OwnedCreatedInstance {
                        local_id: created.local_id,
                        instance_id: created.id.get(),
                    }),
            );
        append_outputs(translated, result.outputs)?;
        translated
            .validate()
            .map_err(|_| RuntimeFailure::runtime("flow result exceeds ABI 1.2 bounds"))?;
        Ok(())
    }

    fn replace_player_inputs(
        result: &mut FlowSessionResultHandle,
        inputs: Vec<core::FlowInputSnapshot>,
    ) -> Result<(), RuntimeFailure> {
        result.player_inputs.clear();
        for input in inputs {
            let kind = match input.kind {
                nuxie::StateMachineInputKind::Bool => NUX_FLOW_PLAYER_INPUT_KIND_BOOL,
                nuxie::StateMachineInputKind::Number => NUX_FLOW_PLAYER_INPUT_KIND_NUMBER,
                nuxie::StateMachineInputKind::Trigger => NUX_FLOW_PLAYER_INPUT_KIND_TRIGGER,
            };
            let value_root_index = push_scalar(result, input.value)?;
            result.player_inputs.push(OwnedPlayerInput {
                kind,
                value_root_index,
                name: input.name.unwrap_or_default().into_bytes(),
            });
        }
        Ok(())
    }

    fn append_outputs(
        result: &mut FlowSessionResultHandle,
        outputs: Vec<core::FlowOutput>,
    ) -> Result<(), RuntimeFailure> {
        for output in outputs {
            let phase = match output.phase {
                core::FlowOutputPhase::DelayedEventCallbacks => {
                    NUX_FLOW_OUTPUT_PHASE_DELAYED_EVENT_CALLBACKS
                }
                core::FlowOutputPhase::ReportedEvents => NUX_FLOW_OUTPUT_PHASE_REPORTED_EVENTS,
                core::FlowOutputPhase::RuntimeAdvance => NUX_FLOW_OUTPUT_PHASE_RUNTIME_ADVANCE,
                core::FlowOutputPhase::ViewModelChanges => NUX_FLOW_OUTPUT_PHASE_VIEW_MODEL_CHANGES,
                core::FlowOutputPhase::HostWork => NUX_FLOW_OUTPUT_PHASE_HOST_WORK,
                core::FlowOutputPhase::Render => NUX_FLOW_OUTPUT_PHASE_RENDER,
            };
            let mut translated = OwnedOutput {
                phase,
                kind: NUX_FLOW_OUTPUT_KIND_QUERY_RESULT,
                payload_root_index: None,
                sequence: output.sequence,
                cycle: output.cycle,
                origin_mutation_id: None,
                instance_id: None,
                event_type: 0,
                first_event_property: 0,
                event_property_count: 0,
                delay_seconds: 0.0,
                name: Vec::new(),
                path: Vec::new(),
                payload: Vec::new(),
            };
            match output.payload {
                core::FlowOutputPayload::ReportedEvent {
                    name,
                    event_type,
                    delay_seconds,
                    properties,
                } => {
                    translated.kind = NUX_FLOW_OUTPUT_KIND_REPORTED_EVENT;
                    populate_event_output(
                        result,
                        &mut translated,
                        name,
                        event_type,
                        delay_seconds,
                        properties,
                    )?;
                }
                core::FlowOutputPayload::StateChanged {
                    instance_id,
                    path,
                    value,
                    origin_mutation_id,
                } => {
                    translated.kind = if instance_id.is_some() {
                        NUX_FLOW_OUTPUT_KIND_VIEW_MODEL_CHANGE
                    } else {
                        NUX_FLOW_OUTPUT_KIND_STATE_CHANGE
                    };
                    translated.instance_id = instance_id.map(core::FlowInstanceId::get);
                    translated.path = path.into_bytes();
                    translated.origin_mutation_id = origin_mutation_id;
                    if let Some(value) = value {
                        translated.payload_root_index = Some(push_scalar(result, value)?);
                    }
                }
                core::FlowOutputPayload::HostCommand { name, payload } => {
                    translated.kind = NUX_FLOW_OUTPUT_KIND_HOST_COMMAND;
                    translated.name = name.into_bytes();
                    translated.payload = payload;
                }
                core::FlowOutputPayload::RenderRequested { .. } => {
                    translated.kind = NUX_FLOW_OUTPUT_KIND_RENDER_REQUEST;
                }
                core::FlowOutputPayload::Metadata(metadata) => {
                    translated.kind = NUX_FLOW_OUTPUT_KIND_METADATA;
                    translated.event_type = player_kind_from_core(metadata.kind);
                    translated.name = metadata.name.unwrap_or_default().into_bytes();
                }
                core::FlowOutputPayload::RuntimeAdvanced { delta_seconds } => {
                    translated.kind = NUX_FLOW_OUTPUT_KIND_RUNTIME_ADVANCED;
                    translated.delay_seconds = delta_seconds;
                }
            }
            result.outputs.push(translated);
        }
        Ok(())
    }

    fn populate_event_output(
        result: &mut FlowSessionResultHandle,
        output: &mut OwnedOutput,
        name: Option<String>,
        event_type: u32,
        delay_seconds: f32,
        properties: Vec<core::FlowEventProperty>,
    ) -> Result<(), RuntimeFailure> {
        output.name = name.unwrap_or_default().into_bytes();
        output.event_type = event_type;
        output.delay_seconds = delay_seconds;
        output.first_event_property = u32::try_from(result.event_properties.len())
            .map_err(|_| RuntimeFailure::runtime("event property index overflowed"))?;
        output.event_property_count = u32::try_from(properties.len())
            .map_err(|_| RuntimeFailure::runtime("event property count overflowed"))?;
        for property in properties {
            let (value_root_index, trigger_count) = match property.value {
                core::FlowScalarValue::Trigger(count) => (None, Some(count)),
                value => (Some(push_scalar(result, value)?), None),
            };
            result.event_properties.push(OwnedEventProperty {
                value_root_index,
                trigger_count,
                name: property.name.unwrap_or_default().into_bytes(),
            });
        }
        Ok(())
    }

    fn push_scalar(
        result: &mut FlowSessionResultHandle,
        value: core::FlowScalarValue,
    ) -> Result<u32, RuntimeFailure> {
        let index = u32::try_from(result.value_arena.nodes.len())
            .map_err(|_| RuntimeFailure::runtime("value node index overflowed"))?;
        let (kind, number_value, color_value, bool_value, identity_value, string_value) =
            match value {
                core::FlowScalarValue::Null => {
                    (NUX_FLOW_VALUE_KIND_NULL, 0.0, 0, false, 0, Vec::new())
                }
                core::FlowScalarValue::String(value) => (
                    NUX_FLOW_VALUE_KIND_STRING,
                    0.0,
                    0,
                    false,
                    0,
                    value.into_bytes(),
                ),
                core::FlowScalarValue::Number(value) => (
                    NUX_FLOW_VALUE_KIND_NUMBER,
                    f64::from(value),
                    0,
                    false,
                    0,
                    Vec::new(),
                ),
                core::FlowScalarValue::Bool(value) => {
                    (NUX_FLOW_VALUE_KIND_BOOL, 0.0, 0, value, 0, Vec::new())
                }
                core::FlowScalarValue::Enum(value) => {
                    (NUX_FLOW_VALUE_KIND_ENUM, 0.0, 0, false, value, Vec::new())
                }
                core::FlowScalarValue::Color(value) => {
                    (NUX_FLOW_VALUE_KIND_COLOR, 0.0, value, false, 0, Vec::new())
                }
                core::FlowScalarValue::Image(value) => {
                    (NUX_FLOW_VALUE_KIND_IMAGE, 0.0, 0, false, value, Vec::new())
                }
                core::FlowScalarValue::Trigger(_) => {
                    return Err(RuntimeFailure::runtime(
                        "trigger counts are valid only in event properties",
                    ));
                }
            };
        result.value_arena.nodes.push(OwnedValueNode {
            kind,
            number_value,
            color_value,
            bool_value,
            first_edge: 0,
            edge_count: 0,
            instance_id: None,
            identity_value,
            string_value,
            schema_id: Vec::new(),
        });
        Ok(index)
    }
}

#[cfg(feature = "apple-product")]
#[unsafe(no_mangle)]
/// Creates one independent screen session using the ABI 1.2 player-selection
/// and bootstrap-result contract. Creation never performs an observable
/// advance. The returned result owns player metadata, bounds, catalog, and
/// bootstrap value views until explicitly freed.
///
/// # Safety
///
/// `context` must be live. Non-null pointers must be properly aligned and valid
/// for this synchronous call. Output pointers must address writable storage.
pub unsafe extern "C" fn nux_flow_render_session_create_configured(
    context: *const NuxFlowRuntimeContext,
    descriptor: *const NuxFlowConfiguredSessionDescriptor,
    out_session: *mut *mut NuxFlowRenderSession,
    out_result: *mut *mut NuxFlowSessionResult,
) -> NuxStatus {
    ffi_guard_with_session_result(
        out_result,
        || {},
        || {
            reset_out_handle(out_session);
            reset_session_result(out_result);
            if context.is_null()
                || descriptor.is_null()
                || out_session.is_null()
                || out_result.is_null()
            {
                return write_session_failure(
                    out_result,
                    NuxStatus::NullArgument,
                    "configured session creation requires context, descriptor, session output, and result output",
                );
            }
            let descriptor = match unsafe { copy_configured_session_descriptor(descriptor) } {
                Ok(descriptor) => descriptor,
                Err(status) => {
                    return write_session_failure(
                        out_result,
                        status,
                        if status == NuxStatus::AbiMismatch {
                            "configured session requires ABI 1.2"
                        } else {
                            "configured session descriptor is malformed or oversized"
                        },
                    );
                }
            };
            let context = unsafe { &*context.cast::<FlowRuntimeContextHandle>() };
            match configured_session_seam::create(context, descriptor) {
                Ok((session, result)) => {
                    let result_status = replace_session_result(out_result, result);
                    if result_status == NuxStatus::Ok {
                        unsafe {
                            *out_session = Box::into_raw(session).cast();
                        }
                    }
                    result_status
                }
                Err(failure) => {
                    write_session_failure(out_result, failure.status, failure.diagnostic)
                }
            }
        },
    )
}

#[cfg(feature = "apple-product")]
#[unsafe(no_mangle)]
/// Performs one fully copied ABI 1.2 operation on the session's pinned worker.
/// Rust never calls Swift reentrantly; ordered outputs are returned in the owned
/// result. State batches are atomic and pointer batches preserve immediate
/// subcycles inside their returned `cycle` values.
///
/// # Safety
///
/// `session` must be live. The operation and every selected nested array/view
/// must remain readable for this synchronous call. `out_result` must be writable.
pub unsafe extern "C" fn nux_flow_render_session_perform(
    session: *const NuxFlowRenderSession,
    operation: *const NuxFlowSessionOperation,
    out_result: *mut *mut NuxFlowSessionResult,
) -> NuxStatus {
    ffi_guard_with_session_result(
        out_result,
        || unsafe { poison_session_handle(session) },
        || {
            reset_session_result(out_result);
            if operation.is_null() || out_result.is_null() {
                return write_session_failure(
                    out_result,
                    NuxStatus::NullArgument,
                    "session perform requires an operation and result output",
                );
            }
            let operation = match unsafe { copy_session_operation(operation) } {
                Ok(operation) => operation,
                Err(status) => {
                    return write_session_failure(
                        out_result,
                        status,
                        if status == NuxStatus::AbiMismatch {
                            "session operation requires ABI 1.2"
                        } else {
                            "session operation is malformed or exceeds a published bound"
                        },
                    );
                }
            };
            if session.is_null() {
                return write_session_failure(
                    out_result,
                    NuxStatus::NullArgument,
                    "session perform requires a live session",
                );
            }
            let (status, result) = configured_session_seam::perform(session, operation);
            let result_status = replace_session_result(out_result, result);
            if result_status == NuxStatus::Ok {
                status
            } else {
                result_status
            }
        },
    )
}

#[unsafe(no_mangle)]
/// Returns an ABI 1.2 session result's status, or `NULL_ARGUMENT` for null.
///
/// # Safety
///
/// A non-null pointer must identify a live result owned by this library.
pub unsafe extern "C" fn nux_flow_session_result_status(
    result: *const NuxFlowSessionResult,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if result.is_null() {
            NuxStatus::NullArgument
        } else {
            unsafe { (*result.cast::<FlowSessionResultHandle>()).status }
        }
    })
}

#[unsafe(no_mangle)]
/// Returns the exact Apple-surface disposition for this operation.
///
/// # Safety
///
/// A non-null pointer must identify a live result owned by this library.
pub unsafe extern "C" fn nux_flow_session_result_surface_disposition(
    result: *const NuxFlowSessionResult,
) -> NuxSurfaceDisposition {
    ffi_guard(NuxSurfaceDisposition::Fatal, || {
        if result.is_null() {
            NuxSurfaceDisposition::Fatal
        } else {
            unsafe { (*result.cast::<FlowSessionResultHandle>()).surface_disposition }
        }
    })
}

#[unsafe(no_mangle)]
/// Dirty and settled are independent runtime facts; this returns dirty only.
///
/// # Safety
///
/// A non-null pointer must identify a live result owned by this library.
pub unsafe extern "C" fn nux_flow_session_result_is_dirty(
    result: *const NuxFlowSessionResult,
) -> bool {
    ffi_guard(false, || {
        !result.is_null() && unsafe { (*result.cast::<FlowSessionResultHandle>()).is_dirty }
    })
}

#[unsafe(no_mangle)]
/// Dirty and settled are independent runtime facts; this returns settled only.
///
/// # Safety
///
/// A non-null pointer must identify a live result owned by this library.
pub unsafe extern "C" fn nux_flow_session_result_is_settled(
    result: *const NuxFlowSessionResult,
) -> bool {
    ffi_guard(false, || {
        !result.is_null() && unsafe { (*result.cast::<FlowSessionResultHandle>()).is_settled }
    })
}

#[unsafe(no_mangle)]
/// Writes the optional nonnegative app-clock delay until runtime work is due.
/// Returns `NOT_FOUND` when no wake is scheduled.
///
/// # Safety
///
/// `result` must be live and `out_wake_after_seconds` writable.
pub unsafe extern "C" fn nux_flow_session_result_wake_after_seconds(
    result: *const NuxFlowSessionResult,
    out_wake_after_seconds: *mut f64,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if out_wake_after_seconds.is_null() {
            return NuxStatus::NullArgument;
        }
        unsafe {
            *out_wake_after_seconds = 0.0;
        }
        if result.is_null() {
            return NuxStatus::NullArgument;
        }
        let Some(wake_after) = (unsafe { (*result.cast::<FlowSessionResultHandle>()).wake_after })
        else {
            return NuxStatus::NotFound;
        };
        unsafe {
            *out_wake_after_seconds = wake_after;
        }
        NuxStatus::Ok
    })
}

#[unsafe(no_mangle)]
/// Borrows bootstrap player metadata and exact authored artboard bounds.
/// Returns `NOT_FOUND` for operation results that do not carry bootstrap data.
///
/// # Safety
///
/// `result` must be live. `out_metadata` must be writable with its exact
/// published `struct_size`; returned views expire when `result` is freed.
pub unsafe extern "C" fn nux_flow_session_result_player_metadata(
    result: *const NuxFlowSessionResult,
    out_metadata: *mut NuxFlowPlayerMetadataView,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if out_metadata.is_null() {
            return NuxStatus::NullArgument;
        }
        if unsafe { read_struct_size(out_metadata) } != size_u32::<NuxFlowPlayerMetadataView>() {
            return NuxStatus::InvalidArgument;
        }
        unsafe {
            ptr::write_bytes(out_metadata, 0, 1);
            (*out_metadata).struct_size = size_u32::<NuxFlowPlayerMetadataView>();
        }
        if result.is_null() {
            return NuxStatus::NullArgument;
        }
        let handle = unsafe { &*result.cast::<FlowSessionResultHandle>() };
        let Some(metadata) = handle.player_metadata.as_ref() else {
            return NuxStatus::NotFound;
        };
        unsafe {
            *out_metadata = NuxFlowPlayerMetadataView {
                struct_size: size_u32::<NuxFlowPlayerMetadataView>(),
                kind: metadata.kind,
                selection: metadata.selection,
                player_index: metadata.player_index.unwrap_or(u32::MAX),
                artboard_name: borrowed_view(&metadata.artboard_name),
                player_name: borrowed_view(&metadata.player_name),
                min_x: metadata.min_x,
                min_y: metadata.min_y,
                max_x: metadata.max_x,
                max_y: metadata.max_y,
            };
        }
        NuxStatus::Ok
    })
}

#[unsafe(no_mangle)]
/// Returns the number of state-machine inputs returned by a player-input query.
///
/// # Safety
///
/// A non-null pointer must identify a live result owned by this library.
pub unsafe extern "C" fn nux_flow_session_result_player_input_count(
    result: *const NuxFlowSessionResult,
) -> u64 {
    ffi_guard(0, || {
        if result.is_null() {
            0
        } else {
            u64::try_from(
                unsafe { &*result.cast::<FlowSessionResultHandle>() }
                    .player_inputs
                    .len(),
            )
            .unwrap_or(u64::MAX)
        }
    })
}

#[unsafe(no_mangle)]
/// Borrows one state-machine input snapshot by authored order.
///
/// # Safety
///
/// `result` must be live. `out_input` must have the exact published size.
pub unsafe extern "C" fn nux_flow_session_result_player_input_at(
    result: *const NuxFlowSessionResult,
    index: u64,
    out_input: *mut NuxFlowPlayerInputView,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if out_input.is_null() {
            return NuxStatus::NullArgument;
        }
        if unsafe { read_struct_size(out_input) } != size_u32::<NuxFlowPlayerInputView>() {
            return NuxStatus::InvalidArgument;
        }
        unsafe {
            ptr::write_bytes(out_input, 0, 1);
            (*out_input).struct_size = size_u32::<NuxFlowPlayerInputView>();
        }
        if result.is_null() {
            return NuxStatus::NullArgument;
        }
        let Ok(index) = usize::try_from(index) else {
            return NuxStatus::NotFound;
        };
        let handle = unsafe { &*result.cast::<FlowSessionResultHandle>() };
        let Some(input) = handle.player_inputs.get(index) else {
            return NuxStatus::NotFound;
        };
        unsafe {
            *out_input = NuxFlowPlayerInputView {
                struct_size: size_u32::<NuxFlowPlayerInputView>(),
                kind: input.kind,
                value_root_index: input.value_root_index,
                name: borrowed_view(&input.name),
            };
        }
        NuxStatus::Ok
    })
}

#[unsafe(no_mangle)]
/// Returns the number of view-model schemas in this result.
///
/// # Safety
///
/// A non-null pointer must identify a live result owned by this library.
pub unsafe extern "C" fn nux_flow_session_result_schema_count(
    result: *const NuxFlowSessionResult,
) -> u64 {
    ffi_guard(0, || {
        if result.is_null() {
            0
        } else {
            u64::try_from(
                unsafe { &*result.cast::<FlowSessionResultHandle>() }
                    .schemas
                    .len(),
            )
            .unwrap_or(u64::MAX)
        }
    })
}

#[unsafe(no_mangle)]
/// Borrows one schema by stable result order.
///
/// # Safety
///
/// `result` must be live. `out_schema` must have the exact published size.
pub unsafe extern "C" fn nux_flow_session_result_schema_at(
    result: *const NuxFlowSessionResult,
    index: u64,
    out_schema: *mut NuxFlowSchemaView,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if out_schema.is_null() {
            return NuxStatus::NullArgument;
        }
        if unsafe { read_struct_size(out_schema) } != size_u32::<NuxFlowSchemaView>() {
            return NuxStatus::InvalidArgument;
        }
        unsafe {
            ptr::write_bytes(out_schema, 0, 1);
            (*out_schema).struct_size = size_u32::<NuxFlowSchemaView>();
        }
        if result.is_null() {
            return NuxStatus::NullArgument;
        }
        let Ok(index) = usize::try_from(index) else {
            return NuxStatus::NotFound;
        };
        let handle = unsafe { &*result.cast::<FlowSessionResultHandle>() };
        let Some(schema) = handle.schemas.get(index) else {
            return NuxStatus::NotFound;
        };
        unsafe {
            *out_schema = NuxFlowSchemaView {
                struct_size: size_u32::<NuxFlowSchemaView>(),
                first_property: schema.first_property,
                property_count: schema.property_count,
                schema_id: borrowed_view(&schema.schema_id),
                name: borrowed_view(&schema.name),
            };
        }
        NuxStatus::Ok
    })
}

#[unsafe(no_mangle)]
/// Returns the number of flattened schema properties in this result.
///
/// # Safety
///
/// A non-null pointer must identify a live result owned by this library.
pub unsafe extern "C" fn nux_flow_session_result_schema_property_count(
    result: *const NuxFlowSessionResult,
) -> u64 {
    ffi_guard(0, || {
        if result.is_null() {
            0
        } else {
            u64::try_from(
                unsafe { &*result.cast::<FlowSessionResultHandle>() }
                    .schema_properties
                    .len(),
            )
            .unwrap_or(u64::MAX)
        }
    })
}

#[unsafe(no_mangle)]
/// Borrows one flattened schema property by stable result order.
///
/// # Safety
///
/// `result` must be live. `out_property` must have the exact published size.
pub unsafe extern "C" fn nux_flow_session_result_schema_property_at(
    result: *const NuxFlowSessionResult,
    index: u64,
    out_property: *mut NuxFlowSchemaPropertyView,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if out_property.is_null() {
            return NuxStatus::NullArgument;
        }
        if unsafe { read_struct_size(out_property) } != size_u32::<NuxFlowSchemaPropertyView>() {
            return NuxStatus::InvalidArgument;
        }
        unsafe {
            ptr::write_bytes(out_property, 0, 1);
            (*out_property).struct_size = size_u32::<NuxFlowSchemaPropertyView>();
        }
        if result.is_null() {
            return NuxStatus::NullArgument;
        }
        let Ok(index) = usize::try_from(index) else {
            return NuxStatus::NotFound;
        };
        let handle = unsafe { &*result.cast::<FlowSessionResultHandle>() };
        let Some(property) = handle.schema_properties.get(index) else {
            return NuxStatus::NotFound;
        };
        unsafe {
            *out_property = NuxFlowSchemaPropertyView {
                struct_size: size_u32::<NuxFlowSchemaPropertyView>(),
                kind: property.kind,
                schema_id: borrowed_view(&property.schema_id),
                property_id: borrowed_view(&property.property_id),
                name: borrowed_view(&property.name),
            };
        }
        NuxStatus::Ok
    })
}

#[unsafe(no_mangle)]
/// Returns the number of authored immutable instance templates in this result.
///
/// # Safety
///
/// A non-null pointer must identify a live result owned by this library.
pub unsafe extern "C" fn nux_flow_session_result_instance_template_count(
    result: *const NuxFlowSessionResult,
) -> u64 {
    ffi_guard(0, || {
        if result.is_null() {
            0
        } else {
            u64::try_from(
                unsafe { &*result.cast::<FlowSessionResultHandle>() }
                    .instance_templates
                    .len(),
            )
            .unwrap_or(u64::MAX)
        }
    })
}

#[unsafe(no_mangle)]
/// Borrows one authored immutable instance template by result order.
///
/// # Safety
///
/// `result` must be live. `out_template` must have the exact published size.
pub unsafe extern "C" fn nux_flow_session_result_instance_template_at(
    result: *const NuxFlowSessionResult,
    index: u64,
    out_template: *mut NuxFlowInstanceTemplateView,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if out_template.is_null() {
            return NuxStatus::NullArgument;
        }
        if unsafe { read_struct_size(out_template) } != size_u32::<NuxFlowInstanceTemplateView>() {
            return NuxStatus::InvalidArgument;
        }
        unsafe {
            ptr::write_bytes(out_template, 0, 1);
            (*out_template).struct_size = size_u32::<NuxFlowInstanceTemplateView>();
        }
        if result.is_null() {
            return NuxStatus::NullArgument;
        }
        let Ok(index) = usize::try_from(index) else {
            return NuxStatus::NotFound;
        };
        let handle = unsafe { &*result.cast::<FlowSessionResultHandle>() };
        let Some(template) = handle.instance_templates.get(index) else {
            return NuxStatus::NotFound;
        };
        unsafe {
            *out_template = NuxFlowInstanceTemplateView {
                struct_size: size_u32::<NuxFlowInstanceTemplateView>(),
                authored_index: template.authored_index,
                schema_id: borrowed_view(&template.schema_id),
                authored_name: borrowed_view(&template.authored_name),
            };
        }
        NuxStatus::Ok
    })
}

#[unsafe(no_mangle)]
/// Returns the number of stable external instances in this result.
///
/// # Safety
///
/// A non-null pointer must identify a live result owned by this library.
pub unsafe extern "C" fn nux_flow_session_result_instance_count(
    result: *const NuxFlowSessionResult,
) -> u64 {
    ffi_guard(0, || {
        if result.is_null() {
            0
        } else {
            u64::try_from(
                unsafe { &*result.cast::<FlowSessionResultHandle>() }
                    .instances
                    .len(),
            )
            .unwrap_or(u64::MAX)
        }
    })
}

#[unsafe(no_mangle)]
/// Borrows one stable external instance by result order.
///
/// # Safety
///
/// `result` must be live. `out_instance` must have the exact published size.
pub unsafe extern "C" fn nux_flow_session_result_instance_at(
    result: *const NuxFlowSessionResult,
    index: u64,
    out_instance: *mut NuxFlowInstanceView,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if out_instance.is_null() {
            return NuxStatus::NullArgument;
        }
        if unsafe { read_struct_size(out_instance) } != size_u32::<NuxFlowInstanceView>() {
            return NuxStatus::InvalidArgument;
        }
        unsafe {
            ptr::write_bytes(out_instance, 0, 1);
            (*out_instance).struct_size = size_u32::<NuxFlowInstanceView>();
        }
        if result.is_null() {
            return NuxStatus::NullArgument;
        }
        let Ok(index) = usize::try_from(index) else {
            return NuxStatus::NotFound;
        };
        let handle = unsafe { &*result.cast::<FlowSessionResultHandle>() };
        let Some(instance) = handle.instances.get(index) else {
            return NuxStatus::NotFound;
        };
        unsafe {
            *out_instance = NuxFlowInstanceView {
                struct_size: size_u32::<NuxFlowInstanceView>(),
                value_root_index: instance.value_root_index.unwrap_or(NO_VALUE_ROOT),
                is_root: u32::from(instance.is_root),
                instance_id: instance.instance_id,
                schema_id: borrowed_view(&instance.schema_id),
                name: borrowed_view(&instance.name),
            };
        }
        NuxStatus::Ok
    })
}

#[unsafe(no_mangle)]
/// Returns the number of instance-to-value roots in this result.
///
/// # Safety
///
/// A non-null pointer must identify a live result owned by this library.
pub unsafe extern "C" fn nux_flow_session_result_value_root_count(
    result: *const NuxFlowSessionResult,
) -> u64 {
    ffi_guard(0, || {
        if result.is_null() {
            0
        } else {
            u64::try_from(
                unsafe { &*result.cast::<FlowSessionResultHandle>() }
                    .value_roots
                    .len(),
            )
            .unwrap_or(u64::MAX)
        }
    })
}

#[unsafe(no_mangle)]
/// Borrows one instance-to-value root by result order.
///
/// # Safety
///
/// `result` must be live. `out_root` must have the exact published size.
pub unsafe extern "C" fn nux_flow_session_result_value_root_at(
    result: *const NuxFlowSessionResult,
    index: u64,
    out_root: *mut NuxFlowValueRootView,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if out_root.is_null() {
            return NuxStatus::NullArgument;
        }
        if unsafe { read_struct_size(out_root) } != size_u32::<NuxFlowValueRootView>() {
            return NuxStatus::InvalidArgument;
        }
        unsafe {
            ptr::write_bytes(out_root, 0, 1);
            (*out_root).struct_size = size_u32::<NuxFlowValueRootView>();
        }
        if result.is_null() {
            return NuxStatus::NullArgument;
        }
        let Ok(index) = usize::try_from(index) else {
            return NuxStatus::NotFound;
        };
        let handle = unsafe { &*result.cast::<FlowSessionResultHandle>() };
        let Some(root) = handle.value_roots.get(index) else {
            return NuxStatus::NotFound;
        };
        unsafe {
            *out_root = NuxFlowValueRootView {
                struct_size: size_u32::<NuxFlowValueRootView>(),
                value_root_index: root.value_root_index,
                instance_id: root.instance_id,
            };
        }
        NuxStatus::Ok
    })
}

#[unsafe(no_mangle)]
/// Returns the number of local-to-stable instance mappings created by a batch.
///
/// # Safety
///
/// A non-null pointer must identify a live result owned by this library.
pub unsafe extern "C" fn nux_flow_session_result_created_instance_count(
    result: *const NuxFlowSessionResult,
) -> u64 {
    ffi_guard(0, || {
        if result.is_null() {
            0
        } else {
            u64::try_from(
                unsafe { &*result.cast::<FlowSessionResultHandle>() }
                    .created_instances
                    .len(),
            )
            .unwrap_or(u64::MAX)
        }
    })
}

#[unsafe(no_mangle)]
/// Borrows one local-to-stable instance mapping by result order.
///
/// # Safety
///
/// `result` must be live. `out_created` must have the exact published size.
pub unsafe extern "C" fn nux_flow_session_result_created_instance_at(
    result: *const NuxFlowSessionResult,
    index: u64,
    out_created: *mut NuxFlowCreatedInstanceView,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if out_created.is_null() {
            return NuxStatus::NullArgument;
        }
        if unsafe { read_struct_size(out_created) } != size_u32::<NuxFlowCreatedInstanceView>() {
            return NuxStatus::InvalidArgument;
        }
        unsafe {
            ptr::write_bytes(out_created, 0, 1);
            (*out_created).struct_size = size_u32::<NuxFlowCreatedInstanceView>();
        }
        if result.is_null() {
            return NuxStatus::NullArgument;
        }
        let Ok(index) = usize::try_from(index) else {
            return NuxStatus::NotFound;
        };
        let handle = unsafe { &*result.cast::<FlowSessionResultHandle>() };
        let Some(created) = handle.created_instances.get(index) else {
            return NuxStatus::NotFound;
        };
        unsafe {
            *out_created = NuxFlowCreatedInstanceView {
                struct_size: size_u32::<NuxFlowCreatedInstanceView>(),
                local_id: created.local_id,
                instance_id: created.instance_id,
            };
        }
        NuxStatus::Ok
    })
}

#[unsafe(no_mangle)]
/// Returns the number of nodes in the result-owned value arena.
///
/// # Safety
///
/// A non-null pointer must identify a live result owned by this library.
pub unsafe extern "C" fn nux_flow_session_result_value_node_count(
    result: *const NuxFlowSessionResult,
) -> u64 {
    ffi_guard(0, || {
        if result.is_null() {
            0
        } else {
            u64::try_from(
                unsafe { &*result.cast::<FlowSessionResultHandle>() }
                    .value_arena
                    .nodes
                    .len(),
            )
            .unwrap_or(u64::MAX)
        }
    })
}

#[unsafe(no_mangle)]
/// Borrows one result-owned value node by arena index.
///
/// # Safety
///
/// `result` must be live. `out_node` must have the exact published size.
pub unsafe extern "C" fn nux_flow_session_result_value_node_at(
    result: *const NuxFlowSessionResult,
    index: u64,
    out_node: *mut NuxFlowValueNode,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if out_node.is_null() {
            return NuxStatus::NullArgument;
        }
        if unsafe { read_struct_size(out_node) } != size_u32::<NuxFlowValueNode>() {
            return NuxStatus::InvalidArgument;
        }
        unsafe {
            ptr::write_bytes(out_node, 0, 1);
            (*out_node).struct_size = size_u32::<NuxFlowValueNode>();
        }
        if result.is_null() {
            return NuxStatus::NullArgument;
        }
        let Ok(index) = usize::try_from(index) else {
            return NuxStatus::NotFound;
        };
        let handle = unsafe { &*result.cast::<FlowSessionResultHandle>() };
        let Some(node) = handle.value_arena.nodes.get(index) else {
            return NuxStatus::NotFound;
        };
        unsafe {
            *out_node = NuxFlowValueNode {
                struct_size: size_u32::<NuxFlowValueNode>(),
                kind: node.kind,
                number_value: node.number_value,
                color_value: node.color_value,
                bool_value: u32::from(node.bool_value),
                first_edge: node.first_edge,
                edge_count: node.edge_count,
                has_instance_id: u32::from(node.instance_id.is_some()),
                instance_id: node.instance_id.unwrap_or(0),
                identity_value: node.identity_value,
                string_value: borrowed_view(&node.string_value),
                schema_id: borrowed_view(&node.schema_id),
            };
        }
        NuxStatus::Ok
    })
}

#[unsafe(no_mangle)]
/// Returns the number of edges in the result-owned value arena.
///
/// # Safety
///
/// A non-null pointer must identify a live result owned by this library.
pub unsafe extern "C" fn nux_flow_session_result_value_edge_count(
    result: *const NuxFlowSessionResult,
) -> u64 {
    ffi_guard(0, || {
        if result.is_null() {
            0
        } else {
            u64::try_from(
                unsafe { &*result.cast::<FlowSessionResultHandle>() }
                    .value_arena
                    .edges
                    .len(),
            )
            .unwrap_or(u64::MAX)
        }
    })
}

#[unsafe(no_mangle)]
/// Borrows one result-owned value edge by arena index.
///
/// # Safety
///
/// `result` must be live. `out_edge` must have the exact published size.
pub unsafe extern "C" fn nux_flow_session_result_value_edge_at(
    result: *const NuxFlowSessionResult,
    index: u64,
    out_edge: *mut NuxFlowValueEdge,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if out_edge.is_null() {
            return NuxStatus::NullArgument;
        }
        if unsafe { read_struct_size(out_edge) } != size_u32::<NuxFlowValueEdge>() {
            return NuxStatus::InvalidArgument;
        }
        unsafe {
            ptr::write_bytes(out_edge, 0, 1);
            (*out_edge).struct_size = size_u32::<NuxFlowValueEdge>();
        }
        if result.is_null() {
            return NuxStatus::NullArgument;
        }
        let Ok(index) = usize::try_from(index) else {
            return NuxStatus::NotFound;
        };
        let handle = unsafe { &*result.cast::<FlowSessionResultHandle>() };
        let Some(edge) = handle.value_arena.edges.get(index) else {
            return NuxStatus::NotFound;
        };
        unsafe {
            *out_edge = NuxFlowValueEdge {
                struct_size: size_u32::<NuxFlowValueEdge>(),
                node_index: edge.node_index,
                key: borrowed_view(&edge.key),
            };
        }
        NuxStatus::Ok
    })
}

#[unsafe(no_mangle)]
/// Returns the number of exact-order outputs in this result.
///
/// # Safety
///
/// A non-null pointer must identify a live result owned by this library.
pub unsafe extern "C" fn nux_flow_session_result_output_count(
    result: *const NuxFlowSessionResult,
) -> u64 {
    ffi_guard(0, || {
        if result.is_null() {
            0
        } else {
            u64::try_from(
                unsafe { &*result.cast::<FlowSessionResultHandle>() }
                    .outputs
                    .len(),
            )
            .unwrap_or(u64::MAX)
        }
    })
}

#[unsafe(no_mangle)]
/// Borrows one exact-order output by result order.
///
/// # Safety
///
/// `result` must be live. `out_output` must have the exact published size.
pub unsafe extern "C" fn nux_flow_session_result_output_at(
    result: *const NuxFlowSessionResult,
    index: u64,
    out_output: *mut NuxFlowOutputView,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if out_output.is_null() {
            return NuxStatus::NullArgument;
        }
        if unsafe { read_struct_size(out_output) } != size_u32::<NuxFlowOutputView>() {
            return NuxStatus::InvalidArgument;
        }
        unsafe {
            ptr::write_bytes(out_output, 0, 1);
            (*out_output).struct_size = size_u32::<NuxFlowOutputView>();
        }
        if result.is_null() {
            return NuxStatus::NullArgument;
        }
        let Ok(index) = usize::try_from(index) else {
            return NuxStatus::NotFound;
        };
        let handle = unsafe { &*result.cast::<FlowSessionResultHandle>() };
        let Some(output) = handle.outputs.get(index) else {
            return NuxStatus::NotFound;
        };
        unsafe {
            *out_output = NuxFlowOutputView {
                struct_size: size_u32::<NuxFlowOutputView>(),
                phase: output.phase,
                kind: output.kind,
                payload_root_index: output.payload_root_index.unwrap_or(NO_VALUE_ROOT),
                has_origin_mutation_id: u32::from(output.origin_mutation_id.is_some()),
                has_instance_id: u32::from(output.instance_id.is_some()),
                sequence: output.sequence,
                cycle: output.cycle,
                origin_mutation_id: output.origin_mutation_id.unwrap_or(0),
                instance_id: output.instance_id.unwrap_or(0),
                event_type: output.event_type,
                first_event_property: output.first_event_property,
                event_property_count: output.event_property_count,
                delay_seconds: output.delay_seconds,
                name: borrowed_view(&output.name),
                path: borrowed_view(&output.path),
                payload: borrowed_view(&output.payload),
            };
        }
        NuxStatus::Ok
    })
}

#[unsafe(no_mangle)]
/// Returns the number of flattened typed event properties in this result.
///
/// # Safety
///
/// A non-null pointer must identify a live result owned by this library.
pub unsafe extern "C" fn nux_flow_session_result_event_property_count(
    result: *const NuxFlowSessionResult,
) -> u64 {
    ffi_guard(0, || {
        if result.is_null() {
            0
        } else {
            u64::try_from(
                unsafe { &*result.cast::<FlowSessionResultHandle>() }
                    .event_properties
                    .len(),
            )
            .unwrap_or(u64::MAX)
        }
    })
}

#[unsafe(no_mangle)]
/// Borrows one flattened typed event property by result order.
///
/// # Safety
///
/// `result` must be live. `out_property` must have the exact published size.
pub unsafe extern "C" fn nux_flow_session_result_event_property_at(
    result: *const NuxFlowSessionResult,
    index: u64,
    out_property: *mut NuxFlowEventPropertyView,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if out_property.is_null() {
            return NuxStatus::NullArgument;
        }
        if unsafe { read_struct_size(out_property) } != size_u32::<NuxFlowEventPropertyView>() {
            return NuxStatus::InvalidArgument;
        }
        unsafe {
            ptr::write_bytes(out_property, 0, 1);
            (*out_property).struct_size = size_u32::<NuxFlowEventPropertyView>();
        }
        if result.is_null() {
            return NuxStatus::NullArgument;
        }
        let Ok(index) = usize::try_from(index) else {
            return NuxStatus::NotFound;
        };
        let handle = unsafe { &*result.cast::<FlowSessionResultHandle>() };
        let Some(property) = handle.event_properties.get(index) else {
            return NuxStatus::NotFound;
        };
        unsafe {
            *out_property = NuxFlowEventPropertyView {
                struct_size: size_u32::<NuxFlowEventPropertyView>(),
                value_root_index: property.value_root_index.unwrap_or(NO_VALUE_ROOT),
                has_trigger_count: u32::from(property.trigger_count.is_some()),
                trigger_count: property.trigger_count.unwrap_or(0),
                name: borrowed_view(&property.name),
            };
        }
        NuxStatus::Ok
    })
}

#[unsafe(no_mangle)]
/// Returns the number of phase-ordered diagnostics in this result.
///
/// # Safety
///
/// A non-null pointer must identify a live result owned by this library.
pub unsafe extern "C" fn nux_flow_session_result_diagnostic_count(
    result: *const NuxFlowSessionResult,
) -> u64 {
    ffi_guard(0, || {
        if result.is_null() {
            0
        } else {
            u64::try_from(
                unsafe { &*result.cast::<FlowSessionResultHandle>() }
                    .diagnostics
                    .len(),
            )
            .unwrap_or(u64::MAX)
        }
    })
}

#[unsafe(no_mangle)]
/// Borrows one structured diagnostic by result order.
///
/// # Safety
///
/// `result` must be live. `out_diagnostic` must have ABI 1.1's exact frozen
/// diagnostic-view size; returned byte views expire when `result` is freed.
pub unsafe extern "C" fn nux_flow_session_result_diagnostic_at(
    result: *const NuxFlowSessionResult,
    index: u64,
    out_diagnostic: *mut NuxDiagnosticView,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if out_diagnostic.is_null() {
            return NuxStatus::NullArgument;
        }
        if unsafe { read_struct_size(out_diagnostic) } != size_u32::<NuxDiagnosticView>() {
            return NuxStatus::InvalidArgument;
        }
        unsafe {
            *out_diagnostic = NuxDiagnosticView::default();
        }
        if result.is_null() {
            return NuxStatus::NullArgument;
        }
        let Ok(index) = usize::try_from(index) else {
            return NuxStatus::NotFound;
        };
        let handle = unsafe { &*result.cast::<FlowSessionResultHandle>() };
        let Some(diagnostic) = handle.diagnostics.get(index) else {
            return NuxStatus::NotFound;
        };
        unsafe {
            *out_diagnostic = NuxDiagnosticView {
                struct_size: size_u32::<NuxDiagnosticView>(),
                severity: diagnostic.severity,
                code: borrowed_view(&diagnostic.code),
                message: borrowed_view(&diagnostic.message),
            };
        }
        NuxStatus::Ok
    })
}

#[unsafe(no_mangle)]
/// Releases one ABI 1.2 session result. Null is a no-op.
///
/// # Safety
///
/// A non-null pointer must be an owned result returned by this library and must
/// not have been freed before. No borrowed view may be used after this call.
pub unsafe extern "C" fn nux_flow_session_result_free(result: *mut NuxFlowSessionResult) {
    ffi_guard((), || {
        if !result.is_null() {
            unsafe {
                drop(Box::from_raw(result.cast::<FlowSessionResultHandle>()));
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn bytes(value: &[u8]) -> NuxByteView {
        NuxByteView {
            data: value.as_ptr(),
            len: value.len() as u64,
        }
    }

    fn configured_descriptor() -> NuxFlowConfiguredSessionDescriptor {
        NuxFlowConfiguredSessionDescriptor {
            struct_size: size_u32::<NuxFlowConfiguredSessionDescriptor>(),
            required_abi_major: 1,
            minimum_abi_minor: 2,
            artboard_name: NuxByteView::default(),
            player_name: NuxByteView::default(),
        }
    }

    fn operation(kind: NuxFlowSessionOperationKind) -> NuxFlowSessionOperation {
        NuxFlowSessionOperation {
            struct_size: size_u32::<NuxFlowSessionOperation>(),
            required_abi_major: 1,
            minimum_abi_minor: 2,
            kind,
            state_batch: ptr::null(),
            pointer_batch: ptr::null(),
            advance: ptr::null(),
            query_batch: ptr::null(),
        }
    }

    fn null_node(kind: NuxFlowValueKind) -> NuxFlowValueNode {
        NuxFlowValueNode {
            struct_size: size_u32::<NuxFlowValueNode>(),
            kind,
            number_value: 0.0,
            color_value: 0,
            bool_value: 0,
            first_edge: 0,
            edge_count: 0,
            has_instance_id: 0,
            instance_id: 0,
            identity_value: 0,
            string_value: NuxByteView::default(),
            schema_id: NuxByteView::default(),
        }
    }

    #[test]
    fn abi_12_handshake_preserves_abi_11_compatibility() {
        assert_eq!(NUX_RUNTIME_ABI_MAJOR, 1);
        assert_eq!(NUX_RUNTIME_ABI_MINOR, 2);
        assert_eq!(MINIMUM_SUPPORTED_ABI_MINOR, 1);
        assert_eq!(nux_runtime_require_abi(1, 1), NuxStatus::Ok);
        assert_eq!(nux_runtime_require_abi(1, 2), NuxStatus::Ok);
        assert_eq!(nux_runtime_require_abi(1, 3), NuxStatus::AbiMismatch);
        assert_eq!(nux_runtime_require_abi(2, 1), NuxStatus::AbiMismatch);
    }

    #[test]
    fn abi_12_layouts_are_fixed_width_and_abi_11_layouts_are_unchanged() {
        assert_eq!(std::mem::size_of::<NuxFlowSessionDescriptor>(), 40);
        assert_eq!(std::mem::size_of::<NuxFrameOperation>(), 40);
        assert_eq!(
            std::mem::size_of::<NuxFlowConfiguredSessionDescriptor>(),
            40
        );
        assert_eq!(std::mem::size_of::<NuxFlowValueNode>(), 88);
        assert_eq!(std::mem::size_of::<NuxFlowValueEdge>(), 24);
        assert_eq!(std::mem::size_of::<NuxFlowValueArena>(), 40);
        assert_eq!(std::mem::size_of::<NuxFlowNewInstance>(), 40);
        assert_eq!(std::mem::size_of::<NuxFlowInstanceReference>(), 16);
        assert_eq!(std::mem::size_of::<NuxFlowStateMutation>(), 88);
        assert_eq!(std::mem::size_of::<NuxFlowStateBatch>(), 56);
        assert_eq!(std::mem::size_of::<NuxFlowPointerEvent>(), 20);
        assert_eq!(std::mem::size_of::<NuxFlowPointerBatch>(), 24);
        assert_eq!(std::mem::size_of::<NuxFlowAdvanceOperation>(), 48);
        assert_eq!(std::mem::size_of::<NuxFlowQuery>(), 8);
        assert_eq!(std::mem::size_of::<NuxFlowSessionOperation>(), 48);
        assert_eq!(std::mem::size_of::<NuxFlowPlayerMetadataView>(), 64);
        assert_eq!(std::mem::size_of::<NuxFlowPlayerInputView>(), 32);
    }

    #[test]
    fn configured_descriptor_rejects_wrong_versions_and_malformed_selectors() {
        let mut descriptor = configured_descriptor();
        descriptor.minimum_abi_minor = 1;
        assert!(matches!(
            unsafe { copy_configured_session_descriptor(&descriptor) },
            Err(NUX_STATUS_ABI_MISMATCH)
        ));
        descriptor.minimum_abi_minor = 3;
        assert!(matches!(
            unsafe { copy_configured_session_descriptor(&descriptor) },
            Err(NUX_STATUS_ABI_MISMATCH)
        ));
        descriptor.minimum_abi_minor = 2;
        descriptor.required_abi_major = 2;
        assert!(matches!(
            unsafe { copy_configured_session_descriptor(&descriptor) },
            Err(NUX_STATUS_ABI_MISMATCH)
        ));
        descriptor.required_abi_major = 1;
        descriptor.player_name = NuxByteView {
            data: ptr::dangling(),
            len: 0,
        };
        assert!(matches!(
            unsafe { copy_configured_session_descriptor(&descriptor) },
            Err(NUX_STATUS_INVALID_ARGUMENT)
        ));
    }

    #[test]
    fn value_arena_rejects_cycles_nonfinite_values_and_total_payload_overflow() {
        let edge = NuxFlowValueEdge {
            struct_size: size_u32::<NuxFlowValueEdge>(),
            node_index: 0,
            key: bytes(b"self"),
        };
        let mut object = null_node(NUX_FLOW_VALUE_KIND_OBJECT);
        object.edge_count = 1;
        let arena = NuxFlowValueArena {
            struct_size: size_u32::<NuxFlowValueArena>(),
            nodes: &object,
            node_count: 1,
            edges: &edge,
            edge_count: 1,
        };
        assert!(matches!(
            unsafe { copy_value_arena(&arena, &mut PayloadBudget::default()) },
            Err(NUX_STATUS_INVALID_ARGUMENT)
        ));

        let mut number = null_node(NUX_FLOW_VALUE_KIND_NUMBER);
        number.number_value = f64::NAN;
        let arena = NuxFlowValueArena {
            struct_size: size_u32::<NuxFlowValueArena>(),
            nodes: &number,
            node_count: 1,
            edges: ptr::null(),
            edge_count: 0,
        };
        assert!(matches!(
            unsafe { copy_value_arena(&arena, &mut PayloadBudget::default()) },
            Err(NUX_STATUS_INVALID_ARGUMENT)
        ));

        let mut overflow_number = null_node(NUX_FLOW_VALUE_KIND_NUMBER);
        overflow_number.number_value = f64::MAX;
        let arena = NuxFlowValueArena {
            nodes: &overflow_number,
            ..arena
        };
        assert!(matches!(
            unsafe { copy_value_arena(&arena, &mut PayloadBudget::default()) },
            Err(NUX_STATUS_INVALID_ARGUMENT)
        ));

        let megabyte = vec![b'x'; NUX_FLOW_MAX_STRING_BYTE_LENGTH as usize];
        let nodes = (0..5)
            .map(|_| {
                let mut node = null_node(NUX_FLOW_VALUE_KIND_STRING);
                node.string_value = bytes(&megabyte);
                node
            })
            .collect::<Vec<_>>();
        let arena = NuxFlowValueArena {
            struct_size: size_u32::<NuxFlowValueArena>(),
            nodes: nodes.as_ptr(),
            node_count: nodes.len() as u64,
            edges: ptr::null(),
            edge_count: 0,
        };
        assert!(matches!(
            unsafe { copy_value_arena(&arena, &mut PayloadBudget::default()) },
            Err(NUX_STATUS_INVALID_ARGUMENT)
        ));
    }

    #[test]
    fn value_depth_counts_the_root_at_zero() {
        let arena_with_edge_depth = |edge_depth: usize| {
            let mut nodes = (0..=edge_depth)
                .map(|_| null_node(NUX_FLOW_VALUE_KIND_LIST))
                .collect::<Vec<_>>();
            let edges = (0..edge_depth)
                .map(|index| NuxFlowValueEdge {
                    struct_size: size_u32::<NuxFlowValueEdge>(),
                    node_index: (index + 1) as u32,
                    key: NuxByteView::default(),
                })
                .collect::<Vec<_>>();
            for (index, node) in nodes.iter_mut().take(edge_depth).enumerate() {
                node.first_edge = index as u32;
                node.edge_count = 1;
            }
            (nodes, edges)
        };

        let (nodes, edges) = arena_with_edge_depth(NUX_FLOW_MAX_VALUE_DEPTH as usize);
        let arena = NuxFlowValueArena {
            struct_size: size_u32::<NuxFlowValueArena>(),
            nodes: nodes.as_ptr(),
            node_count: nodes.len() as u64,
            edges: edges.as_ptr(),
            edge_count: edges.len() as u64,
        };
        assert!(unsafe { copy_value_arena(&arena, &mut PayloadBudget::default()) }.is_ok());

        let (nodes, edges) = arena_with_edge_depth(NUX_FLOW_MAX_VALUE_DEPTH as usize + 1);
        let arena = NuxFlowValueArena {
            nodes: nodes.as_ptr(),
            node_count: nodes.len() as u64,
            edges: edges.as_ptr(),
            edge_count: edges.len() as u64,
            ..arena
        };
        assert!(matches!(
            unsafe { copy_value_arena(&arena, &mut PayloadBudget::default()) },
            Err(NUX_STATUS_INVALID_ARGUMENT)
        ));
    }

    #[test]
    fn pointer_and_operation_bounds_fail_before_the_worker_seam() {
        let event = NuxFlowPointerEvent {
            struct_size: size_u32::<NuxFlowPointerEvent>(),
            kind: NUX_FLOW_POINTER_EVENT_KIND_DOWN,
            pointer_id: 1,
            x: 0.0,
            y: 0.0,
        };
        let batch = NuxFlowPointerBatch {
            struct_size: size_u32::<NuxFlowPointerBatch>(),
            events: &event,
            event_count: NUX_FLOW_MAX_POINTER_COUNT + 1,
        };
        assert!(matches!(
            unsafe { copy_pointer_batch(&batch) },
            Err(NUX_STATUS_INVALID_ARGUMENT)
        ));

        let mut invalid_event = event;
        invalid_event.pointer_id = 0;
        let batch = NuxFlowPointerBatch {
            struct_size: size_u32::<NuxFlowPointerBatch>(),
            events: &invalid_event,
            event_count: 1,
        };
        assert!(matches!(
            unsafe { copy_pointer_batch(&batch) },
            Err(NUX_STATUS_INVALID_ARGUMENT)
        ));

        let mut request = operation(NUX_FLOW_SESSION_OPERATION_KIND_POINTER_BATCH);
        request.pointer_batch = &batch;
        request.query_batch = ptr::dangling();
        assert!(matches!(
            unsafe { copy_session_operation(&request) },
            Err(NUX_STATUS_INVALID_ARGUMENT)
        ));
    }

    #[cfg(feature = "apple-product")]
    #[test]
    fn state_batch_is_fully_validated_before_the_worker_seam() {
        let mut value = null_node(NUX_FLOW_VALUE_KIND_NUMBER);
        value.number_value = 42.0;
        let arena = NuxFlowValueArena {
            struct_size: size_u32::<NuxFlowValueArena>(),
            nodes: &value,
            node_count: 1,
            edges: ptr::null(),
            edge_count: 0,
        };
        let mutation = NuxFlowStateMutation {
            struct_size: size_u32::<NuxFlowStateMutation>(),
            kind: NUX_FLOW_STATE_MUTATION_KIND_SET,
            instance: NuxFlowInstanceReference {
                kind: NUX_FLOW_INSTANCE_REFERENCE_KIND_EXISTING,
                local_id: 0,
                instance_id: 7,
            },
            item: NuxFlowInstanceReference {
                kind: 0,
                local_id: 0,
                instance_id: 0,
            },
            path: bytes(b"score"),
            input_name: NuxByteView::default(),
            value_root_index: 0,
            index: 0,
            other_index: 0,
        };
        let batch = NuxFlowStateBatch {
            struct_size: size_u32::<NuxFlowStateBatch>(),
            has_host_mutation_id: 1,
            host_mutation_id: 99,
            value_arena: &arena,
            new_instances: ptr::null(),
            new_instance_count: 0,
            mutations: &mutation,
            mutation_count: 1,
        };
        let mut request = operation(NUX_FLOW_SESSION_OPERATION_KIND_STATE_BATCH);
        request.state_batch = &batch;
        let mut result = ptr::null_mut();

        assert_eq!(
            unsafe { nux_flow_render_session_perform(ptr::null(), &request, &mut result) },
            NuxStatus::NullArgument,
            "a valid copied batch reaches live-session validation"
        );
        unsafe { nux_flow_session_result_free(result) };

        let present_zero_batch = NuxFlowStateBatch {
            has_host_mutation_id: 1,
            host_mutation_id: 0,
            ..batch
        };
        request.state_batch = &present_zero_batch;
        result = ptr::null_mut();
        assert_eq!(
            unsafe { nux_flow_render_session_perform(ptr::null(), &request, &mut result) },
            NuxStatus::NullArgument,
            "the presence bit preserves host mutation ID zero"
        );
        unsafe { nux_flow_session_result_free(result) };

        let invalid_batch = NuxFlowStateBatch {
            has_host_mutation_id: 0,
            ..batch
        };
        request.state_batch = &invalid_batch;
        result = ptr::null_mut();
        assert_eq!(
            unsafe { nux_flow_render_session_perform(ptr::null(), &request, &mut result) },
            NuxStatus::InvalidArgument,
            "noncanonical optional IDs are rejected before worker dispatch"
        );
        unsafe { nux_flow_session_result_free(result) };

        let new_instance = NuxFlowNewInstance {
            struct_size: size_u32::<NuxFlowNewInstance>(),
            local_id: 0,
            schema_name: NuxByteView::default(),
            authored_instance_name: NuxByteView::default(),
        };
        let instances = vec![new_instance; 2_048];
        let mutations = vec![mutation; 2_049];
        let oversized_batch = NuxFlowStateBatch {
            new_instances: instances.as_ptr(),
            new_instance_count: instances.len() as u64,
            mutations: mutations.as_ptr(),
            mutation_count: mutations.len() as u64,
            ..batch
        };
        request.state_batch = &oversized_batch;
        result = ptr::null_mut();
        assert_eq!(
            unsafe { nux_flow_render_session_perform(ptr::null(), &request, &mut result) },
            NuxStatus::InvalidArgument,
            "new instances and mutations share the 4096-item batch cap"
        );
        unsafe { nux_flow_session_result_free(result) };
    }

    #[cfg(feature = "apple-product")]
    #[test]
    fn player_input_mutations_require_a_name_and_matching_scalar_kind() {
        let mut value = null_node(NUX_FLOW_VALUE_KIND_BOOL);
        value.bool_value = 1;
        let arena = NuxFlowValueArena {
            struct_size: size_u32::<NuxFlowValueArena>(),
            nodes: &value,
            node_count: 1,
            edges: ptr::null(),
            edge_count: 0,
        };
        let zero_reference = NuxFlowInstanceReference {
            kind: 0,
            local_id: 0,
            instance_id: 0,
        };
        let mutation = NuxFlowStateMutation {
            struct_size: size_u32::<NuxFlowStateMutation>(),
            kind: NUX_FLOW_STATE_MUTATION_KIND_SET_INPUT_BOOL,
            instance: zero_reference,
            item: zero_reference,
            path: NuxByteView::default(),
            input_name: bytes(b"enabled"),
            value_root_index: 0,
            index: 0,
            other_index: 0,
        };
        let perform_without_session = |mutation: &NuxFlowStateMutation| {
            let batch = NuxFlowStateBatch {
                struct_size: size_u32::<NuxFlowStateBatch>(),
                has_host_mutation_id: 0,
                host_mutation_id: 0,
                value_arena: &arena,
                new_instances: ptr::null(),
                new_instance_count: 0,
                mutations: mutation,
                mutation_count: 1,
            };
            let mut request = operation(NUX_FLOW_SESSION_OPERATION_KIND_STATE_BATCH);
            request.state_batch = &batch;
            let mut result = ptr::null_mut();
            let status =
                unsafe { nux_flow_render_session_perform(ptr::null(), &request, &mut result) };
            unsafe { nux_flow_session_result_free(result) };
            status
        };

        assert_eq!(perform_without_session(&mutation), NuxStatus::NullArgument);

        let wrong_kind = NuxFlowStateMutation {
            kind: NUX_FLOW_STATE_MUTATION_KIND_SET_INPUT_NUMBER,
            ..mutation
        };
        assert_eq!(
            perform_without_session(&wrong_kind),
            NuxStatus::InvalidArgument
        );

        let missing_name = NuxFlowStateMutation {
            input_name: NuxByteView::default(),
            ..mutation
        };
        assert_eq!(
            perform_without_session(&missing_name),
            NuxStatus::InvalidArgument
        );

        let oversized_name = vec![b'x'; NUX_FLOW_MAX_ID_BYTE_LENGTH as usize + 1];
        let oversized = NuxFlowStateMutation {
            input_name: bytes(&oversized_name),
            ..mutation
        };
        assert_eq!(
            perform_without_session(&oversized),
            NuxStatus::InvalidArgument
        );
    }

    unsafe extern "C" fn count_completion(context: *mut c_void) {
        let counter = unsafe { &*context.cast::<AtomicUsize>() };
        counter.fetch_add(1, Ordering::SeqCst);
    }

    #[cfg(feature = "apple-product")]
    #[test]
    fn rejected_advance_consumes_the_completion_exactly_once() {
        let counter = AtomicUsize::new(0);
        let advance = NuxFlowAdvanceOperation {
            struct_size: size_u32::<NuxFlowAdvanceOperation>(),
            timestamp_seconds: 1.0,
            delta_seconds: 0.0,
            render: 1,
            apple_drawable: ptr::dangling_mut(),
            completion_context: (&counter as *const AtomicUsize).cast_mut().cast(),
            completion_callback: Some(count_completion),
        };
        let mut request = operation(NUX_FLOW_SESSION_OPERATION_KIND_ADVANCE);
        request.advance = &advance;
        let mut result = ptr::null_mut();
        assert_eq!(
            unsafe { nux_flow_render_session_perform(ptr::null(), &request, &mut result) },
            NuxStatus::NullArgument
        );
        assert_eq!(counter.load(Ordering::SeqCst), 1);
        assert!(!result.is_null());
        unsafe { nux_flow_session_result_free(result) };
    }

    #[cfg(feature = "apple-product")]
    #[test]
    fn advance_rejects_noncanonical_render_flags_before_worker_dispatch() {
        let advance = NuxFlowAdvanceOperation {
            struct_size: size_u32::<NuxFlowAdvanceOperation>(),
            timestamp_seconds: 1.0,
            delta_seconds: 0.0,
            render: 2,
            apple_drawable: ptr::null_mut(),
            completion_context: ptr::null_mut(),
            completion_callback: None,
        };
        let mut request = operation(NUX_FLOW_SESSION_OPERATION_KIND_ADVANCE);
        request.advance = &advance;
        let mut result = ptr::null_mut();
        assert_eq!(
            unsafe { nux_flow_render_session_perform(ptr::null(), &request, &mut result) },
            NuxStatus::InvalidArgument
        );
        unsafe { nux_flow_session_result_free(result) };
    }

    #[test]
    fn result_views_borrow_owned_storage_until_explicit_free() {
        let mut result = FlowSessionResultHandle::empty_success();
        result.player_metadata = Some(OwnedPlayerMetadata {
            kind: NUX_FLOW_PLAYER_KIND_STATIC,
            selection: NUX_FLOW_PLAYER_SELECTION_STATIC,
            player_index: None,
            artboard_name: b"Owned Artboard".to_vec(),
            player_name: Vec::new(),
            min_x: -10.0,
            min_y: 4.0,
            max_x: 90.0,
            max_y: 54.0,
        });
        result.value_arena.nodes.push(OwnedValueNode {
            kind: NUX_FLOW_VALUE_KIND_BOOL,
            number_value: 0.0,
            color_value: 0,
            bool_value: true,
            first_edge: 0,
            edge_count: 0,
            instance_id: None,
            identity_value: 0,
            string_value: Vec::new(),
            schema_id: Vec::new(),
        });
        result.player_inputs.push(OwnedPlayerInput {
            kind: NUX_FLOW_PLAYER_INPUT_KIND_BOOL,
            value_root_index: 0,
            name: b"enabled".to_vec(),
        });
        let result = Box::into_raw(Box::new(result)).cast::<NuxFlowSessionResult>();
        let mut metadata: NuxFlowPlayerMetadataView = unsafe { std::mem::zeroed() };
        metadata.struct_size = size_u32::<NuxFlowPlayerMetadataView>();
        assert_eq!(
            unsafe { nux_flow_session_result_player_metadata(result, &mut metadata) },
            NuxStatus::Ok
        );
        let copied = unsafe {
            slice::from_raw_parts(
                metadata.artboard_name.data,
                metadata.artboard_name.len as usize,
            )
        }
        .to_vec();
        assert_eq!(copied, b"Owned Artboard");
        assert_eq!(metadata.min_x, -10.0);
        assert_eq!(metadata.selection, NUX_FLOW_PLAYER_SELECTION_STATIC);
        assert_eq!(metadata.player_index, u32::MAX);
        let mut input: NuxFlowPlayerInputView = unsafe { std::mem::zeroed() };
        input.struct_size = size_u32::<NuxFlowPlayerInputView>();
        assert_eq!(
            unsafe { nux_flow_session_result_player_input_at(result, 0, &mut input) },
            NuxStatus::Ok
        );
        assert_eq!(input.kind, NUX_FLOW_PLAYER_INPUT_KIND_BOOL);
        let _noise = vec![0xA5_u8; 64 * 1024];
        let borrowed = unsafe {
            slice::from_raw_parts(
                metadata.artboard_name.data,
                metadata.artboard_name.len as usize,
            )
        };
        assert_eq!(borrowed, b"Owned Artboard");
        let input_name = unsafe { slice::from_raw_parts(input.name.data, input.name.len as usize) };
        assert_eq!(input_name, b"enabled");
        unsafe { nux_flow_session_result_free(result) };
    }

    #[test]
    fn result_rejects_output_sequence_or_phase_regression_but_allows_new_cycles() {
        let mut result = FlowSessionResultHandle::empty_success();
        let output = |sequence, cycle, phase| OwnedOutput {
            phase,
            kind: NUX_FLOW_OUTPUT_KIND_RUNTIME_ADVANCED,
            payload_root_index: None,
            sequence,
            cycle,
            origin_mutation_id: None,
            instance_id: None,
            event_type: 0,
            first_event_property: 0,
            event_property_count: 0,
            delay_seconds: 0.0,
            name: Vec::new(),
            path: Vec::new(),
            payload: Vec::new(),
        };
        result.outputs = vec![
            output(1, 1, NUX_FLOW_OUTPUT_PHASE_HOST_WORK),
            output(2, 2, NUX_FLOW_OUTPUT_PHASE_REPORTED_EVENTS),
        ];
        assert_eq!(result.validate(), Ok(()));
        result.outputs[1].sequence = 1;
        assert_eq!(result.validate(), Err(NuxStatus::RuntimeError));
        result.outputs[1].sequence = 2;
        result.outputs[1].cycle = 1;
        assert_eq!(result.validate(), Err(NuxStatus::RuntimeError));
    }

    #[test]
    fn player_metadata_requires_consistent_kind_selection_and_index() {
        let mut result = FlowSessionResultHandle::empty_success();
        result.player_metadata = Some(OwnedPlayerMetadata {
            kind: NUX_FLOW_PLAYER_KIND_STATE_MACHINE,
            selection: NUX_FLOW_PLAYER_SELECTION_EXPLICIT_STATE_MACHINE,
            player_index: Some(0),
            artboard_name: b"screen".to_vec(),
            player_name: b"machine".to_vec(),
            min_x: 0.0,
            min_y: 0.0,
            max_x: 100.0,
            max_y: 100.0,
        });
        assert_eq!(result.validate(), Ok(()));

        let metadata = result
            .player_metadata
            .as_mut()
            .expect("metadata is present");
        metadata.selection = NUX_FLOW_PLAYER_SELECTION_STATIC;
        assert_eq!(result.validate(), Err(NuxStatus::RuntimeError));

        let metadata = result
            .player_metadata
            .as_mut()
            .expect("metadata is present");
        metadata.kind = NUX_FLOW_PLAYER_KIND_STATIC;
        assert_eq!(result.validate(), Err(NuxStatus::RuntimeError));
        result
            .player_metadata
            .as_mut()
            .expect("metadata is present")
            .player_index = None;
        assert_eq!(result.validate(), Ok(()));
    }

    #[cfg(feature = "apple-product")]
    #[test]
    fn configured_create_bootstrap_and_query_round_trip_use_the_real_flow_session() {
        const FIXTURE: &[u8] = include_bytes!("../../../fixtures/animation/smi_test.riv");
        let worker = match RuntimeWorker::spawn(FIXTURE.to_vec()) {
            Ok(worker) => worker,
            Err(_) => panic!("import fixture"),
        };
        let context = Box::into_raw(Box::new(FlowRuntimeContextHandle { worker }))
            .cast::<NuxFlowRuntimeContext>();
        let mut descriptor = configured_descriptor();
        descriptor.artboard_name = bytes(b"artboard to nest");
        descriptor.player_name = bytes(b"State Machine 1");
        let mut session = ptr::null_mut();
        let mut create_result = ptr::null_mut();
        assert_eq!(
            unsafe {
                nux_flow_render_session_create_configured(
                    context,
                    &descriptor,
                    &mut session,
                    &mut create_result,
                )
            },
            NuxStatus::Ok
        );
        assert!(!session.is_null());
        assert_eq!(
            unsafe { nux_flow_session_result_status(create_result) },
            NuxStatus::Ok
        );
        let mut metadata: NuxFlowPlayerMetadataView = unsafe { std::mem::zeroed() };
        metadata.struct_size = size_u32::<NuxFlowPlayerMetadataView>();
        assert_eq!(
            unsafe { nux_flow_session_result_player_metadata(create_result, &mut metadata) },
            NuxStatus::Ok
        );
        assert_eq!(metadata.kind, NUX_FLOW_PLAYER_KIND_STATE_MACHINE);
        assert_eq!(
            metadata.selection,
            NUX_FLOW_PLAYER_SELECTION_EXPLICIT_STATE_MACHINE
        );
        assert_ne!(metadata.player_index, u32::MAX);
        assert_eq!(
            unsafe { nux_flow_session_result_output_count(create_result) },
            0
        );
        unsafe { nux_flow_session_result_free(create_result) };

        let queries = [
            NuxFlowQuery {
                struct_size: size_u32::<NuxFlowQuery>(),
                kind: NUX_FLOW_QUERY_KIND_PLAYER_INPUTS,
            },
            NuxFlowQuery {
                struct_size: size_u32::<NuxFlowQuery>(),
                kind: NUX_FLOW_QUERY_KIND_BOOTSTRAP,
            },
        ];
        let query_batch = NuxFlowQueryBatch {
            struct_size: size_u32::<NuxFlowQueryBatch>(),
            queries: queries.as_ptr(),
            query_count: queries.len() as u64,
        };
        let mut request = operation(NUX_FLOW_SESSION_OPERATION_KIND_QUERY);
        request.query_batch = &query_batch;
        let mut query_result = ptr::null_mut();
        assert_eq!(
            unsafe { nux_flow_render_session_perform(session, &request, &mut query_result) },
            NuxStatus::Ok
        );
        assert_eq!(
            unsafe { nux_flow_session_result_status(query_result) },
            NuxStatus::Ok
        );
        metadata.struct_size = size_u32::<NuxFlowPlayerMetadataView>();
        assert_eq!(
            unsafe { nux_flow_session_result_player_metadata(query_result, &mut metadata) },
            NuxStatus::Ok
        );
        assert_eq!(metadata.kind, NUX_FLOW_PLAYER_KIND_STATE_MACHINE);
        assert!(unsafe { nux_flow_session_result_player_input_count(query_result) } >= 3);
        unsafe { nux_flow_session_result_free(query_result) };

        let query_player_inputs = || {
            let query = NuxFlowQuery {
                struct_size: size_u32::<NuxFlowQuery>(),
                kind: NUX_FLOW_QUERY_KIND_PLAYER_INPUTS,
            };
            let query_batch = NuxFlowQueryBatch {
                struct_size: size_u32::<NuxFlowQueryBatch>(),
                queries: &query,
                query_count: 1,
            };
            let mut request = operation(NUX_FLOW_SESSION_OPERATION_KIND_QUERY);
            request.query_batch = &query_batch;
            let mut result = ptr::null_mut();
            assert_eq!(
                unsafe { nux_flow_render_session_perform(session, &request, &mut result) },
                NuxStatus::Ok
            );
            result
        };
        let bool_input_value = |result: *mut NuxFlowSessionResult| {
            let input_count = unsafe { nux_flow_session_result_player_input_count(result) };
            assert!(input_count >= 3);
            for index in 0..input_count {
                let mut input: NuxFlowPlayerInputView = unsafe { std::mem::zeroed() };
                input.struct_size = size_u32::<NuxFlowPlayerInputView>();
                assert_eq!(
                    unsafe { nux_flow_session_result_player_input_at(result, index, &mut input) },
                    NuxStatus::Ok
                );
                let name =
                    unsafe { slice::from_raw_parts(input.name.data, input.name.len as usize) };
                if name == b"bool" {
                    assert_eq!(input.kind, NUX_FLOW_PLAYER_INPUT_KIND_BOOL);
                    let mut node: NuxFlowValueNode = unsafe { std::mem::zeroed() };
                    node.struct_size = size_u32::<NuxFlowValueNode>();
                    assert_eq!(
                        unsafe {
                            nux_flow_session_result_value_node_at(
                                result,
                                u64::from(input.value_root_index),
                                &mut node,
                            )
                        },
                        NuxStatus::Ok
                    );
                    assert_eq!(node.kind, NUX_FLOW_VALUE_KIND_BOOL);
                    return node.bool_value == 1;
                }
            }
            panic!("fixture bool input is missing")
        };

        let input_result = query_player_inputs();
        assert!(!bool_input_value(input_result));
        unsafe { nux_flow_session_result_free(input_result) };

        let mut bool_value = null_node(NUX_FLOW_VALUE_KIND_BOOL);
        bool_value.bool_value = 1;
        let value_arena = NuxFlowValueArena {
            struct_size: size_u32::<NuxFlowValueArena>(),
            nodes: &bool_value,
            node_count: 1,
            edges: ptr::null(),
            edge_count: 0,
        };
        let zero_reference = NuxFlowInstanceReference {
            kind: 0,
            local_id: 0,
            instance_id: 0,
        };
        let input_mutation = NuxFlowStateMutation {
            struct_size: size_u32::<NuxFlowStateMutation>(),
            kind: NUX_FLOW_STATE_MUTATION_KIND_SET_INPUT_BOOL,
            instance: zero_reference,
            item: zero_reference,
            path: NuxByteView::default(),
            input_name: bytes(b"bool"),
            value_root_index: 0,
            index: 0,
            other_index: 0,
        };
        let state_batch = NuxFlowStateBatch {
            struct_size: size_u32::<NuxFlowStateBatch>(),
            has_host_mutation_id: 1,
            host_mutation_id: 0,
            value_arena: &value_arena,
            new_instances: ptr::null(),
            new_instance_count: 0,
            mutations: &input_mutation,
            mutation_count: 1,
        };
        let mut request = operation(NUX_FLOW_SESSION_OPERATION_KIND_STATE_BATCH);
        request.state_batch = &state_batch;
        let mut state_result = ptr::null_mut();
        assert_eq!(
            unsafe { nux_flow_render_session_perform(session, &request, &mut state_result) },
            NuxStatus::Ok
        );
        assert_eq!(
            unsafe { nux_flow_session_result_output_count(state_result) },
            1
        );
        let mut output: NuxFlowOutputView = unsafe { std::mem::zeroed() };
        output.struct_size = size_u32::<NuxFlowOutputView>();
        assert_eq!(
            unsafe { nux_flow_session_result_output_at(state_result, 0, &mut output) },
            NuxStatus::Ok
        );
        assert_eq!(output.kind, NUX_FLOW_OUTPUT_KIND_STATE_CHANGE);
        assert_eq!(output.has_origin_mutation_id, 1);
        assert_eq!(output.origin_mutation_id, 0);
        unsafe { nux_flow_session_result_free(state_result) };

        let input_result = query_player_inputs();
        assert!(bool_input_value(input_result));
        unsafe { nux_flow_session_result_free(input_result) };

        let advance = NuxFlowAdvanceOperation {
            struct_size: size_u32::<NuxFlowAdvanceOperation>(),
            timestamp_seconds: 0.0,
            delta_seconds: 0.0,
            render: 0,
            apple_drawable: ptr::null_mut(),
            completion_context: ptr::null_mut(),
            completion_callback: None,
        };
        let mut request = operation(NUX_FLOW_SESSION_OPERATION_KIND_ADVANCE);
        request.advance = &advance;
        let mut advance_result = ptr::null_mut();
        assert_eq!(
            unsafe { nux_flow_render_session_perform(session, &request, &mut advance_result) },
            NuxStatus::Ok
        );
        assert_eq!(
            unsafe { nux_flow_session_result_status(advance_result) },
            NuxStatus::Ok
        );
        assert!(unsafe { nux_flow_session_result_output_count(advance_result) } >= 1);
        unsafe { nux_flow_session_result_free(advance_result) };

        let pointer = NuxFlowPointerEvent {
            struct_size: size_u32::<NuxFlowPointerEvent>(),
            kind: NUX_FLOW_POINTER_EVENT_KIND_DOWN,
            pointer_id: 1,
            x: 0.0,
            y: 0.0,
        };
        let pointer_batch = NuxFlowPointerBatch {
            struct_size: size_u32::<NuxFlowPointerBatch>(),
            events: &pointer,
            event_count: 1,
        };
        let mut request = operation(NUX_FLOW_SESSION_OPERATION_KIND_POINTER_BATCH);
        request.pointer_batch = &pointer_batch;
        let mut pointer_result = ptr::null_mut();
        assert_eq!(
            unsafe { nux_flow_render_session_perform(session, &request, &mut pointer_result) },
            NuxStatus::Ok
        );
        assert_eq!(
            unsafe { nux_flow_session_result_status(pointer_result) },
            NuxStatus::Ok
        );
        unsafe { nux_flow_session_result_free(pointer_result) };

        let legacy_advance = NuxFrameOperation {
            struct_size: size_u32::<NuxFrameOperation>(),
            elapsed_seconds: 0.25,
            render: false,
            apple_drawable: ptr::null_mut(),
            completion_context: ptr::null_mut(),
            completion_callback: None,
        };
        let mut legacy_result = ptr::null_mut();
        assert_eq!(
            unsafe {
                nux_flow_render_session_advance(session, &legacy_advance, &mut legacy_result)
            },
            NuxStatus::Ok
        );
        unsafe { nux_operation_result_free(legacy_result) };

        let mixed_advance = NuxFlowAdvanceOperation {
            timestamp_seconds: 0.5,
            delta_seconds: 0.25,
            ..advance
        };
        let mut request = operation(NUX_FLOW_SESSION_OPERATION_KIND_ADVANCE);
        request.advance = &mixed_advance;
        let mut mixed_result = ptr::null_mut();
        assert_eq!(
            unsafe { nux_flow_render_session_perform(session, &request, &mut mixed_result) },
            NuxStatus::Ok
        );
        unsafe { nux_flow_session_result_free(mixed_result) };

        legacy_result = ptr::null_mut();
        assert_eq!(
            unsafe {
                nux_flow_render_session_advance(session, &legacy_advance, &mut legacy_result)
            },
            NuxStatus::Ok
        );
        unsafe { nux_operation_result_free(legacy_result) };

        unsafe {
            nux_flow_render_session_free(session);
            nux_flow_runtime_context_free(context);
        }
    }

    #[test]
    fn every_v12_export_has_a_panic_firewall() {
        let source = include_str!("session_v12.rs");
        let mut checked = 0usize;
        for prefix in ["pub unsafe extern \"C\" fn ", "pub extern \"C\" fn "] {
            for (index, _) in source.match_indices(prefix) {
                let rest = &source[index..];
                let body_start = rest.find('{').expect("extern function body");
                let body = &rest[body_start + 1..];
                let first_statement = body.trim_start();
                assert!(
                    first_statement.starts_with("ffi_guard(")
                        || first_statement.starts_with("ffi_guard_with_session_result("),
                    "v1.2 export is missing its panic firewall: {}",
                    rest.lines().next().unwrap_or_default()
                );
                checked += 1;
            }
        }
        assert!(checked >= 20);
    }
}
