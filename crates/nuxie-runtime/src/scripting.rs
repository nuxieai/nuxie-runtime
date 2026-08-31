use std::cell::{Cell, RefCell, RefMut};
use std::collections::BTreeMap;
use std::rc::{Rc, Weak};
use std::sync::Arc;
use std::{error::Error, fmt};

use nuxie_render_api::{
    BlendMode, Factory as RenderFactory, RawPath, RenderPaintStyle, Renderer, StrokeCap, StrokeJoin,
};

use crate::state_machine::ScriptListenerInvocation;
use crate::view_model_cell::{
    RuntimeBlobAsset, RuntimeCellDirtSink, RuntimeHostMutationNotifications,
};
mod native_artboard;
pub use native_artboard::native_script_artboard;
#[cfg(test)]
#[path = "scripting/pending_native_hydration.rs"]
mod pending_native_hydration;

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

/// Which pinned C++ state-machine `ScriptedObject` owner this occurrence
/// represents.
#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptedStateMachineObjectKind {
    ListenerAction,
    TransitionCondition,
}

/// One imported state-machine scripted object and the protocol asset it
/// resolves.
///
/// `asset_ordinal` is the dense file-asset ordinal serialized in
/// `ScriptedObject.scriptAssetId`; it is deliberately not the semantic
/// `FileAsset.assetId` or the asset object's global id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptListenerActionDefinition {
    action_global_id: u32,
    kind: ScriptedStateMachineObjectKind,
    asset_ordinal: usize,
    asset_name: String,
    has_protocol_asset: bool,
    serialized_implemented_methods: u32,
    inputs: Vec<ScriptListenerInputDefinition>,
}

impl ScriptListenerActionDefinition {
    #[cfg(test)]
    pub(crate) fn new(action_global_id: u32, asset_ordinal: usize, asset_name: String) -> Self {
        Self {
            action_global_id,
            kind: ScriptedStateMachineObjectKind::ListenerAction,
            asset_ordinal,
            asset_name,
            has_protocol_asset: true,
            serialized_implemented_methods:
                crate::mechanical_port::source::assets::script_asset::OptionalScriptedMethods::METHOD_MASK,
            inputs: Vec::new(),
        }
    }

    pub(crate) fn with_inputs(
        action_global_id: u32,
        asset_ordinal: usize,
        asset_name: String,
        has_protocol_asset: bool,
        serialized_implemented_methods: u32,
        inputs: Vec<ScriptListenerInputDefinition>,
    ) -> Self {
        Self::with_inputs_and_kind(
            action_global_id,
            ScriptedStateMachineObjectKind::ListenerAction,
            asset_ordinal,
            asset_name,
            has_protocol_asset,
            serialized_implemented_methods,
            inputs,
        )
    }

    pub(crate) fn with_inputs_and_kind(
        action_global_id: u32,
        kind: ScriptedStateMachineObjectKind,
        asset_ordinal: usize,
        asset_name: String,
        has_protocol_asset: bool,
        serialized_implemented_methods: u32,
        inputs: Vec<ScriptListenerInputDefinition>,
    ) -> Self {
        Self {
            action_global_id,
            kind,
            asset_ordinal,
            asset_name,
            has_protocol_asset,
            serialized_implemented_methods,
            inputs,
        }
    }

    pub fn action_global_id(&self) -> u32 {
        self.action_global_id
    }

    #[doc(hidden)]
    pub fn scripted_object_global_id(&self) -> u32 {
        self.action_global_id
    }

    #[doc(hidden)]
    pub fn scripted_object_kind(&self) -> ScriptedStateMachineObjectKind {
        self.kind
    }

    pub fn asset_ordinal(&self) -> usize {
        self.asset_ordinal
    }

    pub fn asset_name(&self) -> &str {
        &self.asset_name
    }

    /// Whether C++ resolved this occurrence to a non-module ScriptAsset.
    ///
    /// The authored ScriptedListenerAction and its ordered inputs remain
    /// present when this is false. Such an occurrence simply has no live Lua
    /// table and is inert at init/perform time.
    pub fn has_protocol_asset(&self) -> bool {
        self.has_protocol_asset
    }

    pub fn serialized_implemented_methods(&self) -> u32 {
        self.serialized_implemented_methods
    }

    pub fn inits(&self) -> bool {
        let mut methods =
            crate::mechanical_port::source::assets::script_asset::OptionalScriptedMethods::default(
            );
        methods.set_implemented_methods(self.serialized_implemented_methods as i32);
        methods.inits()
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

/// Lifecycle/input methods carried by scripted object instance tables.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScriptMethod {
    Init,
    Measure,
    Resize,
    Advance,
    Update,
    Draw,
    Evaluate,
    Transform,
    TransformValue,
    PointerDown,
    PointerMove,
    PointerUp,
    PointerEnter,
    PointerExit,
    KeyboardEvent,
    TextEvent,
    GamepadConnected,
    GamepadEvent,
    GamepadDisconnected,
    PerformAction,
    Perform,
}

impl ScriptMethod {
    pub fn as_str(self) -> &'static str {
        match self {
            ScriptMethod::Init => "init",
            ScriptMethod::Measure => "measure",
            ScriptMethod::Resize => "resize",
            ScriptMethod::Advance => "advance",
            ScriptMethod::Update => "update",
            ScriptMethod::Draw => "draw",
            ScriptMethod::Evaluate => "evaluate",
            ScriptMethod::Transform => "transform",
            ScriptMethod::TransformValue => "transformValue",
            ScriptMethod::PointerDown => "pointerDown",
            ScriptMethod::PointerMove => "pointerMove",
            ScriptMethod::PointerUp => "pointerUp",
            ScriptMethod::PointerEnter => "pointerEnter",
            ScriptMethod::PointerExit => "pointerExit",
            ScriptMethod::KeyboardEvent => "keyboardEvent",
            ScriptMethod::TextEvent => "textEvent",
            ScriptMethod::GamepadConnected => "gamepadConnected",
            ScriptMethod::GamepadEvent => "gamepadEvent",
            ScriptMethod::GamepadDisconnected => "gamepadDisconnected",
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

/// Outcome of one direct keyboard, text, or gamepad callback lookup.
///
/// C++ wakes the scripted drawable only after it finds and attempts the
/// current table function. `handled` is the callback's propagation result for
/// keyboard/text and true for an attempted gamepad callback.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ScriptedDrawableInputResult {
    pub invoked: bool,
    pub handled: bool,
}

/// Exact result written by a scripted drawable's `PointerEvent:hit()` call.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ScriptedDrawablePointerHit {
    #[default]
    None,
    Hit,
    HitOpaque,
}

/// Outcome of one direct scripted-drawable pointer callback.
///
/// `invoked` is distinct from `hit`: native code wakes the scripted owner
/// whenever the selected function is attempted, even when it leaves the
/// pointer event at `none` or raises an ordinary protected-call error.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ScriptedDrawablePointerResult {
    pub invoked: bool,
    pub hit: ScriptedDrawablePointerHit,
}

/// Byte-preserving Rive `CoreString` storage.
///
/// The binary format and C++ `std::string` retain arbitrary bytes, including
/// embedded NUL. C++ projects those values through `lua_pushstring` and
/// `lua_setfield`, which expose only the prefix before the first NUL. Keeping
/// both views here prevents an earlier UTF-8 replacement from changing the
/// authored Core value.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct ScriptCoreString(Vec<u8>);

impl ScriptCoreString {
    pub fn from_bytes(bytes: impl Into<Vec<u8>>) -> Self {
        Self(bytes.into())
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn as_c_str_bytes(&self) -> &[u8] {
        let end = self
            .0
            .iter()
            .position(|byte| *byte == 0)
            .unwrap_or(self.0.len());
        &self.0[..end]
    }

    pub fn into_bytes(self) -> Vec<u8> {
        self.0
    }
}

impl From<String> for ScriptCoreString {
    fn from(value: String) -> Self {
        Self(value.into_bytes())
    }
}

impl From<&str> for ScriptCoreString {
    fn from(value: &str) -> Self {
        Self(value.as_bytes().to_vec())
    }
}

impl PartialEq<str> for ScriptCoreString {
    fn eq(&self, other: &str) -> bool {
        self.as_bytes() == other.as_bytes()
    }
}

impl PartialEq<&str> for ScriptCoreString {
    fn eq(&self, other: &&str) -> bool {
        self == *other
    }
}

/// VM-neutral values crossing the scripting seam.
#[derive(Debug, Clone, PartialEq)]
pub enum ScriptValue {
    Nil,
    Bool(bool),
    Number(f64),
    String(String),
    CoreString(ScriptCoreString),
    Color(u32),
    Vec2 { x: f32, y: f32 },
    Vec3 { x: f32, y: f32, z: f32 },
}

/// Result of resolving and invoking one optional script callback atomically.
///
/// Backends with dynamic member lookup override
/// [`ScriptInstance::call_optional_method`] so presence and invocation observe
/// the same resolved function value.
#[derive(Debug, Clone, PartialEq)]
pub enum ScriptOptionalMethodResult {
    Missing,
    Returned(ScriptValue),
}

/// Lua callbacks implemented by a `ScriptedInterpolator` protocol table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptInterpolatorMethod {
    Transform,
    TransformValue,
}

