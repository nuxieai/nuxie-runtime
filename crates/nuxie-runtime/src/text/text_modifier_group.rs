#[derive(Debug, Clone)]
struct StaticTextModifierGroup {
    local_id: usize,
    global_id: u32,
    ranges: Vec<StaticTextModifierRange>,
    follow_path_modifiers: Vec<StaticTextFollowPathModifier>,
}
impl StaticTextModifierGroup {
    fn from_graph(runtime: &RuntimeFile, graph: &ArtboardGraph, local_id: usize) -> Result<Self> {
        let global_id = global_for_local(graph, local_id)?;
        let object = runtime
            .object(global_id as usize)
            .with_context(|| format!("missing TextModifierGroup global {global_id}"))?;
        let flags = object.uint_property("modifierFlags").unwrap_or(0);
        const MODIFY_TRANSLATION: u64 = 1 << 2;
        const MODIFY_ROTATION: u64 = 1 << 3;
        const MODIFY_SCALE: u64 = 1 << 4;
        const MODIFY_OPACITY: u64 = 1 << 5;
        const INVERT_OPACITY: u64 = 1 << 6;
        if flags
            & !(MODIFY_TRANSLATION
                | MODIFY_ROTATION
                | MODIFY_SCALE
                | MODIFY_OPACITY
                | INVERT_OPACITY)
            != 0
        {
            bail!(
                "static text subset only supports translation/rotation/scale/opacity TextModifierGroup flags, found {flags}"
            );
        }

        let component = component_for_local(graph, local_id)
            .with_context(|| format!("TextModifierGroup local {local_id} component is missing"))?;
        let mut ranges = Vec::new();
        let mut follow_path_modifiers = Vec::new();
        for child_local in &component.children {
            match type_for_local(graph, *child_local) {
                Some("TextModifierRange") => {
                    ranges.push(StaticTextModifierRange::from_graph(
                        runtime,
                        graph,
                        *child_local,
                    )?);
                }
                Some("TextFollowPathModifier") => {
                    follow_path_modifiers.push(StaticTextFollowPathModifier::from_graph(
                        runtime,
                        graph,
                        *child_local,
                    )?);
                }
                Some(type_name) => {
                    bail!("static text subset does not support TextModifierGroup child {type_name}")
                }
                None => bail!(
                    "static text subset does not support unknown TextModifierGroup child local {child_local}"
                ),
            }
        }

        Ok(Self {
            local_id,
            global_id,
            ranges,
            follow_path_modifiers,
        })
    }

    fn transform(
        &self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
        amount: f32,
        glyph: &StaticTextGlyphContext<'_>,
    ) -> Result<Mat2D> {
        let flags = runtime_uint_property(
            runtime,
            instance,
            "TextModifierGroup",
            self.local_id,
            self.global_id,
            "modifierFlags",
            0,
        )?;
        const MODIFY_TRANSLATION: u64 = 1 << 2;
        const MODIFY_ROTATION: u64 = 1 << 3;
        const MODIFY_SCALE: u64 = 1 << 4;
        let follows_path = !self.follow_path_modifiers.is_empty();
        if amount == 0.0 && !follows_path {
            return Ok(Mat2D::IDENTITY);
        }

        let mut x = 0.0;
        let mut y = 0.0;
        let mut rotation = 0.0;
        let mut scale_x = 1.0;
        let mut scale_y = 1.0;

        if follows_path {
            // Ported from C++ `src/text/text_modifier_group.cpp`
            // `TextModifierGroup::transform` for follow-path modifiers.
            let mut current = StaticFollowPathGlyphTransform {
                x: glyph.origin_x,
                y: glyph.origin_y,
                rotation: 0.0,
            };
            let offset = if flags & MODIFY_TRANSLATION != 0 {
                (
                    runtime_double_property(
                        runtime,
                        instance,
                        "TextModifierGroup",
                        self.local_id,
                        self.global_id,
                        "x",
                        0.0,
                    )?,
                    runtime_double_property(
                        runtime,
                        instance,
                        "TextModifierGroup",
                        self.local_id,
                        self.global_id,
                        "y",
                        0.0,
                    )?,
                )
            } else {
                (0.0, 0.0)
            };
            for modifier in &self.follow_path_modifiers {
                current = modifier.transform_glyph(runtime, instance, current, glyph, offset)?;
            }
            x = (current.x - glyph.origin_x) * amount;
            y = (current.y - glyph.origin_y) * amount;
            rotation += current.rotation * amount;
        } else if flags & MODIFY_TRANSLATION != 0 {
            x = runtime_double_property(
                runtime,
                instance,
                "TextModifierGroup",
                self.local_id,
                self.global_id,
                "x",
                0.0,
            )? * amount;
            y = runtime_double_property(
                runtime,
                instance,
                "TextModifierGroup",
                self.local_id,
                self.global_id,
                "y",
                0.0,
            )? * amount;
        }

        if flags & MODIFY_ROTATION != 0 {
            rotation += runtime_double_property(
                runtime,
                instance,
                "TextModifierGroup",
                self.local_id,
                self.global_id,
                "rotation",
                0.0,
            )? * amount;
        }
        if flags & MODIFY_SCALE != 0 {
            let inverse_amount = 1.0 - amount;
            scale_x = inverse_amount
                + runtime_double_property(
                    runtime,
                    instance,
                    "TextModifierGroup",
                    self.local_id,
                    self.global_id,
                    "scaleX",
                    1.0,
                )? * amount;
            scale_y = inverse_amount
                + runtime_double_property(
                    runtime,
                    instance,
                    "TextModifierGroup",
                    self.local_id,
                    self.global_id,
                    "scaleY",
                    1.0,
                )? * amount;
        }
        let mut transform = Mat2D::from_rotation(rotation);
        transform.0[4] = x;
        transform.0[5] = y;
        transform.scale_by_values(scale_x, scale_y);
        Ok(transform)
    }

