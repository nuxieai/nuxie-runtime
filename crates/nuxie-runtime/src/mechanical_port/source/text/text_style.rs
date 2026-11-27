use super::{
    text_interface::TextInterface, text_style_axis::TextStyleAxis,
    text_style_feature::TextStyleFeature, text_variation_helper::RuntimeTextVariationHelperHandle,
};
use crate::mechanical_port::source::{
    assets::{file_asset_referencer::FileAssetReferencer, font_asset::FontAsset},
    component_dirt::ComponentDirt,
    core::CoreHandle,
    core_context::CoreContext,
    generated::text::text_style_base::TextStyleBase,
    importers::import_stack::ImportStack,
    status_code::StatusCode,
    text_engine::{FontCoord, FontFeature, FontRef},
};
impl std::ops::Deref for TextStyle {
    type Target = TextStyleBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for TextStyle {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

impl TextStyle {
    pub const TYPE_KEY: u16 = TextStyleBase::TYPE_KEY;
}

pub trait TextStyleOwner {
    fn style_shape_changed(&self);
}
impl<F: Fn()> TextStyleOwner for F {
    fn style_shape_changed(&self) {
        self();
    }
}

pub struct TextStyle {
    pub base: TextStyleBase,
    variation_helper: Option<RuntimeTextVariationHelperHandle>,
    file_asset_referencer: FileAssetReferencer,
    variable_font: Option<FontRef>,
    variable_font_base: Option<FontRef>,
    coords: Vec<FontCoord>,
    variations: Vec<CoreHandle>,
    style_features: Vec<CoreHandle>,
    features: Vec<FontFeature>,
    text: Option<CoreHandle>,
    owner: Option<std::rc::Weak<dyn TextStyleOwner>>,
    // Keyed font IDs index the importing file, including library style clones.
    file: Option<crate::source::file::RuntimeFileWeakHandle>,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            base: TextStyleBase::default(),
            variation_helper: None,
            file_asset_referencer: FileAssetReferencer::default(),
            variable_font: None,
            variable_font_base: None,
            coords: Vec::new(),
            variations: Vec::new(),
            style_features: Vec::new(),
            features: Vec::new(),
            text: None,
            owner: None,
            file: None,
        }
    }
}
impl TextStyle {
    pub fn set_owner(&mut self, owner: Option<std::rc::Weak<dyn TextStyleOwner>>) {
        self.owner = owner;
    }
    pub fn owner(&self) -> Option<std::rc::Rc<dyn TextStyleOwner>> {
        self.owner.as_ref().and_then(std::rc::Weak::upgrade)
    }
    pub fn asset(&self) -> Option<CoreHandle> {
        self.file_asset_referencer.asset()
    }
    fn font_asset(&self) -> Option<CoreHandle> {
        self.file_asset_referencer
            .asset()
            .filter(|asset| asset.is_type_of(crate::mechanical_port::source::generated::assets::font_asset_base::FontAssetBase::TYPE_KEY))
    }

    /// The lazily-created owned graph node, if this style needed one.
    pub fn variation_helper(&self) -> Option<&RuntimeTextVariationHelperHandle> {
        self.variation_helper.as_ref()
    }

    pub fn add_variation(&mut self, axis: CoreHandle) {
        self.variations.push(axis);
    }
    pub fn add_feature(&mut self, feature: CoreHandle) {
        self.style_features.push(feature);
    }
    /// The value of the variation axis tagged `tag` (big-endian, as `wght`)
    /// this style sets, or None when it sets none.
    pub(crate) fn axis_value(&self, tag: u32) -> Option<f32> {
        // The last one with the tag wins, as the font takes them in order
        // (`update_variable_font`).
        self.variations.iter().rev().find_map(|axis| {
            axis.with_downcast::<TextStyleAxis, _>(|axis| {
                (axis.base.tag() == tag).then(|| axis.base.axis_value())
            })
            .flatten()
        })
    }
    pub fn on_dirty(&mut self, dirt: ComponentDirt) {
        // The borrowed form runs only for adds on this style itself, each of
        // which carries the shape bit, so it keeps upstream's whole-dirt check.
        if let Some(text) = self.text.as_ref() {
            if dirt.contains(ComponentDirt::TEXT_SHAPE) {
                text.with_mut(|text| text.text_interface_mark_shape_dirty());
                if let Some(helper) = &mut self.variation_helper {
                    helper
                        .occurrence()
                        .add_dirt(ComponentDirt::TEXT_SHAPE, false);
                }
            }
        } else if dirt.contains(ComponentDirt::TEXT_SHAPE) {
            if let Some(owner) = self.owner() {
                if let Some(helper) = &mut self.variation_helper {
                    helper
                        .occurrence()
                        .add_dirt(ComponentDirt::TEXT_SHAPE, false);
                }
                owner.style_shape_changed();
            }
        }
    }

