//! Regression for GradientStop's inherited LinearGradient parent dispatch.
//! Pinned gradient_stop.cpp uses is/as<LinearGradient>, which includes radial.
#![cfg(feature = "tools")]

use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::{
    File, RuntimeArtboardInstanceHandle, RuntimeFactoryHandle, RuntimeFileHandle,
    source::{
        component_dirt::ComponentDirt,
        core::CoreHandle,
        generated::{
            core_registry::CoreRegistry,
            shapes::paint::{
                gradient_stop_base::GradientStopBase, linear_gradient_base::LinearGradientBase,
                radial_gradient_base::RadialGradientBase,
            },
        },
        shapes::paint::{
            gradient_stop::GradientStop, linear_gradient::LinearGradient,
            radial_gradient::RadialGradient,
        },
    },
};
use std::path::PathBuf;

fn fixture(
    name: &str,
    artboard_name: Option<&str>,
) -> (RuntimeFileHandle, RuntimeArtboardInstanceHandle) {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let bytes = std::fs::read(
        PathBuf::from(root)
            .join("tests/unit_tests/assets")
            .join(name),
    )
    .expect("pinned gradient fixture");
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let retained = RuntimeFactoryHandle::from_factory(&mut factory).expect("retained factory");
    let file = File::import(&bytes, retained, None, None, None).expect("native import");
    let artboard = file
        .with_file(|file| match artboard_name {
            Some(name) => file.artboard_named(name),
            None => file.artboard_default(),
        })
        .expect("native instance");
    (file, artboard)
}

fn stops(gradient: &CoreHandle) -> Vec<CoreHandle> {
    if gradient.is_type_of(RadialGradientBase::TYPE_KEY) {
        gradient
            .with_downcast::<RadialGradient, _>(|gradient| gradient.base.base.stops().to_vec())
            .expect("radial's inherited LinearGradient owner")
    } else {
        gradient
            .with_downcast::<LinearGradient, _>(|gradient| gradient.stops().to_vec())
            .expect("linear owner")
    }
}

fn assert_inherited_stops(
    name: &str,
    artboard_name: Option<&str>,
    expected_radial_stop_count: usize,
) {
    let (_file, artboard) = fixture(name, artboard_name);
    let gradients = artboard.with_artboard(|artboard| {
        artboard
            .objects()
            .iter()
            .flatten()
            .filter(|object| object.is_type_of(LinearGradientBase::TYPE_KEY))
            .cloned()
            .collect::<Vec<_>>()
    });
    let mut saw_expected_radial = false;
    for gradient in gradients {
        let expected = gradient
            .with(|owner| {
                owner
                    .as_container_component()
                    .expect("gradient container")
                    .children()
                    .iter()
                    .filter(|child| child.is_type_of(GradientStopBase::TYPE_KEY))
                    .cloned()
                    .collect::<Vec<_>>()
            })
            .expect("live gradient");
        let retained = stops(&gradient);
        assert_eq!(
            retained, expected,
            "{name}: every authored stop registers in source order"
        );
        assert!(!retained.is_empty(), "{name}: authored gradient has stops");
        if !gradient.is_type_of(RadialGradientBase::TYPE_KEY) {
            continue;
        }
        saw_expected_radial |= retained.len() == expected_radial_stop_count;
        let stop = &retained[0];
        let (color, position) = stop
            .with_downcast::<GradientStop, _>(|stop| {
                (stop.base.color_value(), stop.base.position())
            })
            .expect("actual registered stop");
        gradient.with_mut(|owner| {
            owner
                .as_component_mut()
                .unwrap()
                .set_dirt(ComponentDirt::NONE)
        });
        assert!(CoreRegistry::set_color_handle(
            stop,
            GradientStopBase::COLOR_VALUE_PROPERTY_KEY.into(),
            color ^ 1
        ));
        assert_eq!(
            gradient.with(|owner| owner.as_component().unwrap().dirt()),
            Some(ComponentDirt::PAINT)
        );
        gradient.with_mut(|owner| {
            owner
                .as_component_mut()
                .unwrap()
                .set_dirt(ComponentDirt::NONE)
        });
        assert!(CoreRegistry::set_double_handle(
            stop,
            GradientStopBase::POSITION_PROPERTY_KEY.into(),
            position + 0.125
        ));
        assert_eq!(
            gradient.with(|owner| owner.as_component().unwrap().dirt()),
            Some(ComponentDirt::PAINT | ComponentDirt::STOPS)
        );
    }
    assert!(
        saw_expected_radial,
        "{name}: pinned radial gradient has {expected_radial_stop_count} stops"
    );
}

#[test]
fn bankcard_radial_stops_register_and_dirty_the_inherited_gradient() {
    // makeRadialGradient's pinned 14 fields: four scalar fields + five stop pairs.
    // The pinned fixture's nested "Artboard" owns these gradients; its default
    // "main" artboard has no radial gradients (confirmed by the C++ probe).
    assert_inherited_stops("bankcard.riv", Some("Artboard"), 5);
}

#[test]
fn car_widgets_radial_stops_register_and_dirty_the_inherited_gradient() {
    // makeRadialGradient's pinned eight fields: four scalar fields + two stop pairs.
    assert_inherited_stops("car_widgets_v01.riv", None, 2);
}
