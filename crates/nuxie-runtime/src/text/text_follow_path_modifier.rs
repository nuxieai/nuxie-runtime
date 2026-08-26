#[derive(Debug, Clone)]
struct StaticTextFollowPathModifier {
    local_id: usize,
    global_id: u32,
}
#[derive(Debug, Clone, Copy)]
struct StaticFollowPathGlyphTransform {
    x: f32,
    y: f32,
    rotation: f32,
}
impl StaticTextFollowPathModifier {
    fn from_graph(runtime: &RuntimeFile, graph: &ArtboardGraph, local_id: usize) -> Result<Self> {
        let (global_id, _) = text_target_modifier_resolution(runtime, graph, local_id)?;
        Ok(Self {
            local_id,
            global_id,
        })
    }

    fn transform_glyph(
        &self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
        current: StaticFollowPathGlyphTransform,
        glyph: &StaticTextGlyphContext<'_>,
        offset: (f32, f32),
    ) -> Result<StaticFollowPathGlyphTransform> {
        let path_measure = instance
            .component(self.local_id)
            .and_then(|component| component.concrete.text_follow_path.as_ref())
            .map(|state| state.local_measure())
            .unwrap_or_else(|| RuntimePathMeasure::from_commands(&[]));
        let path_length = path_measure.length();
        if path_length == 0.0 {
            return Ok(current);
        }

        let position_on_path = (glyph.origin_x + offset.0, glyph.origin_y + offset.1);
        let start = self.double_property(runtime, instance, "start", 0.0)?;
        let end = self.double_property(runtime, instance, "end", 1.0)?;
        let start_pct = text_follow_path_math_clamp(cpp_std_min(start, end), 0.0, 1.0);
        let end_pct = text_follow_path_math_clamp(cpp_std_max(start, end), 0.0, 1.0);
        let can_wrap = path_measure.raw_is_closed() && (end_pct - start_pct) == 1.0;
        let valid_length = (end_pct - start_pct) * path_length;
        let offset_pct = text_follow_path_positive_unit_mod(
            self.double_property(runtime, instance, "offset", 0.0)?,
        );
        let start_pct = start_pct + offset_pct;
        let end_pct = end_pct + offset_pct;

        let sample = if (!can_wrap && position_on_path.0 < 0.0) || start_pct == end_pct {
            let result = path_measure.at_percentage(start_pct);
            let tangent = normalize_point(result.tan);
            let extra = -position_on_path.0;
            RuntimePathSampleParts {
                position: (
                    result.pos.0 - tangent.0 * extra,
                    result.pos.1 - tangent.1 * extra,
                ),
                tangent,
            }
        } else if !can_wrap && position_on_path.0 > valid_length {
            let result = path_measure.at_percentage(end_pct);
            let tangent = normalize_point(result.tan);
            let extra = position_on_path.0 - valid_length;
            RuntimePathSampleParts {
                position: (
                    result.pos.0 + tangent.0 * extra,
                    result.pos.1 + tangent.1 * extra,
                ),
                tangent,
            }
        } else {
            let result = path_measure.at_percentage(start_pct + position_on_path.0 / path_length);
            RuntimePathSampleParts {
                position: result.pos,
                tangent: normalize_point(result.tan),
            }
        };

        let last_line_index = glyph.line_index_in_paragraph.checked_sub(1);
        let last_baseline = last_line_index
            .and_then(|index| glyph.paragraph_baselines.get(index).copied())
            .unwrap_or(0.0);
        let current_baseline = glyph
            .paragraph_baselines
            .get(glyph.line_index_in_paragraph)
            .copied()
            .unwrap_or(0.0);
        let translation = if self.bool_property(runtime, instance, "radial", false)? {
            let vertical_spacing = position_on_path.1 - current_baseline;
            let perpendicular = (-sample.tangent.1, sample.tangent.0);
            (
                sample.position.0 + vertical_spacing * perpendicular.0,
                sample.position.1 + vertical_spacing * perpendicular.1,
            )
        } else {
            (
                sample.position.0,
                position_on_path.1 - current_baseline + sample.position.1 + last_baseline,
            )
        };
        let rotation = if self.bool_property(runtime, instance, "orient", true)? {
            sample.tangent.1.atan2(sample.tangent.0)
        } else {
            0.0
        };

        let strength = text_follow_path_math_clamp(
            self.double_property(runtime, instance, "strength", 1.0)?,
            0.0,
            1.0,
        );
        let inverse_strength = 1.0 - strength;
        Ok(StaticFollowPathGlyphTransform {
            x: translation.0 * strength + current.x * inverse_strength,
            y: translation.1 * strength + current.y * inverse_strength,
            rotation: rotation * strength + current.rotation * inverse_strength,
        })
    }