    pub(crate) fn add_dirt_occurrence(
        owner: &CoreHandle,
        value: ComponentDirt,
        recurse: bool,
    ) -> bool {
        crate::mechanical_port::source::component::ComponentOccurrenceHandle::Authored(
            owner.clone(),
        )
        .add_dirt(value, recurse)
    }

    /// TextStyle::onDirty. The text is marked only when this add carries the
    /// shape bit. While the bit stays set from an earlier add, that add's mark
    /// of the text and its layout still stands, and the text marks its layout
    /// again itself if its new shape differs from what layout measured.
    /// Upstream marks again on every add while the bit is set; here that
    /// second mark would walk into an object a setter still holds.
    pub(crate) fn on_dirty_occurrence(
        owner: &CoreHandle,
        _dirt: ComponentDirt,
        value: ComponentDirt,
    ) {
        Self::on_dirty_with(owner, value, None);
    }

    /// [`Self::on_dirty_occurrence`] inside a dirt cascade that a Layout
    /// owner started. Marking the Text's shape walks up to that owner, which
    /// is already borrowed, so the walk and its dirt go through `active`.
    pub(crate) fn on_dirty_from_layout(
        owner: &CoreHandle,
        value: ComponentDirt,
        active: &mut crate::mechanical_port::source::component::ActiveLayoutOwner<'_>,
        active_handle: &CoreHandle,
    ) {
        Self::on_dirty_with(owner, value, Some((active, active_handle)));
    }