impl ScriptInterpolatorMethod {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Transform => "transform",
            Self::TransformValue => "transformValue",
        }
    }
}

/// Result of atomically resolving and invoking one interpolator callback.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScriptOptionalNumberResult {
    Missing,
    Returned(f32),
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

/// Result of resolving one optional scripted data-converter callback.
///
/// The backend must resolve the field exactly once. `UnsupportedInput`
/// means the callback existed but the concrete Rive `DataValue` could not be
/// represented by the scripting backend, so the callback was not invoked.
#[derive(Debug, Clone, PartialEq)]
pub enum ScriptDataConverterOptionalCall {
    Missing,
    UnsupportedInput,
    Returned(ScriptValue),
}

/// Runtime-owned node data exposed by C++ `ScriptedArtboard::node`.
#[derive(Debug, Clone)]
pub struct ScriptNode {
    pub path: Option<RawPath>,
    pub paint: Option<ScriptPaint>,
    live: Option<LiveScriptNode>,
}

#[derive(Debug, Clone)]
struct LiveScriptNode {
    component: crate::mechanical_port::source::core::CoreHandle,
    paint: Option<crate::mechanical_port::source::core::CoreHandle>,
    // The effect callback runs while its ShapePaint is borrowed. During that
    // scope use its exact most-derived snapshot; retained nodes subsequently
    // construct new PaintData from the live paint occurrence.
    active_paint: Weak<ScriptPaint>,
    artboard_owner: Option<Rc<native_artboard::NativeScriptArtboardOwner>>,
}

impl ScriptNode {
    pub fn snapshot(path: Option<RawPath>, paint: Option<ScriptPaint>) -> Self {
        Self {
            path,
            paint,
            live: None,
        }
    }

    pub(crate) fn from_component(
        component: crate::mechanical_port::source::core::CoreHandle,
    ) -> Self {
        Self {
            path: None,
            paint: None,
            live: Some(LiveScriptNode {
                component,
                paint: None,
                active_paint: Weak::new(),
                artboard_owner: None,
            }),
        }
    }

    pub(crate) fn from_path_effect(
        paint: &crate::mechanical_port::source::shapes::paint::shape_paint::ShapePaint,
    ) -> Self {
        let component = paint
            .parent_transform_component()
            .expect("path effect parent TransformComponent");
        Self {
            path: None,
            paint: None,
            live: Some(LiveScriptNode {
                component,
                paint: paint.handle(),
                active_paint: paint.script_paint_scope(),
                artboard_owner: None,
            }),
        }
    }

    fn component(&self) -> &crate::mechanical_port::source::core::CoreHandle {
        &self
            .live
            .as_ref()
            .expect("transform access requires a live ScriptNode")
            .component
    }

    fn position(&self) -> (f32, f32) {
        self.component()
            .with(|object| {
                if let Some(layout) = object.as_layout_component() {
                    (layout.layout_x(), layout.layout_y())
                } else if let Some(root) = object.as_root_bone() {
                    (root.x(), root.y())
                } else if object.as_bone().is_some() {
                    let parent = object.component_parent_handle().expect("Bone parent");
                    (
                        parent
                            .with(|parent| parent.as_bone().expect("Bone parent").length())
                            .expect("live Bone parent"),
                        0.0,
                    )
                } else {
                    let node = object.as_node().expect("concrete TransformComponent x/y");
                    (node.base.x(), node.base.y())
                }
            })
            .expect("live ScriptNode occurrence")
    }

    pub fn x(&self) -> f32 {
        self.position().0
    }
    pub fn y(&self) -> f32 {
        self.position().1
    }

    fn set_position_axis(&mut self, x: bool, value: f32) {
        use crate::mechanical_port::source::generated::core_registry::CoreField;
        self.component().with_mut(|object| {
            let field = if object.as_node().is_some() {
                if x {
                    CoreField::NodeX
                } else {
                    CoreField::NodeY
                }
            } else if object.as_root_bone().is_some() {
                if x {
                    CoreField::RootBoneX
                } else {
                    CoreField::RootBoneY
                }
            } else {
                return;
            };
            object.set_double(field, value);
        });
    }
    pub fn set_x(&mut self, value: f32) {
        self.set_position_axis(true, value);
    }
    pub fn set_y(&mut self, value: f32) {
        self.set_position_axis(false, value);
    }

