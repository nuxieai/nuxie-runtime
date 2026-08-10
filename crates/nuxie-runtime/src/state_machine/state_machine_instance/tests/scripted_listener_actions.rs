use super::*;
use crate::state_machine::focus_action_clear::RuntimeFocusActionClear;
use crate::state_machine::focus_action_target::RuntimeFocusActionTarget;
use crate::state_machine::focus_listener_group::RuntimeFocusListenerGroup;
use crate::state_machine::gamepad_listener_group::RuntimeGamepadListenerGroup;
use crate::state_machine::keyboard_listener_group::RuntimeKeyboardListenerGroup;
use nuxie_binary::{
    FixtureProperty, FixtureRecord, FixtureValue, RuntimeFile, read_runtime_file,
};
use nuxie_graph::GraphFile;
use std::cell::{Cell, RefCell};
use std::path::PathBuf;
use std::rc::Rc;

#[derive(Default)]
struct ProfilerListenerCapture {
    tick: u64,
}

impl crate::ProfileCapture for ProfilerListenerCapture {
    fn tick(&mut self) -> u64 {
        let tick = self.tick;
        self.tick += 1;
        tick
    }

    fn metadata(&self) -> crate::ProfileCaptureMetadata {
        crate::ProfileCaptureMetadata::default()
    }

    fn current_frame_index(&self) -> u64 {
        0
    }

    fn gpu_frame_delay(&self) -> u64 {
        1
    }

    fn max_frame_history(&self) -> u64 {
        8
    }

    fn captured_frame(&self, _frame_index: u64) -> Option<crate::ProfileCaptureFrame> {
        None
    }
}

#[derive(Debug, Clone, PartialEq)]
struct RecordedCall {
    label: &'static str,
    method: ScriptListenerActionMethod,
    invocation: ScriptListenerInvocation,
    state_before_call: usize,
}

struct RecordingListenerScript {
    label: &'static str,
    has_perform_action: bool,
    has_perform: bool,
    failure: ListenerFailure,
    state: usize,
    calls: Rc<RefCell<Vec<RecordedCall>>>,
}

