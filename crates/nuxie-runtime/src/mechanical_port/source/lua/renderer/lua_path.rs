use crate::mechanical_port::source::{
    artboard::Artboard,
    lua::rive_lua_libs::*,
    math::{mat2d::Mat2D, path_verb::PathVerb, vec2d::Vec2D},
    renderer::FillRule,
};

impl ScriptedPathData {
    pub fn render_path(&mut self, state: &mut LuaState) -> &mut RenderPath {
        if self.is_render_path_dirty {
            self.is_render_path_dirty = false;
            let frame_id = Artboard::frame_id();
            let same_frame_rebuild = self.render_path.is_some() && self.render_frame_id == frame_id;
            self.render_frame_id = frame_id;
            if self.render_path.is_none() || same_frame_rebuild {
                let mut path = state
                    .thread_data::<dyn ScriptingContext>()
                    .factory()
                    .make_empty_render_path();
                path.set_fill_rule(FillRule::Clockwise);
                self.render_path = Some(path);
            } else {
                self.render_path.as_mut().unwrap().rewind();
            }
            self.render_path
                .as_mut()
                .unwrap()
                .add_raw_path(&self.raw_path);
        }
        self.render_path.as_mut().unwrap()
    }

    pub fn from_raw_path(path: &RawPath) -> Self {
        let mut result = Self::new();
        result.raw_path.add_path(path, None);
        result
    }

    pub fn total_commands(&self) -> i32 {
        self.raw_path.verbs().len() as i32
    }
}

fn path_new(state: &mut LuaState) -> i32 {
    state.new_rive(ScriptedPath::new());
    1
}

fn path_move_to(state: &mut LuaState) -> i32 {
    let point = *state.check_vec2d(2);
    let path = state.to_rive_mut::<ScriptedPath>(1);
    path.raw_path.move_to(point);
    path.mark_dirty();
    0
}

fn path_line_to(state: &mut LuaState) -> i32 {
    let point = *state.check_vec2d(2);
    let path = state.to_rive_mut::<ScriptedPath>(1);
    path.raw_path.line_to(point);
    path.mark_dirty();
    0
}

fn path_quad_to(state: &mut LuaState) -> i32 {
    let first = *state.check_vec2d(2);
    let second = *state.check_vec2d(3);
    let path = state.to_rive_mut::<ScriptedPath>(1);
    path.raw_path.quad_to(first, second);
    path.mark_dirty();
    0
}

fn path_cubic_to(state: &mut LuaState) -> i32 {
    let first = *state.check_vec2d(2);
    let second = *state.check_vec2d(3);
    let third = *state.check_vec2d(4);
    let path = state.to_rive_mut::<ScriptedPath>(1);
    path.raw_path.cubic_to(first, second, third);
    path.mark_dirty();
    0
}

fn path_close(state: &mut LuaState) -> i32 {
    let path = state.to_rive_mut::<ScriptedPath>(1);
    path.raw_path.close();
    path.mark_dirty();
    0
}

fn path_reset(state: &mut LuaState) -> i32 {
    let path = state.to_rive_mut::<ScriptedPath>(1);
    path.raw_path.reset();
    path.mark_dirty();
    0
}

fn path_add(state: &mut LuaState) -> i32 {
    let transform = if state.top() == 3 {
        Some(state.to_rive::<ScriptedMat2D>(3).value)
    } else {
        None
    };
    let (path, other) = state.rive2_mut::<ScriptedPathData, ScriptedPathData>();
    path.raw_path.add_path(&other.raw_path, transform.as_ref());
    path.mark_dirty();
    0
}

fn path_command(state: &mut LuaState) -> i32 {
    let path = state.to_rive::<ScriptedPath>(1);
    let verb_index = state.check_number(2) as i32;
    let verbs = path.raw_path.verbs();
    let points = path.raw_path.points();
    let mut name = "none";
    let mut verb_points = Vec::new();
    if verb_index >= 0 && verb_index < verbs.len() as i32 {
        let verb = verbs[verb_index as usize];
        let point_index: usize = verbs[..verb_index as usize]
            .iter()
            .map(|verb| path_verb_to_point_count(*verb))
            .sum();
        match verb {
            PathVerb::Move => {
                name = "moveTo";
                if point_index < points.len() {
                    verb_points.push(points[point_index]);
                }
            }
            PathVerb::Line => {
                name = "lineTo";
                if point_index < points.len() {
                    verb_points.push(points[point_index]);
                }
            }
            PathVerb::Quad => {
                name = "quadTo";
                if point_index + 1 < points.len() {
                    verb_points.extend_from_slice(&points[point_index..=point_index + 1]);
                }
            }
            PathVerb::Cubic => {
                name = "cubicTo";
                if point_index + 2 < points.len() {
                    verb_points.extend_from_slice(&points[point_index..=point_index + 2]);
                }
            }
            PathVerb::Close => name = "close",
        }
    }
    state.new_rive(ScriptedPathCommand::new(name, verb_points));
    1
}

