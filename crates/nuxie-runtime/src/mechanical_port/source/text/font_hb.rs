use std::{
    collections::{HashMap, HashSet, VecDeque},
    sync::{
        Arc, Mutex, OnceLock,
        atomic::{AtomicU8, Ordering},
    },
};

use harfrust::{
    Direction, Feature as ShapingFeature, FontRef as ShapingFont, ShapeOptions, ShaperData,
    ShaperInstance, Tag, UnicodeBuffer,
};
use skrifa::instance::{Location, LocationRef, Size};
use skrifa::outline::{DrawSettings, OutlinePen};
use skrifa::raw::TableProvider;
use skrifa::raw::tables::cmap::CmapSubtable;
use skrifa::setting::VariationSetting;
use skrifa::{FontRef as OutlineFont, GlyphId as OutlineGlyphId, MetadataProvider};
use unicode_properties::{GeneralCategory, UnicodeGeneralCategory};
use unicode_script::UnicodeScript;

use crate::mechanical_port::source::math::{mat2d::Mat2D, raw_path::RawPath, vec2d::Vec2D};
use crate::mechanical_port::source::shapes::paint::color::ColorInt;
pub use crate::mechanical_port::source::text_engine::HostPictureScope;
use crate::mechanical_port::source::text_engine::{
    Axis, ColorGlyphLayer, ColorGlyphPaintType, Coord, Feature, Font, FontBase, FontRef, GlyphId,
    GlyphRun, GradientStop, LineMetrics, Paragraph, TextRun, Unichar, fallback_proc,
    fallback_proc_enabled, note_fallback_call,
};

// Test-only observation of real shaping, scoped to each test thread.
#[cfg(test)]
thread_local! {
    static TEST_SHAPE_CALLS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}
#[cfg(test)]
impl HbFont {
    pub(crate) fn test_shape_count() -> usize {
        TEST_SHAPE_CALLS.with(std::cell::Cell::get)
    }
}

const STANDARD_SCALE: i32 = 2048;
const INVERSE_SCALE: f32 = 1.0 / STANDARD_SCALE as f32;
const KERN_TAG: u32 = u32::from_be_bytes(*b"kern");

// The nominal query and coverage enumeration must use the same selected
// subtable, not the union of mappings. Select the encoding record before
// parsing its format: an unsupported preferred table must not fall through
// to a lower-priority Unicode table.
fn nominal_cmap<'a>(font: &OutlineFont<'a>) -> Option<CmapSubtable<'a>> {
    let cmap = font.cmap().ok()?;
    let records = cmap.encoding_records();
    for (platform, encoding) in [
        (3, 0),
        (3, 10),
        (0, 6),
        (0, 4),
        (3, 1),
        (0, 3),
        (0, 2),
        (0, 1),
        (0, 0),
        (1, 0),
        (1, 0xffff),
    ] {
        let mut low = 0;
        let mut high = records.len();
        while low < high {
            let mid = (low + high - 1) / 2;
            let record = &records[mid];
            let order = (record.platform_id() as u16).cmp(&platform).then_with(|| {
                if encoding == 0xffff {
                    std::cmp::Ordering::Equal
                } else {
                    record.encoding_id().cmp(&encoding)
                }
            });
            match order {
                std::cmp::Ordering::Less => low = mid + 1,
                std::cmp::Ordering::Greater => high = mid,
                std::cmp::Ordering::Equal => {
                    if record.subtable_offset().is_null() {
                        break;
                    }
                    let table = record.subtable(cmap.offset_data()).ok()?;
                    if let CmapSubtable::Format4(table) = &table {
                        // HarfBuzz sanitizes the four complete segment arrays
                        // before exposing this cmap. read-fonts validates only
                        // the fixed header on construction, so reproduce the
                        // source's `values, 2 + 4 * segCountX2` range check here.
                        if table.offset_data().len() < 16 + 4 * usize::from(table.seg_count_x2()) {
                            return None;
                        }
                    }
                    return Some(table);
                }
            }
        }
    }
    None
}

fn cmap_has_glyph(table: &CmapSubtable<'_>, cp: u32) -> bool {
    // The pinned HB_NO_LEGACY configuration disables both symbol aliases and
    // Macintosh remapping. Query the selected table's raw nominal mapping.
    if let CmapSubtable::Format10(table) = table {
        return table
            .glyph_id_array()
            .get(cp.wrapping_sub(table.start_char_code()) as usize)
            .is_some_and(|glyph| glyph.get() != 0);
    }
    table
        .map_codepoint(cp)
        .is_some_and(|glyph| glyph.to_u32() != 0)
}

fn nominal_has_glyph(font: &OutlineFont<'_>, cp: u32) -> bool {
    nominal_cmap(font).is_some_and(|table| cmap_has_glyph(&table, cp))
}

fn nominal_codepoints(font: &OutlineFont<'_>) -> Vec<u32> {
    let Some(table) = nominal_cmap(font) else {
        return Vec::new();
    };
    // Like hb_set_t, union during collection: repeated or overlapping cmap
    // ranges must not allocate one copy per encoded group before deduplication.
    let mut coverage = std::collections::BTreeSet::new();
    match &table {
        CmapSubtable::Format4(table) => {
            let seg_count = table.start_code().len();
            let count = seg_count
                - usize::from(
                    table
                        .start_code()
                        .last()
                        .is_some_and(|start| start.get() == 0xffff),
                );
            let mut points = std::collections::BTreeSet::new();
            // Skip the sentinel segment, not the U+FFFF codepoint: an earlier
            // overlapping segment may legitimately contribute it. Preserve
            // HarfBuzz's per-segment insertion followed by missing-glyph removal.
            for i in 0..count {
                let start = u32::from(table.start_code()[i].get());
                let end = u32::from(table.end_code()[i].get());
                let range_offset = usize::from(table.id_range_offsets()[i].get());
                points.extend(start..=end);
                for cp in start..=end {
                    let glyph = if range_offset == 0 {
                        Some((cp as u16).wrapping_add(table.id_delta()[i].get() as u16))
                    } else {
                        (range_offset / 2 + (cp - start) as usize + i)
                            .checked_sub(seg_count)
                            .and_then(|index| table.glyph_id_array().get(index))
                            .map(|glyph| glyph.get())
                    };
                    let Some(glyph) = glyph else {
                        for missing in cp..=end {
                            points.remove(&missing);
                        }
                        break;
                    };
                    if glyph == 0 {
                        points.remove(&cp);
                    }
                }
            }
            coverage.extend(points);
        }
        CmapSubtable::Format0(table) => {
            coverage.extend(
                table
                    .glyph_id_array()
                    .iter()
                    .enumerate()
                    .filter_map(|(cp, &gid)| (gid != 0).then_some(cp as u32)),
            );
        }
        CmapSubtable::Format10(table) => {
            coverage.extend(
                table
                    .glyph_id_array()
                    .iter()
                    .enumerate()
                    .filter_map(|(i, glyph)| {
                        (glyph.get() != 0).then_some(table.start_char_code().wrapping_add(i as u32))
                    }),
            );
        }
        CmapSubtable::Format12(_) | CmapSubtable::Format13(_) => {
            let num_glyphs = font.maxp().map_or(0, |maxp| u32::from(maxp.num_glyphs()));
            // Match hb_face_collect_unicodes' format12/13 clipping exactly,
            // including its inclusive end. Library iterators use a different
            // (exclusive) glyph-count limit and therefore cannot supply this.
            let mut collect = |mut start: u32, mut end: u32, mut gid: u32, constant: bool| {
                end = end.min(0x10ffff);
                if gid == 0 {
                    if constant || start > end || end == start {
                        return;
                    }
                    start = start.wrapping_add(1);
                    gid = gid.wrapping_add(1);
                }
                if gid >= num_glyphs {
                    return;
                }
                if gid.wrapping_add(end).wrapping_sub(start) >= num_glyphs {
                    end = start.wrapping_add(num_glyphs).wrapping_sub(gid);
                }
                coverage.extend(start..=end.min(0x10ffff));
            };
            match &table {
                CmapSubtable::Format12(table) => {
                    for group in table.groups() {
                        collect(
                            group.start_char_code(),
                            group.end_char_code(),
                            group.start_glyph_id(),
                            false,
                        );
                    }
                }
                CmapSubtable::Format13(table) => {
                    for group in table.groups() {
                        collect(
                            group.start_char_code(),
                            group.end_char_code(),
                            group.glyph_id(),
                            true,
                        );
                    }
                }
                _ => unreachable!(),
            }
        }
        _ => coverage.extend(
            table
                .iter()
                .filter_map(|(cp, gid)| (gid.to_u32() != 0).then_some(cp)),
        ),
    }
    coverage.retain(|&cp| char::from_u32(cp).is_some() && cmap_has_glyph(&table, cp));
    coverage.into_iter().collect()
}

#[derive(Clone)]
enum FontBytes {
    Owned(Arc<[u8]>),
    Adopted(Arc<Vec<u8>>),
    #[cfg(any(target_vendor = "apple", target_os = "linux", target_os = "android"))]
    Mapped(Arc<FontFileMapping>),
}

impl std::ops::Deref for FontBytes {
    type Target = [u8];
    fn deref(&self) -> &[u8] {
        match self {
            Self::Owned(bytes) => bytes,
            Self::Adopted(bytes) => bytes,
            #[cfg(any(target_vendor = "apple", target_os = "linux", target_os = "android"))]
            Self::Mapped(mapping) => {
                // decode_file's caller guarantees immutable, live file pages;
                // the Arc retains the mapping for this entire borrow.
                unsafe { std::slice::from_raw_parts(mapping.data.cast(), mapping.size) }
            }
        }
    }
}

#[cfg(any(target_vendor = "apple", target_os = "linux", target_os = "android"))]
struct FontFileMapping {
    data: *mut libc::c_void,
    size: usize,
}

// The mapping is read-only and decode_file requires its backing bytes to stay
// unchanged. Shared ownership prevents unmapping while another thread reads.
#[cfg(any(target_vendor = "apple", target_os = "linux", target_os = "android"))]
unsafe impl Send for FontFileMapping {}
#[cfg(any(target_vendor = "apple", target_os = "linux", target_os = "android"))]
unsafe impl Sync for FontFileMapping {}

#[cfg(any(target_vendor = "apple", target_os = "linux", target_os = "android"))]
impl Drop for FontFileMapping {
    fn drop(&mut self) {
        unsafe {
            libc::munmap(self.data, self.size);
        }
    }
}

fn suppress_legacy_kern(font: &OutlineFont<'_>) -> bool {
    font.kern().is_ok() && font.gpos().is_err() && font.kerx().is_err()
}

fn shaping_features(
    feature_values: &HashMap<u32, u32>,
    suppress_legacy_kern: bool,
) -> Vec<ShapingFeature> {
    let kern_tag = Tag::from_u32(KERN_TAG);
    let mut features: Vec<_> = feature_values
        .iter()
        .map(|(&tag, &value)| ShapingFeature::new(Tag::from_u32(tag), value, ..))
        .collect();
    if suppress_legacy_kern {
        if let Some(kern) = features.iter_mut().find(|feature| feature.tag == kern_tag) {
            kern.value = 0;
        } else {
            features.push(ShapingFeature::new(kern_tag, 0, ..));
        }
    }
    features
}

