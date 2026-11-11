//! Part of the string library (C++ `lstrlib.cpp`): `len`, `sub`, `upper`,
//! `lower`, `rep`, `reverse`, `byte`, `char`, `find`, `match`, `gsub`,
//! `format`, and the string metatable. Lane 5 adds the rest (`gmatch`,
//! `split`, `pack`, `unpack`, `packsize`).
//!
//! The pattern matcher is C++'s, on byte indices: recursion is bounded by
//! the same match depth (`LUAI_MAXCCALLS`, "pattern too complex"), so the
//! Rust stack it uses is bounded too. Character classes follow the vendored
//! VM, which uses Rust's ASCII predicates (`%s` does not match `\v`).

use crate::error::{Throw, VmResult};
use crate::libs::{append, check_buffer};
use crate::native::Args;
use crate::value::Value;
use crate::vm::Vm;

const MAX_CAPTURES: usize = 32;
const MATCH_DEPTH: u32 = 200;
const SPECIALS: &[u8] = b"^$*+?.([%-";

pub(crate) fn open(vm: &mut Vm) -> VmResult<()> {
    let t = vm.open_lib(
        "string",
        &[
            ("len", len),
            ("sub", sub),
            ("upper", upper),
            ("lower", lower),
            ("rep", rep),
            ("reverse", reverse),
            ("byte", byte),
            ("char", char_),
            ("find", find),
            ("match", match_),
            ("gsub", gsub),
            ("format", format),
        ],
    )?;
    // C++ `createmetatable`: strings index the library table.
    let mt = vm.new_table(0, 1)?;
    let index = vm.tm(crate::vm::Tm::Index);
    vm.raw_set(mt, Value::String(index), Value::Table(t))?;
    vm.string_meta = Some(mt);
    Ok(())
}

/// C++ `posrelat`: a negative position counts from the end.
fn posrelat(pos: i32, len: usize) -> i64 {
    let pos = i64::from(pos);
    let len = i64::try_from(len).unwrap_or(i64::MAX);
    if pos >= 0 {
        pos
    } else if pos.unsigned_abs() > len.unsigned_abs() {
        0
    } else {
        len.saturating_add(pos).saturating_add(1)
    }
}

fn check_bytes(vm: &mut Vm, args: Args, n: usize) -> VmResult<std::rc::Rc<[u8]>> {
    let id = vm.check_string(args, n)?;
    Ok(vm.heap.str_rc(id))
}

fn len(vm: &mut Vm, args: Args) -> VmResult<usize> {
    let s = check_bytes(vm, args, 1)?;
    vm.ret(&[Value::Number(s.len() as f64)])
}

fn sub(vm: &mut Vm, args: Args) -> VmResult<usize> {
    let s = check_bytes(vm, args, 1)?;
    let l = s.len();
    let start = posrelat(vm.check_integer(args, 2)?, l).max(1);
    let end = posrelat(vm.opt_integer(args, 3, -1)?, l).min(i64::try_from(l).unwrap_or(i64::MAX));
    if start <= end {
        let from = usize::try_from(start.saturating_sub(1)).unwrap_or(0);
        let to = usize::try_from(end).unwrap_or(0);
        vm.ret_str(s.get(from..to).unwrap_or_default())
    } else {
        vm.ret_str(b"")
    }
}

fn upper(vm: &mut Vm, args: Args) -> VmResult<usize> {
    let s = check_bytes(vm, args, 1)?;
    let out: Vec<u8> = s.iter().map(u8::to_ascii_uppercase).collect();
    vm.ret_str(&out)
}

fn lower(vm: &mut Vm, args: Args) -> VmResult<usize> {
    let s = check_bytes(vm, args, 1)?;
    let out: Vec<u8> = s.iter().map(u8::to_ascii_lowercase).collect();
    vm.ret_str(&out)
}

fn reverse(vm: &mut Vm, args: Args) -> VmResult<usize> {
    let s = check_bytes(vm, args, 1)?;
    let out: Vec<u8> = s.iter().rev().copied().collect();
    vm.ret_str(&out)
}

fn rep(vm: &mut Vm, args: Args) -> VmResult<usize> {
    let s = check_bytes(vm, args, 1)?;
    let n = vm.check_integer(args, 2)?;
    let Ok(n) = usize::try_from(n) else {
        return vm.ret_str(b"");
    };
    if n == 0 {
        return vm.ret_str(b"");
    }
    let total = s
        .len()
        .checked_mul(n)
        .filter(|&t| t <= vm.limits.max_string_bytes);
    let Some(total) = total else {
        return Err(vm.lib_error("resulting string too large"));
    };
    vm.heap.check_alloc(total)?;
    let mut out = Vec::with_capacity(total);
    for _ in 0..n {
        out.extend_from_slice(&s);
    }
    vm.ret_str(&out)
}

