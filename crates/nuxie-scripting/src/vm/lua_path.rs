// Translated from:
// /Users/levi/dev/oss/rive-runtime/src/lua/renderer/lua_path.cpp
use std::{cell::RefCell, rc::Rc};

use crate::luau::{
    AnyUserData, Error, Lua, MultiValue, Result, Table, UserData, UserDataFields, UserDataMethods,
    Value, Vector as LuaVector,
};
use nuxie_render_api::{Factory as RenderFactory, FillRule, RawPath as RenderRawPath, RenderPath};
use nuxie_runtime::ScriptNode;
use nuxie_runtime::mechanical_port::source::{
    artboard::Artboard,
    math::{
        contour_measure::{ContourMeasure, ContourMeasureIter, RefCntContourMeasureIter},
        mat2d::Mat2D,
        path_measure::PathMeasure,
        path_types::{PathVerb, path_verb_to_point_count},
        raw_path::RawPath,
        vec2d::Vec2D,
    },
    renderer::{from_render_raw_path, to_render_raw_path},
};

use super::ProtectedScriptCall;
use super::host_memory_budget::{HostMemoryCharge, grown_capacity};
use super::lua_artboards::ScriptedNode as LuaScriptedNode;
use super::lua_mat2d::{ScriptedMat2D, number_arg};

/// Host bytes a path is charged: its two `RawPath` buffers at `capacity`
/// (verbs, points) plus the render path the renderer builds from the same
/// commands at draw, counted as one copy of its `len` commands.
fn path_bytes(capacity: (usize, usize), len: (usize, usize)) -> usize {
    RawPath::bytes_for(capacity.0, capacity.1).saturating_add(RawPath::bytes_for(len.0, len.1))
}

pub(super) struct ScriptedPath {
    pub(super) raw_path: RawPath,
    dirty: bool,
    render_path: Option<Box<dyn RenderPath>>,
    render_frame_id: u64,
    charge: HostMemoryCharge,
}

impl ScriptedPath {
    fn new() -> Self {
        Self {
            raw_path: RawPath::default(),
            dirty: true,
            render_path: None,
            render_frame_id: 0,
            charge: HostMemoryCharge::default(),
        }
    }

    /// A path built by host code; it is charged when it is handed to a
    /// script (`create_scripted_path`, [`Self::charged`]).
    pub(super) fn from_render_raw_path(raw_path: RenderRawPath) -> Self {
        Self {
            raw_path: from_render_raw_path(&raw_path),
            dirty: true,
            render_path: None,
            render_frame_id: 0,
            charge: HostMemoryCharge::default(),
        }
    }

    /// A host-built path charged to the VM's budget at its current size.
    pub(super) fn charged(lua: &Lua, raw_path: RenderRawPath) -> Result<Self> {
        let mut path = Self::from_render_raw_path(raw_path);
        path.charge_current(lua)?;
        Ok(path)
    }

    fn verb_and_point_counts(&self) -> (usize, usize) {
        (self.raw_path.verbs().len(), self.raw_path.points().len())
    }

    /// Makes room for `verbs` more verbs and `points` more points: charges
    /// the buffers at the capacity they grow to (doubling, so pushes stay
    /// amortized) before reserving exactly that, so the commands that
    /// follow append without reallocating. Running out of memory is an
    /// error, not an abort.
    fn reserve(&mut self, lua: &Lua, verbs: usize, points: usize) -> Result<()> {
        let (current_verbs, current_points) = self.verb_and_point_counts();
        let (verb_capacity, point_capacity) = self.raw_path.capacity();
        let capacity = (
            grown_capacity(verb_capacity, current_verbs, verbs),
            grown_capacity(point_capacity, current_points, points),
        );
        let len = (
            current_verbs.saturating_add(verbs),
            current_points.saturating_add(points),
        );
        self.charge.attach(lua)?;
        self.charge.reserve_to(path_bytes(capacity, len))?;
        self.raw_path
            .try_reserve(capacity.0, capacity.1)
            .map_err(|_| Error::runtime("not enough memory"))
    }

