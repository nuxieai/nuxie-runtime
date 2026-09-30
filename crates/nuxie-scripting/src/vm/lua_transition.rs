//! Native Luau translation of `src/lua/lua_transition.cpp` and the transition
//! calls in `lua_script_backend.cpp`. Retained handles are scoped to one call.
use super::{
    ProtectedScriptCall, lua_renderer::ScriptedRenderer, lua_renderer_library::RendererBindings,
};
use luaur_rt::{
    AnyUserData, Error, Lua, MultiValue, Result, Table, UserData, UserDataFields, UserDataMethods,
    Value,
};
use nuxie_render_api::Renderer;
use nuxie_runtime::{ScriptTransitionChildRef, source::artboard::Artboard};
use std::{cell::RefCell, rc::Rc};

struct TransitionChild(Rc<RefCell<Option<ScriptTransitionChildRef>>>);
struct ChildScope(Rc<RefCell<Option<ScriptTransitionChildRef>>>);
impl Drop for ChildScope {
    fn drop(&mut self) {
        self.0.borrow_mut().take();
    }
}

fn push_child(lua: &Lua, child: &ScriptTransitionChildRef) -> Result<(Value, Option<ChildScope>)> {
    if child.artboard.is_none() {
        return Ok((Value::Nil, None));
    }
    let state = Rc::new(RefCell::new(Some(child.clone())));
    let guard = ChildScope(state.clone());
    let userdata = lua.create_userdata(TransitionChild(state))?;
    prepare_metatable(lua, userdata.clone())?;
    Ok((Value::UserData(userdata), Some(guard)))
}

pub(super) fn install(lua: &Lua) -> Result<()> {
    let userdata = lua.create_userdata(TransitionChild(Rc::new(RefCell::new(None))))?;
    prepare_metatable(lua, userdata)
}

fn prepare_metatable(lua: &Lua, userdata: AnyUserData) -> Result<()> {
    // Preserve luaur's typed userdata cell, layering the source's strict index
    // and readonly metatable over its field dispatcher, as the Path owner does.
    let metatable: Table = unsafe {
        lua.exec_raw(userdata, |state| {
            luaur_vm::functions::lua_getmetatable::lua_getmetatable(state, 1)?;
            Ok(())
        })?
    };
    if !metatable.is_readonly() {
        metatable.set(
            "__index",
            lua.create_function(move |lua, (userdata, key): (AnyUserData, Value)| {
                let key: luaur_rt::LuaString = lua.unpack(key)?;
                let child = userdata.borrow::<TransitionChild>()?;
                let value = match key.to_string_lossy().as_str() {
                    "width" => child
                        .0
                        .borrow()
                        .as_ref()
                        .and_then(|child| child.artboard.as_ref())
                        .and_then(|artboard| artboard.with_downcast::<Artboard, _>(Artboard::width))
                        .unwrap_or(0.0),
                    "height" => child
                        .0
                        .borrow()
                        .as_ref()
                        .and_then(|child| child.artboard.as_ref())
                        .and_then(|artboard| {
                            artboard.with_downcast::<Artboard, _>(Artboard::height)
                        })
                        .unwrap_or(0.0),
                    _ => {
                        return Err(Error::runtime(format!(
                            "'{}' is not a valid index of TransitionChild",
                            key.to_string_lossy()
                        )));
                    }
                };
                Ok(value)
            })?,
        )?;
        metatable.set_readonly(true);
    }
    Ok(())
}

impl UserData for TransitionChild {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("width", |_, this| {
            Ok(this
                .0
                .borrow()
                .as_ref()
                .and_then(|child| child.artboard.as_ref())
                .and_then(|artboard| artboard.with_downcast::<Artboard, _>(Artboard::width))
                .unwrap_or(0.0))
        });
        fields.add_field_method_get("height", |_, this| {
            Ok(this
                .0
                .borrow()
                .as_ref()
                .and_then(|child| child.artboard.as_ref())
                .and_then(|artboard| artboard.with_downcast::<Artboard, _>(Artboard::height))
                .unwrap_or(0.0))
        });
    }
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_meta_method("__namecall", |lua, this, args: MultiValue| {
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
            if name != "draw" {
                return Err(Error::runtime(format!(
                    "{name} is not a valid method of TransitionChild"
                )));
            }
            let renderer: AnyUserData =
                lua.unpack(args.into_iter().next().unwrap_or(Value::Nil))?;
            let renderer = renderer.borrow::<ScriptedRenderer>()?;
            // Validate the renderer even when this child has expired.
            renderer.with_renderer_mut(|renderer| {
                let child = this.0.borrow().clone();
                if let Some(child) = child
                    && let Some(artboard) = child.artboard
                {
                    renderer.save();
                    renderer.transform(child.transform);
                    Artboard::draw_internal_handle(&artboard, renderer);
                    renderer.restore();
                }
                Ok(())
            })
        });
    }
}

pub(super) fn call_changed(
    table: &Table,
    from: &ScriptTransitionChildRef,
    to: &ScriptTransitionChildRef,
    direction: i32,
    owner: Option<super::view_model::ScriptedPropertyListenerOwner>,
) -> Result<()> {
    let Value::Function(function) = table.get::<Value>("changed")? else {
        return Ok(());
    };
    let lua = table.lua();
    let (from, _from_scope) = push_child(&lua, from)?;
    let (to, _to_scope) = push_child(&lua, to)?;
    let _property_owner = owner.map(|owner| {
        super::view_model::ScriptViewModelFrameContext::for_lua(&lua).enter_owner(owner)
    });
    function.protected_call((table.clone(), from, to, direction))
}

pub(super) fn call_draw(
    bindings: &RendererBindings,
    table: &Table,
    renderer: &mut dyn Renderer,
    from: &ScriptTransitionChildRef,
    to: &ScriptTransitionChildRef,
    owner: Option<super::view_model::ScriptedPropertyListenerOwner>,
) -> Result<()> {
    let lua = table.lua();
    let (scripted_renderer, _renderer_scope) =
        ScriptedRenderer::create_call_scoped_userdata(&lua, renderer, bindings.clone())?;
    let result = (|| {
        let Value::Function(function) = table.get::<Value>("draw")? else {
            return Ok(());
        };
        let (from, _from_scope) = push_child(&lua, from)?;
        let (to, _to_scope) = push_child(&lua, to)?;
        let _property_owner = owner.map(|owner| {
            super::view_model::ScriptViewModelFrameContext::for_lua(&lua).enter_owner(owner)
        });
        function.protected_call::<()>((table.clone(), scripted_renderer.clone(), from, to))
    })();
    scripted_renderer.borrow::<ScriptedRenderer>()?.end();
    result
}