fn byte(vm: &mut Vm, args: Args) -> VmResult<usize> {
    let s = check_bytes(vm, args, 1)?;
    let l = s.len();
    let posi_arg = vm.opt_integer(args, 2, 1)?;
    let posi = posrelat(posi_arg, l);
    let pose = posrelat(
        vm.opt_integer(args, 3, i32::try_from(posi).unwrap_or(i32::MAX))?,
        l,
    );
    let posi = posi.max(1);
    let pose = pose.min(i64::try_from(l).unwrap_or(i64::MAX));
    if posi > pose {
        return Ok(0);
    }
    let from = usize::try_from(posi.saturating_sub(1)).unwrap_or(0);
    let to = usize::try_from(pose).unwrap_or(0);
    let bytes = s.get(from..to).unwrap_or_default();
    vm.ensure_stack(vm.top.saturating_add(bytes.len()))?;
    for &b in bytes {
        vm.push(Value::Number(f64::from(b)))?;
    }
    Ok(bytes.len())
}

fn char_(vm: &mut Vm, args: Args) -> VmResult<usize> {
    let mut out = Vec::with_capacity(args.nargs);
    for i in 1..=args.nargs {
        let c = vm.check_integer(args, i)?;
        match u8::try_from(c) {
            Ok(b) => out.push(b),
            Err(_) => return Err(vm.arg_error(i, "invalid value")),
        }
    }
    vm.ret_str(&out)
}

// ---- pattern matching ----------------------------------------------------

#[derive(Clone, Copy)]
enum CapLen {
    Unfinished,
    Position,
    Len(usize),
}

struct MatchState<'a> {
    src: &'a [u8],
    pat: &'a [u8],
    depth: u32,
    level: usize,
    captures: [(usize, CapLen); MAX_CAPTURES],
}

/// A pattern error, raised by the caller as `luaL_error`.
struct PatternError(String);

type MatchResult = Result<Option<usize>, PatternError>;

#[inline]
fn byte_at(s: &[u8], i: usize) -> u8 {
    s.get(i).copied().unwrap_or(0)
}

/// The vendored `match_class`.
fn match_class(c: u8, cl: u8) -> bool {
    let res = match cl.to_ascii_lowercase() {
        b'a' => c.is_ascii_alphabetic(),
        b'c' => c.is_ascii_control(),
        b'd' => c.is_ascii_digit(),
        b'g' => c.is_ascii_graphic(),
        b'l' => c.is_ascii_lowercase(),
        b'p' => c.is_ascii_punctuation(),
        b's' => c.is_ascii_whitespace(),
        b'u' => c.is_ascii_uppercase(),
        b'w' => c.is_ascii_alphanumeric(),
        b'x' => c.is_ascii_hexdigit(),
        b'z' => c == 0,
        _ => return cl == c,
    };
    if cl.is_ascii_lowercase() { res } else { !res }
}

