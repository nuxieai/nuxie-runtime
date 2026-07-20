use std::cell::{RefCell, RefMut};
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;
use std::{error::Error, fmt};

use nuxie_binary::{RuntimeFile, RuntimeObject};
use nuxie_graph::{ArtboardGraph, ShapePaintKind, ShapePaintNode, ShapePaintStateNode};
use nuxie_render_api::{
    BlendMode, Factory as RenderFactory, RawPath, RenderPaintStyle, Renderer, StrokeCap, StrokeJoin,
};

use crate::data_bind_graph::{
    RuntimeDataBindGraphConverter, RuntimeDataBindGraphValue,
    runtime_data_bind_graph_convert_value, runtime_data_bind_graph_converter,
};
use crate::properties::property_key_for_name;
use crate::{
    ArtboardInstance, LinearAnimationInstance, RuntimeOwnedViewModelContextHandle,
    RuntimeOwnedViewModelHandle, RuntimeOwnedViewModelInstance,
};

/// Runtime-owned scripting error type.
///
/// The concrete VM crate maps its native error into this type so
/// `nuxie-runtime` can keep the scripting seam free of VM dependencies.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptError {
    message: String,
    resource_code: Option<String>,
}

impl ScriptError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            resource_code: None,
        }
    }

    /// Construct a terminal script-resource failure with its stable identity.
    ///
    /// The concrete scripting backend owns the resource taxonomy. The runtime
    /// carries only its stable code so higher layers can classify the failure
    /// without depending on a VM crate or matching human-readable text.
    pub fn with_resource_code(
        message: impl Into<String>,
        resource_code: impl Into<String>,
    ) -> Self {
        Self {
            message: message.into(),
            resource_code: Some(resource_code.into()),
        }
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn resource_code(&self) -> Option<&str> {
        self.resource_code.as_deref()
    }

    /// Add human-readable execution context without erasing typed provenance.
    pub fn with_context(mut self, context: impl fmt::Display) -> Self {
        self.message = format!("{context}: {}", self.message);
        self
    }
}

impl fmt::Display for ScriptError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl Error for ScriptError {}

/// A script module/script asset payload as stored in a `.riv` file.
#[derive(Debug, Clone, Copy)]
pub struct ScriptModule<'a> {
    pub name: &'a str,
    pub payload: &'a [u8],
}

impl<'a> ScriptModule<'a> {
    pub fn new(name: &'a str, payload: &'a [u8]) -> Self {
        Self { name, payload }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptModuleFailure {
    pub name: String,
    pub error: ScriptError,
}

/// One imported scripted listener action and the protocol asset it resolves.
///
/// `asset_ordinal` is the dense file-asset ordinal serialized in
/// `ScriptedListenerAction.scriptAssetId`; it is deliberately not the
/// semantic `FileAsset.assetId` or the asset object's global id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptListenerActionDefinition {
    action_global_id: u32,
    asset_ordinal: usize,
    asset_name: String,
    inputs: Vec<ScriptListenerInputDefinition>,
}

impl ScriptListenerActionDefinition {
    #[cfg(test)]
    pub(crate) fn new(action_global_id: u32, asset_ordinal: usize, asset_name: String) -> Self {
        Self {
            action_global_id,
            asset_ordinal,
            asset_name,
            inputs: Vec::new(),
        }
    }

    pub(crate) fn with_inputs(
        action_global_id: u32,
        asset_ordinal: usize,
        asset_name: String,
        inputs: Vec<ScriptListenerInputDefinition>,
    ) -> Self {
        Self {
            action_global_id,
            asset_ordinal,
            asset_name,
            inputs,
        }
    }

    pub fn action_global_id(&self) -> u32 {
        self.action_global_id
    }

    pub fn asset_ordinal(&self) -> usize {
        self.asset_ordinal
    }

    pub fn asset_name(&self) -> &str {
        &self.asset_name
    }

    /// Authored inputs owned by this exact listener-action occurrence.
    ///
    /// The global object id remains stable for the lifetime of the imported
    /// file and lets the facade resolve the complete binary object (including
    /// its data-bind metadata) only when a concrete occurrence is hydrated.
    pub fn inputs(&self) -> &[ScriptListenerInputDefinition] {
        &self.inputs
    }
}

/// One authored input belonging to a scripted listener-action occurrence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScriptListenerInputDefinition {
    input_global_id: u32,
    kind: ScriptListenerInputKind,
}

impl ScriptListenerInputDefinition {
    pub(crate) fn new(input_global_id: u32, kind: ScriptListenerInputKind) -> Self {
        Self {
            input_global_id,
            kind,
        }
    }

    pub fn input_global_id(self) -> u32 {
        self.input_global_id
    }

    pub fn kind(self) -> ScriptListenerInputKind {
        self.kind
    }
}

/// The seven input kinds accepted by Rive scripted objects.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptListenerInputKind {
    Boolean,
    Number,
    Color,
    String,
    Trigger,
    Artboard,
    ViewModelProperty,
}

/// A fully resolved, occurrence-local listener hydration batch.
///
/// Callers construct the whole batch before touching a script table. Applying
/// it always installs `Context.viewModel` first, then the authored/bound input
/// values in source order. Trigger entries represent a bound trigger edge;
/// ordinary initial trigger hydration intentionally produces no entry because
/// the table field is the authored callback itself.
pub struct ScriptListenerActionHydration {
    context_view_model: Option<ScriptViewModel>,
    inputs: Vec<ScriptListenerInputHydration>,
}

impl ScriptListenerActionHydration {
    pub fn new(
        context_view_model: Option<ScriptViewModel>,
        inputs: Vec<ScriptListenerInputHydration>,
    ) -> Self {
        Self {
            context_view_model,
            inputs,
        }
    }

    pub fn apply(
        self,
        instance: &mut dyn ScriptInstance,
        host: &mut dyn ScriptHost,
    ) -> Result<(), ScriptError> {
        instance.set_context_view_model(self.context_view_model)?;
        for input in self.inputs {
            match input {
                ScriptListenerInputHydration::Value { name, value } => {
                    instance.set_input(&name, value)?;
                }
                ScriptListenerInputHydration::Artboard { name, artboard } => {
                    instance.set_artboard_input(&name, artboard)?;
                }
                ScriptListenerInputHydration::ViewModel { name, view_model } => {
                    instance.set_view_model_input(&name, view_model)?;
                }
                ScriptListenerInputHydration::Trigger { name } => {
                    instance.call_input_trigger(&name, host)?;
                }
            }
        }
        Ok(())
    }
}

/// One resolved listener input operation.
pub enum ScriptListenerInputHydration {
    Value {
        name: String,
        value: ScriptValue,
    },
    Artboard {
        name: String,
        artboard: Box<dyn ScriptArtboard>,
    },
    ViewModel {
        name: String,
        view_model: ScriptViewModel,
    },
    Trigger {
        name: String,
    },
}

