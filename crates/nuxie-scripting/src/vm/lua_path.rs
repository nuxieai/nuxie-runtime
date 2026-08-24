// Translated from:
// /Users/levi/dev/oss/rive-runtime/src/lua/renderer/lua_path.cpp
use std::rc::Rc;

use luaur_rt::{
    AnyUserData, Error, Lua, MultiValue, Result, Table, UserData, UserDataFields, UserDataMethods,
    Value, Vector as LuaVector,
};
use luaur_vm::functions::lua_getmetatable::lua_getmetatable;
use nuxie_render_api::{
    Factory as RenderFactory, FillRule, Mat2D, PathVerb, RawPath, RenderPath, Vec2D,
};
use nuxie_runtime::{
    RuntimeContourMeasure, RuntimePathMeasure, ScriptNode, artboard_draw_frame_id,
    runtime_path_commands_from_raw_path,
};

use super::lua_artboards::ScriptedNode as LuaScriptedNode;
use super::lua_mat2d::{ScriptedMat2D, number_arg};

pub(super) struct ScriptedPath {
    pub(super) raw_path: RawPath,
    dirty: bool,
    render_path: Option<Box<dyn RenderPath>>,
    render_frame_id: u64,
}

impl ScriptedPath {
    fn new() -> Self {
        Self {
            raw_path: RawPath::new(),
            dirty: true,
            render_path: None,
            render_frame_id: 0,
        }
    }

    pub(super) fn from_raw_path(raw_path: RawPath) -> Self {
        Self {
            raw_path,
            dirty: true,
            render_path: None,
            render_frame_id: 0,
        }
    }

    fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    pub(super) fn render_path(&mut self, factory: &mut dyn RenderFactory) -> &dyn RenderPath {
        if self.dirty {
            self.dirty = false;
            // Pinned lua_path.cpp replaces a path rebuilt in the same frame:
            // the renderer may retain the first backend path until submission.
            let render_frame_id = artboard_draw_frame_id();
            let same_frame_rebuild =
                self.render_path.is_some() && self.render_frame_id == render_frame_id;
            self.render_frame_id = render_frame_id;
            if self.render_path.is_none() || same_frame_rebuild {
                let mut path = factory.make_empty_render_path();
                path.fill_rule(FillRule::Clockwise);
                self.render_path = Some(path);
            } else if let Some(path) = self.render_path.as_mut() {
                path.rewind();
            }
            self.render_path
                .as_mut()
                .expect("render path is initialized")
                .add_raw_path(&self.raw_path);
        }
        self.render_path
            .as_ref()
            .expect("render path is initialized")
            .as_ref()
    }

    fn commands(&self) -> Vec<nuxie_runtime::RuntimePathCommand> {
        runtime_path_commands_from_raw_path(&self.raw_path)
    }

    fn command(&self, lua_index: i64) -> ScriptedPathCommand {
        let Some(verb_index) = lua_index
            .checked_sub(1)
            .and_then(|index| usize::try_from(index).ok())
        else {
            return ScriptedPathCommand::none();
        };
        let Some(verb) = self.raw_path.verbs().get(verb_index).copied() else {
            return ScriptedPathCommand::none();
        };

        let point_index = self.raw_path.verbs()[..verb_index]
            .iter()
            .map(|verb| path_verb_point_count(*verb))
            .sum::<usize>();
        let point_count = path_verb_point_count(verb);
        let points = self
            .raw_path
            .points()
            .get(point_index..point_index + point_count)
            .map_or_else(Vec::new, <[Vec2D]>::to_vec);

        ScriptedPathCommand {
            command_type: match verb {
                PathVerb::Move => "moveTo",
                PathVerb::Line => "lineTo",
                PathVerb::Quad => "quadTo",
                PathVerb::Cubic => "cubicTo",
                PathVerb::Close => "close",
            },
            points,
        }
    }
}

fn path_verb_point_count(verb: PathVerb) -> usize {
    match verb {
        PathVerb::Move | PathVerb::Line => 1,
        PathVerb::Quad => 2,
        PathVerb::Cubic => 3,
        PathVerb::Close => 0,
    }
}

struct ScriptedPathCommand {
    command_type: &'static str,
    points: Vec<Vec2D>,
}