impl MatchState<'_> {
    fn class_end(&self, mut p: usize) -> Result<usize, PatternError> {
        let c = byte_at(self.pat, p);
        p = p.saturating_add(1);
        match c {
            b'%' => {
                if p >= self.pat.len() {
                    // The vendored VM passes C++'s `%%` through unformatted.
                    return Err(PatternError(
                        "malformed pattern (ends with '%%')".to_owned(),
                    ));
                }
                Ok(p.saturating_add(1))
            }
            b'[' => {
                if byte_at(self.pat, p) == b'^' {
                    p = p.saturating_add(1);
                }
                loop {
                    if p >= self.pat.len() {
                        return Err(PatternError("malformed pattern (missing ']')".to_owned()));
                    }
                    let c = byte_at(self.pat, p);
                    p = p.saturating_add(1);
                    if c == b'%' && p < self.pat.len() {
                        p = p.saturating_add(1);
                    }
                    if byte_at(self.pat, p) == b']' {
                        break;
                    }
                }
                Ok(p.saturating_add(1))
            }
            _ => Ok(p),
        }
    }

    /// `[set]` from `p` (the `[`) to `ec` (the `]`).
    fn match_bracket_class(&self, c: u8, mut p: usize, ec: usize) -> bool {
        let mut sig = true;
        if byte_at(self.pat, p.saturating_add(1)) == b'^' {
            sig = false;
            p = p.saturating_add(1);
        }
        loop {
            p = p.saturating_add(1);
            if p >= ec {
                break;
            }
            let pc = byte_at(self.pat, p);
            if pc == b'%' {
                p = p.saturating_add(1);
                if match_class(c, byte_at(self.pat, p)) {
                    return sig;
                }
            } else if byte_at(self.pat, p.saturating_add(1)) == b'-' && p.saturating_add(2) < ec {
                p = p.saturating_add(2);
                if pc <= c && c <= byte_at(self.pat, p) {
                    return sig;
                }
            } else if pc == c {
                return sig;
            }
        }
        !sig
    }

    fn single_match(&self, s: usize, p: usize, ep: usize) -> bool {
        if s >= self.src.len() {
            return false;
        }
        let c = byte_at(self.src, s);
        match byte_at(self.pat, p) {
            b'.' => true,
            b'%' => match_class(c, byte_at(self.pat, p.saturating_add(1))),
            b'[' => self.match_bracket_class(c, p, ep.saturating_sub(1)),
            pc => pc == c,
        }
    }

    fn do_match(&mut self, s: usize, p: usize) -> MatchResult {
        if self.depth == 0 {
            return Err(PatternError("pattern too complex".to_owned()));
        }
        self.depth = self.depth.saturating_sub(1);
        let result = self.match_body(s, p);
        self.depth = self.depth.saturating_add(1);
        result
    }

    fn match_body(&mut self, mut s: usize, mut p: usize) -> MatchResult {
        loop {
            if p >= self.pat.len() {
                return Ok(Some(s));
            }
            match byte_at(self.pat, p) {
                b'(' => {
                    return if byte_at(self.pat, p.saturating_add(1)) == b')' {
                        self.start_capture(s, p.saturating_add(2), CapLen::Position)
                    } else {
                        self.start_capture(s, p.saturating_add(1), CapLen::Unfinished)
                    };
                }
                b')' => return self.end_capture(s, p.saturating_add(1)),
                b'$' if p.saturating_add(1) == self.pat.len() => {
                    return Ok(if s == self.src.len() { Some(s) } else { None });
                }
                b'%' if byte_at(self.pat, p.saturating_add(1)) == b'b' => {
                    match self.match_balance(s, p.saturating_add(2))? {
                        Some(next) => {
                            s = next;
                            p = p.saturating_add(4);
                            continue;
                        }
                        None => return Ok(None),
                    }
                }
                b'%' if byte_at(self.pat, p.saturating_add(1)) == b'f' => {
                    p = p.saturating_add(2);
                    if byte_at(self.pat, p) != b'[' {
                        return Err(PatternError("missing '[' after '%f' in pattern".to_owned()));
                    }
                    let ep = self.class_end(p)?;
                    let previous = if s == 0 {
                        0
                    } else {
                        byte_at(self.src, s.saturating_sub(1))
                    };
                    let current = byte_at(self.src, s);
                    if !self.match_bracket_class(previous, p, ep.saturating_sub(1))
                        && self.match_bracket_class(current, p, ep.saturating_sub(1))
                    {
                        p = ep;
                        continue;
                    }
                    return Ok(None);
                }
                b'%' if byte_at(self.pat, p.saturating_add(1)).is_ascii_digit() => {
                    match self.match_capture(s, byte_at(self.pat, p.saturating_add(1)))? {
                        Some(next) => {
                            s = next;
                            p = p.saturating_add(2);
                            continue;
                        }
                        None => return Ok(None),
                    }
                }
                _ => {
                    let ep = self.class_end(p)?;
                    let next = byte_at(self.pat, ep);
                    if !self.single_match(s, p, ep) {
                        if matches!(next, b'*' | b'?' | b'-') {
                            p = ep.saturating_add(1);
                            continue;
                        }
                        return Ok(None);
                    }
                    match next {
                        b'?' => {
                            if let Some(res) =
                                self.do_match(s.saturating_add(1), ep.saturating_add(1))?
                            {
                                return Ok(Some(res));
                            }
                            p = ep.saturating_add(1);
                        }
                        b'+' => return self.max_expand(s.saturating_add(1), p, ep),
                        b'*' => return self.max_expand(s, p, ep),
                        b'-' => return self.min_expand(s, p, ep),
                        _ => {
                            s = s.saturating_add(1);
                            p = ep;
                        }
                    }
                }
            }
        }
    }

    fn max_expand(&mut self, s: usize, p: usize, ep: usize) -> MatchResult {
        let mut i = 0usize;
        while self.single_match(s.saturating_add(i), p, ep) {
            i = i.saturating_add(1);
        }
        loop {
            if let Some(res) = self.do_match(s.saturating_add(i), ep.saturating_add(1))? {
                return Ok(Some(res));
            }
            match i.checked_sub(1) {
                Some(j) => i = j,
                None => return Ok(None),
            }
        }
    }

    fn min_expand(&mut self, mut s: usize, p: usize, ep: usize) -> MatchResult {
        loop {
            if let Some(res) = self.do_match(s, ep.saturating_add(1))? {
                return Ok(Some(res));
            }
            if self.single_match(s, p, ep) {
                s = s.saturating_add(1);
            } else {
                return Ok(None);
            }
        }
    }

    fn start_capture(&mut self, s: usize, p: usize, what: CapLen) -> MatchResult {
        let level = self.level;
        let Some(slot) = self.captures.get_mut(level) else {
            return Err(PatternError("too many captures".to_owned()));
        };
        *slot = (s, what);
        self.level = level.saturating_add(1);
        let res = self.do_match(s, p)?;
        if res.is_none() {
            self.level = self.level.saturating_sub(1);
        }
        Ok(res)
    }

    fn end_capture(&mut self, s: usize, p: usize) -> MatchResult {
        let open = (0..self.level)
            .rev()
            .find(|&l| matches!(self.captures.get(l), Some((_, CapLen::Unfinished))));
        let Some(l) = open else {
            return Err(PatternError("invalid pattern capture".to_owned()));
        };
        if let Some(cap) = self.captures.get_mut(l) {
            cap.1 = CapLen::Len(s.saturating_sub(cap.0));
        }
        let res = self.do_match(s, p)?;
        if res.is_none()
            && let Some(cap) = self.captures.get_mut(l)
        {
            cap.1 = CapLen::Unfinished;
        }
        Ok(res)
    }

    fn match_balance(&self, s: usize, p: usize) -> MatchResult {
        if p.saturating_add(1) >= self.pat.len() {
            return Err(PatternError(
                "malformed pattern (missing arguments to '%b')".to_owned(),
            ));
        }
        let (open, close) = (byte_at(self.pat, p), byte_at(self.pat, p.saturating_add(1)));
        if s >= self.src.len() || byte_at(self.src, s) != open {
            return Ok(None);
        }
        let mut depth = 1usize;
        let mut i = s;
        loop {
            i = i.saturating_add(1);
            if i >= self.src.len() {
                return Ok(None);
            }
            let c = byte_at(self.src, i);
            if c == close {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Ok(Some(i.saturating_add(1)));
                }
            } else if c == open {
                depth = depth.saturating_add(1);
            }
        }
    }

    fn match_capture(&self, s: usize, l: u8) -> MatchResult {
        let index = usize::from(l).checked_sub(usize::from(b'1'));
        let cap = index
            .filter(|&i| i < self.level)
            .and_then(|i| self.captures.get(i));
        let len = match cap {
            Some((_, CapLen::Len(len))) => *len,
            // C++ reads a position capture's length as `(size_t)CAP_POSITION`,
            // which no subject is long enough to match.
            Some((_, CapLen::Position)) => return Ok(None),
            _ => {
                // The vendored VM formats only the index here.
                let shown = i32::from(l)
                    .saturating_sub(i32::from(b'1'))
                    .saturating_add(1);
                return Err(PatternError(shown.to_string()));
            }
        };
        let start = cap.map_or(0, |c| c.0);
        let captured = self
            .src
            .get(start..start.saturating_add(len))
            .unwrap_or_default();
        let rest = self.src.get(s..).unwrap_or_default();
        if rest.len() >= len && rest.get(..len) == Some(captured) {
            Ok(Some(s.saturating_add(len)))
        } else {
            Ok(None)
        }
    }

    /// Capture `i` as a value (C++ `push_onecapture`).
    fn capture(&self, vm: &mut Vm, i: usize, s: usize, e: usize) -> Result<Value, CapError> {
        if i >= self.level {
            if i == 0 {
                let id = vm
                    .heap
                    .intern(self.src.get(s..e).unwrap_or_default())
                    .map_err(CapError::Throw)?;
                return Ok(Value::String(id));
            }
            return Err(CapError::Pattern("invalid capture index".to_owned()));
        }
        match self.captures.get(i) {
            Some((_, CapLen::Unfinished)) | None => {
                Err(CapError::Pattern("unfinished capture".to_owned()))
            }
            Some((init, CapLen::Position)) => Ok(Value::Number(init.saturating_add(1) as f64)),
            Some((init, CapLen::Len(len))) => {
                let bytes = self
                    .src
                    .get(*init..init.saturating_add(*len))
                    .unwrap_or_default();
                Ok(Value::String(
                    vm.heap.intern(bytes).map_err(CapError::Throw)?,
                ))
            }
        }
    }

    /// All captures (or the whole match when there are none).
    fn captures(
        &self,
        vm: &mut Vm,
        s: usize,
        e: usize,
        whole: bool,
    ) -> Result<Vec<Value>, CapError> {
        let n = if self.level == 0 && whole {
            1
        } else {
            self.level
        };
        (0..n).map(|i| self.capture(vm, i, s, e)).collect()
    }
}

