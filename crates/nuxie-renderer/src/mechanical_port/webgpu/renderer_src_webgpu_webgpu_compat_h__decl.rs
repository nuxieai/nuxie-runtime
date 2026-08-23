//! Complete mechanical declaration translation of
//! `renderer/src/webgpu/webgpu_compat.h`.

#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

use std::ffi::{c_char, CStr};
use std::ptr;

pub(crate) const PINNED_SOURCE: &str =
    include_str!("source/renderer_src_webgpu_webgpu_compat.h");
pub(crate) const PINNED_SOURCE_LINE_COUNT: usize = 640;
pub(crate) const PINNED_SOURCE_BYTE_COUNT: usize = 43_972;

pub(crate) const WGPU_DEPTH_CLEAR_VALUE_UNDEFINED: f64 = f64::NAN;
pub(crate) type WGPUOptionalBool = i32;
pub(crate) const WGPUOptionalBool_False: WGPUOptionalBool = 0x0000_0000;
pub(crate) const WGPUOptionalBool_True: WGPUOptionalBool = 0x0000_0001;
pub(crate) const WGPUOptionalBool_Undefined: WGPUOptionalBool = 0x0000_0002;
pub(crate) const WGPUOptionalBool_Force32: WGPUOptionalBool = 0x7fff_ffff;
pub(crate) type WGPUStatus = i32;
pub(crate) const WGPUStatus_Success: WGPUStatus = 1;
pub(crate) const WGPU_RENDER_PASS_MAX_DRAW_COUNT_DEFAULT: u64 = 50_000_000;

pub(crate) type WGPUStringView = *const c_char;
pub(crate) const WGPU_STRING_VIEW_INIT: WGPUStringView = ptr::null();

pub(crate) const fn WGPU_STRING_VIEW(value: WGPUStringView) -> WGPUStringView {
    value
}

pub(crate) const fn WGPU_STRING_VIEW_TO_CSTR(value: WGPUStringView) -> WGPUStringView {
    value
}

/// Source `std::string(s)` conversion. As in the source, `value` must point to
/// a live NUL-terminated string.
pub(crate) unsafe fn WGPU_STRING_VIEW_TO_STRING(value: WGPUStringView) -> Vec<u8> {
    // SAFETY: The caller supplies the same validity contract as the source C
    // string macro.
    unsafe { CStr::from_ptr(value) }.to_bytes().to_vec()
}

/// Source `strdup(s.c_str())`; ownership transfers to
/// `WGPU_STRING_VIEW_FREE` exactly once.
pub(crate) fn WGPU_STRING_VIEW_FROM_STRING(value: &CStr) -> WGPUStringView {
    // SAFETY: `value` is NUL terminated and `strdup` returns independent C
    // allocation or null on allocation failure, exactly as the source macro.
    unsafe { libc::strdup(value.as_ptr()) }
}

