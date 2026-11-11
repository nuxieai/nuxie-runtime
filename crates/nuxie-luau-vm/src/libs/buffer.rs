//! The `buffer` library (C++ `lbuflib.cpp`), with `readinteger` and
//! `writeinteger` (the `LuauIntegerLibrary` table, which every profile the
//! harnesses run opens).
//!
//! The fast bodies (`luauF_readinteger`, `luauF_writefp` and the rest)
//! compute what the library functions compute on the inputs they accept,
//! so none is ported to `libs/fast.rs`. Values are little-endian.
//!
//! Allocating builtins: `create` and `fromstring` (a buffer, charged by
//! `Heap::new_buffer` before it is allocated), `tostring` and `readstring`
//! (a string, charged by `Heap::intern`).

use crate::error::VmResult;
use crate::heap::MAX_BUFFER_SIZE;
use crate::native::{Args, BuiltinFn};
use crate::value::{BufId, Value};
use crate::vm::Vm;

impl Vm {
    /// C++ `luaL_checkbuffer`.
    pub(crate) fn check_buffer_arg(&mut self, args: Args, n: usize) -> VmResult<BufId> {
        match self.arg(args, n) {
            Some(Value::Buffer(id)) => Ok(id),
            _ => Err(self.arg_type_error(args, n, "buffer")),
        }
    }

    /// A new zero-filled buffer value (C++ `lua_newbuffer`).
    pub(crate) fn new_buffer(&mut self, len: usize) -> VmResult<BufId> {
        if len > MAX_BUFFER_SIZE {
            return Err(self.runtime_error("memory allocation error: block too big"));
        }
        self.heap.new_buffer(len)
    }
}

/// C++ `isoutofbounds`: the offset is read as unsigned, so a negative one
/// is out of bounds too.
fn out_of_bounds(offset: i32, len: usize, size: usize) -> bool {
    u64::from(offset as u32).saturating_add(size as u64) > len as u64
}

/// The byte range `offset .. offset + size` of a buffer of `len` bytes,
/// or the bounds error.
fn range(vm: &mut Vm, offset: i32, len: usize, size: usize) -> VmResult<std::ops::Range<usize>> {
    if out_of_bounds(offset, len, size) {
        return Err(vm.lib_error("buffer access out of bounds"));
    }
    let start = offset as u32 as usize;
    Ok(start..start.saturating_add(size))
}

fn create(vm: &mut Vm, args: Args) -> VmResult<usize> {
    let size = vm.check_integer(args, 1)?;
    if size < 0 {
        return Err(vm.arg_error(1, "size"));
    }
    let id = vm.new_buffer(size.unsigned_abs() as usize)?;
    vm.ret(&[Value::Buffer(id)])
}

fn fromstring(vm: &mut Vm, args: Args) -> VmResult<usize> {
    let s = vm.check_string(args, 1)?;
    let len = vm.heap.str_bytes(s).len();
    let id = vm.new_buffer(len)?;
    let bytes = vm.heap.str_rc(s);
    vm.heap.buffer_mut(id).copy_from_slice(&bytes);
    vm.ret(&[Value::Buffer(id)])
}

fn tostring(vm: &mut Vm, args: Args) -> VmResult<usize> {
    let id = vm.check_buffer_arg(args, 1)?;
    let id = vm.heap.intern_buffer_slice(id, None)?;
    vm.ret(&[Value::String(id)])
}

fn len(vm: &mut Vm, args: Args) -> VmResult<usize> {
    let id = vm.check_buffer_arg(args, 1)?;
    let n = vm.heap.buffer(id).len();
    vm.ret(&[Value::Number(n as f64)])
}

/// Reads `N` bytes at argument 2's offset.
fn read_bytes<const N: usize>(vm: &mut Vm, args: Args) -> VmResult<[u8; N]> {
    let id = vm.check_buffer_arg(args, 1)?;
    let offset = vm.check_integer(args, 2)?;
    let len = vm.heap.buffer(id).len();
    let r = range(vm, offset, len, N)?;
    let mut out = [0u8; N];
    if let Some(src) = vm.heap.buffer(id).get(r) {
        out.copy_from_slice(src);
    }
    Ok(out)
}

/// Checks the buffer and offset, then argument 3 with `value`, then the
/// bounds (C++'s order), and writes the bytes `value` returns.
fn write_bytes<const N: usize>(
    vm: &mut Vm,
    args: Args,
    value: impl FnOnce(&mut Vm) -> VmResult<[u8; N]>,
) -> VmResult<usize> {
    let id = vm.check_buffer_arg(args, 1)?;
    let offset = vm.check_integer(args, 2)?;
    let bytes = value(vm)?;
    let len = vm.heap.buffer(id).len();
    let r = range(vm, offset, len, N)?;
    if let Some(dst) = vm.heap.buffer_mut(id).get_mut(r) {
        dst.copy_from_slice(&bytes);
    }
    Ok(0)
}