fn path_contours(state: &mut LuaState) -> i32 {
    let path = state.to_rive::<ScriptedPath>(1);
    let mut iterator = RefCntContourMeasureIter::new(path.raw_path.clone());
    if let Some(first) = iterator.next() {
        state.new_rive(ScriptedContourMeasure::new(first, Some(iterator)));
    } else {
        state.push_nil();
    }
    1
}

fn path_measure(state: &mut LuaState) -> i32 {
    let path = state.to_rive::<ScriptedPath>(1);
    state.new_rive(ScriptedPathMeasure::new(PathMeasure::new(&path.raw_path)));
    1
}

fn path_namecall(state: &mut LuaState) -> i32 {
    let (name, atom) = state.namecall_atom();
    match atom {
        LuaAtoms::MoveTo => path_move_to(state),
        LuaAtoms::LineTo => path_line_to(state),
        LuaAtoms::QuadTo => path_quad_to(state),
        LuaAtoms::CubicTo => path_cubic_to(state),
        LuaAtoms::Close => path_close(state),
        LuaAtoms::Reset => path_reset(state),
        LuaAtoms::Add => path_add(state),
        LuaAtoms::Contours => path_contours(state),
        LuaAtoms::Measure => path_measure(state),
        _ => state.error(format!(
            "{} is not a valid method of {}",
            name.unwrap_or_default(),
            ScriptedPath::LUA_NAME
        )),
    }
}

fn contour_measure_position_and_tangent(state: &mut LuaState) -> i32 {
    let distance = state.check_number(2) as f32;
    let value = state
        .to_rive_mut::<ScriptedContourMeasure>(1)
        .measure_mut()
        .get_pos_tan(distance);
    state.push_vec2d(value.position);
    state.push_vec2d(value.tangent);
    2
}

fn contour_measure_warp(state: &mut LuaState) -> i32 {
    let source = *state.check_vec2d(2);
    let result = state
        .to_rive_mut::<ScriptedContourMeasure>(1)
        .measure_mut()
        .warp(source);
    state.push_vec2d(result);
    1
}

fn contour_measure_extract(state: &mut LuaState) -> i32 {
    let start = state.check_number(2) as f32;
    let end = state.check_number(3) as f32;
    let start_with_move = if state.is_boolean(5) {
        state.to_boolean(5)
    } else {
        true
    };
    let (measure, destination) = state.rive2_at_mut::<ScriptedContourMeasure, ScriptedPath>(1, 4);
    measure
        .measure_mut()
        .get_segment(start, end, &mut destination.raw_path, start_with_move);
    destination.mark_dirty();
    0
}

fn contour_measure_next(state: &mut LuaState) -> i32 {
    let scripted = state.to_rive_mut::<ScriptedContourMeasure>(1);
    if let Some(iterator) = scripted.iterator_mut() {
        if let Some(next) = iterator.next() {
            state.new_rive(ScriptedContourMeasure::new(next, Some(iterator.clone())));
            return 1;
        }
    }
    state.push_nil();
    1
}

fn path_index(state: &mut LuaState) -> i32 {
    let (key, _) = state.to_string_atom(2);
    if key.is_none() {
        let index = state.check_integer(2);
        state.push_integer(index - 1);
        state.replace(2);
        return path_command(state);
    }
    0
}

fn path_length(state: &mut LuaState) -> i32 {
    let path = state.to_rive::<ScriptedPath>(1);
    state.push_number(path.total_commands() as f64);
    1
}

fn contour_measure_index(state: &mut LuaState) -> i32 {
    let (key, atom) = state.to_string_atom(2);
    if key.is_none() {
        return state.type_error(2, state.type_name(LuaType::String));
    }
    let measure = state.to_rive::<ScriptedContourMeasure>(1);
    match atom {
        LuaAtoms::Length => state.push_number(measure.measure().length() as f64),
        LuaAtoms::IsClosed => state.push_boolean(measure.measure().is_closed()),
        LuaAtoms::Next => return contour_measure_next(state),
        _ => return 0,
    }
    1
}

fn contour_measure_namecall(state: &mut LuaState) -> i32 {
    let (name, atom) = state.namecall_atom();
    match atom {
        LuaAtoms::PositionAndTangent => contour_measure_position_and_tangent(state),
        LuaAtoms::Warp => contour_measure_warp(state),
        LuaAtoms::Extract => contour_measure_extract(state),
        _ => state.error(format!(
            "{} is not a valid method of {}",
            name.unwrap_or_default(),
            ScriptedContourMeasure::LUA_NAME
        )),
    }
}

fn path_measure_index(state: &mut LuaState) -> i32 {
    let (key, atom) = state.to_string_atom(2);
    if key.is_none() {
        return state.type_error(2, state.type_name(LuaType::String));
    }
    let measure = state.to_rive::<ScriptedPathMeasure>(1).measure();
    match atom {
        LuaAtoms::Length => state.push_number(measure.length() as f64),
        LuaAtoms::IsClosed => state.push_boolean(measure.is_closed()),
        _ => return 0,
    }
    1
}