struct ReportingViewModelListenerScript {
    label: &'static str,
    queue: RuntimeCellNotificationQueue,
    listener_index: usize,
    calls: Rc<RefCell<Vec<RecordedCall>>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ListenerFailure {
    None,
    Ordinary,
    Terminal(&'static str),
}

struct AtomicScriptHost;

impl crate::ScriptHost for AtomicScriptHost {
    fn requires_atomic_script_callbacks(&self) -> bool {
        true
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProjectionFailure {
    Ordinary,
    Resource,
}

struct InputProjectionScript {
    scalar_values: Rc<RefCell<Vec<(String, ScriptValue)>>>,
    trigger_calls: Rc<Cell<usize>>,
    trigger_failure: ProjectionFailure,
    artboard_widths: Rc<RefCell<Vec<f32>>>,
    lifetime_valid: bool,
}

impl ScriptInstance for InputProjectionScript {
    fn script_lifetime_valid(&self) -> bool {
        self.lifetime_valid
    }

    fn has_method(&self, _method: ScriptMethod) -> Result<bool, ScriptError> {
        Ok(false)
    }

    fn call_method(
        &mut self,
        _method: ScriptMethod,
        _args: &[ScriptValue],
        _host: &mut dyn ScriptHost,
    ) -> Result<ScriptValue, ScriptError> {
        Ok(ScriptValue::Nil)
    }

    fn call_input_trigger(
        &mut self,
        _name: &str,
        _host: &mut dyn ScriptHost,
    ) -> Result<(), ScriptError> {
        self.trigger_calls.set(self.trigger_calls.get() + 1);
        match self.trigger_failure {
            ProjectionFailure::Ordinary => Err(ScriptError::new("ordinary trigger failure")),
            ProjectionFailure::Resource => Err(ScriptError::with_resource_code(
                "terminal trigger resource failure",
                "script.resource.test",
            )),
        }
    }

    fn get_input(&self, _name: &str) -> Result<ScriptValue, ScriptError> {
        Ok(ScriptValue::Nil)
    }

    fn set_input(&mut self, name: &str, value: ScriptValue) -> Result<(), ScriptError> {
        self.scalar_values
            .borrow_mut()
            .push((name.to_owned(), value));
        Ok(())
    }

    fn set_artboard_input(
        &mut self,
        _name: &str,
        artboard: Box<dyn crate::ScriptArtboard>,
    ) -> Result<(), ScriptError> {
        self.artboard_widths.borrow_mut().push(artboard.width());
        Ok(())
    }
}

#[derive(Debug, Clone)]
struct ProjectionArtboard {
    width: f32,
}

impl crate::ScriptArtboard for ProjectionArtboard {
    fn width(&self) -> f32 {
        self.width
    }

    fn height(&self) -> f32 {
        1.0
    }

    fn frame_origin(&self) -> bool {
        false
    }

    fn set_width(&mut self, width: f32) {
        self.width = width;
    }

    fn set_height(&mut self, _height: f32) {}

    fn set_frame_origin(&mut self, _frame_origin: bool) {}

    fn instance(
        &self,
        _view_model: Option<crate::ScriptViewModel>,
    ) -> Result<Box<dyn crate::ScriptArtboard>, ScriptError> {
        Ok(Box::new(self.clone()))
    }

    fn draw(
        &mut self,
        _factory: &mut dyn nuxie_render_api::Factory,
        _renderer: &mut dyn nuxie_render_api::Renderer,
    ) -> Result<(), ScriptError> {
        Ok(())
    }
}

#[derive(Debug)]
struct ProjectionArtboardResolver;

impl ScriptArtboardResolver for ProjectionArtboardResolver {
    fn resolve_script_artboard(
        &self,
        artboard_id: u64,
        _parent_context: Option<&crate::ScriptArtboardParentContext>,
    ) -> Result<Box<dyn crate::ScriptArtboard>, ScriptError> {
        match artboard_id {
            7 => Ok(Box::new(ProjectionArtboard { width: 7.0 })),
            8 => Err(ScriptError::new("ordinary missing artboard")),
            _ => Err(ScriptError::with_resource_code(
                "terminal artboard resource failure",
                "script.resource.test",
            )),
        }
    }
}

struct HydrationTraceScript {
    trace: Rc<RefCell<Vec<String>>>,
    artboard_applied: Rc<Cell<bool>>,
}

impl ScriptInstance for HydrationTraceScript {
    fn set_context_view_model_chain(
        &mut self,
        _view_model: Option<ScriptViewModel>,
        _parents: Vec<Option<ScriptViewModel>>,
    ) -> Result<(), ScriptError> {
        self.trace.borrow_mut().push("context".to_owned());
        Ok(())
    }

    fn has_method(&self, _method: ScriptMethod) -> Result<bool, ScriptError> {
        Ok(false)
    }

    fn call_method(
        &mut self,
        method: ScriptMethod,
        _args: &[ScriptValue],
        _host: &mut dyn ScriptHost,
    ) -> Result<ScriptValue, ScriptError> {
        if method == ScriptMethod::Init {
            self.trace.borrow_mut().push("init".to_owned());
            return Ok(ScriptValue::Bool(true));
        }
        Ok(ScriptValue::Nil)
    }

    fn user_init_pending(&mut self) -> Result<bool, ScriptError> {
        self.trace.borrow_mut().push("init-check".to_owned());
        Ok(true)
    }

    fn get_input(&self, _name: &str) -> Result<ScriptValue, ScriptError> {
        Ok(ScriptValue::Nil)
    }

    fn set_input(&mut self, name: &str, _value: ScriptValue) -> Result<(), ScriptError> {
        self.trace.borrow_mut().push(format!("set:{name}"));
        Ok(())
    }

    fn set_artboard_input(
        &mut self,
        name: &str,
        _artboard: Box<dyn crate::ScriptArtboard>,
    ) -> Result<(), ScriptError> {
        self.trace.borrow_mut().push(format!("set-artboard:{name}"));
        self.artboard_applied.set(true);
        Ok(())
    }
}

#[derive(Debug)]
struct HydrationArtboardResolver {
    trace: Rc<RefCell<Vec<String>>>,
}

impl ScriptArtboardResolver for HydrationArtboardResolver {
    fn resolve_script_artboard(
        &self,
        artboard_id: u64,
        _parent_context: Option<&crate::ScriptArtboardParentContext>,
    ) -> Result<Box<dyn crate::ScriptArtboard>, ScriptError> {
        self.trace
            .borrow_mut()
            .push(format!("resolve-artboard:{artboard_id}"));
        if artboard_id == 7 {
            Ok(Box::new(ProjectionArtboard { width: 7.0 }))
        } else {
            Err(ScriptError::with_resource_code(
                "terminal artboard resource failure",
                "script.resource.test",
            ))
        }
    }
}

#[derive(Debug)]
struct AfterArtboardViewModelResolver {
    trace: Rc<RefCell<Vec<String>>>,
    artboard_applied: Rc<Cell<bool>>,
}

impl crate::ScriptViewModelInputResolver for AfterArtboardViewModelResolver {
    fn resolve_script_view_model(
        &self,
        _input_global_id: u32,
        _path: &crate::ScriptInputViewModelPropertyPath,
    ) -> Result<Option<ScriptViewModel>, ScriptError> {
        self.trace
            .borrow_mut()
            .push("resolve-view-model".to_owned());
        assert!(
            self.artboard_applied.get(),
            "the earlier authored Artboard setter must run before the later ViewModel lookup"
        );
        Err(ScriptError::new("intentional late ViewModel miss"))
    }
}

#[derive(Debug)]
struct NullViewModelResolver {
    trace: Rc<RefCell<Vec<String>>>,
}

impl crate::ScriptViewModelInputResolver for NullViewModelResolver {
    fn resolve_script_view_model(
        &self,
        _input_global_id: u32,
        _path: &crate::ScriptInputViewModelPropertyPath,
    ) -> Result<Option<ScriptViewModel>, ScriptError> {
        self.trace
            .borrow_mut()
            .push("resolve-null-view-model".to_owned());
        Ok(None)
    }
}

fn hydration_trace_machine(
    trace: &Rc<RefCell<Vec<String>>>,
    artboard_applied: &Rc<Cell<bool>>,
) -> (StateMachineInstance, u32) {
    let mut machine = scripted_listener_machine();
    let action_global_id = machine
        .scripted_listener_actions()
        .first()
        .expect("scripted listener fixture action")
        .action_global_id();
    machine
        .set_scripted_listener_action_instance(
            action_global_id,
            Box::new(HydrationTraceScript {
                trace: Rc::clone(trace),
                artboard_applied: Rc::clone(artboard_applied),
            }),
        )
        .expect("attach hydration trace script");
    (machine, action_global_id)
}

#[derive(Debug, Clone, PartialEq)]
struct RecordedDrawableInputCall {
    label: &'static str,
    invocation: ScriptListenerInvocation,
}

struct RecordingDrawableInputScript {
    label: &'static str,
    methods: Vec<ScriptMethod>,
    handled: bool,
    calls: Rc<RefCell<Vec<RecordedDrawableInputCall>>>,
}

#[derive(Debug, Clone, PartialEq)]
struct RecordedDrawablePointerCall {
    method: ScriptMethod,
    pointer_id: i32,
    local_x: f32,
    local_y: f32,
}

struct RecordingDrawablePointerScript {
    hit: crate::ScriptedDrawablePointerHit,
    calls: Rc<RefCell<Vec<RecordedDrawablePointerCall>>>,
}

struct ResourceFailingDrawablePointerScript;

impl ScriptInstance for ResourceFailingDrawablePointerScript {
    fn has_method(&self, method: ScriptMethod) -> Result<bool, ScriptError> {
        Ok(method == ScriptMethod::PointerDown)
    }

    fn call_method(
        &mut self,
        _method: ScriptMethod,
        _args: &[crate::ScriptValue],
        _host: &mut dyn ScriptHost,
    ) -> Result<crate::ScriptValue, ScriptError> {
        unreachable!("typed scripted-drawable pointer dispatch owns this callback")
    }

    fn call_scripted_drawable_pointer(
        &mut self,
        _method: ScriptMethod,
        _pointer_id: i32,
        _local_x: f32,
        _local_y: f32,
        _host: &mut dyn ScriptHost,
    ) -> Result<crate::ScriptedDrawablePointerResult, ScriptError> {
        Err(ScriptError::with_resource_code(
            "terminal pointer resource fence",
            "script.resource.pointer",
        ))
    }

    fn get_input(&self, _name: &str) -> Result<crate::ScriptValue, ScriptError> {
        Ok(crate::ScriptValue::Nil)
    }

    fn set_input(&mut self, _name: &str, _value: crate::ScriptValue) -> Result<(), ScriptError> {
        Ok(())
    }
}

impl ScriptInstance for RecordingDrawablePointerScript {
    fn has_method(&self, method: ScriptMethod) -> Result<bool, ScriptError> {
        Ok(matches!(
            method,
            ScriptMethod::PointerDown
                | ScriptMethod::PointerMove
                | ScriptMethod::PointerUp
                | ScriptMethod::PointerExit
        ))
    }

    fn call_method(
        &mut self,
        _method: ScriptMethod,
        _args: &[crate::ScriptValue],
        _host: &mut dyn ScriptHost,
    ) -> Result<crate::ScriptValue, ScriptError> {
        unreachable!("typed scripted-drawable pointer dispatch owns this callback")
    }

    fn call_scripted_drawable_pointer(
        &mut self,
        method: ScriptMethod,
        pointer_id: i32,
        local_x: f32,
        local_y: f32,
        _host: &mut dyn ScriptHost,
    ) -> Result<crate::ScriptedDrawablePointerResult, ScriptError> {
        self.calls.borrow_mut().push(RecordedDrawablePointerCall {
            method,
            pointer_id,
            local_x,
            local_y,
        });
        Ok(crate::ScriptedDrawablePointerResult {
            invoked: true,
            hit: self.hit,
        })
    }

    fn get_input(&self, _name: &str) -> Result<crate::ScriptValue, ScriptError> {
        Ok(crate::ScriptValue::Nil)
    }

    fn set_input(&mut self, _name: &str, _value: crate::ScriptValue) -> Result<(), ScriptError> {
        Ok(())
    }
}

fn scripted_input_method(invocation: &ScriptListenerInvocation) -> Option<ScriptMethod> {
    match invocation {
        ScriptListenerInvocation::Keyboard { .. } => Some(ScriptMethod::KeyboardEvent),
        ScriptListenerInvocation::TextInput { .. } => Some(ScriptMethod::TextEvent),
        ScriptListenerInvocation::GamepadConnected { .. } => Some(ScriptMethod::GamepadConnected),
        ScriptListenerInvocation::GamepadEvent { .. } => Some(ScriptMethod::GamepadEvent),
        ScriptListenerInvocation::GamepadDisconnected { .. } => {
            Some(ScriptMethod::GamepadDisconnected)
        }
        ScriptListenerInvocation::Pointer { .. }
        | ScriptListenerInvocation::Focus { .. }
        | ScriptListenerInvocation::ReportedEvent { .. }
        | ScriptListenerInvocation::ViewModelChange { .. }
        | ScriptListenerInvocation::None
        | ScriptListenerInvocation::Semantic { .. } => None,
    }
}

struct ResourceFailingDrawableInputScript;

impl ScriptInstance for ResourceFailingDrawableInputScript {
    fn has_method(&self, method: ScriptMethod) -> Result<bool, ScriptError> {
        Ok(method == ScriptMethod::KeyboardEvent)
    }

    fn call_method(
        &mut self,
        _method: ScriptMethod,
        _args: &[crate::ScriptValue],
        _host: &mut dyn ScriptHost,
    ) -> Result<crate::ScriptValue, ScriptError> {
        unreachable!("typed direct-input dispatch owns this callback")
    }

    fn call_scripted_drawable_input(
        &mut self,
        invocation: &ScriptListenerInvocation,
        _host: &mut dyn ScriptHost,
    ) -> Result<crate::ScriptedDrawableInputResult, ScriptError> {
        if scripted_input_method(invocation) != Some(ScriptMethod::KeyboardEvent) {
            return Ok(crate::ScriptedDrawableInputResult::default());
        }
        Err(ScriptError::with_resource_code(
            "script cycle exceeds 256 host commands",
            "script.resource.host_commands",
        ))
    }

    fn get_input(&self, _name: &str) -> Result<crate::ScriptValue, ScriptError> {
        Ok(crate::ScriptValue::Nil)
    }

    fn set_input(&mut self, _name: &str, _value: crate::ScriptValue) -> Result<(), ScriptError> {
        Ok(())
    }
}

struct FailingDrawableInputScript {
    label: &'static str,
    methods: Vec<ScriptMethod>,
    resource_code: Option<&'static str>,
    calls: Rc<RefCell<Vec<RecordedDrawableInputCall>>>,
}

impl ScriptInstance for FailingDrawableInputScript {
    fn has_method(&self, method: ScriptMethod) -> Result<bool, ScriptError> {
        Ok(self.methods.contains(&method))
    }

    fn call_method(
        &mut self,
        _method: ScriptMethod,
        _args: &[crate::ScriptValue],
        _host: &mut dyn ScriptHost,
    ) -> Result<crate::ScriptValue, ScriptError> {
        unreachable!("typed direct-input dispatch owns this callback")
    }

    fn call_scripted_drawable_input(
        &mut self,
        invocation: &ScriptListenerInvocation,
        _host: &mut dyn ScriptHost,
    ) -> Result<crate::ScriptedDrawableInputResult, ScriptError> {
        if !scripted_input_method(invocation).is_some_and(|method| self.methods.contains(&method)) {
            return Ok(crate::ScriptedDrawableInputResult::default());
        }
        self.calls.borrow_mut().push(RecordedDrawableInputCall {
            label: self.label,
            invocation: invocation.clone(),
        });
        Err(match self.resource_code {
            Some(code) => ScriptError::with_resource_code("terminal resource fence", code),
            None => ScriptError::new("ordinary protected-call failure"),
        })
    }

    fn get_input(&self, _name: &str) -> Result<crate::ScriptValue, ScriptError> {
        Ok(crate::ScriptValue::Nil)
    }

    fn set_input(&mut self, _name: &str, _value: crate::ScriptValue) -> Result<(), ScriptError> {
        Ok(())
    }
}

impl ScriptInstance for RecordingDrawableInputScript {
    fn has_method(&self, method: ScriptMethod) -> Result<bool, ScriptError> {
        Ok(self.methods.contains(&method))
    }

    fn call_method(
        &mut self,
        _method: ScriptMethod,
        _args: &[crate::ScriptValue],
        _host: &mut dyn ScriptHost,
    ) -> Result<crate::ScriptValue, ScriptError> {
        Err(ScriptError::new(
            "scripted drawable input must use the typed invocation seam",
        ))
    }

    fn call_scripted_drawable_input(
        &mut self,
        invocation: &ScriptListenerInvocation,
        _host: &mut dyn ScriptHost,
    ) -> Result<crate::ScriptedDrawableInputResult, ScriptError> {
        if !scripted_input_method(invocation).is_some_and(|method| self.methods.contains(&method)) {
            return Ok(crate::ScriptedDrawableInputResult::default());
        }
        self.calls.borrow_mut().push(RecordedDrawableInputCall {
            label: self.label,
            invocation: invocation.clone(),
        });
        let handled = matches!(
            invocation,
            ScriptListenerInvocation::GamepadConnected { .. }
                | ScriptListenerInvocation::GamepadEvent { .. }
                | ScriptListenerInvocation::GamepadDisconnected { .. }
        ) || self.handled;
        Ok(crate::ScriptedDrawableInputResult {
            invoked: true,
            handled,
        })
    }

    fn get_input(&self, _name: &str) -> Result<crate::ScriptValue, ScriptError> {
        Ok(crate::ScriptValue::Nil)
    }

    fn set_input(&mut self, _name: &str, _value: crate::ScriptValue) -> Result<(), ScriptError> {
        Ok(())
    }
}

impl ScriptInstance for RecordingListenerScript {
    fn has_method(&self, method: ScriptMethod) -> Result<bool, ScriptError> {
        Ok(match method {
            ScriptMethod::PerformAction => self.has_perform_action,
            ScriptMethod::Perform => self.has_perform,
            _ => false,
        })
    }

    fn call_method(
        &mut self,
        _method: ScriptMethod,
        _args: &[crate::ScriptValue],
        _host: &mut dyn ScriptHost,
    ) -> Result<crate::ScriptValue, ScriptError> {
        Err(ScriptError::new(
            "listener dispatch must use the typed invocation seam",
        ))
    }

    fn call_listener_action(
        &mut self,
        method: ScriptListenerActionMethod,
        invocation: &ScriptListenerInvocation,
        _host: &mut dyn ScriptHost,
    ) -> Result<(), ScriptError> {
        self.calls.borrow_mut().push(RecordedCall {
            label: self.label,
            method,
            invocation: invocation.clone(),
            state_before_call: self.state,
        });
        self.state = self.state.checked_add(1).expect("bounded test call count");
        match self.failure {
            ListenerFailure::None => Ok(()),
            ListenerFailure::Ordinary => Err(ScriptError::new(format!("{} failed", self.label))),
            ListenerFailure::Terminal(code) => Err(ScriptError::with_resource_code(
                format!("{} exhausted a resource", self.label),
                code,
            )),
        }
    }

    fn call_preferred_listener_action(
        &mut self,
        invocation: &ScriptListenerInvocation,
        host: &mut dyn ScriptHost,
    ) -> Result<bool, ScriptError> {
        let method = if self.has_perform_action {
            Some(ScriptListenerActionMethod::PerformAction)
        } else if self.has_perform {
            Some(ScriptListenerActionMethod::Perform)
        } else {
            None
        };
        let Some(method) = method else {
            return Ok(false);
        };
        self.call_listener_action(method, invocation, host)?;
        Ok(true)
    }

    fn get_input(&self, _name: &str) -> Result<crate::ScriptValue, ScriptError> {
        Ok(crate::ScriptValue::Nil)
    }

    fn set_input(&mut self, _name: &str, _value: crate::ScriptValue) -> Result<(), ScriptError> {
        Ok(())
    }
}

impl ScriptInstance for ReportingViewModelListenerScript {
    fn has_method(&self, method: ScriptMethod) -> Result<bool, ScriptError> {
        Ok(method == ScriptMethod::PerformAction)
    }

    fn call_method(
        &mut self,
        _method: ScriptMethod,
        _args: &[crate::ScriptValue],
        _host: &mut dyn ScriptHost,
    ) -> Result<crate::ScriptValue, ScriptError> {
        Err(ScriptError::new(
            "listener dispatch must use the typed invocation seam",
        ))
    }

    fn call_listener_action(
        &mut self,
        method: ScriptListenerActionMethod,
        invocation: &ScriptListenerInvocation,
        _host: &mut dyn ScriptHost,
    ) -> Result<(), ScriptError> {
        self.calls.borrow_mut().push(RecordedCall {
            label: self.label,
            method,
            invocation: invocation.clone(),
            state_before_call: 0,
        });
        self.queue.report_data_bind(self.listener_index);
        Ok(())
    }

    fn call_preferred_listener_action(
        &mut self,
        invocation: &ScriptListenerInvocation,
        host: &mut dyn ScriptHost,
    ) -> Result<bool, ScriptError> {
        self.call_listener_action(ScriptListenerActionMethod::PerformAction, invocation, host)?;
        Ok(true)
    }

    fn get_input(&self, _name: &str) -> Result<crate::ScriptValue, ScriptError> {
        Ok(crate::ScriptValue::Nil)
    }

    fn set_input(&mut self, _name: &str, _value: crate::ScriptValue) -> Result<(), ScriptError> {
        Ok(())
    }
}

#[test]
fn scripted_input_scalar_trigger_and_artboard_projection_failures_match_cpp() {
    let scalar_values = Rc::new(RefCell::new(Vec::new()));
    let artboard_widths = Rc::new(RefCell::new(Vec::new()));
    let ordinary_trigger_calls = Rc::new(Cell::new(0));
    let ordinary = RuntimeScriptInstanceHandle::new(Box::new(InputProjectionScript {
        scalar_values: Rc::clone(&scalar_values),
        trigger_calls: Rc::clone(&ordinary_trigger_calls),
        trigger_failure: ProjectionFailure::Ordinary,
        artboard_widths: Rc::clone(&artboard_widths),
        lifetime_valid: true,
    }));
    let resolver = ProjectionArtboardResolver;
    let mut host = NoopScriptHost;

    for (name, value) in [
        ("enabled", ScriptValue::Bool(true)),
        ("amount", ScriptValue::Number(-0.0)),
        ("tint", ScriptValue::Color(0x1122_3344)),
        ("label", ScriptValue::String("ready".to_owned())),
    ] {
        assert!(
            apply_scripted_input_update(
                &ordinary,
                &ScriptCoreString::from(name),
                crate::state_machine::RuntimeScriptedListenerBoundValue::Value(value),
                Some(&resolver),
                None,
                &mut host,
            )
            .unwrap()
        );
    }
    assert_eq!(scalar_values.borrow().len(), 4);

    assert!(
        !apply_scripted_input_update(
            &ordinary,
            &ScriptCoreString::from("pulse"),
            crate::state_machine::RuntimeScriptedListenerBoundValue::Trigger(0),
            Some(&resolver),
            None,
            &mut host,
        )
        .unwrap(),
        "zero is not a trigger edge"
    );
    assert_eq!(ordinary_trigger_calls.get(), 0);
    assert!(
        !apply_scripted_input_update(
            &ordinary,
            &ScriptCoreString::from("pulse"),
            crate::state_machine::RuntimeScriptedListenerBoundValue::Trigger(1),
            Some(&resolver),
            None,
            &mut host,
        )
        .unwrap(),
        "an ordinary protected-call failure is swallowed and later inputs continue"
    );
    assert_eq!(ordinary_trigger_calls.get(), 1);

    assert!(
        apply_scripted_input_update(
            &ordinary,
            &ScriptCoreString::from("panel"),
            crate::state_machine::RuntimeScriptedListenerBoundValue::Artboard(7),
            Some(&resolver),
            None,
            &mut host,
        )
        .unwrap()
    );
    assert_eq!(&*artboard_widths.borrow(), &[7.0]);
    assert!(
        !apply_scripted_input_update(
            &ordinary,
            &ScriptCoreString::from("panel"),
            crate::state_machine::RuntimeScriptedListenerBoundValue::Artboard(8),
            Some(&resolver),
            None,
            &mut host,
        )
        .unwrap(),
        "an unresolved authored artboard leaves the prior table field untouched"
    );
    assert_eq!(&*artboard_widths.borrow(), &[7.0]);
    assert!(
        apply_scripted_input_update(
            &ordinary,
            &ScriptCoreString::from("after_panel"),
            crate::state_machine::RuntimeScriptedListenerBoundValue::Value(ScriptValue::Number(
                5.0
            ),),
            Some(&resolver),
            None,
            &mut host,
        )
        .unwrap(),
        "an ordinary artboard resolution failure does not abort later authored inputs"
    );
    assert_eq!(scalar_values.borrow().last().unwrap().0, "after_panel");

    let resource_trigger_calls = Rc::new(Cell::new(0));
    let terminal = RuntimeScriptInstanceHandle::new(Box::new(InputProjectionScript {
        scalar_values: Rc::new(RefCell::new(Vec::new())),
        trigger_calls: Rc::clone(&resource_trigger_calls),
        trigger_failure: ProjectionFailure::Resource,
        artboard_widths: Rc::new(RefCell::new(Vec::new())),
        lifetime_valid: true,
    }));
    let trigger_error = apply_scripted_input_update(
        &terminal,
        &ScriptCoreString::from("pulse"),
        crate::state_machine::RuntimeScriptedListenerBoundValue::Trigger(1),
        Some(&resolver),
        None,
        &mut host,
    )
    .expect_err("typed resource failures are the Rust safety fence");
    assert_eq!(trigger_error.resource_code(), Some("script.resource.test"));
    assert_eq!(resource_trigger_calls.get(), 1);
    let artboard_error = apply_scripted_input_update(
        &terminal,
        &ScriptCoreString::from("panel"),
        crate::state_machine::RuntimeScriptedListenerBoundValue::Artboard(9),
        Some(&resolver),
        None,
        &mut host,
    )
    .expect_err("artboard construction resource failures remain terminal");
    assert_eq!(artboard_error.resource_code(), Some("script.resource.test"));

    let invalid_scalar_values = Rc::new(RefCell::new(Vec::new()));
    let invalid_trigger_calls = Rc::new(Cell::new(0));
    let invalid_artboard_widths = Rc::new(RefCell::new(Vec::new()));
    let invalid = RuntimeScriptInstanceHandle::new(Box::new(InputProjectionScript {
        scalar_values: Rc::clone(&invalid_scalar_values),
        trigger_calls: Rc::clone(&invalid_trigger_calls),
        trigger_failure: ProjectionFailure::Ordinary,
        artboard_widths: Rc::clone(&invalid_artboard_widths),
        lifetime_valid: false,
    }));
    for value in [
        crate::state_machine::RuntimeScriptedListenerBoundValue::Value(ScriptValue::Bool(true)),
        crate::state_machine::RuntimeScriptedListenerBoundValue::Trigger(1),
        crate::state_machine::RuntimeScriptedListenerBoundValue::Artboard(7),
    ] {
        assert!(
            !apply_scripted_input_update(
                &invalid,
                &ScriptCoreString::from("disposed"),
                value,
                Some(&resolver),
                None,
                &mut host,
            )
            .unwrap(),
            "a disposed C++ ScriptedObject has no state and rejects every ScriptInput update"
        );
    }
    assert!(invalid_scalar_values.borrow().is_empty());
    assert_eq!(invalid_trigger_calls.get(), 0);
    assert!(invalid_artboard_widths.borrow().is_empty());
}

#[test]
fn scripted_hydration_validation_failure_applies_no_inputs_or_init() {
    let trace = Rc::new(RefCell::new(Vec::new()));
    let artboard_applied = Rc::new(Cell::new(false));
    let (mut machine, action_global_id) = hydration_trace_machine(&trace, &artboard_applied);

    let error = machine
        .hydrate_and_initialize_scripted_listener_action_instance(
            action_global_id,
            crate::ScriptListenerActionHydration::new(None, Vec::new()),
            true,
            None,
            |_| {
                trace.borrow_mut().push("validate".to_owned());
                Err(ScriptError::new("intentional validation miss"))
            },
        )
        .expect_err("validation miss keeps the occurrence pending");

    assert_eq!(error.message(), "intentional validation miss");
    assert_eq!(
        trace.borrow().as_slice(),
        ["context", "validate"],
        "C++ installs the occurrence context before validation, but validation failure performs no input setter, resolver, or init work (`scripted_object.cpp:399-426`)"
    );
    assert!(!artboard_applied.get());
}

#[test]
fn scripted_hydration_resolves_artboard_then_viewmodel_in_authored_apply_order() {
    let trace = Rc::new(RefCell::new(Vec::new()));
    let artboard_applied = Rc::new(Cell::new(false));
    let (mut machine, action_global_id) = hydration_trace_machine(&trace, &artboard_applied);
    let artboard_resolver: Rc<dyn ScriptArtboardResolver> = Rc::new(HydrationArtboardResolver {
        trace: Rc::clone(&trace),
    });
    let view_model_resolver: Rc<dyn crate::ScriptViewModelInputResolver> =
        Rc::new(AfterArtboardViewModelResolver {
            trace: Rc::clone(&trace),
            artboard_applied: Rc::clone(&artboard_applied),
        });

    let error = machine
        .hydrate_and_initialize_scripted_listener_action_instance(
            action_global_id,
            crate::ScriptListenerActionHydration::new(None, Vec::new()),
            false,
            None,
            |_| {
                trace.borrow_mut().push("validate".to_owned());
                Ok(crate::ScriptListenerActionHydration::new(
                    None,
                    vec![
                        crate::ScriptListenerInputHydration::Artboard {
                            name: ScriptCoreString::from("panel"),
                            artboard_id: 7,
                            resolver: Rc::clone(&artboard_resolver),
                            parent_context: None,
                        },
                        crate::ScriptListenerInputHydration::ViewModel {
                            name: ScriptCoreString::from("child"),
                            input_global_id: 42,
                            path: crate::ScriptInputViewModelPropertyPath {
                                path_ids: vec![1, 2],
                                resolved_path_ids: vec![1, 2],
                                is_relative: false,
                            },
                            resolver: Rc::clone(&view_model_resolver),
                        },
                    ],
                ))
            },
        )
        .expect_err("the intentional late ViewModel miss ends phase two");

    assert_eq!(error.message(), "intentional late ViewModel miss");
    assert_eq!(
        trace.borrow().as_slice(),
        [
            "context",
            "validate",
            "resolve-artboard:7",
            "set-artboard:panel",
            "resolve-view-model",
        ],
        "phase two re-resolves each typed input at its authored position, so the later ViewModel lookup observes the earlier Artboard setter (`scripted_object.cpp:417-426`; `script_input_viewmodel_property.cpp:77-113`)"
    );
}

#[test]
fn scripted_hydration_accepts_valid_null_viewmodel_and_continues_to_init() {
    let trace = Rc::new(RefCell::new(Vec::new()));
    let artboard_applied = Rc::new(Cell::new(false));
    let (mut machine, action_global_id) = hydration_trace_machine(&trace, &artboard_applied);
    let resolver: Rc<dyn crate::ScriptViewModelInputResolver> = Rc::new(NullViewModelResolver {
        trace: Rc::clone(&trace),
    });

    let hydrated = machine
        .hydrate_and_initialize_scripted_listener_action_instance(
            action_global_id,
            crate::ScriptListenerActionHydration::new(None, Vec::new()),
            true,
            None,
            |_| {
                trace.borrow_mut().push("validate".to_owned());
                Ok(crate::ScriptListenerActionHydration::new(
                    None,
                    vec![
                        crate::ScriptListenerInputHydration::ViewModel {
                            name: ScriptCoreString::from("child"),
                            input_global_id: 42,
                            path: crate::ScriptInputViewModelPropertyPath {
                                path_ids: vec![1, 2],
                                resolved_path_ids: vec![1, 2],
                                is_relative: false,
                            },
                            resolver: Rc::clone(&resolver),
                        },
                        crate::ScriptListenerInputHydration::Value {
                            name: ScriptCoreString::from("after"),
                            value: ScriptValue::Number(2.0),
                        },
                    ],
                ))
            },
        )
        .expect("a valid nullable ViewModel property hydrates");

    assert!(hydrated);
    assert_eq!(
        trace.borrow().as_slice(),
        [
            "context",
            "validate",
            "resolve-null-view-model",
            "set:after",
            "init-check",
            "init",
        ],
        "C++ accepts the ViewModel-valued property cell, leaves its existing table field unchanged when referenceViewModelInstance is null, then hydrates later inputs and calls init (`script_input_viewmodel_property.cpp:60-113`; `scripted_object.cpp:399-426`)"
    );
}

#[test]
fn scripted_hydration_typed_artboard_failure_stops_later_inputs_and_init() {
    let trace = Rc::new(RefCell::new(Vec::new()));
    let artboard_applied = Rc::new(Cell::new(false));
    let (mut machine, action_global_id) = hydration_trace_machine(&trace, &artboard_applied);
    let artboard_resolver: Rc<dyn ScriptArtboardResolver> = Rc::new(HydrationArtboardResolver {
        trace: Rc::clone(&trace),
    });

    let error = machine
        .hydrate_and_initialize_scripted_listener_action_instance(
            action_global_id,
            crate::ScriptListenerActionHydration::new(None, Vec::new()),
            true,
            None,
            |_| {
                trace.borrow_mut().push("validate".to_owned());
                Ok(crate::ScriptListenerActionHydration::new(
                    None,
                    vec![
                        crate::ScriptListenerInputHydration::Value {
                            name: ScriptCoreString::from("before"),
                            value: ScriptValue::Number(1.0),
                        },
                        crate::ScriptListenerInputHydration::Artboard {
                            name: ScriptCoreString::from("panel"),
                            artboard_id: 9,
                            resolver: Rc::clone(&artboard_resolver),
                            parent_context: None,
                        },
                        crate::ScriptListenerInputHydration::Value {
                            name: ScriptCoreString::from("after"),
                            value: ScriptValue::Number(2.0),
                        },
                    ],
                ))
            },
        )
        .expect_err("typed Artboard construction failure remains terminal");

    assert_eq!(error.resource_code(), Some("script.resource.test"));
    assert_eq!(
        trace.borrow().as_slice(),
        ["context", "validate", "set:before", "resolve-artboard:9"],
        "a phase-two failure preserves earlier authored writes but suppresses every later input and user init (`scripted_object.cpp:417-437`)"
    );
    assert!(!artboard_applied.get());
}

fn scripted_listener_artboard_and_machine() -> (ArtboardInstance, StateMachineInstance) {
    let fixture = PathBuf::from(
        std::env::var_os("RIVE_RUNTIME_DIR")
            .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into()),
    )
    .join("tests/unit_tests/assets/scripted_listener_action.riv");
    let file = read_runtime_file(&std::fs::read(fixture).expect("read listener fixture"))
        .expect("import listener fixture");
    let graph = GraphFile::from_runtime_file(&file).expect("build listener graph");
    let mut artboard = ArtboardInstance::from_graph_with_artboards(
        &file,
        graph.artboards.first().expect("fixture artboard"),
        &graph.artboards,
    )
    .expect("instantiate listener artboard");
    let machine = artboard
        .state_machine_instance(0)
        .expect("fixture state machine");
    (artboard, machine)
}

fn scripted_listener_machine() -> StateMachineInstance {
    scripted_listener_artboard_and_machine().1
}

#[test]
fn audio_event_seam_plays_the_resolved_sound_fixture_asset() {
    let fixture = PathBuf::from(
        std::env::var_os("RIVE_RUNTIME_DIR")
            .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into()),
    )
    .join("tests/unit_tests/assets/sound.riv");
    let file = read_runtime_file(&std::fs::read(fixture).expect("read sound fixture"))
        .expect("import sound fixture");
    let graph = GraphFile::from_runtime_file(&file).expect("build sound graph");
    let mut artboard = ArtboardInstance::from_graph_with_artboards(
        &file,
        graph.artboards.first().expect("sound artboard"),
        &graph.artboards,
    )
    .expect("instantiate sound artboard");
    let owners = crate::RuntimeFileAssetOwners::from_runtime(&file, None);
    artboard.attach_runtime_file_asset_owners(&owners);
    let engine = crate::AudioEngine::new(2, 44_100).expect("headless audio engine");
    artboard.set_audio_engine(Some(engine.clone()));
    let event_local_id = artboard
        .components()
        .iter()
        .find(|component| component.type_name == "AudioEvent")
        .expect("sound AudioEvent")
        .local_id;
    let mut machine = artboard
        .state_machine_instance(0)
        .expect("sound state machine");
    let (event, _) = fl_c5_test_audio_event(event_local_id);

    assert_eq!(engine.playing_sound_count(), 0);
    machine.notify_events(&mut artboard, None, &[event]);
    assert_eq!(engine.playing_sound_count(), 1);
}

#[derive(Debug, Clone)]
struct RecordingHitComponent {
    label: &'static str,
    result: HitResult,
    trace: Rc<RefCell<Vec<String>>>,
    component: Option<ComponentHandle>,
}

impl HitComponent for RecordingHitComponent {
    fn clone_box(&self) -> Box<dyn HitComponent> {
        Box::new(self.clone())
    }

    fn component(&self) -> Option<ComponentHandle> {
        self.component
    }

    fn prepare_event(
        &mut self,
        _artboard: &ArtboardInstance,
        _groups: &mut [ListenerGroup],
        _position: (f32, f32),
        _hit_type: RuntimeListenerType,
        _pointer_id: i32,
    ) {
        self.trace
            .borrow_mut()
            .push(format!("prepare:{}", self.label));
    }

    fn process_event(
        &mut self,
        _instance: &mut StateMachineInstance,
        _artboard: &mut ArtboardInstance,
        _groups: &mut [ListenerGroup],
        _position: (f32, f32),
        hit_type: RuntimeListenerType,
        can_hit: bool,
        timestamp_seconds: f32,
        _pointer_id: i32,
        _owned_context: Option<&mut RuntimeOwnedViewModelInstance>,
        _event_context: Option<&StateMachineEventContext>,
        _host: &mut dyn ScriptHost,
    ) -> Result<HitResult, ScriptError> {
        self.trace.borrow_mut().push(format!(
            "process:{}:{can_hit}:{hit_type:?}:{timestamp_seconds:?}",
            self.label
        ));
        Ok(self.result)
    }

    fn hit_test(
        &self,
        _instance: &StateMachineInstance,
        _artboard: &ArtboardInstance,
        _position: (f32, f32),
    ) -> bool {
        self.result.is_hit()
    }

    fn enable_pointer_events(&mut self, _groups: &mut [ListenerGroup], pointer_id: i32) {
        self.trace
            .borrow_mut()
            .push(format!("enable:{}:{pointer_id}", self.label));
    }

    fn disable_pointer_events(&mut self, _groups: &mut [ListenerGroup], pointer_id: i32) {
        self.trace
            .borrow_mut()
            .push(format!("disable:{}:{pointer_id}", self.label));
    }
}

#[test]
fn fl_c5_hit_result_is_tristate_and_aggregates_strongest() {
    assert!(!HitResult::None.is_hit());
    assert!(HitResult::Hit.is_hit());
    assert_eq!(HitResult::None.strongest(HitResult::Hit), HitResult::Hit);
    assert_eq!(
        HitResult::Hit.strongest(HitResult::HitOpaque),
        HitResult::HitOpaque
    );
    assert_eq!(
        HitResult::HitOpaque.strongest(HitResult::None),
        HitResult::HitOpaque
    );
}

#[test]
fn fl_c5_hit_three_passes_continue_after_opaque_with_can_hit_false() {
    let (mut artboard, mut machine) = scripted_listener_artboard_and_machine();
    let trace = Rc::new(RefCell::new(Vec::new()));
    machine.listener_groups.clear();
    machine.hit_components = vec![
        Box::new(RecordingHitComponent {
            label: "front",
            result: HitResult::HitOpaque,
            trace: Rc::clone(&trace),
            component: None,
        }),
        Box::new(RecordingHitComponent {
            label: "back",
            result: HitResult::Hit,
            trace: Rc::clone(&trace),
            component: None,
        }),
    ];

    let result = machine
        .update_listeners(
            &mut artboard,
            RuntimeListenerType::Move,
            7.0,
            9.0,
            3,
            -2.5,
            None,
            None,
            &mut NoopScriptHost,
        )
        .expect("hit passes");

    assert_eq!(result, HitResult::HitOpaque);
    assert_eq!(
        trace.borrow().as_slice(),
        [
            "prepare:front",
            "prepare:back",
            "process:front:true:Move:-2.5",
            "process:back:false:Move:-2.5",
        ]
    );
}

#[test]
fn fl_c5_hit_sort_preserves_the_exact_adversarial_swap_order() {
    let (artboard, mut machine) = scripted_listener_artboard_and_machine();
    let artboard_component = artboard
        .components()
        .iter()
        .find(|component| component.type_name == "Artboard")
        .and_then(|component| artboard.component_handle(component.local_id))
        .expect("fixture root artboard component");
    let drawables = artboard
        .runtime_hit_component_order()
        .into_iter()
        .filter(|component| *component != artboard_component)
        .take(3)
        .collect::<Vec<_>>();
    assert_eq!(
        drawables.len(),
        3,
        "the adversarial fixture needs three distinct draw-order identities"
    );
    let trace = Rc::new(RefCell::new(Vec::new()));
    let hit = |label, component| {
        Box::new(RecordingHitComponent {
            label,
            result: HitResult::None,
            trace: Rc::clone(&trace),
            component: Some(component),
        }) as Box<dyn HitComponent>
    };
    machine.hit_components = vec![
        hit("third", drawables[2]),
        hit("root", artboard_component),
        hit("first-a", drawables[0]),
        hit("first-b", drawables[0]),
        hit("second", drawables[1]),
    ];

    machine.sort_hit_components(&artboard);

    assert_eq!(
        machine
            .hit_components
            .iter()
            .map(|hit| hit.component())
            .collect::<Vec<_>>(),
        [
            Some(artboard_component),
            Some(drawables[0]),
            Some(drawables[0]),
            Some(drawables[1]),
            Some(drawables[2]),
        ],
        "the in-place scan must continue after each swap so duplicate identities retain the pinned swap sequence"
    );
}

#[test]
fn fl_c5_pointer_drag_discards_event_timestamps_then_follows_with_move() {
    let (mut artboard, mut machine) = scripted_listener_artboard_and_machine();
    let trace = Rc::new(RefCell::new(Vec::new()));
    machine.listener_groups.clear();
    machine.hit_components = vec![Box::new(RecordingHitComponent {
        label: "drag",
        result: HitResult::None,
        trace: Rc::clone(&trace),
        component: None,
    })];

    assert!(!machine.drag_start(&mut artboard, 4.0, 5.0, 9.5, 11));
    assert!(!machine.drag_end(&mut artboard, 6.0, 7.0, -3.25, 11));

    assert_eq!(
        trace.borrow().as_slice(),
        [
            "disable:drag:11",
            "prepare:drag",
            "process:drag:true:DragStart:0.0",
            "enable:drag:11",
            "prepare:drag",
            "process:drag:true:DragEnd:0.0",
            "prepare:drag",
            "process:drag:true:Move:-3.25",
        ]
    );
}

#[test]
fn fl_c5_hit_click_only_duplicate_groups_require_down_and_up() {
    let (artboard, _) = scripted_listener_artboard_and_machine();
    let target = artboard
        .components()
        .iter()
        .find_map(|component| artboard.component_handle(component.local_id))
        .expect("component");
    let listener = RuntimeStateMachineListener {
        name: None,
        target_local_id: artboard.component_at(target).local_id,
        is_single: false,
        listener_types: vec![RuntimeListenerType::Click],
        event_local_indices: Vec::new(),
        view_model_path: None,
        view_model_input_types: Vec::new(),
        gamepad_input_types: Vec::new(),
        keyboard_input_types: Vec::new(),
        semantic_input_types: Vec::new(),
        hit_paths: Vec::new(),
        listener_actions: Vec::new(),
    };
    let groups = vec![ListenerGroup::authored(0), ListenerGroup::authored(0)];
    let listeners = vec![listener];
    let mut hit = HitDrawable::new(&artboard, Some(target), Some(target), false);

    assert!(hit.add_listener_impl(0, &groups, &listeners));
    assert!(hit.add_listener_impl(1, &groups, &listeners));
    assert_eq!(hit.listeners, [0, 1]);
    assert!(hit.needs_down_listener);
    assert!(hit.needs_up_listener);
    assert!(hit.can_early_out);
}

#[test]
fn fl_c5_pointer_exit_releases_group_history_and_drag_state() {
    let (mut artboard, mut machine) = scripted_listener_artboard_and_machine();
    for group in &mut machine.listener_groups {
        group.reset(-7);
        group.hover(-7);
        group.process(-7, (1.0, 2.0), true, true, false);
        group.record_position(-7, (1.0, 2.0));
    }
    for group in &mut machine.listener_groups {
        group.mark_dragged();
        group.disable(-7);
    }

    let _ = machine.pointer_exit(&mut artboard, 0.0, -0.0, -7);

    assert!(
        machine
            .listener_groups
            .iter()
            .all(|group| group.previous_position(-7).is_none())
    );
    assert!(
        machine
            .listener_groups
            .iter()
            .all(|group| !group.disabled(-7))
    );
}

#[test]
fn fl_c5_pointer_cpp_paths_accept_nonfinite_coordinates_and_timestamps() {
    let (mut artboard, mut machine) = scripted_listener_artboard_and_machine();
    for (x, y, timestamp) in [
        (f32::NAN, f32::INFINITY, f32::NEG_INFINITY),
        (f32::NEG_INFINITY, -0.0, f32::NAN),
        (0.0, f32::NAN, -42.0),
    ] {
        machine
            .update_listeners(
                &mut artboard,
                RuntimeListenerType::Move,
                x,
                y,
                77,
                timestamp,
                None,
                None,
                &mut NoopScriptHost,
            )
            .expect("C++-corresponding path forwards every f32 value");
    }
    let group_index = machine
        .listener_groups
        .iter()
        .position(|group| matches!(group.kind, ListenerGroupKind::Authored { .. }))
        .expect("fixture authored listener group");
    let mut group = machine.listener_groups.remove(group_index);
    group.reset(77);
    group.hover(77);
    machine
        .process_listener_group_event(
            &mut group,
            &mut artboard,
            (0.0, f32::NAN),
            RuntimeListenerType::Move,
            true,
            -42.0,
            77,
            None,
            None,
            &mut NoopScriptHost,
        )
        .expect("StateMachine-to-ListenerGroup integration retains non-finite values");
    let position = group.previous_position(77).expect("group pointer history");
    assert_eq!(position.0.to_bits(), 0.0_f32.to_bits());
    assert!(position.1.is_nan());
    machine.listener_groups.insert(group_index, group);
}

#[test]
fn fl_c5_constructor_order_phase_trace_and_explicit_fields() {
    let (artboard, machine) = scripted_listener_artboard_and_machine();
    assert_eq!(
        machine.constructor_phases,
        [
            RuntimeConstructorPhase::Inputs,
            RuntimeConstructorPhase::LayersAnyEntry,
            RuntimeConstructorPhase::MachineBinds,
            RuntimeConstructorPhase::AuthoredListenerCategories,
            RuntimeConstructorPhase::ComponentProvidedGroups,
            RuntimeConstructorPhase::NestedListTextHits,
            RuntimeConstructorPhase::ScriptedClonesAndFacilities,
            RuntimeConstructorPhase::HitSort,
            RuntimeConstructorPhase::FocusTree,
        ],
        "constructor boundaries follow state_machine_instance.cpp:1711-2127"
    );
    assert_eq!(machine.layer_count, machine.layers.len());
    assert_eq!(
        machine.layer_count,
        artboard.state_machine(0).unwrap().layers.len()
    );
    assert_eq!(machine.draw_order_change_counter, 0);
    assert!(!machine.disposed);
    assert_eq!(machine.has_listeners(), machine.hit_components_count() != 0);
    assert_eq!(
        machine
            .hit_component(machine.hit_components_count())
            .map(HitComponent::component),
        None
    );
}

#[test]
fn fl_c5_constructor_order_retains_unresolved_pointer_group_occurrence() {
    let (mut artboard, _) = scripted_listener_artboard_and_machine();
    let mut definition = reset_input_state_machine(reset_input_actions());
    definition.listeners = Arc::new(vec![RuntimeStateMachineListener {
        name: None,
        target_local_id: usize::MAX,
        is_single: false,
        listener_types: vec![RuntimeListenerType::Down],
        event_local_indices: Vec::new(),
        view_model_path: None,
        view_model_input_types: Vec::new(),
        gamepad_input_types: Vec::new(),
        keyboard_input_types: Vec::new(),
        semantic_input_types: Vec::new(),
        hit_paths: Vec::new(),
        listener_actions: Vec::new(),
    }]);
    artboard.state_machines = Arc::new(vec![definition]);
    let machine = artboard
        .state_machine_instance(0)
        .expect("state machine with unresolved pointer target");

    assert_eq!(
        machine
            .input(0)
            .and_then(StateMachineInputInstance::bool_value),
        Some(true),
        "entry actions execute during the layer phase"
    );
    assert_eq!(machine.listener_groups.len(), 1);
    assert!(
        machine
            .hit_components
            .iter()
            .all(|owner| owner.component().is_some()),
        "an unresolved target retains its group but creates no hit owner"
    );
}

#[test]
fn fl_c5_hit_component_identity_reuses_owner_but_retains_duplicate_groups() {
    let (mut artboard, _) = scripted_listener_artboard_and_machine();
    let target_local_id = artboard
        .components()
        .iter()
        .find(|component| {
            component.type_name == "Shape"
                || component.type_name == "TextValueRun"
                || nuxie_schema::definition_by_name(component.type_name)
                    .is_some_and(|definition| definition.is_a("LayoutComponent"))
        })
        .expect("fixture pointer target")
        .local_id;
    let listener = RuntimeStateMachineListener {
        name: None,
        target_local_id,
        is_single: false,
        listener_types: vec![RuntimeListenerType::Down],
        event_local_indices: Vec::new(),
        view_model_path: None,
        view_model_input_types: Vec::new(),
        gamepad_input_types: Vec::new(),
        keyboard_input_types: Vec::new(),
        semantic_input_types: Vec::new(),
        hit_paths: Vec::new(),
        listener_actions: Vec::new(),
    };
    let mut definition = reset_input_state_machine(reset_input_actions());
    definition.listeners = Arc::new(vec![listener.clone(), listener]);
    definition.transition_duration_bindings = Arc::new(vec![
        RuntimeTransitionDurationBinding {
            data_bind_index: 3,
            transition_global_id: 44,
        },
        RuntimeTransitionDurationBinding {
            data_bind_index: 7,
            transition_global_id: 44,
        },
    ]);
    artboard.state_machines = Arc::new(vec![definition]);

    let machine = artboard
        .state_machine_instance(0)
        .expect("state machine with duplicate pointer and bind occurrences");

    assert_eq!(machine.listener_groups.len(), 2);
    let target = artboard
        .component_handle(target_local_id)
        .expect("pointer target handle");
    assert_eq!(
        machine
            .hit_components
            .iter()
            .filter(|owner| owner.component() == Some(target))
            .count(),
        1,
        "duplicate groups share one component-identity hit owner"
    );
    assert_eq!(
        machine
            .transition_durations
            .iter()
            .map(|occurrence| occurrence.transition_global_id)
            .collect::<Vec<_>>(),
        [44, 44],
        "duplicate transition-property binds retain distinct authored occurrences"
    );
}

fn reset_input_state_machine(
    listener_actions: Vec<RuntimeScheduledListenerAction>,
) -> RuntimeStateMachine {
    RuntimeStateMachine {
        global_id: 900,
        name: Some(Arc::from("reset inputs")),
        default_view_model_index: None,
        inputs: Arc::new(vec![
            Some(RuntimeStateMachineInput::new_bool(
                901,
                Some("enabled".to_owned()),
                false,
            )),
            Some(RuntimeStateMachineInput::new_number(
                902,
                Some("amount".to_owned()),
                0.0,
            )),
            Some(RuntimeStateMachineInput::new_trigger(
                903,
                Some("fire".to_owned()),
            )),
        ]),
        listeners: Arc::new(Vec::new()),
        layers: Arc::new(vec![RuntimeStateMachineLayer {
            global_id: 904,
            name: None,
            states: vec![RuntimeLayerState {
                global_id: Some(905),
                type_name: Some("EntryState"),
                animation: None,
                blend_state_1d: None,
                blend_state_direct: None,
                speed: 1.0,
                flags: 0,
                fire_actions: Vec::new(),
                listener_actions,
                transitions: Vec::new(),
            }],
            entry_state_index: Some(0),
            any_state_index: None,
            exit_state_index: None,
        }]),
        bindable_numbers: Arc::new(Vec::new()),
        bindable_integers: Arc::new(Vec::new()),
        bindable_colors: Arc::new(Vec::new()),
        bindable_strings: Arc::new(Vec::new()),
        bindable_enums: Arc::new(Vec::new()),
        bindable_assets: Arc::new(Vec::new()),
        bindable_artboards: Arc::new(Vec::new()),
        bindable_lists: Arc::new(Vec::new()),
        bindable_triggers: Arc::new(Vec::new()),
        bindable_view_models: Arc::new(Vec::new()),
        bindable_booleans: Arc::new(Vec::new()),
        view_model_triggers: Arc::new(Vec::new()),
        transition_duration_bindings: Arc::new(Vec::new()),
        data_bind_templates: Arc::new(Vec::new()),
        scripted_objects: Vec::new(),
        scripted_listener_actions: Vec::new(),
        scripted_object_bindings: Vec::new(),
        action_owners: RuntimeActionCoreArena::empty(),
    }
}

#[test]
fn typed_named_inputs_match_type_and_name_in_authored_order() {
    let (_, mut artboard) = fl_c5_bind_file_and_artboard();
    let mut definition = reset_input_state_machine(Vec::new());
    definition.inputs = Arc::new(vec![
        Some(RuntimeStateMachineInput::new_number(
            901,
            Some("x".to_owned()),
            7.0,
        )),
        Some(RuntimeStateMachineInput::new_bool(
            902,
            Some("x".to_owned()),
            true,
        )),
        Some(RuntimeStateMachineInput::new_trigger(
            903,
            Some("x".to_owned()),
        )),
    ]);
    artboard.state_machines = Arc::new(vec![definition]);
    let machine = artboard
        .state_machine_instance(0)
        .expect("typed named-input machine");

    assert_eq!(
        machine
            .input_named("x")
            .and_then(|input| input.number_value()),
        Some(7.0),
        "the untyped Rust convenience keeps first-name semantics"
    );
    assert_eq!(
        machine.get_bool("x").and_then(|input| input.bool_value()),
        Some(true),
        "getBool skips the earlier same-name Number occurrence"
    );
    assert_eq!(
        machine
            .get_number("x")
            .and_then(|input| input.number_value()),
        Some(7.0)
    );
    assert_eq!(
        machine
            .get_trigger("x")
            .and_then(|input| input.trigger_fired()),
        Some(false)
    );
    assert!(machine.get_bool("missing").is_none());
}

fn reset_input_actions() -> Vec<RuntimeScheduledListenerAction> {
    let direct = |index| RuntimeListenerInputTarget {
        direct_input_index: Some(index),
        nested_input_local_id: None,
    };
    vec![
        RuntimeScheduledListenerAction::BoolChange(RuntimeListenerBoolChange::for_test(
            StateMachineFireOccurrence::AtStart.value(),
            direct(0),
            1,
        )),
        RuntimeScheduledListenerAction::NumberChange(RuntimeListenerNumberChange::for_test(
            StateMachineFireOccurrence::AtStart.value(),
            direct(1),
            4.0,
        )),
        RuntimeScheduledListenerAction::TriggerChange(RuntimeListenerTriggerChange::for_test(
            StateMachineFireOccurrence::AtStart.value(),
            direct(2),
        )),
    ]
}

fn fl_c5_state_transition(
    global_id: u32,
    state_to_index: usize,
    conditions: Vec<RuntimeTransitionCondition>,
) -> RuntimeStateTransition {
    RuntimeStateTransition {
        global_id,
        state_to_index: Some(state_to_index),
        exit_blend_animation_index: None,
        duration: 0,
        exit_time: 0,
        flags: 0,
        random_weight: 0,
        direct_input_conditions_only: conditions
            .iter()
            .all(RuntimeTransitionCondition::is_direct_input),
        conditions,
        fire_actions: Vec::new(),
        listener_actions: Vec::new(),
        interpolator: None,
        has_unsupported_interpolator: false,
    }
}

fn fl_c5_state(
    global_id: u32,
    type_name: &'static str,
    animation: bool,
    transitions: Vec<RuntimeStateTransition>,
) -> RuntimeLayerState {
    RuntimeLayerState {
        global_id: Some(global_id),
        type_name: Some(type_name),
        animation: animation.then(RuntimeLinearAnimationHandle::empty),
        blend_state_1d: None,
        blend_state_direct: None,
        speed: 1.0,
        flags: 0,
        fire_actions: Vec::new(),
        listener_actions: Vec::new(),
        transitions,
    }
}

fn fl_c5_state_query_machine() -> RuntimeStateMachine {
    let enabled = || {
        RuntimeTransitionCondition::Bool(RuntimeTransitionBoolCondition::new(
            0,
            TransitionConditionOp::Equal,
        ))
    };
    let changing_layer = |layer_global_id, state_global_id| RuntimeStateMachineLayer {
        global_id: layer_global_id,
        name: None,
        states: vec![
            fl_c5_state(
                state_global_id,
                "EntryState",
                false,
                vec![fl_c5_state_transition(state_global_id + 1, 1, Vec::new())],
            ),
            fl_c5_state(
                state_global_id + 2,
                "AnimationState",
                true,
                vec![fl_c5_state_transition(
                    state_global_id + 3,
                    2,
                    vec![enabled()],
                )],
            ),
            fl_c5_state(
                state_global_id + 4,
                "AnimationState",
                true,
                vec![fl_c5_state_transition(state_global_id + 5, 3, Vec::new())],
            ),
            fl_c5_state(state_global_id + 6, "AnimationState", true, Vec::new()),
        ],
        entry_state_index: Some(0),
        any_state_index: None,
        exit_state_index: None,
    };
    let inert_layer = RuntimeStateMachineLayer {
        global_id: 920,
        name: None,
        states: vec![
            fl_c5_state(
                921,
                "EntryState",
                false,
                vec![fl_c5_state_transition(922, 1, Vec::new())],
            ),
            fl_c5_state(923, "ExitState", false, Vec::new()),
        ],
        entry_state_index: Some(0),
        any_state_index: None,
        exit_state_index: Some(1),
    };
    let mut machine = reset_input_state_machine(Vec::new());
    machine.layers = Arc::new(vec![
        changing_layer(910, 1_000),
        inert_layer,
        changing_layer(930, 2_000),
    ]);
    machine
}

fn fl_c5_advance_fixture() -> (ArtboardInstance, StateMachineInstance) {
    let (mut artboard, _) = scripted_listener_artboard_and_machine();
    artboard.state_machines = Arc::new(vec![reset_input_state_machine(Vec::new())]);
    let machine = artboard
        .state_machine_instance(0)
        .expect("WP7 advance fixture state machine");
    (artboard, machine)
}

#[test]
fn fl_c5_advance_raw_order_and_clean_zero_bookkeeping() {
    let (mut artboard, mut machine) = fl_c5_advance_fixture();

    let _ = artboard.advance_state_machine_instance(&mut machine, 0.25);
    assert!(machine.fire_trigger(2));
    let _ = artboard.advance_state_machine_instance(&mut machine, -0.0);

    assert_eq!(
        machine.advance_phase_trace,
        [
            "draw-sort-check",
            "focus-snapshot",
            "semantic-snapshot",
            "apply-events",
            "clear-latch",
            "pre-layer-binds",
            "authored-layers",
            "converter-advance",
            "inputs-advanced",
        ],
        "raw order matches state_machine_instance.cpp:2546-2585"
    );
    assert_eq!(
        machine.input(2).and_then(|input| input.trigger_fired()),
        Some(false),
        "clean signed-zero advances still run every input advanced()"
    );
}

#[test]
fn fl_c5_advance_new_frame_false_preserves_the_sticky_latch() {
    let (mut artboard, mut machine) = fl_c5_advance_fixture();
    let _ = artboard.advance_state_machine_instance(&mut machine, 0.25);
    assert!(machine.set_bool(0, true));

    let definitions = artboard.state_machine_definition_owner(&machine);
    let definition = definitions.first().expect("advance definition");
    assert!(machine.advance(&mut artboard, definition, 0.0, false, None));
    assert!(machine.needs_advance());
    assert!(
        !machine.advance_phase_trace.contains(&"clear-latch"),
        "newFrame=false must never clear m_needsAdvance"
    );
}

#[test]
fn fl_c5_advance_fp_values_forward_without_validation_and_zero_forces_facade() {
    let (mut artboard, mut machine) = fl_c5_advance_fixture();
    let definitions = artboard.state_machine_definition_owner(&machine);
    let definition = definitions.first().expect("advance definition");

    for seconds in [
        f32::NAN,
        f32::INFINITY,
        f32::NEG_INFINITY,
        -17.25,
        0.0,
        -0.0,
    ] {
        let _ = machine.advance(&mut artboard, definition, seconds, true, None);
        assert_eq!(
            machine.advance_phase_trace.last(),
            Some(&"inputs-advanced"),
            "every f32 value reaches the end of raw bookkeeping"
        );
    }

    assert!(StateMachineInstance::advance_and_apply_return(
        false,
        0.0,
        std::slice::from_ref(&machine),
    ));
    assert!(StateMachineInstance::advance_and_apply_return(
        false,
        -0.0,
        std::slice::from_ref(&machine),
    ));
    assert!(!StateMachineInstance::advance_and_apply_return(
        false,
        f32::NAN,
        &[],
    ));
    assert!(!StateMachineInstance::advance_and_apply_return(
        false,
        -1.0,
        &[],
    ));

    machine.reported_events.push(fl_c5_test_reported_event(7));
    assert!(
        StateMachineInstance::advance_and_apply_return(false, 0.25, std::slice::from_ref(&machine),),
        "a pending event keeps the facade going"
    );
    machine.reported_events.clear();
    machine.reported_listener_view_models.report_data_bind(0);
    assert!(
        StateMachineInstance::advance_and_apply_return(false, 0.25, std::slice::from_ref(&machine),),
        "a pending listener ViewModel keeps the facade going"
    );
}

#[test]
fn fl_c5_advance_bind_generated_report_is_a_raw_return_term() {
    let (mut artboard, mut machine) = fl_c5_advance_fixture();
    machine.bind_advance_test_report = Some(fl_c5_test_reported_event(17));

    assert!(
        artboard.advance_state_machine_instance(&mut machine, 0.25),
        "a report created during converter/bind advance is a raw return term"
    );
    assert_eq!(
        machine.reported_event_count(),
        1,
        "the bind-generated report remains pending for the next applyEvents snapshot"
    );
}

#[test]
fn fl_c5_advance_and_apply_persistent_dirt_component_stops_after_five_passes() {
    let (mut artboard, mut machine) = fl_c5_advance_fixture();
    artboard.install_persistent_dirt_component_fixture();
    let probe_count = machine.transition_probe_count;

    let advanced = machine
        .advance_and_apply(&mut artboard, 0.25)
        .expect("public advance_and_apply facade");
    let (advance_count, update_count, dirt_remaining) =
        artboard.persistent_dirt_component_fixture_receipt();

    assert_eq!(
        machine.transition_probe_count - probe_count,
        5,
        "persistent sixth-pass dirt is capped after five unconditional probes"
    );
    assert_eq!(
        machine.data_context_advance_call_count, 5,
        "ViewModels advance once per settlement iteration before the Artboard reset"
    );
    assert_eq!(
        (advanced, advance_count, update_count, dirt_remaining),
        (true, 6, 5, true),
        "one main component advance plus five settlement advances leaves sixth-pass dirt pending"
    );
    println!(
        "FL_C5_PERSISTENT_DIRT_RECEIPT advanced={advanced} \
         advance_count={advance_count} update_count={update_count} \
         dirt_remaining={dirt_remaining}"
    );
}

#[test]
fn fl_c5_settlement_pending_bind_work_does_not_extend_a_clean_component_pass() {
    let (mut artboard, mut machine) = fl_c5_advance_fixture();
    let _ = artboard.update_pass();
    assert!(
        !artboard.has_dirt(ComponentDirt::COMPONENTS),
        "fixture prelude must start the settlement probe with clean component dirt"
    );
    let mut persisting_bind = crate::retained_data_bind::RuntimeRetainedDataBind::new(0, false);
    machine
        .data_bind_container
        .add_data_bind(&mut persisting_bind, true);
    let update_count = std::cell::Cell::new(0);

    StateMachineInstance::settle_artboard_update_passes(
        &mut artboard,
        std::slice::from_mut(&mut machine),
        true,
        |_| {
            update_count.set(update_count.get() + 1);
            false
        },
    );

    assert_eq!(
        update_count.get(),
        1,
        "pinned C++ breaks on clean component dirt even when bind bookkeeping remains pending"
    );
    assert_eq!(
        machine.data_context_advance_call_count, 1,
        "the completed pass still advances the DataContext before checking component dirt"
    );
}

#[test]
fn fl_c5_advance_view_models_false_skips_only_data_context_advancement() {
    let (mut artboard, mut machine) = fl_c5_advance_fixture();
    StateMachineInstance::settle_artboard_update_passes(
        &mut artboard,
        std::slice::from_mut(&mut machine),
        false,
        |artboard| artboard.update_pass(),
    );
    assert_eq!(machine.data_context_advance_call_count, 0);

    StateMachineInstance::settle_artboard_update_passes(
        &mut artboard,
        std::slice::from_mut(&mut machine),
        true,
        |artboard| artboard.update_pass(),
    );
    assert_eq!(machine.data_context_advance_call_count, 1);

    let detached_advance_calls = std::cell::Cell::new(0);
    let (mut artboard, mut machine) = fl_c5_advance_fixture();
    StateMachineInstance::advance_and_apply_state_machines_with_view_models(
        &mut artboard,
        std::slice::from_mut(&mut machine),
        0.25,
        false,
        || {
            detached_advance_calls.set(detached_advance_calls.get() + 1);
            true
        },
    )
    .expect("advanceViewModels=false facade");
    assert_eq!(
        detached_advance_calls.get(),
        0,
        "advanceViewModels=false skips detached scripted ViewModels"
    );

    let (mut artboard, mut machine) = fl_c5_advance_fixture();
    StateMachineInstance::advance_and_apply_state_machines_with_view_models(
        &mut artboard,
        std::slice::from_mut(&mut machine),
        0.25,
        true,
        || {
            detached_advance_calls.set(detached_advance_calls.get() + 1);
            true
        },
    )
    .expect("advanceViewModels=true facade");
    assert_eq!(
        detached_advance_calls.get(),
        1,
        "advanceViewModels=true advances detached scripted ViewModels exactly once"
    );
}

#[test]
fn fl_c5_advance_focus_chaining_and_hidden_target_boundaries() {
    let (mut artboard, mut machine, _) =
        scripted_drawable_subtype_input_artboard_and_machine_with_optional_script(
            "ScriptedDrawable",
            None,
            true,
        );
    machine.focus.take_owner_events();
    machine.queued_focus_events.clear();
    machine.listener_definitions = Arc::new(vec![RuntimeStateMachineListener {
        name: None,
        target_local_id: 1,
        is_single: false,
        listener_types: vec![RuntimeListenerType::Focus, RuntimeListenerType::Blur],
        event_local_indices: Vec::new(),
        view_model_path: None,
        view_model_input_types: Vec::new(),
        gamepad_input_types: Vec::new(),
        keyboard_input_types: Vec::new(),
        semantic_input_types: Vec::new(),
        hit_paths: Vec::new(),
        listener_actions: vec![RuntimeScheduledListenerAction::FocusClear(
            RuntimeFocusActionClear::for_test(0),
        )],
    }]);
    machine.focus_listener_groups = vec![
        RuntimeFocusListenerGroup::new(0, 2, &machine.listener_definitions[0])
            .expect("advance focus group"),
    ];
    assert!(machine.focus.clear_focus());
    machine.capture_focus_callbacks();
    machine.queued_focus_events.clear();
    assert!(machine.focus.set_focus_target(1));
    machine.capture_focus_callbacks();

    assert!(
        !artboard.advance_state_machine_instance(&mut machine, 0.25),
        "C++ clears the continuation set by focus generated during the active focus snapshot"
    );
    assert_eq!(
        machine.queued_focus_events,
        [RuntimeQueuedFocusEvent {
            listener_index: 0,
            is_focus: false,
        }],
        "the chained blur callback remains queued despite the pinned lost-latch edge"
    );

    let opacity_key = crate::properties::property_key_for_name("Node", "opacity")
        .expect("Node.opacity property key");
    assert!(artboard.set_double_property(1, opacity_key, 0.0));
    artboard.update_components();
    let _ = machine
        .advance_and_apply(&mut artboard, 0.25)
        .expect("hidden-focus facade advance");
    assert!(
        machine.focus.focused_listener_chain().is_empty(),
        "the facade drops a focus target made ineligible by retained Artboard state"
    );
}

#[test]
fn fl_c5_state_changed_queries_retain_same_frame_flags_in_authored_layer_order() {
    let (mut artboard, _) = scripted_listener_artboard_and_machine();
    artboard.state_machines = Arc::new(vec![fl_c5_state_query_machine()]);
    let mut machine = artboard
        .state_machine_instance(0)
        .expect("state query machine");

    assert!(artboard.advance_state_machine_instance(&mut machine, 0.0));
    assert_eq!(machine.changed_state_count(), 3);
    assert_eq!(
        machine.changed_state(1).and_then(|state| state.global_id),
        Some(923),
        "initial Entry convergence includes the authored non-animation layer"
    );

    assert!(machine.set_bool(0, true));
    assert!(
        artboard.settle_state_machine_update_passes_with_state_machines(std::slice::from_mut(
            &mut machine
        ),)
    );
    assert_eq!(
        machine.changed_state_count(),
        2,
        "several transitions in one layer still count one changed layer"
    );
    assert_eq!(
        machine.changed_state(0).and_then(|state| state.global_id),
        Some(1_006)
    );
    assert_eq!(
        machine.changed_state(1).and_then(|state| state.global_id),
        Some(2_006),
        "the compressed index skips unchanged authored layer 1"
    );
    assert!(machine.changed_state(2).is_none());
    assert_eq!(
        machine.layer_state(0).and_then(|state| state.global_id),
        Some(1_006)
    );
    assert_eq!(
        machine.layer_state(1).and_then(|state| state.global_id),
        Some(923)
    );
    assert_eq!(
        machine.layer_state(2).and_then(|state| state.global_id),
        Some(2_006)
    );
    assert!(machine.layer_state(3).is_none());

    assert!(
        !artboard.settle_state_machine_update_passes_with_state_machines(std::slice::from_mut(
            &mut machine
        ),)
    );
    assert_eq!(
        machine.changed_state_count(),
        0,
        "the next standalone new-frame settlement clears retained flags"
    );
}

#[test]
fn fl_c5_state_changed_current_animation_queries_compress_the_same_authored_layers() {
    let (mut artboard, _) = scripted_listener_artboard_and_machine();
    artboard.state_machines = Arc::new(vec![fl_c5_state_query_machine()]);
    let mut machine = artboard
        .state_machine_instance(0)
        .expect("state query machine");

    assert!(artboard.advance_state_machine_instance(&mut machine, 0.0));
    assert_eq!(machine.current_animation_count(), 2);
    assert!(machine.current_animation(0).is_some());
    assert!(machine.current_animation(1).is_some());
    assert!(machine.current_animation(2).is_none());
    assert_eq!(
        machine.layer_state(1).and_then(|state| state.global_id),
        Some(923),
        "the interleaved non-animation layer remains visible by raw layer index"
    );
}

#[test]
fn fl_c5_state_changed_layer_state_handles_null_current_and_owner_length_disagreement() {
    let (mut artboard, _) = scripted_listener_artboard_and_machine();
    artboard.state_machines = Arc::new(vec![fl_c5_state_query_machine()]);
    let mut machine = artboard
        .state_machine_instance(0)
        .expect("state query machine");

    machine.layers.truncate(2);
    assert!(
        machine.layer_state(2).is_none(),
        "a retained definition without an occurrence is safely absent"
    );

    let extra_layer = machine.layers[0].clone();
    machine.layers.push(extra_layer.clone());
    machine.layers.push(extra_layer);
    assert!(
        machine.layer_state(3).is_none(),
        "the retained machine definition bounds the testing query"
    );

    let null_layer = RuntimeStateMachineLayer {
        global_id: 940,
        name: None,
        states: Vec::new(),
        entry_state_index: None,
        any_state_index: None,
        exit_state_index: None,
    };
    let mut null_machine = reset_input_state_machine(Vec::new());
    null_machine.layers = Arc::new(vec![null_layer]);
    artboard.state_machines = Arc::new(vec![null_machine]);
    let null_instance = artboard
        .state_machine_instance(0)
        .expect("null-current state query machine");
    assert!(
        null_instance.layer_state(0).is_none(),
        "a layer occurrence with no current state projects null"
    );
    assert!(null_instance.layer_state(1).is_none());
}

#[test]
fn fl_c5_state_changed_reset_during_active_transition_keeps_current_state_query_live() {
    let (mut artboard, _) = scripted_listener_artboard_and_machine();
    let mut definition = fl_c5_state_query_machine();
    Arc::make_mut(&mut definition.layers)[0].states[1].transitions[0].duration = 1_000;
    artboard.state_machines = Arc::new(vec![definition]);
    let mut machine = artboard
        .state_machine_instance(0)
        .expect("state query machine");

    assert!(artboard.advance_state_machine_instance(&mut machine, 0.0));
    assert!(machine.set_bool(0, true));
    assert!(artboard.advance_state_machine_instance(&mut machine, 0.0));
    assert_eq!(machine.changed_state_count(), 2);

    machine.reset_state(&mut artboard);
    assert_eq!(
        machine.layer_state(0).and_then(|state| state.type_name),
        Some("EntryState")
    );
    assert_eq!(
        machine.changed_state(0).and_then(|state| state.type_name),
        Some("EntryState"),
        "reset replaces the current occurrence without clearing this frame's flag"
    );
    assert_eq!(
        machine.changed_state_count(),
        2,
        "reset during an active transition does not invent or erase changed layers"
    );
}

#[test]
fn fl_c5_state_changed_random_weight_scratch_is_isolated_between_instances() {
    let (mut artboard, _) = scripted_listener_artboard_and_machine();
    let mut definition = reset_input_state_machine(Vec::new());
    let mut first = fl_c5_state_transition(3_003, 2, Vec::new());
    first.random_weight = 1;
    let mut second = fl_c5_state_transition(3_004, 3, Vec::new());
    second.random_weight = 3;
    definition.layers = Arc::new(vec![RuntimeStateMachineLayer {
        global_id: 3_000,
        name: None,
        states: vec![
            fl_c5_state(
                3_001,
                "EntryState",
                false,
                vec![fl_c5_state_transition(3_002, 1, Vec::new())],
            ),
            RuntimeLayerState {
                flags: 1,
                transitions: vec![first, second],
                ..fl_c5_state(3_005, "AnimationState", true, Vec::new())
            },
            fl_c5_state(3_006, "AnimationState", true, Vec::new()),
            fl_c5_state(3_007, "AnimationState", true, Vec::new()),
        ],
        entry_state_index: Some(0),
        any_state_index: None,
        exit_state_index: None,
    }]);
    artboard.state_machines = Arc::new(vec![definition]);
    let mut first = artboard
        .state_machine_instance(0)
        .expect("first random state-machine instance");
    let mut second = artboard
        .state_machine_instance(0)
        .expect("second random state-machine instance");
    let _random_values = crate::set_runtime_random_test_values(&[0.0, 0.75]);

    assert!(artboard.advance_state_machine_instance(&mut first, 0.0));
    assert!(artboard.advance_state_machine_instance(&mut second, 0.0));
    assert_eq!(
        first.layer_state(0).and_then(|state| state.global_id),
        Some(3_006)
    );
    assert_eq!(
        second.layer_state(0).and_then(|state| state.global_id),
        Some(3_007)
    );
    let first_scratch = first.layers[0].evaluated_random_weights();
    let second_scratch = second.layers[0].evaluated_random_weights();
    assert_eq!(first_scratch, [1, 3]);
    assert_eq!(second_scratch, [1, 3]);
    assert_ne!(
        first_scratch.as_ptr(),
        second_scratch.as_ptr(),
        "shared definitions never own mutable evaluated-weight scratch"
    );
}

#[test]
fn reset_state_marks_advance_only_for_genuine_entry_action_changes() {
    let (mut artboard, _) = scripted_listener_artboard_and_machine();

    artboard.state_machines = Arc::new(vec![reset_input_state_machine(Vec::new())]);
    let mut inert = artboard
        .state_machine_instance(0)
        .expect("inert reset state machine");
    inert.needs_advance = false;
    inert.reset_state(&mut artboard);
    assert!(
        !inert.needs_advance(),
        "StateMachineInstance::resetState itself does not call markNeedsAdvance"
    );

    artboard.state_machines = Arc::new(vec![reset_input_state_machine(reset_input_actions())]);
    let mut machine = artboard
        .state_machine_instance(0)
        .expect("input reset state machine");
    assert_eq!(
        machine.input(0).and_then(|input| input.bool_value()),
        Some(true)
    );
    assert_eq!(
        machine.input(1).and_then(|input| input.number_value()),
        Some(4.0)
    );
    assert_eq!(
        machine.input(2).and_then(|input| input.trigger_fired()),
        Some(true)
    );

    machine.needs_advance = false;
    machine.reset_state(&mut artboard);
    assert!(
        !machine.needs_advance(),
        "equal bool/number writes and an already-fired trigger do not call SMIInput::valueChanged"
    );

    assert!(machine.set_bool(0, false));
    assert!(machine.set_number(1, 0.0));
    machine.inputs[2].advanced();
    machine.needs_advance = false;
    machine.reset_state(&mut artboard);
    assert!(
        machine.needs_advance(),
        "genuine bool/number/trigger entry-action changes call the owning StateMachineInstance::markNeedsAdvance"
    );
    assert_eq!(
        machine.input(0).and_then(|input| input.bool_value()),
        Some(true)
    );
    assert_eq!(
        machine.input(1).and_then(|input| input.number_value()),
        Some(4.0)
    );
    assert_eq!(
        machine.input(2).and_then(|input| input.trigger_fired()),
        Some(true)
    );
}

#[test]
fn event_or_viewmodel_listener_excludes_other_constructor_groups() {
    // Pinned C++ continues the constructor loop immediately after either
    // report-only owner (`state_machine_instance.cpp:1829-1842`).
    assert!(listener_types_use_report_queue(&[
        RuntimeListenerType::Keyboard,
        RuntimeListenerType::Event,
        RuntimeListenerType::Focus,
    ]));
    assert!(listener_types_use_report_queue(&[
        RuntimeListenerType::Gamepad,
        RuntimeListenerType::ViewModel,
        RuntimeListenerType::SemanticAction,
    ]));
    assert!(!listener_types_use_report_queue(&[
        RuntimeListenerType::Keyboard,
        RuntimeListenerType::Focus,
        RuntimeListenerType::SemanticAction,
    ]));
}

#[test]
fn malformed_local_event_listener_targeting_a_node_stays_blocked() {
    let (mut artboard, mut machine, _) =
        scripted_drawable_input_artboard_and_machine(Box::new(RecordingDrawableInputScript {
            label: "event target",
            methods: Vec::new(),
            handled: false,
            calls: Rc::new(RefCell::new(Vec::new())),
        }));
    machine.listener_definitions = Arc::new(vec![RuntimeStateMachineListener {
        name: None,
        target_local_id: 1,
        is_single: false,
        listener_types: vec![RuntimeListenerType::Event],
        event_local_indices: vec![7],
        view_model_path: None,
        view_model_input_types: Vec::new(),
        gamepad_input_types: Vec::new(),
        keyboard_input_types: Vec::new(),
        semantic_input_types: Vec::new(),
        hit_paths: Vec::new(),
        listener_actions: vec![RuntimeScheduledListenerAction::FocusClear(
            RuntimeFocusActionClear::for_test(0),
        )],
    }]);
    let event = StateMachineReportedEvent {
        event_local_index: 7,
        event_core_type: 128,
        name: Some("local".to_owned()),
        url: None,
        target: None,
        properties: Vec::new(),
        string_properties: Vec::new(),
        seconds_delay: 0.0,
        context: None,
    };

    assert!(!machine.notify_events(&mut artboard, None, &[event]));
    assert!(
        machine.focus.target_has_focus(1),
        "pinned C++ does not deliver a local report to a listener whose target resolves to an ordinary Node"
    );
}

fn scripted_drawable_input_artboard_and_machine(
    script: Box<dyn ScriptInstance>,
) -> (ArtboardInstance, StateMachineInstance, u32) {
    scripted_drawable_subtype_input_artboard_and_machine("ScriptedDrawable", script)
}

#[test]
fn scripted_drawable_pointer_hit_flows_through_the_state_machine_hit_aggregate() {
    let calls = Rc::new(RefCell::new(Vec::new()));
    let (mut artboard, mut machine, _) =
        scripted_drawable_input_artboard_and_machine(Box::new(RecordingDrawablePointerScript {
            hit: crate::ScriptedDrawablePointerHit::HitOpaque,
            calls: Rc::clone(&calls),
        }));

    assert!(machine.pointer_down(&mut artboard, 11.0, 12.0, 7));
    assert_eq!(
        calls.borrow().as_slice(),
        [RecordedDrawablePointerCall {
            method: ScriptMethod::PointerDown,
            pointer_id: 7,
            local_x: 11.0,
            local_y: 12.0,
        }]
    );
}

#[test]
fn scripted_drawable_pointer_resource_error_restores_hit_ownership_before_returning() {
    let (mut artboard, mut machine, _) = scripted_drawable_input_artboard_and_machine(Box::new(
        ResourceFailingDrawablePointerScript,
    ));
    let hit_count = machine.hit_components.len();

    for pointer_id in [1, 2] {
        let error = machine
            .try_pointer_down_with_timestamp_and_script_host(
                &mut artboard,
                1.0,
                2.0,
                pointer_id,
                0.0,
                &mut NoopScriptHost,
            )
            .expect_err("resource-coded callback failure remains terminal");
        assert_eq!(error.resource_code(), Some("script.resource.pointer"));
        assert_eq!(machine.hit_components.len(), hit_count);
    }
}

fn scripted_drawable_subtype_input_artboard_and_machine(
    scripted_type_name: &str,
    script: Box<dyn ScriptInstance>,
) -> (ArtboardInstance, StateMachineInstance, u32) {
    scripted_drawable_subtype_input_artboard_and_machine_with_mount_order(
        scripted_type_name,
        script,
        true,
    )
}

fn scripted_drawable_subtype_input_artboard_and_machine_with_mount_order(
    scripted_type_name: &str,
    script: Box<dyn ScriptInstance>,
    mount_before_machine: bool,
) -> (ArtboardInstance, StateMachineInstance, u32) {
    scripted_drawable_subtype_input_artboard_and_machine_with_optional_script(
        scripted_type_name,
        Some(script),
        mount_before_machine,
    )
}

fn scripted_drawable_subtype_input_artboard_and_machine_with_optional_script(
    scripted_type_name: &str,
    script: Option<Box<dyn ScriptInstance>>,
    mount_before_machine: bool,
) -> (ArtboardInstance, StateMachineInstance, u32) {
    fn record(type_name: &str, properties: Vec<FixtureProperty>) -> FixtureRecord {
        FixtureRecord {
            type_key: nuxie_schema::definition_by_name(type_name)
                .unwrap_or_else(|| panic!("missing schema definition {type_name}"))
                .type_key
                .int,
            properties,
        }
    }
    fn parent(type_name: &str, local_id: u64) -> FixtureProperty {
        FixtureProperty {
            key: crate::properties::property_key_for_name(type_name, "parentId")
                .unwrap_or_else(|| panic!("missing {type_name}.parentId")),
            value: FixtureValue::Uint(local_id),
        }
    }
    fn uint(type_name: &str, property_name: &str, value: u64) -> FixtureProperty {
        FixtureProperty {
            key: crate::properties::property_key_for_name(type_name, property_name)
                .unwrap_or_else(|| panic!("missing {type_name}.{property_name}")),
            value: FixtureValue::Uint(value),
        }
    }
    fn double(type_name: &str, property_name: &str, value: f32) -> FixtureProperty {
        FixtureProperty {
            key: crate::properties::property_key_for_name(type_name, property_name)
                .unwrap_or_else(|| panic!("missing {type_name}.{property_name}")),
            value: FixtureValue::Double(value),
        }
    }

    let file = RuntimeFile::from_fixture_records(vec![
        record("Backboard", Vec::new()),
        record("Artboard", Vec::new()),
        record(
            scripted_type_name,
            vec![
                parent(scripted_type_name, 0),
                // Generated C++ `WorldTransformComponentBase` initializes
                // `m_Opacity = 1`; make that omitted default explicit for
                // this synthetic low-level record fixture.
                double(scripted_type_name, "opacity", 1.0),
            ],
        ),
        record(
            "FocusData",
            vec![
                parent("FocusData", 1),
                // Generated C++ `FocusDataBase` initializes
                // `m_FocusFlags = 7`; author it explicitly because this
                // low-level synthetic record builder does not materialize
                // omitted generated defaults.
                uint("FocusData", "focusFlags", 7),
            ],
        ),
        record("SemanticData", vec![parent("SemanticData", 1)]),
        record("StateMachine", Vec::new()),
    ])
    .expect("scripted-input records import");
    let graph = GraphFile::from_runtime_file(&file).expect("scripted-input graph builds");
    let mut artboard = ArtboardInstance::from_graph_with_artboards(
        &file,
        graph.artboards.first().expect("scripted-input artboard"),
        &graph.artboards,
    )
    .expect("scripted-input artboard instantiates");
    let global_id = artboard
        .component(1)
        .expect("scripted drawable occurrence")
        .global_id;
    let mut script = script;
    if mount_before_machine && script.is_some() {
        artboard.set_script_instance_for_global(
            global_id,
            script.take().expect("script mounts exactly once"),
        );
    }
    // C++ constructs state-machine focus groups after the Artboard's
    // initial component update has produced world opacity. This synthetic
    // fixture bypasses the normal facade initialization, so perform that
    // same owner update explicitly.
    artboard.update_components();
    let mut machine = artboard
        .state_machine_instance(0)
        .expect("scripted-input state machine");
    if !mount_before_machine && script.is_some() {
        artboard.set_script_instance_for_global(
            global_id,
            script.take().expect("script mounts exactly once"),
        );
    }
    assert!(machine.focus.set_focus_target(1));
    // The synthetic fixture is already at the post-constructor boundary:
    // its ScriptedDrawable table was mounted directly instead of through
    // the facade's synchronous C++ ScriptedObject initialization pass.
    machine.mark_scripted_object_initialization_complete(None);
    (artboard, machine, global_id)
}

#[test]
fn facade_late_script_mount_completes_cpp_input_group_construction_once() {
    let calls = Rc::new(RefCell::new(Vec::new()));
    let (mut artboard, mut machine, _) =
        scripted_drawable_subtype_input_artboard_and_machine_with_mount_order(
            "ScriptedDrawable",
            Box::new(RecordingDrawableInputScript {
                label: "late",
                methods: vec![ScriptMethod::KeyboardEvent, ScriptMethod::GamepadConnected],
                handled: true,
                calls: Rc::clone(&calls),
            }),
            false,
        );

    assert!(
        machine.key_input(&mut artboard, 66, 0, true, false),
        "first dispatch completes C++'s post-script input-group scan"
    );
    machine.synchronize_scripted_input_groups(&artboard);
    machine.synchronize_scripted_input_groups(&artboard);
    assert!(machine.gamepad_dispatch(
        &mut artboard,
        ScriptListenerInvocation::GamepadConnected {
            snapshot: ScriptGamepadSnapshot {
                device_id: 7,
                button_mask: 1,
                button_values: vec![1.0],
                axes: vec![0.25],
                mapping: crate::ScriptGamepadMappingKind::Standard,
            },
        },
    ));
    assert_eq!(
        calls
            .borrow()
            .iter()
            .map(|call| call.invocation.clone())
            .collect::<Vec<_>>(),
        [
            ScriptListenerInvocation::Keyboard {
                key: 66,
                modifiers: 0,
                is_pressed: true,
                is_repeat: false,
            },
            ScriptListenerInvocation::GamepadConnected {
                snapshot: ScriptGamepadSnapshot {
                    device_id: 7,
                    button_mask: 1,
                    button_values: vec![1.0],
                    axes: vec![0.25],
                    mapping: crate::ScriptGamepadMappingKind::Standard,
                },
            },
        ],
        "idempotent completion must retain one authored input occurrence"
    );
}

#[test]
fn replacing_scripted_input_occurrence_rebuilds_groups_without_duplicates() {
    let initial_calls = Rc::new(RefCell::new(Vec::new()));
    let (mut artboard, mut machine, global_id) =
        scripted_drawable_input_artboard_and_machine(Box::new(RecordingDrawableInputScript {
            label: "initial",
            methods: vec![ScriptMethod::KeyboardEvent],
            handled: true,
            calls: Rc::clone(&initial_calls),
        }));

    let no_method_calls = Rc::new(RefCell::new(Vec::new()));
    artboard.set_script_instance_for_global(
        global_id,
        Box::new(RecordingDrawableInputScript {
            label: "no method",
            methods: Vec::new(),
            handled: true,
            calls: Rc::clone(&no_method_calls),
        }),
    );
    assert!(!machine.key_input(&mut artboard, 65, 0, true, false));
    assert!(no_method_calls.borrow().is_empty());

    let replacement_calls = Rc::new(RefCell::new(Vec::new()));
    artboard.set_script_instance_for_global(
        global_id,
        Box::new(RecordingDrawableInputScript {
            label: "replacement",
            methods: vec![ScriptMethod::KeyboardEvent],
            handled: true,
            calls: Rc::clone(&replacement_calls),
        }),
    );
    machine.synchronize_scripted_input_groups(&artboard);
    machine.synchronize_scripted_input_groups(&artboard);
    assert!(machine.key_input(&mut artboard, 66, 0, true, false));
    assert_eq!(
        replacement_calls.borrow().len(),
        1,
        "the replacement occurrence owns exactly one C++ KeyboardListenerGroup"
    );
    assert!(initial_calls.borrow().is_empty());
}

#[test]
fn scripted_drawable_subtypes_register_keyboard_text_and_gamepad_paths() {
    let calls = Rc::new(RefCell::new(Vec::new()));
    let (mut artboard, mut machine, _) = scripted_drawable_subtype_input_artboard_and_machine(
        "ScriptedLayout",
        Box::new(RecordingDrawableInputScript {
            label: "layout",
            methods: vec![
                ScriptMethod::KeyboardEvent,
                ScriptMethod::TextEvent,
                ScriptMethod::GamepadConnected,
            ],
            handled: true,
            calls: Rc::clone(&calls),
        }),
    );

    assert!(machine.key_input(&mut artboard, 65, 0, true, false));
    assert!(machine.text_input(&mut artboard, "owned"));
    assert!(machine.gamepad_dispatch(
        &mut artboard,
        ScriptListenerInvocation::GamepadConnected {
            snapshot: ScriptGamepadSnapshot {
                device_id: 7,
                button_mask: 1,
                button_values: vec![1.0],
                axes: vec![0.25],
                mapping: crate::ScriptGamepadMappingKind::Standard,
            },
        },
    ));
    assert_eq!(calls.borrow().len(), 3);
}

#[test]
fn serialized_script_method_mask_controls_listenerless_input_membership() {
    let off_calls = Rc::new(RefCell::new(Vec::new()));
    let (mut off_artboard, mut off_machine, off_global_id) =
        scripted_drawable_subtype_input_artboard_and_machine(
            "ScriptedDrawable",
            Box::new(RecordingDrawableInputScript {
                label: "mask off",
                methods: vec![
                    ScriptMethod::KeyboardEvent,
                    ScriptMethod::TextEvent,
                    ScriptMethod::GamepadConnected,
                ],
                handled: true,
                calls: Rc::clone(&off_calls),
            }),
        );
    off_artboard.set_script_instance_for_global_with_implemented_methods(
        off_global_id,
        Box::new(RecordingDrawableInputScript {
            label: "mask off",
            methods: vec![
                ScriptMethod::KeyboardEvent,
                ScriptMethod::TextEvent,
                ScriptMethod::GamepadConnected,
            ],
            handled: true,
            calls: Rc::clone(&off_calls),
        }),
        0,
    );
    off_machine.synchronize_scripted_input_groups(&off_artboard);
    assert!(!off_machine.key_input(&mut off_artboard, 65, 0, true, false));
    assert!(!off_machine.text_input(&mut off_artboard, "masked"));
    assert!(!off_machine.gamepad_dispatch(
        &mut off_artboard,
        ScriptListenerInvocation::GamepadConnected {
            snapshot: ScriptGamepadSnapshot {
                device_id: 7,
                button_mask: 0,
                button_values: Vec::new(),
                axes: Vec::new(),
                mapping: crate::ScriptGamepadMappingKind::Unknown,
            },
        },
    ));
    assert!(off_calls.borrow().is_empty());

    let missing_calls = Rc::new(RefCell::new(Vec::new()));
    let (mut missing_artboard, mut missing_machine, missing_global_id) =
        scripted_drawable_subtype_input_artboard_and_machine(
            "ScriptedDrawable",
            Box::new(RecordingDrawableInputScript {
                label: "mask on, fields absent",
                methods: Vec::new(),
                handled: true,
                calls: Rc::clone(&missing_calls),
            }),
        );
    let mask = crate::script_asset::RuntimeScriptImplementedMethods::KEYBOARD
        | crate::script_asset::RuntimeScriptImplementedMethods::TEXT
        | crate::script_asset::RuntimeScriptImplementedMethods::GAMEPAD_CONNECT;
    missing_artboard.set_script_instance_for_global_with_implemented_methods(
        missing_global_id,
        Box::new(RecordingDrawableInputScript {
            label: "mask on, fields absent",
            methods: Vec::new(),
            handled: true,
            calls: Rc::clone(&missing_calls),
        }),
        mask,
    );
    missing_machine.synchronize_scripted_input_groups(&missing_artboard);
    assert_eq!(missing_machine.keyboard_listener_groups.len(), 1);
    assert_eq!(missing_machine.gamepad_scripted_drawables.len(), 1);
    assert!(!missing_machine.key_input(&mut missing_artboard, 65, 0, true, false));
    assert!(!missing_machine.text_input(&mut missing_artboard, "missing"));
    assert!(!missing_machine.gamepad_dispatch(
        &mut missing_artboard,
        ScriptListenerInvocation::GamepadConnected {
            snapshot: ScriptGamepadSnapshot {
                device_id: 7,
                button_mask: 0,
                button_values: Vec::new(),
                axes: Vec::new(),
                mapping: crate::ScriptGamepadMappingKind::Unknown,
            },
        },
    ));
    assert!(
        missing_calls.borrow().is_empty(),
        "C++ retains the group from serialized wants bits but missing Lua fields remain inert"
    );
}

fn nested_scripted_drawable_input_artboard_and_machine(
    ancestor_script: Box<dyn ScriptInstance>,
    leaf_script: Box<dyn ScriptInstance>,
) -> (ArtboardInstance, StateMachineInstance) {
    fn record(type_name: &str, properties: Vec<FixtureProperty>) -> FixtureRecord {
        FixtureRecord {
            type_key: nuxie_schema::definition_by_name(type_name)
                .unwrap_or_else(|| panic!("missing schema definition {type_name}"))
                .type_key
                .int,
            properties,
        }
    }
    fn parent(type_name: &str, local_id: u64) -> FixtureProperty {
        FixtureProperty {
            key: crate::properties::property_key_for_name(type_name, "parentId")
                .unwrap_or_else(|| panic!("missing {type_name}.parentId")),
            value: FixtureValue::Uint(local_id),
        }
    }
    fn uint(type_name: &str, name: &str, value: u64) -> FixtureProperty {
        FixtureProperty {
            key: crate::properties::property_key_for_name(type_name, name)
                .unwrap_or_else(|| panic!("missing {type_name}.{name}")),
            value: FixtureValue::Uint(value),
        }
    }
    fn double(type_name: &str, name: &str, value: f32) -> FixtureProperty {
        FixtureProperty {
            key: crate::properties::property_key_for_name(type_name, name)
                .unwrap_or_else(|| panic!("missing {type_name}.{name}")),
            value: FixtureValue::Double(value),
        }
    }

    let file = RuntimeFile::from_fixture_records(vec![
        record("Backboard", Vec::new()),
        record("Artboard", Vec::new()),
        record(
            "ScriptedDrawable",
            vec![
                parent("ScriptedDrawable", 0),
                double("ScriptedDrawable", "opacity", 1.0),
            ],
        ),
        record(
            "FocusData",
            vec![parent("FocusData", 1), uint("FocusData", "focusFlags", 7)],
        ),
        record(
            "ScriptedDrawable",
            vec![
                parent("ScriptedDrawable", 1),
                double("ScriptedDrawable", "opacity", 1.0),
            ],
        ),
        record(
            "FocusData",
            vec![parent("FocusData", 3), uint("FocusData", "focusFlags", 7)],
        ),
        record("StateMachine", Vec::new()),
    ])
    .expect("nested scripted-input records import");
    let graph = GraphFile::from_runtime_file(&file).expect("nested scripted-input graph builds");
    let mut artboard = ArtboardInstance::from_graph_with_artboards(
        &file,
        graph
            .artboards
            .first()
            .expect("nested scripted-input artboard"),
        &graph.artboards,
    )
    .expect("nested scripted-input artboard instantiates");
    let ancestor_global = artboard.component(1).expect("ancestor drawable").global_id;
    let leaf_global = artboard.component(3).expect("leaf drawable").global_id;
    artboard.set_script_instance_for_global(ancestor_global, ancestor_script);
    artboard.set_script_instance_for_global(leaf_global, leaf_script);
    artboard.update_components();
    let mut machine = artboard
        .state_machine_instance(0)
        .expect("nested scripted-input state machine");
    assert!(machine.focus.set_focus_target(3));
    (artboard, machine)
}

#[test]
fn listenerless_scripted_keyboard_and_text_dispatch_precede_listener_paths() {
    let calls = Rc::new(RefCell::new(Vec::new()));
    let (mut artboard, mut machine, _) =
        scripted_drawable_input_artboard_and_machine(Box::new(RecordingDrawableInputScript {
            label: "focused",
            methods: vec![ScriptMethod::KeyboardEvent, ScriptMethod::TextEvent],
            handled: true,
            calls: Rc::clone(&calls),
        }));

    assert!(machine.key_input(&mut artboard, 65, 3, true, true));
    assert!(machine.text_input(&mut artboard, "owned"));
    assert_eq!(
        calls
            .borrow()
            .iter()
            .map(|call| call.invocation.clone())
            .collect::<Vec<_>>(),
        [
            ScriptListenerInvocation::Keyboard {
                key: 65,
                modifiers: 3,
                is_pressed: true,
                is_repeat: true,
            },
            ScriptListenerInvocation::TextInput {
                text: "owned".to_owned(),
            },
        ]
    );
}

#[test]
fn direct_scripted_input_retains_terminal_resource_failure() {
    let (mut artboard, mut machine, _) =
        scripted_drawable_input_artboard_and_machine(Box::new(ResourceFailingDrawableInputScript));

    assert!(!machine.key_input(&mut artboard, 65, 0, true, false));
    assert_eq!(
        machine.script_error().and_then(ScriptError::resource_code),
        Some("script.resource.host_commands")
    );
}

#[test]
fn terminal_resource_failure_stops_every_later_focused_input_callback() {
    let resource_code = "script.resource.host_commands";

    let calls = Rc::new(RefCell::new(Vec::new()));
    let (mut artboard, mut machine) = nested_scripted_drawable_input_artboard_and_machine(
        Box::new(RecordingDrawableInputScript {
            label: "ancestor",
            methods: vec![ScriptMethod::KeyboardEvent],
            handled: true,
            calls: Rc::clone(&calls),
        }),
        Box::new(FailingDrawableInputScript {
            label: "leaf",
            methods: vec![ScriptMethod::KeyboardEvent],
            resource_code: Some(resource_code),
            calls: Rc::clone(&calls),
        }),
    );
    assert!(!machine.key_input(&mut artboard, 65, 0, true, false));
    assert_eq!(
        calls
            .borrow()
            .iter()
            .map(|call| call.label)
            .collect::<Vec<_>>(),
        ["leaf"]
    );
    assert_eq!(
        machine.script_error().and_then(ScriptError::resource_code),
        Some(resource_code)
    );

    let calls = Rc::new(RefCell::new(Vec::new()));
    let (mut artboard, mut machine) = nested_scripted_drawable_input_artboard_and_machine(
        Box::new(RecordingDrawableInputScript {
            label: "ancestor",
            methods: vec![ScriptMethod::TextEvent],
            handled: true,
            calls: Rc::clone(&calls),
        }),
        Box::new(FailingDrawableInputScript {
            label: "leaf",
            methods: vec![ScriptMethod::TextEvent],
            resource_code: Some(resource_code),
            calls: Rc::clone(&calls),
        }),
    );
    assert!(!machine.text_input(&mut artboard, "owned"));
    assert_eq!(
        calls
            .borrow()
            .iter()
            .map(|call| call.label)
            .collect::<Vec<_>>(),
        ["leaf"]
    );
    assert_eq!(
        machine.script_error().and_then(ScriptError::resource_code),
        Some(resource_code)
    );

    let calls = Rc::new(RefCell::new(Vec::new()));
    let (mut artboard, mut machine) = nested_scripted_drawable_input_artboard_and_machine(
        Box::new(FailingDrawableInputScript {
            label: "ancestor",
            methods: vec![ScriptMethod::GamepadConnected],
            resource_code: Some(resource_code),
            calls: Rc::clone(&calls),
        }),
        Box::new(RecordingDrawableInputScript {
            label: "leaf",
            methods: vec![ScriptMethod::GamepadConnected],
            handled: true,
            calls: Rc::clone(&calls),
        }),
    );
    assert!(!machine.gamepad_dispatch(
        &mut artboard,
        ScriptListenerInvocation::GamepadConnected {
            snapshot: ScriptGamepadSnapshot {
                device_id: 7,
                button_mask: 0,
                button_values: Vec::new(),
                axes: Vec::new(),
                mapping: ScriptGamepadMappingKind::Standard,
            },
        },
    ));
    assert_eq!(
        calls
            .borrow()
            .iter()
            .map(|call| call.label)
            .collect::<Vec<_>>(),
        ["ancestor"]
    );
    assert_eq!(
        machine.script_error().and_then(ScriptError::resource_code),
        Some(resource_code)
    );
}

#[test]
fn ordinary_protected_input_failure_continues_the_cpp_callback_order() {
    let calls = Rc::new(RefCell::new(Vec::new()));
    let (mut artboard, mut machine) = nested_scripted_drawable_input_artboard_and_machine(
        Box::new(RecordingDrawableInputScript {
            label: "ancestor",
            methods: vec![ScriptMethod::KeyboardEvent],
            handled: true,
            calls: Rc::clone(&calls),
        }),
        Box::new(FailingDrawableInputScript {
            label: "leaf",
            methods: vec![ScriptMethod::KeyboardEvent],
            resource_code: None,
            calls: Rc::clone(&calls),
        }),
    );
    assert!(machine.key_input(&mut artboard, 65, 0, true, false));
    assert_eq!(
        calls
            .borrow()
            .iter()
            .map(|call| call.label)
            .collect::<Vec<_>>(),
        ["leaf", "ancestor"]
    );
    assert!(machine.script_error().is_none());

    let calls = Rc::new(RefCell::new(Vec::new()));
    let (mut artboard, mut machine) = nested_scripted_drawable_input_artboard_and_machine(
        Box::new(RecordingDrawableInputScript {
            label: "ancestor",
            methods: vec![ScriptMethod::TextEvent],
            handled: true,
            calls: Rc::clone(&calls),
        }),
        Box::new(FailingDrawableInputScript {
            label: "leaf",
            methods: vec![ScriptMethod::TextEvent],
            resource_code: None,
            calls: Rc::clone(&calls),
        }),
    );
    assert!(machine.text_input(&mut artboard, "owned"));
    assert_eq!(
        calls
            .borrow()
            .iter()
            .map(|call| call.label)
            .collect::<Vec<_>>(),
        ["leaf", "ancestor"]
    );
    assert!(machine.script_error().is_none());

    let calls = Rc::new(RefCell::new(Vec::new()));
    let (mut artboard, mut machine) = nested_scripted_drawable_input_artboard_and_machine(
        Box::new(FailingDrawableInputScript {
            label: "ancestor",
            methods: vec![ScriptMethod::GamepadConnected],
            resource_code: None,
            calls: Rc::clone(&calls),
        }),
        Box::new(RecordingDrawableInputScript {
            label: "leaf",
            methods: vec![ScriptMethod::GamepadConnected],
            handled: true,
            calls: Rc::clone(&calls),
        }),
    );
    assert!(machine.gamepad_dispatch(
        &mut artboard,
        ScriptListenerInvocation::GamepadConnected {
            snapshot: ScriptGamepadSnapshot {
                device_id: 7,
                button_mask: 0,
                button_values: Vec::new(),
                axes: Vec::new(),
                mapping: ScriptGamepadMappingKind::Standard,
            },
        },
    ));
    assert_eq!(
        calls
            .borrow()
            .iter()
            .map(|call| call.label)
            .collect::<Vec<_>>(),
        ["ancestor", "leaf"]
    );
    assert!(machine.script_error().is_none());
}

#[test]
fn focused_keyboard_dispatch_bubbles_leaf_to_parent_and_stops_when_handled() {
    let calls = Rc::new(RefCell::new(Vec::new()));
    let (mut artboard, mut machine) = nested_scripted_drawable_input_artboard_and_machine(
        Box::new(RecordingDrawableInputScript {
            label: "ancestor",
            methods: vec![ScriptMethod::KeyboardEvent],
            handled: true,
            calls: Rc::clone(&calls),
        }),
        Box::new(RecordingDrawableInputScript {
            label: "leaf",
            methods: vec![ScriptMethod::KeyboardEvent],
            handled: true,
            calls: Rc::clone(&calls),
        }),
    );

    assert!(machine.key_input(&mut artboard, 65, 0, true, false));
    assert_eq!(
        calls
            .borrow()
            .iter()
            .map(|call| call.label)
            .collect::<Vec<_>>(),
        ["leaf"]
    );

    calls.borrow_mut().clear();
    let leaf_global = artboard.component(3).expect("leaf drawable").global_id;
    artboard.set_script_instance_for_global(
        leaf_global,
        Box::new(RecordingDrawableInputScript {
            label: "leaf",
            methods: vec![ScriptMethod::KeyboardEvent],
            handled: false,
            calls: Rc::clone(&calls),
        }),
    );
    assert!(machine.key_input(&mut artboard, 66, 0, true, false));
    assert_eq!(
        calls
            .borrow()
            .iter()
            .map(|call| call.label)
            .collect::<Vec<_>>(),
        ["leaf", "ancestor"]
    );
}

#[test]
fn text_input_parent_precedes_scripted_and_listener_keyboard_dispatch() {
    fn record(type_name: &str, properties: Vec<FixtureProperty>) -> FixtureRecord {
        FixtureRecord {
            type_key: nuxie_schema::definition_by_name(type_name)
                .unwrap_or_else(|| panic!("missing schema definition {type_name}"))
                .type_key
                .int,
            properties,
        }
    }
    fn property(type_name: &str, name: &str, value: FixtureValue) -> FixtureProperty {
        FixtureProperty {
            key: crate::properties::property_key_for_name(type_name, name)
                .unwrap_or_else(|| panic!("missing {type_name}.{name}")),
            value,
        }
    }

    let file = RuntimeFile::from_fixture_records(vec![
        record("Backboard", Vec::new()),
        record("Artboard", Vec::new()),
        record(
            "TextInput",
            vec![
                property("TextInput", "parentId", FixtureValue::Uint(0)),
                property("TextInput", "opacity", FixtureValue::Double(1.0)),
                property("TextInput", "multiline", FixtureValue::Bool(true)),
                property(
                    "TextInput",
                    "text",
                    FixtureValue::String("seed".to_owned()),
                ),
            ],
        ),
        record(
            "FocusData",
            vec![
                property("FocusData", "parentId", FixtureValue::Uint(1)),
                property("FocusData", "focusFlags", FixtureValue::Uint(7)),
            ],
        ),
        record("StateMachine", Vec::new()),
    ])
    .expect("TextInput precedence records import");
    let graph = GraphFile::from_runtime_file(&file).expect("TextInput precedence graph builds");
    let mut artboard = ArtboardInstance::from_graph_with_artboards(
        &file,
        graph
            .artboards
            .first()
            .expect("TextInput precedence artboard"),
        &graph.artboards,
    )
    .expect("TextInput precedence artboard instantiates");
    artboard.update_components();
    let mut machine = artboard
        .state_machine_instance(0)
        .expect("TextInput precedence state machine");
    assert!(machine.focus.set_focus_target(1));
    let listener = RuntimeStateMachineListener {
        name: None,
        target_local_id: 1,
        is_single: false,
        listener_types: vec![
            RuntimeListenerType::Keyboard,
            RuntimeListenerType::TextInput,
        ],
        event_local_indices: Vec::new(),
        view_model_path: None,
        view_model_input_types: Vec::new(),
        gamepad_input_types: Vec::new(),
        keyboard_input_types: Vec::new(),
        semantic_input_types: Vec::new(),
        hit_paths: Vec::new(),
        listener_actions: vec![RuntimeScheduledListenerAction::FocusClear(
            RuntimeFocusActionClear::for_test(0),
        )],
    };
    machine.keyboard_listener_groups =
        vec![RuntimeKeyboardListenerGroup::new(0, 2, &listener).expect("TextInput listener group")];
    machine.listener_definitions = Arc::new(vec![listener]);

    assert!(machine.key_input(&mut artboard, 259, 0, true, false));
    assert_eq!(artboard.text_input_display_text(1).as_deref(), Some("seed"));
    assert!(!machine.focus.focused_listener_chain().is_empty());
    assert!(!machine.key_input(&mut artboard, 66, 0, true, false));
    assert!(!machine.focus.focused_listener_chain().is_empty());
    assert!(machine.text_input(&mut artboard, "owned"));
    assert_eq!(
        artboard.text_input_display_text(1).as_deref(),
        Some("ownedseed")
    );
    assert!(!machine.focus.focused_listener_chain().is_empty());
}

#[test]
fn mixed_report_listener_does_not_register_semantic_or_focus_groups() {
    let (mut artboard, mut machine, _) =
        scripted_drawable_input_artboard_and_machine(Box::new(RecordingDrawableInputScript {
            label: "focused",
            methods: Vec::new(),
            handled: false,
            calls: Rc::new(RefCell::new(Vec::new())),
        }));
    machine.listener_definitions = Arc::new(vec![RuntimeStateMachineListener {
        name: None,
        target_local_id: 1,
        is_single: false,
        listener_types: vec![
            RuntimeListenerType::Event,
            RuntimeListenerType::SemanticAction,
            RuntimeListenerType::Focus,
            RuntimeListenerType::Keyboard,
            RuntimeListenerType::Gamepad,
            RuntimeListenerType::DragEnd,
        ],
        event_local_indices: Vec::new(),
        view_model_path: None,
        view_model_input_types: Vec::new(),
        gamepad_input_types: Vec::new(),
        keyboard_input_types: Vec::new(),
        semantic_input_types: Vec::new(),
        hit_paths: Vec::new(),
        listener_actions: Vec::new(),
    }]);
    machine.focus_listener_groups.clear();
    machine.keyboard_listener_groups.clear();
    machine.gamepad_listener_groups.clear();
    machine.semantic_listener_groups.clear();

    machine.initialize_authored_listener_categories(&mut artboard);

    assert!(machine.focus_listener_groups.is_empty());
    assert!(
        machine
            .keyboard_listener_groups
            .iter()
            .all(|group| group.listener_index != Some(0)),
        "the mixed report listener must not register; an independent listener-less scripted group may still exist"
    );
    assert!(machine.gamepad_listener_groups.is_empty());
    assert!(machine.semantic_listener_groups.is_empty());

    let pointer = RuntimePointerInput {
        x: 12.0,
        y: 34.0,
        previous_x: 12.0,
        previous_y: 34.0,
        timestamp_seconds: 0.0,
        id: 7,
    };
    assert!(
        !machine
            .dispatch_pointer_listener_type_for_target(
                &mut artboard,
                1,
                pointer,
                RuntimeListenerType::DragEnd,
                None,
                &mut NoopScriptHost,
                None,
            )
            .expect("mixed report listener dispatch")
    );
}

#[test]
fn missing_pointer_hit_path_disables_only_pointer_not_focus_dispatch() {
    let (mut artboard, mut machine, _) =
        scripted_drawable_input_artboard_and_machine(Box::new(RecordingDrawableInputScript {
            label: "mixed",
            methods: Vec::new(),
            handled: false,
            calls: Rc::new(RefCell::new(Vec::new())),
        }));
    machine.listener_definitions = Arc::new(vec![RuntimeStateMachineListener {
        name: None,
        target_local_id: 1,
        is_single: false,
        listener_types: vec![RuntimeListenerType::Down, RuntimeListenerType::Blur],
        event_local_indices: Vec::new(),
        view_model_path: None,
        view_model_input_types: Vec::new(),
        gamepad_input_types: Vec::new(),
        keyboard_input_types: Vec::new(),
        semantic_input_types: Vec::new(),
        hit_paths: Vec::new(),
        listener_actions: Vec::new(),
    }]);
    machine.focus_listener_groups.clear();
    machine.keyboard_listener_groups.clear();
    machine.gamepad_listener_groups.clear();
    machine.semantic_listener_groups.clear();
    machine.initialize_authored_listener_categories(&mut artboard);

    assert!(
        !machine.pointer_down(&mut artboard, 0.0, 0.0, 1),
        "C++ retains the mixed listener but registers no pointer hit target"
    );
    assert!(
        machine.clear_focus(),
        "the independent focus channel remains registered"
    );
    assert_eq!(
        machine.queued_focus_events.len(),
        1,
        "C++ queues the matching non-pointer listener occurrence"
    );
}

#[test]
fn semantic_callbacks_apply_constraints_preserve_duplicates_and_defer_actions() {
    #[derive(Debug)]
    struct StubSemanticNodeResolver {
        calls: Rc<RefCell<Vec<u32>>>,
    }

    impl SemanticNodeResolver for StubSemanticNodeResolver {
        fn semantic_data_local_id(&self, semantic_node_id: u32) -> Option<usize> {
            self.calls.borrow_mut().push(semantic_node_id);
            (semantic_node_id == 77).then_some(2)
        }
    }

    fn record(type_name: &str, properties: Vec<FixtureProperty>) -> FixtureRecord {
        FixtureRecord {
            type_key: nuxie_schema::definition_by_name(type_name)
                .unwrap_or_else(|| panic!("missing schema definition {type_name}"))
                .type_key
                .int,
            properties,
        }
    }
    fn uint(type_name: &str, name: &str, value: u64) -> FixtureProperty {
        FixtureProperty {
            key: crate::properties::property_key_for_name(type_name, name)
                .unwrap_or_else(|| panic!("missing {type_name}.{name}")),
            value: FixtureValue::Uint(value),
        }
    }

    let file = RuntimeFile::from_fixture_records(vec![
        record("Backboard", Vec::new()),
        record("Artboard", Vec::new()),
        record("Node", vec![uint("Node", "parentId", 0)]),
        record("SemanticData", vec![uint("SemanticData", "parentId", 1)]),
        record("StateMachine", Vec::new()),
        record("StateMachineBool", Vec::new()),
        record(
            "StateMachineListener",
            vec![uint("StateMachineListener", "targetId", 1)],
        ),
        record(
            "ListenerInputTypeSemantic",
            vec![uint(
                "ListenerInputTypeSemantic",
                "listenerTypeValue",
                RuntimeListenerType::SemanticAction as u64,
            )],
        ),
        record(
            "SemanticInput",
            vec![uint("SemanticInput", "actionType", 0)],
        ),
        record(
            "ListenerBoolChange",
            vec![
                uint("ListenerBoolChange", "inputId", 0),
                // Values other than 0/1 toggle in pinned C++.
                uint("ListenerBoolChange", "value", 2),
            ],
        ),
    ])
    .expect("semantic listener records");
    let graph = GraphFile::from_runtime_file(&file).expect("semantic listener graph");
    let mut artboard = ArtboardInstance::from_graph_with_artboards(
        &file,
        graph.artboards.first().expect("semantic listener artboard"),
        &graph.artboards,
    )
    .expect("semantic listener instance");
    let mut machine = artboard
        .state_machine_instance(0)
        .expect("semantic listener machine");

    assert!(machine.enable_semantics());
    assert_eq!(
        machine.semantic_listener_groups[0].semantic_data_local_id,
        2
    );
    let semantic_id = machine
        .drain_semantics_diff(&mut artboard)
        .expect("production retained semantic tree drains")
        .added
        .into_iter()
        .find(|node| node.id != 0)
        .expect("retained SemanticData emits a node")
        .id;
    assert!(machine.fire_semantic_action(semantic_id, 1));
    assert!(
        machine.queued_semantic_events.is_empty(),
        "the retained listener applies its authored tap-only constraint"
    );
    assert!(machine.fire_semantic_action(semantic_id, 0));
    assert_eq!(machine.queued_semantic_events.len(), 1);
    machine.queued_semantic_events.clear();
    assert!(
        !machine.fire_semantic_action(77, 0),
        "W41's recorded-seam contract makes the production-default absent resolver a silent no-op"
    );
    let resolver_calls = Rc::new(RefCell::new(Vec::new()));
    machine.set_semantic_node_resolver(Some(Rc::new(StubSemanticNodeResolver {
        calls: Rc::clone(&resolver_calls),
    })));
    assert!(
        machine.fire_semantic_action(77, 1),
        "increase dispatch reaches SemanticData even though this listener accepts only tap"
    );
    assert!(
        machine.fire_semantic_action(77, 2),
        "decrease dispatch reaches the injected SemanticData resolver seam"
    );
    assert!(
        machine.queued_semantic_events.is_empty(),
        "SemanticData applies the listener's action constraint"
    );
    assert!(
        !machine.fire_semantic_action(77, 3),
        "an out-of-range action is a no-op after resolving a valid node"
    );
    assert!(
        !machine.semantic_action_for_target(1, 1),
        "a nonmatching action is not registered"
    );
    assert!(
        machine.fire_semantic_action(77, 0),
        "tap selects the SemanticData action and queues its listener callback"
    );
    assert_eq!(
        resolver_calls.borrow().as_slice(),
        [77, 77, 77, 77],
        "tap, increase, decrease, and invalid actions all reach the injected node resolver"
    );
    assert_eq!(
        machine.semantic_manager_phase_trace,
        [
            "create-internal-recorded-seam",
            "build-tree-recorded-seam",
            "node-by-id-recorded-seam",
            "semantic-data-recorded-seam",
            "fire-increase-recorded-data-seam",
            "node-by-id-recorded-seam",
            "semantic-data-recorded-seam",
            "fire-decrease-recorded-data-seam",
            "node-by-id-recorded-seam",
            "semantic-data-recorded-seam",
            "node-by-id-recorded-seam",
            "semantic-data-recorded-seam",
            "fire-tap-recorded-data-seam",
        ],
        "the family-owned action switch selects the recorded SemanticData callback"
    );
    assert_eq!(
        machine
            .input(0)
            .and_then(StateMachineInputInstance::bool_value),
        Some(false),
        "C++ queues the callback instead of executing its actions inline"
    );
    assert!(machine.apply_local_event_listeners(&mut artboard, 0, None));
    assert_eq!(
        machine
            .input(0)
            .and_then(StateMachineInputInstance::bool_value),
        Some(true)
    );

    assert!(machine.fire_semantic_action(77, 0));
    assert!(machine.fire_semantic_action(77, 0));
    assert!(
        machine.apply_local_event_listeners(&mut artboard, 0, None),
        "both duplicate callback occurrences execute in FIFO order"
    );
    assert_eq!(
        machine
            .input(0)
            .and_then(StateMachineInputInstance::bool_value),
        Some(true),
        "two retained toggle callbacks leave the value unchanged"
    );
}

#[test]
fn focus_listener_groups_queue_matching_duplicate_occurrences_in_registration_order() {
    let (mut artboard, mut machine, _) =
        scripted_drawable_input_artboard_and_machine(Box::new(RecordingDrawableInputScript {
            label: "focused",
            methods: Vec::new(),
            handled: false,
            calls: Rc::new(RefCell::new(Vec::new())),
        }));
    // Discard the constructor fixture's unregistered initial focus event.
    machine.focus.take_owner_events();
    let listener = |listener_types| RuntimeStateMachineListener {
        name: None,
        target_local_id: 1,
        is_single: false,
        listener_types,
        event_local_indices: Vec::new(),
        view_model_path: None,
        view_model_input_types: Vec::new(),
        gamepad_input_types: Vec::new(),
        keyboard_input_types: Vec::new(),
        semantic_input_types: Vec::new(),
        hit_paths: Vec::new(),
        listener_actions: Vec::new(),
    };
    machine.listener_definitions = Arc::new(vec![
        listener(vec![RuntimeListenerType::Focus, RuntimeListenerType::Blur]),
        listener(vec![RuntimeListenerType::Focus, RuntimeListenerType::Blur]),
    ]);
    machine.focus_listener_groups = machine
        .listener_definitions
        .iter()
        .enumerate()
        .map(|(index, listener)| {
            RuntimeFocusListenerGroup::new(index, 2, listener).expect("focus listener group")
        })
        .collect();

    assert!(machine.focus.clear_focus());
    machine.capture_focus_callbacks();
    assert_eq!(
        machine.queued_focus_events,
        [
            RuntimeQueuedFocusEvent {
                listener_index: 0,
                is_focus: false,
            },
            RuntimeQueuedFocusEvent {
                listener_index: 1,
                is_focus: false,
            },
        ],
        "C++ queues one callback per registered group occurrence, in registration order"
    );
    machine.queued_focus_events.clear();

    assert!(machine.focus.set_focus_target(1));
    machine.capture_focus_callbacks();
    assert_eq!(
        machine.queued_focus_events,
        [
            RuntimeQueuedFocusEvent {
                listener_index: 0,
                is_focus: true,
            },
            RuntimeQueuedFocusEvent {
                listener_index: 1,
                is_focus: true,
            },
        ]
    );

    // Removing the occurrence-owned groups is Rust's exact registration
    // teardown boundary: later manager callbacks have no retained sink.
    machine.focus_listener_groups.clear();
    machine.queued_focus_events.clear();
    assert!(machine.focus.clear_focus());
    machine.capture_focus_callbacks();
    assert!(machine.queued_focus_events.is_empty());
    // Keep the artboard live through the teardown proof.
    artboard.update_components();
}

#[test]
fn focus_action_marks_machine_only_through_a_registered_focus_callback() {
    let calls = Rc::new(RefCell::new(Vec::new()));
    let (mut artboard, mut machine, _) =
        scripted_drawable_input_artboard_and_machine(Box::new(RecordingDrawableInputScript {
            label: "focused",
            methods: Vec::new(),
            handled: false,
            calls,
        }));
    let action = RuntimeScheduledListenerAction::FocusClear(RuntimeFocusActionClear::for_test(0));
    let invocation = ScriptListenerInvocation::Keyboard {
        key: 0,
        modifiers: 0,
        is_pressed: true,
        is_repeat: false,
    };

    // FocusManager still changes focus when no group is registered, but
    // C++ has no queueFocusEvent callback and therefore does not mark the
    // owning StateMachineInstance.
    machine.focus_listener_groups.clear();
    machine.focus.take_owner_events();
    machine.needs_advance = false;
    assert!(
        machine
            .perform_listener_actions(
                &mut artboard,
                std::slice::from_ref(&action),
                None,
                &invocation,
                &mut NoopScriptHost,
            )
            .expect("focus action")
    );
    assert!(!machine.needs_advance);
    assert!(machine.queued_focus_events.is_empty());

    assert!(machine.focus.set_focus_target(1));
    machine.focus.take_owner_events();
    machine.listener_definitions = Arc::new(vec![RuntimeStateMachineListener {
        name: None,
        target_local_id: 1,
        is_single: false,
        listener_types: vec![RuntimeListenerType::Focus, RuntimeListenerType::Blur],
        event_local_indices: Vec::new(),
        view_model_path: None,
        view_model_input_types: Vec::new(),
        gamepad_input_types: Vec::new(),
        keyboard_input_types: Vec::new(),
        semantic_input_types: Vec::new(),
        hit_paths: Vec::new(),
        listener_actions: Vec::new(),
    }]);
    machine.focus_listener_groups = vec![
        RuntimeFocusListenerGroup::new(0, 2, &machine.listener_definitions[0])
            .expect("focus group"),
    ];
    machine.needs_advance = false;
    assert!(
        machine
            .perform_listener_actions(
                &mut artboard,
                &[action],
                None,
                &invocation,
                &mut NoopScriptHost,
            )
            .expect("focus action")
    );
    assert!(machine.needs_advance);
    assert_eq!(
        machine.queued_focus_events,
        [RuntimeQueuedFocusEvent {
            listener_index: 0,
            is_focus: false,
        }]
    );
}

#[test]
fn completed_focus_callback_survives_a_later_terminal_action() {
    let calls = Rc::new(RefCell::new(Vec::new()));
    let (mut artboard, mut machine, _) =
        scripted_drawable_input_artboard_and_machine(Box::new(RecordingDrawableInputScript {
            label: "focused",
            methods: Vec::new(),
            handled: false,
            calls: Rc::new(RefCell::new(Vec::new())),
        }));
    machine.listener_definitions = Arc::new(vec![RuntimeStateMachineListener {
        name: None,
        target_local_id: 1,
        is_single: false,
        listener_types: vec![RuntimeListenerType::Focus, RuntimeListenerType::Blur],
        event_local_indices: Vec::new(),
        view_model_path: None,
        view_model_input_types: Vec::new(),
        gamepad_input_types: Vec::new(),
        keyboard_input_types: Vec::new(),
        semantic_input_types: Vec::new(),
        hit_paths: Vec::new(),
        listener_actions: Vec::new(),
    }]);
    machine.focus_listener_groups = vec![
        RuntimeFocusListenerGroup::new(0, 2, &machine.listener_definitions[0])
            .expect("focus group"),
    ];
    machine.focus.take_owner_events();
    machine.queued_focus_events.clear();
    let definition = ScriptListenerActionDefinition::new(777, 0, "terminal-after-focus".to_owned());
    machine.scripted_listener_action_definitions = vec![definition.clone()];
    machine
        .set_scripted_listener_action_instance(
            definition.action_global_id(),
            script(
                "terminal",
                true,
                false,
                ListenerFailure::Terminal("script.resource.host_commands"),
                &calls,
            ),
        )
        .expect("attach terminal scripted action");
    let actions = [
        RuntimeScheduledListenerAction::FocusClear(RuntimeFocusActionClear::for_test(0)),
        RuntimeScheduledListenerAction::scripted_for_test(0, Some(definition)),
        RuntimeScheduledListenerAction::FocusTarget(RuntimeFocusActionTarget::for_test(0, Some(1))),
    ];

    let error = machine
        .perform_listener_actions(
            &mut artboard,
            &actions,
            None,
            &ScriptListenerInvocation::None,
            &mut NoopScriptHost,
        )
        .expect_err("typed resource exhaustion remains terminal");

    assert_eq!(error.resource_code(), Some("script.resource.host_commands"));
    assert_eq!(
        machine.queued_focus_events,
        [RuntimeQueuedFocusEvent {
            listener_index: 0,
            is_focus: false,
        }],
        "the focus callback completed synchronously before the later action failed"
    );
    assert!(
        machine.focus.focused_listener_chain().is_empty(),
        "the action after the terminal fence must not run"
    );
    assert_eq!(calls.borrow().len(), 1);
}

#[test]
fn gamepad_broadcast_uses_authored_script_identity_once() {
    let calls = Rc::new(RefCell::new(Vec::new()));
    let (mut artboard, mut machine, global_id) =
        scripted_drawable_input_artboard_and_machine(Box::new(RecordingDrawableInputScript {
            label: "focused",
            methods: vec![ScriptMethod::GamepadEvent],
            // C++ gamepad methods do not return a handled boolean. The
            // method's existence makes dispatch handled even if Rust's
            // shared test seam supplies false.
            handled: false,
            calls: Rc::clone(&calls),
        }));
    assert_eq!(
        machine
            .gamepad_scripted_drawables
            .iter()
            .map(|scripted| scripted.global_id)
            .collect::<Vec<_>>(),
        [global_id]
    );
    let invocation = ScriptListenerInvocation::GamepadEvent {
        full_state: ScriptGamepadSnapshot {
            device_id: 7,
            button_mask: 2,
            button_values: vec![0.0, 0.75],
            axes: vec![-0.5],
            mapping: crate::ScriptGamepadMappingKind::Standard,
        },
        change: crate::ScriptGamepadInputChange::Button {
            index: 1,
            value: 0.75,
        },
        standard_button_intent: Some(1),
        standard_axis_intent: None,
    };

    assert!(machine.gamepad_dispatch(&mut artboard, invocation.clone()));
    assert_eq!(
        calls.borrow().as_slice(),
        [RecordedDrawableInputCall {
            label: "focused",
            invocation
        }]
    );

    assert!(!machine.gamepad_dispatch(
        &mut artboard,
        ScriptListenerInvocation::GamepadConnected {
            snapshot: ScriptGamepadSnapshot {
                device_id: 7,
                button_mask: 0,
                button_values: Vec::new(),
                axes: Vec::new(),
                mapping: crate::ScriptGamepadMappingKind::Standard,
            },
        },
    ));
    assert_eq!(
        calls.borrow().len(),
        1,
        "C++ broadcasts only the invocation methods the scripted drawable declared"
    );

    let event = ScriptListenerInvocation::GamepadEvent {
        full_state: ScriptGamepadSnapshot {
            device_id: 7,
            button_mask: 2,
            button_values: vec![0.0, 0.75],
            axes: vec![-0.5],
            mapping: crate::ScriptGamepadMappingKind::Standard,
        },
        change: crate::ScriptGamepadInputChange::Button {
            index: 1,
            value: 0.75,
        },
        standard_button_intent: Some(1),
        standard_axis_intent: None,
    };
    assert!(
        machine
            .broadcast_gamepad_to_scripted_drawables(
                &mut artboard,
                &event,
                Some((u64::MAX, global_id)),
            )
            .handled,
        "the same authored id in another artboard occurrence is a distinct C++ pointer"
    );
    let owner_identity = artboard.instance_identity();
    assert!(
        !machine
            .broadcast_gamepad_to_scripted_drawables(
                &mut artboard,
                &event,
                Some((owner_identity, global_id)),
            )
            .handled,
        "only the exact focused scripted-drawable occurrence is skipped"
    );
    assert_eq!(calls.borrow().len(), 2);
}

#[test]
fn scripted_gamepad_parent_never_falls_through_to_listener_actions() {
    let (mut artboard, mut machine, global_id) =
        scripted_drawable_input_artboard_and_machine(Box::new(RecordingDrawableInputScript {
            label: "no gamepad method",
            methods: Vec::new(),
            handled: false,
            calls: Rc::new(RefCell::new(Vec::new())),
        }));
    machine.listener_definitions = Arc::new(vec![RuntimeStateMachineListener {
        name: None,
        target_local_id: 1,
        is_single: false,
        listener_types: vec![RuntimeListenerType::Gamepad],
        event_local_indices: Vec::new(),
        view_model_path: None,
        view_model_input_types: Vec::new(),
        gamepad_input_types: Vec::new(),
        keyboard_input_types: Vec::new(),
        semantic_input_types: Vec::new(),
        hit_paths: Vec::new(),
        listener_actions: vec![RuntimeScheduledListenerAction::noop_for_test(0)],
    }]);
    machine.gamepad_listener_groups = vec![
        RuntimeGamepadListenerGroup::new(0, 2, &machine.listener_definitions[0])
            .expect("gamepad listener group"),
    ];
    machine.needs_advance = false;

    let (outcome, dispatched) = machine.gamepad_dispatch_at_focus_data(
        &mut artboard,
        2,
        &ScriptListenerInvocation::GamepadDisconnected { device_id: 7 },
    );

    assert!(!outcome.handled);
    assert!(!outcome.terminal_resource_failure);
    assert_eq!(dispatched, Some((artboard.instance_identity(), global_id)));
    assert!(
        !machine.needs_advance,
        "C++ returns the ScriptedDrawable result immediately; it never runs or marks the ordinary listener branch"
    );
}

#[test]
fn scripted_drawable_without_attached_script_still_owns_gamepad_dispatch() {
    let (mut artboard, mut machine, _) =
        scripted_drawable_subtype_input_artboard_and_machine_with_optional_script(
            "ScriptedDrawable",
            None,
            true,
        );
    machine.listener_definitions = Arc::new(vec![RuntimeStateMachineListener {
        name: None,
        target_local_id: 1,
        is_single: false,
        listener_types: vec![RuntimeListenerType::Gamepad],
        event_local_indices: Vec::new(),
        view_model_path: None,
        view_model_input_types: Vec::new(),
        gamepad_input_types: vec![
            super::listener_types::RuntimeListenerInputTypeGamepad::catch_all_for_test(1),
        ],
        keyboard_input_types: Vec::new(),
        semantic_input_types: Vec::new(),
        hit_paths: Vec::new(),
        listener_actions: vec![RuntimeScheduledListenerAction::noop_for_test(0)],
    }]);
    machine.gamepad_listener_groups = vec![
        RuntimeGamepadListenerGroup::new(0, 2, &machine.listener_definitions[0])
            .expect("gamepad listener group"),
    ];
    machine.needs_advance = false;

    let (outcome, dispatched) = machine.gamepad_dispatch_at_focus_data(
        &mut artboard,
        2,
        &ScriptListenerInvocation::GamepadDisconnected { device_id: 7 },
    );

    assert!(!outcome.handled);
    assert!(!outcome.terminal_resource_failure);
    assert_eq!(
        dispatched,
        Some((
            artboard.instance_identity(),
            artboard.component(1).unwrap().global_id
        ))
    );
    assert!(
        !machine.needs_advance,
        "C++ selects the ScriptedDrawable branch from the concrete parent type; a null VM returns false without running the ordinary listener"
    );
}

#[test]
fn gamepad_listener_dispatches_all_payloads_fifo_marks_advance_and_returns_false() {
    fn record(type_name: &str, properties: Vec<FixtureProperty>) -> FixtureRecord {
        FixtureRecord {
            type_key: nuxie_schema::definition_by_name(type_name)
                .unwrap_or_else(|| panic!("missing schema definition {type_name}"))
                .type_key
                .int,
            properties,
        }
    }
    fn uint(type_name: &str, name: &str, value: u64) -> FixtureProperty {
        FixtureProperty {
            key: crate::properties::property_key_for_name(type_name, name)
                .unwrap_or_else(|| panic!("missing {type_name}.{name}")),
            value: FixtureValue::Uint(value),
        }
    }

    let file = RuntimeFile::from_fixture_records(vec![
        record("Backboard", Vec::new()),
        record("Artboard", Vec::new()),
        record("Node", vec![uint("Node", "parentId", 0)]),
        record("FocusData", vec![uint("FocusData", "parentId", 1)]),
        record("StateMachine", Vec::new()),
        record("StateMachineBool", Vec::new()),
        record(
            "StateMachineListener",
            vec![uint("StateMachineListener", "targetId", 1)],
        ),
        record(
            "ListenerInputTypeGamepad",
            vec![uint(
                "ListenerInputTypeGamepad",
                "listenerTypeValue",
                RuntimeListenerType::Gamepad as u64,
            )],
        ),
        record(
            "ListenerBoolChange",
            vec![
                uint("ListenerBoolChange", "inputId", 0),
                uint("ListenerBoolChange", "value", 1),
            ],
        ),
        record(
            "ListenerBoolChange",
            vec![
                uint("ListenerBoolChange", "inputId", 0),
                uint("ListenerBoolChange", "value", 2),
            ],
        ),
    ])
    .expect("gamepad listener records");
    let graph = GraphFile::from_runtime_file(&file).expect("gamepad listener graph");
    let mut artboard = ArtboardInstance::from_graph_with_artboards(
        &file,
        graph.artboards.first().expect("gamepad listener artboard"),
        &graph.artboards,
    )
    .expect("gamepad listener instance");
    artboard.update_components();
    let mut machine = artboard
        .state_machine_instance(0)
        .expect("gamepad listener machine");
    assert!(machine.focus.set_focus_target(1));

    let invocations = [
        RuntimeGamepadListenerGroup::connected(ScriptGamepadSnapshot {
            device_id: 9,
            button_mask: 0,
            button_values: Vec::new(),
            axes: Vec::new(),
            mapping: crate::ScriptGamepadMappingKind::Unknown,
        }),
        ScriptListenerInvocation::GamepadEvent {
            full_state: ScriptGamepadSnapshot {
                device_id: 9,
                button_mask: 1,
                button_values: vec![1.0],
                axes: vec![0.25],
                mapping: crate::ScriptGamepadMappingKind::Standard,
            },
            change: crate::ScriptGamepadInputChange::Axis {
                index: 0,
                value: 0.25,
            },
            standard_button_intent: None,
            standard_axis_intent: Some(0),
        },
        RuntimeGamepadListenerGroup::disconnected(9),
    ];
    for invocation in invocations {
        machine.needs_advance = false;
        assert!(
            !machine.gamepad_dispatch(&mut artboard, invocation),
            "the authored listener branch never handles propagation in C++"
        );
        assert!(machine.needs_advance());
        assert_eq!(
            machine
                .input(0)
                .and_then(StateMachineInputInstance::bool_value),
            Some(false),
            "set-true then toggle executes both authored actions in FIFO order"
        );
    }
}

#[test]
fn fl_c5_event_host_drain_leaves_the_core_queue_for_apply_events() {
    let (artboard, mut machine) = scripted_listener_artboard_and_machine();
    let event = StateMachineReportedEvent {
        event_local_index: 7,
        event_core_type: 128,
        name: Some("next-frame".to_owned()),
        url: None,
        target: None,
        properties: Vec::new(),
        string_properties: Vec::new(),
        seconds_delay: 0.0,
        context: None,
    };
    machine.reported_events.push(event.clone());

    let drained = machine.take_reported_events(&artboard);

    assert_eq!(drained.len(), 1);
    assert_eq!(drained[0].event_local_index(), event.event_local_index());
    assert!(machine.take_reported_events(&artboard).is_empty());
    assert_eq!(machine.reported_event_count(), 1);
    assert_eq!(machine.next_unapplied_reported_event_index(), 0);
}

#[test]
fn fl_c5_event_apply_batches_chaining_and_exact_100_cap() {
    fn record(type_name: &str, properties: Vec<FixtureProperty>) -> FixtureRecord {
        FixtureRecord {
            type_key: nuxie_schema::definition_by_name(type_name)
                .unwrap_or_else(|| panic!("missing schema definition {type_name}"))
                .type_key
                .int,
            properties,
        }
    }
    fn uint(type_name: &str, property_name: &str, value: u64) -> FixtureProperty {
        FixtureProperty {
            key: crate::properties::property_key_for_name(type_name, property_name)
                .unwrap_or_else(|| panic!("missing {type_name}.{property_name}")),
            value: FixtureValue::Uint(value),
        }
    }
    fn event_listener(
        event_local_id: usize,
        fire_event_local_id: Option<usize>,
    ) -> RuntimeStateMachineListener {
        RuntimeStateMachineListener {
            name: None,
            target_local_id: 0,
            is_single: false,
            listener_types: vec![RuntimeListenerType::Event],
            event_local_indices: vec![event_local_id],
            view_model_path: None,
            view_model_input_types: Vec::new(),
            gamepad_input_types: Vec::new(),
            keyboard_input_types: Vec::new(),
            semantic_input_types: Vec::new(),
            hit_paths: Vec::new(),
            listener_actions: fire_event_local_id
                .map(|event_local_id| {
                    vec![RuntimeScheduledListenerAction::FireEvent(
                        super::listener_fire_event::RuntimeListenerFireEvent::for_test(
                            0,
                            Some(event_local_id),
                        ),
                    )]
                })
                .unwrap_or_default(),
        }
    }

    let file = RuntimeFile::from_fixture_records(vec![
        record("Backboard", Vec::new()),
        record("Artboard", Vec::new()),
        record("Event", vec![uint("Event", "parentId", 0)]),
        record("Event", vec![uint("Event", "parentId", 0)]),
        record("Event", vec![uint("Event", "parentId", 0)]),
        record("StateMachine", Vec::new()),
    ])
    .expect("chained event records import");
    let graph = GraphFile::from_runtime_file(&file).expect("chained event graph builds");
    let mut artboard = ArtboardInstance::from_graph_with_artboards(
        &file,
        graph.artboards.first().expect("chained event artboard"),
        &graph.artboards,
    )
    .expect("chained event artboard instantiates");
    let mut machine = artboard
        .state_machine_instance(0)
        .expect("chained event state machine");
    machine.listener_definitions = Arc::new(vec![
        event_listener(1, Some(2)),
        event_listener(2, Some(3)),
        event_listener(3, None),
    ]);
    machine.reported_events.push(StateMachineReportedEvent {
        event_local_index: 1,
        event_core_type: 128,
        name: Some("first".to_owned()),
        url: None,
        target: None,
        properties: Vec::new(),
        string_properties: Vec::new(),
        seconds_delay: 0.0,
        context: None,
    });

    assert!(
        artboard.advance_state_machine_instance(&mut machine, 0.25),
        "events first reported inside applyEvents remain host visible after listener delivery"
    );
    assert_eq!(
        machine
            .reporting_events
            .iter()
            .map(StateMachineReportedEvent::event_local_index)
            .collect::<Vec<_>>(),
        [3],
        "E1 -> E2 -> E3 must reach the final listener in this one applyEvents call"
    );
    assert_eq!(
        machine
            .events_applied_during_loop
            .iter()
            .map(StateMachineReportedEvent::event_local_index)
            .collect::<Vec<_>>(),
        [2, 3]
    );
    assert_eq!(machine.reported_event_count(), 2);
    assert_eq!(machine.next_unapplied_reported_event_index(), 0);
    assert_eq!(
        machine
            .take_reported_events(&artboard)
            .iter()
            .map(StateMachineReportedEvent::event_local_index)
            .collect::<Vec<_>>(),
        [2, 3]
    );
    assert!(machine.take_reported_events(&artboard).is_empty());

    let mut finite_records = vec![
        record("Backboard", Vec::new()),
        record("Artboard", Vec::new()),
    ];
    finite_records.extend((0..100).map(|_| record("Event", vec![uint("Event", "parentId", 0)])));
    finite_records.push(record("StateMachine", Vec::new()));
    let finite_file =
        RuntimeFile::from_fixture_records(finite_records).expect("finite chain imports");
    let finite_graph =
        GraphFile::from_runtime_file(&finite_file).expect("finite chain graph builds");
    let mut finite_artboard = ArtboardInstance::from_graph_with_artboards(
        &finite_file,
        finite_graph
            .artboards
            .first()
            .expect("finite chain artboard"),
        &finite_graph.artboards,
    )
    .expect("finite chain artboard instantiates");
    let mut finite_machine = finite_artboard
        .state_machine_instance(0)
        .expect("finite chain state machine");
    finite_machine.listener_definitions = Arc::new(
        (1..=100)
            .map(|event_local_id| {
                event_listener(
                    event_local_id,
                    (event_local_id < 100).then_some(event_local_id + 1),
                )
            })
            .collect(),
    );
    finite_machine
        .reported_events
        .push(fl_c5_test_reported_event(1));
    finite_machine.apply_local_event_listeners(&mut finite_artboard, 0, None);
    assert_eq!(
        finite_machine.next_unapplied_reported_event_index(),
        100,
        "a finite chain consumes its hundredth batch"
    );
    assert_eq!(
        finite_machine.reported_event_count(),
        99,
        "events first reported in batches 2 through 100 remain host visible"
    );
    assert_eq!(
        finite_machine
            .reporting_events
            .first()
            .map(StateMachineReportedEvent::event_local_index),
        Some(100)
    );

    let vm_listener_definition = RuntimeStateMachineListener {
        name: None,
        target_local_id: 0,
        is_single: false,
        listener_types: vec![RuntimeListenerType::ViewModel],
        event_local_indices: Vec::new(),
        view_model_path: Some(RuntimeListenerViewModelPath::Absolute {
            view_model_index: 0,
            property_path: vec![0],
        }),
        view_model_input_types: Vec::new(),
        gamepad_input_types: Vec::new(),
        keyboard_input_types: Vec::new(),
        semantic_input_types: Vec::new(),
        hit_paths: Vec::new(),
        listener_actions: vec![RuntimeScheduledListenerAction::FireEvent(
            super::listener_fire_event::RuntimeListenerFireEvent::for_test(0, Some(2)),
        )],
    };
    let mixed_definitions = Arc::new(vec![
        event_listener(1, None),
        vm_listener_definition,
        event_listener(2, None),
    ]);
    finite_machine.listener_definitions = Arc::clone(&mixed_definitions);
    finite_machine.view_model_listeners = vec![
        RuntimeViewModelListenerInstance::new(Arc::clone(&mixed_definitions), 1)
            .expect("mixed ViewModel listener"),
    ];
    finite_machine.reported_events.clear();
    finite_machine.reported_event_listener_index = 0;
    finite_machine
        .reported_events
        .push(fl_c5_test_reported_event(1));
    finite_machine
        .reported_listener_view_models
        .report_data_bind(0);
    finite_machine.apply_local_event_listeners(&mut finite_artboard, 0, None);
    assert_eq!(
        finite_machine
            .reporting_events
            .iter()
            .map(StateMachineReportedEvent::event_local_index)
            .collect::<Vec<_>>(),
        [2],
        "the ViewModel callback fires an event into the next same-call batch after the event phase"
    );
    assert_eq!(
        finite_machine.reported_event_count(),
        1,
        "the event fired by the ViewModel listener remains host visible exactly once"
    );

    let calls = Rc::new(RefCell::new(Vec::new()));
    let mut event_to_vm = scripted_test_listener(
        &mut finite_machine,
        985,
        "event-to-vm",
        ListenerFailure::None,
        vec![RuntimeListenerType::Event],
        &calls,
    );
    event_to_vm.target_local_id = 1;
    event_to_vm.event_local_indices = vec![1];
    let event_to_vm_queue = finite_machine.reported_listener_view_models.clone();
    let event_to_vm_script =
        RuntimeScriptInstanceHandle::new(Box::new(ReportingViewModelListenerScript {
            label: "event-to-vm",
            queue: event_to_vm_queue,
            listener_index: 0,
            calls: Rc::clone(&calls),
        }));
    finite_machine
        .scripted_instances_by_global
        .insert(985, event_to_vm_script.clone());
    finite_machine
        .scripted_listener_action_instances
        .insert(985, event_to_vm_script);
    finite_machine.scripted_object_initialization_complete = true;
    let mut event_after_vm = scripted_test_listener(
        &mut finite_machine,
        986,
        "after-vm",
        ListenerFailure::None,
        vec![RuntimeListenerType::Event],
        &calls,
    );
    event_after_vm.target_local_id = 2;
    event_after_vm.event_local_indices = vec![2];
    let event_to_vm_definitions = Arc::new(vec![
        event_to_vm,
        RuntimeStateMachineListener {
            name: None,
            target_local_id: 0,
            is_single: false,
            listener_types: vec![RuntimeListenerType::ViewModel],
            event_local_indices: Vec::new(),
            view_model_path: Some(RuntimeListenerViewModelPath::Absolute {
                view_model_index: 0,
                property_path: vec![0],
            }),
            view_model_input_types: Vec::new(),
            gamepad_input_types: Vec::new(),
            keyboard_input_types: Vec::new(),
            semantic_input_types: Vec::new(),
            hit_paths: Vec::new(),
            listener_actions: vec![RuntimeScheduledListenerAction::FireEvent(
                super::listener_fire_event::RuntimeListenerFireEvent::for_test(0, Some(2)),
            )],
        },
        event_after_vm,
    ]);
    finite_machine.listener_definitions = Arc::clone(&event_to_vm_definitions);
    finite_machine.view_model_listeners = vec![
        RuntimeViewModelListenerInstance::new(Arc::clone(&event_to_vm_definitions), 1)
            .expect("event-generated ViewModel listener"),
    ];
    finite_machine.reported_events.clear();
    finite_machine.reported_event_listener_index = 0;
    finite_machine
        .reported_events
        .push(fl_c5_test_reported_event(1));
    finite_machine.apply_local_event_listeners(&mut finite_artboard, 0, None);
    assert_eq!(
        calls
            .borrow()
            .iter()
            .map(|call| call.label)
            .collect::<Vec<_>>(),
        ["event-to-vm", "after-vm"],
        "event-generated ViewModel work runs in the next batch, then its generated event runs in the following batch"
    );

    machine.listener_definitions = Arc::new(vec![event_listener(1, Some(1))]);
    machine.reported_events.push(StateMachineReportedEvent {
        event_local_index: 1,
        event_core_type: 128,
        name: Some("loop".to_owned()),
        url: None,
        target: None,
        properties: Vec::new(),
        string_properties: Vec::new(),
        seconds_delay: 0.0,
        context: None,
    });
    let start = machine.next_unapplied_reported_event_index();
    machine.apply_local_event_listeners(&mut artboard, start, None);
    assert_eq!(
        machine.next_unapplied_reported_event_index(),
        100,
        "exactly 100 finite callback batches must be consumed"
    );
    assert_eq!(
        machine.reported_event_count(),
        100,
        "batches 2 through 100 remain host visible and the event generated by batch 100 is pending as batch 101"
    );
    assert_eq!(
        machine
            .reported_event_snapshot(0)
            .map(StateMachineReportedEvent::event_local_index),
        Some(1)
    );
}

#[test]
fn fl_c5_event_listener_fire_reports_live_payload_before_advance() {
    fn record(type_name: &str, properties: Vec<FixtureProperty>) -> FixtureRecord {
        FixtureRecord {
            type_key: nuxie_schema::definition_by_name(type_name)
                .unwrap_or_else(|| panic!("missing schema definition {type_name}"))
                .type_key
                .int,
            properties,
        }
    }
    fn uint(type_name: &str, property_name: &str, value: u64) -> FixtureProperty {
        FixtureProperty {
            key: crate::properties::property_key_for_name(type_name, property_name)
                .unwrap_or_else(|| panic!("missing {type_name}.{property_name}")),
            value: FixtureValue::Uint(value),
        }
    }
    fn string(type_name: &str, property_name: &str, value: &str) -> FixtureProperty {
        FixtureProperty {
            key: crate::properties::property_key_for_name(type_name, property_name)
                .unwrap_or_else(|| panic!("missing {type_name}.{property_name}")),
            value: FixtureValue::String(value.to_owned()),
        }
    }

    let file = RuntimeFile::from_fixture_records(vec![
        record("Backboard", Vec::new()),
        record("Artboard", Vec::new()),
        record(
            "Event",
            vec![
                uint("Event", "parentId", 0),
                string("Event", "name", "imported"),
            ],
        ),
        record(
            "CustomPropertyString",
            vec![
                uint("CustomPropertyString", "parentId", 1),
                string("CustomPropertyString", "name", "payload"),
                string("CustomPropertyString", "propertyValue", "old"),
            ],
        ),
        record("StateMachine", Vec::new()),
    ])
    .expect("event listener records import");
    let graph = GraphFile::from_runtime_file(&file).expect("event listener graph builds");
    let mut artboard = ArtboardInstance::from_graph_with_artboards(
        &file,
        graph.artboards.first().expect("event listener artboard"),
        &graph.artboards,
    )
    .expect("event listener artboard instantiates");
    let mut machine = artboard
        .state_machine_instance(0)
        .expect("event listener state machine");

    let event_name =
        crate::properties::property_key_for_name("Event", "name").expect("Event.name property");
    let property_value =
        crate::properties::property_key_for_name("CustomPropertyString", "propertyValue")
            .expect("CustomPropertyString.propertyValue property");
    assert!(artboard.set_string_property(1, event_name, b"live".to_vec()));
    assert!(artboard.set_string_property(2, property_value, b"new".to_vec()));

    let actions = [RuntimeScheduledListenerAction::FireEvent(
        super::listener_fire_event::RuntimeListenerFireEvent::for_test(
            StateMachineFireOccurrence::AtStart.value(),
            Some(1),
        ),
    )];
    let facade_hit_context = StateMachineEventContext {
        path: Vec::new(),
        occurrence: Vec::new(),
    };
    assert!(
        machine
            .perform_listener_actions_with_event_context(
                &mut artboard,
                &actions,
                None,
                &ScriptListenerInvocation::None,
                &mut NoopScriptHost,
                Some(&facade_hit_context),
            )
            .expect("fire live event")
    );
    // C++ EventReport retains Event*, so edits made after reportEvent and
    // before host observation are visible too.
    assert!(artboard.set_string_property(1, event_name, b"latest".to_vec()));
    assert!(artboard.set_string_property(2, property_value, b"after-fire".to_vec()));

    let mut snapshot = machine.clone();
    let drained = machine.take_reported_events(&artboard);
    assert_eq!(drained.len(), 1);
    assert_eq!(drained[0].name(), Some("latest"));
    assert_eq!(
        drained[0].context(),
        Some(&facade_hit_context),
        "the Rust facade's rendered occurrence metadata is orthogonal to the ignored C++ ListenerInvocation payload"
    );
    assert_eq!(
        drained[0]
            .string_properties()
            .iter()
            .map(|property| (property.name(), property.value()))
            .collect::<Vec<_>>(),
        [("payload", "after-fire")]
    );
    assert_eq!(machine.reported_event_count(), 1);
    assert_eq!(machine.next_unapplied_reported_event_index(), 0);
    assert_eq!(
        snapshot.take_reported_events(&artboard).len(),
        1,
        "Rust's explicit Clone snapshot retains pending values in non-aliased storage"
    );
    assert!(
        snapshot.take_reported_events(&artboard).is_empty(),
        "draining the snapshot does not mutate the source cursor"
    );
    assert!(
        machine.take_reported_events(&artboard).is_empty(),
        "draining the source does not replay after the snapshot drain"
    );
}

fn fl_c5_test_reported_event(local_index: usize) -> StateMachineReportedEvent {
    StateMachineReportedEvent {
        event_local_index: local_index,
        event_core_type: 128,
        name: Some(format!("event-{local_index}")),
        url: None,
        target: None,
        properties: Vec::new(),
        string_properties: Vec::new(),
        seconds_delay: 0.0,
        context: None,
    }
}

fn fl_c5_test_audio_event(local_index: usize) -> (StateMachineReportedEvent, u32) {
    let audio_file = RuntimeFile::from_fixture_records(vec![
        FixtureRecord {
            type_key: nuxie_schema::definition_by_name("Backboard")
                .expect("Backboard schema definition")
                .type_key
                .int,
            properties: Vec::new(),
        },
        FixtureRecord {
            type_key: nuxie_schema::definition_by_name("Artboard")
                .expect("Artboard schema definition")
                .type_key
                .int,
            properties: Vec::new(),
        },
        FixtureRecord {
            type_key: nuxie_schema::definition_by_name("AudioEvent")
                .expect("AudioEvent schema definition")
                .type_key
                .int,
            properties: vec![FixtureProperty {
                key: crate::properties::property_key_for_name("AudioEvent", "parentId")
                    .expect("AudioEvent.parentId"),
                value: FixtureValue::Uint(0),
            }],
        },
    ])
    .expect("import live AudioEvent fixture");
    let audio_object = audio_file
        .objects
        .iter()
        .flatten()
        .find(|object| object.type_name == "AudioEvent")
        .expect("live AudioEvent-typed object");
    (
        StateMachineReportedEvent::from_runtime_event(local_index, audio_object),
        u32::from(audio_object.type_key),
    )
}

#[test]
fn fl_c5_event_mid_callback_visibility_excludes_the_reporting_snapshot() {
    let (_artboard, mut machine) = scripted_listener_artboard_and_machine();
    machine.reported_events.push(fl_c5_test_reported_event(7));
    let mut reporting = std::mem::take(&mut machine.reporting_events);
    reporting.clear();
    reporting.extend_from_slice(&machine.reported_events);
    machine.reported_event_listener_index = machine.reported_events.len();

    assert_eq!(machine.reported_event_count(), 0);
    assert!(machine.reported_event_snapshot(0).is_none());
    machine.reported_events.push(fl_c5_test_reported_event(8));
    assert_eq!(machine.reported_event_count(), 1);
    assert_eq!(
        machine
            .reported_event_snapshot(0)
            .map(StateMachineReportedEvent::event_local_index),
        Some(8),
        "callback inspection sees only work appended for a later batch"
    );
    assert_eq!(reporting[0].event_local_index(), 7);
}

#[test]
fn view_model_listener_binding_reports_a_trigger_fired_before_relink() {
    let definitions = Arc::new(vec![RuntimeStateMachineListener {
        name: None,
        target_local_id: 0,
        is_single: false,
        listener_types: vec![RuntimeListenerType::ViewModel],
        event_local_indices: Vec::new(),
        view_model_path: Some(RuntimeListenerViewModelPath::Absolute {
            view_model_index: 0,
            property_path: vec![0],
        }),
        view_model_input_types: Vec::new(),
        gamepad_input_types: Vec::new(),
        keyboard_input_types: Vec::new(),
        semantic_input_types: Vec::new(),
        hit_paths: Vec::new(),
        listener_actions: Vec::new(),
    }]);
    let mut listener =
        RuntimeViewModelListenerInstance::new(definitions, 0).expect("ViewModel listener instance");
    let queue = RuntimeCellNotificationQueue::default();
    let trigger = RuntimeViewModelCell::new(RuntimeViewModelCellValue::Trigger(1));
    relink_view_model_listener_cell(&mut listener.property_bindings[0], Some(trigger), &queue, 0);
    assert!(queue.is_empty(), "relink alone does not synthesize dirt");

    listener.report_pending_trigger_bindings(&queue, 0);
    assert_eq!(queue.len(), 1);
}

#[test]
fn upstream_view_model_listener_fixture_keeps_loop_fired_event_host_visible_once() {
    let file = read_runtime_file(include_bytes!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../fixtures/sync/vm_listener_fire_event.riv"
    )))
    .expect("upstream ViewModel-listener fixture imports");
    let graphs = GraphFile::from_runtime_file(&file).expect("fixture graph builds");
    let graph = graphs.artboards.first().expect("default artboard graph");
    let mut artboard = ArtboardInstance::from_graph_with_artboards(&file, graph, &graphs.artboards)
        .expect("default artboard instantiates");
    let mut machine = artboard
        .state_machine_instance(0)
        .expect("fixture state machine");
    let mut context = artboard
        .imported_view_model_instance_context(0, 0)
        .expect("fixture ViewModel instance");
    assert!(machine.bind_imported_view_model_context(&file, &context));
    artboard.advance_state_machine_instance(&mut machine, 0.0);
    assert_eq!(machine.reported_event_count(), 0);