/// Host outline recovery for an otherwise ordinary decoded font.
pub type HostGlyphOutline = Arc<dyn Fn(GlyphId, &[Coord]) -> RawPath + Send + Sync>;

/// Host-rasterized PNG in em units, using ColorGlyphLayer's y-down bearings.
/// `bearing_y` is minus the top edge's height above the baseline.
#[derive(Clone, Debug)]
pub struct HostGlyphPicture {
    pub png: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub bearing_x: f32,
    pub bearing_y: f32,
    pub extent_x: f32,
    pub extent_y: f32,
}

pub type HostGlyphPictures =
    Arc<dyn Fn(GlyphId, &[Unichar]) -> Option<HostGlyphPicture> + Send + Sync>;
type PictureAnswer = Arc<OnceLock<Option<Arc<ColorGlyphLayer>>>>;

const HOST_PICTURE_CACHE_CAPACITY: usize = 4096;

/// FIFO eviction bounds both positive and absent answers. Borrowed slice
/// lookup allocates only on a miss; the queue shares the map's key storage.
#[derive(Default)]
struct HostPictureCache {
    answers: HashMap<GlyphId, HashMap<Arc<[Unichar]>, PictureAnswer>>,
    order: VecDeque<(GlyphId, Arc<[Unichar]>)>,
}
impl HostPictureCache {
    fn answer(&mut self, glyph: GlyphId, cluster: &[Unichar]) -> PictureAnswer {
        if let Some(answer) = self
            .answers
            .get(&glyph)
            .and_then(|answers| answers.get(cluster))
        {
            return answer.clone();
        }
        if self.order.len() == HOST_PICTURE_CACHE_CAPACITY {
            let (old_glyph, old_cluster) = self.order.pop_front().unwrap();
            let answers = self.answers.get_mut(&old_glyph).unwrap();
            answers.remove(old_cluster.as_ref());
            if answers.is_empty() {
                self.answers.remove(&old_glyph);
            }
        }
        let cluster: Arc<[Unichar]> = cluster.into();
        let answer = Arc::new(OnceLock::new());
        self.answers
            .entry(glyph)
            .or_default()
            .insert(cluster.clone(), answer.clone());
        self.order.push_back((glyph, cluster));
        answer
    }
}

/// The HBFont owner retains Rive's run/fallback/color semantics. Only its
/// backend is adapted: harfrust shapes, skrifa reads outlines and color tables,
/// and unicode-bidi supplies paragraph levels. No C font or bidi pointer lives
/// in this owner, and no legacy packed text implementation is consulted.
pub struct HbFont {
    base: FontBase,
    host_outline: Option<HostGlyphOutline>,
    host_pictures: Option<HostGlyphPictures>,
    host_picture_scope: HostPictureScope,
    host_picture_cache: Mutex<HostPictureCache>,
    bytes: FontBytes,
    face_index: u32,
    // The face's shaping caches and shape plans, shared by every variant
    // with_options derives from this face, as HarfBuzz keeps them on the face.
    face: Arc<HbFace>,
    shaper_instance: ShaperInstance,
    features: Vec<ShapingFeature>,
    feature_values: HashMap<u32, u32>,
    axis_values: HashMap<u32, f32>,
    color_layer_cache: Mutex<HashMap<GlyphId, Vec<ColorGlyphLayer>>>,
}

/// What a font face's variants share: its shaper data, the shape plans built
/// for it (a plan is a function of its segment properties, variation
/// instance and features), and the facts read from its tables alone.
struct HbFace {
    shaper_data: ShaperData,
    shape_plans: Mutex<Vec<harfrust::ShapePlan>>,
    has_color_layers: bool,
    has_color_paint: bool,
    has_png: bool,
    // 0 unknown, 1 outline, 2 colour. Table classification is immutable and
    // shared by every variant; relaxed atomics avoid a hot-path face mutex.
    native_color_glyphs: Box<[AtomicU8]>,
    palette_colors: Vec<skrifa::color::Color>,
    suppress_legacy_kern: bool,
}

/// A mapped face for nominal coverage, without font metrics or drawing setup.
pub struct FileProbe {
    bytes: Option<FontBytes>,
    face_index: u32,
}

impl FileProbe {
    pub fn has_glyph(&self, codepoint: Unichar) -> bool {
        self.bytes.as_ref().is_some_and(|bytes| {
            nominal_has_glyph(
                &OutlineFont::from_index(bytes, self.face_index)
                    .expect("probe retains its validated face"),
                codepoint,
            )
        })
    }

    pub fn make_font(&mut self) -> Option<FontRef> {
        let bytes = self.bytes.take()?;
        Some(Arc::new(HbFont::with_stored_options(
            bytes,
            self.face_index,
            HashMap::new(),
            HashMap::new(),
        )))
    }
}

impl HbFont {
    pub fn source_bytes(&self) -> Arc<[u8]> {
        match &self.bytes {
            FontBytes::Owned(bytes) => bytes.clone(),
            FontBytes::Adopted(bytes) => Arc::from(bytes.as_slice()),
            #[cfg(any(target_vendor = "apple", target_os = "linux", target_os = "android"))]
            FontBytes::Mapped(_) => Arc::from(&*self.bytes),
        }
    }
    pub fn face_index(&self) -> u32 {
        self.face_index
    }
    /// Borrow the retained allocation, without materializing a byte snapshot.
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn decode_owned(bytes: Vec<u8>) -> Option<FontRef> {
        Self::from_decoded_font(nuxie_render_api::decode_font_owned_bytes(bytes).ok()?)
    }

    pub fn from_decoded_font(decoded: nuxie_render_api::DecodedFont) -> Option<FontRef> {
        let bytes = decoded.into_bytes();
        ShapingFont::from_index(&bytes, 0).ok()?;
        OutlineFont::from_index(&bytes, 0).ok()?;
        Some(Arc::new(Self::with_stored_options(
            FontBytes::Adopted(bytes),
            0,
            HashMap::new(),
            HashMap::new(),
        )))
    }
    pub fn decode(bytes: &[u8]) -> Option<FontRef> {
        Self::decode_face(bytes, 0)
    }

    pub fn decode_face(bytes: &[u8], face_index: u32) -> Option<FontRef> {
        let shaping = ShapingFont::from_index(bytes, face_index).ok()?;
        let outline = OutlineFont::from_index(bytes, face_index).ok()?;
        let _ = (shaping, outline);
        Some(Arc::new(Self::with_stored_options(
            FontBytes::Owned(Arc::from(bytes)),
            face_index,
            HashMap::new(),
            HashMap::new(),
        )))
    }

    /// Maps a trusted, stable local font file without copying its font bytes.
    /// Unsupported platforms and failures detected during this call return
    /// `None`; callers may separately choose the ordinary byte decode path.
    ///
    /// # Safety
    /// The file's bytes must remain unchanged and its backing storage available
    /// for the entire lifetime of this font and every font derived from it
    /// (including through `with_options`). Truncation or vanished storage can
    /// cause SIGBUS on later access. Do not use untrusted paths, removable media,
    /// or network volumes. Mapping ownership alone cannot enforce this contract.
    pub unsafe fn decode_file(path: Option<&std::path::Path>, face_index: u32) -> Option<FontRef> {
        // SAFETY: the caller supplies the same stable-file contract as probe_file.
        unsafe { Self::probe_file(path, face_index) }?.make_font()
    }

    /// Map a font face for nominal coverage without full font initialization.
    ///
    /// # Safety
    /// The trusted file must remain unchanged and available until this probe
    /// and every font promoted or derived from it have been dropped.
    pub unsafe fn probe_file(path: Option<&std::path::Path>, face_index: u32) -> Option<FileProbe> {
        #[cfg(any(target_vendor = "apple", target_os = "linux", target_os = "android"))]
        {
            use std::os::fd::AsRawFd;
            let file = std::fs::File::open(path?).ok()?;
            let size = file.metadata().ok()?.len();
            if size == 0 || size >= (1u64 << 31) {
                return None;
            }
            let size = size as usize;
            let data = unsafe {
                libc::mmap(
                    std::ptr::null_mut(),
                    size,
                    libc::PROT_READ,
                    libc::MAP_PRIVATE,
                    file.as_raw_fd(),
                    0,
                )
            };
            // The mapping, not the descriptor, now owns the file reference.
            drop(file);
            if data == libc::MAP_FAILED {
                return None;
            }
            let mapping = Arc::new(FontFileMapping { data, size });
            // Rust slices cannot represent an address-zero mapping.
            if data.is_null() {
                return None;
            }
            let bytes = FontBytes::Mapped(mapping);
            // Preserve the approved harfrust/skrifa validation boundary rather
            // than introducing a HarfBuzz face or a different shaping backend.
            ShapingFont::from_index(&bytes, face_index).ok()?;
            OutlineFont::from_index(&bytes, face_index).ok()?;
            Some(FileProbe {
                bytes: Some(bytes),
                face_index,
            })
        }
        #[cfg(not(any(target_vendor = "apple", target_os = "linux", target_os = "android")))]
        {
            let _ = (path, face_index);
            None
        }
    }

    /// The public host font retains this exact native occurrence.
    pub fn from_raw_text(font: &crate::text::RawTextFont) -> Option<FontRef> {
        Some(font.native_handle())
    }

    /// System fonts cross the approved Rust host boundary as owned bytes plus
    /// a face index; the runtime does not take a platform font pointer.
    pub fn from_system_bytes(bytes: &[u8], face_index: u32) -> Option<FontRef> {
        Self::decode_face(bytes, face_index)
    }

