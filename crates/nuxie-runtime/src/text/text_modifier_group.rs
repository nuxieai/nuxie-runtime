#[derive(Debug, Clone)]
struct StaticTextModifierGroup {
    local_id: usize,
    global_id: u32,
    ranges: Vec<StaticTextModifierRange>,
    modifiers: Vec<StaticTextModifier>,
    shape_modifier_indices: Vec<usize>,
    follow_path_modifiers: Vec<StaticTextFollowPathModifier>,
}

fn apply_text_modifier_transform(
    transform: Mat2D,
    mut ctm: Mat2D,
    origin: Option<(f32, f32)>,
) -> Mat2D {
    if let Some((origin_x, origin_y)) = origin {
        ctm.0[4] += origin_x;
        ctm.0[5] += origin_y;
    }
    ctm = transform.multiply(ctm);
    if let Some((origin_x, origin_y)) = origin {
        ctm.0[4] -= origin_x;
        ctm.0[5] -= origin_y;
    }
    ctm
}

fn text_modifier_group_modifies_transform(flags: u64) -> bool {
    const MODIFY_ORIGIN: u64 = 1 << 0;
    const MODIFY_TRANSLATION: u64 = 1 << 2;
    const MODIFY_ROTATION: u64 = 1 << 3;
    const MODIFY_SCALE: u64 = 1 << 4;
    flags & (MODIFY_ORIGIN | MODIFY_TRANSLATION | MODIFY_ROTATION | MODIFY_SCALE) != 0
}

/// Pinned `TextModifierGroup::transform` contracts the authored scale product
/// into the final addition on the supported clang/AArch64 build.
fn text_modifier_group_scale_component(amount: f32, scale: f32) -> f32 {
    scale.mul_add(amount, 1.0 - amount)
}

/// Pinned `TextModifierGroup::computeOpacity` computes the opacity product,
/// then contracts the current-opacity product into that add.
fn text_modifier_group_inverted_opacity(current: f32, amount: f32, opacity: f32) -> f32 {
    current.mul_add(1.0 - amount, opacity * amount)
}

impl StaticTextModifierGroup {
    fn from_graph(runtime: &RuntimeFile, graph: &ArtboardGraph, local_id: usize) -> Result<Self> {
        let global_id = global_for_local(graph, local_id)?;
        let object = runtime
            .object(global_id as usize)
            .with_context(|| format!("missing TextModifierGroup global {global_id}"))?;
        let flags = object.uint_property("modifierFlags").unwrap_or(0);
        const MODIFY_ORIGIN: u64 = 1 << 0;
        const MODIFY_TRANSLATION: u64 = 1 << 2;
        const MODIFY_ROTATION: u64 = 1 << 3;
        const MODIFY_SCALE: u64 = 1 << 4;
        const MODIFY_OPACITY: u64 = 1 << 5;
        const INVERT_OPACITY: u64 = 1 << 6;
        if flags
            & !(MODIFY_ORIGIN
                | MODIFY_TRANSLATION
                | MODIFY_ROTATION
                | MODIFY_SCALE
                | MODIFY_OPACITY
                | INVERT_OPACITY)
            != 0
        {
            bail!(
                "TextModifierGroup has unsupported modifier flags {flags}"
            );
        }

        let component = component_for_local(graph, local_id)
            .with_context(|| format!("TextModifierGroup local {local_id} component is missing"))?;
        let mut ranges = Vec::new();
        let mut modifiers = Vec::new();
        let mut shape_modifier_indices = Vec::new();
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
                Some(type_name) => {
                    let modifier = StaticTextModifier::from_group_child(
                        runtime,
                        graph,
                        *child_local,
                    )?
                    .with_context(|| {
                        format!(
                            "static text subset does not support TextModifierGroup child {type_name}"
                        )
                    })?;
                    if modifier.is_shape_modifier() {
                        shape_modifier_indices.push(modifiers.len());
                    }
                    if let Some(follow_path) = modifier.follow_path() {
                        follow_path_modifiers.push(follow_path.clone());
                    }
                    modifiers.push(modifier);
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
            modifiers,
            shape_modifier_indices,
            follow_path_modifiers,
        })
    }

    fn transform(
        &self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
        amount: f32,
        ctm: Mat2D,
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
        const MODIFY_ORIGIN: u64 = 1 << 0;
        let follows_path = !self.follow_path_modifiers.is_empty();
        if amount == 0.0 || (!text_modifier_group_modifies_transform(flags) && !follows_path) {
            return Ok(ctm);
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
            scale_x = text_modifier_group_scale_component(
                amount,
                runtime_double_property(
                    runtime,
                    instance,
                    "TextModifierGroup",
                    self.local_id,
                    self.global_id,
                    "scaleX",
                    1.0,
                )?,
            );
            scale_y = text_modifier_group_scale_component(
                amount,
                runtime_double_property(
                    runtime,
                    instance,
                    "TextModifierGroup",
                    self.local_id,
                    self.global_id,
                    "scaleY",
                    1.0,
                )?,
            );
        }
        let mut transform = Mat2D::from_rotation(rotation);
        transform.0[4] = x;
        transform.0[5] = y;
        transform.scale_by_values(scale_x, scale_y);
        let origin = if flags & MODIFY_ORIGIN != 0 {
            Some((
                runtime_double_property(
                    runtime,
                    instance,
                    "TextModifierGroup",
                    self.local_id,
                    self.global_id,
                    "originX",
                    0.0,
                )?,
                runtime_double_property(
                    runtime,
                    instance,
                    "TextModifierGroup",
                    self.local_id,
                    self.global_id,
                    "originY",
                    0.0,
                )?,
            ))
        } else {
            None
        };
        Ok(apply_text_modifier_transform(transform, ctm, origin))
    }

    fn reset_text_follow_path(&self, instance: &ArtboardInstance, inverse_text: Option<Mat2D>) {
        let Some(inverse_text) = inverse_text else {
            return;
        };
        for modifier in &self.follow_path_modifiers {
            modifier.reset(instance, inverse_text);
        }
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
            Ok(text_modifier_group_inverted_opacity(
                current, amount, opacity,
            ))
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
        lines: &[StaticTextLine],
        glyph_lookup_counts: &[usize],
    ) -> Result<Vec<f32>> {
        let mut coverage = vec![0.0; text.chars().count()];
        for range in &self.ranges {
            range.apply_coverage(
                runtime,
                instance,
                text,
                runs,
                lines,
                glyph_lookup_counts,
                &mut coverage,
            )?;
        }
        Ok(coverage)
    }

    fn variation_map(
        &self,
        instance: &ArtboardInstance,
        font: &SkrifaFontRef<'_>,
        strength: f32,
        font_axes: &BTreeMap<u32, f32>,
    ) -> BTreeMap<u32, f32> {
        let mut variations = BTreeMap::new();
        for index in &self.shape_modifier_indices {
            if let Some(StaticTextModifier::Variation(modifier)) = self.modifiers.get(*index) {
                modifier.modify(instance, font, font_axes, &mut variations, strength);
            }
        }
        variations
    }

    fn has_shape_modifiers(&self) -> bool {
        !self.shape_modifier_indices.is_empty()
    }
}

