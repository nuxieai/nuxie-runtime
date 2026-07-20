#[allow(non_snake_case)]
#[macro_export]
macro_rules! luaC_init {
    ($L:expr, $o:expr, $tt_:expr) => {{
        let l_state = $L;
        // every GC type embeds GCheader at offset 0 (the C++ casts to GCObject*)
        let hdr = $o as *mut $crate::records::g_cheader::GCheader;
        let tt = $tt_;
        (*hdr).marked = $crate::macros::lua_c_white::luaC_white!((*l_state).global);
        (*hdr).tt = tt as u8;
        (*hdr).memcat = (*l_state).activememcat;
    }};
}

pub use luaC_init;