    fn outline_font(&self) -> OutlineFont<'_> {
        OutlineFont::from_index(&self.bytes, self.face_index)
            .expect("HBFont retains the face validated at decode")
    }

    fn location(&self, font: &OutlineFont<'_>) -> Location {
        font.axes().location(
            self.axis_values
                .iter()
                .map(|(&tag, &value)| VariationSetting::new(skrifa::Tag::from_u32(tag), value)),
        )
    }

    fn with_stored_options(
        bytes: FontBytes,
        face_index: u32,
        axis_values: HashMap<u32, f32>,
        feature_values: HashMap<u32, u32>,
    ) -> Self {
        Self::with_face_options(bytes, face_index, None, axis_values, feature_values)
    }

    /// A font of this face at these options, reusing `face` when given (a
    /// variant of a font already built for the same bytes and index).
    fn with_face_options(
        bytes: FontBytes,
        face_index: u32,
        face: Option<Arc<HbFace>>,
        axis_values: HashMap<u32, f32>,
        feature_values: HashMap<u32, u32>,
    ) -> Self {
        let shaping = ShapingFont::from_index(&bytes, face_index).expect("validated shaping face");
        let outline = OutlineFont::from_index(&bytes, face_index).expect("validated outline face");
        let face = face.unwrap_or_else(|| {
            let has_color_layers = outline.colr().is_ok();
            let has_color_paint = outline.colr().is_ok_and(|colr| colr.version() >= 1);
            let has_png = font_has_png_data(&outline);
            let palette_colors = if has_color_layers || has_color_paint {
                outline
                    .color_palettes()
                    .get(0)
                    .map(|palette| palette.colors().to_vec())
                    .unwrap_or_default()
            } else {
                Vec::new()
            };
            Arc::new(HbFace {
                shaper_data: ShaperData::new(&shaping),
                shape_plans: Mutex::new(Vec::new()),
                has_color_layers,
                has_color_paint,
                has_png,
                native_color_glyphs: std::iter::repeat_with(|| AtomicU8::new(0))
                    .take(usize::from(u16::MAX) + 1)
                    .collect(),
                palette_colors,
                // Pinned Rive HarfBuzz is compiled with HB_NO_LEGACY. HarfRust
                // has no equivalent build switch, so suppress only its legacy
                // `kern` fallback. A GPOS or kerx font keeps its normal
                // positioning path.
                suppress_legacy_kern: suppress_legacy_kern(&outline),
            })
        });
        let shaper_instance = ShaperInstance::from_variations(
            &shaping,
            axis_values
                .iter()
                .map(|(&tag, &value)| harfrust::Variation {
                    tag: Tag::from_u32(tag),
                    value,
                }),
        );
        let location = outline.axes().location(
            axis_values
                .iter()
                .map(|(&tag, &value)| VariationSetting::new(skrifa::Tag::from_u32(tag), value)),
        );
        let line_metrics = make_line_metrics(&outline, LocationRef::from(&location));
        let features = shaping_features(&feature_values, face.suppress_legacy_kern);
        Self {
            base: FontBase::new(line_metrics),
            host_outline: None,
            host_pictures: None,
            host_picture_scope: HostPictureScope::Glyph,
            host_picture_cache: Mutex::new(HostPictureCache::default()),
            bytes,
            face_index,
            face,
            shaper_instance,
            features,
            feature_values,
            axis_values,
            color_layer_cache: Mutex::new(HashMap::new()),
        }
    }

    /// Host equivalent of CoreTextHBFont::getPath's empty-HB recovery.
    /// Ordinary byte/file fonts never install this provider.
    pub fn with_host_outline(&self, outline: HostGlyphOutline) -> FontRef {
        // A variant of this face: it shares the face's shaping caches, as
        // with_options variants do.
        let mut font = Self::with_face_options(
            self.bytes.clone(),
            self.face_index,
            Some(Arc::clone(&self.face)),
            self.axis_values.clone(),
            self.feature_values.clone(),
        );
        font.host_outline = Some(outline);
        font.host_pictures = self.host_pictures.clone();
        font.host_picture_scope = self.host_picture_scope;
        Arc::new(font)
    }

    /// Install an additive picture source. Native COLR/readable PNG always wins.
    /// Glyph scope caches by glyph; Cluster scope caches by (glyph, full text)
    /// and replaces this font's entire cluster. Each variant has its own FIFO
    /// cache of up to 4096 answers, including None; eviction permits requery.
    /// Glyph zero is never asked. Glyph hosts ignore no-ink glyphs themselves.
    /// Callbacks can run concurrently for distinct keys and must not reenter.
    pub fn with_host_pictures(
        &self,
        pictures: HostGlyphPictures,
        scope: HostPictureScope,
    ) -> FontRef {
        let mut font = Self::with_face_options(
            self.bytes.clone(),
            self.face_index,
            Some(Arc::clone(&self.face)),
            self.axis_values.clone(),
            self.feature_values.clone(),
        );
        font.host_outline = self.host_outline.clone();
        font.host_pictures = Some(pictures);
        font.host_picture_scope = scope;
        Arc::new(font)
    }

    fn native_color_glyph(&self, glyph: GlyphId) -> bool {
        if !(self.face.has_color_layers || self.face.has_color_paint || self.face.has_png) {
            return false;
        }
        let cached = &self.face.native_color_glyphs[usize::from(glyph)];
        match cached.load(Ordering::Relaxed) {
            1 => return false,
            2 => return true,
            _ => {}
        }
        let font = self.outline_font();
        let id = OutlineGlyphId::new(glyph as u32);
        let color = font.color_glyphs().get(id).is_some() || native_png_exists(&font, glyph);
        cached.store(if color { 2 } else { 1 }, Ordering::Relaxed);
        color
    }

    pub fn shape_fallback_run(
        &self,
        glyph_runs: &mut Vec<GlyphRun>,
        text: &[Unichar],
        text_start: u32,
        text_run: &TextRun,
        _original_text_run: &TextRun,
        fallback_index: u32,
    ) {
        let glyph_run = shape_run(&text[text_start as usize..], text_run, text_start);
        if let Some(index) = glyph_run.glyphs.iter().position(|glyph| *glyph == 0) {
            let missing = text[glyph_run.text_indices[index] as usize];
            let fallback = fallback_proc().and_then(|callback| {
                note_fallback_call();
                callback(missing, fallback_index, self)
            });
            if let Some(fallback_font) = fallback {
                let fallback_hb = fallback_font
                    .as_any()
                    .downcast_ref::<HbFont>()
                    .expect("fallback font must be an HBFont");
                if !std::ptr::eq(fallback_hb, self) {
                    perform_fallback(
                        fallback_font,
                        glyph_runs,
                        text,
                        &glyph_run,
                        // Match HBFont::shapeFallbackRun: retries are bounded by
                        // this fallback subrun, not the original paragraph run.
                        text_run,
                        fallback_index + 1,
                    );
                } else if !glyph_run.glyphs.is_empty() {
                    glyph_runs.push(glyph_run);
                }
            } else if !glyph_run.glyphs.is_empty() {
                glyph_runs.push(glyph_run);
            }
        } else if !glyph_run.glyphs.is_empty() {
            glyph_runs.push(glyph_run);
        }
    }
}

impl Font for HbFont {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
    fn base(&self) -> &FontBase {
        &self.base
    }
    fn get_axis(&self, index: u16) -> Axis {
        let font = self.outline_font();
        let axis = font
            .axes()
            .get(index as usize)
            .expect("font axis index in range");
        Axis {
            tag: u32::from_be_bytes(axis.tag().to_be_bytes()),
            min: axis.min_value(),
            def: axis.default_value(),
            max: axis.max_value(),
        }
    }
    fn get_axis_count(&self) -> u16 {
        self.outline_font().axes().len() as u16
    }
    fn get_axis_value(&self, axis_tag: u32) -> f32 {
        self.axis_values.get(&axis_tag).copied().unwrap_or_else(|| {
            self.outline_font()
                .axes()
                .get_by_tag(skrifa::Tag::from_u32(axis_tag))
                .map_or(0.0, |axis| axis.default_value())
        })
    }
    fn get_feature_value(&self, feature_tag: u32) -> u32 {
        self.feature_values
            .get(&feature_tag)
            .copied()
            .unwrap_or(u32::MAX)
    }
    fn get_weight(&self) -> u16 {
        let font = self.outline_font();
        let tag = u32::from_be_bytes(*b"wght");
        if font.axes().get_by_tag(skrifa::Tag::from_u32(tag)).is_some() {
            self.get_axis_value(tag) as u16
        } else {
            font.attributes().weight.value() as u16
        }
    }
    fn is_italic(&self) -> bool {
        let font = self.outline_font();
        let tag = u32::from_be_bytes(*b"ital");
        if font.axes().get_by_tag(skrifa::Tag::from_u32(tag)).is_some() {
            self.get_axis_value(tag) != 0.0
        } else {
            !matches!(font.attributes().style, skrifa::attribute::Style::Normal)
        }
    }
    fn features(&self) -> Vec<u32> {
        let font = self.outline_font();
        let mut features = HashSet::new();
        if let Ok(table) = font.gsub() {
            if let (Ok(scripts), Ok(list)) = (table.script_list(), table.feature_list()) {
                fill_features(scripts, list, &mut features);
            }
        }
        if let Ok(table) = font.gpos() {
            if let (Ok(scripts), Ok(list)) = (table.script_list(), table.feature_list()) {
                fill_features(scripts, list, &mut features);
            }
        }
        features.into_iter().collect()
    }
    fn has_glyph(&self, value: Unichar) -> bool {
        nominal_has_glyph(&self.outline_font(), value)
    }
    fn codepoints(&self) -> Vec<Unichar> {
        nominal_codepoints(&self.outline_font())
    }
    fn with_options(&self, coords: &[Coord], features: &[Feature]) -> FontRef {
        let mut axes = self.axis_values.clone();
        for coord in coords {
            axes.insert(coord.axis, coord.value);
        }
        let mut values = self.feature_values.clone();
        for feature in features {
            values.insert(feature.tag, feature.value);
        }
        let mut font = Self::with_face_options(
            self.bytes.clone(),
            self.face_index,
            Some(Arc::clone(&self.face)),
            axes,
            values,
        );
        // Keep upstream's plain-HB outline variant policy. Pictures are our
        // opt-in host extension and survive derived shaping variants.
        font.host_pictures = self.host_pictures.clone();
        font.host_picture_scope = self.host_picture_scope;
        Arc::new(font)
    }
    fn get_path(&self, glyph: GlyphId) -> RawPath {
        let font = self.outline_font();
        let location = self.location(&font);
        let path = glyph_path(&font, LocationRef::from(&location), glyph);
        if path.empty() {
            if let Some(outline) = &self.host_outline {
                let coords: Vec<_> = self
                    .axis_values
                    .iter()
                    .map(|(&axis, &value)| Coord { axis, value })
                    .collect();
                return outline(glyph, &coords);
            }
        }
        path
    }
    fn has_color_glyphs(&self) -> bool {
        self.face.has_color_layers
            || self.face.has_color_paint
            || self.face.has_png
            || self.host_pictures.is_some()
    }
    fn is_color_glyph(&self, glyph: GlyphId) -> bool {
        self.native_color_glyph(glyph)
    }
    fn host_picture_scope(&self) -> Option<HostPictureScope> {
        self.host_pictures.as_ref().map(|_| self.host_picture_scope)
    }
    fn host_picture_layer(
        &self,
        glyph: GlyphId,
        cluster: &[Unichar],
    ) -> Option<Arc<ColorGlyphLayer>> {
        let source = self.host_pictures.as_ref()?;
        if glyph == 0 || self.native_color_glyph(glyph) {
            return None;
        }
        let key_text = match self.host_picture_scope {
            HostPictureScope::Glyph => &[][..],
            HostPictureScope::Cluster => cluster,
        };
        let answer = self
            .host_picture_cache
            .lock()
            .unwrap()
            .answer(glyph, key_text);
        answer
            .get_or_init(|| {
                source(glyph, cluster).map(|picture| {
                    Arc::new(ColorGlyphLayer {
                        paint_type: ColorGlyphPaintType::Image,
                        image_bytes: picture.png.into(),
                        image_width: picture.width,
                        image_height: picture.height,
                        image_bearing_x: picture.bearing_x,
                        image_bearing_y: picture.bearing_y,
                        image_extent_x: picture.extent_x,
                        image_extent_y: picture.extent_y,
                        ..Default::default()
                    })
                })
            })
            .clone()
    }
    fn get_color_layers(
        &self,
        glyph: GlyphId,
        out: &mut Vec<ColorGlyphLayer>,
        foreground: ColorInt,
    ) -> usize {
        if !self.has_color_glyphs() && self.host_pictures.is_none() {
            return 0;
        }
        if let Some(cached) = self.color_layer_cache.lock().unwrap().get(&glyph).cloned() {
            let count = cached.len();
            out.extend(cached.into_iter().map(|mut layer| {
                if layer.use_foreground {
                    layer.color = foreground;
                }
                for stop in &mut layer.stops {
                    if stop.is_foreground {
                        stop.color = foreground;
                    }
                }
                layer
            }));
            return count;
        }
        let font = self.outline_font();
        let location = self.location(&font);
        let id = OutlineGlyphId::new(glyph as u32);
        let mut layers = Vec::new();
        // Pinned order is COLRv0 layers, then COLRv1/PNG only if no v0 layers.
        if self.face.has_color_layers {
            if let Some(color) = font
                .color_glyphs()
                .get_with_format(id, skrifa::color::ColorGlyphFormat::ColrV0)
            {
                let mut collector = PaintState::new(self, &mut layers, foreground);
                let _ = color.paint(LocationRef::from(&location), &mut collector);
            }
        }
        if layers.is_empty() && self.face.has_color_paint {
            if let Some(color) = font
                .color_glyphs()
                .get_with_format(id, skrifa::color::ColorGlyphFormat::ColrV1)
            {
                let mut collector = PaintState::new(self, &mut layers, foreground);
                let _ = color.paint(LocationRef::from(&location), &mut collector);
            }
        }
        if layers.is_empty() && self.face.has_png {
            if let Some(layer) = png_color_layer(&font, glyph) {
                layers.push(layer);
            }
        }
        if layers.is_empty() {
            return 0;
        }
        let count = layers.len();
        self.color_layer_cache
            .lock()
            .unwrap()
            .insert(glyph, layers.clone());
        out.extend(layers);
        count
    }
    fn on_shape_text(&self, text: &[Unichar], runs: &[TextRun], direction: i32) -> Vec<Paragraph> {
        self.on_shape_text_native(text, runs, direction)
    }
}

