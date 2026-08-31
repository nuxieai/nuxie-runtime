use crate::mechanical_port::source::{
    animation::listener_invocation::ListenerInvocation, lua::rive_lua_libs::*, math::mat2d::Mat2D,
};

impl ScriptReffedArtboard {
    pub fn new(
        file: RuntimeFileWeakHandle,
        artboard: Box<ArtboardInstance>,
        view_model_instance: Option<CoreHandle>,
        parent_data_context: Option<Rc<DataContext>>,
        scripting_context: *mut dyn ScriptingContext,
    ) -> Self {
        let mut result = Self {
            file,
            artboard: Some(artboard),
            state_machine: None,
            view_model_instance,
            scripting_context,
        };
        result.state_machine = result.artboard_mut().default_state_machine();
        if result.view_model_instance.is_none() {
            let artboard = result.artboard();
            result.view_model_instance = result
                .file
                .with_file_mut(|file| file.create_view_model_instance(artboard))
                .flatten();
        }
        if let (Some(machine), Some(view_model)) = (
            result.state_machine.as_mut(),
            result.view_model_instance.as_ref(),
        ) {
            if let Some(parent) = parent_data_context {
                let mut context = DataContext::new(Some(view_model.clone()));
                context.set_parent(Some(parent));
                machine.bind_data_context(Rc::new(context));
            } else {
                machine.bind_view_model_instance(view_model.clone());
            }
        }
        if !result.scripting_context.is_null() {
            unsafe { &mut *result.scripting_context }
                .track_view_model_instance(result.view_model_instance.clone());
        }
        result
    }

    pub fn file(&self) -> RuntimeFileWeakHandle {
        self.file.clone()
    }

    pub fn artboard(&self) -> &Artboard {
        self.artboard.as_deref().unwrap()
    }

    pub fn artboard_mut(&mut self) -> &mut ArtboardInstance {
        self.artboard.as_deref_mut().unwrap()
    }

    pub fn state_machine(&self) -> Option<&StateMachineInstance> {
        self.state_machine.as_deref()
    }

    pub fn state_machine_mut(&mut self) -> Option<&mut StateMachineInstance> {
        self.state_machine.as_deref_mut()
    }
}

impl Drop for ScriptReffedArtboard {
    fn drop(&mut self) {
        if !self.scripting_context.is_null() {
            unsafe { &mut *self.scripting_context }
                .untrack_view_model_instance(self.view_model_instance.as_ref());
        }
        self.state_machine.take();
        self.artboard.take();
    }
}

impl ScriptedArtboard {
    pub fn new(
        state: &mut LuaState,
        file: RuntimeFileWeakHandle,
        artboard: Box<ArtboardInstance>,
        view_model: Option<CoreHandle>,
        data_context: Option<Rc<DataContext>>,
    ) -> Self {
        let scripting_context = state.thread_data::<dyn ScriptingContext>();
        Self {
            state: state.handle(),
            script_reffed_artboard: Some(Rc::new(ScriptReffedArtboard::new(
                file,
                artboard,
                view_model,
                data_context.clone(),
                scripting_context,
            ))),
            data_context,
            data_ref: 0,
        }
    }

    pub fn advance(&mut self, seconds: f32) -> bool {
        if let Some(machine) = self.state_machine_mut() {
            machine.advance_and_apply(seconds, false)
        } else {
            self.artboard_mut().advance(seconds)
        }
    }

    pub fn push_data(&mut self, state: &mut LuaState) -> i32 {
        if self.data_ref != 0 {
            state.raw_get_i(LuaState::REGISTRY_INDEX, self.data_ref);
            return 1;
        }
        if let Some(view_model) = self.view_model_instance() {
            let model = view_model
                .with(|instance| {
                    instance
                        .as_view_model_instance()
                        .and_then(ViewModelInstance::get_view_model)
                })
                .flatten();
            state.new_rive(ScriptedViewModel::new(state, model, Some(view_model)));
        } else {
            state.push_nil();
        }
        self.data_ref = state.reference(-1);
        1
    }

