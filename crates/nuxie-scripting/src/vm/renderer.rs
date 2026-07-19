// Coarsely translated from the first draw-path binding slice in:
// /Users/levi/dev/oss/rive-runtime/src/lua/renderer/lua_renderer.cpp
// /Users/levi/dev/oss/rive-runtime/src/lua/renderer/lua_path.cpp
// /Users/levi/dev/oss/rive-runtime/src/lua/renderer/lua_paint.cpp
// /Users/levi/dev/oss/rive-runtime/src/lua/math/lua_mat2d.cpp
use std::cell::{Cell, RefCell};
use std::mem;
use std::ptr::NonNull;
use std::rc::Rc;

use luaur_rt::{
    AnyUserData, Error, Lua, MultiValue, Result, Table, UserData, UserDataFields, UserDataMethods,
    Value, Vector as LuaVector,
};
use luaur_vm::functions::lua_getmetatable::lua_getmetatable;
use nuxie_render_api::{
    BlendMode, ColorInt, Factory as RenderFactory, FillRule, Mat2D, PathVerb, RawPath,
    RenderPaint as RenderPaintTrait, RenderPaintStyle, RenderPath, Renderer, StrokeCap, StrokeJoin,
    Vec2D,
};
use nuxie_runtime::{
    RuntimeContourMeasure, RuntimePathMeasure, ScriptAnimation, ScriptAnimationTime,
    ScriptArtboard, ScriptNode, ScriptPaint as RuntimeScriptPaint,
    runtime_path_commands_from_raw_path,
};

use super::view_model::{
    ScriptViewModelFrameContext, ScriptViewModelRegistration, create_scripted_view_model,
    model_from_table,
};

#[derive(Clone, Default)]
pub(crate) struct RendererBindings {
    factory: Rc<Cell<Option<NonNull<dyn RenderFactory>>>>,
    view_model_frame_context: ScriptViewModelFrameContext,
}

impl RendererBindings {
    pub(crate) fn new(view_model_frame_context: ScriptViewModelFrameContext) -> Self {
        Self {
            factory: Rc::new(Cell::new(None)),
            view_model_frame_context,
        }
    }

    pub(crate) fn with_factory_context<R>(
        &self,
        factory: &mut dyn RenderFactory,
        f: impl FnOnce() -> R,
    ) -> R {
        let previous = self.factory.replace(Some(erase_factory_lifetime(factory)));
        let _guard = FactoryContextGuard {
            factory: Rc::clone(&self.factory),
            previous,
        };
        f()
    }

    pub(crate) fn install(&self, lua: &Lua) -> Result<()> {
        install_color_global(lua)?;
        install_mat2d_global(lua)?;
        install_path_global(lua)?;
        self.install_paint_global(lua)?;
        Ok(())
    }

    pub(crate) fn call_draw(
        &self,
        table: &Table,
        factory: &mut dyn RenderFactory,
        renderer: &mut dyn Renderer,
    ) -> Result<()> {
        let value: Value = table.get("draw")?;
        let Value::Function(function) = value else {
            return Ok(());
        };

        let lua = table.lua();
        self.with_factory_context(factory, || {
            let save_count = Rc::new(Cell::new(0usize));
            let valid = Rc::new(Cell::new(true));
            let renderer_ref = Rc::new(RefCell::new(erase_renderer_lifetime(renderer)));

            let scripted_renderer = lua.create_userdata(ScriptedRenderer {
                renderer: Rc::clone(&renderer_ref),
                bindings: self.clone(),
                save_count: Rc::clone(&save_count),
                valid: Rc::clone(&valid),
            })?;
            let result = function.call::<()>((table.clone(), scripted_renderer));

            while save_count.get() > 0 {
                let mut renderer = renderer_ref.borrow_mut();
                // The renderer userdata is still valid while this cleanup runs;
                // the pointer is invalidated immediately after the save stack is
                // balanced.
                unsafe { renderer.as_mut().restore() };
                save_count.set(save_count.get() - 1);
            }
            valid.set(false);
            result
        })
    }

    fn install_paint_global(&self, lua: &Lua) -> Result<()> {
        let table = lua.create_table();

        let bindings = self.clone();
        table.set(
            "new",
            lua.create_function(move |lua, ()| {
                let paint = bindings
                    .with_factory(|factory| Ok(ScriptedPaint::new(factory)))?
                    .with_context(bindings.clone());
                lua.create_userdata(paint)
            })?,
        )?;

        let bindings = self.clone();
        table.set(
            "with",
            lua.create_function(move |lua, definition: Table| {
                let mut paint = bindings
                    .with_factory(|factory| Ok(ScriptedPaint::new(factory)))?
                    .with_context(bindings.clone());
                paint.apply_definition(definition)?;
                lua.create_userdata(paint)
            })?,
        )?;

        table.set_readonly(true);
        lua.globals().set("Paint", table)?;
        Ok(())
    }

    fn with_factory<R>(&self, f: impl FnOnce(&mut dyn RenderFactory) -> Result<R>) -> Result<R> {
        let Some(mut factory) = self.factory.get() else {
            return Err(Error::runtime(
                "Paint allocation requires an active scripted draw context",
            ));
        };
        // The pointer is installed only for the duration of call_draw, and the
        // guard restores it before any borrowed factory can be dropped.
        unsafe { f(factory.as_mut()) }
    }

    pub(crate) fn create_scripted_artboard(
        &self,
        lua: &Lua,
        artboard: Box<dyn ScriptArtboard>,
    ) -> Result<AnyUserData> {
        lua.create_userdata(ScriptedArtboard::new(artboard, self.clone()))
    }
}

