use std::cell::{RefCell, RefMut};
use std::collections::BTreeMap;
use std::rc::Rc;
use std::{error::Error, fmt};

use rive_binary::{RuntimeFile, RuntimeObject};
use rive_graph::{ArtboardGraph, ShapePaintKind, ShapePaintNode, ShapePaintStateNode};
use rive_render_api::{
    BlendMode, Factory as RenderFactory, RawPath, RenderPaintStyle, Renderer, StrokeCap, StrokeJoin,
};

use crate::properties::property_key_for_name;
use crate::{ArtboardInstance, LinearAnimationInstance, RuntimeOwnedViewModelInstance};

/// Runtime-owned scripting error type.
///
/// The concrete VM crate maps its native error into this type so
/// `rive-runtime` can keep the scripting seam free of VM dependencies.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptError {
    message: String,
}

impl ScriptError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    pub fn message(&self) -> &str {
        &self.message
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

/// Lifecycle/input methods carried by scripted object instance tables.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScriptMethod {
    Init,
    Resize,
    Advance,
    Update,
    Draw,
    PointerDown,
    PointerMove,
    PointerUp,
    PointerEnter,
    PointerExit,
}

impl ScriptMethod {
    pub fn as_str(self) -> &'static str {
        match self {
            ScriptMethod::Init => "init",
            ScriptMethod::Resize => "resize",
            ScriptMethod::Advance => "advance",
            ScriptMethod::Update => "update",
            ScriptMethod::Draw => "draw",
            ScriptMethod::PointerDown => "pointerDown",
            ScriptMethod::PointerMove => "pointerMove",
            ScriptMethod::PointerUp => "pointerUp",
            ScriptMethod::PointerEnter => "pointerEnter",
            ScriptMethod::PointerExit => "pointerExit",
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
    instance: Rc<RefCell<RuntimeOwnedViewModelInstance>>,
    file: Rc<RuntimeFile>,
    view_model_index: usize,
    ancestors: Rc<Vec<usize>>,
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

    pub fn owned_instance(&self) -> Rc<RefCell<RuntimeOwnedViewModelInstance>> {
        Rc::clone(&self.instance)
    }

    pub fn number(&self, name: &str) -> Option<f32> {
        self.instance.borrow().number_value_by_property_name(name)
    }

    pub fn set_number(&self, name: &str, value: f32) -> bool {
        self.instance
            .borrow_mut()
            .set_number_by_property_name(name, value)
    }

    pub fn string(&self, name: &str) -> Option<String> {
        self.instance
            .borrow()
            .string_value_by_property_name(name)
            .map(|value| String::from_utf8_lossy(value).into_owned())
    }

    pub fn set_string(&self, name: &str, value: &str) -> bool {
        self.instance
            .borrow_mut()
            .set_string_by_property_name(name, value.as_bytes())
    }

    pub fn boolean(&self, name: &str) -> Option<bool> {
        self.instance.borrow().boolean_value_by_property_name(name)
    }

    pub fn set_boolean(&self, name: &str, value: bool) -> bool {
        self.instance
            .borrow_mut()
            .set_boolean_by_property_name(name, value)
    }

    pub fn view_model(&self, name: &str) -> Option<Self> {
        self.nested_view_models.get(name).cloned()
    }

    pub fn list_len(&self, name: &str) -> Option<usize> {
        self.instance
            .borrow()
            .list_items_by_property_name(name)
            .map(|items| items.len())
    }

    pub fn list_item(&self, name: &str, index: usize) -> Option<Self> {
        let item = self
            .instance
            .borrow()
            .list_items_by_property_name(name)?
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
        self.instance
            .borrow_mut()
            .push_list_item_by_property_name(name, Rc::clone(&item.instance))
    }

    pub fn insert_list_item(&self, name: &str, index: usize, item: &ScriptViewModel) -> bool {
        self.instance
            .borrow_mut()
            .insert_list_item_by_property_name(name, index, Rc::clone(&item.instance))
    }

    pub fn pop_list_item(&self, name: &str) -> Option<Self> {
        let item = self
            .instance
            .borrow_mut()
            .pop_list_item_by_property_name(name)?;
        let view_model_index = item.borrow().view_model_index();
        build_script_view_model_shared(
            Rc::clone(&self.file),
            view_model_index,
            item,
            self.ancestors.as_slice(),
        )
    }

    pub fn shift_list_item(&self, name: &str) -> Option<Self> {
        let item = self
            .instance
            .borrow_mut()
            .shift_list_item_by_property_name(name)?;
        let view_model_index = item.borrow().view_model_index();
        build_script_view_model_shared(
            Rc::clone(&self.file),
            view_model_index,
            item,
            self.ancestors.as_slice(),
        )
    }

    pub fn swap_list_items(&self, name: &str, first: usize, second: usize) -> bool {
        self.instance
            .borrow_mut()
            .swap_list_items_by_property_name(name, first, second)
    }

    pub fn clear_list_items(&self, name: &str) -> bool {
        self.instance
            .borrow_mut()
            .clear_list_items_by_property_name(name)
    }

    pub fn remove_list_item_at(&self, name: &str, index: usize) -> bool {
        self.instance
            .borrow_mut()
            .remove_list_item_at_by_property_name(name, index)
    }

    pub fn remove_list_item(&self, name: &str, item: &ScriptViewModel, remove_all: bool) -> bool {
        self.instance
            .borrow_mut()
            .remove_list_items_by_identity(name, &item.instance, remove_all)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptViewModelProperty {
    Number,
    String,
    Boolean,
    Trigger,
    List,
    ViewModel,
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
    instance: &RuntimeOwnedViewModelInstance,
) -> Option<ScriptViewModel> {
    let view_model_index = instance.view_model_index();
    build_script_view_model(
        Rc::new(file.clone()),
        view_model_index,
        instance.clone(),
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
        Rc::new(RefCell::new(instance)),
        ancestors,
    )
}

fn build_script_view_model_shared(
    file: Rc<RuntimeFile>,
    view_model_index: usize,
    instance: Rc<RefCell<RuntimeOwnedViewModelInstance>>,
    ancestors: &[usize],
) -> Option<ScriptViewModel> {
    let view_model = file.view_model(view_model_index)?;
    let properties = view_model
        .properties
        .iter()
        .filter_map(|property| {
            let kind = match property.type_name {
                "ViewModelPropertyNumber" => ScriptViewModelProperty::Number,
                "ViewModelPropertyString" => ScriptViewModelProperty::String,
                "ViewModelPropertyBoolean" => ScriptViewModelProperty::Boolean,
                "ViewModelPropertyTrigger" => ScriptViewModelProperty::Trigger,
                "ViewModelPropertyList" => ScriptViewModelProperty::List,
                "ViewModelPropertyViewModel" => ScriptViewModelProperty::ViewModel,
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
        .filter(|property| property.type_name == "ViewModelPropertyViewModel")
        .filter_map(|property| {
            let name = property.string_property("name")?.to_owned();
            let (nested_index, instance_index) = instance
                .borrow()
                .nested_view_model_selection_by_property_name(&name)?;
            if child_ancestors.contains(&nested_index) {
                return None;
            }
            let nested_instance = match instance_index {
                Some(instance_index) => RuntimeOwnedViewModelInstance::from_instance(
                    &file,
                    nested_index,
                    instance_index,
                )?,
                None => RuntimeOwnedViewModelInstance::new(&file, nested_index)?,
            };
            Some((
                name,
                build_script_view_model(
                    Rc::clone(&file),
                    nested_index,
                    nested_instance,
                    &child_ancestors,
                )?,
            ))
        })
        .collect();
    Some(ScriptViewModel {
        properties,
        nested_view_models,
        instance,
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
) -> Option<ScriptValue> {
    let property_key = property_key_for_name(input.type_name, "propertyValue")?;
    let data_bind = (0..file.object_count()).find_map(|id| {
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
    })?;
    let source_path = file.data_bind_context_resolved_source_path_ids_for_object(data_bind)?;
    let name_based = file
        .data_bind_is_name_based_for_object(data_bind)
        .unwrap_or(false);

    match input.type_name {
        "ScriptInputBoolean" => context
            .boolean_value_by_context_source_path(file, &[], &source_path, name_based)
            .map(ScriptValue::Bool),
        "ScriptInputNumber" => context
            .number_value_by_context_source_path(file, &[], &source_path, name_based)
            .map(|value| ScriptValue::Number(f64::from(value))),
        "ScriptInputColor" => context
            .color_value_by_context_source_path(file, &[], &source_path, name_based)
            .map(|value| ScriptValue::Number(value as f64)),
        "ScriptInputString" => context
            .string_value_by_context_source_path(file, &[], &source_path, name_based)
            .map(|value| ScriptValue::String(String::from_utf8_lossy(value).into_owned())),
        _ => None,
    }
}

/// Resolves a `ScriptInputViewModelProperty` after its scripted object has a
/// data context. C++ treats hydration as all-or-nothing, so `None` means the
/// caller must defer every input and user `init`, not install a nil stand-in.
pub fn bound_script_view_model(
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
