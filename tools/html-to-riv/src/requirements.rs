//! Portable capability requirements accompany the Rive bytes through publish.
use crate::Diagnostic;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RuntimeCapability {
    #[serde(rename = "layout-css-linear-gradient-v1")]
    LayoutCssLinearGradientV1,
    /// Requires isolated subtree compositing, including stacking isolation,
    /// undithered intermediate surfaces and checked renderer capacity.
    #[serde(rename = "layout-css-group-opacity-v1")]
    LayoutCssGroupOpacityV1,
    #[serde(rename = "layout-css-corner-radii-v1")]
    LayoutCssCornerRadiiV1,
    #[serde(rename = "layout-css-border-sides-v1")]
    LayoutCssBorderSidesV1,
    #[serde(rename = "layout-css-solid-borders-v1")]
    LayoutCssSolidBordersV1,
    #[serde(rename = "layout-css-overflow-clip-margin-v1")]
    LayoutCssOverflowClipMarginV1,
    #[serde(rename = "layout-css-axis-overflow-v1")]
    LayoutCssAxisOverflowV1,
    #[serde(rename = "layout-css-stacking-v1")]
    LayoutCssStackingV1,
    #[serde(rename = "layout-css-absolute-position-v1")]
    LayoutCssAbsolutePositionV1,
    #[serde(rename = "layout-css-positioned-paint-v1")]
    LayoutCssPositionedPaintV1,
    #[serde(rename = "layout-css-aspect-ratio-pair-v1")]
    LayoutCssAspectRatioPairV1,
    #[serde(rename = "layout-css-aspect-ratio-v1")]
    LayoutCssAspectRatioV1,
    #[serde(rename = "layout-css-content-box-v1")]
    LayoutCssContentBoxV1,
    #[serde(rename = "layout-css-indefinite-basis-v1")]
    LayoutCssIndefiniteBasisV1,
    #[serde(rename = "layout-css-intrinsic-sizing-v1")]
    LayoutCssIntrinsicSizingV1,
    #[serde(rename = "layout-css-partial-flex-factors-v1")]
    LayoutCssPartialFlexFactorsV1,
    #[serde(rename = "layout-css-flex-factors-v1")]
    LayoutCssFlexFactorsV1,
    #[serde(rename = "layout-css-distributed-spacing-v1")]
    LayoutCssDistributedSpacingV1,
    #[serde(rename = "layout-css-align-self-v1")]
    LayoutCssAlignSelfV1,
    #[serde(rename = "layout-css-align-content-v1")]
    LayoutCssAlignContentV1,
    /// Artboard-wide ordering of compiler-emitted layout paint groups.
    #[serde(rename = "layout-css-paint-order-v1")]
    LayoutCssPaintOrderV1,
    #[serde(rename = "layout-css-pixel-bounds-v1")]
    LayoutCssPixelBoundsV1,
    #[serde(rename = "layout-css-percentage-spacing-v1")]
    LayoutCssPercentageSpacingV1,
    #[serde(rename = "text-css-single-line-ellipsis-v1")]
    TextCssSingleLineEllipsisV1,
    #[serde(rename = "text-solid-strikethroughs-v1")]
    TextSolidStrikethroughsV1,
    #[serde(rename = "text-css-shaping-precision-v1")]
    TextCssShapingPrecisionV1,
    #[serde(rename = "text-solid-underlines-v1")]
    TextSolidUnderlinesV1,
    #[serde(rename = "text-css-wrapped-tabs-v1")]
    TextCssWrappedTabsV1,
    #[serde(rename = "text-css-pre-wrap-v1")]
    TextCssPreWrapV1,
    #[serde(rename = "text-css-pre-line-v1")]
    TextCssPreLineV1,
    #[serde(rename = "text-css-normal-wrap-v1")]
    TextCssNormalWrapV1,
    #[serde(rename = "text-css-tabs-v1")]
    TextCssTabsV1,
    #[serde(rename = "text-css-nowrap-alignment-v1")]
    TextCssNowrapAlignmentV1,
    #[serde(rename = "text-cluster-spacing-v1")]
    TextClusterSpacingV1,
    #[serde(rename = "text-css-letter-spacing-v1")]
    TextCssLetterSpacingV1,
    #[serde(rename = "text-preserved-space-breaks-v1")]
    TextPreservedSpaceBreaksV1,
}

/// Ratio semantics are occurrence-local. Version 15 retains the exact computed
/// pair in addition to the legacy scalar stored in Rive.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LayoutAspectRatioRequirement {
    pub object_id: u32,
    pub content_box: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pair: Option<[u32; 2]>,
}

