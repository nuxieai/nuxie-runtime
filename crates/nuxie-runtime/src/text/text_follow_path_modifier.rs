#[derive(Debug, Clone)]
struct StaticTextFollowPathModifier {
    local_id: usize,
    global_id: u32,
    target_id: u32,
    resolved_transform_local: Option<usize>,
    paths: Vec<StaticTextFollowPathPath>,
}
#[derive(Debug, Clone)]
struct StaticTextFollowPathPath {
    local_id: usize,
    geometry: PathGeometryNode,
}
#[derive(Debug, Clone, Copy)]
struct StaticFollowPathGlyphTransform {
    x: f32,
    y: f32,
    rotation: f32,
}
impl StaticTextFollowPathModifier {
    fn from_graph(runtime: &RuntimeFile, graph: &ArtboardGraph, local_id: usize) -> Result<Self> {
        let global_id = global_for_local(graph, local_id)?;
        let object = runtime
            .object(global_id as usize)
            .with_context(|| format!("missing TextFollowPathModifier global {global_id}"))?;
        let target_id = object.uint_property("targetId").unwrap_or(u32::MAX as u64) as u32;
        let target_local = usize::try_from(target_id).ok();
        let resolved_transform_local = target_local.filter(|target| {
            component_for_local(graph, *target).is_some_and(|component| {
                nuxie_schema::definition_by_name(component.type_name)
                    .is_some_and(|definition| definition.is_a("TransformComponent"))
            })
        });
        let paths = match target_local {
            Some(target_local) if type_for_local(graph, target_local) == Some("Shape") => graph
                .path_composers
                .iter()
                .find(|composer| composer.shape_local == target_local)
                .map(|composer| {
                    composer
                        .paths
                        .iter()
                        .filter_map(|path_ref| {
                            graph
                                .paths
                                .iter()
                                .find(|path| path.local_id == path_ref.local_id)
                                .cloned()
                                .map(|geometry| StaticTextFollowPathPath {
                                    local_id: path_ref.local_id,
                                    geometry,
                                })
                        })
                        .collect()
                })
                .unwrap_or_default(),
            Some(target_local) => graph
                .paths
                .iter()
                .find(|path| path.local_id == target_local)
                .cloned()
                .map(|geometry| {
                    vec![StaticTextFollowPathPath {
                        local_id: target_local,
                        geometry,
                    }]
                })
                .unwrap_or_default(),
            None => Vec::new(),
        };

        Ok(Self {
            local_id,
            global_id,
            target_id,
            resolved_transform_local,
            paths,
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
        let path_measure = self.path_measure(instance, glyph.text_world_inverse);
        let path_length = path_measure.length();
        if path_length == 0.0 {
            return Ok(current);
        }

        let position_on_path = (glyph.origin_x + offset.0, glyph.origin_y + offset.1);
        let start = self.double_property(runtime, instance, "start", 0.0)?;
        let end = self.double_property(runtime, instance, "end", 1.0)?;
        let start_pct = start.min(end).clamp(0.0, 1.0);
        let end_pct = start.max(end).clamp(0.0, 1.0);
        let can_wrap = path_measure.raw_is_closed() && (end_pct - start_pct) == 1.0;
        let valid_length = (end_pct - start_pct) * path_length;
        let offset_pct = self
            .double_property(runtime, instance, "offset", 0.0)?
            .rem_euclid(1.0);
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

        let strength = self
            .double_property(runtime, instance, "strength", 1.0)?
            .clamp(0.0, 1.0);
        let inverse_strength = 1.0 - strength;
        Ok(StaticFollowPathGlyphTransform {
            x: translation.0 * strength + current.x * inverse_strength,
            y: translation.1 * strength + current.y * inverse_strength,
            rotation: rotation * strength + current.rotation * inverse_strength,
        })
    }

    fn path_measure(
        &self,
        instance: &ArtboardInstance,
        text_world_inverse: Mat2D,
    ) -> RuntimePathMeasure {
        let mut commands = Vec::new();
        for path in &self.paths {
            let Some(path_world) = instance
                .component(path.local_id)
                .map(|component| component.transform.world_transform)
            else {
                continue;
            };
            let mut path_commands =
                runtime_path_geometry_commands(instance, &path.geometry, path_world);
            transform_path_commands(&mut path_commands, text_world_inverse);
            commands.extend(path_commands);
        }
        RuntimePathMeasure::from_commands_with_tolerance(&commands, 0.1)
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
