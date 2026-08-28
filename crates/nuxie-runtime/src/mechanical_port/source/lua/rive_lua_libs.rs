use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
    time::{Duration, Instant},
};

#[cfg(feature = "tools")]
pub use super::renderer::lua_blob::push_blob as lua_push_blob;
pub use super::renderer::lua_gpu::{
    lua_gpu_find_shader_asset, lua_gpu_load_shader_by_name, lua_gpu_push_shader_by_name,
    rive_lua_close_orphan_render_pass,
};
pub use super::{
    lua_listener_invocation::{
        push_pointer_arg_for_perform as rive_lua_push_pointer_arg_for_perform,
        push_scripted_invocation as rive_lua_push_scripted_invocation,
        register_listener_invocation_types as rive_lua_register_listener_invocation_types,
    },
    lua_scripted_context::push_gpu_features as lua_push_gpu_features,
    renderer::lua_paint::lua_to_blend_mode,
    scripting_vm::ScriptingVM,
};

pub use crate::mechanical_port::source::{
    animation::{
        linear_animation_instance::LinearAnimationInstance,
        listener_invocation::ListenerInvocation, state_machine_instance::StateMachineInstance,
    },
    artboard::{Artboard, ArtboardInstance},
    assets::{
        blob_asset::BlobAsset, file_asset::FileAsset, font_asset::FontAsset,
        image_asset::ImageAsset, script_asset::RuntimeModuleDetailsHandle,
        shader_asset::ShaderAsset,
    },
    core::CoreHandle,
    data_bind::{
        data_context::DataContext,
        data_values::{
            DataValue, DataValueBoolean, DataValueColor, DataValueNumber, DataValueString,
        },
    },
    event::Event,
    factory::Factory,
    file::{File, RuntimeFileWeakHandle},
    hit_result::HitResult,
    input::focusable::{Key, KeyModifiers},
    input::gamepad_snapshot::{GamepadEventInvocation, GamepadSnapshot},
    math::{
        contour_measure::{ContourMeasure, RefCntContourMeasureIter},
        mat2d::Mat2D,
        mat4::Mat4,
        path_measure::PathMeasure,
        raw_path::RawPath,
        vec2d::Vec2D,
    },
    renderer::{
        ImageFilter, ImageSampler, ImageWrap, RenderBuffer, RenderImageRef, RenderPaint,
        RenderPaintStyle, RenderPath, RenderShader, Renderer,
    },
    shapes::paint::{
        blend_mode::BlendMode, color::ColorInt, shape_paint::ShapePaint, stroke_cap::StrokeCap,
        stroke_join::StrokeJoin,
    },
    text::font_hb::Font,
    transform_component::TransformComponent,
    viewmodel::{
        data_enum::DataEnum,
        viewmodel::ViewModel,
        viewmodel_instance::ViewModelInstance,
        viewmodel_instance_asset_blob::ViewModelInstanceAssetBlob,
        viewmodel_instance_asset_font::ViewModelInstanceAssetFont,
        viewmodel_instance_asset_image::ViewModelInstanceAssetImage,
        viewmodel_instance_boolean::ViewModelInstanceBoolean,
        viewmodel_instance_color::ViewModelInstanceColor,
        viewmodel_instance_enum::ViewModelInstanceEnum,
        viewmodel_instance_list::ViewModelInstanceList,
        viewmodel_instance_number::ViewModelInstanceNumber,
        viewmodel_instance_string::ViewModelInstanceString,
        viewmodel_instance_trigger::ViewModelInstanceTrigger,
        viewmodel_instance_value::{ViewModelInstanceValue, ViewModelInstanceValueDelegateHandle},
        viewmodel_instance_viewmodel::ViewModelInstanceViewModel,
    },
};

pub use crate::mechanical_port::source::r#async::work_pool::WorkPool;
pub use crate::mechanical_port::source::audio::{
    audio_engine::AudioEngine, audio_sound::AudioSoundRef, audio_source::AudioSource,
};

pub const MAX_C_STACK: i32 = 8_000;
pub const LUA_GLOBALS_INDEX: i32 = -MAX_C_STACK - 2_002;
pub const LUA_REGISTRY_INDEX: i32 = -MAX_C_STACK - 2_000;
pub const LUA_MIN_STACK: i32 = 20;
pub const LUA_NOREF: i32 = -2;
pub const LUA_OK: i32 = 0;
pub const LUA_YIELD: i32 = 1;
pub const LUA_T_COUNT: u8 = 14;

pub type LuaFunction = fn(&mut LuaState) -> i32;

#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LuaType {
    None = -1,
    Nil = 0,
    Boolean = 1,
    LightUserdata = 2,
    Number = 3,
    Integer = 4,
    Vector = 5,
    String = 6,
    Table = 7,
    Function = 8,
    Userdata = 9,
    Thread = 10,
    Buffer = 11,
    Class = 12,
    Object = 13,
}

#[derive(Clone, Copy)]
pub struct LuaReg {
    pub name: &'static str,
    pub function: Option<LuaFunction>,
}

impl LuaReg {
    pub const END: Self = Self {
        name: "",
        function: None,
    };

    pub const fn new(name: &'static str, function: LuaFunction) -> Self {
        Self {
            name,
            function: Some(function),
        }
    }
}

/// Rust-side handle for the Luau `lua_State`.
///
/// Its stack operations are supplied by the `lua_state.hpp` owner. This owner
/// defines the shared handle because every userdata declaration below stores
/// or accepts it.
#[repr(C)]
pub struct LuaState {
    _opaque: [u8; 0],
}

#[derive(Default)]
pub struct DirectFieldResult {
    pub value: LuaDirectFieldValue,
}

pub type LuaDirectFieldResult = DirectFieldResult;

#[derive(Default)]
pub enum LuaDirectFieldValue {
    #[default]
    Nil,
    Boolean(bool),
    Number(f64),
    Vector([f32; 4]),
    String(String),
}

impl DirectFieldResult {
    pub fn set_nil(&mut self) {
        self.value = LuaDirectFieldValue::Nil;
    }

    pub fn set_boolean(&mut self, value: bool) {
        self.value = LuaDirectFieldValue::Boolean(value);
    }

    pub fn set_number(&mut self, value: f64) {
        self.value = LuaDirectFieldValue::Number(value);
    }

    pub fn set_vector(&mut self, x: f32, y: f32, z: f32, w: f32) {
        self.value = LuaDirectFieldValue::Vector([x, y, z, w]);
    }

    pub fn set_string(&mut self, value: impl Into<String>) {
        self.value = LuaDirectFieldValue::String(value.into());
    }
}

pub trait LuaRive {
    const LUA_TAG: u8;
    const LUA_NAME: &'static str;
    const HAS_METATABLE: bool = true;
}

pub trait LuaRiveDataValue: LuaRive {
    fn data_value(&self) -> &ScriptedDataValue;
    fn data_value_mut(&mut self) -> &mut ScriptedDataValue;
}

pub fn lua_new_rive<T: LuaRive>(state: &mut LuaState, value: T) -> &mut T {
    state.new_rive(value)
}

pub fn lua_to_rive<T: LuaRive>(
    state: &mut LuaState,
    index: i32,
    allow_nil: bool,
) -> Option<&mut T> {
    state.to_rive_optional(index, allow_nil)
}

pub fn lua_register_rive<T: LuaRive>(state: &mut LuaState) {
    state.register_rive::<T>();
}

#[repr(i16)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LuaAtoms {
    Length,
    LengthSquared,
    Normalized,
    Distance,
    DistanceSquared,
    Dot,
    Lerp,
    MoveTo,
    LineTo,
    QuadTo,
    CubicTo,
    Close,
    Reset,
    Add,
    Contours,
    Measure,
    Type,
    Points,
    Invert,
    IsIdentity,
    Width,
    Height,
    Clamp,
    Repeat,
    Mirror,
    Bilinear,
    Nearest,
    Style,
    Join,
    Cap,
    Thickness,
    BlendMode,
    Feather,
    Gradient,
    Color,
    Stroke,
    Fill,
    Miter,
    Round,
    Bevel,
    Butt,
    Square,
    SrcOver,
    Screen,
    Overlay,
    Darken,
    Lighten,
    ColorDodge,
    ColorBurn,
    HardLight,
    SoftLight,
    Difference,
    Exclusion,
    Multiply,
    Hue,
    Saturation,
    Luminosity,
    Copy,
    DrawPath,
    DrawImage,
    DrawImageMesh,
    ClipPath,
    Save,
    Restore,
    Transform,
    Value,
    Red,
    Green,
    Blue,
    Alpha,
    GetNumber,
    GetTrigger,
    GetString,
    GetBoolean,
    GetColor,
    GetList,
    GetViewModel,
    GetEnum,
    GetIndex,
    GetImage,
    GetFont,
    GetBlob,
    Values,
    AddListener,
    RemoveListener,
    Fire,
    Push,
    Insert,
    Shift,
    Pop,
    Swap,
    Clear,
    Draw,
    Advance,
    FrameOrigin,
    Data,
    Instance,
    Animation,
    New,
    Bounds,
    PointerDown,
    PointerMove,
    PointerUp,
    PointerExit,
    AddToPath,
    Name,
    IsNumber,
    IsString,
    IsBoolean,
    IsColor,
    Hit,
    Id,
    Position,
    Rotation,
    Scale,
    WorldTransform,
    ScaleX,
    ScaleY,
    Decompose,
    Children,
    Parent,
    Node,
    Paint,
    AsPaint,
    AsPath,
    PositionAndTangent,
    Warp,
    Extract,
    Next,
    IsClosed,
    MarkNeedsUpdate,
    ViewModel,
    RootViewModel,
    Image,
    Blob,
    Size,
    DataContext,
    Audio,
    Play,
    PlayAtTime,
    PlayInTime,
    PlayAtFrame,
    PlayInFrame,
    Stop,
    Pause,
    Resume,
    Seek,
    SeekFrame,
    Volume,
    Completed,
    Time,
    TimeFrame,
    SampleRate,
    Duration,
    SetTime,
    SetTimeFrames,
    SetTimePercentage,
    PreviousPosition,
    TimeStamp,
    IsPointerEvent,
    IsKeyboardEvent,
    IsTextInput,
    IsFocus,
    IsReportedEvent,
    IsViewModelChange,
    IsNone,
    IsGamepadConnected,
    IsGamepadEvent,
    IsGamepadDisconnected,
    AsPointerEvent,
    AsKeyboardEvent,
    AsTextInput,
    AsFocus,
    AsReportedEvent,
    AsViewModelChange,
    AsGamepadConnected,
    AsGamepadEvent,
    AsGamepadDisconnected,
    GamepadEvent,
    GamepadConnected,
    GamepadDisconnected,
    AsNone,
    Key,
    Alt,
    Control,
    Meta,
    Text,
    Phase,
    DelaySeconds,
    DeviceId,
    ButtonMask,
    Remove,
    RemoveAt,
    RemoveAllOf,
    Write,
    Upload,
    View,
    SetPipeline,
    SetVertexBuffer,
    SetIndexBuffer,
    SetBindGroup,
    SetViewport,
    SetScissorRect,
    SetStencilReference,
    SetBlendColor,
    DrawIndexed,
    Finish,
    BeginRenderPass,
    BeginFrame,
    EndFrame,
    ColorView,
    DepthView,
    Resize,
    Canvas,
    GpuCanvas,
    DrawCanvas,
    Features,
    Shader,
    Format,
    AndThen,
    Catch,
    Finally,
    Cancel,
    OnCancel,
    GetStatus,
    DecodeImage,
    Transpose,
    TransformPoint,
    TransformVec4,
    WriteToBuffer,
    InvertAffine,
    WriteVec4,
    Axes,
    GamepadMapping,
    Mapping,
    IsStandardMapping,
    Buttons,
    ButtonPressed,
    ButtonValue,
    Axis,
    West,
    South,
    North,
    East,
    LeftShoulder,
    RightShoulder,
    GamepadBack,
    GamepadForward,
    LeftStickButton,
    RightStickButton,
    DpadUp,
    DpadDown,
    DpadLeft,
    DpadRight,
    LeftStick,
    RightStick,
    Start,
    LeftTrigger,
    RightTrigger,
    LeftTriggerPressed,
    RightTriggerPressed,
    ChangeKind,
    ChangeIndex,
    ChangeValue,
    HasStandardButtonIntent,
    HasStandardAxisIntent,
    IntentButton,
    IntentAxis,
}

