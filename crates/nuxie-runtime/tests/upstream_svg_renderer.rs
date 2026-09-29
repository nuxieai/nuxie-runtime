//! Exact assertions from tests/unit_tests/runtime/svg_renderer_test.cpp.
use nuxie_render_api::{
    BlendMode, Factory, FillRule, Mat2D, PersistentFactory, RawPath, RenderPaint, RenderPaintStyle,
    RenderPath, Renderer, StrokeCap, StrokeJoin, Vec2D,
};
use nuxie_runtime::{
    File, RuntimeFactoryHandle,
    utils::{svg_factory::SVGFactory, svg_renderer::SVGRenderer},
};

fn body(out: &str) -> &str {
    let end = out.rfind("</svg>").expect("closing svg tag");
    let start = if let Some(defs_close) = out.find("</defs>") {
        defs_close + "</defs>".len()
    } else {
        out.find('>').expect("opening svg tag") + 1
    };
    &out[start..end]
}

fn count_occurrences(s: &str, needle: &str) -> usize {
    s.matches(needle).count()
}

fn make_fill_paint(f: &mut SVGFactory, color: u32, blend: BlendMode) -> Box<dyn RenderPaint> {
    let mut p = f.make_render_paint();
    p.style(RenderPaintStyle::Fill);
    p.color(color);
    p.blend_mode(blend);
    p
}

fn make_stroke_paint(
    f: &mut SVGFactory,
    color: u32,
    thickness: f32,
    join: StrokeJoin,
    cap: StrokeCap,
) -> Box<dyn RenderPaint> {
    let mut p = f.make_render_paint();
    p.style(RenderPaintStyle::Stroke);
    p.color(color);
    p.thickness(thickness);
    p.join(join);
    p.cap(cap);
    p
}

fn make_square_path(f: &mut SVGFactory) -> Box<dyn RenderPath> {
    let mut rp = RawPath::default();
    rp.move_to(0.0, 0.0);
    rp.line(Vec2D::new(1.0, 0.0));
    rp.line(Vec2D::new(1.0, 1.0));
    rp.line(Vec2D::new(0.0, 1.0));
    rp.close();
    f.make_render_path(rp, FillRule::NonZero)
}

#[test]
fn empty_save_restore_produces_no_group_wrappers() {
    let mut r = SVGRenderer::default();
    r.save();
    r.restore();
    let out = r.finalize(10, 10);
    let b = body(&out);
    assert_eq!(count_occurrences(b, "<g"), 0);
    assert_eq!(count_occurrences(b, "</g>"), 0);
}

#[test]
fn pure_translation_inlines_on_path() {
    let mut f = SVGFactory::default();
    let mut r = SVGRenderer::default();
    let path = make_square_path(&mut f);
    let paint = make_fill_paint(&mut f, 0xffff0000, BlendMode::SrcOver);
    r.save();
    r.transform(Mat2D([1.0, 0.0, 0.0, 1.0, 10.0, 20.0]));
    r.draw_path(path.as_ref(), paint.as_ref());
    r.restore();
    let out = r.finalize(64, 64);
    let b = body(&out);
    assert!(b.contains("transform=\"translate(10 20)\""));
    assert!(!b.contains("matrix("));
    assert_eq!(count_occurrences(b, "<g"), 0);
    assert_eq!(count_occurrences(b, "<path"), 1);
}

#[test]
fn identity_transform_is_omitted() {
    let mut f = SVGFactory::default();
    let mut r = SVGRenderer::default();
    let path = make_square_path(&mut f);
    let paint = make_fill_paint(&mut f, 0xffff0000, BlendMode::SrcOver);
    r.save();
    r.transform(Mat2D::IDENTITY);
    r.draw_path(path.as_ref(), paint.as_ref());
    r.restore();
    let out = r.finalize(64, 64);
    let b = body(&out);
    assert!(!b.contains("transform="));
    assert_eq!(count_occurrences(b, "<path"), 1);
}