// hb_ot_color_has_png reads sanitized CBDT/sbix has_data, independently of
// whether a glyph can actually be extracted. CBLC participates only in lookup.
fn font_has_png_data(font: &OutlineFont<'_>) -> bool {
    let cbdt = font.table_data(skrifa::Tag::new(b"CBDT"));
    if cbdt.is_some_and(|table| cbdt_has_data(table.as_bytes())) {
        return true;
    }
    let num_glyphs = font.maxp().map_or(0, |maxp| u32::from(maxp.num_glyphs()));
    font.table_data(skrifa::Tag::new(b"sbix"))
        .is_some_and(|table| sbix_has_data(table.as_bytes(), num_glyphs))
}

fn cbdt_has_data(data: &[u8]) -> bool {
    // Pinned CBDT::sanitize checks only the four-byte version header and its
    // major version. The sanitized table's has_data tests that major != 0.
    data.len() >= 4 && matches!(u16::from_be_bytes([data[0], data[1]]), 2 | 3)
}

fn sbix_has_data(data: &[u8], num_glyphs: u32) -> bool {
    // Pinned sbix::sanitize checks its eight-byte header, nonzero version,
    // complete strike-offset array, and each non-null SBIXStrike. A strike
    // needs its four-byte header plus num_glyphs + 1 shallow image offsets;
    // individual glyph image data and ppem are deliberately not checked here.
    if data.len() < 8 || u16::from_be_bytes([data[0], data[1]]) == 0 {
        return false;
    }
    let count = u32::from_be_bytes(data[4..8].try_into().unwrap()) as usize;
    let Some(offsets_end) = count.checked_mul(4).and_then(|size| size.checked_add(8)) else {
        return false;
    };
    let Some(offsets) = data.get(8..offsets_end) else {
        return false;
    };
    let strike_size = (u64::from(num_glyphs) + 1) * 4 + 4;
    offsets.chunks_exact(4).all(|bytes| {
        let offset = u32::from_be_bytes(bytes.try_into().unwrap());
        offset == 0 || u64::from(offset) + strike_size <= data.len() as u64
    })
}

#[derive(Clone)]
struct PngGlyph<'a> {
    bytes: &'a [u8],
    width: u32,
    height: u32,
    extents: [i32; 4],
}

// hb_font_t::scale_glyph_extents uses fixed 16.16 em_mult on signed 16-bit
// endpoints, including its narrowing and asymmetric half rounding.
fn scale_bitmap_extents(extents: [i32; 4], upem: u16) -> [i32; 4] {
    // changed() keeps UPEM as float, so the fixed multiplier division rounds
    // in f32 before its conversion to int64_t.
    let mult = ((i64::from(STANDARD_SCALE) << 16) as f32 / f32::from(upem)) as i64;
    let scale = |value: i32| ((i64::from(value as i16) * mult + 32768) >> 16) as i32;
    let [x, y, width, height] = extents;
    let x1 = scale(x);
    let y1 = scale(y);
    [x1, y1, scale(x + width) - x1, scale(y + height) - y1]
}

fn sbix_picture<'a>(
    font: &OutlineFont<'a>,
    glyph: GlyphId,
) -> Option<(skrifa::raw::tables::sbix::GlyphData<'a>, u16)> {
    let table = font.sbix().ok()?;
    if table.version() == 0 {
        return None;
    }
    // Rive leaves x/y_ppem unset. HarfBuzz chooses the first largest strike
    // before looking up the glyph, never a smaller strike containing it.
    let strikes = table.strikes();
    let mut strike = strikes.get(0).ok()?;
    for index in 1..strikes.len() {
        let candidate = strikes.get(index).ok()?;
        if candidate.ppem() > strike.ppem() {
            strike = candidate;
        }
    }
    if strike.ppem() == 0 {
        return None;
    }
    let mut id = OutlineGlyphId::new(u32::from(glyph));
    let mut retries = 8;
    let picture = loop {
        let data = strike.glyph_data(id).ok()??;
        if data.graphic_type() != skrifa::Tag::new(b"dupe") {
            break data;
        }
        if retries == 0 {
            return None;
        }
        retries -= 1;
        let target = u16::from_be_bytes(data.data().get(..2)?.try_into().ok()?);
        id = OutlineGlyphId::new(u32::from(target));
    };
    if picture.graphic_type() != skrifa::Tag::new(b"png ") {
        return None;
    }
    if picture.data().is_empty() {
        return None;
    }
    Some((picture, strike.ppem()))
}

fn sbix_png<'a>(font: &OutlineFont<'a>, glyph: GlyphId, upem: u16) -> Option<PngGlyph<'a>> {
    let (picture, ppem) = sbix_picture(font, glyph)?;
    let bytes = picture.data();
    // blob->as<PNGHeader> returns the zero Null header for a short payload.
    // Otherwise get_png_extents rejects dimensions >= 64K.
    let (width, height) = if bytes.len() >= 29 {
        (
            u32::from_be_bytes(bytes.get(16..20)?.try_into().ok()?),
            u32::from_be_bytes(bytes.get(20..24)?.try_into().ok()?),
        )
    } else {
        (0, 0)
    };
    if width >= 65536 || height >= 65536 {
        return None;
    }
    let scale = f32::from(upem) / f32::from(ppem);
    let extents = [
        i32::from(picture.origin_offset_x()),
        i32::from(picture.origin_offset_y()) + height as i32,
        width as i32,
        -(height as i32),
    ]
    .map(|value| (value as f32 * scale).round() as i32);
    Some(PngGlyph {
        bytes,
        width,
        height,
        extents: scale_bitmap_extents(extents, upem),
    })
}

fn cbdt_strike(font: &OutlineFont<'_>) -> Option<skrifa::raw::tables::bitmap::BitmapSize> {
    let location = font.cblc().ok()?;
    if !matches!(location.major_version(), 2 | 3) {
        return None;
    }
    let sizes = location.bitmap_sizes();
    let mut strike = sizes.first()?;
    for candidate in &sizes[1..] {
        if candidate.ppem_x().max(candidate.ppem_y()) > strike.ppem_x().max(strike.ppem_y()) {
            strike = candidate;
        }
    }
    (strike.ppem_x() != 0 && strike.ppem_y() != 0).then_some(*strike)
}

fn cbdt_picture_bytes(data: &[u8], entry_size: usize, format: u16) -> Option<&[u8]> {
    let header = match format {
        17 => 5,
        18 => 8,
        19 => 0,
        _ => return None,
    };
    // HB first validates the indexed entry and its fixed header, then creates
    // a sub-blob clamped to the entire CBDT table, not the indexed entry.
    data.get(..entry_size)?.get(..header + 4)?;
    let length = u32::from_be_bytes(data.get(header..header + 4)?.try_into().ok()?) as usize;
    let payload = data.get(header + 4..)?;
    let bytes = payload.get(..length.min(payload.len()))?;
    (!bytes.is_empty()).then_some(bytes)
}

fn cbdt_glyph_location(
    table: &skrifa::raw::tables::cblc::Cblc<'_>,
    strike: &skrifa::raw::tables::bitmap::BitmapSize,
    glyph: GlyphId,
) -> Option<skrifa::raw::tables::bitmap::BitmapLocation> {
    let id = OutlineGlyphId::new(u32::from(glyph));
    let list = strike.index_subtable_list(table.offset_data()).ok()?;
    let record = list
        .index_subtable_records()
        .iter()
        .find(|record| (record.first_glyph_index()..=record.last_glyph_index()).contains(&id))?;
    let subtable = record.index_subtable(list.offset_data()).ok()?;
    // Pinned HarfBuzz IndexSubtable::get_image_data implements only 1/3.
    // skrifa also implements 2/4/5; do not expand upstream's file behavior.
    if !matches!(subtable.index_format(), 1 | 3) {
        return None;
    }
    strike.location(table.offset_data(), id).ok()
}

fn native_png_exists(font: &OutlineFont<'_>, glyph: GlyphId) -> bool {
    // isColorGlyph asks reference_png, which accepts even format-19 CBDT and
    // short PNG payloads whose extents the paint route cannot read.
    let cbdt = || -> Option<()> {
        let location = font.cblc().ok()?;
        if !matches!(location.major_version(), 2 | 3) {
            return None;
        }
        let table = font.cbdt().ok()?;
        if !matches!(table.major_version(), 2 | 3) {
            return None;
        }
        let strike = cbdt_strike(font)?;
        let entry = cbdt_glyph_location(&location, &strike, glyph)?;
        let data = table.offset_data().as_bytes().get(entry.data_offset..)?;
        cbdt_picture_bytes(data, entry.data_size, entry.format)?;
        Some(())
    };
    cbdt().is_some() || sbix_picture(font, glyph).is_some()
}