    fn charge_current(&mut self, lua: &Lua) -> Result<()> {
        self.reserve(lua, 0, 0)
    }

    pub(super) fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    pub(super) fn with_render_raw_path_mut<R>(
        &mut self,
        f: impl FnOnce(&mut RenderRawPath) -> R,
    ) -> R {
        let mut render_path = to_render_raw_path(&self.raw_path);
        let result = f(&mut render_path);
        self.raw_path = from_render_raw_path(&render_path);
        result
    }

    pub(super) fn render_path(&mut self, factory: &mut dyn RenderFactory) -> &dyn RenderPath {
        if self.dirty {
            self.dirty = false;
            // Pinned lua_path.cpp replaces a path rebuilt in the same frame:
            // the renderer may retain the first backend path until submission.
            let render_frame_id = Artboard::frame_id();
            let same_frame_rebuild =
                self.render_path.is_some() && self.render_frame_id == render_frame_id;
            self.render_frame_id = render_frame_id;
            let path = match self.render_path.take() {
                Some(mut path) if !same_frame_rebuild => {
                    path.rewind();
                    path
                }
                _ => {
                    let mut path = factory.make_empty_render_path();
                    path.fill_rule(FillRule::Clockwise);
                    path
                }
            };
            self.render_path
                .insert(path)
                .add_untrusted_raw_path(&to_render_raw_path(&self.raw_path));
        }
        match &mut self.render_path {
            Some(path) => &**path,
            // First use while clean: a path is dirty from construction, so
            // this is reached only if a host cleared `dirty` itself.
            render_path => &**render_path.insert(factory.make_empty_render_path()),
        }
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

        let point_index = self
            .raw_path
            .verbs()
            .iter()
            .take(verb_index)
            .map(|verb| path_verb_to_point_count(*verb))
            .fold(0_usize, usize::saturating_add);
        let point_count = path_verb_to_point_count(verb);
        let points = self
            .raw_path
            .points()
            .iter()
            .skip(point_index)
            .take(point_count)
            .copied()
            .collect();

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
    // Path layers C++'s numeric indexing over the userdata's ordinary method
    // table without replacing its typed userdata cell.
    lua.getmetatable(userdata)
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

#[derive(Default)]
struct PathMetatablePatched;

#[derive(Default)]
struct PathCommandMetatablePatched;

pub(super) fn create_scripted_path(lua: &Lua, mut path: ScriptedPath) -> Result<AnyUserData> {
    path.charge_current(lua)?;
    let userdata = lua.create_userdata(path)?;
    super::host_dispatch::patch_metatable_once::<PathMetatablePatched>(lua, || {
        patch_path_metatable(lua, userdata.clone())
    })?;
    Ok(userdata)
}

fn patch_path_metatable(lua: &Lua, userdata: AnyUserData) -> Result<()> {
    let metatable = userdata_metatable(lua, userdata)?;
    if !metatable.is_readonly() {
        let methods: Table = metatable.get("__index")?;
        let fallback_methods = methods.clone();
        let fallback = lua.create_function(move |lua, (userdata, key): (AnyUserData, Value)| {
            match lua_path_index(&key)? {
                Some(index) => {
                    let command = {
                        let path = userdata.borrow::<ScriptedPath>()?;
                        path.command(index)
                    };
                    create_scripted_path_command(lua, command).map(Value::UserData)
                }
                None => fallback_methods.get(key),
            }
        })?;
        // Named methods resolve in the VM; numeric keys reach the fallback.
        let index = super::host_dispatch::methods_first_index(lua, &methods, fallback)?;
        metatable.set("__index", index)?;
        metatable.set_readonly(true);
    }
    Ok(())
}

fn create_scripted_path_command(lua: &Lua, command: ScriptedPathCommand) -> Result<AnyUserData> {
    let userdata = lua.create_userdata(command)?;
    super::host_dispatch::patch_metatable_once::<PathCommandMetatablePatched>(lua, || {
        patch_path_command_metatable(lua, userdata.clone())
    })?;
    Ok(userdata)
}

fn patch_path_command_metatable(lua: &Lua, userdata: AnyUserData) -> Result<()> {
    let metatable = userdata_metatable(lua, userdata)?;
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
    Ok(())
}

impl UserData for ScriptedPath {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_meta_method("__len", |_, this, ()| Ok(this.raw_path.verbs().len()));
        // A drawing command adds one verb and its points, plus the move a
        // `RawPath` injects before a curve that starts no contour.
        methods.add_method_mut("moveTo", |lua, this, point: LuaVector| {
            this.reserve(lua, 1, 1)?;
            this.raw_path.move_to(point.x(), point.y());
            this.mark_dirty();
            Ok(())
        });
        methods.add_method_mut("lineTo", |lua, this, point: LuaVector| {
            this.reserve(lua, 2, 2)?;
            this.raw_path.line_to(point.x(), point.y());
            this.mark_dirty();
            Ok(())
        });
        methods.add_method_mut(
            "quadTo",
            |lua, this, (control, point): (LuaVector, LuaVector)| {
                this.reserve(lua, 2, 3)?;
                this.raw_path
                    .quad_to(control.x(), control.y(), point.x(), point.y());
                this.mark_dirty();
                Ok(())
            },
        );
        methods.add_method_mut(
            "cubicTo",
            |lua, this, (out, inn, point): (LuaVector, LuaVector, LuaVector)| {
                this.reserve(lua, 2, 4)?;
                this.raw_path
                    .cubic_to(out.x(), out.y(), inn.x(), inn.y(), point.x(), point.y());
                this.mark_dirty();
                Ok(())
            },
        );
        methods.add_method_mut("close", |lua, this, ()| {
            this.reserve(lua, 1, 0)?;
            this.raw_path.close();
            this.mark_dirty();
            Ok(())
        });
        methods.add_method_mut("reset", |_, this, ()| {
            // `RawPath::reset` releases both vectors' capacity. A render
            // path already built is kept (the next draw rewinds it, the
            // factory is not asked for another) and keeps its allocation,
            // so then the charge stays as a high-water mark.
            this.raw_path.reset();
            if this.render_path.is_none() {
                // Both buffers were released: nothing is left to charge.
                this.charge.set(0)?;
            }
            this.mark_dirty();
            Ok(())
        });
        methods.add_function("add", |lua, (userdata, args): (AnyUserData, MultiValue)| {
            let Some(Value::UserData(path)) = args.front() else {
                return Err(Error::runtime("Path.add expects a Path"));
            };
            let transform = match args.get(1) {
                Some(Value::UserData(matrix)) => {
                    let [a, b, c, d, x, y] = matrix.borrow::<ScriptedMat2D>()?.0.0;
                    Some(Mat2D::new(a, b, c, d, x, y))
                }
                None => None,
                _ => return Err(Error::runtime("Path.add transform must be a Mat2D")),
            };
            let mut this = userdata.borrow_mut::<ScriptedPath>()?;
            if userdata.to_pointer() == path.to_pointer() {
                // Doubles the path (`add_self_path` copies it first; the
                // copy is transient and no larger than the charge).
                let (verbs, points) = this.verb_and_point_counts();
                this.reserve(lua, verbs, points)?;
                this.raw_path.add_self_path(transform.as_ref());
            } else {
                let path = path.borrow::<ScriptedPath>()?;
                let (verbs, points) = path.verb_and_point_counts();
                this.reserve(lua, verbs, points)?;
                this.raw_path.add_path(&path.raw_path, transform.as_ref());
            }
            this.mark_dirty();
            Ok(())
        });
        methods.add_method("contours", |lua, this, ()| {
            // The iterator copies the path: charged before it does.
            let charge =
                HostMemoryCharge::for_lua(lua, ContourMeasureIter::copy_bytes(&this.raw_path))?;
            let iter = Rc::new(RefCell::new(ChargedContourIter {
                iter: RefCntContourMeasureIter::new(
                    &this.raw_path,
                    ContourMeasureIter::DEFAULT_TOLERANCE,
                ),
                _charge: charge,
            }));
            let first = next_contour(lua, &iter)?;
            Ok(match first {
                None => Value::Nil,
                Some((measure, charge)) => {
                    Value::UserData(lua.create_userdata(ScriptedContourMeasure {
                        measure,
                        iter,
                        _charge: charge,
                    })?)
                }
            })
        });
        methods.add_method("measure", |lua, this, ()| {
            // Each contour is charged before its tables are built.
            let mut charge = HostMemoryCharge::default();
            charge.attach(lua)?;
            let measure = PathMeasure::from_path_admitted(
                &this.raw_path,
                ContourMeasureIter::DEFAULT_TOLERANCE,
                |bytes| charge.reserve_to(bytes),
            )?;
            charge.set(measure.heap_bytes())?;
            lua.create_userdata(ScriptedPathMeasure {
                measure,
                _charge: charge,
            })
        });
    }
}

/// A contour iterator, which holds a copy of the measured path, and its
/// charge; shared by every contour measure it returned.
struct ChargedContourIter {
    iter: RefCntContourMeasureIter,
    _charge: HostMemoryCharge,
}

/// The iterator's next contour and the charge for its tables, made before
/// they are built.
fn next_contour(
    lua: &Lua,
    iter: &RefCell<ChargedContourIter>,
) -> Result<Option<(Rc<ContourMeasure>, HostMemoryCharge)>> {
    let mut charge = None;
    let next = iter.borrow_mut().iter.get().next_admitted(|bytes| {
        // A contour of no length is built and skipped: its tables are gone
        // by now, so its charge goes before the next one's is taken.
        drop(charge.take());
        charge = Some(HostMemoryCharge::for_lua(lua, bytes)?);
        Ok::<(), Error>(())
    })?;
    Ok(next.map(|measure| (measure, charge.unwrap_or_default())))
}

struct ScriptedContourMeasure {
    measure: Rc<ContourMeasure>,
    iter: Rc<RefCell<ChargedContourIter>>,
    _charge: HostMemoryCharge,
}

impl ScriptedContourMeasure {
    fn measure(&self) -> &ContourMeasure {
        &self.measure
    }
}

impl UserData for ScriptedContourMeasure {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("length", |_, this| Ok(this.measure().length()));
        fields.add_field_method_get("isClosed", |_, this| Ok(this.measure().is_closed()));
        fields.add_field_method_get("next", |lua, this| {
            let next = next_contour(lua, &this.iter)?;
            Ok(match next {
                Some((measure, charge)) => Value::UserData(lua.create_userdata(Self {
                    measure,
                    iter: Rc::clone(&this.iter),
                    _charge: charge,
                })?),
                None => Value::Nil,
            })
        });
    }

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("positionAndTangent", |_, this, distance: f32| {
            let sample = this.measure().get_pos_tan(distance);
            Ok((
                LuaVector::new(sample.pos.x, sample.pos.y, 0.0),
                LuaVector::new(sample.tan.x, sample.tan.y, 0.0),
            ))
        });
        methods.add_method("warp", |_, this, point: LuaVector| {
            let result = this.measure().warp(Vec2D::new(point.x(), point.y()));
            Ok(LuaVector::new(result.x, result.y, 0.0))
        });
        methods.add_method("extract", |lua, this, args: MultiValue| {
            let start = number_arg(args.front(), "startDistance")?;
            let end = number_arg(args.get(1), "endDistance")?;
            let start_with_move = bool_arg_or(args.get(3), true)?;
            let room = this.measure().segment_room(start, end);
            extract_measure_segment(lua, args.get(2), room, |destination| {
                this.measure()
                    .get_segment(start, end, destination, start_with_move);
            })
        });
    }
}