struct LuaAtomName {
    name: &'static str,
    atom: LuaAtoms,
}

const ATOMS: &[LuaAtomName] = &[
    LuaAtomName {
        name: "length",
        atom: LuaAtoms::Length,
    },
    LuaAtomName {
        name: "lengthSquared",
        atom: LuaAtoms::LengthSquared,
    },
    LuaAtomName {
        name: "normalized",
        atom: LuaAtoms::Normalized,
    },
    LuaAtomName {
        name: "distance",
        atom: LuaAtoms::Distance,
    },
    LuaAtomName {
        name: "distanceSquared",
        atom: LuaAtoms::DistanceSquared,
    },
    LuaAtomName {
        name: "dot",
        atom: LuaAtoms::Dot,
    },
    LuaAtomName {
        name: "lerp",
        atom: LuaAtoms::Lerp,
    },
    LuaAtomName {
        name: "moveTo",
        atom: LuaAtoms::MoveTo,
    },
    LuaAtomName {
        name: "lineTo",
        atom: LuaAtoms::LineTo,
    },
    LuaAtomName {
        name: "quadTo",
        atom: LuaAtoms::QuadTo,
    },
    LuaAtomName {
        name: "cubicTo",
        atom: LuaAtoms::CubicTo,
    },
    LuaAtomName {
        name: "close",
        atom: LuaAtoms::Close,
    },
    LuaAtomName {
        name: "reset",
        atom: LuaAtoms::Reset,
    },
    LuaAtomName {
        name: "add",
        atom: LuaAtoms::Add,
    },
    LuaAtomName {
        name: "contours",
        atom: LuaAtoms::Contours,
    },
    LuaAtomName {
        name: "measure",
        atom: LuaAtoms::Measure,
    },
    LuaAtomName {
        name: "type",
        atom: LuaAtoms::Type,
    },
    LuaAtomName {
        name: "invert",
        atom: LuaAtoms::Invert,
    },
    LuaAtomName {
        name: "isIdentity",
        atom: LuaAtoms::IsIdentity,
    },
    LuaAtomName {
        name: "width",
        atom: LuaAtoms::Width,
    },
    LuaAtomName {
        name: "height",
        atom: LuaAtoms::Height,
    },
    LuaAtomName {
        name: "clamp",
        atom: LuaAtoms::Clamp,
    },
    LuaAtomName {
        name: "repeat",
        atom: LuaAtoms::Repeat,
    },
    LuaAtomName {
        name: "mirror",
        atom: LuaAtoms::Mirror,
    },
    LuaAtomName {
        name: "bilinear",
        atom: LuaAtoms::Bilinear,
    },
    LuaAtomName {
        name: "nearest",
        atom: LuaAtoms::Nearest,
    },
    LuaAtomName {
        name: "style",
        atom: LuaAtoms::Style,
    },
    LuaAtomName {
        name: "join",
        atom: LuaAtoms::Join,
    },
    LuaAtomName {
        name: "cap",
        atom: LuaAtoms::Cap,
    },
    LuaAtomName {
        name: "thickness",
        atom: LuaAtoms::Thickness,
    },
    LuaAtomName {
        name: "blendMode",
        atom: LuaAtoms::BlendMode,
    },
    LuaAtomName {
        name: "feather",
        atom: LuaAtoms::Feather,
    },
    LuaAtomName {
        name: "gradient",
        atom: LuaAtoms::Gradient,
    },
    LuaAtomName {
        name: "color",
        atom: LuaAtoms::Color,
    },
    LuaAtomName {
        name: "stroke",
        atom: LuaAtoms::Stroke,
    },
    LuaAtomName {
        name: "fill",
        atom: LuaAtoms::Fill,
    },
    LuaAtomName {
        name: "miter",
        atom: LuaAtoms::Miter,
    },
    LuaAtomName {
        name: "round",
        atom: LuaAtoms::Round,
    },
    LuaAtomName {
        name: "bevel",
        atom: LuaAtoms::Bevel,
    },
    LuaAtomName {
        name: "butt",
        atom: LuaAtoms::Butt,
    },
    LuaAtomName {
        name: "square",
        atom: LuaAtoms::Square,
    },
    LuaAtomName {
        name: "srcOver",
        atom: LuaAtoms::SrcOver,
    },
    LuaAtomName {
        name: "screen",
        atom: LuaAtoms::Screen,
    },
    LuaAtomName {
        name: "overlay",
        atom: LuaAtoms::Overlay,
    },
    LuaAtomName {
        name: "darken",
        atom: LuaAtoms::Darken,
    },
    LuaAtomName {
        name: "lighten",
        atom: LuaAtoms::Lighten,
    },
    LuaAtomName {
        name: "colorDodge",
        atom: LuaAtoms::ColorDodge,
    },
    LuaAtomName {
        name: "colorBurn",
        atom: LuaAtoms::ColorBurn,
    },
    LuaAtomName {
        name: "hardLight",
        atom: LuaAtoms::HardLight,
    },
    LuaAtomName {
        name: "softLight",
        atom: LuaAtoms::SoftLight,
    },
    LuaAtomName {
        name: "difference",
        atom: LuaAtoms::Difference,
    },
    LuaAtomName {
        name: "exclusion",
        atom: LuaAtoms::Exclusion,
    },
    LuaAtomName {
        name: "multiply",
        atom: LuaAtoms::Multiply,
    },
    LuaAtomName {
        name: "hue",
        atom: LuaAtoms::Hue,
    },
    LuaAtomName {
        name: "saturation",
        atom: LuaAtoms::Saturation,
    },
    LuaAtomName {
        name: "luminosity",
        atom: LuaAtoms::Luminosity,
    },
    LuaAtomName {
        name: "copy",
        atom: LuaAtoms::Copy,
    },
];

const ATOM_SLOT_COUNT: usize = 1_024;

const fn hash_atom_name(name: &str) -> u32 {
    let bytes = name.as_bytes();
    let mut hash = 2_166_136_261_u32;
    let mut index = 0;
    while index < bytes.len() {
        hash = (hash ^ bytes[index] as u32).wrapping_mul(16_777_619);
        index += 1;
    }
    hash
}

