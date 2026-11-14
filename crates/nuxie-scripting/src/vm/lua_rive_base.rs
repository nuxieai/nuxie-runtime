//! Rive's `_G.print` replacement, mirroring `lua_rive_base.cpp`.

use std::cell::Cell;
use std::rc::Rc;

use crate::luau::{Function, Lua, LuaString, MultiValue, Result};

use super::logging_scripting_context::LoggingScriptingContext;

/// Longest line `print` hands the host log, in bytes. A line is built in a
/// host buffer from copies of VM strings, so its length is not bounded by
/// the VM's memory (one 8 MiB string printed 8,000 times is 64 GiB). Past
/// this the rest of the line is dropped and the marker appended; every
/// argument is still converted, so a script sees no difference.
const MAX_PRINT_LINE_BYTES: usize = 1024 * 1024;
const TRUNCATED_MARKER: &[u8] = b" [truncated]";

/// The bytes of the shared line buffer, against the cap. The logging
/// context has one line buffer, and a `__tostring` may `print` (and fail
/// part-way, leaving its bytes there) while an outer line is being built,
/// so the count is kept for the buffer, not per call: `begin_line` and
/// `end_line` empty it, every append adds to it.
#[derive(Clone, Default)]
struct LineBytes(Rc<Cell<usize>>);

impl LineBytes {
    /// The part of `bytes` that fits under the cap, counted as appended.
    fn admit<'a>(&self, bytes: &'a [u8], truncated: &mut bool) -> &'a [u8] {
        let remaining = MAX_PRINT_LINE_BYTES.saturating_sub(self.0.get());
        let (kept, dropped) = bytes.split_at(bytes.len().min(remaining));
        self.0.set(self.0.get().saturating_add(kept.len()));
        if !dropped.is_empty() {
            *truncated = true;
        }
        kept
    }

    fn reset(&self) {
        self.0.set(0);
    }
}

/// Install the host-routed `print` before the VM globals are sandboxed.
pub(super) fn install_host_print(lua: &Lua, logging: LoggingScriptingContext) -> Result<()> {
    lua.set_app_data(logging.clone());
    let line = LineBytes::default();
    let print = lua.create_function_lent(move |lua, mut loan, args: MultiValue| {
        if args.is_empty() {
            return Ok(());
        }

        // Resolve the function from the invoking state. A handle captured from
        // the main thread cannot be called safely from an async coroutine.
        let tostring: Function = lua.globals().get("tostring")?;
        logging.begin_line();
        line.reset();
        let mut truncated = false;
        for (index, value) in args.into_iter().enumerate() {
            let value: LuaString =
                tostring.call_with(crate::luau::reborrow_loan(&mut loan), value)?;
            if truncated {
                continue;
            }
            if index > 0 {
                if line.admit(b"\t", &mut truncated).is_empty() {
                    truncated = true;
                    continue;
                }
                logging.print_separator();
            }
            let bytes = value.as_bytes();
            logging.append(line.admit(&bytes, &mut truncated));
        }
        if truncated {
            logging.append(TRUNCATED_MARKER);
        }
        logging.end_line();
        line.reset();
        Ok(())
    })?;
    lua.globals().set("print", print)
}

#[cfg(all(test, feature = "compiler"))]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    fn printed(script: &str) -> Vec<Vec<u8>> {
        let lines = Rc::new(RefCell::new(Vec::new()));
        let sink = Rc::clone(&lines);
        let vm = super::super::ScriptVm::new_with_log_sink(move |_, line| {
            sink.borrow_mut().push(line.to_vec());
        });
        vm.install_rive_globals().unwrap();
        vm.eval::<()>(script).unwrap();
        lines.take()
    }

    #[test]
    fn ordinary_prints_reach_the_log_unchanged() {
        assert_eq!(
            printed("print('a', 1, true) print() print('', '')"),
            vec![b"a\t1\ttrue".to_vec(), b"\t".to_vec()]
        );
    }

    #[test]
    fn a_print_line_is_cut_at_the_cap() {
        // Eight references to one 512 KiB string: 4 MiB of host line before.
        let lines = printed(
            "local s = string.rep('x', 512 * 1024) \
             local calls = 0 \
             local t = setmetatable({}, {__tostring = function() calls += 1 return 'm' end}) \
             print(s, s, s, t) \
             assert(calls == 1)",
        );
        let [line] = lines.as_slice() else {
            panic!("one line expected, got {}", lines.len());
        };
        assert_eq!(line.len(), MAX_PRINT_LINE_BYTES + TRUNCATED_MARKER.len());
        assert!(line.ends_with(TRUNCATED_MARKER));
        assert_eq!(line.get(512 * 1024), Some(&b'\t'));
    }

    // Codex review: a `print` from `__tostring` that fails part-way leaves
    // its bytes in the shared line, which the outer line used to ignore.
    #[test]
    fn a_failed_nested_print_counts_against_the_line() {
        let lines = printed(
            "local s = string.rep('x', 1024 * 1024) \
             local bad = setmetatable({}, {__tostring = function() error('no') end}) \
             local function nest(depth) \
               return setmetatable({}, {__tostring = function() \
                 if depth > 0 then pcall(print, s, nest(depth - 1), bad) end \
                 return 'n' \
               end}) \
             end \
             print(s, nest(4))",
        );
        let total: usize = lines.iter().map(Vec::len).sum();
        assert!(
            total <= MAX_PRINT_LINE_BYTES + TRUNCATED_MARKER.len(),
            "{total} bytes logged"
        );
    }
}
