#![cfg(feature = "rive_scripting")]
use crate::mechanical_port::source::{
    lua::rive_lua_libs::{LuaAtoms, LuaState, LuaType, ScriptedImage, ScriptedImageSampler},
    renderer::{ImageFilter, ImageSampler, ImageWrap},
};
fn direct_width(image: &ScriptedImage, result: &mut DirectFieldResult) {
    result.set_number(
        image
            .image
            .as_ref()
            .map(|v| v.width() as f64)
            .unwrap_or(0.0),
    );
}
fn direct_height(image: &ScriptedImage, result: &mut DirectFieldResult) {
    result.set_number(
        image
            .image
            .as_ref()
            .map(|v| v.height() as f64)
            .unwrap_or(0.0),
    );
}
fn image_index(state: &mut LuaState) -> i32 {
    let image = state.to_rive::<ScriptedImage>(1);
    let (name, atom) = state.to_string_atom(2);
    let Some(name) = name else {
        return state.type_error(2, state.type_name(LuaType::String));
    };
    match atom {
        LuaAtoms::Width => state.push_number(
            image
                .image
                .as_ref()
                .map(|v| v.width() as f64)
                .unwrap_or(0.0),
        ),
        LuaAtoms::Height => state.push_number(
            image
                .image
                .as_ref()
                .map(|v| v.height() as f64)
                .unwrap_or(0.0),
        ),
        #[cfg(feature = "rive_ore")]
        LuaAtoms::View => state.push_function_named(rive_image_view_impl, "Image.view"),
        _ => {
            return state.error(format!(
                "'{name}' is not a valid index of {}",
                ScriptedImage::LUA_NAME
            ));
        }
    }
    1
}
fn sampler_construct(state: &mut LuaState) -> i32 {
    let (wx_name, wx) = state.to_string_atom(1);
    let (wy_name, wy) = state.to_string_atom(2);
    let (filter_name, filter) = state.to_string_atom(3);
    let Some(wx_name) = wx_name else {
        return state.type_error(1, state.type_name(LuaType::String));
    };
    let Some(wy_name) = wy_name else {
        return state.type_error(2, state.type_name(LuaType::String));
    };
    let Some(filter_name) = filter_name else {
        return state.type_error(3, state.type_name(LuaType::String));
    };
    let wx = match wx {
        LuaAtoms::Clamp => ImageWrap::Clamp,
        LuaAtoms::Repeat => ImageWrap::Repeat,
        LuaAtoms::Mirror => ImageWrap::Mirror,
        _ => return state.error(format!("'{wx_name}' is not a valid ImageWrap")),
    };
    let wy = match wy {
        LuaAtoms::Clamp => ImageWrap::Clamp,
        LuaAtoms::Repeat => ImageWrap::Repeat,
        LuaAtoms::Mirror => ImageWrap::Mirror,
        _ => return state.error(format!("'{wy_name}' is not a valid ImageWrap")),
    };
    let filter = match filter {
        LuaAtoms::Bilinear => ImageFilter::Bilinear,
        LuaAtoms::Nearest => ImageFilter::Nearest,
        _ => return state.error(format!("'{filter_name}' is not a valid ImageFilter")),
    };
    state.new_rive(ScriptedImageSampler {
        sampler: ImageSampler::new(wx, wy, filter),
    });
    1
}
pub fn luaopen_rive_image(state: &mut LuaState) -> i32 {
    state.register_rive::<ScriptedImage>();
    state.push_function(image_index);
    state.set_field(-2, "__index");
    state.set_readonly(-1, true);
    state.pop(1);
    state.register_userdata_direct_field_get::<ScriptedImage>("width", direct_width);
    state.register_userdata_direct_field_get::<ScriptedImage>("height", direct_height);
    state.push_function_named(sampler_construct, ScriptedImageSampler::LUA_NAME);
    state.set_global_field(ScriptedImageSampler::LUA_NAME);
    state.register_rive::<ScriptedImageSampler>();
    0
}
