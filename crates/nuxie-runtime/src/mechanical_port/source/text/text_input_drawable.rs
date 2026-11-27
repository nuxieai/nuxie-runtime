use crate::mechanical_port::source::{
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
    pub fn on_added_clean(&mut self, _context: &mut dyn CoreContext) -> StatusCode {
        if self.base.parent_handle().is_some_and(|parent| {
            parent
                .is_type_of(crate::mechanical_port::source::generated::text::text_input_base::TextInputBase::TYPE_KEY)
        }) {
            StatusCode::Ok
        } else {
            StatusCode::InvalidObject
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
}
