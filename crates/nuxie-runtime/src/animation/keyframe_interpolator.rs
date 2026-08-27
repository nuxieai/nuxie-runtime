#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum RuntimeInterpolator {
    Scripted {
        global_id: u32,
    },
    CubicEase {
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
    },
    CubicValue(RuntimeCubicValueInterpolator),
    Elastic {
        amplitude: f32,
        period: f32,
        easing_value: u64,
    },
}

impl RuntimeInterpolator {
    pub(crate) fn from_object(object: &RuntimeObject) -> Option<Self> {
        match object.type_name {
            "ScriptedInterpolator" => Some(Self::Scripted {
                global_id: object.id,
            }),
            "CubicEaseInterpolator" => Some(Self::CubicEase {
                x1: object.double_property("x1").unwrap_or(0.42),
                y1: object.double_property("y1").unwrap_or(0.0),
                x2: object.double_property("x2").unwrap_or(0.58),
                y2: object.double_property("y2").unwrap_or(1.0),
            }),
            "CubicValueInterpolator" => Some(Self::CubicValue(
                RuntimeCubicValueInterpolator::on_added_dirty(
                    object.double_property("x1").unwrap_or(0.42),
                    object.double_property("y1").unwrap_or(0.0),
                    object.double_property("x2").unwrap_or(0.58),
                    object.double_property("y2").unwrap_or(1.0),
                ),
            )),
            "ElasticInterpolator" => Some(Self::Elastic {
                amplitude: object.double_property("amplitude").unwrap_or(1.0),
                period: object.double_property("period").unwrap_or(1.0),
                easing_value: object.uint_property("easingValue").unwrap_or(1),
            }),
            _ => None,
        }
    }

    pub(crate) fn transform_value(mut self, value_from: f32, value_to: f32, factor: f32) -> f32 {
        match &mut self {
            Self::Scripted { .. } => value_from + (value_to - value_from) * factor,
            Self::CubicEase { x1, y1, x2, y2 } => cubic_ease_interpolator_transform_value(
                value_from, value_to, factor, *x1, *y1, *x2, *y2,
            ),
            Self::CubicValue(interpolator) => {
                interpolator.transform_value(value_from, value_to, factor)
            }
            Self::Elastic {
                amplitude,
                period,
                easing_value,
            } => RuntimeElasticInterpolator::on_added_dirty(*easing_value, *amplitude, *period)
                .transform_value(value_from, value_to, factor),
        }
    }

    pub(crate) fn transform(self, factor: f32) -> f32 {
        match self {
            Self::Scripted { .. } => factor,
            Self::CubicEase { x1, y1, x2, y2 } => {
                cubic_ease_interpolator_transform(factor, x1, y1, x2, y2)
            }
            Self::CubicValue(interpolator) => interpolator.transform(factor),
            Self::Elastic {
                amplitude,
                period,
                easing_value,
            } => elastic_interpolator_transform(factor, amplitude, period, easing_value),
        }
    }
}

/// Mirrors `InterpolatorHost::from` followed by
/// `InterpolatorHost::overridesKeyedInterpolation`.
///
/// The pinned static dispatch checks `coreType()` rather than `isTypeOf()`, so
/// only the concrete `LayoutComponent` type is an interpolator host. Its
/// implementation overrides the caller's keyed mix for width and height only
/// while the component's own layout animation is active.
fn interpolator_host_overrides_keyed_interpolation(
    artboard: &ArtboardInstance,
    target_local_id: usize,
    property_key: u16,
) -> bool {
    let Some(component) = artboard.component(target_local_id) else {
        return false;
    };
    if component.type_name != "LayoutComponent" {
        return false;
    }
    let Some(layout) = component.concrete.layout.as_ref() else {
        return false;
    };

    layout.animates()
        && ["width", "height"].into_iter().any(|property_name| {
            crate::properties::property_key_for_name("LayoutComponent", property_name)
                == Some(property_key)
        })
}

/// Pinned `KeyedProperty::apply` forces a host-owned property to its complete
/// keyed value, leaving the host to perform its own interpolation.
fn keyed_property_actual_mix(
    artboard: &ArtboardInstance,
    target_local_id: usize,
    property_key: u16,
    mix: f32,
) -> f32 {
    if interpolator_host_overrides_keyed_interpolation(
        artboard,
        target_local_id,
        property_key,
    ) {
        1.0
    } else {
        mix
    }
}
