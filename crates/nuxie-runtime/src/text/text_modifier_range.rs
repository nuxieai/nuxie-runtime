#[derive(Debug, Clone)]
struct StaticTextModifierRange {
    local_id: usize,
    global_id: u32,
    run_local: Option<usize>,
    interpolator: Option<StaticCubicInterpolator>,
}
#[derive(Debug, Clone, Copy)]
struct StaticRangeUnit {
    start: usize,
    len: usize,
}

#[derive(Debug, Clone)]
struct StaticRangeMap {
    units: Vec<StaticRangeUnit>,
    terminal_index: Option<usize>,
}

fn text_range_std_min(left: f32, right: f32) -> f32 {
    if right < left { right } else { left }
}

fn text_range_std_max(left: f32, right: f32) -> f32 {
    if left < right { right } else { left }
}

impl StaticRangeMap {
    fn from_words(text: &str, start: usize, end: usize) -> Self {
        Self {
            units: StaticTextModifierRange::word_range_units(text, start, end),
            terminal_index: (!text.is_empty()).then_some(end),
        }
    }

    fn unit_count(&self) -> usize {
        self.units.len()
    }

    fn unit_character_index_count(&self) -> usize {
        self.units.len() + usize::from(self.terminal_index.is_some())
    }

    fn empty(&self) -> bool {
        self.units.is_empty()
    }

    fn unit_character_index(&self, at: usize) -> usize {
        self.units
            .get(at)
            .map(|unit| unit.start)
            .or_else(|| {
                (at == self.units.len())
                    .then_some(self.terminal_index)
                    .flatten()
            })
            .expect("range-map character index must be in bounds")
    }

    fn unit_length(&self, at: usize) -> usize {
        self.units
            .get(at)
            .map(|unit| unit.len)
            .expect("range-map unit length must be in bounds")
    }

    fn unit_to_character_range(&self, unit: f32) -> f32 {
        if self.unit_character_index_count() == 0 {
            return 0.0;
        }
        let clamped = text_range_std_min(
            text_range_std_max(unit, 0.0),
            (self.unit_character_index_count() - 1) as f32,
        );
        let integer = clamped as usize;
        let mut characters = self.unit_character_index(integer) as f32;
        if integer < self.unit_count() {
            characters += self.unit_length(integer) as f32 * (clamped - integer as f32);
        }
        characters
    }
}

fn add_range_unit(
    units: &mut Vec<StaticRangeUnit>,
    index_from: usize,
    index_to: usize,
    start_offset: usize,
    end_offset: usize,
) {
    if index_to > start_offset && end_offset > index_from {
        let actual_start = start_offset.max(index_from);
        let actual_end = end_offset.min(index_to);
        if actual_end > actual_start {
            units.push(StaticRangeUnit {
                start: actual_start,
                len: actual_end - actual_start,
            });
        }
    }
}

impl StaticTextModifierRange {
    fn from_graph_with_instance(
        runtime: &RuntimeFile,
        graph: &ArtboardGraph,
        instance: Option<&ArtboardInstance>,
        local_id: usize,
    ) -> Result<Self> {
        let mut range = Self::from_graph(runtime, graph, local_id)?;
        let Some(instance) = instance else {
            return Ok(range);
        };
        let component = instance.component(local_id).with_context(|| {
            format!("TextModifierRange local {local_id} occurrence component is missing")
        })?;
        range.interpolator = component
            .children
            .iter()
            .filter_map(|child| instance.component_local_id(*child))
            .filter(|child_local| {
                instance
                    .component(*child_local)
                    .and_then(|child| nuxie_schema::definition_by_name(child.type_name))
                    .is_some_and(|definition| definition.is_a("CubicInterpolatorComponent"))
            })
            .map(|child_local| -> Result<StaticCubicInterpolator> {
                StaticCubicInterpolator::from_graph(runtime, graph, child_local)
            })
            .last()
            .transpose()?;
        Ok(range)
    }

