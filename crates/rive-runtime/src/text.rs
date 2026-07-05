use anyhow::{Context, Result, bail};
use harfrust::{
    Direction, Feature, FontRef as HarfFontRef, ShapeOptions, ShaperData, ShaperInstance,
    Tag as HarfTag, UnicodeBuffer,
};
use rive_binary::RuntimeFile;
use rive_graph::{ArtboardGraph, ShapePaintContainerNode, ShapePaintKind, ShapePaintStateNode};
use skrifa::instance::{LocationRef, Size};
use skrifa::outline::pen::PathStyle;
use skrifa::outline::{DrawSettings, OutlinePen};
use skrifa::raw::TableProvider;
use skrifa::setting::VariationSetting;
use skrifa::{FontRef as SkrifaFontRef, GlyphId, MetadataProvider, Tag as SkrifaTag};
use std::collections::BTreeSet;

use crate::draw::runtime_shape_paint_command;
use crate::properties::property_key_for_name;
use crate::{ArtboardInstance, Mat2D, RuntimeDrawCommand, RuntimePathCommand};
use crate::{RuntimeShapePaintCommand, RuntimeShapePaintKind, RuntimeShapePaintPathKind};

const TEXT_SHAPE_SCALE: i32 = 2048;
const TEXT_SHAPE_SCALE_F32: f32 = TEXT_SHAPE_SCALE as f32;

pub fn static_text_support_error(
    runtime: &RuntimeFile,
    graph: &ArtboardGraph,
    text_local: usize,
) -> Option<String> {
    StaticTextSlice::from_graph(runtime, graph, text_local)
        .err()
        .map(|error| error.to_string())
}

pub(crate) fn static_text_constraint_bounds(
    runtime: &RuntimeFile,
    graph: &ArtboardGraph,
    instance: &ArtboardInstance,
    text_local: usize,
) -> Option<(f32, f32, f32, f32)> {
    StaticTextSlice::from_graph(runtime, graph, text_local)
        .ok()?
        .local_bounds(runtime, instance)
        .ok()
        .flatten()
}

pub(crate) fn runtime_text_shape_paint_commands(
    runtime: &RuntimeFile,
    instance: &ArtboardInstance,
    graph: &ArtboardGraph,
    command: &RuntimeDrawCommand,
) -> Result<Vec<RuntimeShapePaintCommand>> {
    let text_local = command
        .local_id
        .context("text draw command missing local id")?;
    let slice = StaticTextSlice::from_graph(runtime, graph, text_local)?;
    let path_commands = slice.path_commands(runtime, instance)?;
    if path_commands.is_empty() {
        return Ok(Vec::new());
    }
    let render_opacity = instance
        .component(text_local)
        .map(|component| component.transform.render_opacity)
        .unwrap_or(1.0);
    let shape_world = instance
        .component(text_local)
        .map(|component| component.transform.world_transform)
        .unwrap_or(Mat2D::IDENTITY);
    // C++ text draw isolates the glyph path transform even when clipping
    // elides the drawable-level save.
    let needs_save_operation = true;

    Ok(slice
        .container
        .paints
        .iter()
        .filter_map(|paint| {
            let mut command = runtime_shape_paint_command(
                instance,
                paint,
                slice.container.blend_mode_value,
                needs_save_operation,
                render_opacity,
                shape_world,
                path_commands.clone(),
            )?;
            if command.paint_type == RuntimeShapePaintKind::Fill {
                command.path_kind = RuntimeShapePaintPathKind::LocalClockwise;
            }
            Some(command)
        })
        .collect())
}

struct StaticTextSlice<'a> {
    text_local: usize,
    text_global: u32,
    runs: Vec<StaticTextRun>,
    style_local: usize,
    style_global: u32,
    container: &'a ShapePaintContainerNode,
    font_bytes: Option<&'a [u8]>,
    variations: Vec<(u32, f32)>,
    modifiers: Vec<StaticTextModifierGroup>,
}

#[derive(Debug, Clone)]
struct StaticTextRun {
    local_id: usize,
    global_id: u32,
}

#[derive(Debug, Clone, Copy)]
struct StaticTextLine<'a> {
    text: &'a str,
    char_start: usize,
    line_index: usize,
}

#[derive(Debug, Clone)]
struct StaticTextModifierGroup {
    local_id: usize,
    global_id: u32,
    ranges: Vec<StaticTextModifierRange>,
}

#[derive(Debug, Clone)]
struct StaticTextModifierRange {
    local_id: usize,
    global_id: u32,
    interpolator: Option<StaticCubicInterpolator>,
}

#[derive(Debug, Clone, Copy)]
struct StaticCubicInterpolator {
    local_id: usize,
    global_id: u32,
}

