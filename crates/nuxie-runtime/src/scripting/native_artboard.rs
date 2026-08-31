//! Approved Rust scripting projection of `lua_artboards.cpp`'s retained owners.
//! All behavior is executed by the translated runtime, never the packed graph.

use super::*;
use crate::mechanical_port::source::{
    animation::{
        listener_invocation::{GamepadEventInvocation, ListenerInvocation},
        state_machine_instance::RuntimeStateMachineInstanceHandle,
    },
    artboard::{Artboard, RuntimeArtboardInstanceHandle},
    core::CoreHandle,
    data_bind::data_context::{DataContext, RuntimeDataContextHandle},
    file::RuntimeFileHandle,
    generated::{
        core_registry::{CoreField, CoreRegistry},
        layout_component_base::LayoutComponentBase,
    },
    input::{
        gamepad_snapshot::{
            GamepadInputChange, GamepadInputChangeKind, GamepadMappingKind, GamepadSnapshot,
        },
        standard_gamepad::{StandardGamepadAxis, StandardGamepadButton},
    },
    math::{mat2d::Mat2D, vec2d::Vec2D},
    renderer::{from_render_raw_path, to_render_raw_path},
};

// Field drop order mirrors ScriptReffedArtboard's explicit destructor: the
// machine can still access its artboard, and the artboard can still access File.
pub(super) struct NativeScriptArtboardOwner {
    machine: Option<RuntimeStateMachineInstanceHandle>,
    artboard: RuntimeArtboardInstanceHandle,
    view_model: Option<ScriptViewModel>,
    file: RuntimeFileHandle,
}

impl fmt::Debug for NativeScriptArtboardOwner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NativeScriptArtboardOwner")
            .finish_non_exhaustive()
    }
}

struct NativeScriptArtboard {
    owner: Rc<NativeScriptArtboardOwner>,
    parent_context: Option<RuntimeDataContextHandle>,
}

pub fn native_script_artboard(
    file: RuntimeFileHandle,
    artboard: RuntimeArtboardInstanceHandle,
    view_model: Option<CoreHandle>,
    parent_context: Option<RuntimeDataContextHandle>,
) -> Result<Box<dyn ScriptArtboard>, ScriptError> {
    let machine = artboard.default_state_machine_handle();
    let view_model = view_model.or_else(|| {
        file.with_file(|file| file.create_view_model_instance_for_artboard(artboard.core_handle()))
    });
    if let (Some(machine), Some(instance)) = (&machine, &view_model) {
        if let Some(parent) = &parent_context {
            let mut context = DataContext::new(Some(instance.clone()));
            context.set_parent(Some(parent.clone()));
            machine.with_instance_mut(|machine| {
                machine.bind_data_context_handle(RuntimeDataContextHandle::new(context))
            });
        } else {
            machine.with_instance_mut(|machine| {
                machine.bind_view_model_instance_handle(instance.clone())
            });
        }
    }
    let view_model = view_model.map(|instance| {
        ScriptViewModel::from_native(instance, file.clone())
            .expect("a File-created or supplied view model retains its definition")
    });
    Ok(Box::new(NativeScriptArtboard {
        owner: Rc::new(NativeScriptArtboardOwner {
            machine,
            artboard,
            view_model,
            file,
        }),
        parent_context,
    }))
}

impl NativeScriptArtboard {
    fn native_animation<'a>(
        &self,
        animation: &'a ScriptAnimation,
    ) -> Result<&'a Rc<RefCell<NativeLinearAnimation>>, ScriptError> {
        Ok(&animation.instance)
    }
}

impl ScriptArtboard for NativeScriptArtboard {
    fn retained_handle(&self) -> Box<dyn ScriptArtboard> {
        Box::new(Self {
            owner: self.owner.clone(),
            parent_context: self.parent_context.clone(),
        })
    }
    fn width(&self) -> f32 {
        self.owner
            .artboard
            .with_artboard(|artboard| artboard.base.width())
    }
    fn height(&self) -> f32 {
        self.owner
            .artboard
            .with_artboard(|artboard| artboard.base.height())
    }
    fn frame_origin(&self) -> bool {
        self.owner
            .artboard
            .with_artboard(|artboard| artboard.base.frame_origin())
    }
    fn set_width(&mut self, value: f32) {
        CoreRegistry::set_double_handle(
            &self.owner.artboard.core_handle(),
            LayoutComponentBase::WIDTH_PROPERTY_KEY.into(),
            value,
        );
    }
    fn set_height(&mut self, value: f32) {
        CoreRegistry::set_double_handle(
            &self.owner.artboard.core_handle(),
            LayoutComponentBase::HEIGHT_PROPERTY_KEY.into(),
            value,
        );
    }
    fn set_frame_origin(&mut self, value: bool) {
        self.owner
            .artboard
            .with_artboard_mut(|artboard| artboard.base.set_frame_origin(value));
    }
    fn data(&self) -> Option<ScriptViewModel> {
        self.owner.view_model.clone()
    }