    fn from_graph(runtime: &RuntimeFile, graph: &ArtboardGraph, local_id: usize) -> Result<Self> {
        let global_id = global_for_local(graph, local_id)?;
        let object = runtime
            .object(global_id as usize)
            .with_context(|| format!("missing TextModifierRange global {global_id}"))?;
        let run_id = object.uint_property("runId").unwrap_or(u32::MAX as u64);
        let run_local = if run_id == u32::MAX as u64 {
            None
        } else {
            let run_local = usize::try_from(run_id)
                .context("TextModifierRange runId does not fit a local object id")?;
            let run_type = type_for_local(graph, run_local)
                .with_context(|| format!("TextModifierRange runId {run_id} did not resolve"))?;
            if !nuxie_schema::definition_by_name(run_type)
                .is_some_and(|definition| definition.is_a("TextValueRun"))
            {
                bail!("TextModifierRange runId {run_id} is not a TextValueRun");
            }
            Some(run_local)
        };

        let component = component_for_local(graph, local_id)
            .with_context(|| format!("TextModifierRange local {local_id} component is missing"))?;
        let interpolator = component
            .children
            .iter()
            .filter(|child_local| {
                type_for_local(graph, **child_local)
                    .and_then(nuxie_schema::definition_by_name)
                    .is_some_and(|definition| definition.is_a("CubicInterpolatorComponent"))
            })
            .map(|child_local| -> Result<StaticCubicInterpolator> {
                StaticCubicInterpolator::from_graph(runtime, graph, *child_local)
            })
            .last()
            .transpose()?;

        Ok(Self {
            local_id,
            global_id,
            run_local,
            interpolator,
        })
    }

    fn needs_shape(&self, runtime: &RuntimeFile, instance: &ArtboardInstance) -> Result<bool> {
        Ok(self.uint_property(runtime, instance, "unitsValue", 0)? == 3)
    }

    fn apply_coverage(
        &self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
        text: &str,
        runs: &[StaticResolvedRun],
        lines: &[StaticTextLine],
        glyph_lookup_counts: &[usize],
        coverage: &mut [f32],
    ) -> Result<()> {
        if coverage.is_empty() {
            return Ok(());
        }
        let (start, end) = self.character_range(instance, runs, coverage.len())?;
        let units_value = self.uint_property(runtime, instance, "unitsValue", 0)?;
        let range_map = self.range_map(
            instance,
            units_value,
            text,
            start,
            end,
            lines,
            glyph_lookup_counts,
        )?;
        if range_map.empty() {
            return Ok(());
        }
        let unit_count = range_map.unit_count() as f32;
        let offset = self.double_property(runtime, instance, "offset", 0.0)?;
        let range_type = self.uint_property(runtime, instance, "typeValue", 0)?;
        let next_indices = match range_type {
            0 => Some([
                unit_count * (self.double_property(runtime, instance, "modifyFrom", 0.0)? + offset),
                unit_count * (self.double_property(runtime, instance, "modifyTo", 1.0)? + offset),
                unit_count
                    * (self.double_property(runtime, instance, "falloffFrom", 0.0)? + offset),
                unit_count * (self.double_property(runtime, instance, "falloffTo", 1.0)? + offset),
            ]),
            1 => Some([
                self.double_property(runtime, instance, "modifyFrom", 0.0)? + offset,
                self.double_property(runtime, instance, "modifyTo", 1.0)? + offset,
                self.double_property(runtime, instance, "falloffFrom", 0.0)? + offset,
                self.double_property(runtime, instance, "falloffTo", 1.0)? + offset,
            ]),
            _ => None,
        };
        let [index_from, index_to, falloff_from, falloff_to] = instance
            .component_parent_local(self.local_id)
            .and_then(|group| instance.component_parent_local(group))
            .and_then(|text| instance.component(text))
            .and_then(|text| text.concrete.text.as_ref())
            .map(|text| text.modifier_range_indices(self.local_id, next_indices))
            .unwrap_or_else(|| next_indices.unwrap_or([0.0; 4]));
        let strength = self.double_property(runtime, instance, "strength", 1.0)?;
        let mode = self.uint_property(runtime, instance, "modeValue", 0)?;
        let clamp = self.bool_property(runtime, instance, "clamp", false)?;

        for unit_index in 0..range_map.unit_count() {
            let unit_start = range_map.unit_character_index(unit_index);
            let unit_length = range_map.unit_length(unit_index);
            let t = unit_index as f32 + 0.5;
            let c = strength
                * self.coverage_at(
                    runtime,
                    instance,
                    t,
                    index_from,
                    index_to,
                    falloff_from,
                    falloff_to,
                )?;
            for character_index in unit_start..unit_start + unit_length {
                let current = coverage[character_index];
                let next = match mode {
                    0 => current + c,
                    1 => current - c,
                    2 => current * c,
                    3 => c.lt(&current).then_some(c).unwrap_or(current),
                    4 => current.lt(&c).then_some(c).unwrap_or(current),
                    5 => (current - c).abs(),
                    _ => current,
                };
                coverage[character_index] = if clamp {
                    let upper = next.lt(&1.0).then_some(next).unwrap_or(1.0);
                    0.0f32.lt(&upper).then_some(upper).unwrap_or(0.0)
                } else {
                    next
                };
            }

            if unit_index + 1 < range_map.unit_character_index_count() {
                let next_start = range_map.unit_character_index(unit_index + 1);
                for character_index in unit_start + unit_length..next_start {
                    coverage[character_index] = 0.0;
                }
            }
        }

        Ok(())
    }

