//! The bytecode loader (C++ `luau_load`, `VM/src/lvmload.cpp`).
//!
//! Every read is checked; a short or malformed chunk is a
//! [`LoadError::Malformed`], never a panic. Each prototype is verified
//! (`verify.rs`) as soon as it is parsed, before any closure over it can
//! exist, so the interpreter only ever sees verified code.
//!
//! Loading never runs Lua code and never collects. Objects it creates
//! (constant tables, `DUPCLOSURE` closures, interned strings) are reachable
//! from the prototypes, and the prototypes from the returned closure, which
//! the caller roots on the stack before the next safepoint.


use super::verify::verify;
use super::tag::{
    CONSTANT_BOOLEAN, CONSTANT_CLASS_SHAPE, CONSTANT_CLOSURE, CONSTANT_IMPORT, CONSTANT_INTEGER,
    CONSTANT_NIL, CONSTANT_NUMBER, CONSTANT_STRING, CONSTANT_TABLE, CONSTANT_TABLE_WITH_CONSTANTS,
    CONSTANT_VECTOR, CONSTANT_VECTORD, FEEDBACK_CALL_TARGET, TYPE_VERSION_MAX, TYPE_VERSION_MIN,
};
use super::{LineInfo, Proto, VERSION_MAX, VERSION_MIN};
use crate::error::{Error, ErrorKind, LoadError};
use crate::heap::{Function, LuaClosure, MEMORY_ERROR_MESSAGE};
use crate::table::TableError;
use crate::value::{FnId, ProtoId, StrId, TableId, Value};
use crate::vm::Core;


fn malformed(what: &str) -> Error {
    Error::Load(LoadError::Malformed(what.to_owned()))
}

fn unsupported(what: &str) -> Error {
    Error::Load(LoadError::Unsupported(what.to_owned()))
}

struct Reader<'a> {
    data: &'a [u8],
    pos: usize,
}

impl Reader<'_> {
    fn u8(&mut self) -> Result<u8, Error> {
        let byte = *self
            .data
            .get(self.pos)
            .ok_or_else(|| malformed("unexpected end of chunk"))?;
        self.pos = self.pos.saturating_add(1);
        Ok(byte)
    }

    fn bytes(&mut self, n: usize) -> Result<&[u8], Error> {
        let end = self
            .pos
            .checked_add(n)
            .ok_or_else(|| malformed("length overflow"))?;
        let slice = self
            .data
            .get(self.pos..end)
            .ok_or_else(|| malformed("unexpected end of chunk"))?;
        self.pos = end;
        Ok(slice)
    }

    fn array<const N: usize>(&mut self) -> Result<[u8; N], Error> {
        let slice = self.bytes(N)?;
        <[u8; N]>::try_from(slice).map_err(|_| malformed("unexpected end of chunk"))
    }

    fn u32(&mut self) -> Result<u32, Error> {
        Ok(u32::from_le_bytes(self.array()?))
    }

    fn i32(&mut self) -> Result<i32, Error> {
        Ok(i32::from_le_bytes(self.array()?))
    }

    fn f32(&mut self) -> Result<f32, Error> {
        Ok(f32::from_le_bytes(self.array()?))
    }

    fn f64(&mut self) -> Result<f64, Error> {
        Ok(f64::from_le_bytes(self.array()?))
    }

    fn varint(&mut self) -> Result<u32, Error> {
        let mut result: u32 = 0;
        let mut shift: u32 = 0;
        loop {
            let byte = self.u8()?;
            if shift >= 32 {
                return Err(malformed("varint overflow"));
            }
            result |= u32::from(byte & 127).checked_shl(shift).unwrap_or(0);
            shift = shift.saturating_add(7);
            if byte & 128 == 0 {
                return Ok(result);
            }
        }
    }

    /// A 64-bit varint (`readVarInt64`). Bits shifted past 64 are dropped,
    /// as upstream's shift drops them; more than ten bytes is malformed.
    fn varint64(&mut self) -> Result<u64, Error> {
        let mut result: u64 = 0;
        let mut shift: u32 = 0;
        loop {
            let byte = self.u8()?;
            if shift >= 70 {
                return Err(malformed("varint overflow"));
            }
            result |= u64::from(byte & 127).checked_shl(shift).unwrap_or(0);
            shift = shift.saturating_add(7);
            if byte & 128 == 0 {
                return Ok(result);
            }
        }
    }

    fn count(&mut self) -> Result<usize, Error> {
        let n = self.varint()?;
        let n = usize::try_from(n).map_err(|_| malformed("count overflow"))?;
        // A count can never exceed the bytes left; this bounds every
        // allocation sized from the chunk by the chunk's own size.
        if n > self.data.len().saturating_sub(self.pos) {
            return Err(malformed("count larger than the chunk"));
        }
        Ok(n)
    }
}