pub fn find_atom(name: &str) -> Option<LuaAtoms> {
    let _slot = hash_atom_name(name) as usize & (ATOM_SLOT_COUNT - 1);
    if let Some(entry) = ATOMS.iter().find(|entry| entry.name == name) {
        return Some(entry.atom);
    }
    match name {
        "drawPath" => Some(LuaAtoms::DrawPath),
        "drawImage" => Some(LuaAtoms::DrawImage),
        "drawImageMesh" => Some(LuaAtoms::DrawImageMesh),
        "clipPath" => Some(LuaAtoms::ClipPath),
        "save" => Some(LuaAtoms::Save),
        "restore" => Some(LuaAtoms::Restore),
        "transform" => Some(LuaAtoms::Transform),
        "value" => Some(LuaAtoms::Value),
        "red" => Some(LuaAtoms::Red),
        "green" => Some(LuaAtoms::Green),
        "blue" => Some(LuaAtoms::Blue),
        "alpha" => Some(LuaAtoms::Alpha),
        "getNumber" => Some(LuaAtoms::GetNumber),
        "getTrigger" => Some(LuaAtoms::GetTrigger),
        "getString" => Some(LuaAtoms::GetString),
        "getBoolean" => Some(LuaAtoms::GetBoolean),
        "getColor" => Some(LuaAtoms::GetColor),
        "getList" => Some(LuaAtoms::GetList),
        "getViewModel" => Some(LuaAtoms::GetViewModel),
        "getEnum" => Some(LuaAtoms::GetEnum),
        "getIndex" => Some(LuaAtoms::GetIndex),
        "getImage" => Some(LuaAtoms::GetImage),
        "getFont" => Some(LuaAtoms::GetFont),
        "getBlob" => Some(LuaAtoms::GetBlob),
        "values" => Some(LuaAtoms::Values),
        "addListener" => Some(LuaAtoms::AddListener),
        "removeListener" => Some(LuaAtoms::RemoveListener),
        "fire" => Some(LuaAtoms::Fire),
        "push" => Some(LuaAtoms::Push),
        "insert" => Some(LuaAtoms::Insert),
        "pop" => Some(LuaAtoms::Pop),
        "swap" => Some(LuaAtoms::Swap),
        "shift" => Some(LuaAtoms::Shift),
        "clear" => Some(LuaAtoms::Clear),
        "draw" => Some(LuaAtoms::Draw),
        "advance" => Some(LuaAtoms::Advance),
        "frameOrigin" => Some(LuaAtoms::FrameOrigin),
        "data" => Some(LuaAtoms::Data),
        "instance" => Some(LuaAtoms::Instance),
        "animation" => Some(LuaAtoms::Animation),
        "new" => Some(LuaAtoms::New),
        "bounds" => Some(LuaAtoms::Bounds),
        "pointerDown" => Some(LuaAtoms::PointerDown),
        "pointerUp" => Some(LuaAtoms::PointerUp),
        "pointerMove" => Some(LuaAtoms::PointerMove),
        "pointerExit" => Some(LuaAtoms::PointerExit),
        "isNumber" => Some(LuaAtoms::IsNumber),
        "isString" => Some(LuaAtoms::IsString),
        "isBoolean" => Some(LuaAtoms::IsBoolean),
        "isColor" => Some(LuaAtoms::IsColor),
        "hit" => Some(LuaAtoms::Hit),
        "id" => Some(LuaAtoms::Id),
        "position" => Some(LuaAtoms::Position),
        "rotation" => Some(LuaAtoms::Rotation),
        "scale" => Some(LuaAtoms::Scale),
        "worldTransform" => Some(LuaAtoms::WorldTransform),
        "scaleX" => Some(LuaAtoms::ScaleX),
        "scaleY" => Some(LuaAtoms::ScaleY),
        "decompose" => Some(LuaAtoms::Decompose),
        "children" => Some(LuaAtoms::Children),
        "parent" => Some(LuaAtoms::Parent),
        "node" => Some(LuaAtoms::Node),
        "paint" => Some(LuaAtoms::Paint),
        "asPath" => Some(LuaAtoms::AsPath),
        "asPaint" => Some(LuaAtoms::AsPaint),
        "addToPath" => Some(LuaAtoms::AddToPath),
        "positionAndTangent" => Some(LuaAtoms::PositionAndTangent),
        "warp" => Some(LuaAtoms::Warp),
        "extract" => Some(LuaAtoms::Extract),
        "next" => Some(LuaAtoms::Next),
        "isClosed" => Some(LuaAtoms::IsClosed),
        "markNeedsUpdate" => Some(LuaAtoms::MarkNeedsUpdate),
        "viewModel" => Some(LuaAtoms::ViewModel),
        "rootViewModel" => Some(LuaAtoms::RootViewModel),
        "dataContext" => Some(LuaAtoms::DataContext),
        "image" => Some(LuaAtoms::Image),
        "blob" => Some(LuaAtoms::Blob),
        "size" => Some(LuaAtoms::Size),
        "name" => Some(LuaAtoms::Name),
        "duration" => Some(LuaAtoms::Duration),
        "setTime" => Some(LuaAtoms::SetTime),
        "setTimeFrames" => Some(LuaAtoms::SetTimeFrames),
        "setTimePercentage" => Some(LuaAtoms::SetTimePercentage),
        "isPointerEvent" => Some(LuaAtoms::IsPointerEvent),
        "isKeyboardEvent" => Some(LuaAtoms::IsKeyboardEvent),
        "isTextInput" => Some(LuaAtoms::IsTextInput),
        "previousPosition" => Some(LuaAtoms::PreviousPosition),
        "timeStamp" => Some(LuaAtoms::TimeStamp),
        "isFocus" => Some(LuaAtoms::IsFocus),
        "isReportedEvent" => Some(LuaAtoms::IsReportedEvent),
        "isViewModelChange" => Some(LuaAtoms::IsViewModelChange),
        "isNone" => Some(LuaAtoms::IsNone),
        "isGamepadConnected" => Some(LuaAtoms::IsGamepadConnected),
        "isGamepadEvent" => Some(LuaAtoms::IsGamepadEvent),
        "isGamepadDisconnected" => Some(LuaAtoms::IsGamepadDisconnected),
        "asPointerEvent" => Some(LuaAtoms::AsPointerEvent),
        "asKeyboardEvent" => Some(LuaAtoms::AsKeyboardEvent),
        "asTextInput" => Some(LuaAtoms::AsTextInput),
        "asFocus" => Some(LuaAtoms::AsFocus),
        "asReportedEvent" => Some(LuaAtoms::AsReportedEvent),
        "asViewModelChange" => Some(LuaAtoms::AsViewModelChange),
        "asGamepadConnected" => Some(LuaAtoms::AsGamepadConnected),
        "asGamepadEvent" => Some(LuaAtoms::AsGamepadEvent),
        "asGamepadDisconnected" => Some(LuaAtoms::AsGamepadDisconnected),
        "gamepadEvent" => Some(LuaAtoms::GamepadEvent),
        "gamepadConnected" => Some(LuaAtoms::GamepadConnected),
        "gamepadDisconnected" => Some(LuaAtoms::GamepadDisconnected),
        "asNone" => Some(LuaAtoms::AsNone),
        "key" => Some(LuaAtoms::Key),
        "alt" => Some(LuaAtoms::Alt),
        "control" => Some(LuaAtoms::Control),
        "meta" => Some(LuaAtoms::Meta),
        "text" => Some(LuaAtoms::Text),
        "phase" => Some(LuaAtoms::Phase),
        "delaySeconds" => Some(LuaAtoms::DelaySeconds),
        "deviceId" => Some(LuaAtoms::DeviceId),
        "buttonMask" => Some(LuaAtoms::ButtonMask),
        "remove" => Some(LuaAtoms::Remove),
        "removeAt" => Some(LuaAtoms::RemoveAt),
        "removeAllOf" => Some(LuaAtoms::RemoveAllOf),
        "axes" => Some(LuaAtoms::Axes),
        "gamepadMapping" => Some(LuaAtoms::GamepadMapping),
        "mapping" => Some(LuaAtoms::Mapping),
        "isStandardMapping" => Some(LuaAtoms::IsStandardMapping),
        "buttons" => Some(LuaAtoms::Buttons),
        "buttonPressed" => Some(LuaAtoms::ButtonPressed),
        "buttonValue" => Some(LuaAtoms::ButtonValue),
        "axis" => Some(LuaAtoms::Axis),
        "west" => Some(LuaAtoms::West),
        "south" => Some(LuaAtoms::South),
        "north" => Some(LuaAtoms::North),
        "east" => Some(LuaAtoms::East),
        "leftShoulder" => Some(LuaAtoms::LeftShoulder),
        "rightShoulder" => Some(LuaAtoms::RightShoulder),
        "back" => Some(LuaAtoms::GamepadBack),
        "forward" => Some(LuaAtoms::GamepadForward),
        "leftStickButton" => Some(LuaAtoms::LeftStickButton),
        "rightStickButton" => Some(LuaAtoms::RightStickButton),
        "dpadUp" => Some(LuaAtoms::DpadUp),
        "dpadDown" => Some(LuaAtoms::DpadDown),
        "dpadLeft" => Some(LuaAtoms::DpadLeft),
        "dpadRight" => Some(LuaAtoms::DpadRight),
        "start" => Some(LuaAtoms::Start),
        "leftStick" => Some(LuaAtoms::LeftStick),
        "rightStick" => Some(LuaAtoms::RightStick),
        "leftTrigger" => Some(LuaAtoms::LeftTrigger),
        "rightTrigger" => Some(LuaAtoms::RightTrigger),
        "leftTriggerPressed" => Some(LuaAtoms::LeftTriggerPressed),
        "rightTriggerPressed" => Some(LuaAtoms::RightTriggerPressed),
        "changeKind" => Some(LuaAtoms::ChangeKind),
        "changeIndex" => Some(LuaAtoms::ChangeIndex),
        "changeValue" => Some(LuaAtoms::ChangeValue),
        "hasStandardButtonIntent" => Some(LuaAtoms::HasStandardButtonIntent),
        "hasStandardAxisIntent" => Some(LuaAtoms::HasStandardAxisIntent),
        "intentButton" => Some(LuaAtoms::IntentButton),
        "intentAxis" => Some(LuaAtoms::IntentAxis),
        "audio" => Some(LuaAtoms::Audio),
        "play" => Some(LuaAtoms::Play),
        "playAtTime" => Some(LuaAtoms::PlayAtTime),
        "playInTime" => Some(LuaAtoms::PlayInTime),
        "playAtFrame" => Some(LuaAtoms::PlayAtFrame),
        "playInFrame" => Some(LuaAtoms::PlayInFrame),
        "stop" => Some(LuaAtoms::Stop),
        "pause" => Some(LuaAtoms::Pause),
        "resume" => Some(LuaAtoms::Resume),
        "seek" => Some(LuaAtoms::Seek),
        "seekFrame" => Some(LuaAtoms::SeekFrame),
        "volume" => Some(LuaAtoms::Volume),
        "completed" => Some(LuaAtoms::Completed),
        "time" => Some(LuaAtoms::Time),
        "timeFrame" => Some(LuaAtoms::TimeFrame),
        "sampleRate" => Some(LuaAtoms::SampleRate),
        "write" => Some(LuaAtoms::Write),
        "upload" => Some(LuaAtoms::Upload),
        "view" => Some(LuaAtoms::View),
        "setPipeline" => Some(LuaAtoms::SetPipeline),
        "setVertexBuffer" => Some(LuaAtoms::SetVertexBuffer),
        "setIndexBuffer" => Some(LuaAtoms::SetIndexBuffer),
        "setBindGroup" => Some(LuaAtoms::SetBindGroup),
        "setViewport" => Some(LuaAtoms::SetViewport),
        "setScissorRect" => Some(LuaAtoms::SetScissorRect),
        "setStencilReference" => Some(LuaAtoms::SetStencilReference),
        "drawIndexed" => Some(LuaAtoms::DrawIndexed),
        "finish" => Some(LuaAtoms::Finish),
        "beginRenderPass" => Some(LuaAtoms::BeginRenderPass),
        "beginFrame" => Some(LuaAtoms::BeginFrame),
        "endFrame" => Some(LuaAtoms::EndFrame),
        "colorView" => Some(LuaAtoms::ColorView),
        "depthView" => Some(LuaAtoms::DepthView),
        "setBlendColor" => Some(LuaAtoms::SetBlendColor),
        "resize" => Some(LuaAtoms::Resize),
        "canvas" => Some(LuaAtoms::Canvas),
        "gpuCanvas" => Some(LuaAtoms::GpuCanvas),
        "features" => Some(LuaAtoms::Features),
        "drawCanvas" => Some(LuaAtoms::DrawCanvas),
        "shader" => Some(LuaAtoms::Shader),
        "format" => Some(LuaAtoms::Format),
        "andThen" => Some(LuaAtoms::AndThen),
        "catch" => Some(LuaAtoms::Catch),
        "finally" => Some(LuaAtoms::Finally),
        "cancel" => Some(LuaAtoms::Cancel),
        "onCancel" => Some(LuaAtoms::OnCancel),
        "getStatus" => Some(LuaAtoms::GetStatus),
        "decodeImage" => Some(LuaAtoms::DecodeImage),
        "transpose" => Some(LuaAtoms::Transpose),
        "transformPoint" => Some(LuaAtoms::TransformPoint),
        "transformVec4" => Some(LuaAtoms::TransformVec4),
        "writeToBuffer" => Some(LuaAtoms::WriteToBuffer),
        "invertAffine" => Some(LuaAtoms::InvertAffine),
        "writeVec4" => Some(LuaAtoms::WriteVec4),
        _ => None,
    }
}

macro_rules! impl_lua_rive {
    ($type:ty, $offset:expr, $name:literal) => {
        impl LuaRive for $type {
            const LUA_TAG: u8 = LUA_T_COUNT + $offset;
            const LUA_NAME: &'static str = $name;
        }

        impl $type {
            pub const LUA_TAG: u8 = <Self as LuaRive>::LUA_TAG;
            pub const LUA_NAME: &'static str = <Self as LuaRive>::LUA_NAME;
            pub const HAS_METATABLE: bool = <Self as LuaRive>::HAS_METATABLE;
        }
    };
    ($type:ty, $offset:expr, $name:literal, no_metatable) => {
        impl LuaRive for $type {
            const LUA_TAG: u8 = LUA_T_COUNT + $offset;
            const LUA_NAME: &'static str = $name;
            const HAS_METATABLE: bool = false;
        }

        impl $type {
            pub const LUA_TAG: u8 = <Self as LuaRive>::LUA_TAG;
            pub const LUA_NAME: &'static str = <Self as LuaRive>::LUA_NAME;
            pub const HAS_METATABLE: bool = <Self as LuaRive>::HAS_METATABLE;
        }
    };
}

#[derive(Default)]
pub struct ScriptedMat2D {
    pub value: Mat2D,
}

impl_lua_rive!(ScriptedMat2D, 1, "Mat2D");

#[derive(Default)]
pub struct ScriptedMat4 {
    pub value: Mat4,
}

impl_lua_rive!(ScriptedMat4, 62, "Mat4");

pub struct ScriptedPathCommand {
    pub command_type: String,
    pub points: Vec<Vec2D>,
}

impl ScriptedPathCommand {
    pub fn new(command_type: impl Into<String>, points: Vec<Vec2D>) -> Self {
        Self {
            command_type: command_type.into(),
            points,
        }
    }

    pub fn command_type(&self) -> &str {
        &self.command_type
    }
}

impl_lua_rive!(ScriptedPathCommand, 29, "PathCommand");

pub struct ScriptedPathData {
    pub raw_path: RawPath,
    pub render_path: Option<Box<RenderPath>>,
    pub is_render_path_dirty: bool,
    pub render_frame_id: u64,
}

impl Default for ScriptedPathData {
    fn default() -> Self {
        Self::new()
    }
}

impl ScriptedPathData {
    pub fn new() -> Self {
        Self {
            raw_path: RawPath::new(),
            render_path: None,
            is_render_path_dirty: true,
            render_frame_id: 0,
        }
    }

    pub fn mark_dirty(&mut self) {
        self.is_render_path_dirty = true;
    }
}

