//! C access to the engine's fonts and process-wide fallback hook.
use super::*;
use nuxie::runtime::text::font_hb::{FileProbe, HbFont, HostGlyphPicture, HostPictureScope};
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
/// struct_size must cover the ABI-v4 prefix; appended fields are ignored.
/// Coordinates are borrowed only for creation and have a fixed element stride;
/// face_index and coordinates select a collection face.
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

const FONT_SOURCE_V4_MIN_SIZE: usize =
    std::mem::offset_of!(NuxFontSource, coord_count) + std::mem::size_of::<usize>();

unsafe fn decode_face(source: *const NuxFontSource) -> Result<(FontRef, usize), NuxStatus> {
    let caller_size = unsafe { source.cast::<u32>().read() };
    if !struct_size_supports(caller_size, FONT_SOURCE_V4_MIN_SIZE) {
        return Err(NuxStatus::InvalidStructSize);
    }
    let mut face = NuxFontSource::default();
    unsafe {
        ptr::copy_nonoverlapping(
            source.cast::<u8>(),
            ptr::from_mut(&mut face).cast(),
            FONT_SOURCE_V4_MIN_SIZE,
        );
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
    font_ffi_guard(NuxStatus::RuntimeError, || {
        if out_font.is_null() {
            return NuxStatus::NullArgument;
        }
        unsafe {
            *out_font = ptr::null_mut();
        }
        if source.is_null() {
            return NuxStatus::NullArgument;
        }
        match unsafe { decode_face(source) } {
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

/// Emit one normalized, y-down path command synchronously. Verbs: 0 move,
/// 1 line, 2 quadratic, 3 cubic, 4 close. Points has 2/2/4/6/0 floats.
/// The sink and its context expire when get_path returns.
pub type NuxFontOutlineSink = unsafe extern "C" fn(*mut c_void, u8, *const f32);

/// A borrowed PNG picture. All storage is copied before the sink returns.
/// Bearings and extents are in em units; top is the top edge above the baseline,
/// positive up. Width and height describe the PNG in pixels, at most 1024 each.
/// PNG data is limited to 4 MiB; dimensions must match its IHDR, and all metrics
/// must be finite.
/// struct_size must cover this prefix; appended fields are ignored.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct NuxFontPicture {
    pub struct_size: u32,
    pub png: *const u8,
    pub png_len: usize,
    pub width: u32,
    pub height: u32,
    pub bearing_x: f32,
    pub top: f32,
    pub extent_x: f32,
    pub extent_y: f32,
}

/// Emit at most one picture synchronously. Passing NULL or making no sink call
/// supplies no picture. A second call discards the entire answer. The sink and
/// its context expire when get_picture returns.
pub type NuxFontPictureSink = unsafe extern "C" fn(*mut c_void, *const NuxFontPicture);

/// Host callbacks are copied at creation. Context ownership transfers only on
/// success and release runs once after the last retained host font dies,
/// possibly on any thread. get_path and get_picture may run on any thread,
/// concurrently; callbacks and context must be thread-safe, must not unwind,
/// and must not reenter the font API. At least one callback must be supplied.
/// get_picture is glyph-scoped: every nonzero glyph without native colour is
/// requested once while cached, independently per font variant. Glyph 0 is never
/// requested. Cluster code points are borrowed context only; the first context
/// seen for a glyph supplies its cached answer. Present and absent answers share
/// a 4096-entry FIFO cache per variant; evicted glyphs may be requested again.
/// with_options variants have independent caches. Every cluster component may
/// draw a picture; return no picture for spaces or hidden glyphs without ink.
/// Native COLR and readable PNG pictures take precedence.
/// struct_size must cover the ABI-v4 prefix through release. get_picture is
/// optional and read only when struct_size covers it; future fields are ignored.
#[repr(C)]
#[derive(Clone, Copy)]
pub struct NuxFontOutlineCallbacks {
    pub struct_size: u32,
    pub user_data: *mut c_void,
    pub get_path: Option<
        unsafe extern "C" fn(
            *mut c_void,
            u16,
            *const NuxFontAxisCoord,
            usize,
            NuxFontOutlineSink,
            *mut c_void,
        ),
    >,
    pub release: Option<unsafe extern "C" fn(*mut c_void)>,
    pub get_picture: Option<
        unsafe extern "C" fn(*mut c_void, u16, *const u32, usize, NuxFontPictureSink, *mut c_void),
    >,
}

const FONT_OUTLINE_CALLBACKS_V4_MIN_SIZE: usize =
    std::mem::offset_of!(NuxFontOutlineCallbacks, release)
        + std::mem::size_of::<Option<unsafe extern "C" fn(*mut c_void)>>();

unsafe fn read_font_outline_callbacks(
    callbacks: *const NuxFontOutlineCallbacks,
) -> Result<NuxFontOutlineCallbacks, NuxStatus> {
    let caller_size = unsafe { callbacks.cast::<u32>().read() };
    if !struct_size_supports(caller_size, FONT_OUTLINE_CALLBACKS_V4_MIN_SIZE) {
        return Err(NuxStatus::InvalidStructSize);
    }
    let mut value: NuxFontOutlineCallbacks = unsafe { std::mem::zeroed() };
    unsafe {
        ptr::copy_nonoverlapping(
            callbacks.cast::<u8>(),
            ptr::from_mut(&mut value).cast(),
            FONT_OUTLINE_CALLBACKS_V4_MIN_SIZE,
        );
    }
    if struct_size_supports(caller_size, std::mem::size_of::<NuxFontOutlineCallbacks>()) {
        value.get_picture = unsafe { ptr::addr_of!((*callbacks).get_picture).read() };
    }
    Ok(value)
}

struct HostOutline(NuxFontOutlineCallbacks);
// The caller accepts the documented concurrent-callback contract.
unsafe impl Send for HostOutline {}
unsafe impl Sync for HostOutline {}
impl Drop for HostOutline {
    fn drop(&mut self) {
        if let Some(release) = self.0.release {
            unsafe { release(self.0.user_data) };
        }
    }
}

struct OutlineBuilder {
    path: nuxie::runtime::math::raw_path::RawPath,
    count: usize,
    valid: bool,
    started: bool,
}

unsafe extern "C" fn append_outline(context: *mut c_void, verb: u8, points: *const f32) {
    let builder = unsafe { &mut *context.cast::<OutlineBuilder>() };
    builder.count += 1;
    let count = match verb {
        0 | 1 => 2,
        2 => 4,
        3 => 6,
        4 => 0,
        _ => {
            builder.valid = false;
            return;
        }
    };
    if !builder.valid
        || builder.count > 65_536
        || (count != 0 && points.is_null())
        || (verb != 0 && !builder.started)
    {
        builder.valid = false;
        return;
    }
    let points = if count == 0 {
        &[]
    } else {
        unsafe { slice::from_raw_parts(points, count) }
    };
    if points.iter().any(|point| !point.is_finite()) {
        builder.valid = false;
        return;
    }
    match verb {
        0 => {
            builder.path.move_to(points[0], points[1]);
            builder.started = true;
        }
        1 => builder.path.line_to(points[0], points[1]),
        2 => builder
            .path
            .quad_to_cubic(points[0], points[1], points[2], points[3]),
        3 => builder.path.cubic_to(
            points[0], points[1], points[2], points[3], points[4], points[5],
        ),
        4 => builder.path.close(),
        _ => unreachable!(),
    }
}

const FONT_PICTURE_MAX_PNG_BYTES: usize = 4 * 1024 * 1024;
const FONT_PICTURE_MAX_SIDE: u32 = 1024;
const FONT_PICTURE_MIN_SIZE: usize =
    std::mem::offset_of!(NuxFontPicture, extent_y) + std::mem::size_of::<f32>();

#[derive(Default)]
struct PictureBuilder {
    called: bool,
    picture: Option<HostGlyphPicture>,
}

unsafe extern "C" fn append_picture(context: *mut c_void, picture: *const NuxFontPicture) {
    let builder = unsafe { &mut *context.cast::<PictureBuilder>() };
    if builder.called {
        builder.picture = None;
        return;
    }
    builder.called = true;
    if picture.is_null() {
        return;
    }
    let caller_size = unsafe { picture.cast::<u32>().read() };
    if !struct_size_supports(caller_size, FONT_PICTURE_MIN_SIZE) {
        return;
    }
    // Read exactly the settled prefix, including on platforms where the struct
    // has trailing padding. The foreign allocation need only cover its prefix.
    let mut value: NuxFontPicture = unsafe { std::mem::zeroed() };
    unsafe {
        ptr::copy_nonoverlapping(
            picture.cast::<u8>(),
            ptr::from_mut(&mut value).cast(),
            FONT_PICTURE_MIN_SIZE,
        );
    }
    if value.png_len == 0
        || value.png_len > FONT_PICTURE_MAX_PNG_BYTES
        || value.png.is_null()
        || value.width == 0
        || value.height == 0
        || value.width > FONT_PICTURE_MAX_SIDE
        || value.height > FONT_PICTURE_MAX_SIDE
        || [value.bearing_x, value.top, value.extent_x, value.extent_y]
            .iter()
            .any(|metric| !metric.is_finite())
    {
        return;
    }
    let png = unsafe { slice::from_raw_parts(value.png, value.png_len) };
    // Check the encoded dimensions too, so a small reported size cannot bypass
    // the bitmap bound. Full PNG decoding remains the renderer's responsibility.
    if png.len() < 33
        || &png[..8] != b"\x89PNG\r\n\x1a\n"
        || png[8..12] != 13u32.to_be_bytes()
        || &png[12..16] != b"IHDR"
        || png[16..20] != value.width.to_be_bytes()
        || png[20..24] != value.height.to_be_bytes()
    {
        return;
    }
    builder.picture = Some(HostGlyphPicture {
        png: png.to_vec(),
        width: value.width,
        height: value.height,
        bearing_x: value.bearing_x,
        bearing_y: -value.top,
        extent_x: value.extent_x,
        extent_y: value.extent_y,
    });
}

impl HostOutline {
    fn picture(&self, glyph: u16, cluster: &[u32]) -> Option<HostGlyphPicture> {
        let mut builder = PictureBuilder::default();
        unsafe {
            self.0.get_picture.unwrap()(
                self.0.user_data,
                glyph,
                cluster.as_ptr(),
                cluster.len(),
                append_picture,
                (&mut builder as *mut PictureBuilder).cast(),
            );
        }
        builder.picture
    }

    fn path(&self, glyph: u16, coords: &[Coord]) -> nuxie::runtime::math::raw_path::RawPath {
        let coords: Vec<_> = coords
            .iter()
            .map(|coord| NuxFontAxisCoord {
                axis: coord.axis,
                value: coord.value,
            })
            .collect();
        let mut builder = OutlineBuilder {
            path: Default::default(),
            count: 0,
            valid: true,
            started: false,
        };
        unsafe {
            self.0.get_path.unwrap()(
                self.0.user_data,
                glyph,
                coords.as_ptr(),
                coords.len(),
                append_outline,
                (&mut builder as *mut OutlineBuilder).cast(),
            );
        }
        if builder.valid {
            builder.path
        } else {
            Default::default()
        }
    }
}

/// Decode a host font with optional empty-outline recovery and colour pictures.
/// Outline recovery mirrors upstream's first CoreTextHBFont attempt; pictures
/// extend it for native formats the runtime cannot decode. File shaping,
/// metrics and nonempty outlines remain unchanged. No fallback hook is installed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_font_new_with_outline(
    source: *const NuxFontSource,
    callbacks: *const NuxFontOutlineCallbacks,
    out_font: *mut *mut NuxFont,
) -> NuxStatus {
    font_ffi_guard(NuxStatus::RuntimeError, || {
        if out_font.is_null() {
            return NuxStatus::NullArgument;
        }
        unsafe {
            *out_font = ptr::null_mut();
        }
        if source.is_null() || callbacks.is_null() {
            return NuxStatus::NullArgument;
        }
        let callbacks = match unsafe { read_font_outline_callbacks(callbacks) } {
            Ok(callbacks) => callbacks,
            Err(status) => return status,
        };
        if callbacks.get_path.is_none() && callbacks.get_picture.is_none() {
            return NuxStatus::InvalidArgument;
        }
        let font = match unsafe { decode_face(source) } {
            Ok((font, _)) => font,
            Err(status) => return status,
        };
        let host = std::sync::Arc::new(HostOutline(callbacks));
        let mut font = font;
        if callbacks.get_path.is_some() {
            let host = host.clone();
            font = font
                .as_any()
                .downcast_ref::<HbFont>()
                .expect("decoded HBFont")
                .with_host_outline(std::sync::Arc::new(move |glyph, coords| {
                    host.path(glyph, coords)
                }));
        }
        if callbacks.get_picture.is_some() {
            let host = host.clone();
            font = font
                .as_any()
                .downcast_ref::<HbFont>()
                .expect("decoded HBFont")
                .with_host_pictures(
                    std::sync::Arc::new(move |glyph, cluster| host.picture(glyph, cluster)),
                    HostPictureScope::Glyph,
                );
        }
        unsafe {
            *out_font = publish_font(font);
        }
        NuxStatus::Ok
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_font_free(font: *mut NuxFont) -> NuxStatus {
    font_ffi_guard(NuxStatus::RuntimeError, || {
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
    font_ffi_guard(NuxStatus::RuntimeError, || {
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
/// A weighted copy of a variable host-outline font is a plain font without
/// outline recovery; a fixed host font is returned as is.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_font_with_weight(
    font: *const NuxFont,
    weight: u16,
    out_font: *mut *mut NuxFont,
) -> NuxStatus {
    font_ffi_guard(NuxStatus::RuntimeError, || {
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
    font_ffi_guard(NuxStatus::RuntimeError, || {
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
    font_ffi_guard(NuxStatus::RuntimeError, || {
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
    font_ffi_guard(NuxStatus::RuntimeError, || {
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
    font_ffi_guard(NuxStatus::RuntimeError, || {
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

std::thread_local! {
    static FALLBACK_HOOK_ACTIVE: Cell<bool> = const { Cell::new(false) };
}

pub(super) fn fallback_hook_active() -> bool {
    FALLBACK_HOOK_ACTIVE.with(Cell::get)
}

struct FallbackHookGuard;
impl Drop for FallbackHookGuard {
    fn drop(&mut self) {
        FALLBACK_HOOK_ACTIVE.with(|active| active.set(false));
    }
}

#[cfg(test)]
pub(super) fn with_fallback_hook_for_test<R>(work: impl FnOnce() -> R) -> R {
    assert!(!fallback_hook_active());
    FALLBACK_HOOK_ACTIVE.with(|active| active.set(true));
    let _guard = FallbackHookGuard;
    work()
}

fn c_fallback(missing: u32, index: u32, asking_font: &dyn Font) -> Option<FontRef> {
    if fallback_hook_active() {
        return None;
    }
    // Hold the read lease through callback invocation and answer retention.
    // Replacement may then safely release the previous foreign context.
    let registration = C_FALLBACK
        .read()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let registration = registration.as_ref()?;
    FALLBACK_HOOK_ACTIVE.with(|active| active.set(true));
    let _guard = FallbackHookGuard;
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
/// must remain valid until replacement completes. Replacement from inside its
/// callback returns REENTRANT_CALL. Other replacement waits for current C
/// callbacks to return. The hook may call font APIs, but not other C entry points.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nux_font_set_fallback_proc(
    _callback: NuxFontFallbackProc,
    _user_data: *mut c_void,
) -> NuxStatus {
    font_ffi_guard(NuxStatus::RuntimeError, || {
        if fallback_hook_active() {
            return NuxStatus::ReentrantCall;
        }
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

#[cfg(test)]
mod outline_tests {
    use super::*;
    use nuxie::runtime::text::text_engine::TextRun;
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    struct Context {
        calls: Arc<AtomicUsize>,
        releases: Arc<AtomicUsize>,
        invalid: bool,
    }

    unsafe extern "C" fn outline(
        context: *mut c_void,
        _: u16,
        _: *const NuxFontAxisCoord,
        _: usize,
        sink: NuxFontOutlineSink,
        destination: *mut c_void,
    ) {
        let context = unsafe { &*context.cast::<Context>() };
        context.calls.fetch_add(1, Ordering::SeqCst);
        unsafe {
            sink(destination, 0, [0.0, 0.0].as_ptr());
            sink(destination, 1, [1.0, 0.0].as_ptr());
            sink(destination, 2, [1.0, -1.0, 0.0, -1.0].as_ptr());
            if context.invalid {
                sink(destination, 1, [f32::NAN, 0.0].as_ptr());
            }
            sink(destination, 4, ptr::null());
        }
    }

    unsafe extern "C" fn release(context: *mut c_void) {
        let context = unsafe { Box::from_raw(context.cast::<Context>()) };
        context.releases.fetch_add(1, Ordering::SeqCst);
    }

    fn create(invalid: bool) -> (*mut NuxFont, Arc<AtomicUsize>, Arc<AtomicUsize>) {
        let bytes = include_bytes!("../../../fixtures/fonts/roboto-input.ttf");
        let source = NuxFontSource {
            bytes: NuxByteView {
                data: bytes.as_ptr(),
                len: bytes.len(),
            },
            ..Default::default()
        };
        let calls = Arc::new(AtomicUsize::new(0));
        let releases = Arc::new(AtomicUsize::new(0));
        let context = Box::into_raw(Box::new(Context {
            calls: calls.clone(),
            releases: releases.clone(),
            invalid,
        }));
        let callbacks = NuxFontOutlineCallbacks {
            struct_size: std::mem::size_of::<NuxFontOutlineCallbacks>() as u32,
            user_data: context.cast(),
            get_path: Some(outline),
            release: Some(release),
            get_picture: None,
        };
        let mut font = ptr::null_mut();
        assert_eq!(
            unsafe { nux_font_new_with_outline(&source, &callbacks, &mut font) },
            NuxStatus::Ok
        );
        (font, calls, releases)
    }

    fn glyph(font: &FontRef, character: char) -> u16 {
        let run = TextRun {
            font: Some(font.clone()),
            size: 32.0,
            line_height: -1.0,
            letter_spacing: 0.0,
            unichar_count: 1,
            script: u32::from_be_bytes(*b"Latn"),
            style_id: 0,
            level: 0,
        };
        font.shape_text(&[character as u32], &[run], 0)[0].runs[0].glyphs[0]
    }

    const PICTURE_PNG: &[u8] = &[
        137, 80, 78, 71, 13, 10, 26, 10, 0, 0, 0, 13, 73, 72, 68, 82, 0, 0, 0, 1, 0, 0, 0, 1, 8, 6,
        0, 0, 0, 31, 21, 196, 137, 0, 0, 0, 13, 73, 68, 65, 84, 120, 156, 99, 248, 207, 192, 240,
        31, 0, 5, 0, 1, 255, 137, 153, 61, 29, 0, 0, 0, 0, 73, 69, 78, 68, 174, 66, 96, 130,
    ];

    fn picture_value(png: &[u8]) -> NuxFontPicture {
        NuxFontPicture {
            struct_size: std::mem::size_of::<NuxFontPicture>() as u32,
            png: png.as_ptr(),
            png_len: png.len(),
            width: 1,
            height: 1,
            bearing_x: 0.125,
            top: 0.75,
            extent_x: 1.0,
            extent_y: 1.0,
        }
    }

    #[derive(Default)]
    struct PictureContext {
        calls: AtomicUsize,
        clusters: std::sync::Mutex<Vec<Vec<u32>>>,
        absent: bool,
    }

    unsafe extern "C" fn picture_callback(
        context: *mut c_void,
        _: u16,
        cluster: *const u32,
        cluster_len: usize,
        sink: NuxFontPictureSink,
        destination: *mut c_void,
    ) {
        let context = unsafe { &*context.cast::<PictureContext>() };
        context.calls.fetch_add(1, Ordering::SeqCst);
        context
            .clusters
            .lock()
            .unwrap()
            .push(unsafe { slice::from_raw_parts(cluster, cluster_len) }.to_vec());
        if !context.absent {
            let mut png = PICTURE_PNG.to_vec();
            unsafe { sink(destination, &picture_value(&png)) };
            png.fill(0); // Prove the runtime copied before returning to the host.
        }
    }

    fn create_picture_font(context: &mut PictureContext, callback_size: usize) -> *mut NuxFont {
        let bytes = include_bytes!("../../../fixtures/fonts/roboto-input.ttf");
        let source = NuxFontSource {
            bytes: NuxByteView {
                data: bytes.as_ptr(),
                len: bytes.len(),
            },
            ..Default::default()
        };
        let callbacks = NuxFontOutlineCallbacks {
            struct_size: callback_size as u32,
            user_data: ptr::from_mut(context).cast(),
            get_path: None,
            release: None,
            get_picture: Some(picture_callback),
        };
        let mut font = ptr::null_mut();
        assert_eq!(
            unsafe { nux_font_new_with_outline(&source, &callbacks, &mut font) },
            NuxStatus::Ok
        );
        font
    }

    #[test]
    fn host_picture_is_copied_cached_and_exposed_as_one_image_layer() {
        use nuxie::runtime::text::text_engine::ColorGlyphPaintType;
        let mut context = PictureContext::default();
        let handle =
            create_picture_font(&mut context, std::mem::size_of::<NuxFontOutlineCallbacks>());
        let font = clone_font(handle).unwrap();
        let glyph = glyph(&font, 'A');
        let cluster = [0x1f469, 0x200d, 0x1f4bb];
        assert!(font.is_color_glyph_with_text(glyph, &cluster));
        assert!(font.is_color_glyph_with_text(glyph, &[0x41]));
        assert!(font.is_color_glyph_with_text(glyph, &cluster));
        // Native-only queries stay independent of host answer cache state.
        assert!(!font.is_color_glyph(glyph));
        let mut layers = Vec::new();
        assert_eq!(
            font.get_color_layers_with_text(glyph, &cluster, &mut layers, 0xff000000),
            1
        );
        assert_eq!(layers.len(), 1);
        let layer = &layers[0];
        assert_eq!(layer.paint_type, ColorGlyphPaintType::Image);
        assert_eq!(layer.image_bytes.as_ref(), PICTURE_PNG);
        assert_eq!((layer.image_width, layer.image_height), (1, 1));
        assert_eq!(
            (layer.image_bearing_x, layer.image_bearing_y),
            (0.125, -0.75)
        );
        assert_eq!((layer.image_extent_x, layer.image_extent_y), (1.0, 1.0));
        let second_glyph = self::glyph(&font, 'B');
        std::thread::scope(|scope| {
            for _ in 0..8 {
                let font = &font;
                scope.spawn(move || assert!(font.is_color_glyph_with_text(second_glyph, &[0x42])));
            }
        });
        // Cluster text is context only: the first one for this glyph wins.
        // Concurrent requests for one glyph still call the host exactly once.
        assert!(!font.is_color_glyph_with_text(0, &[0x10ffff]));
        assert_eq!(context.calls.load(Ordering::SeqCst), 2);
        assert_eq!(
            *context.clusters.lock().unwrap(),
            [cluster.to_vec(), vec![0x42]]
        );
        let variant = font.with_options(&[], &[]);
        assert!(variant.is_color_glyph_with_text(glyph, &cluster));
        assert_eq!(context.calls.load(Ordering::SeqCst), 3);
        assert_eq!(unsafe { nux_font_free(handle) }, NuxStatus::Ok);
    }

    #[test]
    fn outline_and_picture_share_context_until_the_last_font_dies() {
        struct SharedContext {
            pictures: PictureContext,
            releases: Arc<AtomicUsize>,
        }
        unsafe extern "C" fn shared_picture(
            context: *mut c_void,
            glyph: u16,
            cluster: *const u32,
            cluster_len: usize,
            sink: NuxFontPictureSink,
            destination: *mut c_void,
        ) {
            let shared = unsafe { &*context.cast::<SharedContext>() };
            unsafe {
                picture_callback(
                    ptr::from_ref(&shared.pictures).cast_mut().cast(),
                    glyph,
                    cluster,
                    cluster_len,
                    sink,
                    destination,
                );
            }
        }
        unsafe extern "C" fn shared_outline(
            _: *mut c_void,
            _: u16,
            _: *const NuxFontAxisCoord,
            _: usize,
            sink: NuxFontOutlineSink,
            destination: *mut c_void,
        ) {
            unsafe {
                sink(destination, 0, [0.0, 0.0].as_ptr());
                sink(destination, 1, [1.0, 1.0].as_ptr());
            }
        }
        unsafe extern "C" fn shared_release(context: *mut c_void) {
            let shared = unsafe { Box::from_raw(context.cast::<SharedContext>()) };
            shared.releases.fetch_add(1, Ordering::SeqCst);
        }
        let releases = Arc::new(AtomicUsize::new(0));
        let context = Box::into_raw(Box::new(SharedContext {
            pictures: PictureContext::default(),
            releases: releases.clone(),
        }));
        let bytes = include_bytes!("../../../fixtures/fonts/roboto-input.ttf");
        let source = NuxFontSource {
            bytes: NuxByteView {
                data: bytes.as_ptr(),
                len: bytes.len(),
            },
            ..Default::default()
        };
        let callbacks = NuxFontOutlineCallbacks {
            struct_size: std::mem::size_of::<NuxFontOutlineCallbacks>() as u32,
            user_data: context.cast(),
            get_path: Some(shared_outline),
            release: Some(shared_release),
            get_picture: Some(shared_picture),
        };
        let mut handle = ptr::null_mut();
        assert_eq!(
            unsafe { nux_font_new_with_outline(&source, &callbacks, &mut handle) },
            NuxStatus::Ok
        );
        let font = clone_font(handle).unwrap();
        assert!(!font.get_path(glyph(&font, ' ')).empty());
        assert!(font.is_color_glyph_with_text(glyph(&font, 'A'), &[0x41]));
        let variant = font.with_options(&[], &[]);
        assert_eq!(unsafe { nux_font_free(handle) }, NuxStatus::Ok);
        drop(font);
        assert_eq!(releases.load(Ordering::SeqCst), 0);
        std::thread::spawn(move || {
            assert!(variant.is_color_glyph_with_text(glyph(&variant, 'B'), &[0x42]));
        })
        .join()
        .unwrap();
        assert_eq!(releases.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn absent_host_picture_is_cached_and_preserves_outline() {
        let mut context = PictureContext {
            absent: true,
            ..Default::default()
        };
        let handle =
            create_picture_font(&mut context, std::mem::size_of::<NuxFontOutlineCallbacks>());
        let font = clone_font(handle).unwrap();
        let original =
            HbFont::decode(include_bytes!("../../../fixtures/fonts/roboto-input.ttf")).unwrap();
        let glyph = glyph(&font, 'A');
        for cluster in [[0x41], [0x42], [0x43]] {
            assert!(!font.is_color_glyph_with_text(glyph, &cluster));
            assert!(!font.is_color_glyph(glyph));
        }
        assert_eq!(context.calls.load(Ordering::SeqCst), 1);
        assert_eq!(*context.clusters.lock().unwrap(), [vec![0x41]]);
        assert_eq!(
            font.get_path(glyph).points(),
            original.get_path(glyph).points()
        );
        assert_eq!(
            font.get_path(glyph).verbs(),
            original.get_path(glyph).verbs()
        );
        assert_eq!(unsafe { nux_font_free(handle) }, NuxStatus::Ok);
    }

    #[test]
    fn picture_sink_bounds_and_single_answer_are_enforced_before_copying() {
        fn collect(value: &NuxFontPicture, twice: bool) -> Option<HostGlyphPicture> {
            let mut builder = PictureBuilder::default();
            let context = ptr::from_mut(&mut builder).cast();
            unsafe { append_picture(context, value) };
            if twice {
                unsafe { append_picture(context, value) };
            }
            builder.picture
        }
        let valid = picture_value(PICTURE_PNG);
        assert!(collect(&valid, false).is_some());
        assert!(collect(&valid, true).is_none());
        for invalid in [
            NuxFontPicture {
                struct_size: 4,
                ..valid
            },
            NuxFontPicture {
                png: ptr::null(),
                ..valid
            },
            NuxFontPicture {
                png_len: 0,
                ..valid
            },
            NuxFontPicture {
                png_len: FONT_PICTURE_MAX_PNG_BYTES + 1,
                ..valid
            },
            NuxFontPicture {
                width: 1025,
                ..valid
            },
            NuxFontPicture {
                height: 1025,
                ..valid
            },
            NuxFontPicture { width: 0, ..valid },
            NuxFontPicture { height: 0, ..valid },
            NuxFontPicture {
                bearing_x: f32::NAN,
                ..valid
            },
            NuxFontPicture {
                top: f32::INFINITY,
                ..valid
            },
            NuxFontPicture {
                extent_x: f32::NEG_INFINITY,
                ..valid
            },
            NuxFontPicture {
                extent_y: f32::NAN,
                ..valid
            },
        ] {
            assert!(collect(&invalid, false).is_none());
        }
        let mut maximum_png = vec![0; FONT_PICTURE_MAX_PNG_BYTES];
        maximum_png[..PICTURE_PNG.len()].copy_from_slice(PICTURE_PNG);
        maximum_png[16..20].copy_from_slice(&1024u32.to_be_bytes());
        maximum_png[20..24].copy_from_slice(&1024u32.to_be_bytes());
        let maximum = NuxFontPicture {
            width: 1024,
            height: 1024,
            ..picture_value(&maximum_png)
        };
        assert_eq!(
            collect(&maximum, false).unwrap().png.len(),
            FONT_PICTURE_MAX_PNG_BYTES
        );
        for encoded in [&b"not a PNG"[..], &PICTURE_PNG[..20]] {
            assert!(collect(&picture_value(encoded), false).is_none());
        }
        let mut wrong_dimensions = PICTURE_PNG.to_vec();
        wrong_dimensions[16..20].copy_from_slice(&1025u32.to_be_bytes());
        assert!(collect(&picture_value(&wrong_dimensions), false).is_none());
        assert!(collect(&NuxFontPicture { width: 2, ..valid }, false).is_none());
        let mut builder = PictureBuilder::default();
        let destination = ptr::from_mut(&mut builder).cast();
        unsafe {
            append_picture(destination, ptr::null());
            append_picture(destination, &valid);
        }
        assert!(builder.picture.is_none());
    }

    #[test]
    fn old_callback_prefix_is_not_read_past_release_and_never_calls_picture() {
        #[repr(C)]
        struct OldCallbacks {
            struct_size: u32,
            user_data: *mut c_void,
            get_path: Option<
                unsafe extern "C" fn(
                    *mut c_void,
                    u16,
                    *const NuxFontAxisCoord,
                    usize,
                    NuxFontOutlineSink,
                    *mut c_void,
                ),
            >,
            release: Option<unsafe extern "C" fn(*mut c_void)>,
        }
        let bytes = include_bytes!("../../../fixtures/fonts/roboto-input.ttf");
        let source = NuxFontSource {
            bytes: NuxByteView {
                data: bytes.as_ptr(),
                len: bytes.len(),
            },
            ..Default::default()
        };
        let callbacks = OldCallbacks {
            struct_size: std::mem::size_of::<OldCallbacks>() as u32,
            user_data: ptr::null_mut(),
            get_path: Some(outline),
            release: None,
        };
        assert_eq!(
            std::mem::size_of::<OldCallbacks>(),
            FONT_OUTLINE_CALLBACKS_V4_MIN_SIZE
        );
        let decoded =
            unsafe { read_font_outline_callbacks(ptr::from_ref(&callbacks).cast()) }.unwrap();
        assert!(decoded.get_picture.is_none());
        let mut handle = ptr::null_mut();
        assert_eq!(
            unsafe {
                nux_font_new_with_outline(&source, ptr::from_ref(&callbacks).cast(), &mut handle)
            },
            NuxStatus::Ok
        );
        let font = clone_font(handle).unwrap();
        assert!(!font.is_color_glyph_with_text(glyph(&font, 'A'), &[0x41]));
        assert_eq!(unsafe { nux_font_free(handle) }, NuxStatus::Ok);
        // Even a physically present callback is ignored unless the size covers it.
        let mut context = PictureContext::default();
        let callbacks = NuxFontOutlineCallbacks {
            struct_size: FONT_OUTLINE_CALLBACKS_V4_MIN_SIZE as u32,
            user_data: ptr::from_mut(&mut context).cast(),
            get_path: Some(outline),
            release: None,
            get_picture: Some(picture_callback),
        };
        assert_eq!(
            unsafe { nux_font_new_with_outline(&source, &callbacks, &mut handle) },
            NuxStatus::Ok
        );
        let font = clone_font(handle).unwrap();
        assert!(!font.is_color_glyph_with_text(glyph(&font, 'A'), &[0x41]));
        assert_eq!(context.calls.load(Ordering::SeqCst), 0);
        assert_eq!(unsafe { nux_font_free(handle) }, NuxStatus::Ok);
    }

    #[test]
    fn host_outline_recovers_only_empty_paths_and_retains_context_across_threads() {
        let (handle, calls, releases) = create(false);
        let font = clone_font(handle).unwrap();
        let original =
            HbFont::decode(include_bytes!("../../../fixtures/fonts/roboto-input.ttf")).unwrap();
        let a = glyph(&font, 'A');
        let space = glyph(&font, ' ');
        assert!(!original.get_path(a).empty());
        assert!(original.get_path(space).empty());
        assert_eq!(font.get_path(a).points(), original.get_path(a).points());
        assert_eq!(font.get_path(a).verbs(), original.get_path(a).verbs());
        assert_eq!(calls.load(Ordering::SeqCst), 0);
        let path = font.get_path(space);
        assert_eq!(path.points().len(), 5); // move, line, converted quadratic
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert_eq!(font.line_metrics().ascent, original.line_metrics().ascent);
        let variant = font.with_options(&[], &[]);
        assert!(
            variant.get_path(space).empty(),
            "upstream variants are plain HB fonts"
        );
        assert_eq!(unsafe { nux_font_free(handle) }, NuxStatus::Ok);
        let retained = font.clone();
        drop(font);
        assert_eq!(releases.load(Ordering::SeqCst), 0);
        std::thread::spawn(move || assert!(!retained.get_path(space).empty()))
            .join()
            .unwrap();
        assert_eq!(releases.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn host_outline_receives_source_coordinates() {
        unsafe extern "C" fn capture(
            context: *mut c_void,
            _: u16,
            coords: *const NuxFontAxisCoord,
            count: usize,
            _: NuxFontOutlineSink,
            _: *mut c_void,
        ) {
            let observed = unsafe { &*context.cast::<std::sync::Mutex<Vec<(u32, f32)>>>() };
            observed.lock().unwrap().extend(
                unsafe { slice::from_raw_parts(coords, count) }
                    .iter()
                    .map(|c| (c.axis, c.value)),
            );
        }
        let bytes = include_bytes!("../../../fixtures/fonts/roboto-input.ttf");
        let coordinates = [NuxFontAxisCoord {
            axis: u32::from_be_bytes(*b"wght"),
            value: 650.0,
        }];
        let source = NuxFontSource {
            bytes: NuxByteView {
                data: bytes.as_ptr(),
                len: bytes.len(),
            },
            coords: coordinates.as_ptr(),
            coord_count: coordinates.len(),
            ..Default::default()
        };
        let mut observed = std::sync::Mutex::new(Vec::<(u32, f32)>::new());
        let callbacks = NuxFontOutlineCallbacks {
            struct_size: std::mem::size_of::<NuxFontOutlineCallbacks>() as u32,
            user_data: ptr::from_mut(&mut observed).cast(),
            get_path: Some(capture),
            release: None,
            get_picture: None,
        };
        let mut handle = ptr::null_mut();
        assert_eq!(
            unsafe { nux_font_new_with_outline(&source, &callbacks, &mut handle) },
            NuxStatus::Ok
        );
        let font = clone_font(handle).unwrap();
        font.get_path(glyph(&font, ' '));
        assert_eq!(
            *observed.lock().unwrap(),
            [(u32::from_be_bytes(*b"wght"), 650.0)]
        );
        assert_eq!(unsafe { nux_font_free(handle) }, NuxStatus::Ok);
    }

    #[test]
    fn font_inputs_accept_larger_callers_and_reject_short_prefixes() {
        #[repr(C)]
        struct Extended<T> {
            current: T,
            future: [u64; 2],
        }
        let bytes = include_bytes!("../../../fixtures/fonts/roboto-input.ttf");
        let source = Extended {
            current: NuxFontSource {
                struct_size: std::mem::size_of::<Extended<NuxFontSource>>() as u32,
                bytes: NuxByteView {
                    data: bytes.as_ptr(),
                    len: bytes.len(),
                },
                ..Default::default()
            },
            future: [u64::MAX; 2],
        };
        let mut observed = std::sync::Mutex::new(Vec::<(u32, f32)>::new());
        unsafe extern "C" fn empty(
            _: *mut c_void,
            _: u16,
            _: *const NuxFontAxisCoord,
            _: usize,
            _: NuxFontOutlineSink,
            _: *mut c_void,
        ) {
        }
        let callbacks = Extended {
            current: NuxFontOutlineCallbacks {
                struct_size: std::mem::size_of::<Extended<NuxFontOutlineCallbacks>>() as u32,
                user_data: ptr::from_mut(&mut observed).cast(),
                get_path: Some(empty),
                release: None,
                get_picture: None,
            },
            future: [u64::MAX; 2],
        };
        let mut handle = ptr::null_mut();
        assert_eq!(
            unsafe { nux_font_new(&source.current, &mut handle) },
            NuxStatus::Ok
        );
        assert_eq!(unsafe { nux_font_free(handle) }, NuxStatus::Ok);
        assert_eq!(
            unsafe { nux_font_new_with_outline(&source.current, &callbacks.current, &mut handle) },
            NuxStatus::Ok
        );
        assert_eq!(unsafe { nux_font_free(handle) }, NuxStatus::Ok);
        let short = std::mem::size_of::<u32>() as u32;
        assert_eq!(
            unsafe { nux_font_new(ptr::from_ref(&short).cast(), &mut handle) },
            NuxStatus::InvalidStructSize
        );
        assert!(handle.is_null());
        assert_eq!(
            unsafe {
                nux_font_new_with_outline(
                    &source.current,
                    ptr::from_ref(&short).cast(),
                    &mut handle,
                )
            },
            NuxStatus::InvalidStructSize
        );
        assert!(handle.is_null());
        assert_eq!(source.future, [u64::MAX; 2]);
        assert_eq!(callbacks.future, [u64::MAX; 2]);
    }

    #[test]
    fn malformed_host_outline_is_discarded() {
        let (handle, _, releases) = create(true);
        let font = clone_font(handle).unwrap();
        assert!(font.get_path(glyph(&font, ' ')).empty());
        assert_eq!(unsafe { nux_font_free(handle) }, NuxStatus::Ok);
        drop(font);
        assert_eq!(releases.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn failed_creation_keeps_context_with_caller() {
        let calls = Arc::new(AtomicUsize::new(0));
        let releases = Arc::new(AtomicUsize::new(0));
        let context = Box::into_raw(Box::new(Context {
            calls,
            releases: releases.clone(),
            invalid: false,
        }));
        let callbacks = NuxFontOutlineCallbacks {
            struct_size: std::mem::size_of::<NuxFontOutlineCallbacks>() as u32,
            user_data: context.cast(),
            get_path: Some(outline),
            release: Some(release),
            get_picture: None,
        };
        let mut font = ptr::null_mut();
        assert_eq!(
            unsafe { nux_font_new_with_outline(ptr::null(), &callbacks, &mut font) },
            NuxStatus::NullArgument
        );
        assert!(font.is_null());
        assert_eq!(releases.load(Ordering::SeqCst), 0);
        unsafe {
            release(context.cast());
        }
        assert_eq!(releases.load(Ordering::SeqCst), 1);
    }
}
