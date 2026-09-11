use nuxie_runtime::source::{
    math::{raw_path::RawPath, vec2d::Vec2D},
    text::css_decoration::glyph_stripe_intercept,
};

#[test]
fn stripe_uses_ink_crossings_and_the_implicit_closing_edge() {
    let mut path = RawPath::default();
    path.move_to_point(Vec2D::new(0.0, 0.0));
    path.line_to_point(Vec2D::new(10.0, 10.0));
    path.line_to_point(Vec2D::new(0.0, 10.0));
    path.close();
    assert_eq!(glyph_stripe_intercept(&path, 2.0, 4.0), Some((0.0, 4.0)));
    assert_eq!(glyph_stripe_intercept(&path, 11.0, 12.0), None);
}

#[test]
fn stripe_finds_both_sides_of_quadratic_and_cubic_arches() {
    for cubic in [false, true] {
        let mut path = RawPath::default();
        path.move_to_point(Vec2D::new(0.0, 0.0));
        if cubic {
            // x=12t, y=12t(1-t): y=2.25 at t=1/4 and 3/4.
            path.cubic_to_points(
                Vec2D::new(4.0, 4.0),
                Vec2D::new(8.0, 4.0),
                Vec2D::new(12.0, 0.0),
            );
        } else {
            // x=10t, y=20t(1-t): y=3.75 at t=1/4 and 3/4.
            path.quad_to_points(Vec2D::new(5.0, 10.0), Vec2D::new(10.0, 0.0));
        }
        path.close();
        let height = if cubic { 2.25 } else { 3.75 };
        let expected = if cubic { (3.0, 9.0) } else { (2.5, 7.5) };
        let actual =
            glyph_stripe_intercept(&path, height, height).expect("curve crosses the stripe");
        assert!((actual.0 - expected.0).abs() < 0.0001);
        assert!((actual.1 - expected.1).abs() < 0.0001);
    }
}

#[test]
fn stripe_keeps_outer_crossings_of_a_three_crossing_cubic() {
    let mut path = RawPath::default();
    // y=64(t-1/4)(t-1/2)(t-3/4), x=12t. Three crossings;
    // one glyph interval must cover the two outermost crossings.
    path.move_to_point(Vec2D::new(0.0, -6.0));
    path.cubic_to_points(
        Vec2D::new(4.0, 26.0 / 3.0),
        Vec2D::new(8.0, -26.0 / 3.0),
        Vec2D::new(12.0, 6.0),
    );
    let span = glyph_stripe_intercept(&path, 0.0, 0.0).unwrap();
    assert!((span.0 - 3.0).abs() < 0.0001, "{span:?}");
    assert!((span.1 - 9.0).abs() < 0.0001, "{span:?}");
}

#[test]
fn stripe_rejects_invalid_inputs_and_ignores_disjoint_contours() {
    let mut path = RawPath::default();
    path.add_poly(
        &[
            Vec2D::new(2.0, 1.0),
            Vec2D::new(4.0, 1.0),
            Vec2D::new(4.0, 5.0),
            Vec2D::new(2.0, 5.0),
        ],
        true,
    );
    path.add_poly(
        &[
            Vec2D::new(100.0, 10.0),
            Vec2D::new(200.0, 10.0),
            Vec2D::new(200.0, 20.0),
        ],
        true,
    );
    assert_eq!(glyph_stripe_intercept(&path, 2.0, 3.0), Some((2.0, 4.0)));
    assert_eq!(glyph_stripe_intercept(&path, 3.0, 2.0), None);
    assert_eq!(glyph_stripe_intercept(&path, f32::NAN, 3.0), None);
    assert_eq!(glyph_stripe_intercept(&path, 2.0, f32::INFINITY), None);
    assert_eq!(glyph_stripe_intercept(&RawPath::default(), 2.0, 3.0), None);
}

