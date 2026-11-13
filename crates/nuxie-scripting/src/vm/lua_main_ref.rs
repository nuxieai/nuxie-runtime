//! Main-thread-owned registry references that push onto the calling coroutine.
//!
//! On the safe VM (the default, seam rows R6 to R9) a reference is a VM
//! handle: handles are VM-scoped (valid on every thread) and dropping one
//! does not touch the VM, so neither the main thread pointer nor `lua_ref`
//! is needed.
use crate::luau::{Lua, Result};
use std::rc::Rc;

struct MainLua(crate::luau::WeakLua);

pub(super) fn main_lua(lua: &Lua) -> Option<Lua> {
    lua.app_data_ref::<MainLua>()?.0.try_upgrade()
}

pub(super) fn install_main_lua(lua: &Lua) {
    if lua.app_data_ref::<MainLua>().is_none() {
        lua.set_app_data(MainLua(lua.weak()));
    }
}

/// A rooted VM handle and a weak owner: no strong reference to the VM, so
/// a userdata holding one does not keep its VM alive.
struct MainReference {
    lua: crate::luau::WeakLua,
    value: crate::luau::vm::Value,
}

#[derive(Clone)]
pub(super) struct MainThreadRef(Rc<MainReference>);

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
