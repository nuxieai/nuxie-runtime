#![cfg(feature = "rive_scripting")]
use crate::mechanical_port::source::{
    data_bind::data_context::DataContext,
    lua::rive_lua_libs::{LuaAtoms, LuaState, ScriptedDataContext, ScriptedViewModel},
};
impl ScriptedDataContext {
    pub fn new(state: *mut LuaState, data_context: DataContext) -> Self {
        Self {
            state,
            data_context,
        }
    }
    pub fn push_parent(&self) -> i32 {
        let state = unsafe { &mut *self.state };
        if let Some(parent) = self.data_context.parent() {
            state.new_rive(ScriptedDataContext::new(self.state, parent.clone()));
        } else {
            state.push_nil();
        }
        1
    }
    pub fn push_viewmodel(&self) -> i32 {
        let state = unsafe { &mut *self.state };
        if let Some(instance) = self.data_context.main_viewmodel_instance() {
            state.new_rive(ScriptedViewModel::new(
                instance.viewmodel(),
                instance.clone(),
            ));
        } else {
            state.push_nil();
        }
        1
    }
}
fn data_context_namecall(state: &mut LuaState) -> i32 {
    let (name, atom) = state.namecall_atom();
    if name.is_some() {
        let context = state.to_rive::<ScriptedDataContext>(1);
        match atom {
            LuaAtoms::Parent => {
                assert!(std::ptr::eq(context.state(), state));
                return context.push_parent();
            }
            LuaAtoms::ViewModel => {
                assert!(std::ptr::eq(context.state(), state));
                return context.push_viewmodel();
            }
            _ => {}
        }
    }
    state.error(format!(
        "{} is not a valid method of {}",
        name.unwrap_or_default(),
        ScriptedDataContext::LUA_NAME
    ))
}
pub fn luaopen_rive_data_context(state: &mut LuaState) -> i32 {
    state.register_rive::<ScriptedDataContext>();
    state.push_function(data_context_namecall);
    state.set_field(-2, "__namecall");
    state.set_readonly(-1, true);
    state.pop(1);
    1
}
