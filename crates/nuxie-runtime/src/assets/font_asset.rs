use crate::ArtboardInstance;
#[cfg(test)]
use crate::ComponentDirt;
use harfrust::{FontRef as HarfFontRef, ShaperData};
use nuxie_binary::RuntimeFile;
use nuxie_render_api::{Factory as RenderFactory, NullFactory};
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::rc::{Rc, Weak};
use std::sync::Arc;

/// File-owned decoded font state, keyed by the concrete imported FontAsset.
///
/// Bytes are the portable counterpart of C++ `rcp<Font>`: validation happens
/// when the owner is replaced, while the text backends materialize their
/// backend-specific font views while shaping.
#[derive(Default)]
pub struct RuntimeFontAssetOwners {
    fonts: RefCell<BTreeMap<u32, Arc<[u8]>>>,
    shaper_data: RefCell<BTreeMap<u32, Rc<ShaperData>>>,
    referencers: RefCell<Vec<Weak<RuntimeFontAssetReferencerQueue>>>,
}

impl std::fmt::Debug for RuntimeFontAssetOwners {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RuntimeFontAssetOwners")
            .field("font_count", &self.fonts.borrow().len())
            .field("shaper_data_count", &self.shaper_data.borrow().len())
            .field("referencer_count", &self.referencers.borrow().len())
            .finish()
    }
}

#[derive(Debug, Default)]
pub(crate) struct RuntimeFontAssetReferencerQueue {
    styles_by_asset: RefCell<BTreeMap<u32, Vec<usize>>>,
    pending: RefCell<BTreeSet<(u32, usize)>>,
}

impl RuntimeFontAssetReferencerQueue {
    pub(crate) fn replace_styles(&self, styles: impl IntoIterator<Item = (u32, usize)>) {
        let mut styles_by_asset = BTreeMap::<u32, Vec<usize>>::new();
        for (asset_global, style_local) in styles {
            styles_by_asset
                .entry(asset_global)
                .or_default()
                .push(style_local);
        }
        *self.styles_by_asset.borrow_mut() = styles_by_asset;
        self.pending.borrow_mut().clear();
    }

    fn publish(&self, asset_global: u32) {
        let Some(styles) = self.styles_by_asset.borrow().get(&asset_global).cloned() else {
            return;
        };
        self.pending
            .borrow_mut()
            .extend(styles.into_iter().map(|style| (asset_global, style)));
    }

    fn take(&self) -> BTreeSet<(u32, usize)> {
        std::mem::take(&mut *self.pending.borrow_mut())
    }

    #[cfg(test)]
    pub(crate) fn is_pending(&self) -> bool {
        !self.pending.borrow().is_empty()
    }
}

impl RuntimeFontAssetOwners {
    pub fn from_runtime(runtime: &RuntimeFile) -> Self {
        let owners = Self::default();
        for entry in runtime.imported_file_assets_with_contents() {
            if entry.asset.type_name == "FontAsset"
                && let Some(bytes) = entry.contents
            {
                owners.decode_with_portable_factory(entry.asset.id, bytes);
            }
        }
        owners
    }

    pub fn from_runtime_with_external_fonts(
        runtime: &RuntimeFile,
        external_fonts: &BTreeMap<u32, Arc<[u8]>>,
    ) -> Self {
        let owners = Self::from_runtime(runtime);
        for asset in runtime.file_assets() {
            if asset.type_name != "FontAsset"
                || runtime.imported_file_asset_contents(asset.id).is_some()
            {
                continue;
            }
            let Some(asset_id) = asset
                .uint_property("assetId")
                .and_then(|asset_id| u32::try_from(asset_id).ok())
            else {
                continue;
            };
            if let Some(bytes) = external_fonts.get(&asset_id) {
                owners.decode_with_portable_factory(asset.id, bytes);
            }
        }
        owners
    }

    pub fn get(&self, asset_global: u32) -> Option<Arc<[u8]>> {
        self.fonts.borrow().get(&asset_global).cloned()
    }

