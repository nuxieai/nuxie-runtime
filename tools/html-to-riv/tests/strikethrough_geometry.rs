use nuxie_runtime::source::{math::aabb::Aabb, text::css_decoration::StrikethroughStripe};

#[test]
fn paint_snapping_matches_chrome_phase_reference() {
    let reference: serde_json::Value = serde_json::from_str(include_str!(
        "../validation/strikethrough-phase-reference.json"
    ))
    .unwrap();
    let cases = reference["cases"].as_array().unwrap();
    assert_eq!(cases.len(), 384);
    for case in cases {
        let number = |key: &str| case[key].as_f64().unwrap() as f32;
        let padding = number("padding");
        let thickness = number("thickness");
        // The host supplies a baseline offset using resolved ascent. The
        // runtime receives unsnapped baseline geometry and the layout line top.
        let ascent = (-(number("offset") + thickness / 2.0) * 3.0).round();
        let top = number("baseline") - padding - ascent / 3.0 - thickness / 2.0;
        let stripe = StrikethroughStripe {
            thickness,
            bounds: Aabb::new(0.25, top, 18.125, top + thickness),
            line_top: number("lineTop") - padding,
        };
        let painted = stripe.paint_bounds(padding);
        assert_eq!(
            painted.min_y + padding,
            case["rows"][0]["y"].as_f64().unwrap() as f32,
            "{}",
            case["name"]
        );
        assert_eq!(painted.height(), thickness);
        assert_eq!((painted.min_x, painted.max_x), (0.25, 18.125));
    }
}

#[test]
fn paint_snapping_floors_thickness_and_repositions_without_mutating_layout() {
    let stripe = StrikethroughStripe {
        thickness: 2.4,
        bounds: Aabb::new(0.0, 14.666667, 20.0, 17.066668),
        line_top: 0.0,
    };
    for (offset, top) in [(0.0, 15.0), (0.25, 15.0), (0.75, 16.0), (-0.75, 14.0)] {
        let result = stripe.paint_bounds(offset);
        assert_eq!(result.min_y + offset, top);
        assert_eq!(result.height(), 2.0);
    }
    assert_eq!(stripe.line_top, 0.0);
    assert_eq!(stripe.bounds.min_y, 14.666667);
}

#[test]
fn stripe_thickness_does_not_depend_on_subtracting_rounded_edges() {
    let stripe = StrikethroughStripe {
        bounds: Aabb::new(0.0, 14.583333, 20.0, 16.583332),
        thickness: 2.0,
        line_top: 0.0,
    };
    assert!(
        stripe.bounds.height() < 2.0,
        "regression requires cancellation below two"
    );
    assert_eq!(stripe.paint_bounds(0.0).height(), 2.0);
}