#[test]
fn inter_descender_gaps_agree_with_chromium_reference_edges() {
    use nuxie_runtime::source::{text::font_hb::HbFont, text_engine::TextRun};
    let font_path = std::env::var_os("NUXIE_TEXT_TEST_FONT")
        .expect("Set NUXIE_TEXT_TEST_FONT to the pinned Inter fixture for this reference test");
    let bytes = std::fs::read(font_path).expect("read pinned Inter fixture");
    use sha2::{Digest, Sha256};
    assert_eq!(
        format!("{:x}", Sha256::digest(&bytes)),
        "3e5f90a0138b38de4cf4d779ad78391974ea1df776b9164842bdcbb60ce383c5",
        "Chrome gap reference requires the exact pinned Inter font"
    );
    let font = HbFont::decode(&bytes).unwrap();
    let chars: Vec<u32> = "agypqj M".chars().map(u32::from).collect();
    let shape = font.shape_text(
        &chars,
        &[TextRun {
            font: Some(font.clone()),
            size: 20.0,
            line_height: 60.0,
            letter_spacing: 0.0,
            unichar_count: chars.len() as u32,
            script: u32::from_be_bytes(*b"Latn"),
            style_id: 0,
            level: 0,
        }],
        0,
    );
    // Keep unsnapped exclusions for the renderer, including the vertical
    // outset Chrome applies after measuring the inset stripe. Moving a line
    // must translate both stripe and exclusions without changing their widths.
    use nuxie_runtime::source::{
        text::css_decoration::{ResolvedUnderline, SkipInk},
        text_engine::{GlyphLine, GlyphRun, OrderedLine},
    };
    let line = GlyphLine {
        end_run_index: shape[0].runs.len() as u32 - 1,
        end_glyph_index: shape[0].runs.last().unwrap().glyphs.len() as u32,
        start_x: 0.25,
        ..GlyphLine::default()
    };
    let ordered = OrderedLine::new(
        &shape[0],
        &line,
        300.0,
        false,
        false,
        &mut GlyphRun::default(),
        30.5,
    );
    let strike = nuxie_runtime::source::text::css_decoration::ResolvedStrikethrough::solid(
        0xff0000ff, 2.0, -7.25, 20.0,
    )
    .unwrap()
    .build_stripes(std::slice::from_ref(&ordered));
    assert_eq!(strike.len(), 1);
    assert_eq!(strike[0].bounds.min_x, 0.25);
    assert_eq!(strike[0].bounds.min_y, 23.25);
    assert_eq!(strike[0].bounds.max_y, 25.25);

    let underline = ResolvedUnderline::solid(0xffff0000, 2.0, 1.0, SkipInk::All).unwrap();
    let stripes = underline.build_stripes(std::slice::from_ref(&ordered), &chars);
    assert_eq!(stripes.len(), 1);
    let stripe = &stripes[0];
    assert_eq!(
        (
            stripe.bounds.min_x,
            stripe.bounds.min_y,
            stripe.bounds.max_y
        ),
        (0.25, 31.5, 33.5)
    );
    assert!(!stripe.exclusions.is_empty());
    assert!(
        stripe
            .exclusions
            .iter()
            .all(|clip| clip.min_y == 31.0 && clip.max_y == 34.0)
    );
    assert!(
        stripe
            .exclusions
            .iter()
            .any(|clip| clip.min_x.fract() != 0.0)
    );
    let plain = ResolvedUnderline::solid(0xffff0000, 2.0, 1.0, SkipInk::None)
        .unwrap()
        .build_stripes(std::slice::from_ref(&ordered), &chars);
    assert_eq!(plain[0].bounds, stripe.bounds);
    assert!(plain[0].exclusions.is_empty());
    let shifted = OrderedLine::new(
        &shape[0],
        &GlyphLine {
            start_x: 7.5,
            ..line
        },
        300.0,
        false,
        false,
        &mut GlyphRun::default(),
        41.75,
    );
    let moved_strike = nuxie_runtime::source::text::css_decoration::ResolvedStrikethrough::solid(
        0xff0000ff, 2.0, -7.25, 20.0,
    )
    .unwrap()
    .build_stripes(std::slice::from_ref(&shifted));
    assert_eq!(moved_strike[0].bounds.min_x - strike[0].bounds.min_x, 7.25);
    assert_eq!(moved_strike[0].bounds.min_y - strike[0].bounds.min_y, 11.25);
    assert!((moved_strike[0].bounds.width() - strike[0].bounds.width()).abs() < 0.00001);

    let moved = underline.build_stripes(&[shifted], &chars);
    assert_eq!(moved[0].exclusions.len(), stripe.exclusions.len());
    for (original, shifted) in stripe.exclusions.iter().zip(&moved[0].exclusions) {
        assert!((shifted.min_x - original.min_x - 7.25).abs() < 0.00001);
        assert!((shifted.max_x - original.max_x - 7.25).abs() < 0.00001);
        assert_eq!(shifted.min_y - original.min_y, 11.25);
        assert_eq!(shifted.max_y - original.max_y, 11.25);
    }
    let mut gaps: Vec<(f32, f32)> = Vec::new();
    let mut x = 0.0;
    for run in &shape[0].runs {
        for (index, &glyph) in run.glyphs.iter().enumerate() {
            if let Some((start, end)) =
                glyph_stripe_intercept(&font.get_path(glyph), 1.5 / 20.0, 2.5 / 20.0)
            {
                let next = (
                    x + run.offsets[index].x + start * 20.0 - 2.0,
                    x + run.offsets[index].x + end * 20.0 + 2.0,
                );
                if let Some(last) = gaps.last_mut().filter(|last| last.1 >= next.0) {
                    last.1 = last.1.max(next.1);
                } else {
                    gaps.push(next);
                }
            }
            x += run.advances[index];
        }
    }
    // Chrome's independently captured red scanlines have gaps [10,31),
    // [34,39), [53,63). Compare geometric edges to the painted pixel edges;
    // this checks outlines, not final native rasterization (which is pending).
    let expected = [(10.0, 31.0), (34.0, 39.0), (53.0, 63.0)];
    assert_eq!(gaps.len(), expected.len(), "{gaps:?}");
    for (actual, expected) in gaps.iter().zip(expected) {
        assert!(
            (actual.0 - expected.0).abs() <= 1.0 && (actual.1 - expected.1).abs() <= 1.0,
            "{gaps:?}"
        );
    }
}