    assert!(context.set_trigger_by_property_name(&file, "go", 1));
    artboard.advance_state_machine_instance(&mut machine, 0.016);
    assert_eq!(machine.reported_event_count(), 1);
    assert_eq!(
        machine
            .reported_event(&artboard, 0)
            .and_then(|event| event.name()),
        Some("ding")
    );

    artboard.advance_state_machine_instance(&mut machine, 0.016);
    assert_eq!(machine.reported_event_count(), 0);
}

#[test]
fn fl_c5_event_trigger_zero_suppression_and_duplicate_listener_fifo() {
    let cell = RuntimeViewModelCell::new(RuntimeViewModelCellValue::Trigger(0));
    let queue = RuntimeCellNotificationQueue::default();
    let first = RuntimeCellDirtSink::reporting_listener(&queue, 3);
    let duplicate = RuntimeCellDirtSink::reporting_listener(&queue, 3);
    cell.add_dependent(&first);
    cell.add_dependent(&duplicate);

    assert!(cell.fire_trigger());
    cell.advanced();
    let mut reporting = Vec::new();
    queue.swap_into(&mut reporting);
    assert_eq!(
        reporting,
        [3, 3],
        "one genuine mutation preserves duplicate dependent registrations"
    );
    assert_eq!(cell.value(), RuntimeViewModelCellValue::Trigger(0));

    let signed_zero = RuntimeViewModelCell::new(RuntimeViewModelCellValue::Number(-0.0));
    assert_eq!(
        signed_zero.value(),
        RuntimeViewModelCellValue::Number(-0.0),
        "signed zero remains ordinary number payload data"
    );
    let signed_zero_trigger = RuntimeViewModelCell::new(RuntimeViewModelCellValue::Trigger(1));
    let signed_zero_queue = RuntimeCellNotificationQueue::default();
    let signed_zero_sink = RuntimeCellDirtSink::reporting_listener(&signed_zero_queue, 4);
    signed_zero_trigger.add_dependent(&signed_zero_sink);
    assert!(signed_zero_trigger.set_value(RuntimeViewModelCellValue::Trigger((-0.0_f32) as u64)));
    let mut signed_zero_reports = Vec::new();
    signed_zero_queue.swap_into(&mut signed_zero_reports);
    assert!(
        signed_zero_reports.is_empty(),
        "a trigger reset expressed through signed zero is the same suppressed zero counter"
    );
}