impl<'a> StaticTextSlice<'a> {
    fn from_graph(
        runtime: &'a RuntimeFile,
        graph: &'a ArtboardGraph,
        text_local: usize,
    ) -> Result<Self> {
        let text_object = graph
            .local_objects
            .iter()
            .find(|object| object.local_id == text_local)
            .context("text local object is missing")?;
        if text_object.type_name != Some("Text") {
            bail!("static text subset expected Text local {text_local}");
        }
        if !graph.data_binds.is_empty() {
            bail!("static text subset does not support text data binding");
        }
        if let Some(object) = graph.local_objects.iter().find(|object| {
            object.type_name.is_some_and(|type_name| {
                !matches!(
                    type_name,
                    "Artboard"
                        | "Text"
                        | "TextValueRun"
                        | "TextStylePaint"
                        | "TextStyleAxis"
                        | "TextModifierGroup"
                        | "TextModifierRange"
                        | "CubicInterpolatorComponent"
                        | "Shape"
                        | "PointsPath"
                        | "StraightVertex"
                        | "Triangle"
                        | "Ellipse"
                        | "Rectangle"
                        | "ClippingShape"
                        | "SolidColor"
                        | "Fill"
                        | "Backboard"
                        | "RootBone"
                        | "Skin"
                        | "Tendon"
                        | "Weight"
                        | "KeyedObject"
                        | "KeyedProperty"
                        | "LinearAnimation"
                        | "KeyFrameColor"
                        | "KeyFrameBool"
                        | "TransformConstraint"
                        | "LayoutComponentStyle"
                        | "ViewModel"
                        | "ViewModelInstance"
                        | "StateMachine"
                        | "StateMachineLayer"
                        | "StateMachineBool"
                        | "ListenerBoolChange"
                        | "AnimationState"
                        | "AnyState"
                        | "EntryState"
                        | "ExitState"
                        | "StateTransition"
                        | "TransitionBoolCondition"
                )
            })
        }) {
            bail!(
                "static text subset does not support sibling {} global {}",
                object.type_name.unwrap_or("unknown"),
                object.global_id
            );
        }
        if let Some(object) = graph.local_objects.iter().find(|object| {
            matches!(
                object.type_name,
                Some(
                    "TextInput"
                        | "TextInputDrawable"
                        | "TextInputCursor"
                        | "TextInputText"
                        | "TextInputSelection"
                        | "TextInputSelectedText"
                        | "TextStyleFeature"
                        | "TextModifier"
                        | "TextShapeModifier"
                        | "TextVariationModifier"
                        | "TextTargetModifier"
                        | "TextFollowPathModifier"
                        | "LayoutComponent"
                        | "NestedArtboardLayout"
                        | "NestedArtboardLeaf"
                        | "ArtboardComponentList"
                        | "ForegroundLayoutDrawable"
                )
            )
        }) {
            bail!(
                "static text subset does not support {} global {}",
                object.type_name.unwrap_or("unknown"),
                object.global_id
            );
        }

        let text_component = graph
            .components
            .iter()
            .find(|component| component.local_id == text_local)
            .context("text component is missing")?;
        if text_component.parent_local != Some(0) {
            bail!("static text subset only supports top-level Text");
        }

        let child_type = |local_id| {
            graph
                .local_objects
                .iter()
                .find(|object| object.local_id == local_id)
                .and_then(|object| object.type_name)
        };
        let run_locals = text_component
            .children
            .iter()
            .copied()
            .filter(|local| child_type(*local) == Some("TextValueRun"))
            .collect::<Vec<_>>();
        let style_locals = text_component
            .children
            .iter()
            .copied()
            .filter(|local| child_type(*local) == Some("TextStylePaint"))
            .collect::<Vec<_>>();
        if run_locals.is_empty() {
            bail!(
                "static text subset requires at least one TextValueRun child, found {}",
                run_locals.len()
            );
        }
        if style_locals.len() != 1 {
            bail!(
                "static text subset requires exactly one TextStylePaint child, found {}",
                style_locals.len()
            );
        }
        let style_local = style_locals[0];
        let text_global = global_for_local(graph, text_local)?;
        let style_global = global_for_local(graph, style_local)?;
        let mut runs = Vec::new();
        let mut authored_text = String::new();
        for run_local in run_locals {
            let run_global = global_for_local(graph, run_local)?;
            let run = runtime
                .object(run_global as usize)
                .with_context(|| format!("missing TextValueRun global {run_global}"))?;
            if run.uint_property("styleId") != Some(style_local as u64) {
                bail!(
                    "static text subset requires every TextValueRun to reference the only TextStylePaint"
                );
            }
            let bytes = run
                .string_property_bytes("text")
                .context("static text subset requires serialized TextValueRun.text")?;
            let text = std::str::from_utf8(bytes).context("TextValueRun text is not UTF-8")?;
            authored_text.push_str(text);
            runs.push(StaticTextRun {
                local_id: run_local,
                global_id: run_global,
            });
        }
        if runs.len() > 1 && !has_forced_text_break(&authored_text) {
            bail!(
                "static text subset only supports multiple TextValueRun children for authored line breaks"
            );
        }

        let container = graph
            .shape_paint_containers
            .iter()
            .find(|container| container.local_id == style_local)
            .context("TextStylePaint shape paint container is missing")?;
        if container.paints.len() != 1 {
            bail!(
                "static text subset requires one TextStylePaint paint, found {}",
                container.paints.len()
            );
        }
        let paint = &container.paints[0];
        if paint.paint_type != ShapePaintKind::Fill {
            bail!("static text subset only supports Fill text paints");
        }
        if !matches!(
            paint.paint_state,
            Some(ShapePaintStateNode::SolidColor { .. })
        ) {
            bail!("static text subset only supports solid text fill");
        }
        if paint.feather.is_some() || !paint.effects.is_empty() {
            bail!("static text subset does not support text paint effects");
        }

        let style = runtime
            .object(style_global as usize)
            .with_context(|| format!("missing TextStylePaint global {style_global}"))?;
        let font_asset_index = style
            .uint_property("fontAssetId")
            .context("TextStylePaint missing fontAssetId")?;
        let font_asset = runtime
            .file_asset(usize::try_from(font_asset_index).context("font asset id is too large")?)
            .context("TextStylePaint fontAssetId did not resolve to a file asset")?;
        if font_asset.type_name != "FontAsset" {
            bail!(
                "static text subset expected FontAsset, found {} global {}",
                font_asset.type_name,
                font_asset.id
            );
        }
        let font_bytes = embedded_file_asset_bytes(runtime, font_asset.id);
        let style_component = graph
            .components
            .iter()
            .find(|component| component.local_id == style_local)
            .context("TextStylePaint component is missing")?;
        let mut variations = Vec::new();
        for axis_local in style_component
            .children
            .iter()
            .copied()
            .filter(|local| child_type(*local) == Some("TextStyleAxis"))
        {
            let axis_global = global_for_local(graph, axis_local)?;
            let axis = runtime
                .object(axis_global as usize)
                .with_context(|| format!("missing TextStyleAxis global {axis_global}"))?;
            let tag = axis
                .uint_property("tag")
                .with_context(|| format!("TextStyleAxis global {axis_global} missing tag"))?
                as u32;
            let axis_value = axis.double_property("axisValue").unwrap_or(0.0);
            variations.push((tag, axis_value));
        }
        let modifiers = text_component
            .children
            .iter()
            .copied()
            .filter(|local| child_type(*local) == Some("TextModifierGroup"))
            .map(|group_local| StaticTextModifierGroup::from_graph(runtime, graph, group_local))
            .collect::<Result<Vec<_>>>()?;

        Ok(Self {
            text_local,
            text_global,
            runs,
            style_local,
            style_global,
            container,
            font_bytes,
            variations,
            modifiers,
        })
    }