    fn modifies_opacity(&self, runtime: &RuntimeFile, instance: &ArtboardInstance) -> Result<bool> {
        let flags = runtime_uint_property(
            runtime,
            instance,
            "TextModifierGroup",
            self.local_id,
            self.global_id,
            "modifierFlags",
            0,
        )?;
        const MODIFY_OPACITY: u64 = 1 << 5;
        Ok(flags & MODIFY_OPACITY != 0)
    }

    fn opacity(
        &self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
        current: f32,
        amount: f32,
    ) -> Result<f32> {
        let flags = runtime_uint_property(
            runtime,
            instance,
            "TextModifierGroup",
            self.local_id,
            self.global_id,
            "modifierFlags",
            0,
        )?;
        let opacity = runtime_double_property(
            runtime,
            instance,
            "TextModifierGroup",
            self.local_id,
            self.global_id,
            "opacity",
            1.0,
        )?;
        const INVERT_OPACITY: u64 = 1 << 6;
        if flags & INVERT_OPACITY != 0 {
            Ok(current * (1.0 - amount) + opacity * amount)
        } else {
            Ok(current * opacity * amount)
        }
    }

    fn coverage_by_character(
        &self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
        text: &str,
        runs: &[StaticResolvedRun],
        lines: &[StaticTextLine<'_>],
    ) -> Result<Vec<f32>> {
        let mut coverage = vec![0.0; text.chars().count()];
        for range in &self.ranges {
            range.apply_coverage(runtime, instance, text, runs, lines, &mut coverage)?;
        }
        Ok(coverage)
    }
}

#[derive(Debug, Clone)]
struct StaticTextGlyphContext<'a> {
    origin_x: f32,
    origin_y: f32,
    line_index_in_paragraph: usize,
    paragraph_baselines: &'a [f32],
    text_world_inverse: Mat2D,
}

fn transform_path_commands(commands: &mut [RuntimePathCommand], transform: Mat2D) {
    for command in commands {
        match command {
            RuntimePathCommand::Move { x, y } | RuntimePathCommand::Line { x, y } => {
                (*x, *y) = transform.transform_point(*x, *y);
            }
            RuntimePathCommand::Cubic {
                x1,
                y1,
                x2,
                y2,
                x3,
                y3,
            } => {
                (*x1, *y1) = transform.transform_point(*x1, *y1);
                (*x2, *y2) = transform.transform_point(*x2, *y2);
                (*x3, *y3) = transform.transform_point(*x3, *y3);
            }
            RuntimePathCommand::Close => {}
        }
    }
}

#[derive(Debug, Clone)]
struct StaticTextPathBucket {
    opacity: f32,
    commands: Vec<RuntimePathCommand>,
}

fn append_opacity_bucket(
    buckets: &mut Vec<StaticTextPathBucket>,
    opacity: f32,
    commands: Vec<RuntimePathCommand>,
) {
    if opacity <= 0.0 {
        return;
    }
    if let Some(bucket) = buckets.iter_mut().find(|bucket| bucket.opacity == opacity) {
        bucket.commands.extend(commands);
    } else {
        buckets.push(StaticTextPathBucket { opacity, commands });
    }
}

fn order_opacity_buckets_like_cpp(
    mut buckets: Vec<StaticTextPathBucket>,
) -> Vec<StaticTextPathBucket> {
    // `TextModifierGroup::computeOpacity` supplies the coverage buckets;
    // TextStylePaint retains them in the pinned ascending float map.
    buckets.sort_by(|a, b| {
        a.opacity
            .partial_cmp(&b.opacity)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    buckets
}