enum CapError {
    Pattern(String),
    Throw(Throw),
}

fn cap_error(vm: &mut Vm, e: CapError) -> Throw {
    match e {
        CapError::Pattern(m) => vm.lib_error(m),
        CapError::Throw(t) => t,
    }
}

fn new_state<'a>(src: &'a [u8], pat: &'a [u8]) -> MatchState<'a> {
    MatchState {
        src,
        pat,
        depth: MATCH_DEPTH,
        level: 0,
        captures: [(0, CapLen::Unfinished); MAX_CAPTURES],
    }
}

/// C++ `str_find_aux`.
fn find_aux(vm: &mut Vm, args: Args, find: bool) -> VmResult<usize> {
    let s = check_bytes(vm, args, 1)?;
    let p = check_bytes(vm, args, 2)?;
    let ls = s.len();
    let init = posrelat(vm.opt_integer(args, 3, 1)?, ls);
    let init = if init < 1 {
        1
    } else if init > i64::try_from(ls).unwrap_or(i64::MAX).saturating_add(1) {
        return vm.ret(&[Value::Nil]);
    } else {
        init
    };
    let start = usize::try_from(init.saturating_sub(1)).unwrap_or(0);
    let plain =
        find && (vm.arg_or_nil(args, 4).truthy() || !p.iter().any(|c| SPECIALS.contains(c)));
    if plain {
        let hay = s.get(start..).unwrap_or_default();
        let found = if p.is_empty() {
            Some(0)
        } else {
            hay.windows(p.len()).position(|w| w == &*p)
        };
        if let Some(i) = found {
            let at = start.saturating_add(i);
            return vm.ret(&[
                Value::Number(at.saturating_add(1) as f64),
                Value::Number(at.saturating_add(p.len()) as f64),
            ]);
        }
        return vm.ret(&[Value::Nil]);
    }
    let anchor = p.first() == Some(&b'^');
    let pat = if anchor {
        p.get(1..).unwrap_or_default()
    } else {
        &p
    };
    let mut s1 = start;
    loop {
        let mut ms = new_state(&s, pat);
        let res = ms
            .do_match(s1, 0)
            .map_err(|PatternError(m)| vm.lib_error(m))?;
        if let Some(e) = res {
            let caps = ms
                .captures(vm, s1, e, !find)
                .map_err(|err| cap_error(vm, err))?;
            let mut out = Vec::with_capacity(caps.len().saturating_add(2));
            if find {
                out.push(Value::Number(s1.saturating_add(1) as f64));
                out.push(Value::Number(e as f64));
            }
            out.extend(caps);
            return vm.ret(&out);
        }
        if s1 >= s.len() || anchor {
            break;
        }
        s1 = s1.saturating_add(1);
    }
    vm.ret(&[Value::Nil])
}