    fn character_range(
        &self,
        instance: &ArtboardInstance,
        runs: &[StaticResolvedRun],
        text_len: usize,
    ) -> Result<(usize, usize)> {
        let Some(run_local) = self.run_local else {
            return Ok((0, text_len));
        };
        let run = runs
            .iter()
            .find(|run| run.local_id == run_local)
            .with_context(|| {
                format!("TextModifierRange run local {run_local} has no Text offset")
            })?;
        if let Some((offset, length)) = crate::text_value_run_owner::offset(instance, run_local)
            .zip(crate::text_value_run_owner::length(instance, run_local))
        {
            return Ok((offset as usize, offset.wrapping_add(length) as usize));
        }
        Ok((
            run.value_run_offset,
            run.value_run_offset + run.value_run_length,
        ))
    }

    fn range_map(
        &self,
        instance: &ArtboardInstance,
        units_value: u64,
        text: &str,
        start: usize,
        end: usize,
        lines: &[StaticTextLine],
        glyph_lookup_counts: &[usize],
    ) -> Result<StaticRangeMap> {
        let compute = || match units_value {
            1 => Self::character_range_units(text, start, end, glyph_lookup_counts, true),
            2 => StaticRangeMap::from_words(text, start, end).units,
            3 => Self::line_range_units(lines, start, end),
            _ => Self::character_range_units(text, start, end, glyph_lookup_counts, false),
        };
        let Some(text_state) = instance
            .component_parent_local(self.local_id)
            .and_then(|group| instance.component_parent_local(group))
            .and_then(|text| instance.component(text))
            .and_then(|text| text.concrete.text.as_ref())
        else {
            return Ok(StaticRangeMap {
                units: compute(),
                terminal_index: (!text.is_empty()).then_some(end),
            });
        };
        let units = text_state.modifier_range_units(self.local_id, || {
            compute()
                .into_iter()
                .map(|unit| (unit.start, unit.len))
                .collect()
        });
        Ok(StaticRangeMap {
            units: units
                .into_iter()
                .map(|(start, len)| StaticRangeUnit { start, len })
                .collect(),
            terminal_index: (!text.is_empty()).then_some(end),
        })
    }

    fn character_range_units(
        text: &str,
        start: usize,
        end: usize,
        glyph_lookup_counts: &[usize],
        without_spaces: bool,
    ) -> Vec<StaticRangeUnit> {
        let characters = text.chars().collect::<Vec<_>>();
        let mut units = Vec::new();
        let mut index = start;
        while index < end {
            if without_spaces
                && characters
                    .get(index)
                    .is_some_and(|character| nuxie_render_api::is_white_space(*character))
            {
                index += 1;
                continue;
            }
            let length = glyph_lookup_counts
                .get(index)
                .copied()
                .filter(|length| *length != 0)
                .unwrap_or(1);
            units.push(StaticRangeUnit {
                start: index,
                len: length,
            });
            index += length;
        }
        units
    }

    fn word_range_units(text: &str, start: usize, end: usize) -> Vec<StaticRangeUnit> {
        let mut units = Vec::new();
        let mut word_start = None;
        for (index, ch) in text.chars().enumerate() {
            if nuxie_render_api::is_white_space(ch) {
                if let Some(index_from) = word_start.take() {
                    add_range_unit(&mut units, index_from, index, start, end);
                }
            } else if word_start.is_none() {
                word_start = Some(index);
            }
        }
        if let Some(index_from) = word_start {
            add_range_unit(&mut units, index_from, text.chars().count(), start, end);
        }
        units
    }

    fn line_range_units(
        lines: &[StaticTextLine],
        start: usize,
        end: usize,
    ) -> Vec<StaticRangeUnit> {
        let mut units = Vec::new();
        for line in lines {
            add_range_unit(
                &mut units,
                line.char_start,
                line.char_start + line.text.chars().count(),
                start,
                end,
            );
        }
        units
    }

