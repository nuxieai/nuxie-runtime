use crate::mechanical_port::source::{
    component_dirt::ComponentDirt,
    core::CoreHandle,
    core_context::CoreContext,
    generated::text::text_input_drawable_base::TextInputDrawableBase,
    math::mat2d::Mat2D,
    renderer::Renderer,
    shapes::{
        paint::{
            color::ColorInt, fill::Fill, shape_paint::ShapePaintPathKind,
            shape_paint_path::ShapePaintPath, solid_color::SolidColor,
        },
        shape_paint_container::ShapePaintContainer,
    },
    status_code::StatusCode,
};

#[derive(Default)]
pub struct TextInputDrawable {
    pub base: TextInputDrawableBase,
    pub paints: ShapePaintContainer,
}
impl TextInputDrawable {
    pub fn text_input_handle(&self) -> CoreHandle {
        self.base
            .parent_handle()
            .expect("TextInputDrawable TextInput parent")
    }
    /// Links the parent's transform and opacity first, as TextInput does, so
    /// the field's text, caret and selection fade with its ancestors.
    /// Upstream's skips this link.
    pub fn on_added_clean(&mut self, context: &mut dyn CoreContext) -> StatusCode {
        self.base.on_added_clean(context);
        if self.base.parent_handle().is_some_and(|parent| {
            parent
                .is_type_of(crate::mechanical_port::source::generated::text::text_input_base::TextInputBase::TYPE_KEY)
        }) {
            StatusCode::Ok
        } else {
            StatusCode::InvalidObject
        }
    }
    /// Hands its render opacity to its paints, as a Shape does, so a fade
    /// between 0 and 1 reaches the field's fills and strokes. Our addition:
    /// upstream's TextInputDrawable has no update of its own.
    pub(crate) fn update_after_transform_super(&mut self, value: ComponentDirt) {
        if value.contains(ComponentDirt::RENDER_OPACITY) {
            self.paints.propagate_opacity(self.base.render_opacity());
        }
    }
    pub fn shape_world_transform(&self) -> Mat2D {
        self.base.world_transform()
    }
    pub fn path_builder(&self) -> CoreHandle {
        self.text_input_handle()
    }

    /// The concrete cursor/selection/text owner supplies its virtual path.
    /// Paths stay borrowed inside the parent TextInput occurrence.
    pub fn draw_with_path(
        &mut self,
        renderer: &mut Renderer,
        path: Option<&mut ShapePaintPath>,
        world: Mat2D,
    ) {
        let Some(path) = path else {
            return;
        };
        for handle in self.paints.shape_paints().iter().cloned() {
            handle.with_mut(|object| {
                let Some(paint) = object.as_shape_paint_behavior_mut() else {
                    return;
                };
                if !paint.is_visible() {
                    return;
                }
                if paint.pick_path_kind() == ShapePaintPathKind::World {
                    unreachable!("TextInputDrawable::worldPath is unreachable upstream");
                }
                let fill_rule = paint.fill_rule();
                paint.shape_paint_mut().draw_with_active_container(
                    renderer,
                    path,
                    world,
                    false,
                    None,
                    true,
                    fill_rule,
                    &|| self.shape_world_transform(),
                    None,
                );
            });
        }
    }
    // Our additive text-field color route. Outline paints keep their existing
    // traversal; foreground lookup matches TextStylePaint::foreground_color.
    // Like upstream Text's color commands, color glyph opacity is always 1.0.
    // The file route leaves both outlines and color glyphs unclipped; the direct
    // RawTextInput route wraps both in its existing overflow clip.
    pub(crate) fn draw_text(
        &mut self,
        renderer: &mut Renderer,
        input: &mut super::raw_text_input::RawTextInput,
        world: Mat2D,
        selected: bool,
    ) {
        let path = if selected {
            input.selected_text_path()
        } else {
            input.text_path()
        };
        self.draw_with_path(renderer, Some(path), world);
        if !input.has_color_glyphs(Some(selected)) {
            return;
        }
        let foreground = self.fill_color();
        let factory = self
            .base
            .with_artboard(|artboard| artboard.factory())
            .flatten()
            .expect("TextInputDrawable requires its imported renderer factory");
        input.draw_color_glyphs(&factory, renderer, world, Some(selected), foreground, 1.0);
    }