#[test]
fn nested_transforms_multiply_into_single_matrix() {
    let mut f = SVGFactory::default();
    let mut r = SVGRenderer::default();
    let path = make_square_path(&mut f);
    let paint = make_fill_paint(&mut f, 0xffff0000, BlendMode::SrcOver);
    let a = Mat2D([1.0, 0.0, 0.0, 1.0, 10.0, 20.0]);
    let b = Mat2D([2.0, 0.0, 0.0, 3.0, 0.0, 0.0]);
    use nuxie_runtime::source::math::mat2d::Mat2D as SourceMat2D;
    let expected = SourceMat2D::new(1.0, 0.0, 0.0, 1.0, 10.0, 20.0)
        * SourceMat2D::new(2.0, 0.0, 0.0, 3.0, 0.0, 0.0);
    r.save();
    r.transform(a);
    r.save();
    r.transform(b);
    r.draw_path(path.as_ref(), paint.as_ref());
    r.restore();
    r.restore();
    let out = r.finalize(64, 64);
    let b = body(&out);
    assert_eq!(count_occurrences(b, "<path"), 1);
    let e = expected;
    let want = format!(
        "transform=\"matrix({} {} {} {} {} {})\"",
        e[0], e[1], e[2], e[3], e[4], e[5]
    );
    assert!(b.contains(&want));
}

#[test]
fn default_black_fill_omitted_non_black_hex() {
    let mut f = SVGFactory::default();
    {
        let mut r = SVGRenderer::default();
        let path = make_square_path(&mut f);
        let paint = make_fill_paint(&mut f, 0xff000000, BlendMode::SrcOver);
        r.draw_path(path.as_ref(), paint.as_ref());
        let out = r.finalize(8, 8);
        assert!(!body(&out).contains("fill="));
    }
    {
        let mut r = SVGRenderer::default();
        let path = make_square_path(&mut f);
        let paint = make_fill_paint(&mut f, 0xff12ab34, BlendMode::SrcOver);
        r.draw_path(path.as_ref(), paint.as_ref());
        let out = r.finalize(8, 8);
        let b = body(&out);
        assert!(b.contains("fill=\"#12ab34\""));
        assert!(!b.contains("rgb("));
    }
}

#[test]
fn modulate_opacity_multiplies_fill_opacity() {
    let mut f = SVGFactory::default();
    {
        let mut r = SVGRenderer::default();
        let path = make_square_path(&mut f);
        let paint = make_fill_paint(&mut f, 0xffff0000, BlendMode::SrcOver);
        r.modulate_opacity(1.0);
        r.draw_path(path.as_ref(), paint.as_ref());
        let out = r.finalize(8, 8);
        assert!(!body(&out).contains("fill-opacity"));
    }
    {
        let mut r = SVGRenderer::default();
        let path = make_square_path(&mut f);
        let paint = make_fill_paint(&mut f, 0x80ff0000, BlendMode::SrcOver);
        r.save();
        r.modulate_opacity(0.5);
        r.draw_path(path.as_ref(), paint.as_ref());
        r.restore();
        let out = r.finalize(8, 8);
        let b = body(&out);
        let p = b.find("fill-opacity=\"").expect("fill-opacity attribute");
        let start = p + "fill-opacity=\"".len();
        let val = &b[start..start + 4];
        assert_eq!(&val[..3], "0.2");
    }
}

