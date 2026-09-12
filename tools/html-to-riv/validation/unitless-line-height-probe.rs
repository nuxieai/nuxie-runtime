//! Read-only Text metrics plus ordinary world/internal transforms for the
//! unitless-line-height experiment. Unchanged immutable linked runtime.
//! Usage: text-probe SCENE.riv NEW_OUTPUT WIDTHxHEIGHT [WIDTHxHEIGHT ...]
use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::{File, RuntimeFactoryHandle, source::text::text::Text};
use std::{error::Error, fs, path::PathBuf};
type Result<T> = std::result::Result<T, Box<dyn Error>>;
fn numbers(values: &[f32]) -> Result<String> {
    if values.iter().any(|v| !v.is_finite()) {
        return Err("non-finite observed text metric".into());
    }
    Ok(format!(
        "[{}]",
        values
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join(",")
    ))
}
fn observe(text: &Text, id: usize) -> Result<String> {
    let world = numbers(text.base.world_transform().values())?;
    let internal = numbers(text.internal_transform().values())?;
    let bounds = text.local_bounds();
    let bounds = numbers(&[bounds.min_x, bounds.min_y, bounds.width(), bounds.height()])?;
    let dimensions = numbers(&[
        text.effective_width(),
        text.effective_height(),
        text.computed_width(),
        text.computed_height(),
    ])?;
    let origin = numbers(&[text.base.origin_x(), text.base.origin_y()])?;
    let mut lines = Vec::new();
    for line in text.ordered_lines() {
        let g = line.glyph_line();
        lines.push(format!(
            "{{\"metrics\":{},\"startRun\":{},\"endRun\":{},\"startGlyph\":{},\"endGlyph\":{}}}",
            numbers(&[
                g.top,
                g.baseline,
                g.bottom,
                g.start_x,
                line.y(),
                line.bottom()
            ])?,
            g.start_run_index,
            g.end_run_index,
            g.start_glyph_index,
            g.end_glyph_index
        ));
    }
    let mut paragraphs = Vec::new();
    for paragraph in text.shape() {
        let mut runs = Vec::new();
        for run in &paragraph.runs {
            let font = match &run.font {
                Some(font) => {
                    let m = font.line_metrics();
                    numbers(&[m.ascent, m.descent, m.cap_height, m.x_height])?
                }
                None => "null".into(),
            };
            runs.push(format!("{{\"styleId\":{},\"sizeLineHeightSpacing\":{},\"fontMetrics\":{},\"glyphs\":{:?},\"textIndices\":{:?},\"advances\":{},\"xpos\":{}}}",
                run.style_id,numbers(&[run.size,run.line_height,run.letter_spacing])?,font,run.glyphs,run.text_indices,numbers(&run.advances)?,numbers(&run.xpos)?));
        }
        paragraphs.push(format!("[{}]", runs.join(",")));
    }
    Ok(format!(
        "{{\"objectId\":{id},\"worldMatrix\":{world},\"internalMatrix\":{internal},\"localBounds\":{bounds},\"dimensions\":{dimensions},\"originXY\":{origin},\"originValue\":{},\"lines\":[{}],\"paragraphRuns\":[{}]}}",
        text.base.origin_value(),
        lines.join(","),
        paragraphs.join(",")
    ))
}
fn main() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() < 3 {
        return Err(
            "usage: text-probe SCENE.riv NEW_OUTPUT WIDTHxHEIGHT [WIDTHxHEIGHT ...]".into(),
        );
    }
    let sizes = args[2..]
        .iter()
        .map(|arg| -> Result<[f32; 2]> {
            let (w, h) = arg
                .to_str()
                .ok_or("viewport must be UTF-8")?
                .split_once('x')
                .ok_or("viewport must be WIDTHxHEIGHT")?;
            let size = [w.parse::<f32>()?, h.parse::<f32>()?];
            if !size
                .iter()
                .all(|v| v.is_finite() && *v > 0. && *v <= 16384.)
            {
                return Err("viewport dimensions must be finite and in (0,16384]".into());
            }
            Ok(size)
        })
        .collect::<Result<Vec<_>>>()?;
    let output = PathBuf::from(&args[1]);
    if output.exists() {
        return Err("use a fresh output directory".into());
    }
    let bytes = fs::read(&args[0])?;
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let context = RuntimeFactoryHandle::from_factory(&mut factory)
        .ok_or("recording factory context unavailable")?;
    let file =
        File::import(&bytes, context, None, None, None).ok_or("ordinary RIV import failed")?;
    let original = file
        .with_file(File::artboard_default)
        .ok_or("default artboard missing")?;
    original.update_pass(true);
    let clone = original
        .instance()
        .ok_or("ordinary artboard clone failed")?;
    let mut frames = Vec::new();
    for (instance_index, instance) in [&original, &clone].into_iter().enumerate() {
        for (step, &[width, height]) in sizes.iter().enumerate() {
            instance.set_size(width, height);
            instance.update_pass(true);
            let objects = instance.with_artboard(|artboard| -> Result<Vec<String>> {
                let mut texts = Vec::new();
                for (id, object) in artboard.objects().iter().enumerate() {
                    if let Some(object) = object {
                        if let Some(observed) = object
                            .with(|o| {
                                o.as_any()
                                    .downcast_ref::<Text>()
                                    .map(|text| observe(text, id))
                            })
                            .flatten()
                        {
                            texts.push(observed?);
                        }
                    }
                }
                Ok(texts)
            })?;
            frames.push(format!(
                "{{\"instance\":\"{}\",\"step\":{step},\"viewport\":{},\"texts\":[{}]}}",
                if instance_index == 0 {
                    "original"
                } else {
                    "clone"
                },
                numbers(&[width, height])?,
                objects.join(",")
            ));
        }
    }
    let json = format!(
        "{{\"schema\":1,\"scope\":\"read-only immutable ordinary import/clone/resize text observation; no drawing\",\"arrayFields\":{{\"localBounds\":[\"x\",\"y\",\"width\",\"height\"],\"dimensions\":[\"effectiveWidth\",\"effectiveHeight\",\"computedWidth\",\"computedHeight\"],\"lineMetrics\":[\"top\",\"baseline\",\"bottom\",\"startX\",\"orderedY\",\"orderedBottom\"],\"fontMetrics\":[\"ascent\",\"descent\",\"capHeight\",\"xHeight\"],\"sizeLineHeightSpacing\":[\"fontSize\",\"lineHeight\",\"letterSpacing\"]}},\"frames\":[{}]}}",
        frames.join(",")
    );
    fs::create_dir(&output)?;
    fs::write(output.join("textmetrics.json"), json)?;
    Ok(())
}
