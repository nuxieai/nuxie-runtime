//! Dynamic authoring facade backed by the same runtime file and graph used by imported scenes.

use std::{
    collections::BTreeMap,
    marker::PhantomData,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

use nuxie_binary::{AuthoringProperty, AuthoringRecord, AuthoringValue, RuntimeFile};
use nuxie_render_api::{Factory, Renderer};
use nuxie_runtime::ArtboardInstance as RuntimeArtboardInstance;

use crate::{ArtboardRenderCache, File, OwnedArtboardInstance};

/// Stable identity of an authored artboard.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ArtboardId(u64);

/// Stable identity of an authored object.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ObjectId(u64);

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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Parent {
    Artboard(ArtboardId),
    Object(ObjectId),
}

/// Stable identities involved in a failed edit operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EditId {
    Artboard(ArtboardId),
    Object(ObjectId),
    Instance(InstanceId),
}

/// Machine-readable reason for rejecting an edit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditReason {
    Requested {
        message: String,
    },
    IdentityExhausted,
    OperationLimitExceeded,
    UnknownArtboard,
    UnknownObject,
    InvalidParent {
        parent: Option<NodeKind>,
        child: NodeKind,
    },
    PropertyOwnerMismatch {
        property: &'static str,
        actual: NodeKind,
    },
    NonFiniteProperty {
        property: &'static str,
    },
    EmptyScene,
    CapacityExceeded,
    RuntimeRejected,
    EpochExhausted,
    InternalInvariant,
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
    apply_to_definition: fn(&mut NodeSpec, T) -> std::result::Result<(), EditReason>,
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
    DifferentArtboard,
    UnsupportedProperty,
}

impl std::fmt::Display for ResolveError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::UnknownInstance => "unknown scene instance",
            Self::UnknownObject => "unknown authored object",
            Self::DifferentArtboard => "authored object belongs to a different artboard",
            Self::UnsupportedProperty => "property is not valid for the authored object type",
        })
    }
}

impl std::error::Error for ResolveError {}

/// Failure while drawing an authored instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DrawError {
    UnknownInstance,
    RuntimeRejected,
}

impl std::fmt::Display for DrawError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::UnknownInstance => "unknown scene instance",
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
    artboards: Vec<ArtboardDefinition>,
}

#[derive(Default)]
struct EditOrigins {
    artboard_specs: BTreeMap<ArtboardId, usize>,
    nodes: BTreeMap<ObjectId, usize>,
}

impl EditOrigins {
    fn artboard(&self, id: ArtboardId, fallback: usize) -> usize {
        self.artboard_specs.get(&id).copied().unwrap_or(fallback)
    }

    fn object(&self, id: ObjectId, fallback: usize) -> usize {
        self.nodes.get(&id).copied().unwrap_or(fallback)
    }
}

#[derive(Debug, Clone)]
struct ArtboardDefinition {
    id: ArtboardId,
    spec: ArtboardSpec,
    nodes: Vec<NodeDefinition>,
}

#[derive(Debug, Clone)]
struct NodeDefinition {
    id: ObjectId,
    parent: Parent,
    spec: NodeSpec,
}

struct RuntimeSlot {
    local_id: usize,
    kind: NodeKind,
}

