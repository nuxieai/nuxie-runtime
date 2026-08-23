//! Complete mechanical implementation translation of
//! `renderer/src/webgpu/wagyu-port/src/webgpu.c`.

#![allow(non_snake_case)]

pub(crate) const PINNED_SOURCE: &str =
    include_str!("source/renderer_src_webgpu_wagyu-port_src_webgpu.c");
pub(crate) const PINNED_SOURCE_LINE_COUNT: usize = 22;
pub(crate) const PINNED_SOURCE_BYTE_COUNT: usize = 679;
pub(crate) const WGPU_WAGYU_HEADER_VERSION_MAJOR: u32 = 1;
pub(crate) const WGPU_WAGYU_HEADER_VERSION_MINOR: u32 = 0;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) struct WGPUWagyuVersionInfo {
    pub(crate) webgpuMajor: u32,
    pub(crate) webgpuMinor: u32,
    pub(crate) wagyuExtensionLevel: u32,
}

/// Exact exported implementation. A null output pointer is accepted and is a
/// no-op; a non-null pointer receives all three pinned version fields.
#[no_mangle]
pub(crate) unsafe extern "C" fn wgpuWagyuGetCompiledVersion(
    versionInfo: *mut WGPUWagyuVersionInfo,
) {
    if let Some(versionInfo) = unsafe { versionInfo.as_mut() } {
        versionInfo.webgpuMajor = WGPU_WAGYU_HEADER_VERSION_MAJOR;
        versionInfo.webgpuMinor = WGPU_WAGYU_HEADER_VERSION_MINOR;
        versionInfo.wagyuExtensionLevel = super::webgpu_wagyu_decl::WGPU_WAGYU_EXTENSION_LEVEL;
    }
}

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];

#[cfg(test)]
mod tests {
    use super::*;
    use std::ptr;

    #[test]
    fn null_version_output_is_an_exact_noop() {
        // SAFETY: Null is an explicitly accepted source input.
        unsafe { wgpuWagyuGetCompiledVersion(ptr::null_mut()) };
    }

    #[test]
    fn compiled_version_reports_both_header_versions_and_extension_level() {
        let mut version = WGPUWagyuVersionInfo::default();
        // SAFETY: The pointer is live and uniquely borrowed for the call.
        unsafe { wgpuWagyuGetCompiledVersion(&mut version) };
        assert_eq!(
            version,
            WGPUWagyuVersionInfo {
                webgpuMajor: 1,
                webgpuMinor: 0,
                wagyuExtensionLevel: 1,
            }
        );
    }
}
