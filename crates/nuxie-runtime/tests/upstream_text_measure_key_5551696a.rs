//! Public text_test.cpp additions at upstream 5551696a.
//! Relative and before/after assertions are preserved across the Taffy boundary.
use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::{
    advance_flags::AdvanceFlags,
    core::{CoreHandle, CoreType},
    generated::{
        core_registry::CoreRegistry,
        text::{
            text_base::TextBase, text_value_run_base::TextValueRunBase,
            text_variation_modifier_base::TextVariationModifierBase,
        },
    },
    layout_component::LayoutComponent,
    text::{
        text::Text, text_value_run::TextValueRun, text_variation_modifier::TextVariationModifier,
    },
};
use nuxie_runtime::{Artboard, File, RuntimeFactoryHandle, RuntimeFileHandle};

fn fixture(asset: &str) -> (RuntimeFileHandle, CoreHandle) {
    let root = std::path::PathBuf::from(
        std::env::var_os("RIVE_RUNTIME_DIR").expect("pinned upstream checkout"),
    );
    let bytes = std::fs::read(root.join("tests/unit_tests/assets").join(asset)).unwrap();
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let factory = RuntimeFactoryHandle::from_factory(&mut factory).unwrap();
    let file = File::import(&bytes, factory, None, None, None).unwrap();
    let artboard = file
        .with_file(|file| file.artboard())
        .expect("default artboard");
    (file, artboard)
}
fn advance(artboard: &CoreHandle, seconds: f32) {
    Artboard::advance_handle(
        artboard,
        seconds,
        AdvanceFlags::ADVANCE_NESTED | AdvanceFlags::ANIMATE | AdvanceFlags::NEW_FRAME,
    );
}
fn text(run: &CoreHandle) -> CoreHandle {
    run.with_downcast::<TextValueRun, _>(TextValueRun::text_component)
        .flatten()
        .expect("run's text")
}
fn parent(text: &CoreHandle) -> CoreHandle {
    text.with(|object| object.as_component().unwrap().parent_handle())
        .flatten()
        .expect("text's parent")
}
fn first_hug_run(artboard: &CoreHandle) -> (CoreHandle, CoreHandle) {
    let runs = artboard
        .with_downcast::<Artboard, _>(|a| a.find_all_handles::<TextValueRun>())
        .unwrap();
    for run in runs {
        let parent = text(&run)
            .with(|object| object.as_component().unwrap().parent_handle())
            .flatten();
        if let Some(parent) = parent {
            if parent.is_type_of(LayoutComponent::TYPE_KEY)
                && !parent.is_type_of(Artboard::TYPE_KEY)
            {
                return (run, parent);
            }
        }
    }
    panic!("first text run in a non-artboard layout");
}
fn size(layout: &CoreHandle) -> (f32, f32) {
    layout
        .with_downcast::<LayoutComponent, _>(|layout| {
            (layout.layout_width(), layout.layout_height())
        })
        .unwrap()
}
fn set_text(run: &CoreHandle, value: &str) {
    assert!(CoreRegistry::set_string_handle(
        run,
        TextValueRunBase::TEXT_PROPERTY_KEY.into(),
        value.to_owned()
    ));
}
fn append_text(run: &CoreHandle, suffix: &str) {
    let value = run
        .with_downcast::<TextValueRun, _>(|run| format!("{}{suffix}", run.base.text()))
        .unwrap();
    set_text(run, &value);
}

#[cfg(feature = "tools")]
thread_local! { static DIRTIED: std::cell::Cell<u32> = const { std::cell::Cell::new(0) }; }
#[cfg(feature = "tools")]
fn observe(artboard: &CoreHandle) {
    DIRTIED.with(|count| count.set(0));
    artboard
        .with_downcast_mut::<Artboard, _>(|a| {
            a.on_layout_dirty(Some(|_| {
                DIRTIED.with(|count| count.set(count.get() + 1));
            }))
        })
        .unwrap();
}
#[cfg(feature = "tools")]
#[test]
fn a_text_change_re_solves_its_hug_layout_once() {
    let (_file, artboard) = fixture("layout/text_layout_7_3.riv");
    advance(&artboard, 0.0);
    let (run, layout) = first_hug_run(&artboard);
    let width_before = size(&layout).0;
    observe(&artboard);
    advance(&artboard, 0.0);
    assert_eq!(DIRTIED.with(std::cell::Cell::get), 0);
    append_text(&run, " and then some more words");
    advance(&artboard, 0.016);
    assert_eq!(DIRTIED.with(std::cell::Cell::get), 1);
    assert!(size(&layout).0 > width_before);
    DIRTIED.with(|count| count.set(0));
    advance(&artboard, 0.016);
    assert_eq!(DIRTIED.with(std::cell::Cell::get), 0);
}