struct MaterializedArtboard {
    file: Arc<File>,
    objects: BTreeMap<ObjectId, RuntimeSlot>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MountId(u64);

struct LiveInstance {
    id: InstanceId,
    artboard: ArtboardId,
    mount: MountId,
    runtime: OwnedArtboardInstance,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SceneId(u64);

struct SceneIdentity {
    id: SceneId,
}

static NEXT_SCENE_ID: AtomicU64 = AtomicU64::new(0);
static NEXT_ARTBOARD_ID: AtomicU64 = AtomicU64::new(0);
static NEXT_OBJECT_ID: AtomicU64 = AtomicU64::new(0);
static NEXT_INSTANCE_ID: AtomicU64 = AtomicU64::new(0);

/// Render resources retained for one mount of one live authored instance.
///
/// The wrapper detects a remount of its artboard (and accidental reuse with another scene or
/// instance) and recreates its underlying runtime cache before the next draw. Structural edits to
/// another artboard do not invalidate this cache.
pub struct SceneRenderCache {
    scene_identity: Arc<SceneIdentity>,
    instance: InstanceId,
    mount: MountId,
    inner: ArtboardRenderCache,
}

/// Schema-backed object kinds in the deterministic publish record stream.
///
/// The typed vocabulary is intentionally key-free: raw runtime type keys stay
/// behind the `Scene` implementation and the publish compiler maps these
/// variants through its own schema-generated vocabulary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportedObjectKind {
    Backboard,
    Artboard,
    Shape,
    Rectangle,
    Fill,
    SolidColor,
    Stroke,
    DashPath,
    Dash,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportedFillRule {
    NonZero,
}

/// Typed properties in a deterministic exported scene record.
#[derive(Debug, Clone, PartialEq)]
pub enum ExportedProperty {
    ComponentName(String),
    ParentId(u32),
    LayoutWidth(f32),
    LayoutHeight(f32),
    TranslateX(f32),
    TranslateY(f32),
    WorldOpacity(f32),
    Rotation(f32),
    ScaleX(f32),
    ScaleY(f32),
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
}

impl ExportedProperty {
    fn schema_key(&self) -> u16 {
        match self {
            Self::ComponentName(_) => PROPERTY_COMPONENT_NAME,
            Self::ParentId(_) => PROPERTY_PARENT_ID,
            Self::LayoutWidth(_) => PROPERTY_LAYOUT_WIDTH,
            Self::LayoutHeight(_) => PROPERTY_LAYOUT_HEIGHT,
            Self::TranslateX(_) => PROPERTY_TRANSLATE_X,
            Self::TranslateY(_) => PROPERTY_TRANSLATE_Y,
            Self::WorldOpacity(_) => PROPERTY_WORLD_OPACITY,
            Self::Rotation(_) => PROPERTY_ROTATION,
            Self::ScaleX(_) => PROPERTY_SCALE_X,
            Self::ScaleY(_) => PROPERTY_SCALE_Y,
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
        }
    }

