use crate::functions::classend::classend;
use crate::functions::end_capture::end_capture;
use crate::functions::match_capture::match_capture;
use crate::functions::matchbalance::matchbalance;
use crate::functions::matchbracketclass::matchbracketclass;
use crate::functions::max_expand::max_expand;
use crate::functions::min_expand::min_expand;
use crate::functions::singlematch::singlematch;
use crate::functions::start_capture::start_capture;
use crate::macros::cap_position::CAP_POSITION;
use crate::macros::cap_unfinished::CAP_UNFINISHED;
use crate::macros::l_esc::L_ESC;
use crate::macros::lua_l_error::luaL_error;
use crate::macros::uchar::uchar;
use crate::records::match_state::MatchState;
use core::ffi::{c_char, c_int};

pub(crate) unsafe fn match_item(
    ms: *mut MatchState,
    mut s: *const c_char,
    mut p: *const c_char,
) -> *const c_char {
    if (*ms).matchdepth == 0 {
        luaL_error!((*ms).L, "pattern too complex");
    }
    (*ms).matchdepth -= 1;

    let L = (*ms).L;
    if let Some(interrupt) = (*(*L).global).cb.interrupt {
        (*L).nCcalls = (*L).nCcalls.wrapping_add(1);
        interrupt(L, -1);
        (*L).nCcalls = (*L).nCcalls.wrapping_sub(1);
    }

    'init: loop {
        if p != (*ms).p_end {
            match *p as u8 {
                b'(' => {
                    if *p.add(1) == b')' as c_char {
                        s = start_capture(ms, s, p.add(2), CAP_POSITION);
                    } else {
                        s = start_capture(ms, s, p.add(1), CAP_UNFINISHED);
                    }
                    break 'init; // C++ outer-switch `break`: do not fall into dflt
                }
                b')' => {
                    s = end_capture(ms, s, p.add(1));
                    break 'init; // C++ outer-switch `break`: do not fall into dflt
                }
                b'$' => {
                    if p.add(1) != (*ms).p_end {
                        // default case below
                    } else {
                        s = if s == (*ms).src_end {
                            s
                        } else {
                            core::ptr::null()
                        };
                        break 'init;
                    }
                }
                x if x == L_ESC as u8 => match *p.add(1) as u8 {
                    b'b' => {
                        s = matchbalance(ms, s, p.add(2));
                        if !s.is_null() {
                            p = p.add(4);
                            continue 'init;
                        }
                        break 'init;
                    }
                    b'f' => {
                        p = p.add(2);
                        if *p != b'[' as c_char {
                            luaL_error!((*ms).L, "missing '[' after '%%f' in pattern");
                        }
                        let ep = classend(ms, p);
                        let previous = if s == (*ms).src_init {
                            0
                        } else {
                            *s.offset(-1)
                        };
                        if matchbracketclass(uchar(previous as c_int) as c_int, p, ep.offset(-1))
                            == 0
                            && matchbracketclass(uchar(*s as c_int) as c_int, p, ep.offset(-1)) != 0
                        {
                            p = ep;
                            continue 'init;
                        }
                        s = core::ptr::null();
                        break 'init;
                    }
                    b'0'..=b'9' => {
                        s = match_capture(ms, s, uchar(*p.add(1) as c_int) as c_int);
                        if !s.is_null() {
                            p = p.add(2);
                            continue 'init;
                        }
                        break 'init;
                    }
                    _ => {}
                },
                _ => {}
            }

            if !s.is_null() {
                let ep = classend(ms, p);
                if singlematch(ms, s, p, ep) == 0 {
                    if *ep == b'*' as c_char || *ep == b'?' as c_char || *ep == b'-' as c_char {
                        p = ep.add(1);
                        continue 'init;
                    }
                    s = core::ptr::null();
                } else {
                    match *ep as u8 {
                        b'?' => {
                            let res = match_item(ms, s.add(1), ep.add(1));
                            if !res.is_null() {
                                s = res;
                            } else {
                                p = ep.add(1);
                                continue 'init;
                            }
                        }
                        b'+' => {
                            s = s.add(1);
                            s = max_expand(ms, s, p, ep);
                        }
                        b'*' => {
                            s = max_expand(ms, s, p, ep);
                        }
                        b'-' => {
                            s = min_expand(ms, s, p, ep);
                        }
                        _ => {
                            s = s.add(1);
                            p = ep;
                            continue 'init;
                        }
                    }
                }
            }
        }

        break 'init;
    }

    (*ms).matchdepth += 1;
    s
}
