//! Owner-thread Lua promise adapter for `lua_scriptnet.cpp`.
use super::lua_promise;
use luaur_rt::{AnyUserData, Lua, Result, UserData, UserDataFields, UserDataMethods, Value};
use nuxie_runtime::{
    WorkPool,
    source::scriptnet::{
        http::{HttpHeader, HttpRequest, HttpResponse, NetError, NetErrorCode},
        net::{self, FetchListener},
    },
};
use std::{cell::RefCell, collections::HashMap, rc::Rc, sync::Arc};

enum Outcome {
    Response(HttpResponse),
    Error(String),
}
type Delivery = Rc<dyn Fn(u64, Outcome)>;
thread_local! { static DELIVERIES: RefCell<HashMap<u64, Delivery>> = RefCell::new(HashMap::new()); }
struct Registry {
    owner: u64,
    pending: RefCell<HashMap<u64, super::lua_rive_file::MainThreadRef>>,
}
impl Drop for Registry {
    fn drop(&mut self) {
        // App-data may be released during thread teardown, after this TLS
        // dispatch table is already gone. Owner cancellation still runs.
        let _ = DELIVERIES.try_with(|deliveries| {
            deliveries.borrow_mut().remove(&self.owner);
        });
        net::cancel_all_for_owner(self.owner);
    }
}
struct Listener {
    owner: u64,
    token: u64,
}
impl Listener {
    fn deliver(&self, outcome: Outcome) {
        let callback = DELIVERIES.with(|deliveries| deliveries.borrow().get(&self.owner).cloned());
        if let Some(callback) = callback {
            callback(self.token, outcome);
        }
    }
}
impl FetchListener for Listener {
    fn on_response(&self, response: HttpResponse) {
        self.deliver(Outcome::Response(response));
    }
    fn on_error(&self, error: &NetError) {
        let code = match error.code {
            NetErrorCode::Disabled => "disabled",
            NetErrorCode::Policy => "policy",
            NetErrorCode::RateLimited => "rateLimited",
            NetErrorCode::Timeout => "timeout",
            NetErrorCode::Aborted => "aborted",
            NetErrorCode::TooLarge => "tooLarge",
            NetErrorCode::Network => "network",
        };
        self.deliver(Outcome::Error(format!(
            "{code}: {}",
            error.message.split('\0').next().unwrap_or_default()
        )));
    }
    fn on_cancel(&self) {} // May run while the Lua VM is closing; never enter Lua.
}
struct Response(HttpResponse);
fn create_response(lua: &Lua, response: HttpResponse) -> Result<AnyUserData> {
    let value = lua.create_userdata(Response(response))?;
    super::lua_font::source_metatable(
        lua,
        &value,
        "Response",
        &["status", "statusText", "ok", "url", "headers"],
        &["header", "text", "arrayBuffer"],
        None,
        |value| value.borrow::<Response>().map(|_| ()),
    )?;
    super::lua_rive_file::c_prefix_index(
        lua,
        &value,
        "Response",
        &["status", "statusText", "ok", "url", "headers"],
    )?;
    Ok(value)
}
impl Response {
    fn header(&self, name: &[u8]) -> Option<Vec<u8>> {
        let values = self
            .0
            .headers
            .iter()
            .filter(|h| h.name.eq_ignore_ascii_case(name))
            .map(|h| h.value.as_slice())
            .collect::<Vec<_>>();
        (!values.is_empty()).then(|| values.join(b", ".as_slice()))
    }
}
impl UserData for Response {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("status", |_, this| Ok(this.0.status));
        fields.add_field_method_get("statusText", |lua, this| {
            Ok(lua.create_string(&this.0.status_text))
        });
        fields.add_field_method_get("ok", |_, this| Ok((200..300).contains(&this.0.status)));
        fields.add_field_method_get("url", |lua, this| Ok(lua.create_string(&this.0.url)));
        fields.add_field_method_get("headers", |lua, this| {
            let headers = lua.create_table();
            for header in &this.0.headers {
                let name = header.name.to_ascii_lowercase();
                let key = name.split(|byte| *byte == 0).next().unwrap_or_default();
                let value = match headers.get::<Value>(lua.create_string(key))? {
                    Value::String(previous) => {
                        let mut value = previous.as_bytes();
                        value.extend_from_slice(b", ");
                        value.extend_from_slice(&header.value);
                        value
                    }
                    _ => header.value.clone(),
                };
                headers.set(lua.create_string(key), lua.create_string(value))?;
            }
            Ok(headers)
        });
    }
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("header", |lua, this, name: luaur_rt::LuaString| {
            Ok(this
                .header(&name.as_bytes())
                .map(|value| lua.create_string(value)))
        });
        methods.add_method("text", |lua, this, ()| {
            let promise = lua_promise::new_pending(lua)?;
            lua_promise::resolve(
                lua,
                promise.clone(),
                Value::String(lua.create_string(&this.0.body)),
            )?;
            Ok(promise)
        });
        methods.add_method("arrayBuffer", |lua, this, ()| {
            let promise = lua_promise::new_pending(lua)?;
            lua_promise::resolve(
                lua,
                promise.clone(),
                Value::Buffer(lua.create_buffer(&this.0.body)?),
            )?;
            Ok(promise)
        });
    }
}
fn bad(message: &str) -> luaur_rt::Error {
    luaur_rt::Error::runtime(format!("fetch: {message}"))
}
pub(super) fn shutdown(lua: &Lua) {
    drop(lua.remove_app_data::<Registry>());
}
pub(super) fn has_pending_work(lua: &Lua) -> bool {
    lua.app_data_ref::<Registry>()
        .is_some_and(|registry| net::has_pending_work_for_owner(registry.owner))
}
pub(super) fn install(lua: &Lua) -> Result<()> {
    super::lua_main_ref::install_main_lua(lua);
    if lua.app_data_ref::<Registry>().is_none() {
        let owner = WorkPool::next_owner_id();
        let weak = lua.weak();
        DELIVERIES.with(|deliveries| {
            deliveries.borrow_mut().insert(
                owner,
                Rc::new(move |token, outcome| {
                    let Some(lua) = weak.try_upgrade() else {
                        return;
                    };
                    let key = lua
                        .app_data_ref::<Registry>()
                        .and_then(|registry| registry.pending.borrow_mut().remove(&token));
                    let Some(key) = key else {
                        return;
                    };
                    let result = (|| -> Result<()> {
                        let promise: AnyUserData = key.get(&lua)?;
                        match outcome {
                            Outcome::Response(response) => lua_promise::resolve(
                                &lua,
                                promise,
                                Value::UserData(create_response(&lua, response)?),
                            ),
                            Outcome::Error(error) => lua_promise::reject(&lua, promise, error),
                        }
                    })();
                    drop(key);
                    if let Err(error) = result {
                        eprintln!("fetch completion: {error}");
                    }
                }),
            )
        });
        lua.set_app_data(Registry {
            owner,
            pending: RefCell::new(HashMap::new()),
        });
    }
    lua.globals().set(
        "fetch",
        lua.create_function(|lua, (url, options): (Value, Value)| {
            // Match luaL_checkstring followed by luaL_checktype before
            // checking any option fields or allocating the promise.
            let url = lua
                .unpack::<luaur_rt::LuaString>(url)
                .map_err(|_| bad("bad argument #1 (string expected)"))?;
            let options = match options {
                Value::Nil => None,
                Value::Table(options) => Some(options),
                _ => return Err(bad("bad argument #2 (table expected)")),
            };
            if let Some(options) = options.as_ref() {
                match options.get::<Value>("method")? {
                    Value::Nil => {}
                    Value::String(_) => {}
                    _ => return Err(bad("options.method must be a string")),
                }
                match options.get::<Value>("headers")? {
                    Value::Nil => {}
                    Value::Table(headers) => {
                        for pair in headers.pairs::<Value, Value>() {
                            let (Value::String(_), Value::String(_)) = pair? else {
                                return Err(bad(
                                    "options.headers must map header names to string values",
                                ));
                            };
                        }
                    }
                    _ => return Err(bad("options.headers must be a table")),
                }
                match options.get::<Value>("body")? {
                    Value::Nil => {}
                    Value::String(_) | Value::Buffer(_) => {}
                    _ => return Err(bad("options.body must be a string or a buffer")),
                }
                let timeout = match options.get::<Value>("timeout")? {
                    Value::Nil => None,
                    Value::Integer(value) => Some(value as f64),
                    Value::Number(value) => Some(value),
                    _ => {
                        return Err(bad(
                            "options.timeout must be a non-negative number of milliseconds",
                        ));
                    }
                };
                if let Some(timeout) = timeout {
                    if !(timeout >= 0.0) {
                        return Err(bad(
                            "options.timeout must be a non-negative number of milliseconds",
                        ));
                    }
                }
            }
            let owner = lua
                .app_data_ref::<Registry>()
                .ok_or_else(|| bad("this Lua state has no scripting context"))?
                .owner;
            let promise = lua_promise::new_pending(lua)?;
            let token = WorkPool::next_owner_id();
            {
                let registry = lua
                    .app_data_ref::<Registry>()
                    .ok_or_else(|| bad("this Lua state has no scripting context"))?;
                registry.pending.borrow_mut().insert(
                    token,
                    super::lua_rive_file::MainThreadRef::new(lua, promise.clone())?,
                );
            }
            // Upstream reads fields again after retaining the promise.
            // __index may return different values on this second pass.
            let mut request = HttpRequest {
                url: url.as_bytes(),
                ..HttpRequest::default()
            };
            if let Some(options) = options {
                if let Value::String(value) = options.get::<Value>("method")? {
                    request.method = value.as_bytes();
                }
                if let Value::Table(headers) = options.get::<Value>("headers")? {
                    for pair in headers.pairs::<Value, Value>() {
                        let (name, value) = pair?;
                        let bytes = |value| {
                            lua.unpack::<luaur_rt::LuaString>(value)
                                .map(|s| s.as_bytes())
                                .unwrap_or_default()
                        };
                        request.headers.push(HttpHeader {
                            name: bytes(name),
                            value: bytes(value),
                        });
                    }
                }
                match options.get::<Value>("body")? {
                    Value::String(value) => request.body = value.as_bytes(),
                    Value::Buffer(value) => request.body = value.to_vec(),
                    _ => {}
                }
                match options.get::<Value>("timeout")? {
                    Value::Integer(value) => {
                        request.timeout_ms = (value as f64).min(u32::MAX as f64) as u32
                    }
                    Value::Number(value) => request.timeout_ms = value.min(u32::MAX as f64) as u32,
                    _ => {}
                }
            }
            let id = net::fetch(owner, request, Arc::new(Listener { owner, token }));
            lua_promise::set_on_cancel(
                lua,
                promise.clone(),
                lua.create_function(move |lua, ()| {
                    if net::cancel(id) {
                        let key = lua
                            .app_data_ref::<Registry>()
                            .and_then(|registry| registry.pending.borrow_mut().remove(&token));
                        if let Some(key) = key {
                            drop(key);
                        }
                    }
                    Ok(())
                })?,
            )?;
            Ok(promise)
        })?,
    )
}
