//! The `debug` library (C++ `ldblib.cpp`): `info` and `traceback`, as the
//! vendored VM registers them.
//!
//! This VM has one thread (coroutines are lane 4), so the optional thread
//! argument is never present: `getthread` always reports the running
//! thread. Lane 4 adds the thread forms.
//!
//! An oracle quirk kept: the vendored `db_traceback` reads the message as
//! UTF-8, so a message that is not valid UTF-8 becomes empty, and one with
//! an embedded NUL is cut there.
//!
//! Allocating builtins: `traceback` (the result string, checked with
//! `append` as it grows); `info` pushes only existing strings and numbers,
//! plus the short source, which it interns (charged by `Heap::intern`).

use crate::error::VmResult;
use crate::heap::Function;
use crate::libs::append;
use crate::native::{Args, BuiltinFn};
use crate::value::{FnId, Value};
use crate::vm::Vm;

/// C++ `LUA_IDSIZE`: the short-source buffer.
const IDSIZE: usize = 256;

/// What `lua_getinfo` found for one function or frame.
struct Info {
    func: FnId,
    /// The frame index, when the request named a level.
    frame: Option<usize>,
}

/// C++ `luaO_chunkid`: the printable form of a chunk name.
fn chunkid(source: &[u8]) -> Vec<u8> {
    // C reads the name through its NUL terminator.
    let end = source.iter().position(|&b| b == 0).unwrap_or(source.len());
    let source = source.get(..end).unwrap_or_default();
    let srclen = source.len();
    match source.split_first() {
        Some((b'=', rest)) => {
            if srclen <= IDSIZE {
                rest.to_vec()
            } else {
                rest.get(..IDSIZE.saturating_sub(1)).unwrap_or(rest).to_vec()
            }
        }
        Some((b'@', rest)) => {
            if srclen <= IDSIZE {
                rest.to_vec()
            } else {
                let tail = IDSIZE.saturating_sub(4);
                let mut out = b"...".to_vec();
                out.extend_from_slice(source.get(srclen.saturating_sub(tail)..).unwrap_or_default());
                out
            }
        }
        _ => {
            let line_end = source
                .iter()
                .position(|&b| b == b'\n' || b == b'\r')
                .unwrap_or(srclen);
            let len = line_end.min(IDSIZE.saturating_sub(15));
            let mut out = b"[string \"".to_vec();
            out.extend_from_slice(source.get(..len).unwrap_or_default());
            if len < srclen {
                out.extend_from_slice(b"...");
            }
            out.extend_from_slice(b"\"]");
            out
        }
    }
}

impl Vm {
    /// C++ `lua_getinfo` on a level: the frame `level` calls below the
    /// running one, if there is one.
    fn info_at_level(&self, level: i32) -> Option<Info> {
        let level = usize::try_from(level).ok()?;
        let index = self.frames.len().checked_sub(1)?.checked_sub(level)?;
        let frame = self.frames.get(index)?;
        Some(Info {
            func: frame.closure,
            frame: Some(index),
        })
    }

    /// The current line of frame `index` (C++ `currentline`), or -1 for a
    /// native frame.
    fn frame_line(&self, index: usize) -> i32 {
        let Some(frame) = self.frames.get(index) else {
            return -1;
        };
        let Some(proto) = frame.proto.as_ref() else {
            return -1;
        };
        let pc = if index.saturating_add(1) == self.frames.len() {
            self.pc
        } else {
            frame.savedpc
        };
        proto.line(pc.saturating_sub(1))
    }

    /// `ar.short_src` (option `s`).
    fn info_short_src(&self, info: &Info) -> Vec<u8> {
        match self.heap.function(info.func) {
            Function::Native(_) => b"[C]".to_vec(),
            Function::Lua(l) => {
                let proto = &l.proto;
                let source = proto.source.map(|s| self.heap.str_bytes(s)).unwrap_or_default();
                chunkid(source)
            }
        }
    }

    /// `ar.currentline` (option `l`): the running line for a frame, the
    /// definition line for a bare Lua function, -1 for natives.
    fn info_line(&self, info: &Info) -> i32 {
        if let Some(index) = info.frame {
            return self.frame_line(index);
        }
        match self.heap.function(info.func) {
            Function::Native(_) => -1,
            Function::Lua(l) => {
                let line = l.proto.linedefined;
                i32::try_from(line).unwrap_or(i32::MAX)
            }
        }
    }

    /// `ar.name` (option `n`, C++ `getfuncname`).
    fn info_name(&self, info: &Info) -> Option<Vec<u8>> {
        match self.heap.function(info.func) {
            Function::Native(n) if n.name.is_empty() => None,
            Function::Native(n) => Some(n.name.as_bytes().to_vec()),
            Function::Lua(l) => l
                .proto
                .debugname
                .map(|s| self.heap.str_bytes(s).to_vec()),
        }
    }