/// Lifecycle/input methods carried by scripted object instance tables.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScriptMethod {
    Init,
    Resize,
    Advance,
    Update,
    Draw,
    Evaluate,
    PointerDown,
    PointerMove,
    PointerUp,
    PointerEnter,
    PointerExit,
    PerformAction,
    Perform,
}

impl ScriptMethod {
    pub fn as_str(self) -> &'static str {
        match self {
            ScriptMethod::Init => "init",
            ScriptMethod::Resize => "resize",
            ScriptMethod::Advance => "advance",
            ScriptMethod::Update => "update",
            ScriptMethod::Draw => "draw",
            ScriptMethod::Evaluate => "evaluate",
            ScriptMethod::PointerDown => "pointerDown",
            ScriptMethod::PointerMove => "pointerMove",
            ScriptMethod::PointerUp => "pointerUp",
            ScriptMethod::PointerEnter => "pointerEnter",
            ScriptMethod::PointerExit => "pointerExit",
            ScriptMethod::PerformAction => "performAction",
            ScriptMethod::Perform => "perform",
        }
    }
}

/// The method selected for one scripted listener dispatch.
///
/// Runtime dispatch probes `performAction` first and only selects the legacy
/// `perform` callback when the newer method is absent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptListenerActionMethod {
    PerformAction,
    Perform,
}

impl ScriptListenerActionMethod {
    pub fn as_script_method(self) -> ScriptMethod {
        match self {
            Self::PerformAction => ScriptMethod::PerformAction,
            Self::Perform => ScriptMethod::Perform,
        }
    }
}

/// The state-machine invocation supplied to a scripted listener action.
///
/// Scheduled state/transition actions use [`Self::None`]. Pointer listeners
/// retain the concrete pointer payload so scripting backends can expose the
/// same legacy `PointerEvent` shape as the C++ runtime.
#[derive(Debug, Clone, PartialEq)]
pub enum ScriptListenerInvocation {
    Pointer {
        pointer_id: i32,
        x: f32,
        y: f32,
        previous_x: f32,
        previous_y: f32,
        event: ScriptPointerEventKind,
        timestamp_seconds: f32,
    },
    ReportedEvent {
        event_local_index: usize,
        seconds_delay: f32,
    },
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptPointerEventKind {
    Enter,
    Exit,
    Down,
    Up,
    Move,
    Click,
    DragStart,
    DragEnd,
    Drag,
}

impl ScriptPointerEventKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Enter => "pointerEnter",
            Self::Exit => "pointerExit",
            Self::Down => "pointerDown",
            Self::Up => "pointerUp",
            Self::Move => "pointerMove",
            Self::Click => "click",
            Self::DragStart => "pointerDragStart",
            Self::DragEnd => "pointerDragEnd",
            Self::Drag => "pointerDrag",
        }
    }
}

/// VM-neutral values crossing the scripting seam.
#[derive(Debug, Clone, PartialEq)]
pub enum ScriptValue {
    Nil,
    Bool(bool),
    Number(f64),
    String(String),
    Color(u32),
    Vec2 { x: f32, y: f32 },
    Vec3 { x: f32, y: f32, z: f32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptDataConverterMethod {
    Convert,
    ReverseConvert,
}

impl ScriptDataConverterMethod {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Convert => "convert",
            Self::ReverseConvert => "reverseConvert",
        }
    }
}

/// Runtime-owned node data exposed by C++ `ScriptedArtboard::node`.
#[derive(Debug, Clone)]
pub struct ScriptNode {
    pub path: Option<RawPath>,
    pub paint: Option<ScriptPaint>,
}

#[derive(Debug, Clone, Copy)]
pub struct ScriptPaint {
    pub style: RenderPaintStyle,
    pub color: u32,
    pub thickness: f32,
    pub join: StrokeJoin,
    pub cap: StrokeCap,
    pub feather: f32,
    pub blend_mode: BlendMode,
}

/// Ports the lookup/snapshot portion of C++ `src/lua/lua_artboards.cpp`'s
/// `ScriptedNode`, leaving userdata construction to the scripting backend.
pub fn script_node_for_artboard(
    instance: &ArtboardInstance,
    graph: &ArtboardGraph,
    name: &str,
) -> Option<ScriptNode> {
    let component = graph.component_named(name)?;
    let path = graph
        .paths
        .iter()
        .find(|path| path.local_id == component.local_id)
        // C++ exposes the retained `Path::rawPath()` at lookup time. Before
        // the child artboard's first update that path is intentionally empty.
        .map(|_| RawPath::new());
    let paint = graph
        .shape_paint_containers
        .iter()
        .flat_map(|container| &container.paints)
        .find(|paint| paint.local_id == component.local_id)
        .map(|paint| script_paint_for_shape(instance, paint));
    Some(ScriptNode { path, paint })
}

pub(crate) fn script_paint_for_shape(
    instance: &ArtboardInstance,
    paint: &ShapePaintNode,
) -> ScriptPaint {
    let object = instance
        .runtime_file()
        .and_then(|file| file.object(paint.global_id as usize));
    let authored_color = match paint.paint_state {
        Some(ShapePaintStateNode::SolidColor { color }) => color,
        _ => 0xff000000,
    };
    let color = paint
        .mutator_local
        .zip(property_key_for_name("SolidColor", "colorValue"))
        .and_then(|(local_id, key)| instance.color_property(local_id, key))
        .unwrap_or(authored_color);
    ScriptPaint {
        style: match paint.paint_type {
            ShapePaintKind::Stroke => RenderPaintStyle::Stroke,
            _ => RenderPaintStyle::Fill,
        },
        color,
        thickness: object
            .and_then(|object| object.double_property("thickness"))
            .unwrap_or(1.0),
        join: match object.and_then(|object| object.uint_property("join")) {
            Some(1) => StrokeJoin::Round,
            Some(2) => StrokeJoin::Bevel,
            _ => StrokeJoin::Miter,
        },
        cap: match object.and_then(|object| object.uint_property("cap")) {
            Some(1) => StrokeCap::Round,
            Some(2) => StrokeCap::Square,
            _ => StrokeCap::Butt,
        },
        feather: paint
            .feather
            .as_ref()
            .map(|feather| feather.strength)
            .unwrap_or(0.0),
        blend_mode: script_blend_mode(paint.blend_mode_value),
    }
}

fn script_blend_mode(value: u32) -> BlendMode {
    match value {
        14 => BlendMode::Screen,
        15 => BlendMode::Overlay,
        16 => BlendMode::Darken,
        17 => BlendMode::Lighten,
        18 => BlendMode::ColorDodge,
        19 => BlendMode::ColorBurn,
        20 => BlendMode::HardLight,
        21 => BlendMode::SoftLight,
        22 => BlendMode::Difference,
        23 => BlendMode::Exclusion,
        24 => BlendMode::Multiply,
        25 => BlendMode::Hue,
        26 => BlendMode::Saturation,
        27 => BlendMode::Color,
        28 => BlendMode::Luminosity,
        _ => BlendMode::SrcOver,
    }
}

