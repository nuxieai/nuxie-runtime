//! ShapePaintPath owns RawPath plus one lazily materialized RenderPath. Initial
//! construction performs `addRawPath` before `fillRule`; later dirt performs
//! `rewind` then `addRawPath` without replaying construction-only fill rules.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Materialization {
    Create,
    Refresh,
    Reuse,
}

pub(crate) fn materialization(
    has_render_path: bool,
    raw_mutation_changed: bool,
) -> Materialization {
    if !has_render_path {
        Materialization::Create
    } else if raw_mutation_changed {
        Materialization::Refresh
    } else {
        Materialization::Reuse
    }
}
