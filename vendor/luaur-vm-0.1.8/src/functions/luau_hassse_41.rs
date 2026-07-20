#[allow(non_snake_case)]
pub(crate) fn luau_hassse41() -> bool {
    #[cfg(any(target_arch = "x86", target_arch = "x86_64"))]
    {
        #[cfg(target_arch = "x86")]
        use core::arch::x86::__cpuid;
        #[cfg(target_arch = "x86_64")]
        use core::arch::x86_64::__cpuid;

        unsafe {
            let result = __cpuid(1);
            // We require SSE4.1 support for ROUNDSD
            // https://en.wikipedia.org/wiki/CPUID#EAX=1:_Processor_Info_and_Feature_Bits
            // SSE4.1 is bit 19 of ECX
            (result.ecx & (1 << 19)) != 0
        }
    }
    #[cfg(not(any(target_arch = "x86", target_arch = "x86_64")))]
    {
        false
    }
}