macro_rules! read_int {
    ($name:ident, $t:ty) => {
        fn $name(vm: &mut Vm, args: Args) -> VmResult<usize> {
            let v = <$t>::from_le_bytes(read_bytes(vm, args)?);
            vm.ret(&[Value::Number(f64::from(v))])
        }
    };
}

read_int!(readi8, i8);
read_int!(readu8, u8);
read_int!(readi16, i16);
read_int!(readu16, u16);
read_int!(readi32, i32);
read_int!(readu32, u32);
read_int!(readf32, f32);
read_int!(readf64, f64);

macro_rules! write_int {
    ($name:ident, $t:ty) => {
        fn $name(vm: &mut Vm, args: Args) -> VmResult<usize> {
            // `luaL_checkunsigned`, then the low bytes (C++ copies the
            // first `sizeof(T)` bytes of the `uint32`).
            write_bytes(vm, args, |vm| {
                Ok((vm.check_unsigned(args, 3)? as $t).to_le_bytes())
            })
        }
    };
}

write_int!(writei8, u8);
write_int!(writeu8, u8);
write_int!(writei16, u16);
write_int!(writeu16, u16);
write_int!(writei32, u32);
write_int!(writeu32, u32);

fn writef32(vm: &mut Vm, args: Args) -> VmResult<usize> {
    write_bytes(vm, args, |vm| {
        Ok((vm.check_number(args, 3)? as f32).to_le_bytes())
    })
}

fn writef64(vm: &mut Vm, args: Args) -> VmResult<usize> {
    write_bytes(vm, args, |vm| Ok(vm.check_number(args, 3)?.to_le_bytes()))
}

fn readinteger(vm: &mut Vm, args: Args) -> VmResult<usize> {
    let v = i64::from_le_bytes(read_bytes(vm, args)?);
    vm.ret(&[Value::Integer(v)])
}

fn writeinteger(vm: &mut Vm, args: Args) -> VmResult<usize> {
    write_bytes(vm, args, |vm| match vm.arg(args, 3) {
        // `luaL_checkinteger64` takes the integer type only.
        Some(Value::Integer(v)) => Ok(v.to_le_bytes()),
        _ => Err(vm.arg_type_error(args, 3, "integer")),
    })
}

fn readstring(vm: &mut Vm, args: Args) -> VmResult<usize> {
    let id = vm.check_buffer_arg(args, 1)?;
    let offset = vm.check_integer(args, 2)?;
    let size = vm.check_integer(args, 3)?;
    if size < 0 {
        return Err(vm.arg_error(3, "size"));
    }
    let len = vm.heap.buffer(id).len();
    let r = range(vm, offset, len, size.unsigned_abs() as usize)?;
    let s = vm.heap.intern_buffer_slice(id, Some(r))?;
    vm.ret(&[Value::String(s)])
}

fn writestring(vm: &mut Vm, args: Args) -> VmResult<usize> {
    let id = vm.check_buffer_arg(args, 1)?;
    let offset = vm.check_integer(args, 2)?;
    let s = vm.check_string(args, 3)?;
    let size = vm.heap.str_bytes(s).len();
    let count = vm.opt_integer(args, 4, size as i32)?;
    if count < 0 {
        return Err(vm.arg_error(4, "count"));
    }
    let count = count.unsigned_abs() as usize;
    if count > size {
        return Err(vm.lib_error("string length overflow"));
    }
    let len = vm.heap.buffer(id).len();
    let r = range(vm, offset, len, count)?;
    let bytes = vm.heap.str_rc(s);
    if let (Some(dst), Some(src)) = (vm.heap.buffer_mut(id).get_mut(r), bytes.get(..count)) {
        dst.copy_from_slice(src);
    }
    Ok(0)
}

fn copy(vm: &mut Vm, args: Args) -> VmResult<usize> {
    let target = vm.check_buffer_arg(args, 1)?;
    let toffset = vm.check_integer(args, 2)?;
    let source = vm.check_buffer_arg(args, 3)?;
    let soffset = vm.opt_integer(args, 4, 0)?;
    let slen = vm.heap.buffer(source).len();
    let size = vm.opt_integer(args, 5, (slen as i32).wrapping_sub(soffset))?;
    if size < 0 {
        return Err(vm.lib_error("buffer access out of bounds"));
    }
    let size = size.unsigned_abs() as usize;
    let src = range(vm, soffset, slen, size)?;
    let tlen = vm.heap.buffer(target).len();
    let dst = range(vm, toffset, tlen, size)?;
    if target == source {
        vm.heap.buffer_mut(target).copy_within(src, dst.start);
    } else {
        let bytes = vm.heap.buffer(source).get(src).map(<[u8]>::to_vec);
        if let (Some(bytes), Some(out)) = (bytes, vm.heap.buffer_mut(target).get_mut(dst)) {
            out.copy_from_slice(&bytes);
        }
    }
    Ok(0)
}