fn erase_factory_lifetime(factory: &mut dyn RenderFactory) -> NonNull<dyn RenderFactory> {
    let ptr: NonNull<dyn RenderFactory + '_> = NonNull::from(factory);
    // The pointer is restored by FactoryContextGuard before the scoped factory
    // context returns. Paint.new/with closures may run only within that scope.
    unsafe { mem::transmute::<NonNull<dyn RenderFactory + '_>, NonNull<dyn RenderFactory>>(ptr) }
}

fn erase_renderer_lifetime(renderer: &mut dyn Renderer) -> NonNull<dyn Renderer> {
    let ptr: NonNull<dyn Renderer + '_> = NonNull::from(renderer);
    // The pointer is held only by userdata created for one draw call; `valid`
    // is cleared before `call_draw` returns.
    unsafe { mem::transmute::<NonNull<dyn Renderer + '_>, NonNull<dyn Renderer>>(ptr) }
}

struct FactoryContextGuard {
    factory: Rc<Cell<Option<NonNull<dyn RenderFactory>>>>,
    previous: Option<NonNull<dyn RenderFactory>>,
}

impl Drop for FactoryContextGuard {
    fn drop(&mut self) {
        self.factory.set(self.previous);
    }
}

struct ScriptedArtboardOwner {
    artboard: RefCell<Box<dyn ScriptArtboard>>,
    _registration: Option<ScriptViewModelRegistration>,
}

struct ScriptedArtboard {
    owner: Rc<ScriptedArtboardOwner>,
    bindings: RendererBindings,
}

struct ScriptedAnimation {
    owner: Rc<ScriptedArtboardOwner>,
    animation: ScriptAnimation,
}

impl UserData for ScriptedAnimation {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("duration", |_, this| Ok(this.animation.duration()));
    }

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut("advance", |_, this, seconds: f32| {
            this.owner
                .artboard
                .borrow_mut()
                .advance_animation(&mut this.animation, seconds)
                .map_err(|error| Error::runtime(error.to_string()))
        });
        for (name, mode) in [
            ("setTime", ScriptAnimationTime::Seconds),
            ("setTimeFrames", ScriptAnimationTime::Frames),
            ("setTimePercentage", ScriptAnimationTime::Percentage),
        ] {
            methods.add_method_mut(name, move |_, this, value: f32| {
                this.owner
                    .artboard
                    .borrow_mut()
                    .set_animation_time(&mut this.animation, value, mode)
                    .map_err(|error| Error::runtime(error.to_string()))
            });
        }
    }
}

impl ScriptedArtboard {
    fn new(artboard: Box<dyn ScriptArtboard>, bindings: RendererBindings) -> Self {
        let registration = artboard
            .data()
            .as_ref()
            .map(|model| bindings.view_model_frame_context.register(model));
        Self {
            owner: Rc::new(ScriptedArtboardOwner {
                artboard: RefCell::new(artboard),
                _registration: registration,
            }),
            bindings,
        }
    }
}

impl UserData for ScriptedArtboard {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("width", |_, this| Ok(this.owner.artboard.borrow().width()));
        fields.add_field_method_set("width", |_, this, value: f32| {
            this.owner.artboard.borrow_mut().set_width(value);
            Ok(())
        });
        fields.add_field_method_get("height", |_, this| {
            Ok(this.owner.artboard.borrow().height())
        });
        fields.add_field_method_set("height", |_, this, value: f32| {
            this.owner.artboard.borrow_mut().set_height(value);
            Ok(())
        });
        fields.add_field_method_get("frameOrigin", |_, this| {
            Ok(this.owner.artboard.borrow().frame_origin())
        });
        fields.add_field_method_set("frameOrigin", |_, this, value: bool| {
            this.owner.artboard.borrow_mut().set_frame_origin(value);
            Ok(())
        });
        fields.add_field_method_get("data", |lua, this| {
            Ok(match this.owner.artboard.borrow().data() {
                Some(model) => Value::Table(create_scripted_view_model(lua, model)?),
                None => Value::Nil,
            })
        });
    }

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("instance", |lua, this, view_model: Option<Table>| {
            let view_model = view_model.as_ref().map(model_from_table).transpose()?;
            let instance = this
                .owner
                .artboard
                .borrow()
                .instance(view_model)
                .map_err(|error| Error::runtime(error.to_string()))?;
            lua.create_userdata(ScriptedArtboard::new(instance, this.bindings.clone()))
        });
        methods.add_method_mut("advance", |_, this, seconds: f32| {
            this.owner
                .artboard
                .borrow_mut()
                .advance(seconds)
                .map_err(|error| Error::runtime(error.to_string()))
        });
        methods.add_method("animation", |lua, this, name: String| {
            let animation = this
                .owner
                .artboard
                .borrow()
                .animation(&name)
                .map_err(|error| Error::runtime(error.to_string()))?;
            Ok(match animation {
                Some(animation) => Value::UserData(lua.create_userdata(ScriptedAnimation {
                    owner: Rc::clone(&this.owner),
                    animation,
                })?),
                None => Value::Nil,
            })
        });
        methods.add_method("node", |lua, this, name: String| {
            let node = this
                .owner
                .artboard
                .borrow()
                .node(&name)
                .map_err(|error| Error::runtime(error.to_string()))?;
            Ok(match node {
                Some(node) => Value::UserData(lua.create_userdata(ScriptedNode::new(node))?),
                None => Value::Nil,
            })
        });
        methods.add_method_mut("draw", |_, this, args: MultiValue| {
            let arg_types = args
                .iter()
                .map(|value| match value {
                    Value::UserData(userdata) if userdata.borrow::<ScriptedRenderer>().is_ok() => {
                        "Renderer"
                    }
                    Value::UserData(userdata) if userdata.borrow::<ScriptedArtboard>().is_ok() => {
                        "ScriptedArtboard"
                    }
                    other => other.type_name(),
                })
                .collect::<Vec<_>>()
                .join(",");
            let renderer = args
                .into_iter()
                .filter_map(|value| match value {
                    Value::UserData(userdata) if userdata.borrow::<ScriptedRenderer>().is_ok() => {
                        Some(userdata)
                    }
                    _ => None,
                })
                .next()
                .ok_or_else(|| {
                    Error::runtime(format!(
                        "ScriptedArtboard.draw expected Renderer userdata, got [{arg_types}]"
                    ))
                })?;
            let scripted_renderer = renderer.borrow::<ScriptedRenderer>()?;
            scripted_renderer.bindings.with_factory(|factory| {
                let mut renderer_ref = scripted_renderer.renderer_mut()?;
                this.owner
                    .artboard
                    .borrow_mut()
                    .draw(factory, unsafe { renderer_ref.as_mut() })
                    .map_err(|error| Error::runtime(error.to_string()))
            })
        });
    }
}