impl_lua_rive!(ScriptedPathData, 30, "PathData");

#[derive(Default)]
pub struct ScriptedPath {
    pub data: ScriptedPathData,
}

impl ScriptedPath {
    pub fn new() -> Self {
        Self::default()
    }
}

impl std::ops::Deref for ScriptedPath {
    type Target = ScriptedPathData;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl std::ops::DerefMut for ScriptedPath {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl_lua_rive!(ScriptedPath, 2, "Path");

#[derive(Default)]
pub struct ScriptedGradient {
    pub shader: Option<Rc<RenderShader>>,
}

impl_lua_rive!(ScriptedGradient, 3, "Gradient", no_metatable);

#[derive(Default)]
pub struct ScriptedVertexBuffer {
    pub values: Vec<Vec2D>,
    pub vertex_buffer: Option<Box<RenderBuffer>>,
}

impl_lua_rive!(ScriptedVertexBuffer, 4, "VertexBuffer");

#[derive(Default)]
pub struct ScriptedTriangleBuffer {
    pub values: Vec<u16>,
    pub index_buffer: Option<Box<RenderBuffer>>,
    pub max: u16,
}

impl_lua_rive!(ScriptedTriangleBuffer, 5, "TriangleBuffer");

#[derive(Default)]
pub struct ScriptedImage {
    pub image: Option<RenderImageRef>,
    pub cached_ore_view: Option<OreTextureView>,
    pub cached_mirror_image: Option<RenderImageRef>,
}

impl ScriptedImage {
    pub fn lua_new(state: &mut LuaState) -> &mut Self {
        state.new_rive(Self::default())
    }
}

impl_lua_rive!(ScriptedImage, 6, "Image");

#[derive(Default)]
pub struct ScriptedBlob {
    pub asset: Option<CoreHandle>,
}

impl_lua_rive!(ScriptedBlob, 35, "Blob");

pub struct ScriptedAudio;

impl_lua_rive!(ScriptedAudio, 40, "Audio");

#[derive(Default)]
pub struct ScriptedAudioSource {
    pub source: Option<std::sync::Arc<AudioSource>>,
}

impl_lua_rive!(ScriptedAudioSource, 38, "AudioSource");

pub struct ScriptedAudioSound {
    pub sound: Option<AudioSoundRef>,
    pub artboard: Option<crate::mechanical_port::source::core::CoreHandle>,
}

impl_lua_rive!(ScriptedAudioSound, 39, "AudioSound");
pub struct OreBuffer;
pub struct OreTexture;
pub struct OreTextureView;
pub struct OreSampler;
pub struct OreBindGroup;
pub struct OreBindGroupLayout;
pub struct OreShaderModule;
pub struct OrePipeline;
pub struct OreRenderPass;
pub struct OreContext;
#[derive(Default)]
pub struct ScriptedGPUBuffer {
    pub buffer: Option<OreBuffer>,
    pub immutable: bool,
}
impl_lua_rive!(ScriptedGPUBuffer, 41, "GPUBuffer");
#[derive(Default)]
pub struct ScriptedGPUTexture {
    pub texture: Option<OreTexture>,
}
impl_lua_rive!(ScriptedGPUTexture, 42, "GPUTexture");
#[derive(Default)]
pub struct ScriptedGPUSampler {
    pub sampler: Option<OreSampler>,
}
impl_lua_rive!(ScriptedGPUSampler, 43, "GPUSampler", no_metatable);
pub struct ScriptedShaderEntry {
    pub stage: u8,
    pub logical: String,
    pub physical: String,
    pub module: Option<Rc<OreShaderModule>>,
}
#[derive(Default)]
pub struct ScriptedShader {
    pub entries: Vec<ScriptedShaderEntry>,
}
impl ScriptedShader {
    pub fn has_module(&self) -> bool {
        !self.entries.is_empty()
    }

    pub fn first_of_stage(&self, stage: u8) -> Option<&ScriptedShaderEntry> {
        self.entries.iter().find(|entry| entry.stage == stage)
    }

    pub fn resolve_entry(&self, stage: u8, logical: Option<&str>) -> Option<&ScriptedShaderEntry> {
        match logical.filter(|logical| !logical.is_empty()) {
            None => self.first_of_stage(stage),
            Some(logical) => self
                .entries
                .iter()
                .find(|entry| entry.stage == stage && entry.logical == logical),
        }
    }

    pub fn vertex_mod(&self) -> Option<&OreShaderModule> {
        self.first_of_stage(0)?.module.as_deref()
    }

    pub fn fragment_mod(&self) -> Option<&OreShaderModule> {
        self.first_of_stage(1)
            .or_else(|| self.first_of_stage(0))?
            .module
            .as_deref()
    }
}
impl_lua_rive!(ScriptedShader, 44, "Shader", no_metatable);
pub struct ScriptedGPUPipeline {
    pub pipeline: Option<OrePipeline>,
    pub sample_count: u32,
    pub owned_vertex_layout_data: Vec<u8>,
    pub auto_bind_group_layouts: Vec<OreBindGroupLayout>,
}
impl Default for ScriptedGPUPipeline {
    fn default() -> Self {
        Self {
            pipeline: None,
            sample_count: 1,
            owned_vertex_layout_data: Vec::new(),
            auto_bind_group_layouts: Vec::new(),
        }
    }
}
impl_lua_rive!(ScriptedGPUPipeline, 45, "GPUPipeline");
pub struct ScriptedGPUBindGroup {
    pub bind_group: Option<OreBindGroup>,
}
impl_lua_rive!(ScriptedGPUBindGroup, 52, "GPUBindGroup", no_metatable);
pub struct ScriptedGPUBindGroupLayout {
    pub layout: Option<OreBindGroupLayout>,
}
impl_lua_rive!(
    ScriptedGPUBindGroupLayout,
    60,
    "GPUBindGroupLayout",
    no_metatable
);
pub struct ScriptedGPURenderPass {
    pub pass: Option<Box<OreRenderPass>>,
    pub context: Option<*mut OreContext>,
    pub finished: bool,
    pub pipeline_set: bool,
    pub sample_count: u32,
    pub label: String,
    pub draw_call_count: u32,
}
impl_lua_rive!(ScriptedGPURenderPass, 46, "GPURenderPass");
pub struct ScriptedGPUTextureView {
    pub view: Option<OreTextureView>,
    pub retained_image: Option<RenderImageRef>,
}
impl_lua_rive!(ScriptedGPUTextureView, 51, "GPUTextureView", no_metatable);

#[repr(i32)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum CanvasState {
    #[default]
    Idle = 0,
    Rendering = 1,
}
pub struct ScriptedCanvas {
    pub canvas: Option<RenderCanvas>,
    pub state: *mut LuaState,
    pub image_ref: i32,
    pub render_context: Option<*mut RenderContext>,
    pub canvas_state: CanvasState,
    pub rive_renderer: Option<Box<RiveRenderer>>,
    pub renderer_ref: i32,
}
impl_lua_rive!(ScriptedCanvas, 50, "Canvas");
pub struct ScriptedGPUCanvas {
    pub canvas: Option<RenderCanvas>,
    pub color_view: Option<OreTextureView>,
    pub state: *mut LuaState,
    pub image_ref: i32,
    pub render_context: Option<*mut RenderContext>,
}
impl_lua_rive!(ScriptedGPUCanvas, 47, "GPUCanvas");

#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum PromiseState {
    #[default]
    Pending = 0,
    Fulfilled = 1,
    Rejected = 2,
    Cancelled = 3,
}

pub struct ThenCallback {
    pub success_ref: i32,
    pub failure_ref: i32,
    pub chained_promise_ref: i32,
    pub cancel_ref: i32,
}

impl Default for ThenCallback {
    fn default() -> Self {
        Self {
            success_ref: LUA_NOREF,
            failure_ref: LUA_NOREF,
            chained_promise_ref: LUA_NOREF,
            cancel_ref: LUA_NOREF,
        }
    }
}

pub struct FinallyCallback {
    pub callback_ref: i32,
    pub chained_promise_ref: i32,
}

impl Default for FinallyCallback {
    fn default() -> Self {
        Self {
            callback_ref: LUA_NOREF,
            chained_promise_ref: LUA_NOREF,
        }
    }
}

pub struct ScriptedPromise {
    pub state: *mut LuaState,
    pub promise_state: PromiseState,
    pub result_ref: i32,
    pub then_callbacks: Vec<ThenCallback>,
    pub finally_callbacks: Vec<FinallyCallback>,
    pub parent_ref: i32,
    pub consumer_refs: Vec<i32>,
    pub on_cancel_ref: i32,
}

impl ScriptedPromise {
    pub fn is_fulfilled(&self) -> bool {
        self.promise_state == PromiseState::Fulfilled
    }

    pub fn is_rejected(&self) -> bool {
        self.promise_state == PromiseState::Rejected
    }

    pub fn is_cancelled(&self) -> bool {
        self.promise_state == PromiseState::Cancelled
    }

    pub fn is_pending(&self) -> bool {
        self.promise_state == PromiseState::Pending
    }

    pub fn result_ref(&self) -> i32 {
        self.result_ref
    }
}

impl_lua_rive!(ScriptedPromise, 53, "Promise");

pub struct ScriptedImageSampler {
    pub sampler: ImageSampler,
}

impl ScriptedImageSampler {
    pub fn new(wrap_x: ImageWrap, wrap_y: ImageWrap, filter: ImageFilter) -> Self {
        Self {
            sampler: ImageSampler {
                wrap_x,
                wrap_y,
                filter,
            },
        }
    }
}

impl_lua_rive!(ScriptedImageSampler, 7, "ImageSampler", no_metatable);

pub struct ScriptedPaintData {
    pub style: RenderPaintStyle,
    pub gradient: Option<Rc<RenderShader>>,
    pub thickness: f32,
    pub join: StrokeJoin,
    pub cap: StrokeCap,
    pub feather: f32,
    pub blend_mode: BlendMode,
    pub color: ColorInt,
}

impl Default for ScriptedPaintData {
    fn default() -> Self {
        Self::new()
    }
}

impl ScriptedPaintData {
    pub fn new() -> Self {
        Self {
            style: RenderPaintStyle::Fill,
            gradient: None,
            thickness: 1.0,
            join: StrokeJoin::Miter,
            cap: StrokeCap::Butt,
            feather: 0.0,
            blend_mode: BlendMode::SrcOver,
            color: 0xff00_0000,
        }
    }

    pub fn style(&self) -> RenderPaintStyle {
        self.style
    }

    pub fn set_style(&mut self, value: RenderPaintStyle) {
        self.style = value;
    }

    pub fn gradient(&self) -> Option<&RenderShader> {
        self.gradient.as_deref()
    }

    pub fn set_gradient(&mut self, value: Option<Rc<RenderShader>>) {
        self.gradient = value;
    }

    pub fn thickness(&self) -> f32 {
        self.thickness
    }

    pub fn set_thickness(&mut self, value: f32) {
        self.thickness = value;
    }

    pub fn join(&self) -> StrokeJoin {
        self.join
    }

    pub fn set_join(&mut self, value: StrokeJoin) {
        self.join = value;
    }

    pub fn cap(&self) -> StrokeCap {
        self.cap
    }

    pub fn set_cap(&mut self, value: StrokeCap) {
        self.cap = value;
    }

    pub fn feather(&self) -> f32 {
        self.feather
    }

    pub fn set_feather(&mut self, value: f32) {
        self.feather = value;
    }

    pub fn blend_mode(&self) -> BlendMode {
        self.blend_mode
    }

    pub fn set_blend_mode(&mut self, value: BlendMode) {
        self.blend_mode = value;
    }

