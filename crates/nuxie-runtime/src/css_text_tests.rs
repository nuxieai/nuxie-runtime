use super::*;
use crate::source::{
    advance_flags::AdvanceFlags,
    generated::core_registry::CoreRegistry,
    math::vec2d::Vec2D,
    text::{
        text::Text, text_modifier_group::TextModifierGroup,
        text_selection_controller::TextSelectionController, text_value_run::TextValueRun,
    },
    viewmodel::{
        viewmodel_instance::ViewModelInstance, viewmodel_instance_string::ViewModelInstanceString,
    },
};
use crate::{Artboard, ArtboardInstance, File, RuntimeFactoryHandle, RuntimeFileHandle};
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

struct Scene {
    file: RuntimeFileHandle,
    board: ArtboardInstance,
}
impl Scene {
    fn new(
        runs: &[(&str, Option<(u32, u32)>)],
        css: bool,
        width: f32,
        wrap: bool,
        bound: bool,
        modifier: bool,
    ) -> Self {
        use FixtureValue::{Bytes, Double, String as Str, Uint};
        let mut records = vec![record("Backboard", vec![])];
        if bound {
            records.extend([
                record("ViewModel", vec![("name", Str("Product".into()))]),
                record(
                    "ViewModelPropertyString",
                    vec![("name", Str("name".into()))],
                ),
            ]);
        }
        records.extend([
            record("FontAsset", vec![("assetId", Uint(0))]),
            record(
                "FileAssetContents",
                vec![(
                    "bytes",
                    Bytes(include_bytes!("../../../fixtures/fonts/roboto-input.ttf").to_vec()),
                )],
            ),
        ]);
        let artboard = records.len();
        records.push(record(
            "Artboard",
            vec![
                ("width", Double(width)),
                ("height", Double(400.0)),
                ("styleId", Uint(1)),
                ("viewModelId", Uint(0)),
            ],
        ));
        records.push(record("LayoutComponentStyle", vec![("parentId", Uint(0))]));
        if css {
            records.push(record("LayoutCssStyle", vec![("parentId", Uint(0))]));
        }
        let text_id = (records.len() - artboard) as u64;
        records.push(record(
            "Text",
            vec![
                ("parentId", Uint(0)),
                ("sizingValue", Uint(2)),
                ("width", Double(width)),
                ("height", Double(400.0)),
                ("wrapValue", Uint(if wrap { 0 } else { 1 })),
            ],
        ));
        let mut last_run = 0;
        for (index, (source, options)) in runs.iter().enumerate() {
            let style = (records.len() - artboard) as u64;
            records.extend([
                record(
                    "TextStylePaint",
                    vec![
                        ("parentId", Uint(text_id)),
                        ("fontSize", Double(20.0)),
                        ("fontAssetId", Uint(0)),
                        ("lineHeight", Double(24.0)),
                    ],
                ),
                record("Fill", vec![("parentId", Uint(style))]),
                record("SolidColor", vec![("parentId", Uint(style + 1))]),
            ]);
            if let Some((transform, whitespace)) = options {
                records.push(record(
                    "TextStyleCssText",
                    vec![
                        ("parentId", Uint(style)),
                        ("textTransform", Uint(u64::from(*transform))),
                        ("whiteSpace", Uint(u64::from(*whitespace))),
                    ],
                ));
            }
            last_run = (records.len() - artboard) as u64;
            records.push(record(
                "TextValueRun",
                vec![
                    ("parentId", Uint(text_id)),
                    ("styleId", Uint(style)),
                    ("text", Str((*source).into())),
                ],
            ));
            if bound && index == 0 {
                records.push(record(
                    "DataBindContext",
                    vec![
                        ("propertyKey", Uint(268)),
                        ("sourcePathIds", Bytes(vec![0, 0])),
                    ],
                ));
            }
        }
        if modifier {
            let group = (records.len() - artboard) as u64;
            records.push(record(
                "TextModifierGroup",
                vec![("parentId", Uint(text_id))],
            ));
            records.push(record(
                "TextModifierRange",
                vec![("parentId", Uint(group)), ("runId", Uint(last_run))],
            ));
        }
        let bytes =
            encode_runtime_file(&RuntimeFile::from_fixture_records(records).unwrap()).unwrap();
        let mut factory = PersistentFactory::new(RecordingFactory::new());
        let file = File::import(
            &bytes,
            RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
            None,
            None,
            None,
        )
        .expect("CSS text imports");
        let board = ArtboardInstance::from_native(file.clone(), 0).unwrap();
        let mut result = Self { file, board };
        result.advance();
        result
    }
    fn advance(&mut self) {
        for _ in 0..3 {
            Artboard::advance_handle(
                &self.board.native_handle().core_handle(),
                0.0,
                AdvanceFlags::ADVANCE_NESTED | AdvanceFlags::ANIMATE | AdvanceFlags::NEW_FRAME,
            );
        }
    }
    fn handles<T: CoreType>(&self) -> Vec<CoreHandle> {
        self.board
            .native_handle()
            .with_artboard(|a| a.find_all_handles::<T>())
    }
    fn text(&self) -> CoreHandle {
        self.handles::<Text>().remove(0)
    }
    fn display(&self) -> String {
        self.text()
            .with_downcast::<Text, _>(|t| {
                t.unichars()
                    .iter()
                    .map(|&cp| char::from_u32(cp).unwrap())
                    .collect()
            })
            .unwrap()
    }
    fn bind(&mut self, value: &str) -> CoreHandle {
        let native = self.board.native_handle();
        let vm = self
            .file
            .with_file(|f| f.create_view_model_instance_for_artboard(native.core_handle()))
            .unwrap();
        let property = vm
            .with_downcast::<ViewModelInstance, _>(|vm| vm.property_values()[0].clone())
            .unwrap();
        property.with_downcast_mut::<ViewModelInstanceString, _>(|p| p.set_value(value));
        native.bind_view_model_instance(Some(vm));
        self.advance();
        property
    }
    fn line_widths(&self) -> Vec<f32> {
        self.text()
            .with_downcast::<Text, _>(|t| {
                t.ordered_lines()
                    .iter()
                    .map(|line| {
                        line.into_iter()
                            .map(|(run, index)| run.advances[index as usize])
                            .sum()
                    })
                    .collect()
            })
            .unwrap()
    }
}