    fn reset(&self, instance: &ArtboardInstance, inverse_text: Mat2D) {
        let Some(component) = instance.component(self.local_id) else {
            return;
        };
        let Some(state) = component.concrete.text_follow_path.as_ref() else {
            return;
        };
        if component
            .concrete
            .text_target
            .as_ref()
            .and_then(|target| target.target_local())
            .is_none()
        {
            state.retain_local_measure(RuntimePathMeasure::from_commands(&[]));
            return;
        }
        let mut commands = state.world_commands();
        transform_path_commands(&mut commands, inverse_text);
        state.retain_local_measure(RuntimePathMeasure::from_commands_with_tolerance(
            &commands, 0.1,
        ));
    }

    fn double_property(
        &self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
        property_name: &str,
        default: f32,
    ) -> Result<f32> {
        runtime_double_property(
            runtime,
            instance,
            "TextFollowPathModifier",
            self.local_id,
            self.global_id,
            property_name,
            default,
        )
    }

    fn bool_property(
        &self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
        property_name: &str,
        default: bool,
    ) -> Result<bool> {
        runtime_bool_property(
            runtime,
            instance,
            "TextFollowPathModifier",
            self.local_id,
            self.global_id,
            property_name,
            default,
        )
    }
}

pub(crate) fn update_text_follow_path_world_path(
    runtime: &RuntimeFile,
    graph: &ArtboardGraph,
    instance: &ArtboardInstance,
    local_id: usize,
) -> Option<Vec<RuntimePathCommand>> {
    StaticTextFollowPathModifier::from_graph(runtime, graph, local_id).ok()?;
    let target_local = instance
        .component(local_id)?
        .concrete
        .text_target
        .as_ref()?
        .target_local()?;
    let path_locals = if type_for_local(graph, target_local) == Some("Shape") {
        graph
            .path_composers
            .iter()
            .find(|composer| composer.shape_local == target_local)
            .map(|composer| {
                composer
                    .paths
                    .iter()
                    .map(|path| path.local_id)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    } else if type_for_local(graph, target_local) == Some("Path") {
        vec![target_local]
    } else {
        Vec::new()
    };
    let mut commands = Vec::new();
    for path_local in path_locals {
        let path = graph
            .paths
            .iter()
            .find(|path| path.local_id == path_local)?;
        let path_world = instance.component(path_local)?.transform.world_transform;
        commands.extend(runtime_path_geometry_commands(instance, path, path_world));
    }
    Some(commands)
}

pub(crate) fn text_follow_path_modifier_double_property_changed(
    instance: &mut ArtboardInstance,
    local_id: usize,
    type_name: Option<&str>,
    property_key: u16,
) -> Option<bool> {
    (type_name == Some("TextFollowPathModifier")
        && ["start", "end", "offset", "strength"]
            .into_iter()
            .any(|name| {
                property_key_for_name("TextFollowPathModifier", name) == Some(property_key)
            }))
    .then(|| text_follow_path_modifier_shape_dirty(instance, local_id))
}

pub(crate) fn text_follow_path_modifier_bool_property_changed(
    instance: &mut ArtboardInstance,
    local_id: usize,
    type_name: Option<&str>,
    property_key: u16,
) -> Option<bool> {
    (type_name == Some("TextFollowPathModifier")
        && ["radial", "orient"].into_iter().any(|name| {
            property_key_for_name("TextFollowPathModifier", name) == Some(property_key)
        }))
    .then(|| text_follow_path_modifier_shape_dirty(instance, local_id))
}

fn text_follow_path_modifier_shape_dirty(instance: &mut ArtboardInstance, local_id: usize) -> bool {
    text_target_modifier_text_component(instance, local_id)
        .is_some_and(|text| crate::text_owner::modifier_shape_dirty(instance, text))
}

// Literal two-argument `std::min`/`std::max` comparison order. Equal signed
// zero retains the left operand and a NaN left operand remains NaN.
fn cpp_std_min(left: f32, right: f32) -> f32 {
    if right < left { right } else { left }
}

fn cpp_std_max(left: f32, right: f32) -> f32 {
    if left < right { right } else { left }
}

// Pinned `math::clamp` is `fminf(fmaxf(lo, value), hi)`.
fn text_follow_path_math_clamp(value: f32, lo: f32, hi: f32) -> f32 {
    lo.max(value).min(hi)
}

fn text_follow_path_positive_unit_mod(value: f32) -> f32 {
    ((value % 1.0) + 1.0) % 1.0
}
#[derive(Debug, Clone, Copy)]
struct RuntimePathSampleParts {
    position: (f32, f32),
    tangent: (f32, f32),
}
fn normalize_point(point: (f32, f32)) -> (f32, f32) {
    let length = (point.0 * point.0 + point.1 * point.1).sqrt();
    if length == 0.0 {
        (0.0, 0.0)
    } else {
        (point.0 / length, point.1 / length)
    }
}
