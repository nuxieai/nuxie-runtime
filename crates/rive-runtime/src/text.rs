use anyhow::{Context, Result, bail};
use harfrust::{Direction, FontRef as HarfFontRef, ShapeOptions, ShaperData, UnicodeBuffer};
use rive_binary::RuntimeFile;
use rive_graph::{ArtboardGraph, ShapePaintContainerNode, ShapePaintKind, ShapePaintStateNode};
use skrifa::outline::pen::PathStyle;
use skrifa::outline::{DrawSettings, OutlinePen};
use skrifa::raw::TableProvider;
use skrifa::{FontRef as SkrifaFontRef, GlyphId, MetadataProvider};
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
    let render_opacity = instance
        .component(text_local)
        .map(|component| component.transform.render_opacity)
        .unwrap_or(1.0);
    let shape_world = instance
        .component(text_local)
        .map(|component| component.transform.world_transform)
        .unwrap_or(Mat2D::IDENTITY);
    let needs_save_operation = command.needs_save_operation || slice.container.paints.len() > 1;

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
    run_local: usize,
    run_global: u32,
    style_local: usize,
    style_global: u32,
    container: &'a ShapePaintContainerNode,
    font_bytes: &'a [u8],
}

impl<'a> StaticTextSlice<'a> {
    fn from_graph(
        runtime: &'a RuntimeFile,
        graph: &'a ArtboardGraph,
        text_local: usize,
    ) -> Result<Self> {
        let text_count = graph
            .local_objects
            .iter()
            .filter(|object| object.type_name == Some("Text"))
            .count();
        if text_count != 1 {
            bail!("static text subset requires exactly one Text object, found {text_count}");
        }
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
                        | "SolidColor"
                        | "Fill"
                        | "Backboard"
                        | "LinearAnimation"
                        | "StateMachine"
                        | "StateMachineLayer"
                        | "AnimationState"
                        | "AnyState"
                        | "EntryState"
                        | "ExitState"
                        | "StateTransition"
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
                        | "TextStyleAxis"
                        | "TextStyleFeature"
                        | "TextModifierRange"
                        | "TextModifierGroup"
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
        if run_locals.len() != 1 {
            bail!(
                "static text subset requires exactly one TextValueRun child, found {}",
                run_locals.len()
            );
        }
        if style_locals.len() != 1 {
            bail!(
                "static text subset requires exactly one TextStylePaint child, found {}",
                style_locals.len()
            );
        }
        let run_local = run_locals[0];
        let style_local = style_locals[0];
        let run_global = global_for_local(graph, run_local)?;
        let style_global = global_for_local(graph, style_local)?;
        let run = runtime
            .object(run_global as usize)
            .with_context(|| format!("missing TextValueRun global {run_global}"))?;
        if run.uint_property("styleId") != Some(style_local as u64) {
            bail!("static text subset requires the run to reference the only TextStylePaint");
        }
        if run.string_property_bytes("text").is_none() {
            bail!("static text subset requires serialized TextValueRun.text");
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
        let font_bytes = embedded_file_asset_bytes(runtime, font_asset.id)
            .context("static text subset requires embedded FontAsset bytes")?;

        Ok(Self {
            run_local,
            run_global,
            style_local,
            style_global,
            container,
            font_bytes,
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

        let harf_font =
            HarfFontRef::new(self.font_bytes).context("failed to parse font for shaping")?;
        let shaper_data = ShaperData::new(&harf_font);
        let shaper = shaper_data.shaper(&harf_font).build();
        let mut buffer = UnicodeBuffer::new();
        buffer.push_str(&text);
        buffer.set_direction(Direction::LeftToRight);
        buffer.guess_segment_properties();
        let glyphs = shaper.shape(buffer, ShapeOptions::new().scale(Some(TEXT_SHAPE_SCALE)));

        let skrifa_font =
            SkrifaFontRef::new(self.font_bytes).context("failed to parse font for outlines")?;
        let upem = skrifa_font
            .head()
            .context("font missing head table")?
            .units_per_em()
            .max(1) as f32;
        let ascent = skrifa_font
            .hhea()
            .context("font missing hhea table")?
            .ascender()
            .to_i16() as f32
            * (TEXT_SHAPE_SCALE_F32 / upem);
        let baseline = ascent * font_size / TEXT_SHAPE_SCALE_F32;
        let outlines = skrifa_font.outline_glyphs();
        let scale = font_size / TEXT_SHAPE_SCALE_F32;
        let mut commands = Vec::new();
        let mut cursor_x = 0.0f32;
        for (info, position) in glyphs.glyph_infos().iter().zip(glyphs.glyph_positions()) {
            let offset_x = 0.0;
            let offset_y = 0.0;
            let glyph_id = GlyphId::new(info.glyph_id);
            let mut advance = position.x_advance as f32;
            if let Some(outline) = outlines.get(glyph_id) {
                let mut pen = TextOutlinePen::new(cursor_x + offset_x, baseline + offset_y, scale);
                let draw_settings = DrawSettings::unhinted(
                    skrifa::instance::Size::new(TEXT_SHAPE_SCALE_F32),
                    skrifa::instance::LocationRef::default(),
                )
                .with_path_style(PathStyle::HarfBuzz);
                let metrics = outline
                    .draw(draw_settings, &mut pen)
                    .with_context(|| format!("failed to draw glyph {}", info.glyph_id))?;
                if let Some(width) = metrics.advance_width {
                    advance = width;
                }
                commands.extend(pen.commands);
            }
            cursor_x += advance * scale + letter_spacing;
        }

        Ok(commands)
    }

    fn text_value(&self, runtime: &RuntimeFile, instance: &ArtboardInstance) -> Result<String> {
        let property_key = property_key_for_name("TextValueRun", "text")
            .context("missing TextValueRun.text key")?;
        let bytes = instance
            .string_property(self.run_local, property_key)
            .or_else(|| {
                runtime
                    .object(self.run_global as usize)
                    .and_then(|object| object.string_property_bytes("text"))
            })
            .context("TextValueRun missing text")?;
        String::from_utf8(bytes.to_vec()).context("TextValueRun text is not UTF-8")
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