    pub fn rotation(&self) -> f32 {
        self.component()
            .with(|object| object.as_transform_component().unwrap().rotation())
            .expect("live ScriptNode")
    }
    pub fn scale_x(&self) -> f32 {
        self.component()
            .with(|object| object.as_transform_component().unwrap().scale_x())
            .expect("live ScriptNode")
    }
    pub fn scale_y(&self) -> f32 {
        self.component()
            .with(|object| object.as_transform_component().unwrap().scale_y())
            .expect("live ScriptNode")
    }
    pub fn set_rotation(&mut self, value: f32) {
        self.component().with_mut(|object| object.set_double(crate::mechanical_port::source::generated::core_registry::CoreField::TransformComponentRotation, value));
    }
    pub fn set_scale_x(&mut self, value: f32) {
        self.component().with_mut(|object| object.set_double(crate::mechanical_port::source::generated::core_registry::CoreField::TransformComponentScaleX, value));
    }
    pub fn set_scale_y(&mut self, value: f32) {
        self.component().with_mut(|object| object.set_double(crate::mechanical_port::source::generated::core_registry::CoreField::TransformComponentScaleY, value));
    }
    pub fn world_transform(&self) -> nuxie_render_api::Mat2D {
        self.component()
            .with(|object| {
                nuxie_render_api::Mat2D(
                    *object
                        .as_world_transform_component()
                        .unwrap()
                        .world_transform()
                        .values(),
                )
            })
            .expect("live ScriptNode")
    }
    pub fn set_world_transform(&mut self, transform: nuxie_render_api::Mat2D) {
        let [a, b, c, d, x, y] = transform.0;
        self.component().with_mut(|object| {
            object
                .as_world_transform_component_mut()
                .unwrap()
                .set_world_transform(crate::mechanical_port::source::math::mat2d::Mat2D::new(
                    a, b, c, d, x, y,
                ))
        });
    }
    pub fn children(&self) -> Vec<Self> {
        let children = self
            .component()
            .with(|object| {
                object
                    .as_container_component()
                    .map(|container| container.children().to_vec())
                    .unwrap_or_default()
            })
            .expect("live ScriptNode");
        children
            .into_iter()
            .filter(|child| {
                // An effect's own ShapePaint is currently mutably borrowed and
                // cannot be a TransformComponent child.
                if self.live.as_ref().and_then(|live| live.paint.as_ref()) == Some(child) {
                    return false;
                }
                child
                    .with(|child| child.as_transform_component().is_some())
                    .unwrap_or(false)
            })
            .map(|component| self.related_node(component))
            .collect()
    }
    pub fn parent(&self) -> Option<Self> {
        self.component()
            .with(|object| object.component_parent_handle())
            .flatten()
            .filter(|parent| {
                parent
                    .with(|parent| parent.as_transform_component().is_some())
                    .unwrap_or(false)
            })
            .map(|component| self.related_node(component))
    }
    fn related_node(&self, component: crate::mechanical_port::source::core::CoreHandle) -> Self {
        let mut node = Self::from_component(component);
        node.live.as_mut().unwrap().artboard_owner =
            self.live.as_ref().unwrap().artboard_owner.clone();
        node
    }
    pub fn decompose(&mut self, transform: nuxie_render_api::Mat2D) {
        use crate::mechanical_port::source::math::mat2d::Mat2D;
        let parent = self
            .component()
            .with(|object| object.component_parent_handle())
            .flatten();
        let parent_world = parent
            .and_then(|parent| {
                parent
                    .with(|parent| {
                        parent
                            .as_world_transform_component()
                            .map(|parent| *parent.world_transform())
                    })
                    .flatten()
            })
            .unwrap_or_else(Mat2D::identity);
        let [a, b, c, d, x, y] = transform.0;
        let components =
            (parent_world.invert_or_identity() * Mat2D::new(a, b, c, d, x, y)).decompose();
        self.set_x(components.x());
        self.set_y(components.y());
        self.set_scale_x(components.scale_x());
        self.set_scale_y(components.scale_y());
        self.set_rotation(components.rotation());
    }
    pub fn path(&self) -> Option<RawPath> {
        match &self.live {
            Some(live) => live
                .component
                .with(|object| {
                    object.as_path().map(|path| {
                        crate::mechanical_port::source::renderer::to_render_raw_path(
                            path.raw_path(),
                        )
                    })
                })
                .flatten(),
            None => self.path.clone(),
        }
    }
    pub fn paint(&self) -> Option<ScriptPaint> {
        let Some(live) = &self.live else {
            return self.paint;
        };
        if let Some(paint) = live.active_paint.upgrade() {
            return Some(*paint);
        }
        live.paint
            .as_ref()
            .unwrap_or(&live.component)
            .with(|object| {
                let paint = object.as_shape_paint()?;
                let stroke = object
                    .as_any()
                    .downcast_ref::<crate::mechanical_port::source::shapes::paint::stroke::Stroke>()
                    .map(|stroke| {
                        (
                            stroke.base.thickness(),
                            stroke.base.cap(),
                            stroke.base.join(),
                        )
                    });
                Some(ScriptPaint::from_fresh(paint, stroke))
            })
            .flatten()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ScriptPaint {
    pub style: RenderPaintStyle,
    pub color: u32,
    pub thickness: f32,
    pub join: StrokeJoin,
    pub cap: StrokeCap,
    pub feather: f32,
    pub blend_mode: u8,
}

impl ScriptPaint {
    pub fn blend_mode(&self) -> BlendMode {
        // ScriptedPaintData stores the fixed-underlying C++ enum verbatim;
        // upstream checks its named values only in pushBlendMode, on Lua read.
        script_blend_mode(self.blend_mode)
    }

    pub(crate) fn from_fresh(
        paint: &crate::mechanical_port::source::shapes::paint::shape_paint::ShapePaint,
        stroke: Option<(f32, u32, u32)>,
    ) -> Self {
        use crate::mechanical_port::source::shapes::paint::{
            solid_color::SolidColor, stroke_cap::StrokeCap as RiveCap,
            stroke_join::StrokeJoin as RiveJoin,
        };
        let mut result = Self {
            style: RenderPaintStyle::Fill,
            color: 0xff000000,
            thickness: 1.0,
            join: StrokeJoin::Miter,
            cap: StrokeCap::Butt,
            feather: 0.0,
            blend_mode: paint.blend_mode_value() as u8,
        };
        if let Some((thickness, cap, join)) = stroke {
            result.style = RenderPaintStyle::Stroke;
            result.thickness = thickness;
            result.cap = RiveCap::from(cap).into();
            result.join = RiveJoin::from(join).into();
        }
        for child in paint.children() {
            if let Some(color) = child
                .with(|child| {
                    child
                        .as_any()
                        .downcast_ref::<SolidColor>()
                        .map(|color| color.base.color_value())
                })
                .flatten()
            {
                result.color = color as u32;
                break;
            }
        }
        if let Some(strength) = paint.feather().and_then(|feather| feather.with(|feather| feather.as_any().downcast_ref::<crate::mechanical_port::source::shapes::paint::feather::Feather>().map(|feather| feather.base.strength())).flatten()) {
            result.feather = strength;
        }
        result
    }
}

fn script_blend_mode(value: u8) -> BlendMode {
    match value {
        3 => BlendMode::SrcOver,
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
        _ => panic!("invalid ScriptedPaintData blend mode: {value}"),
    }
}

/// Ports the lookup/snapshot portion of C++ `src/lua/lua_artboards.cpp`'s
/// `ScriptedNode`, leaving userdata construction to the scripting backend.
#[derive(Debug, Clone)]
pub struct ScriptViewModel {
    properties: BTreeMap<String, ScriptViewModelProperty>,
    backing: NativeScriptViewModel,
    change_callbacks: ScriptViewModelChangeCallbacks,
}

#[path = "scripting/native_view_model.rs"]
mod native_view_model;
use native_view_model::NativeScriptViewModel;

type ScriptViewModelChangeCallback = Rc<dyn Fn()>;
type ScriptViewModelChangeCallbackEntry = (u64, ScriptViewModelChangeCallback);

#[derive(Clone, Default)]
struct ScriptViewModelChangeCallbacks {
    callbacks: Rc<RefCell<BTreeMap<Vec<usize>, Vec<ScriptViewModelChangeCallbackEntry>>>>,
    next_id: Rc<Cell<u64>>,
    suppressed: Rc<Cell<usize>>,
    pending: Rc<RefCell<Vec<Vec<usize>>>>,
}

impl std::fmt::Debug for ScriptViewModelChangeCallbacks {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("ScriptViewModelChangeCallbacks")
            .field("property_count", &self.callbacks.borrow().len())
            .field("suppressed", &self.suppressed.get())
            .field("pending", &self.pending.borrow().len())
            .finish()
    }
}

struct ScriptViewModelChangeSuppression<'a>(&'a Cell<usize>);

impl Drop for ScriptViewModelChangeSuppression<'_> {
    fn drop(&mut self) {
        self.0.set(self.0.get().saturating_sub(1));
    }
}

/// Lifetime token for one scripting property observer.
#[doc(hidden)]
pub struct ScriptViewModelChangeRegistration {
    callbacks: Weak<RefCell<BTreeMap<Vec<usize>, Vec<ScriptViewModelChangeCallbackEntry>>>>,
    path: Vec<usize>,
    id: u64,
}

impl Drop for ScriptViewModelChangeRegistration {
    fn drop(&mut self) {
        let Some(callbacks) = self.callbacks.upgrade() else {
            return;
        };
        let mut callbacks = callbacks.borrow_mut();
        let Some(entries) = callbacks.get_mut(&self.path) else {
            return;
        };
        entries.retain(|(id, _)| *id != self.id);
        if entries.is_empty() {
            callbacks.remove(&self.path);
        }
    }
}

/// An image selected from the runtime file's dense asset registry.
///
/// C++ exposes a retained `RenderImage` through Lua. The runtime-neutral seam
/// retains its registry identity instead; assigning the handle to an image
/// property resolves to the same decoded file asset during data binding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScriptImage {
    file_asset_index: u64,
    asset_global_id: u32,
}

/// File-scoped image identities exposed through `ScriptedContext:image()`.
///
/// Pinned C++ resolves this lookup through `scriptAsset()->file()->assets()`;
/// keeping the catalog separate from a DataContext preserves that ownership
/// when the script has no ViewModel.
#[derive(Debug, Clone, Default)]
pub struct ScriptImageAssets {
    by_name: BTreeMap<String, ScriptImage>,
}

impl ScriptImageAssets {
    pub fn named(&self, name: &str) -> Option<ScriptImage> {
        self.by_name.get(name).copied()
    }
}

pub fn script_image_assets(source: &impl ScriptFileSource) -> ScriptImageAssets {
    let file = source.script_file();
    let assets = file.with_file(|file| file.assets().to_vec());
    let mut by_name = BTreeMap::new();
    for (index, asset) in assets.iter().enumerate() {
        if let Some((name, id)) = asset
            .with_downcast::<crate::mechanical_port::source::assets::image_asset::ImageAsset, _>(
            |asset| (asset.base.name().to_owned(), asset.base.asset_id()),
        ) {
            by_name.entry(name).or_insert(ScriptImage {
                file_asset_index: index as u64,
                asset_global_id: id,
            });
        }
    }
    ScriptImageAssets { by_name }
}

/// A retained font selected from a view-model property.
///
/// File-backed values preserve their asset identity until the scripting VM
/// resolves them through its file-owned font registry. Live values retain the
/// exact byte owner installed by the host or another scripted property.
#[derive(Clone)]
pub struct ScriptFont {
    asset_global_id: Option<u32>,
    live_font_bytes: Option<Arc<[u8]>>,
    native_font: Option<crate::mechanical_port::source::text_engine::FontRef>,
}

impl std::fmt::Debug for ScriptFont {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ScriptFont")
            .field("asset_global_id", &self.asset_global_id)
            .field("live_font_bytes", &self.live_font_bytes)
            .field("native_font", &self.native_font.is_some())
            .finish()
    }
}