    pub fn color(&self) -> ColorInt {
        self.color
    }

    pub fn set_color(&mut self, value: ColorInt) {
        self.color = value;
    }
}

impl_lua_rive!(ScriptedPaintData, 33, "PaintData");

pub struct ScriptedPaint {
    pub data: ScriptedPaintData,
    pub render_paint: Box<RenderPaint>,
}

impl ScriptedPaint {
    pub fn with_render_paint(render_paint: Box<RenderPaint>) -> Self {
        Self {
            data: ScriptedPaintData::new(),
            render_paint,
        }
    }

    pub fn set_style(&mut self, value: RenderPaintStyle) {
        self.data.set_style(value);
        self.render_paint.style(value);
    }

    pub fn set_color(&mut self, value: ColorInt) {
        self.data.set_color(value);
        self.render_paint.color(value);
    }

    pub fn set_thickness(&mut self, value: f32) {
        self.data.set_thickness(value);
        self.render_paint.thickness(value);
    }

    pub fn set_join(&mut self, value: StrokeJoin) {
        self.data.set_join(value);
        self.render_paint.join(value.into());
    }

    pub fn set_cap(&mut self, value: StrokeCap) {
        self.data.set_cap(value);
        self.render_paint.cap(value.into());
    }

    pub fn set_feather(&mut self, value: f32) {
        self.data.set_feather(value);
        self.render_paint.feather(value);
    }

    pub fn set_blend_mode(&mut self, value: BlendMode) {
        self.data.set_blend_mode(value);
        self.render_paint.blend_mode(value.into());
    }

    pub fn set_gradient(&mut self, value: Option<Rc<RenderShader>>) {
        self.data.set_gradient(value.clone());
        self.render_paint.shader(value.as_deref());
    }
}

impl std::ops::Deref for ScriptedPaint {
    type Target = ScriptedPaintData;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl std::ops::DerefMut for ScriptedPaint {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

impl_lua_rive!(ScriptedPaint, 8, "Paint");

pub struct ScriptedRenderer {
    pub renderer: Option<*mut Renderer>,
    pub save_count: u32,
}

impl ScriptedRenderer {
    pub fn new(renderer: &mut Renderer) -> Self {
        Self {
            renderer: Some(renderer),
            save_count: 0,
        }
    }
}

impl_lua_rive!(ScriptedRenderer, 9, "Renderer");

pub struct ScriptReffedArtboard {
    pub file: RuntimeFileWeakHandle,
    pub artboard: Option<Box<ArtboardInstance>>,
    pub state_machine: Option<Box<StateMachineInstance>>,
    pub view_model_instance: Option<CoreHandle>,
    pub scripting_context: *mut dyn ScriptingContext,
}

pub struct ScriptedArtboard {
    pub state: *mut LuaState,
    pub script_reffed_artboard: Option<Rc<ScriptReffedArtboard>>,
    pub data_context: Option<Rc<DataContext>>,
    pub data_ref: i32,
}

impl ScriptedArtboard {
    pub fn artboard(&self) -> &Artboard {
        self.script_reffed_artboard.as_ref().unwrap().artboard()
    }

    pub fn artboard_mut(&mut self) -> &mut ArtboardInstance {
        Rc::get_mut(self.script_reffed_artboard.as_mut().unwrap())
            .unwrap()
            .artboard_mut()
    }

    pub fn state_machine(&self) -> Option<&StateMachineInstance> {
        self.script_reffed_artboard.as_ref()?.state_machine()
    }

    pub fn state_machine_mut(&mut self) -> Option<&mut StateMachineInstance> {
        Rc::get_mut(self.script_reffed_artboard.as_mut()?)?.state_machine_mut()
    }

    pub fn view_model_instance(&self) -> Option<CoreHandle> {
        self.script_reffed_artboard
            .as_ref()
            .and_then(|artboard| artboard.view_model_instance.clone())
    }
}

impl_lua_rive!(ScriptedArtboard, 10, "Artboard");

pub struct ScriptedAnimation {
    pub state: *mut LuaState,
    pub animation: Box<LinearAnimationInstance>,
}

impl_lua_rive!(ScriptedAnimation, 32, "Animation");

pub struct ScriptedListener {
    pub function: i32,
    pub userdata: i32,
    pub property_self_ref: i32,
}

pub struct ScriptedPropertyRuntime {
    pub listeners: Vec<ScriptedListener>,
    pub state: *mut LuaState,
    pub cached_value_ref: i32,
}

pub struct ScriptedProperty {
    pub runtime: Rc<RefCell<ScriptedPropertyRuntime>>,
    pub delegate: Option<ViewModelInstanceValueDelegateHandle>,
    pub owner: Option<CoreHandle>,
    #[cfg(feature = "tools")]
    pub orphan_context: Option<*mut dyn ScriptingContext>,
    #[cfg(feature = "tools")]
    pub orphan_owner_tag: u32,
    pub disposed: bool,
    pub instance_value: Option<CoreHandle>,
}

impl ScriptedProperty {
    pub fn state(&self) -> *mut LuaState {
        self.runtime.borrow().state
    }

    pub fn instance_value(&self) -> Option<CoreHandle> {
        self.instance_value.clone()
    }

    pub fn instance_value_mut(&mut self) -> Option<CoreHandle> {
        self.instance_value.clone()
    }

    pub fn cached_value_ref(&self) -> i32 {
        self.runtime.borrow().cached_value_ref
    }

    pub fn set_cached_value_ref(&mut self, value: i32) {
        self.runtime.borrow_mut().cached_value_ref = value;
    }
}

pub struct ScriptedViewModel {
    pub state: *mut LuaState,
    pub view_model: Option<CoreHandle>,
    pub view_model_instance: Option<CoreHandle>,
    pub property_refs: HashMap<String, i32>,
    pub scripting_context: Option<*mut dyn ScriptingContext>,
}

impl_lua_rive!(ScriptedViewModel, 11, "ViewModel");

macro_rules! scripted_property_type {
    ($name:ident, $offset:expr, $lua_name:literal) => {
        pub struct $name {
            pub property: ScriptedProperty,
        }

        impl std::ops::Deref for $name {
            type Target = ScriptedProperty;

            fn deref(&self) -> &Self::Target {
                &self.property
            }
        }

        impl std::ops::DerefMut for $name {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.property
            }
        }

        impl_lua_rive!($name, $offset, $lua_name);
    };
}

pub struct ScriptedPropertyViewModel {
    pub property: ScriptedProperty,
    pub view_model: Option<CoreHandle>,
    pub value_ref: i32,
}

impl_lua_rive!(ScriptedPropertyViewModel, 12, "PropertyViewModel");
scripted_property_type!(ScriptedPropertyNumber, 13, "Property<number>");
scripted_property_type!(ScriptedPropertyTrigger, 14, "PropertyTrigger");

pub struct ScriptedPropertyList {
    pub property: ScriptedProperty,
    pub changed: bool,
    pub property_refs: HashMap<CoreHandle, i32>,
}

impl_lua_rive!(ScriptedPropertyList, 15, "PropertyList");
scripted_property_type!(ScriptedPropertyColor, 16, "PropertyColor");
scripted_property_type!(ScriptedPropertyString, 17, "PropertyString");
scripted_property_type!(ScriptedPropertyBoolean, 18, "Property<bool>");

pub struct ScriptedEnumValues {
    pub state: *mut LuaState,
    pub data_enum: Option<CoreHandle>,
}

impl ScriptedEnumValues {
    pub fn state(&self) -> *mut LuaState {
        self.state
    }
}

impl_lua_rive!(ScriptedEnumValues, 34, "EnumValues");
scripted_property_type!(ScriptedPropertyEnum, 19, "Property<enum>");
scripted_property_type!(ScriptedPropertyImage, 49, "Property<Image>");

#[derive(Default)]
pub struct ScriptedFont {
    pub font: Option<Font>,
}

impl_lua_rive!(ScriptedFont, 65, "Font", no_metatable);
scripted_property_type!(ScriptedPropertyFont, 66, "Property<Font>");
scripted_property_type!(ScriptedPropertyBlob, 67, "Property<Blob>");

pub struct ScriptedDataValue {
    pub state: *mut LuaState,
    pub data_value: Option<DataValue>,
}

impl ScriptedDataValue {
    pub const LUA_NAME: &'static str = "DataValue";

    pub fn new(state: &mut LuaState, data_value: DataValue) -> Self {
        Self {
            state,
            data_value: Some(data_value),
        }
    }

    pub fn is_number(&self) -> bool {
        matches!(self.data_value, Some(DataValue::Number(_)))
    }

    pub fn is_string(&self) -> bool {
        matches!(self.data_value, Some(DataValue::String(_)))
    }

    pub fn is_boolean(&self) -> bool {
        matches!(self.data_value, Some(DataValue::Boolean(_)))
    }

    pub fn is_color(&self) -> bool {
        matches!(self.data_value, Some(DataValue::Color(_)))
    }
}

macro_rules! scripted_data_value {
    ($name:ident, $offset:expr, $lua_name:literal, $variant:ident, $value:ty, $constructor:expr) => {
        pub struct $name {
            pub base: ScriptedDataValue,
        }

        impl $name {
            pub fn new(state: &mut LuaState, value: $value) -> Self {
                Self {
                    base: ScriptedDataValue::new(state, DataValue::$variant(($constructor)(value))),
                }
            }
        }

        impl LuaRiveDataValue for $name {
            fn data_value(&self) -> &ScriptedDataValue {
                &self.base
            }

            fn data_value_mut(&mut self) -> &mut ScriptedDataValue {
                &mut self.base
            }
        }

        impl std::ops::Deref for $name {
            type Target = ScriptedDataValue;

            fn deref(&self) -> &Self::Target {
                &self.base
            }
        }

        impl std::ops::DerefMut for $name {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.base
            }
        }

        impl_lua_rive!($name, $offset, $lua_name);
    };
}

scripted_data_value!(
    ScriptedDataValueNumber,
    20,
    "DataValueNumber",
    Number,
    f32,
    DataValueNumber::new
);
scripted_data_value!(
    ScriptedDataValueString,
    21,
    "DataValueString",
    String,
    &str,
    DataValueString::new
);
scripted_data_value!(
    ScriptedDataValueBoolean,
    22,
    "DataValueBoolean",
    Boolean,
    bool,
    DataValueBoolean::new
);
scripted_data_value!(
    ScriptedDataValueColor,
    23,
    "DataValueColor",
    Color,
    i32,
    DataValueColor::new
);

pub struct ScriptedPointerEvent {
    pub id: u8,
    pub position: Vec2D,
    pub previous_position: Vec2D,
    pub hit_listener_type: i32,
    pub time_stamp: f64,
    pub hit_result: HitResult,
}

impl ScriptedPointerEvent {
    pub fn new(
        id: u8,
        position: Vec2D,
        previous_position: Vec2D,
        hit_listener_type: i32,
        time_stamp: f64,
    ) -> Self {
        Self {
            id,
            position,
            previous_position,
            hit_listener_type,
            time_stamp,
            hit_result: HitResult::None,
        }
    }
}

impl_lua_rive!(ScriptedPointerEvent, 24, "PointerEvent");

pub struct ScriptedNode {
    pub artboard: Rc<ScriptReffedArtboard>,
    pub component: CoreHandle,
    pub shape_paint: Option<CoreHandle>,
}

impl_lua_rive!(ScriptedNode, 25, "NodeData");

pub struct ScriptedContourMeasure {
    pub measure: Rc<ContourMeasure>,
    pub iterator: Option<Rc<RefCell<RefCntContourMeasureIter>>>,
}

