//! `string.pack`, `string.packsize` and `string.unpack` (C++ `lstrlib.cpp`
//! `str_pack` and friends, as the vendored VM ports them).
//!
//! An oracle quirk kept: an invalid format byte at or above 0x80 is named
//! in the error as the Latin-1 character of that byte (UTF-8 encoded), as
//! the vendored `opt as u8 as char` formats it.
//!
//! Allocating builtins: `pack` (the result, checked with `check_buffer`
//! before every growth, padding included) and `unpack` (each `c`, `s` and
//! `z` field is a new string, charged by `Heap::intern`; the results are
//! bounded by the 8000-slot `luaL_checkstack` limit).

use crate::error::VmResult;
use crate::libs::check_buffer;
use crate::native::{Args, BuiltinFn};
use crate::value::Value;
use crate::vm::Vm;

/// C++ `MAXINTSIZE`.
const MAXINTSIZE: i32 = 16;
/// C++ `MAXALIGN`.
const MAXALIGN: i32 = 8;
/// C++ `MAXSSIZE`.
const MAXSSIZE: usize = 1 << 30;
/// `sizeof(long long)`.
const SZINT: i32 = 8;
/// C++ `LUAI_MAXCSTACK`.
const MAX_C_STACK: usize = 8000;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum KOption {
    Int,
    Uint,
    Float,
    Char,
    String,
    Zstr,
    Padding,
    PaddAlign,
    Nop,
}

/// C++ `Header`, plus the format cursor.
struct Format<'a> {
    fmt: &'a [u8],
    pos: usize,
    little: bool,
    maxalign: i32,
}

impl Format<'_> {
    fn peek(&self) -> u8 {
        self.fmt.get(self.pos).copied().unwrap_or(0)
    }

    fn at_end(&self) -> bool {
        self.peek() == 0
    }

    /// C++ `getnum`.
    fn getnum(&mut self, vm: &mut Vm, default: i32) -> VmResult<i32> {
        if !self.peek().is_ascii_digit() {
            return Ok(default);
        }
        let mut a: i32 = 0;
        loop {
            let d = i32::from(self.peek().wrapping_sub(b'0'));
            a = a.saturating_mul(10).saturating_add(d);
            self.pos = self.pos.saturating_add(1);
            if !self.peek().is_ascii_digit() || a > (i32::MAX - 9) / 10 {
                break;
            }
        }
        if a > 1_073_741_824 || self.peek().is_ascii_digit() {
            return Err(vm.lib_error("size specifier is too large"));
        }
        Ok(a)
    }

    /// C++ `getnumlimit`.
    fn getnumlimit(&mut self, vm: &mut Vm, default: i32) -> VmResult<i32> {
        let sz = self.getnum(vm, default)?;
        if sz > MAXINTSIZE || sz <= 0 {
            return Err(vm.lib_error(format!(
                "integral size ({sz}) out of limits [1,{MAXINTSIZE}]"
            )));
        }
        Ok(sz)
    }

    /// C++ `getoption`: the next option and its size.
    fn getoption(&mut self, vm: &mut Vm) -> VmResult<(KOption, i32)> {
        let opt = self.peek();
        self.pos = self.pos.saturating_add(1);
        Ok(match opt {
            b'b' => (KOption::Int, 1),
            b'B' => (KOption::Uint, 1),
            b'h' => (KOption::Int, 2),
            b'H' => (KOption::Uint, 2),
            b'l' => (KOption::Int, 8),
            b'L' => (KOption::Uint, 8),
            b'j' => (KOption::Int, 4),
            b'J' | b'T' => (KOption::Uint, 4),
            b'f' => (KOption::Float, 4),
            b'd' | b'n' => (KOption::Float, 8),
            b'i' => (KOption::Int, self.getnumlimit(vm, 4)?),
            b'I' => (KOption::Uint, self.getnumlimit(vm, 4)?),
            b's' => (KOption::String, self.getnumlimit(vm, 4)?),
            b'c' => {
                let size = self.getnum(vm, -1)?;
                if size == -1 {
                    return Err(vm.lib_error("missing size for format option 'c'"));
                }
                (KOption::Char, size)
            }
            b'z' => (KOption::Zstr, 0),
            b'x' => (KOption::Padding, 1),
            b'X' => (KOption::PaddAlign, 0),
            b' ' => (KOption::Nop, 0),
            b'<' => {
                self.little = true;
                (KOption::Nop, 0)
            }
            b'>' => {
                self.little = false;
                (KOption::Nop, 0)
            }
            b'=' => {
                self.little = cfg!(target_endian = "little");
                (KOption::Nop, 0)
            }
            b'!' => {
                self.maxalign = self.getnumlimit(vm, MAXALIGN)?;
                (KOption::Nop, 0)
            }
            _ => {
                let c = char::from(opt);
                return Err(vm.lib_error(format!("invalid format option '{c}'")));
            }
        })
    }

    /// C++ `getdetails`: the next option, its size, and the padding that
    /// aligns it at `totalsize`.
    fn getdetails(&mut self, vm: &mut Vm, totalsize: usize) -> VmResult<(KOption, i32, i32)> {
        let (opt, size) = self.getoption(vm)?;
        let mut align = size;
        if opt == KOption::PaddAlign {
            let bad = if self.at_end() {
                true
            } else {
                let (next, next_size) = self.getoption(vm)?;
                align = next_size;
                next == KOption::Char || align == 0
            };
            if bad {
                return Err(vm.arg_error(1, "invalid next option for option 'X'"));
            }
        }
        let ntoalign = if align <= 1 || opt == KOption::Char {
            0
        } else {
            let align = align.min(self.maxalign);
            if align & align.wrapping_sub(1) != 0 {
                return Err(vm.arg_error(1, "format asks for alignment not power of 2"));
            }
            // C `(int)totalsize & (align - 1)`.
            let mask = align.wrapping_sub(1);
            align.wrapping_sub(totalsize as i32 & mask) & mask
        };
        Ok((opt, size, ntoalign))
    }
}

