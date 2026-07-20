pub const keepinvariant: fn(*const crate::records::global_state::global_State) -> bool = |g| unsafe {
    let gcstate = (*g).gcstate as i32;
    gcstate == crate::macros::gc_spropagate::GCSpropagate
        || gcstate == crate::macros::gc_spropagateagain::GCSpropagateagain
        || gcstate == crate::macros::gc_satomic::GCSatomic
};