fn path_measure_position_and_tangent(state: &mut LuaState) -> i32 {
    let distance = state.check_number(2) as f32;
    let value = state
        .to_rive::<ScriptedPathMeasure>(1)
        .measure()
        .at_distance(distance);
    state.push_vec2d(value.position);
    state.push_vec2d(value.tangent);
    2
}

fn path_measure_warp(state: &mut LuaState) -> i32 {
    let source = *state.check_vec2d(2);
    let value = state
        .to_rive::<ScriptedPathMeasure>(1)
        .measure()
        .at_distance(source.x);
    state.push_vec2d(Vec2D::new(
        value.position.x - value.tangent.y * source.y,
        value.position.y + value.tangent.x * source.y,
    ));
    1
}

fn path_measure_extract(state: &mut LuaState) -> i32 {
    let start = state.check_number(2) as f32;
    let end = state.check_number(3) as f32;
    let start_with_move = if state.is_boolean(5) {
        state.to_boolean(5)
    } else {
        true
    };
    let (measure, destination) = state.rive2_at_mut::<ScriptedPathMeasure, ScriptedPath>(1, 4);
    measure
        .measure_mut()
        .get_segment(start, end, &mut destination.raw_path, start_with_move);
    destination.mark_dirty();
    0
}

fn path_measure_namecall(state: &mut LuaState) -> i32 {
    let (name, atom) = state.namecall_atom();
    match atom {
        LuaAtoms::PositionAndTangent => path_measure_position_and_tangent(state),
        LuaAtoms::Warp => path_measure_warp(state),
        LuaAtoms::Extract => path_measure_extract(state),
        _ => state.error(format!(
            "{} is not a valid method of {}",
            name.unwrap_or_default(),
            ScriptedPathMeasure::LUA_NAME
        )),
    }
}

fn path_command_index(state: &mut LuaState) -> i32 {
    let (key, atom) = state.to_string_atom(2);
    let command = state.to_rive::<ScriptedPathCommand>(1);
    if key.is_none() {
        let index = state.check_integer(2) - 1;
        if index >= 0 && (index as usize) < command.points().len() {
            state.push_vec2d(command.points()[index as usize]);
            return 1;
        }
        return 0;
    }
    if atom == LuaAtoms::Type {
        state.push_string(command.command_type());
        return 1;
    }
    state.error(format!(
        "'{}' is not a valid index of PathCommand",
        key.unwrap_or_default()
    ))
}

fn path_command_length(state: &mut LuaState) -> i32 {
    let command = state.to_rive::<ScriptedPathCommand>(1);
    state.push_number(command.points().len() as f64);
    1
}

fn path_command_namecall(state: &mut LuaState) -> i32 {
    let (name, _) = state.namecall_atom();
    state.error(format!(
        "{} is not a valid method of {}",
        name.unwrap_or_default(),
        ScriptedPathCommand::LUA_NAME
    ))
}

fn register_metatable<T: LuaRive>(
    state: &mut LuaState,
    index: LuaFunction,
    length: Option<LuaFunction>,
    namecall: LuaFunction,
) {
    state.register_rive::<T>();
    state.push_function(index);
    state.set_field(-2, "__index");
    if let Some(length) = length {
        state.push_function(length);
        state.set_field(-2, "__len");
    }
    state.push_function(namecall);
    state.set_field(-2, "__namecall");
    state.set_readonly(-1, true);
    state.pop(1);
}

pub fn luaopen_rive_path(state: &mut LuaState) -> i32 {
    register_metatable::<ScriptedPathData>(state, path_index, Some(path_length), path_namecall);
    state.register(
        ScriptedPath::LUA_NAME,
        &[LuaReg::new("new", path_new), LuaReg::END],
    );
    register_metatable::<ScriptedPath>(state, path_index, Some(path_length), path_namecall);
    register_metatable::<ScriptedPathCommand>(
        state,
        path_command_index,
        Some(path_command_length),
        path_command_namecall,
    );
    register_metatable::<ScriptedContourMeasure>(
        state,
        contour_measure_index,
        None,
        contour_measure_namecall,
    );
    state.register_userdata_direct_number_field::<ScriptedContourMeasure>("length", |value| {
        value.measure().length()
    });
    state.register_userdata_direct_boolean_field::<ScriptedContourMeasure>("isClosed", |value| {
        value.measure().is_closed()
    });
    register_metatable::<ScriptedPathMeasure>(
        state,
        path_measure_index,
        None,
        path_measure_namecall,
    );
    state.register_userdata_direct_number_field::<ScriptedPathMeasure>("length", |value| {
        value.measure().length()
    });
    state.register_userdata_direct_boolean_field::<ScriptedPathMeasure>("isClosed", |value| {
        value.measure().is_closed()
    });
    1
}