impl ScriptFont {
    #[doc(hidden)]
    pub fn with_native_font(
        mut self,
        font: crate::mechanical_port::source::text_engine::FontRef,
    ) -> Self {
        self.native_font = Some(font);
        self
    }
    #[doc(hidden)]
    pub fn with_resolved_font_bytes(mut self, bytes: Arc<[u8]>) -> Self {
        self.live_font_bytes = Some(bytes);
        self
    }

    #[doc(hidden)]
    pub fn asset_global_id(&self) -> Option<u32> {
        self.asset_global_id
    }

    #[doc(hidden)]
    pub fn live_font_bytes_arc(&self) -> Option<&Arc<[u8]>> {
        self.live_font_bytes.as_ref()
    }
}

impl ScriptImage {
    pub fn file_asset_index(self) -> u64 {
        self.file_asset_index
    }

    #[doc(hidden)]
    pub fn asset_global_id(self) -> u32 {
        self.asset_global_id
    }
}

impl ScriptViewModel {
    /// Construct the approved scripting facade around the translated occurrence.
    pub fn from_native(
        instance: crate::mechanical_port::source::core::CoreHandle,
        file: crate::mechanical_port::source::file::RuntimeFileHandle,
    ) -> Option<Self> {
        let model = instance
            .with(|instance| instance.as_view_model_instance()?.get_view_model())
            .flatten()?;
        Some(Self::from_native_definition(model, Some(instance), file))
    }

    pub(crate) fn from_native_definition(
        model: crate::mechanical_port::source::core::CoreHandle,
        instance: Option<crate::mechanical_port::source::core::CoreHandle>,
        file: crate::mechanical_port::source::file::RuntimeFileHandle,
    ) -> Self {
        Self::from_native_parts(Some(model), instance, file)
    }

    fn from_native_parts(
        model: Option<crate::mechanical_port::source::core::CoreHandle>,
        instance: Option<crate::mechanical_port::source::core::CoreHandle>,
        file: crate::mechanical_port::source::file::RuntimeFileHandle,
    ) -> Self {
        let native = NativeScriptViewModel {
            instance,
            model,
            file: native_view_model::NativeScriptFile::owning(file),
        };
        let properties = native.properties();
        Self {
            properties,
            backing: native,
            change_callbacks: ScriptViewModelChangeCallbacks::default(),
        }
    }

    fn native(&self) -> &NativeScriptViewModel {
        &self.backing
    }

    pub(crate) fn from_native_file_definition(
        model: crate::mechanical_port::source::core::CoreHandle,
        file: &crate::mechanical_port::source::file::RuntimeFileHandle,
    ) -> Self {
        let mut facade = Self::from_native_definition(model, None, file.clone());
        facade.backing.file = native_view_model::NativeScriptFile::definition(file);
        facade
    }

    pub fn native_instance(&self) -> Option<crate::mechanical_port::source::core::CoreHandle> {
        self.native().instance.clone()
    }

    pub fn native_model(&self) -> Option<crate::mechanical_port::source::core::CoreHandle> {
        self.native().model.clone()
    }

    /// Exact identity for VM userdata caches and owner-counted registrations.
    pub fn identity_key(&self) -> (u8, usize, usize, u64) {
        let native = self.native();
        let Some(handle) = native.instance.as_ref().or(native.model.as_ref()) else {
            return (0, 0, 0, 0);
        };
        let (arena, slot, generation) = handle.identity_key();
        return (
            if native.instance.is_some() { 1 } else { 2 },
            arena,
            slot,
            generation,
        );
    }

    pub fn advanced(&self) -> bool {
        self.native().advance()
    }

    /// Read the retained runtime's structural parent topology.
    pub fn has_parents(&self) -> bool {
        let native = self.native();
        return native.has_parents();
    }

    pub fn property(&self, name: &str) -> Option<ScriptViewModelProperty> {
        self.properties.get(name).copied()
    }

    pub fn properties(&self) -> &BTreeMap<String, ScriptViewModelProperty> {
        &self.properties
    }

    /// Retain a dirt observer for one named property on this scoped instance.
    ///
    /// Lua property delegates use this to observe host/state-machine writes
    /// before the end-of-frame reset consumes transient trigger values.
    pub fn property_dirt_sink(&self, name: &str) -> Option<RuntimeCellDirtSink> {
        let native = self.native();
        return native.property_dirt_sink(name);
    }

    /// Retain a property observer whose callback runs synchronously before
    /// the change is published to ordinary dirty queues.
    ///
    /// Script asset wrappers use this as the Rust counterpart of
    /// `ScriptedProperty::valueChanged()`: cached Lua registry references are
    /// released at the mutation boundary, even when several writes coalesce
    /// before the next scripted read.
    #[doc(hidden)]
    pub fn property_change_sink(
        &self,
        name: &str,
        callback: impl Fn() + 'static,
    ) -> Option<RuntimeCellDirtSink> {
        let sink = self.property_dirt_sink(name)?;
        sink.set_before_notify(Some(Rc::new(move |_| {
            callback();
            // The callback is the complete notification path for this
            // observer. Staying clean makes every later valueChanged edge
            // release the then-current cached wrapper immediately.
            false
        })));
        Some(sink)
    }

    /// Retain the C++ `ScriptedProperty` delegate edge. Host-side mutations
    /// can invoke the listener immediately; mutations made through the Lua
    /// userdata are deferred until that userdata borrow has been released.
    #[doc(hidden)]
    pub fn property_listener_sink(
        &self,
        name: &str,
        callback: impl Fn() + 'static,
    ) -> Option<RuntimeCellDirtSink> {
        let sink = self.property_dirt_sink(name)?;
        let suppressed = Rc::clone(&self.change_callbacks.suppressed);
        sink.set_before_notify(Some(Rc::new(move |_| {
            if suppressed.get() == 0 {
                callback();
                false
            } else {
                true
            }
        })));
        Some(sink)
    }

    /// Register a scripting callback that is invoked after a mutation through
    /// this facade has released the mutable runtime borrow.
    #[doc(hidden)]
    pub fn add_property_change_callback(
        &self,
        name: &str,
        callback: Rc<dyn Fn()>,
    ) -> Option<ScriptViewModelChangeRegistration> {
        let Some(path) = self.scoped_property_path(name) else {
            return None;
        };
        let id = self.change_callbacks.next_id.get();
        self.change_callbacks.next_id.set(id.wrapping_add(1));
        self.change_callbacks
            .callbacks
            .borrow_mut()
            .entry(path.clone())
            .or_default()
            .push((id, callback));
        Some(ScriptViewModelChangeRegistration {
            callbacks: Rc::downgrade(&self.change_callbacks.callbacks),
            path,
            id,
        })
    }

    /// Defer facade callbacks while a VM-owned userdata mutation is active;
    /// the VM binding flushes them after its userdata borrow is released.
    #[doc(hidden)]
    pub fn defer_property_change_callbacks<R>(&self, callback: impl FnOnce() -> R) -> R {
        let suppressed = &self.change_callbacks.suppressed;
        suppressed.set(suppressed.get().saturating_add(1));
        let _suppression = ScriptViewModelChangeSuppression(suppressed);
        callback()
    }

    fn notify_property_change(&self, path: &[usize]) {
        if self.change_callbacks.suppressed.get() != 0 {
            self.change_callbacks
                .pending
                .borrow_mut()
                .push(path.to_vec());
            return;
        }
        self.dispatch_property_change(path);
    }

    fn dispatch_property_change(&self, path: &[usize]) {
        let callbacks = self
            .change_callbacks
            .callbacks
            .borrow()
            .get(path)
            .cloned()
            .unwrap_or_default();
        for (_, callback) in callbacks {
            callback();
        }
    }

    /// Dispatch mutations deferred until a VM-owned userdata borrow ended.
    /// Paths remain FIFO and retain duplicates, matching delegate invocation
    /// order for repeated writes.
    #[doc(hidden)]
    pub fn flush_property_change_callbacks(&self) {
        if self.change_callbacks.suppressed.get() != 0 {
            return;
        }
        let pending = std::mem::take(&mut *self.change_callbacks.pending.borrow_mut());
        for path in pending {
            self.dispatch_property_change(&path);
        }
    }

    fn finish_property_change(&self, path: &[usize], changed: bool) -> bool {
        if changed {
            self.notify_property_change(path);
        }
        changed
    }

    pub fn named_instance(&self, name: Option<&str>) -> Option<Self> {
        let native = self.native();
        return native.named_instance(name);
    }

    pub fn number(&self, name: &str) -> Option<f32> {
        let native = self.native();
        return native.number(name);
    }

    pub fn set_number(&self, name: &str, value: f32) -> bool {
        let native = self.native();
        let changed = native.set_number(name, value);
        return self
            .finish_property_change(&native.property_path(name).unwrap_or_default(), changed);
    }