    fn instance(
        &self,
        view_model: Option<ScriptViewModel>,
    ) -> Result<Box<dyn ScriptArtboard>, ScriptError> {
        let view_model = view_model
            .map(|model| {
                model.native_instance().ok_or_else(|| {
                    ScriptError::new("script artboard requires an instantiated native view model")
                })
            })
            .transpose()?;
        let artboard = Artboard::instance_from_handle(&self.owner.artboard.core_handle())
            .ok_or_else(|| ScriptError::new("script artboard instance initialization failed"))?;
        artboard.with_artboard_mut(|artboard| artboard.base.set_frame_origin(false));
        native_script_artboard(
            self.owner.file.clone(),
            artboard,
            view_model,
            self.parent_context.clone(),
        )
    }

    fn advance(&mut self, seconds: f32) -> Result<bool, ScriptError> {
        Ok(match &self.owner.machine {
            Some(machine) => machine.advance_and_apply_view_models(seconds, false),
            None => self.owner.artboard.advance_default(seconds),
        })
    }

    fn animation(&self, name: &str) -> Result<Option<ScriptAnimation>, ScriptError> {
        Ok(self
            .owner
            .artboard
            .animation_named(name)
            .map(|instance| ScriptAnimation {
                instance: Rc::new(RefCell::new(*instance)),
            }))
    }

    fn advance_animation(
        &mut self,
        animation: &mut ScriptAnimation,
        seconds: f32,
    ) -> Result<bool, ScriptError> {
        let instance = self.native_animation(animation)?;
        // The one-argument C++ advance passes a null callback reporter. It does
        // not advance the artboard or consume the view model after applying.
        let advanced = instance.borrow_mut().advance(seconds, None);
        instance.borrow().apply(1.0);
        Ok(advanced)
    }

    fn set_animation_time(
        &mut self,
        animation: &mut ScriptAnimation,
        value: f32,
        mode: ScriptAnimationTime,
    ) -> Result<(), ScriptError> {
        let mut instance = self.native_animation(animation)?.borrow_mut();
        let seconds = match mode {
            ScriptAnimationTime::Seconds => value,
            ScriptAnimationTime::Frames => value / instance.fps() as f32,
            ScriptAnimationTime::Percentage => {
                value * (instance.duration() as f32 / instance.fps() as f32)
            }
        };
        let local = instance.global_to_local_seconds(seconds);
        instance.set_time(local);
        drop(instance);
        self.native_animation(animation)?.borrow().apply(1.0);
        Ok(())
    }

    fn node(&self, name: &str) -> Result<Option<ScriptNode>, ScriptError> {
        // Release the root before inspecting objects: object zero is this exact
        // root, and TransformComponent matching includes all derived owners.
        let objects = self
            .owner
            .artboard
            .with_artboard(|artboard| artboard.base.objects().to_vec());
        let component = objects.into_iter().flatten().find(|object| {
            object
                .with(|object| object.as_transform_component().is_some())
                .unwrap_or(false)
                && object
                    .with_mut(|object| object.get_string(CoreField::ComponentName) == name)
                    .unwrap_or(false)
        });
        Ok(component.map(|component| {
            let mut node = ScriptNode::from_component(component);
            node.live.as_mut().unwrap().artboard_owner = Some(self.owner.clone());
            node
        }))
    }

    fn bounds(&self) -> nuxie_render_api::Aabb {
        let bounds = self
            .owner
            .artboard
            .with_artboard(|artboard| artboard.base.bounds());
        nuxie_render_api::Aabb::new(bounds.min_x, bounds.min_y, bounds.max_x, bounds.max_y)
    }

