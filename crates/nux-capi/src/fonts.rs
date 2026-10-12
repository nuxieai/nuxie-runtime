//! C access to the engine's fonts and process-wide fallback hook.
use super::*;
use nuxie::runtime::text::font_hb::{FileProbe, HbFont};
use nuxie::runtime::text::text_engine::{Coord, Font, FontRef, set_fallback_proc};

pub const NUX_FONT_MAX_BYTES: usize = 256 * 1024 * 1024;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NuxFontAxisCoord {
    /// Big-endian OpenType tag, for example 0x77676874 for wght.
    pub axis: u32,
    pub value: f32,
}

/// One font face. Supply exactly one of bytes or immutable_path.
/// Byte storage is borrowed for font creation and copied once. A path is
/// UTF-8, without a terminating NUL. Its trusted local file must remain
/// immutable and available until this font and all its variants die.
/// Never supply authored, removable or network files through the mapped path.
/// Arrays use an exact struct_size and fixed stride. Coordinates are borrowed
/// only for creation; face_index and coordinates select a collection face.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NuxFontSource {
    pub struct_size: u32,
    pub bytes: NuxByteView,
    pub immutable_path: NuxStringView,
    pub face_index: u32,
    pub coords: *const NuxFontAxisCoord,
    pub coord_count: usize,
}

impl Default for NuxFontSource {
    fn default() -> Self {
        Self {
            struct_size: std::mem::size_of::<Self>() as u32,
            bytes: NuxByteView::default(),
            immutable_path: NuxStringView::default(),
            face_index: 0,
            coords: ptr::null(),
            coord_count: 0,
        }
    }
}

/// Immutable font, usable on any thread. The engine retains callback answers
/// independently of the C handle. Do not free while a callback is returning it.
pub struct NuxFont {
    font: FontRef,
}

/// A mapped file coverage probe, affine to its creation thread. Promotion
/// transfers its mapping to a font; further coverage queries return false.
pub struct NuxFontFileProbe {
    probe: FileProbe,
}

unsafe fn decode_face(face: &NuxFontSource) -> Result<(FontRef, usize), NuxStatus> {
    if face.struct_size as usize != std::mem::size_of::<NuxFontSource>() {
        return Err(NuxStatus::InvalidStructSize);
    }
    if face.bytes.len > NUX_FONT_MAX_BYTES
        || face.immutable_path.len > 4096
        || face.coord_count > 64
    {
        return Err(NuxStatus::LimitExceeded);
    }
    if face.coord_count != 0 && face.coords.is_null() {
        return Err(NuxStatus::NullArgument);
    }
    let coords = if face.coord_count == 0 {
        &[]
    } else {
        unsafe { slice::from_raw_parts(face.coords, face.coord_count) }
    };
    let mut native_coords = Vec::with_capacity(coords.len());
    for coord in coords {
        if !coord.value.is_finite() || native_coords.iter().any(|c: &Coord| c.axis == coord.axis) {
            return Err(NuxStatus::InvalidArgument);
        }
        native_coords.push(Coord {
            axis: coord.axis,
            value: coord.value,
        });
    }
    let (native, size) = match (face.bytes.len != 0, face.immutable_path.len != 0) {
        (true, false) => {
            if face.bytes.data.is_null() {
                return Err(NuxStatus::NullArgument);
            }
            let bytes = unsafe { slice::from_raw_parts(face.bytes.data, face.bytes.len) };
            (HbFont::decode_face(bytes, face.face_index), bytes.len())
        }
        (false, true) => {
            let path = with_utf8_view(face.immutable_path, str::to_owned)?;
            if path.contains('\0') || !std::path::Path::new(&path).is_absolute() {
                return Err(NuxStatus::InvalidArgument);
            }
            let path = std::path::Path::new(&path);
            let metadata = std::fs::metadata(path).map_err(|_| NuxStatus::NotFound)?;
            if !metadata.is_file() || metadata.len() == 0 {
                return Err(NuxStatus::InvalidArgument);
            }
            if metadata.len() > NUX_FONT_MAX_BYTES as u64 {
                return Err(NuxStatus::LimitExceeded);
            }
            // The C caller explicitly accepts the documented stable-file contract.
            (
                unsafe { HbFont::decode_file(Some(path), face.face_index) },
                metadata.len() as usize,
            )
        }
        _ => return Err(NuxStatus::InvalidArgument),
    };
    let font = native.ok_or(NuxStatus::ImportError)?;
    Ok((font.make_at_coords(&native_coords), size))
}

