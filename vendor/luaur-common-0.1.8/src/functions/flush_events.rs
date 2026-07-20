//! Source: `Common/src/TimeTrace.cpp:142-251` (hand-ported)
//! Appends the buffered events for one thread to the Chrome-trace `trace.json`
//! file, byte-for-byte matching the C++ formatter (Enter/Leave/ArgName/ArgValue).
use crate::enums::event_type::EventType;
use crate::functions::format_append::formatAppend;
use crate::macros::luau_assert::LUAU_ASSERT;
use crate::records::event::Event;
use crate::records::global_context::GlobalContext;
use alloc::string::String;
use core::ffi::{c_char, CStr};
use std::io::Write;

/// Reads the NUL-terminated C string at byte offset `pos` in the data buffer
/// (`rawData + ev.data.dataPos`). Returns it as a UTF-8-lossy owned string.
fn data_str(data: &[c_char], pos: u32) -> alloc::borrow::Cow<'_, str> {
    let ptr = unsafe { data.as_ptr().add(pos as usize) };
    unsafe { CStr::from_ptr(ptr) }.to_string_lossy()
}

pub fn flush_events(context: &GlobalContext, thread_id: u32, events: &[Event], data: &[c_char]) {
    // std::scoped_lock lock(context.mutex);
    let mut state = context
        .state
        .lock()
        .expect("TimeTrace GlobalContext mutex poisoned");

    // if (!context.traceFile) { context.traceFile = fopen("trace.json", "w"); ... fprintf(..., "[\n"); }
    if state.trace_file.is_none() {
        match std::fs::File::create("trace.json") {
            Ok(mut file) => {
                let _ = file.write_all(b"[\n");
                state.trace_file = Some(file);
            }
            Err(_) => return,
        }
    }

    let mut temp = String::new();
    const TEMP_RESERVE: usize = 64 * 1024;
    temp.reserve(TEMP_RESERVE);

    // Formatting state
    let mut unfinished_enter = false;
    let mut unfinished_args = false;

    for ev in events {
        match ev.r#type {
            EventType::Enter => {
                if unfinished_args {
                    formatAppend(&mut temp, format_args!("}}"));
                    unfinished_args = false;
                }
                if unfinished_enter {
                    formatAppend(&mut temp, format_args!("}},\n"));
                    unfinished_enter = false;
                }

                let token = state.tokens[ev.token as usize];
                let name = unsafe { CStr::from_ptr(token.name) }.to_string_lossy();
                let category = unsafe { CStr::from_ptr(token.category) }.to_string_lossy();
                let microsec = unsafe { ev.data.microsec };

                formatAppend(
                    &mut temp,
                    format_args!(
                        r#"{{"name": "{}", "cat": "{}", "ph": "B", "ts": {}, "pid": 0, "tid": {}"#,
                        name, category, microsec, thread_id
                    ),
                );
                unfinished_enter = true;
            }
            EventType::Leave => {
                if unfinished_args {
                    formatAppend(&mut temp, format_args!("}}"));
                    unfinished_args = false;
                }
                if unfinished_enter {
                    formatAppend(&mut temp, format_args!("}},\n"));
                    unfinished_enter = false;
                }

                let microsec = unsafe { ev.data.microsec };
                formatAppend(
                    &mut temp,
                    format_args!(
                        "{{\"ph\": \"E\", \"ts\": {}, \"pid\": 0, \"tid\": {}}},\n",
                        microsec, thread_id
                    ),
                );
            }
            EventType::ArgName => {
                LUAU_ASSERT!(unfinished_enter);

                let pos = unsafe { ev.data.dataPos };
                let arg = data_str(data, pos);
                if !unfinished_args {
                    formatAppend(&mut temp, format_args!(r#", "args": {{ "{}": "#, arg));
                    unfinished_args = true;
                } else {
                    formatAppend(&mut temp, format_args!(r#", "{}": "#, arg));
                }
            }
            EventType::ArgValue => {
                LUAU_ASSERT!(unfinished_args);
                let pos = unsafe { ev.data.dataPos };
                let value = data_str(data, pos);
                formatAppend(&mut temp, format_args!(r#""{}""#, value));
            }
        }

        // Don't want to hit the string capacity and reallocate
        if temp.len() > TEMP_RESERVE - 1024 {
            if let Some(file) = state.trace_file.as_mut() {
                let _ = file.write_all(temp.as_bytes());
            }
            temp.clear();
        }
    }

    if unfinished_args {
        formatAppend(&mut temp, format_args!("}}"));
    }
    if unfinished_enter {
        formatAppend(&mut temp, format_args!("}},\n"));
    }

    if let Some(file) = state.trace_file.as_mut() {
        let _ = file.write_all(temp.as_bytes());
        let _ = file.flush();
    }
}
