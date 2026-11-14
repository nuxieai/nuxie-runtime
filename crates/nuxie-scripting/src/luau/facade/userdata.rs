//! Userdata (the old binding crate's `UserData`, `AnyUserData`): the same
//! registration traits, and a per-type metatable built the same way: methods in
//! a method table, metamethods in the metatable, and for a type with fields an
//! `__index`/`__newindex` pair that tries a getter or setter first
//! (`Vm::userdata_dispatchers`, the native form of the old binding crate's
//! `USERDATA_DISPATCH` chunk).
//!
//! The value lives in the safe VM's userdata as a plain `T` (the VM keeps it in
//! an `Rc<RefCell<T>>`). A facade [`AnyUserData`] holds that cell beside the VM
//! handle, so [`AnyUserData::borrow`] returns a guard that borrows from the
//! handle, as the old binding crate's does. Differences from the old binding
//! crate: no `take` (the bindings never take a value out), so the
//! `UserDataDestructed` error never arises.

use std::any::{Any, TypeId};
use std::cell::{Ref, RefCell, RefMut};
use std::ffi::c_void;
use std::fmt;
use std::rc::Rc;

use super::conversion::{FromLua, FromLuaMulti, IntoLua, IntoLuaMulti};
use super::error::{Error, Result};
use super::handles::Table;
use super::loan::Loan;
use super::state::{BoxedCallback, Lua};
use super::value::Value;
use super::vm;

/// Kinds of metamethods that can be overridden on userdata.
///
/// Mirrors `mlua::MetaMethod`. The variant set is the Luau-relevant subset
/// (bitwise operators specific to Lua 5.3/5.4 are omitted; Luau does not use
/// them as metamethods).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum MetaMethod {
    /// The `+` operator (`__add`).
    Add,
    /// The `-` operator (`__sub`).
    Sub,
    /// The `*` operator (`__mul`).
    Mul,
    /// The `/` operator (`__div`).
    Div,
    /// The `%` operator (`__mod`).
    Mod,
    /// The `^` operator (`__pow`).
    Pow,
    /// The unary minus operator (`__unm`).
    Unm,
    /// The floor-division `//` operator (`__idiv`).
    IDiv,
    /// The string concatenation operator `..` (`__concat`).
    Concat,
    /// The length operator `#` (`__len`).
    Len,
    /// The `==` operator (`__eq`).
    Eq,
    /// The `<` operator (`__lt`).
    Lt,
    /// The `<=` operator (`__le`).
    Le,
    /// Index access `obj[key]` (`__index`).
    Index,
    /// Index write access `obj[key] = value` (`__newindex`).
    NewIndex,
    /// The call operator `obj(...)` (`__call`).
    Call,
    /// The `__tostring` metamethod.
    ToString,
    /// The `__iter` metamethod (Luau).
    Iter,
    /// The `__type`/`__name` metafield.
    Type,
}

impl MetaMethod {
    /// The Lua metamethod name (e.g. `"__add"`). Mirrors `mlua::MetaMethod::name`.
    pub const fn name(self) -> &'static str {
        match self {
            MetaMethod::Add => "__add",
            MetaMethod::Sub => "__sub",
            MetaMethod::Mul => "__mul",
            MetaMethod::Div => "__div",
            MetaMethod::Mod => "__mod",
            MetaMethod::Pow => "__pow",
            MetaMethod::Unm => "__unm",
            MetaMethod::IDiv => "__idiv",
            MetaMethod::Concat => "__concat",
            MetaMethod::Len => "__len",
            MetaMethod::Eq => "__eq",
            MetaMethod::Lt => "__lt",
            MetaMethod::Le => "__le",
            MetaMethod::Index => "__index",
            MetaMethod::NewIndex => "__newindex",
            MetaMethod::Call => "__call",
            MetaMethod::ToString => "__tostring",
            MetaMethod::Iter => "__iter",
            MetaMethod::Type => "__type",
        }
    }
}

impl fmt::Display for MetaMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

impl From<MetaMethod> for String {
    fn from(mm: MetaMethod) -> String {
        mm.name().to_string()
    }
}

impl PartialEq<MetaMethod> for &str {
    fn eq(&self, other: &MetaMethod) -> bool {
        *self == other.name()
    }
}

