#![cfg(feature = "rive_scripting")]

use crate::mechanical_port::source::{
    lua::rive_lua_libs::*,
    renderer::RenderPaintStyle,
    shapes::paint::{blend_mode::BlendMode, stroke_cap::StrokeCap, stroke_join::StrokeJoin},
};

fn read_style(state: &mut LuaState, paint: &mut ScriptedPaint, index: i32) {
    let (name, atom) = state.to_string_atom(index);
    if name.is_none() {
        state.type_error::<()>(index, state.type_name(LuaType::String));
    }
    match atom {
        LuaAtoms::Stroke => paint.set_style(RenderPaintStyle::Stroke),
        LuaAtoms::Fill => paint.set_style(RenderPaintStyle::Fill),
        _ => state.error::<()>(format!(
            "'{}' is not a valid PaintStyle",
            name.unwrap_or_default()
        )),
    }
}

fn read_join(state: &mut LuaState, paint: &mut ScriptedPaint, index: i32) {
    let (name, atom) = state.to_string_atom(index);
    if name.is_none() {
        state.type_error::<()>(index, state.type_name(LuaType::String));
    }
    match atom {
        LuaAtoms::Miter => paint.set_join(StrokeJoin::Miter),
        LuaAtoms::Round => paint.set_join(StrokeJoin::Round),
        LuaAtoms::Bevel => paint.set_join(StrokeJoin::Bevel),
        _ => state.error::<()>(format!(
            "'{}' is not a valid StrokeJoin",
            name.unwrap_or_default()
        )),
    }
}

fn read_cap(state: &mut LuaState, paint: &mut ScriptedPaint, index: i32) {
    let (name, atom) = state.to_string_atom(index);
    if name.is_none() {
        state.type_error::<()>(index, state.type_name(LuaType::String));
    }
    match atom {
        LuaAtoms::Butt => paint.set_cap(StrokeCap::Butt),
        LuaAtoms::Round => paint.set_cap(StrokeCap::Round),
        LuaAtoms::Square => paint.set_cap(StrokeCap::Square),
        _ => state.error::<()>(format!(
            "'{}' is not a valid StrokeCap",
            name.unwrap_or_default()
        )),
    }
}

pub fn lua_to_blend_mode(state: &mut LuaState, index: i32) -> BlendMode {
    let (name, atom) = state.to_string_atom(index);
    if name.is_none() {
        state.type_error::<()>(index, state.type_name(LuaType::String));
    }
    match atom {
        LuaAtoms::SrcOver => BlendMode::SrcOver,
        LuaAtoms::Screen => BlendMode::Screen,
        LuaAtoms::Overlay => BlendMode::Overlay,
        LuaAtoms::Darken => BlendMode::Darken,
        LuaAtoms::Lighten => BlendMode::Lighten,
        LuaAtoms::ColorDodge => BlendMode::ColorDodge,
        LuaAtoms::ColorBurn => BlendMode::ColorBurn,
        LuaAtoms::HardLight => BlendMode::HardLight,
        LuaAtoms::SoftLight => BlendMode::SoftLight,
        LuaAtoms::Difference => BlendMode::Difference,
        LuaAtoms::Exclusion => BlendMode::Exclusion,
        LuaAtoms::Multiply => BlendMode::Multiply,
        LuaAtoms::Hue => BlendMode::Hue,
        LuaAtoms::Saturation => BlendMode::Saturation,
        LuaAtoms::Color => BlendMode::Color,
        LuaAtoms::Luminosity => BlendMode::Luminosity,
        _ => state.error(format!(
            "'{}' is not a valid BlendMode",
            name.unwrap_or_default()
        )),
    }
}

impl ScriptedPaintData {
    pub fn from_shape_paint(shape_paint: &ShapePaint) -> Self {
        let mut result = Self::new();
        if shape_paint.is_fill() {
            result.set_style(RenderPaintStyle::Fill);
        } else if let Some(stroke) = shape_paint.as_stroke() {
            result.set_style(RenderPaintStyle::Stroke);
            result.set_thickness(stroke.thickness());
            result.set_cap(stroke.cap());
            result.set_join(stroke.join());
        }
        for child in shape_paint.children() {
            if let Some(solid_color) = child.as_solid_color() {
                result.set_color(solid_color.color_value());
                break;
            }
        }
        if let Some(feather) = shape_paint.feather() {
            result.set_feather(feather.strength());
        }
        result.set_blend_mode(shape_paint.blend_mode_value());
        result
    }

    fn push_style(&self, state: &mut LuaState) {
        state.push_string(match self.style() {
            RenderPaintStyle::Fill => "fill",
            RenderPaintStyle::Stroke => "stroke",
        });
    }

    fn push_join(&self, state: &mut LuaState) {
        state.push_string(match self.join() {
            StrokeJoin::Miter => "miter",
            StrokeJoin::Bevel => "bevel",
            StrokeJoin::Round => "round",
        });
    }

    fn push_cap(&self, state: &mut LuaState) {
        state.push_string(match self.cap() {
            StrokeCap::Butt => "butt",
            StrokeCap::Square => "square",
            StrokeCap::Round => "round",
        });
    }

    fn push_blend_mode(&self, state: &mut LuaState) {
        state.push_string(match self.blend_mode() {
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
        });
    }

    fn push_gradient(&self, state: &mut LuaState) {
        if let Some(shader) = self.gradient() {
            state.new_rive(ScriptedGradient {
                shader: Some(shader.clone()),
            });
        } else {
            state.push_nil();
        }
    }
}

impl ScriptedPaint {
    pub fn new(factory: &mut dyn Factory) -> Self {
        Self::with_render_paint(factory.make_render_paint())
    }