struct ScriptedPathMeasure {
    measure: PathMeasure,
    _charge: HostMemoryCharge,
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
                LuaVector::new(sample.pos.x, sample.pos.y, 0.0),
                LuaVector::new(sample.tan.x, sample.tan.y, 0.0),
            ))
        });
        methods.add_method("warp", |_, this, point: LuaVector| {
            let sample = this.measure.at_distance(point.x());
            Ok(LuaVector::new(
                sample.pos.x - sample.tan.y * point.y(),
                sample.pos.y + sample.tan.x * point.y(),
                0.0,
            ))
        });
        methods.add_method("extract", |lua, this, args: MultiValue| {
            let start = number_arg(args.front(), "startDistance")?;
            let end = number_arg(args.get(1), "endDistance")?;
            let start_with_move = bool_arg_or(args.get(3), true)?;
            let room = this.measure.segment_room(start, end);
            extract_measure_segment(lua, args.get(2), room, |destination| {
                this.measure
                    .get_segment(start, end, Some(destination), start_with_move);
            })
        });
    }
}

/// Appends a measured segment to the destination path, charged and
/// reserved before it grows. `room` is the most verbs and points the
/// interval appends (`segment_room`), bounded by the segments the interval
/// covers, not the whole measured path; an empty interval needs none.
fn extract_measure_segment(
    lua: &Lua,
    destination: Option<&Value>,
    room: (usize, usize),
    append: impl FnOnce(&mut RawPath),
) -> Result<()> {
    let Some(Value::UserData(destination)) = destination else {
        return Err(Error::runtime(
            "Path measure extract expects a destination Path",
        ));
    };
    let mut destination = destination.borrow_mut::<ScriptedPath>()?;
    destination.reserve(lua, room.0, room.1)?;
    append(&mut destination.raw_path);
    destination.charge_current(lua)?;
    destination.mark_dirty();
    Ok(())
}