impl ScriptedPathCommand {
    fn none() -> Self {
        Self {
            command_type: "none",
            points: Vec::new(),
        }
    }
}

impl UserData for ScriptedPathCommand {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_meta_method("__len", |_, this, ()| Ok(this.points.len()));
    }
}

fn userdata_metatable(lua: &Lua, userdata: AnyUserData) -> Result<Table> {
    // luaur-rt intentionally keeps userdata metatables private. We only need
    // the table it just created so Path can layer C++'s numeric indexing over
    // luaur's ordinary method table without replacing its typed userdata cell.
    unsafe {
        lua.exec_raw(userdata, |state| {
            lua_getmetatable(state, 1);
        })
    }
}

fn lua_path_index(key: &Value) -> Result<Option<i64>> {
    match key {
        Value::Integer(index) => Ok(Some(*index)),
        // C++ uses luaL_checkinteger, whose Luau implementation converts the
        // number to an integer rather than requiring a mathematically integral
        // value.
        Value::Number(index) => Ok(Some(*index as i64)),
        Value::String(_) => Ok(None),
        _ => Err(Error::runtime("Path index must be a string or number")),
    }
}

pub(super) fn create_scripted_path(lua: &Lua, path: ScriptedPath) -> Result<AnyUserData> {
    let userdata = lua.create_userdata(path)?;
    let metatable = userdata_metatable(lua, userdata.clone())?;
    if !metatable.is_readonly() {
        let methods: Table = metatable.get("__index")?;
        let index = lua.create_function(move |lua, (userdata, key): (AnyUserData, Value)| {
            match lua_path_index(&key)? {
                Some(index) => {
                    let command = {
                        let path = userdata.borrow::<ScriptedPath>()?;
                        path.command(index)
                    };
                    create_scripted_path_command(lua, command).map(Value::UserData)
                }
                None => methods.get(key),
            }
        })?;
        metatable.set("__index", index)?;
        metatable.set_readonly(true);
    }
    Ok(userdata)
}

fn create_scripted_path_command(lua: &Lua, command: ScriptedPathCommand) -> Result<AnyUserData> {
    let userdata = lua.create_userdata(command)?;
    let metatable = userdata_metatable(lua, userdata.clone())?;
    if !metatable.is_readonly() {
        let index = lua.create_function(|_, (userdata, key): (AnyUserData, Value)| {
            let command = userdata.borrow::<ScriptedPathCommand>()?;
            match key {
                Value::Integer(index) => command
                    .points
                    .get(index.saturating_sub(1) as usize)
                    .map_or(Ok(Value::Nil), |point| {
                        Ok(Value::Vector(LuaVector::new(point.x, point.y, 0.0)))
                    }),
                Value::Number(index) => command
                    .points
                    .get((index as i64).saturating_sub(1) as usize)
                    .map_or(Ok(Value::Nil), |point| {
                        Ok(Value::Vector(LuaVector::new(point.x, point.y, 0.0)))
                    }),
                Value::String(name) if name.as_bytes() == b"type" => Ok(Value::String(
                    userdata.lua().create_string(command.command_type),
                )),
                Value::String(name) => Err(Error::runtime(format!(
                    "'{}' is not a valid index of PathCommand",
                    name.to_string_lossy()
                ))),
                _ => Err(Error::runtime(
                    "PathCommand index must be a string or number",
                )),
            }
        })?;
        metatable.set("__index", index)?;
        metatable.set_readonly(true);
    }
    Ok(userdata)
}

