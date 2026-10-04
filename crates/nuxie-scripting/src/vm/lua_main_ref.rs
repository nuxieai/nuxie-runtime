//! Main-thread-owned registry references that push onto the calling coroutine.
use luaur_rt::{Lua, Result};
use std::rc::Rc;

struct MainLua(luaur_rt::WeakLua, *mut luaur_rt::lua_State);
pub(super) fn main_lua(lua: &Lua) -> Option<Lua> {
    lua.app_data_ref::<MainLua>()?.0.try_upgrade()
}
pub(super) fn install_main_lua(lua: &Lua) {
    if lua.app_data_ref::<MainLua>().is_none() {
        let main = unsafe {
            luaur_vm::functions::lua_mainthread::lua_mainthread(lua.current_thread().state())
        };
        lua.set_app_data(MainLua(lua.weak(), main));
    }
}
struct MainReference {
    lua: luaur_rt::WeakLua,
    main: *mut luaur_rt::lua_State,
    id: i32,
}
impl Drop for MainReference {
    fn drop(&mut self) {
        // The registry is VM-wide. Never retain or dereference the coroutine
        // that happened to mint this value, nor strongly own the VM from it.
        if let Some(_lua) = self.lua.try_upgrade() {
            // Unref does not allocate or enter Lua callbacks, including when
            // this cache is released by a userdata finalizer.
            unsafe {
                luaur_vm::functions::lua_unref::lua_unref(self.main, self.id);
            }
        }
        // During VM shutdown the weak owner is gone; lua_close frees registry.
    }
}
#[derive(Clone)]
pub(super) struct MainThreadRef(Rc<MainReference>);
impl MainThreadRef {
    pub(super) fn new(lua: &Lua, value: impl luaur_rt::IntoLuaMulti) -> Result<Self> {
        let (owner, main) = {
            let main = lua
                .app_data_ref::<MainLua>()
                .ok_or_else(|| luaur_rt::Error::runtime("missing main scripting state"))?;
            (main.0.clone(), main.1)
        };
        let mut id = 0;
        // lua_ref's slot belongs to the global registry even when the value
        // is on a coroutine stack. Cleanup always uses the weak main owner.
        let _: () = unsafe {
            lua.exec_raw(value, |state| {
                id = luaur_vm::functions::lua_ref::lua_ref(state, 1)?;
                Ok(())
            })?
        };
        Ok(Self(Rc::new(MainReference {
            lua: owner,
            main,
            id,
        })))
    }
    pub(super) fn main_lua(&self) -> Option<Lua> {
        self.0.lua.try_upgrade()
    }
    pub(super) fn get<T: luaur_rt::FromLuaMulti>(&self, lua: &Lua) -> Result<T> {
        unsafe {
            lua.exec_raw((), |state| {
                luaur_vm::functions::lua_rawgeti::lua_rawgeti(
                    state,
                    luaur_vm::macros::lua_registryindex::LUA_REGISTRYINDEX,
                    self.0.id,
                )?;
                Ok(())
            })
        }
    }
}