#[test]
fn fl_c5_event_listener_major_event_minor_single_and_multi_order() {
    let (mut artboard, mut machine) = scripted_listener_artboard_and_machine();
    let calls = Rc::new(RefCell::new(Vec::new()));
    let mut single = scripted_test_listener(
        &mut machine,
        980,
        "single",
        ListenerFailure::None,
        vec![RuntimeListenerType::Event],
        &calls,
    );
    single.target_local_id = 0;
    single.is_single = true;
    single.event_local_indices = vec![7];
    let mut multi = scripted_test_listener(
        &mut machine,
        981,
        "multi",
        ListenerFailure::None,
        vec![RuntimeListenerType::Event],
        &calls,
    );
    multi.target_local_id = 0;
    multi.event_local_indices = vec![7];
    machine.listener_definitions = Arc::new(vec![single, multi]);

    let events = [fl_c5_test_reported_event(7), fl_c5_test_reported_event(7)];
    machine.notify_events(&mut artboard, None, &events);
    assert_eq!(
        calls
            .borrow()
            .iter()
            .map(|call| call.label)
            .collect::<Vec<_>>(),
        ["single", "multi", "multi"],
        "listeners are outermost; single breaks at the first [A,A] match while multi scans both"
    );
}

#[test]
fn fl_c5_event_bubbling_precedes_the_recorded_audio_seam_through_two_ancestors() {
    let (mut leaf_artboard, mut leaf) = scripted_listener_artboard_and_machine();
    let (mut parent_artboard, mut parent) = scripted_listener_artboard_and_machine();
    let (mut root_artboard, mut root) = scripted_listener_artboard_and_machine();
    let total_order = Rc::new(RefCell::new(Vec::new()));
    leaf.event_total_order_trace = Some(("leaf-local", "leaf-audio", Rc::clone(&total_order)));
    parent.event_total_order_trace =
        Some(("parent-local", "parent-audio", Rc::clone(&total_order)));
    root.event_total_order_trace = Some(("root-local", "root-audio", Rc::clone(&total_order)));
    leaf.attach_event_bubble_owner();
    parent.attach_event_bubble_owner();
    let ordinary_event = fl_c5_test_reported_event(6);
    let (audio_event, audio_event_core_type) = fl_c5_test_audio_event(7);
    let event = [ordinary_event, audio_event];
    let calls = Rc::new(RefCell::new(Vec::new()));
    let mut mismatch = scripted_test_listener(
        &mut parent,
        982,
        "mismatch",
        ListenerFailure::None,
        vec![RuntimeListenerType::Event],
        &calls,
    );
    mismatch.target_local_id = 99;
    mismatch.event_local_indices = vec![7];
    let mut parent_listener = scripted_test_listener(
        &mut parent,
        983,
        "parent",
        ListenerFailure::None,
        vec![RuntimeListenerType::Event],
        &calls,
    );
    parent_listener.target_local_id = 7;
    parent_listener.event_local_indices = vec![7];
    parent.listener_definitions = Arc::new(vec![mismatch, parent_listener]);
    parent
        .nested_event_registrations
        .push(RuntimeNestedEventRegistration {
            source_local_id: 7,
            notifier_local_id: 70,
            kind: RuntimeNestedEventNotifierKind::StateMachine,
        });
    let mut root_listener = scripted_test_listener(
        &mut root,
        984,
        "root",
        ListenerFailure::None,
        vec![RuntimeListenerType::Event],
        &calls,
    );
    root_listener.target_local_id = 8;
    root_listener.event_local_indices = vec![7];
    root.listener_definitions = Arc::new(vec![root_listener]);
    root.nested_event_registrations
        .push(RuntimeNestedEventRegistration {
            source_local_id: 8,
            notifier_local_id: 80,
            kind: RuntimeNestedEventNotifierKind::StateMachine,
        });

    root_artboard.set_frame_origin(false);
    let _ = root_artboard.advance_state_machine_instance(&mut root, 0.0);
    leaf.notify_events(&mut leaf_artboard, None, &event);
    let parent_events = leaf.take_bubbled_event_reports();
    assert_eq!(
        parent_events
            .iter()
            .map(StateMachineReportedEvent::event_local_index)
            .collect::<Vec<_>>(),
        [6, 7],
        "ordinary and audio reports both bubble in authored order"
    );
    parent.notify_events(&mut parent_artboard, Some(7), &parent_events);
    let root_events = parent.take_bubbled_event_reports();
    assert_eq!(
        root_events
            .iter()
            .map(StateMachineReportedEvent::event_local_index)
            .collect::<Vec<_>>(),
        [6, 7]
    );
    root.notify_events(&mut root_artboard, Some(8), &root_events);
    parent.flush_deferred_owner_audio_events();
    leaf.flush_deferred_owner_audio_events();
    assert_eq!(
        *total_order.borrow(),
        [
            "leaf-local",
            "parent-local",
            "root-local",
            "root-audio",
            "parent-audio",
            "leaf-audio",
        ],
        "nested bubbling is synchronous depth-first and audio tails unwind root-first"
    );
    assert!(
        root.take_bubbled_event_reports().is_empty(),
        "root draw state and a post-update probe do not invent an outgoing parent edge"
    );
    assert_eq!(
        calls
            .borrow()
            .iter()
            .map(|call| call.label)
            .collect::<Vec<_>>(),
        ["parent", "root"],
        "each owner dispatches the realistic nested source once; the mismatched target stays inert"
    );
    for machine in [&leaf, &parent] {
        assert_eq!(
            machine.event_dispatch_phase_trace,
            ["local-dispatch", "bubble-to-owner", "recorded-audio-seam"]
        );
        assert_eq!(
            machine.audio_event_seam_receipt(),
            (1, Some((7, audio_event_core_type))),
            "only the imported AudioEvent occurrence reaches the production handoff"
        );
    }
    assert_eq!(
        root.event_dispatch_phase_trace,
        ["local-dispatch", "recorded-audio-seam"]
    );
    assert_eq!(
        root.audio_event_seam_receipt(),
        (1, Some((7, audio_event_core_type)))
    );

    root.event_dispatch_phase_trace.clear();
    assert!(
        !root.notify_events(&mut root_artboard, Some(usize::MAX), &event),
        "an unregistered nested source must not dispatch or bubble"
    );
    assert!(root.event_dispatch_phase_trace.is_empty());

    leaf.notify_events(&mut leaf_artboard, None, &event);
    assert_eq!(leaf.reported_event_count(), 2);
    assert!(leaf.reported_event(&leaf_artboard, 1).is_some());
    assert_eq!(leaf.reported_event_count(), 0);
    leaf.notify_events(&mut leaf_artboard, None, &event);
    assert_eq!(
        leaf.bubbled_event_reports.len(),
        2,
        "the next bubble batch reclaims the production cursor's consumed prefix"
    );
}