    /// Return the shaping tables retained by the decoded FontAsset owner.
    ///
    /// C++ retains HarfBuzz face/font state on `Font`; rebuilding the OpenType
    /// lookup caches for every Yoga measurement makes text-heavy nested layout
    /// scale with font-table parsing rather than with the measured text.
    pub(crate) fn shaper_data(&self, asset_global: u32) -> Option<Rc<ShaperData>> {
        if let Some(shaper_data) = self.shaper_data.borrow().get(&asset_global) {
            return Some(Rc::clone(shaper_data));
        }
        let fonts = self.fonts.borrow();
        let bytes = fonts.get(&asset_global)?;
        let font = HarfFontRef::new(bytes.as_ref()).ok()?;
        let shaper_data = Rc::new(ShaperData::new(&font));
        drop(fonts);
        self.shaper_data
            .borrow_mut()
            .insert(asset_global, Rc::clone(&shaper_data));
        Some(shaper_data)
    }

    pub(crate) fn shaper_data_for_bytes(
        &self,
        asset_global: u32,
        bytes: &[u8],
    ) -> Option<Rc<ShaperData>> {
        let fonts = self.fonts.borrow();
        let retained = fonts.get(&asset_global)?;
        if retained.len() != bytes.len() || !std::ptr::eq(retained.as_ptr(), bytes.as_ptr()) {
            return None;
        }
        drop(fonts);
        self.shaper_data(asset_global)
    }

    /// Decode and atomically replace one FontAsset's retained font.
    ///
    /// Every decode result replaces the current owner: valid bytes install a
    /// font and invalid bytes clear it. Both synchronously publish to every
    /// live TextStyle referencer, matching `src/assets/font_asset.cpp:8-25`;
    /// decode itself routes through pinned `Factory::decodeFont`.
    pub fn decode(&self, asset_global: u32, bytes: &[u8], factory: &mut dyn RenderFactory) -> bool {
        let decoded = factory.decode_font(bytes).ok().filter(|font| {
            // The safe Rust text adapters additionally require every outline
            // to be consumable by Skrifa before publishing the retained font.
            crate::text::embedded_font_is_parseable(font.bytes())
        });
        if let Some(decoded) = decoded {
            self.fonts
                .borrow_mut()
                .insert(asset_global, decoded.into_bytes());
        } else {
            self.fonts.borrow_mut().remove(&asset_global);
        }
        self.shaper_data.borrow_mut().remove(&asset_global);
        self.referencers.borrow_mut().retain(|referencer| {
            let Some(referencer) = referencer.upgrade() else {
                return false;
            };
            referencer.publish(asset_global);
            true
        });
        self.fonts.borrow().contains_key(&asset_global)
    }

    pub fn decode_with_portable_factory(&self, asset_global: u32, bytes: &[u8]) {
        // RuntimeFile construction precedes renderer selection, but pinned
        // Factory::decodeFont is nonvirtual. The portable adapter therefore
        // executes the same Factory default without inventing a backend.
        let mut factory = NullFactory::new();
        let _ = self.decode(asset_global, bytes, &mut factory);
    }

    pub(crate) fn register_referencer(&self, referencer: &Rc<RuntimeFontAssetReferencerQueue>) {
        let mut referencers = self.referencers.borrow_mut();
        referencers.retain(|candidate| candidate.strong_count() != 0);
        let weak = Rc::downgrade(referencer);
        if !referencers.iter().any(|candidate| candidate.ptr_eq(&weak)) {
            referencers.push(weak);
        }
    }
}

