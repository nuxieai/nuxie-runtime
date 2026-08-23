//! Complete mechanical declaration translation of
//! `renderer/src/webgpu/wagyu-port/include/webgpu/webgpu_cpp_chained_struct.h`.

#![allow(non_snake_case)]

use std::ptr;

/// Source `enum class SType : uint32_t` forward declaration.
///
/// The complete enumerator authority belongs to `webgpu_cpp.h`; this owner
/// supplies only the frozen underlying representation needed by both chained
/// structures. The transparent newtype preserves that representation without
/// inventing enumerators ahead of their source owner.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct SType(pub(crate) u32);

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ChainedStruct {
    pub(crate) nextInChain: *const ChainedStruct,
    pub(crate) sType: SType,
}

impl Default for ChainedStruct {
    fn default() -> Self {
        Self {
            nextInChain: ptr::null(),
            sType: SType(0),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ChainedStructOut {
    pub(crate) nextInChain: *mut ChainedStructOut,
    pub(crate) sType: SType,
}

impl Default for ChainedStructOut {
    fn default() -> Self {
        Self {
            nextInChain: ptr::null_mut(),
            sType: SType(0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::mem::{align_of, offset_of, size_of};

    #[test]
    fn source_defaults_are_null_chain_and_zero_stype() {
        let input = ChainedStruct::default();
        assert!(input.nextInChain.is_null());
        assert_eq!(input.sType, SType(0));

        let output = ChainedStructOut::default();
        assert!(output.nextInChain.is_null());
        assert_eq!(output.sType, SType(0));
    }

    #[test]
    fn source_abi_layout_is_preserved() {
        assert_eq!(size_of::<SType>(), size_of::<u32>());
        assert_eq!(align_of::<SType>(), align_of::<u32>());

        assert_eq!(offset_of!(ChainedStruct, nextInChain), 0);
        assert_eq!(offset_of!(ChainedStruct, sType), size_of::<*const ChainedStruct>());
        assert_eq!(size_of::<ChainedStruct>(), 2 * size_of::<usize>());
        assert_eq!(align_of::<ChainedStruct>(), align_of::<usize>());

        assert_eq!(offset_of!(ChainedStructOut, nextInChain), 0);
        assert_eq!(
            offset_of!(ChainedStructOut, sType),
            size_of::<*mut ChainedStructOut>()
        );
        assert_eq!(size_of::<ChainedStructOut>(), 2 * size_of::<usize>());
        assert_eq!(align_of::<ChainedStructOut>(), align_of::<usize>());
    }

    #[test]
    fn input_and_output_chain_pointer_mutability_remains_distinct() {
        fn accepts_const(_: *const ChainedStruct) {}
        fn accepts_mut(_: *mut ChainedStructOut) {}

        accepts_const(ChainedStruct::default().nextInChain);
        accepts_mut(ChainedStructOut::default().nextInChain);
    }
}