/// A string-table reference: 0 is none, `i` is the `i`-th string.
fn read_string(r: &mut Reader<'_>, strings: &[StrId]) -> Result<Option<StrId>, Error> {
    let index = r.varint()? as usize;
    if index == 0 {
        return Ok(None);
    }
    strings
        .get(index.saturating_sub(1))
        .copied()
        .map(Some)
        .ok_or_else(|| malformed("string index out of range"))
}

/// An empty vector with room for `n` elements, refused with the memory error
/// when they would pass the budget: the counts come from the chunk, and the
/// decoded buffers can be several times its size.
fn reserve<T>(vm: &Core, n: usize) -> Result<Vec<T>, Error> {
    vm.heap
        .check_alloc(n.saturating_mul(std::mem::size_of::<T>()))
        .map_err(|_| memory(vm))?;
    Ok(Vec::with_capacity(n))
}

/// A store into a table constant being built: a memory error if the
/// budget refuses it, a malformed-bytecode error for a bad key.
fn template_set(vm: &mut Core, table: TableId, key: Value, value: Value) -> Result<(), Error> {
    match vm.heap.table_set(table, key, value) {
        Ok(()) => Ok(()),
        Err(TableError::Memory) => Err(memory(vm)),
        Err(_) => Err(malformed("bad table template key")),
    }
}

fn memory(vm: &Core) -> Error {
    Error::Lua {
        kind: ErrorKind::Memory,
        message: vm.heap.str_bytes(MEMORY_ERROR_MESSAGE).to_vec(),
    }
}

/// Loads a chunk and returns the main closure, whose environment is `env`.
pub(crate) fn load(
    vm: &mut Core,
    chunk_name: &[u8],
    data: &[u8],
    env: TableId,
) -> Result<FnId, Error> {
    let limit = vm.limits.max_bytecode_bytes;
    if data.len() > limit {
        return Err(Error::Load(LoadError::TooLarge {
            size: data.len(),
            limit,
        }));
    }
    let mut r = Reader { data, pos: 0 };
    let version = r.u8()?;
    if version == 0 {
        let message = String::from_utf8_lossy(data.get(1..).unwrap_or_default()).into_owned();
        return Err(Error::Load(LoadError::CompileError(message)));
    }
    if !(VERSION_MIN..=VERSION_MAX).contains(&version) {
        return Err(Error::Load(LoadError::UnsupportedVersion(version)));
    }
    let types_version = if version >= 4 { r.u8()? } else { 0 };
    if version >= 4 && !(TYPE_VERSION_MIN..=TYPE_VERSION_MAX).contains(&types_version) {
        return Err(Error::Load(LoadError::UnsupportedTypesVersion(
            types_version,
        )));
    }

    let source = vm.heap.intern(chunk_name).map_err(|_| memory(vm))?;

    let string_count = r.count()?;
    let mut strings = reserve(vm, string_count)?;
    for _ in 0..string_count {
        let len = r.count()?;
        let bytes = r.bytes(len)?;
        let id = vm.heap.intern(bytes).map_err(|_| memory(vm))?;
        strings.push(id);
    }

    // Userdata type names (type-info version 3); the remapping only
    // affects type info, which this VM does not use.
    if types_version == 3 {
        let mut index = r.u8()?;
        while index != 0 {
            read_string(&mut r, &strings)?;
            index = r.u8()?;
        }
    }

    let proto_count = r.count()?;
    let mut protos: Vec<ProtoId> = reserve(vm, proto_count)?;
    for index in 0..proto_count {
        let proto = read_proto(
            vm,
            &mut r,
            version,
            types_version,
            &strings,
            &protos,
            env,
            source,
        )
        .map_err(|error| match error {
            Error::Load(LoadError::Malformed(message)) => {
                Error::Load(LoadError::Malformed(format!("proto {index}: {message}")))
            }
            other => other,
        })?;
        let id = vm.heap.new_proto(proto).map_err(|_| memory(vm))?;
        protos.push(id);
    }

    let main = r.varint()? as usize;
    let main_id = protos
        .get(main)
        .copied()
        .ok_or_else(|| malformed("main proto out of range"))?;
    if vm.heap.proto(main_id).nups != 0 {
        return Err(malformed("main proto has upvalues"));
    }
    if r.pos != data.len() {
        return Err(malformed("trailing bytes after the chunk"));
    }
    let closure = LuaClosure {
        proto_id: main_id,
        env,
        upvals: Vec::new(),
        preload: false,
    };
    vm.heap
        .new_function(Function::Lua(closure))
        .map_err(|_| memory(vm))
}

