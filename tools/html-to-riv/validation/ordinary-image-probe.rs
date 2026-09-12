//! Read-only ordinary Image metrics from the immutable baseline. No draw or policy hooks.
//! Usage: ordinary-image-probe SCENE.riv NEW_OUTPUT WIDTHxHEIGHT [WIDTHxHEIGHT ...]
//! RecordingFactory observes encoded headers; it does not certify full image decoding.
use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::{
    File, RuntimeFactoryHandle,
    source::{
        assets::image_asset::ImageAsset, layout::layout_participant::LayoutParticipant,
        shapes::image::Image,
    },
};
use std::{error::Error, fs, path::PathBuf};
type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn numbers(values: &[f32]) -> Result<String> {
    if values.iter().any(|v| !v.is_finite()) {
        return Err("non-finite observed image metric".into());
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

fn observe(image: &Image, id: usize) -> Result<String> {
    let bounds = image.local_bounds();
    let dimensions = numbers(&[
        image.width(),
        image.height(),
        image.computed_width(),
        image.computed_height(),
    ])?;
    let local_bounds = numbers(&[bounds.min_x, bounds.min_y, bounds.width(), bounds.height()])?;
    let world = numbers(image.base.world_transform().values())?;
    let origin_alignment = numbers(&[
        image.base.origin_x(),
        image.base.origin_y(),
        image.base.alignment_x(),
        image.base.alignment_y(),
    ])?;
    let scales = numbers(&[
        image.base.scale_x(),
        image.base.scale_y(),
        image.render_scale_x(),
        image.render_scale_y(),
    ])?;
    let render = image.render_image().map_or_else(
        || "null".into(),
        |render| {
            format!(
                "{{\"identity\":\"{}\",\"headerWidth\":{},\"headerHeight\":{}}}",
                render.image_identity(),
                render.width(),
                render.height()
            )
        },
    );
    let asset = match image.image_asset() {
        Some(handle) => {
            let (arena, slot, generation) = handle.identity_key();
            let metadata = handle
                .with_downcast::<ImageAsset, _>(|asset| {
                    numbers(&[asset.base.width(), asset.base.height()])
                })
                .ok_or("Image asset handle lost ImageAsset identity")??;
            format!(
                "{{\"identity\":\"{arena}:{slot}:{generation}\",\"authoredDimensions\":{metadata}}}"
            )
        }
        None => "null".into(),
    };
    let participant = match image.layout_participant() {
        Some(handle) => {
            let (arena, slot, generation) = handle.identity_key();
            let rect = handle
                .with_downcast::<LayoutParticipant, _>(|participant| {
                    numbers(&[
                        participant.resolved_left(),
                        participant.resolved_top(),
                        participant.resolved_width(),
                        participant.resolved_height(),
                    ])
                })
                .ok_or("Image participant handle lost LayoutParticipant identity")??;
            format!("{{\"identity\":\"{arena}:{slot}:{generation}\",\"resolvedRect\":{rect}}}")
        }
        None => "null".into(),
    };
    Ok(format!(
        "{{\"objectId\":{id},\"assetIndex\":{},\"fit\":{},\"dimensions\":{dimensions},\"localBounds\":{local_bounds},\"worldMatrix\":{world},\"originAlignment\":{origin_alignment},\"scales\":{scales},\"recordedImage\":{render},\"asset\":{asset},\"participant\":{participant}}}",
        image.asset_id(),
        image.base.fit()
    ))
}

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).collect();
    if args.len() < 3 {
        return Err(
            "usage: ordinary-image-probe SCENE.riv NEW_OUTPUT WIDTHxHEIGHT [WIDTHxHEIGHT ...]"
                .into(),
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
            let observations = instance.with_artboard(|artboard| -> Result<Vec<String>> {
                let mut images = Vec::new();
                for (id, object) in artboard.objects().iter().enumerate() {
                    if let Some(object) = object {
                        if let Some(observed) = object
                            .with(|o| {
                                o.as_any()
                                    .downcast_ref::<Image>()
                                    .map(|image| observe(image, id))
                            })
                            .flatten()
                        {
                            images.push(observed?);
                        }
                    }
                }
                Ok(images)
            })?;
            frames.push(format!(
                "{{\"instance\":\"{}\",\"step\":{step},\"viewport\":{},\"images\":[{}]}}",
                if instance_index == 0 {
                    "original"
                } else {
                    "clone"
                },
                numbers(&[width, height])?,
                observations.join(",")
            ));
        }
    }
    let json = format!(
        "{{\"schema\":1,\"scope\":\"read-only immutable ordinary import/clone/resize image observation; recording header inspection only; no full decode or drawing\",\"arrayFields\":{{\"dimensions\":[\"intrinsicWidth\",\"intrinsicHeight\",\"computedWidth\",\"computedHeight\"],\"localBounds\":[\"x\",\"y\",\"width\",\"height\"],\"resolvedRect\":[\"left\",\"top\",\"width\",\"height\"],\"originAlignment\":[\"originX\",\"originY\",\"alignmentX\",\"alignmentY\"],\"scales\":[\"authoredX\",\"authoredY\",\"renderX\",\"renderY\"]}},\"identityScope\":\"process-local occurrence identities, strings for lossless JSON; compare only within this run\",\"frames\":[{}]}}",
        frames.join(",")
    );
    fs::create_dir(&output)?;
    fs::write(output.join("scene.riv"), bytes)?;
    fs::write(output.join("image-metrics.json"), json)?;
    fs::write(output.join("import.stream"), factory.borrow().stream())?;
    Ok(())
}
