//! Mechanical API owner for `src/lua/lua_transition.cpp`.
//! The compiled native Luau registration and scoped invalidation live in
//! `nuxie-scripting/src/vm/lua_transition.rs`, through the existing backend seam.
use crate::mechanical_port::source::{
    artboard::Artboard,
    lua::rive_lua_libs::{LuaAtoms, LuaState, LuaType, ScriptedRenderer, TransitionChild},
    renderer::Renderer,
};

impl TransitionChild {
    pub fn draw(&self, renderer: &mut Renderer) {
        let Some(artboard) = &self.artboard else {
            return;
        };
        renderer.save();
        renderer.transform(nuxie_render_api::Mat2D(*self.world_transform.values()));
        Artboard::draw_internal_handle(artboard, renderer);
        renderer.restore();
    }
    pub fn width(&self) -> f32 {
        self.artboard
            .as_ref()
            .and_then(|artboard| artboard.with_downcast::<Artboard, _>(Artboard::width))
            .unwrap_or(0.0)
    }
    pub fn height(&self) -> f32 {
        self.artboard
            .as_ref()
            .and_then(|artboard| artboard.with_downcast::<Artboard, _>(Artboard::height))
            .unwrap_or(0.0)
    }
}

fn namecall(state: &mut LuaState) -> i32 {
    let (name, atom) = state.namecall_atom();
    if atom == LuaAtoms::Draw {
        let (child, renderer) = state.rive2_mut::<TransitionChild, ScriptedRenderer>();
        child.draw(renderer.validate(state));
        return 0;
    }
    state.error(format!(
        "{name} is not a valid method of {}",
        TransitionChild::LUA_NAME
    ))
}

fn index(state: &mut LuaState) -> i32 {
    let (name, atom) = state.to_string_atom(2);
    let Some(name) = name else {
        return state.type_error(2, state.type_name(LuaType::String));
    };
    let child = state.to_rive::<TransitionChild>(1);
    match atom {
        LuaAtoms::Width => state.push_number(child.width() as f64),
        LuaAtoms::Height => state.push_number(child.height() as f64),
        _ => {
            return state.error(format!(
                "'{name}' is not a valid index of {}",
                TransitionChild::LUA_NAME
            ));
        }
    }
    1
}

pub fn luaopen_rive_transition(state: &mut LuaState) -> i32 {
    state.register_rive::<TransitionChild>();
    state.push_function(index);
    state.set_field(-2, "__index");
    state.push_function(namecall);
    state.set_field(-2, "__namecall");
    state.set_readonly(-1, true);
    state.pop(1);
    0
}