    /// `ar.nparams` and `ar.isvararg` (option `a`).
    fn info_arity(&self, info: &Info) -> (u8, bool) {
        match self.heap.function(info.func) {
            Function::Native(_) => (0, true),
            Function::Lua(l) => {
                let proto = &l.proto;
                (proto.numparams, proto.is_vararg)
            }
        }
    }

    fn is_native(&self, func: FnId) -> bool {
        matches!(self.heap.function(func), Function::Native(_))
    }
}

/// C++ `db_info`.
fn info(vm: &mut Vm, args: Args) -> VmResult<usize> {
    // `getthread`: no thread values exist yet, so `arg` is 0.
    let first = vm.arg_or_nil(args, 1);
    let found = if let Some(x) = vm.to_number(first) {
        // `lua_tointeger`: a cast to C `int`.
        let level = x as i32;
        if level < 0 {
            return Err(vm.arg_error(1, "level can't be negative"));
        }
        let options = vm.check_string(args, 2)?;
        (vm.info_at_level(level), options)
    } else if let Value::Function(f) = first {
        let options = vm.check_string(args, 2)?;
        (Some(Info { func: f, frame: None }), options)
    } else {
        return Err(vm.arg_error(1, "function or level expected"));
    };
    let (Some(info), options) = found else {
        return Ok(0);
    };
    let options = vm.heap.str_rc(options);
    let end = options.iter().position(|&b| b == 0).unwrap_or(options.len());
    let mut occurs = [false; 26];
    let mut results = Vec::new();
    for &ch in options.get(..end).unwrap_or_default() {
        if ch.is_ascii_lowercase()
            && let Some(seen) = occurs.get_mut(usize::from(ch.wrapping_sub(b'a')))
        {
            if *seen {
                return Err(vm.arg_error(2, "duplicate option"));
            }
            *seen = true;
        }
        match ch {
            b's' => {
                let src = vm.info_short_src(&info);
                let id = vm.heap.intern(&src)?;
                results.push(Value::String(id));
            }
            b'l' => results.push(Value::Number(f64::from(vm.info_line(&info)))),
            b'n' => {
                let name = vm.info_name(&info).unwrap_or_default();
                let id = vm.heap.intern(&name)?;
                results.push(Value::String(id));
            }
            b'f' => results.push(Value::Function(info.func)),
            b'a' => {
                let (nparams, vararg) = vm.info_arity(&info);
                results.push(Value::Number(f64::from(nparams)));
                results.push(Value::Boolean(vararg));
            }
            _ => return Err(vm.arg_error(2, "invalid option")),
        }
    }
    vm.ensure_stack(vm.top.saturating_add(results.len()))?;
    vm.ret(&results)
}

/// C++ `db_traceback` and `luaL_traceback`.
fn traceback(vm: &mut Vm, args: Args) -> VmResult<usize> {
    let msg = vm.opt_string(args, 1)?.map(|s| vm.heap.str_rc(s));
    let level = vm.opt_integer(args, 2, 1)?;
    if level < 0 {
        return Err(vm.arg_error(2, "level can't be negative"));
    }
    let mut out = Vec::new();
    if let Some(msg) = msg {
        let end = msg.iter().position(|&b| b == 0).unwrap_or(msg.len());
        let msg = msg.get(..end).unwrap_or_default();
        // The vendored `CStr::to_str().unwrap_or("")`.
        if std::str::from_utf8(msg).is_ok() {
            append(vm, &mut out, msg)?;
        }
        append(vm, &mut out, b"\n")?;
    }
    let mut level = level;
    while let Some(info) = vm.info_at_level(level) {
        level = level.saturating_add(1);
        if vm.is_native(info.func) {
            continue;
        }
        let src = vm.info_short_src(&info);
        append(vm, &mut out, &src)?;
        let line = vm.info_line(&info);
        if line > 0 {
            append(vm, &mut out, format!(":{line}").as_bytes())?;
        }
        if let Some(name) = vm.info_name(&info) {
            append(vm, &mut out, b" function ")?;
            append(vm, &mut out, &name)?;
        }
        append(vm, &mut out, b"\n")?;
    }
    vm.ret_str(&out)
}

pub(crate) fn open(vm: &mut Vm) -> VmResult<()> {
    let functions: &[(&str, BuiltinFn)] = &[("info", info), ("traceback", traceback)];
    vm.open_lib("debug", functions)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::chunkid;

    #[test]
    fn chunkid_forms() {
        assert_eq!(chunkid(b"=case"), b"case");
        assert_eq!(chunkid(b"@a/b.luau"), b"a/b.luau");
        assert_eq!(chunkid(b"local x\nprint(x)"), b"[string \"local x...\"]");
        assert_eq!(chunkid(b"x = 1"), b"[string \"x = 1\"]");
        let long = [b'@'; 300];
        assert_eq!(chunkid(&long).len(), 255);
        let long = [b'='; 300];
        assert_eq!(chunkid(&long).len(), 255);
    }
}