#[test]
fn fl_c5_event_bubbling_cross_instance_total_order_through_one_ancestor() {
    let (mut leaf_artboard, mut leaf) = scripted_listener_artboard_and_machine();
    let (mut parent_artboard, mut parent) = scripted_listener_artboard_and_machine();
    let total_order = Rc::new(RefCell::new(Vec::new()));
    leaf.event_total_order_trace = Some(("leaf-local", "leaf-audio", Rc::clone(&total_order)));
    parent.event_total_order_trace =
        Some(("parent-local", "parent-audio", Rc::clone(&total_order)));
    leaf.attach_event_bubble_owner();
    parent
        .nested_event_registrations
        .push(RuntimeNestedEventRegistration {
            source_local_id: 7,
            notifier_local_id: 70,
            kind: RuntimeNestedEventNotifierKind::StateMachine,
        });
    let (audio_event, _) = fl_c5_test_audio_event(7);

    leaf.notify_events(&mut leaf_artboard, None, &[audio_event]);
    let events = leaf.take_bubbled_event_reports();
    parent.notify_events(&mut parent_artboard, Some(7), &events);
    leaf.flush_deferred_owner_audio_events();

    assert_eq!(
        *total_order.borrow(),
        ["leaf-local", "parent-local", "parent-audio", "leaf-audio"],
        "the two-level owner seam uses the same depth-first unwind policy"
    );
}

