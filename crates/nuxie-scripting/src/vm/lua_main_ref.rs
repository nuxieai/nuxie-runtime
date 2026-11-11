//! Main-thread-owned registry references that push onto the calling coroutine.
//!
//! On the safe VM (`nuxie-vm`, seam rows R6 to R9) a reference is a VM
//! handle: handles are VM-scoped (valid on every thread) and dropping one
//! does not touch the VM, so neither the main thread pointer nor `lua_ref`
//! is needed.
use crate::luau::{Lua, Result};
use std::rc::Rc;

#[cfg(not(feature = "nuxie-vm"))]
struct MainLua(crate::luau::WeakLua, *mut luaur_rt::lua_State);
#[cfg(not(feature = "nuxie-vm"))]
pub(super) fn main_lua(lua: &Lua) -> Option<Lua> {
    lua.app_data_ref::<MainLua>()?.0.try_upgrade()
}
#[cfg(not(feature = "nuxie-vm"))]
pub(super) fn install_main_lua(lua: &Lua) {
    if lua.app_data_ref::<MainLua>().is_none() {
        let main = unsafe {
            luaur_vm::functions::lua_mainthread::lua_mainthread(lua.current_thread().state())
        };
        lua.set_app_data(MainLua(lua.weak(), main));
    }
}
#[cfg(not(feature = "nuxie-vm"))]
struct MainReference {
    lua: crate::luau::WeakLua,
    main: *mut luaur_rt::lua_State,
    id: i32,
}
#[cfg(not(feature = "nuxie-vm"))]
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
#[cfg(not(feature = "nuxie-vm"))]
#[derive(Clone)]
pub(super) struct MainThreadRef(Rc<MainReference>);
#[cfg(not(feature = "nuxie-vm"))]
impl MainThreadRef {
    pub(super) fn new(lua: &Lua, value: impl crate::luau::IntoLuaMulti) -> Result<Self> {
        let (owner, main) = {
            let main = lua
                .app_data_ref::<MainLua>()
                .ok_or_else(|| crate::luau::Error::runtime("missing main scripting state"))?;
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
    pub(super) fn get<T: crate::luau::FromLuaMulti>(&self, lua: &Lua) -> Result<T> {
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

#[cfg(feature = "nuxie-vm")]
struct MainLua(crate::luau::WeakLua);

#[cfg(feature = "nuxie-vm")]
pub(super) fn main_lua(lua: &Lua) -> Option<Lua> {
    lua.app_data_ref::<MainLua>()?.0.try_upgrade()
}

#[cfg(feature = "nuxie-vm")]
pub(super) fn install_main_lua(lua: &Lua) {
    if lua.app_data_ref::<MainLua>().is_none() {
        lua.set_app_data(MainLua(lua.weak()));
    }
}

/// A rooted VM handle and a weak owner: no strong reference to the VM, so
/// a userdata holding one does not keep its VM alive.
#[cfg(feature = "nuxie-vm")]
struct MainReference {
    lua: crate::luau::WeakLua,
    value: crate::luau::vm::Value,
}

#[cfg(feature = "nuxie-vm")]
#[derive(Clone)]
pub(super) struct MainThreadRef(Rc<MainReference>);

#[cfg(feature = "nuxie-vm")]
impl MainThreadRef {
    /// References the first of `value`'s values, as `lua_ref(state, 1)`.
    pub(super) fn new(lua: &Lua, value: impl crate::luau::IntoLuaMulti) -> Result<Self> {
        let owner = lua
            .app_data_ref::<MainLua>()
            .ok_or_else(|| crate::luau::Error::runtime("missing main scripting state"))?
            .0
            .clone();
        let values = value.into_lua_multi(lua)?;
        let first = values.into_iter().next().unwrap_or(crate::luau::Value::Nil);
        let value = lua.to_vm_value(&first)?;
        Ok(Self(Rc::new(MainReference { lua: owner, value })))
    }

    pub(super) fn main_lua(&self) -> Option<Lua> {
        self.0.lua.try_upgrade()
    }

    pub(super) fn get<T: crate::luau::FromLuaMulti>(&self, lua: &Lua) -> Result<T> {
        let value = lua.from_vm_value(self.0.value.clone())?;
        lua.unpack_multi(crate::luau::MultiValue::from_vec(vec![value]))
    }
}