impl ScriptedContourMeasure {
    pub fn new(
        measure: Rc<ContourMeasure>,
        iterator: Option<Rc<RefCell<RefCntContourMeasureIter>>>,
    ) -> Self {
        Self { measure, iterator }
    }
}

impl_lua_rive!(ScriptedContourMeasure, 26, "ContourMeasure");

pub struct ScriptedPathMeasure {
    pub measure: PathMeasure,
}

impl ScriptedPathMeasure {
    pub fn new(measure: PathMeasure) -> Self {
        Self { measure }
    }
}

impl_lua_rive!(ScriptedPathMeasure, 27, "PathMeasure");

pub struct ScriptedContext {
    pub scripted_object: Option<CoreHandle>,
    pub missing_requested_data: bool,
}

impl ScriptedContext {
    pub fn scripted_object(&self) -> Option<CoreHandle> {
        self.scripted_object.clone()
    }

    pub fn clear_scripted_object(&mut self) {
        self.scripted_object = None;
    }

    pub fn missing_requested_data(&self) -> bool {
        self.missing_requested_data
    }
}

impl_lua_rive!(ScriptedContext, 28, "Context");

pub struct ScriptedInvocation {
    pub invocation: ListenerInvocation,
}

impl ScriptedInvocation {
    pub fn new(invocation: ListenerInvocation) -> Self {
        Self { invocation }
    }

    pub fn invocation(&self) -> &ListenerInvocation {
        &self.invocation
    }

    pub fn invocation_mut(&mut self) -> &mut ListenerInvocation {
        &mut self.invocation
    }
}

impl_lua_rive!(ScriptedInvocation, 54, "Invocation");

pub struct ScriptedKeyboardInvocation {
    pub key: Key,
    pub modifiers: KeyModifiers,
    pub is_pressed: bool,
    pub is_repeat: bool,
}

impl ScriptedKeyboardInvocation {
    pub fn new(key: Key, modifiers: KeyModifiers, is_pressed: bool, is_repeat: bool) -> Self {
        Self {
            key,
            modifiers,
            is_pressed,
            is_repeat,
        }
    }
}

impl_lua_rive!(ScriptedKeyboardInvocation, 55, "KeyboardInvocation");

pub struct ScriptedTextInputInvocation {
    pub text: String,
}

impl ScriptedTextInputInvocation {
    pub fn new(text: impl Into<String>) -> Self {
        Self { text: text.into() }
    }
}

impl_lua_rive!(ScriptedTextInputInvocation, 56, "TextInputInvocation");

pub struct ScriptedFocusInvocation {
    pub is_focus: bool,
}

impl ScriptedFocusInvocation {
    pub fn new(is_focus: bool) -> Self {
        Self { is_focus }
    }
}

impl_lua_rive!(ScriptedFocusInvocation, 57, "FocusInvocation");

pub struct ScriptedReportedEventInvocation {
    pub event: CoreHandle,
    pub delay_seconds: f32,
}

impl ScriptedReportedEventInvocation {
    pub fn new(event: CoreHandle, delay_seconds: f32) -> Self {
        Self {
            event,
            delay_seconds,
        }
    }
}

impl_lua_rive!(
    ScriptedReportedEventInvocation,
    58,
    "ReportedEventInvocation"
);

pub struct ScriptedViewModelChangeInvocation;
impl ScriptedViewModelChangeInvocation {
    pub fn new() -> Self {
        Self
    }
}
impl_lua_rive!(
    ScriptedViewModelChangeInvocation,
    59,
    "ViewModelChangeInvocation"
);

pub struct ScriptedGamepadConnected {
    pub snapshot: GamepadSnapshot,
}

impl ScriptedGamepadConnected {
    pub fn new(snapshot: GamepadSnapshot) -> Self {
        Self { snapshot }
    }
}

impl_lua_rive!(ScriptedGamepadConnected, 48, "GamepadConnected");

pub struct ScriptedGamepadEvent {
    pub data: GamepadEventInvocation,
}

impl ScriptedGamepadEvent {
    pub fn new(data: GamepadEventInvocation) -> Self {
        Self { data }
    }
}

impl_lua_rive!(ScriptedGamepadEvent, 63, "GamepadEvent");

pub struct ScriptedGamepadDisconnected {
    pub device_id: i32,
}

impl ScriptedGamepadDisconnected {
    pub fn new(device_id: i32) -> Self {
        Self { device_id }
    }
}

impl_lua_rive!(ScriptedGamepadDisconnected, 64, "GamepadDisconnected");

pub struct ScriptedNoneInvocation;
impl ScriptedNoneInvocation {
    pub fn new() -> Self {
        Self
    }
}
impl_lua_rive!(ScriptedNoneInvocation, 61, "NoneInvocation");

pub enum ScriptedDataContextHandle {
    Shared(Rc<DataContext>),
    Mutable(Rc<RefCell<DataContext>>),
}

pub struct ScriptedDataContext {
    pub data_context: ScriptedDataContextHandle,
}

impl ScriptedDataContextHandle {
    pub fn parent(&self) -> Option<Rc<DataContext>> {
        match self {
            Self::Shared(context) => context.parent(),
            Self::Mutable(context) => context.borrow().parent(),
        }
    }

    pub fn main_view_model_instance(&self) -> Option<CoreHandle> {
        match self {
            Self::Shared(context) => context.main_view_model_instance(),
            Self::Mutable(context) => context.borrow().main_view_model_instance(),
        }
    }
}

impl_lua_rive!(ScriptedDataContext, 36, "DataContext");

#[derive(Default)]
pub struct ScopedAssetReference {
    label: String,
    path: String,
    scope_prefix: String,
    bare: String,
}

impl ScopedAssetReference {
    pub fn new(state: Option<&LuaState>, reference: &str) -> Self {
        const LIBRARY_PREFIX: &str = "lib:";
        if let Some(request) = reference.strip_prefix(LIBRARY_PREFIX) {
            if let Some((label, path)) = request.split_once('/') {
                return Self {
                    label: label.to_owned(),
                    path: path.to_owned(),
                    scope_prefix: String::new(),
                    bare: String::new(),
                };
            }
        }

        let mut result = Self {
            bare: reference.to_owned(),
            ..Self::default()
        };
        let Some(state) = state else {
            return result;
        };
        let Some(chunk_name) = state.calling_chunk_name_skipping_c_frames() else {
            return result;
        };
        let Some(slash) = chunk_name.find('/') else {
            return result;
        };
        if let Some(at) = chunk_name.find('@') {
            if at > 0 && at < slash {
                result.scope_prefix = chunk_name[..slash].to_owned();
            }
        }
        result
    }

    fn matches_library(&self, registered_name: &str) -> bool {
        let Some(mut rest) = registered_name.strip_prefix(&self.label) else {
            return false;
        };
        if rest.is_empty() {
            return false;
        }
        if let Some(numbered) = rest.strip_prefix('#') {
            let digits = numbered.bytes().take_while(u8::is_ascii_digit).count();
            if digits == 0 {
                return false;
            }
            rest = &numbered[digits..];
        }
        let Some(versioned) = rest.strip_prefix('@') else {
            return false;
        };
        let digits = versioned.bytes().take_while(u8::is_ascii_digit).count();
        if digits == 0 || versioned.as_bytes().get(digits) != Some(&b'/') {
            return false;
        }
        &versioned[digits + 1..] == self.path
    }

    pub fn rank(&self, registered_name: &str, short_name: &str) -> i32 {
        if !self.label.is_empty() {
            return i32::from(self.matches_library(registered_name));
        }
        if let Some(relative) = registered_name
            .strip_prefix(&self.scope_prefix)
            .and_then(|rest| rest.strip_prefix('/'))
            .filter(|_| !self.scope_prefix.is_empty())
        {
            return if relative == self.bare || short_name == self.bare {
                2
            } else {
                0
            };
        }
        let first_segment = registered_name.split('/').next().unwrap_or_default();
        if first_segment.find('@').is_some_and(|at| at > 0) {
            return 0;
        }
        i32::from(registered_name == self.bare || short_name == self.bare)
    }
}

pub struct TrackedViewModelInstance {
    pub instance: CoreHandle,
    pub registrations: i32,
}

pub struct ScriptingContextData {
    pub render_context: Option<*mut Factory>,
    pub owner_id: u64,
    pub ore_frame_open: bool,
    pub canvas_drawing_phase: bool,
    pub gpu_canvas_defer_only: bool,
    pub previous_gl_context: isize,
    #[cfg(target_family = "wasm")]
    pub gl_handle: i32,
    pub factory: *mut Factory,
    pub current_scripted_object: Option<CoreHandle>,
    pub modules_to_register: Vec<RuntimeModuleDetailsHandle>,
    pub module_lookup: HashMap<String, RuntimeModuleDetailsHandle>,
    pub pending_modules: HashSet<RuntimeModuleDetailsHandle>,
    pub tracked_view_model_instances: HashMap<CoreHandle, TrackedViewModelInstance>,
    #[cfg(feature = "tools")]
    pub asset_generator_refs: HashMap<u32, i32>,
    #[cfg(feature = "tools")]
    pub is_playing: bool,
    #[cfg(feature = "tools")]
    pub orphan_scripted_properties: Vec<*mut ScriptedProperty>,
    #[cfg(feature = "tools")]
    pub orphan_owner_tag: u32,
    #[cfg(feature = "tools")]
    pub shader_rstbs: HashMap<String, Vec<u8>>,
}

impl ScriptingContextData {
    pub fn new(factory: &mut Factory) -> Self {
        Self {
            render_context: None,
            owner_id: 0,
            ore_frame_open: false,
            canvas_drawing_phase: false,
            gpu_canvas_defer_only: false,
            previous_gl_context: 0,
            #[cfg(target_family = "wasm")]
            gl_handle: 0,
            factory,
            current_scripted_object: None,
            modules_to_register: Vec::new(),
            module_lookup: HashMap::new(),
            pending_modules: HashSet::new(),
            tracked_view_model_instances: HashMap::new(),
            #[cfg(feature = "tools")]
            asset_generator_refs: HashMap::new(),
            #[cfg(feature = "tools")]
            is_playing: false,
            #[cfg(feature = "tools")]
            orphan_scripted_properties: Vec::new(),
            #[cfg(feature = "tools")]
            orphan_owner_tag: 0,
            #[cfg(feature = "tools")]
            shader_rstbs: HashMap::new(),
        }
    }
}

pub trait ScriptingContext {
    fn data(&self) -> &ScriptingContextData;
    fn data_mut(&mut self) -> &mut ScriptingContextData;

    fn print_error(&mut self, state: &mut LuaState);
    fn print_begin_line(&mut self, state: &mut LuaState);
    fn print(&mut self, data: &[u8]);
    fn print_end_line(&mut self);

    fn p_call(&mut self, state: &mut LuaState, argument_count: i32, result_count: i32) -> i32 {
        state.pcall(argument_count, result_count, 0)
    }

    fn initializes_data_global_externally(&self) -> bool {
        false
    }

    fn factory(&mut self) -> &mut Factory {
        unsafe { &mut *self.data().factory }
    }

    fn current_scripted_object(&self) -> Option<CoreHandle> {
        self.data().current_scripted_object.clone()
    }

    fn set_current_scripted_object(&mut self, value: Option<CoreHandle>) {
        self.data_mut().current_scripted_object = value;
    }

    fn add_module(&mut self, module: RuntimeModuleDetailsHandle) {
        let Some(name) = module.with_module(|module| module.module_name()) else {
            return;
        };
        self.data_mut().modules_to_register.push(module.clone());
        self.data_mut().module_lookup.insert(name, module);
    }