impl PartialEq<MetaMethod> for String {
    fn eq(&self, other: &MetaMethod) -> bool {
        self == other.name()
    }
}

/// A type Lua code can hold (the old binding crate's `UserData`).
pub trait UserData: Sized {
    fn add_fields<F: UserDataFields<Self>>(_fields: &mut F) {}
    fn add_methods<M: UserDataMethods<Self>>(_methods: &mut M) {}
}

/// Method registration (the old binding crate's `UserDataMethods`).
pub trait UserDataMethods<T> {
    fn add_method<M, A, R>(&mut self, name: impl Into<String>, method: M)
    where
        M: Fn(&Lua, &T, A) -> Result<R> + 'static,
        A: FromLuaMulti,
        R: IntoLuaMulti;

    fn add_method_mut<M, A, R>(&mut self, name: impl Into<String>, method: M)
    where
        M: Fn(&Lua, &mut T, A) -> Result<R> + 'static,
        A: FromLuaMulti,
        R: IntoLuaMulti;

    /// [`UserDataMethods::add_method`] for a method that uses the loan its
    /// call carries ([`Loan`]).
    fn add_method_lent<M, A, R>(&mut self, name: impl Into<String>, method: M)
    where
        M: Fn(&Lua, &T, Loan<'_>, A) -> Result<R> + 'static,
        A: FromLuaMulti,
        R: IntoLuaMulti;

    fn add_function<F, A, R>(&mut self, name: impl Into<String>, function: F)
    where
        F: Fn(&Lua, A) -> Result<R> + 'static,
        A: FromLuaMulti,
        R: IntoLuaMulti;

    fn add_meta_method<M, A, R>(&mut self, name: impl Into<String>, method: M)
    where
        M: Fn(&Lua, &T, A) -> Result<R> + 'static,
        A: FromLuaMulti,
        R: IntoLuaMulti;

    fn add_meta_method_mut<M, A, R>(&mut self, name: impl Into<String>, method: M)
    where
        M: Fn(&Lua, &mut T, A) -> Result<R> + 'static,
        A: FromLuaMulti,
        R: IntoLuaMulti;

    /// [`UserDataMethods::add_meta_method`] for a metamethod that uses, or
    /// passes on, the loan its call carries ([`Loan`]).
    fn add_meta_method_lent<M, A, R>(&mut self, name: impl Into<String>, method: M)
    where
        M: Fn(&Lua, &T, Loan<'_>, A) -> Result<R> + 'static,
        A: FromLuaMulti,
        R: IntoLuaMulti;
}

/// Field registration (the old binding crate's `UserDataFields`).
pub trait UserDataFields<T> {
    fn add_field<V>(&mut self, name: impl Into<String>, value: V)
    where
        V: IntoLua + Clone + 'static;

    fn add_field_method_get<M, R>(&mut self, name: impl Into<String>, method: M)
    where
        M: Fn(&Lua, &T) -> Result<R> + 'static,
        R: IntoLua;

    fn add_field_index_method_get<M, R>(&mut self, index: i64, method: M)
    where
        M: Fn(&Lua, &T) -> Result<R> + 'static,
        R: IntoLua;

    fn add_field_method_set<M, A>(&mut self, name: impl Into<String>, method: M)
    where
        M: Fn(&Lua, &mut T, A) -> Result<()> + 'static,
        A: FromLua;

    fn add_field_index_method_set<M, A>(&mut self, index: i64, method: M)
    where
        M: Fn(&Lua, &mut T, A) -> Result<()> + 'static,
        A: FromLua;

    fn add_field_function_get<F, R>(&mut self, name: impl Into<String>, function: F)
    where
        F: Fn(&Lua, AnyUserData) -> Result<R> + 'static,
        R: IntoLua;

    fn add_field_function_set<F, A>(&mut self, name: impl Into<String>, function: F)
    where
        F: Fn(&Lua, AnyUserData, A) -> Result<()> + 'static,
        A: FromLua;
}

// ---- AnyUserData ------------------------------------------------------------

/// A userdata handle (the old binding crate's `AnyUserData`).
#[derive(Clone)]
pub struct AnyUserData {
    lua: Lua,
    raw: vm::AnyUserData,
    /// The VM's `Rc<RefCell<T>>`, erased; `None` for a userdata the VM
    /// could not report a payload for.
    payload: Option<Rc<dyn Any>>,
}

impl AnyUserData {
    pub(crate) fn wrap(lua: &Lua, vm: &vm::Vm, raw: vm::AnyUserData) -> AnyUserData {
        let payload = raw.payload(vm).ok();
        AnyUserData {
            lua: lua.clone(),
            raw,
            payload,
        }
    }

    pub(crate) fn raw(&self) -> &vm::AnyUserData {
        &self.raw
    }

    pub fn lua(&self) -> Lua {
        self.lua.clone()
    }

    fn cell<T: 'static>(&self) -> Result<&RefCell<T>> {
        self.payload
            .as_deref()
            .and_then(|p| p.downcast_ref::<RefCell<T>>())
            .ok_or(Error::UserDataTypeMismatch)
    }

    /// The value's cell, shared (for a guard that must outlive the handle).
    pub(crate) fn shared_cell<T: 'static>(&self) -> Result<Rc<RefCell<T>>> {
        let payload = self.payload.clone().ok_or(Error::UserDataTypeMismatch)?;
        payload
            .downcast::<RefCell<T>>()
            .map_err(|_| Error::UserDataTypeMismatch)
    }