    pub fn color(&self, name: &str) -> Option<u32> {
        let native = self.native();
        return native.color(name);
    }

    pub fn set_color(&self, name: &str, value: u32) -> bool {
        let native = self.native();
        let changed = native.set_color(name, value);
        return self
            .finish_property_change(&native.property_path(name).unwrap_or_default(), changed);
    }

    pub fn string(&self, name: &str) -> Option<String> {
        let native = self.native();
        return native.string(name);
    }

    pub fn set_string(&self, name: &str, value: &str) -> bool {
        let native = self.native();
        let changed = native.set_string(name, value);
        return self
            .finish_property_change(&native.property_path(name).unwrap_or_default(), changed);
    }

    pub fn boolean(&self, name: &str) -> Option<bool> {
        let native = self.native();
        return native.boolean(name);
    }

    pub fn enum_value(&self, name: &str) -> Option<String> {
        let native = self.native();
        return native.enum_value(name);
    }

    pub fn enum_values(&self, name: &str) -> Option<Vec<String>> {
        let native = self.native();
        return native.enum_values(name);
    }

    pub fn set_enum_value(&self, name: &str, value: &str) -> bool {
        let native = self.native();
        let changed = native.set_enum_value(name, value);
        return self
            .finish_property_change(&native.property_path(name).unwrap_or_default(), changed);
    }

    pub fn image(&self, name: &str) -> Option<ScriptImage> {
        let native = self.native();
        return native.image(name);
    }

    pub fn font(&self, name: &str) -> Option<ScriptFont> {
        let native = self.native();
        return native.font(name);
    }

    pub fn set_font(&self, name: &str, font: Option<&ScriptFont>) -> bool {
        let native = self.native();
        let changed = native.set_font(name, font);
        return self
            .finish_property_change(&native.property_path(name).unwrap_or_default(), changed);
    }

    #[doc(hidden)]
    pub fn set_font_bytes(&self, name: &str, font_bytes: Option<Arc<[u8]>>) -> bool {
        let native = self.native();
        let changed = native.set_font_bytes(name, font_bytes);
        return self
            .finish_property_change(&native.property_path(name).unwrap_or_default(), changed);
    }

    pub fn render_image(&self, name: &str) -> Option<Rc<dyn nuxie_render_api::RenderImage>> {
        let native = self.native();
        return native.render_image(name);
    }

    pub fn image_asset_named(&self, name: &str) -> Option<ScriptImage> {
        let native = self.native();
        return native.image_asset_named(name);
    }

    pub fn set_image(&self, name: &str, image: Option<ScriptImage>) -> bool {
        let native = self.native();
        let changed = native.set_image(name, image);
        return self
            .finish_property_change(&native.property_path(name).unwrap_or_default(), changed);
    }

    pub fn set_render_image(
        &self,
        name: &str,
        image: Option<Rc<dyn nuxie_render_api::RenderImage>>,
    ) -> bool {
        let native = self.native();
        let changed = native.set_render_image(name, image);
        return self
            .finish_property_change(&native.property_path(name).unwrap_or_default(), changed);
    }

    pub fn blob_asset(&self, name: &str) -> Option<Arc<RuntimeBlobAsset>> {
        let native = self.native();
        return native.blob_asset(name);
    }

    pub fn blob(&self, name: &str) -> Option<Arc<[u8]>> {
        self.blob_asset(name).map(|asset| asset.bytes_arc())
    }

    pub fn set_blob(&self, name: &str, bytes: Option<Arc<[u8]>>) -> bool {
        let native = self.native();
        let changed = native.set_blob(name, bytes);
        return self
            .finish_property_change(&native.property_path(name).unwrap_or_default(), changed);
    }

    pub fn set_blob_asset(&self, name: &str, asset: Option<Arc<RuntimeBlobAsset>>) -> bool {
        let native = self.native();
        let changed = native.set_blob_asset(name, asset);
        return self
            .finish_property_change(&native.property_path(name).unwrap_or_default(), changed);
    }

    /// Mirrors C++ `ScriptedViewModel::pushIndex` for component-list rows.
    pub fn component_list_item_index(&self) -> Option<u64> {
        let native = self.native();
        return native.component_list_item_index();
    }

    pub fn set_boolean(&self, name: &str, value: bool) -> bool {
        let native = self.native();
        let changed = native.set_boolean(name, value);
        return self
            .finish_property_change(&native.property_path(name).unwrap_or_default(), changed);
    }

    pub fn trigger(&self, name: &str) -> Option<u64> {
        let native = self.native();
        return native.trigger(name);
    }

    /// Fire a trigger the same way C++ `ViewModelInstanceTrigger::trigger()`
    /// does: increment the backing counter and leave consumption/reset to the
    /// end-of-frame `advanced()` pass.
    pub fn fire_trigger(&self, name: &str) -> bool {
        let native = self.native();
        let changed = native.fire_trigger(name);
        return self
            .finish_property_change(&native.property_path(name).unwrap_or_default(), changed);
    }

    /// Consume transient values at the end of a script host frame.
    ///
    /// This mirrors C++ `ViewModelInstance::advanced()`: triggers are reset
    /// without invoking script listeners, embedded view models recurse, and
    /// shared list instances recurse exactly once even if the graph cycles.
    pub fn advance_script_frame(&self) -> bool {
        let native = self.native();
        return native.advance();
    }

    /// Advance a shared owned instance without requiring its schema wrapper.
    /// Scripting backends use this for owner-counted registrations that retain
    /// precisely the backing instance, matching C++ `rcp<ViewModelInstance>`.

    /// Advance several owned roots with one identity set shared across their
    /// complete embedded/list graphs. This is the frame-context entry point:
    /// registry relationships can name an instance that is also reachable
    /// structurally, and it must still be consumed only once per frame.

    pub fn view_model(&self, name: &str) -> Option<Self> {
        let native = self.native();
        return native.view_model(name, false);
    }

    /// Return only the currently linked nested ViewModel occurrence.
    ///
    /// The schema wrapper returned by [`Self::view_model`] remains available
    /// while an authored ViewModel property is null. This accessor only returns
    /// a linked occurrence; Lua's non-nil, possibly null-reference value wrapper
    /// is constructed by [`Self::referenced_view_model_value`].
    pub fn active_view_model(&self, name: &str) -> Option<Self> {
        let native = self.native();
        return native.view_model(name, true);
    }

    /// The exact Lua property value: current referenced type/instance, or the
    /// property's creation-time type paired with a null instance. The fallback
    /// type can itself be null; it is never inferred from the owner's schema.
    pub fn referenced_view_model_value(
        &self,
        name: &str,
        creation_time_model: Option<crate::mechanical_port::source::core::CoreHandle>,
    ) -> Self {
        self.native()
            .referenced_view_model_value(name, creation_time_model)
    }

    /// Port of `ScriptedPropertyViewModel::setValue`: replace the actual
    /// retained child occurrence and synchronously notify/relink its parent.
    pub fn set_view_model(&self, name: &str, value: &ScriptViewModel) -> bool {
        let native = self.native();
        let changed = native.set_view_model(name, value);
        return self
            .finish_property_change(&native.property_path(name).unwrap_or_default(), changed);
    }

    pub fn list_len(&self, name: &str) -> Option<usize> {
        let native = self.native();
        return native.list_len(name);
    }

    pub fn list_item(&self, name: &str, index: usize) -> Option<Self> {
        let native = self.native();
        return native.list_item(name, index);
    }

    pub fn push_list_item(&self, name: &str, item: &ScriptViewModel) -> bool {
        let native = self.native();
        let changed = native.insert_list_item(name, None, item);
        return self
            .finish_property_change(&native.property_path(name).unwrap_or_default(), changed);
    }

    pub fn insert_list_item(&self, name: &str, index: usize, item: &ScriptViewModel) -> bool {
        let native = self.native();
        let changed = native.insert_list_item(name, Some(index), item);
        return self
            .finish_property_change(&native.property_path(name).unwrap_or_default(), changed);
    }

    pub fn pop_list_item(&self, name: &str) -> Option<Self> {
        let native = self.native();
        return native.pop_list_item(name, false);
    }

    pub fn shift_list_item(&self, name: &str) -> Option<Self> {
        let native = self.native();
        return native.pop_list_item(name, true);
    }

    pub fn swap_list_items(&self, name: &str, first: usize, second: usize) -> bool {
        let native = self.native();
        let changed = native.swap_list_items(name, first, second);
        return self
            .finish_property_change(&native.property_path(name).unwrap_or_default(), changed);
    }