fn cbdt_png<'a>(font: &OutlineFont<'a>, glyph: GlyphId, upem: u16) -> Option<PngGlyph<'a>> {
    let location = font.cblc().ok()?;
    if !matches!(location.major_version(), 2 | 3) {
        return None;
    }
    let table = font.cbdt().ok()?;
    if !matches!(table.major_version(), 2 | 3) {
        return None;
    }
    let strike = cbdt_strike(font)?;
    let glyph_location = cbdt_glyph_location(&location, &strike, glyph)?;
    // Pinned CBDT supports metrics only in formats 17/18 (not 19).
    if !matches!(glyph_location.format, 17 | 18) {
        return None;
    }
    let data = table
        .offset_data()
        .as_bytes()
        .get(glyph_location.data_offset..)?;
    let bytes = cbdt_picture_bytes(data, glyph_location.data_size, glyph_location.format)?;
    // BigGlyphMetrics inherits the same first five horizontal metrics as
    // SmallGlyphMetrics; format 18 only appends vertical metrics.
    let height = u32::from(*data.first()?);
    let width = u32::from(*data.get(1)?);
    let x = i32::from(*data.get(2)? as i8);
    let y = i32::from(*data.get(3)? as i8);
    // HarfBuzz scales metric endpoints first, then converts each rounded
    // component by upem/strike_ppem; preserve this order even for non-2048 UPEM.
    let extents = scale_bitmap_extents([x, y, width as i32, -(height as i32)], upem);
    let x_scale = f32::from(upem) / f32::from(strike.ppem_x());
    let y_scale = f32::from(upem) / f32::from(strike.ppem_y());
    Some(PngGlyph {
        bytes,
        width,
        height,
        extents: [
            (extents[0] as f32 * x_scale).round() as i32,
            (extents[1] as f32 * y_scale).round() as i32,
            (extents[2] as f32 * x_scale).round() as i32,
            (extents[3] as f32 * y_scale).round() as i32,
        ],
    })
}

fn png_glyph<'a>(font: &OutlineFont<'a>, glyph: GlyphId) -> Option<PngGlyph<'a>> {
    let upem = font.head().map(|head| head.units_per_em()).unwrap_or(1000);
    let upem = if (16..=16384).contains(&upem) {
        upem
    } else {
        1000
    };
    let sbix = sbix_png(font, glyph, upem);
    // hb_ot_paint_glyph tries CBDT before sbix, whereas glyph extents prefer
    // sbix. Keep those independent priorities for fonts containing both.
    let mut picture = cbdt_png(font, glyph, upem).or_else(|| sbix.clone())?;
    if let Some(sbix) = sbix {
        picture.extents = sbix.extents;
    }
    Some(picture)
}

fn png_color_layer(font: &OutlineFont<'_>, glyph: GlyphId) -> Option<ColorGlyphLayer> {
    let picture = png_glyph(font, glyph)?;
    let [x, y, width, height] = picture.extents;
    Some(ColorGlyphLayer {
        paint_type: ColorGlyphPaintType::Image,
        image_bytes: Arc::from(picture.bytes),
        image_width: picture.width,
        image_height: picture.height,
        image_bearing_x: x as f32 * INVERSE_SCALE,
        image_bearing_y: -y as f32 * INVERSE_SCALE,
        image_extent_x: width as f32 * INVERSE_SCALE,
        image_extent_y: -height as f32 * INVERSE_SCALE,
        ..ColorGlyphLayer::default()
    })
}

fn make_line_metrics(font: &OutlineFont<'_>, location: LocationRef<'_>) -> LineMetrics {
    let metrics = font.metrics(Size::new(STANDARD_SCALE as f32), location);
    let ascent = -metrics.ascent.round() * INVERSE_SCALE;
    let glyph_scale = STANDARD_SCALE as f32 / metrics.units_per_em as f32;
    let top = |value| {
        font.charmap()
            .map(value)
            .and_then(|glyph| font.glyph_metrics(Size::unscaled(), location).bounds(glyph))
            .map(|bounds| {
                // The pinned glyf extents round the varied font-unit bound,
                // then hb_font_t::scale_glyph_extents rounds at the 2048 scale.
                -(bounds.y_max.round() * glyph_scale).round() * INVERSE_SCALE
            })
            .unwrap_or(ascent)
    };
    LineMetrics {
        ascent,
        descent: -metrics.descent.round() * INVERSE_SCALE,
        cap_height: top('H'),
        x_height: top('x'),
    }
}

fn fill_features(
    scripts: skrifa::raw::tables::layout::ScriptList<'_>,
    features: skrifa::raw::tables::layout::FeatureList<'_>,
    out: &mut HashSet<u32>,
) {
    let mut language_features = |language: skrifa::raw::tables::layout::LangSys<'_>| {
        // hb_ot_layout_language_get_feature_tags enumerates feature_indices,
        // not the separately exposed required_feature_index.
        for index in language.feature_indices() {
            if let Ok(record) = features.get(index.get()) {
                out.insert(u32::from_be_bytes(record.tag.to_be_bytes()));
            }
        }
    };
    for index in 0..scripts.script_count() {
        let Ok(script) = scripts.get(index) else {
            continue;
        };
        if script.lang_sys_count() == 0 {
            if let Some(Ok(language)) = script.default_lang_sys() {
                language_features(language);
            }
        } else {
            for index in 0..script.lang_sys_count() {
                if let Ok(language) = script.lang_sys(index) {
                    language_features(language.element);
                }
            }
        }
    }
}

fn unicode_script(point: u32) -> u32 {
    char::from_u32(point)
        .unwrap_or(char::REPLACEMENT_CHARACTER)
        .script()
        .as_iso15924_tag()
}

fn shape_run(text: &[Unichar], text_run: &TextRun, text_offset: u32) -> GlyphRun {
    let font = text_run
        .font
        .as_ref()
        .expect("text run font")
        .as_any()
        .downcast_ref::<HbFont>()
        .expect("text font must be an HBFont");
    let face =
        ShapingFont::from_index(&font.bytes, font.face_index).expect("validated shaping face");
    let shaper = font
        .face
        .shaper_data
        .shaper(&face)
        .instance(Some(&font.shaper_instance))
        .build();
    let mut buffer = UnicodeBuffer::new();
    for (index, &point) in text[..text_run.unichar_count as usize].iter().enumerate() {
        buffer.add(
            char::from_u32(point).unwrap_or(char::REPLACEMENT_CHARACTER),
            index as u32,
        );
    }
    buffer.set_direction(if text_run.level & 1 != 0 {
        Direction::RightToLeft
    } else {
        Direction::LeftToRight
    });
    buffer.set_script(
        harfrust::Script::from_iso15924_tag(Tag::from_u32(text_run.script))
            .unwrap_or(harfrust::script::UNKNOWN),
    );
    buffer.guess_segment_properties();
    // The plan Shaper::shape would build for this buffer, built once per
    // distinct segment properties.
    let direction = buffer.direction();
    let script = Some(buffer.script());
    let language = buffer.language();
    let mut plans = font.face.shape_plans.lock().expect("shape plan cache");
    let key = harfrust::ShapePlanKey::new(script, direction)
        .language(language.as_ref())
        .instance(Some(&font.shaper_instance))
        .features(&font.features);
    let plan = match plans.iter().position(|plan| key.matches(plan)) {
        Some(index) => index,
        None => {
            plans.push(harfrust::ShapePlan::new(
                &shaper,
                direction,
                script,
                language.as_ref(),
                &font.features,
            ));
            plans.len() - 1
        }
    };
    let shaped = shaper.shape(
        buffer,
        ShapeOptions::new()
            .scale(Some(STANDARD_SCALE))
            .features(&font.features)
            .plan(Some(&plans[plan])),
    );
    drop(plans);
    let infos = shaped.glyph_infos();
    let positions = shaped.glyph_positions();
    let count = infos.len();
    let mut run = GlyphRun::new(count);
    run.font = text_run.font.clone();
    run.size = text_run.size;
    run.line_height = text_run.line_height;
    run.letter_spacing = text_run.letter_spacing;
    run.style_id = text_run.style_id;
    run.level = text_run.level;
    let scale = text_run.size / STANDARD_SCALE as f32;
    for index in 0..count {
        let source = if text_run.level & 1 != 0 {
            count - 1 - index
        } else {
            index
        };
        run.glyphs[index] = infos[source].glyph_id as GlyphId;
        run.text_indices[index] = text_offset + infos[source].cluster;
        let advance = positions[source].x_advance as f32 * scale + text_run.letter_spacing;
        run.advances[index] = advance;
        run.xpos[index] = advance;
        run.offsets[index] = Vec2D::new(
            positions[source].x_offset as f32 * scale,
            -positions[source].y_offset as f32 * scale,
        );
    }
    if font.host_pictures.is_some() {
        let mut boundaries: Vec<_> = run
            .text_indices
            .iter()
            .map(|i| (*i - text_offset) as usize)
            .collect();
        boundaries.push(text_run.unichar_count as usize);
        boundaries.sort_unstable();
        boundaries.dedup();
        let mut cluster_texts: HashMap<u32, Arc<[Unichar]>> = HashMap::new();
        let mut leaders: HashMap<u32, GlyphId> = HashMap::new();
        for index in 0..run.glyphs.len() {
            let text_index = run.text_indices[index];
            let start = (text_index - text_offset) as usize;
            let end = boundaries[boundaries.binary_search(&start).unwrap() + 1];
            let cluster = cluster_texts
                .entry(text_index)
                .or_insert_with(|| Arc::from(&text[start..end]))
                .clone();
            run.cluster_texts.push(cluster);
            let is_leader = !leaders.contains_key(&text_index);
            let leader = *leaders.entry(text_index).or_insert(run.glyphs[index]);
            run.cluster_leaders.push(leader);
            run.cluster_leader_flags.push(is_leader);
        }
    }
    run.xpos[count] = 0.0;
    run
}

fn glyph_path(font: &OutlineFont<'_>, location: LocationRef<'_>, glyph: GlyphId) -> RawPath {
    let mut path = RawPath::default();
    if let Some(outline) = font.outline_glyphs().get(OutlineGlyphId::new(glyph as u32)) {
        let settings = DrawSettings::unhinted(Size::new(STANDARD_SCALE as f32), location)
            .with_path_style(skrifa::outline::pen::PathStyle::HarfBuzz);
        let mut pen = RawPathPen {
            path: &mut path,
            current: Vec2D::default(),
            start: Vec2D::default(),
            open: false,
        };
        let _ = outline.draw(settings, &mut pen);
        pen.close();
    }
    path
}
// Match hb_draw_session_t's callback protocol at the Rust outline boundary:
// move is deferred until a segment opens the contour, and close emits the
// explicit closing line when its endpoint differs from the contour start.
// RawPath::close itself must not grow that line for other callers.
struct RawPathPen<'a> {
    path: &'a mut RawPath,
    current: Vec2D,
    start: Vec2D,
    open: bool,
}
impl RawPathPen<'_> {
    fn start_path(&mut self) {
        if !self.open {
            self.path.move_to_point(self.current);
            self.start = self.current;
            self.open = true;
        }
    }
}
impl OutlinePen for RawPathPen<'_> {
    fn move_to(&mut self, x: f32, y: f32) {
        if self.open {
            self.close();
        }
        self.current = Vec2D::new(x * INVERSE_SCALE, -y * INVERSE_SCALE);
    }
    fn line_to(&mut self, x: f32, y: f32) {
        self.start_path();
        self.current = Vec2D::new(x * INVERSE_SCALE, -y * INVERSE_SCALE);
        self.path.line_to_point(self.current);
    }
    fn quad_to(&mut self, x: f32, y: f32, end_x: f32, end_y: f32) {
        self.start_path();
        self.path.quad_to_cubic(
            x * INVERSE_SCALE,
            -y * INVERSE_SCALE,
            end_x * INVERSE_SCALE,
            -end_y * INVERSE_SCALE,
        );
        self.current = Vec2D::new(end_x * INVERSE_SCALE, -end_y * INVERSE_SCALE);
    }
    fn curve_to(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, x2: f32, y2: f32) {
        self.start_path();
        self.path.cubic_to(
            x0 * INVERSE_SCALE,
            -y0 * INVERSE_SCALE,
            x1 * INVERSE_SCALE,
            -y1 * INVERSE_SCALE,
            x2 * INVERSE_SCALE,
            -y2 * INVERSE_SCALE,
        );
        self.current = Vec2D::new(x2 * INVERSE_SCALE, -y2 * INVERSE_SCALE);
    }
    fn close(&mut self) {
        if self.open {
            if self.current != self.start {
                self.path.line_to_point(self.start);
            }
            self.path.close();
        }
        self.open = false;
        self.current = Vec2D::default();
        self.start = Vec2D::default();
    }
}