/// A shared runtime-owned view-model instance exposed to scripting backends.
#[derive(Debug, Clone)]
pub struct ScriptViewModel {
    properties: BTreeMap<String, ScriptViewModelProperty>,
    nested_view_models: BTreeMap<String, ScriptViewModel>,
    context: RuntimeOwnedViewModelContextHandle,
    file: Rc<RuntimeFile>,
    view_model_index: usize,
    ancestors: Rc<Vec<usize>>,
}

/// An image selected from the runtime file's dense asset registry.
///
/// C++ exposes a retained `RenderImage` through Lua. The runtime-neutral seam
/// retains its registry identity instead; assigning the handle to an image
/// property resolves to the same decoded file asset during data binding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScriptImage {
    file_asset_index: u64,
}

impl ScriptImage {
    pub fn file_asset_index(self) -> u64 {
        self.file_asset_index
    }
}

impl ScriptViewModel {
    pub fn property(&self, name: &str) -> Option<ScriptViewModelProperty> {
        self.properties.get(name).copied()
    }

    pub fn properties(&self) -> &BTreeMap<String, ScriptViewModelProperty> {
        &self.properties
    }

    pub fn named_instance(&self, name: Option<&str>) -> Option<Self> {
        let instance = match name {
            Some(name) => {
                let view_model = self.file.view_model(self.view_model_index)?;
                let instance_index = view_model
                    .instances
                    .iter()
                    .position(|instance| instance.object.string_property("name") == Some(name));
                instance_index
                    .and_then(|index| {
                        RuntimeOwnedViewModelInstance::from_instance(
                            &self.file,
                            self.view_model_index,
                            index,
                        )
                    })
                    .or_else(|| {
                        RuntimeOwnedViewModelInstance::new(&self.file, self.view_model_index)
                    })?
            }
            None => RuntimeOwnedViewModelInstance::new(&self.file, self.view_model_index)?,
        };
        build_script_view_model(
            Rc::clone(&self.file),
            self.view_model_index,
            instance,
            self.ancestors.as_slice(),
        )
    }

    /// Compatibility access to the retained graph root.
    ///
    /// Scoped integrations should prefer [`Self::owned_handle`] so a nested
    /// view model keeps its property path as well as the shared root identity.
    pub fn owned_instance(&self) -> Rc<RefCell<RuntimeOwnedViewModelInstance>> {
        self.context.root_handle().shared()
    }

    pub fn owned_handle(&self) -> RuntimeOwnedViewModelContextHandle {
        self.context.clone()
    }

    pub fn number(&self, name: &str) -> Option<f32> {
        let path = self.scoped_property_path(name)?;
        self.context
            .root_handle()
            .borrow()
            .number_value_by_property_path(&path)
    }

    pub fn set_number(&self, name: &str, value: f32) -> bool {
        let Some(path) = self.scoped_property_path(name) else {
            return false;
        };
        self.context
            .root_handle()
            .borrow_mut()
            .set_number_by_property_path(&path, value)
    }

    pub fn color(&self, name: &str) -> Option<u32> {
        let path = self.scoped_property_path(name)?;
        self.context
            .root_handle()
            .borrow()
            .color_value_by_property_path(&path)
    }

    pub fn set_color(&self, name: &str, value: u32) -> bool {
        let Some(path) = self.scoped_property_path(name) else {
            return false;
        };
        self.context
            .root_handle()
            .borrow_mut()
            .set_color_by_property_path(&path, value)
    }

    pub fn string(&self, name: &str) -> Option<String> {
        let path = self.scoped_property_path(name)?;
        self.context
            .root_handle()
            .borrow()
            .string_value_by_property_path(&path)
            .map(|value| String::from_utf8_lossy(value).into_owned())
    }

    pub fn set_string(&self, name: &str, value: &str) -> bool {
        let Some(path) = self.scoped_property_path(name) else {
            return false;
        };
        self.context
            .root_handle()
            .borrow_mut()
            .set_string_by_property_path(&path, value.as_bytes())
    }

    pub fn boolean(&self, name: &str) -> Option<bool> {
        let path = self.scoped_property_path(name)?;
        self.context
            .root_handle()
            .borrow()
            .boolean_value_by_property_path(&path)
    }

    pub fn image(&self, name: &str) -> Option<ScriptImage> {
        if self.property(name) != Some(ScriptViewModelProperty::Image) {
            return None;
        }
        let path = self.scoped_property_path(name)?;
        let file_asset_index = self
            .context
            .root_handle()
            .borrow()
            .asset_value_by_property_path(&path)?;
        let asset = self
            .file
            .file_asset(usize::try_from(file_asset_index).ok()?)?;
        (asset.type_name == "ImageAsset").then_some(ScriptImage { file_asset_index })
    }

    pub fn image_asset_named(&self, name: &str) -> Option<ScriptImage> {
        self.file
            .file_assets()
            .into_iter()
            .enumerate()
            .find(|(_, asset)| {
                asset.type_name == "ImageAsset" && asset.string_property("name") == Some(name)
            })
            .and_then(|(file_asset_index, _)| {
                u64::try_from(file_asset_index)
                    .ok()
                    .map(|file_asset_index| ScriptImage { file_asset_index })
            })
    }

    pub fn set_image(&self, name: &str, image: Option<ScriptImage>) -> bool {
        if self.property(name) != Some(ScriptViewModelProperty::Image) {
            return false;
        }
        let file_asset_index = image
            .map(ScriptImage::file_asset_index)
            .unwrap_or(u64::from(u32::MAX));
        let Some(path) = self.scoped_property_path(name) else {
            return false;
        };
        self.context
            .root_handle()
            .borrow_mut()
            .set_asset_by_property_path(&path, file_asset_index)
    }

    /// Mirrors C++ `ScriptedViewModel::pushIndex` for component-list rows.
    pub fn component_list_item_index(&self) -> Option<u64> {
        self.context
            .detached_snapshot()
            .and_then(|instance| instance.component_list_item_index())
    }

    pub fn set_boolean(&self, name: &str, value: bool) -> bool {
        let Some(path) = self.scoped_property_path(name) else {
            return false;
        };
        self.context
            .root_handle()
            .borrow_mut()
            .set_boolean_by_property_path(&path, value)
    }

    pub fn trigger(&self, name: &str) -> Option<u64> {
        let path = self.scoped_property_path(name)?;
        self.context
            .root_handle()
            .borrow()
            .trigger_value_by_property_path(&path)
    }

    /// Fire a trigger the same way C++ `ViewModelInstanceTrigger::trigger()`
    /// does: increment the backing counter and leave consumption/reset to the
    /// end-of-frame `advanced()` pass.
    pub fn fire_trigger(&self, name: &str) -> bool {
        let Some(value) = self.trigger(name) else {
            return false;
        };
        let Some(path) = self.scoped_property_path(name) else {
            return false;
        };
        self.context
            .root_handle()
            .borrow_mut()
            .set_trigger_by_property_path(&path, value.wrapping_add(1))
    }

