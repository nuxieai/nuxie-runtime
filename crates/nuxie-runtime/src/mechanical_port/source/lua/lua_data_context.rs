use crate::mechanical_port::source::{
    data_bind::data_context::DataContext,
    lua::rive_lua_libs::{
        LuaAtoms, LuaState, ScriptedDataContext, ScriptedDataContextHandle, ScriptedViewModel,
    },
};
impl ScriptedDataContext {
    pub fn new(data_context: ScriptedDataContextHandle) -> Self {
        Self { data_context }
    }
    pub fn push_parent(&self, state: &mut LuaState) -> i32 {
        if let Some(parent) = self.data_context.parent() {
            state.new_rive(ScriptedDataContext::new(ScriptedDataContextHandle::Shared(
                parent,
            )));
        } else {
            state.push_nil();
        }
        1
    }
    pub fn push_viewmodel(&self, state: &mut LuaState) -> i32 {
        if let Some(instance) = self.data_context.main_view_model_instance() {
            let model = instance
                .with(|instance| {
                    instance
                        .as_view_model_instance()
                        .and_then(|instance| instance.get_view_model())
                })
                .flatten();
            state.new_rive(ScriptedViewModel::new(state, model, Some(instance)));
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
                return context.push_parent(state);
            }
            LuaAtoms::ViewModel => {
                return context.push_viewmodel(state);
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