struct ScriptedRenderer {
    renderer: Rc<RefCell<NonNull<dyn Renderer>>>,
    bindings: RendererBindings,
    save_count: Rc<Cell<usize>>,
    valid: Rc<Cell<bool>>,
}

impl ScriptedRenderer {
    fn validate(&self) -> Result<()> {
        if self.valid.get() {
            Ok(())
        } else {
            Err(Error::runtime("Renderer is no longer valid"))
        }
    }

    fn renderer_mut(&self) -> Result<std::cell::RefMut<'_, NonNull<dyn Renderer>>> {
        self.validate()?;
        Ok(self.renderer.borrow_mut())
    }

    fn save(&self) -> Result<()> {
        let mut renderer = self.renderer_mut()?;
        unsafe { renderer.as_mut().save() };
        self.save_count.set(self.save_count.get() + 1);
        Ok(())
    }

    fn restore(&self) -> Result<()> {
        if self.save_count.get() == 0 {
            return Err(Error::runtime("Renderer save/restore stack was unbalanced"));
        }
        let mut renderer = self.renderer_mut()?;
        unsafe { renderer.as_mut().restore() };
        self.save_count.set(self.save_count.get() - 1);
        Ok(())
    }
}

impl UserData for ScriptedRenderer {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("save", |_, this, ()| this.save());
        methods.add_method("restore", |_, this, ()| this.restore());
        methods.add_method("transform", |_, this, matrix: AnyUserData| {
            this.validate()?;
            let matrix = matrix.borrow::<ScriptedMat2D>()?;
            let mut renderer = this.renderer_mut()?;
            unsafe { renderer.as_mut().transform(matrix.0) };
            Ok(())
        });
        methods.add_method("clipPath", |_, this, path: AnyUserData| {
            this.validate()?;
            let mut path = path.borrow_mut::<ScriptedPath>()?;
            this.bindings.with_factory(|factory| {
                let render_path = path.render_path(factory);
                let mut renderer = this.renderer_mut()?;
                unsafe { renderer.as_mut().clip_path(render_path) };
                Ok(())
            })
        });
        methods.add_method(
            "drawPath",
            |_, this, (path, paint): (AnyUserData, AnyUserData)| {
                this.validate()?;
                let mut path = path.borrow_mut::<ScriptedPath>()?;
                let paint = paint.borrow::<ScriptedPaint>()?;
                this.bindings.with_factory(|factory| {
                    let render_path = path.render_path(factory);
                    let mut renderer = this.renderer_mut()?;
                    unsafe {
                        renderer
                            .as_mut()
                            .draw_path(render_path, paint.render_paint.as_ref())
                    };
                    Ok(())
                })
            },
        );
    }
}

struct ScriptedPath {
    raw_path: RawPath,
    dirty: bool,
    render_path: Option<Box<dyn RenderPath>>,
}

impl ScriptedPath {
    fn new() -> Self {
        Self {
            raw_path: RawPath::new(),
            dirty: true,
            render_path: None,
        }
    }

    fn from_raw_path(raw_path: RawPath) -> Self {
        Self {
            raw_path,
            dirty: true,
            render_path: None,
        }
    }

    fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    fn render_path(&mut self, factory: &mut dyn RenderFactory) -> &dyn RenderPath {
        if self.dirty {
            self.dirty = false;
            if let Some(path) = self.render_path.as_mut() {
                path.rewind();
            } else {
                let mut path = factory.make_empty_render_path();
                path.fill_rule(FillRule::Clockwise);
                self.render_path = Some(path);
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

fn create_scripted_path(lua: &Lua, path: ScriptedPath) -> Result<AnyUserData> {
    let userdata = lua.create_userdata(path)?;
    let metatable = userdata_metatable(lua, userdata.clone())?;
    let methods: Table = metatable.get("__index")?;
    let index =
        lua.create_function(
            move |lua, (userdata, key): (AnyUserData, Value)| match lua_path_index(&key)? {
                Some(index) => {
                    let command = {
                        let path = userdata.borrow::<ScriptedPath>()?;
                        path.command(index)
                    };
                    create_scripted_path_command(lua, command).map(Value::UserData)
                }
                None => methods.get(key),
            },
        )?;
    metatable.set("__index", index)?;
    metatable.set_readonly(true);
    Ok(userdata)
}

fn create_scripted_path_command(lua: &Lua, command: ScriptedPathCommand) -> Result<AnyUserData> {
    let userdata = lua.create_userdata(command)?;
    let metatable = userdata_metatable(lua, userdata.clone())?;
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
    Ok(userdata)
}

struct ScriptedNode {
    path: Option<RawPath>,
    paint: Option<RuntimeScriptPaint>,
}

impl ScriptedNode {
    fn new(node: ScriptNode) -> Self {
        Self {
            path: node.path,
            paint: node.paint,
        }
    }
}

impl UserData for ScriptedNode {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("asPath", |lua, this, ()| {
            Ok(match this.path.clone() {
                Some(path) => Value::UserData(create_scripted_path(
                    lua,
                    ScriptedPath::from_raw_path(path),
                )?),
                None => Value::Nil,
            })
        });
        methods.add_method("asPaint", |lua, this, ()| {
            Ok(match this.paint {
                Some(paint) => Value::UserData(lua.create_userdata(ScriptedPaintData(paint))?),
                None => Value::Nil,
            })
        });
    }
}

struct ScriptedPaintData(RuntimeScriptPaint);

impl UserData for ScriptedPaintData {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("style", |_, this| Ok(style_name(this.0.style)));
        fields.add_field_method_get("join", |_, this| Ok(join_name(this.0.join)));
        fields.add_field_method_get("cap", |_, this| Ok(cap_name(this.0.cap)));
        fields.add_field_method_get("thickness", |_, this| Ok(this.0.thickness));
        fields.add_field_method_get("blendMode", |_, this| {
            Ok(blend_mode_name(this.0.blend_mode))
        });
        fields.add_field_method_get("feather", |_, this| Ok(this.0.feather));
        fields.add_field_method_get("color", |_, this| Ok(this.0.color));
    }
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
    let node = lua.create_userdata(ScriptedNode::new(node))?;
    let output: AnyUserData = function.call((table.clone(), source, node))?;
    let output = output.borrow::<ScriptedPath>()?;
    Ok(output.raw_path.clone())
}

fn install_path_global(lua: &Lua) -> Result<()> {
    let table = lua.create_table();
    table.set(
        "new",
        lua.create_function(|lua, ()| create_scripted_path(lua, ScriptedPath::new()))?,
    )?;
    table.set_readonly(true);
    lua.globals().set("Path", table)?;
    Ok(())
}

struct ScriptedPaint {
    context: RendererBindings,
    render_paint: Box<dyn RenderPaintTrait>,
    style: RenderPaintStyle,
    color: ColorInt,
    thickness: f32,
    join: StrokeJoin,
    cap: StrokeCap,
    feather: f32,
    blend_mode: BlendMode,
}

impl ScriptedPaint {
    fn new(factory: &mut dyn RenderFactory) -> Self {
        Self {
            context: RendererBindings::default(),
            render_paint: factory.make_render_paint(),
            style: RenderPaintStyle::Fill,
            color: 0xff000000,
            thickness: 1.0,
            join: StrokeJoin::Miter,
            cap: StrokeCap::Butt,
            feather: 0.0,
            blend_mode: BlendMode::SrcOver,
        }
    }

    fn with_context(mut self, context: RendererBindings) -> Self {
        self.context = context;
        self
    }

    fn copy_from(factory: &mut dyn RenderFactory, source: &Self) -> Self {
        let mut copy = Self::new(factory).with_context(source.context.clone());
        copy.set_style(source.style);
        copy.set_color(source.color);
        copy.set_thickness(source.thickness);
        copy.set_join(source.join);
        copy.set_cap(source.cap);
        copy.set_feather(source.feather);
        copy.set_blend_mode(source.blend_mode);
        copy
    }

    fn apply_definition(&mut self, definition: Table) -> Result<()> {
        for pair in definition.pairs::<String, Value>() {
            let (key, value) = pair?;
            self.apply_value(&key, value)?;
        }
        Ok(())
    }

    fn apply_value(&mut self, key: &str, value: Value) -> Result<()> {
        match key {
            "style" => self.set_style(parse_style(value)?),
            "join" => self.set_join(parse_join(value)?),
            "cap" => self.set_cap(parse_cap(value)?),
            "thickness" => self.set_thickness(number_value(value, "thickness")?),
            "blendMode" => self.set_blend_mode(parse_blend_mode(value)?),
            "feather" => self.set_feather(number_value(value, "feather")?),
            "gradient" => self.render_paint.shader(None),
            "color" => self.set_color(color_value(value)?),
            _ => {}
        }
        Ok(())
    }

    fn set_style(&mut self, style: RenderPaintStyle) {
        self.style = style;
        self.render_paint.style(style);
    }

    fn set_color(&mut self, color: ColorInt) {
        self.color = color;
        self.render_paint.color(color);
    }

    fn set_thickness(&mut self, thickness: f32) {
        self.thickness = thickness;
        self.render_paint.thickness(thickness);
    }

    fn set_join(&mut self, join: StrokeJoin) {
        self.join = join;
        self.render_paint.join(join);
    }

    fn set_cap(&mut self, cap: StrokeCap) {
        self.cap = cap;
        self.render_paint.cap(cap);
    }

    fn set_feather(&mut self, feather: f32) {
        self.feather = feather;
        self.render_paint.feather(feather);
    }

    fn set_blend_mode(&mut self, blend_mode: BlendMode) {
        self.blend_mode = blend_mode;
        self.render_paint.blend_mode(blend_mode);
    }
}

impl UserData for ScriptedPaint {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("style", |_, this| Ok(style_name(this.style)));
        fields.add_field_method_set("style", |_, this, value: Value| {
            this.set_style(parse_style(value)?);
            Ok(())
        });
        fields.add_field_method_get("join", |_, this| Ok(join_name(this.join)));
        fields.add_field_method_set("join", |_, this, value: Value| {
            this.set_join(parse_join(value)?);
            Ok(())
        });
        fields.add_field_method_get("cap", |_, this| Ok(cap_name(this.cap)));
        fields.add_field_method_set("cap", |_, this, value: Value| {
            this.set_cap(parse_cap(value)?);
            Ok(())
        });
        fields.add_field_method_get("thickness", |_, this| Ok(this.thickness));
        fields.add_field_method_set("thickness", |_, this, value: f32| {
            this.set_thickness(value);
            Ok(())
        });
        fields.add_field_method_get("blendMode", |_, this| Ok(blend_mode_name(this.blend_mode)));
        fields.add_field_method_set("blendMode", |_, this, value: Value| {
            this.set_blend_mode(parse_blend_mode(value)?);
            Ok(())
        });
        fields.add_field_method_get("feather", |_, this| Ok(this.feather));
        fields.add_field_method_set("feather", |_, this, value: f32| {
            this.set_feather(value);
            Ok(())
        });
        fields.add_field_method_get("color", |_, this| Ok(this.color));
        fields.add_field_method_set("color", |_, this, value: Value| {
            this.set_color(color_value(value)?);
            Ok(())
        });
    }

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("copy", |lua, this, definition: Option<Table>| {
            let mut copy = this
                .context
                .with_factory(|factory| Ok(ScriptedPaint::copy_from(factory, this)))?;
            if let Some(definition) = definition {
                copy.apply_definition(definition)?;
            }
            lua.create_userdata(copy)
        });
    }
}