fn publish_font(font: FontRef) -> *mut NuxFont {
    let handle = Box::into_raw(Box::new(NuxFont { font }));
    register_handle(handle, HandleKind::Font, thread::current().id());
    handle
}

fn clone_font(handle: *const NuxFont) -> Result<FontRef, NuxStatus> {
    if handle.is_null() {
        return Err(NuxStatus::NullArgument);
    }
    // Clone under the registry lock so concurrent free cannot race the read.
    let registry = registry_lock();
    if !registry
        .get(&(handle as usize))
        .is_some_and(|r| r.kind == HandleKind::Font)
    {
        return Err(NuxStatus::HandleMismatch);
    }
    Ok(unsafe { &*handle }.font.clone())
}

/// Decode copied bytes or map a trusted immutable file, with collection face
/// and optional axis coordinates. All input views are borrowed for this call.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_font_new(
    source: *const NuxFontSource,
    out_font: *mut *mut NuxFont,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if out_font.is_null() {
            return NuxStatus::NullArgument;
        }
        unsafe {
            *out_font = ptr::null_mut();
        }
        if source.is_null() {
            return NuxStatus::NullArgument;
        }
        match unsafe { decode_face(&*source) } {
            Ok((font, _)) => {
                unsafe {
                    *out_font = publish_font(font);
                }
                NuxStatus::Ok
            }
            Err(status) => status,
        }
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_font_free(font: *mut NuxFont) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if font.is_null() {
            return NuxStatus::Ok;
        }
        if let Err(status) = remove_handle(font, HandleKind::Font) {
            return status;
        }
        unsafe {
            drop(Box::from_raw(font));
        }
        NuxStatus::Ok
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_font_has_glyph(
    font: *const NuxFont,
    character: u32,
    out_has_glyph: *mut bool,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if out_has_glyph.is_null() {
            return NuxStatus::NullArgument;
        }
        unsafe {
            *out_has_glyph = false;
        }
        match clone_font(font) {
            Ok(font) => {
                unsafe {
                    *out_has_glyph = font.has_glyph(character);
                }
                NuxStatus::Ok
            }
            Err(status) => status,
        }
    })
}

/// Copy at the requested weight only when the font has a wght axis; fixed
/// fonts retain their own weight, as in the upstream Android system provider.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_font_with_weight(
    font: *const NuxFont,
    weight: u16,
    out_font: *mut *mut NuxFont,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if out_font.is_null() {
            return NuxStatus::NullArgument;
        }
        unsafe {
            *out_font = ptr::null_mut();
        }
        let font = match clone_font(font) {
            Ok(font) => font,
            Err(status) => return status,
        };
        let tag = u32::from_be_bytes(*b"wght");
        let font = if (0..font.get_axis_count()).any(|index| font.get_axis(index).tag == tag) {
            font.with_options(
                &[Coord {
                    axis: tag,
                    value: f32::from(weight),
                }],
                &[],
            )
        } else {
            font
        };
        unsafe {
            *out_font = publish_font(font);
        }
        NuxStatus::Ok
    })
}