    /// The color its first solid fill paints, as ARGB; black when it has none.
    pub(crate) fn fill_color(&self) -> ColorInt {
        self.paints
            .shape_paints()
            .iter()
            .find_map(|paint| {
                paint
                    .with_downcast::<Fill, _>(|fill| {
                        fill.base.paint().and_then(|mutator| {
                            mutator.with_downcast::<SolidColor, _>(|color| {
                                color.base.color_value() as u32
                            })
                        })
                    })
                    .flatten()
            })
            .unwrap_or(0xff000000)
    }

    pub fn will_draw(&self) -> bool {
        self.base.will_draw() && self.base.render_opacity() != 0.0
    }
}
impl std::ops::Deref for TextInputDrawable {
    type Target = TextInputDrawableBase;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl std::ops::DerefMut for TextInputDrawable {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

#[cfg(test)]
mod tests {
    use super::super::text_input_text::TextInputText;
    use crate::{File, RuntimeFactoryHandle};
    use nuxie_binary::{
        FixtureProperty, FixtureRecord, FixtureValue, RuntimeFile, encode_runtime_file,
    };
    use nuxie_render_api::{PersistentFactory, RecordingFactory};

    fn record(owner: &str, values: Vec<(&str, FixtureValue)>) -> FixtureRecord {
        let def = nuxie_schema::definition_by_name(owner).unwrap();
        FixtureRecord {
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
                    FixtureProperty { key, value }
                })
                .collect(),
        }
    }

    /// The text of a field whose text drawable has a solid fill of
    /// 0xFF336699 paints that color; the text of a field whose drawable
    /// has no fill paints black, as `draw_text` draws its color glyphs.
    #[test]
    fn fill_color_is_the_first_solid_fill_or_black() {
        use FixtureValue::{Bytes, Color, Double, Uint};
        let records = vec![
            record("Backboard", vec![]),
            record("FontAsset", vec![("assetId", Uint(0))]),
            record(
                "FileAssetContents",
                vec![(
                    "bytes",
                    Bytes(
                        include_bytes!("../../../../../../fixtures/fonts/roboto-input.ttf")
                            .to_vec(),
                    ),
                )],
            ),
            record(
                "Artboard",
                vec![("width", Double(200.0)), ("height", Double(100.0))],
            ), // 0
            record("TextInput", vec![("parentId", Uint(0))]), // 1
            record(
                "TextStylePaint",
                vec![
                    ("parentId", Uint(1)),
                    ("fontAssetId", Uint(0)),
                    ("fontSize", Double(16.0)),
                ],
            ), // 2
            record("TextInputText", vec![("parentId", Uint(1))]), // 3
            record("Fill", vec![("parentId", Uint(3))]),      // 4
            record(
                "SolidColor",
                vec![("parentId", Uint(4)), ("colorValue", Color(0xff33_6699))],
            ), // 5
            record("TextInput", vec![("parentId", Uint(0))]), // 6
            record(
                "TextStylePaint",
                vec![
                    ("parentId", Uint(6)),
                    ("fontAssetId", Uint(0)),
                    ("fontSize", Double(16.0)),
                ],
            ), // 7
            record("TextInputText", vec![("parentId", Uint(6))]), // 8
        ];
        let bytes =
            encode_runtime_file(&RuntimeFile::from_fixture_records(records).unwrap()).unwrap();
        let mut factory = PersistentFactory::new(RecordingFactory::new());
        let file = File::import(
            &bytes,
            RuntimeFactoryHandle::from_factory(&mut factory).expect("retained factory"),
            None,
            None,
            None,
        )
        .expect("the file imports");
        let page = file
            .with_file(File::artboard_default)
            .expect("default artboard");
        let color = |id| {
            page.with_artboard(|artboard| artboard.base.resolve_handle(id))
                .expect("the text drawable")
                .with_downcast::<TextInputText, _>(|text| text.base.base.fill_color())
                .expect("a TextInputText")
        };
        assert_eq!(color(3), 0xff33_6699);
        assert_eq!(color(8), 0xff00_0000);
    }

