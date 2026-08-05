//! Bounds-polarity contracts for the buffer fast-call builtins.
//!
//! `checkoutofbounds` returns TRUE when the access is out of bounds, so every C
//! caller in `VM/src/lbuiltins.cpp` bails on it:
//!
//! ```c
//! if (checkoutofbounds(offset, bufvalue(arg0)->len, sizeof(int64_t)))
//!     return -1;
//! ```
//!
//! A twin that inverts that test writes exactly when the access is illegal.
//! These tests pin the polarity from both directions — a valid offset must be
//! served by the fast path, and an invalid one must fall back without touching
//! memory — so an inversion cannot be reintroduced silently.

use crate::functions::lua_close::lua_close;
use crate::functions::lua_l_newstate::lua_l_newstate;
use crate::functions::lua_newbuffer::lua_newbuffer;
use crate::functions::luau_f_bufferreadlong::luauF_bufferreadlong;
use crate::functions::luau_f_bufferwritelong::luau_f_bufferwritelong;
use crate::macros::lvalue::lvalue;
use crate::macros::setlvalue::setlvalue;
use crate::macros::setnvalue::setnvalue;
use crate::type_aliases::t_value::TValue;

const BUFFER_LEN: usize = 16;
const SENTINEL: i64 = 0x0102_0304_0506_0708;

/// `nparams` counts the buffer plus the offset and value operands.
const WRITE_NPARAMS: core::ffi::c_int = 3;
const READ_NPARAMS: core::ffi::c_int = 2;

#[test]
fn bufferwritelong_serves_in_bounds_offsets_through_the_fast_path() {
    unsafe {
        let state = lua_l_newstate();
        assert!(!state.is_null());
        let data = lua_newbuffer(state, BUFFER_LEN).cast::<u8>();
        let buffer = (*state).top.sub(1);

        // len 16, access 8: offsets 0..=8 are legal, and 8 is the last one.
        for offset in [0usize, 1, 7, 8] {
            let mut args = [TValue::default(); 2];
            setnvalue!(args.as_mut_ptr(), offset as f64);
            setlvalue!(args.as_mut_ptr().add(1), SENTINEL);

            assert_eq!(
                luau_f_bufferwritelong(
                    state,
                    core::ptr::null_mut(),
                    buffer,
                    0,
                    args.as_mut_ptr(),
                    WRITE_NPARAMS,
                ),
                0,
                "offset {offset} is in bounds and must be handled by the fast path"
            );
            assert_eq!(
                core::ptr::read_unaligned(data.add(offset).cast::<i64>()),
                SENTINEL,
                "offset {offset} must receive the written value"
            );
            core::ptr::write_bytes(data, 0, BUFFER_LEN);
        }

        lua_close(state);
    }
}

#[test]
fn bufferwritelong_falls_back_on_out_of_bounds_offsets_without_writing() {
    unsafe {
        let state = lua_l_newstate();
        assert!(!state.is_null());
        let data = lua_newbuffer(state, BUFFER_LEN).cast::<u8>();
        let buffer = (*state).top.sub(1);
        core::ptr::write_bytes(data, 0, BUFFER_LEN);

        // 9 is the first offset whose 8-byte access overruns a 16-byte buffer;
        // 16 starts past the end; negative offsets convert to huge unsigned
        // displacements, matching C's `unsigned(offset)` cast.
        for offset in [9.0f64, 15.0, 16.0, 4096.0, -1.0, -8.0] {
            let mut args = [TValue::default(); 2];
            setnvalue!(args.as_mut_ptr(), offset);
            setlvalue!(args.as_mut_ptr().add(1), SENTINEL);

            assert_eq!(
                luau_f_bufferwritelong(
                    state,
                    core::ptr::null_mut(),
                    buffer,
                    0,
                    args.as_mut_ptr(),
                    WRITE_NPARAMS,
                ),
                -1,
                "offset {offset} is out of bounds and must fall back to the ordinary call"
            );
            assert_eq!(
                core::slice::from_raw_parts(data, BUFFER_LEN),
                &[0u8; BUFFER_LEN],
                "a rejected offset must not write through the buffer"
            );
        }

        lua_close(state);
    }
}

#[test]
fn bufferwritelong_and_bufferreadlong_agree_on_the_same_offset() {
    unsafe {
        let state = lua_l_newstate();
        assert!(!state.is_null());
        lua_newbuffer(state, BUFFER_LEN);
        let buffer = (*state).top.sub(1);

        let mut write_args = [TValue::default(); 2];
        setnvalue!(write_args.as_mut_ptr(), 8.0f64);
        setlvalue!(write_args.as_mut_ptr().add(1), SENTINEL);
        assert_eq!(
            luau_f_bufferwritelong(
                state,
                core::ptr::null_mut(),
                buffer,
                0,
                write_args.as_mut_ptr(),
                WRITE_NPARAMS,
            ),
            0
        );

        // The read twin already bails on `checkoutofbounds`; pinning the pair
        // together keeps the two polarities from drifting apart again.
        let mut result = [TValue::default(); 1];
        let mut read_args = [TValue::default(); 1];
        setnvalue!(read_args.as_mut_ptr(), 8.0f64);
        assert_eq!(
            luauF_bufferreadlong(
                state,
                result.as_mut_ptr(),
                buffer,
                1,
                read_args.as_mut_ptr(),
                READ_NPARAMS,
            ),
            1
        );
        assert_eq!(lvalue!(result.as_ptr()), SENTINEL);

        let mut oob_read_args = [TValue::default(); 1];
        setnvalue!(oob_read_args.as_mut_ptr(), 9.0f64);
        assert_eq!(
            luauF_bufferreadlong(
                state,
                result.as_mut_ptr(),
                buffer,
                1,
                oob_read_args.as_mut_ptr(),
                READ_NPARAMS,
            ),
            -1,
            "the read twin must reject the same offsets the write twin rejects"
        );

        lua_close(state);
    }
}