#[test]
fn css_text_bound_product_name_retransforms_after_binding_changes() {
    let mut scene = Scene::new(
        &[("fallback", Some((1, 0)))],
        true,
        300.0,
        true,
        true,
        false,
    );
    let name = scene.bind("Annual plan");
    assert_eq!(scene.display(), "ANNUAL PLAN");
    name.with_downcast_mut::<ViewModelInstanceString, _>(|p| p.set_value("Monthly plan"));
    scene.advance();
    assert_eq!(scene.display(), "MONTHLY PLAN");
    assert_eq!(
        scene.handles::<TextValueRun>()[0]
            .with_downcast::<TextValueRun, _>(|r| r.base.text().to_owned())
            .unwrap(),
        "Monthly plan"
    );
}

#[test]
fn css_text_lowercase_and_capitalize_cross_run_boundaries() {
    let lower = Scene::new(
        &[("İ HELLO Ο", Some((2, 0))), ("Σ", Some((2, 0)))],
        true,
        300.0,
        true,
        false,
        false,
    );
    assert_eq!(lower.display(), "i\u{307} hello ος");
    let capital = Scene::new(
        &[("hello ", Some((3, 0))), ("world", Some((3, 0)))],
        true,
        300.0,
        true,
        false,
        false,
    );
    assert_eq!(capital.display(), "Hello World");
    let continued = Scene::new(
        &[("hel", Some((3, 0))), ("lo world", Some((3, 0)))],
        true,
        300.0,
        true,
        false,
        false,
    );
    assert_eq!(continued.display(), "Hello World");
}

#[test]
fn css_text_capitalize_uses_full_unicode_titlecase() {
    for (source, expected) in [
        ("ǳuro", "ǲuro"),
        ("ßeta", "Sseta"),
        ("ﬀoo", "Ffoo"),
        ("ǆuro", "ǅuro"),
    ] {
        let scene = Scene::new(&[(source, Some((3, 0)))], true, 300.0, true, false, false);
        assert_eq!(scene.display(), expected, "capitalize {source}");
    }
}