    fn try_register_module(
        &mut self,
        state: &mut LuaState,
        module: &RuntimeModuleDetailsHandle,
    ) -> bool {
        let Some((verified, name, protocol_script, bytecode)) = module.with_module(|module| {
            (
                module.verified(),
                module.module_name(),
                module.is_protocol_script(),
                module.module_bytecode().to_vec(),
            )
        }) else {
            return false;
        };
        #[cfg(not(feature = "tools"))]
        if !verified {
            return false;
        }

        let mut function_ref = 0;
        let registered = if protocol_script {
            if scripting_vm::ScriptingVM::register_script_on(state, &name, &bytecode, None) {
                if state.value_type(-1) == LuaType::Function {
                    function_ref = state.reference(-1);
                }
                state.pop(1);
                true
            } else {
                false
            }
        } else {
            scripting_vm::ScriptingVM::register_module_on(state, &name, &bytecode, None)
        };
        if registered {
            module.with_module_mut(|module| module.registration_complete(function_ref));
            self.on_module_registered(module);
        }
        registered
    }

    fn perform_registration(&mut self, state: &mut LuaState) {
        let modules = self.data().modules_to_register.clone();
        for module in modules {
            let Some(cache_key) = module.with_module(|module| module.module_name()) else {
                continue;
            };
            if check_registered_modules(state, &cache_key) {
                state.pop(1);
                continue;
            }
            self.try_register_module(state, &module);
        }

        if !self.data().pending_modules.is_empty() {
            let mut pending: Vec<_> = self.data().pending_modules.iter().copied().collect();
            let mut sorted = Vec::new();
            let mut visited = HashSet::new();
            if let Some(module) = pending.pop() {
                self.sort_next_module(module, &mut pending, &mut sorted, &mut visited);
            }
            for module in sorted {
                self.try_register_module(state, &module);
            }
        }

        self.data_mut().modules_to_register.clear();
        self.data_mut().pending_modules.clear();
    }

    fn sort_next_module(
        &self,
        module: RuntimeModuleDetailsHandle,
        pending: &mut Vec<RuntimeModuleDetailsHandle>,
        sorted: &mut Vec<RuntimeModuleDetailsHandle>,
        visited: &mut HashSet<RuntimeModuleDetailsHandle>,
    ) {
        if !visited.insert(module.clone()) {
            return;
        }
        let dependencies = module
            .with_module(|module| module.missing_dependencies())
            .unwrap_or_default();
        for dependency in dependencies {
            if let Some(dependency_module) = self.data().module_lookup.get(&dependency) {
                self.sort_next_module(dependency_module.clone(), pending, sorted, visited);
            }
        }
        if !sorted.contains(&module) {
            sorted.push(module.clone());
        }
        if let Some(next) = pending.pop() {
            self.sort_next_module(next, pending, sorted, visited);
        }
    }

    fn record_missing_dependency(&mut self, requiring_module: &str, missing_module: &str) {
        if requiring_module.is_empty() {
            return;
        }
        let Some(module) = self.data().module_lookup.get(requiring_module).cloned() else {
            return;
        };
        module.with_module_mut(|module| module.add_missing_dependency(missing_module.to_owned()));
        self.data_mut().pending_modules.insert(module);
    }

    fn on_module_registered(&mut self, registered: &RuntimeModuleDetailsHandle) {
        let Some(key) = registered.with_module(|module| module.module_name()) else {
            return;
        };
        let modules = self.data().modules_to_register.clone();
        for module in modules {
            module.with_module_mut(|module| {
                if !module.missing_dependencies().is_empty() {
                    module.clear_missing_dependency(&key);
                }
            });
        }
        self.data_mut().pending_modules.remove(registered);
    }

    fn track_view_model_instance(&mut self, instance: Option<CoreHandle>) {
        let Some(instance) = instance else {
            return;
        };
        let tracked = self
            .data_mut()
            .tracked_view_model_instances
            .entry(instance.clone())
            .or_insert(TrackedViewModelInstance {
                instance,
                registrations: 0,
            });
        tracked.registrations += 1;
    }

    fn untrack_view_model_instance(&mut self, instance: Option<&CoreHandle>) {
        let Some(instance) = instance else {
            return;
        };
        if let Some(tracked) = self
            .data_mut()
            .tracked_view_model_instances
            .get_mut(instance)
        {
            tracked.registrations -= 1;
            if tracked.registrations <= 0 {
                self.data_mut()
                    .tracked_view_model_instances
                    .remove(instance);
            }
        }
    }

    fn advance_detached_view_models(&mut self) {
        for tracked in self.data_mut().tracked_view_model_instances.values_mut() {
            tracked.instance.with_mut(|instance| {
                if let Some(instance) = instance.as_view_model_instance_mut()
                    && !instance.has_parents()
                {
                    instance.advanced();
                }
            });
        }
    }

    fn set_render_context(&mut self, context: Option<&mut Factory>) {
        self.data_mut().render_context = context.map(|context| context as *mut Factory);
    }

    fn render_context(&mut self) -> Option<&mut Factory> {
        self.data()
            .render_context
            .map(|context| unsafe { &mut *context })
    }
    fn ore_context(&mut self) -> Option<&mut OreContext> {
        self.render_context()?.ore()
    }

    fn owner_id(&self) -> u64 {
        self.data().owner_id
    }

    fn work_pool(&mut self) -> &mut WorkPool {
        if self.data().owner_id == 0 {
            self.data_mut().owner_id = WorkPool::next_owner_id();
        }
        get_global_work_pool()
    }

    fn shutdown_async(&mut self) {
        let owner_id = self.data().owner_id;
        if owner_id != 0 {
            if let Some(pool) = get_global_work_pool_if_exists() {
                pool.cancel_all_for_owner(owner_id);
            }
            self.data_mut().owner_id = 0;
        }
    }

    fn shutdown_async_for_state(&mut self, main_thread: &mut LuaState) {
        self.shutdown_async();
        #[cfg(target_family = "wasm")]
        wasm_cancel_pending_decodes(main_thread);
    }

    fn set_ore_frame_open(&mut self, value: bool) {
        self.data_mut().ore_frame_open = value;
    }

    fn ore_frame_open(&self) -> bool {
        self.data().ore_frame_open
    }

    fn set_canvas_drawing_phase(&mut self, value: bool) {
        self.data_mut().canvas_drawing_phase = value;
    }

    fn canvas_drawing_phase(&self) -> bool {
        self.data().canvas_drawing_phase
    }

    fn set_gpu_canvas_defer_only(&mut self, value: bool) {
        self.data_mut().gpu_canvas_defer_only = value;
    }

    fn gpu_canvas_defer_only(&self) -> bool {
        self.data().gpu_canvas_defer_only
    }

    fn set_previous_gl_context(&mut self, value: isize) {
        self.data_mut().previous_gl_context = value;
    }

    fn previous_gl_context(&self) -> isize {
        self.data().previous_gl_context
    }

    #[cfg(feature = "tools")]
    fn set_generator_ref(&mut self, asset_id: u32, reference: i32) {
        self.data_mut()
            .asset_generator_refs
            .insert(asset_id, reference);
    }

    #[cfg(feature = "tools")]
    fn generator_ref(&self, asset_id: u32) -> i32 {
        self.data()
            .asset_generator_refs
            .get(&asset_id)
            .copied()
            .unwrap_or(0)
    }

    #[cfg(feature = "tools")]
    fn clear_generator_refs(&mut self) {
        self.data_mut().asset_generator_refs.clear();
    }

    #[cfg(feature = "tools")]
    fn has_generator_ref(&self, asset_id: u32) -> bool {
        self.data().asset_generator_refs.contains_key(&asset_id)
    }

    #[cfg(feature = "tools")]
    fn set_is_playing(&mut self, value: bool) {
        self.data_mut().is_playing = value;
    }

    #[cfg(feature = "tools")]
    fn is_playing(&self) -> bool {
        self.data().is_playing
    }

    #[cfg(feature = "tools")]
    fn track_orphan_scripted_property(&mut self, property: &mut ScriptedProperty) {
        self.data_mut().orphan_scripted_properties.push(property);
    }

    #[cfg(feature = "tools")]
    fn untrack_orphan_scripted_property(&mut self, property: &mut ScriptedProperty) {
        let pointer = property as *mut ScriptedProperty;
        self.data_mut()
            .orphan_scripted_properties
            .retain(|candidate| *candidate != pointer);
    }

    #[cfg(feature = "tools")]
    fn dispose_orphan_scripted_properties(&mut self, owner_tag: Option<u32>) {
        if owner_tag == Some(0) {
            return;
        }
        let orphans = self.data().orphan_scripted_properties.clone();
        for property in orphans {
            let property = unsafe { &mut *property };
            if owner_tag.is_none() || owner_tag == Some(property.orphan_owner_tag) {
                property.dispose();
            }
        }
        if owner_tag.is_none() {
            self.data_mut().orphan_scripted_properties.clear();
        }
    }

    #[cfg(feature = "tools")]
    fn orphan_owner_tag(&self) -> u32 {
        self.data().orphan_owner_tag
    }

    #[cfg(feature = "tools")]
    fn set_orphan_owner_tag(&mut self, value: u32) {
        self.data_mut().orphan_owner_tag = value;
    }

    #[cfg(feature = "tools")]
    fn register_shader_rstb(&mut self, name: String, bytes: Vec<u8>) {
        self.data_mut().shader_rstbs.insert(name, bytes);
    }

    #[cfg(feature = "tools")]
    fn find_shader_rstb(&self, reference: &ScopedAssetReference) -> Option<&[u8]> {
        self.data()
            .shader_rstbs
            .iter()
            .filter_map(|(name, bytes)| {
                let short_name = name.rsplit('/').next().unwrap_or(name);
                let rank = reference.rank(name, short_name);
                (rank > 0).then_some((rank, bytes.as_slice()))
            })
            .max_by_key(|(rank, _)| *rank)
            .map(|(_, bytes)| bytes)
    }

    #[cfg(feature = "tools")]
    fn take_shader_rstbs(&mut self) -> HashMap<String, Vec<u8>> {
        std::mem::take(&mut self.data_mut().shader_rstbs)
    }
}

pub struct ScopedScriptedObjectContext {
    context: Option<*mut dyn ScriptingContext>,
    previous: Option<CoreHandle>,
}

impl ScopedScriptedObjectContext {
    pub fn new(
        context: Option<&mut dyn ScriptingContext>,
        scripted_object: Option<CoreHandle>,
    ) -> Self {
        let previous = context
            .as_ref()
            .and_then(|context| context.current_scripted_object());
        let context_pointer = context.map(|context| context as *mut dyn ScriptingContext);
        if let Some(context) = context_pointer {
            unsafe { &mut *context }.set_current_scripted_object(scripted_object);
        }
        Self {
            context: context_pointer,
            previous,
        }
    }
}

impl Drop for ScopedScriptedObjectContext {
    fn drop(&mut self) {
        if let Some(context) = self.context {
            unsafe { &mut *context }.set_current_scripted_object(self.previous);
        }
    }
}

pub struct ScopedCanvasDrawingPhase {
    context: Option<*mut dyn ScriptingContext>,
    previous: bool,
}

impl ScopedCanvasDrawingPhase {
    pub fn new(context: Option<&mut dyn ScriptingContext>) -> Self {
        let previous = context
            .as_ref()
            .is_some_and(|context| context.canvas_drawing_phase());
        let context_pointer = context.map(|context| context as *mut dyn ScriptingContext);
        if let Some(context) = context_pointer {
            unsafe { &mut *context }.set_canvas_drawing_phase(true);
        }
        Self {
            context: context_pointer,
            previous,
        }
    }
}