    pub fn copy(factory: &mut dyn Factory, source: &Self) -> Self {
        let mut result = Self::new(factory);
        result.set_style(source.style());
        result.set_color(source.color());
        result.set_thickness(source.thickness());
        result.set_join(source.join());
        result.set_cap(source.cap());
        result.set_feather(source.feather());
        result.set_blend_mode(source.blend_mode());
        result.set_gradient(source.gradient().cloned());
        result
    }
}

fn paint_set_value(
    state: &mut LuaState,
    paint: &mut ScriptedPaint,
    atom: LuaAtoms,
    value_index: i32,
) -> bool {
    match atom {
        LuaAtoms::Style => read_style(state, paint, value_index),
        LuaAtoms::Join => read_join(state, paint, value_index),
        LuaAtoms::Cap => read_cap(state, paint, value_index),
        LuaAtoms::Thickness => paint.set_thickness(state.check_number(value_index) as f32),
        LuaAtoms::BlendMode => {
            let mode = lua_to_blend_mode(state, value_index);
            paint.set_blend_mode(mode);
        }
        LuaAtoms::Feather => paint.set_feather(state.check_number(value_index) as f32),
        LuaAtoms::Gradient => {
            let gradient = state.to_rive_optional::<ScriptedGradient>(value_index);
            paint.set_gradient(gradient.and_then(|value| value.shader.clone()));
        }
        LuaAtoms::Color => paint.set_color(state.check_unsigned(value_index)),
        _ => return false,
    }
    true
}

fn set_properties_from_definition_table(
    state: &mut LuaState,
    paint: &mut ScriptedPaint,
    table_index: i32,
) {
    state.check_type(table_index, LuaType::Table);
    state.push_value(table_index);
    state.push_nil();
    while state.next(-2) {
        let (key, atom) = state.to_string_atom(-2);
        if key.is_some() {
            paint_set_value(state, paint, atom, -1);
        }
        state.pop(1);
    }
    state.pop(1);
}

fn paint_new(state: &mut LuaState) -> i32 {
    let factory = state.thread_data::<dyn ScriptingContext>().factory();
    state.new_rive(ScriptedPaint::new(factory));
    1
}

fn paint_with(state: &mut LuaState) -> i32 {
    let factory = state.thread_data::<dyn ScriptingContext>().factory();
    let paint = state.new_rive(ScriptedPaint::new(factory));
    set_properties_from_definition_table(state, paint, 1);
    1
}

fn paint_newindex(state: &mut LuaState) -> i32 {
    let (key, atom) = state.to_string_atom(2);
    if key.is_none() {
        return state.type_error(2, state.type_name(LuaType::String));
    }
    let paint = state.to_rive_mut::<ScriptedPaint>(1);
    paint_set_value(state, paint, atom, 3);
    0
}

fn paint_index(state: &mut LuaState) -> i32 {
    let (key, atom) = state.to_string_atom(2);
    if key.is_none() {
        return state.type_error(2, state.type_name(LuaType::String));
    }
    let paint = unsafe { &*state.to_userdata::<ScriptedPaintData>(1) };
    match atom {
        LuaAtoms::Style => paint.push_style(state),
        LuaAtoms::Join => paint.push_join(state),
        LuaAtoms::Cap => paint.push_cap(state),
        LuaAtoms::Thickness => state.push_number(paint.thickness() as f64),
        LuaAtoms::BlendMode => paint.push_blend_mode(state),
        LuaAtoms::Feather => state.push_number(paint.feather() as f64),
        LuaAtoms::Gradient => paint.push_gradient(state),
        LuaAtoms::Color => state.push_unsigned(paint.color()),
        _ => return 0,
    }
    1
}

fn paint_copy(state: &mut LuaState) -> i32 {
    let argument_count = state.top();
    let source = state.to_rive::<ScriptedPaint>(1);
    let factory = state.thread_data::<dyn ScriptingContext>().factory();
    let copy = state.new_rive(ScriptedPaint::copy(factory, source));
    if argument_count == 2 {
        set_properties_from_definition_table(state, copy, 2);
    }
    1
}

fn paint_namecall(state: &mut LuaState) -> i32 {
    let (name, atom) = state.namecall_atom();
    if atom == LuaAtoms::Copy {
        return paint_copy(state);
    }
    state.error(format!(
        "{} is not a valid method of {}",
        name.unwrap_or_default(),
        ScriptedMat2D::LUA_NAME
    ))
}

const PAINT_STATIC_METHODS: &[LuaReg] = &[
    LuaReg::new("new", paint_new),
    LuaReg::new("with", paint_with),
    LuaReg::END,
];

fn register_direct_fields<T: LuaRive>(state: &mut LuaState) {
    state.register_userdata_direct_number_field::<T>("thickness", |paint| paint.thickness());
    state.register_userdata_direct_number_field::<T>("feather", |paint| paint.feather());
    state.register_userdata_direct_number_field::<T>("color", |paint| paint.color() as f64);
}

pub fn luaopen_rive_paint(state: &mut LuaState) -> i32 {
    state.register_rive::<ScriptedPaintData>();
    state.push_function(paint_index);
    state.set_field(-2, "__index");
    state.set_readonly(-1, true);
    state.pop(1);
    register_direct_fields::<ScriptedPaintData>(state);

    state.register(ScriptedPaint::LUA_NAME, PAINT_STATIC_METHODS);
    state.register_rive::<ScriptedPaint>();
    for (name, function) in [
        ("__index", paint_index as LuaFunction),
        ("__newindex", paint_newindex),
        ("__namecall", paint_namecall),
    ] {
        state.push_function(function);
        state.set_field(-2, name);
    }
    state.set_readonly(-1, true);
    state.pop(1);
    register_direct_fields::<ScriptedPaint>(state);
    1
}
