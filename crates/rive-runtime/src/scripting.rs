use std::cell::{RefCell, RefMut};
use std::rc::Rc;
use std::{error::Error, fmt};

use rive_render_api::{Factory as RenderFactory, Renderer};

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

impl ScriptValue {
    pub fn as_number(&self) -> Option<f64> {
        match self {
            ScriptValue::Number(value) => Some(*value),
            _ => None,
        }
    }
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
