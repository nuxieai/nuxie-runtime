use crate::mechanical_port::source::{
    lua::rive_lua_libs::{LuaState, ScriptedViewModel},
    viewmodel::viewmodel::ViewModel,
};

fn viewmodel_new(state: &mut LuaState) -> i32 {
    let Some(viewmodel) = state.upvalue_light_userdata::<ViewModel>(1) else {
        state.push_nil();
        return 1;
    };
    if state.top() == 1 {
        if state.is_nil(-1) {
            let instance = viewmodel.create_instance();
            state.new_rive(ScriptedViewModel::new(viewmodel, instance));
        } else if state.is_string(-1) {
            let name = state.to_string(-1).unwrap();
            let instance = viewmodel
                .create_from_instance(&name)
                .unwrap_or_else(|| viewmodel.create_instance());
            state.new_rive(ScriptedViewModel::new(viewmodel, instance));
        } else {
            state.push_nil();
        }
    } else {
        let instance = viewmodel.create_instance();
        state.new_rive(ScriptedViewModel::new(viewmodel, instance));
    }
    1
}

pub fn initialize_lua_data(state: Option<&mut LuaState>, viewmodels: &mut [&mut ViewModel]) {
    let Some(state) = state else { return };
    state.new_metatable("Data");
    state.pop(1);
    state.get_registry_field("Data");
    state.set_global("Data");
    state.get_registry_field("Data");
    for viewmodel in viewmodels {
        state.create_table(0, 1);
        state.push_light_userdata(*viewmodel);
        state.push_closure(viewmodel_new, "new", 1);
        state.set_field(-2, "new");
        state.set_field(-2, viewmodel.name());
    }
    state.pop(1);
}
