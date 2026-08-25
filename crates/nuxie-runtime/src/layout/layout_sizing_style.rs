// Direct source-correspondence owner for pinned `src/layout/layout_sizing_style.cpp`.
#[derive(Clone, Copy)]
enum RuntimeLayoutStyleProperty {
    DisplayValue,
    JustifyItemsValue,
    JustifySelfValue,
    LayoutTypeValue,
    PositionTypeValue,
    DirectionValue,
    FlexDirectionValue,
    FlexWrapValue,
    LayoutAlignmentType,
    LayoutWidthScaleType,
    LayoutHeightScaleType,
    WidthUnitsValue,
    HeightUnitsValue,
    FlexBasis,
    FlexBasisUnitsValue,
    GapHorizontal,
    GapHorizontalUnitsValue,
    GapVertical,
    GapVerticalUnitsValue,
    PaddingLeft,
    PaddingLeftUnitsValue,
    PaddingRight,
    PaddingRightUnitsValue,
    PaddingTop,
    PaddingTopUnitsValue,
    PaddingBottom,
    PaddingBottomUnitsValue,
    BorderLeft,
    BorderLeftUnitsValue,
    BorderRight,
    BorderRightUnitsValue,
    BorderTop,
    BorderTopUnitsValue,
    BorderBottom,
    BorderBottomUnitsValue,
    MarginLeft,
    MarginLeftUnitsValue,
    MarginRight,
    MarginRightUnitsValue,
    MarginTop,
    MarginTopUnitsValue,
    MarginBottom,
    MarginBottomUnitsValue,
    PositionLeft,
    PositionLeftUnitsValue,
    PositionRight,
    PositionRightUnitsValue,
    PositionTop,
    PositionTopUnitsValue,
    PositionBottom,
    PositionBottomUnitsValue,
    MinWidth,
    MinWidthUnitsValue,
    MinHeight,
    MinHeightUnitsValue,
    MaxWidth,
    MaxWidthUnitsValue,
    MaxHeight,
    MaxHeightUnitsValue,
    AspectRatio,
    IntrinsicallySizedValue,
    LinkCornerRadius,
    CornerRadiusTL,
    CornerRadiusTR,
    CornerRadiusBR,
    CornerRadiusBL,
}

