#[derive(Debug, Clone)]
struct StaticTextModifierRange {
    local_id: usize,
    global_id: u32,
    interpolator: Option<StaticCubicInterpolator>,
}
#[derive(Debug, Clone, Copy)]
struct StaticRangeUnit {
    start: usize,
    len: usize,
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

trait StaticTextWords {
    fn split_word_bound_indices(&self) -> Vec<(usize, &str)>;
}

impl StaticTextWords for str {
    fn split_word_bound_indices(&self) -> Vec<(usize, &str)> {
        let mut words = Vec::new();
        let mut start = None;
        for (index, ch) in self.char_indices() {
            if nuxie_render_api::is_white_space(ch) {
                if let Some(word_start) = start.take() {
                    words.push((word_start, &self[word_start..index]));
                }
            } else if start.is_none() {
                start = Some(index);
            }
        }
        if let Some(word_start) = start {
            words.push((word_start, &self[word_start..]));
        }
        words
    }
}
impl StaticTextModifierRange {
    fn from_graph(runtime: &RuntimeFile, graph: &ArtboardGraph, local_id: usize) -> Result<Self> {
        let global_id = global_for_local(graph, local_id)?;
        let object = runtime
            .object(global_id as usize)
            .with_context(|| format!("missing TextModifierRange global {global_id}"))?;
        let units = object.uint_property("unitsValue").unwrap_or(0);
        if units > 3 {
            bail!("static text subset does not support TextModifierRange units {units}");
        }

        let component = component_for_local(graph, local_id)
            .with_context(|| format!("TextModifierRange local {local_id} component is missing"))?;
        let mut interpolator = None;
        for child_local in &component.children {
            match type_for_local(graph, *child_local) {
                Some("CubicInterpolatorComponent") => {
                    if interpolator.is_some() {
                        bail!("static text subset supports one TextModifierRange interpolator");
                    }
                    interpolator = Some(StaticCubicInterpolator {
                        local_id: *child_local,
                        global_id: global_for_local(graph, *child_local)?,
                    });
                }
                Some(type_name) => {
                    bail!("static text subset does not support TextModifierRange child {type_name}")
                }
                None => bail!(
                    "static text subset does not support unknown TextModifierRange child local {child_local}"
                ),
            }
        }

        Ok(Self {
            local_id,
            global_id,
            interpolator,
        })
    }