#[test]
fn css_text_capitalize_preserves_uppercase_and_titlecase_initials() {
    let source = "Ǳuro ǲuro Ǆuro ǅuro";
    let scene = Scene::new(&[(source, Some((3, 0)))], true, 300.0, true, false, false);
    assert_eq!(scene.display(), source);
}

#[test]
fn css_text_capitalize_expansion_keeps_following_modifier_range_in_display_indices() {
    let scene = Scene::new(
        &[("ß", Some((3, 0))), ("eta", Some((3, 0)))],
        true,
        300.0,
        true,
        false,
        true,
    );
    assert_eq!(scene.display(), "Sseta");
    let runs = scene.handles::<TextValueRun>();
    assert_eq!(
        runs[0].with_downcast_mut::<TextValueRun, _>(|run| run.length()),
        Some(2)
    );
    assert_eq!(
        runs[1].with_downcast_mut::<TextValueRun, _>(|run| (run.offset(), run.length())),
        Some((2, 3))
    );
    let coverage = scene.handles::<TextModifierGroup>()[0]
        .with_downcast::<TextModifierGroup, _>(|group| {
            (0..5)
                .map(|index| group.glyph_coverage(index, 1))
                .collect::<Vec<_>>()
        })
        .unwrap();
    assert_eq!(coverage, vec![0.0, 0.0, 1.0, 1.0, 1.0]);
    scene.text().with_downcast::<Text, _>(|text| {
        let styled = text.styled_text();
        assert_eq!(styled.source_to_display(1), 2);
        assert_eq!(styled.display_to_source(2), 1);
        assert_eq!(styled.source_to_display(4), 5);
    });
}

#[test]
fn css_text_sharp_s_expansion_keeps_modifier_run_coverage_in_display_indices() {
    let scene = Scene::new(
        &[("ß", Some((1, 0))), ("ab", Some((1, 0)))],
        true,
        300.0,
        true,
        false,
        true,
    );
    assert_eq!(scene.display(), "SSAB");
    let runs = scene.handles::<TextValueRun>();
    assert_eq!(
        runs[0].with_downcast_mut::<TextValueRun, _>(|r| r.length()),
        Some(2)
    );
    assert_eq!(
        runs[1].with_downcast_mut::<TextValueRun, _>(|r| (r.offset(), r.length())),
        Some((2, 2))
    );
    let coverage = scene.handles::<TextModifierGroup>()[0]
        .with_downcast::<TextModifierGroup, _>(|g| {
            (
                g.glyph_coverage(0, 1),
                g.glyph_coverage(1, 1),
                g.glyph_coverage(2, 1),
                g.glyph_coverage(3, 1),
            )
        })
        .unwrap();
    assert_eq!(coverage, (0.0, 0.0, 1.0, 1.0));
}

#[test]
fn css_text_white_space_modes_match_browser_strings_and_wraps() {
    for (mode, expected) in [
        (0, "a b"),
        (1, "  a  \n b  "),
        (2, "  a  \n b  "),
        (3, "a\nb"),
        (4, "a b"),
    ] {
        let scene = Scene::new(
            &[("  a  \n b  ", Some((0, mode)))],
            true,
            300.0,
            mode != 1 && mode != 4,
            false,
            false,
        );
        assert_eq!(scene.display(), expected, "white-space {mode}");
    }
    let pre = Scene::new(
        &[("one two three four", Some((0, 1)))],
        true,
        65.0,
        false,
        false,
        false,
    );
    let pre_wrap = Scene::new(
        &[("one two three four", Some((0, 2)))],
        true,
        65.0,
        true,
        false,
        false,
    );
    assert_eq!(pre.line_widths().len(), 1);
    assert!(pre_wrap.line_widths().len() > 1);
    let across = Scene::new(
        &[("a ", Some((0, 0))), (" b", Some((0, 0)))],
        true,
        300.0,
        true,
        false,
        false,
    );
    assert_eq!(across.display(), "a b");
    let normalized = Scene::new(
        &[("a\r", Some((0, 3))), ("\n b", Some((0, 3)))],
        true,
        300.0,
        true,
        false,
        false,
    );
    assert_eq!(normalized.display(), "a\nb");
}