#[test]
fn atomic_nested_event_settlement_propagates_an_ordinary_callback_failure() {
    let (mut artboard, mut machine) = scripted_listener_artboard_and_machine();
    let calls = Rc::new(RefCell::new(Vec::new()));
    let mut listener = scripted_test_listener(
        &mut machine,
        986,
        "nested ordinary",
        ListenerFailure::Ordinary,
        vec![RuntimeListenerType::Event],
        &calls,
    );
    listener.target_local_id = 7;
    listener.event_local_indices = vec![7];
    machine.listener_definitions = Arc::new(vec![listener]);
    machine
        .nested_event_registrations
        .push(RuntimeNestedEventRegistration {
            source_local_id: 7,
            notifier_local_id: 70,
            kind: RuntimeNestedEventNotifierKind::StateMachine,
        });
    let event = fl_c5_test_reported_event(7);

    let error = StateMachineInstance::advance_artboard_frame_components_with_script_host(
        &mut artboard,
        std::slice::from_mut(&mut machine),
        0.0,
        None,
        &mut AtomicScriptHost,
        |artboard, _elapsed_seconds, nested_event_dispatch| {
            assert!(!nested_event_dispatch(artboard, 7, &[event.clone()]));
            Ok(false)
        },
    )
    .expect_err("atomic nested dispatch must not consume an ordinary callback error");

    assert!(error.to_string().contains("nested ordinary failed"));
    assert_eq!(
        calls
            .borrow()
            .iter()
            .map(|call| call.label)
            .collect::<Vec<_>>(),
        ["nested ordinary"]
    );
}

#[test]
fn fl_c5_failing_reporting_owner_completes_deep_bubble_and_audio_before_error_propagation() {
    let (mut child_artboard, mut parent) = scripted_listener_artboard_and_machine();
    let (mut root_artboard, mut root) = scripted_listener_artboard_and_machine();
    let total_order = Rc::new(RefCell::new(Vec::new()));
    parent.event_total_order_trace =
        Some(("parent-local", "parent-audio", Rc::clone(&total_order)));
    root.event_total_order_trace = Some(("root-local", "root-audio", Rc::clone(&total_order)));
    parent.attach_event_bubble_owner();
    root.nested_event_registrations
        .push(RuntimeNestedEventRegistration {
            source_local_id: 8,
            notifier_local_id: 80,
            kind: RuntimeNestedEventNotifierKind::StateMachine,
        });
    parent
        .nested_event_registrations
        .push(RuntimeNestedEventRegistration {
            source_local_id: 7,
            notifier_local_id: 70,
            kind: RuntimeNestedEventNotifierKind::StateMachine,
        });

    let calls = Rc::new(RefCell::new(Vec::new()));
    let mut failing_listener = scripted_test_listener(
        &mut parent,
        985,
        "reporting-owner",
        ListenerFailure::Terminal("script.resource.reporting_owner"),
        vec![RuntimeListenerType::Event],
        &calls,
    );
    failing_listener.target_local_id = 7;
    failing_listener.event_local_indices = vec![7];
    parent.listener_definitions = Arc::new(vec![failing_listener]);
    let (audio_event, audio_event_core_type) = fl_c5_test_audio_event(7);
    let notifier_local = 80;
    let (_, fallback_machine) = scripted_listener_artboard_and_machine();
    let mut animations = vec![
        crate::artboard::RuntimeNestedAnimationInstance::StateMachine(
            RuntimeNestedStateMachineInstance::new(notifier_local, fallback_machine, Vec::new()),
        ),
    ];
    let mut parent_artboard = scripted_listener_artboard_and_machine().0;
    parent_artboard
        .active_nested_state_machines
        .insert(notifier_local, Box::new(parent));
    let mid_chain_error_was_none = Rc::new(RefCell::new(None));
    let observed_mid_chain = Rc::clone(&mid_chain_error_was_none);
    let mut ancestor_dispatch =
        |artboard: &mut ArtboardInstance,
         _source_local: usize,
         events: &[StateMachineReportedEvent]| {
            *observed_mid_chain.borrow_mut() = Some(
                artboard
                    .active_nested_state_machines
                    .get(&notifier_local)
                    .expect("the failing owner remains mounted")
                    .script_error()
                    .is_none(),
            );
            root.notify_events(&mut root_artboard, Some(8), events)
        };

    assert!(
        !StateMachineInstance::dispatch_nested_events_to_animation_owners(
            &mut parent_artboard,
            8,
            &mut animations,
            &mut child_artboard,
            7,
            &[audio_event],
            None,
            Some(&mut ancestor_dispatch),
        )
    );
    assert_eq!(
        *mid_chain_error_was_none.borrow(),
        Some(true),
        "the terminal ScriptError is withheld during ancestor dispatch"
    );
    let parent = parent_artboard
        .active_nested_state_machines
        .get(&notifier_local)
        .expect("the failing owner remains mounted");

    assert_eq!(
        total_order.borrow().as_slice(),
        ["parent-local", "root-local", "root-audio", "parent-audio",],
        "W63 item 3: the failing reporting owner's full-height bubble and audio tail complete before its ScriptError propagates",
    );
    assert!(parent.script_error().is_some());
    assert_eq!(
        parent.audio_event_seam_receipt(),
        (1, Some((7, audio_event_core_type))),
    );
}

fn script(
    label: &'static str,
    has_perform_action: bool,
    has_perform: bool,
    failure: ListenerFailure,
    calls: &Rc<RefCell<Vec<RecordedCall>>>,
) -> Box<dyn ScriptInstance> {
    Box::new(RecordingListenerScript {
        label,
        has_perform_action,
        has_perform,
        failure,
        state: 0,
        calls: Rc::clone(calls),
    })
}

fn scripted_test_listener(
    machine: &mut StateMachineInstance,
    action_global_id: u32,
    label: &'static str,
    failure: ListenerFailure,
    listener_types: Vec<RuntimeListenerType>,
    calls: &Rc<RefCell<Vec<RecordedCall>>>,
) -> RuntimeStateMachineListener {
    let definition = ScriptListenerActionDefinition::new(action_global_id, 1, label.to_owned());
    machine
        .scripted_listener_action_definitions
        .push(definition.clone());
    machine
        .set_scripted_listener_action_instance(
            action_global_id,
            script(label, true, false, failure, calls),
        )
        .expect("attach scripted test listener");
    RuntimeStateMachineListener {
        name: None,
        target_local_id: 1,
        is_single: false,
        listener_types,
        event_local_indices: Vec::new(),
        view_model_path: None,
        view_model_input_types: Vec::new(),
        gamepad_input_types: Vec::new(),
        keyboard_input_types: Vec::new(),
        semantic_input_types: Vec::new(),
        hit_paths: Vec::new(),
        listener_actions: vec![RuntimeScheduledListenerAction::scripted_for_test(
            0,
            Some(definition),
        )],
    }
}

#[test]
fn deferred_focus_and_semantic_callbacks_continue_after_ordinary_script_failure() {
    let (mut artboard, mut machine) = scripted_listener_artboard_and_machine();
    let calls = Rc::new(RefCell::new(Vec::new()));
    let listeners = vec![
        scripted_test_listener(
            &mut machine,
            700,
            "focus ordinary",
            ListenerFailure::Ordinary,
            vec![RuntimeListenerType::Focus],
            &calls,
        ),
        scripted_test_listener(
            &mut machine,
            701,
            "focus later",
            ListenerFailure::None,
            vec![RuntimeListenerType::Focus],
            &calls,
        ),
        scripted_test_listener(
            &mut machine,
            702,
            "semantic later",
            ListenerFailure::None,
            vec![RuntimeListenerType::SemanticAction],
            &calls,
        ),
    ];
    machine.listener_definitions = Arc::new(listeners);
    machine.queued_focus_events = vec![
        RuntimeQueuedFocusEvent {
            listener_index: 0,
            is_focus: true,
        },
        RuntimeQueuedFocusEvent {
            listener_index: 1,
            is_focus: true,
        },
    ];
    machine.queued_semantic_events = vec![RuntimeQueuedSemanticEvent {
        listener_index: Some(2),
        action_type: 1,
    }];

    assert!(machine.process_deferred_listener_group_events(&mut artboard, None));
    assert_eq!(
        calls
            .borrow()
            .iter()
            .map(|call| call.label)
            .collect::<Vec<_>>(),
        ["focus ordinary", "focus later", "semantic later"]
    );
    assert!(machine.script_error().is_none());
}

#[test]
fn terminal_focus_or_semantic_callback_stops_the_remaining_deferred_batch() {
    let resource_code = "script.resource.host_commands";

    let (mut artboard, mut machine) = scripted_listener_artboard_and_machine();
    let calls = Rc::new(RefCell::new(Vec::new()));
    let listeners = vec![
        scripted_test_listener(
            &mut machine,
            710,
            "focus terminal",
            ListenerFailure::Terminal(resource_code),
            vec![RuntimeListenerType::Focus],
            &calls,
        ),
        scripted_test_listener(
            &mut machine,
            711,
            "focus skipped",
            ListenerFailure::None,
            vec![RuntimeListenerType::Focus],
            &calls,
        ),
        scripted_test_listener(
            &mut machine,
            712,
            "semantic skipped",
            ListenerFailure::None,
            vec![RuntimeListenerType::SemanticAction],
            &calls,
        ),
    ];
    machine.listener_definitions = Arc::new(listeners);
    machine.queued_focus_events = vec![
        RuntimeQueuedFocusEvent {
            listener_index: 0,
            is_focus: true,
        },
        RuntimeQueuedFocusEvent {
            listener_index: 1,
            is_focus: true,
        },
    ];
    machine.queued_semantic_events = vec![RuntimeQueuedSemanticEvent {
        listener_index: Some(2),
        action_type: 1,
    }];

    assert!(!machine.process_deferred_listener_group_events(&mut artboard, None));
    assert_eq!(
        calls
            .borrow()
            .iter()
            .map(|call| call.label)
            .collect::<Vec<_>>(),
        ["focus terminal"]
    );
    assert_eq!(
        machine.script_error().and_then(ScriptError::resource_code),
        Some(resource_code)
    );

    let (mut artboard, mut machine) = scripted_listener_artboard_and_machine();
    let calls = Rc::new(RefCell::new(Vec::new()));
    let listeners = vec![
        scripted_test_listener(
            &mut machine,
            713,
            "focus first",
            ListenerFailure::None,
            vec![RuntimeListenerType::Focus],
            &calls,
        ),
        scripted_test_listener(
            &mut machine,
            714,
            "semantic terminal",
            ListenerFailure::Terminal(resource_code),
            vec![RuntimeListenerType::SemanticAction],
            &calls,
        ),
        scripted_test_listener(
            &mut machine,
            715,
            "semantic skipped",
            ListenerFailure::None,
            vec![RuntimeListenerType::SemanticAction],
            &calls,
        ),
    ];
    machine.listener_definitions = Arc::new(listeners);
    machine.queued_focus_events = vec![RuntimeQueuedFocusEvent {
        listener_index: 0,
        is_focus: true,
    }];
    machine.queued_semantic_events = vec![
        RuntimeQueuedSemanticEvent {
            listener_index: Some(1),
            action_type: 1,
        },
        RuntimeQueuedSemanticEvent {
            listener_index: Some(2),
            action_type: 1,
        },
    ];

    assert!(machine.process_deferred_listener_group_events(&mut artboard, None));
    assert_eq!(
        calls
            .borrow()
            .iter()
            .map(|call| call.label)
            .collect::<Vec<_>>(),
        ["focus first", "semantic terminal"]
    );
    assert_eq!(
        machine.script_error().and_then(ScriptError::resource_code),
        Some(resource_code)
    );
}

#[test]
fn view_model_callback_fifo_continues_after_ordinary_failure_and_stops_on_terminal() {
    fn run(first_failure: ListenerFailure) -> (Vec<&'static str>, Option<String>) {
        let (mut artboard, mut machine) = scripted_listener_artboard_and_machine();
        let calls = Rc::new(RefCell::new(Vec::new()));
        let listeners = Arc::new(vec![
            scripted_test_listener(
                &mut machine,
                720,
                "view model first",
                first_failure,
                vec![RuntimeListenerType::ViewModel],
                &calls,
            ),
            scripted_test_listener(
                &mut machine,
                721,
                "view model later",
                ListenerFailure::None,
                vec![RuntimeListenerType::ViewModel],
                &calls,
            ),
        ]);
        machine.listener_definitions = Arc::clone(&listeners);
        machine.view_model_listeners = (0..listeners.len())
            .filter_map(|index| {
                RuntimeViewModelListenerInstance::new(Arc::clone(&listeners), index)
            })
            .collect();
        machine.reported_listener_view_models.report_data_bind(0);
        machine.reported_listener_view_models.report_data_bind(1);

        let _ = machine.apply_local_event_listeners(&mut artboard, 0, None);
        let labels = calls
            .borrow()
            .iter()
            .map(|call| call.label)
            .collect::<Vec<_>>();
        (
            labels,
            machine
                .script_error()
                .and_then(ScriptError::resource_code)
                .map(str::to_owned),
        )
    }

    assert_eq!(
        run(ListenerFailure::Ordinary),
        (vec!["view model first", "view model later"], None)
    );
    assert_eq!(
        run(ListenerFailure::Terminal("script.resource.host_commands")),
        (
            vec!["view model first"],
            Some("script.resource.host_commands".to_owned())
        )
    );
}

#[test]
fn retained_terminal_input_error_blocks_every_later_apply_callback() {
    let (mut artboard, mut machine, _) =
        scripted_drawable_input_artboard_and_machine(Box::new(ResourceFailingDrawableInputScript));
    assert!(!machine.key_input(&mut artboard, 65, 0, true, false));

    let calls = Rc::new(RefCell::new(Vec::new()));
    let listeners = Arc::new(vec![
        scripted_test_listener(
            &mut machine,
            730,
            "focus blocked",
            ListenerFailure::None,
            vec![RuntimeListenerType::Focus],
            &calls,
        ),
        scripted_test_listener(
            &mut machine,
            731,
            "semantic blocked",
            ListenerFailure::None,
            vec![RuntimeListenerType::SemanticAction],
            &calls,
        ),
        scripted_test_listener(
            &mut machine,
            732,
            "view model blocked",
            ListenerFailure::None,
            vec![RuntimeListenerType::ViewModel],
            &calls,
        ),
    ]);
    machine.listener_definitions = Arc::clone(&listeners);
    machine.view_model_listeners = vec![
        RuntimeViewModelListenerInstance::new(Arc::clone(&listeners), 2)
            .expect("ViewModel listener occurrence"),
    ];
    machine.queued_focus_events = vec![RuntimeQueuedFocusEvent {
        listener_index: 0,
        is_focus: true,
    }];
    machine.queued_semantic_events = vec![RuntimeQueuedSemanticEvent {
        listener_index: Some(1),
        action_type: 1,
    }];
    machine.reported_listener_view_models.report_data_bind(0);

    assert!(!machine.apply_local_event_listeners(&mut artboard, 0, None));
    assert!(calls.borrow().is_empty());
    assert_eq!(machine.queued_focus_events.len(), 1);
    assert_eq!(machine.queued_semantic_events.len(), 1);
    assert!(!machine.reported_listener_view_models.is_empty());
}