    fn coverage_at(
        &self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
        t: f32,
        index_from: f32,
        index_to: f32,
        falloff_from: f32,
        falloff_to: f32,
    ) -> Result<f32> {
        let (mut c, use_interpolator) = if index_to >= index_from {
            if t < index_from || t > index_to {
                (0.0, false)
            } else if t < falloff_from {
                let range = text_range_std_max(0.0, falloff_from - index_from);
                if range == 0.0 {
                    (1.0, true)
                } else {
                    (text_range_std_max(0.0, t - index_from) / range, true)
                }
            } else if t > falloff_to {
                let range = text_range_std_max(0.0, index_to - falloff_to);
                if range == 0.0 {
                    (1.0, true)
                } else {
                    (
                        1.0 - text_range_std_min(1.0, (t - falloff_to) / range),
                        true,
                    )
                }
            } else {
                (1.0, false)
            }
        } else {
            (0.0, false)
        };
        if use_interpolator && let Some(interpolator) = self.interpolator {
            c = interpolator.transform(runtime, instance, c)?;
        }
        Ok(c)
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
            "TextModifierRange",
            self.local_id,
            self.global_id,
            property_name,
            default,
        )
    }

    fn uint_property(
        &self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
        property_name: &str,
        default: u64,
    ) -> Result<u64> {
        runtime_uint_property(
            runtime,
            instance,
            "TextModifierRange",
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
            "TextModifierRange",
            self.local_id,
            self.global_id,
            property_name,
            default,
        )
    }
}

fn text_modifier_range_changed(
    instance: &mut ArtboardInstance,
    range_local: usize,
    range_type_changed: bool,
) -> bool {
    let Some(group) = instance.component_parent_local(range_local) else {
        return false;
    };
    let Some(text) = modifier_group_text(instance, group) else {
        return false;
    };
    let mut changed = if range_type_changed || group_has_shape_modifier(instance, group) {
        crate::text_owner::modifier_shape_dirty(instance, text)
    } else {
        instance.add_dirt(text, crate::components::ComponentDirt::PAINT, false)
    };
    // Every pinned range callback dirties the Text first, then publishes the
    // group's TextCoverage dirt.
    changed |= instance.add_dirt(
        group,
        crate::components::ComponentDirt::TEXT_COVERAGE,
        false,
    );
    changed
}

fn text_modifier_range_double_property_changed(
    instance: &mut ArtboardInstance,
    local_id: usize,
    type_name: Option<&str>,
) -> Option<bool> {
    (type_name == Some("TextModifierRange"))
        .then(|| text_modifier_range_changed(instance, local_id, false))
}

fn text_modifier_range_bool_property_changed(
    instance: &mut ArtboardInstance,
    local_id: usize,
    type_name: Option<&str>,
    property_key: u16,
) -> Option<bool> {
    (type_name == Some("TextModifierRange")
        && property_key_for_name("TextModifierRange", "clamp") == Some(property_key))
    .then(|| text_modifier_range_changed(instance, local_id, false))
}

fn text_modifier_range_uint_property_changed(
    instance: &mut ArtboardInstance,
    local_id: usize,
    type_name: Option<&str>,
    property_key: u16,
) -> Option<bool> {
    if type_name != Some("TextModifierRange") {
        return None;
    }
    if property_key_for_name("TextModifierRange", "unitsValue") == Some(property_key) {
        return Some(text_modifier_range_changed(instance, local_id, true));
    }
    ["typeValue", "modeValue"]
        .into_iter()
        .any(|name| property_key_for_name("TextModifierRange", name) == Some(property_key))
        .then(|| text_modifier_range_changed(instance, local_id, false))
}

#[derive(Debug, Clone, Copy)]
struct StaticCubicInterpolator {
    local_id: usize,
    global_id: u32,
    solver: RuntimeCubicInterpolatorComponent,
}

impl StaticCubicInterpolator {
    fn from_graph(runtime: &RuntimeFile, graph: &ArtboardGraph, local_id: usize) -> Result<Self> {
        let global_id = global_for_local(graph, local_id)?;
        let object = runtime
            .object(global_id as usize)
            .with_context(|| format!("missing CubicInterpolatorComponent global {global_id}"))?;
        Ok(Self {
            local_id,
            global_id,
            solver: RuntimeCubicInterpolatorComponent::on_added_dirty(
                object.double_property("x1").unwrap_or(0.42),
                object.double_property("x2").unwrap_or(0.58),
            ),
        })
    }

    fn transform(
        self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
        factor: f32,
    ) -> Result<f32> {
        let y1 = runtime_double_property(
            runtime,
            instance,
            "CubicInterpolatorComponent",
            self.local_id,
            self.global_id,
            "y1",
            0.0,
        )?;
        let y2 = runtime_double_property(
            runtime,
            instance,
            "CubicInterpolatorComponent",
            self.local_id,
            self.global_id,
            "y2",
            1.0,
        )?;
        Ok(self.solver.transform(factor, y1, y2))
    }
}
