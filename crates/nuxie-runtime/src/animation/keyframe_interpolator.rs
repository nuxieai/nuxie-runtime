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
    CubicValue {
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
    },
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
            "CubicValueInterpolator" => Some(Self::CubicValue {
                x1: object.double_property("x1").unwrap_or(0.42),
                y1: object.double_property("y1").unwrap_or(0.0),
                x2: object.double_property("x2").unwrap_or(0.58),
                y2: object.double_property("y2").unwrap_or(1.0),
            }),
            "ElasticInterpolator" => Some(Self::Elastic {
                amplitude: object.double_property("amplitude").unwrap_or(1.0),
                period: object.double_property("period").unwrap_or(1.0),
                easing_value: object.uint_property("easingValue").unwrap_or(1),
            }),
            _ => None,
        }
    }

    pub(crate) fn transform_value(self, value_from: f32, value_to: f32, factor: f32) -> f32 {
        match self {
            Self::Scripted { .. } => value_from + (value_to - value_from) * factor,
            Self::CubicValue { x1, y1, x2, y2 } => {
                let t = cubic_interpolator_get_t(factor, x1, x2);
                cubic_interpolator_calc_cubic_value(t, value_from, y1, y2, value_to)
            }
            _ => value_from + (value_to - value_from) * self.transform(factor),
        }
    }

    pub(crate) fn transform(self, factor: f32) -> f32 {
        match self {
            Self::Scripted { .. } => factor,
            Self::CubicEase { x1, y1, x2, y2 } => {
                let t = cubic_interpolator_get_t(factor, x1, x2);
                cubic_interpolator_calc_bezier(t, y1, y2)
            }
            Self::CubicValue { .. } => factor,
            Self::Elastic {
                amplitude,
                period,
                easing_value,
            } => elastic_interpolator_transform(factor, amplitude, period, easing_value),
        }
    }
}
