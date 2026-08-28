use crate::mechanical_port::source::lua::rive_lua_libs::{
    LuaFunction, LuaReg, LuaState, ScriptingContext,
};

fn lua_print(state: &mut LuaState) -> i32 {
    let count = state.top();
    if count == 0 {
        return 0;
    }
    let context = state.thread_data_mut::<dyn ScriptingContext>();
    context.print_begin_line(state);
    for index in 1..=count {
        let string = state.to_l_string(index);
        context.print(string.as_bytes());
        state.pop(1);
    }
    context.print_end_line();
    0
}

const BASE_FUNCTIONS: &[LuaReg] = &[LuaReg::new("print", lua_print), LuaReg::END];

pub fn luaopen_rive_base(state: &mut LuaState) -> i32 {
    state.register("_G", BASE_FUNCTIONS);
    1
}
