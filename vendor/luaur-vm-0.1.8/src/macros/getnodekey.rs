//! Node: `cxx:Macro:Luau.VM:VM/src/lobject.h:494:getnodekey`
//! Source: `VM/src/lobject.h:494-501` (hand-checked)
//!
//! C++ copies `n_->key.tt` — the 4-bit `tt` BITFIELD only, not the packed
//! `tt|next` word (the original translation wrote `key.tt_next` into `tt`,
//! smearing the next-pointer bits into the type tag). The Rust `TKey` packs
//! the bitfields as `tt_next` with a `tt()` accessor.

#[allow(non_snake_case)]
#[macro_export]
macro_rules! getnodekey {
    ($L:expr, $obj:expr, $node:expr) => {
        unsafe {
            let i_o: *mut $crate::type_aliases::t_value::TValue = $obj;
            let n_: *const $crate::records::lua_node::LuaNode = $node;
            (*i_o).value = (*n_).key.value;
            core::ptr::copy_nonoverlapping(
                (*n_).key.extra.as_ptr(),
                (*i_o).extra.as_mut_ptr(),
                (*i_o).extra.len(),
            );
            (*i_o).tt = (*n_).key.tt();
            $crate::macros::checkliveness::checkliveness!((*$L).global, i_o);
        }
    };
}

pub use getnodekey;