    pub fn is<T: 'static>(&self) -> bool {
        self.cell::<T>().is_ok()
    }

    pub fn borrow<T: 'static>(&self) -> Result<UserDataRef<'_, T>> {
        let guard = self
            .cell::<T>()?
            .try_borrow()
            .map_err(|_| Error::UserDataBorrowError)?;
        Ok(UserDataRef { guard })
    }

    pub fn borrow_mut<T: 'static>(&self) -> Result<UserDataRefMut<'_, T>> {
        let guard = self
            .cell::<T>()?
            .try_borrow_mut()
            .map_err(|_| Error::UserDataBorrowMutError)?;
        Ok(UserDataRefMut { guard })
    }

    /// The metatable (seam rows R10 to R20: what the bindings read through
    /// `getmetatable` in a raw hatch on the deleted vendored VM).
    pub fn metatable(&self) -> Result<Option<Table>> {
        let lua = self.lua.clone();
        self.lua.0.with_vm(|vm| {
            Ok(self
                .raw
                .metatable(vm)
                .map_err(Error::from_vm)?
                .map(|t| Table::wrap(&lua, t)))
        })
    }

    pub fn to_pointer(&self) -> *const c_void {
        let address = self
            .lua
            .0
            .with_vm(|vm| self.raw.to_pointer(vm).map_err(Error::from_vm))
            .unwrap_or(0);
        std::ptr::without_provenance(usize::try_from(address).unwrap_or(usize::MAX))
    }
}

impl fmt::Debug for AnyUserData {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "UserData")
    }
}

impl PartialEq for AnyUserData {
    /// Identity: the same object of the same VM.
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.lua.0, &other.lua.0) && self.to_pointer() == other.to_pointer()
    }
}

/// A shared borrow of a userdata's value.
pub struct UserDataRef<'a, T> {
    guard: Ref<'a, T>,
}

impl<T> std::ops::Deref for UserDataRef<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.guard
    }
}

impl<T: fmt::Debug> fmt::Debug for UserDataRef<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<T: fmt::Display> fmt::Display for UserDataRef<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}

/// An exclusive borrow of a userdata's value.
pub struct UserDataRefMut<'a, T> {
    guard: RefMut<'a, T>,
}

impl<T> std::ops::Deref for UserDataRefMut<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.guard
    }
}

impl<T> std::ops::DerefMut for UserDataRefMut<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.guard
    }
}

impl<T: fmt::Debug> fmt::Debug for UserDataRefMut<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<T: fmt::Display> fmt::Display for UserDataRefMut<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}

impl<T: UserData + 'static> IntoLua for T {
    fn into_lua(self, lua: &Lua) -> Result<Value> {
        Ok(Value::UserData(lua.create_userdata(self)?))
    }
}

impl IntoLua for AnyUserData {
    fn into_lua(self, _lua: &Lua) -> Result<Value> {
        Ok(Value::UserData(self))
    }
}

impl IntoLua for &AnyUserData {
    fn into_lua(self, _lua: &Lua) -> Result<Value> {
        Ok(Value::UserData(self.clone()))
    }
}