fn hug_text_layout_size(
    asset: &str,
    setup: impl FnOnce(&CoreHandle),
    change: Option<fn(&CoreHandle)>,
) -> (f32, f32) {
    let (_file, artboard) = fixture(asset);
    let (run, layout) = first_hug_run(&artboard);
    setup(&run);
    advance(&artboard, 0.0);
    if let Some(change) = change {
        change(&text(&run));
        advance(&artboard, 0.016);
    }
    size(&layout)
}
fn two_paragraphs(run: &CoreHandle) {
    set_text(run, "first paragraph\nsecond paragraph");
}
fn spaced(text: &CoreHandle) {
    let spacing = text
        .with_downcast::<Text, _>(|text| text.paragraph_spacing())
        .unwrap();
    assert!(CoreRegistry::set_double_handle(
        text,
        TextBase::PARAGRAPH_SPACING_PROPERTY_KEY.into(),
        spacing + 40.0
    ));
}
#[test]
fn paragraph_spacing_re_measures_a_hug_layouts_text() {
    let asset = "layout_text_match_7_3.riv";
    let unspaced = hug_text_layout_size(asset, two_paragraphs, None);
    let fresh = hug_text_layout_size(
        asset,
        |run| {
            two_paragraphs(run);
            spaced(&text(run));
        },
        None,
    );
    let changed = hug_text_layout_size(asset, two_paragraphs, Some(spaced));
    assert!(fresh.1 > unspaced.1);
    assert_eq!(changed.0, fresh.0);
    assert_eq!(changed.1, fresh.1);
}
fn baseline(text: &CoreHandle) {
    assert!(CoreRegistry::set_uint_handle(
        text,
        TextBase::ORIGIN_VALUE_PROPERTY_KEY.into(),
        1
    )); // TextOrigin::baseline
}
#[test]
fn text_origin_re_measures_a_hug_layouts_text() {
    let asset = "layout/text_layout_7_3.riv";
    let top = hug_text_layout_size(asset, |_| {}, None);
    let fresh = hug_text_layout_size(asset, |run| baseline(&text(run)), None);
    let changed = hug_text_layout_size(asset, |_| {}, Some(baseline));
    assert_ne!(fresh.1, top.1);
    assert_eq!(changed.0, fresh.0);
    assert_eq!(changed.1, fresh.1);
}
fn variation_modified_hug(artboard: &CoreHandle) -> (CoreHandle, CoreHandle) {
    let run = artboard
        .with_downcast::<Artboard, _>(|a| a.find_handle::<TextValueRun>("Run"))
        .flatten()
        .expect("Run");
    let layout = parent(&text(&run));
    assert!(layout.is_type_of(LayoutComponent::TYPE_KEY));
    (run, layout)
}
#[cfg(feature = "tools")]
#[test]
fn a_variation_modified_text_re_solves_its_hug_layout_once() {
    let settled = {
        let (_file, artboard) = fixture("text_variation_modifier_hug.riv");
        let (run, layout) = variation_modified_hug(&artboard);
        append_text(&run, " text");
        for _ in 0..10 {
            advance(&artboard, 0.016);
        }
        size(&layout).0
    };
    let (_file, artboard) = fixture("text_variation_modifier_hug.riv");
    let (run, layout) = variation_modified_hug(&artboard);
    for _ in 0..10 {
        advance(&artboard, 0.016);
    }
    observe(&artboard);
    advance(&artboard, 0.0);
    assert_eq!(DIRTIED.with(std::cell::Cell::get), 0);
    append_text(&run, " text");
    advance(&artboard, 0.016);
    assert_eq!(DIRTIED.with(std::cell::Cell::get), 1);
    assert_eq!(size(&layout).0, settled);
    DIRTIED.with(|count| count.set(0));
    advance(&artboard, 0.016);
    assert_eq!(DIRTIED.with(std::cell::Cell::get), 0);
}
fn shaped_width_after(before: bool, after: bool) -> f32 {
    let (_file, artboard) = fixture("text_variation_modifier_hug.riv");
    let modifier = artboard
        .with_downcast::<Artboard, _>(|a| a.find_handle::<TextVariationModifier>("Width"))
        .flatten()
        .expect("Width");
    let (run, _) = variation_modified_hug(&artboard);
    let change = || {
        assert!(CoreRegistry::set_double_handle(
            &modifier,
            TextVariationModifierBase::AXIS_VALUE_PROPERTY_KEY.into(),
            200.0
        ))
    };
    if before {
        change();
    }
    advance(&artboard, 0.0);
    if after {
        change();
        advance(&artboard, 0.016);
    }
    text(&run)
        .with_downcast::<Text, _>(|text| {
            let mut width = 0.0;
            for line in text.ordered_lines() {
                for (glyph_run, glyph_index) in line {
                    width += glyph_run.advances[glyph_index as usize];
                }
            }
            width
        })
        .unwrap()
}
#[test]
fn a_variation_modifier_change_reshapes_its_text() {
    let narrow = shaped_width_after(false, false);
    let fresh = shaped_width_after(true, false);
    let changed = shaped_width_after(false, true);
    assert!(fresh > narrow);
    assert_eq!(changed, fresh);
}