impl UserData for ScriptedPath {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_meta_method("__len", |_, this, ()| Ok(this.raw_path.verbs().len()));
        methods.add_method_mut("moveTo", |_, this, point: LuaVector| {
            this.raw_path.move_to(point.x(), point.y());
            this.mark_dirty();
            Ok(())
        });
        methods.add_method_mut("lineTo", |_, this, point: LuaVector| {
            this.raw_path.line_to(point.x(), point.y());
            this.mark_dirty();
            Ok(())
        });
        methods.add_method_mut(
            "quadTo",
            |_, this, (control, point): (LuaVector, LuaVector)| {
                this.raw_path
                    .quad_to(control.x(), control.y(), point.x(), point.y());
                this.mark_dirty();
                Ok(())
            },
        );
        methods.add_method_mut(
            "cubicTo",
            |_, this, (out, inn, point): (LuaVector, LuaVector, LuaVector)| {
                this.raw_path
                    .cubic_to(out.x(), out.y(), inn.x(), inn.y(), point.x(), point.y());
                this.mark_dirty();
                Ok(())
            },
        );
        methods.add_method_mut("close", |_, this, ()| {
            this.raw_path.close();
            this.mark_dirty();
            Ok(())
        });
        methods.add_method_mut("reset", |_, this, ()| {
            this.raw_path.rewind();
            this.mark_dirty();
            Ok(())
        });
        methods.add_method_mut("add", |_, this, args: MultiValue| {
            let Some(Value::UserData(path)) = args.front() else {
                return Err(Error::runtime("Path.add expects a Path"));
            };
            let path = path.borrow::<ScriptedPath>()?;
            let transform = match args.get(1) {
                Some(Value::UserData(matrix)) => matrix.borrow::<ScriptedMat2D>()?.0,
                Some(Value::Nil) | None => Mat2D::IDENTITY,
                _ => return Err(Error::runtime("Path.add transform must be a Mat2D")),
            };
            this.raw_path.add_path(&path.raw_path, transform);
            this.mark_dirty();
            Ok(())
        });
        methods.add_method("contours", |lua, this, ()| {
            let contours = Rc::new(RuntimeContourMeasure::from_commands(&this.commands()));
            Ok(match contours.is_empty() {
                true => Value::Nil,
                false => Value::UserData(
                    lua.create_userdata(ScriptedContourMeasure { contours, index: 0 })?,
                ),
            })
        });
        methods.add_method("measure", |lua, this, ()| {
            lua.create_userdata(ScriptedPathMeasure {
                measure: RuntimePathMeasure::from_commands(&this.commands()),
            })
        });
    }
}

struct ScriptedContourMeasure {
    contours: Rc<Vec<RuntimeContourMeasure>>,
    index: usize,
}

impl ScriptedContourMeasure {
    fn measure(&self) -> &RuntimeContourMeasure {
        &self.contours[self.index]
    }
}

impl UserData for ScriptedContourMeasure {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("length", |_, this| Ok(this.measure().length()));
        fields.add_field_method_get("isClosed", |_, this| Ok(this.measure().is_closed()));
        fields.add_field_method_get("next", |lua, this| {
            let next = this.index + 1;
            Ok(match next < this.contours.len() {
                true => Value::UserData(lua.create_userdata(Self {
                    contours: Rc::clone(&this.contours),
                    index: next,
                })?),
                false => Value::Nil,
            })
        });
    }

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("positionAndTangent", |_, this, distance: f32| {
            let sample = this.measure().at_distance(distance);
            Ok((
                LuaVector::new(sample.pos.0, sample.pos.1, 0.0),
                LuaVector::new(sample.tan.0, sample.tan.1, 0.0),
            ))
        });
        methods.add_method("warp", |_, this, point: LuaVector| {
            let sample = this.measure().at_distance(point.x());
            Ok(LuaVector::new(
                sample.pos.0 - sample.tan.1 * point.y(),
                sample.pos.1 + sample.tan.0 * point.y(),
                0.0,
            ))
        });
        methods.add_method("extract", |_, this, args: MultiValue| {
            extract_measure_segment(
                this.measure().segment(
                    number_arg(args.front(), "startDistance")?,
                    number_arg(args.get(1), "endDistance")?,
                    bool_arg_or(args.get(3), true)?,
                ),
                args.get(2),
            )
        });
    }
}

struct ScriptedPathMeasure {
    measure: RuntimePathMeasure,
}