    pub fn clear_list_items(&self, name: &str) -> bool {
        let native = self.native();
        let changed = native.clear_list_items(name);
        return self
            .finish_property_change(&native.property_path(name).unwrap_or_default(), changed);
    }

    pub fn remove_list_item_at(&self, name: &str, index: usize) -> bool {
        let native = self.native();
        let changed = native.remove_list_item_at(name, index);
        return self
            .finish_property_change(&native.property_path(name).unwrap_or_default(), changed);
    }

    pub fn remove_list_item(&self, name: &str, item: &ScriptViewModel, remove_all: bool) -> bool {
        let native = self.native();
        let changed = native.remove_list_item(name, item, remove_all);
        return self
            .finish_property_change(&native.property_path(name).unwrap_or_default(), changed);
    }

    fn scoped_property_path(&self, name: &str) -> Option<Vec<usize>> {
        let native = self.native();
        return native.property_path(name);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptViewModelProperty {
    Number,
    Color,
    String,
    Boolean,
    Enum,
    Trigger,
    Image,
    Blob,
    Font,
    List,
    ViewModel,
    SymbolListIndex,
}

/// An already-imported native File. Scripting never reimports descriptors or
/// creates an implicit renderer factory.
pub trait ScriptFileSource {
    fn script_file(&self) -> crate::mechanical_port::source::file::RuntimeFileHandle;
}
impl ScriptFileSource for crate::mechanical_port::source::file::RuntimeFileHandle {
    fn script_file(&self) -> crate::mechanical_port::source::file::RuntimeFileHandle {
        self.clone()
    }
}
impl<T: ScriptFileSource> ScriptFileSource for Rc<T> {
    fn script_file(&self) -> crate::mechanical_port::source::file::RuntimeFileHandle {
        (**self).script_file()
    }
}
impl<T: ScriptFileSource> ScriptFileSource for Arc<T> {
    fn script_file(&self) -> crate::mechanical_port::source::file::RuntimeFileHandle {
        (**self).script_file()
    }
}

pub fn script_view_models(source: &impl ScriptFileSource) -> BTreeMap<String, ScriptViewModel> {
    let file = source.script_file();
    let models = file.with_file(|file| {
        (0..file.view_model_count())
            .filter_map(|i| file.view_model(i))
            .collect::<Vec<_>>()
    });
    models
        .into_iter()
        .filter_map(|model| {
            let name = model
                .with(|owner| {
                    owner
                        .as_view_model()
                        .map(|model| model.base.name().to_owned())
                })
                .flatten()?;
            Some((
                name,
                ScriptViewModel::from_native_definition(model, None, file.clone()),
            ))
        })
        .collect()
}

impl ScriptValue {
    pub fn as_number(&self) -> Option<f64> {
        match self {
            ScriptValue::Number(value) => Some(*value),
            _ => None,
        }
    }
}

pub trait ScriptHost {
    fn mark_script_update(&mut self) {}

    /// Whether an ordinary protected script-callback failure must abort the
    /// enclosing host transaction. The baseline runtime preserves pinned C++
    /// behavior by default; result-based transactional adapters opt in so
    /// effects emitted earlier in the failed callback can be rolled back.
    fn requires_atomic_script_callbacks(&self) -> bool {
        false
    }
}

#[derive(Debug, Default)]
pub struct NoopScriptHost;

impl ScriptHost for NoopScriptHost {}

type NativeLinearAnimation =
    crate::mechanical_port::source::animation::linear_animation_instance::LinearAnimationInstance;

/// Runtime-owned linear-animation handle exposed to scripts.
#[derive(Clone)]
pub struct ScriptAnimation {
    instance: Rc<RefCell<NativeLinearAnimation>>,
}
impl fmt::Debug for ScriptAnimation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ScriptAnimation")
            .field("duration", &self.duration())
            .finish_non_exhaustive()
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptAnimationTime {
    Seconds,
    Frames,
    Percentage,
}
impl ScriptAnimation {
    pub fn duration(&self) -> f32 {
        let instance = self.instance.borrow();
        instance.duration() as f32 / instance.fps() as f32
    }
}

/// Runtime-owned artboard userdata exposed to scripts.
pub trait ScriptArtboard {
    /// Retain this occurrence for a callback, without cloning the artboard.
    /// The Lua wrapper must release its facade access before invoking user code.
    fn retained_handle(&self) -> Box<dyn ScriptArtboard>;
    fn width(&self) -> f32;
    fn height(&self) -> f32;
    fn frame_origin(&self) -> bool;
    fn set_width(&mut self, width: f32);
    fn set_height(&mut self, height: f32);
    fn set_frame_origin(&mut self, frame_origin: bool);

    fn bounds(&self) -> nuxie_render_api::Aabb;

    fn add_to_path(
        &mut self,
        path: &mut RawPath,
        transform: Option<nuxie_render_api::Mat2D>,
    ) -> Result<(), ScriptError>;

    fn dispatch_input(
        &mut self,
        method: ScriptMethod,
        invocation: &ScriptListenerInvocation,
    ) -> Result<u32, ScriptError>;

    fn data(&self) -> Option<ScriptViewModel> {
        None
    }

    fn instance(
        &self,
        view_model: Option<ScriptViewModel>,
    ) -> Result<Box<dyn ScriptArtboard>, ScriptError>;

    /// Construct an instance while preserving C++'s File-owned renderer
    /// factory boundary. Backends without retained renderer members may use
    /// the ordinary factory-free construction path.
    fn instance_with_factory(
        &self,
        view_model: Option<ScriptViewModel>,
        _factory: &mut dyn RenderFactory,
    ) -> Result<Box<dyn ScriptArtboard>, ScriptError> {
        self.instance(view_model)
    }

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

/// Deferred Artboard construction produced by resolver validation.
///
/// The prepared value is a non-bypassable type-state fence, but it retains
/// constructor authority rather than a preconstructed occurrence. Rust-only
/// semantic failures therefore surface at the input's authored phase-two
/// position instead of being moved into the validation loop or hidden by an
/// `expect`.
pub trait PreparedScriptArtboard: fmt::Debug {
    fn construct(self: Box<Self>) -> Result<Box<dyn ScriptArtboard>, ScriptError>;
}

/// Runtime-owned handle for one scripted object instance.
pub trait ScriptInstance {
    /// Settle backend-owned asynchronous work on the script VM's owning
    /// thread. Root scene advance calls this immediately after the shared
    /// WorkPool completion poll, including for parked/event-only scripts.
    fn poll_async_work(&mut self) -> Result<bool, ScriptError> {
        Ok(false)
    }

    /// Advance clone-owned DataBind converter state parked behind a scripted
    /// object. ScriptedInterpolator uses this seam because its lazy clones are
    /// owned by LinearAnimationInstance rather than Artboard's ordinary bind
    /// collection (`linear_animation_instance.cpp:109-172`).
    fn advance_scripted_data_binds(
        &mut self,
        _elapsed_seconds: f32,
        _host: &mut dyn ScriptHost,
    ) -> bool {
        false
    }

    fn set_context_view_model(
        &mut self,
        _view_model: Option<ScriptViewModel>,
    ) -> Result<(), ScriptError> {
        Ok(())
    }

    fn set_context_view_model_chain(
        &mut self,
        view_model: Option<ScriptViewModel>,
        _parents: Vec<Option<ScriptViewModel>>,
    ) -> Result<(), ScriptError> {
        self.set_context_view_model(view_model)
    }

    /// Clear a context before its first host bind without declaring the
    /// authored absence to be resolved. A later `init` read of view-model
    /// data must keep the occurrence cold so the host can recreate it after
    /// the real data context arrives.
    fn clear_unresolved_context_view_model(&mut self) -> Result<(), ScriptError> {
        self.set_context_view_model(None)
    }

    fn has_method(&self, method: ScriptMethod) -> Result<bool, ScriptError>;

    fn call_method(
        &mut self,
        method: ScriptMethod,
        args: &[ScriptValue],
        host: &mut dyn ScriptHost,
    ) -> Result<ScriptValue, ScriptError>;

    /// Resolve an optional callback once and invoke that exact value when it
    /// is callable. Missing and non-function fields are a no-op.
    fn call_optional_method(
        &mut self,
        method: ScriptMethod,
        args: &[ScriptValue],
        host: &mut dyn ScriptHost,
    ) -> Result<ScriptOptionalMethodResult, ScriptError> {
        if !self.has_method(method)? {
            return Ok(ScriptOptionalMethodResult::Missing);
        }
        self.call_method(method, args, host)
            .map(ScriptOptionalMethodResult::Returned)
    }

    /// Resolve an interpolator callback once and invoke that exact function.
    ///
    /// The default adapter uses the VM-neutral value bridge. Backends override
    /// this when they can preserve the source VM's numeric coercion rules.
    fn call_interpolator(
        &mut self,
        method: ScriptInterpolatorMethod,
        args: &[f32],
        host: &mut dyn ScriptHost,
    ) -> Result<ScriptOptionalNumberResult, ScriptError> {
        let script_method = match method {
            ScriptInterpolatorMethod::Transform => ScriptMethod::Transform,
            ScriptInterpolatorMethod::TransformValue => ScriptMethod::TransformValue,
        };
        let args = args
            .iter()
            .map(|value| ScriptValue::Number(f64::from(*value)))
            .collect::<Vec<_>>();
        Ok(
            match self.call_optional_method(script_method, &args, host)? {
                ScriptOptionalMethodResult::Missing => ScriptOptionalNumberResult::Missing,
                ScriptOptionalMethodResult::Returned(ScriptValue::Number(value)) => {
                    ScriptOptionalNumberResult::Returned(value as f32)
                }
                ScriptOptionalMethodResult::Returned(_) => {
                    ScriptOptionalNumberResult::Returned(0.0)
                }
            },
        )
    }

    /// Run `advance(self, seconds)` and apply the VM's native truthiness rules.
    ///
    /// The generic [`ScriptValue`] bridge intentionally exposes only the
    /// runtime value kinds used by authored properties. Lua `advance` may
    /// nevertheless return any value, and pinned C++ treats every value except
    /// `nil` and `false` as true (`scripted_data_converter.cpp:285-304`).
    /// Backends with additional value kinds override this boundary rather than
    /// rejecting a truthy table, function, userdata, or thread while trying to
    /// project it into [`ScriptValue`].
    fn call_advance_truthy(
        &mut self,
        elapsed_seconds: f32,
        host: &mut dyn ScriptHost,
    ) -> Result<bool, ScriptError> {
        let value = self.call_method(
            ScriptMethod::Advance,
            &[ScriptValue::Number(f64::from(elapsed_seconds))],
            host,
        )?;
        Ok(!matches!(
            value,
            ScriptValue::Nil | ScriptValue::Bool(false)
        ))
    }

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

    /// Resolve and invoke the preferred scripted-listener callback as one
    /// backend operation.
    ///
    /// Pinned C++ reads `performAction` once and calls that exact Lua value;
    /// only when it is not a function does it read `perform` once
    /// (`scripted_listener_action.cpp:49-100`). Backends must not split this
    /// into a presence probe followed by a second lookup because a metatable
    /// may return a different value on every access.
    fn call_preferred_listener_action(
        &mut self,
        _invocation: &ScriptListenerInvocation,
        _host: &mut dyn ScriptHost,
    ) -> Result<bool, ScriptError> {
        Ok(false)
    }

    /// Dispatch one keyboard, committed-text, or gamepad invocation directly
    /// to a scripted drawable.
    ///
    /// This is distinct from `ScriptedListenerAction::performAction`: pinned
    /// C++ `KeyboardListenerGroup` and `GamepadListenerGroup` call the
    /// drawable's `keyboardEvent`, `textEvent`, or gamepad method before any
    /// authored listener branch. The outcome distinguishes a missing/currently
    /// non-function field from an invoked callback so the owner wakes only at
    /// the C++ boundary.
    fn call_scripted_drawable_input(
        &mut self,
        _invocation: &ScriptListenerInvocation,
        _host: &mut dyn ScriptHost,
    ) -> Result<ScriptedDrawableInputResult, ScriptError> {
        Ok(ScriptedDrawableInputResult::default())
    }

    /// Invoke one pointer method selected by a scripted-drawable hit owner.
    ///
    /// The owner has already transformed the world position into the
    /// drawable's local coordinates. Concrete VMs create a fresh
    /// `PointerEvent` with the pinned constructor defaults and return the
    /// tri-state mutation after the callback completes.
    fn call_scripted_drawable_pointer(
        &mut self,
        _method: ScriptMethod,
        _pointer_id: i32,
        _local_x: f32,
        _local_y: f32,
        _host: &mut dyn ScriptHost,
    ) -> Result<ScriptedDrawablePointerResult, ScriptError> {
        Ok(ScriptedDrawablePointerResult::default())
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

    /// Byte-preserving companion used by authored `CoreString` input names.
    fn call_input_trigger_core(
        &mut self,
        name: &ScriptCoreString,
        host: &mut dyn ScriptHost,
    ) -> Result<(), ScriptError> {
        let name = std::str::from_utf8(name.as_c_str_bytes()).map_err(|_| {
            ScriptError::new("script backend does not support non-UTF-8 authored input names")
        })?;
        self.call_input_trigger(name, host)
    }

    /// Run an implemented user `init(self, context)` and apply Lua truthiness
    /// without requiring every backend value kind to cross the VM-neutral
    /// [`ScriptValue`] seam.
    ///
    /// Pinned C++ initialization is a scripting-VM operation and does not
    /// require a callback-local renderer factory. Renderer-capable VMs retain
    /// the render context installed by their file owner before import;
    /// [`ScriptInstance::call_init_with_factory`] remains an identity-checking
    /// integration adapter.
    fn call_init(&mut self, host: &mut dyn ScriptHost) -> Result<bool, ScriptError> {
        let value = self.call_method(ScriptMethod::Init, &[], host)?;
        Ok(!matches!(
            value,
            ScriptValue::Nil | ScriptValue::Bool(false)
        ))
    }

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
    fn user_init_pending(&mut self) -> Result<bool, ScriptError> {
        Ok(false)
    }

    /// Whether this occurrence still owns the concrete script table/context
    /// that C++ stores in `ScriptedObject::m_self`.
    ///
    /// A missing hydration prerequisite leaves `m_self` alive even though
    /// user init is still pending. By contrast, init false/error or a missing
    /// requested context value disposes `m_self` until the occurrence is
    /// recreated (`scripted_object.cpp:277-303,399-435`).
    fn script_lifetime_valid(&self) -> bool {
        true
    }

    /// Discard the current scripted-object lifetime before the next input
    /// hydration. VM backends use this when cold-init prerequisites are not
    /// available, matching C++ `ensureScriptInitialized` retry semantics.
    fn invalidate_for_init_retry(&mut self) {}

    /// Recreate a lifetime invalidated by a failed/deferred init. Hosts call
    /// this before hydrating inputs so a new script table observes the bound
    /// context and receives the complete input set.
    fn prepare_init_retry(&mut self) -> Result<(), ScriptError> {
        Ok(())
    }

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

    /// Resolve and invoke one optional converter method in one backend
    /// operation.
    ///
    /// `None` means the field was missing or not a function. That is distinct
    /// from a called function returning an unsupported value or failing:
    /// pinned C++ leaves its conversion cache untouched only in the missing
    /// method case (`scripted_data_converter.cpp:89-147`).
    fn call_data_converter_if_present(
        &mut self,
        method: ScriptDataConverterMethod,
        value: ScriptValue,
    ) -> Result<Option<ScriptValue>, ScriptError> {
        self.call_data_converter(method, value).map(Some)
    }

    /// Resolve an optional converter method once and invoke that exact
    /// function when the input is representable.
    ///
    /// Pinned C++ performs one `lua_getfield` before attempting to push the
    /// input (`scripted_data_converter.cpp:96-147`). Keeping the lookup and
    /// call in one backend operation prevents a dynamic `__index` from
    /// returning different functions (or a function and then nil) across two
    /// lookups.
    fn call_optional_data_converter(
        &mut self,
        method: ScriptDataConverterMethod,
        value: Option<ScriptValue>,
    ) -> Result<ScriptDataConverterOptionalCall, ScriptError> {
        let Some(value) = value else {
            return if self.has_data_converter_method(method)? {
                Ok(ScriptDataConverterOptionalCall::UnsupportedInput)
            } else {
                Ok(ScriptDataConverterOptionalCall::Missing)
            };
        };
        Ok(match self.call_data_converter_if_present(method, value)? {
            Some(value) => ScriptDataConverterOptionalCall::Returned(value),
            None => ScriptDataConverterOptionalCall::Missing,
        })
    }

    fn has_data_converter_method(
        &self,
        _method: ScriptDataConverterMethod,
    ) -> Result<bool, ScriptError> {
        Ok(true)
    }

    fn get_input(&self, name: &str) -> Result<ScriptValue, ScriptError>;

    fn set_input(&mut self, name: &str, value: ScriptValue) -> Result<(), ScriptError>;

    /// Byte-preserving companion used by authored `CoreString` input names.
    fn set_input_core(
        &mut self,
        name: &ScriptCoreString,
        value: ScriptValue,
    ) -> Result<(), ScriptError> {
        let name = std::str::from_utf8(name.as_c_str_bytes()).map_err(|_| {
            ScriptError::new("script backend does not support non-UTF-8 authored input names")
        })?;
        self.set_input(name, value)
    }

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

    fn set_artboard_input_core(
        &mut self,
        name: &ScriptCoreString,
        artboard: Box<dyn ScriptArtboard>,
    ) -> Result<(), ScriptError> {
        let name = std::str::from_utf8(name.as_c_str_bytes()).map_err(|_| {
            ScriptError::new("script backend does not support non-UTF-8 authored input names")
        })?;
        self.set_artboard_input(name, artboard)
    }

    /// Apply one validated Artboard input only when the concrete occurrence
    /// still owns the state/table/script-asset context used by the pinned
    /// setter. Construction remains at this authored phase-two position but is
    /// deferred until after that backend-owned guard.
    fn set_prepared_artboard_input_core(
        &mut self,
        name: &ScriptCoreString,
        recipe: Box<dyn PreparedScriptArtboard>,
    ) -> Result<(), ScriptError> {
        if !self.script_artboard_input_context_live() {
            return Ok(());
        }
        self.set_artboard_input_core(name, recipe.construct()?)
    }

    /// Backend-owned equivalent of pinned `state() != nullptr`, `m_self != 0`,
    /// and `scriptAsset() != nullptr` for Artboard setters. Generic test and
    /// adaptation backends inherit the occurrence-lifetime check; the Luau
    /// backend additionally verifies its retained protocol generator.
    fn script_artboard_input_context_live(&self) -> bool {
        self.script_lifetime_valid()
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

    fn set_view_model_input_core(
        &mut self,
        name: &ScriptCoreString,
        view_model: ScriptViewModel,
    ) -> Result<(), ScriptError> {
        let name = std::str::from_utf8(name.as_c_str_bytes()).map_err(|_| {
            ScriptError::new("script backend does not support non-UTF-8 authored input names")
        })?;
        self.set_view_model_input(name, view_model)
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

/// Opaque retained generator owned by one ScriptAsset, not a module-name cache.
#[derive(Clone)]
pub struct RuntimeScriptProgram(Rc<dyn std::any::Any>);
impl RuntimeScriptProgram {
    pub fn from_backend<T: 'static>(program: T) -> Self {
        Self(Rc::new(program))
    }
    pub fn backend<T: 'static>(&self) -> Option<&T> {
        self.0.downcast_ref()
    }
}

pub struct ScriptAssetRegistration<'a> {
    pub name: &'a str,
    /// Already validated and envelope-stripped by ScriptAsset::decode.
    pub bytecode: &'a [u8],
    pub is_protocol: bool,
    pub missing_dependencies: Vec<String>,
}

#[derive(Default)]
pub struct ScriptAssetRegistrationResult {
    pub completed: bool,
    pub program: Option<RuntimeScriptProgram>,
    pub missing_dependencies: Vec<String>,
    pub error: Option<ScriptError>,
}

/// Product-owned script program family layered over the translated runtime's
/// opaque `ScriptingVm` registration and instantiation seam.
///
/// Adapters may claim only payloads they recognize. Returning `None` delegates
/// the exact asset to the ordinary scripting backend unchanged.
pub trait ScriptProgramAdapter: std::fmt::Debug {
    fn register_script_asset(
        &self,
        registration: &ScriptAssetRegistration<'_>,
    ) -> Option<ScriptAssetRegistrationResult>;

    fn instantiate_program(
        &self,
        program: &RuntimeScriptProgram,
        context_present: bool,
        view_model: Option<ScriptViewModel>,
        parent_view_models: Vec<Option<ScriptViewModel>>,
        host: &mut dyn ScriptHost,
    ) -> Option<Result<Box<dyn ScriptInstance>, ScriptError>>;
}

/// Runtime-owned VM seam implemented by concrete scripting backends.

impl<T: ScriptingVm + ?Sized> ScriptingVm for Rc<T> {
    fn route_to_import_factory(&self, factory: &mut dyn RenderFactory) {
        (**self).route_to_import_factory(factory)
    }
    fn install_native_file_assets(
        &self,
        file: crate::mechanical_port::source::file::RuntimeFileWeakHandle,
    ) -> Result<(), ScriptError> {
        (**self).install_native_file_assets(file)
    }
    fn initializes_data_global_externally(&self) -> bool {
        (**self).initializes_data_global_externally()
    }
    fn initialize_data_global(
        &self,
        models: BTreeMap<String, ScriptViewModel>,
    ) -> Result<(), ScriptError> {
        (**self).initialize_data_global(models)
    }
    fn install_render_factory(&self, factory: &mut dyn RenderFactory) -> Result<(), ScriptError> {
        (**self).install_render_factory(factory)
    }
    fn install_rive_globals(&self) -> Result<(), ScriptError> {
        (**self).install_rive_globals()
    }
    fn register_module(&self, name: &str, payload: &[u8]) -> Result<(), ScriptError> {
        (**self).register_module(name, payload)
    }
    fn register_script_assets(
        &self,
        scripts: &[ScriptAssetRegistration<'_>],
    ) -> Vec<ScriptAssetRegistrationResult> {
        (**self).register_script_assets(scripts)
    }
    fn instantiate_program(
        &self,
        program: &RuntimeScriptProgram,
        present: bool,
        model: Option<ScriptViewModel>,
        parents: Vec<Option<ScriptViewModel>>,
        host: &mut dyn ScriptHost,
    ) -> Result<Box<dyn ScriptInstance>, ScriptError> {
        (**self).instantiate_program(program, present, model, parents, host)
    }
    fn instantiate_script(
        &self,
        name: &str,
        payload: &[u8],
        host: &mut dyn ScriptHost,
    ) -> Result<Box<dyn ScriptInstance>, ScriptError> {
        (**self).instantiate_script(name, payload, host)
    }
    fn poll_async_work(&self) -> Result<bool, ScriptError> {
        (**self).poll_async_work()
    }
    fn advance_detached_view_models(&self) -> bool {
        (**self).advance_detached_view_models()
    }
    fn perform_registration(&self, modules: &[ScriptModule<'_>]) -> Vec<ScriptModuleFailure> {
        (**self).perform_registration(modules)
    }
}

pub trait ScriptingVm {
    fn route_to_import_factory(&self, _factory: &mut dyn RenderFactory) {}
    /// Install the importing file's asset catalog without making a File → VM
    /// → catalog → File ownership cycle. Executing chunks comes afterward.
    fn install_native_file_assets(
        &self,
        file: crate::mechanical_port::source::file::RuntimeFileWeakHandle,
    ) -> Result<(), ScriptError>;
    fn initializes_data_global_externally(&self) -> bool {
        false
    }
    fn initialize_data_global(
        &self,
        models: BTreeMap<String, ScriptViewModel>,
    ) -> Result<(), ScriptError>;
    /// Retain the one renderer factory identity before any imported script is
    /// registered or executed.
    fn install_render_factory(&self, factory: &mut dyn RenderFactory) -> Result<(), ScriptError>;

    fn install_rive_globals(&self) -> Result<(), ScriptError>;

    fn register_module(&self, name: &str, payload: &[u8]) -> Result<(), ScriptError>;

    fn register_script_assets(
        &self,
        scripts: &[ScriptAssetRegistration<'_>],
    ) -> Vec<ScriptAssetRegistrationResult>;

    fn instantiate_program(
        &self,
        program: &RuntimeScriptProgram,
        context_present: bool,
        view_model: Option<ScriptViewModel>,
        parent_view_models: Vec<Option<ScriptViewModel>>,
        host: &mut dyn ScriptHost,
    ) -> Result<Box<dyn ScriptInstance>, ScriptError>;

    fn instantiate_script(
        &self,
        name: &str,
        payload: &[u8],
        host: &mut dyn ScriptHost,
    ) -> Result<Box<dyn ScriptInstance>, ScriptError>;

    /// Settle backend-owned async completions after the shared work-pool poll
    /// and before any root-frame script callbacks run.
    fn poll_async_work(&self) -> Result<bool, ScriptError> {
        Ok(false)
    }

    /// Consume detached script-created view-model instances once at the end
    /// of a root host frame. Child/script-driven artboard advances must not
    /// call this hook.
    fn advance_detached_view_models(&self) -> bool {
        false
    }

    fn perform_registration(&self, modules: &[ScriptModule<'_>]) -> Vec<ScriptModuleFailure> {
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
