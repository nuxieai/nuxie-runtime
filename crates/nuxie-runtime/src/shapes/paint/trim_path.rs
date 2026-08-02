use nuxie_graph::{ShapePaintKind, ShapePaintNode, StrokeEffectNode};

use crate::{
    ArtboardInstance,
    draw::{RuntimePathCommand, TrimContour, runtime_draw_property_key_for_name},
    properties::property_key_for_name,
};

fn runtime_trim_path_mode_value(
    artboard: &ArtboardInstance,
    effect: &StrokeEffectNode,
) -> Option<u32> {
    runtime_draw_property_key_for_name("TrimPath", "modeValue")
        .and_then(|key| artboard.uint_property(effect.local_id, key))
        .and_then(|value| u32::try_from(value).ok())
        .or(effect.trim_mode_value)
}

fn runtime_trim_path_double_property(
    artboard: &ArtboardInstance,
    effect: &StrokeEffectNode,
    property_name: &str,
    fallback: Option<f32>,
) -> f32 {
    runtime_draw_property_key_for_name("TrimPath", property_name)
        .and_then(|key| artboard.double_property(effect.local_id, key))
        .or(fallback)
        .unwrap_or(0.0)
}

// Coarsely translated from:
// /Users/levi/dev/oss/rive-runtime/src/shapes/paint/trim_path.cpp TrimPath::trimPath
pub(crate) fn runtime_trim_path_line_effect_commands(
    artboard: &ArtboardInstance,
    effect: &StrokeEffectNode,
    paint: &ShapePaintNode,
    source: &[RuntimePathCommand],
) -> Option<Vec<RuntimePathCommand>> {
    let mode = runtime_trim_path_mode_value(artboard, effect)?;
    if !matches!(mode, 1 | 2) {
        return None;
    }
    let contours = TrimContour::from_commands(source);
    if contours.is_empty() {
        return Some(Vec::new());
    }

    let render_offset = positive_unit_mod(runtime_trim_path_double_property(
        artboard,
        effect,
        "offset",
        effect.trim_offset,
    ));
    let trim_start =
        runtime_trim_path_double_property(artboard, effect, "start", effect.trim_start);
    let trim_end = runtime_trim_path_double_property(artboard, effect, "end", effect.trim_end);
    let close_shape = paint.paint_type == ShapePaintKind::Fill;
    match mode {
        1 => Some(trim_path_sequential(
            &contours,
            trim_start,
            trim_end,
            render_offset,
            close_shape,
        )),
        2 => Some(trim_path_synchronized(
            &contours,
            trim_start,
            trim_end,
            render_offset,
            close_shape,
        )),
        _ => None,
    }
}

pub(crate) fn positive_unit_mod(value: f32) -> f32 {
    // Match C++ `fmodf(fmodf(value, 1) + 1, 1)` exactly. The intermediate
    // addition is observable at f32 precision even for already-positive
    // values and feeds directly into trim distances.
    ((value % 1.0) + 1.0) % 1.0
}

fn trim_path_sequential(
    contours: &[TrimContour],
    trim_start: f32,
    trim_end: f32,
    render_offset: f32,
    close_shape: bool,
) -> Vec<RuntimePathCommand> {
    let total_length = contours.iter().map(|contour| contour.length).sum::<f32>();
    if total_length == 0.0 {
        return Vec::new();
    }

    let mut start_length = total_length * (trim_start + render_offset);
    let mut end_length = total_length * (trim_end + render_offset);
    if end_length < start_length {
        std::mem::swap(&mut start_length, &mut end_length);
    }
    if start_length > total_length {
        start_length -= total_length;
        end_length -= total_length;
    }

    let mut indices = Vec::new();
    let mut lengths = Vec::new();
    let mut contour_index = 0usize;
    while end_length > 0.0 {
        let current_index = contour_index % contours.len();
        let contour = &contours[current_index];
        if start_length < contour.length {
            indices.push(current_index);
            lengths.push(start_length);
            lengths.push(end_length);
            end_length -= contour.length;
            start_length = 0.0;
        } else {
            start_length -= contour.length;
            end_length -= contour.length;
        }
        contour_index += 1;
    }

    let mut commands = Vec::new();
    let mut starting_index = 0isize;
    let mut index_count = 0usize;
    let mut previous_contour_index = None::<usize>;
    while index_count < indices.len() {
        let index = starting_index.rem_euclid(indices.len() as isize) as usize;
        let contour_index = indices[index];
        let contour = &contours[contour_index];
        let length_index = index * 2;
        let start_length = lengths[length_index];
        let end_length = lengths[length_index + 1];
        contour.get_segment(
            start_length,
            end_length,
            &mut commands,
            previous_contour_index != Some(contour_index) || !contour.is_closed,
        );
        if (start_length == 0.0 && end_length - start_length >= contour.length && contour.is_closed)
            || close_shape
        {
            commands.push(RuntimePathCommand::Close);
        }
        previous_contour_index = Some(contour_index);
        index_count += 1;
        starting_index -= 1;
    }
    commands
}

fn trim_path_synchronized(
    contours: &[TrimContour],
    trim_start: f32,
    trim_end: f32,
    render_offset: f32,
    close_shape: bool,
) -> Vec<RuntimePathCommand> {
    let mut commands = Vec::new();
    for contour in contours {
        let mut start_length = contour.length * (trim_start + render_offset);
        let mut end_length = contour.length * (trim_end + render_offset);
        if end_length < start_length {
            std::mem::swap(&mut start_length, &mut end_length);
        }

        if start_length >= contour.length {
            start_length -= contour.length;
            end_length -= contour.length;
        }
        contour.get_segment(start_length, end_length, &mut commands, true);
        while end_length > contour.length {
            start_length = 0.0;
            end_length -= contour.length;
            contour.get_segment(start_length, end_length, &mut commands, !contour.is_closed);
        }

        if (trim_start == 0.0 && trim_end == 1.0 && contour.is_closed) || close_shape {
            commands.push(RuntimePathCommand::Close);
        }
    }
    commands
}

pub(crate) fn double_property_changed(
    artboard: &mut ArtboardInstance,
    local_id: usize,
    property_key: u16,
) -> Option<bool> {
    if !["start", "end", "offset"]
        .into_iter()
        .any(|name| property_key_for_name("TrimPath", name) == Some(property_key))
    {
        return None;
    }
    Some(super::stroke_effect::invalidate_effect_from_local(
        artboard, local_id,
    ))
}

pub(crate) fn uint_property_changed(
    artboard: &mut ArtboardInstance,
    local_id: usize,
    property_key: u16,
) -> Option<bool> {
    if property_key_for_name("TrimPath", "modeValue") != Some(property_key) {
        return None;
    }
    Some(super::stroke_effect::invalidate_effect_from_local(
        artboard, local_id,
    ))
}
