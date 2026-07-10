use std::cell::{RefCell, RefMut};
use std::collections::BTreeMap;
use std::rc::Rc;
use std::{error::Error, fmt};

use rive_binary::{RuntimeFile, RuntimeObject};
use rive_render_api::{Factory as RenderFactory, Renderer};

use crate::RuntimeOwnedViewModelInstance;
use crate::properties::property_key_for_name;

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
    Vec2 { x: f32, y: f32 },
    Vec3 { x: f32, y: f32, z: f32 },
}

/// A runtime-neutral snapshot of a bound view-model exposed to Luau.
///
/// Ported from the lookup shape in C++ `src/lua/lua_properties.cpp` and
/// `src/script_input_viewmodel_property.cpp`. Property variants are added as
/// their corpus-backed bindings land; unknown properties remain absent in
/// Luau, matching C++ lookup failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptViewModel {
    properties: BTreeMap<String, ScriptViewModelProperty>,
}

impl ScriptViewModel {
    pub fn new(properties: BTreeMap<String, ScriptViewModelProperty>) -> Self {
        Self { properties }
    }

    pub fn property(&self, name: &str) -> Option<ScriptViewModelProperty> {
        self.properties.get(name).copied()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptViewModelProperty {
    Trigger,
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
    let properties = file
        .view_model(view_model_index)?
        .properties
        .into_iter()
        .filter_map(|property| {
            let kind = match property.type_name {
                "ViewModelPropertyTrigger" => ScriptViewModelProperty::Trigger,
                _ => return None,
            };
            Some((property.string_property("name")?.to_owned(), kind))
        })
        .collect();
    Some(ScriptViewModel::new(properties))
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

/// Runtime-owned artboard userdata exposed to scripts.
pub trait ScriptArtboard {
    fn width(&self) -> f32;
    fn height(&self) -> f32;
    fn frame_origin(&self) -> bool;
    fn set_width(&mut self, width: f32);
    fn set_height(&mut self, height: f32);
    fn set_frame_origin(&mut self, frame_origin: bool);

    fn instance(&self) -> Result<Box<dyn ScriptArtboard>, ScriptError>;

    fn draw(
        &mut self,
        factory: &mut dyn RenderFactory,
        renderer: &mut dyn Renderer,
    ) -> Result<(), ScriptError>;
}

/// Runtime-owned handle for one scripted object instance.
pub trait ScriptInstance {
    fn has_method(&self, method: ScriptMethod) -> Result<bool, ScriptError>;

    fn call_method(
        &mut self,
        method: ScriptMethod,
        args: &[ScriptValue],
        host: &mut dyn ScriptHost,
    ) -> Result<ScriptValue, ScriptError>;

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