#[allow(clippy::too_many_arguments)]
fn read_proto(
    vm: &mut Core,
    r: &mut Reader<'_>,
    version: u8,
    types_version: u8,
    strings: &[StrId],
    loaded: &[ProtoId],
    env: TableId,
    source: StrId,
) -> Result<Proto, Error> {
    let maxstacksize = r.u8()?;
    let numparams = r.u8()?;
    let nups = r.u8()?;
    let is_vararg = r.u8()? != 0;
    if version >= 4 {
        let _flags = r.u8()?;
        if types_version >= 1 {
            let typesize = r.count()?;
            r.bytes(typesize)?;
        }
    }

    let sizecode = r.count()?;
    let mut code = reserve(vm, sizecode)?;
    for _ in 0..sizecode {
        code.push(r.u32()?);
    }

    let sizek = r.count()?;
    let mut k: Vec<Value> = reserve(vm, sizek)?;
    for _ in 0..sizek {
        let tag = r.u8()?;
        let value = match tag {
            CONSTANT_NIL => Value::Nil,
            CONSTANT_BOOLEAN => Value::Boolean(r.u8()? != 0),
            CONSTANT_NUMBER => Value::Number(r.f64()?),
            CONSTANT_STRING => match read_string(r, strings)? {
                Some(id) => Value::String(id),
                None => return Err(malformed("string constant without a string")),
            },
            CONSTANT_IMPORT => {
                let id = r.u32()?;
                resolve_import(vm, env, &k, id)?
            }
            CONSTANT_TABLE => {
                let keys = r.count()?;
                // Charged before it is built (`Heap::new_table`).
                let table = vm.heap.new_table(0, keys).map_err(|_| memory(vm))?;
                for _ in 0..keys {
                    let key = k
                        .get(r.varint()? as usize)
                        .copied()
                        .ok_or_else(|| malformed("table key out of range"))?;
                    template_set(vm, table, key, Value::Number(0.0))?;
                }
                Value::Table(table)
            }
            CONSTANT_TABLE_WITH_CONSTANTS => {
                let keys = r.count()?;
                // Charged before it is built (`Heap::new_table`).
                let table = vm.heap.new_table(0, keys).map_err(|_| memory(vm))?;
                let mut nil_keys = Vec::new();
                for _ in 0..keys {
                    let key = k
                        .get(r.varint()? as usize)
                        .copied()
                        .ok_or_else(|| malformed("table key out of range"))?;
                    let constant = r.i32()?;
                    let mut value = Value::Number(0.0);
                    if let Ok(constant) = usize::try_from(constant) {
                        let c = k
                            .get(constant)
                            .copied()
                            .ok_or_else(|| malformed("table value out of range"))?;
                        if c.is_nil() {
                            nil_keys.push(key);
                        } else {
                            value = c;
                        }
                    }
                    template_set(vm, table, key, value)?;
                }
                for key in nil_keys {
                    template_set(vm, table, key, Value::Nil)?;
                }
                Value::Table(table)
            }
            CONSTANT_CLOSURE => {
                let fid = r.varint()? as usize;
                let proto_id = loaded
                    .get(fid)
                    .copied()
                    .ok_or_else(|| malformed("closure constant names a later proto"))?;
                let nups = usize::from(vm.heap.proto(proto_id).nups);
                let closure = LuaClosure {
                    proto_id,
                    env,
                    upvals: vec![Value::Nil; nups],
                    preload: nups > 0,
                };
                Value::Function(
                    vm.heap
                        .new_function(Function::Lua(closure))
                        .map_err(|_| memory(vm))?,
                )
            }
            CONSTANT_VECTOR => {
                let x = r.f32()?;
                let y = r.f32()?;
                let z = r.f32()?;
                let _w = r.f32()?;
                Value::Vector([x, y, z])
            }
            CONSTANT_INTEGER => {
                // A sign byte, then the magnitude as a 64-bit varint
                // (`loadsafe`'s `LBC_CONSTANT_INTEGER`).
                let negative = r.u8()? != 0;
                let magnitude = r.varint64()?;
                let value = if negative {
                    (!magnitude).wrapping_add(1)
                } else {
                    magnitude
                };
                Value::Integer(value as i64)
            }
            CONSTANT_CLASS_SHAPE => {
                return Err(unsupported("class shape constants (user-defined classes)"));
            }
            CONSTANT_VECTORD => {
                return Err(unsupported(
                    "double-precision vector constants (bytecode v13)",
                ));
            }
            other => return Err(malformed(&format!("unknown constant kind {other}"))),
        };
        k.push(value);
    }

    let sizep = r.count()?;
    let mut children = reserve(vm, sizep)?;
    let mut child_nups = reserve(vm, sizep)?;
    for _ in 0..sizep {
        let fid = r.varint()? as usize;
        let id = *loaded
            .get(fid)
            .ok_or_else(|| malformed("child proto out of range"))?;
        children.push(id);
        child_nups.push(vm.heap.proto(id).nups);
    }

    let linedefined = r.varint()?;
    let debugname = read_string(r, strings)?;

    let lineinfo = if r.u8()? != 0 {
        let gaplog2 = u32::from(r.u8()?);
        if gaplog2 >= 32 {
            return Err(malformed("line gap too large"));
        }
        let intervals = match sizecode.checked_sub(1) {
            Some(last) => last.checked_shr(gaplog2).unwrap_or(0).saturating_add(1),
            None => 0,
        };
        let mut offsets = reserve(vm, sizecode)?;
        let mut last: u8 = 0;
        for _ in 0..sizecode {
            last = last.wrapping_add(r.u8()?);
            offsets.push(last);
        }
        let mut bases = reserve(vm, intervals)?;
        let mut line: i32 = 0;
        for _ in 0..intervals {
            line = line.wrapping_add(r.i32()?);
            bases.push(line);
        }
        Some(LineInfo {
            gaplog2,
            offsets: offsets.into(),
            bases: bases.into(),
        })
    } else {
        None
    };

    if r.u8()? != 0 {
        // Local and upvalue names: the debug library (lane 5) reads them;
        // lane 1 only checks they are well formed.
        let locals = r.count()?;
        for _ in 0..locals {
            read_string(r, strings)?;
            r.varint()?;
            r.varint()?;
            r.u8()?;
        }
        let upvalues = r.count()?;
        if upvalues != usize::from(nups) {
            return Err(malformed("upvalue name count differs from nups"));
        }
        for _ in 0..upvalues {
            read_string(r, strings)?;
        }
    }

    if version >= 11 {
        let feedback = r.count()?;
        for _ in 0..feedback {
            let kind = r.u8()?;
            if kind != FEEDBACK_CALL_TARGET {
                return Err(malformed("unknown feedback slot kind"));
            }
            r.varint()?;
        }
    }

    let proto = Proto {
        maxstacksize,
        numparams,
        nups,
        is_vararg,
        code: code.into(),
        k: k.into(),
        children: children.into(),
        source: Some(source),
        debugname,
        linedefined,
        lineinfo,
    };
    verify(&proto, &child_nups, &vm.heap)?;
    Ok(proto)
}

/// C++ `resolveImportSafe`: resolves an import path at load time when the
/// environment is safe, or leaves it nil for `GETIMPORT` to resolve at run
/// time. C++ runs a protected `gettable` per step; this walks raw gets and
/// `__index` tables only, and leaves the constant nil if it meets an
/// `__index` function, which `GETIMPORT` then calls at run time.
fn resolve_import(vm: &Core, env: TableId, k: &[Value], id: u32) -> Result<Value, Error> {
    let count = id >> 30;
    let ids = [(id >> 20) & 1023, (id >> 10) & 1023, id & 1023];
    for &key in ids.iter().take(count as usize) {
        if k.get(key as usize).is_none() {
            return Err(malformed("import names a later constant"));
        }
    }
    if !vm.heap.table(env).safeenv {
        return Ok(Value::Nil);
    }
    let mut current = Value::Table(env);
    for &key in ids.iter().take(count as usize) {
        let key = k.get(key as usize).copied().unwrap_or_default();
        current = match vm.raw_index_chain(current, key) {
            Some(v) => v,
            None => return Ok(Value::Nil),
        };
    }
    Ok(current)
}
