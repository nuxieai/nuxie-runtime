#![cfg(feature = "rive_scripting")]
use crate::mechanical_port::source::{
    assets::blob_asset::BlobAsset,
    lua::rive_lua_libs::{LuaAtoms, LuaState, LuaType, ScriptedBlob},
};
fn direct_size(data: &ScriptedBlob, result: &mut DirectFieldResult) {
    if let Some(asset) = data.asset.as_ref().and_then(|a| a.as_blob_asset()) {
        result.set_number(asset.bytes().len() as f64);
    } else {
        result.set_nil();
    }
}
fn index(state: &mut LuaState) -> i32 {
    let blob = state.to_rive::<ScriptedBlob>(1);
    let (name, atom) = state.to_string_atom(2);
    let Some(name) = name else {
        return state.type_error(2, state.type_name(LuaType::String));
    };
    let Some(asset) = blob.asset.as_ref().and_then(|a| a.as_blob_asset()) else {
        state.push_nil();
        return 1;
    };
    match atom {
        LuaAtoms::Size => state.push_number(asset.bytes().len() as f64),
        LuaAtoms::Name => state.push_string(asset.name()),
        LuaAtoms::Data => {
            if asset.bytes().is_empty() {
                state.push_nil()
            } else {
                state.new_buffer(asset.bytes())
            }
        }
        _ => {
            return state.error(format!(
                "'{name}' is not a valid index of {}",
                ScriptedBlob::LUA_NAME
            ));
        }
    }
    1
}
#[cfg(feature = "rive_tools")]
pub fn push_blob(state: &mut LuaState, name: Option<&str>, data: &[u8]) -> i32 {
    let mut blob = ScriptedBlob::default();
    if !data.is_empty() {
        let mut asset = BlobAsset::default();
        if let Some(name) = name {
            asset.set_name(name);
        }
        asset.decode(data, None);
        blob.asset = Some(asset.into_file_asset());
    }
    state.new_rive(blob);
    1
}
pub fn luaopen_rive_blob(state: &mut LuaState) -> i32 {
    state.register_rive::<ScriptedBlob>();
    state.push_function(index);
    state.set_field(-2, "__index");
    state.set_readonly(-1, true);
    state.pop(1);
    state.register_userdata_direct_field_get::<ScriptedBlob>("size", direct_size);
    0
}