fn bool_arg_or(value: Option<&Value>, fallback: bool) -> Result<bool> {
    match value {
        Some(Value::Boolean(value)) => Ok(*value),
        _ => Ok(fallback),
    }
}

/// Checked counterpart of lua_topathdata. Path.new() and the PathData passed
/// to update() share ScriptedPath in the approved Rust userdata representation.
pub(super) fn lua_topathdata(value: &Value) -> Result<Option<RenderRawPath>> {
    let Value::UserData(value) = value else {
        return Ok(None);
    };
    if !value.is::<ScriptedPath>() {
        return Ok(None);
    }
    let path = value.borrow::<ScriptedPath>()?;
    Ok(Some(to_render_raw_path(&path.raw_path)))
}

pub(super) fn call_path_effect_update(
    table: &Table,
    source: RenderRawPath,
    node: ScriptNode,
    owner: Option<super::view_model::ScriptedPropertyListenerOwner>,
) -> Result<Option<RenderRawPath>> {
    let lua = table.lua();
    let function: crate::luau::Function = table.get_field("update")?;
    let source = create_scripted_path(&lua, ScriptedPath::from_render_raw_path(source))?;
    let node = lua.create_userdata(LuaScriptedNode::new(node))?;
    let _property_owner = owner.map(|owner| {
        super::view_model::ScriptViewModelFrameContext::for_lua(&lua).enter_owner(owner)
    });
    let output: Value = function.protected_call((table.clone(), source, node))?;
    // None is upstream's false result, distinct from a valid empty Path.
    // Rejection itself is not a Lua execution error.
    lua_topathdata(&output)
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
    use crate::luau::FromLuaMulti;
    use crate::vm::ScriptVm;

    #[test]
    fn lua_topathdata_accepts_path_and_path_data_and_rejects_the_rest() {
        let vm = ScriptVm::new();
        vm.install_rive_globals().unwrap();
        let lua = vm.lua();
        let mut source = RenderRawPath::new();
        source.move_to(0.0, 0.0);
        source.line_to(10.0, 0.0);
        // The approved Rust wrapper uses ScriptedPath for both the PathData
        // callback input and script-constructed Path. Exercise both routes.
        let path_data = Value::UserData(
            create_scripted_path(lua, ScriptedPath::from_render_raw_path(source)).unwrap(),
        );
        assert_eq!(
            lua_topathdata(&path_data)
                .unwrap()
                .expect("PathData accepted")
                .verbs()
                .len(),
            2
        );
        let path: Value = vm.eval_bytecode("path_return_type", &compile_source("local p = Path.new(); p:moveTo(Vector(0, 0)); p:lineTo(Vector(10, 0)); return p")).unwrap();
        assert_eq!(
            lua_topathdata(&path)
                .unwrap()
                .expect("Path accepted")
                .verbs()
                .len(),
            2
        );
        assert!(lua_topathdata(&Value::Nil).unwrap().is_none());
        assert!(lua_topathdata(&Value::Number(42.0)).unwrap().is_none());
        assert!(
            lua_topathdata(&Value::Table(lua.create_table()))
                .unwrap()
                .is_none()
        );
        let unrelated = Value::UserData(
            super::super::lua_data_value::create_data_value(
                lua,
                nuxie_runtime::ScriptValue::Number(1.0),
            )
            .unwrap(),
        );
        assert!(matches!(&unrelated, Value::UserData(_)));
        assert!(lua_topathdata(&unrelated).unwrap().is_none());
    }

    // Supplemental seam regression: invalid returns must not take the valid
    // empty-path branch, while actual Lua failures remain callback errors.
    #[test]
    fn path_effect_return_rejection_is_distinct_from_empty_geometry_and_errors() {
        let vm = ScriptVm::new();
        vm.install_rive_globals().unwrap();
        for body in ["return nil", "return 42", "return DataValue.number()"] {
            let table: Table = vm
                .eval_bytecode(
                    "rejected_effect",
                    &compile_source(&format!(
                        "return {{update = function(self, path, node) {body} end}}"
                    )),
                )
                .unwrap();
            assert!(
                call_path_effect_update(
                    &table,
                    RenderRawPath::new(),
                    ScriptNode::snapshot(None, None),
                    None,
                )
                .unwrap()
                .is_none()
            );
        }
        let empty: Table = vm
            .eval_bytecode(
                "empty_effect",
                &compile_source(
                    "return {update = function(self, path, node) return Path.new() end}",
                ),
            )
            .unwrap();
        assert!(
            call_path_effect_update(
                &empty,
                RenderRawPath::new(),
                ScriptNode::snapshot(None, None),
                None,
            )
            .unwrap()
            .expect("valid empty Path")
            .verbs()
            .is_empty()
        );

        let identity: Table = vm
            .eval_bytecode(
                "identity_effect",
                &compile_source("return {update = function(self, path, node) return path end}"),
            )
            .unwrap();
        let mut source = RenderRawPath::new();
        source.move_to(0.0, 0.0);
        source.line_to(10.0, 0.0);
        assert_eq!(
            call_path_effect_update(&identity, source, ScriptNode::snapshot(None, None), None)
                .unwrap()
                .expect("valid input PathData")
                .verbs()
                .len(),
            2
        );

        let throwing: Table = vm
            .eval_bytecode(
                "throwing_effect",
                &compile_source(
                    "return {update = function(self, path, node) error('path update failed') end}",
                ),
            )
            .unwrap();
        assert!(
            call_path_effect_update(
                &throwing,
                RenderRawPath::new(),
                ScriptNode::snapshot(None, None),
                None,
            )
            .unwrap_err()
            .to_string()
            .contains("path update failed")
        );
    }

    #[allow(unsafe_code)]
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

/// What paths and their measures are charged against the VM's host memory
/// budget.
#[cfg(all(test, feature = "compiler"))]
mod budget_tests {
    use super::super::host_memory_budget::HostMemoryBudget;
    use super::ScriptedPath;
    use crate::luau::AnyUserData;
    use crate::vm::{ScriptExecutionLimits, ScriptVm};
    use nuxie_render_api::{PersistentFactory, RecordingFactory};
    use nuxie_runtime::mechanical_port::source::math::raw_path::RawPath;

    struct Vm {
        vm: ScriptVm,
        _factory: PersistentFactory<RecordingFactory>,
    }

    impl Vm {
        fn with_budget(bytes: usize) -> Self {
            let vm = ScriptVm::new_with_execution_limits(
                ScriptExecutionLimits::new()
                    .with_max_host_memory_bytes(bytes)
                    .with_max_interrupts_per_callback(10_000_000),
            )
            .unwrap();
            let mut factory = PersistentFactory::new(RecordingFactory::new());
            vm.install_render_factory(&mut factory).unwrap();
            vm.install_rive_globals().unwrap();
            Self {
                vm,
                _factory: factory,
            }
        }

        fn used(&self) -> usize {
            HostMemoryBudget::for_lua(self.vm.lua()).unwrap().used()
        }

        fn path_bytes(&self, name: &str) -> usize {
            let path: AnyUserData = self.vm.lua().globals().get(name).unwrap();
            let path = path.borrow::<ScriptedPath>().unwrap();
            let raw = &path.raw_path;
            // The capacity exactly, and one render copy of the commands: of
            // what they held, or at most the one extra verb and point a
            // command reserves for a move it may not have needed.
            let exact = raw
                .heap_bytes()
                .saturating_add(RawPath::bytes_for(raw.verbs().len(), raw.points().len()));
            let charged = path.charge.bytes();
            assert!(
                (exact..=exact + RawPath::bytes_for(1, 1)).contains(&charged),
                "charged {charged}, holds {exact}"
            );
            charged
        }

        fn refused(&self, script: &str) -> String {
            let (failed, error): (bool, String) = self.vm.eval(script).unwrap();
            assert!(failed, "expected a refusal, got success");
            assert!(error.contains("host memory budget"), "{error}");
            error
        }
    }

    /// A source of 200 cubic curves: its measure has about 50 segments per
    /// curve.
    const WAVY: &str = "source = Path.new() \
         source:moveTo(Vector.xy(0, 0)) \
         for i = 1, 200 do \
           source:cubicTo(Vector.xy(i * 10 - 7, 300), Vector.xy(i * 10 - 3, -300), Vector.xy(i * 10, 0)) \
         end ";

    #[test]
    fn a_path_is_charged_its_capacity_and_one_render_copy() {
        let vm = Vm::with_budget(64 * 1024 * 1024);
        vm.vm
            .eval::<()>("p = Path.new() for i = 1, 1000 do p:lineTo(Vector.xy(i, i)) end")
            .unwrap();
        let bytes = vm.path_bytes("p");
        assert_eq!(vm.used(), bytes);
        let estimate = (1001 + 1001 * 8) * 3;
        println!("1,000 lineTo: charged {bytes} bytes (estimate was {estimate})");
    }

    #[test]
    fn a_measure_is_charged_its_tables_and_refunds_when_collected() {
        let vm = Vm::with_budget(64 * 1024 * 1024);
        vm.vm.eval::<()>(WAVY).unwrap();
        let source = vm.used();
        vm.vm.eval::<()>("m = source:measure()").unwrap();
        let measure = vm.used() - source;
        println!("measure of 200 cubics: charged {measure} bytes (was 0)");
        assert!(measure > 50_000, "{measure}");
        vm.vm.eval::<()>("m = nil").unwrap();
        vm.vm.lua().gc_collect().unwrap();
        assert_eq!(vm.used(), source);
    }

    #[test]
    fn contours_charge_the_iterator_copy_and_each_contour() {
        let vm = Vm::with_budget(64 * 1024 * 1024);
        vm.vm
            .eval::<()>(&format!(
                "{WAVY} source:moveTo(Vector.xy(0, 500)) source:lineTo(Vector.xy(10, 500))"
            ))
            .unwrap();
        let source = vm.used();
        vm.vm.eval::<()>("c = source:contours()").unwrap();
        let first = vm.used() - source;
        vm.vm.eval::<()>("d = c.next").unwrap();
        let second = vm.used() - source - first;
        println!("contours: iterator and first contour {first} bytes, second {second} (was 0)");
        assert!(first > 50_000, "{first}");
        assert!(second > 0 && second < 1_000, "{second}");
        // Keeping the measures after resetting the source keeps their charge.
        vm.vm.eval::<()>("source:reset()").unwrap();
        vm.vm.lua().gc_collect().unwrap();
        assert_eq!(vm.used(), first + second);
        vm.vm.eval::<()>("c = nil d = nil").unwrap();
        vm.vm.lua().gc_collect().unwrap();
        assert_eq!(vm.used(), 0);
    }

    // Codex review: a skipped contour of no length kept its charge while the
    // next contour's was taken, so a next that fit was refused.
    #[test]
    fn a_skipped_contour_gives_its_charge_back_before_the_next() {
        const PATH: &str = "source = Path.new() \
             source:moveTo(Vector.xy(0, 0)) source:lineTo(Vector.xy(10, 0)) \
             source:moveTo(Vector.xy(5, 5)) source:lineTo(Vector.xy(5, 5)) \
             source:moveTo(Vector.xy(0, 9)) source:lineTo(Vector.xy(10, 9)) \
             c = source:contours()";
        let probe = Vm::with_budget(64 * 1024 * 1024);
        probe.vm.eval::<()>(PATH).unwrap();
        probe.vm.eval::<()>("d = c.next").unwrap();
        assert!(probe.vm.eval::<bool>("return d ~= nil").unwrap());
        // Exactly room for the source, the iterator and both contours kept.
        let vm = Vm::with_budget(probe.used());
        vm.vm.eval::<()>(PATH).unwrap();
        vm.vm.eval::<()>("d = c.next").unwrap();
        assert!(vm.vm.eval::<bool>("return d ~= nil").unwrap());
    }

    #[test]
    fn a_measure_over_the_budget_is_refused_before_it_is_built() {
        let vm = Vm::with_budget(40_000);
        vm.vm.eval::<()>(WAVY).unwrap();
        vm.refused(
            "local ok, err = pcall(function() return source:measure() end) \
             return not ok, tostring(err)",
        );
        vm.refused(
            "local ok, err = pcall(function() return source:contours() end) \
             return not ok, tostring(err)",
        );
        vm.vm.lua().gc_collect().unwrap();
        assert_eq!(vm.used(), vm.path_bytes("source"));
    }

    // Lane E2b admitted an extract on a bound of the whole measured source,
    // so a short extract from a large source could be refused.
    #[test]
    fn an_extract_is_bounded_by_its_interval_not_the_source() {
        let vm = Vm::with_budget(2 * 1024 * 1024);
        vm.vm
            .eval::<()>(
                "source = Path.new() source:moveTo(Vector.xy(0, 0)) \
                 for i = 1, 20000 do source:lineTo(Vector.xy(i, i % 2)) end \
                 m = source:measure() \
                 dest = Path.new()",
            )
            .unwrap();
        vm.vm.eval::<()>("m:extract(0, 3, dest)").unwrap();
        assert_eq!(vm.vm.eval::<usize>("return #dest").unwrap(), 4);
        // The whole source still does not fit.
        vm.refused(
            "local ok, err = pcall(function() \
               for i = 1, 10 do m:extract(0, 1000000, dest) end \
             end) \
             return not ok, tostring(err)",
        );
    }
}