/// Source `free(const_cast<char*>(s))`.
pub(crate) unsafe fn WGPU_STRING_VIEW_FREE(value: WGPUStringView) {
    // SAFETY: The caller transfers a pointer returned by the matching source
    // duplication operation, or null.
    unsafe { libc::free(value.cast_mut().cast()) };
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct WagyuStringViewParts {
    pub(crate) data: WGPUStringView,
    pub(crate) length: usize,
}

/// Both the C and C++ `WGPU_WAGYU_STRING_VIEW` branches produce these exact
/// pointer/length fields; only their aggregate-construction syntax differs.
pub(crate) unsafe fn WGPU_WAGYU_STRING_VIEW(value: WGPUStringView) -> WagyuStringViewParts {
    WagyuStringViewParts {
        data: value,
        length: if value.is_null() {
            0
        } else {
            // SAFETY: The non-null input carries the source `strlen` contract.
            unsafe { libc::strlen(value) }
        },
    }
}

pub(crate) trait WgpuReference {
    fn reference(&self);
}

pub(crate) fn WGPU_ADDREF<T: WgpuReference>(object: &T) {
    object.reference();
}

pub(crate) const fn WGPU_CHECK_STATUS<T>(status: T) -> T {
    status
}

pub(crate) const fn wgpu_bool(value: bool) -> WGPUOptionalBool {
    if value {
        WGPUOptionalBool_True
    } else {
        WGPUOptionalBool_False
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct CompatAlias {
    pub(crate) compatibility_name: &'static str,
    pub(crate) pinned_name: &'static str,
    pub(crate) source_lines: &'static str,
}

pub(crate) const COMPAT_ALIASES: &[CompatAlias] = &[
    CompatAlias { compatibility_name: "WGPUTexelCopyBufferInfo", pinned_name: "WGPUImageCopyBuffer", source_lines: "6" },
    CompatAlias { compatibility_name: "WGPUTexelCopyTextureInfo", pinned_name: "WGPUImageCopyTexture", source_lines: "7" },
    CompatAlias { compatibility_name: "WGPUTexelCopyBufferLayout", pinned_name: "WGPUTextureDataLayout", source_lines: "8" },
    CompatAlias { compatibility_name: "WGPUInstanceCapabilities", pinned_name: "WGPUInstanceFeatures", source_lines: "9" },
    CompatAlias { compatibility_name: "WGPUSurfaceGetCurrentTextureStatus_SuccessOptimal", pinned_name: "WGPUSurfaceGetCurrentTextureStatus_Success", source_lines: "19-20" },
    CompatAlias { compatibility_name: "WGPUMapAsyncStatus_Success", pinned_name: "WGPUBufferMapAsyncStatus_Success", source_lines: "22" },
    CompatAlias { compatibility_name: "WGPUMapAsyncStatus_Force32", pinned_name: "WGPUBufferMapAsyncStatus_Force32", source_lines: "23" },
    CompatAlias { compatibility_name: "WGPUSType_ShaderSourceWGSL", pinned_name: "WGPUSType_ShaderModuleWGSLDescriptor", source_lines: "25,44" },
    CompatAlias { compatibility_name: "wgpuGetInstanceCapabilities", pinned_name: "wgpuGetInstanceFeatures", source_lines: "30" },
    CompatAlias { compatibility_name: "WGPUMapAsyncStatus", pinned_name: "WGPUBufferMapAsyncStatus", source_lines: "32" },
    CompatAlias { compatibility_name: "WGPUShaderSourceWGSL", pinned_name: "WGPUShaderModuleWGSLDescriptor", source_lines: "33" },
    CompatAlias { compatibility_name: "WGPURenderPassMaxDrawCount", pinned_name: "WGPURenderPassDescriptorMaxDrawCount", source_lines: "34" },
    CompatAlias { compatibility_name: "WGPUWGSLLanguageFeatureName", pinned_name: "WGPUWGSLFeatureName", source_lines: "35" },
    CompatAlias { compatibility_name: "WGPUEmscriptenSurfaceSourceCanvasHTMLSelector", pinned_name: "WGPUSurfaceDescriptorFromCanvasHTMLSelector", source_lines: "37-38" },
    CompatAlias { compatibility_name: "WGPUWGSLLanguageFeatureName_Force32", pinned_name: "WGPUWGSLFeatureName_Force32", source_lines: "39" },
    CompatAlias { compatibility_name: "WGPUSType_EmscriptenSurfaceSourceCanvasHTMLSelector", pinned_name: "WGPUSType_SurfaceDescriptorFromCanvasHTMLSelector", source_lines: "40-41" },
    CompatAlias { compatibility_name: "WGPUSType_RenderPassMaxDrawCount", pinned_name: "WGPUSType_RenderPassDescriptorMaxDrawCount", source_lines: "42-43" },
    CompatAlias { compatibility_name: "WGPUWGSLLanguageFeatureName_Packed4x8IntegerDotProduct", pinned_name: "WGPUWGSLFeatureName_Packed4x8IntegerDotProduct", source_lines: "45-46" },
    CompatAlias { compatibility_name: "WGPUWGSLLanguageFeatureName_PointerCompositeAccess", pinned_name: "WGPUWGSLFeatureName_PointerCompositeAccess", source_lines: "47-48" },
    CompatAlias { compatibility_name: "WGPUWGSLLanguageFeatureName_ReadonlyAndReadwriteStorageTextures", pinned_name: "WGPUWGSLFeatureName_ReadonlyAndReadwriteStorageTextures", source_lines: "49-50" },
    CompatAlias { compatibility_name: "WGPUWGSLLanguageFeatureName_UnrestrictedPointerParameters", pinned_name: "WGPUWGSLFeatureName_UnrestrictedPointerParameters", source_lines: "52-53" },
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct MacroAuthority {
    pub(crate) name: &'static str,
    pub(crate) source_lines: &'static str,
}

/// Exact 114-definition preprocessor denominator. Repeated names preserve the
/// duplicate alias and the language-configuration branches instead of being
/// collapsed into one candidate-derived definition.
pub(crate) const MACRO_AUTHORITIES: &[MacroAuthority] = &[
    MacroAuthority { name: "WEBGPU_COMPAT_H", source_lines: "2" },
    MacroAuthority { name: "WGPU_DEPTH_CLEAR_VALUE_UNDEFINED", source_lines: "4" },
    MacroAuthority { name: "WGPUTexelCopyBufferInfo", source_lines: "6" },
    MacroAuthority { name: "WGPUTexelCopyTextureInfo", source_lines: "7" },
    MacroAuthority { name: "WGPUTexelCopyBufferLayout", source_lines: "8" },
    MacroAuthority { name: "WGPUInstanceCapabilities", source_lines: "9" },
    MacroAuthority { name: "WGPUOptionalBool", source_lines: "12" },
    MacroAuthority { name: "WGPUOptionalBool_False", source_lines: "13" },
    MacroAuthority { name: "WGPUOptionalBool_True", source_lines: "14" },
    MacroAuthority { name: "WGPUOptionalBool_Undefined", source_lines: "15" },
    MacroAuthority { name: "WGPUOptionalBool_Force32", source_lines: "16" },
    MacroAuthority { name: "WGPUSurfaceGetCurrentTextureStatus_SuccessOptimal", source_lines: "19-20" },
    MacroAuthority { name: "WGPUMapAsyncStatus_Success", source_lines: "22" },
    MacroAuthority { name: "WGPUMapAsyncStatus_Force32", source_lines: "23" },
    MacroAuthority { name: "WGPUSType_ShaderSourceWGSL", source_lines: "25" },
    MacroAuthority { name: "WGPUStatus", source_lines: "26" },
    MacroAuthority { name: "WGPUStatus_Success", source_lines: "28" },
    MacroAuthority { name: "wgpuGetInstanceCapabilities", source_lines: "30" },
    MacroAuthority { name: "WGPUMapAsyncStatus", source_lines: "32" },
    MacroAuthority { name: "WGPUShaderSourceWGSL", source_lines: "33" },
    MacroAuthority { name: "WGPURenderPassMaxDrawCount", source_lines: "34" },
    MacroAuthority { name: "WGPUWGSLLanguageFeatureName", source_lines: "35" },
    MacroAuthority { name: "WGPUEmscriptenSurfaceSourceCanvasHTMLSelector", source_lines: "37-38" },
    MacroAuthority { name: "WGPUWGSLLanguageFeatureName_Force32", source_lines: "39" },
    MacroAuthority { name: "WGPUSType_EmscriptenSurfaceSourceCanvasHTMLSelector", source_lines: "40-41" },
    MacroAuthority { name: "WGPUSType_RenderPassMaxDrawCount", source_lines: "42-43" },
    MacroAuthority { name: "WGPUSType_ShaderSourceWGSL", source_lines: "44" },
    MacroAuthority { name: "WGPUWGSLLanguageFeatureName_Packed4x8IntegerDotProduct", source_lines: "45-46" },
    MacroAuthority { name: "WGPUWGSLLanguageFeatureName_PointerCompositeAccess", source_lines: "47-48" },
    MacroAuthority { name: "WGPUWGSLLanguageFeatureName_ReadonlyAndReadwriteStorageTextures", source_lines: "49-50" },
    MacroAuthority { name: "WGPUWGSLLanguageFeatureName_UnrestrictedPointerParameters", source_lines: "52-53" },
    MacroAuthority { name: "WGPU_STRING_VIEW", source_lines: "55" },
    MacroAuthority { name: "WGPU_STRING_VIEW_INIT", source_lines: "56" },
    MacroAuthority { name: "WGPU_STRING_VIEW_TO_STRING", source_lines: "57" },
    MacroAuthority { name: "WGPU_STRING_VIEW_TO_CSTR", source_lines: "58" },
    MacroAuthority { name: "WGPU_STRING_VIEW_FROM_STRING", source_lines: "59" },
    MacroAuthority { name: "WGPU_STRING_VIEW_FREE", source_lines: "60" },
    MacroAuthority { name: "WGPUStringView", source_lines: "61" },
    MacroAuthority { name: "WGPU_ADDREF", source_lines: "63" },
    MacroAuthority { name: "WGPU_CHECK_STATUS", source_lines: "65" },
    MacroAuthority { name: "WGPU_MAKE_INIT_STRUCT", source_lines: "69" },
    MacroAuthority { name: "WGPU_MAKE_INIT_STRUCT", source_lines: "71" },
    MacroAuthority { name: "WGPU_MAKE_INIT_STRUCT", source_lines: "74" },
    MacroAuthority { name: "WGPU_MAKE_INIT_STRUCT", source_lines: "76" },
    MacroAuthority { name: "_wgpu_COMMA", source_lines: "80" },
    MacroAuthority { name: "WGPU_ADAPTER_INFO_INIT", source_lines: "83-91" },
    MacroAuthority { name: "WGPU_BIND_GROUP_DESCRIPTOR_INIT", source_lines: "93-98" },
    MacroAuthority { name: "WGPU_BIND_GROUP_ENTRY_INIT", source_lines: "100-107" },
    MacroAuthority { name: "WGPU_BIND_GROUP_LAYOUT_DESCRIPTOR_INIT", source_lines: "109-113" },
    MacroAuthority { name: "WGPU_BIND_GROUP_LAYOUT_ENTRY_INIT", source_lines: "115-125" },
    MacroAuthority { name: "WGPU_BLEND_COMPONENT_INIT", source_lines: "127-132" },
    MacroAuthority { name: "WGPU_BLEND_STATE_INIT", source_lines: "134-138" },
    MacroAuthority { name: "WGPU_BUFFER_BINDING_LAYOUT_INIT", source_lines: "140-145" },
    MacroAuthority { name: "WGPU_BUFFER_DESCRIPTOR_INIT", source_lines: "147-152" },
    MacroAuthority { name: "WGPU_BUFFER_MAP_CALLBACK_INFO_INIT", source_lines: "154-160" },
    MacroAuthority { name: "WGPU_COLOR_INIT", source_lines: "162-165" },
    MacroAuthority { name: "WGPU_COLOR_TARGET_STATE_INIT", source_lines: "167-173" },
    MacroAuthority { name: "WGPU_COMMAND_BUFFER_DESCRIPTOR_INIT", source_lines: "175-178" },
    MacroAuthority { name: "WGPU_COMMAND_ENCODER_DESCRIPTOR_INIT", source_lines: "180-183" },
    MacroAuthority { name: "WGPU_COMPILATION_INFO_INIT", source_lines: "185-189" },
    MacroAuthority { name: "WGPU_COMPILATION_MESSAGE_INIT", source_lines: "191-200" },
    MacroAuthority { name: "WGPU_COMPUTE_PASS_DESCRIPTOR_INIT", source_lines: "202-206" },
    MacroAuthority { name: "WGPU_COMPUTE_PASS_TIMESTAMP_WRITES_INIT", source_lines: "208-214" },
    MacroAuthority { name: "WGPU_COMPUTE_PIPELINE_DESCRIPTOR_INIT", source_lines: "216-221" },
    MacroAuthority { name: "WGPU_COMPUTE_STATE_INIT", source_lines: "223-229" },
    MacroAuthority { name: "WGPU_CONSTANT_ENTRY_INIT", source_lines: "231-234" },
    MacroAuthority { name: "WGPU_DEPTH_STENCIL_STATE_INIT", source_lines: "236-248" },
    MacroAuthority { name: "WGPU_DEVICE_DESCRIPTOR_INIT", source_lines: "250-260" },
    MacroAuthority { name: "WGPU_EXTENT_3D_INIT", source_lines: "262-266" },
    MacroAuthority { name: "WGPU_FRAGMENT_STATE_INIT", source_lines: "268-275" },
    MacroAuthority { name: "WGPU_TEXEL_COPY_BUFFER_INFO_INIT", source_lines: "277-281" },
    MacroAuthority { name: "WGPU_TEXEL_COPY_TEXTURE_INFO_INIT", source_lines: "283-289" },
    MacroAuthority { name: "WGPU_INSTANCE_DESCRIPTOR_INIT", source_lines: "291-294" },
    MacroAuthority { name: "WGPU_INSTANCE_CAPABILITIES_INIT", source_lines: "296-300" },
    MacroAuthority { name: "WGPU_LIMITS_INIT", source_lines: "302-333" },
    MacroAuthority { name: "WGPU_MULTISAMPLE_STATE_INIT", source_lines: "335-339" },
    MacroAuthority { name: "WGPU_ORIGIN_3D_INIT", source_lines: "341-344" },
    MacroAuthority { name: "WGPU_PIPELINE_LAYOUT_DESCRIPTOR_INIT", source_lines: "346-351" },
    MacroAuthority { name: "WGPU_PRIMITIVE_STATE_INIT", source_lines: "353-361" },
    MacroAuthority { name: "WGPU_QUERY_SET_DESCRIPTOR_INIT", source_lines: "363-367" },
    MacroAuthority { name: "WGPU_QUEUE_DESCRIPTOR_INIT", source_lines: "369-372" },
    MacroAuthority { name: "WGPU_RENDER_BUNDLE_DESCRIPTOR_INIT", source_lines: "374-377" },
    MacroAuthority { name: "WGPU_RENDER_BUNDLE_ENCODER_DESCRIPTOR_INIT", source_lines: "379-387" },
    MacroAuthority { name: "WGPU_RENDER_PASS_COLOR_ATTACHMENT_INIT", source_lines: "389-396" },
    MacroAuthority { name: "WGPU_RENDER_PASS_DEPTH_STENCIL_ATTACHMENT_INIT", source_lines: "398-409" },
    MacroAuthority { name: "WGPU_RENDER_PASS_DESCRIPTOR_INIT", source_lines: "411-419" },
    MacroAuthority { name: "WGPU_RENDER_PASS_MAX_DRAW_COUNT_INIT", source_lines: "421-428" },
    MacroAuthority { name: "WGPU_RENDER_PASS_TIMESTAMP_WRITES_INIT", source_lines: "430-436" },
    MacroAuthority { name: "WGPU_RENDER_PIPELINE_DESCRIPTOR_INIT", source_lines: "438-447" },
    MacroAuthority { name: "WGPU_REQUEST_ADAPTER_OPTIONS_INIT", source_lines: "449-456" },
    MacroAuthority { name: "WGPU_REQUIRED_LIMITS_INIT", source_lines: "458-462" },
    MacroAuthority { name: "WGPU_SAMPLER_BINDING_LAYOUT_INIT", source_lines: "464-467" },
    MacroAuthority { name: "WGPU_SAMPLER_DESCRIPTOR_INIT", source_lines: "469-483" },
    MacroAuthority { name: "WGPU_SHADER_MODULE_DESCRIPTOR_INIT", source_lines: "485-488" },
    MacroAuthority { name: "WGPU_SHADER_SOURCE_SPIRV_INIT", source_lines: "490-496" },
    MacroAuthority { name: "WGPU_SHADER_SOURCE_WGSL_INIT", source_lines: "498-503" },
    MacroAuthority { name: "WGPU_STENCIL_FACE_STATE_INIT", source_lines: "505-512" },
    MacroAuthority { name: "WGPU_STORAGE_TEXTURE_BINDING_LAYOUT_INIT", source_lines: "514-520" },
    MacroAuthority { name: "WGPU_SUPPORTED_LIMITS_INIT", source_lines: "522-526" },
    MacroAuthority { name: "WGPU_SURFACE_CAPABILITIES_INIT", source_lines: "528-534" },
    MacroAuthority { name: "WGPU_SURFACE_CONFIGURATION_INIT", source_lines: "536-545" },
    MacroAuthority { name: "WGPU_SURFACE_DESCRIPTOR_INIT", source_lines: "547-550" },
    MacroAuthority { name: "WGPU_SURFACE_TEXTURE_INIT", source_lines: "552-555" },
    MacroAuthority { name: "WGPU_TEXTURE_BINDING_LAYOUT_INIT", source_lines: "557-563" },
    MacroAuthority { name: "WGPU_TEXTURE_BINDING_VIEW_DIMENSION_DESCRIPTOR_INIT", source_lines: "565-572" },
    MacroAuthority { name: "WGPU_TEXTURE_DATA_LAYOUT_INIT", source_lines: "574-580" },
    MacroAuthority { name: "WGPU_TEXTURE_DESCRIPTOR_INIT", source_lines: "582-591" },
    MacroAuthority { name: "WGPU_TEXTURE_VIEW_DESCRIPTOR_INIT", source_lines: "593-603" },
    MacroAuthority { name: "WGPU_VERTEX_ATTRIBUTE_INIT", source_lines: "605-608" },
    MacroAuthority { name: "WGPU_VERTEX_BUFFER_LAYOUT_INIT", source_lines: "610-614" },
    MacroAuthority { name: "WGPU_VERTEX_STATE_INIT", source_lines: "616-623" },
    MacroAuthority { name: "WGPU_QUEUE_WORK_DONE_CALLBACK_INFO_INIT", source_lines: "625-630" },
    MacroAuthority { name: "WGPU_WAGYU_STRING_VIEW", source_lines: "633-634" },
    MacroAuthority { name: "WGPU_WAGYU_STRING_VIEW", source_lines: "636-637" },
];

/// Returns the exact frozen source text covered by one preprocessor definition.
/// This retains every aggregate field value and nested initializer token until
/// the corresponding generated ABI struct owner is admitted.
pub(crate) fn macroSource(authority: MacroAuthority) -> String {
    let mut output = String::new();
    for range in authority.source_lines.split(',') {
        let (start, end) = range
            .split_once('-')
            .map(|(start, end)| (start, end))
            .unwrap_or((range, range));
        let start: usize = start.parse().expect("frozen macro line start");
        let end: usize = end.parse().expect("frozen macro line end");
        for line in PINNED_SOURCE.lines().skip(start - 1).take(end - start + 1) {
            if !output.is_empty() {
                output.push('\n');
            }
            output.push_str(line);
        }
    }
    output
}

const _: [(); PINNED_SOURCE_BYTE_COUNT] = [(); PINNED_SOURCE.len()];
const _: [(); 114] = [(); MACRO_AUTHORITIES.len()];

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use std::ffi::CString;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn complete_macro_denominator_and_duplicate_branches_are_frozen() {
        let mut counts = BTreeMap::new();
        for authority in MACRO_AUTHORITIES {
            *counts.entry(authority.name).or_insert(0usize) += 1;
            assert!(macroSource(*authority).starts_with("#define "));
        }
        assert_eq!(MACRO_AUTHORITIES.len(), 114);
        assert_eq!(counts.len(), 109);
        assert_eq!(counts["WGPUSType_ShaderSourceWGSL"], 2);
        assert_eq!(counts["WGPU_MAKE_INIT_STRUCT"], 4);
        assert_eq!(counts["WGPU_WAGYU_STRING_VIEW"], 2);
    }

    #[test]
    fn compatibility_alias_denominator_is_frozen() {
        assert_eq!(COMPAT_ALIASES.len(), 21);
        assert_eq!(COMPAT_ALIASES[0].compatibility_name, "WGPUTexelCopyBufferInfo");
        assert_eq!(COMPAT_ALIASES[20].pinned_name, "WGPUWGSLFeatureName_UnrestrictedPointerParameters");
    }

    #[test]
    fn optional_bool_and_status_values_match_source() {
        assert_eq!(wgpu_bool(false), WGPUOptionalBool_False);
        assert_eq!(wgpu_bool(true), WGPUOptionalBool_True);
        assert_eq!(WGPUOptionalBool_Undefined, 2);
        assert_eq!(WGPUOptionalBool_Force32, i32::MAX);
        assert_eq!(WGPU_CHECK_STATUS(WGPUStatus_Success), 1);
        assert!(WGPU_DEPTH_CLEAR_VALUE_UNDEFINED.is_nan());
        assert_eq!(WGPU_RENDER_PASS_MAX_DRAW_COUNT_DEFAULT, 50_000_000);
    }

    #[test]
    fn string_view_duplication_and_free_preserve_source_ownership() {
        let source = CString::new("rive-webgpu").unwrap();
        let duplicate = WGPU_STRING_VIEW_FROM_STRING(&source);
        assert!(!duplicate.is_null());
        assert_ne!(duplicate, source.as_ptr());
        // SAFETY: `duplicate` came from the matching duplication function and
        // remains live until the one free below.
        unsafe {
            assert_eq!(WGPU_STRING_VIEW_TO_STRING(duplicate), b"rive-webgpu");
            assert_eq!(WGPU_STRING_VIEW_TO_CSTR(duplicate), duplicate);
            WGPU_STRING_VIEW_FREE(duplicate);
        }
    }

    #[test]
    fn wagyu_string_view_branches_share_pointer_and_strlen_semantics() {
        let source = CString::new("wgsl").unwrap();
        // SAFETY: Both inputs are null or live NUL-terminated C strings.
        unsafe {
            assert_eq!(
                WGPU_WAGYU_STRING_VIEW(ptr::null()),
                WagyuStringViewParts { data: ptr::null(), length: 0 }
            );
            assert_eq!(
                WGPU_WAGYU_STRING_VIEW(source.as_ptr()),
                WagyuStringViewParts { data: source.as_ptr(), length: 4 }
            );
        }
    }

    #[test]
    fn addref_dispatches_exactly_once() {
        struct Handle(AtomicUsize);
        impl WgpuReference for Handle {
            fn reference(&self) {
                self.0.fetch_add(1, Ordering::Relaxed);
            }
        }

        let handle = Handle(AtomicUsize::new(0));
        WGPU_ADDREF(&handle);
        assert_eq!(handle.0.load(Ordering::Relaxed), 1);
    }
}