impl RuntimeLayoutStyleProperty {
    #[inline]
    fn key(self) -> Option<u16> {
        match self {
            Self::DisplayValue => {
                cached_runtime_property_key!("LayoutComponentStyle", "displayValue")
            }
            Self::JustifyItemsValue => {
                cached_runtime_property_key!("LayoutComponentStyle", "justifyItemsValue")
            }
            Self::JustifySelfValue => {
                cached_runtime_property_key!("LayoutComponentStyle", "justifySelfValue")
            }
            Self::LayoutTypeValue => {
                cached_runtime_property_key!("LayoutComponentStyle", "layoutTypeValue")
            }
            Self::PositionTypeValue => {
                cached_runtime_property_key!("LayoutComponentStyle", "positionTypeValue")
            }
            Self::DirectionValue => {
                cached_runtime_property_key!("LayoutComponentStyle", "directionValue")
            }
            Self::FlexDirectionValue => {
                cached_runtime_property_key!("LayoutComponentStyle", "flexDirectionValue")
            }
            Self::FlexWrapValue => {
                cached_runtime_property_key!("LayoutComponentStyle", "flexWrapValue")
            }
            Self::LayoutAlignmentType => {
                cached_runtime_property_key!("LayoutComponentStyle", "layoutAlignmentType")
            }
            Self::LayoutWidthScaleType => {
                cached_runtime_property_key!("LayoutComponentStyle", "layoutWidthScaleType")
            }
            Self::LayoutHeightScaleType => {
                cached_runtime_property_key!("LayoutComponentStyle", "layoutHeightScaleType")
            }
            Self::WidthUnitsValue => {
                cached_runtime_property_key!("LayoutComponentStyle", "widthUnitsValue")
            }
            Self::HeightUnitsValue => {
                cached_runtime_property_key!("LayoutComponentStyle", "heightUnitsValue")
            }
            Self::FlexBasis => cached_runtime_property_key!("LayoutComponentStyle", "flexBasis"),
            Self::FlexBasisUnitsValue => {
                cached_runtime_property_key!("LayoutComponentStyle", "flexBasisUnitsValue")
            }
            Self::GapHorizontal => {
                cached_runtime_property_key!("LayoutComponentStyle", "gapHorizontal")
            }
            Self::GapHorizontalUnitsValue => {
                cached_runtime_property_key!("LayoutComponentStyle", "gapHorizontalUnitsValue")
            }
            Self::GapVertical => {
                cached_runtime_property_key!("LayoutComponentStyle", "gapVertical")
            }
            Self::GapVerticalUnitsValue => {
                cached_runtime_property_key!("LayoutComponentStyle", "gapVerticalUnitsValue")
            }
            Self::PaddingLeft => {
                cached_runtime_property_key!("LayoutComponentStyle", "paddingLeft")
            }
            Self::PaddingLeftUnitsValue => {
                cached_runtime_property_key!("LayoutComponentStyle", "paddingLeftUnitsValue")
            }
            Self::PaddingRight => {
                cached_runtime_property_key!("LayoutComponentStyle", "paddingRight")
            }
            Self::PaddingRightUnitsValue => {
                cached_runtime_property_key!("LayoutComponentStyle", "paddingRightUnitsValue")
            }
            Self::PaddingTop => {
                cached_runtime_property_key!("LayoutComponentStyle", "paddingTop")
            }
            Self::PaddingTopUnitsValue => {
                cached_runtime_property_key!("LayoutComponentStyle", "paddingTopUnitsValue")
            }
            Self::PaddingBottom => {
                cached_runtime_property_key!("LayoutComponentStyle", "paddingBottom")
            }
            Self::PaddingBottomUnitsValue => {
                cached_runtime_property_key!("LayoutComponentStyle", "paddingBottomUnitsValue")
            }
            Self::BorderLeft => {
                cached_runtime_property_key!("LayoutComponentStyle", "borderLeft")
            }
            Self::BorderLeftUnitsValue => {
                cached_runtime_property_key!("LayoutComponentStyle", "borderLeftUnitsValue")
            }
            Self::BorderRight => {
                cached_runtime_property_key!("LayoutComponentStyle", "borderRight")
            }
            Self::BorderRightUnitsValue => {
                cached_runtime_property_key!("LayoutComponentStyle", "borderRightUnitsValue")
            }
            Self::BorderTop => cached_runtime_property_key!("LayoutComponentStyle", "borderTop"),
            Self::BorderTopUnitsValue => {
                cached_runtime_property_key!("LayoutComponentStyle", "borderTopUnitsValue")
            }
            Self::BorderBottom => {
                cached_runtime_property_key!("LayoutComponentStyle", "borderBottom")
            }
            Self::BorderBottomUnitsValue => {
                cached_runtime_property_key!("LayoutComponentStyle", "borderBottomUnitsValue")
            }
            Self::MarginLeft => {
                cached_runtime_property_key!("LayoutComponentStyle", "marginLeft")
            }
            Self::MarginLeftUnitsValue => {
                cached_runtime_property_key!("LayoutComponentStyle", "marginLeftUnitsValue")
            }
            Self::MarginRight => {
                cached_runtime_property_key!("LayoutComponentStyle", "marginRight")
            }
            Self::MarginRightUnitsValue => {
                cached_runtime_property_key!("LayoutComponentStyle", "marginRightUnitsValue")
            }
            Self::MarginTop => cached_runtime_property_key!("LayoutComponentStyle", "marginTop"),
            Self::MarginTopUnitsValue => {
                cached_runtime_property_key!("LayoutComponentStyle", "marginTopUnitsValue")
            }
            Self::MarginBottom => {
                cached_runtime_property_key!("LayoutComponentStyle", "marginBottom")
            }
            Self::MarginBottomUnitsValue => {
                cached_runtime_property_key!("LayoutComponentStyle", "marginBottomUnitsValue")
            }
            Self::PositionLeft => {
                cached_runtime_property_key!("LayoutComponentStyle", "positionLeft")
            }
            Self::PositionLeftUnitsValue => {
                cached_runtime_property_key!("LayoutComponentStyle", "positionLeftUnitsValue")
            }
            Self::PositionRight => {
                cached_runtime_property_key!("LayoutComponentStyle", "positionRight")
            }
            Self::PositionRightUnitsValue => {
                cached_runtime_property_key!("LayoutComponentStyle", "positionRightUnitsValue")
            }
            Self::PositionTop => {
                cached_runtime_property_key!("LayoutComponentStyle", "positionTop")
            }
            Self::PositionTopUnitsValue => {
                cached_runtime_property_key!("LayoutComponentStyle", "positionTopUnitsValue")
            }
            Self::PositionBottom => {
                cached_runtime_property_key!("LayoutComponentStyle", "positionBottom")
            }
            Self::PositionBottomUnitsValue => {
                cached_runtime_property_key!("LayoutComponentStyle", "positionBottomUnitsValue")
            }
            Self::MinWidth => cached_runtime_property_key!("LayoutComponentStyle", "minWidth"),
            Self::MinWidthUnitsValue => {
                cached_runtime_property_key!("LayoutComponentStyle", "minWidthUnitsValue")
            }
            Self::MinHeight => cached_runtime_property_key!("LayoutComponentStyle", "minHeight"),
            Self::MinHeightUnitsValue => {
                cached_runtime_property_key!("LayoutComponentStyle", "minHeightUnitsValue")
            }
            Self::MaxWidth => cached_runtime_property_key!("LayoutComponentStyle", "maxWidth"),
            Self::MaxWidthUnitsValue => {
                cached_runtime_property_key!("LayoutComponentStyle", "maxWidthUnitsValue")
            }
            Self::MaxHeight => cached_runtime_property_key!("LayoutComponentStyle", "maxHeight"),
            Self::MaxHeightUnitsValue => {
                cached_runtime_property_key!("LayoutComponentStyle", "maxHeightUnitsValue")
            }
            Self::AspectRatio => {
                cached_runtime_property_key!("LayoutComponentStyle", "aspectRatio")
            }
            Self::IntrinsicallySizedValue => {
                cached_runtime_property_key!("LayoutComponentStyle", "intrinsicallySizedValue")
            }
            Self::LinkCornerRadius => {
                cached_runtime_property_key!("LayoutComponentStyle", "linkCornerRadius")
            }
            Self::CornerRadiusTL => {
                cached_runtime_property_key!("LayoutComponentStyle", "cornerRadiusTL")
            }
            Self::CornerRadiusTR => {
                cached_runtime_property_key!("LayoutComponentStyle", "cornerRadiusTR")
            }
            Self::CornerRadiusBR => {
                cached_runtime_property_key!("LayoutComponentStyle", "cornerRadiusBR")
            }
            Self::CornerRadiusBL => {
                cached_runtime_property_key!("LayoutComponentStyle", "cornerRadiusBL")
            }
        }
    }
}