    /// An ancestor of the field, outermost first.
    enum Ancestor {
        /// A Node at this opacity.
        Node(f32),
        /// A LayoutComponent, with its style, at this opacity.
        Layout(f32),
        /// A Node at opacity 1, scaled by this much along x.
        ScaledX(f32),
    }

    /// A field that reads "Hi", under `ancestors`, with all four text input
    /// drawables, each holding a Fill of 0xFF336699. With `feather_x`, the
    /// text's Fill has an inner Feather of strength 0, offset by that much
    /// along x in world space. The file has a second font asset with the same
    /// face, and, with ancestors, an animation "Fade" that keys the outermost
    /// one's opacity to 0 at frame 0.
    struct Field {
        _factory: PersistentFactory<RecordingFactory>,
        _file: crate::RuntimeFileHandle,
        artboard: crate::RuntimeArtboardInstanceHandle,
        /// Each ancestor, outermost first.
        ancestors: Vec<u32>,
        input: u32,
        style: u32,
        /// TextInputText, TextInputCursor, TextInputSelection and
        /// TextInputSelectedText, with each one's Fill.
        drawables: Vec<(u32, u32)>,
        feather: Option<u32>,
    }

    fn field(ancestors: &[Ancestor], feather_x: Option<f32>) -> Field {
        let field = unadvanced(ancestors, feather_x);
        field.artboard.advance_default(0.0);
        field
    }

    /// The field before its first advance.
    fn unadvanced(ancestors: &[Ancestor], feather_x: Option<f32>) -> Field {
        use FixtureValue::{Bool, Bytes, Color, Double, String, Uint};
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
        let next = |records: &Vec<FixtureRecord>| (records.len() - 5) as u64;
        let mut parent = 0;
        let mut ancestor_ids = vec![];
        for ancestor in ancestors {
            let id = next(&records);
            ancestor_ids.push(id as u32);
            match *ancestor {
                Ancestor::Node(opacity) => records.push(record(
                    "Node",
                    vec![("parentId", Uint(parent)), ("opacity", Double(opacity))],
                )),
                Ancestor::Layout(opacity) => {
                    records.push(record(
                        "LayoutComponent",
                        vec![
                            ("parentId", Uint(parent)),
                            ("opacity", Double(opacity)),
                            ("styleId", Uint(id + 1)),
                        ],
                    ));
                    records.push(record("LayoutComponentStyle", vec![("parentId", Uint(id))]));
                }
                Ancestor::ScaledX(scale) => records.push(record(
                    "Node",
                    vec![("parentId", Uint(parent)), ("scaleX", Double(scale))],
                )),
            }
            parent = id;
        }
        let input = next(&records);
        records.push(record(
            "TextInput",
            vec![("parentId", Uint(parent)), ("text", String("Hi".into()))],
        ));
        let style = next(&records);
        records.push(record(
            "TextStylePaint",
            vec![
                ("parentId", Uint(input)),
                ("fontAssetId", Uint(0)),
                ("fontSize", Double(16.0)),
            ],
        ));
        let mut drawables = vec![];
        let mut feather = None;
        for owner in [
            "TextInputText",
            "TextInputCursor",
            "TextInputSelection",
            "TextInputSelectedText",
        ] {
            let drawable = next(&records);
            records.push(record(owner, vec![("parentId", Uint(input))]));
            let fill = next(&records);
            records.push(record("Fill", vec![("parentId", Uint(drawable))]));
            records.push(record(
                "SolidColor",
                vec![("parentId", Uint(fill)), ("colorValue", Color(0xff33_6699))],
            ));
            if let (Some(offset), "TextInputText") = (feather_x, owner) {
                feather = Some(next(&records) as u32);
                records.push(record(
                    "Feather",
                    vec![
                        ("parentId", Uint(fill)),
                        ("inner", Bool(true)),
                        ("strength", Double(0.0)),
                        ("offsetX", Double(offset)),
                    ],
                ));
            }
            drawables.push((drawable as u32, fill as u32));
        }
        if let Some(&outermost) = ancestor_ids.first() {
            records.push(record(
                "LinearAnimation",
                vec![
                    ("name", String("Fade".into())),
                    ("fps", Uint(60)),
                    ("duration", Uint(60)),
                    ("loopValue", Uint(0)),
                ],
            ));
            records.push(record(
                "KeyedObject",
                vec![("objectId", Uint(u64::from(outermost)))],
            ));
            records.push(record("KeyedProperty", vec![("propertyKey", Uint(18))]));
            records.push(record(
                "KeyFrameDouble",
                vec![
                    ("frame", Uint(0)),
                    ("value", Double(0.0)),
                    ("interpolationType", Uint(0)),
                ],
            ));
        }
        let bytes =
            encode_runtime_file(&RuntimeFile::from_fixture_records(records).unwrap()).unwrap();
        let mut factory = PersistentFactory::new(RecordingFactory::new());
        let file = File::import(
            &bytes,
            RuntimeFactoryHandle::from_factory(&mut factory).expect("retained factory"),
            None,
            None,
            None,
        )
        .expect("the file imports");
        let artboard = file
            .with_file(File::artboard_default)
            .expect("default artboard");
        Field {
            _factory: factory,
            _file: file,
            artboard,
            ancestors: ancestor_ids,
            input: input as u32,
            style: style as u32,
            drawables,
            feather,
        }
    }