impl HbFont {
    fn on_shape_text_native(
        &self,
        text: &[Unichar],
        text_runs: &[TextRun],
        text_direction_flag: i32,
    ) -> Vec<Paragraph> {
        #[cfg(test)]
        TEST_SHAPE_CALLS.with(|calls| calls.set(calls.get() + 1));
        let mut paragraphs = Vec::new();
        let mut text_index = 0usize;
        let mut run_index = 0usize;
        let mut run_start_text_index = 0usize;
        let mut unichar_index = 0usize;
        let mut run_text_index = 0u32;
        let default_level = match text_direction_flag {
            0 => Some(unicode_bidi::Level::ltr()),
            1 => Some(unicode_bidi::Level::rtl()),
            _ => None,
        };
        // Bidi's UTF-8 offsets never enter Rive's UTF-32 index domain.
        let mut utf8 = String::new();
        let mut byte_offsets = Vec::with_capacity(text.len() + 1);
        for &point in text {
            byte_offsets.push(utf8.len());
            utf8.push(char::from_u32(point).unwrap_or(char::REPLACEMENT_CHARACTER));
        }
        byte_offsets.push(utf8.len());
        let bidi = unicode_bidi::BidiInfo::new(&utf8, default_level);
        let mut bidi_paragraphs = bidi.paragraphs.iter();
        while let Some(paragraph) = bidi_paragraphs.next() {
            let start = byte_offsets
                .binary_search(&paragraph.range.start)
                .expect("paragraph scalar boundary");
            let mut end = byte_offsets
                .binary_search(&paragraph.range.end)
                .expect("paragraph scalar boundary");
            let mut bidi_levels: Vec<u8> = byte_offsets[start..end]
                .iter()
                .map(|&offset| bidi.levels[offset].number())
                .collect();
            let paragraph_level = paragraph.level.number();
            // SheenBidi's SBAlgorithmGetSeparatorLength includes LF after CR
            // in the same paragraph. unicode-bidi splits at every B instead.
            // X8 assigns both separators this paragraph's base level, not the
            // automatic LTR level of unicode-bidi's standalone LF paragraph.
            if end > start
                && text[end - 1] == u32::from(b'\r')
                && end < text.len()
                && text[end] == u32::from(b'\n')
            {
                let lf = bidi_paragraphs.next().expect("CRLF suffix paragraph");
                assert_eq!(lf.range.start, paragraph.range.end);
                assert_eq!(lf.range.end, byte_offsets[end + 1]);
                bidi_levels.push(paragraph_level);
                end += 1;
            }
            let paragraph_length = end - start;
            let mut paragraph_text_index = 0usize;
            let mut bidi_runs = Vec::with_capacity(text_runs.len());

            while run_index < text_runs.len() {
                let text_run = &text_runs[run_index];
                assert_ne!(text_run.unichar_count, 0);
                let mut last_level = bidi_levels[paragraph_text_index];
                let point = text[text_index];
                let mut last_script = unicode_script(point);
                let split_run = TextRun {
                    font: text_run.font.clone(),
                    size: text_run.size,
                    line_height: text_run.line_height,
                    letter_spacing: text_run.letter_spacing,
                    unichar_count: text_run.unichar_count - run_text_index,
                    script: last_script as u32,
                    style_id: text_run.style_id,
                    level: last_level,
                };
                run_start_text_index = text_index;
                run_text_index += 1;
                text_index += 1;
                paragraph_text_index += 1;
                bidi_runs.push(split_run);

                while run_text_index < text_run.unichar_count
                    && paragraph_text_index < paragraph_length
                {
                    let point = text[text_index];
                    let mut script = if char::from_u32(point).is_some_and(|point| {
                        point.general_category() == GeneralCategory::NonspacingMark
                    }) {
                        u32::from_be_bytes(*b"Zinh")
                    } else {
                        unicode_script(point)
                    };
                    if script == u32::from_be_bytes(*b"Zyyy")
                        || script == u32::from_be_bytes(*b"Zinh")
                    {
                        script = last_script;
                    }
                    if bidi_levels[paragraph_text_index] != last_level || script != last_script {
                        last_script = script;
                        let back = bidi_runs.last_mut().unwrap();
                        back.unichar_count = (text_index - run_start_text_index) as u32;
                        last_level = bidi_levels[paragraph_text_index];
                        let next_run = TextRun {
                            font: back.font.clone(),
                            size: back.size,
                            line_height: back.line_height,
                            letter_spacing: back.letter_spacing,
                            unichar_count: text_run.unichar_count - run_text_index,
                            script: script as u32,
                            style_id: back.style_id,
                            level: last_level,
                        };
                        bidi_runs.push(next_run);
                        run_start_text_index = text_index;
                    }
                    run_text_index += 1;
                    text_index += 1;
                    paragraph_text_index += 1;
                }
                if run_text_index == text_run.unichar_count {
                    run_index += 1;
                    run_text_index = 0;
                }
                if paragraph_text_index == paragraph_length {
                    bidi_runs.last_mut().unwrap().unichar_count =
                        (text_index - run_start_text_index) as u32;
                    break;
                }
            }

            let mut glyph_runs = Vec::with_capacity(bidi_runs.len());
            for text_run in &bidi_runs {
                let mut glyph_run =
                    shape_run(&text[unichar_index..], text_run, unichar_index as u32);
                unichar_index += text_run.unichar_count as usize;

                if fallback_proc().is_some() && fallback_proc_enabled() && !self.has_color_glyphs()
                {
                    for glyph_index in 0..glyph_run.glyphs.len() {
                        if glyph_run.glyphs[glyph_index] != 0 {
                            let text_position = glyph_run.text_indices[glyph_index] as usize;
                            if text_position + 1 < text.len() && text[text_position + 1] == 0xfe0f {
                                let emoji_font = fallback_proc().and_then(|callback| {
                                    note_fallback_call();
                                    callback(text[text_position], 0, self)
                                });
                                if emoji_font
                                    .as_ref()
                                    .is_some_and(|font| font.has_color_glyphs())
                                {
                                    glyph_run.glyphs[glyph_index] = 0;
                                }
                            }
                        }
                    }
                }

                let missing_index = glyph_run.glyphs.iter().position(|glyph| *glyph == 0);
                if fallback_proc().is_none() || missing_index.is_none() || !fallback_proc_enabled()
                {
                    if !glyph_run.glyphs.is_empty() {
                        glyph_runs.push(glyph_run);
                    }
                } else {
                    let index = missing_index.unwrap();
                    let missing = text[glyph_run.text_indices[index] as usize];
                    let fallback = fallback_proc().and_then(|callback| {
                        note_fallback_call();
                        callback(missing, 0, self)
                    });
                    if let Some(fallback) = fallback {
                        perform_fallback(fallback, &mut glyph_runs, text, &glyph_run, text_run, 1);
                    } else if !glyph_run.glyphs.is_empty() {
                        glyph_runs.push(glyph_run);
                    }
                }
            }

            let mut position = 0.0;
            for glyph_run in &mut glyph_runs {
                for x_position in &mut glyph_run.xpos {
                    let advance = *x_position;
                    *x_position = position;
                    position += advance;
                }
            }
            paragraphs.push(Paragraph {
                runs: glyph_runs,
                level: paragraph_level,
            });
        }
        paragraphs
    }
}

fn extract_subset(original: &GlyphRun, start: usize, end: usize) -> GlyphRun {
    let mut subset = GlyphRun::from_arrays(
        original.glyphs[start..end].to_vec(),
        original.text_indices[start..end].to_vec(),
        original.advances[start..end].to_vec(),
        original.xpos[start..=end].to_vec(),
        original.offsets[start..end].to_vec(),
    );
    if !original.cluster_texts.is_empty() {
        subset.cluster_texts = original.cluster_texts[start..end].to_vec();
        subset.cluster_leaders = original.cluster_leaders[start..end].to_vec();
        subset.cluster_leader_flags = original.cluster_leader_flags[start..end].to_vec();
    }
    subset.font = original.font.clone();
    subset.size = original.size;
    subset.line_height = original.line_height;
    subset.letter_spacing = original.letter_spacing;
    subset.level = original.level;
    *subset.xpos.last_mut().unwrap() = 0.0;
    subset.style_id = original.style_id;
    subset
}

fn perform_fallback(
    fallback_font: FontRef,
    glyph_runs: &mut Vec<GlyphRun>,
    text: &[Unichar],
    original: &GlyphRun,
    original_text_run: &TextRun,
    fallback_index: u32,
) {
    assert!(!original.glyphs.is_empty());
    let count = original.glyphs.len();
    let mut start = 0;
    while start < count {
        let mut end = start + 1;
        if original.glyphs[start] == 0 {
            while end < count && original.glyphs[end] == 0 {
                end += 1;
            }
            let text_start = original.text_indices[start];
            let text_count = if end == count {
                original_text_run.unichar_count
                    - (original.text_indices[start] - original.text_indices[0])
            } else {
                original.text_indices[end] - text_start
            };
            let text_run = TextRun {
                font: Some(fallback_font.clone()),
                size: original.size,
                line_height: original.line_height,
                letter_spacing: original_text_run.letter_spacing,
                unichar_count: text_count,
                script: original_text_run.script,
                style_id: original.style_id,
                level: original.level,
            };
            fallback_font
                .as_any()
                .downcast_ref::<HbFont>()
                .expect("fallback font must be an HBFont")
                .shape_fallback_run(
                    glyph_runs,
                    text,
                    text_start,
                    &text_run,
                    original_text_run,
                    fallback_index,
                );
        } else {
            while end < count && original.glyphs[end] != 0 {
                end += 1;
            }
            glyph_runs.push(extract_subset(original, start, end));
        }
        start = end;
    }
}

struct PaintState<'a> {
    font: &'a HbFont,
    layers: &'a mut Vec<ColorGlyphLayer>,
    clip_path: RawPath,
    has_clip: bool,
    foreground: ColorInt,
    transform_stack: Vec<Mat2D>,
}

