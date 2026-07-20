#[allow(non_snake_case)]
#[macro_export]
macro_rules! LUAU_DEBUGBREAK {
    () => {
        #[cfg(target_arch = "x86")]
        unsafe {
            core::arch::asm!("int 3");
        }
        #[cfg(target_arch = "x86_64")]
        unsafe {
            core::arch::asm!("int 3");
        }
        #[cfg(target_arch = "aarch64")]
        unsafe {
            core::arch::asm!("brk #0xf000");
        }
        #[cfg(target_arch = "wasm32")]
        unsafe {
            core::arch::wasm32::unreachable();
        }
        #[cfg(not(any(
            target_arch = "x86",
            target_arch = "x86_64",
            target_arch = "aarch64",
            target_arch = "wasm32"
        )))]
        {
            panic!("LUAU_DEBUGBREAK");
        }
    };
}

pub use LUAU_DEBUGBREAK;