    /// Consume transient values at the end of a script host frame.
    ///
    /// This mirrors C++ `ViewModelInstance::advanced()`: triggers are reset
    /// without invoking script listeners, embedded view models recurse, and
    /// shared list instances recurse exactly once even if the graph cycles.
    pub fn advance_script_frame(&self) -> bool {
        Self::advance_owned_instance(&self.context.root_handle().shared())
    }

    /// Advance a shared owned instance without requiring its schema wrapper.
    /// Scripting backends use this for owner-counted registrations that retain
    /// precisely the backing instance, matching C++ `rcp<ViewModelInstance>`.
    pub fn advance_owned_instance(instance: &Rc<RefCell<RuntimeOwnedViewModelInstance>>) -> bool {
        Self::advance_owned_instances(std::slice::from_ref(instance))
    }

    /// Advance several owned roots with one identity set shared across their
    /// complete embedded/list graphs. This is the frame-context entry point:
    /// registry relationships can name an instance that is also reachable
    /// structurally, and it must still be consumed only once per frame.
    pub fn advance_owned_instances(
        instances: &[Rc<RefCell<RuntimeOwnedViewModelInstance>>],
    ) -> bool {
        let mut visited = BTreeSet::new();
        let mut changed = false;
        for instance in instances {
            changed |= advance_owned_view_model_instance(instance, &mut visited);
        }
        changed
    }

    /// Snapshot the shared instances currently parented through this
    /// instance's list properties. The scripting registry refreshes these
    /// edges at frame end so host/data-binding list mutations cannot leave a
    /// retained wrapper incorrectly classified as attached or detached.
    pub fn owned_list_children(
        instance: &Rc<RefCell<RuntimeOwnedViewModelInstance>>,
    ) -> Vec<Rc<RefCell<RuntimeOwnedViewModelInstance>>> {
        instance.borrow().script_list_children()
    }

    pub fn view_model(&self, name: &str) -> Option<Self> {
        self.nested_view_models.get(name).cloned()
    }

    pub fn list_len(&self, name: &str) -> Option<usize> {
        let path = self.scoped_property_path(name)?;
        self.context
            .root_handle()
            .borrow()
            .list_item_count_by_property_path(&path)
    }

    pub fn list_item(&self, name: &str, index: usize) -> Option<Self> {
        let path = self.scoped_property_path(name)?;
        let item = self
            .context
            .root_handle()
            .borrow()
            .list_handle_by_property_path(&path)?
            .items()
            .get(index)
            .cloned()?;
        let view_model_index = item.borrow().view_model_index();
        build_script_view_model_shared(
            Rc::clone(&self.file),
            view_model_index,
            item,
            self.ancestors.as_slice(),
        )
    }

    pub fn push_list_item(&self, name: &str, item: &ScriptViewModel) -> bool {
        let Some(path) = self.scoped_property_path(name) else {
            return false;
        };
        let item_context = item.owned_handle();
        if !item_context.is_root() {
            return false;
        }
        let root = self.context.root_handle();
        root.push_list_item_by_property_path(&path, item_context.root_handle().shared())
    }

    pub fn insert_list_item(&self, name: &str, index: usize, item: &ScriptViewModel) -> bool {
        let Some(path) = self.scoped_property_path(name) else {
            return false;
        };
        let item_context = item.owned_handle();
        if !item_context.is_root() {
            return false;
        }
        let root = self.context.root_handle();
        root.insert_list_item_by_property_path(&path, index, item_context.root_handle().shared())
    }

    pub fn pop_list_item(&self, name: &str) -> Option<Self> {
        let path = self.scoped_property_path(name)?;
        let item = self
            .context
            .root_handle()
            .borrow_mut()
            .pop_list_item_by_property_path(&path)?;
        let view_model_index = item.borrow().view_model_index();
        build_script_view_model_shared(
            Rc::clone(&self.file),
            view_model_index,
            RuntimeOwnedViewModelHandle::from_shared(item),
            self.ancestors.as_slice(),
        )
    }

    pub fn shift_list_item(&self, name: &str) -> Option<Self> {
        let path = self.scoped_property_path(name)?;
        let item = self
            .context
            .root_handle()
            .borrow_mut()
            .shift_list_item_by_property_path(&path)?;
        let view_model_index = item.borrow().view_model_index();
        build_script_view_model_shared(
            Rc::clone(&self.file),
            view_model_index,
            RuntimeOwnedViewModelHandle::from_shared(item),
            self.ancestors.as_slice(),
        )
    }

    pub fn swap_list_items(&self, name: &str, first: usize, second: usize) -> bool {
        let Some(path) = self.scoped_property_path(name) else {
            return false;
        };
        self.context
            .root_handle()
            .borrow_mut()
            .swap_list_items_by_property_path(&path, first, second)
    }

    pub fn clear_list_items(&self, name: &str) -> bool {
        let Some(path) = self.scoped_property_path(name) else {
            return false;
        };
        self.context
            .root_handle()
            .borrow_mut()
            .clear_list_items_by_property_path(&path)
    }

    pub fn remove_list_item_at(&self, name: &str, index: usize) -> bool {
        let Some(path) = self.scoped_property_path(name) else {
            return false;
        };
        self.context
            .root_handle()
            .borrow_mut()
            .remove_list_item_at_by_property_path(&path, index)
    }

    pub fn remove_list_item(&self, name: &str, item: &ScriptViewModel, remove_all: bool) -> bool {
        let Some(path) = self.scoped_property_path(name) else {
            return false;
        };
        let item_context = item.owned_handle();
        if !item_context.is_root() {
            return false;
        }
        let item = item_context.root_handle().shared();
        self.context
            .root_handle()
            .borrow_mut()
            .remove_list_items_by_identity_at_property_path(&path, &item, remove_all)
    }

    fn scoped_property_path(&self, name: &str) -> Option<Vec<usize>> {
        let property_index = self
            .file
            .view_model(self.view_model_index)?
            .properties
            .iter()
            .position(|property| property.string_property("name") == Some(name))?;
        let mut path = self.context.scope_path().to_vec();
        path.push(property_index);
        Some(path)
    }
}

