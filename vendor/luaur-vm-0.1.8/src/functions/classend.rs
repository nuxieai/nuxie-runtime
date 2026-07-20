use crate::functions::lua_l_error_l::lua_l_error_l;
use crate::macros::l_esc::L_ESC;
use crate::records::match_state::MatchState;
use crate::type_aliases::match_state::MatchState as MatchStateAlias;
use core::ffi::c_char;

pub fn classend(ms: *mut MatchState, p: *const c_char) -> *const c_char {
    unsafe {
        let p = p.add(1);
        match *p.offset(-1) as u8 {
            x if x == L_ESC as u8 => {
                if p == (*ms).p_end {
                    lua_l_error_l(
                        (*ms).L,
                        c"malformed pattern (ends with '%%')".as_ptr(),
                        core::format_args!("malformed pattern (ends with '%%')"),
                    );
                }
                p.add(1)
            }
            b'[' => {
                let mut p = p;
                if *p == b'^' as c_char {
                    p = p.add(1);
                }
                loop {
                    if p == (*ms).p_end {
                        lua_l_error_l(
                            (*ms).L,
                            c"malformed pattern (missing ']')".as_ptr(),
                            core::format_args!("malformed pattern (missing ']')"),
                        );
                    }
                    p = p.add(1);
                    if *p.offset(-1) == L_ESC && p < (*ms).p_end {
                        p = p.add(1);
                    }
                    if *p == b']' as c_char {
                        break;
                    }
                }
                p.add(1)
            }
            _ => p,
        }
    }
}