impl ArtboardInstance {
    pub(crate) fn settle_runtime_font_asset_updates(&mut self) -> bool {
        self.runtime_font_asset_referencer.take().into_iter().fold(
            false,
            |changed, (asset_global, local_id)| {
                match self.runtime_font_assets.get(asset_global) {
                    Some(bytes) => {
                        self.runtime_font_asset_snapshots
                            .insert(asset_global, bytes);
                    }
                    None => {
                        self.runtime_font_asset_snapshots.remove(&asset_global);
                    }
                }
                self.mark_text_style_shape_dirty(local_id) | changed
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RuntimeFileAsset, RuntimeFileAssetKind, RuntimeFileAssetOwners};
    use nuxie_binary::{FixtureProperty, FixtureRecord, FixtureValue, RuntimeFile};
    use nuxie_graph::GraphFile;
    use nuxie_render_api::{
        ColorInt, DecodedFont, Factory, FillRule, FontDecodeError, GpuCanvasError, GpuCanvasPlan,
        GpuCanvasShader, ImageDecodeError, NullFactory, RawPath, RenderBuffer, RenderBufferFlags,
        RenderBufferType, RenderGpuCanvasShader, RenderImage, RenderPaint, RenderPath,
        RenderShader,
    };
    use nuxie_schema::definition_by_name;

    struct ObservingFontFactory {
        inner: NullFactory,
        decode_calls: usize,
    }

    impl ObservingFontFactory {
        fn new() -> Self {
            Self {
                inner: NullFactory::new(),
                decode_calls: 0,
            }
        }
    }

    impl Factory for ObservingFontFactory {
        fn make_render_buffer(
            &mut self,
            buffer_type: RenderBufferType,
            flags: RenderBufferFlags,
            size_in_bytes: usize,
        ) -> Box<dyn RenderBuffer> {
            self.inner
                .make_render_buffer(buffer_type, flags, size_in_bytes)
        }

        fn make_linear_gradient(
            &mut self,
            sx: f32,
            sy: f32,
            ex: f32,
            ey: f32,
            colors: &[ColorInt],
            stops: &[f32],
        ) -> Box<dyn RenderShader> {
            self.inner
                .make_linear_gradient(sx, sy, ex, ey, colors, stops)
        }

        fn make_radial_gradient(
            &mut self,
            cx: f32,
            cy: f32,
            radius: f32,
            colors: &[ColorInt],
            stops: &[f32],
        ) -> Box<dyn RenderShader> {
            self.inner
                .make_radial_gradient(cx, cy, radius, colors, stops)
        }

        fn make_render_path(
            &mut self,
            raw_path: RawPath,
            fill_rule: FillRule,
        ) -> Box<dyn RenderPath> {
            self.inner.make_render_path(raw_path, fill_rule)
        }

        fn make_empty_render_path(&mut self) -> Box<dyn RenderPath> {
            self.inner.make_empty_render_path()
        }

        fn make_render_paint(&mut self) -> Box<dyn RenderPaint> {
            self.inner.make_render_paint()
        }

        fn decode_image(&mut self, data: &[u8]) -> Result<Box<dyn RenderImage>, ImageDecodeError> {
            self.inner.decode_image(data)
        }

        fn decode_font(&mut self, data: &[u8]) -> Result<DecodedFont, FontDecodeError> {
            self.decode_calls += 1;
            self.inner.decode_font(data)
        }

        fn make_gpu_canvas_shader(
            &mut self,
            shader: &GpuCanvasShader,
        ) -> Result<Arc<dyn RenderGpuCanvasShader>, GpuCanvasError> {
            self.inner.make_gpu_canvas_shader(shader)
        }

        fn make_gpu_canvas_image(
            &mut self,
            vertex_shader: &Arc<dyn RenderGpuCanvasShader>,
            fragment_shader: &Arc<dyn RenderGpuCanvasShader>,
            plan: &GpuCanvasPlan,
        ) -> Result<Box<dyn RenderImage>, GpuCanvasError> {
            self.inner
                .make_gpu_canvas_image(vertex_shader, fragment_shader, plan)
        }
    }

    fn record(type_name: &str, properties: Vec<FixtureProperty>) -> FixtureRecord {
        FixtureRecord {
            type_key: definition_by_name(type_name)
                .unwrap_or_else(|| panic!("missing {type_name} definition"))
                .type_key
                .int,
            properties,
        }
    }

    fn property(type_name: &str, name: &str, value: FixtureValue) -> FixtureProperty {
        FixtureProperty {
            key: crate::properties::property_key_for_name(type_name, name)
                .unwrap_or_else(|| panic!("missing {type_name}.{name}")),
            value,
        }
    }

    fn fixture_font_bytes() -> Vec<u8> {
        include_bytes!("../../../../fixtures/fonts/roboto-a.ttf").to_vec()
    }

    #[test]
    fn asynchronously_decoded_font_notifies_live_text_style_shape_dirt() {
        let runtime = RuntimeFile::from_fixture_records(vec![
            record("Backboard", Vec::new()),
            record(
                "FontAsset",
                vec![property("FontAsset", "assetId", FixtureValue::Uint(10))],
            ),
            record("Artboard", Vec::new()),
            record(
                "Text",
                vec![property("Component", "parentId", FixtureValue::Uint(0))],
            ),
            record(
                "TextStylePaint",
                vec![
                    property("Component", "parentId", FixtureValue::Uint(1)),
                    property("TextStyle", "fontAssetId", FixtureValue::Uint(0)),
                ],
            ),
        ])
        .expect("async font fixture imports");
        let graphs = GraphFile::from_runtime_file(&runtime).expect("async font graph builds");
        let graph = graphs.artboards.first().expect("fixture artboard");
        let retained = Rc::new(RefCell::new(None::<RuntimeFileAsset>));
        let retained_for_loader = Rc::clone(&retained);
        let mut loader =
            move |asset: &RuntimeFileAsset,
                  in_band: &[u8],
                  _factory: &mut dyn nuxie_render_api::Factory| {
                assert_eq!(asset.kind(), RuntimeFileAssetKind::Font);
                assert!(in_band.is_empty());
                *retained_for_loader.borrow_mut() = Some(asset.clone());
                true
            };
        let mut factory = NullFactory::new();
        let owners =
            RuntimeFileAssetOwners::import_with_loader(&runtime, None, &mut factory, &mut loader);
        let mut instance = ArtboardInstance::from_graph(&runtime, graph).expect("instance builds");
        instance.attach_runtime_file_asset_owners(&owners);
        instance.update_pass();

        let font = retained
            .borrow_mut()
            .take()
            .expect("loader retained the async FontAsset handle");
        assert!(font.decode(&fixture_font_bytes(), &mut factory));
        assert!(
            instance.runtime_font_asset_referencer.is_pending(),
            "FontAsset::font publishes synchronously to the live TextStyle"
        );
        assert!(instance.settle_runtime_font_asset_updates());
        assert!(
            instance
                .component(2)
                .expect("TextStylePaint occurrence")
                .dirt
                .contains(ComponentDirt::TEXT_SHAPE),
            "the async callback adds exact TextShape dirt to its referencer"
        );
        assert!(
            instance
                .component(1)
                .expect("owning Text occurrence")
                .dirt
                .contains(ComponentDirt::TEXT_SHAPE),
            "TextStyle::onDirty invalidates its owning Text shape"
        );

        instance.clear_component_dirt(1);
        instance.clear_component_dirt(2);
        assert!(
            !font.decode(b"not a font", &mut factory),
            "decode failure reports false"
        );
        assert!(
            owners.font_assets().get(font.descriptor().id).is_none(),
            "C++ FontAsset::decode installs the null decode result"
        );
        assert!(
            instance.runtime_font_asset_referencer.is_pending(),
            "the failed replacement still publishes FontAsset::font(nullptr)"
        );
        assert!(instance.settle_runtime_font_asset_updates());
        assert!(
            instance
                .component(2)
                .expect("TextStylePaint occurrence")
                .dirt
                .contains(ComponentDirt::TEXT_SHAPE)
        );
    }

    #[test]
    fn p3g_font_asset_decode_routes_through_the_factory_helper() {
        let owners = RuntimeFontAssetOwners::default();
        let mut factory = ObservingFontFactory::new();

        assert!(owners.decode(7, &fixture_font_bytes(), &mut factory));

        assert_eq!(factory.decode_calls, 1);
        assert!(owners.get(7).is_some());
    }

    #[test]
    fn retained_font_owner_reuses_shaper_tables_until_font_replacement() {
        let owners = RuntimeFontAssetOwners::default();
        let mut factory = NullFactory::new();
        let font = fixture_font_bytes();

        assert!(owners.decode(7, &font, &mut factory));
        let first = owners
            .shaper_data(7)
            .expect("decoded font exposes retained shaper data");
        let second = owners
            .shaper_data(7)
            .expect("unchanged font reuses retained shaper data");
        assert!(
            Rc::ptr_eq(&first, &second),
            "C++ retains shaping tables on the decoded Font owner"
        );

        assert!(owners.decode(7, &font, &mut factory));
        let replacement = owners
            .shaper_data(7)
            .expect("replacement font rebuilds retained shaper data");
        assert!(
            !Rc::ptr_eq(&first, &replacement),
            "FontAsset::decode must invalidate caches from the replaced bytes"
        );
    }
}