impl UserData for ScriptedPathMeasure {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("length", |_, this| Ok(this.measure.length()));
        fields.add_field_method_get("isClosed", |_, this| Ok(this.measure.is_closed()));
    }

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("positionAndTangent", |_, this, distance: f32| {
            let sample = this.measure.at_distance(distance);
            Ok((
                LuaVector::new(sample.pos.0, sample.pos.1, 0.0),
                LuaVector::new(sample.tan.0, sample.tan.1, 0.0),
            ))
        });
        methods.add_method("warp", |_, this, point: LuaVector| {
            let sample = this.measure.at_distance(point.x());
            Ok(LuaVector::new(
                sample.pos.0 - sample.tan.1 * point.y(),
                sample.pos.1 + sample.tan.0 * point.y(),
                0.0,
            ))
        });
        methods.add_method("extract", |_, this, args: MultiValue| {
            extract_measure_segment(
                this.measure.segment(
                    number_arg(args.front(), "startDistance")?,
                    number_arg(args.get(1), "endDistance")?,
                    bool_arg_or(args.get(3), true)?,
                ),
                args.get(2),
            )
        });
    }
}

fn extract_measure_segment(segment: RawPath, destination: Option<&Value>) -> Result<()> {
    let Some(Value::UserData(destination)) = destination else {
        return Err(Error::runtime(
            "Path measure extract expects a destination Path",
        ));
    };
    let mut destination = destination.borrow_mut::<ScriptedPath>()?;
    destination.raw_path.add_path(&segment, Mat2D::IDENTITY);
    destination.mark_dirty();
    Ok(())
}

fn bool_arg_or(value: Option<&Value>, fallback: bool) -> Result<bool> {
    match value {
        None | Some(Value::Nil) => Ok(fallback),
        Some(Value::Boolean(value)) => Ok(*value),
        _ => Err(Error::runtime("expected boolean")),
    }
}

pub(super) fn call_path_effect_update(
    table: &Table,
    source: RawPath,
    node: ScriptNode,
) -> Result<RawPath> {
    let lua = table.lua();
    let function: luaur_rt::Function = table.get("update")?;
    let source = create_scripted_path(&lua, ScriptedPath::from_raw_path(source))?;
    let node = lua.create_userdata(LuaScriptedNode::new(node))?;
    let output: AnyUserData = function.call((table.clone(), source, node))?;
    let output = output.borrow::<ScriptedPath>()?;
    Ok(output.raw_path.clone())
}

pub(super) fn install_path_global(lua: &Lua) -> Result<()> {
    let table = lua.create_table();
    table.set(
        "new",
        lua.create_function(|lua, ()| create_scripted_path(lua, ScriptedPath::new()))?,
    )?;
    table.set_readonly(true);
    lua.globals().set("Path", table)?;
    Ok(())
}

#[cfg(test)]
mod upstream_scripted_path_tests {
    use super::*;
    use crate::vm::ScriptVm;
    use luaur_rt::FromLuaMulti;

    fn compile_source(source: &str) -> Vec<u8> {
        use luaur_compiler::functions::luau_compile::luau_compile;

        luaur_common::set_all_flags(true);
        let mut output_size = 0;
        let output = luau_compile(
            source.as_ptr().cast(),
            source.len(),
            std::ptr::null_mut(),
            &mut output_size,
        );
        assert!(!output.is_null());
        assert_ne!(output_size, 0);
        // SAFETY: luau_compile returns a malloc allocation of output_size bytes.
        let bytecode =
            unsafe { std::slice::from_raw_parts(output.cast::<u8>(), output_size) }.to_vec();
        unsafe extern "C" {
            fn free(pointer: *mut std::ffi::c_void);
        }
        // SAFETY: output is the allocation returned by luau_compile above.
        unsafe { free(output.cast()) };
        assert_ne!(bytecode.first(), Some(&0));
        bytecode
    }

    fn eval<R: FromLuaMulti>(source: &str) -> R {
        let vm = ScriptVm::new();
        vm.install_rive_globals().expect("install Rive globals");
        vm.eval_bytecode("upstream_scripted_path", &compile_source(source))
            .expect("evaluate exact upstream script")
    }

    fn assert_approx(actual: f64, expected: f64) {
        assert!((actual - expected).abs() <= f64::EPSILON);
    }

    #[test]
    fn path_contours_returns_first_contour() {
        let value: bool = eval(
            "local path: Path = Path.new()\n\
             path:moveTo(Vector.xy(0, 0))\n\
             path:lineTo(Vector.xy(10, 0))\n\
             path:lineTo(Vector.xy(10, 10))\n\
             path:close()\n\
             local contour = path:contours()\n\
             return contour ~= nil\n",
        );
        assert!(value);
    }