fn install_color_global(lua: &Lua) -> Result<()> {
    let table = lua.create_table();

    for (name, shift) in [("red", 16), ("green", 8), ("blue", 0), ("alpha", 24)] {
        table.set(
            name,
            lua.create_function(move |lua, args: MultiValue| {
                let color = required_unsigned(lua, args.front(), "color")?;
                let replacement = optional_unsigned(lua, args.get(1))?;
                Ok(match replacement {
                    Some(component) => replace_color_component(color, shift, component),
                    None => color_component(color, shift),
                })
            })?,
        )?;
    }

    table.set(
        "opacity",
        lua.create_function(|lua, args: MultiValue| {
            let color = required_unsigned(lua, args.front(), "color")?;
            Ok(match optional_number(lua, args.get(1))? {
                Some(opacity) => {
                    Value::Integer(
                        replace_color_component(color, 24, opacity_to_alpha(opacity)) as i64,
                    )
                }
                None => Value::Number((color_component(color, 24) as f32 / 255.0) as f64),
            })
        })?,
    )?;

    table.set(
        "lerp",
        lua.create_function(|lua, args: MultiValue| {
            let from = required_unsigned(lua, args.front(), "from color")?;
            let to = required_unsigned(lua, args.get(1), "to color")?;
            let mix = required_number(lua, args.get(2), "mix")?;
            Ok(color_lerp(from, to, mix) as f64)
        })?,
    )?;

    table.set(
        "rgb",
        lua.create_function(|lua, args: MultiValue| {
            let red = required_unsigned(lua, args.front(), "red")?;
            let green = required_unsigned(lua, args.get(1), "green")?;
            let blue = required_unsigned(lua, args.get(2), "blue")?;
            Ok(rgba(red, green, blue, 255))
        })?,
    )?;
    table.set(
        "rgba",
        lua.create_function(|lua, args: MultiValue| {
            let red = required_unsigned(lua, args.front(), "red")?;
            let green = required_unsigned(lua, args.get(1), "green")?;
            let blue = required_unsigned(lua, args.get(2), "blue")?;
            let alpha = required_unsigned(lua, args.get(3), "alpha")?;
            Ok(rgba(red, green, blue, alpha))
        })?,
    )?;

    table.set(
        "toFloat",
        lua.create_function(|lua, args: MultiValue| {
            let color = required_unsigned(lua, args.front(), "color")?;
            let components = lua.create_table();
            components.set(1, color_component(color, 16) as f64 / 255.0)?;
            components.set(2, color_component(color, 8) as f64 / 255.0)?;
            components.set(3, color_component(color, 0) as f64 / 255.0)?;
            components.set(4, color_component(color, 24) as f64 / 255.0)?;
            Ok(components)
        })?,
    )?;

    table.set_readonly(true);
    lua.globals().set("Color", table)?;
    Ok(())
}

fn rgba(red: u32, green: u32, blue: u32, alpha: u32) -> ColorInt {
    ((alpha & 0xff) << 24) | ((red & 0xff) << 16) | ((green & 0xff) << 8) | (blue & 0xff)
}

fn required_unsigned(lua: &Lua, value: Option<&Value>, name: &str) -> Result<u32> {
    let value = value
        .cloned()
        .ok_or_else(|| Error::runtime(format!("expected numeric {name}")))?;
    lua.coerce_number(value)?
        .map(|value| (value as i64) as u32)
        .ok_or_else(|| Error::runtime(format!("expected numeric {name}")))
}

fn optional_unsigned(lua: &Lua, value: Option<&Value>) -> Result<Option<u32>> {
    value
        .cloned()
        .map(|value| {
            lua.coerce_number(value)
                .map(|value| value.map(|value| (value as i64) as u32))
        })
        .transpose()
        .map(Option::flatten)
}

fn required_number(lua: &Lua, value: Option<&Value>, name: &str) -> Result<f32> {
    let value = value
        .cloned()
        .ok_or_else(|| Error::runtime(format!("expected numeric {name}")))?;
    lua.coerce_number(value)?
        .map(|value| value as f32)
        .ok_or_else(|| Error::runtime(format!("expected numeric {name}")))
}

fn optional_number(lua: &Lua, value: Option<&Value>) -> Result<Option<f32>> {
    value
        .cloned()
        .map(|value| {
            lua.coerce_number(value)
                .map(|value| value.map(|value| value as f32))
        })
        .transpose()
        .map(Option::flatten)
}

fn color_component(color: ColorInt, shift: u32) -> u32 {
    (color >> shift) & 0xff
}

fn replace_color_component(color: ColorInt, shift: u32, component: u32) -> ColorInt {
    (color & !(0xff << shift)) | ((component & 0xff) << shift)
}

fn opacity_to_alpha(opacity: f32) -> u32 {
    // Keep the comparison order from C++ std::min/std::max, including its
    // behavior for NaN, before applying std::lround-equivalent rounding.
    let opacity = if opacity < 1.0 { opacity } else { 1.0 };
    let opacity = if 0.0 < opacity { opacity } else { 0.0 };
    (255.0 * opacity).round() as u32
}

fn color_lerp(from: ColorInt, to: ColorInt, mix: f32) -> ColorInt {
    fn lerp_component(from: u32, to: u32, mix: f32) -> u32 {
        let value = from as f32 * (1.0 - mix) + to as f32 * mix;
        let value = if value < 255.0 { value } else { 255.0 };
        let value = if 0.0 < value { value } else { 0.0 };
        value.round() as u32
    }

    rgba(
        lerp_component(color_component(from, 16), color_component(to, 16), mix),
        lerp_component(color_component(from, 8), color_component(to, 8), mix),
        lerp_component(color_component(from, 0), color_component(to, 0), mix),
        lerp_component(color_component(from, 24), color_component(to, 24), mix),
    )
}

#[derive(Clone, Copy)]
struct ScriptedMat2D(Mat2D);

impl UserData for ScriptedMat2D {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        for (name, index) in [
            ("xx", 0usize),
            ("yx", 1),
            ("xy", 2),
            ("yy", 3),
            ("tx", 4),
            ("ty", 5),
        ] {
            fields.add_field_method_get(name, move |_, this| Ok(this.0.0[index]));
            fields.add_field_method_set(name, move |_, this, value: f32| {
                this.0.0[index] = value;
                Ok(())
            });
        }
    }

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_meta_method("__mul", |lua, this, rhs: Value| {
            Ok(match rhs {
                Value::Vector(vector) => {
                    let point = this
                        .0
                        .transform_point(nuxie_render_api::Vec2D::new(vector.x(), vector.y()));
                    Value::Vector(LuaVector::new(point.x, point.y, 0.0))
                }
                Value::UserData(rhs) => {
                    let rhs = rhs.borrow::<ScriptedMat2D>()?;
                    Value::UserData(
                        lua.create_userdata(ScriptedMat2D(multiply_mat2d(this.0, rhs.0)))?,
                    )
                }
                _ => return Err(Error::runtime("Mat2D can multiply a Vector or Mat2D")),
            })
        });
    }
}