fn advance_owned_view_model_instance(
    instance: &Rc<RefCell<RuntimeOwnedViewModelInstance>>,
    visited: &mut BTreeSet<usize>,
) -> bool {
    let identity = Rc::as_ptr(instance) as usize;
    if !visited.insert(identity) {
        return false;
    }
    let (mut changed, children) = instance.borrow_mut().advance_script_frame_local();
    for child in children {
        changed |= advance_owned_view_model_instance(&child, visited);
    }
    changed
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptViewModelProperty {
    Number,
    Color,
    String,
    Boolean,
    Trigger,
    Image,
    List,
    ViewModel,
    SymbolListIndex,
}

pub fn script_view_models(file: &RuntimeFile) -> BTreeMap<String, ScriptViewModel> {
    let file = Rc::new(file.clone());
    file.view_models()
        .into_iter()
        .enumerate()
        .filter_map(|(view_model_index, view_model)| {
            let name = view_model.object.string_property("name")?.to_owned();
            let instance = RuntimeOwnedViewModelInstance::new(&file, view_model_index)?;
            Some((
                name,
                build_script_view_model(Rc::clone(&file), view_model_index, instance, &[])?,
            ))
        })
        .collect()
}

pub fn script_view_model_from_owned(
    file: &RuntimeFile,
    instance: &RuntimeOwnedViewModelHandle,
) -> Option<ScriptViewModel> {
    let view_model_index = instance.borrow().view_model_index();
    build_script_view_model_shared(
        Rc::new(file.clone()),
        view_model_index,
        instance.clone(),
        &[],
    )
}

/// Build a detached scripting snapshot from one owned view-model value.
///
/// Product integrations should normally use [`script_view_model_from_owned`]
/// so artboards, state machines, and scripts retain the same mutable graph.
pub fn script_view_model_from_owned_snapshot(
    file: &RuntimeFile,
    instance: &RuntimeOwnedViewModelInstance,
) -> Option<ScriptViewModel> {
    let view_model_index = instance.view_model_index();
    build_script_view_model_shared(
        Rc::new(file.clone()),
        view_model_index,
        RuntimeOwnedViewModelHandle::new(instance.clone()),
        &[],
    )
}

fn build_script_view_model(
    file: Rc<RuntimeFile>,
    view_model_index: usize,
    instance: RuntimeOwnedViewModelInstance,
    ancestors: &[usize],
) -> Option<ScriptViewModel> {
    build_script_view_model_shared(
        file,
        view_model_index,
        RuntimeOwnedViewModelHandle::new(instance),
        ancestors,
    )
}

fn build_script_view_model_shared(
    file: Rc<RuntimeFile>,
    view_model_index: usize,
    instance: RuntimeOwnedViewModelHandle,
    ancestors: &[usize],
) -> Option<ScriptViewModel> {
    let context = RuntimeOwnedViewModelContextHandle::root(&file, instance);
    build_script_view_model_scoped(file, view_model_index, context, ancestors)
}

fn build_script_view_model_scoped(
    file: Rc<RuntimeFile>,
    view_model_index: usize,
    context: RuntimeOwnedViewModelContextHandle,
    ancestors: &[usize],
) -> Option<ScriptViewModel> {
    let view_model = file.view_model(view_model_index)?;
    let properties = view_model
        .properties
        .iter()
        .filter_map(|property| {
            let kind = match property.type_name {
                "ViewModelPropertyNumber" => ScriptViewModelProperty::Number,
                "ViewModelPropertyColor" => ScriptViewModelProperty::Color,
                "ViewModelPropertyString" => ScriptViewModelProperty::String,
                "ViewModelPropertyBoolean" => ScriptViewModelProperty::Boolean,
                "ViewModelPropertyTrigger" => ScriptViewModelProperty::Trigger,
                "ViewModelPropertyAssetImage" => ScriptViewModelProperty::Image,
                "ViewModelPropertyList" => ScriptViewModelProperty::List,
                "ViewModelPropertyViewModel" => ScriptViewModelProperty::ViewModel,
                "ViewModelPropertySymbolListIndex" => ScriptViewModelProperty::SymbolListIndex,
                _ => return None,
            };
            Some((property.string_property("name")?.to_owned(), kind))
        })
        .collect();
    let mut child_ancestors = ancestors.to_vec();
    child_ancestors.push(view_model_index);
    let nested_view_models = view_model
        .properties
        .iter()
        .enumerate()
        .filter(|(_, property)| property.type_name == "ViewModelPropertyViewModel")
        .filter_map(|(property_index, property)| {
            let name = property.string_property("name")?.to_owned();
            let mut nested_scope_path = context.scope_path().to_vec();
            nested_scope_path.push(property_index);
            let nested_context = context.scoped(nested_scope_path)?;
            let nested_index = nested_context.view_model_index()?;
            if child_ancestors.contains(&nested_index) {
                return None;
            }
            Some((
                name,
                build_script_view_model_scoped(
                    Rc::clone(&file),
                    nested_index,
                    nested_context,
                    &child_ancestors,
                )?,
            ))
        })
        .collect();
    Some(ScriptViewModel {
        properties,
        nested_view_models,
        context,
        file,
        view_model_index,
        ancestors: Rc::new(ancestors.to_vec()),
    })
}

impl ScriptValue {
    pub fn as_number(&self) -> Option<f64> {
        match self {
            ScriptValue::Number(value) => Some(*value),
            _ => None,
        }
    }
}

/// Resolves the source-to-target `DataBindContext` value owned by a script input.
///
/// C++ keeps script-input data binds on the scripted object rather than in the
/// artboard data-bind container, so callers must hydrate them separately from
/// ordinary component bindings.
pub fn bound_script_input_value(
    file: &RuntimeFile,
    context: &RuntimeOwnedViewModelInstance,
    input: &RuntimeObject,
) -> Result<Option<ScriptValue>, ScriptError> {
    let Some(value) = bound_script_input_graph_value(file, context, input, "propertyValue")? else {
        return Ok(None);
    };
    let value = match (input.type_name, value) {
        ("ScriptInputBoolean", RuntimeDataBindGraphValue::Boolean(value)) => {
            ScriptValue::Bool(value)
        }
        ("ScriptInputNumber", RuntimeDataBindGraphValue::Number(value)) => {
            ScriptValue::Number(f64::from(value))
        }
        ("ScriptInputColor", RuntimeDataBindGraphValue::Color(value)) => ScriptValue::Color(value),
        ("ScriptInputString", RuntimeDataBindGraphValue::String(value)) => {
            ScriptValue::String(String::from_utf8_lossy(&value).into_owned())
        }
        (_, value) => {
            return Err(ScriptError::new(format!(
                "{} global {} data binding produced incompatible value {value:?}",
                input.type_name, input.id
            )));
        }
    };
    Ok(Some(value))
}

/// Resolves a data-bound `ScriptInputArtboard` to its referenced artboard id.
pub fn bound_script_artboard_input(
    file: &RuntimeFile,
    context: &RuntimeOwnedViewModelInstance,
    input: &RuntimeObject,
) -> Result<Option<u64>, ScriptError> {
    if input.type_name != "ScriptInputArtboard" {
        return Ok(None);
    }
    match bound_script_input_graph_value(file, context, input, "artboardId")? {
        Some(RuntimeDataBindGraphValue::Artboard(value)) => Ok(Some(value)),
        Some(value) => Err(ScriptError::new(format!(
            "{} global {} data binding produced incompatible value {value:?}",
            input.type_name, input.id
        ))),
        None => Ok(None),
    }
}

/// Resolves the current count of a data-bound `ScriptInputTrigger`.
///
/// The listener hydrator compares counts across retained data-context rebinds
/// and calls the table's authored trigger function only for a changed,
/// non-zero value, matching `ScriptInputTrigger::propertyValueChanged`.
pub fn bound_script_trigger_input(
    file: &RuntimeFile,
    context: &RuntimeOwnedViewModelInstance,
    input: &RuntimeObject,
) -> Result<Option<u64>, ScriptError> {
    if input.type_name != "ScriptInputTrigger" {
        return Ok(None);
    }
    match bound_script_input_graph_value(file, context, input, "propertyValue")? {
        Some(RuntimeDataBindGraphValue::Trigger(value)) => Ok(Some(value)),
        Some(value) => Err(ScriptError::new(format!(
            "{} global {} data binding produced incompatible value {value:?}",
            input.type_name, input.id
        ))),
        None => Ok(None),
    }
}

fn bound_script_input_graph_value(
    file: &RuntimeFile,
    context: &RuntimeOwnedViewModelInstance,
    input: &RuntimeObject,
    property_name: &str,
) -> Result<Option<RuntimeDataBindGraphValue>, ScriptError> {
    let Some(property_key) = property_key_for_name(input.type_name, property_name) else {
        return Ok(None);
    };
    let Some(data_bind) = (0..file.object_count()).find_map(|id| {
        let data_bind = file.object(id)?;
        (data_bind.type_name == "DataBindContext"
            && data_bind.uint_property("propertyKey") == Some(u64::from(property_key))
            && file
                .data_bind_target_for_object(data_bind)
                .is_some_and(|target| target.id == input.id)
            && file
                .data_bind_to_target_for_object(data_bind)
                .unwrap_or(false))
        .then_some(data_bind)
    }) else {
        return Ok(None);
    };
    let Some(source_path) = file.data_bind_context_resolved_source_path_ids_for_object(data_bind)
    else {
        return Ok(None);
    };
    let name_based = file
        .data_bind_is_name_based_for_object(data_bind)
        .unwrap_or(false);
    let Some(source) = owned_script_input_source_value(file, context, &source_path, name_based)
    else {
        return Ok(None);
    };

    let Some(converter_object) = file.resolved_data_converter_for_data_bind_object(data_bind)
    else {
        if data_bind
            .uint_property("converterId")
            .is_some_and(|id| id != u64::from(u32::MAX) && id != u64::MAX)
        {
            return Err(ScriptError::new(format!(
                "{} global {} references an unresolved data converter",
                input.type_name, input.id
            )));
        }
        return Ok(Some(source));
    };
    let Some(converter) = runtime_data_bind_graph_converter(file, data_bind) else {
        return Err(ScriptError::new(format!(
            "{} global {} data converter '{}' could not be resolved",
            input.type_name, input.id, converter_object.type_name
        )));
    };
    if !script_input_converter_is_stateless(&converter) {
        return Err(ScriptError::new(format!(
            "{} global {} data converter '{}' requires retained converter state and is unsupported for scripted-listener inputs",
            input.type_name, input.id, converter_object.type_name
        )));
    }
    runtime_data_bind_graph_convert_value(&converter, &source)
        .map(Some)
        .ok_or_else(|| {
            ScriptError::new(format!(
                "{} global {} data converter '{}' rejected its bound source value",
                input.type_name, input.id, converter_object.type_name
            ))
        })
}

fn script_input_converter_is_stateless(converter: &RuntimeDataBindGraphConverter) -> bool {
    match converter {
        RuntimeDataBindGraphConverter::PassThrough
        | RuntimeDataBindGraphConverter::BooleanNegate
        | RuntimeDataBindGraphConverter::TriggerIncrement
        | RuntimeDataBindGraphConverter::ToNumber
        | RuntimeDataBindGraphConverter::ListToLength
        | RuntimeDataBindGraphConverter::NumberToList { .. }
        | RuntimeDataBindGraphConverter::ToString { .. }
        | RuntimeDataBindGraphConverter::OperationValue { .. }
        | RuntimeDataBindGraphConverter::SystemOperationValue { .. }
        | RuntimeDataBindGraphConverter::Rounder { .. }
        | RuntimeDataBindGraphConverter::StringTrim { .. }
        | RuntimeDataBindGraphConverter::StringRemoveZeros
        | RuntimeDataBindGraphConverter::StringPad { .. } => true,
        RuntimeDataBindGraphConverter::Group(converters) => {
            converters.iter().all(script_input_converter_is_stateless)
        }
        RuntimeDataBindGraphConverter::Scripted { .. }
        | RuntimeDataBindGraphConverter::OperationViewModel { .. }
        | RuntimeDataBindGraphConverter::RangeMapper { .. }
        | RuntimeDataBindGraphConverter::Formula { .. }
        | RuntimeDataBindGraphConverter::Interpolator { .. }
        | RuntimeDataBindGraphConverter::Unsupported => false,
    }
}

fn owned_script_input_source_value(
    file: &RuntimeFile,
    context: &RuntimeOwnedViewModelInstance,
    source_path: &[u32],
    name_based: bool,
) -> Option<RuntimeDataBindGraphValue> {
    context
        .number_value_by_context_source_path(file, &[], source_path, name_based)
        .map(RuntimeDataBindGraphValue::Number)
        .or_else(|| {
            context
                .boolean_value_by_context_source_path(file, &[], source_path, name_based)
                .map(RuntimeDataBindGraphValue::Boolean)
        })
        .or_else(|| {
            context
                .string_value_by_context_source_path(file, &[], source_path, name_based)
                .map(|value| RuntimeDataBindGraphValue::String(value.to_vec()))
        })
        .or_else(|| {
            context
                .color_value_by_context_source_path(file, &[], source_path, name_based)
                .map(RuntimeDataBindGraphValue::Color)
        })
        .or_else(|| {
            context
                .enum_value_by_context_source_path(file, &[], source_path, name_based)
                .map(RuntimeDataBindGraphValue::Enum)
        })
        .or_else(|| {
            context
                .symbol_list_index_value_by_context_source_path(file, &[], source_path, name_based)
                .map(RuntimeDataBindGraphValue::SymbolListIndex)
        })
        .or_else(|| {
            context
                .list_item_count_by_context_source_path(file, &[], source_path, name_based)
                .map(|item_count| RuntimeDataBindGraphValue::List { item_count })
        })
        .or_else(|| {
            context
                .asset_value_by_context_source_path(file, &[], source_path, name_based)
                .map(RuntimeDataBindGraphValue::Asset)
        })
        .or_else(|| {
            context
                .artboard_value_by_context_source_path(file, &[], source_path, name_based)
                .map(RuntimeDataBindGraphValue::Artboard)
        })
        .or_else(|| {
            context
                .trigger_value_by_context_source_path(file, &[], source_path, name_based)
                .map(RuntimeDataBindGraphValue::Trigger)
        })
        .or_else(|| {
            context
                .view_model_value_by_context_source_path(file, &[], source_path, name_based)
                .map(RuntimeDataBindGraphValue::ViewModel)
        })
}

/// Resolves a `ScriptInputViewModelProperty` after its scripted object has a
/// data context. C++ treats hydration as all-or-nothing, so `None` means the
/// caller must defer every input and user `init`, not install a nil stand-in.
pub fn bound_script_view_model_from_owned_context(
    file: &RuntimeFile,
    context: &RuntimeOwnedViewModelContextHandle,
    input: &RuntimeObject,
) -> Option<ScriptViewModel> {
    if input.type_name != "ScriptInputViewModelProperty" {
        return None;
    }
    let source_path = file.resolved_data_bind_path_ids_for_referencer_object(input)?;
    let root = context.root_handle();
    let property_path = root.borrow().property_path_for_context_source_path(
        file,
        context.scope_path(),
        &source_path,
        false,
    )?;
    let nested_context = context.scoped(property_path)?;
    let view_model_index = nested_context.view_model_index()?;
    build_script_view_model_scoped(Rc::new(file.clone()), view_model_index, nested_context, &[])
}

/// Hydrate a detached scripting snapshot from a detached owned context.
///
/// Retained runtime integrations should use
/// [`bound_script_view_model_from_owned_context`] so nested mutations keep the
/// same graph identity and invalidation path.
pub fn bound_script_view_model_snapshot(
    file: &RuntimeFile,
    context: &RuntimeOwnedViewModelInstance,
    input: &RuntimeObject,
) -> Option<ScriptViewModel> {
    if input.type_name != "ScriptInputViewModelProperty" {
        return None;
    }
    let source_path = file.resolved_data_bind_path_ids_for_referencer_object(input)?;
    let property_path =
        context.property_path_for_context_source_path(file, &[], &source_path, false)?;
    let view_model_index = context.view_model_index_by_property_path(&property_path)?;
    let instance = context.nested_instance_by_property_path(&property_path)?;
    build_script_view_model(Rc::new(file.clone()), view_model_index, instance, &[])
}

/// Host callbacks exposed to scripted objects.
///
/// The first scripting slice only needs a dirt/update marker; richer access
/// to artboards, renderers, and view-model data lives behind this same trait
/// as the C++ `src/lua/` glue is ported.
pub trait ScriptHost {
    fn mark_script_update(&mut self) {}
}

#[derive(Debug, Default)]
pub struct NoopScriptHost;

impl ScriptHost for NoopScriptHost {}

/// Runtime-owned linear-animation handle exposed to scripts.
///
/// Coarsely translated from `ScriptedAnimation` in
/// `/Users/levi/dev/oss/rive-runtime/src/lua/lua_artboards.cpp`.
#[derive(Debug, Clone)]
pub struct ScriptAnimation {
    instance: LinearAnimationInstance,
    duration: f32,
    fps: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptAnimationTime {
    Seconds,
    Frames,
    Percentage,
}

impl ScriptAnimation {
    pub fn named(artboard: &ArtboardInstance, name: &str) -> Option<Self> {
        let (index, animation) = artboard
            .linear_animations()
            .iter()
            .enumerate()
            .find(|(_, animation)| animation.name.as_deref() == Some(name))?;
        Some(Self {
            instance: artboard.linear_animation_instance(index)?,
            duration: animation.duration as f32 / animation.fps as f32,
            fps: animation.fps as f32,
        })
    }

    pub fn duration(&self) -> f32 {
        self.duration
    }

    pub fn advance(&mut self, artboard: &mut ArtboardInstance, seconds: f32) -> bool {
        let keep_going = artboard.advance_linear_animation_instance(&mut self.instance, seconds);
        artboard.apply_linear_animation_instance(&self.instance, 1.0);
        keep_going
    }

    pub fn set_time(
        &mut self,
        artboard: &mut ArtboardInstance,
        value: f32,
        mode: ScriptAnimationTime,
    ) {
        let seconds = match mode {
            ScriptAnimationTime::Seconds => value,
            ScriptAnimationTime::Frames => value / self.fps,
            ScriptAnimationTime::Percentage => value * self.duration,
        };
        let Some(animation) = artboard
            .linear_animation(self.instance.animation_index())
            .cloned()
        else {
            return;
        };
        self.instance
            .set_time(&animation, animation.global_to_local_seconds(seconds));
        artboard.apply_linear_animation_instance(&self.instance, 1.0);
    }
}

/// Runtime-owned artboard userdata exposed to scripts.
pub trait ScriptArtboard {
    fn width(&self) -> f32;
    fn height(&self) -> f32;
    fn frame_origin(&self) -> bool;
    fn set_width(&mut self, width: f32);
    fn set_height(&mut self, height: f32);
    fn set_frame_origin(&mut self, frame_origin: bool);

    fn data(&self) -> Option<ScriptViewModel> {
        None
    }

    fn instance(
        &self,
        view_model: Option<ScriptViewModel>,
    ) -> Result<Box<dyn ScriptArtboard>, ScriptError>;

    fn advance(&mut self, _seconds: f32) -> Result<bool, ScriptError> {
        Ok(false)
    }

    fn animation(&self, _name: &str) -> Result<Option<ScriptAnimation>, ScriptError> {
        Ok(None)
    }

    fn advance_animation(
        &mut self,
        _animation: &mut ScriptAnimation,
        _seconds: f32,
    ) -> Result<bool, ScriptError> {
        Ok(false)
    }

    fn set_animation_time(
        &mut self,
        _animation: &mut ScriptAnimation,
        _value: f32,
        _mode: ScriptAnimationTime,
    ) -> Result<(), ScriptError> {
        Ok(())
    }

    fn node(&self, _name: &str) -> Result<Option<ScriptNode>, ScriptError> {
        Ok(None)
    }

    fn draw(
        &mut self,
        factory: &mut dyn RenderFactory,
        renderer: &mut dyn Renderer,
    ) -> Result<(), ScriptError>;
}

/// Runtime-owned handle for one scripted object instance.
pub trait ScriptInstance {
    fn set_context_view_model(
        &mut self,
        _view_model: Option<ScriptViewModel>,
    ) -> Result<(), ScriptError> {
        Ok(())
    }

    fn has_method(&self, method: ScriptMethod) -> Result<bool, ScriptError>;

    fn call_method(
        &mut self,
        method: ScriptMethod,
        args: &[ScriptValue],
        host: &mut dyn ScriptHost,
    ) -> Result<ScriptValue, ScriptError>;

    fn call_method_with_factory(
        &mut self,
        method: ScriptMethod,
        args: &[ScriptValue],
        host: &mut dyn ScriptHost,
        factory: &mut dyn RenderFactory,
    ) -> Result<ScriptValue, ScriptError> {
        let _ = factory;
        self.call_method(method, args, host)
    }

    /// Invoke one already-selected listener callback with its typed payload.
    ///
    /// Concrete VMs override this to create the native Invocation userdata
    /// used by `performAction` (or the pointer placeholder used by legacy
    /// `perform`). Keeping that conversion here avoids leaking VM types into
    /// the state-machine module.
    fn call_listener_action(
        &mut self,
        method: ScriptListenerActionMethod,
        invocation: &ScriptListenerInvocation,
        host: &mut dyn ScriptHost,
    ) -> Result<(), ScriptError> {
        let _ = invocation;
        self.call_method(method.as_script_method(), &[], host)
            .map(|_| ())
    }

    /// Invoke an authored `ScriptInputTrigger` callback by its input name.
    /// Missing or non-function fields are a no-op, matching Rive's runtime.
    fn call_input_trigger(
        &mut self,
        _name: &str,
        _host: &mut dyn ScriptHost,
    ) -> Result<(), ScriptError> {
        Ok(())
    }

    /// Run an implemented user `init(self, context)` and apply Lua truthiness
    /// without requiring every backend value kind to cross the VM-neutral
    /// [`ScriptValue`] seam.
    fn call_init_with_factory(
        &mut self,
        host: &mut dyn ScriptHost,
        factory: &mut dyn RenderFactory,
    ) -> Result<bool, ScriptError> {
        let value = self.call_method_with_factory(ScriptMethod::Init, &[], host, factory)?;
        Ok(!matches!(
            value,
            ScriptValue::Nil | ScriptValue::Bool(false)
        ))
    }

    /// Whether this concrete scripted-object occurrence still needs its user
    /// `init` callback. C++ stores the equivalent state in
    /// `ScriptedObject::m_userLuaInitDone`.
    fn user_init_pending(&self) -> Result<bool, ScriptError> {
        Ok(false)
    }

    /// Discard the current scripted-object lifetime before the next input
    /// hydration. VM backends use this when cold-init prerequisites are not
    /// available, matching C++ `ensureScriptInitialized` retry semantics.
    fn invalidate_for_init_retry(&mut self) {}

    /// Recreate a lifetime invalidated by a failed/deferred init. Hosts call
    /// this before hydrating inputs so a new script table observes the bound
    /// context and receives the complete input set.
    fn prepare_init_retry_with_factory(
        &mut self,
        factory: &mut dyn RenderFactory,
    ) -> Result<(), ScriptError> {
        let _ = factory;
        Ok(())
    }

    fn call_path_effect_update(
        &mut self,
        source: RawPath,
        node: ScriptNode,
        host: &mut dyn ScriptHost,
    ) -> Result<RawPath, ScriptError> {
        let _ = (source, node, host);
        Err(ScriptError::new(
            "script path effects require backend path userdata support",
        ))
    }

    fn call_draw(
        &mut self,
        factory: &mut dyn RenderFactory,
        renderer: &mut dyn Renderer,
        host: &mut dyn ScriptHost,
    ) -> Result<(), ScriptError> {
        let _ = (factory, renderer, host);
        Err(ScriptError::new(
            "script draw requires a backend renderer binding",
        ))
    }

    fn call_data_converter(
        &mut self,
        method: ScriptDataConverterMethod,
        value: ScriptValue,
    ) -> Result<ScriptValue, ScriptError> {
        let _ = (method, value);
        Err(ScriptError::new(
            "scripted data conversion requires backend data-value support",
        ))
    }

    fn get_input(&self, name: &str) -> Result<ScriptValue, ScriptError>;

    fn set_input(&mut self, name: &str, value: ScriptValue) -> Result<(), ScriptError>;

    fn set_artboard_input(
        &mut self,
        name: &str,
        artboard: Box<dyn ScriptArtboard>,
    ) -> Result<(), ScriptError> {
        let _ = (name, artboard);
        Err(ScriptError::new(
            "script artboard inputs require backend userdata support",
        ))
    }

    fn set_view_model_input(
        &mut self,
        name: &str,
        view_model: ScriptViewModel,
    ) -> Result<(), ScriptError> {
        let _ = (name, view_model);
        Err(ScriptError::new(
            "script view-model inputs require backend userdata support",
        ))
    }
}

impl ArtboardInstance {
    pub(crate) fn apply_scripted_path_effect(
        &self,
        global_id: u32,
        source: RawPath,
        node: ScriptNode,
    ) -> Result<RawPath, ScriptError> {
        let handle = self
            .script_instance_for_global(global_id)
            .ok_or_else(|| ScriptError::new(format!("missing script path effect {global_id}")))?;
        handle
            .borrow_mut()
            .call_path_effect_update(source, node, &mut NoopScriptHost)
    }
}

#[derive(Clone)]
pub(crate) struct RuntimeScriptInstanceHandle {
    inner: Rc<RefCell<Box<dyn ScriptInstance>>>,
}

impl RuntimeScriptInstanceHandle {
    pub(crate) fn new(instance: Box<dyn ScriptInstance>) -> Self {
        Self {
            inner: Rc::new(RefCell::new(instance)),
        }
    }

    pub(crate) fn borrow_mut(&self) -> RefMut<'_, Box<dyn ScriptInstance>> {
        self.inner.borrow_mut()
    }
}

impl PartialEq for RuntimeScriptInstanceHandle {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.inner, &other.inner)
    }
}

