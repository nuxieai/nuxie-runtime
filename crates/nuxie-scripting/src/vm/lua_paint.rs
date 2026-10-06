// Translated from:
// /Users/levi/dev/oss/rive-runtime/src/lua/renderer/lua_paint.cpp
use std::{cell::RefCell, rc::Rc};

use luaur_rt::{Error, Lua, Result, Table, UserData, UserDataFields, UserDataMethods, Value};
use nuxie_render_api::{
    BlendMode, ColorInt, Factory as RenderFactory, RenderPaint as RenderPaintTrait,
    RenderPaintStyle, RenderShader, StrokeCap, StrokeJoin, Mat2D,
};
use nuxie_runtime::ScriptPaint as RuntimeScriptPaint;

use super::lua_renderer_library::RendererBindings;
use super::renderer::ScriptedGradient;
use super::lua_mat2d::ScriptedMat2D;

impl RendererBindings {
    pub(super) fn install_paint_global(&self, lua: &Lua) -> Result<()> {
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
}

pub(super) struct ScriptedPaintData(pub(super) RuntimeScriptPaint);

impl UserData for ScriptedPaintData {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("style", |_, this| Ok(style_name(this.0.style)));
        fields.add_field_method_get("join", |_, this| Ok(join_name(this.0.join)));
        fields.add_field_method_get("cap", |_, this| Ok(cap_name(this.0.cap)));
        fields.add_field_method_get("thickness", |_, this| Ok(this.0.thickness));
        fields.add_field_method_get("blendMode", |_, this| {
            Ok(blend_mode_name(this.0.blend_mode()))
        });
        fields.add_field_method_get("feather", |_, this| Ok(this.0.feather));
        fields.add_field_method_get("color", |_, this| Ok(this.0.color));
        fields.add_field_method_get("gradientTransform", |lua, _| lua.create_userdata(ScriptedMat2D(Mat2D::IDENTITY)));
    }
}

pub(super) struct ScriptedPaint {
    context: RendererBindings,
    pub(super) render_paint: Rc<RefCell<Box<dyn RenderPaintTrait>>>,
    style: RenderPaintStyle,
    color: ColorInt,
    thickness: f32,
    join: StrokeJoin,
    cap: StrokeCap,
    feather: f32,
    blend_mode: BlendMode,
    gradient: Option<Rc<dyn RenderShader>>,
    gradient_transform: Mat2D,
}

impl ScriptedPaint {
    fn new(factory: &mut dyn RenderFactory) -> Self {
        Self {
            context: RendererBindings::default(),
            render_paint: Rc::new(RefCell::new(factory.make_render_paint())),
            style: RenderPaintStyle::Fill,
            color: 0xff000000,
            thickness: 1.0,
            join: StrokeJoin::Miter,
            cap: StrokeCap::Butt,
            feather: 0.0,
            blend_mode: BlendMode::SrcOver,
            gradient: None,
            gradient_transform: Mat2D::IDENTITY,
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
        copy.set_gradient(source.gradient.clone());
        copy.set_gradient_transform(source.gradient_transform);
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
            "gradient" => self.set_gradient_value(value)?,
            "gradientTransform" => self.set_gradient_transform_value(value)?,
            "color" => self.set_color(color_value(value)?),
            _ => {}
        }
        Ok(())
    }

    fn set_style(&mut self, style: RenderPaintStyle) {
        self.style = style;
        self.render_paint.borrow_mut().style(style);
    }

    fn set_color(&mut self, color: ColorInt) {
        self.color = color;
        self.render_paint.borrow_mut().color(color);
    }

    fn set_thickness(&mut self, thickness: f32) {
        self.thickness = thickness;
        self.render_paint.borrow_mut().thickness(thickness);
    }

    fn set_join(&mut self, join: StrokeJoin) {
        self.join = join;
        self.render_paint.borrow_mut().join(join);
    }

    fn set_cap(&mut self, cap: StrokeCap) {
        self.cap = cap;
        self.render_paint.borrow_mut().cap(cap);
    }

    fn set_feather(&mut self, feather: f32) {
        self.feather = feather;
        self.render_paint.borrow_mut().feather(feather);
    }

    fn set_blend_mode(&mut self, blend_mode: BlendMode) {
        self.blend_mode = blend_mode;
        self.render_paint.borrow_mut().blend_mode(blend_mode);
        self.render_paint.borrow_mut().additiveness(
            nuxie_runtime::source::shapes::paint::blend_mode::additiveness_for(blend_mode, 255),
        );
    }

    fn set_gradient(&mut self, gradient: Option<Rc<dyn RenderShader>>) {
        self.gradient = gradient;
        self.render_paint.borrow_mut().shader(self.gradient.as_deref());
    }

    fn set_gradient_value(&mut self, value: Value) -> Result<()> {
        match value {
            Value::UserData(gradient) => {
                if gradient.is::<ScriptedGradient>() {
                    let gradient = Rc::clone(&gradient.borrow::<ScriptedGradient>()?.0);
                    self.set_gradient(Some(gradient));
                } else {
                    self.set_gradient(None);
                }
            }
            _ => self.set_gradient(None),
        }
        Ok(())
    }

    fn set_gradient_transform(&mut self, transform: Mat2D) {
        self.gradient_transform = transform;
        self.render_paint.borrow_mut().shader_transform(transform);
    }

