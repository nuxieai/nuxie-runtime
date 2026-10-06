//! Scroll input binding from `src/lua/math/lua_input.cpp` and `rive_lua_libs.hpp`.
use super::listener_invocation::{ScriptedPointerHitResult, ScriptedPointerHitResultHandle};
use luaur_rt::{
    AnyUserData, Error, Lua, MultiValue, Result, Table, UserData, UserDataMethods, Value, Vector,
};
use nuxie_runtime::source::{
    math::vec2d::Vec2D,
    scroll_event::{ScrollEvent, ScrollPhase},
};
use std::{cell::Cell, rc::Rc};

pub(super) struct ScriptedScrollEvent {
    pub id: i32,
    pub position: Vec2D,
    pub event: ScrollEvent,
    pub timestamp: f32,
    hit_result: ScriptedPointerHitResultHandle,
}

pub(super) fn scroll_event_argument(
    lua: &Lua,
    id: i32,
    x: f32,
    y: f32,
    event: ScrollEvent,
    timestamp: f32,
) -> Result<(AnyUserData, ScriptedPointerHitResultHandle)> {
    let hit_result = Rc::new(Cell::new(ScriptedPointerHitResult::None));
    let userdata = lua.create_userdata(ScriptedScrollEvent {
        id,
        position: Vec2D::new(x, y),
        event,
        timestamp,
        hit_result: hit_result.clone(),
    })?;
    prepare_metatable(lua, userdata.clone())?;
    Ok((userdata, hit_result))
}

fn phase_name(phase: ScrollPhase) -> &'static str {
    match phase {
        ScrollPhase::Begin => "begin",
        ScrollPhase::Update => "update",
        ScrollPhase::End => "end",
        ScrollPhase::Momentum => "momentum",
        ScrollPhase::InertiaCancel => "inertiaCancel",
    }
}

fn prepare_metatable(lua: &Lua, userdata: AnyUserData) -> Result<()> {
    // SAFETY: exec_raw anchors the userdata at slot 1 and owns the stack frame;
    // lua_getmetatable pushes the registered metatable returned as a Table.
    let metatable: Table = unsafe {
        lua.exec_raw(userdata, |state| {
            luaur_vm::functions::lua_getmetatable::lua_getmetatable(state, 1)?;
            Ok(())
        })?
    };
    if !metatable.is_readonly() {
        metatable.set(
            "__index",
            lua.create_function(|lua, (userdata, key): (AnyUserData, Value)| {
                let key: luaur_rt::LuaString = lua.unpack(key)?;
                let event = userdata.borrow::<ScriptedScrollEvent>()?;
                match key.to_string_lossy().as_str() {
                    "id" => lua.pack(event.id),
                    "position" => lua.pack(Vector::new(event.position.x, event.position.y, 0.0)),
                    "delta" => lua.pack(Vector::new(event.event.delta.x, event.event.delta.y, 0.0)),
                    "phase" => lua.pack(phase_name(event.event.phase)),
                    "precise" => lua.pack(event.event.precise),
                    "timeStamp" => lua.pack(event.timestamp),
                    _ => Err(Error::runtime(format!(
                        "{} is not a valid field of ScrollEvent",
                        key.to_string_lossy()
                    ))),
                }
            })?,
        )?;
        metatable.set_readonly(true);
    }
    Ok(())
}

impl UserData for ScriptedScrollEvent {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_meta_method("__namecall", |lua, this, _: MultiValue| {
            // SAFETY: the active namecall atom belongs to this live Lua thread
            // and its C string is copied before returning to the VM.
            let name = unsafe {
                let mut atom = 0;
                let name = luaur_vm::functions::lua_namecallatom::lua_namecallatom(
                    lua.current_thread().state(),
                    &mut atom,
                );
                if name.is_null() {
                    String::new()
                } else {
                    std::ffi::CStr::from_ptr(name)
                        .to_string_lossy()
                        .into_owned()
                }
            };
            if name == "hit" {
                this.hit_result.set(ScriptedPointerHitResult::HitOpaque);
                Ok(())
            } else {
                Err(Error::runtime(format!(
                    "{name} is not a valid method of ScrollEvent"
                )))
            }
        });
    }
}

pub(super) fn install(lua: &Lua) -> Result<()> {
    let table = lua.create_table();
    table.set(
        "new",
        lua.create_function(
            |lua,
             (id, position, delta, phase, precise, timestamp): (
                i32,
                Vector,
                Vector,
                Option<luaur_rt::LuaString>,
                Value,
                Option<f32>,
            )| {
                // luaL_checkoption compares C strings, including their NUL boundary.
                let phase_bytes = phase.as_ref().map(|phase| phase.as_bytes());
                let phase_name = phase_bytes.as_deref().unwrap_or(b"update");
                let phase_name = phase_name
                    .split(|byte| *byte == 0)
                    .next()
                    .unwrap_or_default();
                let phase = match phase_name {
                    b"begin" => ScrollPhase::Begin,
                    b"update" => ScrollPhase::Update,
                    b"end" => ScrollPhase::End,
                    b"momentum" => ScrollPhase::Momentum,
                    b"inertiaCancel" => ScrollPhase::InertiaCancel,
                    other => {
                        return Err(Error::runtime(format!(
                            "invalid option '{}'",
                            String::from_utf8_lossy(other)
                        )));
                    }
                };
                let precise = match precise {
                    Value::Nil => false,
                    Value::Boolean(value) => value,
                    _ => return Err(Error::runtime("boolean expected")),
                };
                scroll_event_argument(
                    lua,
                    id,
                    position.x(),
                    position.y(),
                    ScrollEvent {
                        delta: Vec2D::new(delta.x(), delta.y()),
                        phase,
                        precise,
                    },
                    timestamp.unwrap_or(0.0),
                )
                .map(|value| value.0)
            },
        )?,
    )?;
    lua.globals().set("ScrollEvent", table)?;
    // Register and freeze the metatable at module initialization, as upstream does.
    let _ = scroll_event_argument(lua, 0, 0.0, 0.0, ScrollEvent::default(), 0.0)?;
    Ok(())
}