fn multiply_mat2d(lhs: Mat2D, rhs: Mat2D) -> Mat2D {
    let a = lhs.0;
    let b = rhs.0;
    Mat2D([
        a[0].mul_add(b[0], a[2] * b[1]),
        a[1].mul_add(b[0], a[3] * b[1]),
        a[0].mul_add(b[2], a[2] * b[3]),
        a[1].mul_add(b[2], a[3] * b[3]),
        a[0].mul_add(b[4], a[2] * b[5]) + a[4],
        a[1].mul_add(b[4], a[3] * b[5]) + a[5],
    ])
}

fn install_mat2d_global(lua: &Lua) -> Result<()> {
    let table = lua.create_table();
    table.set(
        "identity",
        lua.create_function(|lua, ()| lua.create_userdata(ScriptedMat2D(Mat2D::IDENTITY)))?,
    )?;
    table.set(
        "values",
        lua.create_function(
            |lua, (xx, yx, xy, yy, tx, ty): (f32, f32, f32, f32, f32, f32)| {
                lua.create_userdata(ScriptedMat2D(Mat2D([xx, yx, xy, yy, tx, ty])))
            },
        )?,
    )?;
    table.set(
        "withTranslation",
        lua.create_function(|lua, args: MultiValue| {
            let (x, y) = vec_or_numbers(&args)?;
            lua.create_userdata(ScriptedMat2D(Mat2D([1.0, 0.0, 0.0, 1.0, x, y])))
        })?,
    )?;
    table.set(
        "withScale",
        lua.create_function(|lua, args: MultiValue| {
            let (x, y) = vec_or_numbers_or_uniform(&args)?;
            lua.create_userdata(ScriptedMat2D(Mat2D([x, 0.0, 0.0, y, 0.0, 0.0])))
        })?,
    )?;
    table.set(
        "withScaleAndTranslation",
        lua.create_function(|lua, args: MultiValue| {
            let (sx, sy, tx, ty) = scale_translation_args(&args)?;
            lua.create_userdata(ScriptedMat2D(Mat2D([sx, 0.0, 0.0, sy, tx, ty])))
        })?,
    )?;
    table.set(
        "withRotation",
        lua.create_function(|lua, radians: f32| {
            let c = radians.cos();
            let s = radians.sin();
            lua.create_userdata(ScriptedMat2D(Mat2D([c, s, -s, c, 0.0, 0.0])))
        })?,
    )?;
    table.set_readonly(true);
    lua.globals().set("Mat2D", table)?;
    Ok(())
}

fn vec_or_numbers(args: &MultiValue) -> Result<(f32, f32)> {
    match args.front() {
        Some(Value::Vector(value)) => Ok((value.x(), value.y())),
        Some(Value::Integer(x)) => Ok((*x as f32, number_arg(args.get(1), "y")?)),
        Some(Value::Number(x)) => Ok((*x as f32, number_arg(args.get(1), "y")?)),
        _ => Err(Error::runtime("expected Vector or x/y numbers")),
    }
}

fn vec_or_numbers_or_uniform(args: &MultiValue) -> Result<(f32, f32)> {
    match args.front() {
        Some(Value::Vector(value)) => Ok((value.x(), value.y())),
        Some(Value::Integer(x)) => Ok((
            *x as f32,
            number_arg(args.get(1), "scaleY").unwrap_or(*x as f32),
        )),
        Some(Value::Number(x)) => Ok((
            *x as f32,
            number_arg(args.get(1), "scaleY").unwrap_or(*x as f32),
        )),
        _ => Err(Error::runtime("expected Vector or scale numbers")),
    }
}

fn scale_translation_args(args: &MultiValue) -> Result<(f32, f32, f32, f32)> {
    match (args.front(), args.get(1)) {
        (Some(Value::Vector(scale)), Some(Value::Vector(translation))) => {
            Ok((scale.x(), scale.y(), translation.x(), translation.y()))
        }
        _ => Ok((
            number_arg(args.front(), "scaleX")?,
            number_arg(args.get(1), "scaleY")?,
            number_arg(args.get(2), "translationX")?,
            number_arg(args.get(3), "translationY")?,
        )),
    }
}

fn number_arg(value: Option<&Value>, name: &str) -> Result<f32> {
    match value {
        Some(Value::Integer(value)) => Ok(*value as f32),
        Some(Value::Number(value)) => Ok(*value as f32),
        _ => Err(Error::runtime(format!("expected numeric {name}"))),
    }
}

fn number_value(value: Value, name: &str) -> Result<f32> {
    match value {
        Value::Integer(value) => Ok(value as f32),
        Value::Number(value) => Ok(value as f32),
        _ => Err(Error::runtime(format!("expected numeric {name}"))),
    }
}

fn color_value(value: Value) -> Result<ColorInt> {
    match value {
        Value::Integer(value) => Ok(value as ColorInt),
        Value::Number(value) => Ok(value as ColorInt),
        _ => Err(Error::runtime("expected numeric color")),
    }
}

fn parse_style(value: Value) -> Result<RenderPaintStyle> {
    match string_value(value)?.as_str() {
        "stroke" => Ok(RenderPaintStyle::Stroke),
        "fill" => Ok(RenderPaintStyle::Fill),
        other => Err(Error::runtime(format!(
            "'{other}' is not a valid PaintStyle"
        ))),
    }
}

fn style_name(style: RenderPaintStyle) -> &'static str {
    match style {
        RenderPaintStyle::Stroke => "stroke",
        RenderPaintStyle::Fill => "fill",
    }
}

fn parse_join(value: Value) -> Result<StrokeJoin> {
    match string_value(value)?.as_str() {
        "miter" => Ok(StrokeJoin::Miter),
        "round" => Ok(StrokeJoin::Round),
        "bevel" => Ok(StrokeJoin::Bevel),
        other => Err(Error::runtime(format!(
            "'{other}' is not a valid StrokeJoin"
        ))),
    }
}