impl<'a> PaintState<'a> {
    fn new(font: &'a HbFont, layers: &'a mut Vec<ColorGlyphLayer>, foreground: ColorInt) -> Self {
        Self {
            font,
            layers,
            clip_path: RawPath::default(),
            has_clip: false,
            foreground,
            transform_stack: Vec::new(),
        }
    }
    fn push_transform(&mut self, xx: f32, yx: f32, xy: f32, yy: f32, dx: f32, dy: f32) {
        let matrix = Mat2D::new(xx, -yx, -xy, yy, dx * INVERSE_SCALE, -dy * INVERSE_SCALE);
        if self.transform_stack.is_empty() {
            self.transform_stack.push(matrix);
        } else {
            self.transform_stack
                .push(*self.transform_stack.last().unwrap() * matrix);
        }
    }

    fn pop_transform(&mut self) {
        if !self.transform_stack.is_empty() {
            self.transform_stack.pop();
        }
    }

    fn glyph_transform(&self) -> Mat2D {
        let scale = Mat2D::from_scale(INVERSE_SCALE, -INVERSE_SCALE);
        if let Some(matrix) = self.transform_stack.last() {
            *matrix * scale
        } else {
            scale
        }
    }

    fn map_point(&self, x: f32, y: f32) -> Vec2D {
        self.glyph_transform() * Vec2D::new(x, y)
    }

    fn map_radius(&self, radius: f32) -> f32 {
        let scaled = radius * INVERSE_SCALE;
        if let Some(matrix) = self.transform_stack.last() {
            let scale_x = (matrix[0] * matrix[0] + matrix[1] * matrix[1]).sqrt();
            scaled * scale_x
        } else {
            scaled
        }
    }