    fn into_authoring_property(self) -> AuthoringProperty {
        let key = self.schema_key();
        let value = match self {
            Self::ComponentName(value) => AuthoringValue::String(value),
            Self::ParentId(value) => AuthoringValue::Uint(u64::from(value)),
            Self::FillRule(ExportedFillRule::NonZero) => AuthoringValue::Uint(0),
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
            | Self::PathWidth(value)
            | Self::PathHeight(value)
            | Self::RectangleCornerRadiusTopLeft(value)
            | Self::RectangleCornerRadiusTopRight(value)
            | Self::RectangleCornerRadiusBottomRight(value)
            | Self::RectangleCornerRadiusBottomLeft(value)
            | Self::StrokeThickness(value)
            | Self::DashOffset(value)
            | Self::DashLength(value) => AuthoringValue::Double(value),
            Self::RectangleLinkCornerRadius(value)
            | Self::StrokeTransformAffectsStroke(value)
            | Self::DashOffsetIsPercentage(value)
            | Self::DashLengthIsPercentage(value) => AuthoringValue::Bool(value),
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
            ExportedObjectKind::Artboard => TYPE_ARTBOARD,
            ExportedObjectKind::Shape => TYPE_SHAPE,
            ExportedObjectKind::Rectangle => TYPE_RECTANGLE,
            ExportedObjectKind::Fill => TYPE_FILL,
            ExportedObjectKind::SolidColor => TYPE_SOLID_COLOR,
            ExportedObjectKind::Stroke => TYPE_STROKE,
            ExportedObjectKind::DashPath => TYPE_DASH_PATH,
            ExportedObjectKind::Dash => TYPE_DASH,
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
        let mut definitions = self.definitions.clone();
        let (result, created_objects, touched_artboards, edit_origins, commit_operation_index) = {
            let mut transaction = SceneTx {
                definitions: &mut definitions,
                next_operation_index: 0,
                created_objects: Vec::new(),
                touched_artboards: BTreeMap::new(),
                edit_origins: EditOrigins::default(),
            };
            let result = edit(&mut transaction).map_err(EditError::aborted)?;
            (
                result,
                transaction.created_objects,
                transaction.touched_artboards,
                transaction.edit_origins,
                transaction.next_operation_index,
            )
        };

        if definitions.artboards.is_empty() {
            return Err(EditError::commit(EditDiagnostic::new(
                commit_operation_index,
                Vec::new(),
                EditReason::EmptyScene,
            )));
        }

        // Prepare every touched artboard before publishing any of them. A later failure therefore
        // cannot partially replace definitions, files, instances, mounts, or render caches.
        let mut candidates = BTreeMap::new();
        for artboard in definitions
            .artboards
            .iter()
            .filter(|artboard| touched_artboards.contains_key(&artboard.id))
        {
            let Some(touched_operation_index) = touched_artboards.get(&artboard.id).copied() else {
                return Err(EditError::commit(EditDiagnostic::new(
                    commit_operation_index,
                    vec![EditId::Artboard(artboard.id)],
                    EditReason::InternalInvariant,
                )));
            };
            let materialized = MaterializedArtboard::build(
                artboard,
                commit_operation_index,
                &edit_origins,
                touched_operation_index,
            )
            .map_err(EditError::commit)?;
            candidates.insert(artboard.id, materialized);
        }
        if candidates.len() != touched_artboards.len() {
            return Err(EditError::commit(EditDiagnostic::new(
                commit_operation_index,
                touched_artboards
                    .keys()
                    .copied()
                    .map(EditId::Artboard)
                    .collect(),
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
            let Some(touched_operation_index) = touched_artboards.get(&instance.artboard).copied()
            else {
                continue;
            };
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
            let runtime = OwnedArtboardInstance::instantiate(Arc::clone(&materialized.file), 0)
                .map_err(|_| {
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
                },
            ));
        }

        // No operation below this point is fallible: publish the prepared transaction atomically.
        let mut replacements: BTreeMap<_, _> = remounted.into_iter().collect();
        let instances = std::mem::take(&mut self.instances)
            .into_iter()
            .enumerate()
            .map(|(instance_slot, instance)| {
                replacements
                    .remove(&instance_slot)
                    .map(Some)
                    .unwrap_or(instance)
            })
            .collect();
        debug_assert!(replacements.is_empty());
        self.definitions = definitions;
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
        let runtime = OwnedArtboardInstance::instantiate(Arc::clone(&materialized.file), 0)
            .map_err(|_| InstanceError::RuntimeRejected)?;
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
        let (slot_artboard, slot) = self
            .materialized
            .iter()
            .find_map(|(artboard, materialized)| {
                materialized
                    .objects
                    .get(&object)
                    .map(|slot| (*artboard, slot))
            })
            .ok_or(ResolveError::UnknownObject)?;
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

    pub fn new_render_cache(
        &self,
        instance: InstanceId,
        factory: &mut dyn Factory,
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
            inner: live.runtime.new_render_cache(factory),
        })
    }

    pub fn frame(&mut self) -> Frame<'_> {
        Frame { scene: self }
    }

    /// Export one canonical record stream with one Backboard and every authored artboard.
    ///
    /// Export reads authored definitions, not ephemeral instance values written through
    /// [`Frame::set`]. Clients replay those values after a structural remount when needed.
    pub fn export_records(&self) -> ExportedDocument {
        let mut records = vec![backboard_record()];
        let origins = EditOrigins::default();
        for artboard in &self.definitions.artboards {
            let lowered = match lower_artboard(artboard, 0, &origins) {
                Ok(lowered) => lowered,
                Err(_) => {
                    // Committed definitions have already passed this exact lowering path.
                    // Export must never return a partial stream if that invariant is broken.
                    std::process::abort();
                }
            };
            records.extend(lowered.records.into_iter().skip(1));
        }
        ExportedDocument { records }
    }
}

/// Mutable structural transaction over a scene's durable definitions.
pub struct SceneTx<'a> {
    definitions: &'a mut Definitions,
    next_operation_index: usize,
    created_objects: Vec<ObjectId>,
    touched_artboards: BTreeMap<ArtboardId, usize>,
    edit_origins: EditOrigins,
}

impl SceneTx<'_> {
    pub fn create_artboard(
        &mut self,
        spec: ArtboardSpec,
    ) -> std::result::Result<ArtboardId, EditAbort> {
        let operation_index = self.begin_operation()?;
        let id = ArtboardId(allocate_global_identity(&NEXT_ARTBOARD_ID).ok_or_else(|| {
            EditAbort::new(operation_index, Vec::new(), EditReason::IdentityExhausted)
        })?);
        self.definitions.artboards.push(ArtboardDefinition {
            id,
            spec,
            nodes: Vec::new(),
        });
        self.touched_artboards.insert(id, operation_index);
        self.edit_origins.artboard_specs.insert(id, operation_index);
        Ok(id)
    }

    pub fn create(
        &mut self,
        parent: Parent,
        spec: NodeSpec,
    ) -> std::result::Result<ObjectId, EditAbort> {
        let operation_index = self.begin_operation()?;
        let artboard_id = self.validate_parent(operation_index, parent, spec.kind())?;
        let id = ObjectId(allocate_global_identity(&NEXT_OBJECT_ID).ok_or_else(|| {
            EditAbort::new(
                operation_index,
                parent_edit_ids(parent),
                EditReason::IdentityExhausted,
            )
        })?);
        let Some(artboard) = self
            .definitions
            .artboards
            .iter_mut()
            .find(|candidate| candidate.id == artboard_id)
        else {
            return Err(EditAbort::new(
                operation_index,
                vec![EditId::Artboard(artboard_id)],
                EditReason::InternalInvariant,
            ));
        };
        artboard.nodes.push(NodeDefinition { id, parent, spec });
        self.created_objects.push(id);
        self.touched_artboards.insert(artboard_id, operation_index);
        self.edit_origins.nodes.insert(id, operation_index);
        Ok(id)
    }