    fn path_commands(
        &self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
    ) -> Result<Vec<RuntimePathCommand>> {
        let text = self.text_value(runtime, instance)?;
        let font_size = self.font_size(runtime, instance)?;
        if text.is_empty() || font_size <= 0.0 {
            return Ok(Vec::new());
        }
        let letter_spacing = self.letter_spacing(runtime, instance);
        let Some(font_bytes) = self.font_bytes else {
            // Mirrors src/importers/file_asset_importer.cpp: with no
            // FileAssetLoader and no in-band contents, a hosted FontAsset
            // resolves successfully but has no decoded font.
            return Ok(Vec::new());
        };

        let harf_font = HarfFontRef::new(font_bytes).context("failed to parse font for shaping")?;
        let harf_variations = self
            .variations
            .iter()
            .map(|(tag, value)| (HarfTag::from_u32(*tag), *value))
            .collect::<Vec<_>>();
        let shaper_instance = if harf_variations.is_empty() {
            None
        } else {
            Some(ShaperInstance::from_variations(
                &harf_font,
                harf_variations.iter().copied(),
            ))
        };
        let shaper_data = ShaperData::new(&harf_font);
        let shaper = shaper_data
            .shaper(&harf_font)
            .instance(shaper_instance.as_ref())
            .build();

        let skrifa_font =
            SkrifaFontRef::new(font_bytes).context("failed to parse font for outlines")?;
        let disable_legacy_kern = disable_legacy_kern_for_advances(&skrifa_font);
        let skrifa_variations = self
            .variations
            .iter()
            .map(|(tag, value)| VariationSetting::new(SkrifaTag::from_u32(*tag), *value))
            .collect::<Vec<_>>();
        let location = skrifa_font
            .axes()
            .location(skrifa_variations.iter().copied());
        let location_ref = LocationRef::from(&location);
        let (ascent, descent) = harfbuzz_line_metrics(&skrifa_font, location_ref);
        let baseline = ascent * font_size / TEXT_SHAPE_SCALE_F32;
        let line_height = (ascent - descent) * font_size / TEXT_SHAPE_SCALE_F32;
        let outlines = skrifa_font.outline_glyphs();
        let scale = font_size / TEXT_SHAPE_SCALE_F32;
        let apply_ellipsis = self.should_apply_static_ellipsis(runtime, instance)?;
        let lines = split_static_text_lines(&text);
        if apply_ellipsis && lines.len() > 1 {
            bail!("static text subset does not support ellipsis across multiple authored lines");
        }
        let mut commands = Vec::new();
        let modifier_coverages = self
            .modifiers
            .iter()
            .map(|modifier| modifier.coverage_by_character(runtime, instance, &text))
            .collect::<Result<Vec<_>>>()?;
        for line in lines {
            if line.text.is_empty() {
                continue;
            }
            let mut glyphs = shape_text_glyphs(&shaper, line.text, disable_legacy_kern);
            if apply_ellipsis {
                let max_width = self.text_width(runtime, instance)?;
                let ellipsis = shape_text_glyphs(&shaper, "...", disable_legacy_kern);
                let line_end = self.first_static_wrapped_line_end(
                    runtime,
                    instance,
                    line.text,
                    &glyphs,
                    max_width,
                    scale,
                    letter_spacing,
                )?;
                let mut force_ellipsis = false;
                if line_end < glyphs.len()
                    && self.static_fixed_height_shows_first_line_only(
                        runtime,
                        instance,
                        line_height,
                    )?
                {
                    glyphs.truncate(line_end);
                    force_ellipsis = true;
                }
                apply_static_ellipsis(
                    &mut glyphs,
                    ellipsis,
                    max_width,
                    scale,
                    letter_spacing,
                    force_ellipsis,
                );
            }

            let mut cursor_x = 0.0f32;
            let line_baseline = baseline + line.line_index as f32 * line_height;
            for glyph in glyphs {
                let char_index =
                    line.char_start + character_index_for_cluster(line.text, glyph.cluster);
                let mut offset_x = 0.0;
                let mut offset_y = 0.0;
                for (modifier, coverage) in self.modifiers.iter().zip(&modifier_coverages) {
                    let amount = coverage.get(char_index).copied().unwrap_or(0.0);
                    if amount == 0.0 {
                        continue;
                    }
                    let (x, y) = modifier.translation(runtime, instance)?;
                    offset_x += x * amount;
                    offset_y += y * amount;
                }
                let glyph_id = GlyphId::new(glyph.glyph_id);
                if let Some(outline) = outlines.get(glyph_id) {
                    let mut pen =
                        TextOutlinePen::new(cursor_x + offset_x, line_baseline + offset_y, scale);
                    let draw_settings =
                        DrawSettings::unhinted(Size::new(TEXT_SHAPE_SCALE_F32), location_ref)
                            .with_path_style(PathStyle::HarfBuzz);
                    outline
                        .draw(draw_settings, &mut pen)
                        .with_context(|| format!("failed to draw glyph {}", glyph.glyph_id))?;
                    commands.extend(pen.commands);
                }
                cursor_x += glyph.advance * scale + letter_spacing;
            }
        }

        Ok(commands)
    }