#[test]
fn scripted_listener_actions_keep_authored_fifo_and_prefer_perform_action() {
    let (mut artboard, mut machine) = scripted_listener_artboard_and_machine();
    let first = machine
        .scripted_listener_actions()
        .first()
        .expect("fixture scripted listener action")
        .clone();
    let second = ScriptListenerActionDefinition::new(500, 1, "legacy".to_owned());
    machine.scripted_listener_action_definitions = vec![first.clone(), second.clone()];
    let calls = Rc::new(RefCell::new(Vec::new()));
    machine
        .set_scripted_listener_action_instance(
            first.action_global_id(),
            script("first", true, true, ListenerFailure::None, &calls),
        )
        .expect("attach first action");
    machine
        .set_scripted_listener_action_instance(
            second.action_global_id(),
            script("second", false, true, ListenerFailure::None, &calls),
        )
        .expect("attach second action");
    // Pinned C++ resolves one stateful clone from the authored
    // ScriptedListenerAction occurrence (`scripted_listener_action.cpp:
    // 88-99`). A same-id entry in the general scripted-object table must
    // not become a second callback.
    machine.set_script_instance_for_global(
        first.action_global_id(),
        script("wrong-table", true, false, ListenerFailure::None, &calls),
    );
    let actions = vec![
        RuntimeScheduledListenerAction::scripted_for_test(0, Some(first)),
        RuntimeScheduledListenerAction::scripted_for_test(0, Some(second)),
    ];
    let invocation = ScriptListenerInvocation::Pointer {
        x: 12.0,
        y: 34.0,
        previous_x: 12.0,
        previous_y: 34.0,
        pointer_id: 7,
        event: ScriptPointerEventKind::Click,
        timestamp_seconds: 0.0,
    };

    assert!(
        machine
            .perform_listener_actions(
                &mut artboard,
                &actions,
                None,
                &invocation,
                &mut NoopScriptHost,
            )
            .expect("perform listener actions")
    );
    assert_eq!(
        calls.borrow().as_slice(),
        [
            RecordedCall {
                label: "first",
                method: ScriptListenerActionMethod::PerformAction,
                invocation: invocation.clone(),
                state_before_call: 0,
            },
            RecordedCall {
                label: "second",
                method: ScriptListenerActionMethod::Perform,
                invocation,
                state_before_call: 0,
            },
        ]
    );
}

#[test]
fn successive_pointer_events_preserve_previous_position_and_timestamp() {
    let (mut artboard, mut machine) = scripted_listener_artboard_and_machine();
    let action = machine
        .scripted_listener_actions()
        .first()
        .expect("fixture scripted listener action")
        .clone();
    let calls = Rc::new(RefCell::new(Vec::new()));
    machine
        .set_scripted_listener_action_instance(
            action.action_global_id(),
            script("pointer", true, false, ListenerFailure::None, &calls),
        )
        .expect("attach pointer action");

    machine
        .try_pointer_down_with_timestamp_and_script_host(
            &mut artboard,
            200.0,
            20.0,
            1,
            1.25,
            &mut NoopScriptHost,
        )
        .expect("first pointer down");
    machine
        .try_pointer_up_with_timestamp_and_script_host(
            &mut artboard,
            200.0,
            20.0,
            1,
            1.5,
            &mut NoopScriptHost,
        )
        .expect("first pointer up");
    machine
        .try_pointer_down_with_timestamp_and_script_host(
            &mut artboard,
            205.0,
            20.0,
            1,
            2.25,
            &mut NoopScriptHost,
        )
        .expect("second pointer down");
    machine
        .try_pointer_up_with_timestamp_and_script_host(
            &mut artboard,
            210.0,
            20.0,
            1,
            2.5,
            &mut NoopScriptHost,
        )
        .expect("second pointer up");

    let invocations = calls
        .borrow()
        .iter()
        .map(|call| call.invocation.clone())
        .collect::<Vec<_>>();
    assert_eq!(
        invocations,
        [
            ScriptListenerInvocation::Pointer {
                x: 200.0,
                y: 20.0,
                previous_x: 200.0,
                previous_y: 20.0,
                pointer_id: 1,
                event: ScriptPointerEventKind::Click,
                timestamp_seconds: 1.5,
            },
            ScriptListenerInvocation::Pointer {
                x: 210.0,
                y: 20.0,
                previous_x: 205.0,
                previous_y: 20.0,
                pointer_id: 1,
                event: ScriptPointerEventKind::Click,
                timestamp_seconds: 2.5,
            },
        ]
    );
}

#[test]
fn profiler_listener_hook_records_the_runtime_listener_callsite() {
    let (mut artboard, mut machine) = scripted_listener_artboard_and_machine();
    Arc::make_mut(&mut machine.listener_definitions)[0].name =
        Some("Profiler Listener Hook".to_owned());
    let action = machine
        .scripted_listener_actions()
        .first()
        .expect("fixture scripted listener action")
        .clone();
    let calls = Rc::new(RefCell::new(Vec::new()));
    machine
        .set_scripted_listener_action_instance(
            action.action_global_id(),
            script("profiler", true, false, ListenerFailure::None, &calls),
        )
        .expect("attach profiler listener action");

    let records = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
    crate::with_rive_profile(|profile| {
        profile.set_capture(Box::new(ProfilerListenerCapture::default()));
        profile.set_listener_perform_change_flush_callback(Some(Box::new({
            let records = std::sync::Arc::clone(&records);
            move |incoming| records.lock().unwrap().extend_from_slice(incoming)
        })));
        profile.start();
    });

    machine
        .try_pointer_down_with_timestamp_and_script_host(
            &mut artboard,
            200.0,
            20.0,
            101,
            1.25,
            &mut NoopScriptHost,
        )
        .expect("profiler pointer down");
    machine
        .try_pointer_up_with_timestamp_and_script_host(
            &mut artboard,
            200.0,
            20.0,
            101,
            1.5,
            &mut NoopScriptHost,
        )
        .expect("profiler pointer up");

    let strings = crate::with_rive_profile(|profile| {
        profile.flush_listener_perform_change_records();
        profile.stop();
        let strings = profile.string_table().to_vec();
        profile.set_listener_perform_change_flush_callback(None);
        strings
    });
    let records = records.lock().unwrap();
    assert!(records.iter().any(|record| {
        strings
            .get(record.listener_name_id as usize)
            .map(String::as_str)
            == Some("Profiler Listener Hook")
            && record.listener_type == RuntimeListenerType::Click.value()
            && record.hit_event == RuntimeListenerType::Up.value()
            && record.pointer_id == 101
    }));
}

#[test]
fn matched_pointer_listener_marks_advance_even_when_actions_are_noops() {
    let (mut artboard, mut machine) = scripted_listener_artboard_and_machine();
    let scripted_calls = Rc::new(RefCell::new(Vec::new()));
    let _scripted_object = scripted_test_listener(
        &mut machine,
        98_700,
        "unmounted scripted object",
        ListenerFailure::None,
        Vec::new(),
        &scripted_calls,
    );
    assert!(
        machine.scripted_data_context_prepare_pending(),
        "the exposing fixture must retain one not-yet-mounted scripted object"
    );
    let mut listeners = machine.listener_definitions.as_ref().clone();
    assert!(
        listeners.iter_mut().any(|listener| {
            if !listener.has_listener(RuntimeListenerType::Click) {
                return false;
            }
            listener.listener_actions.clear();
            true
        }),
        "fixture must retain a click listener"
    );
    machine.listener_definitions = Arc::new(listeners);
    machine.needs_advance = false;

    assert!(machine.pointer_down(&mut artboard, 200.0, 20.0, 1));
    machine.needs_advance = false;
    assert!(machine.pointer_up(&mut artboard, 200.0, 20.0, 1));
    assert!(
        machine.needs_advance(),
        "C++ ListenerGroup::processEvent marks the machine after every matched listener, \
         even when its action list is empty (`listener_group.cpp:218-225`)"
    );
    let raw_advance_calls_before = machine.raw_advance_call_count;
    let _ = machine
        .advance_and_apply(&mut artboard, 0.25)
        .expect("ordinary bookkeeping is independent of script mount");
    assert!(
        machine.raw_advance_call_count > raw_advance_calls_before,
        "raw advance bookkeeping runs immediately through the public path while the unrelated scripted object remains unavailable"
    );
}

#[test]
fn pointer_history_is_listener_scoped_and_resets_on_first_entry_and_reentry() {
    let mut first = ListenerGroup::authored(0);
    let mut second = ListenerGroup::authored(1);

    first.reset(7);
    first.hover(7);
    let first_entry = first.process(7, (10.0, 20.0), true, false, false);
    assert!(!first_entry.previous_hovered);
    assert_eq!(first_entry.previous_position, (10.0, 20.0));
    first.record_position(7, (10.0, 20.0));

    second.reset(7);
    second.hover(7);
    let overlapping_entry = second.process(7, (100.0, 200.0), true, false, false);
    assert!(!overlapping_entry.previous_hovered);
    assert_eq!(
        overlapping_entry.previous_position,
        (100.0, 200.0),
        "a second listener group must not inherit the first group's history"
    );
    second.record_position(7, (100.0, 200.0));

    first.reset(7);
    first.hover(7);
    let move_inside = first.process(7, (15.0, 25.0), true, false, false);
    assert!(move_inside.previous_hovered);
    assert_eq!(move_inside.previous_position, (10.0, 20.0));
    first.record_position(7, (15.0, 25.0));

    first.reset(7);
    let exit = first.process(7, (30.0, 40.0), true, false, false);
    assert!(exit.previous_hovered);
    assert_eq!(exit.previous_position, (15.0, 25.0));
    first.record_position(7, (30.0, 40.0));

    first.reset(7);
    let outside = first.process(7, (50.0, 60.0), true, false, false);
    assert!(!outside.previous_hovered);
    first.record_position(7, (50.0, 60.0));
    first.reset(7);
    first.hover(7);
    let reentry = first.process(7, (70.0, 80.0), true, false, false);
    assert!(!reentry.previous_hovered);
    assert_eq!(
        reentry.previous_position,
        (70.0, 80.0),
        "reentry resets the prior outside position before dispatch"
    );
}

#[test]
fn pointer_up_position_is_retained_for_exit_then_released() {
    let mut group = ListenerGroup::authored(0);
    group.reset(9);
    group.hover(9);
    group.process(9, (10.0, 20.0), true, true, false);
    group.record_position(9, (10.0, 20.0));
    group.reset(9);
    group.hover(9);
    let up = group.process(9, (15.0, 25.0), true, false, true);
    assert!(up.previous_hovered);
    assert_eq!(up.previous_position, (10.0, 20.0));
    group.record_position(9, (15.0, 25.0));

    group.reset(9);
    let exit = group.process(9, (30.0, 40.0), true, false, false);
    assert!(exit.previous_hovered);
    assert_eq!(exit.previous_position, (15.0, 25.0));
    group.record_position(9, (30.0, 40.0));
    group.release_event(9);

    group.reset(9);
    group.hover(9);
    let next_entry = group.process(9, (50.0, 60.0), true, false, false);
    assert!(!next_entry.previous_hovered);
    assert_eq!(next_entry.previous_position, (50.0, 60.0));
}

#[test]
fn scripted_listener_failure_is_swallowed_and_later_actions_still_run() {
    let (mut artboard, mut machine) = scripted_listener_artboard_and_machine();
    let first = machine
        .scripted_listener_actions()
        .first()
        .expect("fixture scripted listener action")
        .clone();
    let second = ScriptListenerActionDefinition::new(500, 1, "later".to_owned());
    machine.scripted_listener_action_definitions = vec![first.clone(), second.clone()];
    let calls = Rc::new(RefCell::new(Vec::new()));
    machine
        .set_scripted_listener_action_instance(
            first.action_global_id(),
            script("first", true, false, ListenerFailure::Ordinary, &calls),
        )
        .expect("attach failing action");
    machine
        .set_scripted_listener_action_instance(
            second.action_global_id(),
            script("later", true, false, ListenerFailure::None, &calls),
        )
        .expect("attach later action");
    let actions = vec![
        RuntimeScheduledListenerAction::scripted_for_test(0, Some(first)),
        RuntimeScheduledListenerAction::scripted_for_test(0, Some(second)),
    ];

    assert!(
        machine
            .perform_listener_actions(
                &mut artboard,
                &actions,
                None,
                &ScriptListenerInvocation::None,
                &mut NoopScriptHost,
            )
            .expect("C++ consumes the protected-call error")
    );
    assert_eq!(
        calls
            .borrow()
            .iter()
            .map(|call| call.label)
            .collect::<Vec<_>>(),
        ["first", "later"],
        "an authored action after a failing script still runs"
    );
    assert_eq!(
        machine.script_error(),
        None,
        "listener protected-call errors do not poison the machine"
    );

    assert!(
        machine
            .perform_listener_actions(
                &mut artboard,
                &actions,
                None,
                &ScriptListenerInvocation::None,
                &mut NoopScriptHost,
            )
            .expect("later dispatch remains live")
    );
    assert_eq!(
        calls
            .borrow()
            .iter()
            .map(|call| call.label)
            .collect::<Vec<_>>(),
        ["first", "later", "first", "later"]
    );
}

#[test]
fn each_state_machine_occurrence_retains_fresh_listener_script_state() {
    let (mut first_artboard, mut first_machine) = scripted_listener_artboard_and_machine();
    let (mut second_artboard, mut second_machine) = scripted_listener_artboard_and_machine();
    let definition = first_machine
        .scripted_listener_actions()
        .first()
        .expect("fixture scripted listener action")
        .clone();
    let calls = Rc::new(RefCell::new(Vec::new()));
    first_machine
        .set_scripted_listener_action_instance(
            definition.action_global_id(),
            script("occurrence", true, false, ListenerFailure::None, &calls),
        )
        .expect("attach first occurrence");
    second_machine
        .set_scripted_listener_action_instance(
            definition.action_global_id(),
            script("occurrence", true, false, ListenerFailure::None, &calls),
        )
        .expect("attach second occurrence");
    let actions = [RuntimeScheduledListenerAction::scripted_for_test(
        0,
        Some(definition),
    )];

    first_machine
        .perform_listener_actions(
            &mut first_artboard,
            &actions,
            None,
            &ScriptListenerInvocation::None,
            &mut NoopScriptHost,
        )
        .expect("run first occurrence");
    second_machine
        .perform_listener_actions(
            &mut second_artboard,
            &actions,
            None,
            &ScriptListenerInvocation::None,
            &mut NoopScriptHost,
        )
        .expect("run second occurrence");

    assert_eq!(
        calls
            .borrow()
            .iter()
            .map(|call| call.state_before_call)
            .collect::<Vec<_>>(),
        [0, 0]
    );
}

#[test]
fn fl_c5_clone_teardown_rebuilds_mutable_state_without_aliasing() {
    let mut original = scripted_listener_machine();
    original.reported_events = vec![fl_c5_test_reported_event(10)];
    original.reporting_events = vec![fl_c5_test_reported_event(11)];
    original.bubbled_event_reports = vec![fl_c5_test_reported_event(12)];
    original.reporting_listener_view_models = vec![13];
    original.post_apply_listener_view_models = vec![14];
    original.primary_data_context = Some(RuntimeStateMachineDataContext::default());
    original.queued_focus_events = vec![RuntimeQueuedFocusEvent {
        listener_index: 3,
        is_focus: true,
    }];
    original.queued_semantic_events = vec![RuntimeQueuedSemanticEvent {
        listener_index: Some(4),
        action_type: 2,
    }];
    original.listener_groups.push(ListenerGroup::authored(2));
    let pointer_group = original
        .listener_groups
        .last_mut()
        .expect("pointer listener group");
    pointer_group.reset(5);
    pointer_group.hover(5);
    pointer_group.process(5, (-1.0, -2.0), true, true, false);
    pointer_group.begin_capture(5, None);
    pointer_group.record_position(5, (-1.0, -2.0));
    original
        .nested_event_registrations
        .push(RuntimeNestedEventRegistration {
            source_local_id: 7,
            notifier_local_id: 8,
            kind: RuntimeNestedEventNotifierKind::StateMachine,
        });
    let original_layer_ids = original
        .layers
        .iter()
        .map(StateMachineLayerInstance::view_model_trigger_layer_id)
        .collect::<Vec<_>>();
    let definition = original
        .scripted_listener_actions()
        .first()
        .expect("fixture scripted listener action")
        .clone();
    let original_calls = Rc::new(RefCell::new(Vec::new()));
    original
        .set_scripted_listener_action_instance(
            definition.action_global_id(),
            script(
                "original",
                true,
                false,
                ListenerFailure::None,
                &original_calls,
            ),
        )
        .expect("attach original occurrence");

    let mut cloned = original.clone();
    assert_eq!(
        cloned
            .reported_events
            .iter()
            .map(StateMachineReportedEvent::event_local_index)
            .collect::<Vec<_>>(),
        [10]
    );
    assert_eq!(
        cloned
            .reporting_events
            .iter()
            .map(StateMachineReportedEvent::event_local_index)
            .collect::<Vec<_>>(),
        [11]
    );
    assert_eq!(
        cloned
            .bubbled_event_reports
            .iter()
            .map(StateMachineReportedEvent::event_local_index)
            .collect::<Vec<_>>(),
        [12]
    );
    assert_ne!(
        cloned.reported_events.as_ptr(),
        original.reported_events.as_ptr(),
        "pending host reports are copied into distinct Vec storage"
    );
    assert_ne!(
        cloned.reporting_events.as_ptr(),
        original.reporting_events.as_ptr(),
        "the active event batch is copied into distinct Vec storage"
    );
    assert_ne!(
        cloned.bubbled_event_reports.as_ptr(),
        original.bubbled_event_reports.as_ptr(),
        "the nested bubbling FIFO is copied into distinct Vec storage"
    );
    assert!(
        cloned.reporting_listener_view_models.is_empty(),
        "an in-flight callback batch cannot be replayed by a snapshot"
    );
    assert_eq!(
        cloned.post_apply_listener_view_models,
        original.post_apply_listener_view_models
    );
    assert_ne!(
        cloned.post_apply_listener_view_models.as_ptr(),
        original.post_apply_listener_view_models.as_ptr(),
        "post-apply listener reports are copied into distinct Vec storage"
    );
    assert_eq!(cloned.queued_focus_events, original.queued_focus_events);
    assert_eq!(
        cloned.queued_semantic_events,
        original.queued_semantic_events
    );
    assert_ne!(
        cloned.queued_focus_events.as_ptr(),
        original.queued_focus_events.as_ptr(),
        "pending focus values are copied into distinct Vec storage"
    );
    assert_ne!(
        cloned.queued_semantic_events.as_ptr(),
        original.queued_semantic_events.as_ptr(),
        "pending semantic values are copied into distinct Vec storage"
    );
    assert_ne!(
        cloned.listener_groups.as_ptr(),
        original.listener_groups.as_ptr(),
        "listener groups and their pointer records use distinct Vec storage"
    );
    let cloned_pointer_group = cloned
        .listener_groups
        .iter_mut()
        .find(|group| group.kind == (ListenerGroupKind::Authored { listener_index: 2 }))
        .expect("cloned pointer group");
    assert_eq!(
        cloned_pointer_group.previous_position(5),
        Some((-1.0, -2.0))
    );
    cloned_pointer_group.record_position(5, (9.0, 9.0));
    assert_eq!(
        original
            .listener_groups
            .iter()
            .find(|group| group.kind == (ListenerGroupKind::Authored { listener_index: 2 }))
            .and_then(|group| group.previous_position(5)),
        Some((-1.0, -2.0)),
        "snapshot pointer records cannot mutate the source group"
    );
    assert_eq!(
        cloned.nested_event_registrations, original.nested_event_registrations,
        "snapshot registration identities are retained"
    );
    assert_ne!(
        cloned.nested_event_registrations.as_ptr(),
        original.nested_event_registrations.as_ptr(),
        "nested registrations are copied into distinct Vec storage"
    );
    assert_ne!(
        cloned.hit_components.as_ptr(),
        original.hit_components.as_ptr(),
        "the polymorphic hit-owner list has distinct Vec storage"
    );
    assert!(
        cloned
            .hit_components
            .iter()
            .zip(&original.hit_components)
            .all(|(clone, source)| !std::ptr::eq(&**clone, &**source)),
        "every polymorphic hit owner is cloned rather than shared"
    );
    assert_ne!(
        cloned.listener_groups.as_ptr(),
        original.listener_groups.as_ptr(),
        "mutable listener-group state has distinct Vec storage"
    );
    let original_context = original
        .primary_data_context
        .as_ref()
        .expect("source primary context");
    let cloned_context = cloned
        .primary_data_context
        .as_ref()
        .expect("snapshot primary context");
    assert!(
        !original_context.shares_state_for_test(&cloned_context),
        "the primary DataContext carrier is rebuilt with detached state"
    );
    original.owned_view_model_rebind_sink.take_dirt();
    cloned.owned_view_model_rebind_sink.take_dirt();
    cloned
        .owned_view_model_rebind_sink
        .add_dirt(RuntimeCellDirt::BINDINGS);
    assert!(
        original.owned_view_model_rebind_sink.peek_dirt().is_empty(),
        "the snapshot callback dirt sink cannot dirty the source occurrence"
    );
    assert!(
        cloned.scripted_listener_action_instances.is_empty()
            && cloned.scripted_instances_by_global.is_empty(),
        "mutable script tables stay cold"
    );
    let cloned_layer_ids = cloned
        .layers
        .iter()
        .map(StateMachineLayerInstance::view_model_trigger_layer_id)
        .collect::<Vec<_>>();
    assert_eq!(original_layer_ids.len(), cloned_layer_ids.len());
    assert!(
        original_layer_ids
            .iter()
            .zip(&cloned_layer_ids)
            .all(|(original, cloned)| original != cloned),
        "a cloned state-machine occurrence has distinct C++ layer-pointer identities"
    );
    let cloned_calls = Rc::new(RefCell::new(Vec::new()));
    cloned
        .set_scripted_listener_action_instance(
            definition.action_global_id(),
            script("clone", true, false, ListenerFailure::None, &cloned_calls),
        )
        .expect("clone must accept a fresh table");

    let cold_remount = scripted_listener_machine();
    assert!(
        cold_remount.queued_focus_events.is_empty()
            && cold_remount.queued_semantic_events.is_empty()
            && cold_remount
                .listener_groups
                .iter()
                .all(|group| group.previous_position(5).is_none()),
        "a cold remount starts without the snapshot's pending owned values"
    );
    assert!(
        cold_remount.scripted_listener_action_instances.is_empty()
            && cold_remount.scripted_instances_by_global.is_empty(),
        "a cold remount also starts with cold script occurrence state"
    );
}

#[test]
fn fl_c5_clone_teardown_dispose_is_repeatable_and_drop_order_is_observable() {
    let receipt = Rc::new(RefCell::new(Vec::new()));
    {
        let mut machine = scripted_listener_machine();
        machine.drop_phase_receipt = Some(Rc::clone(&receipt));
        machine
            .nested_event_registrations
            .push(RuntimeNestedEventRegistration {
                source_local_id: 7,
                notifier_local_id: 8,
                kind: RuntimeNestedEventNotifierKind::LinearAnimation,
            });
        machine.dispose();
        machine.dispose();
        assert!(machine.disposed);
        assert!(machine.nested_event_registrations.is_empty());
    }
    assert_eq!(
        receipt.borrow().as_slice(),
        ["nested-detach", "focus", "binds", "layers", "scripts"],
        "manual dispose detaches once; Drop then preserves focus → binds → layers → scripts"
    );

    let implicit_receipt = Rc::new(RefCell::new(Vec::new()));
    {
        let mut machine = scripted_listener_machine();
        machine.drop_phase_receipt = Some(Rc::clone(&implicit_receipt));
        machine
            .nested_event_registrations
            .push(RuntimeNestedEventRegistration {
                source_local_id: 9,
                notifier_local_id: 10,
                kind: RuntimeNestedEventNotifierKind::StateMachine,
            });
    }
    assert_eq!(
        implicit_receipt.borrow().as_slice(),
        ["focus", "nested-detach", "binds", "layers", "scripts"],
        "Drop prevents a stale nested registration when explicit dispose was omitted"
    );

    let event_calls = Rc::new(RefCell::new(Vec::new()));
    let (mut event_artboard, mut event_machine, _) =
        scripted_drawable_input_artboard_and_machine(Box::new(RecordingDrawableInputScript {
            label: "dispose event",
            methods: Vec::new(),
            handled: false,
            calls: event_calls,
        }));
    let event_listener = |target_local_id| RuntimeStateMachineListener {
        name: None,
        target_local_id,
        is_single: false,
        listener_types: vec![RuntimeListenerType::Event],
        event_local_indices: vec![7],
        view_model_path: None,
        view_model_input_types: Vec::new(),
        gamepad_input_types: Vec::new(),
        keyboard_input_types: Vec::new(),
        semantic_input_types: Vec::new(),
        hit_paths: Vec::new(),
        listener_actions: vec![RuntimeScheduledListenerAction::FocusClear(
            RuntimeFocusActionClear::for_test(0),
        )],
    };
    let event = StateMachineReportedEvent {
        event_local_index: 7,
        event_core_type: 128,
        name: Some("nested".to_owned()),
        url: None,
        target: None,
        properties: Vec::new(),
        string_properties: Vec::new(),
        seconds_delay: 0.0,
        context: None,
    };
    event_machine.listener_definitions = Arc::new(vec![event_listener(7)]);
    event_machine
        .nested_event_registrations
        .push(RuntimeNestedEventRegistration {
            source_local_id: 7,
            notifier_local_id: 8,
            kind: RuntimeNestedEventNotifierKind::StateMachine,
        });
    assert!(event_machine.notify_events(&mut event_artboard, Some(7), &[event.clone()]));
    assert!(!event_machine.focus.target_has_focus(1));
    assert!(event_machine.focus.set_focus_target(1));
    event_machine.dispose();
    event_machine.dispose();
    assert!(!event_machine.notify_events(&mut event_artboard, Some(7), &[event.clone()]));
    assert!(
        event_machine.focus.target_has_focus(1),
        "a detached child source can no longer clear the parent's focus"
    );
    event_machine.listener_definitions = Arc::new(vec![event_listener(0)]);
    assert!(event_machine.notify_events(&mut event_artboard, None, &[event]));
    assert!(
        !event_machine.focus.target_has_focus(1),
        "dispose detaches nested sources without disabling unrelated local events"
    );

    let focus_calls = Rc::new(RefCell::new(Vec::new()));
    let (_artboard, mut focus_owner, _) =
        scripted_drawable_input_artboard_and_machine(Box::new(RecordingDrawableInputScript {
            label: "focus owner",
            methods: Vec::new(),
            handled: false,
            calls: focus_calls,
        }));
    {
        let mut externally_managed = scripted_listener_machine();
        externally_managed.install_external_focus(&focus_owner.focus, 99);
    }
    assert!(
        focus_owner.focus.target_has_focus(1),
        "dropping an external focus projection leaves its owner's tree intact"
    );

    let mut retained_external = scripted_listener_machine();
    {
        let focus_calls = Rc::new(RefCell::new(Vec::new()));
        let (_artboard, internal_owner, _) =
            scripted_drawable_input_artboard_and_machine(Box::new(RecordingDrawableInputScript {
                label: "internal focus owner",
                methods: Vec::new(),
                handled: false,
                calls: focus_calls,
            }));
        retained_external.install_external_focus(&internal_owner.focus, 101);
        assert!(
            !retained_external.focus.focused_listener_chain().is_empty(),
            "the retained projection observes the internal owner's focus"
        );
    }
    assert!(
        retained_external.focus.focused_listener_chain().is_empty(),
        "dropping the internal owner clears focus before external Rc projections survive"
    );
}

#[test]
fn transactional_candidate_can_adopt_the_same_occurrence_listener_state() {
    let (mut artboard, mut original) = scripted_listener_artboard_and_machine();
    let definition = original
        .scripted_listener_actions()
        .first()
        .expect("fixture scripted listener action")
        .clone();
    let calls = Rc::new(RefCell::new(Vec::new()));
    original
        .set_scripted_listener_action_instance(
            definition.action_global_id(),
            script("transaction", true, false, ListenerFailure::None, &calls),
        )
        .expect("attach original occurrence");
    let mut candidate = original.clone();

    candidate
        .adopt_scripted_listener_action_state_from(&original)
        .expect("validated candidate represents the same occurrence");
    candidate
        .perform_listener_actions(
            &mut artboard,
            &[RuntimeScheduledListenerAction::scripted_for_test(
                0,
                Some(definition),
            )],
            None,
            &ScriptListenerInvocation::None,
            &mut NoopScriptHost,
        )
        .expect("committed candidate retains the listener table");

    assert_eq!(calls.borrow().len(), 1);
}

fn fl_c5_bind_record(type_name: &str, properties: Vec<FixtureProperty>) -> FixtureRecord {
    FixtureRecord {
        type_key: nuxie_schema::definition_by_name(type_name)
            .unwrap_or_else(|| panic!("missing schema definition {type_name}"))
            .type_key
            .int,
        properties,
    }
}

fn fl_c5_bind_property(type_name: &str, name: &str, value: FixtureValue) -> FixtureProperty {
    FixtureProperty {
        key: property_key_for_name(type_name, name)
            .unwrap_or_else(|| panic!("missing property {type_name}.{name}")),
        value,
    }
}

fn fl_c5_bind_file_and_artboard() -> (RuntimeFile, ArtboardInstance) {
    let file = RuntimeFile::from_fixture_records(vec![
        fl_c5_bind_record("Backboard", Vec::new()),
        fl_c5_bind_record(
            "ViewModel",
            vec![fl_c5_bind_property(
                "ViewModel",
                "name",
                FixtureValue::String("Main".to_owned()),
            )],
        ),
        fl_c5_bind_record(
            "ViewModel",
            vec![
                fl_c5_bind_property(
                    "ViewModel",
                    "name",
                    FixtureValue::String("Global A".to_owned()),
                ),
                fl_c5_bind_property("ViewModel", "viewModelType", FixtureValue::Uint(2)),
            ],
        ),
        fl_c5_bind_record(
            "ViewModel",
            vec![
                fl_c5_bind_property(
                    "ViewModel",
                    "name",
                    FixtureValue::String("Global B".to_owned()),
                ),
                fl_c5_bind_property("ViewModel", "viewModelType", FixtureValue::Uint(2)),
            ],
        ),
        fl_c5_bind_record(
            "ViewModel",
            vec![fl_c5_bind_property(
                "ViewModel",
                "name",
                FixtureValue::String("Standard".to_owned()),
            )],
        ),
        fl_c5_bind_record(
            "Artboard",
            vec![
                fl_c5_bind_property("Artboard", "width", FixtureValue::Double(100.0)),
                fl_c5_bind_property("Artboard", "height", FixtureValue::Double(100.0)),
                fl_c5_bind_property("Artboard", "viewModelId", FixtureValue::Uint(0)),
            ],
        ),
    ])
    .expect("WP5 binding fixture imports");
    let graph = GraphFile::from_runtime_file(&file).expect("WP5 binding fixture graphs");
    let artboard =
        ArtboardInstance::from_graph(&file, graph.artboards.first().expect("fixture artboard"))
            .expect("WP5 binding fixture artboard");
    (file, artboard)
}

