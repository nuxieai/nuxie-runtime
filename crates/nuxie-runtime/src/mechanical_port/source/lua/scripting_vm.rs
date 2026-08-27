#![cfg(feature = "rive_scripting")]

use std::collections::HashSet;

use super::rive_lua_libs::{
    LuaState, LuaType, ScriptingContext, check_registered_modules, dump_stack, lua_late,
    lua_require, lua_runtime_error, luaopen_rive,
};
use crate::mechanical_port::source::{
    assets::script_asset::ModuleDetails, scripted::scripted_object::ScriptedObject,
};

/// Owns the main Luau state and its scripting context.
pub struct ScriptingVM {
    state: Option<Box<LuaState>>,
    owned_context: Box<dyn ScriptingContext>,
    scripted_objects: HashSet<*mut ScriptedObject>,
}

impl ScriptingVM {
    pub fn new(context: Box<dyn ScriptingContext>) -> Self {
        let mut state = LuaState::new();
        Self::init(&mut state, context.as_ref());
        Self {
            state: Some(state),
            owned_context: context,
            scripted_objects: HashSet::new(),
        }
    }

    pub fn init(state: &mut LuaState, context: &dyn ScriptingContext) {
        luaopen_rive(state);
        state.set_thread_data(context);
        state.push_closure(lua_require, "require", 0);
        state.set_global("require");
        state.push_closure(lua_runtime_error, "error", 0);
        state.set_global("error");
        state.push_closure(lua_late, "late", 0);
        state.set_global("late");
        state.sandbox();
        state.sandbox_thread();
    }

    pub fn close_lua_state(&mut self) {
        let Some(mut state) = self.state.take() else {
            return;
        };
        self.owned_context.shutdown_async_for_state(&mut state);
        for object in self.scripted_objects.drain() {
            unsafe { &mut *object }.clear_scripting_vm();
        }
        state.close();
    }

    pub fn register_scripted_object(&mut self, object: Option<&mut ScriptedObject>) {
        if let Some(object) = object {
            self.scripted_objects.insert(object);
        }
    }

    pub fn unregister_scripted_object(&mut self, object: Option<&mut ScriptedObject>) {
        if let Some(object) = object {
            self.scripted_objects
                .remove(&(object as *mut ScriptedObject));
        }
    }

    pub fn context(&mut self) -> &mut dyn ScriptingContext {
        self.owned_context.as_mut()
    }

    pub fn state(&mut self) -> Option<&mut LuaState> {
        self.state.as_deref_mut()
    }

    pub fn replace_context(&mut self, mut context: Box<dyn ScriptingContext>) {
        #[cfg(feature = "rive_tools")]
        self.owned_context.dispose_orphan_scripted_properties(None);
        if let Some(state) = self.state.as_deref_mut() {
            state.set_thread_data(context.as_mut());
        }
        self.owned_context = context;
    }

    pub fn add_module(&mut self, module: &mut ModuleDetails) {
        self.owned_context.add_module(module);
    }

    pub fn perform_registration(&mut self) {
        if let Some(state) = self.state.as_deref_mut() {
            self.owned_context.perform_registration(state);
        }
    }

    pub fn load_module(state: &mut LuaState, name: &str, bytecode: &[u8]) -> bool {
        if bytecode.is_empty() {
            return false;
        }
        let main_thread = state.main_thread();
        let module_thread = main_thread.new_thread();
        main_thread.move_values_to(state, 1);
        module_thread.sandbox_thread();
        module_thread.set_thread_data_from(state);
        let status = module_thread.load_bytecode(name, bytecode);
        if status != 0 {
            module_thread.move_values_to(state, 1);
            state
                .thread_data_mut::<dyn ScriptingContext>()
                .print_error(state);
            state.pop(2);
            return false;
        }
        true
    }

    pub fn execute_module(
        state: &mut LuaState,
        name: &str,
        is_utility: bool,
        chunk_name: Option<&str>,
    ) -> bool {
        let display = chunk_name.unwrap_or(name);
        let Some(module_thread) = state.to_thread(-1) else {
            return false;
        };
        let status = module_thread.resume(state, 0);
        if status == 0 {
            if module_thread.top() == 0 {
                module_thread.push_string(format!("{display}:1: module must return a value"));
            } else if !matches!(
                module_thread.value_type(-1),
                LuaType::Table | LuaType::Function
            ) {
                module_thread.push_string(format!(
                    "{display}:1: module must return a table or function"
                ));
            }
        } else if status == super::rive_lua_libs::LUA_YIELD {
            module_thread.push_string(format!("{display}:1: module can not yield"));
        } else if !module_thread.is_string(-1) {
            module_thread.push_string(format!("{display}:1: unknown error while running module"));
        }

        module_thread.move_values_to(state, 1);
        if state.is_string(-1) {
            state
                .thread_data_mut::<dyn ScriptingContext>()
                .print_error(state);
            state.pop(2);
            return false;
        }
        state.remove(-2);
        if is_utility {
            state.find_table(super::rive_lua_libs::LUA_REGISTRY_INDEX, "_MODULES", 1);
            state.push_string(name);
            state.push_value(-3);
            state.set_table(-3);
            state.pop(1);
        }
        true
    }

    pub fn register_script_on(
        state: &mut LuaState,
        name: &str,
        bytecode: &[u8],
        chunk_name: Option<&str>,
    ) -> bool {
        if check_registered_modules(state, name) {
            return true;
        }
        let load_name = chunk_name.unwrap_or(name);
        Self::load_module(state, load_name, bytecode)
            && Self::execute_module(state, name, false, chunk_name)
    }

    pub fn register_module_on(
        state: &mut LuaState,
        name: &str,
        bytecode: &[u8],
        chunk_name: Option<&str>,
    ) -> bool {
        if check_registered_modules(state, name) {
            state.pop(1);
            return true;
        }
        let load_name = chunk_name.unwrap_or(name);
        if !Self::load_module(state, load_name, bytecode)
            || !Self::execute_module(state, name, true, chunk_name)
        {
            return false;
        }
        state.pop(1);
        true
    }

    pub fn unregister_module_on(state: &mut LuaState, name: &str) {
        state.find_table(super::rive_lua_libs::LUA_REGISTRY_INDEX, "_MODULES", 1);
        state.push_string(name);
        state.push_nil();
        state.set_table(-3);
        state.pop(1);
    }

    pub fn register_module(&mut self, name: &str, bytecode: &[u8]) -> bool {
        self.state
            .as_deref_mut()
            .is_some_and(|state| Self::register_module_on(state, name, bytecode, None))
    }

    pub fn unregister_module(&mut self, name: &str) {
        if let Some(state) = self.state.as_deref_mut() {
            Self::unregister_module_on(state, name);
        }
    }

    pub fn register_script(&mut self, name: &str, bytecode: &[u8]) -> bool {
        self.state
            .as_deref_mut()
            .is_some_and(|state| Self::register_script_on(state, name, bytecode, None))
    }

    pub fn dump_stack(state: &mut LuaState) {
        dump_stack(state);
    }
}

impl Drop for ScriptingVM {
    fn drop(&mut self) {
        self.close_lua_state();
    }
}