impl FromLua for AnyUserData {
    fn from_lua(value: Value, _lua: &Lua) -> Result<Self> {
        match value {
            Value::UserData(ud) => Ok(ud),
            other => Err(Error::FromLuaConversionError {
                from: other.type_name(),
                to: "AnyUserData".to_string(),
                message: None,
            }),
        }
    }
}

// ---- registration -----------------------------------------------------------

/// The receiver's cell: argument 1 as a `T` userdata.
fn receiver<T: 'static>(this: &Value) -> Result<Rc<RefCell<T>>> {
    match this {
        Value::UserData(ud) => ud.shared_cell::<T>(),
        _ => Err(Error::UserDataTypeMismatch),
    }
}

struct Registered {
    name: String,
    is_meta: bool,
    callback: BoxedCallback,
}

struct FieldEntry {
    key: FieldKey,
    is_get: bool,
    callback: BoxedCallback,
}

enum FieldKey {
    Name(String),
    Index(i64),
}

struct Collector<T> {
    methods: Vec<Registered>,
    fields: Vec<FieldEntry>,
    _phantom: std::marker::PhantomData<T>,
}

impl<T: 'static> Collector<T> {
    fn method(&mut self, name: String, is_meta: bool, callback: BoxedCallback) {
        self.methods.push(Registered {
            name,
            is_meta,
            callback,
        });
    }

    fn field(&mut self, key: FieldKey, is_get: bool, callback: BoxedCallback) {
        self.fields.push(FieldEntry {
            key,
            is_get,
            callback,
        });
    }
}

fn shared_method<T, M, A, R>(method: M) -> BoxedCallback
where
    T: 'static,
    M: Fn(&Lua, &T, A) -> Result<R> + 'static,
    A: FromLuaMulti,
    R: IntoLuaMulti,
{
    Box::new(move |lua, _loan, mut args| {
        let this = args.pop_front().unwrap_or(Value::Nil);
        let cell = receiver::<T>(&this)?;
        let a = A::from_lua_multi(args, lua)?;
        let data = cell.try_borrow().map_err(|_| Error::UserDataBorrowError)?;
        let r = method(lua, &data, a)?;
        drop(data);
        r.into_lua_multi(lua)
    })
}