/// Map a trusted stable font file for nominal coverage. The same immutable
/// local-file lifetime contract as NuxFontSource applies, including derived fonts.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_font_probe_file(
    path: NuxStringView,
    face_index: u32,
    out_probe: *mut *mut NuxFontFileProbe,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if out_probe.is_null() {
            return NuxStatus::NullArgument;
        }
        unsafe {
            *out_probe = ptr::null_mut();
        }
        if path.len > 4096 {
            return NuxStatus::LimitExceeded;
        }
        let path = match with_utf8_view(path, str::to_owned) {
            Ok(path) => path,
            Err(status) => return status,
        };
        if path.contains('\0') || !std::path::Path::new(&path).is_absolute() {
            return NuxStatus::InvalidArgument;
        }
        let path = std::path::Path::new(&path);
        let metadata = match std::fs::metadata(path) {
            Ok(value) => value,
            Err(_) => return NuxStatus::NotFound,
        };
        if !metadata.is_file() || metadata.len() == 0 {
            return NuxStatus::InvalidArgument;
        }
        if metadata.len() > NUX_FONT_MAX_BYTES as u64 {
            return NuxStatus::LimitExceeded;
        }
        let Some(probe) = (unsafe { HbFont::probe_file(Some(path), face_index) }) else {
            return NuxStatus::ImportError;
        };
        let handle = Box::into_raw(Box::new(NuxFontFileProbe { probe }));
        register_handle(handle, HandleKind::FontFileProbe, thread::current().id());
        unsafe {
            *out_probe = handle;
        }
        NuxStatus::Ok
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_font_probe_has_glyph(
    probe: *const NuxFontFileProbe,
    character: u32,
    out_has_glyph: *mut bool,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if out_has_glyph.is_null() {
            return NuxStatus::NullArgument;
        }
        unsafe {
            *out_has_glyph = false;
        }
        let _call = enter_status_handle!(probe, HandleKind::FontFileProbe);
        unsafe {
            *out_has_glyph = (&*probe).probe.has_glyph(character);
        }
        NuxStatus::Ok
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_font_probe_make_font(
    probe: *mut NuxFontFileProbe,
    out_font: *mut *mut NuxFont,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if out_font.is_null() {
            return NuxStatus::NullArgument;
        }
        unsafe {
            *out_font = ptr::null_mut();
        }
        let _call = enter_status_handle!(probe, HandleKind::FontFileProbe);
        let Some(font) = (unsafe { &mut *probe }).probe.make_font() else {
            return NuxStatus::NotFound;
        };
        unsafe {
            *out_font = publish_font(font);
        }
        NuxStatus::Ok
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_font_probe_free(probe: *mut NuxFontFileProbe) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        if probe.is_null() {
            return NuxStatus::Ok;
        }
        if let Err(status) = remove_handle(probe, HandleKind::FontFileProbe) {
            return status;
        }
        unsafe {
            drop(Box::from_raw(probe));
        }
        NuxStatus::Ok
    })
}

/// Return a borrowed font handle, or NULL to end the fallback walk. Called on
/// the shaping thread, with the first missing character, zero-based retry index
/// and asking font's weight. The engine retains the returned font. The provider
/// must be thread-safe and must terminate its retry walk.
pub type NuxFontFallbackProc = Option<
    unsafe extern "C" fn(
        user_data: *mut c_void,
        missing: u32,
        fallback_index: u32,
        weight: u16,
    ) -> *const NuxFont,
>;

struct FallbackRegistration {
    callback: unsafe extern "C" fn(*mut c_void, u32, u32, u16) -> *const NuxFont,
    context: usize,
}

static C_FALLBACK: std::sync::RwLock<Option<FallbackRegistration>> = std::sync::RwLock::new(None);

fn c_fallback(missing: u32, index: u32, asking_font: &dyn Font) -> Option<FontRef> {
    // Hold the read lease through callback invocation and answer retention.
    // Replacement may then safely release the previous foreign context.
    let registration = C_FALLBACK
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let registration = registration.as_ref()?;
    let handle = unsafe {
        (registration.callback)(
            registration.context as *mut c_void,
            missing,
            index,
            asking_font.get_weight(),
        )
    };
    clone_font(handle).ok()
}

/// Set the engine's process-wide fallback hook; NULL clears it. Applies to all
/// files, including files already open when next reshaped. Callback and context
/// must remain valid until replacement completes. Do not replace the hook from
/// inside its callback. Replacement waits for current C callbacks to return.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_font_set_fallback_proc(
    _callback: NuxFontFallbackProc,
    _user_data: *mut c_void,
) -> NuxStatus {
    ffi_guard(NuxStatus::RuntimeError, || {
        let mut registration = C_FALLBACK
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *registration = _callback.map(|callback| FallbackRegistration {
            callback,
            context: _user_data as usize,
        });
        set_fallback_proc(if _callback.is_some() {
            Some(c_fallback)
        } else {
            None
        });
        NuxStatus::Ok
    })
}