    pub fn instance(
        &mut self,
        state: &mut LuaState,
        view_model: Option<Rc<ViewModelInstance>>,
    ) -> i32 {
        let mut artboard = self.artboard_mut().instance();
        artboard.set_frame_origin(false);
        state.new_rive(ScriptedArtboard::new(
            state,
            self.script_reffed_artboard.file(),
            artboard,
            view_model,
            self.data_context.clone(),
        ));
        1
    }

    pub fn animation(&mut self, state: &mut LuaState, name: &str) -> i32 {
        if self.artboard().is_instance() {
            if let Some(animation) = self.artboard_mut().animation_named(name) {
                state.new_rive(ScriptedAnimation::new(state, animation));
                return 1;
            }
        }
        0
    }
}

impl Drop for ScriptedArtboard {
    fn drop(&mut self) {
        self.state.unref(self.data_ref);
        self.script_reffed_artboard.take();
    }
}

impl ScriptedAnimation {
    pub fn new(state: &mut LuaState, animation: Box<LinearAnimationInstance>) -> Self {
        Self {
            state: state.handle(),
            animation,
        }
    }

    pub fn duration(&self) -> f32 {
        self.animation.duration() as f32 / self.animation.fps() as f32
    }

    pub fn advance_from_lua(&mut self) -> i32 {
        let seconds = self.state.check_number(2) as f32;
        let advanced = self.animation.advance(seconds);
        self.animation.apply();
        self.state.push_boolean(advanced);
        1
    }

    pub fn set_time(&mut self, mode: &str) -> i32 {
        let seconds = match mode {
            "seconds" => self.state.check_number(2) as f32,
            "frames" => self.state.check_number(2) as f32 / self.animation.fps(),
            "percentage" => self.state.check_number(2) as f32 * self.duration(),
            _ => 0.0,
        };
        let local = self.animation.animation().global_to_local_seconds(seconds);
        self.animation.set_time(local);
        self.animation.apply();
        0
    }
}

impl ScriptedNode {
    pub fn new(artboard: Rc<ScriptReffedArtboard>, component: CoreHandle) -> Self {
        Self {
            artboard,
            component,
            shape_paint: None,
        }
    }

    pub fn shape_paint(&self) -> Option<CoreHandle> {
        self.shape_paint.clone().or_else(|| {
            self.component
                .with(|component| component.as_shape_paint().map(|_| self.component.clone()))
                .flatten()
        })
    }
}

fn artboard_draw(state: &mut LuaState) -> i32 {
    let (artboard, renderer) = state.rive2_mut::<ScriptedArtboard, ScriptedRenderer>();
    let renderer = renderer.validate(state);
    artboard.artboard_mut().draw_internal(renderer);
    0
}

fn apply_pointer_event(state: &mut LuaState, atom: LuaAtoms) -> i32 {
    let (artboard, event) = state.rive2_mut::<ScriptedArtboard, ScriptedPointerEvent>();
    let result = if let Some(machine) = artboard.state_machine_mut() {
        match atom {
            LuaAtoms::PointerDown => machine.pointer_down(event.position, event.id),
            LuaAtoms::PointerMove => machine.pointer_move(event.position, 0, event.id),
            LuaAtoms::PointerUp => machine.pointer_up(event.position, event.id),
            LuaAtoms::PointerExit => machine.pointer_exit(event.position, event.id),
            _ => 0,
        }
    } else {
        0
    };
    state.push_integer(result as i64);
    1
}

fn apply_gamepad_event(state: &mut LuaState, atom: LuaAtoms) -> i32 {
    let invocation = match atom {
        LuaAtoms::GamepadConnected => ListenerInvocation::gamepad_connected(
            &state.to_rive::<ScriptedGamepadConnected>(2).snapshot,
        ),
        LuaAtoms::GamepadEvent => {
            ListenerInvocation::gamepad_event(state.to_rive::<ScriptedGamepadEvent>(2).data.clone())
        }
        LuaAtoms::GamepadDisconnected => ListenerInvocation::gamepad_disconnected(
            state.to_rive::<ScriptedGamepadDisconnected>(2).device_id,
        ),
        _ => ListenerInvocation::none(),
    };
    let artboard = state.to_rive_mut::<ScriptedArtboard>(1);
    let result = if let Some(machine) = artboard.state_machine_mut() {
        let mut dispatched = None;
        machine
            .focus_manager_mut()
            .gamepad_dispatch(&invocation, &mut dispatched);
        machine.broadcast_gamepad_to_scripted_drawables(&invocation, dispatched)
    } else {
        0
    };
    state.push_integer(result as i64);
    1
}