impl fmt::Debug for RuntimeScriptInstanceHandle {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RuntimeScriptInstanceHandle")
            .field("shared", &true)
            .finish()
    }
}

/// Runtime-owned VM seam implemented by concrete scripting backends.
pub trait ScriptingVm {
    fn install_rive_globals(&mut self) -> Result<(), ScriptError>;

    fn register_module(&mut self, name: &str, payload: &[u8]) -> Result<(), ScriptError>;

    fn instantiate_script(
        &mut self,
        name: &str,
        payload: &[u8],
        host: &mut dyn ScriptHost,
    ) -> Result<Box<dyn ScriptInstance>, ScriptError>;

    /// Consume detached script-created view-model instances once at the end
    /// of a root host frame. Child/script-driven artboard advances must not
    /// call this hook.
    fn advance_detached_view_models(&mut self) -> bool {
        false
    }

    fn perform_registration(&mut self, modules: &[ScriptModule<'_>]) -> Vec<ScriptModuleFailure> {
        let mut pending: Vec<usize> = (0..modules.len()).collect();
        loop {
            let before = pending.len();
            let mut failures = Vec::new();
            for index in pending {
                let module = modules[index];
                match self.register_module(module.name, module.payload) {
                    Ok(()) => {}
                    Err(error) => failures.push((index, error)),
                }
            }
            if failures.is_empty() {
                return Vec::new();
            }
            if failures.len() == before {
                return failures
                    .into_iter()
                    .map(|(index, error)| ScriptModuleFailure {
                        name: modules[index].name.to_owned(),
                        error,
                    })
                    .collect();
            }
            pending = failures.into_iter().map(|(index, _)| index).collect();
        }
    }
}