#[test]
fn css_text_bound_whitespace_collapses_after_binding_changes() {
    let mut scene = Scene::new(
        &[("fallback", Some((0, 0)))],
        true,
        300.0,
        true,
        true,
        false,
    );
    let value = scene.bind("  a  \n b  ");
    assert_eq!(scene.display(), "a b");
    value.with_downcast_mut::<ViewModelInstanceString, _>(|p| p.set_value("x  \n y"));
    scene.advance();
    assert_eq!(scene.display(), "x y");
}

#[test]
fn css_text_soft_lines_trim_collapsible_spaces_at_both_edges() {
    let scene = Scene::new(
        &[("  aa   aa   aa  ", Some((0, 0)))],
        true,
        30.0,
        true,
        false,
        false,
    );
    assert_eq!(scene.line_widths().len(), 3);
    scene.text().with_downcast::<Text, _>(|text| {
        for line in text.ordered_lines() {
            let glyphs: Vec<_> = line.into_iter().collect();
            assert!(!glyphs.is_empty());
            for &(run, index) in [glyphs.first().unwrap(), glyphs.last().unwrap()] {
                assert_eq!(
                    text.unichars()[run.text_indices[index as usize] as usize],
                    u32::from('a')
                );
            }
        }
    });
    assert!(scene.line_widths().iter().all(|w| *w <= 30.0));
}

#[test]
fn css_text_instances_and_runtime_property_and_style_changes_are_independent() {
    let mut upper = Scene::new(
        &[("Hello", Some((1, 0))), ("World", Some((2, 0)))],
        true,
        300.0,
        true,
        false,
        false,
    );
    let board = ArtboardInstance::from_native(upper.file.clone(), 0).unwrap();
    let mut lower = Scene {
        file: upper.file.clone(),
        board,
    };
    let record = lower.handles::<TextStyleCssText>().remove(0);
    CoreRegistry::set_uint_handle(&record, i32::from(TEXT_TRANSFORM_PROPERTY_KEY), 2);
    lower.advance();
    assert_eq!(lower.display(), "helloworld");
    assert_eq!(upper.display(), "HELLOworld");
    CoreRegistry::set_uint_handle(&record, i32::from(TEXT_TRANSFORM_PROPERTY_KEY), 1);
    lower.advance();
    assert_eq!(lower.display(), "HELLOworld");
    let second_style =
        upper.handles::<crate::source::text::text_style_paint::TextStylePaint>()[1].clone();
    let style_id = upper.board.native_handle().with_artboard(|a| {
        a.objects()
            .iter()
            .position(|h| h.as_ref() == Some(&second_style))
            .unwrap()
    });
    let run = upper.handles::<TextValueRun>().remove(0);
    CoreRegistry::set_uint_handle(&run, 272, style_id as u32);
    upper.advance();
    assert_eq!(upper.display(), "helloworld");
}

#[test]
fn css_text_runtime_white_space_property_rebuilds_display_and_lines() {
    let mut scene = Scene::new(
        &[("a  \n b", Some((0, 0)))],
        true,
        300.0,
        true,
        false,
        false,
    );
    assert_eq!(scene.display(), "a b");
    let record = scene.handles::<TextStyleCssText>().remove(0);
    assert!(CoreRegistry::set_uint_handle(
        &record,
        i32::from(WHITE_SPACE_PROPERTY_KEY),
        1
    ));
    scene.advance();
    assert_eq!(scene.display(), "a  \n b");
    assert_eq!(scene.line_widths().len(), 2);
    assert_eq!(
        CoreRegistry::get_uint_handle(&record, i32::from(WHITE_SPACE_PROPERTY_KEY)),
        Some(1)
    );
}