    fn set_gradient_transform_value(&mut self, value: Value) -> Result<()> {
        let transform = match value {
            Value::UserData(value) if value.is::<ScriptedMat2D>() => value.borrow::<ScriptedMat2D>()?.0,
            _ => Mat2D::IDENTITY,
        };
        self.set_gradient_transform(transform);
        Ok(())
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
        fields.add_field_method_get("gradient", |lua, this| {
            Ok(match &this.gradient {
                Some(gradient) => {
                    Value::UserData(lua.create_userdata(ScriptedGradient(Rc::clone(gradient)))?)
                }
                None => Value::Nil,
            })
        });
        fields.add_field_method_set("gradient", |_, this, value: Value| {
            this.set_gradient_value(value)
        });
        fields.add_field_method_get("gradientTransform", |lua, this| lua.create_userdata(ScriptedMat2D(this.gradient_transform)));
        fields.add_field_method_set("gradientTransform", |_, this, value: Value| this.set_gradient_transform_value(value));
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

pub(super) fn style_name(style: RenderPaintStyle) -> &'static str {
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

pub(super) fn join_name(join: StrokeJoin) -> &'static str {
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

pub(super) fn cap_name(cap: StrokeCap) -> &'static str {
    match cap {
        StrokeCap::Butt => "butt",
        StrokeCap::Round => "round",
        StrokeCap::Square => "square",
    }
}

fn parse_blend_mode(value: Value) -> Result<BlendMode> {
    parse_blend_mode_name(&string_value(value)?)
}

pub(super) fn parse_blend_mode_name(value: &str) -> Result<BlendMode> {
    match value {
        "srcOver" => Ok(BlendMode::SrcOver),
        "additive" => Ok(BlendMode::Additive),
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

pub(super) fn blend_mode_name(blend_mode: BlendMode) -> &'static str {
    match blend_mode {
        BlendMode::SrcOver => "srcOver",
        BlendMode::Additive => "additive",
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
mod upstream_blend_mode_tests {
    use super::ScriptedPaint;
    use nuxie_render_api::*;
    use std::any::Any;

    struct RecordingPaint {
        blend: BlendMode,
        additive: f32,
    }

    impl RenderPaint for RecordingPaint {
        fn as_any(&self) -> &dyn Any {
            self
        }
        fn style(&mut self, _: RenderPaintStyle) {}
        fn color(&mut self, _: ColorInt) {}
        fn thickness(&mut self, _: f32) {}
        fn join(&mut self, _: StrokeJoin) {}
        fn cap(&mut self, _: StrokeCap) {}
        fn feather(&mut self, _: f32) {}
        fn additiveness(&mut self, value: f32) {
            self.additive = value;
        }
        fn blend_mode(&mut self, value: BlendMode) {
            self.blend = value;
        }
        fn shader(&mut self, _: Option<&dyn RenderShader>) {}
        fn shader_transform(&mut self, _: Mat2D) {}
        fn invalidate_stroke(&mut self) {}
    }

    struct RecordingFactory(NullFactory);
    impl Factory for RecordingFactory {
        fn make_render_buffer(
            &mut self,
            kind: RenderBufferType,
            flags: RenderBufferFlags,
            size: usize,
        ) -> Box<dyn RenderBuffer> {
            self.0.make_render_buffer(kind, flags, size)
        }
        fn make_linear_gradient(
            &mut self,
            sx: f32,
            sy: f32,
            ex: f32,
            ey: f32,
            colors: &[ColorInt],
            stops: &[f32],
        ) -> Box<dyn RenderShader> {
            self.0.make_linear_gradient(sx, sy, ex, ey, colors, stops)
        }
        fn make_radial_gradient(
            &mut self,
            cx: f32,
            cy: f32,
            radius: f32,
            colors: &[ColorInt],
            stops: &[f32],
        ) -> Box<dyn RenderShader> {
            self.0.make_radial_gradient(cx, cy, radius, colors, stops)
        }
        fn make_render_path(&mut self, path: RawPath, fill: FillRule) -> Box<dyn RenderPath> {
            self.0.make_render_path(path, fill)
        }
        fn make_empty_render_path(&mut self) -> Box<dyn RenderPath> {
            self.0.make_empty_render_path()
        }
        fn make_render_paint(&mut self) -> Box<dyn RenderPaint> {
            Box::new(RecordingPaint {
                blend: BlendMode::SrcOver,
                additive: 0.0,
            })
        }
        fn decode_image(&mut self, data: &[u8]) -> Result<Box<dyn RenderImage>, ImageDecodeError> {
            self.0.decode_image(data)
        }
    }

    fn recorded(paint: &ScriptedPaint) -> std::cell::Ref<'_, RecordingPaint> {
        std::cell::Ref::map(paint.render_paint.borrow(), |paint| {
            paint.as_any().downcast_ref::<RecordingPaint>().expect("recording render paint")
        })
    }

    // tests/unit_tests/runtime/blend_mode_test.cpp at 3d0d3f56.
    #[test]
    fn a_scripted_paint_sets_additiveness_alongside_the_mode() {
        let mut factory = RecordingFactory(NullFactory::new());
        let mut paint = ScriptedPaint::new(&mut factory);
        let _ = recorded(&paint);

        paint.set_blend_mode(BlendMode::Additive);
        assert_eq!(recorded(&paint).blend, BlendMode::Additive);
        assert!((recorded(&paint).additive - 1.0).abs() <= 100.0 * f32::EPSILON);

        paint.set_blend_mode(BlendMode::Multiply);
        assert_eq!(recorded(&paint).blend, BlendMode::Multiply);
        assert_eq!(recorded(&paint).additive, 0.0);

        paint.set_blend_mode(BlendMode::Additive);
        let clone = ScriptedPaint::copy_from(&mut factory, &paint);
        assert_eq!(recorded(&clone).blend, BlendMode::Additive);
        assert!((recorded(&clone).additive - 1.0).abs() <= 100.0 * f32::EPSILON);
    }
}