fn artboard_namecall(state: &mut LuaState) -> i32 {
    let (name, atom) = state.namecall_atom();
    match atom {
        LuaAtoms::Draw => artboard_draw(state),
        LuaAtoms::Advance => {
            let seconds = state.check_number(2) as f32;
            let advanced = state.to_rive_mut::<ScriptedArtboard>(1).advance(seconds);
            state.push_boolean(advanced);
            1
        }
        LuaAtoms::Instance => {
            let view_model = if state.top() == 2 {
                Some(state.to_rive::<ScriptedViewModel>(2).view_model_instance())
            } else {
                None
            };
            state
                .to_rive_mut::<ScriptedArtboard>(1)
                .instance(state, view_model)
        }
        LuaAtoms::Animation => {
            let animation_name = state.check_string(2).to_owned();
            state
                .to_rive_mut::<ScriptedArtboard>(1)
                .animation(state, &animation_name)
        }
        LuaAtoms::AddToPath => {
            let transform = if state.top() == 3 {
                Some(state.to_rive::<ScriptedMat2D>(3).value)
            } else {
                None
            };
            let (artboard, path) = state.rive2_mut::<ScriptedArtboard, ScriptedPath>();
            artboard
                .artboard_mut()
                .add_to_raw_path(&mut path.raw_path, transform.as_ref());
            path.mark_dirty();
            0
        }
        LuaAtoms::Bounds => {
            let bounds = state.to_rive::<ScriptedArtboard>(1).artboard().bounds();
            state.push_vec2d(bounds.min());
            state.push_vec2d(bounds.max());
            2
        }
        LuaAtoms::Node => {
            let component_name = state.check_string(2).to_owned();
            let artboard = state.to_rive::<ScriptedArtboard>(1);
            if let Some(component) = artboard
                .artboard()
                .find_transform_component(&component_name)
            {
                state.new_rive(ScriptedNode::new(
                    artboard.script_reffed_artboard.clone(),
                    component,
                ));
            } else {
                state.push_nil();
            }
            1
        }
        LuaAtoms::PointerDown
        | LuaAtoms::PointerUp
        | LuaAtoms::PointerMove
        | LuaAtoms::PointerExit => apply_pointer_event(state, atom),
        LuaAtoms::GamepadEvent | LuaAtoms::GamepadConnected | LuaAtoms::GamepadDisconnected => {
            apply_gamepad_event(state, atom)
        }
        _ => state.error(format!(
            "{} is not a valid method of {}",
            name.unwrap_or_default(),
            ScriptedArtboard::LUA_NAME
        )),
    }
}

fn artboard_index(state: &mut LuaState) -> i32 {
    let (key, atom) = state.to_string_atom(2);
    if key.is_none() {
        return state.type_error(2, state.type_name(LuaType::String));
    }
    let artboard = state.to_rive_mut::<ScriptedArtboard>(1);
    match atom {
        LuaAtoms::FrameOrigin => state.push_boolean(artboard.artboard().frame_origin()),
        LuaAtoms::Width => state.push_number(artboard.artboard().width() as f64),
        LuaAtoms::Height => state.push_number(artboard.artboard().height() as f64),
        LuaAtoms::Data => return artboard.push_data(state),
        _ => {
            return state.error(format!(
                "'{}' is not a valid index of {}",
                key.unwrap_or_default(),
                ScriptedArtboard::LUA_NAME
            ));
        }
    }
    1
}

fn artboard_newindex(state: &mut LuaState) -> i32 {
    let (key, atom) = state.to_string_atom(2);
    if key.is_none() {
        return state.type_error(2, state.type_name(LuaType::String));
    }
    let artboard = state.to_rive_mut::<ScriptedArtboard>(1).artboard_mut();
    match atom {
        LuaAtoms::FrameOrigin => artboard.set_frame_origin(state.check_boolean(3)),
        LuaAtoms::Width => artboard.set_width(state.check_number(3) as f32),
        LuaAtoms::Height => artboard.set_height(state.check_number(3) as f32),
        _ => {
            return state.error(format!(
                "'{}' is not a valid index of {}",
                key.unwrap_or_default(),
                ScriptedArtboard::LUA_NAME
            ));
        }
    }
    0
}