    fn local_bounds(
        &self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
    ) -> Result<Option<(f32, f32, f32, f32)>> {
        let text = self.text_value(runtime, instance)?;
        let font_size = self.font_size(runtime, instance)?;
        if text.is_empty() || font_size <= 0.0 {
            return Ok(Some((0.0, 0.0, 0.0, 0.0)));
        }
        let Some(font_bytes) = self.font_bytes else {
            return Ok(Some((0.0, 0.0, 0.0, 0.0)));
        };

        let harf_font = HarfFontRef::new(font_bytes).context("failed to parse font for shaping")?;
        let harf_variations = self
            .variations
            .iter()
            .map(|(tag, value)| (HarfTag::from_u32(*tag), *value))
            .collect::<Vec<_>>();
        let shaper_instance = if harf_variations.is_empty() {
            None
        } else {
            Some(ShaperInstance::from_variations(
                &harf_font,
                harf_variations.iter().copied(),
            ))
        };
        let shaper_data = ShaperData::new(&harf_font);
        let shaper = shaper_data
            .shaper(&harf_font)
            .instance(shaper_instance.as_ref())
            .build();

        let skrifa_font =
            SkrifaFontRef::new(font_bytes).context("failed to parse font for metrics")?;
        let disable_legacy_kern = disable_legacy_kern_for_advances(&skrifa_font);
        let skrifa_variations = self
            .variations
            .iter()
            .map(|(tag, value)| VariationSetting::new(SkrifaTag::from_u32(*tag), *value))
            .collect::<Vec<_>>();
        let location = skrifa_font
            .axes()
            .location(skrifa_variations.iter().copied());
        let location_ref = LocationRef::from(&location);
        let (ascent, descent) = harfbuzz_line_metrics(&skrifa_font, location_ref);
        let line_height = (ascent - descent) * font_size / TEXT_SHAPE_SCALE_F32;
        let scale = font_size / TEXT_SHAPE_SCALE_F32;
        let letter_spacing = self.letter_spacing(runtime, instance);
        let lines = split_static_text_lines(&text);
        let measured_width = lines
            .iter()
            .filter(|line| !line.text.is_empty())
            .map(|line| {
                let glyphs = shape_text_glyphs(&shaper, line.text, disable_legacy_kern);
                text_glyph_width(&glyphs, scale, letter_spacing)
            })
            .fold(0.0f32, f32::max);
        let sizing = self.text_uint_property(runtime, instance, "sizingValue")?;
        let width = match sizing {
            1 | 2 => self.text_width(runtime, instance)?,
            _ => measured_width,
        };
        let measured_height = line_height * lines.len().max(1) as f32;
        let height = match sizing {
            2 => self.text_height(runtime, instance)?,
            _ => measured_height,
        };
        let origin_x = self.text_double_property(runtime, instance, "originX", 0.0)?;
        let origin_y = self.text_double_property(runtime, instance, "originY", 0.0)?;

        Ok(Some((-width * origin_x, -height * origin_y, width, height)))
    }