    #[test]
    fn contour_measure_has_length() {
        let length: f64 = eval(
            "local path: Path = Path.new()\n\
             path:moveTo(Vector.xy(0, 0))\n\
             path:lineTo(Vector.xy(10, 0))\n\
             path:lineTo(Vector.xy(10, 10))\n\
             path:close()\n\
             local contour = path:contours()\n\
             return contour.length\n",
        );
        assert!(length > 0.0);
        assert!(length < 100.0);
    }

    #[test]
    fn contour_measure_is_closed() {
        let value: bool = eval(
            "local path: Path = Path.new()\n\
             path:moveTo(Vector.xy(0, 0))\n\
             path:lineTo(Vector.xy(10, 0))\n\
             path:lineTo(Vector.xy(10, 10))\n\
             path:close()\n\
             local contour = path:contours()\n\
             return contour.isClosed\n",
        );
        assert!(value);
    }

    #[test]
    fn contour_measure_is_closed_false_for_open_path() {
        let value: bool = eval(
            "local path: Path = Path.new()\n\
             path:moveTo(Vector.xy(0, 0))\n\
             path:lineTo(Vector.xy(10, 0))\n\
             path:lineTo(Vector.xy(10, 10))\n\
             local contour = path:contours()\n\
             return contour.isClosed\n",
        );
        assert!(!value);
    }

    #[test]
    fn contour_measure_position_and_tangent() {
        let values: (f64, f64, f64, f64) = eval(
            "local path: Path = Path.new()\n\
             path:moveTo(Vector.xy(0, 0))\n\
             path:lineTo(Vector.xy(10, 0))\n\
             local contour = path:contours()\n\
             local pos, tan = contour:positionAndTangent(0)\n\
             return pos.x, pos.y, tan.x, tan.y\n",
        );
        assert_approx(values.0, 0.0);
        assert_approx(values.1, 0.0);
        assert!(values.2 > 0.0);
        assert_approx(values.3, 0.0);
    }

    #[test]
    fn contour_measure_warp() {
        let values: (f64, f64) = eval(
            "local path: Path = Path.new()\n\
             path:moveTo(Vector.xy(0, 0))\n\
             path:lineTo(Vector.xy(10, 0))\n\
             local contour = path:contours()\n\
             local result = contour:warp(Vector.xy(5, 2))\n\
             return result.x, result.y\n",
        );
        assert_approx(values.0, 5.0);
        assert_approx(values.1, 2.0);
    }

    fn extracted_path(source: &str) -> AnyUserData {
        eval(source)
    }

    #[test]
    fn contour_measure_extract() {
        let path = extracted_path(
            "local path: Path = Path.new()\n\
             path:moveTo(Vector.xy(0, 0))\n\
             path:lineTo(Vector.xy(10, 0))\n\
             path:lineTo(Vector.xy(10, 10))\n\
             local contour = path:contours()\n\
             local destPath: Path = Path.new()\n\
             contour:extract(0, 10, destPath, true)\n\
             return destPath\n",
        );
        let path = path
            .borrow::<ScriptedPath>()
            .expect("ScriptedPath userdata");
        assert!(!path.raw_path.verbs().is_empty());
    }

    #[test]
    fn contour_measure_extract_defaults_to_start_with_move_true() {
        let path = extracted_path(
            "local path: Path = Path.new()\n\
             path:moveTo(Vector.xy(0, 0))\n\
             path:lineTo(Vector.xy(10, 0))\n\
             path:lineTo(Vector.xy(10, 10))\n\
             local contour = path:contours()\n\
             local destPath: Path = Path.new()\n\
             contour:extract(0, 10, destPath)\n\
             return destPath\n",
        );
        let path = path
            .borrow::<ScriptedPath>()
            .expect("ScriptedPath userdata");
        assert!(!path.raw_path.verbs().is_empty());
        assert_eq!(path.raw_path.verbs()[0], PathVerb::Move);
    }