    fn on_dirty_with(
        owner: &CoreHandle,
        value: ComponentDirt,
        mut active: Option<(
            &mut crate::mechanical_port::source::component::ActiveLayoutOwner<'_>,
            &CoreHandle,
        )>,
    ) {
        let text = owner
            .with(|object| object.as_text_style().expect("TextStyle").text.clone())
            .flatten();
        if let Some(text) = text.filter(|_| value.contains(ComponentDirt::TEXT_SHAPE)) {
            if text.is_type_of(
                crate::mechanical_port::source::generated::text::text_base::TextBase::TYPE_KEY,
            ) {
                match active.as_mut() {
                    Some((active, handle)) => {
                        super::text::Text::mark_shape_dirty_from_layout(&text, true, active, handle)
                    }
                    None => super::text::Text::mark_shape_dirty_occurrence(&text, true),
                }
            } else {
                text.with_mut(|object| object.text_interface_mark_shape_dirty());
            }
            let helper = owner
                .with(|object| {
                    object
                        .as_text_style()
                        .expect("TextStyle")
                        .variation_helper
                        .as_ref()
                        .map(|helper| helper.occurrence())
                })
                .flatten();
            if let Some(helper) = helper {
                match active.as_mut() {
                    Some((active, handle)) => {
                        helper.add_dirt_from_layout(
                            active,
                            handle,
                            ComponentDirt::TEXT_SHAPE,
                            false,
                        );
                    }
                    None => {
                        helper.add_dirt(ComponentDirt::TEXT_SHAPE, false);
                    }
                }
            }
        } else if value.contains(ComponentDirt::TEXT_SHAPE) {
            let (callback, helper) = owner
                .with(|object| {
                    let style = object.as_text_style().expect("TextStyle");
                    (
                        style.owner(),
                        style
                            .variation_helper
                            .as_ref()
                            .map(|helper| helper.occurrence()),
                    )
                })
                .expect("live TextStyle");
            if let Some(callback) = callback {
                if let Some(helper) = helper {
                    helper.add_dirt(ComponentDirt::TEXT_SHAPE, false);
                }
                callback.style_shape_changed();
            }
        }
    }
    pub fn on_added_clean(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        self.text = <dyn TextInterface>::from_core(self.base.parent_handle());
        let mut code = self.base.on_added_clean(context);
        if code != StatusCode::Ok {
            return code;
        }
        if !self.variations.is_empty() || !self.style_features.is_empty() {
            let Some(this) = self.base.handle() else {
                return StatusCode::MissingObject;
            };
            self.variation_helper = Some(RuntimeTextVariationHelperHandle::new(this));
        }
        if let Some(helper) = &mut self.variation_helper {
            code = helper.with_mut(|helper| helper.component.on_added_dirty_runtime(context));
            if code != StatusCode::Ok {
                return code;
            }
            code = helper.with_mut(|helper| helper.component.on_added_clean(context));
            if code != StatusCode::Ok {
                return code;
            }
        }
        StatusCode::Ok
    }
    pub fn font(&mut self) -> Option<FontRef> {
        if self.variable_font.is_some() {
            return self.variable_font.clone();
        }
        if !self.variations.is_empty() || !self.style_features.is_empty() {
            self.update_variable_font();
            if self.variable_font.is_some() {
                return self.variable_font.clone();
            }
        }
        self.font_asset()
            .and_then(|asset| asset.with_downcast::<FontAsset, _>(FontAsset::font))
            .flatten()
    }
    pub fn update_variable_font(&mut self) {
        let Some(base_font) = self
            .font_asset()
            .and_then(|asset| asset.with_downcast::<FontAsset, _>(FontAsset::font))
            .flatten()
        else {
            // Not ready yet, or the font was cleared. Drop the variable font
            // cached from the previous base so font() cannot return it.
            self.variable_font = None;
            self.variable_font_base = None;
            return;
        };
        if !self.variations.is_empty() || !self.style_features.is_empty() {
            if self.variable_font.is_some()
                && self
                    .variable_font_base
                    .as_ref()
                    .is_some_and(|cached| std::sync::Arc::ptr_eq(cached, &base_font))
                && self.same_options()
            {
                return;
            }
            self.coords.clear();
            for axis in &self.variations {
                axis.with_downcast::<TextStyleAxis, _>(|axis| {
                    self.coords.push(FontCoord {
                        axis: axis.base.tag(),
                        value: axis.base.axis_value(),
                    });
                });
            }
            self.features.clear();
            for feature in &self.style_features {
                feature.with_downcast::<TextStyleFeature, _>(|feature| {
                    self.features.push(FontFeature {
                        tag: feature.base.tag(),
                        value: feature.base.feature_value(),
                    });
                });
            }
            self.variable_font = Some(base_font.with_options(&self.coords, &self.features));
            self.variable_font_base = Some(base_font);
        } else {
            self.variable_font = None;
            self.variable_font_base = None;
        }
    }
    fn same_options(&self) -> bool {
        if self.coords.len() != self.variations.len()
            || self.features.len() != self.style_features.len()
        {
            return false;
        }
        for (coord, axis) in self.coords.iter().zip(&self.variations) {
            if !axis
                .with_downcast::<TextStyleAxis, _>(|axis| {
                    coord.axis == axis.base.tag() && coord.value == axis.base.axis_value()
                })
                .expect("live TextStyleAxis")
            {
                return false;
            }
        }
        for (cached, feature) in self.features.iter().zip(&self.style_features) {
            if !feature
                .with_downcast::<TextStyleFeature, _>(|feature| {
                    cached.tag == feature.base.tag() && cached.value == feature.base.feature_value()
                })
                .expect("live TextStyleFeature")
            {
                return false;
            }
        }
        true
    }
    pub fn build_dependencies(&mut self) {
        if let Some(helper) = &mut self.variation_helper {
            let text = self.base.parent_handle().expect("TextStyle parent");
            helper.with_mut(|helper| helper.build_dependencies_for_text(text));
        }
        if let (Some(parent), Some(this)) = (self.base.parent_handle(), self.base.handle()) {
            parent.with_mut(|parent| parent.component_add_dependent(this));
        }
        self.base.build_dependencies();
    }
    pub fn asset_id(&self) -> u32 {
        self.base.font_asset_id()
    }
    pub(crate) fn file_asset_referencer_mut(&mut self) -> &mut FileAssetReferencer {
        &mut self.file_asset_referencer
    }
    pub fn set_asset_occurrence(owner: &CoreHandle, asset: Option<CoreHandle>) {
        if !asset.as_ref().is_some_and(|asset| {
            asset.is_type_of(crate::mechanical_port::source::generated::assets::font_asset_base::FontAssetBase::TYPE_KEY)
        }) {
            return;
        }
        let has_dependent = owner
            .with_mut(|object| {
                let style = object
                    .as_text_style_mut()
                    .expect("TextStyle asset referencer");
                style.file_asset_referencer.set_asset(owner.clone(), asset);
                style.text.is_some() || style.owner().is_some()
            })
            .expect("live TextStyle");
        if has_dependent {
            // Upstream setAsset calls the most-derived onDirty through addDirt.
            // Release the style before Text dirt traverses its dependents.
            Self::add_dirt_occurrence(owner, ComponentDirt::TEXT_SHAPE, false);
        }
    }
    pub fn import(&mut self, stack: &mut ImportStack) -> StatusCode {
        let Some(this) = self.base.handle() else {
            return StatusCode::MissingObject;
        };
        let result = self.file_asset_referencer.register_referencer(this, stack);
        if result != StatusCode::Ok {
            return result;
        }
        self.file = stack
            .latest_backboard_importer()
            .and_then(|importer| importer.file());
        self.base.import(stack)
    }
    fn keyed_font_asset(&self) -> Option<CoreHandle> {
        self.file
            .as_ref()?
            .with_file(|file| file.asset(self.base.font_asset_id() as usize))
            .flatten()
            .filter(|asset| {
                asset.is_type_of(
                    crate::source::generated::assets::font_asset_base::FontAssetBase::TYPE_KEY,
                )
            })
    }
    pub(crate) fn font_asset_id_changed_occurrence(owner: &CoreHandle) {
        let asset = owner
            .with(|object| {
                object
                    .as_text_style()
                    .expect("TextStyle font owner")
                    .keyed_font_asset()
            })
            .flatten();
        if let Some(asset) = asset {
            Self::set_asset_occurrence(owner, Some(asset));
        }
    }
    pub fn font_asset_id_changed(&mut self) {
        let Some(asset) = self.keyed_font_asset() else {
            return;
        };
        if let Some(owner) = self.base.handle() {
            self.file_asset_referencer.set_asset(owner, Some(asset));
        } else {
            self.file_asset_referencer.set_asset_unattached(Some(asset));
        }
        if self.text.is_some() || self.owner().is_some() {
            crate::source::generated::core_registry::CoreCapabilities::component_add_dirt(
                self,
                ComponentDirt::TEXT_SHAPE,
                false,
            );
        }
    }
    pub fn font_size_changed(&mut self) {
        self.metrics_changed();
    }
    fn metrics_changed(&self) {
        if let Some(text) = &self.text {
            text.with_mut(|text| text.text_interface_mark_shape_dirty());
        } else if let Some(owner) = self.owner() {
            owner.style_shape_changed();
        }
    }
    pub(crate) fn set_double_occurrence(owner: &CoreHandle, key: u16, value: f32) -> bool {
        let Some(changed) = owner.with_mut(|object| {
            let style = object
                .as_text_style_mut()
                .expect("TextStyle property owner");
            match key {
                TextStyleBase::FONT_SIZE_PROPERTY_KEY => style.base.set_font_size_value(value),
                TextStyleBase::LINE_HEIGHT_PROPERTY_KEY => style.base.set_line_height_value(value),
                TextStyleBase::LETTER_SPACING_PROPERTY_KEY => {
                    style.base.set_letter_spacing_value(value)
                }
                _ => unreachable!("TextStyle generated numeric property"),
            }
        }) else {
            return false;
        };
        if !changed {
            return true;
        }

        // All three pinned changed callbacks call m_text->markShapeDirty().
        // That callback invalidates this same TextStylePaint's stroke effects,
        // so the property owner borrow must end before entering Text.
        let text = owner
            .with(|object| object.as_text_style().expect("TextStyle").text.clone())
            .flatten();
        if let Some(text) = text {
            if text.is_type_of(
                crate::mechanical_port::source::generated::text::text_base::TextBase::TYPE_KEY,
            ) {
                super::text::Text::mark_shape_dirty_occurrence(&text, true);
            } else {
                text.with_mut(|object| object.text_interface_mark_shape_dirty());
            }
        } else if let Some(callback) = owner
            .with(|object| object.as_text_style().expect("TextStyle").owner())
            .flatten()
        {
            callback.style_shape_changed();
        }
        owner.notify_property_changed(key);
        true
    }
    pub fn line_height_changed(&mut self) {
        self.metrics_changed();
    }
    pub fn letter_spacing_changed(&mut self) {
        self.metrics_changed();
    }
    pub fn clone_value(&self) -> Box<Self> {
        let mut twin = Box::new(Self::default());
        let mut base = std::mem::take(&mut twin.base);
        base.copy(&self.base, twin.as_mut());
        twin.base = base;
        self.clone_font_into(&mut twin);
        twin
    }
    /// Share the current font and retain its original file's lookup identity.
    /// The weak file link avoids a file -> style -> file ownership cycle.
    fn clone_font_into(&self, twin: &mut TextStyle) {
        twin.file = self.file.clone();
        if let Some(asset) = self.file_asset_referencer.asset() {
            twin.file_asset_referencer.set_asset_unattached(Some(asset));
        }
    }
    pub fn validate(&self, context: &dyn CoreContext) -> bool {
        let parent = context.resolve(self.base.parent_id());
        // Import validation already borrows the parent Artboard. Its cached
        // type predicate needs no payload borrow; only other parent kinds
        // need the TextInterface capability lookup.
        parent.as_ref().is_some_and(|parent| {
            parent.is_type_of(crate::source::generated::artboard_base::ArtboardBase::TYPE_KEY)
        }) || <dyn TextInterface>::from_core(parent).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mechanical_port::source::core::CoreArena;

    /// A style that sets `wght` to 600 and `opsz` to 12 reads each by its
    /// tag, and None for `wdth`, which it does not set; a style with no
    /// axes reads None.
    #[test]
    fn axis_value_reads_the_axis_with_the_tag() {
        let tag = |name: &[u8; 4]| u32::from_be_bytes(*name);
        let arena = CoreArena::default();
        let axis = |name: &[u8; 4], value: f32| {
            let mut axis = TextStyleAxis::default();
            axis.base.set_tag_value(tag(name));
            axis.base.set_axis_value_value(value);
            arena.insert(axis)
        };
        let mut style = TextStyle::default();
        assert_eq!(style.axis_value(tag(b"wght")), None);
        style.add_variation(axis(b"wght", 600.0));
        style.add_variation(axis(b"opsz", 12.0));
        assert_eq!(style.axis_value(tag(b"wght")), Some(600.0));
        assert_eq!(style.axis_value(tag(b"opsz")), Some(12.0));
        assert_eq!(style.axis_value(tag(b"wdth")), None);
    }

    fn record(
        owner: &str,
        values: Vec<(&str, nuxie_binary::FixtureValue)>,
    ) -> nuxie_binary::FixtureRecord {
        let def = nuxie_schema::definition_by_name(owner).unwrap();
        nuxie_binary::FixtureRecord {
            type_key: def.type_key.int,
            properties: values
                .into_iter()
                .map(|(name, value)| {
                    let key = std::iter::once(def.name)
                        .chain(def.ancestors.iter().copied())
                        .filter_map(nuxie_schema::definition_by_name)
                        .flat_map(|d| d.properties)
                        .find(|p| p.name == name)
                        .unwrap_or_else(|| panic!("{owner}.{name}"))
                        .key
                        .int;
                    nuxie_binary::FixtureProperty { key, value }
                })
                .collect(),
        }
    }

    /// A Text that reads "Hi" with a TextStylePaint holding a Fill of
    /// 0xFF336699, on the Artboard or, with `in_box`, under a LayoutComponent.
    /// The file has two font assets with the same face.
    struct Words {
        _factory: nuxie_render_api::PersistentFactory<nuxie_render_api::RecordingFactory>,
        _file: crate::RuntimeFileHandle,
        artboard: crate::RuntimeArtboardInstanceHandle,
        container: Option<u32>,
        text: u32,
        style: u32,
        fill: u32,
    }

    fn words(in_box: bool) -> Words {
        use nuxie_binary::FixtureValue::{Bytes, Color, Double, String, Uint};
        let face = || {
            record(
                "FileAssetContents",
                vec![(
                    "bytes",
                    Bytes(
                        include_bytes!("../../../../../../fixtures/fonts/roboto-input.ttf")
                            .to_vec(),
                    ),
                )],
            )
        };
        let mut records = vec![
            record("Backboard", vec![]),
            record("FontAsset", vec![("assetId", Uint(0))]),
            face(),
            record("FontAsset", vec![("assetId", Uint(1))]),
            face(),
            record(
                "Artboard",
                vec![("width", Double(200.0)), ("height", Double(100.0))],
            ),
        ];
        // Artboard objects count from the Artboard, at 0.
        let next = |records: &Vec<nuxie_binary::FixtureRecord>| (records.len() - 5) as u64;
        let mut parent = 0;
        let mut container = None;
        if in_box {
            let id = next(&records);
            records.push(record(
                "LayoutComponent",
                vec![("parentId", Uint(0)), ("styleId", Uint(id + 1))],
            ));
            records.push(record("LayoutComponentStyle", vec![("parentId", Uint(id))]));
            container = Some(id as u32);
            parent = id;
        }
        let text = next(&records);
        records.push(record("Text", vec![("parentId", Uint(parent))]));
        let style = next(&records);
        records.push(record(
            "TextStylePaint",
            vec![
                ("parentId", Uint(text)),
                ("fontAssetId", Uint(0)),
                ("fontSize", Double(16.0)),
            ],
        ));
        let fill = next(&records);
        records.push(record("Fill", vec![("parentId", Uint(style))]));
        records.push(record(
            "SolidColor",
            vec![("parentId", Uint(fill)), ("colorValue", Color(0xff33_6699))],
        ));
        records.push(record(
            "TextValueRun",
            vec![
                ("parentId", Uint(text)),
                ("styleId", Uint(style)),
                ("text", String("Hi".into())),
            ],
        ));
        let bytes = nuxie_binary::encode_runtime_file(
            &nuxie_binary::RuntimeFile::from_fixture_records(records).unwrap(),
        )
        .unwrap();
        let mut factory =
            nuxie_render_api::PersistentFactory::new(nuxie_render_api::RecordingFactory::new());
        let file = crate::File::import(
            &bytes,
            crate::RuntimeFactoryHandle::from_factory(&mut factory).expect("retained factory"),
            None,
            None,
            None,
        )
        .expect("the file imports");
        let artboard = file
            .with_file(crate::File::artboard_default)
            .expect("default artboard");
        Words {
            _factory: factory,
            _file: file,
            artboard,
            container,
            text: text as u32,
            style: style as u32,
            fill: fill as u32,
        }
    }

    impl Words {
        fn object(&self, id: u32) -> CoreHandle {
            self.artboard
                .with_artboard(|artboard| artboard.base.resolve_handle(id))
                .expect("a words object")
        }
        fn write_double(&self, id: u32, key: u16, value: f32) -> bool {
            crate::mechanical_port::source::generated::core_registry::CoreRegistry::set_double_handle(
                &self.object(id),
                i32::from(key),
                value,
            )
        }
        fn write_uint(&self, id: u32, key: u16, value: u32) -> bool {
            crate::mechanical_port::source::generated::core_registry::CoreRegistry::set_uint_handle(
                &self.object(id),
                i32::from(key),
                value,
            )
        }
        /// The Text's render opacity, whether it will draw, and its Fill's
        /// render opacity.
        fn drawn(&self) -> (f32, bool, f32) {
            use crate::mechanical_port::source::generated::core_registry::CoreCapabilities;
            let (opacity, will_draw) = self
                .object(self.text)
                .with(|object| {
                    (
                        object.as_transform_component().unwrap().render_opacity(),
                        object.drawable_will_draw(),
                    )
                })
                .unwrap();
            let fill = self
                .object(self.fill)
                .with_downcast::<crate::mechanical_port::source::shapes::paint::fill::Fill, _>(
                    |fill| fill.base.render_opacity(),
                )
                .unwrap();
            (opacity, will_draw, fill)
        }
    }

    const OPACITY: u16 = crate::mechanical_port::source::generated::world_transform_component_base::WorldTransformComponentBase::OPACITY_PROPERTY_KEY;

    /// A Text fades by handing its render opacity to its style's paints
    /// (upstream src/text/text.cpp:2151-2157); it still reports that it will
    /// draw, as upstream's Drawable::willDraw does (src/drawable.cpp:88).
    #[test]
    fn a_text_set_to_opacity_0_before_the_first_advance_fades_its_paints() {
        let words = words(false);
        assert!(words.write_double(words.text, OPACITY, 0.0));
        words.artboard.advance_default(0.0);
        assert_eq!(words.drawn(), (0.0, true, 0.0));
    }

    /// The Text takes the box's opacity into its style's paints (upstream
    /// src/text/text.cpp:2151-2157) and still reports that it will draw
    /// (src/drawable.cpp:88).
    #[test]
    fn a_text_under_a_box_with_its_shape_pending_takes_the_box_opacity() {
        let words = words(true);
        words.artboard.advance_default(0.0);
        assert!(words.write_uint(words.style, TextStyleBase::FONT_ASSET_ID_PROPERTY_KEY, 1));
        assert!(words.write_double(words.container.unwrap(), OPACITY, 0.0));
        words.artboard.advance_default(0.0);
        assert_eq!(words.drawn(), (0.0, true, 0.0));
    }
}