fn node_index(state: &mut LuaState) -> i32 {
    let (key, atom) = state.to_string_atom(2);
    if key.is_none() {
        return state.type_error(2, state.type_name(LuaType::String));
    }
    let node = state.to_rive::<ScriptedNode>(1);
    let component = node.component();
    match atom {
        LuaAtoms::Position => state.push_vector2(component.x(), component.y()),
        LuaAtoms::Rotation => state.push_number(component.rotation() as f64),
        LuaAtoms::Scale => state.push_vector2(component.scale_x(), component.scale_y()),
        LuaAtoms::ScaleX => state.push_number(component.scale_x() as f64),
        LuaAtoms::ScaleY => state.push_number(component.scale_y() as f64),
        LuaAtoms::WorldTransform => state.new_rive(ScriptedMat2D::new(component.world_transform())),
        LuaAtoms::Children => {
            if let Some(container) = component.as_container_component() {
                state.create_table(container.children().len() as i32, 0);
                let mut index = 1;
                for child in container.children() {
                    if let Some(transform) = child.as_transform_component() {
                        state.new_rive(ScriptedNode::new(node.artboard(), transform));
                        state.raw_set_i(-2, index);
                        index += 1;
                    }
                }
            }
            return 1;
        }
        LuaAtoms::Parent => {
            if let Some(parent) = component
                .parent()
                .and_then(|value| value.as_transform_component())
            {
                state.new_rive(ScriptedNode::new(node.artboard(), parent));
            } else {
                state.push_nil();
            }
        }
        LuaAtoms::Paint => {
            if let Some(paint) = node.shape_paint() {
                state.new_rive(ScriptedPaintData::from_shape_paint(paint));
            } else {
                state.push_nil();
            }
        }
        _ if key.as_deref() == Some("x") => state.push_number(component.x() as f64),
        _ if key.as_deref() == Some("y") => state.push_number(component.y() as f64),
        _ => {
            return state.error(format!(
                "'{}' is not a valid index of {}",
                key.unwrap_or_default(),
                ScriptedNode::LUA_NAME
            ));
        }
    }
    1
}

fn node_newindex(state: &mut LuaState) -> i32 {
    let (key, atom) = state.to_string_atom(2);
    if key.is_none() {
        return state.type_error(2, state.type_name(LuaType::String));
    }
    let component = state.to_rive_mut::<ScriptedNode>(1).component_mut();
    match atom {
        LuaAtoms::Position => {
            let value = state.check_vector(3);
            if let Some(node) = component.as_node_mut() {
                node.set_x(value[0]);
                node.set_y(value[1]);
            } else if let Some(bone) = component.as_root_bone_mut() {
                bone.set_x(value[0]);
                bone.set_y(value[1]);
            }
        }
        LuaAtoms::Rotation => component.set_rotation(state.check_number(3) as f32),
        LuaAtoms::Scale => {
            let value = state.check_vector(3);
            component.set_scale_x(value[0]);
            component.set_scale_y(value[1]);
        }
        LuaAtoms::ScaleX => component.set_scale_x(state.check_number(3) as f32),
        LuaAtoms::ScaleY => component.set_scale_y(state.check_number(3) as f32),
        LuaAtoms::WorldTransform => {
            component.set_world_transform(state.to_rive::<ScriptedMat2D>(3).value)
        }
        _ if key.as_deref() == Some("x") => {
            let value = state.check_number(3) as f32;
            if let Some(node) = component.as_node_mut() {
                node.set_x(value);
            } else if let Some(bone) = component.as_root_bone_mut() {
                bone.set_x(value);
            }
        }
        _ if key.as_deref() == Some("y") => {
            let value = state.check_number(3) as f32;
            if let Some(node) = component.as_node_mut() {
                node.set_y(value);
            } else if let Some(bone) = component.as_root_bone_mut() {
                bone.set_y(value);
            }
        }
        _ => {
            return state.error(format!(
                "'{}' is not a valid index of {}",
                key.unwrap_or_default(),
                ScriptedNode::LUA_NAME
            ));
        }
    }
    0
}