fn modifier_group_text(instance: &ArtboardInstance, group_local: usize) -> Option<usize> {
    let text = instance.component_parent_local(group_local)?;
    instance
        .component(text)
        .and_then(|component| nuxie_schema::definition_by_name(component.type_name))
        .is_some_and(|definition| definition.is_a("Text"))
        .then_some(text)
}

fn group_has_shape_modifier(instance: &ArtboardInstance, group_local: usize) -> bool {
    let Some(group) = instance.component(group_local) else {
        return false;
    };
    group.children.iter().any(|child| {
        instance
            .component_local_id(*child)
            .and_then(|local| instance.component(local))
            .is_some_and(|component| {
                nuxie_schema::definition_by_name(component.type_name)
                    .is_some_and(|definition| definition.is_a("TextShapeModifier"))
            })
    })
}

fn range_changed(instance: &mut ArtboardInstance, range_local: usize, path_only: bool) -> bool {
    let Some(group) = instance.component_parent_local(range_local) else {
        return false;
    };
    let Some(text) = modifier_group_text(instance, group) else {
        return false;
    };
    let mut changed = if path_only {
        crate::text_owner::modifier_shape_dirty(instance, text)
    } else if group_has_shape_modifier(instance, group) {
        crate::text_owner::modifier_shape_dirty(instance, text)
    } else {
        instance.add_dirt(text, crate::components::ComponentDirt::PAINT, false)
    };
    // `rangeTypeChanged`/`rangeChanged` dirty Text first, then their group.
    changed |= instance.add_dirt(
        group,
        crate::components::ComponentDirt::TEXT_COVERAGE,
        false,
    );
    changed
}

pub(crate) fn text_modifier_group_double_property_changed(
    instance: &mut ArtboardInstance,
    local_id: usize,
    type_name: Option<&str>,
    property_key: u16,
) -> Option<bool> {
    if type_name == Some("TextModifierGroup") {
        let is_paint = [
            "originX", "originY", "opacity", "x", "y", "rotation", "scaleX", "scaleY",
        ]
        .into_iter()
        .any(|name| property_key_for_name("TextModifierGroup", name) == Some(property_key));
        return is_paint.then(|| {
            modifier_group_text(instance, local_id).is_some_and(|text| {
                instance.add_dirt(text, crate::components::ComponentDirt::PAINT, false)
            })
        });
    }
    (type_name == Some("TextModifierRange")).then(|| range_changed(instance, local_id, false))
}

pub(crate) fn text_modifier_group_bool_property_changed(
    instance: &mut ArtboardInstance,
    local_id: usize,
    type_name: Option<&str>,
    property_key: u16,
) -> Option<bool> {
    (type_name == Some("TextModifierRange")
        && property_key_for_name("TextModifierRange", "clamp") == Some(property_key))
    .then(|| range_changed(instance, local_id, false))
}

pub(crate) fn text_modifier_group_uint_property_changed(
    instance: &mut ArtboardInstance,
    local_id: usize,
    type_name: Option<&str>,
    property_key: u16,
) -> Option<bool> {
    if type_name == Some("TextModifierGroup")
        && property_key_for_name("TextModifierGroup", "modifierFlags") == Some(property_key)
    {
        return Some(modifier_group_text(instance, local_id).is_some_and(|text| {
            instance.add_dirt(text, crate::components::ComponentDirt::PAINT, false)
        }));
    }
    if type_name != Some("TextModifierRange") {
        return None;
    }
    if property_key_for_name("TextModifierRange", "unitsValue") == Some(property_key) {
        return Some(range_changed(instance, local_id, true));
    }
    ["typeValue", "modeValue"]
        .into_iter()
        .any(|name| property_key_for_name("TextModifierRange", name) == Some(property_key))
        .then(|| range_changed(instance, local_id, false))
}

#[derive(Debug, Clone)]
struct StaticTextGlyphContext<'a> {
    origin_x: f32,
    origin_y: f32,
    line_index_in_paragraph: usize,
    paragraph_baselines: &'a [f32],
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