fn fill(vm: &mut Vm, args: Args) -> VmResult<usize> {
    let id = vm.check_buffer_arg(args, 1)?;
    let offset = vm.check_integer(args, 2)?;
    let value = vm.check_unsigned(args, 3)?;
    let len = vm.heap.buffer(id).len();
    let size = vm.opt_integer(args, 4, (len as i32).wrapping_sub(offset))?;
    if size < 0 {
        return Err(vm.lib_error("buffer access out of bounds"));
    }
    let r = range(vm, offset, len, size.unsigned_abs() as usize)?;
    if let Some(dst) = vm.heap.buffer_mut(id).get_mut(r) {
        dst.fill(value as u8);
    }
    Ok(0)
}

/// The checks `readbits` and `writebits` share (C++'s order); returns the
/// bit offset, the bit count and the byte range holding the bits.
fn bits_check(
    vm: &mut Vm,
    id: BufId,
    bitoffset: i64,
    bitcount: i32,
) -> VmResult<(u64, u32, std::ops::Range<usize>)> {
    if bitoffset < 0 {
        return Err(vm.lib_error("buffer access out of bounds"));
    }
    if bitcount.unsigned_abs() > 32 || bitcount < 0 {
        return Err(vm.lib_error("bit count is out of range of [0; 32]"));
    }
    let bitoffset = bitoffset.unsigned_abs();
    let bitcount = bitcount.unsigned_abs();
    let len = vm.heap.buffer(id).len() as u64;
    if bitoffset.saturating_add(u64::from(bitcount)) > len.saturating_mul(8) {
        return Err(vm.lib_error("buffer access out of bounds"));
    }
    let start = bitoffset / 8;
    let end = bitoffset.saturating_add(u64::from(bitcount)).saturating_add(7) / 8;
    Ok((bitoffset, bitcount, start as usize..end as usize))
}

/// Little-endian bytes as a `u64` (at most five of them).
fn load_le(bytes: &[u8]) -> u64 {
    bytes
        .iter()
        .rev()
        .fold(0u64, |acc, &b| (acc << 8) | u64::from(b))
}

fn readbits(vm: &mut Vm, args: Args) -> VmResult<usize> {
    let id = vm.check_buffer_arg(args, 1)?;
    let bitoffset = vm.check_number(args, 2)? as i64;
    let bitcount = vm.check_integer(args, 3)?;
    let (bitoffset, bitcount, r) = bits_check(vm, id, bitoffset, bitcount)?;
    let data = load_le(vm.heap.buffer(id).get(r).unwrap_or_default());
    let mask = (1u64 << bitcount).wrapping_sub(1);
    let result = ((data >> (bitoffset & 7)) & mask) as u32;
    vm.ret(&[Value::Number(f64::from(result))])
}

fn writebits(vm: &mut Vm, args: Args) -> VmResult<usize> {
    let id = vm.check_buffer_arg(args, 1)?;
    let bitoffset = vm.check_number(args, 2)? as i64;
    let bitcount = vm.check_integer(args, 3)?;
    let value = vm.check_unsigned(args, 4)?;
    let (bitoffset, bitcount, r) = bits_check(vm, id, bitoffset, bitcount)?;
    let Some(dst) = vm.heap.buffer_mut(id).get_mut(r) else {
        return Ok(0);
    };
    let sub = bitoffset & 7;
    let mask = (1u64 << bitcount).wrapping_sub(1) << sub;
    let data = (load_le(dst) & !mask) | ((u64::from(value) << sub) & mask);
    for (i, byte) in dst.iter_mut().enumerate() {
        *byte = (data >> (i.saturating_mul(8))) as u8;
    }
    Ok(0)
}

pub(crate) fn open(vm: &mut Vm) -> VmResult<()> {
    let functions: &[(&str, BuiltinFn)] = &[
        ("create", create),
        ("fromstring", fromstring),
        ("tostring", tostring),
        ("readi8", readi8),
        ("readu8", readu8),
        ("readi16", readi16),
        ("readu16", readu16),
        ("readi32", readi32),
        ("readu32", readu32),
        ("readf32", readf32),
        ("readf64", readf64),
        ("writei8", writei8),
        ("writeu8", writeu8),
        ("writei16", writei16),
        ("writeu16", writeu16),
        ("writei32", writei32),
        ("writeu32", writeu32),
        ("writef32", writef32),
        ("writef64", writef64),
        ("readstring", readstring),
        ("writestring", writestring),
        ("len", len),
        ("copy", copy),
        ("fill", fill),
        ("readbits", readbits),
        ("writebits", writebits),
        ("readinteger", readinteger),
        ("writeinteger", writeinteger),
    ];
    vm.open_lib("buffer", functions)?;
    Ok(())
}