/// An occurrence policy is separate from font-wide shaping capabilities.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TextPolicy {
    #[serde(rename = "css-single-line-ellipsis-v1")]
    CssSingleLineEllipsisV1,
    #[serde(rename = "css-pre-wrap-v1")]
    CssPreWrapV1,
    #[serde(rename = "css-pre-line-v1")]
    CssPreLineV1,
    #[serde(rename = "css-normal-wrap-v1")]
    CssNormalWrapV1,
    #[serde(rename = "css-nowrap-alignment-v1")]
    CssNowrapAlignmentV1,
}
impl TextPolicy {
    fn capability(self) -> RuntimeCapability {
        match self {
            Self::CssSingleLineEllipsisV1 => RuntimeCapability::TextCssSingleLineEllipsisV1,
            Self::CssPreWrapV1 => RuntimeCapability::TextCssPreWrapV1,
            Self::CssPreLineV1 => RuntimeCapability::TextCssPreLineV1,
            Self::CssNormalWrapV1 => RuntimeCapability::TextCssNormalWrapV1,
            Self::CssNowrapAlignmentV1 => RuntimeCapability::TextCssNowrapAlignmentV1,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextPolicyRequirement {
    /// Artboard-local Text object index in this compilation's default artboard.
    pub object_id: u32,
    pub policy: TextPolicy,
}

/// Resolved values in text-local CSS pixels, ordered from ancestor to descendant.
/// CSS cascade and font metric resolution happen in the compiler, not the host.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SolidUnderline {
    pub color: u32,
    pub thickness: f32,
    pub offset: f32,
    pub skip_ink: UnderlineSkipInk,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum UnderlineSkipInk {
    None,
    Auto,
    All,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextUnderlineRequirement {
    /// Existing Text object in the default artboard; not a source layout ID.
    pub object_id: u32,
    pub lines: Vec<SolidUnderline>,
}

/// Solid strikethrough metrics resolved at the CSS decoration origin.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SolidStrikethrough {
    pub color: u32,
    pub thickness: f32,
    /// Top edge relative to the line baseline, in text-local CSS pixels.
    pub offset: f32,
    /// Resolved baseline distance from the CSS line-box top.
    pub line_baseline: f32,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextStrikethroughRequirement {
    pub object_id: u32,
    pub lines: Vec<SolidStrikethrough>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AlignSelf {
    #[default]
    Auto,
    FlexStart,
    Center,
    FlexEnd,
    Stretch,
    Baseline,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LayoutAlignSelfRequirement {
    pub object_id: u32,
    pub alignment: AlignSelf,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AlignContent {
    FlexStart,
    Center,
    FlexEnd,
    #[default]
    Stretch,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LayoutAlignContentRequirement {
    pub object_id: u32,
    pub alignment: AlignContent,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum JustifyDistribution { SpaceAround, SpaceEvenly }
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LayoutJustifyContentRequirement {
    pub object_id: u32,
    pub alignment: JustifyDistribution,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LayoutFlexFactorsRequirement {
    pub object_id: u32,
    pub grow: f32,
    pub shrink: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LayoutStackingRequirement {
    pub object_id: u32,
    pub level: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OverflowAxis { X, Y }

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LayoutAxisOverflowRequirement {
    pub object_id: u32,
    pub axis: OverflowAxis,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OverflowClipBox { ContentBox, PaddingBox, BorderBox }

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LayoutOverflowClipMarginRequirement {
    pub object_id: u32,
    pub origin: OverflowClipBox,
    pub pixels: f32,
}

/// Packed ARGB color; used physical border widths are retained in the Rive
/// layout style. Hosts install both border-aware geometry and ring painting.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LayoutBorderRequirement {
    pub object_id: u32,
    pub color: u32,
}

/// Packed ARGB colors in physical top/right/bottom/left order. Used widths
/// remain responsive layout-style properties in the Rive file.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LayoutBorderSidesRequirement {
    pub object_id: u32,
    pub colors: [u32; 4],
}

/// Computed axis value; percentage points remain unresolved until runtime.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CornerRadiusValue { Pixels(f32), Percent(f32) }

impl CornerRadiusValue {
    fn is_valid(self) -> bool {
        let (Self::Pixels(value) | Self::Percent(value)) = self;
        value.is_finite() && value >= 0.
    }
}

/// Physical TL/TR/BR/BL pairs, each horizontal then vertical. Percentages use
/// the live border-box width/height, with shared proportional overlap reduction.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LayoutCornerRadiiRequirement {
    pub object_id: u32,
    pub radii: [[CornerRadiusValue; 2]; 4],
}

/// CSS direction remains layout-relative for corner directions.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum LinearGradientDirection {
    Degrees(f32),
    Corner { right: bool, bottom: bool },
}

/// Signed positions are retained before CSS stop fixup at each layout size.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LinearGradientPosition { Pixels(f32), Percent(f32) }

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LinearGradientStop {
    /// Unpremultiplied ARGB; interpolation is premultiplied sRGB.
    pub color: u32,
    pub position: Option<LinearGradientPosition>,
}

/// One responsive CSS background gradient on a LayoutComponent occurrence.
/// Stops are authored order, including omitted, coincident and exterior stops.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LayoutLinearGradientRequirement {
    pub object_id: u32,
    pub direction: LinearGradientDirection,
    pub stops: Vec<LinearGradientStop>,
}

impl LayoutLinearGradientRequirement {
    fn is_valid(&self) -> bool {
        let direction_valid = match self.direction {
            LinearGradientDirection::Degrees(value) => value.is_finite(),
            LinearGradientDirection::Corner { .. } => true,
        };
        direction_valid && (2..=256).contains(&self.stops.len())
            && self.stops.iter().all(|stop| match stop.position {
                None => true,
                Some(LinearGradientPosition::Pixels(v) | LinearGradientPosition::Percent(v)) => v.is_finite(),
            })
    }
}

/// Normalized subtree alpha. Opaque occurrences are omitted: opacity one does
/// not create a stacking context. Hosts install this on LayoutComponent owners
/// and must provide isolated compositing before drawing any scene content.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LayoutGroupOpacityRequirement {
    pub object_id: u32,
    pub opacity: f32,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeRequirements {
    pub version: u32,
    pub capabilities: BTreeSet<RuntimeCapability>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub text_policies: Vec<TextPolicyRequirement>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub text_underlines: Vec<TextUnderlineRequirement>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub text_strikethroughs: Vec<TextStrikethroughRequirement>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub layout_pixel_bounds: Vec<u32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub layout_percentage_spacing: Vec<u32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub layout_positioned: Vec<u32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub layout_absolute: Vec<u32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub layout_stacking: Vec<LayoutStackingRequirement>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub layout_axis_overflow: Vec<LayoutAxisOverflowRequirement>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub layout_overflow_clip_margins: Vec<LayoutOverflowClipMarginRequirement>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub layout_borders: Vec<LayoutBorderRequirement>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub layout_border_sides: Vec<LayoutBorderSidesRequirement>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub layout_corner_radii: Vec<LayoutCornerRadiiRequirement>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub layout_group_opacity: Vec<LayoutGroupOpacityRequirement>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub layout_linear_gradients: Vec<LayoutLinearGradientRequirement>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub layout_align_self: Vec<LayoutAlignSelfRequirement>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub layout_align_content: Vec<LayoutAlignContentRequirement>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub layout_justify_content: Vec<LayoutJustifyContentRequirement>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub layout_flex_factors: Vec<LayoutFlexFactorsRequirement>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub layout_intrinsic_sizing: Vec<u32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub layout_content_box: Vec<u32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub layout_aspect_ratios: Vec<LayoutAspectRatioRequirement>,
}
impl Default for RuntimeRequirements {
    fn default() -> Self {
        Self {
            version: 1,
            capabilities: BTreeSet::new(),
            text_policies: Vec::new(),
            text_underlines: Vec::new(),
            text_strikethroughs: Vec::new(),
            layout_pixel_bounds: Vec::new(),
            layout_percentage_spacing: Vec::new(),
            layout_positioned: Vec::new(),
            layout_absolute: Vec::new(),
            layout_stacking: Vec::new(),
            layout_axis_overflow: Vec::new(),
            layout_overflow_clip_margins: Vec::new(),
            layout_borders: Vec::new(),
            layout_border_sides: Vec::new(),
            layout_corner_radii: Vec::new(),
            layout_group_opacity: Vec::new(),
            layout_linear_gradients: Vec::new(),
            layout_align_self: Vec::new(),
            layout_align_content: Vec::new(),
            layout_justify_content: Vec::new(),
            layout_flex_factors: Vec::new(),
            layout_intrinsic_sizing: Vec::new(),
            layout_content_box: Vec::new(),
            layout_aspect_ratios: Vec::new(),
        }
    }
}
impl RuntimeRequirements {
    /// Check before loading the scene; accepting a capability also requires the
    /// host to install its behavior and retain it through asset replacement.
    pub fn ensure_supported(&self, supported: &[RuntimeCapability]) -> Result<(), Diagnostic> {
        self.validate_text_policies()?;
        if self
            .capabilities
            .iter()
            .any(|capability| !supported.contains(capability))
        {
            return Err(Diagnostic::new(
                "missing-runtime-capability",
                "runtimeRequirements.capabilities",
                "The scene requires a runtime capability not provided by this host",
            ));
        }
        Ok(())
    }
}

impl RuntimeRequirements {
    fn validate_text_policies(&self) -> Result<(), Diagnostic> {
        if !matches!(self.version, 1..=26) {
            return Err(Diagnostic::new(
                "unsupported-runtime-requirements",
                "runtimeRequirements.version",
                "Expected runtime requirements version 1 through 26",
            ));
        }
        let has_gradients = !self.layout_linear_gradients.is_empty();
        let mut gradient_targets = BTreeSet::new();
        if (self.version == 26) != has_gradients
            || self.capabilities.contains(&RuntimeCapability::LayoutCssLinearGradientV1) != has_gradients
            || self.layout_linear_gradients.iter().any(|entry| entry.object_id == 0
                || !gradient_targets.insert(entry.object_id) || !entry.is_valid()) {
            return Err(Diagnostic::new("invalid-layout-linear-gradient", "runtimeRequirements.layout_linear_gradients",
                "Version 26 requires unique non-root gradient targets, 2–256 finite authored stops and the matching premultiplied gradient capability"));
        }
        let has_opacity = !self.layout_group_opacity.is_empty();
        let mut opacity_targets = BTreeSet::new();
        if (self.version < 26 && (self.version == 25) != has_opacity)
            || self.capabilities.contains(&RuntimeCapability::LayoutCssGroupOpacityV1) != has_opacity
            || self.layout_group_opacity.iter().any(|entry| entry.object_id == 0
                || !opacity_targets.insert(entry.object_id)
                || !entry.opacity.is_finite() || !(0.0..1.0).contains(&entry.opacity)) {
            return Err(Diagnostic::new("invalid-layout-group-opacity", "runtimeRequirements.layout_group_opacity",
                "Version 25 requires unique non-root group targets, finite alpha in [0, 1) and the matching isolated-compositing capability"));
        }
        let has_corners = !self.layout_corner_radii.is_empty();
        let mut corner_targets = BTreeSet::new();
        if (self.version < 25 && (self.version == 24) != has_corners)
            || self.capabilities.contains(&RuntimeCapability::LayoutCssCornerRadiiV1) != has_corners
            || self.layout_corner_radii.iter().any(|entry| entry.object_id == 0
                || !corner_targets.insert(entry.object_id)
                || entry.radii.iter().flatten().any(|value| !value.is_valid())) {
            return Err(Diagnostic::new("invalid-layout-corner-radii", "runtimeRequirements.layout_corner_radii",
                "Version 24 requires unique non-root corner targets, finite nonnegative axis values and the matching capability"));
        }
        let has_borders = !self.layout_borders.is_empty();
        let mut border_targets = BTreeSet::new();
        if (self.version < 23 && (self.version == 22) != has_borders)
            || self.capabilities.contains(&RuntimeCapability::LayoutCssSolidBordersV1) != has_borders
            || self.layout_borders.iter().any(|entry| entry.object_id == 0
                || !border_targets.insert(entry.object_id)) {
            return Err(Diagnostic::new("invalid-layout-borders", "runtimeRequirements.layout_borders",
                "Version 22 requires unique non-root border targets and the matching capability"));
        }
        let has_border_sides = !self.layout_border_sides.is_empty();
        if (self.version < 24 && (self.version == 23) != has_border_sides)
            || self.capabilities.contains(&RuntimeCapability::LayoutCssBorderSidesV1) != has_border_sides
            || self.layout_border_sides.iter().any(|entry| entry.object_id == 0
                || !border_targets.insert(entry.object_id)) {
            return Err(Diagnostic::new("invalid-layout-border-sides", "runtimeRequirements.layout_border_sides",
                "Version 23 requires unique non-root side targets, disjoint from uniform borders, and the matching capability"));
        }
        let has_clip_margins = !self.layout_overflow_clip_margins.is_empty();
        let mut margin_targets = BTreeSet::new();
        if (self.version < 22 && (self.version == 21) != has_clip_margins)
            || self.capabilities.contains(&RuntimeCapability::LayoutCssOverflowClipMarginV1) != has_clip_margins
            || self.layout_overflow_clip_margins.iter().any(|entry| entry.object_id == 0
                || !entry.pixels.is_finite()
                || !margin_targets.insert(entry.object_id)) {
            return Err(Diagnostic::new("invalid-layout-overflow-clip-margin", "runtimeRequirements.layout_overflow_clip_margins",
                "Version 21 requires unique non-root clip-margin targets, finite signed offsets and the matching capability"));
        }
        let has_axis_overflow = !self.layout_axis_overflow.is_empty();
        let mut axis_targets = BTreeSet::new();
        if (self.version < 21 && (self.version == 20) != has_axis_overflow)
            || self.capabilities.contains(&RuntimeCapability::LayoutCssAxisOverflowV1) != has_axis_overflow
            || self.layout_axis_overflow.iter().any(|entry| entry.object_id == 0 || margin_targets.contains(&entry.object_id) || !axis_targets.insert(entry.object_id)) {
            return Err(Diagnostic::new("invalid-layout-axis-overflow", "runtimeRequirements.layout_axis_overflow",
                "Version 20 requires unique non-root axis overflow targets and the matching capability"));
        }
        let has_stacking = !self.layout_stacking.is_empty();
        let mut stacking_targets = BTreeSet::new();
        if (self.version < 20 && (self.version == 19) != has_stacking)
            || self.capabilities.contains(&RuntimeCapability::LayoutCssStackingV1) != has_stacking
            || self.layout_stacking.iter().any(|entry| entry.object_id == 0 || !stacking_targets.insert(entry.object_id)) {
            return Err(Diagnostic::new("invalid-layout-stacking", "runtimeRequirements.layout_stacking",
                "Version 19 requires unique non-root stacking targets and the matching capability"));
        }
        let has_positioned = !self.layout_positioned.is_empty();
        let mut positioned_targets = BTreeSet::new();
        if (self.version < 19 && (self.version >= 17) != has_positioned)
            || self.capabilities.contains(&RuntimeCapability::LayoutCssPositionedPaintV1) != has_positioned
            || self.layout_positioned.iter().any(|id| *id == 0 || !positioned_targets.insert(*id)) {
            return Err(Diagnostic::new("invalid-layout-positioned-paint", "runtimeRequirements.layout_positioned",
                "Version 17 requires unique non-root positioned layout targets and the matching capability"));
        }
        let has_absolute = !self.layout_absolute.is_empty();
        let mut absolute_targets = BTreeSet::new();
        if (self.version < 19 && (self.version == 18) != has_absolute)
            || self.capabilities.contains(&RuntimeCapability::LayoutCssAbsolutePositionV1) != has_absolute
            || self.layout_absolute.iter().any(|id| !positioned_targets.contains(id) || !absolute_targets.insert(*id)) {
            return Err(Diagnostic::new("invalid-layout-absolute-position", "runtimeRequirements.layout_absolute",
                "Version 18 requires unique absolute targets within positioned targets and the matching capability"));
        }
        let has_spacing = !self.layout_percentage_spacing.is_empty();
        let mut spacing_targets = BTreeSet::new();
        if (self.version < 17 && (self.version == 16) != has_spacing)
            || self.capabilities.contains(&RuntimeCapability::LayoutCssPercentageSpacingV1) != has_spacing
            || self.layout_percentage_spacing.iter().any(|id| !spacing_targets.insert(*id)) {
            return Err(Diagnostic::new("invalid-layout-percentage-spacing", "runtimeRequirements.layout_percentage_spacing",
                "Version 16 requires unique percentage-spacing targets and the matching capability"));
        }
        let has_ratios = !self.layout_aspect_ratios.is_empty();
        let mut ratio_targets = BTreeSet::new();
        if (self.version < 16 && (self.version >= 14) != has_ratios)
            || self.capabilities.contains(&RuntimeCapability::LayoutCssAspectRatioV1) != has_ratios
            || self.layout_aspect_ratios.iter().any(|entry| !ratio_targets.insert(entry.object_id)) {
            return Err(Diagnostic::new("invalid-layout-aspect-ratio", "runtimeRequirements.layout_aspect_ratios",
                "Version 14 requires unique ratio targets and the matching capability"));
        }
        let has_pairs = self.layout_aspect_ratios.iter().any(|entry| entry.pair.is_some());
        if (self.version < 16 && (self.version == 15) != has_pairs)
            || self.capabilities.contains(&RuntimeCapability::LayoutCssAspectRatioPairV1) != has_pairs
            || (has_pairs && self.layout_aspect_ratios.iter().any(|entry| match entry.pair {
                Some(pair) => pair.iter().any(|v| *v == 0 || *v > i32::MAX as u32),
                None => true,
            })) {
            return Err(Diagnostic::new("invalid-layout-aspect-ratio-pair", "runtimeRequirements.layout_aspect_ratios",
                "Version 15 requires a positive signed-32-bit pair for every ratio target and the matching capability"));
        }
        let has_content_box = !self.layout_content_box.is_empty();
        let mut content_box_targets = BTreeSet::new();
        if (self.version < 14 && (self.version == 13) != has_content_box)
            || self.capabilities.contains(&RuntimeCapability::LayoutCssContentBoxV1) != has_content_box
            || self.layout_content_box.iter().any(|id| !content_box_targets.insert(*id)) {
            return Err(Diagnostic::new("invalid-layout-content-box", "runtimeRequirements.layout_content_box",
                "Version 13 requires unique content-box targets and the matching capability"));
        }
        let has_indefinite = self.capabilities.contains(&RuntimeCapability::LayoutCssIndefiniteBasisV1);
        if self.version < 13 && (self.version == 12) != has_indefinite {
            return Err(Diagnostic::new("invalid-layout-indefinite-basis", "runtimeRequirements.capabilities",
                "Version 12 requires the matching indefinite-basis capability"));
        }
        let has_intrinsic = !self.layout_intrinsic_sizing.is_empty();
        let mut intrinsic_targets = BTreeSet::new();
        if (self.version < 12 && (self.version == 11) != has_intrinsic)
            || self.capabilities.contains(&RuntimeCapability::LayoutCssIntrinsicSizingV1) != has_intrinsic
            || self.layout_intrinsic_sizing.iter().any(|id| !intrinsic_targets.insert(*id)) {
            return Err(Diagnostic::new("invalid-layout-intrinsic-sizing", "runtimeRequirements.layout_intrinsic_sizing",
                "Version 11 requires unique intrinsic-sizing targets and the matching capability"));
        }
        let has_factors = !self.layout_flex_factors.is_empty();
        let mut factor_targets = BTreeSet::new();
        let has_partial = self.layout_flex_factors.iter().any(|entry|
            [entry.grow, entry.shrink].iter().any(|v| *v > 0.0 && *v < 1.0));
        if (self.version < 11 && (self.version == 10) != has_partial)
            || self.capabilities.contains(&RuntimeCapability::LayoutCssPartialFlexFactorsV1) != has_partial {
            return Err(Diagnostic::new("invalid-layout-partial-flex-factors", "runtimeRequirements.layout_flex_factors",
                "Version 10 requires partial factors and the matching corrected-distribution capability"));
        }
        if (self.version < 11 && (self.version >= 9) != has_factors)
            || self.capabilities.contains(&RuntimeCapability::LayoutCssFlexFactorsV1) != has_factors
            || self.layout_flex_factors.iter().any(|entry|
                !factor_targets.insert(entry.object_id)
                || !entry.grow.is_finite() || entry.grow < 0.0
                || !entry.shrink.is_finite() || entry.shrink < 0.0)
        {
            return Err(Diagnostic::new("invalid-layout-flex-factors", "runtimeRequirements.layout_flex_factors",
                "Versions 9 through 12 require finite nonnegative flex factors, unique targets and the matching capability"));
        }
        let has_distribution = !self.layout_justify_content.is_empty()
            || self.layout_align_content.iter().any(|entry| entry.alignment == AlignContent::SpaceEvenly);
        let mut distributed_targets = BTreeSet::new();
        if (self.version < 9 && (self.version == 8) != has_distribution)
            || self.capabilities.contains(&RuntimeCapability::LayoutCssDistributedSpacingV1) != has_distribution
            || self.layout_justify_content.iter().any(|entry| !distributed_targets.insert(entry.object_id))
        {
            return Err(Diagnostic::new("invalid-layout-distributed-spacing", "runtimeRequirements.layout_justify_content",
                "Version 8 requires distributed spacing, unique justification targets and the matching capability"));
        }
        let has_alignment = !self.layout_align_content.is_empty();
        let mut aligned_targets = BTreeSet::new();
        if (self.version < 8 && (self.version == 7) != has_alignment)
            || self.capabilities.contains(&RuntimeCapability::LayoutCssAlignContentV1) != has_alignment
            || self.layout_align_content.iter().any(|entry| !aligned_targets.insert(entry.object_id))
        {
            return Err(Diagnostic::new("invalid-layout-align-content", "runtimeRequirements.layout_align_content",
                "Version 7 requires unique alignment targets and the matching CSS align-content capability"));
        }
        let has_alignment = !self.layout_align_self.is_empty();
        let mut aligned_targets = BTreeSet::new();
        if (self.version < 7 && (self.version == 6) != has_alignment)
            || self.capabilities.contains(&RuntimeCapability::LayoutCssAlignSelfV1) != has_alignment
            || self.layout_align_self.iter().any(|entry| !aligned_targets.insert(entry.object_id))
        {
            return Err(Diagnostic::new("invalid-layout-align-self", "runtimeRequirements.layout_align_self",
                "Version 6 requires unique alignment targets and the matching CSS align-self capability"));
        }
        let has_layout = !self.layout_pixel_bounds.is_empty();
        let mut layouts = BTreeSet::new();
        if (self.version < 6 && (self.version == 5) != has_layout)
            || self.capabilities.contains(&RuntimeCapability::LayoutCssPixelBoundsV1) != has_layout
            || self.layout_pixel_bounds.iter().any(|id| !layouts.insert(*id))
        {
            return Err(Diagnostic::new("invalid-layout-pixel-bounds", "runtimeRequirements.layout_pixel_bounds",
                "CSS pixel-bounds require version 5 through 12, unique LayoutComponent targets and the matching capability"));
        }
        let mut strike_targets = BTreeSet::new();
        let has_strikes = !self.text_strikethroughs.is_empty();
        if (self.version < 5 && (self.version == 4) != has_strikes)
            || self
                .capabilities
                .contains(&RuntimeCapability::TextSolidStrikethroughsV1)
                != has_strikes
            || self.text_strikethroughs.iter().any(|entry| {
                !strike_targets.insert(entry.object_id)
                    || entry.lines.is_empty()
                    || entry.lines.iter().any(|line| {
                        !line.line_baseline.is_finite()
                            || line.line_baseline.abs() > 1_000_000.0
                            || !line.thickness.is_finite()
                            || line.thickness <= 0.0
                            || line.thickness > 1_000_000.0
                            || !line.offset.is_finite()
                            || line.offset.abs() > 1_000_000.0
                    })
            })
        {
            return Err(Diagnostic::new(
                "invalid-text-strikethroughs",
                "runtimeRequirements.text_strikethroughs",
                "Versions 4 through 11 require unique Text targets with nonempty resolved strikethrough lists, matching capability, and finite bounded metrics",
            ));
        }
        let mut underline_targets = BTreeSet::new();
        let has_underlines = !self.text_underlines.is_empty();
        if (self.version < 4 && (self.version == 3) != has_underlines)
            || self
                .capabilities
                .contains(&RuntimeCapability::TextSolidUnderlinesV1)
                != has_underlines
            || self.text_underlines.iter().any(|entry| {
                !underline_targets.insert(entry.object_id)
                    || entry.lines.is_empty()
                    || entry.lines.iter().any(|line| {
                        !line.thickness.is_finite()
                            || line.thickness <= 0.0
                            || line.thickness > 1_000_000.0
                            || !line.offset.is_finite()
                            || line.offset.abs() > 1_000_000.0
                    })
            })
        {
            return Err(Diagnostic::new(
                "invalid-text-underlines",
                "runtimeRequirements.text_underlines",
                "Underlines require version 3 through 11 and unique Text targets with nonempty resolved underline lists, matching capability, and finite bounded metrics",
            ));
        }
        let mut seen = BTreeSet::new();
        let invalid_tabs = self
            .capabilities
            .contains(&RuntimeCapability::TextCssWrappedTabsV1)
            && (self.version < 2
                || !self
                    .capabilities
                    .contains(&RuntimeCapability::TextCssTabsV1)
                || !self
                    .capabilities
                    .contains(&RuntimeCapability::TextCssPreWrapV1));
        let has_ellipsis = self.capabilities.contains(&RuntimeCapability::TextCssSingleLineEllipsisV1);
        let invalid_ellipsis = has_ellipsis && (self.version < 2
            || !self.capabilities.contains(&RuntimeCapability::TextCssShapingPrecisionV1));
        let invalid = invalid_ellipsis || invalid_tabs
            || if self.version == 1 {
                !self.text_policies.is_empty()
                    || self
                        .capabilities
                        .contains(&RuntimeCapability::TextCssPreWrapV1)
                    || self
                        .capabilities
                        .contains(&RuntimeCapability::TextCssPreLineV1)
                    || self.capabilities.contains(&RuntimeCapability::TextCssNormalWrapV1)
            } else {
                (self.version == 2 && self.text_policies.is_empty())
                    || self.text_policies.iter().any(|entry| {
                        !seen.insert(entry.object_id)
                            || !self.capabilities.contains(&entry.policy.capability())
                    })
                    || [
                        TextPolicy::CssSingleLineEllipsisV1,
                        TextPolicy::CssNowrapAlignmentV1,
                        TextPolicy::CssPreWrapV1,
                        TextPolicy::CssPreLineV1,
                        TextPolicy::CssNormalWrapV1,
                    ]
                    .iter()
                    .any(|policy| {
                        self.capabilities.contains(&policy.capability())
                            && !self
                                .text_policies
                                .iter()
                                .any(|entry| entry.policy == *policy)
                    })
            };
        if invalid {
            return Err(Diagnostic::new(
                "invalid-text-policies",
                "runtimeRequirements.text_policies",
                "Versions 2 through 11 require unique text policies with matching capabilities; version 1 cannot contain occurrence policies",
            ));
        }
        Ok(())
    }

    /// After capability validation and import, validate every target before
    /// installing policies or drawing. The callback must identify Text objects
    /// in the imported default artboard, not source-map layout objects.
    pub fn ensure_text_targets(
        &self,
        mut is_text: impl FnMut(u32) -> bool,
    ) -> Result<(), Diagnostic> {
        self.validate_text_policies()?;
        if self
            .text_strikethroughs
            .iter()
            .any(|entry| !is_text(entry.object_id))
        {
            return Err(Diagnostic::new(
                "invalid-text-strikethrough-target",
                "runtimeRequirements.text_strikethroughs",
                "Every strikethrough must target an existing Text object in the default artboard",
            ));
        }
        if self
            .text_underlines
            .iter()
            .any(|entry| !is_text(entry.object_id))
        {
            return Err(Diagnostic::new(
                "invalid-text-underline-target",
                "runtimeRequirements.text_underlines",
                "Every underline must target an existing Text object in the default artboard",
            ));
        }
        if self
            .text_policies
            .iter()
            .any(|entry| !is_text(entry.object_id))
        {
            return Err(Diagnostic::new(
                "invalid-text-policy-target",
                "runtimeRequirements.text_policies",
                "Every occurrence policy must target an existing Text object in the default artboard",
            ));
        }
        Ok(())
    }
}

impl RuntimeRequirements {
    /// Validate every layout target before installing any scene policies.
    /// The callback must check imported object types, not source-map membership.
    pub fn ensure_layout_targets(&self, mut is_layout: impl FnMut(u32) -> bool) -> Result<(), Diagnostic> {
        self.validate_text_policies()?;
        if self.layout_aspect_ratios.iter().any(|entry| !is_layout(entry.object_id)) {
            return Err(Diagnostic::new("invalid-layout-aspect-ratio-target", "runtimeRequirements.layout_aspect_ratios",
                "Ratio targets must identify layout objects"));
        }
        if self.layout_content_box.iter().any(|id| !is_layout(*id)) {
            return Err(Diagnostic::new("invalid-layout-content-box-target", "runtimeRequirements.layout_content_box",
                "Content-box targets must be LayoutComponents"));
        }
        if self.layout_intrinsic_sizing.iter().any(|id| !is_layout(*id)) {
            return Err(Diagnostic::new("invalid-layout-intrinsic-sizing-target", "runtimeRequirements.layout_intrinsic_sizing",
                "Intrinsic sizing targets must be LayoutComponents"));
        }
        if self.layout_flex_factors.iter().any(|entry| !is_layout(entry.object_id)) {
            return Err(Diagnostic::new("invalid-layout-flex-factors-target", "runtimeRequirements.layout_flex_factors",
                "Flex factor targets must be LayoutComponents"));
        }
        if self.layout_justify_content.iter().any(|entry| !is_layout(entry.object_id)) {
            return Err(Diagnostic::new("invalid-layout-justify-content-target", "runtimeRequirements.layout_justify_content",
                "Every CSS justify-content target must be an existing LayoutComponent in the default artboard"));
        }
        if self.layout_align_content.iter().any(|entry| !is_layout(entry.object_id)) {
            return Err(Diagnostic::new("invalid-layout-align-content-target", "runtimeRequirements.layout_align_content",
                "Every CSS align-content target must be an existing LayoutComponent in the default artboard"));
        }
        if self.layout_align_self.iter().any(|entry| !is_layout(entry.object_id)) {
            return Err(Diagnostic::new("invalid-layout-align-self-target", "runtimeRequirements.layout_align_self",
                "Every CSS align-self target must be an existing LayoutComponent in the default artboard"));
        }
        if self.layout_linear_gradients.iter().any(|entry| !is_layout(entry.object_id)) {
            return Err(Diagnostic::new("invalid-layout-linear-gradient-target", "runtimeRequirements.layout_linear_gradients",
                "Linear gradient targets must be LayoutComponent objects"));
        }
        if self.layout_group_opacity.iter().any(|entry| !is_layout(entry.object_id)) {
            return Err(Diagnostic::new("invalid-layout-group-opacity-target", "runtimeRequirements.layout_group_opacity",
                "Group opacity targets must be LayoutComponent objects"));
        }
        if self.layout_corner_radii.iter().any(|entry| !is_layout(entry.object_id)) {
            return Err(Diagnostic::new("invalid-layout-corner-radii-target", "runtimeRequirements.layout_corner_radii",
                "Corner radii require LayoutComponent targets"));
        }
        if self.layout_border_sides.iter().any(|entry| !is_layout(entry.object_id)) {
            return Err(Diagnostic::new("invalid-layout-border-sides-target", "runtimeRequirements.layout_border_sides",
                "Border sides require LayoutComponent targets"));
        }
        if self.layout_borders.iter().any(|entry| !is_layout(entry.object_id)) {
            return Err(Diagnostic::new("invalid-layout-border-target", "runtimeRequirements.layout_borders",
                "Borders require LayoutComponent targets"));
        }
        if self.layout_overflow_clip_margins.iter().any(|entry| !is_layout(entry.object_id)) {
            return Err(Diagnostic::new("invalid-layout-overflow-clip-margin-target", "runtimeRequirements.layout_overflow_clip_margins",
                "Clip margins require LayoutComponent targets"));
        }
        if self.layout_axis_overflow.iter().any(|entry| !is_layout(entry.object_id)) {
            return Err(Diagnostic::new("invalid-layout-axis-overflow-target", "runtimeRequirements.layout_axis_overflow",
                "Axis overflow requires LayoutComponent targets"));
        }
        if self.layout_stacking.iter().any(|entry| !is_layout(entry.object_id)) {
            return Err(Diagnostic::new("invalid-layout-stacking-target", "runtimeRequirements.layout_stacking",
                "Stacking contexts require LayoutComponent targets"));
        }
        if self.layout_positioned.iter().any(|id| !is_layout(*id)) {
            return Err(Diagnostic::new("invalid-layout-positioned-paint-target", "runtimeRequirements.layout_positioned",
                "Positioned paint requires LayoutComponent targets"));
        }
        if self.layout_percentage_spacing.iter().any(|id| !is_layout(*id)) {
            return Err(Diagnostic::new("invalid-layout-percentage-spacing-target", "runtimeRequirements.layout_percentage_spacing",
                "Percentage spacing requires LayoutComponent targets"));
        }
        if self.layout_pixel_bounds.iter().any(|id| !is_layout(*id)) {
            return Err(Diagnostic::new("invalid-layout-pixel-bounds-target", "runtimeRequirements.layout_pixel_bounds",
                "Every CSS pixel-bounds target must be an existing LayoutComponent in the default artboard"));
        }
        Ok(())
    }
}