#[test]
fn stroke_join_cap_defaults_omitted_non_defaults_emit() {
    let mut f = SVGFactory::default();
    {
        let mut r = SVGRenderer::default();
        let path = make_square_path(&mut f);
        let paint = make_stroke_paint(&mut f, 0xff000000, 2.0, StrokeJoin::Miter, StrokeCap::Butt);
        r.draw_path(path.as_ref(), paint.as_ref());
        let out = r.finalize(8, 8);
        let b = body(&out);
        assert!(!b.contains("stroke-linejoin"));
        assert!(!b.contains("stroke-linecap"));
    }
    {
        let mut r = SVGRenderer::default();
        let path = make_square_path(&mut f);
        let paint = make_stroke_paint(&mut f, 0xff000000, 2.0, StrokeJoin::Round, StrokeCap::Round);
        r.draw_path(path.as_ref(), paint.as_ref());
        let out = r.finalize(8, 8);
        let b = body(&out);
        assert!(b.contains("stroke-linejoin=\"round\""));
        assert!(b.contains("stroke-linecap=\"round\""));
    }
}

#[test]
fn clip_path_emits_one_group_wrapping_children() {
    let mut f = SVGFactory::default();
    let mut r = SVGRenderer::default();
    let clip = make_square_path(&mut f);
    let path = make_square_path(&mut f);
    let paint = make_fill_paint(&mut f, 0xffff0000, BlendMode::SrcOver);
    r.save();
    r.clip_path(clip.as_ref());
    r.draw_path(path.as_ref(), paint.as_ref());
    r.draw_path(path.as_ref(), paint.as_ref());
    r.restore();
    let out = r.finalize(64, 64);
    let b = body(&out);
    assert_eq!(count_occurrences(b, "<g clip-path=\"url(#clip0)\""), 1);
    assert_eq!(count_occurrences(b, "</g>"), 1);
    assert_eq!(count_occurrences(b, "<path"), 2);
    assert!(out.contains("<defs>"));
    assert!(out.contains("<clipPath id=\"clip0\">"));
}

#[test]
fn non_identity_clip_does_not_double_transform_children() {
    let mut f = SVGFactory::default();
    let mut r = SVGRenderer::default();
    let clip = make_square_path(&mut f);
    let path = make_square_path(&mut f);
    let paint = make_fill_paint(&mut f, 0xffff0000, BlendMode::SrcOver);
    r.save();
    r.transform(Mat2D([2.0, 0.0, 0.0, 2.0, 10.0, 10.0]));
    r.save();
    r.clip_path(clip.as_ref());
    r.save();
    r.transform(Mat2D([1.0, 0.0, 0.0, 1.0, 5.0, 7.0]));
    r.draw_path(path.as_ref(), paint.as_ref());
    r.restore();
    r.restore();
    r.restore();
    let out = r.finalize(64, 64);
    let b = body(&out);
    assert!(out.contains("<clipPath id=\"clip0\"><path transform=\"matrix(2 0 0 2 10 10)\""));
    assert!(b.contains("<g clip-path=\"url(#clip0)\">"));
    assert_eq!(
        count_occurrences(b, "transform=\"matrix(2 0 0 2 20 24)\""),
        1
    );
}

#[test]
fn identical_clip_geometry_shares_one_def() {
    let mut f = SVGFactory::default();
    let mut r = SVGRenderer::default();
    let clip = make_square_path(&mut f);
    let path = make_square_path(&mut f);
    let paint = make_fill_paint(&mut f, 0xffff0000, BlendMode::SrcOver);
    r.save();
    r.clip_path(clip.as_ref());
    r.draw_path(path.as_ref(), paint.as_ref());
    r.restore();
    r.draw_path(path.as_ref(), paint.as_ref());
    r.save();
    r.clip_path(clip.as_ref());
    r.draw_path(path.as_ref(), paint.as_ref());
    r.restore();
    let out = r.finalize(64, 64);
    let b = body(&out);
    assert_eq!(count_occurrences(&out, "<clipPath"), 1);
    assert_eq!(count_occurrences(b, "<g clip-path=\"url(#clip0)\">"), 2);
    assert_eq!(count_occurrences(b, "</g>"), 2);
    assert_eq!(count_occurrences(b, "<path"), 3);
}