    fn text_value(&self, runtime: &RuntimeFile, instance: &ArtboardInstance) -> Result<String> {
        let property_key = property_key_for_name("TextValueRun", "text")
            .context("missing TextValueRun.text key")?;
        let mut text = String::new();
        for run in &self.runs {
            let bytes = instance
                .string_property(run.local_id, property_key)
                .or_else(|| {
                    runtime
                        .object(run.global_id as usize)
                        .and_then(|object| object.string_property_bytes("text"))
                })
                .context("TextValueRun missing text")?;
            text.push_str(std::str::from_utf8(bytes).context("TextValueRun text is not UTF-8")?);
        }
        Ok(text)
    }

    fn font_size(&self, runtime: &RuntimeFile, instance: &ArtboardInstance) -> Result<f32> {
        let property_key = property_key_for_name("TextStyle", "fontSize")
            .context("missing TextStyle.fontSize key")?;
        Ok(instance
            .double_property(self.style_local, property_key)
            .or_else(|| {
                runtime
                    .object(self.style_global as usize)
                    .and_then(|object| object.double_property("fontSize"))
            })
            .unwrap_or(12.0))
    }

    fn letter_spacing(&self, runtime: &RuntimeFile, instance: &ArtboardInstance) -> f32 {
        let Some(property_key) = property_key_for_name("TextStyle", "letterSpacing") else {
            return 0.0;
        };
        instance
            .double_property(self.style_local, property_key)
            .or_else(|| {
                runtime
                    .object(self.style_global as usize)
                    .and_then(|object| object.double_property("letterSpacing"))
            })
            .unwrap_or(0.0)
    }

    fn text_width(&self, runtime: &RuntimeFile, instance: &ArtboardInstance) -> Result<f32> {
        let property_key =
            property_key_for_name("Text", "width").context("missing Text.width key")?;
        Ok(instance
            .double_property(self.text_local, property_key)
            .or_else(|| {
                runtime
                    .object(self.text_global as usize)
                    .and_then(|object| object.double_property("width"))
            })
            .unwrap_or(0.0))
    }

    fn text_height(&self, runtime: &RuntimeFile, instance: &ArtboardInstance) -> Result<f32> {
        let property_key =
            property_key_for_name("Text", "height").context("missing Text.height key")?;
        Ok(instance
            .double_property(self.text_local, property_key)
            .or_else(|| {
                runtime
                    .object(self.text_global as usize)
                    .and_then(|object| object.double_property("height"))
            })
            .unwrap_or(0.0))
    }

    fn text_double_property(
        &self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
        property_name: &str,
        default: f32,
    ) -> Result<f32> {
        let property_key = property_key_for_name("Text", property_name)
            .with_context(|| format!("missing Text.{property_name} key"))?;
        Ok(instance
            .double_property(self.text_local, property_key)
            .or_else(|| {
                runtime
                    .object(self.text_global as usize)
                    .and_then(|object| object.double_property(property_name))
            })
            .unwrap_or(default))
    }

    fn text_uint_property(
        &self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
        property_name: &str,
    ) -> Result<u64> {
        let property_key = property_key_for_name("Text", property_name)
            .with_context(|| format!("missing Text.{property_name} key"))?;
        Ok(instance
            .uint_property(self.text_local, property_key)
            .or_else(|| {
                runtime
                    .object(self.text_global as usize)
                    .and_then(|object| object.uint_property(property_name))
            })
            .unwrap_or(0))
    }

    fn should_apply_static_ellipsis(
        &self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
    ) -> Result<bool> {
        let sizing = self.text_uint_property(runtime, instance, "sizingValue")?;
        let overflow = self.text_uint_property(runtime, instance, "overflowValue")?;
        Ok(sizing == 2 && overflow == 3 && self.text_width(runtime, instance)? > 0.0)
    }

    fn first_static_wrapped_line_end(
        &self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
        text: &str,
        glyphs: &[TextGlyph],
        max_width: f32,
        scale: f32,
        letter_spacing: f32,
    ) -> Result<usize> {
        let wrap = self.text_uint_property(runtime, instance, "wrapValue")?;
        if wrap != 0 || glyphs.is_empty() || max_width <= 0.0 {
            return Ok(glyphs.len());
        }

        let mut line_end = 0;
        let mut saw_word = false;
        for (start, word) in text.split_word_bound_indices() {
            if word.chars().all(char::is_whitespace) {
                continue;
            }
            let word_end = start + word.len();
            let candidate_end = glyphs
                .iter()
                .rposition(|glyph| glyph.cluster < word_end as u32)
                .map(|index| index + 1)
                .unwrap_or(line_end);
            if candidate_end <= line_end {
                continue;
            }
            let width = text_glyph_width(&glyphs[..candidate_end], scale, letter_spacing);
            if width > max_width && saw_word {
                return Ok(line_end);
            }
            if width > max_width {
                return Ok(first_fitting_glyph_end(
                    glyphs,
                    max_width,
                    scale,
                    letter_spacing,
                ));
            }
            line_end = candidate_end;
            saw_word = true;
        }

        Ok(glyphs.len())
    }

