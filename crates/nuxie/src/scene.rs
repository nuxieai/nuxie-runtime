//! Dynamic authoring facade backed by the same runtime file and graph used by imported scenes.

use std::{
    collections::{BTreeMap, BTreeSet},
    marker::PhantomData,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

use nuxie_binary::{AuthoringProperty, AuthoringRecord, AuthoringValue, RuntimeFile};
use nuxie_render_api::{Factory, ImageDecodeError, Renderer};
use nuxie_runtime::{
    ArtboardInstance as RuntimeArtboardInstance, StateMachineInputKind, StateMachineInstance,
    embedded_font_is_parseable,
};

use crate::{ArtboardRenderCache, File, OwnedArtboardInstance};

/// Stable identity of an authored artboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ArtboardId(u64);

/// Stable identity of an authored object.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ObjectId(u64);

/// Stable typed identity of an authored linear animation.
///
/// Animation records share the ordinary [`ObjectId`] identity space. This
/// newtype is vocabulary only; it does not introduce a second allocator or a
/// second record mechanism.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AnimationId(ObjectId);

impl AnimationId {
    pub const fn object_id(self) -> ObjectId {
        self.0
    }
}

impl From<AnimationId> for ObjectId {
    fn from(animation: AnimationId) -> Self {
        animation.object_id()
    }
}

macro_rules! ordinary_record_id {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(ObjectId);

        impl $name {
            pub const fn object_id(self) -> ObjectId {
                self.0
            }
        }

        impl From<$name> for ObjectId {
            fn from(id: $name) -> Self {
                id.object_id()
            }
        }
    };
}

ordinary_record_id!(EventId);
ordinary_record_id!(MachineId);
ordinary_record_id!(MachineInputId);
ordinary_record_id!(MachineLayerId);
ordinary_record_id!(MachineStateId);
ordinary_record_id!(MachineTransitionId);

/// Stable identity of an embedded font owned by the authored scene.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FontAssetId(u64);

/// Stable identity of an embedded image owned by the authored scene.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ImageAssetId(u64);

/// Stable identity of compiled Luau bytecode owned by the authored scene.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ScriptAssetId(u64);

/// Stable identity of a compiled Rive shader table owned by the authored scene.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ShaderAssetId(u64);

/// Stable identity of a live artboard instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct InstanceId(u64);

/// Generation of the structurally materialized scene.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StructureEpoch(u64);

impl StructureEpoch {
    pub const INITIAL: Self = Self(0);

    pub const fn get(self) -> u64 {
        self.0
    }

    fn next(self) -> Option<Self> {
        self.0.checked_add(1).map(Self)
    }
}

/// Parent selected when creating an authored object.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Parent {
    Artboard(ArtboardId),
    Object(ObjectId),
}

/// Final position among the direct children of one authored parent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChildIndex {
    First,
    Last,
    At(usize),
}

/// Stable identities involved in a failed edit operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditId {
    Artboard(ArtboardId),
    Object(ObjectId),
    FontAsset(FontAssetId),
    ImageAsset(ImageAssetId),
    ScriptAsset(ScriptAssetId),
    ShaderAsset(ShaderAssetId),
    Instance(InstanceId),
}

/// Machine-readable reason for rejecting an edit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditReason {
    Requested {
        message: String,
    },
    IdentityExhausted,
    IdentityCollision,
    OperationLimitExceeded,
    UnknownArtboard,
    UnknownObject,
    NonVisualObject,
    UnknownFontAsset,
    UnknownImageAsset,
    UnknownScriptAsset,
    EmptyFontAsset,
    InvalidFontAsset,
    CycleDetected,
    ChildIndexOutOfRange,
    ChildSetMismatch,
    InvalidParent {
        parent: Option<NodeKind>,
        child: NodeKind,
    },
    InvalidReference {
        expected: NodeKind,
        actual: Option<NodeKind>,
    },
    CrossArtboardReference {
        source: ArtboardId,
        target: ArtboardId,
    },
    InvalidMachineReference,
    InvalidMachineTopology {
        requirement: &'static str,
        actual: usize,
    },
    EmptyMachineInputName,
    DuplicateMachineInputName,
    PropertyOwnerMismatch {
        property: &'static str,
        actual: NodeKind,
    },
    RecordPropertyOwnerMismatch {
        property: &'static str,
        actual: AuthoredObjectKind,
    },
    NonFiniteProperty {
        property: &'static str,
    },
    NonPositiveProperty {
        property: &'static str,
    },
    EmptyScene,
    CapacityExceeded,
    RuntimeRejected,
    EpochExhausted,
    InternalInvariant,
}

/// Durable authored object path reported by nested hit testing.
///
/// Direct hits contain one object. Hits inside nested artboards are prefixed
/// with each authored `NestedArtboard` host on the descent path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SceneObjectPath {
    objects: Vec<ObjectId>,
}

impl SceneObjectPath {
    pub fn objects(&self) -> &[ObjectId] {
        &self.objects
    }

    pub fn into_objects(self) -> Vec<ObjectId> {
        self.objects
    }
}

/// Exact transaction operation and identities responsible for an edit failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditDiagnostic {
    pub operation_index: usize,
    pub involved_ids: Vec<EditId>,
    pub reason: EditReason,
}

impl EditDiagnostic {
    fn new(operation_index: usize, involved_ids: Vec<EditId>, reason: EditReason) -> Self {
        Self {
            operation_index,
            involved_ids,
            reason,
        }
    }
}

/// Typed transaction-local rejection returned by [`SceneTx`] operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditAbort {
    diagnostic: EditDiagnostic,
}

impl EditAbort {
    fn new(operation_index: usize, involved_ids: Vec<EditId>, reason: EditReason) -> Self {
        Self {
            diagnostic: EditDiagnostic::new(operation_index, involved_ids, reason),
        }
    }

    pub fn diagnostic(&self) -> &EditDiagnostic {
        &self.diagnostic
    }
}

impl std::fmt::Display for EditAbort {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("scene edit transaction was aborted")
    }
}

impl std::error::Error for EditAbort {}

/// Phase in which a public [`Scene::edit`] call failed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditErrorKind {
    Aborted,
    CommitRejected,
}

/// Public edit failure. Runtime and materialization implementation errors remain private.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditError {
    kind: EditErrorKind,
    diagnostic: EditDiagnostic,
}

impl EditError {
    fn aborted(abort: EditAbort) -> Self {
        Self {
            kind: EditErrorKind::Aborted,
            diagnostic: abort.diagnostic,
        }
    }

    fn commit(diagnostic: EditDiagnostic) -> Self {
        Self {
            kind: EditErrorKind::CommitRejected,
            diagnostic,
        }
    }

    pub const fn kind(&self) -> EditErrorKind {
        self.kind
    }

    pub fn diagnostic(&self) -> &EditDiagnostic {
        &self.diagnostic
    }
}

impl std::fmt::Display for EditError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            EditErrorKind::Aborted => formatter.write_str("scene edit transaction was aborted"),
            EditErrorKind::CommitRejected => {
                formatter.write_str("scene edit was rejected during commit")
            }
        }
    }
}

impl std::error::Error for EditError {}

/// A schema-generated typed property token.
pub struct Prop<T> {
    key: u16,
    schema_name: &'static str,
    value_kind: PropValueKind,
    declared_owner: &'static str,
    is_available_on: fn(NodeKind) -> bool,
    apply_to_definition: fn(&mut RecordSpec, T) -> std::result::Result<(), EditReason>,
    apply_to_runtime: fn(&mut RuntimeArtboardInstance, usize, u16, T) -> bool,
    read_from_runtime: fn(&RuntimeArtboardInstance, usize, u16) -> Option<T>,
    marker: PhantomData<fn(T)>,
}

/// Value kinds accepted by generated runtime property tokens.
pub trait PropValue {
    fn is_valid(&self) -> bool;
}

impl PropValue for f32 {
    fn is_valid(&self) -> bool {
        self.is_finite()
    }
}

impl PropValue for u32 {
    fn is_valid(&self) -> bool {
        true
    }
}

impl<T> Prop<T> {
    pub const fn schema_name(self) -> &'static str {
        self.schema_name
    }

    pub const fn value_kind(self) -> PropValueKind {
        self.value_kind
    }

    pub const fn declared_owner(self) -> &'static str {
        self.declared_owner
    }

    /// Whether this property token is available on the current generated
    /// authoring [`NodeSpec`] surface for `kind`.
    ///
    /// This intentionally describes the first-slice authoring vocabulary,
    /// not every property inherited by the corresponding runtime schema type.
    pub fn is_available_on(self, kind: NodeKind) -> bool {
        (self.is_available_on)(kind)
    }
}

impl<T> Copy for Prop<T> {}

impl<T> Clone for Prop<T> {
    fn clone(&self) -> Self {
        *self
    }
}

fn set_runtime_double(
    instance: &mut RuntimeArtboardInstance,
    local_id: usize,
    key: u16,
    value: f32,
) -> bool {
    instance.set_double_property(local_id, key, value)
}

fn set_runtime_color(
    instance: &mut RuntimeArtboardInstance,
    local_id: usize,
    key: u16,
    value: u32,
) -> bool {
    instance.set_color_property(local_id, key, value)
}

fn read_runtime_double(
    instance: &RuntimeArtboardInstance,
    local_id: usize,
    key: u16,
) -> Option<f32> {
    instance.double_property(local_id, key)
}

fn read_runtime_color(
    instance: &RuntimeArtboardInstance,
    local_id: usize,
    key: u16,
) -> Option<u32> {
    instance.color_property(local_id, key)
}

include!(concat!(env!("OUT_DIR"), "/scene_schema.rs"));

/// A direct runtime target. It remains valid only for the scene epoch in which it was resolved.
pub struct Cursor<T> {
    scene: SceneId,
    epoch: StructureEpoch,
    instance_slot: usize,
    instance: InstanceId,
    local_id: usize,
    property: Prop<T>,
}

impl<T> Copy for Cursor<T> {}

impl<T> Clone for Cursor<T> {
    fn clone(&self) -> Self {
        *self
    }
}

/// Pre-resolved trigger input on one retained state-machine instance.
///
/// Like property cursors, this handle is lifetime-free and fenced by scene,
/// structure epoch, instance identity, machine identity, and runtime input kind.
#[derive(Debug, Clone, Copy)]
pub struct InputCursor {
    scene: SceneId,
    epoch: StructureEpoch,
    instance_slot: usize,
    instance: InstanceId,
    machine: MachineId,
    machine_index: usize,
    input_index: usize,
    input_kind: StateMachineInputKind,
}

/// Returned when a cursor predates a structural scene edit or its instance no longer exists.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StaleCursor;

impl std::fmt::Display for StaleCursor {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("scene cursor is stale")
    }
}

impl std::error::Error for StaleCursor {}

/// Failure to create a live instance from an authored artboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstanceError {
    UnknownArtboard,
    IdentityExhausted,
    RuntimeRejected,
}

impl std::fmt::Display for InstanceError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::UnknownArtboard => "unknown authored artboard",
            Self::IdentityExhausted => "scene instance identity exhausted",
            Self::RuntimeRejected => "runtime rejected authored artboard",
        })
    }
}

impl std::error::Error for InstanceError {}

/// Failure to resolve an authored identity to a direct live runtime target.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolveError {
    UnknownInstance,
    UnknownObject,
    NonVisualObject,
    DifferentArtboard,
    UnsupportedProperty,
    UnknownMachine,
    UnknownMachineInput,
    UnsupportedInputKind,
}

impl std::fmt::Display for ResolveError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::UnknownInstance => "unknown scene instance",
            Self::UnknownObject => "unknown authored object",
            Self::NonVisualObject => "authored object is not a visual runtime target",
            Self::DifferentArtboard => "authored object belongs to a different artboard",
            Self::UnsupportedProperty => "property is not valid for the authored object type",
            Self::UnknownMachine => "unknown authored state machine",
            Self::UnknownMachineInput => "unknown state-machine input",
            Self::UnsupportedInputKind => "state-machine input is not a trigger",
        })
    }
}

impl std::error::Error for ResolveError {}

/// Failure while advancing an authored instance with a renderer factory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdvanceError {
    UnknownInstance,
    RuntimeRejected,
}

impl std::fmt::Display for AdvanceError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::UnknownInstance => "unknown scene instance",
            Self::RuntimeRejected => "runtime rejected authored scene advance",
        })
    }
}

impl std::error::Error for AdvanceError {}

/// Failure while drawing an authored instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawError {
    UnknownInstance,
    ImageDecode,
    RuntimeRejected,
}

impl std::fmt::Display for DrawError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::UnknownInstance => "unknown scene instance",
            Self::ImageDecode => "failed to decode authored scene image",
            Self::RuntimeRejected => "runtime rejected authored scene draw",
        })
    }
}

impl std::error::Error for DrawError {}

/// Summary of one committed structural edit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditReceipt {
    pub epoch: StructureEpoch,
    pub created: Vec<ObjectId>,
}

#[derive(Debug, Clone, Default)]
struct Definitions {
    font_assets: Vec<FontAssetDefinition>,
    image_assets: Vec<ImageAssetDefinition>,
    script_assets: Vec<ScriptAssetDefinition>,
    shader_assets: Vec<ShaderAssetDefinition>,
    artboards: Vec<ArtboardDefinition>,
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct SceneWork {
    definition_index_builds: usize,
    definition_index_node_visits: usize,
    receipt_membership_checks: usize,
}

#[cfg(test)]
thread_local! {
    static SCENE_WORK: std::cell::Cell<SceneWork> = const {
        std::cell::Cell::new(SceneWork {
            definition_index_builds: 0,
            definition_index_node_visits: 0,
            receipt_membership_checks: 0,
        })
    };
}

#[cfg(test)]
fn reset_scene_work() {
    SCENE_WORK.set(SceneWork::default());
}

#[cfg(test)]
fn scene_work() -> SceneWork {
    SCENE_WORK.get()
}

#[cfg(test)]
fn record_scene_work(update: impl FnOnce(&mut SceneWork)) {
    SCENE_WORK.with(|cell| {
        let mut work = cell.get();
        update(&mut work);
        cell.set(work);
    });
}

impl Definitions {
    fn canonicalize_and_validate(
        &mut self,
        operation_index: usize,
    ) -> std::result::Result<(), EditAbort> {
        Hierarchy::canonicalize_and_validate(self, operation_index)
    }
}

#[derive(Debug, Clone, Copy)]
struct IndexedObject {
    artboard: ArtboardId,
    artboard_index: usize,
    record_index: usize,
    kind: AuthoredObjectKind,
}

impl IndexedObject {
    const fn visual_kind(self) -> Option<NodeKind> {
        match self.kind {
            AuthoredObjectKind::Visual(kind) => Some(kind),
            _ => None,
        }
    }
}

/// Transaction-local identity and parent lookup. Structural operations may do
/// deep candidate work, but the high-volume create path and receipt filtering
/// never rescan the growing authored graph.
#[derive(Default)]
struct DefinitionIndex {
    font_assets: BTreeMap<FontAssetId, usize>,
    image_assets: BTreeMap<ImageAssetId, usize>,
    script_assets: BTreeMap<ScriptAssetId, usize>,
    shader_assets: BTreeMap<ShaderAssetId, usize>,
    artboards: BTreeMap<ArtboardId, usize>,
    objects: BTreeMap<ObjectId, IndexedObject>,
    children: BTreeMap<Parent, Vec<ObjectId>>,
    owned: BTreeMap<ObjectId, Vec<ObjectId>>,
    keyed_objects: BTreeMap<(ObjectId, ObjectId), ObjectId>,
    keyed_properties: BTreeMap<(ObjectId, ExportedAnimatableProperty), ObjectId>,
    key_frames: BTreeMap<(ObjectId, u32), ObjectId>,
}

impl DefinitionIndex {
    fn build(definitions: &Definitions) -> Self {
        #[cfg(test)]
        record_scene_work(|work| {
            work.definition_index_builds = work.definition_index_builds.saturating_add(1);
            work.definition_index_node_visits = definitions
                .artboards
                .iter()
                .fold(work.definition_index_node_visits, |visits, artboard| {
                    visits.saturating_add(artboard.records.len())
                });
        });
        let mut index = Self::default();
        for (font_index, font) in definitions.font_assets.iter().enumerate() {
            index.font_assets.insert(font.id, font_index);
        }
        for (image_index, image) in definitions.image_assets.iter().enumerate() {
            index.image_assets.insert(image.id, image_index);
        }
        for (script_index, script) in definitions.script_assets.iter().enumerate() {
            index.script_assets.insert(script.id, script_index);
        }
        for (shader_index, shader) in definitions.shader_assets.iter().enumerate() {
            index.shader_assets.insert(shader.id, shader_index);
        }
        for (artboard_index, artboard) in definitions.artboards.iter().enumerate() {
            index.artboards.insert(artboard.id, artboard_index);
            index
                .children
                .entry(Parent::Artboard(artboard.id))
                .or_default();
            for (record_index, record) in artboard.records.iter().enumerate() {
                index.objects.insert(
                    record.id,
                    IndexedObject {
                        artboard: artboard.id,
                        artboard_index,
                        record_index,
                        kind: record.spec.kind(),
                    },
                );
                index.owned.entry(record.id).or_default();
                if let Some(owner) = record.spec.owner() {
                    index.owned.entry(owner).or_default().push(record.id);
                }
                match &record.spec {
                    RecordSpec::Visual { parent, .. } => {
                        index.children.entry(*parent).or_default().push(record.id);
                        index.children.entry(Parent::Object(record.id)).or_default();
                    }
                    RecordSpec::Animation(AnimationRecordSpec::KeyedObject {
                        animation,
                        target,
                    }) => {
                        index.keyed_objects.insert((*animation, *target), record.id);
                    }
                    RecordSpec::Animation(AnimationRecordSpec::KeyedProperty {
                        keyed_object,
                        property,
                    }) => {
                        index
                            .keyed_properties
                            .insert((*keyed_object, *property), record.id);
                    }
                    RecordSpec::Animation(AnimationRecordSpec::KeyFrameDouble {
                        keyed_property,
                        frame,
                        ..
                    }) => {
                        index
                            .key_frames
                            .insert((*keyed_property, *frame), record.id);
                    }
                    RecordSpec::Animation(AnimationRecordSpec::LinearAnimation(_))
                    | RecordSpec::Machine(_) => {}
                }
            }
        }
        index
    }

    fn contains_object(&self, object: ObjectId) -> bool {
        self.objects.contains_key(&object)
    }

    fn rebuild(&mut self, definitions: &Definitions) {
        *self = Self::build(definitions);
    }

    fn validate_parent(
        &self,
        operation_index: usize,
        parent: Parent,
        child: NodeKind,
    ) -> std::result::Result<ArtboardId, EditAbort> {
        match parent {
            Parent::Artboard(artboard) => {
                if !valid_artboard_child(child) {
                    return Err(EditAbort::new(
                        operation_index,
                        vec![EditId::Artboard(artboard)],
                        EditReason::InvalidParent {
                            parent: None,
                            child,
                        },
                    ));
                }
                self.artboards
                    .contains_key(&artboard)
                    .then_some(artboard)
                    .ok_or_else(|| {
                        EditAbort::new(
                            operation_index,
                            vec![EditId::Artboard(artboard)],
                            EditReason::UnknownArtboard,
                        )
                    })
            }
            Parent::Object(object) => {
                let Some(parent) = self.objects.get(&object).copied() else {
                    return Err(EditAbort::new(
                        operation_index,
                        vec![EditId::Object(object)],
                        EditReason::UnknownObject,
                    ));
                };
                let Some(parent_kind) = parent.visual_kind() else {
                    return Err(EditAbort::new(
                        operation_index,
                        vec![EditId::Object(object)],
                        EditReason::NonVisualObject,
                    ));
                };
                if !valid_object_parent(parent_kind, child) {
                    return Err(EditAbort::new(
                        operation_index,
                        vec![EditId::Object(object)],
                        EditReason::InvalidParent {
                            parent: Some(parent_kind),
                            child,
                        },
                    ));
                }
                Ok(parent.artboard)
            }
        }
    }
}

#[derive(Default)]
struct SpecOrigins {
    font_assets: BTreeMap<FontAssetId, usize>,
    image_assets: BTreeMap<ImageAssetId, usize>,
    script_assets: BTreeMap<ScriptAssetId, usize>,
    shader_assets: BTreeMap<ShaderAssetId, usize>,
    artboard_specs: BTreeMap<ArtboardId, usize>,
    nodes: BTreeMap<ObjectId, usize>,
    properties: BTreeMap<(ObjectId, &'static str), usize>,
    relationships: BTreeMap<ObjectId, usize>,
}

impl SpecOrigins {
    fn font_asset(&self, id: FontAssetId, fallback: usize) -> usize {
        self.font_assets.get(&id).copied().unwrap_or(fallback)
    }

    fn image_asset(&self, id: ImageAssetId, fallback: usize) -> usize {
        self.image_assets.get(&id).copied().unwrap_or(fallback)
    }

    fn script_asset(&self, id: ScriptAssetId, fallback: usize) -> usize {
        self.script_assets.get(&id).copied().unwrap_or(fallback)
    }

    fn shader_asset(&self, id: ShaderAssetId, fallback: usize) -> usize {
        self.shader_assets.get(&id).copied().unwrap_or(fallback)
    }

    fn artboard(&self, id: ArtboardId, fallback: usize) -> usize {
        self.artboard_specs.get(&id).copied().unwrap_or(fallback)
    }

    fn object(&self, id: ObjectId, fallback: usize) -> usize {
        self.nodes.get(&id).copied().unwrap_or(fallback)
    }

    fn property(&self, id: ObjectId, property: &'static str, fallback: usize) -> usize {
        self.properties
            .get(&(id, property))
            .copied()
            .unwrap_or_else(|| self.object(id, fallback))
    }

    fn relationship(&self, first: ObjectId, second: ObjectId, fallback: usize) -> usize {
        [first, second]
            .into_iter()
            .flat_map(|id| {
                [
                    self.nodes.get(&id).copied(),
                    self.relationships.get(&id).copied(),
                ]
            })
            .flatten()
            .max()
            .unwrap_or(fallback)
    }
}

#[derive(Debug, Clone)]
struct FontAssetDefinition {
    id: FontAssetId,
    spec: FontAssetSpec,
}

#[derive(Debug, Clone)]
struct ImageAssetDefinition {
    id: ImageAssetId,
    spec: ImageAssetSpec,
}

#[derive(Debug, Clone)]
struct ScriptAssetDefinition {
    id: ScriptAssetId,
    spec: ScriptAssetSpec,
}

#[derive(Debug, Clone)]
struct ShaderAssetDefinition {
    id: ShaderAssetId,
    spec: ShaderAssetSpec,
}

#[derive(Debug, Clone)]
struct ArtboardDefinition {
    id: ArtboardId,
    spec: ArtboardSpec,
    records: Vec<RecordDefinition>,
}

#[derive(Clone, Copy)]
struct VisualRecordRef<'a> {
    id: ObjectId,
    parent: Parent,
    spec: &'a NodeSpec,
}

impl ArtboardDefinition {
    fn visual_records(&self) -> impl Iterator<Item = VisualRecordRef<'_>> {
        self.records.iter().filter_map(|record| {
            record.visual().map(|(parent, spec)| VisualRecordRef {
                id: record.id,
                parent,
                spec,
            })
        })
    }

    fn animation_views(&self) -> impl Iterator<Item = (&RecordDefinition, &AnimationRecordSpec)> {
        self.records
            .iter()
            .filter_map(|record| record.animation().map(|spec| (record, spec)))
    }

    fn machine_views(&self) -> impl Iterator<Item = (&RecordDefinition, &MachineRecordSpec)> {
        self.records
            .iter()
            .filter_map(|record| record.machine().map(|spec| (record, spec)))
    }

    fn visual_record_count(&self) -> usize {
        self.visual_records().count()
    }
}

#[derive(Debug, Clone)]
struct RecordDefinition {
    id: ObjectId,
    spec: RecordSpec,
}

impl RecordDefinition {
    const fn visual(&self) -> Option<(Parent, &NodeSpec)> {
        self.spec.visual()
    }

    const fn animation(&self) -> Option<&AnimationRecordSpec> {
        match &self.spec {
            RecordSpec::Animation(spec) => Some(spec),
            RecordSpec::Visual { .. } | RecordSpec::Machine(_) => None,
        }
    }

    const fn machine(&self) -> Option<&MachineRecordSpec> {
        match &self.spec {
            RecordSpec::Machine(spec) => Some(spec),
            RecordSpec::Visual { .. } | RecordSpec::Animation(_) => None,
        }
    }
}

#[derive(Debug, Clone)]
enum RecordSpec {
    Visual { parent: Parent, node: NodeSpec },
    Animation(AnimationRecordSpec),
    Machine(MachineRecordSpec),
}

/// Semantic kind of any ordinary authored record in the shared ObjectId space.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthoredObjectKind {
    Visual(NodeKind),
    LinearAnimation,
    KeyedObject,
    KeyedProperty,
    KeyFrameDouble,
    Event,
    Machine,
    MachineTrigger,
    MachineLayer,
    AnyState,
    EntryState,
    ExitState,
    AnimationState,
    StateTransition,
    TriggerCondition,
    FireEvent,
}

impl RecordSpec {
    const fn kind(&self) -> AuthoredObjectKind {
        match self {
            Self::Visual { node, .. } => AuthoredObjectKind::Visual(node.kind()),
            Self::Animation(AnimationRecordSpec::LinearAnimation(_)) => {
                AuthoredObjectKind::LinearAnimation
            }
            Self::Animation(AnimationRecordSpec::KeyedObject { .. }) => {
                AuthoredObjectKind::KeyedObject
            }
            Self::Animation(AnimationRecordSpec::KeyedProperty { .. }) => {
                AuthoredObjectKind::KeyedProperty
            }
            Self::Animation(AnimationRecordSpec::KeyFrameDouble { .. }) => {
                AuthoredObjectKind::KeyFrameDouble
            }
            Self::Machine(MachineRecordSpec::Event(_)) => AuthoredObjectKind::Event,
            Self::Machine(MachineRecordSpec::Machine(_)) => AuthoredObjectKind::Machine,
            Self::Machine(MachineRecordSpec::TriggerInput { .. }) => {
                AuthoredObjectKind::MachineTrigger
            }
            Self::Machine(MachineRecordSpec::Layer { .. }) => AuthoredObjectKind::MachineLayer,
            Self::Machine(MachineRecordSpec::AnyState { .. }) => AuthoredObjectKind::AnyState,
            Self::Machine(MachineRecordSpec::EntryState { .. }) => AuthoredObjectKind::EntryState,
            Self::Machine(MachineRecordSpec::ExitState { .. }) => AuthoredObjectKind::ExitState,
            Self::Machine(MachineRecordSpec::AnimationState { .. }) => {
                AuthoredObjectKind::AnimationState
            }
            Self::Machine(MachineRecordSpec::Transition { .. }) => {
                AuthoredObjectKind::StateTransition
            }
            Self::Machine(MachineRecordSpec::TriggerCondition { .. }) => {
                AuthoredObjectKind::TriggerCondition
            }
            Self::Machine(MachineRecordSpec::FireEvent { .. }) => AuthoredObjectKind::FireEvent,
        }
    }

    const fn owner(&self) -> Option<ObjectId> {
        match self {
            Self::Visual {
                parent: Parent::Object(parent),
                ..
            } => Some(*parent),
            Self::Visual {
                parent: Parent::Artboard(_),
                ..
            } => None,
            Self::Animation(spec) => spec.owner(),
            Self::Machine(spec) => spec.owner(),
        }
    }

    const fn visual(&self) -> Option<(Parent, &NodeSpec)> {
        match self {
            Self::Visual { parent, node } => Some((*parent, node)),
            Self::Animation(_) | Self::Machine(_) => None,
        }
    }
}

/// Definition-time shape of one linear timeline. Playback behavior not
/// represented here stays at the Rive schema defaults (speed 1, one-shot,
/// full duration, and no quantization). `fps` must be strictly positive.
#[derive(Debug, Clone, PartialEq)]
pub struct LinearAnimationSpec {
    pub name: String,
    pub fps: u32,
    pub duration: u32,
}

#[derive(Debug, Clone)]
enum AnimationRecordSpec {
    LinearAnimation(LinearAnimationSpec),
    KeyedObject {
        animation: ObjectId,
        target: ObjectId,
    },
    KeyedProperty {
        keyed_object: ObjectId,
        property: ExportedAnimatableProperty,
    },
    KeyFrameDouble {
        keyed_property: ObjectId,
        frame: u32,
        value: f32,
    },
}

impl AnimationRecordSpec {
    const fn owner(&self) -> Option<ObjectId> {
        match self {
            Self::LinearAnimation(_) => None,
            Self::KeyedObject { animation, .. } => Some(*animation),
            Self::KeyedProperty { keyed_object, .. } => Some(*keyed_object),
            Self::KeyFrameDouble { keyed_property, .. } => Some(*keyed_property),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EventSpec {
    pub name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MachineSpec {
    pub name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TriggerInputSpec {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MachineLayerSpec {
    pub name: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnimationStateSpec {
    pub animation: AnimationId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FireEventOccurs {
    AtStart,
    AtEnd,
}

#[derive(Debug, Clone)]
enum MachineRecordSpec {
    Event(EventSpec),
    Machine(MachineSpec),
    TriggerInput {
        machine: ObjectId,
        spec: TriggerInputSpec,
    },
    Layer {
        machine: ObjectId,
        spec: MachineLayerSpec,
    },
    AnyState {
        layer: ObjectId,
    },
    EntryState {
        layer: ObjectId,
    },
    ExitState {
        layer: ObjectId,
    },
    AnimationState {
        layer: ObjectId,
        animation: AnimationId,
    },
    Transition {
        source: ObjectId,
        target: ObjectId,
    },
    TriggerCondition {
        transition: ObjectId,
        input: MachineInputId,
    },
    FireEvent {
        state: ObjectId,
        event: EventId,
        occurs: FireEventOccurs,
    },
}

impl MachineRecordSpec {
    const fn owner(&self) -> Option<ObjectId> {
        match self {
            Self::Event(_) | Self::Machine(_) => None,
            Self::TriggerInput { machine, .. } | Self::Layer { machine, .. } => Some(*machine),
            Self::AnyState { layer }
            | Self::EntryState { layer }
            | Self::ExitState { layer }
            | Self::AnimationState { layer, .. } => Some(*layer),
            Self::Transition { source, .. } => Some(*source),
            Self::TriggerCondition { transition, .. } => Some(*transition),
            Self::FireEvent { state, .. } => Some(*state),
        }
    }

    const fn is_entry_state(&self) -> bool {
        matches!(self, Self::EntryState { .. })
    }

    const fn is_any_state(&self) -> bool {
        matches!(self, Self::AnyState { .. })
    }

    const fn is_exit_state(&self) -> bool {
        matches!(self, Self::ExitState { .. })
    }
}

/// Deep private seam for every authored hierarchy invariant. A transaction
/// keeps one indexed candidate; each method completely preflights one operation
/// before mutating it, and commit validates and stabilizes parent-before-child
/// record order once. This module owns sibling order, subtree movement, cycle
/// and parent validation, and stable topological ordering.
struct Hierarchy<'a> {
    definitions: &'a mut Definitions,
    index: &'a DefinitionIndex,
    operation_index: usize,
}

impl Hierarchy<'_> {
    fn remove_artboard(&mut self, artboard: ArtboardId) -> std::result::Result<(), EditAbort> {
        let index = self
            .index
            .artboards
            .get(&artboard)
            .copied()
            .ok_or_else(|| {
                self.abort(
                    vec![EditId::Artboard(artboard)],
                    EditReason::UnknownArtboard,
                )
            })?;
        self.definitions.artboards.remove(index);
        Ok(())
    }

    fn clear_artboard(&mut self, artboard: ArtboardId) -> std::result::Result<(), EditAbort> {
        let artboard_index = self
            .index
            .artboards
            .get(&artboard)
            .copied()
            .ok_or_else(|| {
                self.abort(
                    vec![EditId::Artboard(artboard)],
                    EditReason::UnknownArtboard,
                )
            })?;
        let definition = self
            .definitions
            .artboards
            .get_mut(artboard_index)
            .ok_or_else(|| {
                EditAbort::new(
                    self.operation_index,
                    vec![EditId::Artboard(artboard)],
                    EditReason::InternalInvariant,
                )
            })?;
        definition.records.clear();
        Ok(())
    }

    fn set_artboard(
        &mut self,
        artboard: ArtboardId,
        spec: ArtboardSpec,
    ) -> std::result::Result<(), EditAbort> {
        let artboard_index = self
            .index
            .artboards
            .get(&artboard)
            .copied()
            .ok_or_else(|| {
                EditAbort::new(
                    self.operation_index,
                    vec![EditId::Artboard(artboard)],
                    EditReason::UnknownArtboard,
                )
            })?;
        let definition = self
            .definitions
            .artboards
            .get_mut(artboard_index)
            .ok_or_else(|| {
                EditAbort::new(
                    self.operation_index,
                    vec![EditId::Artboard(artboard)],
                    EditReason::InternalInvariant,
                )
            })?;
        definition.spec = spec;
        Ok(())
    }

    fn set<T>(
        &mut self,
        object: ObjectId,
        property: Prop<T>,
        value: T,
    ) -> std::result::Result<ArtboardId, EditAbort> {
        let (artboard_index, record_index) = self
            .object_location(object)
            .ok_or_else(|| self.abort(vec![EditId::Object(object)], EditReason::UnknownObject))?;
        let definition = self
            .definitions
            .artboards
            .get_mut(artboard_index)
            .ok_or_else(|| {
                EditAbort::new(
                    self.operation_index,
                    vec![EditId::Object(object)],
                    EditReason::InternalInvariant,
                )
            })?;
        let artboard_id = definition.id;
        let record = definition.records.get_mut(record_index).ok_or_else(|| {
            EditAbort::new(
                self.operation_index,
                vec![EditId::Object(object)],
                EditReason::InternalInvariant,
            )
        })?;
        let mut candidate = record.spec.clone();
        (property.apply_to_definition)(&mut candidate, value).map_err(|reason| {
            EditAbort::new(self.operation_index, vec![EditId::Object(object)], reason)
        })?;
        record.spec = candidate;
        Ok(artboard_id)
    }

    fn reorder_artboard(
        &mut self,
        artboard: ArtboardId,
        index: ChildIndex,
    ) -> std::result::Result<(), EditAbort> {
        let current_index = self
            .index
            .artboards
            .get(&artboard)
            .copied()
            .ok_or_else(|| {
                self.abort(
                    vec![EditId::Artboard(artboard)],
                    EditReason::UnknownArtboard,
                )
            })?;
        let final_index = resolve_child_index(index, self.definitions.artboards.len(), false)
            .ok_or_else(|| {
                self.abort(
                    vec![EditId::Artboard(artboard)],
                    EditReason::ChildIndexOutOfRange,
                )
            })?;
        let definition = self.definitions.artboards.remove(current_index);
        self.definitions.artboards.insert(final_index, definition);
        Ok(())
    }

    fn reorder(
        &mut self,
        object: ObjectId,
        index: ChildIndex,
    ) -> std::result::Result<ArtboardId, EditAbort> {
        let indexed = self
            .indexed_object(object)
            .ok_or_else(|| self.abort(vec![EditId::Object(object)], EditReason::UnknownObject))?;
        if indexed.visual_kind().is_none() {
            return Err(self.abort(vec![EditId::Object(object)], EditReason::NonVisualObject));
        }
        let parent = self
            .definitions
            .artboards
            .get(indexed.artboard_index)
            .and_then(|artboard| artboard.records.get(indexed.record_index))
            .and_then(|record| record.spec.visual().map(|(parent, _)| parent))
            .ok_or_else(|| {
                self.abort(vec![EditId::Object(object)], EditReason::InternalInvariant)
            })?;
        let (source, target) = self.move_subtree(object, parent, index)?;
        debug_assert_eq!(source, target);
        Ok(source)
    }

    fn set_child_order(
        &mut self,
        parent: Parent,
        ordered_children: &[ObjectId],
    ) -> std::result::Result<ArtboardId, EditAbort> {
        let (artboard_id, artboard_index) = match parent {
            Parent::Artboard(artboard) => {
                let artboard_index =
                    self.index
                        .artboards
                        .get(&artboard)
                        .copied()
                        .ok_or_else(|| {
                            self.abort(
                                vec![EditId::Artboard(artboard)],
                                EditReason::UnknownArtboard,
                            )
                        })?;
                (artboard, artboard_index)
            }
            Parent::Object(object) => {
                let indexed = self.indexed_object(object).ok_or_else(|| {
                    self.abort(vec![EditId::Object(object)], EditReason::UnknownObject)
                })?;
                if indexed.visual_kind().is_none() {
                    return Err(
                        self.abort(vec![EditId::Object(object)], EditReason::NonVisualObject)
                    );
                }
                (indexed.artboard, indexed.artboard_index)
            }
        };
        let expected_children = self
            .index
            .children
            .get(&parent)
            .map(Vec::as_slice)
            .unwrap_or_default();
        let mut requested = BTreeSet::new();
        for child in ordered_children {
            let indexed = self.indexed_object(*child).ok_or_else(|| {
                self.abort(vec![EditId::Object(*child)], EditReason::UnknownObject)
            })?;
            if indexed.visual_kind().is_none() {
                return Err(self.abort(vec![EditId::Object(*child)], EditReason::NonVisualObject));
            }
            let actual_parent = self
                .definitions
                .artboards
                .get(indexed.artboard_index)
                .and_then(|artboard| artboard.records.get(indexed.record_index))
                .and_then(|record| record.spec.visual().map(|(parent, _)| parent))
                .ok_or_else(|| {
                    self.abort(vec![EditId::Object(*child)], EditReason::InternalInvariant)
                })?;
            if indexed.artboard != artboard_id || actual_parent != parent {
                let mut involved = parent_edit_ids(parent);
                involved.push(EditId::Object(*child));
                return Err(self.abort(involved, EditReason::ChildSetMismatch));
            }
            if !requested.insert(*child) {
                return Err(self.abort(vec![EditId::Object(*child)], EditReason::ChildSetMismatch));
            }
        }
        if requested.len() != expected_children.len()
            || expected_children
                .iter()
                .any(|child| !requested.contains(child))
        {
            let mut involved = parent_edit_ids(parent);
            involved.extend(
                expected_children
                    .iter()
                    .filter(|child| !requested.contains(child))
                    .copied()
                    .map(EditId::Object),
            );
            return Err(self.abort(involved, EditReason::ChildSetMismatch));
        }

        let mut subtree_owner = BTreeMap::new();
        let mut frontier = expected_children
            .iter()
            .copied()
            .map(|child| (child, child))
            .collect::<Vec<_>>();
        while let Some((object, owner)) = frontier.pop() {
            if subtree_owner.insert(object, owner).is_some() {
                return Err(self.abort(vec![EditId::Object(object)], EditReason::InternalInvariant));
            }
            if let Some(children) = self.index.children.get(&Parent::Object(object)) {
                frontier.extend(children.iter().copied().map(|child| (child, owner)));
            }
        }

        let artboard = self
            .definitions
            .artboards
            .get(artboard_index)
            .ok_or_else(|| self.abort(parent_edit_ids(parent), EditReason::InternalInvariant))?;
        let grouped_node_count = artboard
            .records
            .iter()
            .filter(|node| subtree_owner.contains_key(&node.id))
            .count();
        if grouped_node_count != subtree_owner.len() {
            return Err(self.abort(parent_edit_ids(parent), EditReason::InternalInvariant));
        }

        // Everything below is infallible. Each direct child's complete subtree
        // becomes one stable block, and those blocks are spliced at the first
        // existing child position in the caller's exact order.
        let rank_by_root = ordered_children
            .iter()
            .enumerate()
            .map(|(rank, root)| (*root, rank))
            .collect::<BTreeMap<_, _>>();
        let artboard = self
            .definitions
            .artboards
            .get_mut(artboard_index)
            .expect("preflighted child-order artboard exists");
        let records = std::mem::take(&mut artboard.records);
        let mut remaining = Vec::with_capacity(records.len().saturating_sub(grouped_node_count));
        let mut groups = (0..ordered_children.len())
            .map(|_| Vec::new())
            .collect::<Vec<Vec<RecordDefinition>>>();
        let mut insertion_index = None;
        for record in records {
            if let Some(owner) = subtree_owner.get(&record.id) {
                insertion_index.get_or_insert(remaining.len());
                let rank = rank_by_root
                    .get(owner)
                    .copied()
                    .expect("preflighted subtree owner has an exact rank");
                groups
                    .get_mut(rank)
                    .expect("preflighted subtree rank has a group")
                    .push(record);
            } else {
                remaining.push(record);
            }
        }
        let ordered_nodes = groups.into_iter().flatten().collect::<Vec<_>>();
        let insertion_index = insertion_index.unwrap_or(remaining.len());
        remaining.splice(insertion_index..insertion_index, ordered_nodes);
        artboard.records = remaining;
        Ok(artboard_id)
    }

    fn reparent(
        &mut self,
        object: ObjectId,
        new_parent: Parent,
        index: ChildIndex,
    ) -> std::result::Result<(ArtboardId, ArtboardId), EditAbort> {
        self.move_subtree(object, new_parent, index)
    }

    fn move_subtree(
        &mut self,
        object: ObjectId,
        new_parent: Parent,
        index: ChildIndex,
    ) -> std::result::Result<(ArtboardId, ArtboardId), EditAbort> {
        let source = self
            .indexed_object(object)
            .ok_or_else(|| self.abort(vec![EditId::Object(object)], EditReason::UnknownObject))?;
        let current_parent = self
            .definitions
            .artboards
            .get(source.artboard_index)
            .and_then(|artboard| artboard.records.get(source.record_index))
            .and_then(|record| record.spec.visual().map(|(parent, _)| parent))
            .ok_or_else(|| {
                self.abort(vec![EditId::Object(object)], EditReason::InternalInvariant)
            })?;
        let child_kind = source
            .visual_kind()
            .ok_or_else(|| self.abort(vec![EditId::Object(object)], EditReason::NonVisualObject))?;
        let (target_index, parent_kind) = match new_parent {
            Parent::Artboard(artboard) => {
                let target = self
                    .index
                    .artboards
                    .get(&artboard)
                    .copied()
                    .ok_or_else(|| {
                        self.abort(
                            vec![EditId::Artboard(artboard)],
                            EditReason::UnknownArtboard,
                        )
                    })?;
                (target, None)
            }
            Parent::Object(parent) => {
                let indexed = self.indexed_object(parent).ok_or_else(|| {
                    self.abort(vec![EditId::Object(parent)], EditReason::UnknownObject)
                })?;
                let parent_kind = indexed.visual_kind().ok_or_else(|| {
                    self.abort(
                        vec![EditId::Object(parent)],
                        EditReason::InvalidParent {
                            parent: None,
                            child: child_kind,
                        },
                    )
                })?;
                (indexed.artboard_index, Some(parent_kind))
            }
        };

        let subtree_ids = self.subtree_ids(object);
        debug_assert!(subtree_ids.contains(&object));
        if matches!(new_parent, Parent::Object(parent) if subtree_ids.contains(&parent)) {
            return Err(self.abort(
                vec![
                    EditId::Object(object),
                    EditId::Object(match new_parent {
                        Parent::Object(parent) => parent,
                        Parent::Artboard(_) => unreachable!(),
                    }),
                ],
                EditReason::CycleDetected,
            ));
        }

        let parent_is_valid = match parent_kind {
            None => matches!(
                child_kind,
                NodeKind::Shape | NodeKind::Text | NodeKind::ScriptedDrawable
            ),
            Some(parent) => valid_object_parent(parent, child_kind),
        };
        if !parent_is_valid {
            return Err(self.abort(
                parent_edit_ids(new_parent),
                EditReason::InvalidParent {
                    parent: parent_kind,
                    child: child_kind,
                },
            ));
        }

        let same_parent = current_parent == new_parent;
        let sibling_count = self
            .index
            .children
            .get(&new_parent)
            .map(Vec::len)
            .unwrap_or(0);
        let final_index =
            resolve_child_index(index, sibling_count, !same_parent).ok_or_else(|| {
                self.abort(
                    vec![EditId::Object(object)],
                    EditReason::ChildIndexOutOfRange,
                )
            })?;

        let source_artboard = source.artboard;
        let target_artboard = self
            .definitions
            .artboards
            .get(target_index)
            .map(|artboard| artboard.id)
            .ok_or_else(|| {
                self.abort(parent_edit_ids(new_parent), EditReason::InternalInvariant)
            })?;
        let source_index = source.artboard_index;
        let operation_index = self.operation_index;
        if source_index == target_index {
            let definition = self
                .definitions
                .artboards
                .get_mut(source_index)
                .ok_or_else(|| {
                    EditAbort::new(
                        operation_index,
                        vec![EditId::Object(object)],
                        EditReason::InternalInvariant,
                    )
                })?;
            let subtree = detach_preflighted_subtree(
                &mut definition.records,
                &subtree_ids,
                object,
                new_parent,
            );
            attach_preflighted_subtree(&mut definition.records, new_parent, final_index, subtree);
        } else {
            let [source_definition, target_definition] = self
                .definitions
                .artboards
                .get_disjoint_mut([source_index, target_index])
                .map_err(|_| {
                    EditAbort::new(
                        operation_index,
                        vec![EditId::Object(object)],
                        EditReason::InternalInvariant,
                    )
                })?;
            let subtree = detach_preflighted_subtree(
                &mut source_definition.records,
                &subtree_ids,
                object,
                new_parent,
            );
            attach_preflighted_subtree(
                &mut target_definition.records,
                new_parent,
                final_index,
                subtree,
            );
        }
        Ok((source_artboard, target_artboard))
    }

    fn object_location(&self, object: ObjectId) -> Option<(usize, usize)> {
        self.index
            .objects
            .get(&object)
            .map(|object| (object.artboard_index, object.record_index))
    }

    fn indexed_object(&self, object: ObjectId) -> Option<IndexedObject> {
        self.index.objects.get(&object).copied()
    }

    fn subtree_ids(&self, root: ObjectId) -> BTreeSet<ObjectId> {
        let mut subtree = BTreeSet::from([root]);
        let mut frontier = vec![root];
        while let Some(parent) = frontier.pop() {
            if let Some(children) = self.index.owned.get(&parent) {
                for child in children {
                    if subtree.insert(*child) {
                        frontier.push(*child);
                    }
                }
            }
        }
        subtree
    }

    fn detach_subtree(
        &mut self,
        object: ObjectId,
    ) -> std::result::Result<RemovedSubtree, EditAbort> {
        let (artboard_index, _) = self
            .object_location(object)
            .ok_or_else(|| self.abort(vec![EditId::Object(object)], EditReason::UnknownObject))?;
        let subtree_ids = self.subtree_ids(object);
        let artboard = self
            .definitions
            .artboards
            .get_mut(artboard_index)
            .ok_or_else(|| {
                EditAbort::new(
                    self.operation_index,
                    vec![EditId::Object(object)],
                    EditReason::InternalInvariant,
                )
            })?;
        let mut records = Vec::with_capacity(subtree_ids.len());
        let mut remaining =
            Vec::with_capacity(artboard.records.len().saturating_sub(subtree_ids.len()));
        for (original_index, definition) in std::mem::take(&mut artboard.records)
            .into_iter()
            .enumerate()
        {
            if subtree_ids.contains(&definition.id) {
                records.push(RemovedRecord {
                    original_index,
                    definition,
                });
            } else {
                remaining.push(definition);
            }
        }
        debug_assert!(!records.is_empty());
        artboard.records = remaining;
        Ok(RemovedSubtree {
            artboard: artboard.id,
            root: object,
            records,
        })
    }

    fn attach_subtree(
        &mut self,
        removed: RemovedSubtree,
    ) -> std::result::Result<(ArtboardId, ObjectId, Vec<ObjectId>), EditAbort> {
        let RemovedSubtree {
            artboard: artboard_id,
            root,
            records,
        } = removed;
        if records.is_empty() || !records.iter().any(|record| record.definition.id == root) {
            return Err(self.abort(vec![EditId::Object(root)], EditReason::InternalInvariant));
        }
        let artboard_index = self
            .index
            .artboards
            .get(&artboard_id)
            .copied()
            .ok_or_else(|| {
                self.abort(
                    vec![EditId::Artboard(artboard_id), EditId::Object(root)],
                    EditReason::UnknownArtboard,
                )
            })?;
        let mut restored_kinds = BTreeMap::new();
        for removed_record in &records {
            let id = removed_record.definition.id;
            if self.object_location(id).is_some() {
                return Err(self.abort(vec![EditId::Object(id)], EditReason::IdentityCollision));
            }
            if restored_kinds
                .insert(id, removed_record.definition.spec.kind())
                .is_some()
            {
                return Err(self.abort(vec![EditId::Object(id)], EditReason::IdentityCollision));
            }
        }

        let resolve_kind = |id: ObjectId| {
            restored_kinds
                .get(&id)
                .copied()
                .or_else(|| self.index.objects.get(&id).map(|indexed| indexed.kind))
        };
        let resolve_artboard = |id: ObjectId| {
            restored_kinds
                .contains_key(&id)
                .then_some(artboard_id)
                .or_else(|| self.index.objects.get(&id).map(|indexed| indexed.artboard))
        };
        for removed_record in &records {
            let id = removed_record.definition.id;
            match &removed_record.definition.spec {
                RecordSpec::Visual { parent, node } => {
                    let child = node.kind();
                    let parent_kind = match parent {
                        Parent::Artboard(parent) if *parent == artboard_id => None,
                        Parent::Artboard(parent) if !self.index.artboards.contains_key(parent) => {
                            return Err(self.abort(
                                vec![EditId::Object(id), EditId::Artboard(*parent)],
                                EditReason::UnknownArtboard,
                            ));
                        }
                        Parent::Artboard(parent) => {
                            return Err(self.abort(
                                vec![EditId::Object(id), EditId::Artboard(*parent)],
                                EditReason::CrossArtboardReference {
                                    source: artboard_id,
                                    target: *parent,
                                },
                            ));
                        }
                        Parent::Object(parent) => {
                            let Some(kind) = resolve_kind(*parent) else {
                                return Err(self.abort(
                                    vec![EditId::Object(id), EditId::Object(*parent)],
                                    EditReason::UnknownObject,
                                ));
                            };
                            let Some(parent_artboard) = resolve_artboard(*parent) else {
                                return Err(self.abort(
                                    vec![EditId::Object(id), EditId::Object(*parent)],
                                    EditReason::UnknownObject,
                                ));
                            };
                            if parent_artboard != artboard_id {
                                return Err(self.abort(
                                    vec![EditId::Object(id), EditId::Object(*parent)],
                                    EditReason::CrossArtboardReference {
                                        source: artboard_id,
                                        target: parent_artboard,
                                    },
                                ));
                            }
                            let AuthoredObjectKind::Visual(kind) = kind else {
                                return Err(self.abort(
                                    vec![EditId::Object(id), EditId::Object(*parent)],
                                    EditReason::NonVisualObject,
                                ));
                            };
                            Some(kind)
                        }
                    };
                    let valid = match parent_kind {
                        None => valid_artboard_child(child),
                        Some(parent) => valid_object_parent(parent, child),
                    };
                    if !valid {
                        return Err(self.abort(
                            parent_edit_ids(*parent),
                            EditReason::InvalidParent {
                                parent: parent_kind,
                                child,
                            },
                        ));
                    }
                }
                RecordSpec::Animation(spec) => {
                    if let Some(owner) = spec.owner() {
                        let Some(owner_kind) = resolve_kind(owner) else {
                            return Err(self.abort(
                                vec![EditId::Object(id), EditId::Object(owner)],
                                EditReason::UnknownObject,
                            ));
                        };
                        let Some(owner_artboard) = resolve_artboard(owner) else {
                            return Err(self.abort(
                                vec![EditId::Object(id), EditId::Object(owner)],
                                EditReason::UnknownObject,
                            ));
                        };
                        if owner_artboard != artboard_id {
                            return Err(self.abort(
                                vec![EditId::Object(id), EditId::Object(owner)],
                                EditReason::CrossArtboardReference {
                                    source: artboard_id,
                                    target: owner_artboard,
                                },
                            ));
                        }
                        let valid_owner = matches!(
                            (spec, owner_kind),
                            (
                                AnimationRecordSpec::KeyedObject { .. },
                                AuthoredObjectKind::LinearAnimation
                            ) | (
                                AnimationRecordSpec::KeyedProperty { .. },
                                AuthoredObjectKind::KeyedObject
                            ) | (
                                AnimationRecordSpec::KeyFrameDouble { .. },
                                AuthoredObjectKind::KeyedProperty
                            )
                        );
                        if !valid_owner {
                            return Err(self.abort(
                                vec![EditId::Object(id), EditId::Object(owner)],
                                EditReason::InternalInvariant,
                            ));
                        }
                    }
                    if let AnimationRecordSpec::KeyedObject { target, .. } = spec {
                        let Some(target_kind) = resolve_kind(*target) else {
                            return Err(self.abort(
                                vec![EditId::Object(id), EditId::Object(*target)],
                                EditReason::UnknownObject,
                            ));
                        };
                        let Some(target_artboard) = resolve_artboard(*target) else {
                            return Err(self.abort(
                                vec![EditId::Object(id), EditId::Object(*target)],
                                EditReason::UnknownObject,
                            ));
                        };
                        if target_artboard != artboard_id {
                            return Err(self.abort(
                                vec![EditId::Object(id), EditId::Object(*target)],
                                EditReason::CrossArtboardReference {
                                    source: artboard_id,
                                    target: target_artboard,
                                },
                            ));
                        }
                        if !matches!(target_kind, AuthoredObjectKind::Visual(_)) {
                            return Err(self.abort(
                                vec![EditId::Object(id), EditId::Object(*target)],
                                EditReason::NonVisualObject,
                            ));
                        }
                    }
                }
                RecordSpec::Machine(spec) => {
                    if let Some(owner) = spec.owner() {
                        let Some(owner_kind) = resolve_kind(owner) else {
                            return Err(self.abort(
                                vec![EditId::Object(id), EditId::Object(owner)],
                                EditReason::UnknownObject,
                            ));
                        };
                        let Some(owner_artboard) = resolve_artboard(owner) else {
                            return Err(self.abort(
                                vec![EditId::Object(id), EditId::Object(owner)],
                                EditReason::UnknownObject,
                            ));
                        };
                        if owner_artboard != artboard_id {
                            return Err(self.abort(
                                vec![EditId::Object(id), EditId::Object(owner)],
                                EditReason::CrossArtboardReference {
                                    source: artboard_id,
                                    target: owner_artboard,
                                },
                            ));
                        }
                        let owner_is_valid = matches!(
                            (spec, owner_kind),
                            (
                                MachineRecordSpec::TriggerInput { .. }
                                    | MachineRecordSpec::Layer { .. },
                                AuthoredObjectKind::Machine
                            ) | (
                                MachineRecordSpec::AnyState { .. }
                                    | MachineRecordSpec::EntryState { .. }
                                    | MachineRecordSpec::ExitState { .. }
                                    | MachineRecordSpec::AnimationState { .. },
                                AuthoredObjectKind::MachineLayer
                            ) | (
                                MachineRecordSpec::Transition { .. }
                                    | MachineRecordSpec::FireEvent { .. },
                                AuthoredObjectKind::AnyState
                                    | AuthoredObjectKind::EntryState
                                    | AuthoredObjectKind::ExitState
                                    | AuthoredObjectKind::AnimationState
                            ) | (
                                MachineRecordSpec::TriggerCondition { .. },
                                AuthoredObjectKind::StateTransition
                            )
                        );
                        if !owner_is_valid {
                            return Err(self.abort(
                                vec![EditId::Object(id), EditId::Object(owner)],
                                EditReason::InternalInvariant,
                            ));
                        }
                    }

                    let referenced = match spec {
                        MachineRecordSpec::AnimationState { animation, .. } => {
                            Some((animation.object_id(), AuthoredObjectKind::LinearAnimation))
                        }
                        MachineRecordSpec::Transition { target, .. } => {
                            let Some(kind) = resolve_kind(*target) else {
                                return Err(self.abort(
                                    vec![EditId::Object(id), EditId::Object(*target)],
                                    EditReason::UnknownObject,
                                ));
                            };
                            if !matches!(
                                kind,
                                AuthoredObjectKind::AnyState
                                    | AuthoredObjectKind::EntryState
                                    | AuthoredObjectKind::ExitState
                                    | AuthoredObjectKind::AnimationState
                            ) {
                                return Err(self.abort(
                                    vec![EditId::Object(id), EditId::Object(*target)],
                                    EditReason::InternalInvariant,
                                ));
                            }
                            None
                        }
                        MachineRecordSpec::TriggerCondition { input, .. } => {
                            Some((input.object_id(), AuthoredObjectKind::MachineTrigger))
                        }
                        MachineRecordSpec::FireEvent { event, .. } => {
                            Some((event.object_id(), AuthoredObjectKind::Event))
                        }
                        _ => None,
                    };
                    if let Some((target, expected)) = referenced {
                        let Some(actual) = resolve_kind(target) else {
                            return Err(self.abort(
                                vec![EditId::Object(id), EditId::Object(target)],
                                EditReason::UnknownObject,
                            ));
                        };
                        if actual != expected {
                            return Err(self.abort(
                                vec![EditId::Object(id), EditId::Object(target)],
                                EditReason::InternalInvariant,
                            ));
                        }
                        let Some(target_artboard) = resolve_artboard(target) else {
                            return Err(self.abort(
                                vec![EditId::Object(id), EditId::Object(target)],
                                EditReason::UnknownObject,
                            ));
                        };
                        if target_artboard != artboard_id {
                            return Err(self.abort(
                                vec![EditId::Object(id), EditId::Object(target)],
                                EditReason::CrossArtboardReference {
                                    source: artboard_id,
                                    target: target_artboard,
                                },
                            ));
                        }
                    }
                }
            }
        }

        let existing_len = self
            .definitions
            .artboards
            .get(artboard_index)
            .map(|artboard| artboard.records.len())
            .ok_or_else(|| {
                self.abort(
                    vec![EditId::Artboard(artboard_id), EditId::Object(root)],
                    EditReason::InternalInvariant,
                )
            })?;
        let mut insertions = Vec::with_capacity(records.len());
        let mut last_position = None;
        for (offset, removed_record) in records.into_iter().enumerate() {
            let position = removed_record
                .original_index
                .min(existing_len.saturating_add(offset));
            if last_position.is_some_and(|last| position <= last) {
                return Err(self.abort(vec![EditId::Object(root)], EditReason::InternalInvariant));
            }
            last_position = Some(position);
            insertions.push((position, removed_record.definition));
        }
        let restored = insertions
            .iter()
            .map(|(_, definition)| definition.id)
            .collect::<Vec<_>>();

        // Everything below is infallible. The merge exactly reproduces the
        // previous sequential `original_index.min(current_len)` insertion
        // semantics without repeatedly shifting the authored vector.
        let final_len = existing_len.checked_add(insertions.len()).ok_or_else(|| {
            self.abort(
                vec![EditId::Artboard(artboard_id), EditId::Object(root)],
                EditReason::CapacityExceeded,
            )
        })?;
        let artboard = self
            .definitions
            .artboards
            .get_mut(artboard_index)
            .ok_or_else(|| {
                EditAbort::new(
                    self.operation_index,
                    vec![EditId::Artboard(artboard_id), EditId::Object(root)],
                    EditReason::InternalInvariant,
                )
            })?;
        let existing = std::mem::take(&mut artboard.records);
        let mut existing = existing.into_iter();
        let mut insertions = insertions.into_iter().peekable();
        let mut merged = Vec::with_capacity(final_len);
        for position in 0..final_len {
            if insertions
                .peek()
                .is_some_and(|(insertion_position, _)| *insertion_position == position)
            {
                merged.push(insertions.next().expect("peeked insertion exists").1);
            } else {
                merged.push(existing.next().expect("preflighted merge length is exact"));
            }
        }
        debug_assert!(existing.next().is_none());
        debug_assert!(insertions.next().is_none());
        artboard.records = merged;
        Ok((artboard_id, root, restored))
    }

    fn canonicalize_and_validate(
        definitions: &mut Definitions,
        operation_index: usize,
    ) -> std::result::Result<(), EditAbort> {
        let abort = |involved_ids, reason| EditAbort::new(operation_index, involved_ids, reason);
        let mut artboard_ids = BTreeSet::new();
        let mut objects = BTreeMap::new();
        let script_asset_ids = definitions
            .script_assets
            .iter()
            .map(|script| script.id)
            .collect::<BTreeSet<_>>();
        for artboard in &definitions.artboards {
            if !artboard_ids.insert(artboard.id) {
                return Err(abort(
                    vec![EditId::Artboard(artboard.id)],
                    EditReason::IdentityCollision,
                ));
            }
            for record in &artboard.records {
                let Some((_, node)) = record.visual() else {
                    continue;
                };
                if objects
                    .insert(record.id, (artboard.id, node.kind()))
                    .is_some()
                {
                    return Err(abort(
                        vec![EditId::Object(record.id)],
                        EditReason::IdentityCollision,
                    ));
                }
            }
        }

        // Validate references before following parent chains. Parent-kind
        // validation intentionally happens only after cycle detection.
        let mut artboard_references = BTreeMap::<ArtboardId, Vec<(ArtboardId, ObjectId)>>::new();
        for artboard in &definitions.artboards {
            for record in &artboard.records {
                let Some((parent, node)) = record.visual() else {
                    continue;
                };
                if let NodeSpec::NestedArtboard(spec) = node {
                    if !artboard_ids.contains(&spec.artboard) {
                        return Err(abort(
                            vec![EditId::Object(record.id), EditId::Artboard(spec.artboard)],
                            EditReason::UnknownArtboard,
                        ));
                    }
                    artboard_references
                        .entry(artboard.id)
                        .or_default()
                        .push((spec.artboard, record.id));
                }
                if let NodeSpec::ScriptedDrawable(spec) = node
                    && !script_asset_ids.contains(&spec.script)
                {
                    return Err(abort(
                        vec![EditId::Object(record.id), EditId::ScriptAsset(spec.script)],
                        EditReason::UnknownScriptAsset,
                    ));
                }
                match parent {
                    Parent::Artboard(parent) if parent == artboard.id => {}
                    Parent::Artboard(parent) if !artboard_ids.contains(&parent) => {
                        return Err(abort(
                            vec![EditId::Object(record.id), EditId::Artboard(parent)],
                            EditReason::UnknownArtboard,
                        ));
                    }
                    Parent::Artboard(parent) => {
                        return Err(abort(
                            vec![EditId::Object(record.id), EditId::Artboard(parent)],
                            EditReason::CrossArtboardReference {
                                source: artboard.id,
                                target: parent,
                            },
                        ));
                    }
                    Parent::Object(parent) => match objects.get(&parent) {
                        None => {
                            return Err(abort(
                                vec![EditId::Object(record.id), EditId::Object(parent)],
                                EditReason::UnknownObject,
                            ));
                        }
                        Some((parent_artboard, _)) if *parent_artboard != artboard.id => {
                            return Err(abort(
                                vec![EditId::Object(record.id), EditId::Object(parent)],
                                EditReason::CrossArtboardReference {
                                    source: artboard.id,
                                    target: *parent_artboard,
                                },
                            ));
                        }
                        Some(_) => {}
                    },
                }
            }
        }
        for artboard in &definitions.artboards {
            let mut path = Vec::new();
            if let Some(cycle) =
                first_artboard_reference_cycle(artboard.id, &artboard_references, &mut path)
            {
                return Err(abort(
                    cycle
                        .into_iter()
                        .flat_map(|(artboard, node)| {
                            [EditId::Artboard(artboard), EditId::Object(node)]
                        })
                        .collect(),
                    EditReason::CycleDetected,
                ));
            }
        }

        for artboard in &definitions.artboards {
            let nodes = artboard
                .records
                .iter()
                .filter_map(|record| {
                    record
                        .visual()
                        .map(|(parent, node)| (record.id, (parent, node)))
                })
                .collect::<BTreeMap<_, _>>();
            let mut complete = BTreeSet::new();
            for record in &artboard.records {
                let Some(_) = record.visual() else {
                    continue;
                };
                if complete.contains(&record.id) {
                    continue;
                }
                let mut path = Vec::new();
                let mut path_ids = BTreeSet::new();
                let mut cursor = record.id;
                loop {
                    if complete.contains(&cursor) {
                        break;
                    }
                    if !path_ids.insert(cursor) {
                        return Err(abort(
                            vec![EditId::Object(record.id), EditId::Object(cursor)],
                            EditReason::CycleDetected,
                        ));
                    }
                    path.push(cursor);
                    let Some(current) = nodes.get(&cursor) else {
                        return Err(abort(
                            vec![EditId::Object(cursor)],
                            EditReason::InternalInvariant,
                        ));
                    };
                    match current.0 {
                        Parent::Artboard(_) => break,
                        Parent::Object(parent) => cursor = parent,
                    }
                }
                complete.extend(path);
            }
        }

        for artboard in &definitions.artboards {
            for record in &artboard.records {
                let Some((parent, node)) = record.visual() else {
                    continue;
                };
                let child = node.kind();
                let parent_kind = match parent {
                    Parent::Artboard(_) => None,
                    Parent::Object(parent) => objects.get(&parent).map(|(_, kind)| *kind),
                };
                let valid = match parent_kind {
                    None => valid_artboard_child(child),
                    Some(parent) => valid_object_parent(parent, child),
                };
                if !valid {
                    return Err(abort(
                        parent_edit_ids(parent),
                        EditReason::InvalidParent {
                            parent: parent_kind,
                            child,
                        },
                    ));
                }
            }
        }

        for artboard in &mut definitions.artboards {
            let records = std::mem::take(&mut artboard.records);
            let visual_records = records
                .iter()
                .filter(|record| record.visual().is_some())
                .cloned()
                .collect::<Vec<_>>();
            let positions = visual_records
                .iter()
                .enumerate()
                .map(|(index, record)| (record.id, index))
                .collect::<BTreeMap<_, _>>();
            let already_parent_before_child =
                visual_records.iter().enumerate().all(|(index, record)| {
                    match record.visual().map(|(parent, _)| parent) {
                        Some(Parent::Artboard(_)) => true,
                        Some(Parent::Object(parent)) => positions
                            .get(&parent)
                            .is_some_and(|parent_index| *parent_index < index),
                        None => false,
                    }
                });
            if already_parent_before_child {
                artboard.records = records;
                continue;
            }

            // Preserve authored record order whenever it is already valid. If
            // a reparent makes a parent appear after its child, use the
            // original visual-record rank as Kahn's ready-queue priority. The
            // repaired visual order is written back into the same visual slots,
            // leaving ordinary nonvisual records in their authored positions.
            let mut ready = BTreeSet::new();
            let mut children: BTreeMap<ObjectId, Vec<usize>> = BTreeMap::new();
            for (index, record) in visual_records.iter().enumerate() {
                match record.visual().map(|(parent, _)| parent) {
                    Some(Parent::Artboard(_)) => {
                        ready.insert(index);
                    }
                    Some(Parent::Object(parent)) => children.entry(parent).or_default().push(index),
                    None => {
                        return Err(EditAbort::new(
                            operation_index,
                            vec![EditId::Object(record.id)],
                            EditReason::InternalInvariant,
                        ));
                    }
                }
            }
            let mut stable = Vec::with_capacity(visual_records.len());
            while let Some(index) = ready.pop_first() {
                let Some(record) = visual_records.get(index).cloned() else {
                    return Err(EditAbort::new(
                        operation_index,
                        vec![EditId::Artboard(artboard.id)],
                        EditReason::InternalInvariant,
                    ));
                };
                if let Some(child_indices) = children.get(&record.id) {
                    ready.extend(child_indices.iter().copied());
                }
                stable.push(record);
            }
            if stable.len() != visual_records.len() {
                return Err(EditAbort::new(
                    operation_index,
                    vec![EditId::Artboard(artboard.id)],
                    EditReason::InternalInvariant,
                ));
            }
            let mut stable = stable.into_iter();
            artboard.records = records
                .into_iter()
                .map(|record| {
                    if record.visual().is_some() {
                        stable
                            .next()
                            .expect("validated visual order has exact length")
                    } else {
                        record
                    }
                })
                .collect();
            debug_assert!(stable.next().is_none());
        }
        Ok(())
    }

    fn abort(&self, involved_ids: Vec<EditId>, reason: EditReason) -> EditAbort {
        EditAbort::new(self.operation_index, involved_ids, reason)
    }
}

fn resolve_child_index(index: ChildIndex, sibling_count: usize, inserting: bool) -> Option<usize> {
    let upper_bound = if inserting {
        sibling_count
    } else {
        sibling_count.checked_sub(1)?
    };
    match index {
        ChildIndex::First => Some(0),
        ChildIndex::Last => Some(upper_bound),
        ChildIndex::At(index) if index <= upper_bound => Some(index),
        ChildIndex::At(_) => None,
    }
}

fn detach_preflighted_subtree(
    source: &mut Vec<RecordDefinition>,
    subtree_ids: &BTreeSet<ObjectId>,
    root: ObjectId,
    new_parent: Parent,
) -> Vec<RecordDefinition> {
    let mut subtree = Vec::with_capacity(subtree_ids.len());
    let mut remaining = Vec::with_capacity(source.len().saturating_sub(subtree_ids.len()));
    for mut record in std::mem::take(source) {
        if subtree_ids.contains(&record.id) {
            if record.id == root {
                let RecordSpec::Visual { parent, .. } = &mut record.spec else {
                    unreachable!("hierarchy operations preflight a visual root");
                };
                *parent = new_parent;
            }
            subtree.push(record);
        } else {
            remaining.push(record);
        }
    }
    *source = remaining;
    subtree
}

fn attach_preflighted_subtree(
    target: &mut Vec<RecordDefinition>,
    parent: Parent,
    final_index: usize,
    subtree: Vec<RecordDefinition>,
) {
    let siblings = target
        .iter()
        .filter_map(|record| match &record.spec {
            RecordSpec::Visual {
                parent: record_parent,
                ..
            } if *record_parent == parent => Some(record.id),
            _ => None,
        })
        .collect::<Vec<_>>();
    let insertion_index = siblings
        .get(final_index)
        .and_then(|sibling| target.iter().position(|node| node.id == *sibling))
        .unwrap_or(target.len());
    target.splice(insertion_index..insertion_index, subtree);
}

#[derive(Debug, Clone)]
struct RemovedRecord {
    original_index: usize,
    definition: RecordDefinition,
}

/// Opaque ownership token for an authored object and all of its descendants.
///
/// The token retains the original stable identities, parent relationships, and
/// authored order required by [`SceneTx::restore`]. Cloning a token is useful
/// for retrying an undo after a rejected transaction; restoring two copies is
/// rejected as an identity collision.
#[derive(Clone)]
pub struct RemovedSubtree {
    artboard: ArtboardId,
    root: ObjectId,
    records: Vec<RemovedRecord>,
}

#[derive(Debug, Clone, Copy)]
struct RuntimeSlot {
    local_id: usize,
    kind: NodeKind,
}

struct MaterializedArtboard {
    file: Arc<File>,
    objects: BTreeMap<ObjectId, RuntimeSlot>,
    animations: BTreeMap<AnimationId, usize>,
    machines: BTreeMap<MachineId, usize>,
    events_by_local: Vec<Option<EventId>>,
    objects_by_artboard_local: BTreeMap<ArtboardId, Vec<Option<ObjectId>>>,
    nested_artboard_targets: BTreeMap<ObjectId, ArtboardId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MountId(u64);

struct LiveInstance {
    id: InstanceId,
    artboard: ArtboardId,
    mount: MountId,
    runtime: OwnedArtboardInstance,
    machines: RetainedMachineInstances,
}

struct RetainedMachineInstances {
    ids: Vec<MachineId>,
    values: Vec<StateMachineInstance>,
}

impl RetainedMachineInstances {
    fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    fn get(&self, id: MachineId, index: usize) -> Option<&StateMachineInstance> {
        if self.ids.get(index) != Some(&id) {
            return None;
        }
        self.values.get(index)
    }

    fn get_mut(&mut self, id: MachineId, index: usize) -> Option<&mut StateMachineInstance> {
        if self.ids.get(index) != Some(&id) {
            return None;
        }
        self.values.get_mut(index)
    }
}

fn instantiate_runtime_mount(
    materialized: &MaterializedArtboard,
) -> std::result::Result<(OwnedArtboardInstance, RetainedMachineInstances), ()> {
    let runtime =
        OwnedArtboardInstance::instantiate(Arc::clone(&materialized.file), 0).map_err(|_| ())?;
    let mut machine_indices = materialized
        .machines
        .iter()
        .map(|(id, index)| (*id, *index))
        .collect::<Vec<_>>();
    machine_indices.sort_by_key(|(_, index)| *index);
    let mut ids = Vec::with_capacity(machine_indices.len());
    let mut values = Vec::with_capacity(machine_indices.len());
    for (expected_index, (id, index)) in machine_indices.into_iter().enumerate() {
        if index != expected_index {
            return Err(());
        }
        ids.push(id);
        values.push(runtime.state_machine_instance(index).ok_or(())?);
    }
    let machines = RetainedMachineInstances { ids, values };
    Ok((runtime, machines))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SceneId(u64);

struct SceneIdentity {
    id: SceneId,
}

static NEXT_SCENE_ID: AtomicU64 = AtomicU64::new(0);
static NEXT_ARTBOARD_ID: AtomicU64 = AtomicU64::new(0);
static NEXT_OBJECT_ID: AtomicU64 = AtomicU64::new(0);
static NEXT_FONT_ASSET_ID: AtomicU64 = AtomicU64::new(0);
static NEXT_IMAGE_ASSET_ID: AtomicU64 = AtomicU64::new(0);
static NEXT_SCRIPT_ASSET_ID: AtomicU64 = AtomicU64::new(0);
static NEXT_SHADER_ASSET_ID: AtomicU64 = AtomicU64::new(0);
static NEXT_INSTANCE_ID: AtomicU64 = AtomicU64::new(0);

/// Render resources retained for one mount of one live authored instance.
///
/// The wrapper detects a remount of its artboard (and accidental reuse with another scene or
/// instance) and recreates its underlying runtime cache during the next draw. Structural edits to
/// another artboard do not invalidate this cache. Failed candidate caches are discarded so a
/// decode error never poisons this cache or the authored scene.
pub struct SceneRenderCache {
    scene_identity: Arc<SceneIdentity>,
    instance: InstanceId,
    mount: MountId,
    inner: Option<ArtboardRenderCache>,
}

/// Schema-backed object kinds in the deterministic publish record stream.
///
/// The typed vocabulary is intentionally key-free: raw runtime type keys stay
/// behind the `Scene` implementation and the publish compiler maps these
/// variants through its own schema-generated vocabulary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportedObjectKind {
    Backboard,
    FontAsset,
    ImageAsset,
    ScriptAsset,
    ShaderAsset,
    FileAssetContents,
    Artboard,
    Shape,
    NestedArtboard,
    Image,
    ScriptedDrawable,
    Rectangle,
    Fill,
    SolidColor,
    Stroke,
    DashPath,
    Dash,
    Text,
    TextValueRun,
    TextStylePaint,
    Mesh,
    MeshVertex,
    LinearAnimation,
    KeyedObject,
    KeyedProperty,
    KeyFrameDouble,
    Event,
    StateMachine,
    StateMachineTrigger,
    StateMachineLayer,
    AnyState,
    EntryState,
    ExitState,
    AnimationState,
    StateTransition,
    TransitionTriggerCondition,
    StateMachineFireEvent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportedFillRule {
    NonZero,
}

/// Key-free semantic property selected by an authored animation track.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ExportedAnimatableProperty {
    PathWidth,
    WorldOpacity,
    TranslateX,
    TranslateY,
    Rotation,
    ScaleX,
    ScaleY,
}

impl ExportedAnimatableProperty {
    fn from_double_prop(property: Prop<f32>) -> Option<Self> {
        match property.key {
            PROPERTY_PATH_WIDTH => Some(Self::PathWidth),
            PROPERTY_WORLD_OPACITY => Some(Self::WorldOpacity),
            PROPERTY_TRANSLATE_X => Some(Self::TranslateX),
            PROPERTY_TRANSLATE_Y => Some(Self::TranslateY),
            PROPERTY_ROTATION => Some(Self::Rotation),
            PROPERTY_SCALE_X => Some(Self::ScaleX),
            PROPERTY_SCALE_Y => Some(Self::ScaleY),
            _ => None,
        }
    }

    const fn schema_key(self) -> u16 {
        match self {
            Self::PathWidth => PROPERTY_PATH_WIDTH,
            Self::WorldOpacity => PROPERTY_WORLD_OPACITY,
            Self::TranslateX => PROPERTY_TRANSLATE_X,
            Self::TranslateY => PROPERTY_TRANSLATE_Y,
            Self::Rotation => PROPERTY_ROTATION,
            Self::ScaleX => PROPERTY_SCALE_X,
            Self::ScaleY => PROPERTY_SCALE_Y,
        }
    }

    fn is_available_on(self, kind: NodeKind) -> bool {
        match self {
            Self::PathWidth => props::PATH_WIDTH.is_available_on(kind),
            Self::WorldOpacity => props::WORLD_OPACITY.is_available_on(kind),
            Self::TranslateX => props::TRANSLATE_X.is_available_on(kind),
            Self::TranslateY => props::TRANSLATE_Y.is_available_on(kind),
            Self::Rotation => props::ROTATION.is_available_on(kind),
            Self::ScaleX => props::SCALE_X.is_available_on(kind),
            Self::ScaleY => props::SCALE_Y.is_available_on(kind),
        }
    }
}

/// Typed properties in a deterministic exported scene record.
#[derive(Debug, Clone, PartialEq)]
pub enum ExportedProperty {
    ComponentName(String),
    AssetName(String),
    FileAssetId(u32),
    FileAssetContentsBytes(Vec<u8>),
    ParentId(u32),
    LayoutWidth(f32),
    LayoutHeight(f32),
    TranslateX(f32),
    TranslateY(f32),
    WorldOpacity(f32),
    Rotation(f32),
    ScaleX(f32),
    ScaleY(f32),
    NestedArtboardId(u32),
    ImageAssetId(u32),
    ImageOriginX(f32),
    ImageOriginY(f32),
    ImageFit(u32),
    ImageAlignmentX(f32),
    ImageAlignmentY(f32),
    ScriptAssetIsModule(bool),
    ScriptedDrawableScriptAssetId(u32),
    MeshTriangleIndexBytes(Vec<u8>),
    VertexX(f32),
    VertexY(f32),
    MeshVertexU(f32),
    MeshVertexV(f32),
    PathWidth(f32),
    PathHeight(f32),
    RectangleCornerRadiusTopLeft(f32),
    RectangleCornerRadiusTopRight(f32),
    RectangleCornerRadiusBottomRight(f32),
    RectangleCornerRadiusBottomLeft(f32),
    RectangleLinkCornerRadius(bool),
    FillRule(ExportedFillRule),
    ColorValue(u32),
    StrokeThickness(f32),
    StrokeCap(SceneStrokeCap),
    StrokeJoin(SceneStrokeJoin),
    StrokeTransformAffectsStroke(bool),
    DashOffset(f32),
    DashOffsetIsPercentage(bool),
    DashLength(f32),
    DashLengthIsPercentage(bool),
    TextSizing(SceneTextSizing),
    TextAlign(SceneTextAlign),
    TextWidth(f32),
    TextHeight(f32),
    TextWrap(SceneTextWrap),
    TextOverflow(SceneTextOverflow),
    TextValueRunText(String),
    TextValueRunStyleId(u32),
    TextStyleFontSize(f32),
    TextStyleLineHeight(f32),
    TextStyleLetterSpacing(f32),
    TextStyleFontAssetId(u32),
    AnimationName(String),
    AnimationFps(u32),
    AnimationDuration(u32),
    AnimationSpeed(f32),
    AnimationLoop(u32),
    AnimationWorkStart(u32),
    AnimationWorkEnd(u32),
    AnimationEnableWorkArea(bool),
    AnimationQuantize(bool),
    KeyedObjectId(u32),
    KeyedProperty(ExportedAnimatableProperty),
    KeyFrame(u32),
    KeyFrameInterpolationLinear,
    KeyFrameDoubleValue(f32),
    StateMachineComponentName(String),
    StateAnimationId(u32),
    StateSpeed(f32),
    StateToId(u32),
    StateTransitionFlags(u32),
    StateTransitionDuration(u32),
    StateTransitionExitTime(u32),
    StateTransitionRandomWeight(u32),
    StateMachineInputId(u32),
    EventId(u32),
    FireEventOccurs(FireEventOccurs),
}

impl ExportedProperty {
    fn schema_key(&self) -> u16 {
        match self {
            Self::ComponentName(_) => PROPERTY_COMPONENT_NAME,
            Self::AssetName(_) => PROPERTY_ASSET_NAME,
            Self::FileAssetId(_) => PROPERTY_FILE_ASSET_ID,
            Self::FileAssetContentsBytes(_) => PROPERTY_FILE_ASSET_CONTENTS_BYTES,
            Self::ParentId(_) => PROPERTY_PARENT_ID,
            Self::LayoutWidth(_) => PROPERTY_LAYOUT_WIDTH,
            Self::LayoutHeight(_) => PROPERTY_LAYOUT_HEIGHT,
            Self::TranslateX(_) => PROPERTY_TRANSLATE_X,
            Self::TranslateY(_) => PROPERTY_TRANSLATE_Y,
            Self::WorldOpacity(_) => PROPERTY_WORLD_OPACITY,
            Self::Rotation(_) => PROPERTY_ROTATION,
            Self::ScaleX(_) => PROPERTY_SCALE_X,
            Self::ScaleY(_) => PROPERTY_SCALE_Y,
            Self::NestedArtboardId(_) => PROPERTY_NESTED_ARTBOARD_ID,
            Self::ImageAssetId(_) => PROPERTY_IMAGE_ASSET_ID,
            Self::ImageOriginX(_) => PROPERTY_IMAGE_ORIGIN_X,
            Self::ImageOriginY(_) => PROPERTY_IMAGE_ORIGIN_Y,
            Self::ImageFit(_) => PROPERTY_IMAGE_FIT,
            Self::ImageAlignmentX(_) => PROPERTY_IMAGE_ALIGNMENT_X,
            Self::ImageAlignmentY(_) => PROPERTY_IMAGE_ALIGNMENT_Y,
            Self::ScriptAssetIsModule(_) => PROPERTY_SCRIPT_ASSET_IS_MODULE,
            Self::ScriptedDrawableScriptAssetId(_) => PROPERTY_SCRIPTED_DRAWABLE_SCRIPT_ASSET_ID,
            Self::MeshTriangleIndexBytes(_) => PROPERTY_MESH_TRIANGLE_INDEX_BYTES,
            Self::VertexX(_) => PROPERTY_VERTEX_X,
            Self::VertexY(_) => PROPERTY_VERTEX_Y,
            Self::MeshVertexU(_) => PROPERTY_MESH_VERTEX_U,
            Self::MeshVertexV(_) => PROPERTY_MESH_VERTEX_V,
            Self::PathWidth(_) => PROPERTY_PATH_WIDTH,
            Self::PathHeight(_) => PROPERTY_PATH_HEIGHT,
            Self::RectangleCornerRadiusTopLeft(_) => PROPERTY_RECTANGLE_CORNER_RADIUS_TL,
            Self::RectangleCornerRadiusTopRight(_) => PROPERTY_RECTANGLE_CORNER_RADIUS_TR,
            Self::RectangleCornerRadiusBottomRight(_) => PROPERTY_RECTANGLE_CORNER_RADIUS_BR,
            Self::RectangleCornerRadiusBottomLeft(_) => PROPERTY_RECTANGLE_CORNER_RADIUS_BL,
            Self::RectangleLinkCornerRadius(_) => PROPERTY_RECTANGLE_LINK_CORNER_RADIUS,
            Self::FillRule(_) => PROPERTY_FILL_RULE,
            Self::ColorValue(_) => PROPERTY_COLOR_VALUE,
            Self::StrokeThickness(_) => PROPERTY_STROKE_THICKNESS,
            Self::StrokeCap(_) => PROPERTY_STROKE_CAP,
            Self::StrokeJoin(_) => PROPERTY_STROKE_JOIN,
            Self::StrokeTransformAffectsStroke(_) => PROPERTY_STROKE_TRANSFORM_AFFECTS_STROKE,
            Self::DashOffset(_) => PROPERTY_DASH_OFFSET,
            Self::DashOffsetIsPercentage(_) => PROPERTY_DASH_OFFSET_IS_PERCENTAGE,
            Self::DashLength(_) => PROPERTY_DASH_LENGTH,
            Self::DashLengthIsPercentage(_) => PROPERTY_DASH_LENGTH_IS_PERCENTAGE,
            Self::TextSizing(_) => PROPERTY_TEXT_SIZING,
            Self::TextAlign(_) => PROPERTY_TEXT_ALIGN,
            Self::TextWidth(_) => PROPERTY_TEXT_WIDTH,
            Self::TextHeight(_) => PROPERTY_TEXT_HEIGHT,
            Self::TextWrap(_) => PROPERTY_TEXT_WRAP,
            Self::TextOverflow(_) => PROPERTY_TEXT_OVERFLOW,
            Self::TextValueRunText(_) => PROPERTY_TEXT_VALUE_RUN_TEXT,
            Self::TextValueRunStyleId(_) => PROPERTY_TEXT_VALUE_RUN_STYLE_ID,
            Self::TextStyleFontSize(_) => PROPERTY_TEXT_STYLE_FONT_SIZE,
            Self::TextStyleLineHeight(_) => PROPERTY_TEXT_STYLE_LINE_HEIGHT,
            Self::TextStyleLetterSpacing(_) => PROPERTY_TEXT_STYLE_LETTER_SPACING,
            Self::TextStyleFontAssetId(_) => PROPERTY_TEXT_STYLE_FONT_ASSET_ID,
            Self::AnimationName(_) => PROPERTY_ANIMATION_NAME,
            Self::AnimationFps(_) => PROPERTY_ANIMATION_FPS,
            Self::AnimationDuration(_) => PROPERTY_ANIMATION_DURATION,
            Self::AnimationSpeed(_) => PROPERTY_ANIMATION_SPEED,
            Self::AnimationLoop(_) => PROPERTY_ANIMATION_LOOP,
            Self::AnimationWorkStart(_) => PROPERTY_ANIMATION_WORK_START,
            Self::AnimationWorkEnd(_) => PROPERTY_ANIMATION_WORK_END,
            Self::AnimationEnableWorkArea(_) => PROPERTY_ANIMATION_ENABLE_WORK_AREA,
            Self::AnimationQuantize(_) => PROPERTY_ANIMATION_QUANTIZE,
            Self::KeyedObjectId(_) => PROPERTY_KEYED_OBJECT_ID,
            Self::KeyedProperty(_) => PROPERTY_KEYED_PROPERTY_KEY,
            Self::KeyFrame(_) => PROPERTY_KEY_FRAME,
            Self::KeyFrameInterpolationLinear => PROPERTY_KEY_FRAME_INTERPOLATION_TYPE,
            Self::KeyFrameDoubleValue(_) => PROPERTY_KEY_FRAME_DOUBLE_VALUE,
            Self::StateMachineComponentName(_) => PROPERTY_STATE_MACHINE_COMPONENT_NAME,
            Self::StateAnimationId(_) => PROPERTY_STATE_ANIMATION_ID,
            Self::StateSpeed(_) => PROPERTY_STATE_SPEED,
            Self::StateToId(_) => PROPERTY_STATE_TO_ID,
            Self::StateTransitionFlags(_) => PROPERTY_STATE_TRANSITION_FLAGS,
            Self::StateTransitionDuration(_) => PROPERTY_STATE_TRANSITION_DURATION,
            Self::StateTransitionExitTime(_) => PROPERTY_STATE_TRANSITION_EXIT_TIME,
            Self::StateTransitionRandomWeight(_) => PROPERTY_STATE_TRANSITION_RANDOM_WEIGHT,
            Self::StateMachineInputId(_) => PROPERTY_STATE_MACHINE_INPUT_ID,
            Self::EventId(_) => PROPERTY_STATE_MACHINE_EVENT_ID,
            Self::FireEventOccurs(_) => PROPERTY_STATE_MACHINE_FIRE_OCCURS,
        }
    }

    fn into_authoring_property(self) -> AuthoringProperty {
        let key = self.schema_key();
        let value = match self {
            Self::ComponentName(value)
            | Self::AssetName(value)
            | Self::TextValueRunText(value)
            | Self::AnimationName(value)
            | Self::StateMachineComponentName(value) => AuthoringValue::String(value),
            Self::FileAssetContentsBytes(value) | Self::MeshTriangleIndexBytes(value) => {
                AuthoringValue::Bytes(value)
            }
            Self::ParentId(value)
            | Self::FileAssetId(value)
            | Self::NestedArtboardId(value)
            | Self::ImageAssetId(value)
            | Self::ScriptedDrawableScriptAssetId(value)
            | Self::ImageFit(value)
            | Self::TextValueRunStyleId(value)
            | Self::TextStyleFontAssetId(value)
            | Self::AnimationFps(value)
            | Self::AnimationDuration(value)
            | Self::AnimationLoop(value)
            | Self::AnimationWorkStart(value)
            | Self::AnimationWorkEnd(value)
            | Self::KeyedObjectId(value)
            | Self::KeyFrame(value)
            | Self::StateAnimationId(value)
            | Self::StateToId(value)
            | Self::StateTransitionFlags(value)
            | Self::StateTransitionDuration(value)
            | Self::StateTransitionExitTime(value)
            | Self::StateTransitionRandomWeight(value)
            | Self::StateMachineInputId(value)
            | Self::EventId(value) => AuthoringValue::Uint(u64::from(value)),
            Self::FireEventOccurs(value) => AuthoringValue::Uint(match value {
                FireEventOccurs::AtStart => 0,
                FireEventOccurs::AtEnd => 1,
            }),
            Self::KeyedProperty(property) => AuthoringValue::Uint(u64::from(property.schema_key())),
            Self::KeyFrameInterpolationLinear => AuthoringValue::Uint(1),
            Self::FillRule(ExportedFillRule::NonZero) => AuthoringValue::Uint(0),
            Self::TextSizing(value) => AuthoringValue::Uint(u64::from(value.wire_value())),
            Self::TextAlign(value) => AuthoringValue::Uint(u64::from(value.wire_value())),
            Self::TextWrap(value) => AuthoringValue::Uint(u64::from(value.wire_value())),
            Self::TextOverflow(value) => AuthoringValue::Uint(u64::from(value.wire_value())),
            Self::StrokeCap(value) => AuthoringValue::Uint(u64::from(value.wire_value())),
            Self::StrokeJoin(value) => AuthoringValue::Uint(u64::from(value.wire_value())),
            Self::LayoutWidth(value)
            | Self::LayoutHeight(value)
            | Self::TranslateX(value)
            | Self::TranslateY(value)
            | Self::WorldOpacity(value)
            | Self::Rotation(value)
            | Self::ScaleX(value)
            | Self::ScaleY(value)
            | Self::ImageOriginX(value)
            | Self::ImageOriginY(value)
            | Self::ImageAlignmentX(value)
            | Self::ImageAlignmentY(value)
            | Self::VertexX(value)
            | Self::VertexY(value)
            | Self::MeshVertexU(value)
            | Self::MeshVertexV(value)
            | Self::PathWidth(value)
            | Self::PathHeight(value)
            | Self::RectangleCornerRadiusTopLeft(value)
            | Self::RectangleCornerRadiusTopRight(value)
            | Self::RectangleCornerRadiusBottomRight(value)
            | Self::RectangleCornerRadiusBottomLeft(value)
            | Self::StrokeThickness(value)
            | Self::DashOffset(value)
            | Self::DashLength(value)
            | Self::TextWidth(value)
            | Self::TextHeight(value)
            | Self::TextStyleFontSize(value)
            | Self::TextStyleLineHeight(value)
            | Self::TextStyleLetterSpacing(value)
            | Self::AnimationSpeed(value)
            | Self::StateSpeed(value)
            | Self::KeyFrameDoubleValue(value) => AuthoringValue::Double(value),
            Self::RectangleLinkCornerRadius(value)
            | Self::StrokeTransformAffectsStroke(value)
            | Self::DashOffsetIsPercentage(value)
            | Self::DashLengthIsPercentage(value)
            | Self::ScriptAssetIsModule(value)
            | Self::AnimationEnableWorkArea(value)
            | Self::AnimationQuantize(value) => AuthoringValue::Bool(value),
            Self::ColorValue(value) => AuthoringValue::Color(value),
        };
        AuthoringProperty { key, value }
    }
}

/// One key-free, schema-backed object record in an exported scene.
#[derive(Debug, Clone, PartialEq)]
pub struct ExportedRecord {
    pub kind: ExportedObjectKind,
    pub properties: Vec<ExportedProperty>,
}

impl ExportedRecord {
    fn into_authoring_record(self) -> AuthoringRecord {
        let type_key = match self.kind {
            ExportedObjectKind::Backboard => TYPE_BACKBOARD,
            ExportedObjectKind::FontAsset => TYPE_FONT_ASSET,
            ExportedObjectKind::ImageAsset => TYPE_IMAGE_ASSET,
            ExportedObjectKind::ScriptAsset => TYPE_SCRIPT_ASSET,
            ExportedObjectKind::ShaderAsset => TYPE_SHADER_ASSET,
            ExportedObjectKind::FileAssetContents => TYPE_FILE_ASSET_CONTENTS,
            ExportedObjectKind::Artboard => TYPE_ARTBOARD,
            ExportedObjectKind::Shape => TYPE_SHAPE,
            ExportedObjectKind::NestedArtboard => TYPE_NESTED_ARTBOARD,
            ExportedObjectKind::Image => TYPE_IMAGE,
            ExportedObjectKind::ScriptedDrawable => TYPE_SCRIPTED_DRAWABLE,
            ExportedObjectKind::Rectangle => TYPE_RECTANGLE,
            ExportedObjectKind::Fill => TYPE_FILL,
            ExportedObjectKind::SolidColor => TYPE_SOLID_COLOR,
            ExportedObjectKind::Stroke => TYPE_STROKE,
            ExportedObjectKind::DashPath => TYPE_DASH_PATH,
            ExportedObjectKind::Dash => TYPE_DASH,
            ExportedObjectKind::Text => TYPE_TEXT,
            ExportedObjectKind::TextValueRun => TYPE_TEXT_VALUE_RUN,
            ExportedObjectKind::TextStylePaint => TYPE_TEXT_STYLE_PAINT,
            ExportedObjectKind::Mesh => TYPE_MESH,
            ExportedObjectKind::MeshVertex => TYPE_MESH_VERTEX,
            ExportedObjectKind::LinearAnimation => TYPE_LINEAR_ANIMATION,
            ExportedObjectKind::KeyedObject => TYPE_KEYED_OBJECT,
            ExportedObjectKind::KeyedProperty => TYPE_KEYED_PROPERTY,
            ExportedObjectKind::KeyFrameDouble => TYPE_KEY_FRAME_DOUBLE,
            ExportedObjectKind::Event => TYPE_EVENT,
            ExportedObjectKind::StateMachine => TYPE_STATE_MACHINE,
            ExportedObjectKind::StateMachineTrigger => TYPE_STATE_MACHINE_TRIGGER,
            ExportedObjectKind::StateMachineLayer => TYPE_STATE_MACHINE_LAYER,
            ExportedObjectKind::AnyState => TYPE_ANY_STATE,
            ExportedObjectKind::EntryState => TYPE_ENTRY_STATE,
            ExportedObjectKind::ExitState => TYPE_EXIT_STATE,
            ExportedObjectKind::AnimationState => TYPE_ANIMATION_STATE,
            ExportedObjectKind::StateTransition => TYPE_STATE_TRANSITION,
            ExportedObjectKind::TransitionTriggerCondition => TYPE_TRANSITION_TRIGGER_CONDITION,
            ExportedObjectKind::StateMachineFireEvent => TYPE_STATE_MACHINE_FIRE_EVENT,
        };
        AuthoringRecord {
            type_key,
            properties: self
                .properties
                .into_iter()
                .map(ExportedProperty::into_authoring_property)
                .collect(),
        }
    }
}

/// Deterministic typed record form consumed by the publish encoder.
#[derive(Debug, Clone, PartialEq)]
pub struct ExportedDocument {
    records: Vec<ExportedRecord>,
}

impl ExportedDocument {
    pub fn records(&self) -> &[ExportedRecord] {
        &self.records
    }

    pub fn into_records(self) -> Vec<ExportedRecord> {
        self.records
    }

    fn into_authoring_records(self) -> Vec<AuthoringRecord> {
        self.records
            .into_iter()
            .map(ExportedRecord::into_authoring_record)
            .collect()
    }
}

/// An editable, owning Rive scene.
///
/// Structural edits rebuild only their touched artboards and publish all resulting mounts
/// atomically. Frame writes use cursors to mutate the already-instantiated runtime graph directly.
pub struct Scene {
    identity: Arc<SceneIdentity>,
    definitions: Definitions,
    materialized: BTreeMap<ArtboardId, MaterializedArtboard>,
    instances: Vec<Option<LiveInstance>>,
    epoch: StructureEpoch,
    next_mount_id: u64,
}

impl Default for Scene {
    fn default() -> Self {
        Self::new()
    }
}

impl Scene {
    pub fn new() -> Self {
        let Some(scene_id) = allocate_global_identity(&NEXT_SCENE_ID) else {
            // Scene identities are embedded in lifetime-free cursors. Reuse could turn an old
            // cursor into a write to a different scene, so exhaustion is process-fatal.
            std::process::abort();
        };
        Self {
            identity: Arc::new(SceneIdentity {
                id: SceneId(scene_id),
            }),
            definitions: Definitions::default(),
            materialized: BTreeMap::new(),
            instances: Vec::new(),
            epoch: StructureEpoch::INITIAL,
            next_mount_id: 0,
        }
    }

    pub const fn epoch(&self) -> StructureEpoch {
        self.epoch
    }

    /// Apply a structural edit as an atomic transaction.
    pub fn edit<R>(
        &mut self,
        edit: impl FnOnce(&mut SceneTx<'_>) -> std::result::Result<R, EditAbort>,
    ) -> std::result::Result<(R, EditReceipt), EditError> {
        let previous_artboards = self
            .definitions
            .artboards
            .iter()
            .map(|artboard| artboard.id)
            .collect::<BTreeSet<_>>();
        let mut definitions = self.definitions.clone();
        let (result, created_objects, touched_artboards, spec_origins, commit_operation_index) = {
            let definition_index = DefinitionIndex::build(&definitions);
            let mut transaction = SceneTx {
                definitions: &mut definitions,
                definition_index,
                next_operation_index: 0,
                created_objects: Vec::new(),
                touched_artboards: BTreeMap::new(),
                spec_origins: SpecOrigins::default(),
            };
            let result = edit(&mut transaction).map_err(EditError::aborted)?;
            let created_objects = transaction
                .created_objects
                .iter()
                .copied()
                .filter(|id| {
                    #[cfg(test)]
                    record_scene_work(|work| {
                        work.receipt_membership_checks =
                            work.receipt_membership_checks.saturating_add(1);
                    });
                    transaction.definition_index.contains_object(*id)
                })
                .collect();
            (
                result,
                created_objects,
                transaction.touched_artboards,
                transaction.spec_origins,
                transaction.next_operation_index,
            )
        };

        definitions
            .canonicalize_and_validate(commit_operation_index)
            .map_err(|abort| EditError::commit(abort.diagnostic))?;
        validate_animation_definitions(&definitions, commit_operation_index, &spec_origins)
            .map_err(EditError::commit)?;
        validate_machine_definitions(&definitions, commit_operation_index, &spec_origins)
            .map_err(EditError::commit)?;
        validate_font_assets(
            &definitions.font_assets,
            commit_operation_index,
            &spec_origins,
        )
        .map_err(EditError::commit)?;
        validate_image_assets(
            &definitions.image_assets,
            commit_operation_index,
            &spec_origins,
        )
        .map_err(EditError::commit)?;
        validate_script_assets(
            &definitions.script_assets,
            commit_operation_index,
            &spec_origins,
        )
        .map_err(EditError::commit)?;
        validate_shader_assets(
            &definitions.shader_assets,
            commit_operation_index,
            &spec_origins,
        )
        .map_err(EditError::commit)?;

        let final_artboards = definitions
            .artboards
            .iter()
            .map(|artboard| artboard.id)
            .collect::<BTreeSet<_>>();
        let removed_artboards = previous_artboards
            .difference(&final_artboards)
            .copied()
            .collect::<BTreeSet<_>>();
        let directly_dirty_artboards = final_artboards
            .iter()
            .copied()
            .filter(|artboard| {
                !previous_artboards.contains(artboard) || touched_artboards.contains_key(artboard)
            })
            .collect::<BTreeSet<_>>();
        let dirty_artboards = expand_dirty_artboards(&definitions, &directly_dirty_artboards);

        // Prepare every dirty surviving artboard before publishing any of them. A later failure
        // therefore cannot partially replace definitions, files, instances, mounts, or caches.
        let mut candidates = BTreeMap::new();
        for artboard in definitions
            .artboards
            .iter()
            .filter(|artboard| dirty_artboards.contains(&artboard.id))
        {
            let touched_operation_index = touched_artboards
                .get(&artboard.id)
                .copied()
                .unwrap_or(commit_operation_index);
            let materialized = MaterializedArtboard::build(
                &definitions,
                artboard.id,
                commit_operation_index,
                &spec_origins,
                touched_operation_index,
            )
            .map_err(EditError::commit)?;
            candidates.insert(artboard.id, materialized);
        }
        if candidates.len() != dirty_artboards.len() {
            return Err(EditError::commit(EditDiagnostic::new(
                commit_operation_index,
                dirty_artboards
                    .iter()
                    .copied()
                    .map(EditId::Artboard)
                    .collect(),
                EditReason::InternalInvariant,
            )));
        }
        if let Some(artboard) = final_artboards.iter().find(|artboard| {
            !dirty_artboards.contains(artboard) && !self.materialized.contains_key(artboard)
        }) {
            return Err(EditError::commit(EditDiagnostic::new(
                commit_operation_index,
                vec![EditId::Artboard(*artboard)],
                EditReason::InternalInvariant,
            )));
        }

        let epoch = self.epoch.next().ok_or_else(|| {
            EditError::commit(EditDiagnostic::new(
                commit_operation_index,
                Vec::new(),
                EditReason::EpochExhausted,
            ))
        })?;

        let mut next_mount_id = self.next_mount_id;
        let mut remounted = Vec::new();
        for (instance_slot, instance) in self.instances.iter().enumerate() {
            let Some(instance) = instance.as_ref() else {
                continue;
            };
            if removed_artboards.contains(&instance.artboard) {
                continue;
            }
            if !dirty_artboards.contains(&instance.artboard) {
                continue;
            }
            let touched_operation_index = touched_artboards
                .get(&instance.artboard)
                .copied()
                .unwrap_or(commit_operation_index);
            let involved_ids = vec![
                EditId::Artboard(instance.artboard),
                EditId::Instance(instance.id),
            ];
            let Some(materialized) = candidates.get(&instance.artboard) else {
                return Err(EditError::commit(EditDiagnostic::new(
                    touched_operation_index,
                    involved_ids,
                    EditReason::InternalInvariant,
                )));
            };
            let (runtime, machines) = instantiate_runtime_mount(materialized).map_err(|_| {
                EditError::commit(EditDiagnostic::new(
                    touched_operation_index,
                    involved_ids.clone(),
                    EditReason::InternalInvariant,
                ))
            })?;
            let mount = MountId(allocate_identity(&mut next_mount_id).ok_or_else(|| {
                EditError::commit(EditDiagnostic::new(
                    touched_operation_index,
                    involved_ids,
                    EditReason::IdentityExhausted,
                ))
            })?);
            remounted.push((
                instance_slot,
                LiveInstance {
                    id: instance.id,
                    artboard: instance.artboard,
                    mount,
                    runtime,
                    machines,
                },
            ));
        }

        // No operation below this point is fallible: publish the prepared transaction atomically.
        let mut replacements: BTreeMap<_, _> = remounted.into_iter().collect();
        let instances = std::mem::take(&mut self.instances)
            .into_iter()
            .enumerate()
            .map(|(instance_slot, instance)| {
                if instance
                    .as_ref()
                    .is_some_and(|live| removed_artboards.contains(&live.artboard))
                {
                    return None;
                }
                replacements
                    .remove(&instance_slot)
                    .map(Some)
                    .unwrap_or(instance)
            })
            .collect();
        debug_assert!(replacements.is_empty());
        self.definitions = definitions;
        self.materialized
            .retain(|artboard, _| final_artboards.contains(artboard));
        for (artboard, materialized) in candidates {
            self.materialized.insert(artboard, materialized);
        }
        self.instances = instances;
        self.next_mount_id = next_mount_id;
        self.epoch = epoch;
        Ok((
            result,
            EditReceipt {
                epoch,
                created: created_objects,
            },
        ))
    }

    pub fn instantiate(
        &mut self,
        artboard: ArtboardId,
    ) -> std::result::Result<InstanceId, InstanceError> {
        let materialized = self
            .materialized
            .get(&artboard)
            .ok_or(InstanceError::UnknownArtboard)?;
        let (runtime, machines) =
            instantiate_runtime_mount(materialized).map_err(|_| InstanceError::RuntimeRejected)?;
        let id = InstanceId(
            allocate_global_identity(&NEXT_INSTANCE_ID).ok_or(InstanceError::IdentityExhausted)?,
        );
        let mount = MountId(
            allocate_identity(&mut self.next_mount_id).ok_or(InstanceError::IdentityExhausted)?,
        );
        let live = LiveInstance {
            id,
            artboard,
            mount,
            runtime,
            machines,
        };
        if let Some(vacant) = self.instances.iter_mut().find(|slot| slot.is_none()) {
            *vacant = Some(live);
        } else {
            self.instances.push(Some(live));
        }
        Ok(id)
    }

    /// Drop a live instance without changing authored definitions or their epoch.
    ///
    /// Vacated storage may be reused by a later instance, but the never-reused
    /// [`InstanceId`] embedded in cursors prevents an old cursor from aliasing it.
    pub fn drop_instance(&mut self, instance: InstanceId) {
        if let Some(slot) = self.instances.iter_mut().find(|slot| {
            slot.as_ref()
                .is_some_and(|candidate| candidate.id == instance)
        }) {
            *slot = None;
        }
    }

    pub fn cursor<T>(
        &self,
        instance: InstanceId,
        object: ObjectId,
        property: Prop<T>,
    ) -> std::result::Result<Cursor<T>, ResolveError> {
        let (instance_slot, live) = self
            .instances
            .iter()
            .enumerate()
            .find_map(|(slot, candidate)| {
                candidate
                    .as_ref()
                    .filter(|candidate| candidate.id == instance)
                    .map(|candidate| (slot, candidate))
            })
            .ok_or(ResolveError::UnknownInstance)?;
        let slot = self
            .materialized
            .iter()
            .find_map(|(artboard, materialized)| {
                materialized
                    .objects
                    .get(&object)
                    .map(|slot| (*artboard, slot))
            });
        let Some((slot_artboard, slot)) = slot else {
            let known_nonvisual = self.definitions.artboards.iter().any(|artboard| {
                artboard.records.iter().any(|record| {
                    record.id == object && !matches!(record.spec, RecordSpec::Visual { .. })
                })
            });
            return Err(if known_nonvisual {
                ResolveError::NonVisualObject
            } else {
                ResolveError::UnknownObject
            });
        };
        if live.artboard != slot_artboard {
            return Err(ResolveError::DifferentArtboard);
        }
        if !property.is_available_on(slot.kind) {
            return Err(ResolveError::UnsupportedProperty);
        }
        Ok(Cursor {
            scene: self.identity.id,
            epoch: self.epoch,
            instance_slot,
            instance,
            local_id: slot.local_id,
            property,
        })
    }

    /// Resolve one named trigger input on a retained machine instance.
    pub fn machine_input(
        &self,
        instance: InstanceId,
        machine: MachineId,
        name: &str,
    ) -> std::result::Result<InputCursor, ResolveError> {
        let (instance_slot, live) = self
            .instances
            .iter()
            .enumerate()
            .find_map(|(slot, candidate)| {
                candidate
                    .as_ref()
                    .filter(|candidate| candidate.id == instance)
                    .map(|candidate| (slot, candidate))
            })
            .ok_or(ResolveError::UnknownInstance)?;
        let materialized = self
            .materialized
            .get(&live.artboard)
            .ok_or(ResolveError::UnknownMachine)?;
        let Some(machine_index) = materialized.machines.get(&machine).copied() else {
            return Err(
                if self
                    .materialized
                    .values()
                    .any(|candidate| candidate.machines.contains_key(&machine))
                {
                    ResolveError::DifferentArtboard
                } else {
                    ResolveError::UnknownMachine
                },
            );
        };
        let retained = live
            .machines
            .get(machine, machine_index)
            .ok_or(ResolveError::UnknownMachine)?;
        let input_index = retained
            .input_index_named(name)
            .ok_or(ResolveError::UnknownMachineInput)?;
        let input_kind = retained
            .input(input_index)
            .map(|input| input.kind())
            .ok_or(ResolveError::UnknownMachineInput)?;
        if input_kind != StateMachineInputKind::Trigger {
            return Err(ResolveError::UnsupportedInputKind);
        }
        Ok(InputCursor {
            scene: self.identity.id,
            epoch: self.epoch,
            instance_slot,
            instance,
            machine,
            machine_index,
            input_index,
            input_kind,
        })
    }

    /// Create a renderer-neutral cache handle for one authored instance.
    ///
    /// Render resources, including lazily decoded images, are allocated on the first
    /// [`Frame::draw`]. This keeps decode failures at the draw interface and permits retrying
    /// with the same scene and cache after a recoverable adapter failure.
    pub fn new_render_cache(
        &self,
        instance: InstanceId,
    ) -> std::result::Result<SceneRenderCache, ResolveError> {
        let live = self
            .instances
            .iter()
            .filter_map(Option::as_ref)
            .find(|candidate| candidate.id == instance)
            .ok_or(ResolveError::UnknownInstance)?;
        Ok(SceneRenderCache {
            scene_identity: Arc::clone(&self.identity),
            instance,
            mount: live.mount,
            inner: None,
        })
    }

    pub fn frame(&mut self) -> Frame<'_> {
        Frame { scene: self }
    }

    /// Export one canonical record stream with one Backboard and every authored artboard.
    ///
    /// Export reads authored definitions, not ephemeral instance values written through
    /// [`Frame::set`]. Clients replay those values after a structural remount when needed.
    /// Referenced fonts/images are followed by all scripts and all shaders in canonical asset
    /// phases. One dense record-local file-asset ordinal spans the phases, and every asset record
    /// is adjacent to its contents record.
    pub fn export_records(&self) -> ExportedDocument {
        let mut records = vec![backboard_record()];
        let origins = SpecOrigins::default();
        let all_artboards = self.definitions.artboards.iter().collect::<Vec<_>>();
        let referenced_assets = match CanonicalFileAssets::collect(
            &self.definitions.font_assets,
            &self.definitions.image_assets,
            &self.definitions.script_assets,
            &self.definitions.shader_assets,
            all_artboards.as_slice(),
        )
        .lower(0, &origins)
        {
            Ok(lowered) => lowered,
            Err(_) => std::process::abort(),
        };
        records.extend(referenced_assets.records);
        let artboard_indices = match artboard_indices(all_artboards.as_slice()) {
            Ok(indices) => indices,
            Err(_) => std::process::abort(),
        };
        for artboard in &self.definitions.artboards {
            let lowered = match lower_artboard(
                artboard,
                &referenced_assets.font_indices,
                &referenced_assets.image_indices,
                &referenced_assets.script_indices,
                &artboard_indices,
                0,
                &origins,
            ) {
                Ok(lowered) => lowered,
                Err(_) => {
                    // Committed definitions have already passed this exact lowering path.
                    // Export must never return a partial stream if that invariant is broken.
                    std::process::abort();
                }
            };
            records.extend(lowered.records);
        }
        ExportedDocument { records }
    }
}

/// Mutable structural transaction over a scene's durable definitions.
pub struct SceneTx<'a> {
    definitions: &'a mut Definitions,
    definition_index: DefinitionIndex,
    next_operation_index: usize,
    created_objects: Vec<ObjectId>,
    touched_artboards: BTreeMap<ArtboardId, usize>,
    spec_origins: SpecOrigins,
}

impl SceneTx<'_> {
    /// Enter the animation vocabulary over the same durable record store and
    /// ordinary [`ObjectId`] identity space as visual authoring.
    pub fn animations(&mut self) -> AnimTx<'_> {
        AnimTx {
            definitions: self.definitions,
            definition_index: &mut self.definition_index,
            next_operation_index: &mut self.next_operation_index,
            created_objects: &mut self.created_objects,
            touched_artboards: &mut self.touched_artboards,
            spec_origins: &mut self.spec_origins,
        }
    }

    /// Enter the state-machine vocabulary over the same durable record store
    /// and ordinary [`ObjectId`] identity space as visual authoring.
    pub fn machines(&mut self) -> MachineTx<'_> {
        MachineTx {
            definitions: self.definitions,
            definition_index: &mut self.definition_index,
            next_operation_index: &mut self.next_operation_index,
            created_objects: &mut self.created_objects,
            touched_artboards: &mut self.touched_artboards,
            spec_origins: &mut self.spec_origins,
        }
    }

    /// Add one embedded font to the scene and return its stable semantic identity.
    ///
    /// Each call creates a distinct asset. Callers retain and reuse the returned
    /// identity when multiple text styles share one font. The asset remains part
    /// of the scene's durable definitions across later edits, even while no style
    /// references it. Runtime files and export records omit it until it is referenced.
    ///
    /// Adding the durable definition alone does not touch any artboard: runtime
    /// files project only fonts referenced by that artboard's current text
    /// styles. The structural edit that first creates such a reference touches
    /// and remounts its owning artboard through the ordinary node path.
    pub fn create_font_asset(
        &mut self,
        spec: FontAssetSpec,
    ) -> std::result::Result<FontAssetId, EditAbort> {
        let operation_index = self.begin_operation()?;
        let id = FontAssetId(
            allocate_global_identity(&NEXT_FONT_ASSET_ID).ok_or_else(|| {
                EditAbort::new(operation_index, Vec::new(), EditReason::IdentityExhausted)
            })?,
        );
        let font_index = self.definitions.font_assets.len();
        self.definitions
            .font_assets
            .push(FontAssetDefinition { id, spec });
        self.definition_index.font_assets.insert(id, font_index);
        self.spec_origins.font_assets.insert(id, operation_index);
        Ok(id)
    }

    /// Add one embedded image to the scene and return its stable semantic identity.
    ///
    /// Each call creates a distinct asset. Callers retain and reuse the returned
    /// identity when multiple image nodes share one source. The asset remains
    /// part of the scene's durable definitions across later edits, even while no
    /// image node references it. Runtime files and export records omit it until
    /// it is referenced.
    pub fn create_image_asset(
        &mut self,
        spec: ImageAssetSpec,
    ) -> std::result::Result<ImageAssetId, EditAbort> {
        let operation_index = self.begin_operation()?;
        let id = ImageAssetId(
            allocate_global_identity(&NEXT_IMAGE_ASSET_ID).ok_or_else(|| {
                EditAbort::new(operation_index, Vec::new(), EditReason::IdentityExhausted)
            })?,
        );
        let image_index = self.definitions.image_assets.len();
        self.definitions
            .image_assets
            .push(ImageAssetDefinition { id, spec });
        self.definition_index.image_assets.insert(id, image_index);
        self.spec_origins.image_assets.insert(id, operation_index);
        Ok(id)
    }

    /// Add raw compiled Luau bytecode and return its stable semantic identity.
    ///
    /// `spec.bytes` is the compiler payload, not Rive's signed-content framing.
    /// Scene lowering adds one unsigned version-zero envelope. Script assets are
    /// emitted into every materialized file so by-name module dependencies remain
    /// available even when they are not referenced by a retained node. Rive's
    /// module namespace is file-global and order-sensitive. Scene preserves
    /// empty payloads, empty names, and duplicate names without validating Luau;
    /// the compiler boundary must provide valid bytecode and unambiguous runtime
    /// names when scripts use by-name lookup.
    pub fn create_script_asset(
        &mut self,
        spec: ScriptAssetSpec,
    ) -> std::result::Result<ScriptAssetId, EditAbort> {
        let operation_index = self.begin_operation()?;
        let id = ScriptAssetId(allocate_global_identity(&NEXT_SCRIPT_ASSET_ID).ok_or_else(
            || EditAbort::new(operation_index, Vec::new(), EditReason::IdentityExhausted),
        )?);
        let script_index = self.definitions.script_assets.len();
        self.definitions
            .script_assets
            .push(ScriptAssetDefinition { id, spec });
        self.definition_index.script_assets.insert(id, script_index);
        self.spec_origins.script_assets.insert(id, operation_index);
        for artboard in &self.definitions.artboards {
            self.touched_artboards.insert(artboard.id, operation_index);
        }
        Ok(id)
    }

    /// Add raw compiled RSTB shader bytes and return their stable semantic identity.
    ///
    /// `spec.bytes` excludes Rive's signed-content framing. Scene lowering adds
    /// one unsigned version-zero envelope and emits every shader because script
    /// shader lookup is dynamic and name based. Scene preserves empty payloads,
    /// empty names, and duplicate names without validating RSTB. Native lookup
    /// returns the first file-global name match, so the compiler boundary must
    /// provide valid bytes and unambiguous runtime names.
    pub fn create_shader_asset(
        &mut self,
        spec: ShaderAssetSpec,
    ) -> std::result::Result<ShaderAssetId, EditAbort> {
        let operation_index = self.begin_operation()?;
        let id = ShaderAssetId(allocate_global_identity(&NEXT_SHADER_ASSET_ID).ok_or_else(
            || EditAbort::new(operation_index, Vec::new(), EditReason::IdentityExhausted),
        )?);
        let shader_index = self.definitions.shader_assets.len();
        self.definitions
            .shader_assets
            .push(ShaderAssetDefinition { id, spec });
        self.definition_index.shader_assets.insert(id, shader_index);
        self.spec_origins.shader_assets.insert(id, operation_index);
        for artboard in &self.definitions.artboards {
            self.touched_artboards.insert(artboard.id, operation_index);
        }
        Ok(id)
    }

    pub fn create_artboard(
        &mut self,
        spec: ArtboardSpec,
    ) -> std::result::Result<ArtboardId, EditAbort> {
        let operation_index = self.begin_operation()?;
        let id = ArtboardId(allocate_global_identity(&NEXT_ARTBOARD_ID).ok_or_else(|| {
            EditAbort::new(operation_index, Vec::new(), EditReason::IdentityExhausted)
        })?);
        let artboard_index = self.definitions.artboards.len();
        self.definitions.artboards.push(ArtboardDefinition {
            id,
            spec,
            records: Vec::new(),
        });
        self.definition_index.artboards.insert(id, artboard_index);
        self.definition_index
            .children
            .entry(Parent::Artboard(id))
            .or_default();
        self.touched_artboards.insert(id, operation_index);
        self.spec_origins.artboard_specs.insert(id, operation_index);
        Ok(id)
    }

    /// Replace one authored artboard's typed definition while retaining its
    /// stable identity and live instance identities.
    pub fn set_artboard(
        &mut self,
        artboard: ArtboardId,
        spec: ArtboardSpec,
    ) -> std::result::Result<(), EditAbort> {
        let operation_index = self.begin_operation()?;
        Hierarchy {
            definitions: self.definitions,
            index: &self.definition_index,
            operation_index,
        }
        .set_artboard(artboard, spec)?;
        self.touched_artboards.insert(artboard, operation_index);
        self.spec_origins
            .artboard_specs
            .insert(artboard, operation_index);
        Ok(())
    }

    /// Change only the deterministic definition/export order of authored
    /// artboards. Per-artboard runtime mounts are intentionally untouched.
    pub fn reorder_artboard(
        &mut self,
        artboard: ArtboardId,
        index: ChildIndex,
    ) -> std::result::Result<(), EditAbort> {
        let operation_index = self.begin_operation()?;
        Hierarchy {
            definitions: self.definitions,
            index: &self.definition_index,
            operation_index,
        }
        .reorder_artboard(artboard, index)?;
        self.refresh_definition_index();
        Ok(())
    }

    /// Remove one authored artboard and all of its authored objects. Live
    /// instances of the artboard are discarded atomically at publication.
    pub fn remove_artboard(&mut self, artboard: ArtboardId) -> std::result::Result<(), EditAbort> {
        let operation_index = self.begin_operation()?;
        Hierarchy {
            definitions: self.definitions,
            index: &self.definition_index,
            operation_index,
        }
        .remove_artboard(artboard)?;
        self.refresh_definition_index();
        self.touched_artboards.insert(artboard, operation_index);
        Ok(())
    }

    /// Remove every authored object from one artboard while retaining the
    /// artboard and its stable identity. This is the scoped replacement seam:
    /// callers can clear once and append a complete typed replacement without
    /// issuing one structural remove per old root.
    pub fn clear_artboard(&mut self, artboard: ArtboardId) -> std::result::Result<(), EditAbort> {
        let operation_index = self.begin_operation()?;
        Hierarchy {
            definitions: self.definitions,
            index: &self.definition_index,
            operation_index,
        }
        .clear_artboard(artboard)?;
        self.refresh_definition_index();
        self.touched_artboards.insert(artboard, operation_index);
        Ok(())
    }

    pub fn create(
        &mut self,
        parent: Parent,
        spec: NodeSpec,
    ) -> std::result::Result<ObjectId, EditAbort> {
        let operation_index = self.begin_operation()?;
        let kind = spec.kind();
        let artboard_id = self
            .definition_index
            .validate_parent(operation_index, parent, kind)?;
        let id = ObjectId(allocate_global_identity(&NEXT_OBJECT_ID).ok_or_else(|| {
            EditAbort::new(
                operation_index,
                parent_edit_ids(parent),
                EditReason::IdentityExhausted,
            )
        })?);
        let artboard_index = *self
            .definition_index
            .artboards
            .get(&artboard_id)
            .ok_or_else(|| {
                EditAbort::new(
                    operation_index,
                    vec![EditId::Artboard(artboard_id)],
                    EditReason::InternalInvariant,
                )
            })?;
        let artboard = self
            .definitions
            .artboards
            .get_mut(artboard_index)
            .ok_or_else(|| {
                EditAbort::new(
                    operation_index,
                    vec![EditId::Artboard(artboard_id)],
                    EditReason::InternalInvariant,
                )
            })?;
        let record_index = artboard.records.len();
        artboard.records.push(RecordDefinition {
            id,
            spec: RecordSpec::Visual { parent, node: spec },
        });
        self.definition_index.objects.insert(
            id,
            IndexedObject {
                artboard: artboard_id,
                artboard_index,
                record_index,
                kind: AuthoredObjectKind::Visual(kind),
            },
        );
        self.definition_index.owned.entry(id).or_default();
        if let Parent::Object(owner) = parent {
            self.definition_index
                .owned
                .entry(owner)
                .or_default()
                .push(id);
        }
        self.definition_index
            .children
            .entry(parent)
            .or_default()
            .push(id);
        self.definition_index
            .children
            .entry(Parent::Object(id))
            .or_default();
        self.created_objects.push(id);
        self.touched_artboards.insert(artboard_id, operation_index);
        self.spec_origins.nodes.insert(id, operation_index);
        Ok(id)
    }

    /// Move an authored object subtree to an exact final position among its
    /// current parent's children.
    pub fn reorder(
        &mut self,
        object: ObjectId,
        index: ChildIndex,
    ) -> std::result::Result<(), EditAbort> {
        let operation_index = self.begin_operation()?;
        let artboard_id = Hierarchy {
            definitions: self.definitions,
            index: &self.definition_index,
            operation_index,
        }
        .reorder(object, index)?;
        self.refresh_definition_index();
        self.touched_artboards.insert(artboard_id, operation_index);
        Ok(())
    }

    /// Set the exact final order of every direct child owned by one parent.
    ///
    /// The order must contain the parent's complete current child set exactly
    /// once. Validation is atomic, complete child subtrees move as stable
    /// blocks, and the transaction index is rebuilt once after mutation.
    pub fn set_child_order(
        &mut self,
        parent: Parent,
        ordered_children: &[ObjectId],
    ) -> std::result::Result<(), EditAbort> {
        let operation_index = self.begin_operation()?;
        let artboard_id = Hierarchy {
            definitions: self.definitions,
            index: &self.definition_index,
            operation_index,
        }
        .set_child_order(parent, ordered_children)?;
        self.refresh_definition_index();
        self.touched_artboards.insert(artboard_id, operation_index);
        Ok(())
    }

    /// Move an authored object subtree under a new parent at an exact final
    /// sibling position. Stable object identities are retained.
    pub fn reparent(
        &mut self,
        object: ObjectId,
        new_parent: Parent,
        index: ChildIndex,
    ) -> std::result::Result<(), EditAbort> {
        let operation_index = self.begin_operation()?;
        let (source, target) = Hierarchy {
            definitions: self.definitions,
            index: &self.definition_index,
            operation_index,
        }
        .reparent(object, new_parent, index)?;
        self.refresh_definition_index();
        self.spec_origins
            .relationships
            .insert(object, operation_index);
        self.touched_artboards.insert(source, operation_index);
        self.touched_artboards.insert(target, operation_index);
        Ok(())
    }

    /// Remove an object and its complete descendant subtree.
    ///
    /// The returned token owns everything needed to restore the same authored
    /// identities and ordering in this or a later transaction.
    pub fn remove(&mut self, object: ObjectId) -> std::result::Result<RemovedSubtree, EditAbort> {
        let operation_index = self.begin_operation()?;
        let removed = Hierarchy {
            definitions: self.definitions,
            index: &self.definition_index,
            operation_index,
        }
        .detach_subtree(object)?;
        self.refresh_definition_index();
        for removed_record in &removed.records {
            self.spec_origins
                .relationships
                .insert(removed_record.definition.id, operation_index);
        }
        self.touched_artboards
            .insert(removed.artboard, operation_index);
        Ok(removed)
    }

    /// Restore a previously removed subtree without allocating new identities.
    pub fn restore(&mut self, removed: RemovedSubtree) -> std::result::Result<ObjectId, EditAbort> {
        let operation_index = self.begin_operation()?;
        let (artboard_id, root, restored) = Hierarchy {
            definitions: self.definitions,
            index: &self.definition_index,
            operation_index,
        }
        .attach_subtree(removed)?;
        self.refresh_definition_index();
        for id in restored {
            self.spec_origins.nodes.insert(id, operation_index);
        }
        self.touched_artboards.insert(artboard_id, operation_index);
        Ok(root)
    }

    pub fn set<T>(
        &mut self,
        object: ObjectId,
        property: Prop<T>,
        value: T,
    ) -> std::result::Result<(), EditAbort> {
        let operation_index = self.begin_operation()?;
        let property_name = property.schema_name();
        let artboard_id = Hierarchy {
            definitions: self.definitions,
            index: &self.definition_index,
            operation_index,
        }
        .set(object, property, value)?;
        self.spec_origins
            .properties
            .insert((object, property_name), operation_index);
        self.touched_artboards.insert(artboard_id, operation_index);
        Ok(())
    }

    /// Explicitly abort the transaction while retaining operation diagnostics.
    pub fn abort(&mut self, message: impl Into<String>) -> EditAbort {
        match self.begin_operation() {
            Ok(operation_index) => EditAbort::new(
                operation_index,
                Vec::new(),
                EditReason::Requested {
                    message: message.into(),
                },
            ),
            Err(abort) => abort,
        }
    }

    fn begin_operation(&mut self) -> std::result::Result<usize, EditAbort> {
        let operation_index = self.next_operation_index;
        let Some(next) = operation_index.checked_add(1) else {
            return Err(EditAbort::new(
                operation_index,
                Vec::new(),
                EditReason::OperationLimitExceeded,
            ));
        };
        self.next_operation_index = next;
        Ok(operation_index)
    }

    fn refresh_definition_index(&mut self) {
        self.definition_index.rebuild(self.definitions);
        self.spec_origins
            .artboard_specs
            .retain(|id, _| self.definition_index.artboards.contains_key(id));
        self.spec_origins
            .nodes
            .retain(|id, _| self.definition_index.contains_object(*id));
        self.spec_origins
            .properties
            .retain(|(id, _), _| self.definition_index.contains_object(*id));
    }
}

/// Invariant-enforcing animation vocabulary over a [`SceneTx`].
///
/// This is not a separate graph. Every record created here has an ordinary
/// [`ObjectId`] and is committed, removed, restored, exported, and
/// materialized by the same scene transaction.
pub struct AnimTx<'a> {
    definitions: &'a mut Definitions,
    definition_index: &'a mut DefinitionIndex,
    next_operation_index: &'a mut usize,
    created_objects: &'a mut Vec<ObjectId>,
    touched_artboards: &'a mut BTreeMap<ArtboardId, usize>,
    spec_origins: &'a mut SpecOrigins,
}

impl AnimTx<'_> {
    /// Create one linear timeline owned by `artboard`.
    pub fn create_linear(
        &mut self,
        artboard: ArtboardId,
        spec: LinearAnimationSpec,
    ) -> std::result::Result<AnimationId, EditAbort> {
        let operation_index = self.begin_operation()?;
        let artboard_index = self
            .definition_index
            .artboards
            .get(&artboard)
            .copied()
            .ok_or_else(|| {
                EditAbort::new(
                    operation_index,
                    vec![EditId::Artboard(artboard)],
                    EditReason::UnknownArtboard,
                )
            })?;
        validate_linear_animation_spec(&spec).map_err(|reason| {
            EditAbort::new(operation_index, vec![EditId::Artboard(artboard)], reason)
        })?;
        let id = ObjectId(allocate_global_identity(&NEXT_OBJECT_ID).ok_or_else(|| {
            EditAbort::new(operation_index, Vec::new(), EditReason::IdentityExhausted)
        })?);
        let definition = self
            .definitions
            .artboards
            .get_mut(artboard_index)
            .ok_or_else(|| {
                EditAbort::new(
                    operation_index,
                    vec![EditId::Artboard(artboard)],
                    EditReason::InternalInvariant,
                )
            })?;
        let record_index = definition.records.len();
        definition.records.push(RecordDefinition {
            id,
            spec: RecordSpec::Animation(AnimationRecordSpec::LinearAnimation(spec)),
        });
        self.definition_index.objects.insert(
            id,
            IndexedObject {
                artboard,
                artboard_index,
                record_index,
                kind: AuthoredObjectKind::LinearAnimation,
            },
        );
        self.definition_index.owned.entry(id).or_default();
        self.created_objects.push(id);
        self.touched_artboards.insert(artboard, operation_index);
        self.spec_origins.nodes.insert(id, operation_index);
        Ok(AnimationId(id))
    }

    /// Upsert one linearly interpolated f32 key at `(target, property, frame)`.
    ///
    /// Missing keyed-object/property records are created automatically. The
    /// returned identity is the ordinary [`ObjectId`] of the keyframe record;
    /// setting the same frame again preserves that identity.
    pub fn set_key(
        &mut self,
        animation: AnimationId,
        target: ObjectId,
        property: Prop<f32>,
        frame: u32,
        value: f32,
    ) -> std::result::Result<ObjectId, EditAbort> {
        let operation_index = self.begin_operation()?;
        if !value.is_finite() {
            return Err(EditAbort::new(
                operation_index,
                vec![EditId::Object(animation.object_id())],
                EditReason::NonFiniteProperty {
                    property: "key_frame_value",
                },
            ));
        }
        let Some(animation_location) = self
            .definition_index
            .objects
            .get(&animation.object_id())
            .copied()
        else {
            return Err(EditAbort::new(
                operation_index,
                vec![EditId::Object(animation.object_id())],
                EditReason::UnknownObject,
            ));
        };
        if animation_location.kind != AuthoredObjectKind::LinearAnimation {
            return Err(EditAbort::new(
                operation_index,
                vec![EditId::Object(animation.object_id())],
                EditReason::RecordPropertyOwnerMismatch {
                    property: "animation",
                    actual: animation_location.kind,
                },
            ));
        }
        let target_index = self
            .definition_index
            .objects
            .get(&target)
            .copied()
            .ok_or_else(|| {
                EditAbort::new(
                    operation_index,
                    vec![EditId::Object(target)],
                    EditReason::UnknownObject,
                )
            })?;
        let target_kind = target_index.visual_kind().ok_or_else(|| {
            EditAbort::new(
                operation_index,
                vec![EditId::Object(target)],
                EditReason::NonVisualObject,
            )
        })?;
        if target_index.artboard != animation_location.artboard {
            return Err(EditAbort::new(
                operation_index,
                vec![
                    EditId::Object(animation.object_id()),
                    EditId::Object(target),
                ],
                EditReason::CrossArtboardReference {
                    source: animation_location.artboard,
                    target: target_index.artboard,
                },
            ));
        }
        if !property.is_available_on(target_kind) {
            return Err(EditAbort::new(
                operation_index,
                vec![EditId::Object(target)],
                EditReason::PropertyOwnerMismatch {
                    property: property.schema_name,
                    actual: target_kind,
                },
            ));
        }
        let semantic_property =
            ExportedAnimatableProperty::from_double_prop(property).ok_or_else(|| {
                EditAbort::new(
                    operation_index,
                    vec![EditId::Object(target)],
                    EditReason::InternalInvariant,
                )
            })?;

        let artboard = animation_location.artboard;
        let artboard_index = target_index.artboard_index;
        let (keyed_object_id, keyed_property_id, existing_key_frame_id) = {
            let authored_artboard =
                self.definitions
                    .artboards
                    .get(artboard_index)
                    .ok_or_else(|| {
                        EditAbort::new(
                            operation_index,
                            vec![EditId::Artboard(artboard)],
                            EditReason::InternalInvariant,
                        )
                    })?;
            let authored_animation = authored_artboard
                .records
                .get(animation_location.record_index)
                .filter(|record| record.id == animation.object_id())
                .ok_or_else(|| {
                    EditAbort::new(
                        operation_index,
                        vec![EditId::Object(animation.object_id())],
                        EditReason::InternalInvariant,
                    )
                })?;
            if !matches!(
                &authored_animation.spec,
                RecordSpec::Animation(AnimationRecordSpec::LinearAnimation(_))
            ) {
                return Err(EditAbort::new(
                    operation_index,
                    vec![EditId::Object(animation.object_id())],
                    EditReason::InternalInvariant,
                ));
            }
            let keyed_object_id = self
                .definition_index
                .keyed_objects
                .get(&(animation.object_id(), target))
                .copied();
            let keyed_property_id = keyed_object_id.and_then(|keyed_object| {
                self.definition_index
                    .keyed_properties
                    .get(&(keyed_object, semantic_property))
                    .copied()
            });
            let existing_key_frame_id = keyed_property_id.and_then(|keyed_property| {
                self.definition_index
                    .key_frames
                    .get(&(keyed_property, frame))
                    .copied()
            });
            (keyed_object_id, keyed_property_id, existing_key_frame_id)
        };

        if let Some(key_frame_id) = existing_key_frame_id {
            let key_frame_location = self
                .definition_index
                .objects
                .get(&key_frame_id)
                .copied()
                .ok_or_else(|| {
                    EditAbort::new(
                        operation_index,
                        vec![EditId::Object(key_frame_id)],
                        EditReason::InternalInvariant,
                    )
                })?;
            let key_frame = self
                .definitions
                .artboards
                .get_mut(key_frame_location.artboard_index)
                .and_then(|artboard| artboard.records.get_mut(key_frame_location.record_index))
                .ok_or_else(|| {
                    EditAbort::new(
                        operation_index,
                        vec![EditId::Object(animation.object_id())],
                        EditReason::InternalInvariant,
                    )
                })?;
            let RecordSpec::Animation(AnimationRecordSpec::KeyFrameDouble {
                value: key_frame_value,
                ..
            }) = &mut key_frame.spec
            else {
                return Err(EditAbort::new(
                    operation_index,
                    vec![EditId::Object(key_frame_id)],
                    EditReason::InternalInvariant,
                ));
            };
            *key_frame_value = value;
            self.touched_artboards.insert(artboard, operation_index);
            self.spec_origins
                .nodes
                .insert(key_frame_id, operation_index);
            return Ok(key_frame_id);
        }

        let new_keyed_object = keyed_object_id
            .is_none()
            .then(|| allocate_global_identity(&NEXT_OBJECT_ID).map(ObjectId));
        let new_keyed_property = keyed_property_id
            .is_none()
            .then(|| allocate_global_identity(&NEXT_OBJECT_ID).map(ObjectId));
        let key_frame_id = allocate_global_identity(&NEXT_OBJECT_ID).map(ObjectId);
        let allocated = [
            new_keyed_object.flatten(),
            new_keyed_property.flatten(),
            key_frame_id,
        ];
        if allocated
            .iter()
            .enumerate()
            .any(|(index, id)| index == 2 && id.is_none())
            || (keyed_object_id.is_none() && allocated[0].is_none())
            || (keyed_property_id.is_none() && allocated[1].is_none())
        {
            return Err(EditAbort::new(
                operation_index,
                Vec::new(),
                EditReason::IdentityExhausted,
            ));
        }
        let key_frame_id = allocated[2].ok_or_else(|| {
            EditAbort::new(operation_index, Vec::new(), EditReason::IdentityExhausted)
        })?;
        let authored_artboard = self
            .definitions
            .artboards
            .get_mut(artboard_index)
            .ok_or_else(|| {
                EditAbort::new(
                    operation_index,
                    vec![EditId::Object(animation.object_id())],
                    EditReason::InternalInvariant,
                )
            })?;
        let keyed_object_id = if let Some(id) = keyed_object_id {
            id
        } else {
            let id = allocated[0].ok_or_else(|| {
                EditAbort::new(operation_index, Vec::new(), EditReason::IdentityExhausted)
            })?;
            let record_index = authored_artboard.records.len();
            authored_artboard.records.push(RecordDefinition {
                id,
                spec: RecordSpec::Animation(AnimationRecordSpec::KeyedObject {
                    animation: animation.object_id(),
                    target,
                }),
            });
            self.definition_index.objects.insert(
                id,
                IndexedObject {
                    artboard,
                    artboard_index,
                    record_index,
                    kind: AuthoredObjectKind::KeyedObject,
                },
            );
            self.definition_index
                .owned
                .entry(animation.object_id())
                .or_default()
                .push(id);
            self.definition_index.owned.entry(id).or_default();
            self.definition_index
                .keyed_objects
                .insert((animation.object_id(), target), id);
            self.created_objects.push(id);
            self.spec_origins.nodes.insert(id, operation_index);
            id
        };
        let keyed_property_id = if let Some(id) = keyed_property_id {
            id
        } else {
            let id = allocated[1].ok_or_else(|| {
                EditAbort::new(operation_index, Vec::new(), EditReason::IdentityExhausted)
            })?;
            let record_index = authored_artboard.records.len();
            authored_artboard.records.push(RecordDefinition {
                id,
                spec: RecordSpec::Animation(AnimationRecordSpec::KeyedProperty {
                    keyed_object: keyed_object_id,
                    property: semantic_property,
                }),
            });
            self.definition_index.objects.insert(
                id,
                IndexedObject {
                    artboard,
                    artboard_index,
                    record_index,
                    kind: AuthoredObjectKind::KeyedProperty,
                },
            );
            self.definition_index
                .owned
                .entry(keyed_object_id)
                .or_default()
                .push(id);
            self.definition_index.owned.entry(id).or_default();
            self.definition_index
                .keyed_properties
                .insert((keyed_object_id, semantic_property), id);
            self.created_objects.push(id);
            self.spec_origins.nodes.insert(id, operation_index);
            id
        };
        let key_frame_record_index = authored_artboard.records.len();
        authored_artboard.records.push(RecordDefinition {
            id: key_frame_id,
            spec: RecordSpec::Animation(AnimationRecordSpec::KeyFrameDouble {
                keyed_property: keyed_property_id,
                frame,
                value,
            }),
        });
        self.definition_index.objects.insert(
            key_frame_id,
            IndexedObject {
                artboard,
                artboard_index,
                record_index: key_frame_record_index,
                kind: AuthoredObjectKind::KeyFrameDouble,
            },
        );
        self.definition_index
            .owned
            .entry(keyed_property_id)
            .or_default()
            .push(key_frame_id);
        self.definition_index.owned.entry(key_frame_id).or_default();
        self.definition_index
            .key_frames
            .insert((keyed_property_id, frame), key_frame_id);
        self.created_objects.push(key_frame_id);
        self.spec_origins
            .nodes
            .insert(key_frame_id, operation_index);
        self.touched_artboards.insert(artboard, operation_index);
        Ok(key_frame_id)
    }

    fn begin_operation(&mut self) -> std::result::Result<usize, EditAbort> {
        let operation_index = *self.next_operation_index;
        let Some(next) = operation_index.checked_add(1) else {
            return Err(EditAbort::new(
                operation_index,
                Vec::new(),
                EditReason::OperationLimitExceeded,
            ));
        };
        *self.next_operation_index = next;
        Ok(operation_index)
    }
}

/// Invariant-enforcing state-machine vocabulary over a [`SceneTx`].
///
/// Import-context ordering and runtime-local ordinals are derived during
/// materialization; callers retain only semantic ids in the uniform record
/// store.
pub struct MachineTx<'a> {
    definitions: &'a mut Definitions,
    definition_index: &'a mut DefinitionIndex,
    next_operation_index: &'a mut usize,
    created_objects: &'a mut Vec<ObjectId>,
    touched_artboards: &'a mut BTreeMap<ArtboardId, usize>,
    spec_origins: &'a mut SpecOrigins,
}

impl MachineTx<'_> {
    pub fn create_event(
        &mut self,
        artboard: ArtboardId,
        mut spec: EventSpec,
    ) -> std::result::Result<EventId, EditAbort> {
        let operation_index = self.begin_operation()?;
        let artboard_index = self.artboard_index(artboard, operation_index)?;
        normalize_optional_machine_name(&mut spec.name);
        self.insert_record(
            artboard,
            artboard_index,
            operation_index,
            RecordSpec::Machine(MachineRecordSpec::Event(spec)),
        )
        .map(EventId)
    }

    pub fn create_machine(
        &mut self,
        artboard: ArtboardId,
        mut spec: MachineSpec,
    ) -> std::result::Result<MachineId, EditAbort> {
        let operation_index = self.begin_operation()?;
        let artboard_index = self.artboard_index(artboard, operation_index)?;
        normalize_optional_machine_name(&mut spec.name);
        self.insert_record(
            artboard,
            artboard_index,
            operation_index,
            RecordSpec::Machine(MachineRecordSpec::Machine(spec)),
        )
        .map(MachineId)
    }

    pub fn create_trigger_input(
        &mut self,
        machine: MachineId,
        spec: TriggerInputSpec,
    ) -> std::result::Result<MachineInputId, EditAbort> {
        let operation_index = self.begin_operation()?;
        let owner = self.expect_kind(
            machine.object_id(),
            AuthoredObjectKind::Machine,
            "machine",
            operation_index,
        )?;
        if spec.name.trim().is_empty() {
            return Err(EditAbort::new(
                operation_index,
                vec![EditId::Object(machine.object_id())],
                EditReason::EmptyMachineInputName,
            ));
        }
        if let Some(existing) = self
            .definitions
            .artboards
            .get(owner.artboard_index)
            .into_iter()
            .flat_map(|artboard| &artboard.records)
            .find(|record| {
                matches!(
                    &record.spec,
                    RecordSpec::Machine(MachineRecordSpec::TriggerInput {
                        machine: candidate,
                        spec: candidate_spec,
                    }) if *candidate == machine.object_id() && candidate_spec.name == spec.name
                )
            })
        {
            return Err(EditAbort::new(
                operation_index,
                vec![
                    EditId::Object(machine.object_id()),
                    EditId::Object(existing.id),
                ],
                EditReason::DuplicateMachineInputName,
            ));
        }
        self.insert_record(
            owner.artboard,
            owner.artboard_index,
            operation_index,
            RecordSpec::Machine(MachineRecordSpec::TriggerInput {
                machine: machine.object_id(),
                spec,
            }),
        )
        .map(MachineInputId)
    }

    pub fn create_layer(
        &mut self,
        machine: MachineId,
        mut spec: MachineLayerSpec,
    ) -> std::result::Result<MachineLayerId, EditAbort> {
        let operation_index = self.begin_operation()?;
        let owner = self.expect_kind(
            machine.object_id(),
            AuthoredObjectKind::Machine,
            "machine",
            operation_index,
        )?;
        normalize_optional_machine_name(&mut spec.name);
        self.insert_record(
            owner.artboard,
            owner.artboard_index,
            operation_index,
            RecordSpec::Machine(MachineRecordSpec::Layer {
                machine: machine.object_id(),
                spec,
            }),
        )
        .map(MachineLayerId)
    }

    pub fn create_entry_state(
        &mut self,
        layer: MachineLayerId,
    ) -> std::result::Result<MachineStateId, EditAbort> {
        let operation_index = self.begin_operation()?;
        let owner = self.expect_kind(
            layer.object_id(),
            AuthoredObjectKind::MachineLayer,
            "layer",
            operation_index,
        )?;
        self.insert_record(
            owner.artboard,
            owner.artboard_index,
            operation_index,
            RecordSpec::Machine(MachineRecordSpec::EntryState {
                layer: layer.object_id(),
            }),
        )
        .map(MachineStateId)
    }

    pub fn create_any_state(
        &mut self,
        layer: MachineLayerId,
    ) -> std::result::Result<MachineStateId, EditAbort> {
        let operation_index = self.begin_operation()?;
        let owner = self.expect_kind(
            layer.object_id(),
            AuthoredObjectKind::MachineLayer,
            "layer",
            operation_index,
        )?;
        self.insert_record(
            owner.artboard,
            owner.artboard_index,
            operation_index,
            RecordSpec::Machine(MachineRecordSpec::AnyState {
                layer: layer.object_id(),
            }),
        )
        .map(MachineStateId)
    }

    pub fn create_exit_state(
        &mut self,
        layer: MachineLayerId,
    ) -> std::result::Result<MachineStateId, EditAbort> {
        let operation_index = self.begin_operation()?;
        let owner = self.expect_kind(
            layer.object_id(),
            AuthoredObjectKind::MachineLayer,
            "layer",
            operation_index,
        )?;
        self.insert_record(
            owner.artboard,
            owner.artboard_index,
            operation_index,
            RecordSpec::Machine(MachineRecordSpec::ExitState {
                layer: layer.object_id(),
            }),
        )
        .map(MachineStateId)
    }

    pub fn create_animation_state(
        &mut self,
        layer: MachineLayerId,
        spec: AnimationStateSpec,
    ) -> std::result::Result<MachineStateId, EditAbort> {
        let operation_index = self.begin_operation()?;
        let owner = self.expect_kind(
            layer.object_id(),
            AuthoredObjectKind::MachineLayer,
            "layer",
            operation_index,
        )?;
        let animation = self.expect_kind(
            spec.animation.object_id(),
            AuthoredObjectKind::LinearAnimation,
            "animation",
            operation_index,
        )?;
        self.ensure_same_artboard(
            owner.artboard,
            animation.artboard,
            operation_index,
            [layer.object_id(), spec.animation.object_id()],
        )?;
        self.insert_record(
            owner.artboard,
            owner.artboard_index,
            operation_index,
            RecordSpec::Machine(MachineRecordSpec::AnimationState {
                layer: layer.object_id(),
                animation: spec.animation,
            }),
        )
        .map(MachineStateId)
    }

    pub fn create_transition(
        &mut self,
        source: MachineStateId,
        target: MachineStateId,
    ) -> std::result::Result<MachineTransitionId, EditAbort> {
        let operation_index = self.begin_operation()?;
        let source_record = self.expect_state(source, "source", operation_index)?;
        let target_record = self.expect_state(target, "target", operation_index)?;
        self.ensure_same_artboard(
            source_record.artboard,
            target_record.artboard,
            operation_index,
            [source.object_id(), target.object_id()],
        )?;
        let source_layer = self.owner_of(source.object_id(), operation_index)?;
        let target_layer = self.owner_of(target.object_id(), operation_index)?;
        if source_layer != target_layer {
            return Err(EditAbort::new(
                operation_index,
                vec![
                    EditId::Object(source.object_id()),
                    EditId::Object(target.object_id()),
                ],
                EditReason::InvalidMachineReference,
            ));
        }
        self.insert_record(
            source_record.artboard,
            source_record.artboard_index,
            operation_index,
            RecordSpec::Machine(MachineRecordSpec::Transition {
                source: source.object_id(),
                target: target.object_id(),
            }),
        )
        .map(MachineTransitionId)
    }

    pub fn add_trigger_condition(
        &mut self,
        transition: MachineTransitionId,
        input: MachineInputId,
    ) -> std::result::Result<ObjectId, EditAbort> {
        let operation_index = self.begin_operation()?;
        let transition_record = self.expect_kind(
            transition.object_id(),
            AuthoredObjectKind::StateTransition,
            "transition",
            operation_index,
        )?;
        let input_record = self.expect_kind(
            input.object_id(),
            AuthoredObjectKind::MachineTrigger,
            "input",
            operation_index,
        )?;
        self.ensure_same_artboard(
            transition_record.artboard,
            input_record.artboard,
            operation_index,
            [transition.object_id(), input.object_id()],
        )?;
        let source = self.owner_of(transition.object_id(), operation_index)?;
        let layer = self.owner_of(source, operation_index)?;
        let machine = self.owner_of(layer, operation_index)?;
        if self.owner_of(input.object_id(), operation_index)? != machine {
            return Err(EditAbort::new(
                operation_index,
                vec![
                    EditId::Object(transition.object_id()),
                    EditId::Object(input.object_id()),
                ],
                EditReason::InvalidMachineReference,
            ));
        }
        self.insert_record(
            transition_record.artboard,
            transition_record.artboard_index,
            operation_index,
            RecordSpec::Machine(MachineRecordSpec::TriggerCondition {
                transition: transition.object_id(),
                input,
            }),
        )
    }

    pub fn add_fire_event(
        &mut self,
        state: MachineStateId,
        event: EventId,
        occurs: FireEventOccurs,
    ) -> std::result::Result<ObjectId, EditAbort> {
        let operation_index = self.begin_operation()?;
        let state_record = self.expect_state(state, "state", operation_index)?;
        let event_record = self.expect_kind(
            event.object_id(),
            AuthoredObjectKind::Event,
            "event",
            operation_index,
        )?;
        self.ensure_same_artboard(
            state_record.artboard,
            event_record.artboard,
            operation_index,
            [state.object_id(), event.object_id()],
        )?;
        self.insert_record(
            state_record.artboard,
            state_record.artboard_index,
            operation_index,
            RecordSpec::Machine(MachineRecordSpec::FireEvent {
                state: state.object_id(),
                event,
                occurs,
            }),
        )
    }

    fn insert_record(
        &mut self,
        artboard: ArtboardId,
        artboard_index: usize,
        operation_index: usize,
        spec: RecordSpec,
    ) -> std::result::Result<ObjectId, EditAbort> {
        let id = ObjectId(allocate_global_identity(&NEXT_OBJECT_ID).ok_or_else(|| {
            EditAbort::new(operation_index, Vec::new(), EditReason::IdentityExhausted)
        })?);
        let owner = spec.owner();
        let kind = spec.kind();
        let definition = self
            .definitions
            .artboards
            .get_mut(artboard_index)
            .filter(|definition| definition.id == artboard)
            .ok_or_else(|| {
                EditAbort::new(
                    operation_index,
                    vec![EditId::Artboard(artboard)],
                    EditReason::InternalInvariant,
                )
            })?;
        let record_index = definition.records.len();
        definition.records.push(RecordDefinition { id, spec });
        self.definition_index.objects.insert(
            id,
            IndexedObject {
                artboard,
                artboard_index,
                record_index,
                kind,
            },
        );
        self.definition_index.owned.entry(id).or_default();
        if let Some(owner) = owner {
            self.definition_index
                .owned
                .entry(owner)
                .or_default()
                .push(id);
        }
        self.created_objects.push(id);
        self.touched_artboards.insert(artboard, operation_index);
        self.spec_origins.nodes.insert(id, operation_index);
        Ok(id)
    }

    fn artboard_index(
        &self,
        artboard: ArtboardId,
        operation_index: usize,
    ) -> std::result::Result<usize, EditAbort> {
        self.definition_index
            .artboards
            .get(&artboard)
            .copied()
            .ok_or_else(|| {
                EditAbort::new(
                    operation_index,
                    vec![EditId::Artboard(artboard)],
                    EditReason::UnknownArtboard,
                )
            })
    }

    fn expect_kind(
        &self,
        id: ObjectId,
        expected: AuthoredObjectKind,
        property: &'static str,
        operation_index: usize,
    ) -> std::result::Result<IndexedObject, EditAbort> {
        let actual = self
            .definition_index
            .objects
            .get(&id)
            .copied()
            .ok_or_else(|| {
                EditAbort::new(
                    operation_index,
                    vec![EditId::Object(id)],
                    EditReason::UnknownObject,
                )
            })?;
        if actual.kind != expected {
            return Err(EditAbort::new(
                operation_index,
                vec![EditId::Object(id)],
                EditReason::RecordPropertyOwnerMismatch {
                    property,
                    actual: actual.kind,
                },
            ));
        }
        Ok(actual)
    }

    fn expect_state(
        &self,
        state: MachineStateId,
        property: &'static str,
        operation_index: usize,
    ) -> std::result::Result<IndexedObject, EditAbort> {
        let actual = self
            .definition_index
            .objects
            .get(&state.object_id())
            .copied()
            .ok_or_else(|| {
                EditAbort::new(
                    operation_index,
                    vec![EditId::Object(state.object_id())],
                    EditReason::UnknownObject,
                )
            })?;
        if !matches!(
            actual.kind,
            AuthoredObjectKind::AnyState
                | AuthoredObjectKind::EntryState
                | AuthoredObjectKind::ExitState
                | AuthoredObjectKind::AnimationState
        ) {
            return Err(EditAbort::new(
                operation_index,
                vec![EditId::Object(state.object_id())],
                EditReason::RecordPropertyOwnerMismatch {
                    property,
                    actual: actual.kind,
                },
            ));
        }
        Ok(actual)
    }

    fn owner_of(
        &self,
        id: ObjectId,
        operation_index: usize,
    ) -> std::result::Result<ObjectId, EditAbort> {
        let indexed = self
            .definition_index
            .objects
            .get(&id)
            .copied()
            .ok_or_else(|| {
                EditAbort::new(
                    operation_index,
                    vec![EditId::Object(id)],
                    EditReason::UnknownObject,
                )
            })?;
        self.definitions
            .artboards
            .get(indexed.artboard_index)
            .and_then(|artboard| artboard.records.get(indexed.record_index))
            .and_then(|record| record.spec.owner())
            .ok_or_else(|| {
                EditAbort::new(
                    operation_index,
                    vec![EditId::Object(id)],
                    EditReason::InternalInvariant,
                )
            })
    }

    fn ensure_same_artboard(
        &self,
        source: ArtboardId,
        target: ArtboardId,
        operation_index: usize,
        ids: [ObjectId; 2],
    ) -> std::result::Result<(), EditAbort> {
        if source == target {
            return Ok(());
        }
        Err(EditAbort::new(
            operation_index,
            ids.into_iter().map(EditId::Object).collect(),
            EditReason::CrossArtboardReference { source, target },
        ))
    }

    fn begin_operation(&mut self) -> std::result::Result<usize, EditAbort> {
        let operation_index = *self.next_operation_index;
        let Some(next) = operation_index.checked_add(1) else {
            return Err(EditAbort::new(
                operation_index,
                Vec::new(),
                EditReason::OperationLimitExceeded,
            ));
        };
        *self.next_operation_index = next;
        Ok(operation_index)
    }
}

fn normalize_optional_machine_name(name: &mut Option<String>) {
    if name.as_ref().is_some_and(|name| name.trim().is_empty()) {
        *name = None;
    }
}

fn valid_artboard_child(child: NodeKind) -> bool {
    matches!(
        child,
        NodeKind::Shape
            | NodeKind::NestedArtboard
            | NodeKind::Image
            | NodeKind::Text
            | NodeKind::ScriptedDrawable
    )
}

fn valid_object_parent(parent: NodeKind, child: NodeKind) -> bool {
    matches!(
        (parent, child),
        (
            NodeKind::Shape,
            NodeKind::Rectangle | NodeKind::Fill | NodeKind::Stroke
        ) | (NodeKind::Fill, NodeKind::SolidColor)
            | (NodeKind::Stroke, NodeKind::SolidColor | NodeKind::DashPath)
            | (NodeKind::DashPath, NodeKind::Dash)
            | (
                NodeKind::Text,
                NodeKind::TextValueRun | NodeKind::TextStylePaint
            )
            | (NodeKind::TextStylePaint, NodeKind::Fill | NodeKind::Stroke)
    )
}

fn artboard_references(definition: &ArtboardDefinition) -> impl Iterator<Item = ArtboardId> + '_ {
    definition.records.iter().filter_map(|record| {
        let Some((_, NodeSpec::NestedArtboard(spec))) = record.visual() else {
            return None;
        };
        Some(spec.artboard)
    })
}

fn first_artboard_reference_cycle(
    artboard: ArtboardId,
    references: &BTreeMap<ArtboardId, Vec<(ArtboardId, ObjectId)>>,
    path: &mut Vec<(ArtboardId, ObjectId)>,
) -> Option<Vec<(ArtboardId, ObjectId)>> {
    let outgoing = references.get(&artboard)?;
    for (referenced, node) in outgoing {
        if let Some(cycle_start) = path.iter().position(|(ancestor, _)| ancestor == referenced) {
            let mut cycle = path.get(cycle_start..).unwrap_or_default().to_vec();
            cycle.push((artboard, *node));
            return Some(cycle);
        }
        path.push((artboard, *node));
        if let Some(cycle) = first_artboard_reference_cycle(*referenced, references, path) {
            return Some(cycle);
        }
        path.pop();
    }
    None
}

fn expand_dirty_artboards(
    definitions: &Definitions,
    directly_dirty_artboards: &BTreeSet<ArtboardId>,
) -> BTreeSet<ArtboardId> {
    let mut dirty = directly_dirty_artboards.clone();
    loop {
        let previous_len = dirty.len();
        for artboard in &definitions.artboards {
            if dirty.contains(&artboard.id) {
                continue;
            }
            if artboard_references(artboard).any(|referenced| dirty.contains(&referenced)) {
                dirty.insert(artboard.id);
            }
        }
        if dirty.len() == previous_len {
            break;
        }
    }
    dirty
}

/// World-space endpoints of one shaped Text caret line.
///
/// The public canonical caret uses downstream affinity at a source boundary.
/// Selection and hit testing retain both visual sides internally when a
/// modifier or soft wrap makes upstream and downstream geometry differ.
/// Source boundaries in whitespace omitted at a soft wrap canonically share
/// the next-line caret; static trailing separators have no synthetic caret.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CaretGeometry {
    pub top: crate::Vec2D,
    pub bottom: crate::Vec2D,
}

/// A short-lived facade over direct runtime instance writes and draws.
pub struct Frame<'a> {
    scene: &'a mut Scene,
}

/// One semantic runtime event reported by [`Frame::advance`].
#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
pub enum SceneEvent {
    Authored {
        event: EventId,
        name: Option<String>,
        seconds_delay: f32,
    },
}

impl Frame<'_> {
    /// Reads the cursor's current value directly from its live runtime instance.
    /// Schema defaults are observable even when the materialized record was sparse.
    pub fn get<T: PropValue>(&self, cursor: Cursor<T>) -> std::result::Result<T, StaleCursor> {
        if cursor.scene != self.scene.identity.id || cursor.epoch != self.scene.epoch {
            return Err(StaleCursor);
        }
        let instance = self
            .scene
            .instances
            .get(cursor.instance_slot)
            .and_then(Option::as_ref)
            .filter(|instance| instance.id == cursor.instance)
            .ok_or(StaleCursor)?;
        (cursor.property.read_from_runtime)(
            instance.runtime.raw(),
            cursor.local_id,
            cursor.property.key,
        )
        .ok_or(StaleCursor)
    }

    /// Write one ephemeral live-instance value without changing authored definitions.
    /// A structural remount restores the definition value; the owning app may replay
    /// active gesture or controller state through freshly resolved cursors.
    pub fn set<T: PropValue>(
        &mut self,
        cursor: Cursor<T>,
        value: T,
    ) -> std::result::Result<bool, StaleCursor> {
        if cursor.scene != self.scene.identity.id || cursor.epoch != self.scene.epoch {
            return Err(StaleCursor);
        }
        let instance = self
            .scene
            .instances
            .get_mut(cursor.instance_slot)
            .and_then(Option::as_mut)
            .filter(|instance| instance.id == cursor.instance)
            .ok_or(StaleCursor)?;
        if !value.is_valid() {
            return Ok(false);
        }
        Ok((cursor.property.apply_to_runtime)(
            instance.runtime.raw_mut(),
            cursor.local_id,
            cursor.property.key,
            value,
        ))
    }

    /// Fire one pre-resolved trigger on its retained state-machine instance.
    pub fn fire(&mut self, cursor: InputCursor) -> std::result::Result<(), StaleCursor> {
        if cursor.scene != self.scene.identity.id
            || cursor.epoch != self.scene.epoch
            || cursor.input_kind != StateMachineInputKind::Trigger
        {
            return Err(StaleCursor);
        }
        let live = self
            .scene
            .instances
            .get_mut(cursor.instance_slot)
            .and_then(Option::as_mut)
            .filter(|instance| instance.id == cursor.instance)
            .ok_or(StaleCursor)?;
        let machine = live
            .machines
            .get_mut(cursor.machine, cursor.machine_index)
            .ok_or(StaleCursor)?;
        if machine
            .input(cursor.input_index)
            .is_none_or(|input| input.kind() != StateMachineInputKind::Trigger)
        {
            return Err(StaleCursor);
        }
        let _ = machine.fire_trigger(cursor.input_index);
        Ok(())
    }

    /// Apply one authored linear animation at an absolute time in seconds to
    /// one existing live instance.
    ///
    /// The animation is resolved through the instance's current materialized
    /// file. A removed animation, a foreign/dropped instance, or a nonfinite
    /// time is stale. Scrubbing mutates only runtime visual state; it does not
    /// change durable definitions or the scene epoch.
    pub fn scrub(
        &mut self,
        instance: InstanceId,
        animation: AnimationId,
        time: f32,
    ) -> std::result::Result<(), StaleCursor> {
        if !time.is_finite() {
            return Err(StaleCursor);
        }
        let (artboard, animation_index) = self
            .scene
            .materialized
            .iter()
            .find_map(|(artboard, materialized)| {
                materialized
                    .animations
                    .get(&animation)
                    .copied()
                    .map(|index| (*artboard, index))
            })
            .ok_or(StaleCursor)?;
        let live = self
            .scene
            .instances
            .iter_mut()
            .filter_map(Option::as_mut)
            .find(|candidate| candidate.id == instance && candidate.artboard == artboard)
            .ok_or(StaleCursor)?;
        let _ = live
            .runtime
            .raw_mut()
            .apply_linear_animation(animation_index, time, 1.0);
        let _ = live.runtime.raw_mut().update_pass();
        Ok(())
    }

    /// Advance one live instance and settle its runtime-driven visual state.
    ///
    /// `events` is caller-owned reusable storage. It is cleared before each
    /// advance so it contains only semantic authored events emitted by this
    /// call, ordered by retained machine and runtime report order. An unknown
    /// or dropped instance is an unchanged frame.
    pub fn advance(
        &mut self,
        instance: InstanceId,
        elapsed_seconds: f32,
        events: &mut Vec<SceneEvent>,
    ) -> bool {
        events.clear();
        let (materialized, instances) = (&self.scene.materialized, &mut self.scene.instances);
        let Some(live) = instances
            .iter_mut()
            .filter_map(Option::as_mut)
            .find(|candidate| candidate.id == instance)
        else {
            return false;
        };
        let Some(materialized) = materialized.get(&live.artboard) else {
            return false;
        };
        if live.machines.is_empty() {
            return live.runtime.advance(elapsed_seconds);
        }

        let (runtime, machines) = (&mut live.runtime, &mut live.machines);
        let changed = runtime.advance_with_state_machines(&mut machines.values, elapsed_seconds);
        for machine in &machines.values {
            for index in 0..machine.reported_event_count() {
                let Some(reported) = machine.reported_event(index) else {
                    continue;
                };
                let Some(event) = materialized
                    .events_by_local
                    .get(reported.event_local_index())
                    .copied()
                    .flatten()
                else {
                    continue;
                };
                events.push(SceneEvent::Authored {
                    event,
                    name: reported.name().map(ToOwned::to_owned),
                    seconds_delay: reported.seconds_delay(),
                });
            }
        }
        changed
    }

    /// Advance one live instance with a renderer factory available to script
    /// generator/init/advance/update phases. Unlike [`Self::advance`], script
    /// failures are reported by this call instead of being deferred to draw.
    /// After scripted bootstrap, this and draw must keep using the same live
    /// Factory object for every instance backed by that materialized File.
    pub fn try_advance_with_factory(
        &mut self,
        instance: InstanceId,
        elapsed_seconds: f32,
        events: &mut Vec<SceneEvent>,
        factory: &mut dyn Factory,
    ) -> std::result::Result<bool, AdvanceError> {
        events.clear();
        let (materialized, instances) = (&self.scene.materialized, &mut self.scene.instances);
        let live = instances
            .iter_mut()
            .filter_map(Option::as_mut)
            .find(|candidate| candidate.id == instance)
            .ok_or(AdvanceError::UnknownInstance)?;
        let materialized = materialized
            .get(&live.artboard)
            .ok_or(AdvanceError::RuntimeRejected)?;
        if live.machines.is_empty() {
            return live
                .runtime
                .try_advance_with_factory(factory, elapsed_seconds)
                .map_err(|_| AdvanceError::RuntimeRejected);
        }

        let (runtime, machines) = (&mut live.runtime, &mut live.machines);
        let changed = runtime
            .try_advance_with_state_machines_and_factory(
                &mut machines.values,
                elapsed_seconds,
                factory,
            )
            .map_err(|_| AdvanceError::RuntimeRejected)?;
        for machine in &machines.values {
            for index in 0..machine.reported_event_count() {
                let Some(reported) = machine.reported_event(index) else {
                    continue;
                };
                let Some(event) = materialized
                    .events_by_local
                    .get(reported.event_local_index())
                    .copied()
                    .flatten()
                else {
                    continue;
                };
                events.push(SceneEvent::Authored {
                    event,
                    name: reported.name().map(ToOwned::to_owned),
                    seconds_delay: reported.seconds_delay(),
                });
            }
        }
        Ok(changed)
    }

    /// Return authored shapes under `point`, ordered front to back and deduplicated.
    pub fn hit_test(&mut self, instance: InstanceId, point: crate::Vec2D) -> Vec<ObjectId> {
        self.hit_test_paths(instance, point)
            .into_iter()
            .filter_map(|path| path.into_objects().into_iter().last())
            .collect()
    }

    /// Return authored object paths under `point`, ordered front to back and
    /// deduplicated. Direct hits contain the hit object. Nested hits are
    /// prefixed with each authored nested-artboard host.
    pub fn hit_test_paths(
        &mut self,
        instance: InstanceId,
        point: crate::Vec2D,
    ) -> Vec<SceneObjectPath> {
        let Some((artboard, local_hits)) = self
            .scene
            .instances
            .iter_mut()
            .filter_map(Option::as_mut)
            .find(|candidate| candidate.id == instance)
            .map(|live| (live.artboard, live.runtime.hit_test_paths(point)))
        else {
            return Vec::new();
        };
        let Some(materialized) = self.scene.materialized.get(&artboard) else {
            return Vec::new();
        };
        local_hits
            .into_iter()
            .filter_map(|local_path| materialized.resolve_object_path(artboard, &local_path))
            .collect()
    }

    /// Return exact logical world bounds for an authored object in this instance.
    pub fn world_bounds(&mut self, instance: InstanceId, object: ObjectId) -> Option<crate::Aabb> {
        let (artboard, local_id) = self.resolve_geometry_target(instance, object)?;
        self.scene
            .instances
            .iter_mut()
            .filter_map(Option::as_mut)
            .find(|candidate| candidate.id == instance && candidate.artboard == artboard)?
            .runtime
            .world_bounds(local_id)
    }

    /// Return the settled, layout-aware world transform for an authored object.
    pub fn world_transform(
        &mut self,
        instance: InstanceId,
        object: ObjectId,
    ) -> Option<crate::Mat2D> {
        let (artboard, local_id) = self.resolve_geometry_target(instance, object)?;
        self.scene
            .instances
            .iter_mut()
            .filter_map(Option::as_mut)
            .find(|candidate| candidate.id == instance && candidate.artboard == artboard)?
            .runtime
            .world_transform(local_id)
    }

    /// Return the canonical downstream shaped Text caret in source-artboard
    /// world space for one exact UTF-8 byte boundary.
    ///
    /// A boundary skipped with leading whitespace at a soft wrap snaps to the
    /// next visual line. Static Text does not synthesize a caret after a
    /// trailing newline or other static line separator. CRLF is one authored
    /// separator, so the boundary between its two scalars has no geometry.
    ///
    /// Returns `None` when the instance or object is unknown, stale, foreign,
    /// or not Text; the offset is past the source or inside a UTF-8 scalar;
    /// font data for the base style or any participating nonempty run is
    /// missing or invalid; layout, transform, or modifier geometry is
    /// nonfinite; or overflow is unsupported or unknown. Geometry v1 supports
    /// only `Visible`, `Fit`, and `FitFontSize`; `Hidden`, `Clipped`, and
    /// `Ellipsis` fail closed.
    pub fn text_caret(
        &mut self,
        instance: InstanceId,
        object: ObjectId,
        byte_offset: usize,
    ) -> Option<CaretGeometry> {
        let (artboard, local_id) = self.resolve_geometry_target(instance, object)?;
        self.scene
            .instances
            .iter_mut()
            .filter_map(Option::as_mut)
            .find(|candidate| candidate.id == instance && candidate.artboard == artboard)?
            .runtime
            .text_caret(local_id, byte_offset)
    }

    /// Return the nearest valid UTF-8 byte caret for one source-artboard
    /// world-space point on shaped Text.
    ///
    /// Returns `None` for a nonfinite point; an unknown, stale, foreign, or
    /// non-Text target; unshapeable text; nonfinite layout, transform, or
    /// modifier geometry; a singular/non-invertible world transform; and
    /// unsupported or unknown overflow. Geometry v1 supports only `Visible`,
    /// `Fit`, and `FitFontSize`.
    pub fn text_hit(
        &mut self,
        instance: InstanceId,
        object: ObjectId,
        point: crate::Vec2D,
    ) -> Option<usize> {
        let (artboard, local_id) = self.resolve_geometry_target(instance, object)?;
        self.scene
            .instances
            .iter_mut()
            .filter_map(Option::as_mut)
            .find(|candidate| candidate.id == instance && candidate.artboard == artboard)?
            .runtime
            .text_hit(local_id, point)
    }

    /// Return one source-artboard world-space selection rectangle per shaped
    /// line segment covered by an exact UTF-8 byte range.
    ///
    /// Returns an empty result when either endpoint is past the source or
    /// inside a UTF-8 scalar, the range is empty or reversed, the instance or
    /// object is unknown, stale, foreign, or not Text, the text is unshapeable,
    /// geometry is nonfinite, or overflow is unsupported or unknown. Selection
    /// starts use downstream affinity and ends use upstream affinity, including
    /// source whitespace omitted at soft wraps. A trailing static line
    /// separator does not create a selectable final empty line. CRLF is
    /// treated as one authored separator; its internal scalar boundary is not
    /// selectable.
    pub fn text_selection_rects(
        &mut self,
        instance: InstanceId,
        object: ObjectId,
        range: std::ops::Range<usize>,
    ) -> Vec<crate::Aabb> {
        let Some((artboard, local_id)) = self.resolve_geometry_target(instance, object) else {
            return Vec::new();
        };
        self.scene
            .instances
            .iter_mut()
            .filter_map(Option::as_mut)
            .find(|candidate| candidate.id == instance && candidate.artboard == artboard)
            .map(|live| live.runtime.text_selection_rects(local_id, range))
            .unwrap_or_default()
    }

    fn resolve_geometry_target(
        &self,
        instance: InstanceId,
        object: ObjectId,
    ) -> Option<(ArtboardId, usize)> {
        let live = self
            .scene
            .instances
            .iter()
            .filter_map(Option::as_ref)
            .find(|candidate| candidate.id == instance)?;
        let materialized = self.scene.materialized.get(&live.artboard)?;
        let local_id = materialized.objects.get(&object)?.local_id;
        Some((live.artboard, local_id))
    }

    pub fn draw(
        &mut self,
        instance: InstanceId,
        factory: &mut dyn Factory,
        renderer: &mut dyn Renderer,
        cache: &mut SceneRenderCache,
    ) -> std::result::Result<(), DrawError> {
        let live = self
            .scene
            .instances
            .iter_mut()
            .filter_map(Option::as_mut)
            .find(|candidate| candidate.id == instance)
            .ok_or(DrawError::UnknownInstance)?;
        let needs_refresh = !Arc::ptr_eq(&cache.scene_identity, &self.scene.identity)
            || cache.instance != instance
            || cache.mount != live.mount
            || cache.inner.is_none();
        if needs_refresh {
            let candidate = live.runtime.new_render_cache();
            cache.scene_identity = Arc::clone(&self.scene.identity);
            cache.instance = instance;
            cache.mount = live.mount;
            cache.inner = Some(candidate);
        }
        let inner = cache.inner.as_mut().ok_or(DrawError::RuntimeRejected)?;
        live.runtime
            .draw_with_render_cache(factory, renderer, inner)
            .map_err(|error| {
                if error.downcast_ref::<ImageDecodeError>().is_some() {
                    DrawError::ImageDecode
                } else {
                    DrawError::RuntimeRejected
                }
            })
    }
}

impl MaterializedArtboard {
    fn resolve_object_path(
        &self,
        root_artboard: ArtboardId,
        local_path: &[usize],
    ) -> Option<SceneObjectPath> {
        let mut artboard = root_artboard;
        let mut objects = Vec::with_capacity(local_path.len());
        for (index, local_id) in local_path.iter().copied().enumerate() {
            let object = self
                .objects_by_artboard_local
                .get(&artboard)?
                .get(local_id)
                .copied()
                .flatten()?;
            if index
                .checked_add(1)
                .is_some_and(|next| next < local_path.len())
            {
                artboard = *self.nested_artboard_targets.get(&object)?;
            }
            objects.push(object);
        }
        Some(SceneObjectPath { objects })
    }

    fn build(
        definitions: &Definitions,
        root: ArtboardId,
        fallback_operation_index: usize,
        origins: &SpecOrigins,
        touched_operation_index: usize,
    ) -> std::result::Result<Self, EditDiagnostic> {
        let closure =
            materialized_artboard_closure(&definitions.artboards, root, touched_operation_index)?;
        let referenced_assets = CanonicalFileAssets::collect(
            &definitions.font_assets,
            &definitions.image_assets,
            &definitions.script_assets,
            &definitions.shader_assets,
            closure.as_slice(),
        )
        .lower(fallback_operation_index, origins)?;
        let artboard_indices = artboard_indices(closure.as_slice())
            .map_err(|reason| EditDiagnostic::new(touched_operation_index, vec![], reason))?;
        let mut records = vec![backboard_record()];
        records.extend(referenced_assets.records);
        let mut root_objects = None;
        let mut root_animations = None;
        let mut root_machines = None;
        let mut root_events_by_local = None;
        let mut objects_by_artboard_local = BTreeMap::new();
        let mut nested_artboard_targets = BTreeMap::new();
        for definition in closure {
            let lowered = lower_artboard(
                definition,
                &referenced_assets.font_indices,
                &referenced_assets.image_indices,
                &referenced_assets.script_indices,
                &artboard_indices,
                fallback_operation_index,
                origins,
            )?;
            if definition.id == root {
                root_objects = Some(lowered.objects.clone());
                root_animations = Some(lowered.animations.clone());
                root_machines = Some(lowered.machines.clone());
                root_events_by_local = Some(lowered.events_by_local.clone());
            }
            objects_by_artboard_local.insert(definition.id, lowered.objects_by_local);
            nested_artboard_targets.extend(definition.records.iter().filter_map(|record| {
                let Some((_, NodeSpec::NestedArtboard(spec))) = record.visual() else {
                    return None;
                };
                Some((record.id, spec.artboard))
            }));
            records.extend(lowered.records);
        }
        let authoring_records = ExportedDocument { records }.into_authoring_records();
        let runtime = RuntimeFile::from_authoring_records(authoring_records).map_err(|_| {
            EditDiagnostic::new(
                touched_operation_index,
                vec![EditId::Artboard(root)],
                EditReason::InternalInvariant,
            )
        })?;
        // Arc is the existing owning-instance lifetime contract. A scripting
        // File is intentionally thread-affine because its Luau VM is not Send.
        #[allow(clippy::arc_with_non_send_sync)]
        let file = Arc::new(File::from_runtime(runtime).map_err(|_| {
            EditDiagnostic::new(
                touched_operation_index,
                vec![EditId::Artboard(root)],
                EditReason::InternalInvariant,
            )
        })?);
        Ok(Self {
            file,
            objects: root_objects.ok_or_else(|| {
                EditDiagnostic::new(
                    touched_operation_index,
                    vec![EditId::Artboard(root)],
                    EditReason::InternalInvariant,
                )
            })?,
            animations: root_animations.ok_or_else(|| {
                EditDiagnostic::new(
                    touched_operation_index,
                    vec![EditId::Artboard(root)],
                    EditReason::InternalInvariant,
                )
            })?,
            machines: root_machines.ok_or_else(|| {
                EditDiagnostic::new(
                    touched_operation_index,
                    vec![EditId::Artboard(root)],
                    EditReason::InternalInvariant,
                )
            })?,
            events_by_local: root_events_by_local.ok_or_else(|| {
                EditDiagnostic::new(
                    touched_operation_index,
                    vec![EditId::Artboard(root)],
                    EditReason::InternalInvariant,
                )
            })?,
            objects_by_artboard_local,
            nested_artboard_targets,
        })
    }
}

struct LoweredArtboard {
    records: Vec<ExportedRecord>,
    objects: BTreeMap<ObjectId, RuntimeSlot>,
    animations: BTreeMap<AnimationId, usize>,
    machines: BTreeMap<MachineId, usize>,
    events_by_local: Vec<Option<EventId>>,
    objects_by_local: Vec<Option<ObjectId>>,
}

fn artboard_indices(
    definitions: &[&ArtboardDefinition],
) -> std::result::Result<BTreeMap<ArtboardId, u32>, EditReason> {
    definitions
        .iter()
        .enumerate()
        .map(|(index, definition)| {
            Ok((
                definition.id,
                u32::try_from(index).map_err(|_| EditReason::CapacityExceeded)?,
            ))
        })
        .collect()
}

fn materialized_artboard_closure(
    definitions: &[ArtboardDefinition],
    root: ArtboardId,
    operation_index: usize,
) -> std::result::Result<Vec<&ArtboardDefinition>, EditDiagnostic> {
    let definitions_by_id = definitions
        .iter()
        .map(|definition| (definition.id, definition))
        .collect::<BTreeMap<_, _>>();
    let mut ordered = Vec::new();
    let mut seen = BTreeSet::new();
    let mut stack = Vec::new();
    collect_materialized_artboard_closure(
        root,
        &definitions_by_id,
        &mut seen,
        &mut stack,
        &mut ordered,
        operation_index,
    )?;
    Ok(ordered)
}

fn collect_materialized_artboard_closure<'a>(
    artboard: ArtboardId,
    definitions_by_id: &BTreeMap<ArtboardId, &'a ArtboardDefinition>,
    seen: &mut BTreeSet<ArtboardId>,
    stack: &mut Vec<ArtboardId>,
    ordered: &mut Vec<&'a ArtboardDefinition>,
    operation_index: usize,
) -> std::result::Result<(), EditDiagnostic> {
    if stack.contains(&artboard) {
        return Err(EditDiagnostic::new(
            operation_index,
            vec![EditId::Artboard(artboard)],
            EditReason::CycleDetected,
        ));
    }
    if seen.contains(&artboard) {
        return Ok(());
    }
    let definition = definitions_by_id.get(&artboard).copied().ok_or_else(|| {
        EditDiagnostic::new(
            operation_index,
            vec![EditId::Artboard(artboard)],
            EditReason::UnknownArtboard,
        )
    })?;
    stack.push(artboard);
    ordered.push(definition);
    seen.insert(artboard);
    for referenced in artboard_references(definition) {
        collect_materialized_artboard_closure(
            referenced,
            definitions_by_id,
            seen,
            stack,
            ordered,
            operation_index,
        )?;
    }
    stack.pop();
    Ok(())
}

/// One file asset in the canonical runtime record phase.
enum CanonicalFileAsset<'a> {
    Font(&'a FontAssetDefinition),
    Image(&'a ImageAssetDefinition),
    Script(&'a ScriptAssetDefinition),
    Shader(&'a ShaderAssetDefinition),
}

/// Canonical record-time view of every persistent file-asset catalog.
///
/// Rive's record-local `assetId` is one shared ordinal namespace. To preserve
/// existing font/image records while making dynamic script dependencies sound,
/// the canonical phases are: referenced fonts and images in authored-node
/// first-use order, every script in authored order, then every shader in authored
/// order. Each asset record is immediately followed by its contents record.
struct CanonicalFileAssets<'a> {
    ordered: Vec<CanonicalFileAsset<'a>>,
}

struct LoweredFileAssets {
    records: Vec<ExportedRecord>,
    font_indices: BTreeMap<FontAssetId, u32>,
    image_indices: BTreeMap<ImageAssetId, u32>,
    script_indices: BTreeMap<ScriptAssetId, u32>,
}

impl<'a> CanonicalFileAssets<'a> {
    fn collect(
        font_assets: &'a [FontAssetDefinition],
        image_assets: &'a [ImageAssetDefinition],
        script_assets: &'a [ScriptAssetDefinition],
        shader_assets: &'a [ShaderAssetDefinition],
        artboards: &[&ArtboardDefinition],
    ) -> Self {
        let fonts = font_assets
            .iter()
            .map(|font| (font.id, font))
            .collect::<BTreeMap<_, _>>();
        let images = image_assets
            .iter()
            .map(|image| (image.id, image))
            .collect::<BTreeMap<_, _>>();
        let mut seen_fonts = BTreeSet::new();
        let mut seen_images = BTreeSet::new();
        let mut ordered = Vec::new();
        for artboard in artboards {
            for record in &artboard.records {
                let Some((_, node)) = record.visual() else {
                    continue;
                };
                match node {
                    NodeSpec::TextStylePaint(style) if seen_fonts.insert(style.font) => {
                        // Unknown semantic identities remain absent so
                        // lower_artboard reports the owning object and asset
                        // together with its established diagnostic.
                        if let Some(font) = fonts.get(&style.font).copied() {
                            ordered.push(CanonicalFileAsset::Font(font));
                        }
                    }
                    NodeSpec::Image(spec) if seen_images.insert(spec.image) => {
                        if let Some(image) = images.get(&spec.image).copied() {
                            ordered.push(CanonicalFileAsset::Image(image));
                        }
                    }
                    _ => {}
                }
            }
        }
        ordered.extend(script_assets.iter().map(CanonicalFileAsset::Script));
        ordered.extend(shader_assets.iter().map(CanonicalFileAsset::Shader));
        Self { ordered }
    }

    fn lower(
        self,
        fallback_operation_index: usize,
        origins: &SpecOrigins,
    ) -> std::result::Result<LoweredFileAssets, EditDiagnostic> {
        let record_capacity = self.ordered.len().checked_mul(2).ok_or_else(|| {
            EditDiagnostic::new(
                fallback_operation_index,
                Vec::new(),
                EditReason::CapacityExceeded,
            )
        })?;
        let mut records = Vec::with_capacity(record_capacity);
        let mut font_indices = BTreeMap::new();
        let mut image_indices = BTreeMap::new();
        let mut script_indices = BTreeMap::new();
        for (index, asset) in self.ordered.into_iter().enumerate() {
            let file_asset_id = u32::try_from(index).map_err(|_| {
                let (operation_index, involved_ids) = match &asset {
                    CanonicalFileAsset::Font(font) => (
                        origins.font_asset(font.id, fallback_operation_index),
                        vec![EditId::FontAsset(font.id)],
                    ),
                    CanonicalFileAsset::Image(image) => (
                        origins.image_asset(image.id, fallback_operation_index),
                        vec![EditId::ImageAsset(image.id)],
                    ),
                    CanonicalFileAsset::Script(script) => (
                        origins.script_asset(script.id, fallback_operation_index),
                        vec![EditId::ScriptAsset(script.id)],
                    ),
                    CanonicalFileAsset::Shader(shader) => (
                        origins.shader_asset(shader.id, fallback_operation_index),
                        vec![EditId::ShaderAsset(shader.id)],
                    ),
                };
                EditDiagnostic::new(operation_index, involved_ids, EditReason::CapacityExceeded)
            })?;
            let (kind, name, bytes, is_module) = match asset {
                CanonicalFileAsset::Font(font) => {
                    validate_font_asset(font, fallback_operation_index, origins)?;
                    let operation_index = origins.font_asset(font.id, fallback_operation_index);
                    if font_indices.insert(font.id, file_asset_id).is_some() {
                        return Err(EditDiagnostic::new(
                            operation_index,
                            vec![EditId::FontAsset(font.id)],
                            EditReason::IdentityCollision,
                        ));
                    }
                    (
                        ExportedObjectKind::FontAsset,
                        font.spec.name.clone(),
                        font.spec.bytes.clone(),
                        None,
                    )
                }
                CanonicalFileAsset::Image(image) => {
                    let operation_index = origins.image_asset(image.id, fallback_operation_index);
                    if image_indices.insert(image.id, file_asset_id).is_some() {
                        return Err(EditDiagnostic::new(
                            operation_index,
                            vec![EditId::ImageAsset(image.id)],
                            EditReason::IdentityCollision,
                        ));
                    }
                    (
                        ExportedObjectKind::ImageAsset,
                        image.spec.name.clone(),
                        image.spec.bytes.clone(),
                        None,
                    )
                }
                CanonicalFileAsset::Script(script) => {
                    let operation_index = origins.script_asset(script.id, fallback_operation_index);
                    if script_indices.insert(script.id, file_asset_id).is_some() {
                        return Err(EditDiagnostic::new(
                            operation_index,
                            vec![EditId::ScriptAsset(script.id)],
                            EditReason::IdentityCollision,
                        ));
                    }
                    let mut bytes = script.spec.bytes.clone();
                    bytes.insert(0, 0);
                    (
                        ExportedObjectKind::ScriptAsset,
                        script.spec.name.clone(),
                        bytes,
                        Some(script.spec.is_module),
                    )
                }
                CanonicalFileAsset::Shader(shader) => {
                    let mut bytes = shader.spec.bytes.clone();
                    bytes.insert(0, 0);
                    (
                        ExportedObjectKind::ShaderAsset,
                        shader.spec.name.clone(),
                        bytes,
                        None,
                    )
                }
            };
            let mut properties = vec![
                ExportedProperty::AssetName(name),
                ExportedProperty::FileAssetId(file_asset_id),
            ];
            if is_module == Some(true) {
                properties.push(ExportedProperty::ScriptAssetIsModule(true));
            }
            records.push(ExportedRecord { kind, properties });
            records.push(ExportedRecord {
                kind: ExportedObjectKind::FileAssetContents,
                properties: vec![ExportedProperty::FileAssetContentsBytes(bytes)],
            });
        }
        Ok(LoweredFileAssets {
            records,
            font_indices,
            image_indices,
            script_indices,
        })
    }
}

fn validate_font_assets(
    font_assets: &[FontAssetDefinition],
    fallback_operation_index: usize,
    origins: &SpecOrigins,
) -> std::result::Result<(), EditDiagnostic> {
    let mut identities = BTreeSet::new();
    for (index, font) in font_assets.iter().enumerate() {
        validate_font_asset(font, fallback_operation_index, origins)?;
        let operation_index = origins.font_asset(font.id, fallback_operation_index);
        u32::try_from(index).map_err(|_| {
            EditDiagnostic::new(
                operation_index,
                vec![EditId::FontAsset(font.id)],
                EditReason::CapacityExceeded,
            )
        })?;
        if !identities.insert(font.id) {
            return Err(EditDiagnostic::new(
                operation_index,
                vec![EditId::FontAsset(font.id)],
                EditReason::IdentityCollision,
            ));
        }
    }
    Ok(())
}

fn validate_font_asset(
    font: &FontAssetDefinition,
    fallback_operation_index: usize,
    origins: &SpecOrigins,
) -> std::result::Result<(), EditDiagnostic> {
    let operation_index = origins.font_asset(font.id, fallback_operation_index);
    if font.spec.bytes.is_empty() {
        return Err(EditDiagnostic::new(
            operation_index,
            vec![EditId::FontAsset(font.id)],
            EditReason::EmptyFontAsset,
        ));
    }
    if !embedded_font_is_parseable(&font.spec.bytes) {
        return Err(EditDiagnostic::new(
            operation_index,
            vec![EditId::FontAsset(font.id)],
            EditReason::InvalidFontAsset,
        ));
    }
    Ok(())
}

fn validate_image_assets(
    image_assets: &[ImageAssetDefinition],
    fallback_operation_index: usize,
    origins: &SpecOrigins,
) -> std::result::Result<(), EditDiagnostic> {
    let mut identities = BTreeSet::new();
    for (index, image) in image_assets.iter().enumerate() {
        let operation_index = origins.image_asset(image.id, fallback_operation_index);
        u32::try_from(index).map_err(|_| {
            EditDiagnostic::new(
                operation_index,
                vec![EditId::ImageAsset(image.id)],
                EditReason::CapacityExceeded,
            )
        })?;
        if !identities.insert(image.id) {
            return Err(EditDiagnostic::new(
                operation_index,
                vec![EditId::ImageAsset(image.id)],
                EditReason::IdentityCollision,
            ));
        }
    }
    Ok(())
}

fn validate_script_assets(
    script_assets: &[ScriptAssetDefinition],
    fallback_operation_index: usize,
    origins: &SpecOrigins,
) -> std::result::Result<(), EditDiagnostic> {
    let mut identities = BTreeSet::new();
    for (index, script) in script_assets.iter().enumerate() {
        let operation_index = origins.script_asset(script.id, fallback_operation_index);
        u32::try_from(index).map_err(|_| {
            EditDiagnostic::new(
                operation_index,
                vec![EditId::ScriptAsset(script.id)],
                EditReason::CapacityExceeded,
            )
        })?;
        if !identities.insert(script.id) {
            return Err(EditDiagnostic::new(
                operation_index,
                vec![EditId::ScriptAsset(script.id)],
                EditReason::IdentityCollision,
            ));
        }
    }
    Ok(())
}

fn validate_shader_assets(
    shader_assets: &[ShaderAssetDefinition],
    fallback_operation_index: usize,
    origins: &SpecOrigins,
) -> std::result::Result<(), EditDiagnostic> {
    let mut identities = BTreeSet::new();
    for (index, shader) in shader_assets.iter().enumerate() {
        let operation_index = origins.shader_asset(shader.id, fallback_operation_index);
        u32::try_from(index).map_err(|_| {
            EditDiagnostic::new(
                operation_index,
                vec![EditId::ShaderAsset(shader.id)],
                EditReason::CapacityExceeded,
            )
        })?;
        if !identities.insert(shader.id) {
            return Err(EditDiagnostic::new(
                operation_index,
                vec![EditId::ShaderAsset(shader.id)],
                EditReason::IdentityCollision,
            ));
        }
    }
    Ok(())
}

fn validate_animation_definitions(
    definitions: &Definitions,
    fallback_operation_index: usize,
    origins: &SpecOrigins,
) -> std::result::Result<(), EditDiagnostic> {
    let mut identities = BTreeSet::new();
    let mut objects = BTreeMap::new();
    for artboard in &definitions.artboards {
        for record in &artboard.records {
            if !identities.insert(record.id) {
                return Err(EditDiagnostic::new(
                    origins.object(record.id, fallback_operation_index),
                    vec![EditId::Object(record.id)],
                    EditReason::IdentityCollision,
                ));
            }
            objects.insert(record.id, (artboard.id, record.spec.kind()));
        }
    }

    for artboard in &definitions.artboards {
        let visual_kinds = artboard
            .records
            .iter()
            .filter_map(|record| record.visual().map(|(_, node)| (record.id, node.kind())))
            .collect::<BTreeMap<_, _>>();
        let records_by_id = artboard
            .records
            .iter()
            .map(|record| (record.id, record))
            .collect::<BTreeMap<_, _>>();
        let mut keyed_targets = BTreeSet::new();
        let mut keyed_properties = BTreeSet::new();
        let mut keyed_frames = BTreeSet::new();
        for record in &artboard.records {
            let Some(animation_record) = record.animation() else {
                continue;
            };
            let operation_index = origins.object(record.id, fallback_operation_index);
            match animation_record {
                AnimationRecordSpec::LinearAnimation(spec) => {
                    let operation_index = origins.property(record.id, "fps", operation_index);
                    validate_linear_animation_spec(spec).map_err(|reason| {
                        EditDiagnostic::new(
                            operation_index,
                            vec![EditId::Object(record.id)],
                            reason,
                        )
                    })?;
                }
                AnimationRecordSpec::KeyedObject { animation, target } => {
                    let owner = records_by_id.get(animation).copied().ok_or_else(|| {
                        let reason = match objects.get(animation) {
                            Some((owner_artboard, _)) if *owner_artboard != artboard.id => {
                                EditReason::CrossArtboardReference {
                                    source: artboard.id,
                                    target: *owner_artboard,
                                }
                            }
                            _ => EditReason::UnknownObject,
                        };
                        EditDiagnostic::new(
                            operation_index,
                            vec![EditId::Object(record.id), EditId::Object(*animation)],
                            reason,
                        )
                    })?;
                    if !matches!(
                        &owner.spec,
                        RecordSpec::Animation(AnimationRecordSpec::LinearAnimation(_))
                    ) {
                        return Err(EditDiagnostic::new(
                            operation_index,
                            vec![EditId::Object(record.id), EditId::Object(*animation)],
                            EditReason::InternalInvariant,
                        ));
                    }
                    if !keyed_targets.insert((*animation, *target)) {
                        return Err(EditDiagnostic::new(
                            operation_index,
                            vec![EditId::Object(record.id)],
                            EditReason::IdentityCollision,
                        ));
                    }
                    if !visual_kinds.contains_key(target) {
                        let reason = match objects.get(target) {
                            Some((target_artboard, _)) if *target_artboard != artboard.id => {
                                EditReason::CrossArtboardReference {
                                    source: artboard.id,
                                    target: *target_artboard,
                                }
                            }
                            Some((_, AuthoredObjectKind::Visual(_))) => {
                                EditReason::InternalInvariant
                            }
                            Some((_, _)) => EditReason::NonVisualObject,
                            None => EditReason::UnknownObject,
                        };
                        return Err(EditDiagnostic::new(
                            operation_index,
                            vec![EditId::Object(record.id), EditId::Object(*target)],
                            reason,
                        ));
                    }
                }
                AnimationRecordSpec::KeyedProperty {
                    keyed_object,
                    property,
                } => {
                    let owner = records_by_id.get(keyed_object).copied().ok_or_else(|| {
                        let reason = match objects.get(keyed_object) {
                            Some((owner_artboard, _)) if *owner_artboard != artboard.id => {
                                EditReason::CrossArtboardReference {
                                    source: artboard.id,
                                    target: *owner_artboard,
                                }
                            }
                            _ => EditReason::UnknownObject,
                        };
                        EditDiagnostic::new(
                            operation_index,
                            vec![EditId::Object(record.id), EditId::Object(*keyed_object)],
                            reason,
                        )
                    })?;
                    let RecordSpec::Animation(AnimationRecordSpec::KeyedObject { target, .. }) =
                        &owner.spec
                    else {
                        return Err(EditDiagnostic::new(
                            operation_index,
                            vec![EditId::Object(record.id), EditId::Object(*keyed_object)],
                            EditReason::InternalInvariant,
                        ));
                    };
                    if !keyed_properties.insert((*keyed_object, *property)) {
                        return Err(EditDiagnostic::new(
                            operation_index,
                            vec![EditId::Object(record.id)],
                            EditReason::IdentityCollision,
                        ));
                    }
                    let target_kind = visual_kinds.get(target).copied().ok_or_else(|| {
                        let reason = match objects.get(target) {
                            Some((target_artboard, _)) if *target_artboard != artboard.id => {
                                EditReason::CrossArtboardReference {
                                    source: artboard.id,
                                    target: *target_artboard,
                                }
                            }
                            Some((_, AuthoredObjectKind::Visual(_))) => {
                                EditReason::InternalInvariant
                            }
                            Some((_, _)) => EditReason::NonVisualObject,
                            None => EditReason::UnknownObject,
                        };
                        EditDiagnostic::new(
                            operation_index,
                            vec![EditId::Object(record.id), EditId::Object(*target)],
                            reason,
                        )
                    })?;
                    if !property.is_available_on(target_kind) {
                        return Err(EditDiagnostic::new(
                            operation_index,
                            vec![EditId::Object(record.id), EditId::Object(*target)],
                            EditReason::PropertyOwnerMismatch {
                                property: "animated_property",
                                actual: target_kind,
                            },
                        ));
                    }
                }
                AnimationRecordSpec::KeyFrameDouble {
                    keyed_property,
                    frame,
                    value,
                } => {
                    let owner = records_by_id.get(keyed_property).copied().ok_or_else(|| {
                        let reason = match objects.get(keyed_property) {
                            Some((owner_artboard, _)) if *owner_artboard != artboard.id => {
                                EditReason::CrossArtboardReference {
                                    source: artboard.id,
                                    target: *owner_artboard,
                                }
                            }
                            _ => EditReason::UnknownObject,
                        };
                        EditDiagnostic::new(
                            operation_index,
                            vec![EditId::Object(record.id), EditId::Object(*keyed_property)],
                            reason,
                        )
                    })?;
                    if !matches!(
                        &owner.spec,
                        RecordSpec::Animation(AnimationRecordSpec::KeyedProperty { .. })
                    ) {
                        return Err(EditDiagnostic::new(
                            operation_index,
                            vec![EditId::Object(record.id), EditId::Object(*keyed_property)],
                            EditReason::InternalInvariant,
                        ));
                    }
                    if !keyed_frames.insert((*keyed_property, *frame)) {
                        return Err(EditDiagnostic::new(
                            operation_index,
                            vec![EditId::Object(record.id)],
                            EditReason::IdentityCollision,
                        ));
                    }
                    if !value.is_finite() {
                        return Err(EditDiagnostic::new(
                            operation_index,
                            vec![EditId::Object(record.id)],
                            EditReason::NonFiniteProperty {
                                property: "key_frame_value",
                            },
                        ));
                    }
                }
            }
        }
    }
    Ok(())
}

fn validate_machine_definitions(
    definitions: &Definitions,
    fallback_operation_index: usize,
    origins: &SpecOrigins,
) -> std::result::Result<(), EditDiagnostic> {
    for artboard in &definitions.artboards {
        let mut owned = BTreeMap::<ObjectId, Vec<&RecordDefinition>>::new();
        for record in &artboard.records {
            let RecordSpec::Machine(spec) = &record.spec else {
                continue;
            };
            if let Some(owner) = spec.owner() {
                owned.entry(owner).or_default().push(record);
            }
        }

        for machine in &artboard.records {
            if !matches!(
                machine.spec,
                RecordSpec::Machine(MachineRecordSpec::Machine(_))
            ) {
                continue;
            }
            let layers = owned
                .get(&machine.id)
                .into_iter()
                .flatten()
                .copied()
                .filter(|record| {
                    matches!(
                        record.spec,
                        RecordSpec::Machine(MachineRecordSpec::Layer { .. })
                    )
                })
                .collect::<Vec<_>>();
            if layers.is_empty() {
                return Err(EditDiagnostic::new(
                    origins.object(machine.id, fallback_operation_index),
                    vec![EditId::Object(machine.id)],
                    EditReason::InvalidMachineTopology {
                        requirement: "at least one state-machine layer",
                        actual: 0,
                    },
                ));
            }

            for layer in layers {
                let states = owned
                    .get(&layer.id)
                    .into_iter()
                    .flatten()
                    .copied()
                    .filter(|record| {
                        matches!(
                            record.spec,
                            RecordSpec::Machine(
                                MachineRecordSpec::AnyState { .. }
                                    | MachineRecordSpec::EntryState { .. }
                                    | MachineRecordSpec::ExitState { .. }
                                    | MachineRecordSpec::AnimationState { .. }
                            )
                        )
                    })
                    .collect::<Vec<_>>();
                for (requirement, predicate) in [
                    (
                        "exactly one entry state per layer",
                        MachineRecordSpec::is_entry_state as fn(&MachineRecordSpec) -> bool,
                    ),
                    (
                        "exactly one any state per layer",
                        MachineRecordSpec::is_any_state as fn(&MachineRecordSpec) -> bool,
                    ),
                    (
                        "exactly one exit state per layer",
                        MachineRecordSpec::is_exit_state as fn(&MachineRecordSpec) -> bool,
                    ),
                ] {
                    let matching = states
                        .iter()
                        .copied()
                        .filter(|record| {
                            let RecordSpec::Machine(spec) = &record.spec else {
                                unreachable!("state inventory contains only machine records")
                            };
                            predicate(spec)
                        })
                        .collect::<Vec<_>>();
                    if matching.len() == 1 {
                        continue;
                    }
                    let culprit = matching.last().copied().unwrap_or(layer);
                    let involved_ids = std::iter::once(layer)
                        .chain(matching.iter().copied())
                        .map(|record| EditId::Object(record.id))
                        .collect();
                    return Err(EditDiagnostic::new(
                        origins.object(culprit.id, fallback_operation_index),
                        involved_ids,
                        EditReason::InvalidMachineTopology {
                            requirement,
                            actual: matching.len(),
                        },
                    ));
                }
            }
        }
    }
    Ok(())
}

/// Lower exactly one durable artboard into one runtime-file record stream.
///
/// Preview materialization uses this function today; deterministic export can reuse the same
/// lowering without reconstructing the whole live scene. Parent resolution is deliberately local
/// to the artboard, which hard-gates the current vocabulary against cross-artboard references.
fn lower_artboard(
    artboard: &ArtboardDefinition,
    font_asset_indices: &BTreeMap<FontAssetId, u32>,
    image_asset_indices: &BTreeMap<ImageAssetId, u32>,
    script_asset_indices: &BTreeMap<ScriptAssetId, u32>,
    artboard_indices: &BTreeMap<ArtboardId, u32>,
    fallback_operation_index: usize,
    origins: &SpecOrigins,
) -> std::result::Result<LoweredArtboard, EditDiagnostic> {
    validate_artboard_spec(&artboard.spec).map_err(|reason| {
        EditDiagnostic::new(
            origins.artboard(artboard.id, fallback_operation_index),
            vec![EditId::Artboard(artboard.id)],
            reason,
        )
    })?;

    let mut records = vec![artboard_record(&artboard.spec)];
    let mut all_kinds = BTreeMap::new();
    let mut all_parents = BTreeMap::new();
    let mut all_local_ids = BTreeMap::new();
    for (node_index, node) in artboard.visual_records().enumerate() {
        if all_kinds.insert(node.id, node.spec.kind()).is_some() {
            return Err(EditDiagnostic::new(
                origins.object(node.id, fallback_operation_index),
                vec![EditId::Object(node.id)],
                EditReason::IdentityCollision,
            ));
        }
        all_parents.insert(node.id, node.parent);
        let local_id = node_index.checked_add(1).ok_or_else(|| {
            EditDiagnostic::new(
                origins.object(node.id, fallback_operation_index),
                vec![EditId::Object(node.id)],
                EditReason::CapacityExceeded,
            )
        })?;
        all_local_ids.insert(node.id, local_id);
    }

    let mut local_ids = BTreeMap::new();
    let mut objects = BTreeMap::new();
    let mut objects_by_local = vec![None];
    for (node_index, node) in artboard.visual_records().enumerate() {
        let local_id = node_index.checked_add(1).ok_or_else(|| {
            EditDiagnostic::new(
                origins.object(node.id, fallback_operation_index),
                vec![EditId::Object(node.id)],
                EditReason::CapacityExceeded,
            )
        })?;
        validate_node_spec(node.spec).map_err(|reason| {
            EditDiagnostic::new(
                origins.object(node.id, fallback_operation_index),
                vec![EditId::Object(node.id)],
                reason,
            )
        })?;
        match node.spec {
            NodeSpec::TextValueRun(spec) => {
                let actual = all_kinds.get(&spec.style).copied();
                if actual != Some(NodeKind::TextStylePaint)
                    || all_parents.get(&spec.style).copied() != Some(node.parent)
                {
                    return Err(EditDiagnostic::new(
                        origins.relationship(node.id, spec.style, fallback_operation_index),
                        vec![EditId::Object(node.id), EditId::Object(spec.style)],
                        if actual.is_none() {
                            EditReason::UnknownObject
                        } else {
                            EditReason::InvalidReference {
                                expected: NodeKind::TextStylePaint,
                                actual,
                            }
                        },
                    ));
                }
            }
            NodeSpec::TextStylePaint(spec) => {
                if !font_asset_indices.contains_key(&spec.font) {
                    return Err(EditDiagnostic::new(
                        origins.object(node.id, fallback_operation_index),
                        vec![EditId::Object(node.id), EditId::FontAsset(spec.font)],
                        EditReason::UnknownFontAsset,
                    ));
                }
            }
            NodeSpec::NestedArtboard(spec) => {
                if !artboard_indices.contains_key(&spec.artboard) {
                    return Err(EditDiagnostic::new(
                        origins.object(node.id, fallback_operation_index),
                        vec![EditId::Object(node.id), EditId::Artboard(spec.artboard)],
                        EditReason::UnknownArtboard,
                    ));
                }
            }
            NodeSpec::Image(spec) => {
                if !image_asset_indices.contains_key(&spec.image) {
                    return Err(EditDiagnostic::new(
                        origins.object(node.id, fallback_operation_index),
                        vec![EditId::Object(node.id), EditId::ImageAsset(spec.image)],
                        EditReason::UnknownImageAsset,
                    ));
                }
            }
            NodeSpec::ScriptedDrawable(spec) => {
                if !script_asset_indices.contains_key(&spec.script) {
                    return Err(EditDiagnostic::new(
                        origins.object(node.id, fallback_operation_index),
                        vec![EditId::Object(node.id), EditId::ScriptAsset(spec.script)],
                        EditReason::UnknownScriptAsset,
                    ));
                }
            }
            _ => {}
        }
        let parent_id = match node.parent {
            Parent::Artboard(parent)
                if parent == artboard.id && valid_artboard_child(node.spec.kind()) =>
            {
                0
            }
            Parent::Artboard(parent) if parent == artboard.id => {
                return Err(EditDiagnostic::new(
                    origins.object(node.id, fallback_operation_index),
                    vec![EditId::Object(node.id), EditId::Artboard(parent)],
                    EditReason::InvalidParent {
                        parent: None,
                        child: node.spec.kind(),
                    },
                ));
            }
            Parent::Artboard(parent) => {
                return Err(EditDiagnostic::new(
                    origins.object(node.id, fallback_operation_index),
                    vec![EditId::Object(node.id), EditId::Artboard(parent)],
                    EditReason::InvalidParent {
                        parent: None,
                        child: node.spec.kind(),
                    },
                ));
            }
            Parent::Object(parent) => {
                let Some(parent_id) = local_ids.get(&parent).copied() else {
                    let reason = if all_kinds.contains_key(&parent) {
                        EditReason::InternalInvariant
                    } else {
                        EditReason::InvalidParent {
                            parent: None,
                            child: node.spec.kind(),
                        }
                    };
                    return Err(EditDiagnostic::new(
                        origins.object(node.id, fallback_operation_index),
                        vec![EditId::Object(node.id), EditId::Object(parent)],
                        reason,
                    ));
                };
                let parent_kind = all_kinds.get(&parent).copied().ok_or_else(|| {
                    EditDiagnostic::new(
                        origins.object(node.id, fallback_operation_index),
                        vec![EditId::Object(node.id), EditId::Object(parent)],
                        EditReason::InternalInvariant,
                    )
                })?;
                if !valid_object_parent(parent_kind, node.spec.kind()) {
                    return Err(EditDiagnostic::new(
                        origins.object(node.id, fallback_operation_index),
                        vec![EditId::Object(node.id), EditId::Object(parent)],
                        EditReason::InvalidParent {
                            parent: Some(parent_kind),
                            child: node.spec.kind(),
                        },
                    ));
                }
                parent_id
            }
        };
        records.push(
            node_record(
                node,
                parent_id,
                &all_local_ids,
                font_asset_indices,
                image_asset_indices,
                script_asset_indices,
                artboard_indices,
            )
            .map_err(|reason| {
                EditDiagnostic::new(
                    origins.object(node.id, fallback_operation_index),
                    vec![EditId::Object(node.id)],
                    reason,
                )
            })?,
        );
        if local_ids.insert(node.id, local_id).is_some()
            || objects
                .insert(
                    node.id,
                    RuntimeSlot {
                        local_id,
                        kind: node.spec.kind(),
                    },
                )
                .is_some()
        {
            return Err(EditDiagnostic::new(
                origins.object(node.id, fallback_operation_index),
                vec![EditId::Object(node.id)],
                EditReason::IdentityCollision,
            ));
        }
        if objects_by_local.len() != local_id {
            return Err(EditDiagnostic::new(
                origins.object(node.id, fallback_operation_index),
                vec![EditId::Object(node.id)],
                EditReason::InternalInvariant,
            ));
        }
        objects_by_local.push(Some(node.id));
    }

    for node in artboard.visual_records() {
        let NodeSpec::Image(spec) = node.spec else {
            continue;
        };
        let Some(crop) = spec.crop else {
            continue;
        };
        if !image_crop_requires_mesh(crop) {
            continue;
        }
        let image_local_id = local_ids.get(&node.id).copied().ok_or_else(|| {
            EditDiagnostic::new(
                origins.object(node.id, fallback_operation_index),
                vec![EditId::Object(node.id)],
                EditReason::InternalInvariant,
            )
        })?;
        append_image_crop_mesh_records(
            &mut records,
            &mut objects_by_local,
            image_local_id,
            crop,
            fallback_operation_index,
            artboard.id,
        )?;
    }

    let mut events_by_local = vec![None; objects_by_local.len()];
    let event_local_ids = append_event_export_records(
        &mut records,
        &mut objects_by_local,
        &mut events_by_local,
        artboard,
        fallback_operation_index,
        origins,
    )?;

    let synthetic_local_count = objects_by_local
        .len()
        .checked_sub(
            artboard
                .visual_record_count()
                .checked_add(1)
                .ok_or_else(|| {
                    EditDiagnostic::new(
                        fallback_operation_index,
                        vec![EditId::Artboard(artboard.id)],
                        EditReason::CapacityExceeded,
                    )
                })?,
        )
        .ok_or_else(|| {
            EditDiagnostic::new(
                fallback_operation_index,
                vec![EditId::Artboard(artboard.id)],
                EditReason::InternalInvariant,
            )
        })?
        .checked_sub(event_local_ids.len())
        .ok_or_else(|| {
            EditDiagnostic::new(
                fallback_operation_index,
                vec![EditId::Artboard(artboard.id)],
                EditReason::InternalInvariant,
            )
        })?;
    let exact_record_count = artboard
        .visual_record_count()
        .checked_add(1)
        .and_then(|count| count.checked_add(synthetic_local_count))
        .and_then(|count| count.checked_add(event_local_ids.len()))
        .ok_or_else(|| {
            EditDiagnostic::new(
                fallback_operation_index,
                vec![EditId::Artboard(artboard.id)],
                EditReason::CapacityExceeded,
            )
        })?;
    let exact_local_count = exact_record_count;
    if records.len() != exact_record_count
        || objects.len() != artboard.visual_record_count()
        || objects_by_local.len() != exact_local_count
    {
        return Err(EditDiagnostic::new(
            fallback_operation_index,
            vec![EditId::Artboard(artboard.id)],
            EditReason::InternalInvariant,
        ));
    }

    let animations = append_animation_export_records(
        &mut records,
        artboard,
        &all_local_ids,
        fallback_operation_index,
        origins,
    )?;
    let machines = append_machine_export_records(
        &mut records,
        artboard,
        &animations,
        &event_local_ids,
        fallback_operation_index,
        origins,
    )?;
    canonicalize_exported_records(&mut records);
    Ok(LoweredArtboard {
        records,
        objects,
        animations,
        machines,
        events_by_local,
        objects_by_local,
    })
}

fn append_event_export_records(
    records: &mut Vec<ExportedRecord>,
    objects_by_local: &mut Vec<Option<ObjectId>>,
    events_by_local: &mut Vec<Option<EventId>>,
    artboard: &ArtboardDefinition,
    fallback_operation_index: usize,
    origins: &SpecOrigins,
) -> std::result::Result<BTreeMap<EventId, usize>, EditDiagnostic> {
    let mut event_local_ids = BTreeMap::new();
    for (record, spec) in artboard.machine_views() {
        let MachineRecordSpec::Event(spec) = spec else {
            continue;
        };
        let local_id = objects_by_local.len();
        let event = EventId(record.id);
        if event_local_ids.insert(event, local_id).is_some() {
            return Err(EditDiagnostic::new(
                origins.object(record.id, fallback_operation_index),
                vec![EditId::Object(record.id)],
                EditReason::IdentityCollision,
            ));
        }
        records.push(ExportedRecord {
            kind: ExportedObjectKind::Event,
            properties: spec
                .name
                .iter()
                .cloned()
                .map(ExportedProperty::ComponentName)
                .collect(),
        });
        objects_by_local.push(None);
        events_by_local.push(Some(event));
    }
    Ok(event_local_ids)
}

fn append_machine_export_records(
    records: &mut Vec<ExportedRecord>,
    artboard: &ArtboardDefinition,
    animation_indices: &BTreeMap<AnimationId, usize>,
    event_local_ids: &BTreeMap<EventId, usize>,
    fallback_operation_index: usize,
    origins: &SpecOrigins,
) -> std::result::Result<BTreeMap<MachineId, usize>, EditDiagnostic> {
    let mut owned = BTreeMap::<ObjectId, Vec<(&RecordDefinition, &MachineRecordSpec)>>::new();
    let mut machines = Vec::new();
    for (record, spec) in artboard.machine_views() {
        if let MachineRecordSpec::Machine(spec) = spec {
            machines.push((record, spec));
        }
        if let Some(owner) = spec.owner() {
            owned.entry(owner).or_default().push((record, spec));
        }
    }

    let mut machine_indices = BTreeMap::new();
    for (machine_index, (machine, spec)) in machines.into_iter().enumerate() {
        let machine_id = MachineId(machine.id);
        if machine_indices.insert(machine_id, machine_index).is_some() {
            return Err(EditDiagnostic::new(
                origins.object(machine.id, fallback_operation_index),
                vec![EditId::Object(machine.id)],
                EditReason::IdentityCollision,
            ));
        }
        records.push(ExportedRecord {
            kind: ExportedObjectKind::StateMachine,
            properties: spec
                .name
                .iter()
                .cloned()
                .map(ExportedProperty::AnimationName)
                .collect(),
        });

        let inputs = owned
            .get(&machine.id)
            .into_iter()
            .flatten()
            .filter_map(|(record, spec)| match spec {
                MachineRecordSpec::TriggerInput {
                    machine: owner,
                    spec,
                } if *owner == machine.id => Some((*record, spec)),
                _ => None,
            })
            .collect::<Vec<_>>();
        let mut input_names = BTreeMap::<&str, ObjectId>::new();
        for (input, spec) in &inputs {
            if let Some(existing) = input_names.insert(spec.name.as_str(), input.id) {
                return Err(EditDiagnostic::new(
                    origins.object(input.id, fallback_operation_index),
                    vec![
                        EditId::Object(machine.id),
                        EditId::Object(existing),
                        EditId::Object(input.id),
                    ],
                    EditReason::DuplicateMachineInputName,
                ));
            }
        }
        let input_indices = inputs
            .iter()
            .enumerate()
            .map(|(index, (record, _))| (MachineInputId(record.id), index))
            .collect::<BTreeMap<_, _>>();
        for (_, input) in inputs {
            records.push(ExportedRecord {
                kind: ExportedObjectKind::StateMachineTrigger,
                properties: vec![ExportedProperty::StateMachineComponentName(
                    input.name.clone(),
                )],
            });
        }

        let layers = owned
            .get(&machine.id)
            .into_iter()
            .flatten()
            .filter_map(|(record, spec)| match spec {
                MachineRecordSpec::Layer {
                    machine: owner,
                    spec,
                } if *owner == machine.id => Some((*record, spec)),
                _ => None,
            })
            .collect::<Vec<_>>();
        for (layer, layer_spec) in layers {
            records.push(ExportedRecord {
                kind: ExportedObjectKind::StateMachineLayer,
                properties: layer_spec
                    .name
                    .iter()
                    .cloned()
                    .map(ExportedProperty::StateMachineComponentName)
                    .collect(),
            });
            let states = owned
                .get(&layer.id)
                .into_iter()
                .flatten()
                .filter(|(_, spec)| {
                    matches!(
                        spec,
                        MachineRecordSpec::AnyState { .. }
                            | MachineRecordSpec::EntryState { .. }
                            | MachineRecordSpec::ExitState { .. }
                            | MachineRecordSpec::AnimationState { .. }
                    )
                })
                .copied()
                .collect::<Vec<_>>();
            let state_indices = states
                .iter()
                .enumerate()
                .map(|(index, (record, _))| (record.id, index))
                .collect::<BTreeMap<_, _>>();
            for (state, state_spec) in states {
                let (kind, properties) = match state_spec {
                    MachineRecordSpec::AnyState { .. } => {
                        (ExportedObjectKind::AnyState, Vec::new())
                    }
                    MachineRecordSpec::EntryState { .. } => {
                        (ExportedObjectKind::EntryState, Vec::new())
                    }
                    MachineRecordSpec::ExitState { .. } => {
                        (ExportedObjectKind::ExitState, Vec::new())
                    }
                    MachineRecordSpec::AnimationState { animation, .. } => {
                        let animation_index =
                            animation_indices.get(animation).copied().ok_or_else(|| {
                                EditDiagnostic::new(
                                    origins.object(state.id, fallback_operation_index),
                                    vec![
                                        EditId::Object(state.id),
                                        EditId::Object(animation.object_id()),
                                    ],
                                    EditReason::UnknownObject,
                                )
                            })?;
                        let animation_index = u32::try_from(animation_index).map_err(|_| {
                            EditDiagnostic::new(
                                origins.object(state.id, fallback_operation_index),
                                vec![EditId::Object(state.id)],
                                EditReason::CapacityExceeded,
                            )
                        })?;
                        (
                            ExportedObjectKind::AnimationState,
                            vec![
                                ExportedProperty::StateAnimationId(animation_index),
                                ExportedProperty::StateSpeed(1.0),
                            ],
                        )
                    }
                    _ => {
                        return Err(EditDiagnostic::new(
                            origins.object(state.id, fallback_operation_index),
                            vec![EditId::Object(state.id)],
                            EditReason::InternalInvariant,
                        ));
                    }
                };
                records.push(ExportedRecord { kind, properties });

                append_machine_fire_events(
                    records,
                    state.id,
                    &owned,
                    event_local_ids,
                    fallback_operation_index,
                    origins,
                )?;

                for (transition, transition_spec) in owned
                    .get(&state.id)
                    .into_iter()
                    .flatten()
                    .filter(|(_, spec)| matches!(spec, MachineRecordSpec::Transition { .. }))
                    .copied()
                {
                    let MachineRecordSpec::Transition { source, target } = transition_spec else {
                        unreachable!("filtered transition records")
                    };
                    if *source != state.id {
                        return Err(EditDiagnostic::new(
                            origins.object(transition.id, fallback_operation_index),
                            vec![EditId::Object(transition.id)],
                            EditReason::InternalInvariant,
                        ));
                    }
                    let target_index = state_indices.get(target).copied().ok_or_else(|| {
                        EditDiagnostic::new(
                            origins.object(transition.id, fallback_operation_index),
                            vec![EditId::Object(transition.id), EditId::Object(*target)],
                            EditReason::UnknownObject,
                        )
                    })?;
                    let target_index = u32::try_from(target_index).map_err(|_| {
                        EditDiagnostic::new(
                            origins.object(transition.id, fallback_operation_index),
                            vec![EditId::Object(transition.id)],
                            EditReason::CapacityExceeded,
                        )
                    })?;
                    records.push(ExportedRecord {
                        kind: ExportedObjectKind::StateTransition,
                        properties: vec![
                            ExportedProperty::StateToId(target_index),
                            ExportedProperty::StateTransitionFlags(0),
                            ExportedProperty::StateTransitionDuration(0),
                            ExportedProperty::StateTransitionExitTime(0),
                            ExportedProperty::StateTransitionRandomWeight(1),
                        ],
                    });
                    for (condition, condition_spec) in owned
                        .get(&transition.id)
                        .into_iter()
                        .flatten()
                        .filter(|(_, spec)| {
                            matches!(spec, MachineRecordSpec::TriggerCondition { .. })
                        })
                        .copied()
                    {
                        let MachineRecordSpec::TriggerCondition {
                            transition: owner,
                            input,
                        } = condition_spec
                        else {
                            unreachable!("filtered trigger conditions")
                        };
                        if *owner != transition.id {
                            return Err(EditDiagnostic::new(
                                origins.object(condition.id, fallback_operation_index),
                                vec![EditId::Object(condition.id)],
                                EditReason::InternalInvariant,
                            ));
                        }
                        let input_index = input_indices.get(input).copied().ok_or_else(|| {
                            EditDiagnostic::new(
                                origins.object(condition.id, fallback_operation_index),
                                vec![
                                    EditId::Object(condition.id),
                                    EditId::Object(input.object_id()),
                                ],
                                EditReason::UnknownObject,
                            )
                        })?;
                        let input_index = u32::try_from(input_index).map_err(|_| {
                            EditDiagnostic::new(
                                origins.object(condition.id, fallback_operation_index),
                                vec![EditId::Object(condition.id)],
                                EditReason::CapacityExceeded,
                            )
                        })?;
                        records.push(ExportedRecord {
                            kind: ExportedObjectKind::TransitionTriggerCondition,
                            properties: vec![ExportedProperty::StateMachineInputId(input_index)],
                        });
                    }
                    append_machine_fire_events(
                        records,
                        transition.id,
                        &owned,
                        event_local_ids,
                        fallback_operation_index,
                        origins,
                    )?;
                }
            }
        }
    }
    Ok(machine_indices)
}

fn append_machine_fire_events(
    records: &mut Vec<ExportedRecord>,
    owner: ObjectId,
    owned: &BTreeMap<ObjectId, Vec<(&RecordDefinition, &MachineRecordSpec)>>,
    event_local_ids: &BTreeMap<EventId, usize>,
    fallback_operation_index: usize,
    origins: &SpecOrigins,
) -> std::result::Result<(), EditDiagnostic> {
    for (record, spec) in owned
        .get(&owner)
        .into_iter()
        .flatten()
        .filter(|(_, spec)| matches!(spec, MachineRecordSpec::FireEvent { .. }))
        .copied()
    {
        let MachineRecordSpec::FireEvent {
            state,
            event,
            occurs,
        } = spec
        else {
            unreachable!("filtered fire-event records")
        };
        if *state != owner {
            return Err(EditDiagnostic::new(
                origins.object(record.id, fallback_operation_index),
                vec![EditId::Object(record.id)],
                EditReason::InternalInvariant,
            ));
        }
        let event_local_id = event_local_ids.get(event).copied().ok_or_else(|| {
            EditDiagnostic::new(
                origins.object(record.id, fallback_operation_index),
                vec![EditId::Object(record.id), EditId::Object(event.object_id())],
                EditReason::UnknownObject,
            )
        })?;
        let event_local_id = u32::try_from(event_local_id).map_err(|_| {
            EditDiagnostic::new(
                origins.object(record.id, fallback_operation_index),
                vec![EditId::Object(record.id)],
                EditReason::CapacityExceeded,
            )
        })?;
        records.push(ExportedRecord {
            kind: ExportedObjectKind::StateMachineFireEvent,
            properties: vec![
                ExportedProperty::EventId(event_local_id),
                ExportedProperty::FireEventOccurs(*occurs),
            ],
        });
    }
    Ok(())
}

fn append_animation_export_records(
    records: &mut Vec<ExportedRecord>,
    artboard: &ArtboardDefinition,
    local_ids: &BTreeMap<ObjectId, usize>,
    fallback_operation_index: usize,
    origins: &SpecOrigins,
) -> std::result::Result<BTreeMap<AnimationId, usize>, EditDiagnostic> {
    let mut animation_indices = BTreeMap::new();
    let mut animations = Vec::new();
    let mut owned = BTreeMap::<ObjectId, Vec<(&RecordDefinition, &AnimationRecordSpec)>>::new();
    for (record, spec) in artboard.animation_views() {
        if let AnimationRecordSpec::LinearAnimation(linear) = spec {
            animations.push((record, linear));
        }
        if let Some(owner) = spec.owner() {
            owned.entry(owner).or_default().push((record, spec));
        }
    }
    for (animation_index, (animation, spec)) in animations.into_iter().enumerate() {
        let animation_id = AnimationId(animation.id);
        if animation_indices
            .insert(animation_id, animation_index)
            .is_some()
        {
            return Err(EditDiagnostic::new(
                origins.object(animation.id, fallback_operation_index),
                vec![EditId::Object(animation.id)],
                EditReason::IdentityCollision,
            ));
        }
        let mut properties = vec![
            ExportedProperty::AnimationName(spec.name.clone()),
            ExportedProperty::AnimationDuration(spec.duration),
        ];
        if spec.fps != 60 {
            properties.push(ExportedProperty::AnimationFps(spec.fps));
        }
        records.push(ExportedRecord {
            kind: ExportedObjectKind::LinearAnimation,
            properties,
        });

        let mut keyed_objects = owned
            .get(&animation.id)
            .into_iter()
            .flatten()
            .filter_map(|(record, spec)| match spec {
                AnimationRecordSpec::KeyedObject {
                    animation: owner,
                    target,
                } if *owner == animation.id => Some((record, *target)),
                _ => None,
            })
            .collect::<Vec<_>>();
        keyed_objects.sort_by_key(|(_, target)| local_ids.get(target).copied());
        for (keyed_object, target) in keyed_objects {
            let target_local_id = local_ids.get(&target).copied().ok_or_else(|| {
                EditDiagnostic::new(
                    origins.object(keyed_object.id, fallback_operation_index),
                    vec![EditId::Object(keyed_object.id), EditId::Object(target)],
                    EditReason::UnknownObject,
                )
            })?;
            let target_local_id = u32::try_from(target_local_id).map_err(|_| {
                EditDiagnostic::new(
                    origins.object(keyed_object.id, fallback_operation_index),
                    vec![EditId::Object(keyed_object.id)],
                    EditReason::CapacityExceeded,
                )
            })?;
            records.push(ExportedRecord {
                kind: ExportedObjectKind::KeyedObject,
                properties: vec![ExportedProperty::KeyedObjectId(target_local_id)],
            });

            let mut keyed_properties = owned
                .get(&keyed_object.id)
                .into_iter()
                .flatten()
                .filter_map(|(record, spec)| match spec {
                    AnimationRecordSpec::KeyedProperty {
                        keyed_object: owner,
                        property,
                    } if *owner == keyed_object.id => Some((record, *property)),
                    _ => None,
                })
                .collect::<Vec<_>>();
            keyed_properties.sort_by_key(|(_, property)| property.schema_key());
            for (keyed_property, property) in keyed_properties {
                records.push(ExportedRecord {
                    kind: ExportedObjectKind::KeyedProperty,
                    properties: vec![ExportedProperty::KeyedProperty(property)],
                });
                let mut key_frames = owned
                    .get(&keyed_property.id)
                    .into_iter()
                    .flatten()
                    .filter_map(|(_, spec)| match spec {
                        AnimationRecordSpec::KeyFrameDouble {
                            keyed_property: owner,
                            frame,
                            value,
                        } if *owner == keyed_property.id => Some((*frame, *value)),
                        _ => None,
                    })
                    .collect::<Vec<_>>();
                key_frames.sort_by_key(|(frame, _)| *frame);
                for (frame, value) in key_frames {
                    records.push(ExportedRecord {
                        kind: ExportedObjectKind::KeyFrameDouble,
                        properties: vec![
                            ExportedProperty::KeyFrame(frame),
                            ExportedProperty::KeyFrameInterpolationLinear,
                            ExportedProperty::KeyFrameDoubleValue(value),
                        ],
                    });
                }
            }
        }
    }
    Ok(animation_indices)
}

fn image_crop_requires_mesh(crop: ImageCropRect) -> bool {
    crop.x != 0.0 || crop.y != 0.0 || crop.width != 1.0 || crop.height != 1.0
}

// Rive's Mesh.triangleIndexBytes payload is a sequence of unsigned LEB128
// values, not a packed u16 array. Every index in this fixed quad fits in one
// varuint byte.
const IMAGE_CROP_QUAD_TRIANGLE_INDEX_BYTES: &[u8] = &[0, 1, 2, 0, 2, 3];

fn append_image_crop_mesh_records(
    records: &mut Vec<ExportedRecord>,
    objects_by_local: &mut Vec<Option<ObjectId>>,
    image_local_id: usize,
    crop: ImageCropRect,
    fallback_operation_index: usize,
    artboard_id: ArtboardId,
) -> std::result::Result<(), EditDiagnostic> {
    let mesh_local_id = records.len();
    let mesh_parent_id = u32::try_from(image_local_id).map_err(|_| {
        EditDiagnostic::new(
            fallback_operation_index,
            vec![EditId::Artboard(artboard_id)],
            EditReason::CapacityExceeded,
        )
    })?;
    let vertex_parent_id = u32::try_from(mesh_local_id).map_err(|_| {
        EditDiagnostic::new(
            fallback_operation_index,
            vec![EditId::Artboard(artboard_id)],
            EditReason::CapacityExceeded,
        )
    })?;
    records.push(ExportedRecord {
        kind: ExportedObjectKind::Mesh,
        properties: vec![
            ExportedProperty::ParentId(mesh_parent_id),
            ExportedProperty::MeshTriangleIndexBytes(IMAGE_CROP_QUAD_TRIANGLE_INDEX_BYTES.to_vec()),
        ],
    });
    objects_by_local.push(None);

    let right = crop.x + crop.width;
    let bottom = crop.y + crop.height;
    for (x, y, u, v) in [
        (0.0, 0.0, crop.x, crop.y),
        (1.0, 0.0, right, crop.y),
        (1.0, 1.0, right, bottom),
        (0.0, 1.0, crop.x, bottom),
    ] {
        records.push(ExportedRecord {
            kind: ExportedObjectKind::MeshVertex,
            properties: vec![
                ExportedProperty::ParentId(vertex_parent_id),
                ExportedProperty::VertexX(x),
                ExportedProperty::VertexY(y),
                ExportedProperty::MeshVertexU(u),
                ExportedProperty::MeshVertexV(v),
            ],
        });
        objects_by_local.push(None);
    }
    Ok(())
}

fn parent_edit_ids(parent: Parent) -> Vec<EditId> {
    match parent {
        Parent::Artboard(id) => vec![EditId::Artboard(id)],
        Parent::Object(id) => vec![EditId::Object(id)],
    }
}

fn allocate_global_identity(next: &AtomicU64) -> Option<u64> {
    next.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
        current.checked_add(1)
    })
    .ok()
}

fn allocate_identity(next: &mut u64) -> Option<u64> {
    let allocated = *next;
    *next = allocated.checked_add(1)?;
    Some(allocated)
}

fn validate_artboard_spec(spec: &ArtboardSpec) -> std::result::Result<(), EditReason> {
    if !spec.width.is_finite() {
        return Err(EditReason::NonFiniteProperty { property: "width" });
    }
    if !spec.height.is_finite() {
        return Err(EditReason::NonFiniteProperty { property: "height" });
    }
    Ok(())
}

fn validate_linear_animation_spec(
    spec: &LinearAnimationSpec,
) -> std::result::Result<(), EditReason> {
    if spec.fps == 0 {
        return Err(EditReason::NonPositiveProperty { property: "fps" });
    }
    Ok(())
}

fn validate_node_spec(spec: &NodeSpec) -> std::result::Result<(), EditReason> {
    match spec {
        NodeSpec::Shape(spec) => {
            if !spec.x.is_finite() {
                return Err(EditReason::NonFiniteProperty { property: "x" });
            }
            if !spec.y.is_finite() {
                return Err(EditReason::NonFiniteProperty { property: "y" });
            }
            if !spec.opacity.is_finite() {
                return Err(EditReason::NonFiniteProperty {
                    property: "opacity",
                });
            }
            if !spec.rotation.is_finite() {
                return Err(EditReason::NonFiniteProperty {
                    property: "rotation",
                });
            }
            if !spec.scale_x.is_finite() {
                return Err(EditReason::NonFiniteProperty {
                    property: "scale_x",
                });
            }
            if !spec.scale_y.is_finite() {
                return Err(EditReason::NonFiniteProperty {
                    property: "scale_y",
                });
            }
        }
        NodeSpec::NestedArtboard(spec) => {
            for (property, value) in [
                ("x", spec.x),
                ("y", spec.y),
                ("opacity", spec.opacity),
                ("rotation", spec.rotation),
                ("scale_x", spec.scale_x),
                ("scale_y", spec.scale_y),
            ] {
                if !value.is_finite() {
                    return Err(EditReason::NonFiniteProperty { property });
                }
            }
        }
        NodeSpec::Image(spec) => {
            for (property, value) in [
                ("x", spec.x),
                ("y", spec.y),
                ("opacity", spec.opacity),
                ("rotation", spec.rotation),
                ("scale_x", spec.scale_x),
                ("scale_y", spec.scale_y),
                ("origin_x", spec.origin_x),
                ("origin_y", spec.origin_y),
            ] {
                if !value.is_finite() {
                    return Err(EditReason::NonFiniteProperty { property });
                }
            }
            if let Some(crop) = spec.crop {
                for (property, value) in [
                    ("crop.x", crop.x),
                    ("crop.y", crop.y),
                    ("crop.width", crop.width),
                    ("crop.height", crop.height),
                ] {
                    if !value.is_finite() {
                        return Err(EditReason::NonFiniteProperty { property });
                    }
                }
                if crop.width <= 0.0 {
                    return Err(EditReason::NonFiniteProperty {
                        property: "crop.width",
                    });
                }
                if crop.height <= 0.0 {
                    return Err(EditReason::NonFiniteProperty {
                        property: "crop.height",
                    });
                }
            }
        }
        NodeSpec::ScriptedDrawable(spec) => {
            for (property, value) in [
                ("x", spec.x),
                ("y", spec.y),
                ("opacity", spec.opacity),
                ("rotation", spec.rotation),
                ("scale_x", spec.scale_x),
                ("scale_y", spec.scale_y),
            ] {
                if !value.is_finite() {
                    return Err(EditReason::NonFiniteProperty { property });
                }
            }
        }
        NodeSpec::Rectangle(spec) => {
            if !spec.width.is_finite() {
                return Err(EditReason::NonFiniteProperty { property: "width" });
            }
            if !spec.height.is_finite() {
                return Err(EditReason::NonFiniteProperty { property: "height" });
            }
            if let Some(radii) = spec.corner_radii {
                for (property, value) in [
                    ("corner_radius_tl", radii.top_left),
                    ("corner_radius_tr", radii.top_right),
                    ("corner_radius_br", radii.bottom_right),
                    ("corner_radius_bl", radii.bottom_left),
                ] {
                    if !value.is_finite() {
                        return Err(EditReason::NonFiniteProperty { property });
                    }
                }
            }
        }
        NodeSpec::Stroke(spec) => {
            if !spec.thickness.is_finite() {
                return Err(EditReason::NonFiniteProperty {
                    property: "thickness",
                });
            }
        }
        NodeSpec::DashPath(spec) => {
            if !spec.offset.is_finite() {
                return Err(EditReason::NonFiniteProperty { property: "offset" });
            }
        }
        NodeSpec::Dash(spec) => {
            if !spec.length.is_finite() {
                return Err(EditReason::NonFiniteProperty { property: "length" });
            }
        }
        NodeSpec::Text(spec) => {
            for (property, value) in [
                ("x", spec.x),
                ("y", spec.y),
                ("opacity", spec.opacity),
                ("rotation", spec.rotation),
                ("scale_x", spec.scale_x),
                ("scale_y", spec.scale_y),
                ("width", spec.width),
                ("height", spec.height),
            ] {
                if !value.is_finite() {
                    return Err(EditReason::NonFiniteProperty { property });
                }
            }
        }
        NodeSpec::TextStylePaint(spec) => {
            for (property, value) in [
                ("font_size", spec.font_size),
                ("line_height", spec.line_height),
                ("letter_spacing", spec.letter_spacing),
            ] {
                if !value.is_finite() {
                    return Err(EditReason::NonFiniteProperty { property });
                }
            }
        }
        NodeSpec::Fill(_) | NodeSpec::SolidColor(_) | NodeSpec::TextValueRun(_) => {}
    }
    Ok(())
}

fn backboard_record() -> ExportedRecord {
    ExportedRecord {
        kind: ExportedObjectKind::Backboard,
        properties: Vec::new(),
    }
}

fn artboard_record(spec: &ArtboardSpec) -> ExportedRecord {
    ExportedRecord {
        kind: ExportedObjectKind::Artboard,
        properties: vec![
            ExportedProperty::ComponentName(spec.name.clone()),
            ExportedProperty::LayoutWidth(spec.width),
            ExportedProperty::LayoutHeight(spec.height),
        ],
    }
}

fn node_record(
    node: VisualRecordRef<'_>,
    parent_id: usize,
    local_ids: &BTreeMap<ObjectId, usize>,
    font_asset_indices: &BTreeMap<FontAssetId, u32>,
    image_asset_indices: &BTreeMap<ImageAssetId, u32>,
    script_asset_indices: &BTreeMap<ScriptAssetId, u32>,
    artboard_indices: &BTreeMap<ArtboardId, u32>,
) -> std::result::Result<ExportedRecord, EditReason> {
    let parent_id = u32::try_from(parent_id).map_err(|_| EditReason::CapacityExceeded)?;
    let mut properties = Vec::new();
    if parent_id != 0 {
        properties.push(ExportedProperty::ParentId(parent_id));
    }
    let kind = match node.spec {
        NodeSpec::Shape(spec) => {
            properties.push(ExportedProperty::ComponentName(spec.name.clone()));
            if spec.x != 0.0 {
                properties.push(ExportedProperty::TranslateX(spec.x));
            }
            if spec.y != 0.0 {
                properties.push(ExportedProperty::TranslateY(spec.y));
            }
            if spec.opacity != 1.0 {
                properties.push(ExportedProperty::WorldOpacity(spec.opacity));
            }
            if spec.rotation != 0.0 {
                properties.push(ExportedProperty::Rotation(spec.rotation));
            }
            if spec.scale_x != 1.0 {
                properties.push(ExportedProperty::ScaleX(spec.scale_x));
            }
            if spec.scale_y != 1.0 {
                properties.push(ExportedProperty::ScaleY(spec.scale_y));
            }
            ExportedObjectKind::Shape
        }
        NodeSpec::NestedArtboard(spec) => {
            let artboard_id = artboard_indices
                .get(&spec.artboard)
                .copied()
                .ok_or(EditReason::UnknownArtboard)?;
            properties.push(ExportedProperty::ComponentName(spec.name.clone()));
            properties.push(ExportedProperty::NestedArtboardId(artboard_id));
            if spec.x != 0.0 {
                properties.push(ExportedProperty::TranslateX(spec.x));
            }
            if spec.y != 0.0 {
                properties.push(ExportedProperty::TranslateY(spec.y));
            }
            if spec.opacity != 1.0 {
                properties.push(ExportedProperty::WorldOpacity(spec.opacity));
            }
            if spec.rotation != 0.0 {
                properties.push(ExportedProperty::Rotation(spec.rotation));
            }
            if spec.scale_x != 1.0 {
                properties.push(ExportedProperty::ScaleX(spec.scale_x));
            }
            if spec.scale_y != 1.0 {
                properties.push(ExportedProperty::ScaleY(spec.scale_y));
            }
            ExportedObjectKind::NestedArtboard
        }
        NodeSpec::Image(spec) => {
            let image_asset_id = image_asset_indices
                .get(&spec.image)
                .copied()
                .ok_or(EditReason::UnknownImageAsset)?;
            properties.push(ExportedProperty::ComponentName(spec.name.clone()));
            properties.push(ExportedProperty::ImageAssetId(image_asset_id));
            if spec.x != 0.0 {
                properties.push(ExportedProperty::TranslateX(spec.x));
            }
            if spec.y != 0.0 {
                properties.push(ExportedProperty::TranslateY(spec.y));
            }
            if spec.opacity != 1.0 {
                properties.push(ExportedProperty::WorldOpacity(spec.opacity));
            }
            if spec.rotation != 0.0 {
                properties.push(ExportedProperty::Rotation(spec.rotation));
            }
            if spec.scale_x != 1.0 {
                properties.push(ExportedProperty::ScaleX(spec.scale_x));
            }
            if spec.scale_y != 1.0 {
                properties.push(ExportedProperty::ScaleY(spec.scale_y));
            }
            if spec.origin_x != 0.0 {
                properties.push(ExportedProperty::ImageOriginX(spec.origin_x));
            }
            if spec.origin_y != 0.0 {
                properties.push(ExportedProperty::ImageOriginY(spec.origin_y));
            }
            if spec.fit != 0 {
                properties.push(ExportedProperty::ImageFit(spec.fit));
            }
            if spec.alignment_x != 0.0 {
                properties.push(ExportedProperty::ImageAlignmentX(spec.alignment_x));
            }
            if spec.alignment_y != 0.0 {
                properties.push(ExportedProperty::ImageAlignmentY(spec.alignment_y));
            }
            ExportedObjectKind::Image
        }
        NodeSpec::ScriptedDrawable(spec) => {
            let script_asset_id = script_asset_indices
                .get(&spec.script)
                .copied()
                .ok_or(EditReason::UnknownScriptAsset)?;
            properties.push(ExportedProperty::ComponentName(spec.name.clone()));
            properties.push(ExportedProperty::ScriptedDrawableScriptAssetId(
                script_asset_id,
            ));
            if spec.x != 0.0 {
                properties.push(ExportedProperty::TranslateX(spec.x));
            }
            if spec.y != 0.0 {
                properties.push(ExportedProperty::TranslateY(spec.y));
            }
            if spec.opacity != 1.0 {
                properties.push(ExportedProperty::WorldOpacity(spec.opacity));
            }
            if spec.rotation != 0.0 {
                properties.push(ExportedProperty::Rotation(spec.rotation));
            }
            if spec.scale_x != 1.0 {
                properties.push(ExportedProperty::ScaleX(spec.scale_x));
            }
            if spec.scale_y != 1.0 {
                properties.push(ExportedProperty::ScaleY(spec.scale_y));
            }
            ExportedObjectKind::ScriptedDrawable
        }
        NodeSpec::Rectangle(spec) => {
            properties.push(ExportedProperty::ComponentName(spec.name.clone()));
            properties.push(ExportedProperty::PathWidth(spec.width));
            properties.push(ExportedProperty::PathHeight(spec.height));
            if let Some(radii) = spec.corner_radii {
                properties.push(ExportedProperty::RectangleLinkCornerRadius(radii.linked));
                properties.push(ExportedProperty::RectangleCornerRadiusTopLeft(
                    radii.top_left,
                ));
                properties.push(ExportedProperty::RectangleCornerRadiusTopRight(
                    radii.top_right,
                ));
                properties.push(ExportedProperty::RectangleCornerRadiusBottomRight(
                    radii.bottom_right,
                ));
                properties.push(ExportedProperty::RectangleCornerRadiusBottomLeft(
                    radii.bottom_left,
                ));
            }
            ExportedObjectKind::Rectangle
        }
        NodeSpec::Fill(spec) => {
            properties.push(ExportedProperty::ComponentName(spec.name.clone()));
            properties.push(ExportedProperty::FillRule(ExportedFillRule::NonZero));
            ExportedObjectKind::Fill
        }
        NodeSpec::SolidColor(spec) => {
            properties.push(ExportedProperty::ComponentName(spec.name.clone()));
            properties.push(ExportedProperty::ColorValue(spec.color));
            ExportedObjectKind::SolidColor
        }
        NodeSpec::Stroke(spec) => {
            properties.push(ExportedProperty::ComponentName(spec.name.clone()));
            properties.push(ExportedProperty::StrokeThickness(spec.thickness));
            properties.push(ExportedProperty::StrokeCap(spec.cap));
            properties.push(ExportedProperty::StrokeJoin(spec.join));
            properties.push(ExportedProperty::StrokeTransformAffectsStroke(
                spec.transform_affects_stroke,
            ));
            ExportedObjectKind::Stroke
        }
        NodeSpec::DashPath(spec) => {
            properties.push(ExportedProperty::ComponentName(spec.name.clone()));
            properties.push(ExportedProperty::DashOffset(spec.offset));
            properties.push(ExportedProperty::DashOffsetIsPercentage(
                spec.offset_is_percentage,
            ));
            ExportedObjectKind::DashPath
        }
        NodeSpec::Dash(spec) => {
            properties.push(ExportedProperty::ComponentName(spec.name.clone()));
            properties.push(ExportedProperty::DashLength(spec.length));
            properties.push(ExportedProperty::DashLengthIsPercentage(
                spec.length_is_percentage,
            ));
            ExportedObjectKind::Dash
        }
        NodeSpec::Text(spec) => {
            properties.push(ExportedProperty::ComponentName(spec.name.clone()));
            properties.push(ExportedProperty::TranslateX(spec.x));
            properties.push(ExportedProperty::TranslateY(spec.y));
            if spec.opacity != 1.0 {
                properties.push(ExportedProperty::WorldOpacity(spec.opacity));
            }
            if spec.rotation != 0.0 {
                properties.push(ExportedProperty::Rotation(spec.rotation));
            }
            if spec.scale_x != 1.0 {
                properties.push(ExportedProperty::ScaleX(spec.scale_x));
            }
            if spec.scale_y != 1.0 {
                properties.push(ExportedProperty::ScaleY(spec.scale_y));
            }
            properties.push(ExportedProperty::TextSizing(spec.sizing));
            properties.push(ExportedProperty::TextAlign(spec.align));
            properties.push(ExportedProperty::TextWidth(spec.width));
            properties.push(ExportedProperty::TextHeight(spec.height));
            properties.push(ExportedProperty::TextWrap(spec.wrap));
            properties.push(ExportedProperty::TextOverflow(spec.overflow));
            ExportedObjectKind::Text
        }
        NodeSpec::TextValueRun(spec) => {
            let style_id = local_ids
                .get(&spec.style)
                .copied()
                .ok_or(EditReason::UnknownObject)?;
            let style_id = u32::try_from(style_id).map_err(|_| EditReason::CapacityExceeded)?;
            properties.push(ExportedProperty::ComponentName(spec.name.clone()));
            properties.push(ExportedProperty::TextValueRunText(spec.text.clone()));
            properties.push(ExportedProperty::TextValueRunStyleId(style_id));
            ExportedObjectKind::TextValueRun
        }
        NodeSpec::TextStylePaint(spec) => {
            let font_asset_id = font_asset_indices
                .get(&spec.font)
                .copied()
                .ok_or(EditReason::UnknownFontAsset)?;
            properties.push(ExportedProperty::ComponentName(spec.name.clone()));
            properties.push(ExportedProperty::TextStyleFontSize(spec.font_size));
            properties.push(ExportedProperty::TextStyleLineHeight(spec.line_height));
            properties.push(ExportedProperty::TextStyleLetterSpacing(
                spec.letter_spacing,
            ));
            properties.push(ExportedProperty::TextStyleFontAssetId(font_asset_id));
            ExportedObjectKind::TextStylePaint
        }
    };
    Ok(ExportedRecord { kind, properties })
}

fn canonicalize_exported_records(records: &mut [ExportedRecord]) {
    for record in records {
        record.properties.sort_by_key(ExportedProperty::schema_key);
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    #[cfg(feature = "scripting")]
    use anyhow::Context;
    use anyhow::Result;
    use nuxie_render_stream::RenderStream;

    use super::*;
    use crate::RecordingFactory;

    fn parse_single_frame(stream: &str) -> Result<RenderStream> {
        Ok(RenderStream::parse(&format!("{stream}frame\n"))?)
    }

    #[allow(clippy::arithmetic_side_effects)]
    fn fixture_font_bytes() -> Vec<u8> {
        let mut accumulator = 0u32;
        let mut bit_count = 0u8;
        let mut decoded = Vec::new();
        for byte in include_bytes!("../tests/fixtures/roboto-a.ttf.base64")
            .iter()
            .copied()
            .filter(|byte| !byte.is_ascii_whitespace())
        {
            if byte == b'=' {
                break;
            }
            let value = match byte {
                b'A'..=b'Z' => byte - b'A',
                b'a'..=b'z' => byte - b'a' + 26,
                b'0'..=b'9' => byte - b'0' + 52,
                b'+' => 62,
                b'/' => 63,
                _ => panic!("invalid base64 font fixture"),
            };
            accumulator = (accumulator << 6) | u32::from(value);
            bit_count += 6;
            if bit_count >= 8 {
                bit_count -= 8;
                decoded.push((accumulator >> bit_count) as u8);
                accumulator &= (1u32 << bit_count) - 1;
            }
        }
        decoded
    }

    fn fixture_authoring_record(
        type_name: &str,
        properties: Vec<(&str, AuthoringValue)>,
    ) -> AuthoringRecord {
        let definition = nuxie_schema::definition_by_name(type_name)
            .expect("fixture record type exists in the generated schema");
        let properties = properties
            .into_iter()
            .map(|(property_name, value)| {
                let property = std::iter::once(definition.name)
                    .chain(definition.ancestors.iter().copied())
                    .filter_map(nuxie_schema::definition_by_name)
                    .flat_map(|owner| owner.properties)
                    .find(|property| property.name == property_name)
                    .expect("fixture property exists in the generated schema hierarchy");
                AuthoringProperty {
                    key: property.key.int,
                    value,
                }
            })
            .collect();
        AuthoringRecord {
            type_key: definition.type_key.int,
            properties,
        }
    }

    #[derive(Clone, Copy)]
    struct ImportedModifierFixture {
        x: f32,
        y: f32,
        modify_from: f32,
        modify_to: f32,
    }

    fn imported_modifier_scene_fixture(
        text_value: &str,
        text_width: f32,
        wrap_value: u64,
        overflow_value: u64,
        modifier: Option<ImportedModifierFixture>,
    ) -> Result<(Scene, InstanceId, ObjectId)> {
        imported_modifier_scene_fixture_with_x(
            text_value,
            text_width,
            wrap_value,
            overflow_value,
            10.0,
            modifier,
        )
    }

    fn imported_modifier_scene_fixture_with_x(
        text_value: &str,
        text_width: f32,
        wrap_value: u64,
        overflow_value: u64,
        text_x: f32,
        modifier: Option<ImportedModifierFixture>,
    ) -> Result<(Scene, InstanceId, ObjectId)> {
        imported_modifier_scene_fixture_with_style_values(
            text_value,
            text_width,
            40.0,
            wrap_value,
            overflow_value,
            text_x,
            20.0,
            20.0,
            modifier,
        )
    }

    #[allow(clippy::too_many_arguments)]
    fn imported_modifier_scene_fixture_with_style_values(
        text_value: &str,
        text_width: f32,
        text_height: f32,
        wrap_value: u64,
        overflow_value: u64,
        text_x: f32,
        font_size: f32,
        line_height: f32,
        modifier: Option<ImportedModifierFixture>,
    ) -> Result<(Scene, InstanceId, ObjectId)> {
        let mut records = vec![
            fixture_authoring_record("Backboard", Vec::new()),
            fixture_authoring_record("FontAsset", vec![("assetId", AuthoringValue::Uint(0))]),
            fixture_authoring_record(
                "FileAssetContents",
                vec![("bytes", AuthoringValue::Bytes(fixture_font_bytes()))],
            ),
            fixture_authoring_record(
                "Artboard",
                vec![
                    ("width", AuthoringValue::Double(200.0)),
                    ("height", AuthoringValue::Double(100.0)),
                ],
            ),
            fixture_authoring_record(
                "Text",
                vec![
                    ("x", AuthoringValue::Double(text_x)),
                    ("y", AuthoringValue::Double(10.0)),
                    ("sizingValue", AuthoringValue::Uint(2)),
                    ("width", AuthoringValue::Double(text_width)),
                    ("height", AuthoringValue::Double(text_height)),
                    ("wrapValue", AuthoringValue::Uint(wrap_value)),
                    ("overflowValue", AuthoringValue::Uint(overflow_value)),
                ],
            ),
            fixture_authoring_record(
                "TextStylePaint",
                vec![
                    ("parentId", AuthoringValue::Uint(1)),
                    ("fontSize", AuthoringValue::Double(font_size)),
                    ("lineHeight", AuthoringValue::Double(line_height)),
                    ("fontAssetId", AuthoringValue::Uint(0)),
                ],
            ),
            fixture_authoring_record("Fill", vec![("parentId", AuthoringValue::Uint(2))]),
            fixture_authoring_record(
                "SolidColor",
                vec![
                    ("parentId", AuthoringValue::Uint(3)),
                    ("colorValue", AuthoringValue::Color(0xff11_2233)),
                ],
            ),
            fixture_authoring_record(
                "TextValueRun",
                vec![
                    ("parentId", AuthoringValue::Uint(1)),
                    ("text", AuthoringValue::String(text_value.into())),
                    ("styleId", AuthoringValue::Uint(2)),
                ],
            ),
        ];
        if let Some(modifier) = modifier {
            // This is an actual imported Rive modifier graph, not a mocked
            // post-layout offset: the group and range travel through binary
            // import, graph dependency construction, shaping, and draw.
            records.extend([
                fixture_authoring_record(
                    "TextModifierGroup",
                    vec![
                        ("parentId", AuthoringValue::Uint(1)),
                        ("modifierFlags", AuthoringValue::Uint(1 << 2)),
                        ("x", AuthoringValue::Double(modifier.x)),
                        ("y", AuthoringValue::Double(modifier.y)),
                    ],
                ),
                fixture_authoring_record(
                    "TextModifierRange",
                    vec![
                        ("parentId", AuthoringValue::Uint(6)),
                        ("typeValue", AuthoringValue::Uint(1)),
                        ("modifyFrom", AuthoringValue::Double(modifier.modify_from)),
                        ("modifyTo", AuthoringValue::Double(modifier.modify_to)),
                        ("falloffFrom", AuthoringValue::Double(modifier.modify_from)),
                        ("falloffTo", AuthoringValue::Double(modifier.modify_to)),
                    ],
                ),
            ]);
        }

        let runtime = RuntimeFile::from_authoring_records(records)?;
        let file = Arc::new(File::from_runtime(runtime)?);
        let local_count = file
            .graph
            .artboards
            .first()
            .map(|graph| graph.local_objects.len())
            .ok_or_else(|| anyhow::anyhow!("modifier fixture has no artboard graph"))?;
        let artboard = ArtboardId(
            allocate_global_identity(&NEXT_ARTBOARD_ID)
                .ok_or_else(|| anyhow::anyhow!("artboard identity exhausted"))?,
        );
        let text = ObjectId(
            allocate_global_identity(&NEXT_OBJECT_ID)
                .ok_or_else(|| anyhow::anyhow!("object identity exhausted"))?,
        );
        let mut objects = BTreeMap::new();
        objects.insert(
            text,
            RuntimeSlot {
                local_id: 1,
                kind: NodeKind::Text,
            },
        );
        let mut objects_by_local = vec![None; local_count];
        *objects_by_local
            .get_mut(1)
            .ok_or_else(|| anyhow::anyhow!("modifier fixture Text local is missing"))? = Some(text);

        let mut scene = Scene::new();
        scene.materialized.insert(
            artboard,
            MaterializedArtboard {
                file,
                objects,
                animations: BTreeMap::new(),
                machines: BTreeMap::new(),
                events_by_local: vec![None; local_count],
                objects_by_artboard_local: BTreeMap::from([(artboard, objects_by_local.clone())]),
                nested_artboard_targets: BTreeMap::new(),
            },
        );
        let instance = scene.instantiate(artboard)?;
        Ok((scene, instance, text))
    }

    fn imported_follow_path_scene_fixture() -> Result<(Scene, InstanceId, ObjectId)> {
        let records = vec![
            fixture_authoring_record("Backboard", Vec::new()),
            fixture_authoring_record("FontAsset", vec![("assetId", AuthoringValue::Uint(0))]),
            fixture_authoring_record(
                "FileAssetContents",
                vec![("bytes", AuthoringValue::Bytes(fixture_font_bytes()))],
            ),
            fixture_authoring_record(
                "Artboard",
                vec![
                    ("width", AuthoringValue::Double(200.0)),
                    ("height", AuthoringValue::Double(150.0)),
                ],
            ),
            fixture_authoring_record("Shape", Vec::new()),
            fixture_authoring_record("PointsPath", vec![("parentId", AuthoringValue::Uint(1))]),
            fixture_authoring_record(
                "StraightVertex",
                vec![
                    ("parentId", AuthoringValue::Uint(2)),
                    ("x", AuthoringValue::Double(0.0)),
                    ("y", AuthoringValue::Double(50.0)),
                ],
            ),
            fixture_authoring_record(
                "StraightVertex",
                vec![
                    ("parentId", AuthoringValue::Uint(2)),
                    ("x", AuthoringValue::Double(10.0)),
                    ("y", AuthoringValue::Double(50.0)),
                ],
            ),
            fixture_authoring_record(
                "StraightVertex",
                vec![
                    ("parentId", AuthoringValue::Uint(2)),
                    ("x", AuthoringValue::Double(10.0)),
                    ("y", AuthoringValue::Double(100.0)),
                ],
            ),
            fixture_authoring_record(
                "Text",
                vec![
                    ("sizingValue", AuthoringValue::Uint(2)),
                    ("width", AuthoringValue::Double(80.0)),
                    ("height", AuthoringValue::Double(40.0)),
                    ("wrapValue", AuthoringValue::Uint(1)),
                ],
            ),
            fixture_authoring_record(
                "TextStylePaint",
                vec![
                    ("parentId", AuthoringValue::Uint(6)),
                    ("fontSize", AuthoringValue::Double(20.0)),
                    ("lineHeight", AuthoringValue::Double(20.0)),
                    ("fontAssetId", AuthoringValue::Uint(0)),
                ],
            ),
            fixture_authoring_record("Fill", vec![("parentId", AuthoringValue::Uint(7))]),
            fixture_authoring_record(
                "SolidColor",
                vec![
                    ("parentId", AuthoringValue::Uint(8)),
                    ("colorValue", AuthoringValue::Color(0xff11_2233)),
                ],
            ),
            fixture_authoring_record(
                "TextValueRun",
                vec![
                    ("parentId", AuthoringValue::Uint(6)),
                    ("text", AuthoringValue::String("aa".into())),
                    ("styleId", AuthoringValue::Uint(7)),
                ],
            ),
            fixture_authoring_record(
                "TextModifierGroup",
                vec![("parentId", AuthoringValue::Uint(6))],
            ),
            fixture_authoring_record(
                "TextModifierRange",
                vec![("parentId", AuthoringValue::Uint(11))],
            ),
            fixture_authoring_record(
                "TextFollowPathModifier",
                vec![
                    ("parentId", AuthoringValue::Uint(11)),
                    ("targetId", AuthoringValue::Uint(2)),
                ],
            ),
        ];
        let runtime = RuntimeFile::from_authoring_records(records)?;
        let file = Arc::new(File::from_runtime(runtime)?);
        let graph = file
            .graph
            .artboards
            .first()
            .ok_or_else(|| anyhow::anyhow!("follow-path fixture has no artboard graph"))?;
        let text_local = graph
            .local_objects
            .iter()
            .find(|object| object.type_name == Some("Text"))
            .map(|object| object.local_id)
            .ok_or_else(|| anyhow::anyhow!("follow-path fixture Text local is missing"))?;
        let local_count = graph.local_objects.len();
        let artboard = ArtboardId(
            allocate_global_identity(&NEXT_ARTBOARD_ID)
                .ok_or_else(|| anyhow::anyhow!("artboard identity exhausted"))?,
        );
        let text = ObjectId(
            allocate_global_identity(&NEXT_OBJECT_ID)
                .ok_or_else(|| anyhow::anyhow!("object identity exhausted"))?,
        );
        let mut objects = BTreeMap::new();
        objects.insert(
            text,
            RuntimeSlot {
                local_id: text_local,
                kind: NodeKind::Text,
            },
        );
        let mut objects_by_local = vec![None; local_count];
        *objects_by_local
            .get_mut(text_local)
            .ok_or_else(|| anyhow::anyhow!("follow-path fixture Text local is out of range"))? =
            Some(text);

        let mut scene = Scene::new();
        scene.materialized.insert(
            artboard,
            MaterializedArtboard {
                file,
                objects,
                animations: BTreeMap::new(),
                machines: BTreeMap::new(),
                events_by_local: vec![None; local_count],
                objects_by_artboard_local: BTreeMap::from([(artboard, objects_by_local.clone())]),
                nested_artboard_targets: BTreeMap::new(),
            },
        );
        let instance = scene.instantiate(artboard)?;
        Ok((scene, instance, text))
    }

    fn static_text_file_fixture(
        text_value: &str,
        vertical_align_value: u64,
        overflow_value: u64,
    ) -> Result<Arc<File>> {
        let runtime = RuntimeFile::from_authoring_records(vec![
            fixture_authoring_record("Backboard", Vec::new()),
            fixture_authoring_record("FontAsset", vec![("assetId", AuthoringValue::Uint(0))]),
            fixture_authoring_record(
                "FileAssetContents",
                vec![("bytes", AuthoringValue::Bytes(fixture_font_bytes()))],
            ),
            fixture_authoring_record(
                "Artboard",
                vec![
                    ("width", AuthoringValue::Double(200.0)),
                    ("height", AuthoringValue::Double(100.0)),
                ],
            ),
            fixture_authoring_record(
                "Text",
                vec![
                    ("x", AuthoringValue::Double(10.0)),
                    ("y", AuthoringValue::Double(10.0)),
                    ("sizingValue", AuthoringValue::Uint(2)),
                    ("width", AuthoringValue::Double(80.0)),
                    ("height", AuthoringValue::Double(60.0)),
                    ("wrapValue", AuthoringValue::Uint(1)),
                    (
                        "verticalAlignValue",
                        AuthoringValue::Uint(vertical_align_value),
                    ),
                    ("overflowValue", AuthoringValue::Uint(overflow_value)),
                ],
            ),
            fixture_authoring_record(
                "TextStylePaint",
                vec![
                    ("parentId", AuthoringValue::Uint(1)),
                    ("fontSize", AuthoringValue::Double(20.0)),
                    ("lineHeight", AuthoringValue::Double(20.0)),
                    ("fontAssetId", AuthoringValue::Uint(0)),
                ],
            ),
            fixture_authoring_record("Fill", vec![("parentId", AuthoringValue::Uint(2))]),
            fixture_authoring_record(
                "SolidColor",
                vec![
                    ("parentId", AuthoringValue::Uint(3)),
                    ("colorValue", AuthoringValue::Color(0xff11_2233)),
                ],
            ),
            fixture_authoring_record(
                "TextValueRun",
                vec![
                    ("parentId", AuthoringValue::Uint(1)),
                    ("text", AuthoringValue::String(text_value.into())),
                    ("styleId", AuthoringValue::Uint(2)),
                ],
            ),
        ])?;
        Ok(Arc::new(File::from_runtime(runtime)?))
    }

    fn owned_static_text_fixture(
        text_value: &str,
        vertical_align_value: u64,
        overflow_value: u64,
    ) -> Result<OwnedArtboardInstance> {
        OwnedArtboardInstance::instantiate(
            static_text_file_fixture(text_value, vertical_align_value, overflow_value)?,
            0,
        )
    }

    fn owned_multi_font_text_with_missing_second_font_fixture() -> Result<OwnedArtboardInstance> {
        let runtime = RuntimeFile::from_authoring_records(vec![
            fixture_authoring_record("Backboard", Vec::new()),
            fixture_authoring_record("FontAsset", vec![("assetId", AuthoringValue::Uint(0))]),
            fixture_authoring_record(
                "FileAssetContents",
                vec![("bytes", AuthoringValue::Bytes(fixture_font_bytes()))],
            ),
            fixture_authoring_record("FontAsset", vec![("assetId", AuthoringValue::Uint(1))]),
            fixture_authoring_record(
                "Artboard",
                vec![
                    ("width", AuthoringValue::Double(200.0)),
                    ("height", AuthoringValue::Double(100.0)),
                ],
            ),
            fixture_authoring_record(
                "Text",
                vec![
                    ("sizingValue", AuthoringValue::Uint(2)),
                    ("width", AuthoringValue::Double(80.0)),
                    ("height", AuthoringValue::Double(40.0)),
                    ("wrapValue", AuthoringValue::Uint(1)),
                ],
            ),
            fixture_authoring_record(
                "TextStylePaint",
                vec![
                    ("parentId", AuthoringValue::Uint(1)),
                    ("fontSize", AuthoringValue::Double(20.0)),
                    ("lineHeight", AuthoringValue::Double(20.0)),
                    ("fontAssetId", AuthoringValue::Uint(0)),
                ],
            ),
            fixture_authoring_record(
                "TextStylePaint",
                vec![
                    ("parentId", AuthoringValue::Uint(1)),
                    ("fontSize", AuthoringValue::Double(20.0)),
                    ("lineHeight", AuthoringValue::Double(20.0)),
                    ("fontAssetId", AuthoringValue::Uint(1)),
                ],
            ),
            fixture_authoring_record("Fill", vec![("parentId", AuthoringValue::Uint(2))]),
            fixture_authoring_record(
                "SolidColor",
                vec![
                    ("parentId", AuthoringValue::Uint(4)),
                    ("colorValue", AuthoringValue::Color(0xff11_2233)),
                ],
            ),
            fixture_authoring_record(
                "TextValueRun",
                vec![
                    ("parentId", AuthoringValue::Uint(1)),
                    ("text", AuthoringValue::String("a".into())),
                    ("styleId", AuthoringValue::Uint(2)),
                ],
            ),
            fixture_authoring_record(
                "TextValueRun",
                vec![
                    ("parentId", AuthoringValue::Uint(1)),
                    ("text", AuthoringValue::String("b".into())),
                    ("styleId", AuthoringValue::Uint(3)),
                ],
            ),
        ])?;
        OwnedArtboardInstance::instantiate(Arc::new(File::from_runtime(runtime)?), 0)
    }

    fn owned_font_text_fixture(include_embedded_contents: bool) -> Result<OwnedArtboardInstance> {
        let mut scene = Scene::new();
        scene.edit(|tx| {
            let font = tx.create_font_asset(FontAssetSpec {
                name: "Roboto A".into(),
                bytes: fixture_font_bytes(),
            })?;
            let artboard = tx.create_artboard(ArtboardSpec {
                name: "External font text".into(),
                width: 200.0,
                height: 100.0,
            })?;
            let text = tx.create(
                Parent::Artboard(artboard),
                NodeSpec::Text(TextSpec {
                    name: "Title".into(),
                    x: 10.0,
                    y: 20.0,
                    opacity: 1.0,
                    rotation: 0.0,
                    scale_x: 1.0,
                    scale_y: 1.0,
                    sizing: SceneTextSizing::Fixed,
                    width: 120.0,
                    height: 40.0,
                    align: SceneTextAlign::Left,
                    wrap: SceneTextWrap::Wrap,
                    overflow: SceneTextOverflow::Visible,
                }),
            )?;
            let style = tx.create(
                Parent::Object(text),
                NodeSpec::TextStylePaint(TextStylePaintSpec {
                    name: "Title Style".into(),
                    font_size: 24.0,
                    line_height: 30.0,
                    letter_spacing: 0.0,
                    font,
                }),
            )?;
            let fill = tx.create(
                Parent::Object(style),
                NodeSpec::Fill(FillSpec {
                    name: "Title Fill".into(),
                }),
            )?;
            tx.create(
                Parent::Object(fill),
                NodeSpec::SolidColor(SolidColorSpec {
                    name: "Title Color".into(),
                    color: 0xff11_2233,
                }),
            )?;
            tx.create(
                Parent::Object(text),
                NodeSpec::TextValueRun(TextValueRunSpec {
                    name: "Title Run".into(),
                    text: "external font".into(),
                    style,
                }),
            )?;
            Ok(())
        })?;

        let mut records = scene.export_records().into_authoring_records();
        if !include_embedded_contents {
            records.retain(|record| record.type_key != TYPE_FILE_ASSET_CONTENTS);
        }
        let runtime = RuntimeFile::from_authoring_records(records)?;
        let file = Arc::new(File::from_runtime(runtime)?);
        Ok(OwnedArtboardInstance::instantiate(file, 0)?)
    }

    fn create_colored_rect(
        tx: &mut SceneTx<'_>,
        artboard: ArtboardId,
        name: &str,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        color: u32,
    ) -> std::result::Result<ObjectId, EditAbort> {
        let shape = tx.create(
            Parent::Artboard(artboard),
            NodeSpec::Shape(ShapeSpec {
                name: format!("{name} Shape"),
                x,
                y,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
            }),
        )?;
        tx.create(
            Parent::Object(shape),
            NodeSpec::Rectangle(RectangleSpec::new(
                format!("{name} Rectangle"),
                width,
                height,
            )),
        )?;
        let fill = tx.create(
            Parent::Object(shape),
            NodeSpec::Fill(FillSpec {
                name: format!("{name} Fill"),
            }),
        )?;
        tx.create(
            Parent::Object(fill),
            NodeSpec::SolidColor(SolidColorSpec {
                name: format!("{name} Color"),
                color,
            }),
        )?;
        Ok(shape)
    }

    fn create_nested_artboard_host(
        tx: &mut SceneTx<'_>,
        parent: ArtboardId,
        child: ArtboardId,
        name: &str,
        x: f32,
        y: f32,
    ) -> std::result::Result<ObjectId, EditAbort> {
        tx.create(
            Parent::Artboard(parent),
            NodeSpec::NestedArtboard(NestedArtboardSpec {
                name: name.into(),
                x,
                y,
                opacity: 1.0,
                rotation: 0.0,
                scale_x: 1.0,
                scale_y: 1.0,
                artboard: child,
            }),
        )
    }

    #[test]
    fn shaped_auto_sized_bottom_trim_moves_public_geometry_with_the_trimmed_glyph_layout()
    -> Result<()> {
        let mut scene = Scene::new();
        let ((artboard, text), _) = scene.edit(|tx| {
            let font = tx.create_font_asset(FontAssetSpec {
                name: "Roboto A".into(),
                bytes: fixture_font_bytes(),
            })?;
            let artboard = tx.create_artboard(ArtboardSpec {
                name: "Trim geometry".into(),
                width: 200.0,
                height: 100.0,
            })?;
            let text = tx.create(
                Parent::Artboard(artboard),
                NodeSpec::Text(TextSpec {
                    name: "Trimmed Text".into(),
                    x: 10.0,
                    y: 20.0,
                    opacity: 1.0,
                    rotation: 0.0,
                    scale_x: 1.0,
                    scale_y: 1.0,
                    sizing: SceneTextSizing::Fixed,
                    width: 80.0,
                    height: 40.0,
                    align: SceneTextAlign::Left,
                    wrap: SceneTextWrap::NoWrap,
                    overflow: SceneTextOverflow::Visible,
                }),
            )?;
            let style = tx.create(
                Parent::Object(text),
                NodeSpec::TextStylePaint(TextStylePaintSpec {
                    name: "Trim Style".into(),
                    font_size: 24.0,
                    line_height: 30.0,
                    letter_spacing: 0.0,
                    font,
                }),
            )?;
            tx.create(
                Parent::Object(text),
                NodeSpec::TextValueRun(TextValueRunSpec {
                    name: "Trim Run".into(),
                    text: "a".into(),
                    style,
                }),
            )?;
            Ok((artboard, text))
        })?;
        let instance = scene.instantiate(artboard)?;
        let local_id = scene
            .materialized
            .get(&artboard)
            .and_then(|materialized| materialized.objects.get(&text))
            .map(|slot| slot.local_id)
            .expect("authored Text has a runtime local");
        let text_property_key = |name: &str| {
            nuxie_schema::definition_by_name("Text")
                .and_then(|definition| {
                    definition
                        .properties
                        .iter()
                        .find(|property| property.name == name)
                })
                .map(|property| property.key.int)
                .expect("Text property exists in the generated schema")
        };
        let sizing_key = text_property_key("sizingValue");
        let trim_key = text_property_key("verticalTrimValue");
        let origin_y_key = text_property_key("originY");
        let live = scene
            .instances
            .iter_mut()
            .filter_map(Option::as_mut)
            .find(|live| live.id == instance)
            .expect("instance is live");
        assert!(
            live.runtime
                .raw_mut()
                .set_uint_property(local_id, sizing_key, 0),
            "the runtime fixture switches Text to auto width"
        );
        assert!(
            live.runtime
                .raw_mut()
                .set_double_property(local_id, origin_y_key, 1.0),
            "the runtime fixture anchors Text at its bottom edge"
        );

        let (untrimmed_start, untrimmed_end, untrimmed_selection, untrimmed_bounds) = {
            let mut frame = scene.frame();
            let start = frame
                .text_caret(instance, text, 0)
                .expect("auto-sized text has a start caret");
            let end = frame
                .text_caret(instance, text, 1)
                .expect("auto-sized text has an end caret");
            let selection = frame
                .text_selection_rects(instance, text, 0..1)
                .into_iter()
                .next()
                .expect("auto-sized text has one selected line segment");
            let bounds = frame
                .world_bounds(instance, text)
                .expect("auto-sized text has logical bounds");
            (start, end, selection, bounds)
        };
        assert!(
            untrimmed_bounds.width() < 80.0,
            "the runtime fixture must be auto-sized, got {untrimmed_bounds:?}"
        );

        let live = scene
            .instances
            .iter_mut()
            .filter_map(Option::as_mut)
            .find(|live| live.id == instance)
            .expect("instance remains live");
        assert!(
            live.runtime
                .raw_mut()
                .set_uint_property(local_id, trim_key, 1 << 8),
            "second-byte value 1 enables alphabetic bottom trim"
        );

        let mut frame = scene.frame();
        let trimmed_start = frame
            .text_caret(instance, text, 0)
            .expect("trimmed text has a start caret");
        let trimmed_end = frame
            .text_caret(instance, text, 1)
            .expect("trimmed text has an end caret");
        let trimmed_selection = frame
            .text_selection_rects(instance, text, 0..1)
            .into_iter()
            .next()
            .expect("trimmed text has one selected line segment");
        let trimmed_bounds = frame
            .world_bounds(instance, text)
            .expect("trimmed text has logical bounds");

        let trim_shift = trimmed_start.top.y - untrimmed_start.top.y;
        assert!(
            trim_shift > 0.0,
            "bottom trim must move bottom-anchored shaped geometry down: {untrimmed_start:?} -> {trimmed_start:?}; bounds {untrimmed_bounds:?} -> {trimmed_bounds:?}"
        );
        assert!((untrimmed_start.top.x - trimmed_start.top.x).abs() <= 0.001);
        assert!((untrimmed_end.top.x - trimmed_end.top.x).abs() <= 0.001);
        assert!((trimmed_start.bottom.y - untrimmed_start.bottom.y - trim_shift).abs() <= 0.001);
        assert!((trimmed_end.bottom.y - untrimmed_end.bottom.y - trim_shift).abs() <= 0.001);
        assert!((trimmed_selection.min_y - untrimmed_selection.min_y - trim_shift).abs() <= 0.001);
        assert!((trimmed_selection.max_y - untrimmed_selection.max_y - trim_shift).abs() <= 0.001);
        assert!((untrimmed_selection.width() - trimmed_selection.width()).abs() <= 0.001);
        assert!((untrimmed_selection.height() - trimmed_selection.height()).abs() <= 0.001);
        for (expected, caret) in [(0, trimmed_start), (1, trimmed_end)] {
            let midpoint = crate::Vec2D::new(
                (caret.top.x + caret.bottom.x) / 2.0,
                (caret.top.y + caret.bottom.y) / 2.0,
            );
            assert_eq!(frame.text_hit(instance, text, midpoint), Some(expected));
        }
        Ok(())
    }

    #[test]
    fn public_frame_text_geometry_follows_supported_runtime_modifier_transform() -> Result<()> {
        const MODIFIER_X: f32 = 50.0;
        const MODIFIER_Y: f32 = 30.0;

        let (mut baseline, baseline_instance, baseline_text) =
            imported_modifier_scene_fixture("a", 80.0, 1, 0, None)?;
        let (baseline_start, baseline_end, baseline_selection) = {
            let mut frame = baseline.frame();
            let start = frame
                .text_caret(baseline_instance, baseline_text, 0)
                .expect("baseline fixture has a start caret");
            let end = frame
                .text_caret(baseline_instance, baseline_text, 1)
                .expect("baseline fixture has an end caret");
            let selection = frame
                .text_selection_rects(baseline_instance, baseline_text, 0..1)
                .into_iter()
                .next()
                .expect("baseline fixture has one selected segment");
            (start, end, selection)
        };

        let (mut modified, modified_instance, modified_text) = imported_modifier_scene_fixture(
            "a",
            80.0,
            1,
            0,
            Some(ImportedModifierFixture {
                x: MODIFIER_X,
                y: MODIFIER_Y,
                modify_from: 0.0,
                modify_to: 1.0,
            }),
        )?;
        let mut frame = modified.frame();
        let modified_start = frame
            .text_caret(modified_instance, modified_text, 0)
            .expect("modified fixture has a start caret");
        let modified_end = frame
            .text_caret(modified_instance, modified_text, 1)
            .expect("modified fixture has an end caret");
        let modified_selection = frame
            .text_selection_rects(modified_instance, modified_text, 0..1)
            .into_iter()
            .next()
            .expect("modified fixture has one selected segment");

        for (baseline, modified) in [
            (baseline_start.top, modified_start.top),
            (baseline_start.bottom, modified_start.bottom),
            (baseline_end.top, modified_end.top),
            (baseline_end.bottom, modified_end.bottom),
        ] {
            assert!(
                (modified.x - baseline.x - MODIFIER_X).abs() <= 0.001,
                "modifier x did not reach public geometry: {baseline:?} -> {modified:?}"
            );
            assert!(
                (modified.y - baseline.y - MODIFIER_Y).abs() <= 0.001,
                "modifier y did not reach public geometry: {baseline:?} -> {modified:?}"
            );
        }
        assert!((modified_selection.min_x - baseline_selection.min_x - MODIFIER_X).abs() <= 0.001);
        assert!((modified_selection.max_x - baseline_selection.max_x - MODIFIER_X).abs() <= 0.001);
        assert!((modified_selection.min_y - baseline_selection.min_y - MODIFIER_Y).abs() <= 0.001);
        assert!((modified_selection.max_y - baseline_selection.max_y - MODIFIER_Y).abs() <= 0.001);

        for (expected, caret) in [(0, modified_start), (1, modified_end)] {
            let midpoint = crate::Vec2D::new(
                (caret.top.x + caret.bottom.x) / 2.0,
                (caret.top.y + caret.bottom.y) / 2.0,
            );
            assert_eq!(
                frame.text_hit(modified_instance, modified_text, midpoint),
                Some(expected)
            );
        }
        Ok(())
    }

    #[test]
    fn public_frame_text_geometry_preserves_both_sides_of_a_modifier_boundary() -> Result<()> {
        const SECOND_GLYPH_X: f32 = 200.0;

        let (mut baseline, baseline_instance, baseline_text) =
            imported_modifier_scene_fixture("aa", 80.0, 1, 0, None)?;
        let (baseline_boundary, baseline_selection) = {
            let mut frame = baseline.frame();
            let boundary = frame
                .text_caret(baseline_instance, baseline_text, 1)
                .expect("the unmodified glyph boundary has a caret");
            let selection = frame
                .text_selection_rects(baseline_instance, baseline_text, 0..1)
                .into_iter()
                .next()
                .expect("the first unmodified glyph has one selected segment");
            (boundary, selection)
        };

        let (mut modified, instance, text) = imported_modifier_scene_fixture(
            "aa",
            80.0,
            1,
            0,
            Some(ImportedModifierFixture {
                x: SECOND_GLYPH_X,
                y: 0.0,
                modify_from: 1.0,
                modify_to: 2.0,
            }),
        )?;
        let mut frame = modified.frame();
        let canonical_boundary = frame
            .text_caret(instance, text, 1)
            .expect("the public caret keeps downstream affinity");
        assert!(
            (canonical_boundary.top.x - baseline_boundary.top.x - SECOND_GLYPH_X).abs() <= 0.001,
            "the canonical caret must follow the translated downstream glyph: {baseline_boundary:?} -> {canonical_boundary:?}"
        );

        let first_glyph_selection = frame
            .text_selection_rects(instance, text, 0..1)
            .into_iter()
            .next()
            .expect("the first modified-layout glyph has one selected segment");
        for (actual, expected) in [
            (first_glyph_selection.min_x, baseline_selection.min_x),
            (first_glyph_selection.min_y, baseline_selection.min_y),
            (first_glyph_selection.max_x, baseline_selection.max_x),
            (first_glyph_selection.max_y, baseline_selection.max_y),
        ] {
            assert!(
                (actual - expected).abs() <= 0.001,
                "selection ending at the boundary must use glyph 0's upstream edge: {baseline_selection:?} -> {first_glyph_selection:?}"
            );
        }

        let upstream_midpoint = crate::Vec2D::new(
            (baseline_boundary.top.x + baseline_boundary.bottom.x) / 2.0,
            (baseline_boundary.top.y + baseline_boundary.bottom.y) / 2.0,
        );
        assert_eq!(
            frame.text_hit(instance, text, upstream_midpoint),
            Some(1),
            "hit testing must keep the visible upstream edge for the same source offset"
        );
        Ok(())
    }

    #[test]
    fn public_selection_keeps_modified_glyph_cells_when_range_carets_coincide() -> Result<()> {
        let (mut baseline, baseline_instance, baseline_text) =
            imported_modifier_scene_fixture("aa", 80.0, 1, 0, None)?;
        let closing_offset = {
            let mut frame = baseline.frame();
            let start = frame
                .text_caret(baseline_instance, baseline_text, 0)
                .expect("the baseline text has a start caret");
            let end = frame
                .text_caret(baseline_instance, baseline_text, 2)
                .expect("the baseline text has an end caret");
            start.top.x - end.top.x
        };

        let (mut scene, instance, text) = imported_modifier_scene_fixture(
            "aa",
            80.0,
            1,
            0,
            Some(ImportedModifierFixture {
                x: closing_offset,
                y: 0.0,
                modify_from: 1.0,
                modify_to: 2.0,
            }),
        )?;
        let mut frame = scene.frame();
        let start = frame
            .text_caret(instance, text, 0)
            .expect("the modified text has a start caret");
        let end = frame
            .text_caret(instance, text, 2)
            .expect("the modified text has an end caret");
        assert_eq!(
            start, end,
            "the fixture closes the range while leaving both glyph cells visible"
        );

        let selection = frame.text_selection_rects(instance, text, 0..2);
        assert_eq!(
            selection.len(),
            1,
            "coincident range carets must not erase visible modified glyph cells"
        );
        assert!(
            selection[0].width() > 0.001,
            "the retained selection must cover nonzero modified glyph geometry: {:?}",
            selection[0]
        );
        Ok(())
    }

    #[test]
    fn public_selection_keeps_a_modified_combining_cluster_indivisible() -> Result<()> {
        let (mut scene, instance, text) = imported_modifier_scene_fixture(
            "a\u{0301}",
            80.0,
            1,
            0,
            Some(ImportedModifierFixture {
                x: 50.0,
                y: 0.0,
                modify_from: 0.0,
                modify_to: 2.0,
            }),
        )?;
        let mut frame = scene.frame();
        let internal = frame
            .text_caret(instance, text, 1)
            .expect("the modified cluster keeps its internal source boundary");
        let end = frame
            .text_caret(instance, text, 3)
            .expect("the modified cluster has an end caret");
        assert_eq!(
            internal, end,
            "both source boundaries snap to the indivisible cluster end"
        );
        assert!(
            frame.text_selection_rects(instance, text, 1..3).is_empty(),
            "a range strictly inside one modified cluster has no visual segment"
        );
        Ok(())
    }

    #[test]
    fn public_modified_selection_excludes_the_cluster_before_a_snapped_start() -> Result<()> {
        let (mut scene, instance, text) = imported_modifier_scene_fixture(
            "a\u{0301}b",
            80.0,
            1,
            0,
            Some(ImportedModifierFixture {
                x: 50.0,
                y: 0.0,
                modify_from: 0.0,
                modify_to: 3.0,
            }),
        )?;
        let mut frame = scene.frame();
        assert_eq!(
            frame.text_caret(instance, text, 1),
            frame.text_caret(instance, text, 3),
            "the internal source boundary snaps to the first cluster end"
        );

        let snapped = frame.text_selection_rects(instance, text, 1..4);
        let after_cluster = frame.text_selection_rects(instance, text, 3..4);
        assert_eq!(
            snapped, after_cluster,
            "starting inside an indivisible cluster must not include that cluster's glyph cells"
        );
        Ok(())
    }

    #[test]
    fn public_frame_text_hit_preserves_the_upstream_side_of_a_modified_soft_wrap() -> Result<()> {
        let (mut baseline, baseline_instance, baseline_text) =
            imported_modifier_scene_fixture("aa", 8.0, 0, 0, None)?;
        let upstream_point = {
            let mut frame = baseline.frame();
            let first_line = frame
                .text_selection_rects(baseline_instance, baseline_text, 0..1)
                .into_iter()
                .next()
                .expect("the first wrapped glyph has one selected line segment");
            crate::Vec2D::new(
                first_line.max_x,
                (first_line.min_y + first_line.max_y) / 2.0,
            )
        };

        let (mut modified, instance, text) = imported_modifier_scene_fixture(
            "aa",
            8.0,
            0,
            0,
            Some(ImportedModifierFixture {
                x: 200.0,
                y: 100.0,
                modify_from: 1.0,
                modify_to: 2.0,
            }),
        )?;
        let mut frame = modified.frame();
        assert_eq!(
            frame.text_hit(instance, text, upstream_point),
            Some(1),
            "the source offset at a soft wrap keeps its first-line upstream hit target even when its downstream glyph moves"
        );
        Ok(())
    }

    #[test]
    fn public_flat_and_modified_soft_wrap_hit_choose_the_end_of_skipped_whitespace() -> Result<()> {
        let content = "a  \u{2003}a";
        let (mut baseline, baseline_instance, baseline_text) =
            imported_modifier_scene_fixture(content, 8.0, 0, 0, None)?;
        let upstream_point = {
            let mut frame = baseline.frame();
            let first_line = frame
                .text_selection_rects(baseline_instance, baseline_text, 0..1)
                .into_iter()
                .next()
                .expect("the first wrapped glyph has one selected line segment");
            let upstream = crate::Vec2D::new(
                first_line.max_x,
                (first_line.min_y + first_line.max_y) / 2.0,
            );
            let downstream = frame
                .text_caret(baseline_instance, baseline_text, 6)
                .expect("the skipped whitespace snaps to the next-line glyph");
            let downstream = crate::Vec2D::new(
                (downstream.top.x + downstream.bottom.x) / 2.0,
                (downstream.top.y + downstream.bottom.y) / 2.0,
            );
            assert_eq!(
                frame.text_hit(baseline_instance, baseline_text, upstream),
                Some(6)
            );
            assert_eq!(
                frame.text_hit(baseline_instance, baseline_text, downstream),
                Some(6)
            );
            upstream
        };

        let (mut modified, instance, text) = imported_modifier_scene_fixture(
            content,
            8.0,
            0,
            0,
            Some(ImportedModifierFixture {
                x: 200.0,
                y: 100.0,
                modify_from: 4.0,
                modify_to: 5.0,
            }),
        )?;
        let mut frame = modified.frame();
        let downstream = frame
            .text_caret(instance, text, 6)
            .expect("the end of skipped whitespace snaps to the moved next-line glyph");
        let downstream_point = crate::Vec2D::new(
            (downstream.top.x + downstream.bottom.x) / 2.0,
            (downstream.top.y + downstream.bottom.y) / 2.0,
        );
        assert_eq!(frame.text_hit(instance, text, upstream_point), Some(6));
        assert_eq!(frame.text_hit(instance, text, downstream_point), Some(6));
        Ok(())
    }

    #[test]
    fn public_frame_text_geometry_preserves_both_sides_of_a_real_follow_path_boundary() -> Result<()>
    {
        let (mut scene, instance, text) = imported_follow_path_scene_fixture()?;
        let mut frame = scene.frame();
        let canonical_downstream = frame
            .text_caret(instance, text, 1)
            .expect("the follow-path glyph boundary has a canonical caret");
        let first_glyph = frame
            .text_selection_rects(instance, text, 0..1)
            .into_iter()
            .next()
            .expect("the first follow-path glyph has one selected segment");
        let upstream_point = crate::Vec2D::new(
            first_glyph.max_x,
            (first_glyph.min_y + first_glyph.max_y) / 2.0,
        );
        for (actual, expected) in [
            (canonical_downstream.top.x, 28.554_688),
            (canonical_downstream.top.y, 50.878_906),
            (canonical_downstream.bottom.x, 5.833_334),
            (canonical_downstream.bottom.y, 50.878_906),
            (first_glyph.max_x, 10.878_906),
            (first_glyph.min_y, 31.445_313),
            (upstream_point.x, 10.878_906),
            (upstream_point.y, 42.805_99),
        ] {
            assert!(
                (actual - expected).abs() <= 0.001,
                "follow-path affinity golden changed: expected {expected}, got {actual}"
            );
        }

        assert!(
            (canonical_downstream.top.x - upstream_point.x).abs() > 0.1
                || (canonical_downstream.bottom.x - upstream_point.x).abs() > 0.1,
            "the cornered path fixture must give the boundary visibly distinct upstream and downstream caret segments"
        );
        assert_eq!(
            frame.text_hit(instance, text, upstream_point),
            Some(1),
            "the upstream side of the real follow-path boundary remains hittable"
        );
        let upstream_golden_top = crate::Vec2D::new(10.878_906, 31.445_313);
        assert_eq!(
            frame.text_hit(instance, text, upstream_golden_top),
            Some(1),
            "the draw-derived fixture endpoint must remain a boundary-1 upstream hit"
        );
        let downstream_midpoint = crate::Vec2D::new(
            (canonical_downstream.top.x + canonical_downstream.bottom.x) / 2.0,
            (canonical_downstream.top.y + canonical_downstream.bottom.y) / 2.0,
        );
        assert_eq!(
            frame.text_hit(instance, text, downstream_midpoint),
            Some(1),
            "the same source offset also remains hittable on its downstream side"
        );
        Ok(())
    }

    #[test]
    fn public_frame_text_geometry_supports_imported_fit_font_size_overflow() -> Result<()> {
        let (mut scene, instance, text) = imported_modifier_scene_fixture("aa", 80.0, 1, 5, None)?;
        let mut frame = scene.frame();
        let start = frame
            .text_caret(instance, text, 0)
            .expect("FitFontSize keeps shaped caret geometry in v1");
        let midpoint = crate::Vec2D::new(
            (start.top.x + start.bottom.x) / 2.0,
            (start.top.y + start.bottom.y) / 2.0,
        );
        assert_eq!(frame.text_hit(instance, text, midpoint), Some(0));
        assert_eq!(frame.text_selection_rects(instance, text, 0..2).len(), 1);
        Ok(())
    }

    #[test]
    fn public_text_geometry_fails_closed_for_imported_nonfinite_layout() -> Result<()> {
        let (mut scene, instance, text) =
            imported_modifier_scene_fixture("aa", f32::NAN, 0, 0, None)?;
        let mut frame = scene.frame();
        assert_eq!(frame.text_caret(instance, text, 0), None);
        assert_eq!(
            frame.text_hit(instance, text, crate::Vec2D::new(10.0, 20.0)),
            None
        );
        assert!(frame.text_selection_rects(instance, text, 0..1).is_empty());
        Ok(())
    }

    #[test]
    fn public_text_geometry_preflights_infinite_fit_font_size() -> Result<()> {
        let (mut scene, instance, text) = imported_modifier_scene_fixture_with_style_values(
            "aa",
            80.0,
            40.0,
            0,
            5,
            10.0,
            f32::INFINITY,
            20.0,
            None,
        )?;
        let mut frame = scene.frame();
        assert_eq!(frame.text_caret(instance, text, 0), None);
        assert_eq!(
            frame.text_hit(instance, text, crate::Vec2D::new(10.0, 20.0)),
            None
        );
        assert!(frame.text_selection_rects(instance, text, 0..1).is_empty());
        Ok(())
    }

    #[test]
    fn public_text_geometry_safely_searches_max_finite_fit_font_size() -> Result<()> {
        let (mut scene, instance, text) = imported_modifier_scene_fixture_with_style_values(
            "aa",
            f32::MAX,
            f32::MAX,
            0,
            5,
            10.0,
            f32::MAX,
            20.0,
            None,
        )?;
        let mut frame = scene.frame();
        let start = frame
            .text_caret(instance, text, 0)
            .expect("a huge finite FitFontSize value is searched without integer overflow");
        let midpoint = crate::Vec2D::new(
            (start.top.x + start.bottom.x) / 2.0,
            (start.top.y + start.bottom.y) / 2.0,
        );
        assert_eq!(frame.text_hit(instance, text, midpoint), Some(0));
        assert_eq!(frame.text_selection_rects(instance, text, 0..2).len(), 1);
        Ok(())
    }

    #[test]
    fn public_text_geometry_preflights_nan_line_height() -> Result<()> {
        let (mut scene, instance, text) = imported_modifier_scene_fixture_with_style_values(
            "aa",
            80.0,
            40.0,
            1,
            0,
            10.0,
            20.0,
            f32::NAN,
            None,
        )?;
        let mut frame = scene.frame();
        assert_eq!(frame.text_caret(instance, text, 0), None);
        assert_eq!(
            frame.text_hit(instance, text, crate::Vec2D::new(10.0, 20.0)),
            None
        );
        assert!(frame.text_selection_rects(instance, text, 0..1).is_empty());
        Ok(())
    }

    #[test]
    fn public_text_geometry_fails_closed_for_imported_nonfinite_transform() -> Result<()> {
        let (mut scene, instance, text) =
            imported_modifier_scene_fixture_with_x("aa", 80.0, 1, 0, f32::INFINITY, None)?;
        let mut frame = scene.frame();
        assert_eq!(frame.text_caret(instance, text, 0), None);
        assert_eq!(
            frame.text_hit(instance, text, crate::Vec2D::new(10.0, 20.0)),
            None
        );
        assert!(frame.text_selection_rects(instance, text, 0..1).is_empty());
        Ok(())
    }

    #[test]
    fn public_text_geometry_fails_closed_for_imported_nonfinite_modifier() -> Result<()> {
        for (x, y) in [(f32::NAN, 0.0), (0.0, f32::INFINITY)] {
            let (mut scene, instance, text) = imported_modifier_scene_fixture(
                "aa",
                80.0,
                1,
                0,
                Some(ImportedModifierFixture {
                    x,
                    y,
                    modify_from: 0.0,
                    modify_to: 2.0,
                }),
            )?;
            let mut frame = scene.frame();
            assert_eq!(frame.text_caret(instance, text, 0), None);
            assert_eq!(
                frame.text_hit(instance, text, crate::Vec2D::new(10.0, 20.0)),
                None
            );
            assert!(frame.text_selection_rects(instance, text, 0..1).is_empty());
        }
        Ok(())
    }

    #[test]
    fn static_trailing_newline_preserves_origin_draw_and_bounds_for_all_vertical_alignments()
    -> Result<()> {
        for vertical_align in [0, 1, 2] {
            let mut plain = owned_static_text_fixture("a", vertical_align, 0)?;
            let mut trailing_newline = owned_static_text_fixture("a\n", vertical_align, 0)?;
            assert_eq!(
                trailing_newline.world_bounds(1),
                plain.world_bounds(1),
                "a trailing static newline does not alter logical bounds for vertical alignment {vertical_align}"
            );
            assert_eq!(
                owned_draw_stream(&mut trailing_newline)?,
                owned_draw_stream(&mut plain)?,
                "a trailing static newline does not alter draw placement for vertical alignment {vertical_align}"
            );
        }
        Ok(())
    }

    #[test]
    fn static_empty_text_preserves_origin_draw_and_bounds_for_all_vertical_alignments() -> Result<()>
    {
        let mut top = owned_static_text_fixture("", 0, 0)?;
        let caret: CaretGeometry = top
            .text_caret(1, 0)
            .expect("OwnedArtboardInstance returns the named empty-text caret geometry");
        assert!(top.text_selection_rects(1, 0..0).is_empty());
        let caret_midpoint = crate::Vec2D::new(
            (caret.top.x + caret.bottom.x) / 2.0,
            (caret.top.y + caret.bottom.y) / 2.0,
        );
        assert_eq!(top.text_hit(1, caret_midpoint), Some(0));
        let expected_stream = owned_draw_stream(&mut top)?;
        assert!(
            !stream_draws_path(&expected_stream),
            "empty static Text must not emit a glyph path"
        );
        assert_eq!(
            top.world_bounds(1),
            Some(crate::Aabb::new(10.0, 10.0, 90.0, 70.0))
        );
        for vertical_align in [1, 2] {
            let mut empty = owned_static_text_fixture("", vertical_align, 0)?;
            assert_eq!(empty.world_bounds(1), top.world_bounds(1));
            assert_eq!(
                owned_draw_stream(&mut empty)?,
                expected_stream,
                "empty static draw behavior ignores vertical alignment {vertical_align}"
            );
        }
        Ok(())
    }

    #[test]
    fn borrowed_and_owned_instances_match_for_successful_text_geometry_queries() -> Result<()> {
        let file = static_text_file_fixture("a", 0, 0)?;
        let mut borrowed = file
            .default_artboard()
            .ok_or_else(|| anyhow::anyhow!("fixture has no default artboard"))?
            .instantiate()?;
        let mut owned = OwnedArtboardInstance::instantiate(Arc::clone(&file), 0)?;

        let borrowed_caret = borrowed
            .text_caret(1, 0)
            .expect("borrowed instance exposes shaped Text geometry");
        let owned_caret = owned
            .text_caret(1, 0)
            .expect("owned instance exposes shaped Text geometry");
        assert_eq!(borrowed_caret, owned_caret);
        let midpoint = crate::Vec2D::new(
            (borrowed_caret.top.x + borrowed_caret.bottom.x) / 2.0,
            (borrowed_caret.top.y + borrowed_caret.bottom.y) / 2.0,
        );
        assert_eq!(borrowed.text_hit(1, midpoint), owned.text_hit(1, midpoint));
        assert_eq!(
            borrowed.text_selection_rects(1, 0..1),
            owned.text_selection_rects(1, 0..1)
        );
        Ok(())
    }

    #[test]
    fn owned_text_geometry_applies_the_same_v1_overflow_policy_to_every_query() -> Result<()> {
        for overflow in [1, 2, 3, 77] {
            let mut instance = owned_static_text_fixture("a", 0, overflow)?;
            assert_eq!(instance.text_caret(1, 0), None);
            assert_eq!(instance.text_hit(1, crate::Vec2D::new(10.0, 10.0)), None);
            assert!(instance.text_selection_rects(1, 0..1).is_empty());
        }
        for overflow in [0, 4, 5] {
            let mut instance = owned_static_text_fixture("a", 0, overflow)?;
            let caret = instance
                .text_caret(1, 0)
                .unwrap_or_else(|| panic!("overflow value {overflow} remains supported in v1"));
            let midpoint = crate::Vec2D::new(
                (caret.top.x + caret.bottom.x) / 2.0,
                (caret.top.y + caret.bottom.y) / 2.0,
            );
            assert_eq!(instance.text_hit(1, midpoint), Some(0));
            assert_eq!(instance.text_selection_rects(1, 0..1).len(), 1);
        }
        Ok(())
    }

    #[test]
    fn owned_text_geometry_rejects_a_missing_participating_non_base_run_font() -> Result<()> {
        let mut instance = owned_multi_font_text_with_missing_second_font_fixture()?;
        assert_eq!(instance.text_caret(1, 0), None);
        assert_eq!(instance.text_hit(1, crate::Vec2D::new(0.0, 0.0)), None);
        assert!(instance.text_selection_rects(1, 0..2).is_empty());
        assert!(
            stream_draws_path(&owned_draw_stream(&mut instance)?),
            "render purpose keeps the existing partial-draw behavior"
        );
        Ok(())
    }

    fn owned_draw_stream(instance: &mut OwnedArtboardInstance) -> Result<String> {
        let mut factory = RecordingFactory::new();
        let mut cache = instance.new_render_cache();
        let mut renderer = factory.make_renderer();
        instance.draw_with_render_cache(&mut factory, &mut renderer, &mut cache)?;
        Ok(factory.stream())
    }

    fn stream_draws_path(stream: &str) -> bool {
        stream.lines().any(|line| line.starts_with("drawPath "))
    }

    fn owned_wrong_kind_asset_fixture(asset_id: u32) -> Result<OwnedArtboardInstance> {
        let image_asset_type = nuxie_schema::definition_by_name("ImageAsset")
            .ok_or_else(|| anyhow::anyhow!("ImageAsset schema definition is missing"))?
            .type_key
            .int;
        let runtime = RuntimeFile::from_authoring_records(vec![
            AuthoringRecord {
                type_key: TYPE_BACKBOARD,
                properties: Vec::new(),
            },
            AuthoringRecord {
                type_key: image_asset_type,
                properties: vec![AuthoringProperty {
                    key: PROPERTY_FILE_ASSET_ID,
                    value: AuthoringValue::Uint(u64::from(asset_id)),
                }],
            },
            AuthoringRecord {
                type_key: TYPE_ARTBOARD,
                properties: vec![
                    AuthoringProperty {
                        key: PROPERTY_LAYOUT_WIDTH,
                        value: AuthoringValue::Double(100.0),
                    },
                    AuthoringProperty {
                        key: PROPERTY_LAYOUT_HEIGHT,
                        value: AuthoringValue::Double(100.0),
                    },
                ],
            },
        ])?;
        let file = Arc::new(File::from_runtime(runtime)?);
        Ok(OwnedArtboardInstance::instantiate(file, 0)?)
    }

    #[test]
    fn owned_instance_external_font_attachment_matches_embedded_text() -> Result<()> {
        let mut embedded = owned_font_text_fixture(true)?;
        let mut external = owned_font_text_fixture(false)?;

        let embedded_stream = owned_draw_stream(&mut embedded)?;
        assert!(
            stream_draws_path(&embedded_stream),
            "the embedded oracle must draw at least one real glyph path"
        );

        let mut retained_factory = RecordingFactory::new();
        let mut retained_cache = external.new_render_cache();
        let mut retained_renderer = retained_factory.make_renderer();
        external.draw_with_render_cache(
            &mut retained_factory,
            &mut retained_renderer,
            &mut retained_cache,
        )?;
        assert!(
            !stream_draws_path(&retained_factory.stream()),
            "the unresolved external font must initially draw no glyph paths"
        );

        external.attach_font_asset_bytes(0, fixture_font_bytes())?;
        retained_factory.clear();
        external.draw_with_render_cache(
            &mut retained_factory,
            &mut retained_renderer,
            &mut retained_cache,
        )?;
        assert!(
            stream_draws_path(&retained_factory.stream()),
            "attachment must invalidate the retained cache and draw glyph paths"
        );

        assert_eq!(external.world_bounds(1), embedded.world_bounds(1));
        assert_eq!(owned_draw_stream(&mut external)?, embedded_stream);
        Ok(())
    }

    #[test]
    fn owned_instance_without_external_font_bytes_fails_closed() -> Result<()> {
        let mut external = owned_font_text_fixture(false)?;

        assert_eq!(
            external.world_bounds(1),
            Some(crate::Aabb::new(10.0, 20.0, 130.0, 60.0)),
            "fixed Text retains its logical bounds while its font is unavailable"
        );
        assert!(
            !stream_draws_path(&owned_draw_stream(&mut external)?),
            "an unresolved external font must not draw fallback or corrupt glyph paths"
        );
        Ok(())
    }

    #[test]
    fn rejected_external_font_replacement_is_atomic() -> Result<()> {
        let mut external = owned_font_text_fixture(false)?;
        external.attach_font_asset_bytes(0, fixture_font_bytes())?;
        let bounds_before = external.world_bounds(1);
        let stream_before = owned_draw_stream(&mut external)?;

        assert_eq!(
            external.attach_font_asset_bytes(0, b"not a font".to_vec()),
            Err(crate::ExternalFontAssetError::InvalidFont { asset_id: 0 })
        );

        assert_eq!(external.world_bounds(1), bounds_before);
        assert_eq!(owned_draw_stream(&mut external)?, stream_before);
        Ok(())
    }

    #[test]
    fn external_font_attachment_reports_distinct_identity_and_kind_errors() -> Result<()> {
        let mut external = owned_font_text_fixture(false)?;
        assert_eq!(
            external.attach_font_asset_bytes(99, fixture_font_bytes()),
            Err(crate::ExternalFontAssetError::UnknownAsset { asset_id: 99 })
        );

        let mut wrong_kind = owned_wrong_kind_asset_fixture(7)?;
        assert_eq!(
            wrong_kind.attach_font_asset_bytes(7, fixture_font_bytes()),
            Err(crate::ExternalFontAssetError::WrongAssetKind {
                asset_id: 7,
                actual: "ImageAsset",
            })
        );
        Ok(())
    }

    #[test]
    fn identity_allocator_reports_exhaustion_instead_of_aliasing_the_last_id() {
        let mut next = u64::MAX - 1;

        assert_eq!(
            allocate_identity(&mut next).expect("penultimate id is available"),
            u64::MAX - 1
        );
        assert_eq!(next, u64::MAX);
        assert!(allocate_identity(&mut next).is_none());
        assert_eq!(next, u64::MAX);
        assert!(allocate_identity(&mut next).is_none());
    }

    #[test]
    fn process_global_identity_allocator_never_wraps_or_reuses_an_id() {
        let next = AtomicU64::new(u64::MAX - 1);

        assert_eq!(
            allocate_global_identity(&next),
            Some(u64::MAX - 1),
            "the final representable allocation is still available"
        );
        assert_eq!(next.load(Ordering::Relaxed), u64::MAX);
        assert_eq!(allocate_global_identity(&next), None);
        assert_eq!(next.load(Ordering::Relaxed), u64::MAX);
        assert_eq!(allocate_global_identity(&next), None);
    }

    #[test]
    fn publishing_a_font_for_one_artboard_preserves_other_live_mounts() -> Result<()> {
        let mut scene = Scene::new();
        let ((first, second), _) = scene.edit(|tx| {
            let first = tx.create_artboard(ArtboardSpec {
                name: "First".into(),
                width: 200.0,
                height: 100.0,
            })?;
            let second = tx.create_artboard(ArtboardSpec {
                name: "Second".into(),
                width: 200.0,
                height: 100.0,
            })?;
            Ok((first, second))
        })?;
        let first_instance = scene.instantiate(first)?;
        let second_instance = scene.instantiate(second)?;
        let mount = |scene: &Scene, instance: InstanceId| {
            scene
                .instances
                .iter()
                .filter_map(Option::as_ref)
                .find(|live| live.id == instance)
                .map(|live| live.mount)
                .expect("live instance has a mount")
        };
        let first_mount = mount(&scene, first_instance);
        let second_mount = mount(&scene, second_instance);

        scene.edit(|tx| {
            let font = tx.create_font_asset(FontAssetSpec {
                name: "Roboto A".into(),
                bytes: fixture_font_bytes(),
            })?;
            let text = tx.create(
                Parent::Artboard(first),
                NodeSpec::Text(TextSpec {
                    name: "First label".into(),
                    x: 0.0,
                    y: 0.0,
                    opacity: 1.0,
                    rotation: 0.0,
                    scale_x: 1.0,
                    scale_y: 1.0,
                    sizing: SceneTextSizing::Fixed,
                    width: 120.0,
                    height: 40.0,
                    align: SceneTextAlign::Left,
                    wrap: SceneTextWrap::NoWrap,
                    overflow: SceneTextOverflow::Visible,
                }),
            )?;
            let style = tx.create(
                Parent::Object(text),
                NodeSpec::TextStylePaint(TextStylePaintSpec {
                    name: "First label style".into(),
                    font_size: 20.0,
                    line_height: 24.0,
                    letter_spacing: 0.0,
                    font,
                }),
            )?;
            tx.create(
                Parent::Object(text),
                NodeSpec::TextValueRun(TextValueRunSpec {
                    name: "First label run".into(),
                    text: "a".into(),
                    style,
                }),
            )?;
            Ok(())
        })?;

        assert_ne!(
            mount(&scene, first_instance),
            first_mount,
            "the artboard that first references the new font must remount"
        );
        assert_eq!(
            mount(&scene, second_instance),
            second_mount,
            "an unreferenced durable font definition must not remount another artboard"
        );
        Ok(())
    }

    #[test]
    fn nested_artboard_exports_semantic_target_as_runtime_artboard_index() -> Result<()> {
        let mut scene = Scene::new();
        let ((parent, child, host, child_shape), _) = scene.edit(|tx| {
            let parent = tx.create_artboard(ArtboardSpec {
                name: "Parent".into(),
                width: 200.0,
                height: 120.0,
            })?;
            let child = tx.create_artboard(ArtboardSpec {
                name: "Badge".into(),
                width: 40.0,
                height: 30.0,
            })?;
            let child_shape =
                create_colored_rect(tx, child, "Badge", 0.0, 0.0, 40.0, 30.0, 0xff11_2233)?;
            let host = create_nested_artboard_host(tx, parent, child, "Badge Host", 50.0, 20.0)?;
            Ok((parent, child, host, child_shape))
        })?;

        let exported = scene.export_records();
        let nested = exported
            .records()
            .iter()
            .find(|record| record.kind == ExportedObjectKind::NestedArtboard)
            .expect("export contains authored nested artboard host");
        assert!(
            nested
                .properties
                .contains(&ExportedProperty::NestedArtboardId(1))
        );
        assert!(
            nested
                .properties
                .contains(&ExportedProperty::ComponentName("Badge Host".into()))
        );

        let runtime = RuntimeFile::from_authoring_records(exported.into_authoring_records())?;
        let file = Arc::new(File::from_runtime(runtime)?);
        assert_eq!(file.artboard_count(), 2);
        let mut instance = OwnedArtboardInstance::instantiate(file, 0)?;
        let mut factory = RecordingFactory::new();
        let mut cache = instance.new_render_cache();
        let mut renderer = factory.make_renderer();
        instance.draw_with_render_cache(&mut factory, &mut renderer, &mut cache)?;
        let stream = parse_single_frame(&factory.stream())?;
        let commands = &stream.frames[0].commands;
        assert!(
            commands
                .iter()
                .any(|command| format!("{command:?}").contains("color: 4279312947")),
            "the parent draw should include the nested child rectangle color: {commands:?}"
        );

        let _ = parent;
        let _ = child;
        let instance = scene.instantiate(parent)?;
        let mut frame = scene.frame();
        let paths = frame.hit_test_paths(instance, crate::Vec2D::new(50.0, 20.0));
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].objects(), &[host, child_shape]);
        assert_eq!(
            frame.hit_test(instance, crate::Vec2D::new(50.0, 20.0)),
            vec![child_shape]
        );
        Ok(())
    }

    #[test]
    fn image_export_remaps_semantic_asset_and_reimports_as_runtime_file() -> Result<()> {
        let mut scene = Scene::new();
        scene.edit(|tx| {
            let image = tx.create_image_asset(ImageAssetSpec {
                name: "Photo".into(),
                bytes: b"opaque image bytes".to_vec(),
            })?;
            let artboard = tx.create_artboard(ArtboardSpec {
                name: "Images".into(),
                width: 200.0,
                height: 100.0,
            })?;
            tx.create(
                Parent::Artboard(artboard),
                NodeSpec::Image(ImageSpec {
                    name: "Hero".into(),
                    x: 10.0,
                    y: 20.0,
                    opacity: 0.75,
                    rotation: 0.25,
                    scale_x: 1.5,
                    scale_y: 2.0,
                    image,
                    origin_x: 0.25,
                    origin_y: 0.75,
                    fit: 2,
                    alignment_x: -0.5,
                    alignment_y: 0.5,
                    crop: Some(ImageCropRect {
                        x: 0.25,
                        y: 0.125,
                        width: 0.5,
                        height: 0.75,
                    }),
                }),
            )?;
            Ok(())
        })?;

        let exported = scene.export_records();
        let image_asset = exported
            .records()
            .iter()
            .find(|record| record.kind == ExportedObjectKind::ImageAsset)
            .expect("exported records contain the referenced image asset");
        assert_eq!(
            image_asset.properties,
            vec![
                ExportedProperty::AssetName("Photo".into()),
                ExportedProperty::FileAssetId(0),
            ]
        );
        let image = exported
            .records()
            .iter()
            .find(|record| record.kind == ExportedObjectKind::Image)
            .expect("exported records contain the authored image node");
        assert!(
            image
                .properties
                .contains(&ExportedProperty::ImageAssetId(0)),
            "image nodes use the dense runtime-local asset id"
        );
        assert!(
            image.properties.contains(&ExportedProperty::ImageFit(2)),
            "image nodes export typed fit semantics"
        );
        assert!(
            image
                .properties
                .contains(&ExportedProperty::ImageAlignmentX(-0.5)),
            "image nodes export horizontal crop alignment"
        );
        assert!(
            image
                .properties
                .contains(&ExportedProperty::ImageAlignmentY(0.5)),
            "image nodes export vertical crop alignment"
        );
        let mesh = exported
            .records()
            .iter()
            .find(|record| record.kind == ExportedObjectKind::Mesh)
            .expect("cropped image export synthesizes a Rive mesh");
        assert!(
            mesh.properties
                .contains(&ExportedProperty::MeshTriangleIndexBytes(vec![
                    0, 1, 2, 0, 2, 3,
                ])),
            "cropped image mesh exports two varuint-encoded triangles"
        );
        let vertices = exported
            .records()
            .iter()
            .filter(|record| record.kind == ExportedObjectKind::MeshVertex)
            .collect::<Vec<_>>();
        assert_eq!(vertices.len(), 4, "cropped image export uses one quad mesh");
        for (vertex, expected) in vertices.iter().zip([
            (0.0, 0.0, 0.25, 0.125),
            (1.0, 0.0, 0.75, 0.125),
            (1.0, 1.0, 0.75, 0.875),
            (0.0, 1.0, 0.25, 0.875),
        ]) {
            assert!(
                vertex
                    .properties
                    .contains(&ExportedProperty::VertexX(expected.0))
            );
            assert!(
                vertex
                    .properties
                    .contains(&ExportedProperty::VertexY(expected.1))
            );
            assert!(
                vertex
                    .properties
                    .contains(&ExportedProperty::MeshVertexU(expected.2))
            );
            assert!(
                vertex
                    .properties
                    .contains(&ExportedProperty::MeshVertexV(expected.3))
            );
        }

        let runtime = RuntimeFile::from_authoring_records(exported.into_authoring_records())?;
        assert_eq!(runtime.artboards().len(), 1);
        assert!(
            runtime
                .artboard_local_object_slots(0)
                .expect("artboard has validated local objects")
                .iter()
                .flatten()
                .any(|object| object.type_name == "Image"),
            "the binary importer keeps the image as an artboard-local object"
        );
        let imported_objects = runtime
            .artboard_local_object_slots(0)
            .expect("artboard has validated local objects");
        let imported_mesh = imported_objects
            .iter()
            .flatten()
            .find(|object| object.type_name == "Mesh")
            .expect("the binary importer keeps the synthetic crop mesh");
        assert_eq!(
            imported_mesh.mesh_triangle_indices(),
            Some(vec![0, 1, 2, 0, 2, 3]),
            "the authored varuint stream must round-trip as two nondegenerate triangles",
        );
        Ok(())
    }

    #[test]
    fn mixed_file_assets_share_one_dense_ordinal_and_script_defaults_survive_import() -> Result<()>
    {
        let mut scene = Scene::new();
        scene.edit(|tx| {
            let font = tx.create_font_asset(FontAssetSpec {
                name: "Roboto".into(),
                bytes: fixture_font_bytes(),
            })?;
            let image = tx.create_image_asset(ImageAssetSpec {
                name: "Photo".into(),
                bytes: b"image bytes".to_vec(),
            })?;
            tx.create_script_asset(ScriptAssetSpec {
                name: "helpers".into(),
                is_module: true,
                bytes: b"module bytecode".to_vec(),
            })?;
            let protocol = tx.create_script_asset(ScriptAssetSpec {
                name: "node".into(),
                is_module: false,
                bytes: b"node bytecode".to_vec(),
            })?;
            tx.create_shader_asset(ShaderAssetSpec {
                name: "fill".into(),
                bytes: b"RSTB bytes".to_vec(),
            })?;
            let artboard = tx.create_artboard(ArtboardSpec {
                name: "Mixed assets".into(),
                width: 100.0,
                height: 100.0,
            })?;
            tx.create(
                Parent::Artboard(artboard),
                NodeSpec::Image(ImageSpec {
                    name: "Image first".into(),
                    x: 0.0,
                    y: 0.0,
                    opacity: 1.0,
                    rotation: 0.0,
                    scale_x: 1.0,
                    scale_y: 1.0,
                    image,
                    origin_x: 0.0,
                    origin_y: 0.0,
                    fit: 0,
                    alignment_x: 0.0,
                    alignment_y: 0.0,
                    crop: None,
                }),
            )?;
            let text = tx.create(
                Parent::Artboard(artboard),
                NodeSpec::Text(TextSpec {
                    name: "Text second".into(),
                    x: 0.0,
                    y: 0.0,
                    opacity: 1.0,
                    rotation: 0.0,
                    scale_x: 1.0,
                    scale_y: 1.0,
                    sizing: SceneTextSizing::Fixed,
                    width: 80.0,
                    height: 30.0,
                    align: SceneTextAlign::Left,
                    wrap: SceneTextWrap::Wrap,
                    overflow: SceneTextOverflow::Visible,
                }),
            )?;
            tx.create(
                Parent::Object(text),
                NodeSpec::TextStylePaint(TextStylePaintSpec {
                    name: "Style".into(),
                    font_size: 16.0,
                    line_height: 20.0,
                    letter_spacing: 0.0,
                    font,
                }),
            )?;
            tx.create(
                Parent::Artboard(artboard),
                NodeSpec::ScriptedDrawable(ScriptedDrawableSpec {
                    name: "Scripted".into(),
                    x: 0.0,
                    y: 0.0,
                    opacity: 1.0,
                    rotation: 0.0,
                    scale_x: 1.0,
                    scale_y: 1.0,
                    script: protocol,
                }),
            )?;
            Ok(())
        })?;

        let exported = scene.export_records();
        let [
            backboard,
            image_asset,
            image_contents,
            font_asset,
            font_contents,
            module_asset,
            module_contents,
            protocol_asset,
            protocol_contents,
            shader_asset,
            shader_contents,
            ..,
        ] = exported.records()
        else {
            panic!("mixed scene must contain the complete asset phase");
        };
        assert_eq!(
            [
                backboard,
                image_asset,
                image_contents,
                font_asset,
                font_contents,
                module_asset,
                module_contents,
                protocol_asset,
                protocol_contents,
                shader_asset,
                shader_contents,
            ]
            .into_iter()
            .map(|record| record.kind)
            .collect::<Vec<_>>(),
            vec![
                ExportedObjectKind::Backboard,
                ExportedObjectKind::ImageAsset,
                ExportedObjectKind::FileAssetContents,
                ExportedObjectKind::FontAsset,
                ExportedObjectKind::FileAssetContents,
                ExportedObjectKind::ScriptAsset,
                ExportedObjectKind::FileAssetContents,
                ExportedObjectKind::ScriptAsset,
                ExportedObjectKind::FileAssetContents,
                ExportedObjectKind::ShaderAsset,
                ExportedObjectKind::FileAssetContents,
            ],
            "each phase uses adjacent asset/contents records before the artboard phase"
        );
        let asset_records = exported
            .records()
            .iter()
            .filter(|record| {
                matches!(
                    record.kind,
                    ExportedObjectKind::FontAsset
                        | ExportedObjectKind::ImageAsset
                        | ExportedObjectKind::ScriptAsset
                        | ExportedObjectKind::ShaderAsset
                )
            })
            .collect::<Vec<_>>();
        let [
            image_asset,
            font_asset,
            module_asset,
            protocol_asset,
            shader_asset,
        ] = asset_records.as_slice()
        else {
            panic!("mixed scene must export exactly five file assets");
        };
        assert_eq!(
            [
                *image_asset,
                *font_asset,
                *module_asset,
                *protocol_asset,
                *shader_asset,
            ]
            .into_iter()
            .map(|record| record.kind)
            .collect::<Vec<_>>(),
            vec![
                ExportedObjectKind::ImageAsset,
                ExportedObjectKind::FontAsset,
                ExportedObjectKind::ScriptAsset,
                ExportedObjectKind::ScriptAsset,
                ExportedObjectKind::ShaderAsset,
            ],
            "referenced font/image first-use is followed by authored scripts, then authored shaders"
        );
        for (expected, record) in [
            *image_asset,
            *font_asset,
            *module_asset,
            *protocol_asset,
            *shader_asset,
        ]
        .into_iter()
        .enumerate()
        {
            assert!(record.properties.contains(&ExportedProperty::FileAssetId(
                u32::try_from(expected).expect("five assets fit in u32")
            )));
        }
        assert!(
            module_asset
                .properties
                .contains(&ExportedProperty::ScriptAssetIsModule(true))
        );
        assert!(
            !protocol_asset
                .properties
                .contains(&ExportedProperty::ScriptAssetIsModule(false)),
            "isModule=false stays implicit at its schema default"
        );

        let runtime =
            RuntimeFile::from_authoring_records(exported.clone().into_authoring_records())?;
        assert_eq!(
            runtime
                .file_assets()
                .iter()
                .map(|asset| asset.type_name)
                .collect::<Vec<_>>(),
            vec![
                "ImageAsset",
                "FontAsset",
                "ScriptAsset",
                "ScriptAsset",
                "ShaderAsset",
            ]
        );
        let imported_scripts = runtime
            .file_assets()
            .into_iter()
            .filter(|asset| asset.type_name == "ScriptAsset")
            .collect::<Vec<_>>();
        let [imported_module, imported_protocol] = imported_scripts.as_slice() else {
            panic!("both authored scripts must import");
        };
        assert_eq!(imported_module.bool_property("isModule"), Some(true));
        assert_eq!(imported_protocol.bool_property("isModule"), Some(false));
        for script in [*imported_module, *imported_protocol] {
            assert_eq!(
                script.uint_property("serializedImplementedMethods"),
                Some(2_097_151),
                "the omitted implemented-methods mask imports at the compatibility default"
            );
        }
        let imported_drawable = runtime
            .objects
            .iter()
            .flatten()
            .find(|object| object.type_name == "ScriptedDrawable")
            .expect("the scripted drawable survives record import");
        assert_eq!(imported_drawable.uint_property("scriptAssetId"), Some(3));
        assert_eq!(
            runtime
                .resolved_file_asset_for_referencer(imported_drawable)
                .map(|asset| asset.string_property("name")),
            Some(Some("node"))
        );
        File::from_runtime(runtime)?;
        assert_eq!(
            scene.export_records(),
            exported,
            "export is a record fixpoint"
        );
        Ok(())
    }

    #[test]
    fn nested_artboard_reference_failures_are_structural_edit_errors() -> Result<()> {
        let mut scene = Scene::new();
        let parent = scene
            .edit(|tx| {
                tx.create_artboard(ArtboardSpec {
                    name: "Parent".into(),
                    width: 200.0,
                    height: 120.0,
                })
            })?
            .0;
        let missing = ArtboardId(u64::MAX);
        let err = scene
            .edit(|tx| {
                create_nested_artboard_host(tx, parent, missing, "Missing Host", 0.0, 0.0)?;
                Ok(())
            })
            .expect_err("unknown nested target artboard must reject the transaction");
        assert_eq!(err.diagnostic().reason, EditReason::UnknownArtboard);

        let mut scene = Scene::new();
        let err = scene
            .edit(|tx| {
                let first = tx.create_artboard(ArtboardSpec {
                    name: "First".into(),
                    width: 100.0,
                    height: 100.0,
                })?;
                let second = tx.create_artboard(ArtboardSpec {
                    name: "Second".into(),
                    width: 100.0,
                    height: 100.0,
                })?;
                create_nested_artboard_host(tx, first, second, "First Host", 0.0, 0.0)?;
                create_nested_artboard_host(tx, second, first, "Second Host", 0.0, 0.0)?;
                Ok((first, second))
            })
            .expect_err("cyclic nested artboard references must reject the transaction");
        assert_eq!(err.diagnostic().reason, EditReason::CycleDetected);
        Ok(())
    }

    #[test]
    fn editing_nested_child_artboard_remounts_live_parent_instances() -> Result<()> {
        let mut scene = Scene::new();
        let ((parent, child), _) = scene.edit(|tx| {
            let parent = tx.create_artboard(ArtboardSpec {
                name: "Parent".into(),
                width: 200.0,
                height: 120.0,
            })?;
            let child = tx.create_artboard(ArtboardSpec {
                name: "Badge".into(),
                width: 40.0,
                height: 30.0,
            })?;
            create_colored_rect(tx, child, "Badge", 0.0, 0.0, 40.0, 30.0, 0xff11_2233)?;
            create_nested_artboard_host(tx, parent, child, "Badge Host", 50.0, 20.0)?;
            Ok((parent, child))
        })?;
        let parent_instance = scene.instantiate(parent)?;
        let parent_mount = scene
            .instances
            .iter()
            .filter_map(Option::as_ref)
            .find(|live| live.id == parent_instance)
            .map(|live| live.mount)
            .expect("parent instance has a live mount");

        scene.edit(|tx| {
            create_colored_rect(tx, child, "New Badge", 5.0, 5.0, 10.0, 10.0, 0xff44_5566)?;
            Ok(())
        })?;

        let next_parent_mount = scene
            .instances
            .iter()
            .filter_map(Option::as_ref)
            .find(|live| live.id == parent_instance)
            .map(|live| live.mount)
            .expect("parent instance remains live");
        assert_ne!(
            next_parent_mount, parent_mount,
            "editing a referenced child artboard must remount parent instances"
        );
        Ok(())
    }

    #[test]
    fn multi_artboard_typed_export_reimports_through_the_runtime_and_draws_identically()
    -> Result<()> {
        let mut scene = Scene::new();
        let ((first, second), _) = scene.edit(|tx| {
            let mut create = |name: &str, x: f32, color: u32| {
                let artboard = tx.create_artboard(ArtboardSpec {
                    name: name.to_owned(),
                    width: 100.0,
                    height: 80.0,
                })?;
                let shape = tx.create(
                    Parent::Artboard(artboard),
                    NodeSpec::Shape(ShapeSpec {
                        name: format!("{name} Shape"),
                        x,
                        y: 40.0,
                        opacity: 1.0,
                        rotation: 0.0,
                        scale_x: 1.0,
                        scale_y: 1.0,
                    }),
                )?;
                tx.create(
                    Parent::Object(shape),
                    NodeSpec::Rectangle(RectangleSpec::new(
                        format!("{name} Rectangle"),
                        40.0,
                        30.0,
                    )),
                )?;
                let fill = tx.create(
                    Parent::Object(shape),
                    NodeSpec::Fill(FillSpec {
                        name: format!("{name} Fill"),
                    }),
                )?;
                tx.create(
                    Parent::Object(fill),
                    NodeSpec::SolidColor(SolidColorSpec {
                        name: format!("{name} Color"),
                        color,
                    }),
                )?;
                Ok::<_, EditAbort>(artboard)
            };
            Ok((
                create("First", 25.0, 0xff11_2233)?,
                create("Second", 75.0, 0xff44_5566)?,
            ))
        })?;
        let first_instance = scene.instantiate(first)?;
        let second_instance = scene.instantiate(second)?;
        let live_streams = [first_instance, second_instance]
            .into_iter()
            .map(|instance| {
                let mut factory = RecordingFactory::new();
                let mut cache = scene.new_render_cache(instance)?;
                let mut renderer = factory.make_renderer();
                scene
                    .frame()
                    .draw(instance, &mut factory, &mut renderer, &mut cache)?;
                Ok::<_, anyhow::Error>(factory.stream())
            })
            .collect::<Result<Vec<_>>>()?;

        let runtime =
            RuntimeFile::from_authoring_records(scene.export_records().into_authoring_records())?;
        let file = Arc::new(File::from_runtime(runtime)?);
        assert_eq!(file.artboard_count(), 2);
        for (index, expected) in live_streams.iter().enumerate() {
            let mut instance = OwnedArtboardInstance::instantiate(Arc::clone(&file), index)?;
            let mut factory = RecordingFactory::new();
            let mut cache = instance.new_render_cache();
            let mut renderer = factory.make_renderer();
            instance.draw_with_render_cache(&mut factory, &mut renderer, &mut cache)?;
            let imported = parse_single_frame(&factory.stream())?;
            let live = parse_single_frame(expected)?;
            assert!(imported.resources.is_empty());
            assert!(live.resources.is_empty());
            assert_eq!(imported.frames.len(), 1);
            assert_eq!(live.frames.len(), 1);
            let imported_commands = imported
                .frames
                .first()
                .map(|frame| &frame.commands)
                .ok_or_else(|| anyhow::anyhow!("imported export produced no render frame"))?;
            let live_commands = live
                .frames
                .first()
                .map(|frame| &frame.commands)
                .ok_or_else(|| anyhow::anyhow!("live scene produced no render frame"))?;
            assert_eq!(
                imported_commands, live_commands,
                "combined-file source-paint allocation may renumber resources, but the typed draw commands must remain identical"
            );
        }
        Ok(())
    }

    #[test]
    fn text_export_remaps_semantic_references_and_reimports_with_identical_draw_and_bounds()
    -> Result<()> {
        let mut scene = Scene::new();
        let ((artboard, text), _) = scene.edit(|tx| {
            let first_font = tx.create_font_asset(FontAssetSpec {
                name: "First".into(),
                bytes: fixture_font_bytes(),
            })?;
            let second_font = tx.create_font_asset(FontAssetSpec {
                name: "Second".into(),
                bytes: fixture_font_bytes(),
            })?;
            let artboard = tx.create_artboard(ArtboardSpec {
                name: "Text".into(),
                width: 200.0,
                height: 100.0,
            })?;
            let text = tx.create(
                Parent::Artboard(artboard),
                NodeSpec::Text(TextSpec {
                    name: "Title".into(),
                    x: 0.0,
                    y: 0.0,
                    opacity: 1.0,
                    rotation: 0.0,
                    scale_x: 1.0,
                    scale_y: 1.0,
                    sizing: SceneTextSizing::Fixed,
                    width: 120.0,
                    height: 40.0,
                    align: SceneTextAlign::Center,
                    wrap: SceneTextWrap::NoWrap,
                    overflow: SceneTextOverflow::Ellipsis,
                }),
            )?;
            let first_style = tx.create(
                Parent::Object(text),
                NodeSpec::TextStylePaint(TextStylePaintSpec {
                    name: "First Style".into(),
                    font_size: 12.0,
                    line_height: 14.0,
                    letter_spacing: 0.0,
                    font: first_font,
                }),
            )?;
            let second_style = tx.create(
                Parent::Object(text),
                NodeSpec::TextStylePaint(TextStylePaintSpec {
                    name: "Second Style".into(),
                    font_size: 24.0,
                    line_height: 30.0,
                    letter_spacing: 0.0,
                    font: second_font,
                }),
            )?;
            let first_fill = tx.create(
                Parent::Object(first_style),
                NodeSpec::Fill(FillSpec {
                    name: "First Fill".into(),
                }),
            )?;
            let second_fill = tx.create(
                Parent::Object(second_style),
                NodeSpec::Fill(FillSpec {
                    name: "Second Fill".into(),
                }),
            )?;
            tx.create(
                Parent::Object(first_fill),
                NodeSpec::SolidColor(SolidColorSpec {
                    name: "First Color".into(),
                    color: 0xff11_2233,
                }),
            )?;
            tx.create(
                Parent::Object(second_fill),
                NodeSpec::SolidColor(SolidColorSpec {
                    name: "Second Color".into(),
                    color: 0xff44_5566,
                }),
            )?;
            tx.create(
                Parent::Object(text),
                NodeSpec::TextValueRun(TextValueRunSpec {
                    name: "First Run".into(),
                    text: "a".into(),
                    style: first_style,
                }),
            )?;
            tx.create(
                Parent::Object(text),
                NodeSpec::TextValueRun(TextValueRunSpec {
                    name: "Second Run".into(),
                    text: "a".into(),
                    style: second_style,
                }),
            )?;
            Ok((artboard, text))
        })?;
        let instance = scene.instantiate(artboard)?;
        let mut live_factory = RecordingFactory::new();
        let mut live_cache = scene.new_render_cache(instance)?;
        let mut live_renderer = live_factory.make_renderer();
        scene.frame().draw(
            instance,
            &mut live_factory,
            &mut live_renderer,
            &mut live_cache,
        )?;
        assert_eq!(
            scene.frame().world_bounds(instance, text),
            Some(crate::Aabb::new(0.0, 0.0, 120.0, 40.0))
        );

        let exported = scene.export_records();
        let font_records = exported
            .records()
            .iter()
            .filter(|record| record.kind == ExportedObjectKind::FontAsset)
            .map(|record| record.properties.clone())
            .collect::<Vec<_>>();
        assert_eq!(
            font_records,
            vec![
                vec![
                    ExportedProperty::AssetName("First".into()),
                    ExportedProperty::FileAssetId(0),
                ],
                vec![
                    ExportedProperty::AssetName("Second".into()),
                    ExportedProperty::FileAssetId(1),
                ],
            ]
        );
        let text_record = exported
            .records()
            .iter()
            .find(|record| record.kind == ExportedObjectKind::Text)
            .expect("the text frame is exported");
        assert_eq!(
            text_record.properties,
            vec![
                ExportedProperty::ComponentName("Title".into()),
                ExportedProperty::TranslateX(0.0),
                ExportedProperty::TranslateY(0.0),
                ExportedProperty::TextAlign(SceneTextAlign::Center),
                ExportedProperty::TextSizing(SceneTextSizing::Fixed),
                ExportedProperty::TextWidth(120.0),
                ExportedProperty::TextHeight(40.0),
                ExportedProperty::TextOverflow(SceneTextOverflow::Ellipsis),
                ExportedProperty::TextWrap(SceneTextWrap::NoWrap),
            ]
        );

        let ordered_nodes = exported
            .records()
            .iter()
            .filter_map(|record| {
                matches!(
                    record.kind,
                    ExportedObjectKind::Text
                        | ExportedObjectKind::TextStylePaint
                        | ExportedObjectKind::Fill
                        | ExportedObjectKind::SolidColor
                        | ExportedObjectKind::TextValueRun
                )
                .then(|| {
                    let name = record
                        .properties
                        .iter()
                        .find_map(|property| match property {
                            ExportedProperty::ComponentName(name) => Some(name.clone()),
                            _ => None,
                        })?;
                    let parent = record
                        .properties
                        .iter()
                        .find_map(|property| match property {
                            ExportedProperty::ParentId(parent) => Some(*parent),
                            _ => None,
                        });
                    Some((record.kind, name, parent))
                })?
            })
            .collect::<Vec<_>>();
        assert_eq!(
            ordered_nodes,
            vec![
                (ExportedObjectKind::Text, "Title".into(), None),
                (
                    ExportedObjectKind::TextStylePaint,
                    "First Style".into(),
                    Some(1),
                ),
                (
                    ExportedObjectKind::TextStylePaint,
                    "Second Style".into(),
                    Some(1),
                ),
                (ExportedObjectKind::Fill, "First Fill".into(), Some(2)),
                (ExportedObjectKind::Fill, "Second Fill".into(), Some(3)),
                (
                    ExportedObjectKind::SolidColor,
                    "First Color".into(),
                    Some(4),
                ),
                (
                    ExportedObjectKind::SolidColor,
                    "Second Color".into(),
                    Some(5),
                ),
                (
                    ExportedObjectKind::TextValueRun,
                    "First Run".into(),
                    Some(1),
                ),
                (
                    ExportedObjectKind::TextValueRun,
                    "Second Run".into(),
                    Some(1),
                ),
            ]
        );
        for (name, expected_font) in [("First Style", 0), ("Second Style", 1)] {
            let style = exported
                .records()
                .iter()
                .find(|record| {
                    record.kind == ExportedObjectKind::TextStylePaint
                        && record
                            .properties
                            .contains(&ExportedProperty::ComponentName(name.into()))
                })
                .expect("named text style record");
            assert!(
                style
                    .properties
                    .contains(&ExportedProperty::TextStyleFontAssetId(expected_font))
            );
        }
        for (name, expected_style) in [("First Run", 2), ("Second Run", 3)] {
            let run = exported
                .records()
                .iter()
                .find(|record| {
                    record.kind == ExportedObjectKind::TextValueRun
                        && record
                            .properties
                            .contains(&ExportedProperty::ComponentName(name.into()))
                })
                .expect("named text run record");
            assert!(
                run.properties
                    .contains(&ExportedProperty::TextValueRunStyleId(expected_style))
            );
        }

        let runtime = RuntimeFile::from_authoring_records(exported.into_authoring_records())?;
        let file = Arc::new(File::from_runtime(runtime)?);
        let mut imported = OwnedArtboardInstance::instantiate(file, 0)?;
        let mut imported_factory = RecordingFactory::new();
        let mut imported_cache = imported.new_render_cache();
        let mut imported_renderer = imported_factory.make_renderer();
        imported.draw_with_render_cache(
            &mut imported_factory,
            &mut imported_renderer,
            &mut imported_cache,
        )?;

        assert_eq!(
            imported.world_bounds(1),
            Some(crate::Aabb::new(0.0, 0.0, 120.0, 40.0))
        );
        let imported_frame = parse_single_frame(&imported_factory.stream())?;
        let live_frame = parse_single_frame(&live_factory.stream())?;
        assert_eq!(imported_frame.frames, live_frame.frames);
        Ok(())
    }

    fn work_test_shape(name: &str) -> NodeSpec {
        NodeSpec::Shape(ShapeSpec {
            name: name.to_owned(),
            x: 0.0,
            y: 0.0,
            opacity: 1.0,
            rotation: 0.0,
            scale_x: 1.0,
            scale_y: 1.0,
        })
    }

    #[test]
    fn all_record_kinds_share_one_store_index_and_remove_restore_token() -> Result<()> {
        let mut scene = Scene::new();
        let ((artboard, shape, animation), _) = scene.edit(|tx| {
            let artboard = tx.create_artboard(ArtboardSpec {
                name: "Main".into(),
                width: 100.0,
                height: 100.0,
            })?;
            let shape = tx.create(Parent::Artboard(artboard), work_test_shape("Shape"))?;
            tx.create(
                Parent::Object(shape),
                NodeSpec::Rectangle(RectangleSpec::new("Rect", 50.0, 50.0)),
            )?;
            let animation = tx.animations().create_linear(
                artboard,
                LinearAnimationSpec {
                    name: "Fade".into(),
                    fps: 60,
                    duration: 60,
                },
            )?;
            tx.animations()
                .set_key(animation, shape, props::WORLD_OPACITY, 0, 0.0)?;
            Ok((artboard, shape, animation))
        })?;

        scene.edit(|tx| {
            let removed_animation = tx.remove(animation.object_id())?;
            assert_eq!(removed_animation.records.len(), 4);
            assert!(
                removed_animation
                    .records
                    .iter()
                    .all(|record| matches!(record.definition.spec, RecordSpec::Animation(_)))
            );

            let visual_records = tx.remove(shape)?;
            assert_eq!(visual_records.records.len(), 2);
            assert!(
                visual_records
                    .records
                    .iter()
                    .all(|record| matches!(record.definition.spec, RecordSpec::Visual { .. }))
            );

            assert_eq!(tx.restore(visual_records)?, shape);
            assert_eq!(tx.restore(removed_animation)?, animation.object_id());
            Ok(())
        })?;

        let definition = scene
            .definitions
            .artboards
            .iter()
            .find(|candidate| candidate.id == artboard)
            .expect("artboard definition");
        let index = DefinitionIndex::build(&scene.definitions);
        assert_eq!(definition.records.len(), 6);
        assert_eq!(index.objects.len(), definition.records.len());
        assert_eq!(index.owned.get(&shape).map(Vec::len), Some(1));
        assert_eq!(
            index.owned.get(&animation.object_id()).map(Vec::len),
            Some(1)
        );
        Ok(())
    }

    fn rejected_bulk_animation_keys_work(key_count: usize) -> Result<SceneWork> {
        let mut scene = Scene::new();
        let duration = u32::try_from(key_count)?;
        let ((artboard, shape, animation), _) = scene.edit(|tx| {
            let artboard = tx.create_artboard(ArtboardSpec {
                name: "Main".into(),
                width: 100.0,
                height: 100.0,
            })?;
            let shape = tx.create(Parent::Artboard(artboard), work_test_shape("Shape"))?;
            let animation = tx.animations().create_linear(
                artboard,
                LinearAnimationSpec {
                    name: "Fade".into(),
                    fps: 60,
                    duration,
                },
            )?;
            Ok((artboard, shape, animation))
        })?;

        reset_scene_work();
        scene
            .edit(|tx| {
                tx.set_artboard(
                    artboard,
                    ArtboardSpec {
                        name: "Reject after indexed animation authoring".into(),
                        width: f32::NAN,
                        height: 100.0,
                    },
                )?;
                for frame in 0..duration {
                    tx.animations()
                        .set_key(animation, shape, props::WORLD_OPACITY, frame, 0.5)?;
                }
                Ok(())
            })
            .expect_err("invalid artboard rejects after animation authoring work");
        Ok(scene_work())
    }

    #[test]
    fn bulk_animation_key_upserts_do_not_rebuild_or_rescan_the_growing_record_store() -> Result<()>
    {
        for key_count in [1_024, 2_048] {
            assert_eq!(
                rejected_bulk_animation_keys_work(key_count)?,
                SceneWork {
                    definition_index_builds: 1,
                    definition_index_node_visits: 2,
                    receipt_membership_checks: key_count + 2,
                }
            );
        }
        Ok(())
    }

    #[test]
    fn definition_index_work_counts_visual_and_nonvisual_ordinary_records() -> Result<()> {
        let mut scene = Scene::new();
        scene.edit(|tx| {
            let artboard = tx.create_artboard(ArtboardSpec {
                name: "Main".into(),
                width: 100.0,
                height: 100.0,
            })?;
            let shape = tx.create(Parent::Artboard(artboard), work_test_shape("Shape"))?;
            let animation = tx.animations().create_linear(
                artboard,
                LinearAnimationSpec {
                    name: "Fade".into(),
                    fps: 60,
                    duration: 60,
                },
            )?;
            tx.animations()
                .set_key(animation, shape, props::WORLD_OPACITY, 0, 0.0)?;
            Ok(())
        })?;

        reset_scene_work();
        scene
            .edit(|tx| Err::<(), _>(tx.abort("count the initial record index")))
            .expect_err("requested abort");
        assert_eq!(
            scene_work(),
            SceneWork {
                definition_index_builds: 1,
                definition_index_node_visits: 5,
                receipt_membership_checks: 0,
            }
        );
        Ok(())
    }

    fn child_order_scene(names: &[&str]) -> Result<(Scene, ArtboardId, Vec<ObjectId>)> {
        let mut scene = Scene::new();
        let ((artboard, roots), _) = scene.edit(|tx| {
            let artboard = tx.create_artboard(ArtboardSpec {
                name: "Main".into(),
                width: 100.0,
                height: 100.0,
            })?;
            let roots = names
                .iter()
                .map(|name| tx.create(Parent::Artboard(artboard), work_test_shape(name)))
                .collect::<std::result::Result<Vec<_>, _>>()?;
            Ok((artboard, roots))
        })?;
        Ok((scene, artboard, roots))
    }

    #[test]
    fn exact_child_order_refreshes_the_index_once_and_matches_a_fresh_scene() -> Result<()> {
        let (mut scene, artboard, roots) = child_order_scene(&["First", "Second", "Third"])?;
        let (fresh, _, _) = child_order_scene(&["Third", "First", "Second"])?;

        reset_scene_work();
        scene.edit(|tx| {
            tx.set_child_order(Parent::Artboard(artboard), &[roots[2], roots[0], roots[1]])
        })?;

        assert_eq!(
            scene_work(),
            SceneWork {
                definition_index_builds: 2,
                definition_index_node_visits: 6,
                receipt_membership_checks: 0,
            }
        );
        assert_eq!(
            scene.export_records().records(),
            fresh.export_records().records()
        );
        Ok(())
    }

    #[test]
    fn exact_child_order_rejects_malformed_sets_without_mutating() -> Result<()> {
        let (mut scene, artboard, roots) = child_order_scene(&["First", "Second", "Third"])?;
        let (foreign_artboard, foreign) = scene
            .edit(|tx| {
                let foreign_artboard = tx.create_artboard(ArtboardSpec {
                    name: "Foreign".into(),
                    width: 100.0,
                    height: 100.0,
                })?;
                let foreign = tx.create(
                    Parent::Artboard(foreign_artboard),
                    work_test_shape("Foreign"),
                )?;
                Ok((foreign_artboard, foreign))
            })?
            .0;
        let before = scene.export_records().records().to_vec();
        let cases = [
            (
                vec![roots[0], roots[0], roots[2]],
                EditReason::ChildSetMismatch,
            ),
            (
                vec![roots[0], roots[1], ObjectId(u64::MAX)],
                EditReason::UnknownObject,
            ),
            (
                vec![roots[0], roots[1], foreign],
                EditReason::ChildSetMismatch,
            ),
            (vec![roots[0], roots[1]], EditReason::ChildSetMismatch),
        ];

        for (order, expected_reason) in cases {
            let error = scene
                .edit(|tx| tx.set_child_order(Parent::Artboard(artboard), &order))
                .expect_err("a malformed exact child set must reject");
            assert_eq!(error.diagnostic().reason, expected_reason);
            assert_eq!(scene.export_records().records(), before);
        }
        assert_ne!(foreign_artboard, artboard);
        Ok(())
    }

    fn rejected_bulk_create_work(node_count: usize) -> Result<SceneWork> {
        let mut scene = Scene::new();
        reset_scene_work();
        let error = scene
            .edit(|tx| {
                let artboard = tx.create_artboard(ArtboardSpec {
                    name: "Rejected after hierarchy construction".into(),
                    width: f32::NAN,
                    height: 100.0,
                })?;
                for _ in 0..node_count {
                    tx.create(Parent::Artboard(artboard), work_test_shape("Shape"))?;
                }
                Ok(())
            })
            .expect_err("the invalid artboard must reject after transaction indexing");
        assert_eq!(
            error.diagnostic().reason,
            EditReason::NonFiniteProperty { property: "width" }
        );
        Ok(scene_work())
    }

    #[test]
    fn bulk_create_builds_one_index_and_checks_each_receipt_identity_once() -> Result<()> {
        for node_count in [2_048, 4_096] {
            assert_eq!(
                rejected_bulk_create_work(node_count)?,
                SceneWork {
                    definition_index_builds: 1,
                    definition_index_node_visits: 0,
                    receipt_membership_checks: node_count,
                }
            );
        }
        Ok(())
    }

    fn rejected_scoped_replace_work(root_count: usize) -> Result<SceneWork> {
        let mut scene = Scene::new();
        let (artboard, _) = scene.edit(|tx| {
            let artboard = tx.create_artboard(ArtboardSpec {
                name: "Main".into(),
                width: 100.0,
                height: 100.0,
            })?;
            for _ in 0..root_count {
                tx.create(Parent::Artboard(artboard), work_test_shape("Old"))?;
            }
            Ok(artboard)
        })?;

        reset_scene_work();
        scene
            .edit(|tx| {
                tx.set_artboard(
                    artboard,
                    ArtboardSpec {
                        name: "Rejected after hierarchy work".into(),
                        width: f32::NAN,
                        height: 100.0,
                    },
                )?;
                tx.clear_artboard(artboard)?;
                for _ in 0..root_count {
                    tx.create(Parent::Artboard(artboard), work_test_shape("New"))?;
                }
                Ok(())
            })
            .expect_err("the invalid artboard must reject after replacement indexing");
        Ok(scene_work())
    }

    fn rejected_subtree_round_trip_work(branch_count: usize) -> Result<SceneWork> {
        let mut scene = Scene::new();
        let ((artboard, root), _) = scene.edit(|tx| {
            let artboard = tx.create_artboard(ArtboardSpec {
                name: "Main".into(),
                width: 100.0,
                height: 100.0,
            })?;
            let root = tx.create(Parent::Artboard(artboard), work_test_shape("Root"))?;
            for _ in 0..branch_count {
                let fill = tx.create(
                    Parent::Object(root),
                    NodeSpec::Fill(FillSpec {
                        name: "Fill".into(),
                    }),
                )?;
                tx.create(
                    Parent::Object(fill),
                    NodeSpec::SolidColor(SolidColorSpec {
                        name: "Color".into(),
                        color: 0xff11_2233,
                    }),
                )?;
            }
            Ok((artboard, root))
        })?;

        reset_scene_work();
        scene
            .edit(|tx| {
                tx.set_artboard(
                    artboard,
                    ArtboardSpec {
                        name: "Rejected after hierarchy work".into(),
                        width: f32::NAN,
                        height: 100.0,
                    },
                )?;
                let removed = tx.remove(root)?;
                tx.restore(removed)?;
                Ok(())
            })
            .expect_err("the invalid artboard must reject after subtree indexing");
        Ok(scene_work())
    }

    #[test]
    fn scoped_replacement_and_subtree_round_trip_have_linear_index_work() -> Result<()> {
        for root_count in [300, 600] {
            assert_eq!(
                rejected_scoped_replace_work(root_count)?,
                SceneWork {
                    definition_index_builds: 2,
                    definition_index_node_visits: root_count,
                    receipt_membership_checks: root_count,
                }
            );
        }

        for branch_count in [300_usize, 600] {
            let subtree_size = branch_count.saturating_mul(2).saturating_add(1);
            assert_eq!(
                rejected_subtree_round_trip_work(branch_count)?,
                SceneWork {
                    definition_index_builds: 3,
                    definition_index_node_visits: subtree_size.saturating_mul(2),
                    receipt_membership_checks: 0,
                }
            );
        }
        Ok(())
    }

    #[cfg(feature = "scripting")]
    fn compile_luau(source: &[u8]) -> Vec<u8> {
        use luaur_compiler::functions::luau_compile::luau_compile;

        luaur_common::set_all_flags(true);
        let mut output_size = 0;
        let output = luau_compile(
            source.as_ptr().cast(),
            source.len(),
            std::ptr::null_mut(),
            &mut output_size,
        );
        assert!(!output.is_null(), "pinned Luau compiler returned null");
        // SAFETY: luau_compile returned a non-null allocation containing
        // output_size initialized bytes. Copying detaches the fixture bytes.
        unsafe { std::slice::from_raw_parts(output.cast(), output_size) }.to_vec()
    }

    #[cfg(feature = "scripting")]
    fn scene_with_failing_protocol(bytes: Vec<u8>) -> Result<(Scene, InstanceId, u32)> {
        let mut scene = Scene::new();
        let artboard = scene
            .edit(|tx| {
                let protocol = tx.create_script_asset(ScriptAssetSpec {
                    name: "FailingProtocol".into(),
                    is_module: false,
                    bytes,
                })?;
                let artboard = tx.create_artboard(ArtboardSpec {
                    name: "Failure".into(),
                    width: 100.0,
                    height: 100.0,
                })?;
                tx.create(
                    Parent::Artboard(artboard),
                    NodeSpec::ScriptedDrawable(ScriptedDrawableSpec {
                        name: "Failure".into(),
                        x: 0.0,
                        y: 0.0,
                        opacity: 1.0,
                        rotation: 0.0,
                        scale_x: 1.0,
                        scale_y: 1.0,
                        script: protocol,
                    }),
                )?;
                Ok(artboard)
            })?
            .0;
        let instance = scene.instantiate(artboard)?;
        let global_id = scene
            .instances
            .iter()
            .filter_map(Option::as_ref)
            .find(|live| live.id == instance)
            .and_then(|live| live.runtime.file().graph().artboards.first())
            .and_then(|graph| {
                graph
                    .components
                    .iter()
                    .find(|component| component.type_name == "ScriptedDrawable")
            })
            .map(|component| component.global_id)
            .context("scripted drawable global id")?;
        Ok((scene, instance, global_id))
    }

    #[test]
    fn authored_machine_export_reimports_with_identical_trigger_event_behavior() -> Result<()> {
        let mut scene = Scene::new();
        scene.edit(|tx| {
            let artboard = tx.create_artboard(ArtboardSpec {
                name: "Canvas".into(),
                width: 100.0,
                height: 100.0,
            })?;
            let shape = tx.create(
                Parent::Artboard(artboard),
                NodeSpec::Shape(ShapeSpec {
                    name: "Fader".into(),
                    x: 0.0,
                    y: 0.0,
                    opacity: 1.0,
                    rotation: 0.0,
                    scale_x: 1.0,
                    scale_y: 1.0,
                }),
            )?;
            let idle = tx.animations().create_linear(
                artboard,
                LinearAnimationSpec {
                    name: "Idle".into(),
                    fps: 60,
                    duration: 1,
                },
            )?;
            tx.animations()
                .set_key(idle, shape, props::WORLD_OPACITY, 0, 0.2)?;
            let active = tx.animations().create_linear(
                artboard,
                LinearAnimationSpec {
                    name: "Active".into(),
                    fps: 60,
                    duration: 1,
                },
            )?;
            tx.animations()
                .set_key(active, shape, props::WORLD_OPACITY, 0, 0.8)?;
            let mut machines = tx.machines();
            let event = machines.create_event(
                artboard,
                EventSpec {
                    name: Some("Reached active".into()),
                },
            )?;
            let machine = machines.create_machine(
                artboard,
                MachineSpec {
                    name: Some("Switcher".into()),
                },
            )?;
            let trigger =
                machines.create_trigger_input(machine, TriggerInputSpec { name: "Go".into() })?;
            let layer = machines.create_layer(
                machine,
                MachineLayerSpec {
                    name: Some("Main".into()),
                },
            )?;
            let entry = machines.create_entry_state(layer)?;
            let any = machines.create_any_state(layer)?;
            machines.create_exit_state(layer)?;
            let idle_state =
                machines.create_animation_state(layer, AnimationStateSpec { animation: idle })?;
            let active_state =
                machines.create_animation_state(layer, AnimationStateSpec { animation: active })?;
            machines.create_transition(entry, idle_state)?;
            let transition = machines.create_transition(any, active_state)?;
            machines.add_trigger_condition(transition, trigger)?;
            machines.add_fire_event(active_state, event, FireEventOccurs::AtStart)?;
            Ok(())
        })?;

        let runtime =
            RuntimeFile::from_authoring_records(scene.export_records().into_authoring_records())?;
        let file = Arc::new(File::from_runtime(runtime)?);
        let mut instance = OwnedArtboardInstance::instantiate(file, 0)?;
        let mut machine = instance
            .state_machine_instance(0)
            .ok_or_else(|| anyhow::anyhow!("state machine"))?;
        let trigger = machine
            .input_index_named("Go")
            .ok_or_else(|| anyhow::anyhow!("Go trigger"))?;

        assert!(instance.advance_with_state_machine(&mut machine, 0.0));
        assert_eq!(
            instance.raw().double_property(1, PROPERTY_WORLD_OPACITY),
            Some(0.2)
        );
        assert_eq!(machine.reported_event_count(), 0);
        assert!(machine.fire_trigger(trigger));
        assert!(instance.advance_with_state_machine(&mut machine, 0.0));
        assert_eq!(
            instance.raw().double_property(1, PROPERTY_WORLD_OPACITY),
            Some(0.8)
        );
        let event = machine
            .reported_event(0)
            .ok_or_else(|| anyhow::anyhow!("reported event"))?;
        assert_eq!(event.event_local_index(), 2);
        assert_eq!(event.name(), Some("Reached active"));
        assert_eq!(event.seconds_delay(), 0.0);
        assert_eq!(machine.reported_event_count(), 1);
        instance.advance_with_state_machine(&mut machine, 0.0);
        assert_eq!(machine.reported_event_count(), 0);
        Ok(())
    }

    #[test]
    fn fire_rechecks_machine_input_kind_and_index_fences() -> Result<()> {
        let mut scene = Scene::new();
        let ((artboard, machine), _) = scene.edit(|tx| {
            let artboard = tx.create_artboard(ArtboardSpec {
                name: "Canvas".into(),
                width: 10.0,
                height: 10.0,
            })?;
            let mut machines = tx.machines();
            let machine = machines.create_machine(
                artboard,
                MachineSpec {
                    name: Some("Machine".into()),
                },
            )?;
            let trigger =
                machines.create_trigger_input(machine, TriggerInputSpec { name: "Go".into() })?;
            let layer = machines.create_layer(
                machine,
                MachineLayerSpec {
                    name: Some("Layer".into()),
                },
            )?;
            machines.create_any_state(layer)?;
            let entry = machines.create_entry_state(layer)?;
            let exit = machines.create_exit_state(layer)?;
            let transition = machines.create_transition(entry, exit)?;
            machines.add_trigger_condition(transition, trigger)?;
            Ok((artboard, machine))
        })?;
        let instance = scene.instantiate(artboard)?;
        let valid = scene.machine_input(instance, machine, "Go")?;

        let mut wrong_kind = valid;
        wrong_kind.input_kind = StateMachineInputKind::Bool;
        assert_eq!(scene.frame().fire(wrong_kind), Err(StaleCursor));

        let mut unknown_input = valid;
        unknown_input.input_index = usize::MAX;
        assert_eq!(scene.frame().fire(unknown_input), Err(StaleCursor));

        let mut unknown_machine = valid;
        unknown_machine.machine = MachineId(ObjectId(u64::MAX));
        assert_eq!(scene.frame().fire(unknown_machine), Err(StaleCursor));
        assert!(scene.frame().fire(valid).is_ok());
        Ok(())
    }

    #[cfg(feature = "scripting")]
    #[test]
    fn failed_script_bootstrap_is_cold_retryable_and_attaches_nothing() -> Result<()> {
        let init_false =
            compile_luau(b"return function(_) return { init = function() return false end } end");
        for (case, bytes) in [("malformed", vec![0xff]), ("init false", init_false)] {
            let (mut scene, instance, global_id) = scene_with_failing_protocol(bytes)?;
            let mut factory = RecordingFactory::new();
            let mut renderer = factory.make_renderer();
            let mut cache = scene.new_render_cache(instance)?;

            let error = scene
                .frame()
                .draw(instance, &mut factory, &mut renderer, &mut cache)
                .expect_err(case);
            assert_eq!(error, DrawError::RuntimeRejected, "{case}");
            assert!(
                cache
                    .inner
                    .as_ref()
                    .is_some_and(|inner| inner.paint.is_none()),
                "{case} must discard provisional factory resources"
            );

            let live = scene
                .instances
                .iter()
                .filter_map(Option::as_ref)
                .find(|live| live.id == instance)
                .context("live instance remains mounted")?;
            assert!(
                !live.runtime.raw().has_script_instance_for_global(global_id),
                "{case} must leave zero table attachments"
            );
            assert!(
                live.runtime.file().scripts.borrow().ready.is_none(),
                "{case} must leave the File VM cold and retryable"
            );
        }
        Ok(())
    }

    #[cfg(feature = "scripting")]
    #[test]
    fn script_init_accepts_any_lua_truthy_result() -> Result<()> {
        let truthy_table =
            compile_luau(b"return function(_) return { init = function() return {} end } end");
        let (mut scene, instance, global_id) = scene_with_failing_protocol(truthy_table)?;
        let mut factory = RecordingFactory::new();
        let mut renderer = factory.make_renderer();
        let mut cache = scene.new_render_cache(instance)?;

        scene
            .frame()
            .draw(instance, &mut factory, &mut renderer, &mut cache)?;

        let live = scene
            .instances
            .iter()
            .filter_map(Option::as_ref)
            .find(|live| live.id == instance)
            .context("live instance remains mounted")?;
        assert!(
            live.runtime.raw().has_script_instance_for_global(global_id),
            "a Lua table is truthy and must complete bootstrap"
        );
        Ok(())
    }
}