fn find(vm: &mut Vm, args: Args) -> VmResult<usize> {
    find_aux(vm, args, true)
}

fn match_(vm: &mut Vm, args: Args) -> VmResult<usize> {
    find_aux(vm, args, false)
}

fn gsub(vm: &mut Vm, args: Args) -> VmResult<usize> {
    let s = check_bytes(vm, args, 1)?;
    let p = check_bytes(vm, args, 2)?;
    let repl = vm.arg_or_nil(args, 3);
    let max_s = vm.opt_integer(
        args,
        4,
        i32::try_from(s.len().saturating_add(1)).unwrap_or(i32::MAX),
    )?;
    if !matches!(
        repl,
        Value::Number(_) | Value::String(_) | Value::Function(_) | Value::Table(_)
    ) {
        return Err(vm.arg_type_error(args, 3, "string/function/table"));
    }
    let anchor = p.first() == Some(&b'^');
    let pat = if anchor {
        p.get(1..).unwrap_or_default()
    } else {
        &p
    };
    let repl_bytes = match repl {
        Value::Number(_) | Value::String(_) => vm.coerce_to_str(repl)?.map(|id| vm.heap.str_rc(id)),
        _ => None,
    };
    let mut out: Vec<u8> = Vec::new();
    let mut src = 0usize;
    let mut n: i32 = 0;
    while n < max_s {
        let mut ms = new_state(&s, pat);
        let e = ms
            .do_match(src, 0)
            .map_err(|PatternError(m)| vm.lib_error(m))?;
        if let Some(e) = e {
            n = n.saturating_add(1);
            add_value(vm, &ms, &mut out, src, e, repl, repl_bytes.as_deref())?;
        }
        match e {
            Some(e) if e > src => src = e,
            _ if src < s.len() => {
                append(vm, &mut out, &[byte_at(&s, src)])?;
                src = src.saturating_add(1);
            }
            _ => break,
        }
        if anchor {
            break;
        }
    }
    append(vm, &mut out, s.get(src..).unwrap_or_default())?;
    let id = vm.heap.intern(&out)?;
    vm.ret(&[Value::String(id), Value::Number(f64::from(n))])
}

