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
use rive_render_api::{
    BlendMode, ColorInt, Factory as RenderFactory, FillRule, Mat2D, RawPath,
    RenderPaint as RenderPaintTrait, RenderPaintStyle, RenderPath, Renderer, StrokeCap, StrokeJoin,
};
use rive_runtime::ScriptArtboard;

#[derive(Clone, Default)]
pub(crate) struct RendererBindings {
    factory: Rc<Cell<Option<NonNull<dyn RenderFactory>>>>,
}

impl RendererBindings {
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
        lua.create_userdata(ScriptedArtboard {
            artboard: Rc::new(RefCell::new(artboard)),
        })
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

struct ScriptedArtboard {
    artboard: Rc<RefCell<Box<dyn ScriptArtboard>>>,
}

impl ScriptedArtboard {
    fn new(artboard: Box<dyn ScriptArtboard>) -> Self {
        Self {
            artboard: Rc::new(RefCell::new(artboard)),
        }
    }
}

impl UserData for ScriptedArtboard {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("width", |_, this| Ok(this.artboard.borrow().width()));
        fields.add_field_method_set("width", |_, this, value: f32| {
            this.artboard.borrow_mut().set_width(value);
            Ok(())
        });
        fields.add_field_method_get("height", |_, this| Ok(this.artboard.borrow().height()));
        fields.add_field_method_set("height", |_, this, value: f32| {
            this.artboard.borrow_mut().set_height(value);
            Ok(())
        });
        fields.add_field_method_get("frameOrigin", |_, this| {
            Ok(this.artboard.borrow().frame_origin())
        });
        fields.add_field_method_set("frameOrigin", |_, this, value: bool| {
            this.artboard.borrow_mut().set_frame_origin(value);
            Ok(())
        });
    }

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("instance", |lua, this, ()| {
            let instance = this
                .artboard
                .borrow()
                .instance()
                .map_err(|error| Error::runtime(error.to_string()))?;
            lua.create_userdata(ScriptedArtboard::new(instance))
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
                this.artboard
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
}

impl UserData for ScriptedPath {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
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
    }
}

fn install_path_global(lua: &Lua) -> Result<()> {
    let table = lua.create_table();
    table.set(
        "new",
        lua.create_function(|lua, ()| lua.create_userdata(ScriptedPath::new()))?,
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
    table.set(
        "rgb",
        lua.create_function(|_, (red, green, blue): (u32, u32, u32)| {
            Ok(rgba(red, green, blue, 255))
        })?,
    )?;
    table.set(
        "rgba",
        lua.create_function(|_, (red, green, blue, alpha): (u32, u32, u32, u32)| {
            Ok(rgba(red, green, blue, alpha))
        })?,
    )?;
    table.set_readonly(true);
    lua.globals().set("Color", table)?;
    Ok(())
}

fn rgba(red: u32, green: u32, blue: u32, alpha: u32) -> ColorInt {
    ((alpha & 0xff) << 24) | ((red & 0xff) << 16) | ((green & 0xff) << 8) | (blue & 0xff)
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