    #[test]
    #[ignore = "expected-red: startWithMove=false inserts a second move instead of a line"]
    fn contour_measure_extract_with_start_with_move_false() {
        let path = extracted_path(
            "local path: Path = Path.new()\n\
             path:moveTo(Vector.xy(0, 0))\n\
             path:lineTo(Vector.xy(10, 0))\n\
             path:lineTo(Vector.xy(10, 10))\n\
             local contour = path:contours()\n\
             local destPath: Path = Path.new()\n\
             destPath:moveTo(Vector.xy(100, 100))\n\
             contour:extract(0, 10, destPath, false)\n\
             return destPath\n",
        );
        let path = path
            .borrow::<ScriptedPath>()
            .expect("ScriptedPath userdata");
        assert!(path.raw_path.verbs().len() > 1);
        assert_eq!(path.raw_path.verbs()[0], PathVerb::Move);
        assert_eq!(path.raw_path.verbs()[1], PathVerb::Line);
    }

    #[test]
    fn contour_measure_next_iterates_contours() {
        let values: (bool, bool) = eval(
            "local path: Path = Path.new()\n\
             path:moveTo(Vector.xy(0, 0))\n\
             path:lineTo(Vector.xy(10, 0))\n\
             path:close()\n\
             path:moveTo(Vector.xy(20, 20))\n\
             path:lineTo(Vector.xy(30, 20))\n\
             path:close()\n\
             local contour1 = path:contours()\n\
             local contour2 = contour1.next\n\
             return contour1 ~= nil, contour2 ~= nil\n",
        );
        assert!(values.0);
        assert!(values.1);
    }

    #[test]
    fn contour_measure_next_returns_nil_when_done() {
        let value: bool = eval(
            "local path: Path = Path.new()\n\
             path:moveTo(Vector.xy(0, 0))\n\
             path:lineTo(Vector.xy(10, 0))\n\
             path:close()\n\
             local contour1 = path:contours()\n\
             local contour2 = contour1.next\n\
             return contour2 == nil\n",
        );
        assert!(value);
    }

    #[test]
    fn path_measure_returns_path_measure() {
        let value: bool = eval(
            "local path: Path = Path.new()\n\
             path:moveTo(Vector.xy(0, 0))\n\
             path:lineTo(Vector.xy(10, 0))\n\
             path:lineTo(Vector.xy(10, 10))\n\
             path:close()\n\
             local measure = path:measure()\n\
             return measure ~= nil\n",
        );
        assert!(value);
    }

    #[test]
    fn path_measure_has_length() {
        let length: f64 = eval(
            "local path: Path = Path.new()\n\
             path:moveTo(Vector.xy(0, 0))\n\
             path:lineTo(Vector.xy(10, 0))\n\
             path:lineTo(Vector.xy(10, 10))\n\
             path:close()\n\
             local measure = path:measure()\n\
             return measure.length\n",
        );
        assert!(length > 0.0);
        assert!(length < 100.0);
    }

    #[test]
    fn path_measure_is_closed_for_single_closed_contour() {
        let value: bool = eval(
            "local path: Path = Path.new()\n\
             path:moveTo(Vector.xy(0, 0))\n\
             path:lineTo(Vector.xy(10, 0))\n\
             path:lineTo(Vector.xy(10, 10))\n\
             path:close()\n\
             local measure = path:measure()\n\
             return measure.isClosed\n",
        );
        assert!(value);
    }

    #[test]
    fn path_measure_is_closed_false_for_multiple_contours() {
        let value: bool = eval(
            "local path: Path = Path.new()\n\
             path:moveTo(Vector.xy(0, 0))\n\
             path:lineTo(Vector.xy(10, 0))\n\
             path:close()\n\
             path:moveTo(Vector.xy(20, 20))\n\
             path:lineTo(Vector.xy(30, 20))\n\
             path:close()\n\
             local measure = path:measure()\n\
             return measure.isClosed\n",
        );
        assert!(!value);
    }

    #[test]
    fn path_measure_is_closed_false_for_open_path() {
        let value: bool = eval(
            "local path: Path = Path.new()\n\
             path:moveTo(Vector.xy(0, 0))\n\
             path:lineTo(Vector.xy(10, 0))\n\
             path:lineTo(Vector.xy(10, 10))\n\
             local measure = path:measure()\n\
             return measure.isClosed\n",
        );
        assert!(!value);
    }