fn add_value(
    vm: &mut Vm,
    ms: &MatchState<'_>,
    out: &mut Vec<u8>,
    s: usize,
    e: usize,
    repl: Value,
    repl_bytes: Option<&[u8]>,
) -> VmResult<()> {
    let value = match repl {
        Value::Function(_) => {
            let caps = ms
                .captures(vm, s, e, true)
                .map_err(|err| cap_error(vm, err))?;
            let at = vm.scratch();
            vm.ensure_stack(
                at.saturating_add(caps.len())
                    .saturating_add(crate::vm::MIN_STACK),
            )?;
            vm.set(at, repl);
            for (i, &c) in caps.iter().enumerate() {
                vm.set(at.saturating_add(1).saturating_add(i), c);
            }
            let saved = vm.top;
            vm.top = at.saturating_add(1).saturating_add(caps.len());
            vm.call_value(at, caps.len(), 1)?;
            let v = vm.get(at);
            vm.top = saved;
            v
        }
        Value::Table(_) => {
            let key = ms.capture(vm, 0, s, e).map_err(|err| cap_error(vm, err))?;
            vm.get_table(repl, key)?
        }
        _ => {
            let news = repl_bytes.unwrap_or_default();
            let mut i = 0usize;
            while i < news.len() {
                let c = byte_at(news, i);
                if c != b'%' {
                    append(vm, out, &[c])?;
                } else {
                    i = i.saturating_add(1);
                    let next = byte_at(news, i);
                    if !next.is_ascii_digit() {
                        if next != b'%' {
                            return Err(vm.lib_error("invalid use of '%' in replacement string"));
                        }
                        append(vm, out, &[next])?;
                    } else if next == b'0' {
                        append(vm, out, ms.src.get(s..e).unwrap_or_default())?;
                    } else {
                        let index = usize::from(next.saturating_sub(b'1'));
                        let v = ms
                            .capture(vm, index, s, e)
                            .map_err(|err| cap_error(vm, err))?;
                        if let Some(id) = vm.coerce_to_str(v)? {
                            append(vm, out, vm.heap.str_bytes(id))?;
                        }
                    }
                }
                i = i.saturating_add(1);
            }
            return Ok(());
        }
    };
    if !value.truthy() {
        append(vm, out, ms.src.get(s..e).unwrap_or_default())?;
    } else if let Some(id) = vm.coerce_to_str(value)? {
        append(vm, out, vm.heap.str_bytes(id))?;
    } else {
        let tn = value.type_name();
        return Err(vm.lib_error(format!("invalid replacement value (a {tn})")));
    }
    Ok(())
}

// ---- format ---------------------------------------------------------------

#[derive(Default)]
struct Spec {
    left: bool,
    plus: bool,
    space: bool,
    alt: bool,
    zero: bool,
    width: usize,
    precision: Option<usize>,
}

fn parse_spec(spec: &[u8]) -> Spec {
    let mut fs = Spec::default();
    let mut it = spec.iter().copied().peekable();
    while let Some(&c) = it.peek() {
        match c {
            b'-' => fs.left = true,
            b'+' => fs.plus = true,
            b' ' => fs.space = true,
            b'#' => fs.alt = true,
            b'0' => fs.zero = true,
            _ => break,
        }
        it.next();
    }
    while let Some(d) = it.peek().filter(|c| c.is_ascii_digit()).copied() {
        fs.width = fs
            .width
            .saturating_mul(10)
            .saturating_add(usize::from(d.saturating_sub(b'0')));
        it.next();
    }
    if it.peek() == Some(&b'.') {
        it.next();
        let mut p = 0usize;
        while let Some(d) = it.peek().filter(|c| c.is_ascii_digit()).copied() {
            p = p
                .saturating_mul(10)
                .saturating_add(usize::from(d.saturating_sub(b'0')));
            it.next();
        }
        fs.precision = Some(p);
    }
    fs
}