    pub fn set<T>(
        &mut self,
        object: ObjectId,
        property: Prop<T>,
        value: T,
    ) -> std::result::Result<(), EditAbort> {
        let operation_index = self.begin_operation()?;
        let Some((artboard_id, node)) =
            self.definitions.artboards.iter_mut().find_map(|artboard| {
                artboard
                    .nodes
                    .iter_mut()
                    .find(|candidate| candidate.id == object)
                    .map(|node| (artboard.id, node))
            })
        else {
            return Err(EditAbort::new(
                operation_index,
                vec![EditId::Object(object)],
                EditReason::UnknownObject,
            ));
        };
        let actual = node.spec.kind();
        if !property.is_available_on(actual) {
            return Err(EditAbort::new(
                operation_index,
                vec![EditId::Object(object)],
                EditReason::PropertyOwnerMismatch {
                    property: property.schema_name,
                    actual,
                },
            ));
        }
        (property.apply_to_definition)(&mut node.spec, value).map_err(|reason| {
            EditAbort::new(operation_index, vec![EditId::Object(object)], reason)
        })?;
        self.touched_artboards.insert(artboard_id, operation_index);
        self.edit_origins.nodes.insert(object, operation_index);
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

    fn validate_parent(
        &self,
        operation_index: usize,
        parent: Parent,
        child: NodeKind,
    ) -> std::result::Result<ArtboardId, EditAbort> {
        match parent {
            Parent::Artboard(artboard) => {
                if child != NodeKind::Shape {
                    return Err(EditAbort::new(
                        operation_index,
                        vec![EditId::Artboard(artboard)],
                        EditReason::InvalidParent {
                            parent: None,
                            child,
                        },
                    ));
                }
                if self
                    .definitions
                    .artboards
                    .iter()
                    .any(|candidate| candidate.id == artboard)
                {
                    Ok(artboard)
                } else {
                    Err(EditAbort::new(
                        operation_index,
                        vec![EditId::Artboard(artboard)],
                        EditReason::UnknownArtboard,
                    ))
                }
            }
            Parent::Object(object) => {
                let Some((artboard, parent_kind)) =
                    self.definitions.artboards.iter().find_map(|artboard| {
                        artboard
                            .nodes
                            .iter()
                            .find(|candidate| candidate.id == object)
                            .map(|node| (artboard.id, node.spec.kind()))
                    })
                else {
                    return Err(EditAbort::new(
                        operation_index,
                        vec![EditId::Object(object)],
                        EditReason::UnknownObject,
                    ));
                };
                let valid = matches!(
                    (parent_kind, child),
                    (
                        NodeKind::Shape,
                        NodeKind::Rectangle | NodeKind::Fill | NodeKind::Stroke
                    ) | (NodeKind::Fill, NodeKind::SolidColor)
                        | (NodeKind::Stroke, NodeKind::SolidColor | NodeKind::DashPath)
                        | (NodeKind::DashPath, NodeKind::Dash)
                );
                if !valid {
                    return Err(EditAbort::new(
                        operation_index,
                        vec![EditId::Object(object)],
                        EditReason::InvalidParent {
                            parent: Some(parent_kind),
                            child,
                        },
                    ));
                }
                Ok(artboard)
            }
        }
    }
}

/// A short-lived facade over direct runtime instance writes and draws.
pub struct Frame<'a> {
    scene: &'a mut Scene,
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
            || cache.mount != live.mount;
        if needs_refresh {
            cache.inner = live.runtime.new_render_cache(factory);
            cache.scene_identity = Arc::clone(&self.scene.identity);
            cache.instance = instance;
            cache.mount = live.mount;
        }
        live.runtime
            .draw_with_render_cache(factory, renderer, &mut cache.inner)
            .map_err(|_| DrawError::RuntimeRejected)
    }
}