/// The format argument: a C string, read through its NUL terminator.
fn format_arg(vm: &mut Vm, args: Args) -> VmResult<std::rc::Rc<[u8]>> {
    let s = vm.check_string(args, 1)?;
    Ok(vm.heap.str_rc(s))
}

fn new_format(fmt: &[u8]) -> Format<'_> {
    Format {
        fmt,
        pos: 0,
        little: true,
        maxalign: 1,
    }
}

/// Grows `out` by `bytes` after checking the result limit.
fn put(vm: &Vm, out: &mut Vec<u8>, bytes: &[u8]) -> VmResult<()> {
    check_buffer(vm, out.len().saturating_add(bytes.len()))?;
    out.extend_from_slice(bytes);
    Ok(())
}

/// Appends `n` padding bytes after checking the result limit.
fn pad(vm: &Vm, out: &mut Vec<u8>, n: usize) -> VmResult<()> {
    let len = out.len().saturating_add(n);
    check_buffer(vm, len)?;
    out.resize(len, 0);
    Ok(())
}

/// C++ `packint`: `size` bytes of `n`, sign-extended past 8 bytes.
fn packint(out: &mut [u8; 16], n: u64, little: bool, size: usize, neg: bool) {
    let mut n = n;
    for i in 0..size {
        let idx = if little { i } else { size.saturating_sub(1).saturating_sub(i) };
        let byte = if neg && i >= SZINT as usize { 0xFF } else { (n & 0xFF) as u8 };
        if let Some(slot) = out.get_mut(idx) {
            *slot = byte;
        }
        n >>= 8;
    }
}

/// C++ `str_pack` pushes a nil above its arguments before reading them, so
/// the first argument past the end is a nil, not a missing one, in errors.
fn pack_arg(vm: &mut Vm, args: Args, arg: usize, expected: &str) -> Option<crate::error::Throw> {
    (arg == args.nargs.saturating_add(1))
        .then(|| vm.arg_error(arg, &format!("{expected} expected, got nil")))
}

fn pack_number(vm: &mut Vm, args: Args, arg: usize) -> VmResult<f64> {
    match pack_arg(vm, args, arg, "number") {
        Some(e) => Err(e),
        None => vm.check_number(args, arg),
    }
}

fn pack_string(vm: &mut Vm, args: Args, arg: usize) -> VmResult<std::rc::Rc<[u8]>> {
    if let Some(e) = pack_arg(vm, args, arg, "string") {
        return Err(e);
    }
    let s = vm.check_string(args, arg)?;
    Ok(vm.heap.str_rc(s))
}