/// Pads `prefix` + `body` to the field width.
fn pad(spec: &Spec, prefix: &str, body: &[u8], zero_pad: bool) -> Vec<u8> {
    let content = prefix.len().saturating_add(body.len());
    let fill = spec.width.saturating_sub(content);
    let mut out = Vec::with_capacity(content.saturating_add(fill));
    if spec.left {
        out.extend_from_slice(prefix.as_bytes());
        out.extend_from_slice(body);
        out.resize(content.saturating_add(fill), b' ');
    } else if spec.zero && zero_pad {
        out.extend_from_slice(prefix.as_bytes());
        out.resize(prefix.len().saturating_add(fill), b'0');
        out.extend_from_slice(body);
    } else {
        out.resize(fill, b' ');
        out.extend_from_slice(prefix.as_bytes());
        out.extend_from_slice(body);
    }
    out
}

fn int_digits(mut digits: String, precision: Option<usize>, zero: bool) -> String {
    if let Some(p) = precision {
        if zero && p == 0 {
            digits.clear();
        } else if digits.len() < p {
            digits = format!("{}{digits}", "0".repeat(p.saturating_sub(digits.len())));
        }
    }
    digits
}

fn sign_prefix(spec: &Spec, negative: bool) -> &'static str {
    if negative {
        "-"
    } else if spec.plus {
        "+"
    } else if spec.space {
        " "
    } else {
        ""
    }
}

fn exponent_suffix(out: &mut String, e: char, exp: i32) {
    out.push(e);
    out.push(if exp < 0 { '-' } else { '+' });
    out.push_str(&format!("{:02}", exp.unsigned_abs()));
}

/// Rust `{:.p$e}` output split into mantissa and exponent.
fn sci_parts(a: f64, precision: usize) -> (String, i32) {
    let text = format!("{a:.precision$e}");
    match text.split_once('e') {
        Some((m, e)) => (m.to_owned(), e.parse().unwrap_or(0)),
        None => (text, 0),
    }
}

fn strip_fraction_zeros(s: &mut String) {
    if s.contains('.') {
        while s.ends_with('0') {
            s.pop();
        }
        if s.ends_with('.') {
            s.pop();
        }
    }
}

fn format_float(spec: &Spec, conv: u8, v: f64) -> Vec<u8> {
    let upper = conv.is_ascii_uppercase();
    if v.is_nan() || v.is_infinite() {
        let body = match (v.is_nan(), upper) {
            (true, false) => "nan",
            (true, true) => "NAN",
            (false, false) => "inf",
            (false, true) => "INF",
        };
        return pad(
            spec,
            sign_prefix(spec, v.is_infinite() && v < 0.0),
            body.as_bytes(),
            false,
        );
    }
    let precision = spec.precision.unwrap_or(6);
    let a = v.abs();
    let e = if upper { 'E' } else { 'e' };
    let body = match conv {
        b'f' | b'F' => {
            let mut s = format!("{a:.precision$}");
            if spec.alt && precision == 0 {
                s.push('.');
            }
            s
        }
        b'e' | b'E' => {
            let (mut m, exp) = sci_parts(a, precision);
            if spec.alt && precision == 0 {
                m.push('.');
            }
            exponent_suffix(&mut m, e, exp);
            m
        }
        _ => {
            let p = precision.max(1);
            let (mut m, exp) = sci_parts(a, p.saturating_sub(1));
            let p_i = i32::try_from(p).unwrap_or(i32::MAX);
            if exp < -4 || exp >= p_i {
                if !spec.alt {
                    strip_fraction_zeros(&mut m);
                } else if !m.contains('.') {
                    m.push('.');
                }
                exponent_suffix(&mut m, e, exp);
                m
            } else {
                let frac = usize::try_from(p_i.saturating_sub(1).saturating_sub(exp)).unwrap_or(0);
                let mut s = format!("{a:.frac$}");
                if !spec.alt {
                    strip_fraction_zeros(&mut s);
                } else if !s.contains('.') {
                    s.push('.');
                }
                s
            }
        }
    };
    pad(
        spec,
        sign_prefix(spec, v.is_sign_negative()),
        body.as_bytes(),
        true,
    )
}