fn fl_c5_bind_handle(file: &RuntimeFile, view_model_index: usize) -> RuntimeOwnedViewModelHandle {
    RuntimeOwnedViewModelHandle::new(
        RuntimeOwnedViewModelInstance::new(file, view_model_index)
            .expect("fixture ViewModel instance"),
    )
}

#[test]
fn fl_c5_bind_staged_main_and_globals_apply_only_through_primary_bind() {
    let (file, mut artboard) = fl_c5_bind_file_and_artboard();
    let mut machine = scripted_listener_machine();
    machine.bind_phase_trace.clear();
    let initial_context_kind = machine.data_bind_graph.context_kind;
    let main = fl_c5_bind_handle(&file, 0);
    let override_a = fl_c5_bind_handle(&file, 2);
    let replacement_a = fl_c5_bind_handle(&file, 3);

    let mut invalid_global_machine = scripted_listener_machine();
    assert!(!invalid_global_machine.set_global_view_model_instance(
        Some(&file),
        "missing",
        Some(override_a.clone()),
    ));
    assert!(!invalid_global_machine.set_global_view_model_instance(
        Some(&file),
        "Standard",
        Some(override_a.clone()),
    ));
    assert!(
        invalid_global_machine.data_context().is_none(),
        "failed global validation must not create or register an empty DataContext"
    );

    assert!(!machine.set_view_model_instance(None));
    assert!(machine.data_context().is_none());
    assert!(machine.set_view_model_instance(Some(main.clone())));
    assert_eq!(machine.data_bind_graph.context_kind, initial_context_kind);
    assert!(
        machine
            .primary_data_context
            .as_ref()
            .map(RuntimeStateMachineDataContext::snapshot)
            .as_ref()
            .and_then(RuntimeOwnedViewModelContext::main_handle)
            .is_some_and(|bound| bound.ptr_eq(&main))
    );

    assert!(machine.set_global_view_model_instance(Some(&file), "Global A", None,));
    assert!(
        machine
            .global_view_model_instance(Some(&file), "Global A")
            .is_none()
    );
    assert!(!machine.set_global_view_model_instance(None, "Global A", Some(override_a.clone()),));
    assert!(!machine.set_global_view_model_instance(
        Some(&file),
        "missing",
        Some(override_a.clone()),
    ));
    assert!(!machine.set_global_view_model_instance(
        Some(&file),
        "Standard",
        Some(override_a.clone()),
    ));
    assert!(machine.set_global_view_model_instance(
        Some(&file),
        "Global A",
        Some(override_a.clone()),
    ));
    assert!(
        machine
            .global_view_model_instance(Some(&file), "Global A")
            .is_some_and(|bound| bound.ptr_eq(&override_a)),
        "slot identity comes from the requested global, not the override's ViewModel"
    );
    assert!(machine.set_global_view_model_instance(
        Some(&file),
        "Global A",
        Some(replacement_a.clone()),
    ));
    assert!(
        machine
            .global_view_model_instance(Some(&file), "Global A")
            .is_some_and(|bound| bound.ptr_eq(&replacement_a))
    );
    assert!(machine.set_global_view_model_instance(Some(&file), "Global A", None));
    assert!(
        machine
            .global_view_model_instance(Some(&file), "Global A")
            .is_none(),
        "a null instance empties the named slot"
    );
    assert!(machine.set_global_view_model_instance(
        Some(&file),
        "Global A",
        Some(replacement_a.clone()),
    ));
    assert!(
        machine
            .global_view_model_instance(Some(&file), "Standard")
            .is_none(),
        "the getter performs a pure numeric-slot read and never creates"
    );
    let unusual_slot = fl_c5_bind_handle(&file, 0);
    machine
        .primary_data_context
        .as_ref()
        .expect("primary context")
        .set_unusual_slot_for_test(3, unusual_slot.clone());
    assert!(
        machine
            .global_view_model_instance(Some(&file), "Standard")
            .is_some_and(|bound| bound.ptr_eq(&unusual_slot)),
        "the pure getter reads an occupied numeric slot even when the named ViewModel is non-global"
    );
    assert_eq!(machine.data_bind_graph.context_kind, initial_context_kind);

    let mut empty_machine = scripted_listener_machine();
    empty_machine.view_model_listeners.clear();
    assert!(empty_machine.data_context().is_none());
    assert!(empty_machine.set_global_view_model_instance(Some(&file), "Global A", None));
    assert!(
        empty_machine.data_context().is_none(),
        "clearing an empty valid slot must not allocate a DataContext"
    );

    let (fresh_file, mut fresh_artboard) = fl_c5_bind_file_and_artboard();
    let mut fresh_machine = scripted_listener_machine();
    fresh_machine.view_model_listeners.clear();
    fresh_machine
        .bind(Some(&fresh_file), &mut fresh_artboard)
        .expect("bind without a prior context completes defaults");
    assert!(fresh_machine.data_context().is_some());
    assert!(
        fresh_machine
            .global_view_model_instance(Some(&fresh_file), "Global A")
            .is_some()
    );

    let mut staged_artboard = fresh_artboard;
    let artboard_global = fl_c5_bind_handle(&fresh_file, 2);
    assert!(staged_artboard.set_global_view_model_instance(
        &fresh_file,
        "Global A",
        Some(artboard_global.clone()),
    ));
    assert!(
        staged_artboard
            .global_view_model_instance(&fresh_file, "Global A")
            .is_some_and(|bound| bound.ptr_eq(&artboard_global))
    );
    assert!(staged_artboard.set_global_view_model_instance(&fresh_file, "Global A", None,));
    assert!(
        staged_artboard
            .global_view_model_instance(&fresh_file, "Global A")
            .is_none()
    );

    machine.bind_phase_trace.clear();
    machine
        .bind(Some(&file), &mut artboard)
        .expect("staged primary bind");
    assert_eq!(
        machine.bind_phase_trace,
        [
            "complete-view-models",
            "bind-artboard",
            "bind-machine",
            "assign-context",
            "bind-data-binds",
            "bind-listener-cells",
            "script-context-pass",
            "script-init-pass",
        ],
        "completion and artboard binding precede the machine's exact internal member order"
    );
    let staged = machine
        .primary_data_context
        .as_ref()
        .expect("completed staged context")
        .snapshot();
    assert!(staged.main_handle().is_some());
    assert!(
        staged
            .global_slot_handle(1)
            .is_some_and(|bound| bound.ptr_eq(&replacement_a))
    );
    assert!(staged.global_slot_handle(2).is_some());
    assert_eq!(
        staged.handles().count(),
        3,
        "completion inserts main first, then both globals, without replacing occupied A"
    );
    let staged_main_view_model = staged
        .main_handle()
        .expect("completed main")
        .borrow()
        .view_model_index();
    let staged_global_a_view_model = staged
        .global_slot_handle(1)
        .expect("occupied global A")
        .borrow()
        .view_model_index();
    println!(
        "FLC5_COMPLETE_DIFF main={staged_main_view_model} global_a={staged_global_a_view_model} global_b={}",
        usize::from(staged.global_slot_handle(2).is_some())
    );

    let bound_before_replacement = machine
        .owned_data_context
        .clone()
        .expect("bound machine projection");
    let replacement_main = fl_c5_bind_handle(&file, 0);
    assert!(machine.set_view_model_instance(Some(replacement_main.clone())));
    let staged_after_replacement = machine
        .primary_data_context
        .as_ref()
        .expect("retained primary context")
        .projection();
    assert!(
        !bound_before_replacement.same_binding(&staged_after_replacement),
        "the shared slot table owns the staged replacement identity"
    );
    assert!(
        machine
            .owned_data_context
            .as_ref()
            .is_some_and(|bound| bound.same_binding(&bound_before_replacement))
            && artboard
                .artboard_owned_data_context
                .as_ref()
                .is_some_and(|bound| bound.same_binding(&bound_before_replacement)),
        "setViewModelInstance leaves machine and artboard paths on the old projection"
    );
    assert!(
        machine.owned_view_model_rebind_sink.peek_dirt().is_empty()
            && artboard
                .artboard_owned_view_model_rebind_sink
                .peek_dirt()
                .is_empty(),
        "staging a replacement does not synthesize structural dirt"
    );
    machine
        .bind(Some(&file), &mut artboard)
        .expect("explicit replacement bind");
    assert!(
        machine
            .owned_data_context
            .as_ref()
            .is_some_and(|bound| bound.same_binding(&staged_after_replacement))
            && artboard
                .artboard_owned_data_context
                .as_ref()
                .is_some_and(|bound| bound.same_binding(&staged_after_replacement)),
        "the explicit bind is the first point where staged paths move"
    );
}

#[test]
fn fl_c5_bind_null_matrix_keeps_every_cpp_branch_distinct() {
    let (file, mut artboard) = fl_c5_bind_file_and_artboard();
    let mut machine = scripted_listener_machine();
    machine.view_model_listeners.clear();
    let main = fl_c5_bind_handle(&file, 0);

    assert!(machine.set_view_model_instance(Some(main.clone())));
    machine
        .bind(Some(&file), &mut artboard)
        .expect("initial primary bind");
    assert_eq!(
        machine.data_bind_graph.context_kind,
        RuntimeDataBindGraphContextKind::OwnedViewModel
    );
    assert!(!machine.set_view_model_instance(None));
    assert!(machine.data_context().is_some());

    machine.bind_phase_trace.clear();
    machine
        .bind_view_model_instance(Some(&file), &mut artboard, None)
        .expect("bindViewModelInstance null is the limited clear branch");
    assert!(machine.data_context().is_none());
    assert!(artboard.artboard_owned_data_context.is_none());
    assert_eq!(
        machine.data_bind_graph.context_kind,
        RuntimeDataBindGraphContextKind::OwnedViewModel,
        "bindViewModelInstance(nullptr) does not explicitly unbind machine DataBinds"
    );
    assert_eq!(
        machine.bind_phase_trace,
        ["clear-machine", "unbind-artboard"]
    );

    assert_eq!(
        machine.bind_data_context(&file, &mut artboard, None),
        Err(RuntimeDataContextBindError::NullDataContext),
        "bindDataContext(nullptr) is not a safe clear"
    );
    assert_eq!(machine.inherit_data_context(None), Ok(false));

    assert_eq!(machine.set_data_context(None), Ok(true));
    assert_eq!(
        machine.data_bind_graph.context_kind,
        RuntimeDataBindGraphContextKind::None,
        "dataContext(nullptr) reaches the internal null bind when no VM listener exists"
    );
    machine
        .view_model_listeners
        .push(RuntimeViewModelListenerInstance {
            listener_definitions: Arc::new(Vec::new()),
            listener_index: 0,
            property_bindings: Vec::new(),
        });
    machine.bind_phase_trace.clear();
    assert_eq!(
        machine.set_data_context(None),
        Err(RuntimeDataContextBindError::NullDataContextWithViewModelListeners),
        "the C++ listener dereference hazard remains distinct"
    );
    assert_eq!(
        machine.bind_phase_trace,
        [
            "clear-machine",
            "assign-context",
            "bind-data-binds",
            "bind-listener-cells",
        ],
        "listener failure prevents both scripted context/init passes"
    );
    assert_eq!(
        machine.rebuild_data_bind(None),
        Err(RuntimeDataContextBindError::NullDataBind)
    );

    let mut differential_machine = scripted_listener_machine();
    differential_machine.view_model_listeners.clear();
    let staged_main = fl_c5_bind_handle(&file, 0);
    assert!(differential_machine.set_view_model_instance(Some(staged_main)));
    let staged_view_model = differential_machine
        .data_context()
        .and_then(|context| context.snapshot().main_handle().cloned())
        .expect("staged differential main")
        .borrow()
        .view_model_index();
    let bound_main = fl_c5_bind_handle(&file, 3);
    differential_machine
        .bind_view_model_instance(Some(&file), &mut artboard, Some(bound_main))
        .expect("non-null differential bind");
    let bound_view_model = differential_machine
        .data_context()
        .and_then(|context| context.snapshot().main_handle().cloned())
        .expect("bound differential main")
        .borrow()
        .view_model_index();
    differential_machine
        .bind_view_model_instance(Some(&file), &mut artboard, None)
        .expect("null differential bind");
    println!(
        "FLC5_BIND_NULL_DIFF staged={staged_view_model} bound={bound_view_model} cleared={}",
        usize::from(differential_machine.data_context().is_none())
    );
}

#[test]
fn fl_c5_bind_data_context_and_rebind_preserve_artboard_machine_order() {
    let (file, mut artboard) = fl_c5_bind_file_and_artboard();
    let mut machine = scripted_listener_machine();
    machine.view_model_listeners.clear();
    let context = RuntimeStateMachineDataContext::from_owned_context(
        RuntimeOwnedViewModelContext::from_main_handle(fl_c5_bind_handle(&file, 0)),
    );

    machine.bind_phase_trace.clear();
    machine
        .bind_data_context(&file, &mut artboard, Some(&context))
        .expect("bindDataContext");
    assert_eq!(
        machine.bind_phase_trace,
        [
            "clear-machine",
            "register-machine",
            "clear-artboard",
            "bind-artboard",
            "bind-machine",
            "assign-context",
            "bind-data-binds",
            "bind-listener-cells",
            "script-context-pass",
            "script-init-pass",
        ]
    );
    assert!(
        machine
            .data_context()
            .is_some_and(|bound| bound.ptr_eq(&context))
    );
    assert!(
        artboard
            .artboard_owned_data_context
            .as_ref()
            .is_some_and(|bound| bound.same_binding(&context.projection()))
    );

    machine.bind_phase_trace.clear();
    machine.rebind(&file, &mut artboard).expect("rebind");
    assert_eq!(
        machine.bind_phase_trace,
        [
            "clear-artboard",
            "bind-artboard",
            "bind-machine",
            "assign-context",
            "bind-data-binds",
            "bind-listener-cells",
            "script-context-pass",
            "script-init-pass",
        ]
    );
    machine.bind_phase_trace.clear();
    let _ = machine.relink_data_context(&file, &mut artboard);
    assert!(
        machine.bind_phase_trace.is_empty(),
        "relinkDataContext delegates to the artboard only"
    );
}

#[test]
fn fl_c5_bind_setters_preserve_an_existing_unregistered_context() {
    let (file, _artboard) = fl_c5_bind_file_and_artboard();
    let mut machine = scripted_listener_machine();
    machine.view_model_listeners.clear();
    let context = RuntimeStateMachineDataContext::from_owned_context(
        RuntimeOwnedViewModelContext::from_main_handle(fl_c5_bind_handle(&file, 0)),
    );

    machine
        .set_data_context(Some(&context))
        .expect("dataContext setter");
    machine.owned_view_model_rebind_sink.take_dirt();
    assert!(machine.set_view_model_instance(Some(fl_c5_bind_handle(&file, 0))));
    assert!(machine.set_global_view_model_instance(
        Some(&file),
        "Global A",
        Some(fl_c5_bind_handle(&file, 1)),
    ));
    context.mark_main_rebind_for_test();
    assert!(
        machine.owned_view_model_rebind_sink.peek_dirt().is_empty(),
        "setters reuse the non-registering dataContext(value) carrier without inventing addDependentContainer"
    );
}

#[test]
fn fl_c5_bind_inherit_a_then_b_retains_the_prior_registration_hazard() {
    let (file, _artboard) = fl_c5_bind_file_and_artboard();
    let mut machine = scripted_listener_machine();
    machine.view_model_listeners.clear();
    let context_a = RuntimeStateMachineDataContext::from_owned_context(
        RuntimeOwnedViewModelContext::from_main_handle(fl_c5_bind_handle(&file, 0)),
    );
    let context_b = RuntimeStateMachineDataContext::from_owned_context(
        RuntimeOwnedViewModelContext::from_main_handle(fl_c5_bind_handle(&file, 3)),
    );

    machine
        .inherit_data_context(Some(&context_a))
        .expect("inherit A");
    let inherited_sink = machine.owned_view_model_rebind_sink.clone();
    machine.bind_phase_trace.clear();
    machine
        .inherit_data_context(Some(&context_b))
        .expect("inherit B");
    inherited_sink.take_dirt();
    context_a.mark_main_rebind_for_test();
    assert!(
        machine
            .owned_view_model_rebind_sink
            .peek_dirt()
            .contains(RuntimeCellDirt::BINDINGS),
        "structural dirt from A after inheriting B still reaches the machine because inherit never clears A"
    );
    assert!(
        machine
            .data_context()
            .is_some_and(|bound| bound.ptr_eq(&context_b))
    );
    assert_eq!(
        machine.bind_phase_trace,
        [
            "register-machine-without-clear",
            "assign-context",
            "bind-data-binds",
            "bind-listener-cells",
            "script-context-pass",
            "script-init-pass",
        ],
        "the A→B path contains no clear phase"
    );
    let a_registered = machine
        .owned_view_model_rebind_sink
        .peek_dirt()
        .contains(RuntimeCellDirt::BINDINGS);
    machine.owned_view_model_rebind_sink.take_dirt();
    context_b.mark_main_rebind_for_test();
    let b_registered = machine
        .owned_view_model_rebind_sink
        .peek_dirt()
        .contains(RuntimeCellDirt::BINDINGS);
    let current_view_model = machine
        .data_context()
        .and_then(|context| context.snapshot().main_handle().cloned())
        .expect("current inherited main")
        .borrow()
        .view_model_index();
    println!(
        "FLC5_INHERIT_DIFF current={current_view_model} a_registered={} b_registered={}",
        usize::from(a_registered),
        usize::from(b_registered)
    );
}

#[test]
fn fl_c5_bind_shared_context_repoints_all_registered_machine_sinks() {
    let (file, mut artboard_a) = fl_c5_bind_file_and_artboard();
    let mut artboard_b = artboard_a.clone();
    let context = RuntimeStateMachineDataContext::default();
    let mut machine_a = scripted_listener_machine();
    let mut machine_b = scripted_listener_machine();
    machine_a.view_model_listeners.clear();
    machine_b.view_model_listeners.clear();

    machine_a
        .bind_data_context(&file, &mut artboard_a, Some(&context))
        .expect("bind shared context to A");
    machine_b
        .bind_data_context(&file, &mut artboard_b, Some(&context))
        .expect("bind shared context to B");
    machine_a.owned_view_model_rebind_sink.take_dirt();
    machine_b.owned_view_model_rebind_sink.take_dirt();
    artboard_a.artboard_owned_view_model_rebind_sink.take_dirt();
    artboard_b.artboard_owned_view_model_rebind_sink.take_dirt();

    let replacement = fl_c5_bind_handle(&file, 0);
    context.set_main(replacement.clone());
    assert!(
        machine_a
            .owned_view_model_rebind_sink
            .peek_dirt()
            .is_empty()
            && machine_b
                .owned_view_model_rebind_sink
                .peek_dirt()
                .is_empty(),
        "slot replacement stages identity without scheduling a bind"
    );
    let detached_relay = context
        .main_rebind_dependent_for_test()
        .expect("replacement relay");
    let final_replacement = fl_c5_bind_handle(&file, 0);
    context.set_main(final_replacement.clone());
    assert!(
        !detached_relay.add_dirt(RuntimeCellDirt::BINDINGS),
        "the replaced handle's relay is dropped, making its weak registration inert"
    );
    context.mark_main_rebind_for_test();
    assert!(
        machine_a
            .owned_view_model_rebind_sink
            .peek_dirt()
            .contains(RuntimeCellDirt::BINDINGS)
    );
    assert!(
        machine_b
            .owned_view_model_rebind_sink
            .peek_dirt()
            .contains(RuntimeCellDirt::BINDINGS)
    );
    assert!(
        artboard_a
            .artboard_owned_view_model_rebind_sink
            .peek_dirt()
            .contains(RuntimeCellDirt::BINDINGS)
            && artboard_b
                .artboard_owned_view_model_rebind_sink
                .peek_dirt()
                .contains(RuntimeCellDirt::BINDINGS),
        "the active relay forwards later structural dirt to every registered artboard"
    );
    assert!(
        context
            .snapshot()
            .main_handle()
            .is_some_and(|bound| bound.ptr_eq(&final_replacement)),
        "one mutable primary context retains the replacement identity for every dependent"
    );

    assert!(machine_a.complete_view_model_instances(Some(&file), &artboard_a));
    assert!(
        machine_b
            .global_view_model_instance(Some(&file), "Global A")
            .is_some(),
        "completion on one registered container mutates the shared slot table"
    );
}

#[test]
fn fl_c5_bind_typed_context_apis_delegate_without_signature_changes() {
    let (file, _artboard) = fl_c5_bind_file_and_artboard();
    let mut machine = scripted_listener_machine();
    machine.view_model_listeners.clear();
    let main = fl_c5_bind_handle(&file, 0);
    let context_handle = RuntimeOwnedViewModelContextHandle::root(&file, main.clone());
    let mut contexts = RuntimeOwnedViewModelContext::from_main_handle(main.clone());
    assert!(contexts.set_global_slot_handle(&file, 1, fl_c5_bind_handle(&file, 2)));

    let _: bool = machine.bind_owned_view_model_handle(&main);
    assert!(machine.data_context().is_some());
    let _: bool = machine.bind_owned_view_model_context_handle(&context_handle);
    assert!(machine.owned_data_context.is_some());
    let _: bool = machine.bind_owned_view_model_contexts(&contexts);
    assert!(
        machine
            .primary_data_context
            .as_ref()
            .map(RuntimeStateMachineDataContext::snapshot)
            .as_ref()
            .and_then(|context| context.global_slot_handle(1))
            .is_some()
    );
    let _: bool = machine
        .bind_script_artboard_data_context(&ScriptArtboardDataContext::root(&context_handle));
    assert!(machine.owned_data_context.is_some());
}

#[test]
fn fl_c5_focus_semantic_focus_state_and_owner_safe_focus_accessors() {
    let (_artboard, mut machine, _) =
        scripted_drawable_input_artboard_and_machine(Box::new(RecordingDrawableInputScript {
            label: "focus state",
            methods: Vec::new(),
            handled: false,
            calls: Rc::new(RefCell::new(Vec::new())),
        }));
    machine.keyboard_listener_groups.clear();
    machine.publish_focusable_keyboard_capabilities();

    assert_eq!(
        machine.focus_state(),
        FocusState {
            has_focus: true,
            expects_keyboard_input: false,
        },
        "a focused FocusData without key/text listeners is not a keyboard consumer"
    );
    assert!(machine.internal_focus_manager());
    assert!(!machine.has_external_focus_manager());

    machine.keyboard_listener_groups.push(
        RuntimeKeyboardListenerGroup::scripted(1, 2, 90_901, true, false)
            .expect("keyboard-consuming focus group"),
    );
    machine.publish_focusable_keyboard_capabilities();
    assert_eq!(
        machine.focus_state(),
        FocusState {
            has_focus: true,
            expects_keyboard_input: true,
        }
    );
    let pending_focus_events = machine.queued_focus_events.len();
    assert!(
        !machine.set_focus(Some(1)),
        "setting the already-focused valid FocusData is a no-op"
    );
    assert!(machine.focus_state().has_focus);
    assert_eq!(machine.queued_focus_events.len(), pending_focus_events);

    assert!(machine.set_focus(None));
    assert_eq!(machine.focus_state(), FocusState::default());
    assert!(machine.set_focus(Some(1)));
    assert!(
        machine.set_focus(Some(usize::MAX)),
        "a missing owner-safe retained FocusData/node clears current focus"
    );
    assert_eq!(machine.focus_state(), FocusState::default());
}

#[test]
fn fl_c5_focus_semantic_manager_switch_is_identity_noop_and_restores_internal() {
    let (_artboard, mut machine, _) =
        scripted_drawable_input_artboard_and_machine(Box::new(RecordingDrawableInputScript {
            label: "internal",
            methods: Vec::new(),
            handled: false,
            calls: Rc::new(RefCell::new(Vec::new())),
        }));
    let (_parent_artboard, parent, _) =
        scripted_drawable_input_artboard_and_machine(Box::new(RecordingDrawableInputScript {
            label: "external",
            methods: Vec::new(),
            handled: false,
            calls: Rc::new(RefCell::new(Vec::new())),
        }));
    machine.focus_manager_phase_trace.clear();
    machine.install_external_focus(&parent.focus, 90_910);
    assert!(machine.has_external_focus_manager());
    assert_eq!(machine.focus.owner_identity(), 90_910);
    assert_eq!(
        machine.focus_manager_phase_trace,
        [
            "clean-retained-tree",
            "assign-external",
            "select-retained-tree",
        ]
    );

    machine.focus_manager_phase_trace.clear();
    let same_manager_other_projection = parent.focus.external_for_owner(90_999);
    machine.install_external_focus(&same_manager_other_projection, 90_911);
    assert_eq!(
        machine.focus.owner_identity(),
        90_910,
        "the same shared manager is a no-op even through a different owner projection"
    );
    assert!(machine.focus_manager_phase_trace.is_empty());

    assert!(machine.clear_external_focus_manager());
    assert!(!machine.has_external_focus_manager());
    assert!(machine.internal_focus_manager());
    assert_eq!(
        machine.focus_manager_phase_trace,
        [
            "clean-retained-tree",
            "assign-internal",
            "select-retained-tree",
        ]
    );
    assert_eq!(
        machine.focus_state(),
        FocusState::default(),
        "cleanup clears old focus before external-to-null restores the retained internal manager"
    );
    assert!(machine.has_focus_nodes());
    assert!(
        !machine.clear_external_focus_manager(),
        "null-to-null is the same-manager no-op"
    );
}

#[test]
fn fl_c5_focus_semantic_batches_snapshot_clear_and_keep_focus_then_semantic_fifo() {
    let (mut artboard, mut machine) = scripted_listener_artboard_and_machine();
    let calls = Rc::new(RefCell::new(Vec::new()));
    machine.listener_definitions = Arc::new(vec![
        scripted_test_listener(
            &mut machine,
            90_920,
            "focus",
            ListenerFailure::None,
            vec![RuntimeListenerType::Focus],
            &calls,
        ),
        scripted_test_listener(
            &mut machine,
            90_921,
            "semantic",
            ListenerFailure::None,
            vec![RuntimeListenerType::SemanticAction],
            &calls,
        ),
    ]);

    machine.queue_focus_event(0, true);
    machine.queue_focus_event(0, true);
    machine.queue_semantic_event(None, 0);
    machine.queue_semantic_event(Some(usize::MAX), 0);
    machine.queue_semantic_event(Some(1), 0);
    machine.queue_semantic_event(Some(1), 0);
    assert!(machine.needs_advance);

    assert!(machine.process_focus_events(&mut artboard, None));
    assert!(machine.queued_focus_events.is_empty());
    assert!(
        machine.process_semantic_events(&mut artboard, None),
        "null group/listener records are skipped without suppressing later valid duplicates"
    );
    assert!(machine.queued_semantic_events.is_empty());
    assert_eq!(
        calls
            .borrow()
            .iter()
            .map(|call| call.label)
            .collect::<Vec<_>>(),
        ["focus", "focus", "semantic", "semantic"]
    );
}

#[test]
fn fl_c5_focus_semantic_callback_generated_batches_obey_phase_snapshots() {
    let (mut artboard, mut machine) = scripted_listener_artboard_and_machine();
    let calls = Rc::new(RefCell::new(Vec::new()));
    machine.listener_definitions = Arc::new(vec![
        scripted_test_listener(
            &mut machine,
            90_925,
            "focus",
            ListenerFailure::None,
            vec![RuntimeListenerType::Focus],
            &calls,
        ),
        scripted_test_listener(
            &mut machine,
            90_926,
            "semantic",
            ListenerFailure::None,
            vec![RuntimeListenerType::SemanticAction],
            &calls,
        ),
    ]);
    machine.queue_focus_event(0, true);
    machine.deferred_callback_probe = Some(RuntimeDeferredCallbackProbe::FocusQueuesSemantic {
        listener_index: Some(1),
        action_type: 0,
    });

    assert!(machine.process_deferred_listener_group_events(&mut artboard, None));
    assert_eq!(
        calls
            .borrow()
            .iter()
            .map(|call| call.label)
            .collect::<Vec<_>>(),
        ["focus", "semantic"],
        "semantic work generated by a focus callback joins the later same-frame snapshot"
    );
    assert!(machine.queued_semantic_events.is_empty());

    calls.borrow_mut().clear();
    machine.queue_semantic_event(Some(1), 0);
    machine.deferred_callback_probe = Some(RuntimeDeferredCallbackProbe::SemanticQueuesSemantic {
        listener_index: Some(1),
        action_type: 0,
    });
    assert!(machine.process_semantic_events(&mut artboard, None));
    assert_eq!(calls.borrow().len(), 1);
    assert_eq!(
        machine.queued_semantic_events,
        [RuntimeQueuedSemanticEvent {
            listener_index: Some(1),
            action_type: 0,
        }],
        "semantic work generated inside the active semantic batch waits for a later frame"
    );
    assert!(machine.process_semantic_events(&mut artboard, None));
    assert_eq!(calls.borrow().len(), 2);
    assert!(machine.queued_semantic_events.is_empty());
}

#[test]
fn fl_c5_focus_semantic_recorded_semantic_manager_boundaries_keep_call_order() {
    let mut machine = scripted_listener_machine();
    assert_eq!(
        machine.semantic_manager_selection(),
        RuntimeSemanticManagerSelection::None
    );
    assert!(!machine.semantic_manager());
    assert!(machine.enable_semantics());
    assert!(!machine.enable_semantics());
    assert!(machine.semantic_manager());
    assert_eq!(
        machine.semantic_manager_selection(),
        RuntimeSemanticManagerSelection::InternalRecorded
    );
    assert_eq!(
        machine.semantic_manager_phase_trace,
        ["create-internal-recorded-seam", "build-tree-recorded-seam"]
    );
    assert!(
        !machine.fire_semantic_action(77, 0),
        "node lookup and SemanticData callbacks stop at their recorded seams"
    );

    machine.semantic_manager_phase_trace.clear();
    assert!(machine.set_external_semantic_manager(Some(90_930), Some(4)));
    assert_eq!(
        machine.semantic_manager_phase_trace,
        [
            "clean-tree-recorded-seam",
            "assign-external",
            "build-tree-recorded-seam",
        ]
    );
    machine.semantic_manager_phase_trace.clear();
    assert!(
        !machine.set_external_semantic_manager(Some(90_930), Some(9)),
        "same manager identity is a no-op even when the desired parent changes"
    );
    assert!(machine.semantic_manager_phase_trace.is_empty());

    assert!(machine.set_external_semantic_manager(None, None));
    assert_eq!(
        machine.semantic_manager_selection(),
        RuntimeSemanticManagerSelection::InternalRecorded
    );

    let mut without_internal = scripted_listener_machine();
    assert!(without_internal.set_external_semantic_manager(Some(90_931), None));
    without_internal.semantic_manager_phase_trace.clear();
    assert!(
        !without_internal.enable_semantics(),
        "an already-selected external manager suppresses internal creation"
    );
    assert!(
        without_internal.semantic_manager_phase_trace.is_empty(),
        "external-first enable does not create or rebuild an internal manager"
    );
    assert!(without_internal.set_external_semantic_manager(None, None));
    assert_eq!(
        without_internal.semantic_manager_selection(),
        RuntimeSemanticManagerSelection::None
    );
    assert!(!without_internal.semantic_manager());
    assert!(!without_internal.fire_semantic_action(77, 99));
}