#[test]
fn css_text_source_selection_carets_and_hits_follow_expansions_and_collapsing() {
    let mut scene = Scene::new(&[("ß  ab", Some((1, 0)))], true, 300.0, true, false, false);
    let text = scene.text();
    let id = scene.board.native_handle().with_artboard(|a| {
        a.objects()
            .iter()
            .position(|h| h.as_ref() == Some(&text))
            .unwrap()
    });
    let start = scene.board.text_caret(id, "ß  ".len()).unwrap();
    let end = scene.board.text_caret(id, "ß  a".len()).unwrap();
    assert!(end.top.x > start.top.x);
    assert_eq!(scene.board.text_hit(id, start.top), Some("ß  ".len()));
    let rects = scene
        .board
        .text_selection_rects(id, "ß  ".len().."ß  a".len());
    assert_eq!(rects.len(), 1);
    assert!((rects[0].min_x - start.top.x).abs() < 0.01);
    assert!((rects[0].max_x - end.top.x).abs() < 0.01);
    let collapsed = scene.board.text_caret(id, "ß ".len()).unwrap();
    assert!((collapsed.top.x - start.top.x).abs() < 0.01);
    let mut selection = TextSelectionController::new();
    assert!(selection.add(&text, ""));
    assert!(selection.select(&text, 3, &text, 4));
    assert_eq!(selection.selected_text(), "a");
    assert_eq!(selection.selection(&text).first().code_point_index(), 3);
    let local_rects = selection.selection_rects(&text);
    assert_eq!(local_rects.len(), 1);
    let mut offset = 0;
    let mut distance = 0.0;
    assert!(selection.hit_test(
        &text,
        Vec2D::new(start.top.x, (start.top.y + start.bottom.y) / 2.0),
        true,
        &mut offset,
        &mut distance
    ));
    assert_eq!(offset, 3);
    selection.set_range(&text, 3, 4);
    assert_eq!(selection.selected_text(), "a");
    let mid =
        |caret: crate::CaretGeometry| Vec2D::new(caret.top.x, (caret.top.y + caret.bottom.y) / 2.0);
    assert!(selection.pointer_down(Some(&text), mid(start), 7, false));
    assert!(selection.pointer_up(mid(end), 7));
    assert_eq!(selection.selected_text(), "a");
    assert_eq!(selection.source_text(&text), "ß  ab");
    selection.select_all();
    assert_eq!(selection.selected_text(), "ß  ab");
    let last = scene.board.text_caret(id, "ß  ab".len()).unwrap();
    assert_eq!(scene.board.text_hit(id, last.top), Some("ß  ab".len()));
    selection.set_range(&text, 3, 4);
    let record = scene.handles::<TextStyleCssText>().remove(0);
    CoreRegistry::set_uint_handle(&record, i32::from(TEXT_TRANSFORM_PROPERTY_KEY), 2);
    scene.advance();
    assert_eq!(scene.display(), "ß ab");
    assert_eq!(selection.selected_text(), "a");
    assert_eq!(selection.selection(&text).first().code_point_index(), 3);
    let caret = scene.board.text_caret(id, "ß  ".len()).unwrap();
    assert!((selection.selection_rects(&text)[0].min_x - caret.top.x).abs() < 0.01);
}

#[test]
fn css_text_translated_text_and_styles_without_records_stay_unchanged() {
    let source = "  ß Hello  \n World  ";
    for (owner, options) in [(false, Some((1, 0))), (true, None), (false, None)] {
        let scene = Scene::new(&[(source, options)], owner, 300.0, true, false, false);
        assert_eq!(scene.display(), source);
        scene
            .text()
            .with_downcast::<Text, _>(|t| assert!(t.styled_text().css_indices().is_none()));
    }
}

#[test]
fn css_text_index_map_covers_every_source_boundary() {
    let source: Vec<_> = "  ß  ab  ".chars().map(u32::from).collect();
    let (display, map) = display_text(&source, &vec![Some((1, 0)); source.len()]);
    assert_eq!(display, "SS AB".chars().map(u32::from).collect::<Vec<_>>());
    assert_eq!(
        (0..=source.len() as u32)
            .map(|i| map.to_display(i))
            .collect::<Vec<_>>(),
        vec![0, 0, 0, 2, 3, 3, 4, 5, 5, 5]
    );
    assert_eq!(map.to_source(3), 5);
    assert_eq!(map.to_source(5), 9);
}

#[test]
fn css_text_tabs_collapse_but_nonbreaking_spaces_are_preserved() {
    let scene = Scene::new(
        &[("a\t  \nb\u{a0}  c", Some((0, 0)))],
        true,
        300.0,
        true,
        false,
        false,
    );
    assert_eq!(scene.display(), "a b\u{a0} c");
}