fn pack(vm: &mut Vm, args: Args) -> VmResult<usize> {
    let fmt = format_arg(vm, args)?;
    let end = fmt.iter().position(|&b| b == 0).unwrap_or(fmt.len());
    let mut f = new_format(fmt.get(..end).unwrap_or_default());
    let mut out = Vec::new();
    let mut arg = 1usize;
    let mut totalsize = 0usize;
    while !f.at_end() {
        let (opt, size, ntoalign) = f.getdetails(vm, totalsize)?;
        let usize_of = |x: i32| usize::try_from(x).unwrap_or(0);
        totalsize = totalsize.saturating_add(usize_of(ntoalign.saturating_add(size)));
        pad(vm, &mut out, usize_of(ntoalign))?;
        arg = arg.saturating_add(1);
        let little = f.little;
        let sz = usize_of(size);
        match opt {
            KOption::Int => {
                let n = pack_number(vm, args, arg)? as i64;
                if size < SZINT {
                    let lim = 1i64 << (size.saturating_mul(8).saturating_sub(1));
                    if !(lim.wrapping_neg() <= n && n < lim) {
                        return Err(vm.arg_error(arg, "integer overflow"));
                    }
                }
                let mut buf = [0u8; 16];
                packint(&mut buf, n as u64, little, sz, n < 0);
                put(vm, &mut out, buf.get(..sz).unwrap_or_default())?;
            }
            KOption::Uint => {
                let n = pack_number(vm, args, arg)? as i64;
                if size < SZINT && (n as u64) >= 1u64 << size.saturating_mul(8) {
                    return Err(vm.arg_error(arg, "unsigned overflow"));
                }
                let mut buf = [0u8; 16];
                packint(&mut buf, n as u64, little, sz, false);
                put(vm, &mut out, buf.get(..sz).unwrap_or_default())?;
            }
            KOption::Float => {
                let n = pack_number(vm, args, arg)?;
                let mut bytes = if sz == 4 {
                    (n as f32).to_le_bytes().to_vec()
                } else {
                    n.to_le_bytes().to_vec()
                };
                if !little {
                    bytes.reverse();
                }
                put(vm, &mut out, &bytes)?;
            }
            KOption::Char => {
                let s = pack_string(vm, args, arg)?;
                if s.len() > sz {
                    return Err(vm.arg_error(arg, "string longer than given size"));
                }
                put(vm, &mut out, &s)?;
                pad(vm, &mut out, sz.saturating_sub(s.len()))?;
            }
            KOption::String => {
                let s = pack_string(vm, args, arg)?;
                let len = s.len();
                let fits = size >= 8 || (len as u64) < 1u64 << size.saturating_mul(8);
                if !fits {
                    return Err(vm.arg_error(arg, "string length does not fit in given size"));
                }
                let mut buf = [0u8; 16];
                packint(&mut buf, len as u64, little, sz, false);
                put(vm, &mut out, buf.get(..sz).unwrap_or_default())?;
                put(vm, &mut out, &s)?;
                totalsize = totalsize.saturating_add(len);
            }
            KOption::Zstr => {
                let s = pack_string(vm, args, arg)?;
                if s.contains(&0) {
                    return Err(vm.arg_error(arg, "string contains zeros"));
                }
                put(vm, &mut out, &s)?;
                put(vm, &mut out, &[0])?;
                totalsize = totalsize.saturating_add(s.len()).saturating_add(1);
            }
            KOption::Padding => {
                pad(vm, &mut out, 1)?;
                arg = arg.saturating_sub(1);
            }
            KOption::PaddAlign | KOption::Nop => {
                arg = arg.saturating_sub(1);
            }
        }
    }
    vm.ret_str(&out)
}

fn packsize(vm: &mut Vm, args: Args) -> VmResult<usize> {
    let fmt = format_arg(vm, args)?;
    let end = fmt.iter().position(|&b| b == 0).unwrap_or(fmt.len());
    let mut f = new_format(fmt.get(..end).unwrap_or_default());
    let mut totalsize = 0usize;
    while !f.at_end() {
        let (opt, size, ntoalign) = f.getdetails(vm, totalsize)?;
        if matches!(opt, KOption::String | KOption::Zstr) {
            return Err(vm.arg_error(1, "variable-length format"));
        }
        let option_size = usize::try_from(size.saturating_add(ntoalign)).unwrap_or(0);
        if totalsize > MAXSSIZE.saturating_sub(option_size) {
            return Err(vm.arg_error(1, "format result too large"));
        }
        totalsize = totalsize.saturating_add(option_size);
    }
    // `lua_pushinteger((int)totalsize)`.
    vm.ret(&[Value::Number(f64::from(totalsize as i32))])
}

/// C++ `unpackint`: reads `size` bytes at `at`; the bytes past 8 must be
/// the sign extension.
fn unpackint(
    vm: &mut Vm,
    data: &[u8],
    at: usize,
    little: bool,
    size: usize,
    signed: bool,
) -> VmResult<u64> {
    let byte = |i: usize| -> u8 {
        let idx = if little { i } else { size.saturating_sub(1).saturating_sub(i) };
        data.get(at.saturating_add(idx)).copied().unwrap_or(0)
    };
    let limit = size.min(SZINT as usize);
    let mut res: u64 = 0;
    for i in (0..limit).rev() {
        res = (res << 8) | u64::from(byte(i));
    }
    if size < SZINT as usize {
        if signed && size > 0 {
            let mask = 1u64 << (size.saturating_mul(8).saturating_sub(1));
            res = (res ^ mask).wrapping_sub(mask);
        }
    } else if size > SZINT as usize {
        let mask = if !signed || (res as i64) >= 0 { 0 } else { 0xFF };
        for i in limit..size {
            if byte(i) != mask {
                return Err(vm.lib_error(format!(
                    "{size}-byte integer does not fit into Lua Integer"
                )));
            }
        }
    }
    Ok(res)
}