fn join_name(join: StrokeJoin) -> &'static str {
    match join {
        StrokeJoin::Miter => "miter",
        StrokeJoin::Round => "round",
        StrokeJoin::Bevel => "bevel",
    }
}

fn parse_cap(value: Value) -> Result<StrokeCap> {
    match string_value(value)?.as_str() {
        "butt" => Ok(StrokeCap::Butt),
        "round" => Ok(StrokeCap::Round),
        "square" => Ok(StrokeCap::Square),
        other => Err(Error::runtime(format!(
            "'{other}' is not a valid StrokeCap"
        ))),
    }
}

fn cap_name(cap: StrokeCap) -> &'static str {
    match cap {
        StrokeCap::Butt => "butt",
        StrokeCap::Round => "round",
        StrokeCap::Square => "square",
    }
}

fn parse_blend_mode(value: Value) -> Result<BlendMode> {
    match string_value(value)?.as_str() {
        "srcOver" => Ok(BlendMode::SrcOver),
        "screen" => Ok(BlendMode::Screen),
        "overlay" => Ok(BlendMode::Overlay),
        "darken" => Ok(BlendMode::Darken),
        "lighten" => Ok(BlendMode::Lighten),
        "colorDodge" => Ok(BlendMode::ColorDodge),
        "colorBurn" => Ok(BlendMode::ColorBurn),
        "hardLight" => Ok(BlendMode::HardLight),
        "softLight" => Ok(BlendMode::SoftLight),
        "difference" => Ok(BlendMode::Difference),
        "exclusion" => Ok(BlendMode::Exclusion),
        "multiply" => Ok(BlendMode::Multiply),
        "hue" => Ok(BlendMode::Hue),
        "saturation" => Ok(BlendMode::Saturation),
        "color" => Ok(BlendMode::Color),
        "luminosity" => Ok(BlendMode::Luminosity),
        other => Err(Error::runtime(format!(
            "'{other}' is not a valid BlendMode"
        ))),
    }
}

fn blend_mode_name(blend_mode: BlendMode) -> &'static str {
    match blend_mode {
        BlendMode::SrcOver => "srcOver",
        BlendMode::Screen => "screen",
        BlendMode::Overlay => "overlay",
        BlendMode::Darken => "darken",
        BlendMode::Lighten => "lighten",
        BlendMode::ColorDodge => "colorDodge",
        BlendMode::ColorBurn => "colorBurn",
        BlendMode::HardLight => "hardLight",
        BlendMode::SoftLight => "softLight",
        BlendMode::Difference => "difference",
        BlendMode::Exclusion => "exclusion",
        BlendMode::Multiply => "multiply",
        BlendMode::Hue => "hue",
        BlendMode::Saturation => "saturation",
        BlendMode::Color => "color",
        BlendMode::Luminosity => "luminosity",
    }
}

fn string_value(value: Value) -> Result<String> {
    match value {
        Value::String(value) => Ok(value.to_str()?),
        _ => Err(Error::runtime("expected string")),
    }
}

#[cfg(test)]
mod matrix_tests {
    use super::*;

    #[test]
    fn matrix_multiplication_matches_cpp_contraction_order() {
        let matrix = Mat2D([0.8660254, 0.5, -0.5, 0.8660254, 12.124355, 7.0]);
        let result = multiply_mat2d(matrix, matrix);

        assert_eq!(
            result.0.map(f32::to_bits),
            [
                0x3eff_ffff,
                0x3f5d_b3d7,
                0xbf5d_b3d7,
                0x3f00_0000,
                0x4198_feae,
                0x4198_feae,
            ]
        );
    }
}

#[cfg(test)]
mod color_tests {
    use super::*;

    fn color_lua() -> Lua {
        let lua = Lua::new();
        install_color_global(&lua).expect("Color global installs");
        lua
    }

    #[test]
    fn color_construction_and_component_overloads_match_cpp() {
        let lua = color_lua();
        let result: Table = lua
            .load(
                r#"
                local original = Color.rgba(225, 48, 108, 255)
                local red = Color.red(original, 129)
                local green = Color.green(original, 129)
                local blue = Color.blue(original, 129)
                local alpha = Color.alpha(original, 129)
                local wrapped = Color.red(original, -1)
                local truncated = Color.green(original, 129.9)
                return {
                    white = Color.rgba(255, 255, 255, 255),
                    yellow = Color.rgba(255, 255, 0, 255),
                    opaqueRed = Color.rgb(255, 0, 0),
                    original = original,
                    red = Color.red(original),
                    redSet = Color.red(red),
                    green = Color.green(original, nil),
                    greenSet = Color.green(green),
                    blue = Color.blue(original, false),
                    blueSet = Color.blue(blue),
                    alpha = Color.alpha(original),
                    alphaSet = Color.alpha(alpha),
                    wrapped = Color.red(wrapped),
                    truncated = Color.green(truncated),
                }
                "#,
            )
            .eval()
            .expect("Color component script runs");

        assert_eq!(result.get::<u32>("white").unwrap(), 0xffff_ffff);
        assert_eq!(result.get::<u32>("yellow").unwrap(), 0xffff_ff00);
        assert_eq!(result.get::<u32>("opaqueRed").unwrap(), 0xffff_0000);
        assert_eq!(result.get::<u32>("original").unwrap(), 0xffe1_306c);
        assert_eq!(result.get::<u32>("red").unwrap(), 225);
        assert_eq!(result.get::<u32>("redSet").unwrap(), 129);
        assert_eq!(result.get::<u32>("green").unwrap(), 48);
        assert_eq!(result.get::<u32>("greenSet").unwrap(), 129);
        assert_eq!(result.get::<u32>("blue").unwrap(), 108);
        assert_eq!(result.get::<u32>("blueSet").unwrap(), 129);
        assert_eq!(result.get::<u32>("alpha").unwrap(), 255);
        assert_eq!(result.get::<u32>("alphaSet").unwrap(), 129);
        assert_eq!(result.get::<u32>("wrapped").unwrap(), 255);
        assert_eq!(result.get::<u32>("truncated").unwrap(), 129);
    }

