//! Literal public text_style_keyed_font_test.cpp at upstream b27b97fda.
#[path = "support/text_style_global.rs"]
mod support;

use nuxie_runtime::source::{assets::font_asset::FontAsset, text::text_style::TextStyle};
use support::*;

fn style_asset(style: &CoreHandle) -> Option<CoreHandle> {
    style
        .with(|object| object.as_text_style().expect("TextStyle").asset())
        .expect("live style")
}

fn assert_font_identity(style: &CoreHandle, asset: &CoreHandle) {
    let actual = style
        .with_mut(|object| object.as_text_style_mut().expect("TextStyle").font())
        .expect("live style");
    let expected = asset
        .with_downcast::<FontAsset, _>(FontAsset::font)
        .expect("FontAsset");
    match (actual, expected) {
        (Some(actual), Some(expected)) => assert!(std::sync::Arc::ptr_eq(&actual, &expected)),
        (None, None) => {}
        _ => panic!("style and asset font identities differ"),
    }
}

#[test]
fn a_keyed_font_swaps_the_styles_font() {
    // One style keys Bodoni at frame 0 and Emoji at frame 30 (60 fps).
    // Emoji is included in the file solely because a key references it.
    let file = load("text_style_keyed_font.riv");
    let assets = file.with_file(|file| file.assets().to_vec());
    assert_eq!(assets.len(), 2);
    let mut bodoni = None;
    let mut emoji = None;
    for asset in assets {
        assert!(asset.is_type_of(FontAsset::TYPE_KEY));
        let name = asset
            .with_downcast::<FontAsset, _>(|asset| asset.name().to_owned())
            .expect("FontAsset");
        if name == "Bodoni" {
            bodoni = Some(asset);
        } else {
            emoji = Some(asset);
        }
    }
    let bodoni = bodoni.expect("Bodoni asset");
    let emoji = emoji.expect("Emoji asset");
    assert_eq!(
        emoji.with_downcast::<FontAsset, _>(|asset| asset.name().to_owned()),
        Some("Emoji".to_owned())
    );

    // artboard_default returns an instance, exercising cloned font provenance.
    let artboard = file
        .with_file(File::artboard_default)
        .expect("default artboard");
    let mut style = None;
    let mut text = None;
    artboard.with_artboard(|artboard| {
        for object in artboard.objects().iter().flatten() {
            if object.is_type_of(TextStyle::TYPE_KEY) {
                style = Some(object.clone());
            } else if object.is_type_of(Text::TYPE_KEY) {
                text = Some(object.clone());
            }
        }
    });
    let style = style.expect("TextStyle");
    let text = text.expect("Text");
    let width = || {
        text.with_downcast::<Text, _>(|text| text.local_bounds().width())
            .unwrap()
    };
    let mut swap = artboard.animation_named("swap").expect("swap animation");

    swap.advance_and_apply(0.0);
    artboard.advance_default(0.0);
    assert_eq!(style_asset(&style), Some(bodoni.clone()));
    assert_font_identity(&style, &bodoni);
    let bodoni_width = width();

    swap.advance_and_apply(0.75);
    artboard.advance_default(0.0);
    assert_eq!(style_asset(&style), Some(emoji.clone()));
    assert_font_identity(&style, &emoji);
    assert_ne!(width(), bodoni_width);

    swap.set_time(0.0);
    swap.apply(1.0);
    artboard.advance_default(0.0);
    assert_eq!(style_asset(&style), Some(bodoni));
    assert_eq!(width(), bodoni_width);
}