#[test]
fn same_geometry_different_ctms_gets_distinct_defs() {
    let mut f = SVGFactory::default();
    let mut r = SVGRenderer::default();
    let clip = make_square_path(&mut f);
    let path = make_square_path(&mut f);
    let paint = make_fill_paint(&mut f, 0xffff0000, BlendMode::SrcOver);
    r.save();
    r.transform(Mat2D([2.0, 0.0, 0.0, 2.0, 0.0, 0.0]));
    r.clip_path(clip.as_ref());
    r.draw_path(path.as_ref(), paint.as_ref());
    r.restore();
    r.save();
    r.clip_path(clip.as_ref());
    r.draw_path(path.as_ref(), paint.as_ref());
    r.restore();
    let out = r.finalize(64, 64);
    assert_eq!(count_occurrences(&out, "<clipPath"), 2);
    assert!(out.contains("<clipPath id=\"clip0\">"));
    assert!(out.contains("<clipPath id=\"clip1\">"));
}

#[test]
fn adjacent_scopes_sharing_clip_merge() {
    let mut f = SVGFactory::default();
    let mut r = SVGRenderer::default();
    let clip = make_square_path(&mut f);
    let path = make_square_path(&mut f);
    let paint = make_fill_paint(&mut f, 0xffff0000, BlendMode::SrcOver);
    for _ in 0..3 {
        r.save();
        r.clip_path(clip.as_ref());
        r.draw_path(path.as_ref(), paint.as_ref());
        r.restore();
    }
    let out = r.finalize(64, 64);
    let b = body(&out);
    assert_eq!(count_occurrences(&out, "<clipPath"), 1);
    assert_eq!(count_occurrences(b, "<g clip-path=\"url(#clip0)\">"), 1);
    assert_eq!(count_occurrences(b, "</g>"), 1);
    assert_eq!(count_occurrences(b, "<path"), 3);
}

#[test]
fn clipped_stroke_transform_affects_stroke_on() {
    let mut f = SVGFactory::default();
    let mut r = SVGRenderer::default();
    let clip = make_square_path(&mut f);
    let path = make_square_path(&mut f);
    let paint = make_stroke_paint(&mut f, 0xff00ff00, 4.0, StrokeJoin::Miter, StrokeCap::Butt);
    r.save();
    r.transform(Mat2D([2.0, 0.0, 0.0, 2.0, 10.0, 10.0]));
    r.save();
    r.clip_path(clip.as_ref());
    r.save();
    r.transform(Mat2D([3.0, 0.0, 0.0, 3.0, 5.0, 7.0]));
    r.draw_path(path.as_ref(), paint.as_ref());
    r.restore();
    r.restore();
    r.restore();
    let out = r.finalize(64, 64);
    let b = body(&out);
    assert_eq!(
        count_occurrences(b, "transform=\"matrix(6 0 0 6 20 24)\""),
        1
    );
    assert!(b.contains("stroke-width=\"4\""));
    assert!(b.contains("<g clip-path=\"url(#clip0)\">"));
}

#[test]
fn clipped_stroke_transform_affects_stroke_off() {
    let mut f = SVGFactory::default();
    let mut r = SVGRenderer::default();
    let clip = make_square_path(&mut f);
    let path = make_square_path(&mut f);
    let paint = make_stroke_paint(&mut f, 0xff00ff00, 4.0, StrokeJoin::Miter, StrokeCap::Butt);
    r.save();
    r.transform(Mat2D([2.0, 0.0, 0.0, 2.0, 10.0, 10.0]));
    r.save();
    r.clip_path(clip.as_ref());
    r.draw_path(path.as_ref(), paint.as_ref());
    r.restore();
    r.restore();
    let out = r.finalize(64, 64);
    let b = body(&out);
    assert_eq!(
        count_occurrences(b, "transform=\"matrix(2 0 0 2 10 10)\""),
        1
    );
    assert!(b.contains("stroke-width=\"4\""));
    let group_open = b
        .find("<g clip-path=\"url(#clip0)\">")
        .expect("group opening");
    let stroke_path = b.find("stroke-width").expect("stroke path");
    let group_close = b.find("</g>").expect("group closing");
    assert!(group_open < stroke_path);
    assert!(stroke_path < group_close);
}