impl MaterializedArtboard {
    fn build(
        definition: &ArtboardDefinition,
        fallback_operation_index: usize,
        origins: &EditOrigins,
        touched_operation_index: usize,
    ) -> std::result::Result<Self, EditDiagnostic> {
        let lowered = lower_artboard(definition, fallback_operation_index, origins)?;
        let authoring_records = ExportedDocument {
            records: lowered.records,
        }
        .into_authoring_records();
        let runtime = RuntimeFile::from_authoring_records(authoring_records).map_err(|_| {
            EditDiagnostic::new(
                touched_operation_index,
                vec![EditId::Artboard(definition.id)],
                EditReason::InternalInvariant,
            )
        })?;
        let file = Arc::new(File::from_runtime(runtime).map_err(|_| {
            EditDiagnostic::new(
                touched_operation_index,
                vec![EditId::Artboard(definition.id)],
                EditReason::InternalInvariant,
            )
        })?);
        Ok(Self {
            file,
            objects: lowered.objects,
        })
    }
}

struct LoweredArtboard {
    records: Vec<ExportedRecord>,
    objects: BTreeMap<ObjectId, RuntimeSlot>,
}

/// Lower exactly one durable artboard into one runtime-file record stream.
///
/// Preview materialization uses this function today; deterministic export can reuse the same
/// lowering without reconstructing the whole live scene. Parent resolution is deliberately local
/// to the artboard, which hard-gates the current vocabulary against cross-artboard references.
fn lower_artboard(
    artboard: &ArtboardDefinition,
    fallback_operation_index: usize,
    origins: &EditOrigins,
) -> std::result::Result<LoweredArtboard, EditDiagnostic> {
    validate_artboard_spec(&artboard.spec).map_err(|reason| {
        EditDiagnostic::new(
            origins.artboard(artboard.id, fallback_operation_index),
            vec![EditId::Artboard(artboard.id)],
            reason,
        )
    })?;

    let mut records = vec![backboard_record(), artboard_record(&artboard.spec)];
    let mut local_ids = BTreeMap::new();
    for (node_index, node) in artboard.nodes.iter().enumerate() {
        let local_id = node_index.checked_add(1).ok_or_else(|| {
            EditDiagnostic::new(
                origins.object(node.id, fallback_operation_index),
                vec![EditId::Object(node.id)],
                EditReason::CapacityExceeded,
            )
        })?;
        local_ids.insert(node.id, local_id);
    }

    let mut objects = BTreeMap::new();
    for node in &artboard.nodes {
        validate_node_spec(&node.spec).map_err(|reason| {
            EditDiagnostic::new(
                origins.object(node.id, fallback_operation_index),
                vec![EditId::Object(node.id)],
                reason,
            )
        })?;
        let local_id = local_ids.get(&node.id).copied().ok_or_else(|| {
            EditDiagnostic::new(
                origins.object(node.id, fallback_operation_index),
                vec![EditId::Object(node.id)],
                EditReason::InternalInvariant,
            )
        })?;
        let parent_id = match node.parent {
            Parent::Artboard(parent) if parent == artboard.id => 0,
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
            Parent::Object(parent) => local_ids.get(&parent).copied().ok_or_else(|| {
                EditDiagnostic::new(
                    origins.object(node.id, fallback_operation_index),
                    vec![EditId::Object(node.id), EditId::Object(parent)],
                    EditReason::InvalidParent {
                        parent: None,
                        child: node.spec.kind(),
                    },
                )
            })?,
        };
        records.push(node_record(node, parent_id).map_err(|reason| {
            EditDiagnostic::new(
                origins.object(node.id, fallback_operation_index),
                vec![EditId::Object(node.id)],
                reason,
            )
        })?);
        objects.insert(
            node.id,
            RuntimeSlot {
                local_id,
                kind: node.spec.kind(),
            },
        );
    }

    canonicalize_exported_records(&mut records);
    Ok(LoweredArtboard { records, objects })
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
        NodeSpec::Fill(_) | NodeSpec::SolidColor(_) => {}
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
    node: &NodeDefinition,
    parent_id: usize,
) -> std::result::Result<ExportedRecord, EditReason> {
    let parent_id = u32::try_from(parent_id).map_err(|_| EditReason::CapacityExceeded)?;
    let mut properties = Vec::new();
    if parent_id != 0 {
        properties.push(ExportedProperty::ParentId(parent_id));
    }
    let kind = match &node.spec {
        NodeSpec::Shape(spec) => {
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
            ExportedObjectKind::Shape
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

    use anyhow::Result;
    use nuxie_render_stream::RenderStream;

    use super::*;
    use crate::RecordingFactory;

    fn parse_single_frame(stream: &str) -> Result<RenderStream> {
        Ok(RenderStream::parse(&format!("{stream}frame\n"))?)
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
                let mut cache = scene.new_render_cache(instance, &mut factory)?;
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
            let mut cache = instance.new_render_cache(&mut factory);
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
}