fn unpack(vm: &mut Vm, args: Args) -> VmResult<usize> {
    let fmt = format_arg(vm, args)?;
    let data = vm.check_string(args, 2)?;
    let data = vm.heap.str_rc(data);
    let ld = data.len();
    let init = vm.opt_integer(args, 3, 1)?;
    // The vendored `posrelat`, then `- 1`, clamped at 0.
    let ld_i = i64::try_from(ld).unwrap_or(i64::MAX);
    let rel = if init < 0 {
        i64::from(init).saturating_add(ld_i).saturating_add(1).max(0)
    } else {
        i64::from(init)
    };
    let mut pos = usize::try_from(rel.saturating_sub(1)).unwrap_or(0);
    if pos > ld {
        return Err(vm.arg_error(3, "initial position out of string"));
    }
    let end = fmt.iter().position(|&b| b == 0).unwrap_or(fmt.len());
    let mut f = new_format(fmt.get(..end).unwrap_or_default());
    let mut results: Vec<Value> = Vec::new();
    while !f.at_end() {
        let (opt, size, ntoalign) = f.getdetails(vm, pos)?;
        let sz = usize::try_from(size).unwrap_or(0);
        let ntoalign = usize::try_from(ntoalign).unwrap_or(0);
        if ntoalign.saturating_add(sz) > ld.saturating_sub(pos) {
            return Err(vm.arg_error(2, "data string too short"));
        }
        pos = pos.saturating_add(ntoalign);
        // `luaL_checkstack(L, 2, "too many results")`.
        if args.nargs.saturating_add(results.len()).saturating_add(2) > MAX_C_STACK {
            return Err(vm.lib_error("stack overflow (too many results)"));
        }
        let little = f.little;
        match opt {
            KOption::Int => {
                let res = unpackint(vm, &data, pos, little, sz, true)?;
                results.push(Value::Number(res as i64 as f64));
            }
            KOption::Uint => {
                let res = unpackint(vm, &data, pos, little, sz, false)?;
                results.push(Value::Number(res as f64));
            }
            KOption::Float => {
                let mut bytes = [0u8; 8];
                for (i, slot) in bytes.iter_mut().take(sz).enumerate() {
                    *slot = data.get(pos.saturating_add(i)).copied().unwrap_or(0);
                }
                if !little && let Some(field) = bytes.get_mut(..sz) {
                    field.reverse();
                }
                let n = if sz == 4 {
                    let [a, b, c, d, ..] = bytes;
                    f64::from(f32::from_le_bytes([a, b, c, d]))
                } else {
                    f64::from_le_bytes(bytes)
                };
                results.push(Value::Number(n));
            }
            KOption::Char => {
                let s = data.get(pos..pos.saturating_add(sz)).unwrap_or_default();
                let id = vm.heap.intern(s)?;
                results.push(Value::String(id));
            }
            KOption::String => {
                let len = unpackint(vm, &data, pos, little, sz, false)? as usize;
                if len > ld.saturating_sub(pos).saturating_sub(sz) {
                    return Err(vm.arg_error(2, "data string too short"));
                }
                let start = pos.saturating_add(sz);
                let s = data.get(start..start.saturating_add(len)).unwrap_or_default();
                let id = vm.heap.intern(s)?;
                results.push(Value::String(id));
                pos = pos.saturating_add(len);
            }
            KOption::Zstr => {
                let rest = data.get(pos..).unwrap_or_default();
                let len = rest.iter().position(|&b| b == 0).unwrap_or(rest.len());
                if pos.saturating_add(len) >= ld {
                    return Err(vm.arg_error(2, "unfinished string for format 'z'"));
                }
                let id = vm.heap.intern(rest.get(..len).unwrap_or_default())?;
                results.push(Value::String(id));
                pos = pos.saturating_add(len).saturating_add(1);
            }
            KOption::PaddAlign | KOption::Padding | KOption::Nop => {}
        }
        pos = pos.saturating_add(sz);
    }
    // `lua_pushinteger(pos + 1)` through C `int`.
    results.push(Value::Number(f64::from((pos as i32).wrapping_add(1))));
    vm.ensure_stack(vm.top.saturating_add(results.len()))?;
    vm.ret(&results)
}

/// Adds the pack functions to the `string` library table.
pub(crate) fn open(vm: &mut Vm) -> VmResult<()> {
    let key = vm.heap.intern(b"string")?;
    let globals = vm.globals;
    let Value::Table(t) = vm.heap.table(globals).get(Value::String(key)) else {
        return Ok(());
    };
    let functions: &[(&str, BuiltinFn)] =
        &[("pack", pack), ("packsize", packsize), ("unpack", unpack)];
    vm.register_builtins(t, functions)
}