    #[test]
    fn path_measure_position_and_tangent() {
        let values: (f64, f64, f64, f64) = eval(
            "local path: Path = Path.new()\n\
             path:moveTo(Vector.xy(0, 0))\n\
             path:lineTo(Vector.xy(10, 0))\n\
             local measure = path:measure()\n\
             local pos, tan = measure:positionAndTangent(0)\n\
             return pos.x, pos.y, tan.x, tan.y\n",
        );
        assert_approx(values.0, 0.0);
        assert_approx(values.1, 0.0);
        assert!(values.2 > 0.0);
        assert_approx(values.3, 0.0);
    }

    #[test]
    fn path_measure_warp() {
        let values: (f64, f64) = eval(
            "local path: Path = Path.new()\n\
             path:moveTo(Vector.xy(0, 0))\n\
             path:lineTo(Vector.xy(10, 0))\n\
             local measure = path:measure()\n\
             local result = measure:warp(Vector.xy(5, 2))\n\
             return result.x, result.y\n",
        );
        assert_approx(values.0, 5.0);
        assert_approx(values.1, 2.0);
    }

    #[test]
    fn path_measure_extract() {
        let path = extracted_path(
            "local path: Path = Path.new()\n\
             path:moveTo(Vector.xy(0, 0))\n\
             path:lineTo(Vector.xy(10, 0))\n\
             path:lineTo(Vector.xy(10, 10))\n\
             local measure = path:measure()\n\
             local destPath: Path = Path.new()\n\
             measure:extract(0, 10, destPath, true)\n\
             return destPath\n",
        );
        let path = path
            .borrow::<ScriptedPath>()
            .expect("ScriptedPath userdata");
        assert!(!path.raw_path.verbs().is_empty());
    }

    #[test]
    fn path_measure_extract_defaults_to_start_with_move_true() {
        let path = extracted_path(
            "local path: Path = Path.new()\n\
             path:moveTo(Vector.xy(0, 0))\n\
             path:lineTo(Vector.xy(10, 0))\n\
             path:lineTo(Vector.xy(10, 10))\n\
             local measure = path:measure()\n\
             local destPath: Path = Path.new()\n\
             measure:extract(0, 10, destPath)\n\
             return destPath\n",
        );
        let path = path
            .borrow::<ScriptedPath>()
            .expect("ScriptedPath userdata");
        assert!(!path.raw_path.verbs().is_empty());
        assert_eq!(path.raw_path.verbs()[0], PathVerb::Move);
    }

    #[test]
    #[ignore = "expected-red: startWithMove=false inserts a second move instead of a line"]
    fn path_measure_extract_with_start_with_move_false() {
        let path = extracted_path(
            "local path: Path = Path.new()\n\
             path:moveTo(Vector.xy(0, 0))\n\
             path:lineTo(Vector.xy(10, 0))\n\
             path:lineTo(Vector.xy(10, 10))\n\
             local measure = path:measure()\n\
             local destPath: Path = Path.new()\n\
             destPath:moveTo(Vector.xy(100, 100))\n\
             measure:extract(0, 10, destPath, false)\n\
             return destPath\n",
        );
        let path = path
            .borrow::<ScriptedPath>()
            .expect("ScriptedPath userdata");
        assert!(path.raw_path.verbs().len() > 1);
        assert_eq!(path.raw_path.verbs()[0], PathVerb::Move);
        assert_eq!(path.raw_path.verbs()[1], PathVerb::Line);
    }

    #[test]
    fn path_measure_extract_across_multiple_contours() {
        let path = extracted_path(
            "local path: Path = Path.new()\n\
             path:moveTo(Vector.xy(0, 0))\n\
             path:lineTo(Vector.xy(10, 0))\n\
             path:close()\n\
             path:moveTo(Vector.xy(20, 0))\n\
             path:lineTo(Vector.xy(30, 0))\n\
             path:close()\n\
             local measure = path:measure()\n\
             local destPath: Path = Path.new()\n\
             measure:extract(5, 25, destPath, true)\n\
             return destPath\n",
        );
        let path = path
            .borrow::<ScriptedPath>()
            .expect("ScriptedPath userdata");
        assert!(!path.raw_path.verbs().is_empty());
    }
}