    impl Field {
        fn object(&self, id: u32) -> crate::mechanical_port::source::core::CoreHandle {
            self.artboard
                .with_artboard(|artboard| artboard.base.resolve_handle(id))
                .expect("a field object")
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
        /// Each drawable's render opacity and whether it will draw.
        fn drawn(&self) -> Vec<(f32, bool)> {
            use crate::mechanical_port::source::generated::core_registry::CoreCapabilities;
            self.drawables
                .iter()
                .map(|&(drawable, _)| {
                    self.object(drawable)
                        .with(|object| {
                            (
                                object.as_transform_component().unwrap().render_opacity(),
                                object.drawable_will_draw(),
                            )
                        })
                        .unwrap()
                })
                .collect()
        }
        /// Each drawable's Fill's render opacity.
        fn fill_opacities(&self) -> Vec<f32> {
            self.drawables
                .iter()
                .map(|&(_, fill)| {
                    self.object(fill)
                        .with_downcast::<super::super::super::shapes::paint::fill::Fill, _>(
                            |fill| fill.base.render_opacity(),
                        )
                        .unwrap()
                })
                .collect()
        }
    }

    #[test]
    fn text_input_drawables_under_an_ancestor_at_opacity_0_do_not_draw() {
        let field = field(&[Ancestor::Node(0.0)], None);
        assert_eq!(field.drawn(), vec![(0.0, false); 4]);
    }

    #[test]
    fn text_input_drawables_under_an_ancestor_at_opacity_1_draw() {
        let field = field(&[Ancestor::Node(1.0)], None);
        assert_eq!(field.drawn(), vec![(1.0, true); 4]);
        assert_eq!(field.fill_opacities(), vec![1.0; 4]);
    }

    #[test]
    fn a_field_under_two_ancestors_fades_with_the_outer_one() {
        let field = field(&[Ancestor::Node(0.0), Ancestor::Node(1.0)], None);
        assert_eq!(field.drawn(), vec![(0.0, false); 4]);
    }

    #[test]
    fn a_field_under_a_layout_box_at_opacity_0_does_not_draw() {
        let field = field(&[Ancestor::Layout(0.0)], None);
        assert_eq!(field.drawn(), vec![(0.0, false); 4]);
    }

    #[test]
    fn a_half_faded_ancestor_reaches_the_field_paints() {
        let field = field(&[Ancestor::Node(0.5)], None);
        assert_eq!(field.drawn(), vec![(0.5, true); 4]);
        assert_eq!(field.fill_opacities(), vec![0.5; 4]);
    }