    fn color(&self, palette: u16, alpha: f32) -> (ColorInt, bool) {
        if palette == 0xffff {
            return (self.foreground, true);
        }
        let Some(color) = self.font.face.palette_colors.get(palette as usize) else {
            return (0xff000000, false);
        };
        let alpha = (color.alpha() as f32 * alpha).round().clamp(0.0, 255.0) as u32;
        (
            (alpha << 24)
                | ((color.red() as u32) << 16)
                | ((color.green() as u32) << 8)
                | color.blue() as u32,
            false,
        )
    }
    fn extract_stops(&self, stops: &[skrifa::color::ColorStop]) -> Vec<GradientStop> {
        let mut stops: Vec<_> = stops
            .iter()
            .map(|stop| GradientStop {
                offset: stop.offset.clamp(0.0, 1.0),
                color: self.color(stop.palette_index, stop.alpha).0,
                is_foreground: stop.palette_index == 0xffff,
            })
            .collect();
        stops.sort_by(|a, b| {
            a.offset
                .partial_cmp(&b.offset)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        stops
    }
    fn make_clip_layer(&self) -> ColorGlyphLayer {
        ColorGlyphLayer {
            path: self.clip_path.clone(),
            ..ColorGlyphLayer::default()
        }
    }
    fn font_scale(&self) -> f32 {
        STANDARD_SCALE as f32
            / self
                .font
                .outline_font()
                .metrics(Size::unscaled(), LocationRef::default())
                .units_per_em as f32
    }
}
impl skrifa::color::ColorPainter for PaintState<'_> {
    fn push_transform(&mut self, transform: skrifa::color::Transform) {
        let scale = self.font_scale();
        PaintState::push_transform(
            self,
            transform.xx,
            transform.yx,
            transform.xy,
            transform.yy,
            transform.dx * scale,
            transform.dy * scale,
        );
    }
    fn pop_transform(&mut self) {
        PaintState::pop_transform(self);
    }
    fn push_clip_glyph(&mut self, glyph: OutlineGlyphId) {
        self.clip_path = self.font.get_path(glyph.to_u32() as GlyphId);
        if let Some(matrix) = self.transform_stack.last() {
            self.clip_path.transform_in_place(*matrix);
        }
        self.has_clip = true;
    }
    fn push_clip_box(&mut self, _: skrifa::raw::types::BoundingBox<f32>) {}
    fn pop_clip(&mut self) {
        self.has_clip = false;
    }
    fn fill(&mut self, brush: skrifa::color::Brush<'_>) {
        if !self.has_clip {
            return;
        }
        let mut layer = self.make_clip_layer();
        // Skrifa's callbacks are font-unit based; normalize them to the
        // standard 2048-unit callback inputs before the pinned Rive mapping.
        let scale = self.font_scale();
        match brush {
            skrifa::color::Brush::Solid {
                palette_index,
                alpha,
            } => {
                let (color, foreground) = self.color(palette_index, alpha);
                layer.color = color;
                layer.use_foreground = foreground;
            }
            skrifa::color::Brush::LinearGradient {
                p0,
                p1,
                color_stops,
                ..
            } => {
                layer.paint_type = ColorGlyphPaintType::LinearGradient;
                layer.stops = self.extract_stops(color_stops);
                let a = self.map_point(p0.x * scale, p0.y * scale);
                let b = self.map_point(p1.x * scale, p1.y * scale);
                layer.x0 = a.x;
                layer.y0 = a.y;
                layer.x1 = b.x;
                layer.y1 = b.y;
            }
            skrifa::color::Brush::RadialGradient {
                c0,
                r0,
                c1,
                r1,
                color_stops,
                ..
            } => {
                layer.paint_type = ColorGlyphPaintType::RadialGradient;
                layer.stops = self.extract_stops(color_stops);
                let a = self.map_point(c0.x * scale, c0.y * scale);
                let b = self.map_point(c1.x * scale, c1.y * scale);
                layer.x0 = a.x;
                layer.y0 = a.y;
                layer.x1 = b.x;
                layer.y1 = b.y;
                layer.r0 = self.map_radius(r0 * scale);
                layer.r1 = self.map_radius(r1 * scale);
                layer.radial_transform = self.glyph_transform()
                    * Mat2D::from_scale_and_translation(
                        r1 * scale,
                        r1 * scale,
                        c1.x * scale,
                        c1.y * scale,
                    );
            }
            skrifa::color::Brush::SweepGradient {
                c0,
                start_angle,
                end_angle,
                color_stops,
                ..
            } => {
                layer.paint_type = ColorGlyphPaintType::SweepGradient;
                layer.stops = self.extract_stops(color_stops);
                let center = self.map_point(c0.x * scale, c0.y * scale);
                layer.x0 = center.x;
                layer.y0 = center.y;
                layer.start_angle = start_angle.to_radians();
                layer.end_angle = end_angle.to_radians();
            }
        }
        self.layers.push(layer);
    }
    // Pinned HB callbacks deliberately do not implement group compositing.
    fn push_layer(&mut self, _: skrifa::color::CompositeMode) {}
    fn pop_layer(&mut self) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn png_capability_uses_sanitized_cbdt_version_header() {
        for major in [0u16, 1, 2, 3, 4, u16::MAX] {
            let mut data = Vec::from(major.to_be_bytes());
            // Minor version is unconstrained by the pinned sanitizer.
            data.extend_from_slice(&u16::MAX.to_be_bytes());
            assert_eq!(cbdt_has_data(&data), matches!(major, 2 | 3));
            for length in 0..4 {
                assert!(!cbdt_has_data(&data[..length]));
            }
        }
    }

    #[test]
    fn png_capability_sanitizes_sbix_without_requiring_picture_data() {
        let mut data = vec![0, 1, 0, 0, 0, 0, 0, 0];
        assert!(sbix_has_data(&data, 2), "header-only version one has data");
        data[1] = 2;
        assert!(sbix_has_data(&data, 2), "future nonzero versions accepted");
        data[1] = 0;
        assert!(!sbix_has_data(&data, 2), "version zero has no data");
        data[1] = 1;
        for length in 0..8 {
            assert!(!sbix_has_data(&data[..length], 2));
        }
        data[7] = 1;
        assert!(!sbix_has_data(&data, 2), "missing strike offset");
        data.extend_from_slice(&0u32.to_be_bytes());
        assert!(sbix_has_data(&data, 2), "null strike is sanitized");
        data[8..12].copy_from_slice(&12u32.to_be_bytes());
        assert!(!sbix_has_data(&data, 2), "missing strike header");
        data.extend_from_slice(&[0; 4]);
        assert!(!sbix_has_data(&data, 2), "missing shallow glyph offsets");
        data.extend_from_slice(&[0xff; 12]);
        assert!(
            sbix_has_data(&data, 2),
            "zero ppem and unusable glyph image offsets still have data"
        );
        assert!(!sbix_has_data(&data[..27], 2));
        data[8..12].copy_from_slice(&u32::MAX.to_be_bytes());
        assert!(!sbix_has_data(&data, 2), "invalid strike offset");
        data[4..8].copy_from_slice(&u32::MAX.to_be_bytes());
        assert!(!sbix_has_data(&data, 2), "invalid strike-offset count");
    }

    // Supplemental coverage for the new API; upstream added no test cases.
    // Reuse the retained tiny font, replacing only its cmap table.
    fn font_with_cmaps(records: &[(u16, u16, Vec<u8>)]) -> FontRef {
        let mut bytes = decode_hex(include_str!("testdata/legacy_kern_he.hex"));
        let mut cmap = Vec::new();
        cmap.extend_from_slice(&0u16.to_be_bytes());
        cmap.extend_from_slice(&(records.len() as u16).to_be_bytes());
        let mut offset = 4 + records.len() * 8;
        for (platform, encoding, table) in records {
            cmap.extend_from_slice(&platform.to_be_bytes());
            cmap.extend_from_slice(&encoding.to_be_bytes());
            cmap.extend_from_slice(
                &(if table.is_empty() { 0 } else { offset as u32 }).to_be_bytes(),
            );
            offset += table.len();
        }
        for (_, _, table) in records {
            cmap.extend_from_slice(table);
        }
        let num_tables = u16::from_be_bytes([bytes[4], bytes[5]]) as usize;
        let row = (0..num_tables)
            .map(|i| 12 + i * 16)
            .find(|&offset| &bytes[offset..offset + 4] == b"cmap")
            .unwrap();
        let cmap_offset = bytes.len() as u32;
        bytes[row + 8..row + 12].copy_from_slice(&cmap_offset.to_be_bytes());
        bytes[row + 12..row + 16].copy_from_slice(&(cmap.len() as u32).to_be_bytes());
        bytes.extend(cmap);
        HbFont::decode_owned(bytes).unwrap()
    }

    fn cmap_long(format: u16, groups: &[(u32, u32, u32)]) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&format.to_be_bytes());
        bytes.extend_from_slice(&0u16.to_be_bytes());
        bytes.extend_from_slice(&(16 + 12 * groups.len() as u32).to_be_bytes());
        bytes.extend_from_slice(&0u32.to_be_bytes());
        bytes.extend_from_slice(&(groups.len() as u32).to_be_bytes());
        for &(start, end, glyph) in groups {
            for value in [start, end, glyph] {
                bytes.extend_from_slice(&value.to_be_bytes());
            }
        }
        bytes
    }

    #[test]
    fn nominal_codepoints_are_scalar_sorted_and_without_fallback() {
        let font = font_with_cmaps(&[(
            3,
            10,
            cmap_long(
                12,
                &[
                    (0x41, 0x41, 1),
                    (0x42, 0x42, 0),
                    (0xd800, 0xd800, 1),
                    (0x1f600, 0x1f600, 2),
                    (0x10ffff, 0x10ffff, 1),
                    (0x110000, 0x110000, 1),
                ],
            ),
        )]);
        fn no_fallback(_: u32, _: u32, _: &dyn Font) -> Option<FontRef> {
            panic!("nominal coverage must not invoke fallback");
        }
        crate::source::text_engine::with_host_fallback_proc(no_fallback, || {
            assert_eq!(font.codepoints(), [0x41, 0x1f600, 0x10ffff]);
        });
        assert_eq!(font.with_options(&[], &[]).codepoints(), font.codepoints());
    }

    #[test]
    fn nominal_codepoints_use_symbol_selection_without_synthesizing_aliases() {
        let font = font_with_cmaps(&[
            (3, 0, cmap_long(12, &[(0xf041, 0xf041, 1)])),
            (3, 10, cmap_long(12, &[(0x42, 0x42, 2)])),
        ]);
        assert!(!font.has_glyph(0x41));
        assert!(font.has_glyph(0xf041));
        assert_eq!(font.codepoints(), [0xf041]);
        let font = font_with_cmaps(&[
            (0, 4, cmap_long(12, &[(0x41, 0x41, 1)])),
            (3, 10, cmap_long(13, &[(0x42, 0x43, 1)])),
        ]);
        assert_eq!(font.codepoints(), [0x42, 0x43]);
    }

    #[test]
    fn nominal_codepoints_include_legacy_cmap_formats() {
        let mut format0 = vec![0; 262];
        format0[2..4].copy_from_slice(&262u16.to_be_bytes());
        format0[6 + 0x41] = 1;
        let mut format6 = Vec::new();
        for value in [6u16, 12, 0, 0x41, 1, 1] {
            format6.extend_from_slice(&value.to_be_bytes());
        }
        let mut format10 = Vec::new();
        for value in [10u16, 0] {
            format10.extend_from_slice(&value.to_be_bytes());
        }
        for value in [22u32, 0, 0x41, 1] {
            format10.extend_from_slice(&value.to_be_bytes());
        }
        format10.extend_from_slice(&1u16.to_be_bytes());
        for table in [format0, format6, format10] {
            let font = font_with_cmaps(&[(3, 10, table)]);
            assert!(font.has_glyph(0x41));
            assert_eq!(font.codepoints(), [0x41]);
        }
    }

    #[test]
    fn nominal_codepoints_do_not_remap_macroman_in_no_legacy_build() {
        let mut table = vec![0; 262];
        table[2..4].copy_from_slice(&262u16.to_be_bytes());
        table[6 + 0x41] = 1;
        table[6 + 0x80] = 2; // MacRoman 0x80 is Unicode U+00c4.
        let font = font_with_cmaps(&[(1, 0, table)]);
        assert!(!font.has_glyph(0xc4));
        assert!(font.has_glyph(0x80));
        assert_eq!(font.codepoints(), [0x41, 0x80]);
    }

    #[test]
    fn nominal_codepoints_do_not_fall_through_an_unsupported_selected_table() {
        for format in [2u16, 99] {
            let font = font_with_cmaps(&[
                (3, 0, format.to_be_bytes().to_vec()),
                (3, 10, cmap_long(12, &[(0x41, 0x41, 1)])),
            ]);
            assert!(!font.has_glyph(0x41));
            assert!(font.codepoints().is_empty());
        }
    }

    #[test]
    fn nominal_codepoints_skip_a_null_preferred_subtable_offset() {
        let font = font_with_cmaps(&[
            (3, 0, Vec::new()),
            (3, 10, cmap_long(12, &[(0x41, 0x41, 1)])),
        ]);
        assert_eq!(font.codepoints(), [0x41]);
    }

    #[test]
    fn nominal_codepoints_macintosh_wildcard_uses_binary_search_match() {
        let font = font_with_cmaps(&[
            (1, 1, cmap_long(12, &[(0x41, 0x41, 1)])),
            (1, 2, cmap_long(12, &[(0x42, 0x42, 1)])),
            (1, 3, cmap_long(12, &[(0x43, 0x43, 1)])),
        ]);
        assert_eq!(font.codepoints(), [0x42]);
    }

    #[test]
    fn nominal_codepoints_exclude_format4_terminal_sentinel() {
        let mut table = Vec::new();
        for value in [
            4u16, 32, 0, 4, 4, 1, 0, 0x41, 0xffff, 0, 0x41, 0xffff, 0xffc0, 2, 0, 0,
        ] {
            table.extend_from_slice(&value.to_be_bytes());
        }
        let font = font_with_cmaps(&[(3, 1, table)]);
        assert!(font.has_glyph(0xffff));
        assert_eq!(font.codepoints(), [0x41]);
    }

    #[test]
    fn nominal_codepoints_keep_ffff_from_a_nonsentinel_segment() {
        let mut table = Vec::new();
        for value in [
            4u16, 32, 0, 4, 4, 1, 0, 0xffff, 0xffff, 0, 0xfffe, 0xffff, 2, 2, 0, 0,
        ] {
            table.extend_from_slice(&value.to_be_bytes());
        }
        let font = font_with_cmaps(&[(3, 1, table)]);
        assert!(font.has_glyph(0xffff));
        assert_eq!(font.codepoints(), [0xffff]);
    }

    #[test]
    fn nominal_codepoints_reject_truncated_format4_segment_arrays() {
        let mut table = Vec::new();
        for value in [
            4u16, 32, 0, 4, 4, 1, 0, 0x41, 0xffff, 0, 0x41, 0xffff, 0xffc0, 2, 0, 0,
        ] {
            table.extend_from_slice(&value.to_be_bytes());
        }
        for length in [16, 20, 24, 28, 30] {
            let font = font_with_cmaps(&[(3, 1, table[..length].to_vec())]);
            assert!(!font.has_glyph(0x41));
            assert!(font.codepoints().is_empty());
        }
    }

    #[test]
    fn nominal_codepoints_format10_wraps_like_unsigned_source_arithmetic() {
        let mut table = Vec::new();
        for value in [10u16, 0] {
            table.extend_from_slice(&value.to_be_bytes());
        }
        for value in [24u32, 0, u32::MAX, 2] {
            table.extend_from_slice(&value.to_be_bytes());
        }
        for glyph in [1u16, 2] {
            table.extend_from_slice(&glyph.to_be_bytes());
        }
        let font = font_with_cmaps(&[(3, 10, table)]);
        assert!(font.has_glyph(0));
        assert_eq!(font.codepoints(), [0]);
    }

    #[test]
    fn nominal_codepoints_union_repeated_groups_during_collection() {
        for format in [12, 13] {
            let groups = vec![(0x41, 0x42, 1); 1024];
            let font = font_with_cmaps(&[(3, 10, cmap_long(format, &groups))]);
            assert_eq!(font.codepoints(), [0x41, 0x42]);
        }
    }

    #[test]
    fn recursive_fallback_stays_inside_the_current_subrun() {
        fn fallback(_: u32, index: u32, _: &dyn Font) -> Option<FontRef> {
            (index < 3).then(|| {
                HbFont::decode(&decode_hex(include_str!("testdata/legacy_kern_he.hex")))
                    .expect("fixture font decodes")
            })
        }
        let font = HbFont::decode(&decode_hex(include_str!("testdata/legacy_kern_he.hex")))
            .expect("fixture font decodes");
        let original = TextRun {
            font: Some(font.clone()),
            size: 24.0,
            line_height: -1.0,
            letter_spacing: 0.0,
            unichar_count: 4,
            script: u32::from_be_bytes(*b"Latn"),
            style_id: 0,
            level: 0,
        };
        let current = TextRun {
            unichar_count: 1,
            ..original.clone()
        };
        let mut runs = Vec::new();
        let text = ['H' as u32, 'e' as u32, '\n' as u32, 's' as u32];
        crate::source::text_engine::with_host_fallback_proc(fallback, || {
            font.as_any()
                .downcast_ref::<HbFont>()
                .unwrap()
                .shape_fallback_run(&mut runs, &text, 2, &current, &original, 1);
        });
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].text_indices, [2]);
        assert_eq!(runs[0].glyphs, [0]);
    }

    fn decode_hex(input: &str) -> Vec<u8> {
        let input: String = input
            .chars()
            .filter(|character| !character.is_ascii_whitespace())
            .collect();
        assert_eq!(input.len() % 2, 0, "hex fixture has a complete final byte");
        input
            .as_bytes()
            .chunks_exact(2)
            .map(|pair| {
                let pair = std::str::from_utf8(pair).expect("hex fixture is ASCII");
                u8::from_str_radix(pair, 16).expect("hex fixture contains only hex digits")
            })
            .collect()
    }

    fn shape(font: &FontRef, text: &str) -> GlyphRun {
        let text: Vec<u32> = text.chars().map(u32::from).collect();
        let paragraphs = font.shape_text(
            &text,
            &[TextRun {
                font: Some(font.clone()),
                size: STANDARD_SCALE as f32,
                line_height: -1.0,
                letter_spacing: 0.0,
                unichar_count: text.len() as u32,
                script: u32::from_be_bytes(*b"Latn"),
                style_id: 0,
                level: 0,
            }],
            0,
        );
        assert_eq!(paragraphs.len(), 1);
        assert_eq!(paragraphs[0].runs.len(), 1);
        paragraphs
            .into_iter()
            .next()
            .expect("one paragraph")
            .runs
            .into_iter()
            .next()
            .expect("one glyph run")
    }

    #[test]
    fn legacy_only_kern_is_suppressed_like_pinned_harfbuzz() {
        let bytes = decode_hex(include_str!("testdata/legacy_kern_he.hex"));
        let font = HbFont::decode(&bytes).expect("legacy kern test font decodes");
        let run = shape(&font, "He");

        assert_eq!(run.advances, [1135.0, 930.0]);
        assert_eq!(run.offsets[1], Vec2D::new(0.0, 0.0));
        let font = font
            .as_any()
            .downcast_ref::<HbFont>()
            .expect("decoded font is HbFont");
        assert!(
            font.features
                .iter()
                .any(|feature| { feature.tag == Tag::from_u32(KERN_TAG) && feature.value == 0 })
        );
    }

    #[test]
    fn gpos_kern_remains_enabled_when_legacy_kern_is_suppressed() {
        let runtime = nuxie_binary::read_runtime_file(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../fixtures/semantic/semantic_list_scroll_focus_fixed.riv"
        )))
        .expect("semantic fixture decodes");
        let font = runtime
            .imported_file_assets_with_contents()
            .into_iter()
            .filter_map(|asset| asset.bytes())
            .find_map(HbFont::decode)
            .expect("semantic fixture embeds a font");

        let kerned = shape(&font, "AV");
        let unkerned_font = font.with_options(
            &[],
            &[Feature {
                tag: KERN_TAG,
                value: 0,
            }],
        );
        let unkerned = shape(&unkerned_font, "AV");

        assert_eq!(kerned.advances[0], 1273.0);
        assert_eq!(unkerned.advances[0], 1413.0);
        assert!(kerned.advances[0] < unkerned.advances[0]);
        let font = font
            .as_any()
            .downcast_ref::<HbFont>()
            .expect("decoded font is HbFont");
        assert!(
            !font
                .features
                .iter()
                .any(|feature| { feature.tag == Tag::from_u32(KERN_TAG) && feature.value == 0 })
        );
    }
}
