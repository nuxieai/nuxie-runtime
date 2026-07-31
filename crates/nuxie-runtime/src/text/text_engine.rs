pub(crate) fn static_text_constraint_bounds(
    _runtime: &RuntimeFile,
    _graph: &ArtboardGraph,
    instance: &ArtboardInstance,
    text_local: usize,
) -> Option<(f32, f32, f32, f32)> {
    instance
        .component(text_local)
        .and_then(|component| component.concrete.text.as_ref())
        .and_then(|text| text.bounds())
}
/// Construct the bounds retained by `Text::buildRenderStyles` during the
/// mutable component update. Ordinary bounds readers use
/// `static_text_constraint_bounds` and never repeat this shaping work.
pub(crate) fn build_static_text_constraint_bounds(
    runtime: &RuntimeFile,
    graph: &ArtboardGraph,
    instance: &ArtboardInstance,
    text_local: usize,
    layout_constraint: Option<RuntimeTextLayoutConstraint>,
) -> Option<(f32, f32, f32, f32)> {
    if let Ok(slice) = StaticTextSlice::from_graph(runtime, graph, text_local)
        && let Ok(Some(bounds)) = match layout_constraint {
            Some(constraint) => {
                slice.local_bounds_with_layout_constraint(runtime, instance, constraint)
            }
            None => slice.local_bounds(runtime, instance),
        }
    {
        return Some(bounds);
    }
    static_fixed_text_constraint_bounds(runtime, graph, instance, text_local, None)
}
pub(crate) fn static_text_layout_measure_bounds(
    runtime: &RuntimeFile,
    graph: &ArtboardGraph,
    instance: &ArtboardInstance,
    text_local: usize,
    layout_constraint: RuntimeTextLayoutConstraint,
) -> Option<(f32, f32, f32, f32)> {
    if let Ok(slice) = StaticTextSlice::from_graph(runtime, graph, text_local)
        && let Ok(Some(bounds)) =
            slice.measure_bounds_with_layout_constraint(runtime, instance, layout_constraint)
    {
        return Some(bounds);
    }
    static_fixed_text_constraint_bounds(runtime, graph, instance, text_local, None).map(
        |(_x, _y, width, height)| {
            (
                0.0,
                0.0,
                width.min(layout_constraint.width),
                height.min(layout_constraint.height),
            )
        },
    )
}
pub(crate) fn static_text_controlled_layout_bounds(
    runtime: &RuntimeFile,
    graph: &ArtboardGraph,
    instance: &ArtboardInstance,
    text_local: usize,
    _layout_constraint: RuntimeTextLayoutConstraint,
) -> Option<(f32, f32, f32, f32)> {
    static_text_constraint_bounds(runtime, graph, instance, text_local)
}

fn byte_index_for_glyph_end(text: &str, glyphs: &[TextGlyph], glyph_end: usize) -> usize {
    if glyph_end >= glyphs.len() {
        return text.len();
    }
    let target = (glyphs[glyph_end].cluster as usize).min(text.len());
    if text.is_char_boundary(target) {
        return target;
    }
    text.char_indices()
        .map(|(index, _)| index)
        .take_while(|index| *index <= target)
        .last()
        .unwrap_or(0)
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
    glyphs: &mut Vec<StyledTextGlyph>,
    ellipsis: Vec<StyledTextGlyph>,
    max_width: f32,
    force: bool,
) {
    // Exact `OrderedLine::buildEllipsisRuns`: the final visual line first
    // measures authored advances without reserving ellipsis room
    // (`src/text/text_engine.cpp:165-302`).
    if !force {
        let mut authored_width = 0.0f32;
        let mut fits = true;
        for glyph in glyphs.iter() {
            authored_width += glyph.advance;
            if authored_width > max_width {
                fits = false;
                break;
            }
        }
        if fits {
            return;
        }
    }

    let ellipsis_width = ellipsis.iter().map(|glyph| glyph.advance).sum::<f32>();
    let mut width = 0.0;
    let mut keep = glyphs.len();
    for (index, glyph) in glyphs.iter().enumerate() {
        if width + glyph.advance + ellipsis_width > max_width {
            keep = index;
            break;
        }
        width += glyph.advance;
    }
    if keep < glyphs.len() {
        glyphs.truncate(keep);
        glyphs.extend(ellipsis);
    } else if force {
        glyphs.extend(ellipsis);
    }
}