impl Drop for ScopedCanvasDrawingPhase {
    fn drop(&mut self) {
        if let Some(context) = self.context {
            unsafe { &mut *context }.set_canvas_drawing_phase(self.previous);
        }
    }
}

#[cfg(feature = "tools")]
pub type ConsoleCallback = fn();

pub struct CPPRuntimeScriptingContext {
    pub context: ScriptingContextData,
    pub execution_time: Instant,
    timeout_ms: i32,
    #[cfg(feature = "tools")]
    console_callback: Option<ConsoleCallback>,
    #[cfg(feature = "tools")]
    console_buffer: Vec<u8>,
    #[cfg(feature = "tools")]
    called_console_callback: bool,
}

impl CPPRuntimeScriptingContext {
    pub fn new(factory: &mut Factory) -> Self {
        Self {
            context: ScriptingContextData::new(factory),
            execution_time: Instant::now(),
            timeout_ms: 200,
            #[cfg(feature = "tools")]
            console_callback: None,
            #[cfg(feature = "tools")]
            console_buffer: Vec::new(),
            #[cfg(feature = "tools")]
            called_console_callback: false,
        }
    }

    pub fn with_timeout(factory: &mut Factory, timeout_ms: i32) -> Self {
        let mut context = Self::new(factory);
        context.timeout_ms = timeout_ms;
        context
    }

    #[cfg(feature = "tools")]
    pub fn with_console_callback(
        factory: &mut Factory,
        timeout_ms: i32,
        callback: Option<ConsoleCallback>,
    ) -> Self {
        let mut context = Self::with_timeout(factory, timeout_ms);
        context.console_callback = callback;
        context
    }

    pub fn timeout_ms(&self) -> i32 {
        self.timeout_ms
    }

    pub fn set_timeout_ms(&mut self, value: i32) {
        self.timeout_ms = value;
    }

    pub fn start_timed_execution(&mut self, state: &mut LuaState) {
        if self.timeout_ms == 0 {
            return;
        }
        state.set_interrupt_callback(interrupt_cpp);
        self.execution_time = Instant::now();
    }

    pub fn end_timed_execution(&mut self, state: &mut LuaState) {
        if self.timeout_ms != 0 {
            state.clear_interrupt_callback();
        }
    }

    #[cfg(feature = "tools")]
    pub fn console_memory(&self) -> &[u8] {
        &self.console_buffer
    }

    #[cfg(feature = "tools")]
    pub fn clear_console(&mut self) {
        self.console_buffer.clear();
        self.called_console_callback = false;
    }

    #[cfg(feature = "tools")]
    pub fn has_console_callback(&self) -> bool {
        self.console_callback.is_some()
    }
}

impl ScriptingContext for CPPRuntimeScriptingContext {
    fn data(&self) -> &ScriptingContextData {
        &self.context
    }

    fn data_mut(&mut self) -> &mut ScriptingContextData {
        &mut self.context
    }

    fn print_begin_line(&mut self, state: &mut LuaState) {
        #[cfg(feature = "tools")]
        {
            self.console_buffer.push(0);
            let (source, line) = state
                .debug_source_and_line(1)
                .unwrap_or_else(|| (String::new(), 0));
            write_var_uint(&mut self.console_buffer, source.len() as u64);
            self.console_buffer.extend_from_slice(source.as_bytes());
            write_var_uint(&mut self.console_buffer, line as u64);
        }
    }

    fn print(&mut self, data: &[u8]) {
        #[cfg(feature = "tools")]
        {
            if data.is_empty() {
                return;
            }
            write_var_uint(&mut self.console_buffer, data.len() as u64);
            self.console_buffer.extend_from_slice(data);
            if self.console_callback.is_some() {
                return;
            }
        }
        print!("{}", String::from_utf8_lossy(data));
    }

    fn print_end_line(&mut self) {
        #[cfg(feature = "tools")]
        {
            write_var_uint(&mut self.console_buffer, 0);
            if let Some(callback) = self.console_callback {
                if !self.called_console_callback {
                    self.called_console_callback = true;
                    callback();
                }
                return;
            }
        }
        println!();
    }

    fn print_error(&mut self, state: &mut LuaState) {
        if let Some(error) = state.to_string(-1) {
            eprintln!("{error}");
            #[cfg(feature = "tools")]
            {
                self.print_begin_line(state);
                self.print(error.as_bytes());
                self.print_end_line();
            }
        }
    }

    fn p_call(&mut self, state: &mut LuaState, argument_count: i32, result_count: i32) -> i32 {
        let handler_position = state.top() - argument_count;
        state.push_function(rive_lua_error_handler);
        state.insert(handler_position);
        self.start_timed_execution(state);
        let result = state.pcall(argument_count, result_count, handler_position);
        self.end_timed_execution(state);
        state.remove(handler_position);
        result
    }
}

#[cfg(feature = "tools")]
fn write_var_uint(output: &mut Vec<u8>, mut value: u64) {
    loop {
        let byte = (value & 0x7f) as u8;
        value >>= 7;
        output.push(byte | u8::from(value != 0) * 0x80);
        if value == 0 {
            break;
        }
    }
}

pub fn interrupt_cpp(state: &mut LuaState, gc: i32) {
    if gc >= 0 || !state.is_yieldable() {
        return;
    }
    let context = state.thread_data::<CPPRuntimeScriptingContext>();
    let elapsed = context.execution_time.elapsed();
    let timeout_ms = context.timeout_ms();
    if elapsed > Duration::from_millis(timeout_ms.max(0) as u64) {
        state.clear_interrupt_callback();
        state.raw_check_stack(1);
        let message = if timeout_ms >= 1_000 {
            let seconds = timeout_ms as f64 / 1_000.0;
            format!(
                "execution exceeded {seconds:.1} second{} timeout",
                if seconds == 1.0 { "" } else { "s" }
            )
        } else {
            format!(
                "execution exceeded {timeout_ms} millisecond{} timeout",
                if timeout_ms == 1 { "" } else { "s" }
            )
        };
        state.error::<()>(&message);
    }
}

const REGISTERED_CACHE_TABLE_KEY: &str = "_MODULES";

pub fn luaopen_rive(state: &mut LuaState) -> i32 {
    state.set_user_atom_callback(find_atom);
    let libraries: &[(&str, LuaFunction)] = &[
        ("", state.open_base_function()),
        ("table", state.open_table_function()),
        ("math", state.open_math_function()),
        ("rive", super::lua_rive_base::luaopen_rive_base),
        ("os", state.open_os_function()),
        ("string", state.open_string_function()),
        ("utf8", state.open_utf8_function()),
        ("buffer", state.open_buffer_function()),
        ("bit32", state.open_bit32_function()),
        ("math", super::math::lua_math::luaopen_rive_math),
        (
            "renderer",
            super::renderer::lua_renderer_library::luaopen_rive_renderer_library,
        ),
        ("properties", super::lua_properties::luaopen_rive_properties),
        ("artboard", super::lua_artboards::luaopen_rive_artboards),
        ("dataValue", super::lua_data_value::luaopen_rive_data_values),
        ("input", super::math::lua_input::luaopen_rive_input),
        ("context", super::lua_scripted_context::luaopen_rive_contex),
        (
            "dataContext",
            super::lua_data_context::luaopen_rive_data_context,
        ),
        ("audio", super::lua_audio::luaopen_rive_audio),
        ("promise", super::lua_promise::luaopen_rive_promise),
    ];
    for (name, open) in libraries {
        state.push_function(*open);
        state.push_string(name);
        state.call(1, 0);
    }
    super::lua_buffer_ext::luaopen_rive_buffer_ext(state);
    0
}

pub fn rive_lua_error_handler(state: &mut LuaState) -> i32 {
    let context = state.thread_data_mut::<dyn ScriptingContext>();
    context.print_error(state);
    if let Some(error) = state.to_string(-1) {
        state.push_string(&error);
    } else {
        state.push_nil();
    }
    1
}

pub fn rive_lua_pcall(state: &mut LuaState, argument_count: i32, result_count: i32) -> i32 {
    let context = state.thread_data_mut::<dyn ScriptingContext>();
    let result = context.p_call(state, argument_count, result_count);
    rive_lua_close_orphan_render_pass(state);
    result
}

pub fn rive_lua_pcall_with_context(
    state: &mut LuaState,
    scripted_object: CoreHandle,
    argument_count: i32,
    result_count: i32,
) -> i32 {
    let context = state.thread_data_mut::<dyn ScriptingContext>();
    let _scope = ScopedScriptedObjectContext::new(Some(context), Some(scripted_object));
    let result = context.p_call(state, argument_count, result_count);
    rive_lua_close_orphan_render_pass(state);
    result
}

pub fn rive_lua_push_ref(state: &mut LuaState, reference: i32) -> LuaType {
    state.check_stack(1);
    state.raw_get_i(LUA_REGISTRY_INDEX, reference)
}

pub fn rive_lua_pop(state: &mut LuaState, count: i32) {
    state.set_top(-count - 1);
}

pub(crate) fn check_registered_modules(state: &mut LuaState, path: &str) -> bool {
    state.find_table(LUA_REGISTRY_INDEX, REGISTERED_CACHE_TABLE_KEY, 1);
    state.get_field(-1, path);
    if state.is_nil(-1) {
        state.pop(2);
        false
    } else {
        state.remove(-2);
        true
    }
}

pub(crate) fn lua_require_internal(state: &mut LuaState, requirer_chunk_name: Option<&str>) -> i32 {
    state.set_top(1);
    let path = state.check_string(1);
    if check_registered_modules(state, &path) {
        return 1;
    }
    if let Some(requirer) = requirer_chunk_name {
        state
            .thread_data_mut::<dyn ScriptingContext>()
            .record_missing_dependency(requirer, &path);
    }
    state.error(format!("require could not find a script named {path}"))
}

pub(crate) fn lua_require(state: &mut LuaState) -> i32 {
    let Some(chunk_name) = state.calling_chunk_name_skipping_c_frames() else {
        return state.error("require is not supported in this context");
    };
    lua_require_internal(state, Some(&chunk_name))
}

pub(crate) fn lua_runtime_error(state: &mut LuaState) -> i32 {
    let level = state.optional_integer(2, 1);
    state.set_top(1);
    if state.is_string(1) && level > 0 {
        state.push_where(level);
        state.push_value(1);
        state.concat(2);
    }
    state.raise_error()
}

pub(crate) fn lua_late(state: &mut LuaState) -> i32 {
    state.push_nil();
    1
}

pub fn dump_stack(state: &mut LuaState) {
    for index in 1..=state.top() {
        match state.value_type(index) {
            LuaType::String => eprintln!(
                "  ({index})[STRING] {}",
                state.to_string(index).unwrap_or_default()
            ),
            LuaType::Boolean => eprintln!(
                "  ({index})[BOOLEAN] {}",
                if state.to_boolean(index) {
                    "true"
                } else {
                    "false"
                }
            ),
            LuaType::Number => eprintln!(
                "  ({index})[NUMBER] {}",
                state.to_number(index).unwrap_or_default()
            ),
            value_type => eprintln!("  ({index})[{}]", state.type_name(value_type)),
        }
    }
    eprintln!();
}

pub fn lua_check_vec2d(state: &mut LuaState, stack: i32) -> &Vec2D {
    state.check_vec2d(stack)
}

pub fn lua_to_vec2d(state: &mut LuaState, stack: i32) -> Option<&Vec2D> {
    state.to_vec2d(stack)
}

pub fn lua_push_vec2d(state: &mut LuaState, value: Vec2D) {
    state.push_vector2(value.x, value.y);
}