    fn apply_coverage(
        &self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
        text: &str,
        runs: &[StaticResolvedRun],
        lines: &[StaticTextLine<'_>],
        coverage: &mut [f32],
    ) -> Result<()> {
        if coverage.is_empty() {
            return Ok(());
        }
        let (start, end) = self.character_range(runtime, instance, runs, coverage.len())?;
        let units_value = self.uint_property(runtime, instance, "unitsValue", 0)?;
        let units = self.range_units(units_value, text, start, end, lines)?;
        if units.is_empty() {
            return Ok(());
        }
        let unit_count = units.len() as f32;
        let offset = self.double_property(runtime, instance, "offset", 0.0)?;
        let range_type = self.uint_property(runtime, instance, "typeValue", 0)?;
        let (index_from, index_to, falloff_from, falloff_to) = match range_type {
            0 => (
                unit_count * (self.double_property(runtime, instance, "modifyFrom", 0.0)? + offset),
                unit_count * (self.double_property(runtime, instance, "modifyTo", 1.0)? + offset),
                unit_count
                    * (self.double_property(runtime, instance, "falloffFrom", 0.0)? + offset),
                unit_count * (self.double_property(runtime, instance, "falloffTo", 1.0)? + offset),
            ),
            1 => (
                self.double_property(runtime, instance, "modifyFrom", 0.0)? + offset,
                self.double_property(runtime, instance, "modifyTo", 1.0)? + offset,
                self.double_property(runtime, instance, "falloffFrom", 0.0)? + offset,
                self.double_property(runtime, instance, "falloffTo", 1.0)? + offset,
            ),
            other => bail!("static text subset does not support TextModifierRange type {other}"),
        };
        let strength = self.double_property(runtime, instance, "strength", 1.0)?;
        let mode = self.uint_property(runtime, instance, "modeValue", 0)?;
        let clamp = self.bool_property(runtime, instance, "clamp", false)?;

        for (unit_index, unit) in units.iter().enumerate() {
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
            for character_index in unit.start..unit.start + unit.len {
                let current = coverage[character_index];
                let next = match mode {
                    0 => current + c,
                    1 => current - c,
                    2 => current * c,
                    3 => current.min(c),
                    4 => current.max(c),
                    5 => (current - c).abs(),
                    other => {
                        bail!("static text subset does not support TextModifierRange mode {other}")
                    }
                };
                coverage[character_index] = if clamp { next.clamp(0.0, 1.0) } else { next };
            }

            let next_start = units
                .get(unit_index + 1)
                .map(|next| next.start)
                .unwrap_or(end);
            for character_index in unit.start + unit.len..next_start {
                coverage[character_index] = 0.0;
            }
        }

        Ok(())
    }

    fn character_range(
        &self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
        runs: &[StaticResolvedRun],
        text_len: usize,
    ) -> Result<(usize, usize)> {
        let run_id = self.uint_property(runtime, instance, "runId", u32::MAX as u64)?;
        if run_id == u32::MAX as u64 {
            return Ok((0, text_len));
        }
        let run = runs
            .iter()
            .find(|run| run.local_id as u64 == run_id || run.global_id as u64 == run_id)
            .with_context(|| format!("TextModifierRange runId {run_id} did not resolve"))?;
        Ok((run.char_start, run.char_start + run.char_len))
    }

    fn range_units(
        &self,
        units_value: u64,
        text: &str,
        start: usize,
        end: usize,
        lines: &[StaticTextLine<'_>],
    ) -> Result<Vec<StaticRangeUnit>> {
        match units_value {
            0 => Ok((start..end)
                .map(|index| StaticRangeUnit {
                    start: index,
                    len: 1,
                })
                .collect()),
            1 => Ok(text
                .chars()
                .enumerate()
                .skip(start)
                .take(end.saturating_sub(start))
                .filter_map(|(index, ch)| {
                    (!nuxie_render_api::is_white_space(ch)).then_some(StaticRangeUnit {
                        start: index,
                        len: 1,
                    })
                })
                .collect()),
            2 => Ok(Self::word_range_units(text, start, end)),
            3 => Ok(Self::line_range_units(lines, start, end)),
            other => bail!("static text subset does not support TextModifierRange units {other}"),
        }
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
        lines: &[StaticTextLine<'_>],
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
        let mut c = if index_to < index_from || t < index_from || t > index_to {
            0.0
        } else if t < falloff_from {
            let range = (falloff_from - index_from).max(0.0);
            if range == 0.0 {
                1.0
            } else {
                ((t - index_from).max(0.0) / range).max(0.0)
            }
        } else if t > falloff_to {
            let range = (index_to - falloff_to).max(0.0);
            if range == 0.0 {
                1.0
            } else {
                1.0 - ((t - falloff_to) / range).min(1.0)
            }
        } else {
            1.0
        };
        if c != 0.0
            && c != 1.0
            && let Some(interpolator) = self.interpolator
        {
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

#[derive(Debug, Clone, Copy)]
struct StaticCubicInterpolator {
    local_id: usize,
    global_id: u32,
}

impl StaticCubicInterpolator {
    fn transform(
        self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
        factor: f32,
    ) -> Result<f32> {
        let x1 = runtime_double_property(
            runtime,
            instance,
            "CubicInterpolatorComponent",
            self.local_id,
            self.global_id,
            "x1",
            0.42,
        )?;
        let y1 = runtime_double_property(
            runtime,
            instance,
            "CubicInterpolatorComponent",
            self.local_id,
            self.global_id,
            "y1",
            0.0,
        )?;
        let x2 = runtime_double_property(
            runtime,
            instance,
            "CubicInterpolatorComponent",
            self.local_id,
            self.global_id,
            "x2",
            0.58,
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
        let t = cubic_interpolator_get_t(factor, x1, x2);
        Ok(cubic_interpolator_calc_bezier(t, y1, y2))
    }
}

fn cubic_interpolator_calc_bezier(t: f32, a1: f32, a2: f32) -> f32 {
    (((1.0 - 3.0 * a2 + 3.0 * a1) * t + (3.0 * a2 - 6.0 * a1)) * t + (3.0 * a1)) * t
}

fn cubic_interpolator_slope(t: f32, a1: f32, a2: f32) -> f32 {
    3.0 * (1.0 - 3.0 * a2 + 3.0 * a1) * t * t
        + 2.0 * (3.0 * a2 - 6.0 * a1) * t
        + (3.0 * a1)
}

fn cubic_interpolator_get_t(x: f32, x1: f32, x2: f32) -> f32 {
    const SPLINE_TABLE_SIZE: usize = 11;
    const SAMPLE_STEP_SIZE: f32 = 1.0 / (SPLINE_TABLE_SIZE as f32 - 1.0);
    const NEWTON_ITERATIONS: usize = 4;
    const NEWTON_MIN_SLOPE: f32 = 0.001;
    const SUBDIVISION_PRECISION: f32 = 0.0000001;
    const SUBDIVISION_MAX_ITERATIONS: usize = 10;

    let mut values = [0.0; SPLINE_TABLE_SIZE];
    for (i, value) in values.iter_mut().enumerate() {
        *value = cubic_interpolator_calc_bezier(i as f32 * SAMPLE_STEP_SIZE, x1, x2);
    }

    let mut interval_start = 0.0;
    let mut current_sample = 1;
    let last_sample = SPLINE_TABLE_SIZE - 1;
    while current_sample != last_sample && values[current_sample] <= x {
        interval_start += SAMPLE_STEP_SIZE;
        current_sample += 1;
    }
    current_sample -= 1;

    let dist = (x - values[current_sample]) / (values[current_sample + 1] - values[current_sample]);
    let mut guess_for_t = interval_start + dist * SAMPLE_STEP_SIZE;
    let initial_slope = cubic_interpolator_slope(guess_for_t, x1, x2);
    if initial_slope >= NEWTON_MIN_SLOPE {
        for _ in 0..NEWTON_ITERATIONS {
            let current_slope = cubic_interpolator_slope(guess_for_t, x1, x2);
            if current_slope == 0.0 {
                return guess_for_t;
            }
            let current_x = cubic_interpolator_calc_bezier(guess_for_t, x1, x2) - x;
            guess_for_t -= current_x / current_slope;
        }
        guess_for_t
    } else if initial_slope == 0.0 {
        guess_for_t
    } else {
        let mut upper_bound = interval_start + SAMPLE_STEP_SIZE;
        let mut iterations = 0;
        loop {
            let current_t = interval_start + (upper_bound - interval_start) / 2.0;
            let current_x = cubic_interpolator_calc_bezier(current_t, x1, x2) - x;
            if current_x > 0.0 {
                upper_bound = current_t;
            } else {
                interval_start = current_t;
            }
            iterations += 1;
            if current_x.abs() <= SUBDIVISION_PRECISION
                || iterations >= SUBDIVISION_MAX_ITERATIONS
            {
                return current_t;
            }
        }
    }
}
