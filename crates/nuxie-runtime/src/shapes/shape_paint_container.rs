//! Direct polymorphic owner for pinned `src/shapes/shape_paint_container.cpp`.

use nuxie_graph::{ArtboardGraph, ShapePaintContainerNode};

#[cfg(test)]
thread_local! {
    static OPACITY_OWNER_RESOLUTION_COUNT: std::cell::Cell<usize> = const {
        std::cell::Cell::new(0)
    };
}

pub(crate) const PATH_FLAG_LOCAL: u64 = 1 << 1;
pub(crate) const PATH_FLAG_WORLD: u64 = 1 << 2;
pub(crate) const PATH_FLAG_LOCAL_CLOCKWISE: u64 = 1 << 6;

/// Closed dispatch set of `ShapePaintContainer::from(Component*)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RuntimeShapePaintContainerFamily {
    Artboard,
    LayoutComponent,
    Shape,
    TextStylePaint,
    ForegroundLayoutDrawable,
    TextInputCursor,
    TextInputSelection,
    TextInputText,
    TextInputSelectedText,
}

impl RuntimeShapePaintContainerFamily {
    pub(crate) fn from_type_name(type_name: &str) -> Option<Self> {
        Some(match type_name {
            "Artboard" => Self::Artboard,
            "LayoutComponent" => Self::LayoutComponent,
            "Shape" => Self::Shape,
            "TextStylePaint" => Self::TextStylePaint,
            "ForegroundLayoutDrawable" => Self::ForegroundLayoutDrawable,
            "TextInputCursor" => Self::TextInputCursor,
            "TextInputSelection" => Self::TextInputSelection,
            "TextInputText" => Self::TextInputText,
            "TextInputSelectedText" => Self::TextInputSelectedText,
            _ => return None,
        })
    }

    /// Shape and Artboard own the retained `ShapePaintPath` geometry slots.
    /// Every family still owns the common ordered paint list below; the other
    /// branches retain their geometry in their layout/text/text-input owners.
    pub(crate) fn owns_shape_geometry(self) -> bool {
        matches!(self, Self::Artboard | Self::Shape)
    }

    /// Text and text-input containers keep geometry in their concrete text
    /// owners, but their child ShapePaints still retain the common C++
    /// RenderPaint member.
    pub(crate) fn owns_text_paint(self) -> bool {
        matches!(
            self,
            Self::TextStylePaint
                | Self::TextInputCursor
                | Self::TextInputSelection
                | Self::TextInputText
                | Self::TextInputSelectedText
        )
    }
}

pub(crate) fn family(type_name: &str) -> Option<RuntimeShapePaintContainerFamily> {
    RuntimeShapePaintContainerFamily::from_type_name(type_name)
}

pub(crate) fn runtime_shape_paint_container_is_occurrence_owned(
    container: &ShapePaintContainerNode,
) -> bool {
    crate::shapes::shape_paint_container::family(container.type_name)
        .is_some_and(|family| family.owns_shape_geometry() || family.owns_text_paint())
}

/// Direct owner for C++ `ShapePaintContainer::addPaint`.
pub(crate) fn add_paint<T>(
    _family: RuntimeShapePaintContainerFamily,
    paints: &mut Vec<T>,
    paint: T,
) {
    paints.push(paint);
}

/// Direct owner for C++ `ShapePaintContainer::pathFlags`.
pub(crate) fn path_flags<T>(
    _family: RuntimeShapePaintContainerFamily,
    container_flags: u64,
    paints: &[T],
    mut paint_flags: impl FnMut(&T) -> u64,
) -> u64 {
    paints
        .iter()
        .fold(container_flags, |flags, paint| flags | paint_flags(paint))
}

pub(crate) fn path_kind_flag(kind: crate::draw::RuntimeShapePaintPathKind) -> u64 {
    match kind {
        crate::draw::RuntimeShapePaintPathKind::Local => PATH_FLAG_LOCAL,
        crate::draw::RuntimeShapePaintPathKind::World => PATH_FLAG_WORLD,
        crate::draw::RuntimeShapePaintPathKind::LocalClockwise => PATH_FLAG_LOCAL_CLOCKWISE,
    }
}