    fn static_fixed_height_shows_first_line_only(
        &self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
        line_height: f32,
    ) -> Result<bool> {
        let height = self.text_height(runtime, instance)?;
        Ok(line_height > 0.0 && height > 0.0 && height < line_height * 2.0)
    }
}

// Mirrors the static coverage/translation subset from C++
// src/text/text_modifier_group.cpp and src/text/text_modifier_range.cpp.
impl StaticTextModifierGroup {
    fn from_graph(runtime: &RuntimeFile, graph: &ArtboardGraph, local_id: usize) -> Result<Self> {
        let global_id = global_for_local(graph, local_id)?;
        let object = runtime
            .object(global_id as usize)
            .with_context(|| format!("missing TextModifierGroup global {global_id}"))?;
        let flags = object.uint_property("modifierFlags").unwrap_or(0);
        const MODIFY_TRANSLATION: u64 = 1 << 2;
        if flags & !MODIFY_TRANSLATION != 0 {
            bail!(
                "static text subset only supports translation TextModifierGroup flags, found {flags}"
            );
        }

        let component = component_for_local(graph, local_id)
            .with_context(|| format!("TextModifierGroup local {local_id} component is missing"))?;
        let mut ranges = Vec::new();
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
        })
    }

    fn translation(
        &self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
    ) -> Result<(f32, f32)> {
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
        if flags & MODIFY_TRANSLATION == 0 {
            return Ok((0.0, 0.0));
        }
        Ok((
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
        ))
    }

    fn coverage_by_character(
        &self,
        runtime: &RuntimeFile,
        instance: &ArtboardInstance,
        text: &str,
    ) -> Result<Vec<f32>> {
        let mut coverage = vec![0.0; text.chars().count()];
        for range in &self.ranges {
            range.apply_coverage(runtime, instance, &mut coverage)?;
        }
        Ok(coverage)
    }
}

impl StaticTextModifierRange {
    fn from_graph(runtime: &RuntimeFile, graph: &ArtboardGraph, local_id: usize) -> Result<Self> {
        let global_id = global_for_local(graph, local_id)?;
        let object = runtime
            .object(global_id as usize)
            .with_context(|| format!("missing TextModifierRange global {global_id}"))?;
        let units = object.uint_property("unitsValue").unwrap_or(0);
        if units != 0 {
            bail!(
                "static text subset only supports character TextModifierRange units, found {units}"
            );
        }
        let run_id = object.uint_property("runId").unwrap_or(u32::MAX as u64);
        if run_id != u32::MAX as u64 {
            bail!("static text subset does not support TextModifierRange runId");
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
        coverage: &mut [f32],
    ) -> Result<()> {
        if coverage.is_empty() {
            return Ok(());
        }
        let unit_count = coverage.len() as f32;
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

        for (unit_index, current) in coverage.iter_mut().enumerate() {
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
            let next = match mode {
                0 => *current + c,
                1 => *current - c,
                2 => *current * c,
                3 => current.min(c),
                4 => current.max(c),
                5 => (*current - c).abs(),
                other => {
                    bail!("static text subset does not support TextModifierRange mode {other}")
                }
            };
            *current = if clamp { next.clamp(0.0, 1.0) } else { next };
        }

        Ok(())
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

fn runtime_double_property(
    runtime: &RuntimeFile,
    instance: &ArtboardInstance,
    type_name: &str,
    local_id: usize,
    global_id: u32,
    property_name: &str,
    default: f32,
) -> Result<f32> {
    let property_key = property_key_for_name(type_name, property_name)
        .with_context(|| format!("missing {type_name}.{property_name} key"))?;
    Ok(instance
        .double_property(local_id, property_key)
        .or_else(|| {
            runtime
                .object(global_id as usize)
                .and_then(|object| object.double_property(property_name))
        })
        .unwrap_or(default))
}

fn runtime_uint_property(
    runtime: &RuntimeFile,
    instance: &ArtboardInstance,
    type_name: &str,
    local_id: usize,
    global_id: u32,
    property_name: &str,
    default: u64,
) -> Result<u64> {
    let property_key = property_key_for_name(type_name, property_name)
        .with_context(|| format!("missing {type_name}.{property_name} key"))?;
    Ok(instance
        .uint_property(local_id, property_key)
        .or_else(|| {
            runtime
                .object(global_id as usize)
                .and_then(|object| object.uint_property(property_name))
        })
        .unwrap_or(default))
}

fn runtime_bool_property(
    runtime: &RuntimeFile,
    instance: &ArtboardInstance,
    type_name: &str,
    local_id: usize,
    global_id: u32,
    property_name: &str,
    default: bool,
) -> Result<bool> {
    let property_key = property_key_for_name(type_name, property_name)
        .with_context(|| format!("missing {type_name}.{property_name} key"))?;
    Ok(instance
        .bool_property(local_id, property_key)
        .or_else(|| {
            runtime
                .object(global_id as usize)
                .and_then(|object| object.bool_property(property_name))
        })
        .unwrap_or(default))
}

fn character_index_for_cluster(text: &str, cluster: u32) -> usize {
    let cluster = cluster as usize;
    text.char_indices()
        .take_while(|(byte_index, _)| *byte_index <= cluster)
        .count()
        .saturating_sub(1)
}

fn has_forced_text_break(text: &str) -> bool {
    text.chars()
        .any(|ch| matches!(ch, '\n' | '\r' | '\u{2028}'))
}

fn split_static_text_lines(text: &str) -> Vec<StaticTextLine<'_>> {
    let mut lines = Vec::new();
    let mut line_start_byte = 0;
    let mut line_start_char = 0;
    let mut line_index = 0;
    let mut iter = text.char_indices().peekable();

    while let Some((byte_index, ch)) = iter.next() {
        if !matches!(ch, '\n' | '\r' | '\u{2028}') {
            continue;
        }

        lines.push(StaticTextLine {
            text: &text[line_start_byte..byte_index],
            char_start: line_start_char,
            line_index,
        });

        let mut next_start_byte = byte_index + ch.len_utf8();
        let mut separator_chars = 1;
        if ch == '\r'
            && let Some((next_byte_index, '\n')) = iter.peek().copied()
        {
            iter.next();
            next_start_byte = next_byte_index + '\n'.len_utf8();
            separator_chars = 2;
        }

        line_start_char += text[line_start_byte..byte_index].chars().count() + separator_chars;
        line_start_byte = next_start_byte;
        line_index += 1;
    }

    lines.push(StaticTextLine {
        text: &text[line_start_byte..],
        char_start: line_start_char,
        line_index,
    });
    lines
}

fn cubic_interpolator_calc_bezier(t: f32, a1: f32, a2: f32) -> f32 {
    (((1.0 - 3.0 * a2 + 3.0 * a1) * t + (3.0 * a2 - 6.0 * a1)) * t + (3.0 * a1)) * t
}

fn cubic_interpolator_slope(t: f32, a1: f32, a2: f32) -> f32 {
    3.0 * (1.0 - 3.0 * a2 + 3.0 * a1) * t * t + 2.0 * (3.0 * a2 - 6.0 * a1) * t + (3.0 * a1)
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
            if current_x.abs() <= SUBDIVISION_PRECISION || iterations >= SUBDIVISION_MAX_ITERATIONS
            {
                return current_t;
            }
        }
    }
}