fn lent_method<T, M, A, R>(method: M) -> BoxedCallback
where
    T: 'static,
    M: Fn(&Lua, &T, Loan<'_>, A) -> Result<R> + 'static,
    A: FromLuaMulti,
    R: IntoLuaMulti,
{
    Box::new(move |lua, loan, mut args| {
        let this = args.pop_front().unwrap_or(Value::Nil);
        let cell = receiver::<T>(&this)?;
        let a = A::from_lua_multi(args, lua)?;
        let data = cell.try_borrow().map_err(|_| Error::UserDataBorrowError)?;
        let r = method(lua, &data, loan, a)?;
        drop(data);
        r.into_lua_multi(lua)
    })
}

fn mut_method<T, M, A, R>(method: M) -> BoxedCallback
where
    T: 'static,
    M: Fn(&Lua, &mut T, A) -> Result<R> + 'static,
    A: FromLuaMulti,
    R: IntoLuaMulti,
{
    Box::new(move |lua, _loan, mut args| {
        let this = args.pop_front().unwrap_or(Value::Nil);
        let cell = receiver::<T>(&this)?;
        let a = A::from_lua_multi(args, lua)?;
        let mut data = cell
            .try_borrow_mut()
            .map_err(|_| Error::UserDataBorrowMutError)?;
        let r = method(lua, &mut data, a)?;
        drop(data);
        r.into_lua_multi(lua)
    })
}

fn getter<T, M, R>(method: M) -> BoxedCallback
where
    T: 'static,
    M: Fn(&Lua, &T) -> Result<R> + 'static,
    R: IntoLua,
{
    Box::new(move |lua, _loan, mut args| {
        let this = args.pop_front().unwrap_or(Value::Nil);
        let cell = receiver::<T>(&this)?;
        let data = cell.try_borrow().map_err(|_| Error::UserDataBorrowError)?;
        let r = method(lua, &data)?;
        drop(data);
        r.into_lua_multi(lua)
    })
}

fn setter<T, M, A>(method: M) -> BoxedCallback
where
    T: 'static,
    M: Fn(&Lua, &mut T, A) -> Result<()> + 'static,
    A: FromLua,
{
    Box::new(move |lua, _loan, mut args| {
        let this = args.pop_front().unwrap_or(Value::Nil);
        let cell = receiver::<T>(&this)?;
        let val = A::from_lua(args.pop_front().unwrap_or(Value::Nil), lua)?;
        let mut data = cell
            .try_borrow_mut()
            .map_err(|_| Error::UserDataBorrowMutError)?;
        method(lua, &mut data, val)?;
        drop(data);
        ().into_lua_multi(lua)
    })
}

impl<T: 'static> UserDataMethods<T> for Collector<T> {
    fn add_method<M, A, R>(&mut self, name: impl Into<String>, method: M)
    where
        M: Fn(&Lua, &T, A) -> Result<R> + 'static,
        A: FromLuaMulti,
        R: IntoLuaMulti,
    {
        self.method(name.into(), false, shared_method(method));
    }

    fn add_method_mut<M, A, R>(&mut self, name: impl Into<String>, method: M)
    where
        M: Fn(&Lua, &mut T, A) -> Result<R> + 'static,
        A: FromLuaMulti,
        R: IntoLuaMulti,
    {
        self.method(name.into(), false, mut_method(method));
    }

    fn add_method_lent<M, A, R>(&mut self, name: impl Into<String>, method: M)
    where
        M: Fn(&Lua, &T, Loan<'_>, A) -> Result<R> + 'static,
        A: FromLuaMulti,
        R: IntoLuaMulti,
    {
        self.method(name.into(), false, lent_method(method));
    }

    fn add_function<F, A, R>(&mut self, name: impl Into<String>, function: F)
    where
        F: Fn(&Lua, A) -> Result<R> + 'static,
        A: FromLuaMulti,
        R: IntoLuaMulti,
    {
        let callback: BoxedCallback = Box::new(move |lua, _loan, args| {
            let a = A::from_lua_multi(args, lua)?;
            function(lua, a)?.into_lua_multi(lua)
        });
        self.method(name.into(), false, callback);
    }

    fn add_meta_method<M, A, R>(&mut self, name: impl Into<String>, method: M)
    where
        M: Fn(&Lua, &T, A) -> Result<R> + 'static,
        A: FromLuaMulti,
        R: IntoLuaMulti,
    {
        self.method(name.into(), true, shared_method(method));
    }

    fn add_meta_method_mut<M, A, R>(&mut self, name: impl Into<String>, method: M)
    where
        M: Fn(&Lua, &mut T, A) -> Result<R> + 'static,
        A: FromLuaMulti,
        R: IntoLuaMulti,
    {
        self.method(name.into(), true, mut_method(method));
    }

    fn add_meta_method_lent<M, A, R>(&mut self, name: impl Into<String>, method: M)
    where
        M: Fn(&Lua, &T, Loan<'_>, A) -> Result<R> + 'static,
        A: FromLuaMulti,
        R: IntoLuaMulti,
    {
        self.method(name.into(), true, lent_method(method));
    }
}

impl<T: 'static> UserDataFields<T> for Collector<T> {
    fn add_field<V>(&mut self, name: impl Into<String>, value: V)
    where
        V: IntoLua + Clone + 'static,
    {
        let callback: BoxedCallback = Box::new(move |lua, _loan, _args| {
            value.clone().into_lua(lua)?.into_lua_multi(lua)
        });
        self.field(FieldKey::Name(name.into()), true, callback);
    }

    fn add_field_method_get<M, R>(&mut self, name: impl Into<String>, method: M)
    where
        M: Fn(&Lua, &T) -> Result<R> + 'static,
        R: IntoLua,
    {
        self.field(FieldKey::Name(name.into()), true, getter(method));
    }

    fn add_field_index_method_get<M, R>(&mut self, index: i64, method: M)
    where
        M: Fn(&Lua, &T) -> Result<R> + 'static,
        R: IntoLua,
    {
        self.field(FieldKey::Index(index), true, getter(method));
    }

    fn add_field_method_set<M, A>(&mut self, name: impl Into<String>, method: M)
    where
        M: Fn(&Lua, &mut T, A) -> Result<()> + 'static,
        A: FromLua,
    {
        self.field(FieldKey::Name(name.into()), false, setter(method));
    }

    fn add_field_index_method_set<M, A>(&mut self, index: i64, method: M)
    where
        M: Fn(&Lua, &mut T, A) -> Result<()> + 'static,
        A: FromLua,
    {
        self.field(FieldKey::Index(index), false, setter(method));
    }

    fn add_field_function_get<F, R>(&mut self, name: impl Into<String>, function: F)
    where
        F: Fn(&Lua, AnyUserData) -> Result<R> + 'static,
        R: IntoLua,
    {
        let callback: BoxedCallback = Box::new(move |lua, _loan, mut args| {
            let this = args.pop_front().unwrap_or(Value::Nil);
            let ud = AnyUserData::from_lua(this, lua)?;
            function(lua, ud)?.into_lua_multi(lua)
        });
        self.field(FieldKey::Name(name.into()), true, callback);
    }

    fn add_field_function_set<F, A>(&mut self, name: impl Into<String>, function: F)
    where
        F: Fn(&Lua, AnyUserData, A) -> Result<()> + 'static,
        A: FromLua,
    {
        let callback: BoxedCallback = Box::new(move |lua, _loan, mut args| {
            let this = args.pop_front().unwrap_or(Value::Nil);
            let ud = AnyUserData::from_lua(this, lua)?;
            let val = A::from_lua(args.pop_front().unwrap_or(Value::Nil), lua)?;
            function(lua, ud, val)?;
            ().into_lua_multi(lua)
        });
        self.field(FieldKey::Name(name.into()), false, callback);
    }
}

/// Builds `T`'s metatable once per VM and makes it the safe VM's metatable
/// for `T` (every later `create_userdata::<T>` gets it).
fn ensure_metatable<T: UserData + 'static>(lua: &Lua) -> Result<()> {
    let type_id = TypeId::of::<T>();
    let cached = lua
        .0
        .userdata_metatables
        .try_borrow()
        .map(|m| m.contains_key(&type_id))
        .unwrap_or(false);
    if cached {
        return Ok(());
    }

    let mut collector = Collector::<T> {
        methods: Vec::new(),
        fields: Vec::new(),
        _phantom: std::marker::PhantomData,
    };
    T::add_fields(&mut collector);
    T::add_methods(&mut collector);

    let method_table = lua.create_table_result()?;
    let metatable = lua.create_table_result()?;
    for item in collector.methods {
        let func = lua.create_callback_function(&item.name, item.callback)?;
        if item.is_meta {
            metatable.set(item.name, func)?;
        } else {
            method_table.set(item.name, func)?;
        }
    }

    if collector.fields.is_empty() {
        metatable.set("__index", &method_table)?;
    } else {
        let getters = lua.create_table_result()?;
        let setters = lua.create_table_result()?;
        for field in collector.fields {
            let (key, name): (Value, String) = match field.key {
                FieldKey::Name(name) => (name.as_str().into_lua(lua)?, name),
                FieldKey::Index(index) => (Value::Integer(index), index.to_string()),
            };
            let func = lua.create_callback_function(&name, field.callback)?;
            if field.is_get {
                getters.set(key, func)?;
            } else {
                setters.set(key, func)?;
            }
        }
        let (index, newindex) = lua.0.with_vm(|vm| {
            vm.userdata_dispatchers(getters.raw()?, setters.raw()?, method_table.raw()?)
                .map_err(Error::from_vm)
        })?;
        metatable.raw_set("__index", super::handles::Function::wrap(lua, index))?;
        metatable.raw_set("__newindex", super::handles::Function::wrap(lua, newindex))?;
    }

    let raw = metatable.raw()?.clone();
    lua.0.with_vm(|vm| {
        vm.set_userdata_metatable::<T>(Some(&raw))
            .map_err(Error::from_vm)
    })?;
    if let Ok(mut cache) = lua.0.userdata_metatables.try_borrow_mut() {
        cache.insert(type_id, raw);
    }
    Ok(())
}

/// The old binding crate's `create_userdata`: `data` in a userdata with `T`'s
/// metatable.
pub(crate) fn create_userdata<T: UserData + 'static>(lua: &Lua, data: T) -> Result<AnyUserData> {
    ensure_metatable::<T>(lua)?;
    lua.0.with_vm(|vm| {
        let raw = vm.create_userdata(data).map_err(Error::from_vm)?;
        Ok(AnyUserData::wrap(lua, vm, raw))
    })
}