    #[test]
    fn color_opacity_lerp_and_float_conversion_match_cpp() {
        let lua = color_lua();
        let result: Table = lua
            .load(
                r#"
                local color = Color.rgba(225, 48, 108, 255)
                local sixtyPercent = Color.opacity(color, 0.6)
                local floats = Color.toFloat(Color.rgba(255, 128, 0, 64))
                return {
                    opaque = Color.opacity(color),
                    sixtyPercent = Color.opacity(sixtyPercent),
                    sixtyPercentAlpha = Color.alpha(sixtyPercent),
                    clampedLow = Color.alpha(Color.opacity(color, -1)),
                    clampedHigh = Color.alpha(Color.opacity(color, 2)),
                    halfway = Color.lerp(Color.rgb(0, 0, 0), Color.rgb(255, 255, 255), 0.5),
                    extrapolatedLow = Color.lerp(Color.rgb(64, 64, 64), Color.rgb(255, 255, 255), -1),
                    extrapolatedHigh = Color.lerp(Color.rgb(0, 0, 0), Color.rgb(192, 192, 192), 2),
                    floats = floats,
                }
                "#,
            )
            .eval()
            .expect("Color opacity/lerp script runs");

        assert_eq!(result.get::<f64>("opaque").unwrap(), 1.0);
        assert!((result.get::<f64>("sixtyPercent").unwrap() - 0.6).abs() < 1e-6);
        assert_eq!(result.get::<u32>("sixtyPercentAlpha").unwrap(), 153);
        assert_eq!(result.get::<u32>("clampedLow").unwrap(), 0);
        assert_eq!(result.get::<u32>("clampedHigh").unwrap(), 255);
        assert_eq!(result.get::<u32>("halfway").unwrap(), 0xff80_8080);
        assert_eq!(result.get::<u32>("extrapolatedLow").unwrap(), 0xff00_0000);
        assert_eq!(result.get::<u32>("extrapolatedHigh").unwrap(), 0xffff_ffff);

        let floats = result.get::<Table>("floats").unwrap();
        assert_eq!(floats.get::<f64>(1).unwrap(), 1.0);
        assert_eq!(floats.get::<f64>(2).unwrap(), 128.0 / 255.0);
        assert_eq!(floats.get::<f64>(3).unwrap(), 0.0);
        assert_eq!(floats.get::<f64>(4).unwrap(), 64.0 / 255.0);
    }
}

#[cfg(test)]
mod artboard_owner_tests {
    use super::*;
    use nuxie_runtime::{ScriptError, ScriptViewModel, ScriptViewModelProperty};

    struct TestScriptArtboard {
        model: ScriptViewModel,
    }

    impl ScriptArtboard for TestScriptArtboard {
        fn width(&self) -> f32 {
            0.0
        }

        fn height(&self) -> f32 {
            0.0
        }

        fn frame_origin(&self) -> bool {
            false
        }

        fn set_width(&mut self, _width: f32) {}

        fn set_height(&mut self, _height: f32) {}

        fn set_frame_origin(&mut self, _frame_origin: bool) {}

        fn data(&self) -> Option<ScriptViewModel> {
            Some(self.model.clone())
        }

        fn instance(
            &self,
            _view_model: Option<ScriptViewModel>,
        ) -> std::result::Result<Box<dyn ScriptArtboard>, ScriptError> {
            Err(ScriptError::new("not used by owner-lifetime test"))
        }

        fn draw(
            &mut self,
            _factory: &mut dyn RenderFactory,
            _renderer: &mut dyn Renderer,
        ) -> std::result::Result<(), ScriptError> {
            Ok(())
        }
    }

    fn trigger_model() -> (ScriptViewModel, String) {
        let fixture = std::env::var_os("RIVE_RUNTIME_DIR")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::path::PathBuf::from("/Users/levi/dev/oss/rive-runtime"))
            .join("tests/unit_tests/assets/script_create_viewmodel_instance.riv");
        let bytes = std::fs::read(&fixture)
            .unwrap_or_else(|error| panic!("missing fixture {}: {error}", fixture.display()));
        let file = nuxie_binary::read_runtime_file(&bytes).expect("fixture parses");
        nuxie_runtime::script_view_models(&file)
            .into_values()
            .find_map(|model| {
                let trigger = model.properties().iter().find_map(|(name, kind)| {
                    (*kind == ScriptViewModelProperty::Trigger).then(|| name.clone())
                })?;
                Some((model.named_instance(None)?, trigger))
            })
            .expect("fixture has a trigger model")
    }

    #[test]
    fn scripted_artboard_keeps_its_bound_instance_registered_for_its_lifetime() {
        let (model, trigger) = trigger_model();
        let context = ScriptViewModelFrameContext::default();
        let artboard = ScriptedArtboard::new(
            Box::new(TestScriptArtboard {
                model: model.clone(),
            }),
            RendererBindings::new(context.clone()),
        );

        assert!(model.fire_trigger(&trigger));
        assert!(context.advance_detached());
        assert_eq!(model.trigger(&trigger), Some(0));

        drop(artboard);
        assert!(model.fire_trigger(&trigger));
        assert!(!context.advance_detached());
        assert_eq!(model.trigger(&trigger), Some(1));
    }

    #[test]
    fn scripted_child_artboard_advance_does_not_consume_detached_view_models() {
        let (model, trigger) = trigger_model();
        let context = ScriptViewModelFrameContext::default();
        let artboard = ScriptedArtboard::new(
            Box::new(TestScriptArtboard {
                model: model.clone(),
            }),
            RendererBindings::new(context.clone()),
        );
        let lua = Lua::new();
        let userdata = lua
            .create_userdata(artboard)
            .expect("scripted artboard userdata");
        lua.globals()
            .set("child", userdata)
            .expect("publish scripted child");

        assert!(model.fire_trigger(&trigger));
        lua.load("child:advance(0)")
            .exec()
            .expect("child advance succeeds");
        assert_eq!(model.trigger(&trigger), Some(1));

        assert!(context.advance_detached());
        assert_eq!(model.trigger(&trigger), Some(0));
    }
}