    /// Under an ancestor scaled 2 along x, a world offset of 10 is 5 in the
    /// space the text is drawn in, so the inner path's reversed copy of the
    /// text ends 5 past the text.
    #[test]
    fn an_inner_feather_on_a_field_rebuilds_in_the_drawn_space() {
        use crate::mechanical_port::source::shapes::paint::{
            feather::Feather, shape_paint::ShapePaintPathKind,
        };
        let field = field(&[Ancestor::ScaledX(2.0)], Some(10.0));
        let mut text = None;
        field
            .object(field.drawables[0].0)
            .with_downcast::<TextInputText, _>(|drawable| {
                drawable.with_path_mut(ShapePaintPathKind::Local, &mut |path| {
                    text = Some(path.raw_path().bounds());
                })
            })
            .unwrap();
        let text = text.expect("the text's path");
        assert!(text.width() > 0.0, "the field shaped its text");
        let inner = field
            .object(field.feather.unwrap())
            .with_downcast::<Feather, _>(|feather| {
                feather.inner_path().borrow().raw_path().bounds()
            })
            .unwrap();
        assert_eq!(inner.min_x, text.min_x);
        assert!(
            (inner.max_x - (text.max_x + 5.0)).abs() < 1e-4,
            "{inner:?} {text:?}"
        );
    }

    use crate::mechanical_port::source::generated::{
        text::text_style_base::TextStyleBase,
        world_transform_component_base::WorldTransformComponentBase,
    };
    const OPACITY: u16 = WorldTransformComponentBase::OPACITY_PROPERTY_KEY;

    #[test]
    fn a_box_set_to_opacity_0_before_the_first_advance_does_not_panic_and_hides_the_field() {
        let field = unadvanced(&[Ancestor::Layout(1.0)], None);
        assert!(field.write_double(field.ancestors[0], OPACITY, 0.0));
        field.artboard.advance_default(0.0);
        assert_eq!(field.drawn(), vec![(0.0, false); 4]);
    }

    #[test]
    fn a_box_set_to_opacity_0_after_a_font_change_and_before_the_update_hides_the_field() {
        let field = field(&[Ancestor::Layout(1.0)], None);
        assert!(field.write_uint(field.style, TextStyleBase::FONT_ASSET_ID_PROPERTY_KEY, 1));
        assert!(field.write_double(field.ancestors[0], OPACITY, 0.0));
        field.artboard.advance_default(0.0);
        assert_eq!(field.drawn(), vec![(0.0, false); 4]);
    }

    #[test]
    fn a_box_set_to_opacity_0_after_a_font_size_change_and_before_the_update_hides_the_field() {
        let field = field(&[Ancestor::Layout(1.0)], None);
        assert!(field.write_double(field.style, TextStyleBase::FONT_SIZE_PROPERTY_KEY, 24.0));
        assert!(field.write_double(field.ancestors[0], OPACITY, 0.0));
        field.artboard.advance_default(0.0);
        assert_eq!(field.drawn(), vec![(0.0, false); 4]);
    }

    #[test]
    fn a_keyframe_setting_a_box_to_opacity_0_before_the_first_advance_does_not_panic_and_hides_the_field()
     {
        let field = unadvanced(&[Ancestor::Layout(1.0)], None);
        field.artboard.animation_at(0).expect("the fade").apply(1.0);
        field.artboard.advance_default(0.0);
        assert_eq!(field.drawn(), vec![(0.0, false); 4]);
    }

    #[test]
    fn a_field_set_to_opacity_0_before_the_first_advance_does_not_draw() {
        let field = unadvanced(&[], None);
        assert!(field.write_double(field.input, OPACITY, 0.0));
        field.artboard.advance_default(0.0);
        assert_eq!(field.drawn(), vec![(0.0, false); 4]);
    }

    #[test]
    fn a_box_set_to_opacity_0_after_the_first_advance_hides_the_field() {
        let field = field(&[Ancestor::Layout(1.0)], None);
        assert!(field.write_double(field.ancestors[0], OPACITY, 0.0));
        field.artboard.advance_default(0.0);
        assert_eq!(field.drawn(), vec![(0.0, false); 4]);
    }
}