/// Direct owner for C++ `ShapePaintContainer::invalidateStrokeEffects`.
pub(crate) fn invalidate_stroke_effects<T>(
    _family: RuntimeShapePaintContainerFamily,
    paints: &[T],
    mut invalidate: impl FnMut(&T),
) {
    for paint in paints {
        invalidate(paint);
    }
}

/// Direct owner for C++ `ShapePaintContainer::propagateOpacity`.
pub(crate) fn propagate_opacity<'a, T>(
    _family: RuntimeShapePaintContainerFamily,
    paints: &'a [T],
    opacity: f32,
    mut set_opacity: impl FnMut(usize, &'a T, f32),
) {
    for (index, paint) in paints.iter().enumerate() {
        set_opacity(index, paint, opacity);
    }
}

/// Find the concrete Transform/Artboard opacity owner used by
/// `ShapePaintContainer::propagateOpacity`. Non-transform mixin branches such
/// as TextStylePaint walk to their retained parent exactly once here.
pub(crate) fn opacity_owner_local(graph: &ArtboardGraph, container_local: usize) -> usize {
    #[cfg(test)]
    OPACITY_OWNER_RESOLUTION_COUNT.with(|count| count.set(count.get() + 1));
    let mut current = Some(container_local);
    while let Some(local_id) = current {
        let Some(component) = graph
            .components
            .iter()
            .find(|component| component.local_id == local_id)
        else {
            break;
        };
        if component.capabilities.transform || component.capabilities.artboard {
            return local_id;
        }
        current = component.parent_local;
    }
    container_local
}

#[cfg(test)]
pub(crate) fn reset_opacity_owner_resolution_count() {
    OPACITY_OWNER_RESOLUTION_COUNT.with(|count| count.set(0));
}

#[cfg(test)]
pub(crate) fn opacity_owner_resolution_count() -> usize {
    OPACITY_OWNER_RESOLUTION_COUNT.with(std::cell::Cell::get)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_component_dispatch_matches_the_closed_cpp_switch() {
        for type_name in [
            "Artboard",
            "LayoutComponent",
            "Shape",
            "TextStylePaint",
            "ForegroundLayoutDrawable",
            "TextInputCursor",
            "TextInputSelection",
            "TextInputText",
            "TextInputSelectedText",
        ] {
            assert!(family(type_name).is_some(), "missing {type_name}");
        }
        assert_eq!(family("TextInput"), None);
        assert_eq!(family("ShapePaint"), None);
    }

    #[test]
    fn container_operations_preserve_authored_order_and_cpp_flag_union() {
        for type_name in [
            "Artboard",
            "LayoutComponent",
            "Shape",
            "TextStylePaint",
            "ForegroundLayoutDrawable",
            "TextInputCursor",
            "TextInputSelection",
            "TextInputText",
            "TextInputSelectedText",
        ] {
            let family = family(type_name).expect("closed switch family");
            let mut paints = Vec::new();
            add_paint(family, &mut paints, (1_u8, 0b001_u64));
            add_paint(family, &mut paints, (2_u8, 0b100_u64));
            assert_eq!(
                path_flags(family, 0b010, &paints, |paint| paint.1),
                0b111,
                "{type_name}"
            );

            let invalidated = std::cell::RefCell::new(Vec::new());
            invalidate_stroke_effects(family, &paints, |paint| {
                invalidated.borrow_mut().push(paint.0)
            });
            assert_eq!(*invalidated.borrow(), vec![1, 2], "{type_name}");

            let propagated = std::cell::RefCell::new(Vec::new());
            propagate_opacity(family, &paints, 0.25, |_, paint, opacity| {
                propagated.borrow_mut().push((paint.0, opacity));
            });
            assert_eq!(
                *propagated.borrow(),
                vec![(1, 0.25), (2, 0.25)],
                "{type_name}"
            );
        }
    }
}