fn format(vm: &mut Vm, args: Args) -> VmResult<usize> {
    let fmt = check_bytes(vm, args, 1)?;
    let top = args.nargs;
    let mut arg = 1usize;
    let mut out: Vec<u8> = Vec::new();
    let mut i = 0usize;
    while i < fmt.len() {
        // Every step below adds at most a few hundred bytes, except `%s`,
        // `%q` and `%*`, which check before they append.
        check_buffer(vm, out.len())?;
        let c = byte_at(&fmt, i);
        i = i.saturating_add(1);
        if c != b'%' {
            out.push(c);
            continue;
        }
        let next = byte_at(&fmt, i);
        if next == b'%' {
            out.push(b'%');
            i = i.saturating_add(1);
            continue;
        }
        arg = arg.saturating_add(1);
        if arg > top {
            return Err(vm.lib_error(format!("missing argument #{arg}")));
        }
        if next == b'*' {
            i = i.saturating_add(1);
            let v = vm.arg_or_nil(args, arg);
            let id = vm.tostring_value(v)?;
            append(vm, &mut out, vm.heap.str_bytes(id))?;
            continue;
        }
        // C++ `scanformat`.
        let start = i;
        while b"-+ #0".contains(&byte_at(&fmt, i)) && i < fmt.len() {
            i = i.saturating_add(1);
        }
        if i.saturating_sub(start) >= 6 {
            return Err(vm.lib_error("invalid format (repeated flags)"));
        }
        for _ in 0..2 {
            if byte_at(&fmt, i).is_ascii_digit() {
                i = i.saturating_add(1);
            }
        }
        if byte_at(&fmt, i) == b'.' {
            i = i.saturating_add(1);
            for _ in 0..2 {
                if byte_at(&fmt, i).is_ascii_digit() {
                    i = i.saturating_add(1);
                }
            }
        }
        if byte_at(&fmt, i).is_ascii_digit() {
            return Err(vm.lib_error("invalid format (width or precision too long)"));
        }
        let form = fmt.get(start..i).unwrap_or_default();
        let spec = parse_spec(form);
        let conv = byte_at(&fmt, i);
        i = i.saturating_add(1);
        match conv {
            b'c' => {
                let n = vm.check_number(args, arg)?;
                out.extend(pad(&spec, "", &[(n as i32) as u8], false));
            }
            b'd' | b'i' => {
                let n = vm.check_number(args, arg)? as i64;
                let digits = int_digits(n.unsigned_abs().to_string(), spec.precision, n == 0);
                out.extend(pad(
                    &spec,
                    sign_prefix(&spec, n < 0),
                    digits.as_bytes(),
                    spec.precision.is_none(),
                ));
            }
            b'o' | b'u' | b'x' | b'X' => {
                let n = vm.check_number(args, arg)?;
                let v = if n < 0.0 { (n as i64) as u64 } else { n as u64 };
                let mut digits = match conv {
                    b'o' => format!("{v:o}"),
                    b'x' => format!("{v:x}"),
                    b'X' => format!("{v:X}"),
                    _ => v.to_string(),
                };
                digits = int_digits(digits, spec.precision, v == 0);
                if conv == b'o' && spec.alt && !digits.starts_with('0') {
                    digits.insert(0, '0');
                }
                let prefix = match conv {
                    b'x' if spec.alt && v != 0 => "0x",
                    b'X' if spec.alt && v != 0 => "0X",
                    _ => "",
                };
                out.extend(pad(
                    &spec,
                    prefix,
                    digits.as_bytes(),
                    spec.precision.is_none(),
                ));
            }
            b'e' | b'E' | b'f' | b'g' | b'G' => {
                let n = vm.check_number(args, arg)?;
                out.extend(format_float(&spec, conv, n));
            }
            b'q' => {
                let s = check_bytes(vm, args, arg)?;
                // The exact quoted length: the escapes below, plus the quotes.
                let quoted = s
                    .iter()
                    .map(|b| match b {
                        b'"' | b'\\' | b'\n' | b'\r' => 2usize,
                        0 => 4,
                        _ => 1,
                    })
                    .fold(2usize, usize::saturating_add);
                check_buffer(vm, out.len().saturating_add(quoted))?;
                out.push(b'"');
                for &b in s.iter() {
                    match b {
                        b'"' | b'\\' | b'\n' => {
                            out.push(b'\\');
                            out.push(b);
                        }
                        b'\r' => out.extend_from_slice(b"\\r"),
                        0 => out.extend_from_slice(b"\\000"),
                        _ => out.push(b),
                    }
                }
                out.push(b'"');
            }
            b's' => {
                let s = check_bytes(vm, args, arg)?;
                if form.is_empty() || (spec.precision.is_none() && s.len() >= 100) {
                    append(vm, &mut out, &s)?;
                } else {
                    let end = s.iter().position(|&b| b == 0).unwrap_or(s.len());
                    let end = spec.precision.map_or(end, |p| p.min(end));
                    out.extend(pad(&spec, "", s.get(..end).unwrap_or_default(), false));
                }
            }
            b'*' => return Err(vm.lib_error("'%*' does not take a form")),
            other => {
                return Err(vm.lib_error(format!(
                    "invalid option '%{}' to 'format'",
                    char::from(other)
                )));
            }
        }
    }
    vm.ret_str(&out)
}