#[test]
fn resolved_underline_rejects_invalid_metrics_and_allows_offsets_above_baseline() {
    use nuxie_runtime::source::text::css_decoration::{ResolvedUnderline, SkipInk};
    for thickness in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, 0.0, -1.0] {
        assert!(ResolvedUnderline::solid(0xff123456, thickness, 1.0, SkipInk::Auto).is_none());
    }
    for offset in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        assert!(ResolvedUnderline::solid(0xff123456, 2.0, offset, SkipInk::Auto).is_none());
    }
    for offset in [-2.0, 0.0, 3.0] {
        assert_eq!(
            ResolvedUnderline::solid(0xff123456, 2.0, offset, SkipInk::Auto)
                .unwrap()
                .color(),
            0xff123456
        );
    }
}

#[test]
fn resolved_strikethrough_rejects_nonfinite_metrics() {
    use nuxie_runtime::source::text::css_decoration::ResolvedStrikethrough;
    for thickness in [0.0, -1.0, f32::NAN, f32::INFINITY] {
        assert!(ResolvedStrikethrough::solid(0xff123456, thickness, -7.0, 20.0).is_none());
    }
    for offset in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
        assert!(ResolvedStrikethrough::solid(0xff123456, 2.0, -7.0, offset).is_none());
        assert!(ResolvedStrikethrough::solid(0xff123456, 2.0, offset, 20.0).is_none());
    }
    assert_eq!(
        ResolvedStrikethrough::solid(0xff123456, 2.0, -7.0, 20.0)
            .unwrap()
            .color(),
        0xff123456
    );
}

#[test]
fn css_ellipsis_decorates_retained_source_but_not_synthetic_marker() {
    use nuxie_runtime::source::{
        text::css_decoration::{ResolvedUnderline, ResolvedStrikethrough, SkipInk},
        text_engine::{GlyphLine, GlyphRun, OrderedLine, Paragraph},
    };
    let mut source = GlyphRun::new(2);
    source.advances = vec![7.65625, 7.65625];
    let mut marker = GlyphRun::new(1);
    marker.advances = vec![24.0];
    let line = GlyphLine { start_x: 0.25, end_glyph_index: 2, ..GlyphLine::default() };
    let underline = ResolvedUnderline::solid(0xffff0000, 2.0, 1.0, SkipInk::None).unwrap();
    let strike = ResolvedStrikethrough::solid(0xffff0000, 2.0, -7.0, 20.0).unwrap();
    let ordered = OrderedLine::from_css_runs(vec![source.clone(), marker], &line, 30.0);
    assert_eq!((&ordered).into_iter().count(), 3, "marker must still paint");
    let under = underline.build_stripes(std::slice::from_ref(&ordered), &[]);
    let through = strike.build_stripes(std::slice::from_ref(&ordered));
    assert_eq!(under.len(), 1);
    assert_eq!(through.len(), 1);
    assert_eq!(under[0].bounds.max_x, 15.5625);
    assert_eq!(through[0].bounds.max_x, 15.5625);
    let paragraph = Paragraph { runs: vec![source], level: 0 };
    let normal = OrderedLine::new(&paragraph, &line, 100.0, false, false, &mut GlyphRun::default(), 30.0);
    assert_eq!(underline.build_stripes(&[normal], &[])[0].bounds.max_x, 15.5625);
}