#[derive(Clone)]
struct TextGlyph {
    glyph_id: u32,
    cluster: u32,
    advance: f32,
}

fn harfbuzz_line_metrics(font: &SkrifaFontRef<'_>, location_ref: LocationRef<'_>) -> (f32, f32) {
    // Mirrors src/text/font_hb.cpp::make_lmx: HarfBuzz scales extents to
    // kStdScale and rounds them before Rive applies the authored font size.
    let metrics = font.metrics(Size::new(TEXT_SHAPE_SCALE_F32), location_ref);
    (metrics.ascent.round(), metrics.descent.round())
}

fn disable_legacy_kern_for_advances(font: &SkrifaFontRef<'_>) -> bool {
    font.kern().is_ok() && font.gpos().is_err()
}

fn shape_text_glyphs(
    shaper: &harfrust::Shaper<'_>,
    text: &str,
    disable_legacy_kern: bool,
) -> Vec<TextGlyph> {
    let mut buffer = UnicodeBuffer::new();
    buffer.push_str(text);
    buffer.set_direction(Direction::LeftToRight);
    buffer.guess_segment_properties();
    let kern_off = [Feature::new(HarfTag::new(b"kern"), 0, ..)];
    let shape_options = ShapeOptions::new().scale(Some(TEXT_SHAPE_SCALE));
    let shape_options = if disable_legacy_kern {
        shape_options.features(&kern_off)
    } else {
        shape_options
    };
    let glyphs = shaper.shape(buffer, shape_options);
    glyphs
        .glyph_infos()
        .iter()
        .zip(glyphs.glyph_positions())
        .map(|(info, position)| TextGlyph {
            glyph_id: info.glyph_id,
            cluster: info.cluster,
            advance: position.x_advance as f32,
        })
        .collect()
}

trait StaticTextWords {
    fn split_word_bound_indices(&self) -> Vec<(usize, &str)>;
}