    fn add_to_path(
        &mut self,
        path: &mut RawPath,
        transform: Option<nuxie_render_api::Mat2D>,
    ) -> Result<(), ScriptError> {
        let mut raw = from_render_raw_path(path);
        let transform = transform.map(|value| {
            let [a, b, c, d, x, y] = value.0;
            Mat2D::new(a, b, c, d, x, y)
        });
        Artboard::add_to_raw_path_handle(
            &self.owner.artboard.core_handle(),
            &mut raw,
            transform.as_ref(),
        );
        *path = to_render_raw_path(&raw);
        Ok(())
    }

    fn draw(
        &mut self,
        _factory: &mut dyn RenderFactory,
        renderer: &mut dyn Renderer,
    ) -> Result<(), ScriptError> {
        self.owner.artboard.draw_internal(renderer);
        Ok(())
    }

    fn dispatch_input(
        &mut self,
        method: ScriptMethod,
        invocation: &ScriptListenerInvocation,
    ) -> Result<u32, ScriptError> {
        let Some(machine) = &self.owner.machine else {
            return Ok(0);
        };
        if let ScriptListenerInvocation::Pointer {
            pointer_id, x, y, ..
        } = invocation
        {
            let position = Vec2D::new(*x, *y);
            return machine.with_instance_mut(|machine| {
                Ok(match method {
                    ScriptMethod::PointerDown => machine.pointer_down(position, *pointer_id),
                    ScriptMethod::PointerMove => machine.pointer_move(position, 0.0, *pointer_id),
                    ScriptMethod::PointerUp => machine.pointer_up(position, *pointer_id),
                    ScriptMethod::PointerExit => machine.pointer_exit(position, *pointer_id),
                    _ => return Err(ScriptError::new("invalid artboard pointer method")),
                } as u32)
            });
        }
        let invocation = gamepad_invocation(invocation)?;
        let focus = machine.with_instance(|machine| machine.focus_manager());
        let mut dispatched = None;
        focus.with_focus_manager_mut(|focus| {
            focus.gamepad_dispatch(&invocation, Some(&mut dispatched));
        });
        Ok(
            machine.broadcast_gamepad_to_scripted_drawables(&invocation, dispatched.as_ref())
                as u32,
        )
    }
}

fn gamepad_invocation(
    invocation: &ScriptListenerInvocation,
) -> Result<ListenerInvocation, ScriptError> {
    use crate::{ScriptGamepadInputChange, ScriptGamepadMappingKind, ScriptGamepadSnapshot};
    fn snapshot(value: &ScriptGamepadSnapshot) -> GamepadSnapshot {
        GamepadSnapshot {
            device_id: value.device_id,
            button_mask: value.button_mask,
            button_values: value.button_values.clone(),
            axes: value.axes.clone(),
            mapping: match value.mapping {
                ScriptGamepadMappingKind::Standard => GamepadMappingKind::Standard,
                ScriptGamepadMappingKind::Unknown => GamepadMappingKind::Unknown,
            },
        }
    }
    Ok(match invocation {
        ScriptListenerInvocation::GamepadConnected { snapshot: value } => {
            ListenerInvocation::gamepad_connected(&snapshot(value))
        }
        ScriptListenerInvocation::GamepadDisconnected { device_id } => {
            ListenerInvocation::gamepad_disconnected(*device_id)
        }
        ScriptListenerInvocation::GamepadEvent {
            full_state,
            change,
            standard_button_intent,
            standard_axis_intent,
        } => ListenerInvocation::gamepad_event(GamepadEventInvocation {
            full_state: snapshot(full_state),
            change: match *change {
                ScriptGamepadInputChange::Button { index, value } => GamepadInputChange {
                    kind: GamepadInputChangeKind::Button,
                    index,
                    value,
                },
                ScriptGamepadInputChange::Axis { index, value } => GamepadInputChange {
                    kind: GamepadInputChangeKind::Axis,
                    index,
                    value,
                },
            },
            standard_button: standard_button_intent
                .and_then(|value| u8::try_from(value).ok())
                .and_then(StandardGamepadButton::from_raw),
            standard_axis: standard_axis_intent
                .and_then(|value| u8::try_from(value).ok())
                .and_then(StandardGamepadAxis::from_raw),
        }),
        _ => return Err(ScriptError::new("expected an artboard gamepad event")),
    })
}