fn node_namecall(state: &mut LuaState) -> i32 {
    let (name, atom) = state.namecall_atom();
    let node = state.to_rive_mut::<ScriptedNode>(1);
    match atom {
        LuaAtoms::Decompose => {
            let target = state.to_rive::<ScriptedMat2D>(2).value;
            let world = node.component().parent_world().invert_or_identity() * target;
            let decomposed = world.decompose();
            let component = node.component_mut();
            if let Some(value) = component.as_node_mut() {
                value.set_x(decomposed.x());
                value.set_y(decomposed.y());
            } else if let Some(value) = component.as_root_bone_mut() {
                value.set_x(decomposed.x());
                value.set_y(decomposed.y());
            }
            component.set_scale_x(decomposed.scale_x());
            component.set_scale_y(decomposed.scale_y());
            component.set_rotation(decomposed.rotation());
            0
        }
        LuaAtoms::AsPath => {
            if let Some(path) = node.component().as_path() {
                state.new_rive(ScriptedPathData::from_raw_path(path.raw_path()));
            } else {
                state.push_nil();
            }
            1
        }
        LuaAtoms::AsPaint => {
            if let Some(paint) = node.shape_paint() {
                state.new_rive(ScriptedPaintData::from_shape_paint(paint));
            } else {
                state.push_nil();
            }
            1
        }
        _ => state.error(format!(
            "{} is not a valid method of {}",
            name.unwrap_or_default(),
            ScriptedNode::LUA_NAME
        )),
    }
}

fn animation_index(state: &mut LuaState) -> i32 {
    let (key, atom) = state.to_string_atom(2);
    if atom == LuaAtoms::Duration {
        let duration = state.to_rive::<ScriptedAnimation>(1).duration();
        state.push_number(duration as f64);
        1
    } else {
        state.error(format!(
            "'{}' is not a valid index of {}",
            key.unwrap_or_default(),
            ScriptedNode::LUA_NAME
        ))
    }
}

fn animation_namecall(state: &mut LuaState) -> i32 {
    let (name, atom) = state.namecall_atom();
    let animation = state.to_rive_mut::<ScriptedAnimation>(1);
    match atom {
        LuaAtoms::Advance => animation.advance_from_lua(),
        LuaAtoms::SetTime => animation.set_time("seconds"),
        LuaAtoms::SetTimeFrames => animation.set_time("frames"),
        LuaAtoms::SetTimePercentage => animation.set_time("percentage"),
        _ => state.error(format!(
            "{} is not a valid method of {}",
            name.unwrap_or_default(),
            ScriptedNode::LUA_NAME
        )),
    }
}

fn register<T: LuaRive>(
    state: &mut LuaState,
    index: LuaFunction,
    newindex: Option<LuaFunction>,
    namecall: LuaFunction,
) {
    state.register_rive::<T>();
    state.push_function(index);
    state.set_field(-2, "__index");
    if let Some(newindex) = newindex {
        state.push_function(newindex);
        state.set_field(-2, "__newindex");
    }
    state.push_function(namecall);
    state.set_field(-2, "__namecall");
    state.set_readonly(-1, true);
    state.pop(1);
}

pub fn luaopen_rive_artboards(state: &mut LuaState) -> i32 {
    register::<ScriptedArtboard>(
        state,
        artboard_index,
        Some(artboard_newindex),
        artboard_namecall,
    );
    for field in ["width", "height", "frameOrigin"] {
        state.register_artboard_direct_field(field);
    }
    register::<ScriptedNode>(state, node_index, Some(node_newindex), node_namecall);
    for field in [
        "x", "y", "rotation", "scaleX", "scaleY", "position", "scale",
    ] {
        state.register_node_direct_field(field);
    }
    register::<ScriptedAnimation>(state, animation_index, None, animation_namecall);
    state.register_animation_direct_field("duration");
    0
}