#[test]
fn single_non_src_over_blend_inlines_style() {
    let mut f = SVGFactory::default();
    let mut r = SVGRenderer::default();
    let path = make_square_path(&mut f);
    let paint = make_fill_paint(&mut f, 0xffff0000, BlendMode::Multiply);
    r.save();
    r.draw_path(path.as_ref(), paint.as_ref());
    r.restore();
    let out = r.finalize(8, 8);
    let b = body(&out);
    assert!(b.contains("style=\"mix-blend-mode:multiply\""));
    assert_eq!(count_occurrences(b, "<g"), 0);
    assert_eq!(count_occurrences(b, "<path"), 1);
}

#[test]
fn multiple_shared_blend_draws_wrap_in_group() {
    let mut f = SVGFactory::default();
    let mut r = SVGRenderer::default();
    let path = make_square_path(&mut f);
    let paint = make_fill_paint(&mut f, 0xffff0000, BlendMode::Multiply);
    r.save();
    r.draw_path(path.as_ref(), paint.as_ref());
    r.draw_path(path.as_ref(), paint.as_ref());
    r.restore();
    let out = r.finalize(8, 8);
    let b = body(&out);
    assert_eq!(
        count_occurrences(b, "<g style=\"mix-blend-mode:multiply\">"),
        1
    );
    assert_eq!(count_occurrences(b, "</g>"), 1);
    assert_eq!(count_occurrences(b, "<path"), 2);
    assert!(b.contains(" d="));
    let path_start = b.find("<path").expect("first path");
    let path_end = path_start + b[path_start..].find("/>").expect("path closing");
    assert!(!b[path_start..path_end].contains("mix-blend-mode"));
}

#[test]
fn identity_only_outer_save_produces_no_wrappers() {
    let mut f = SVGFactory::default();
    let mut r = SVGRenderer::default();
    let path = make_square_path(&mut f);
    let paint = make_fill_paint(&mut f, 0xffff0000, BlendMode::SrcOver);
    r.save();
    r.transform(Mat2D::IDENTITY);
    r.draw_path(path.as_ref(), paint.as_ref());
    r.restore();
    let out = r.finalize(64, 64);
    let b = body(&out);
    assert_eq!(count_occurrences(b, "<g"), 0);
    assert_eq!(count_occurrences(b, "<path"), 1);
}

#[test]
fn svg_clip_fixture_renders_one_def_and_merged_group() {
    let mut factory = PersistentFactory::new(SVGFactory::default());
    let retained = RuntimeFactoryHandle::from_factory(&mut factory).expect("retained SVGFactory");
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/sync/svg_clip_test.riv");
    let bytes = std::fs::read(path).expect("pinned svg_clip_test.riv");
    let file = File::import(&bytes, retained, None, None, None).expect("SVG fixture import");
    let artboard = file
        .with_file(File::artboard_default)
        .expect("default artboard");
    artboard.advance_default(0.0);
    let mut r = SVGRenderer::default();
    artboard.draw(&mut r);
    let (width, height) = artboard.with_artboard(|artboard| (artboard.width(), artboard.height()));
    let out = r.finalize(width as i32, height as i32);
    let b = body(&out);
    assert_eq!(count_occurrences(&out, "<clipPath"), 1);
    assert_eq!(count_occurrences(b, "<g clip-path=\"url(#clip0)\">"), 1);
    assert_eq!(count_occurrences(b, "</g>"), 1);
    let open = b
        .find("<g clip-path=\"url(#clip0)\">")
        .expect("group opening");
    let close = b.find("</g>").expect("group closing");
    assert_eq!(count_occurrences(&b[open..close], "<path"), 4);
    assert_eq!(count_occurrences(b, "<path"), 5);
}