impl StaticTextWords for str {
    fn split_word_bound_indices(&self) -> Vec<(usize, &str)> {
        let mut words = Vec::new();
        let mut start = None;
        for (index, ch) in self.char_indices() {
            if ch.is_whitespace() {
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

fn text_glyph_width(glyphs: &[TextGlyph], scale: f32, letter_spacing: f32) -> f32 {
    glyphs
        .iter()
        .map(|glyph| glyph.advance * scale + letter_spacing)
        .sum()
}

fn first_fitting_glyph_end(
    glyphs: &[TextGlyph],
    max_width: f32,
    scale: f32,
    letter_spacing: f32,
) -> usize {
    let mut width = 0.0;
    for (index, glyph) in glyphs.iter().enumerate() {
        let advance = glyph.advance * scale + letter_spacing;
        if width + advance > max_width {
            return index.max(1);
        }
        width += advance;
    }
    glyphs.len()
}

fn apply_static_ellipsis(
    glyphs: &mut Vec<TextGlyph>,
    ellipsis: Vec<TextGlyph>,
    max_width: f32,
    scale: f32,
    letter_spacing: f32,
    force: bool,
) {
    let advance_width = |glyph: &TextGlyph| glyph.advance * scale + letter_spacing;
    let ellipsis_width = ellipsis.iter().map(advance_width).sum::<f32>();
    let mut width = 0.0;
    let mut keep = glyphs.len();
    for (index, glyph) in glyphs.iter().enumerate() {
        let advance = advance_width(glyph);
        if width + advance + ellipsis_width > max_width {
            keep = index;
            break;
        }
        width += advance;
    }
    if keep < glyphs.len() {
        glyphs.truncate(keep);
        glyphs.extend(ellipsis);
    } else if force {
        glyphs.extend(ellipsis);
    }
}

struct TextOutlinePen {
    commands: Vec<RuntimePathCommand>,
    x: f32,
    y: f32,
    scale: f32,
    current: Option<(f32, f32)>,
    contour_start: Option<(f32, f32)>,
}

impl TextOutlinePen {
    fn new(x: f32, y: f32, scale: f32) -> Self {
        Self {
            commands: Vec::new(),
            x,
            y,
            scale,
            current: None,
            contour_start: None,
        }
    }

    fn map(&self, x: f32, y: f32) -> (f32, f32) {
        (self.x + x * self.scale, self.y - y * self.scale)
    }
}

impl OutlinePen for TextOutlinePen {
    fn move_to(&mut self, x: f32, y: f32) {
        let point = self.map(x, y);
        self.commands.push(RuntimePathCommand::Move {
            x: point.0,
            y: point.1,
        });
        self.current = Some(point);
        self.contour_start = Some(point);
    }

    fn line_to(&mut self, x: f32, y: f32) {
        let point = self.map(x, y);
        self.commands.push(RuntimePathCommand::Line {
            x: point.0,
            y: point.1,
        });
        self.current = Some(point);
    }

    fn quad_to(&mut self, cx0: f32, cy0: f32, x: f32, y: f32) {
        let Some(current) = self.current else {
            self.move_to(x, y);
            return;
        };
        let control = self.map(cx0, cy0);
        let end = self.map(x, y);
        let x1 = current.0 + (control.0 - current.0) * (2.0 / 3.0);
        let y1 = current.1 + (control.1 - current.1) * (2.0 / 3.0);
        let x2 = end.0 + (control.0 - end.0) * (2.0 / 3.0);
        let y2 = end.1 + (control.1 - end.1) * (2.0 / 3.0);
        self.commands.push(RuntimePathCommand::Cubic {
            x1,
            y1,
            x2,
            y2,
            x3: end.0,
            y3: end.1,
        });
        self.current = Some(end);
    }

    fn curve_to(&mut self, cx0: f32, cy0: f32, cx1: f32, cy1: f32, x: f32, y: f32) {
        let control0 = self.map(cx0, cy0);
        let control1 = self.map(cx1, cy1);
        let end = self.map(x, y);
        self.commands.push(RuntimePathCommand::Cubic {
            x1: control0.0,
            y1: control0.1,
            x2: control1.0,
            y2: control1.1,
            x3: end.0,
            y3: end.1,
        });
        self.current = Some(end);
    }

    fn close(&mut self) {
        if let (Some(current), Some(start)) = (self.current, self.contour_start)
            && ((current.0 - start.0).abs() > f32::EPSILON
                || (current.1 - start.1).abs() > f32::EPSILON)
        {
            self.commands.push(RuntimePathCommand::Line {
                x: start.0,
                y: start.1,
            });
        }
        self.commands.push(RuntimePathCommand::Close);
        self.current = self.contour_start;
    }
}

fn global_for_local(graph: &ArtboardGraph, local_id: usize) -> Result<u32> {
    graph
        .local_objects
        .iter()
        .find(|object| object.local_id == local_id)
        .map(|object| object.global_id)
        .with_context(|| format!("missing local object {local_id}"))
}

fn type_for_local(graph: &ArtboardGraph, local_id: usize) -> Option<&str> {
    graph
        .local_objects
        .iter()
        .find(|object| object.local_id == local_id)
        .and_then(|object| object.type_name)
}

fn component_for_local(
    graph: &ArtboardGraph,
    local_id: usize,
) -> Option<&rive_graph::ComponentNode> {
    graph
        .components
        .iter()
        .find(|component| component.local_id == local_id)
}

fn embedded_file_asset_bytes(runtime: &RuntimeFile, asset_global: u32) -> Option<&[u8]> {
    let file_asset_globals = runtime
        .file_assets()
        .into_iter()
        .map(|asset| asset.id)
        .collect::<BTreeSet<_>>();
    let mut after_asset = false;
    for object in runtime.objects.iter().flatten() {
        if object.id == asset_global {
            after_asset = true;
            continue;
        }
        if !after_asset {
            continue;
        }
        if file_asset_globals.contains(&object.id) {
            return None;
        }
        if object.type_name == "FileAssetContents" {
            return object.bytes_property("bytes");
        }
    }
    None
}
