use crate::{RuntimePathCommand, artboard::ArtboardInstance, text_style_owner};

/// Occurrence-build state for the custom paths retained by C++
/// `TextStylePaint`. The aggregate path preserves glyph insertion order for
/// effects and inner feather; opacity buckets preserve the exact float key
/// partition consumed by `draw`.
#[derive(Debug, Clone, Default)]
pub(crate) struct RuntimeTextStylePaintPaths {
    has_contents: bool,
    aggregate: Vec<RuntimePathCommand>,
    buckets: Vec<RuntimeTextStylePaintPathBucket>,
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeTextStylePaintPathBucket {
    pub(crate) opacity: f32,
    pub(crate) commands: Vec<RuntimePathCommand>,
}

impl RuntimeTextStylePaintPaths {
    /// Direct owner for `TextStylePaint::addPath` apart from the separately
    /// owned `ShapePaintPath::addPathClockwise` winding normalization.
    pub(crate) fn add_path(
        &mut self,
        commands: Vec<RuntimePathCommand>,
        opacity: f32,
    ) -> bool {
        let had_contents = self.has_contents;
        self.has_contents = true;
        // Pinned C++ spells this as `opacity > 0.0f`. Keep the positive test:
        // unlike `opacity <= 0.0`, it rejects NaN as well as zero/negative.
        if opacity > 0.0 {
            self.aggregate.extend(commands.iter().copied());
            if let Some(bucket) = self
                .buckets
                .iter_mut()
                .find(|bucket| bucket.opacity == opacity)
            {
                bucket.commands.extend(commands);
            } else {
                self.buckets.push(RuntimeTextStylePaintPathBucket {
                    opacity,
                    commands,
                });
            }
        }
        !had_contents
    }

    pub(crate) fn aggregate(&self) -> &[RuntimePathCommand] {
        &self.aggregate
    }

    /// Pinned `localPath()` and `localClockwisePath()` both return `m_path`.
    /// Expose the two virtual views without duplicating retained geometry.
    pub(crate) fn local_path(&self) -> &[RuntimePathCommand] {
        &self.aggregate
    }

    pub(crate) fn local_clockwise_path(&self) -> &[RuntimePathCommand] {
        &self.aggregate
    }

    pub(crate) fn ordered_buckets(&self) -> Vec<RuntimeTextStylePaintPathBucket> {
        let mut buckets = self.buckets.clone();
        // NaN cannot enter through `add_path`, so every retained key has the
        // same total ascending order as pinned `std::map<float, ...>`.
        buckets.sort_by(|left, right| {
            left.opacity
                .partial_cmp(&right.opacity)
                .expect("TextStylePaint opacity keys exclude NaN")
        });
        buckets
    }

    pub(crate) fn is_empty(&self) -> bool {
        !self.has_contents
    }

    #[cfg(test)]
    fn rewind_path(&mut self) {
        self.aggregate.clear();
        self.has_contents = false;
        self.buckets.clear();
    }
}

/// `TextStylePaint` inherits TextStyle's shaping metrics but retains its own
/// paint/backend identity. Paint-only callbacks stay with the shape-paint
/// owner; this file owns the inherited shaping callback.
pub(crate) fn double_property_changed(
    instance: &mut ArtboardInstance,
    local_id: usize,
    type_name: Option<&str>,
    property_key: u16,
) -> Option<bool> {
    (type_name == Some("TextStylePaint"))
        .then(|| text_style_owner::metric_property_changed(instance, local_id, property_key))
        .flatten()
}

#[cfg(test)]
mod tests {
    use super::RuntimeTextStylePaintPaths;
    use crate::RuntimePathCommand;

    fn path(x: f32) -> Vec<RuntimePathCommand> {
        vec![RuntimePathCommand::Move { x, y: 0.0 }]
    }

    #[test]
    fn add_path_retains_first_call_and_exact_positive_float_buckets() {
        let mut paths = RuntimeTextStylePaintPaths::default();

        assert!(paths.add_path(path(0.0), f32::NAN));
        assert!(!paths.is_empty());
        assert!(!paths.add_path(path(1.0), -0.0));
        assert!(!paths.add_path(path(2.0), -1.0));
        assert!(!paths.add_path(path(3.0), f32::INFINITY));
        assert!(!paths.add_path(path(4.0), 0.5));
        assert!(!paths.add_path(path(5.0), 0.5));

        assert_eq!(paths.aggregate(), &[path(3.0), path(4.0), path(5.0)].concat());
        let buckets = paths.ordered_buckets();
        assert_eq!(
            buckets.iter().map(|bucket| bucket.opacity).collect::<Vec<_>>(),
            vec![0.5, f32::INFINITY]
        );
        assert_eq!(buckets[0].commands, [path(4.0), path(5.0)].concat());
        assert_eq!(buckets[1].commands, path(3.0));
        assert!(std::ptr::eq(
            paths.local_path(),
            paths.local_clockwise_path()
        ));

        paths.rewind_path();
        assert!(paths.is_empty());
        assert!(paths.aggregate().is_empty());
        assert!(paths.ordered_buckets().is_empty());
        assert!(paths.add_path(path(6.0), 1.0));
    }
}
