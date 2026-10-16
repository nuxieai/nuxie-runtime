//! Reviewed CSS layout differences from the pinned Rive matrix.
//! Every scalar change retains its original C++ value. All other fields stay
//! exact under the matrix's existing epsilon. The first difference is pinned
//! as text too, so numeric drift or a disappeared difference requires review.

pub struct ScalarPin {
    pub field: &'static str,
    pub index: usize,
    pub upstream: f32,
    pub css: f32,
}

pub struct LayoutDivergence {
    pub fixture: &'static str,
    pub reason: &'static str,
    pub first_difference: &'static str,
    pub pins: &'static [ScalarPin],
}

pub const LAYOUT_DIVERGENCES: &[LayoutDivergence] = &[
    LayoutDivergence {
        fixture: "s7__flex-row__none__layout-component__k-min-over-fill",
        reason: "layout-diverges:CSS flex-shrink 1 lets the empty Fixed sibling give up width: its 60 basis shrinks to 20 beside the subject's authored min-width 180 in a 200 row; the subject keeps its size and its origin shifts with the sibling.",
        first_difference: "Subject.world[4]: 20 != 60",
        pins: &[
            ScalarPin {
                field: "world",
                index: 4,
                upstream: 60.0,
                css: 20.0,
            },
            ScalarPin {
                field: "layoutBounds",
                index: 0,
                upstream: 60.0,
                css: 20.0,
            },
            ScalarPin {
                field: "worldBounds",
                index: 0,
                upstream: 60.0,
                css: 20.0,
            },
            ScalarPin {
                field: "computedWorld",
                index: 0,
                upstream: 60.0,
                css: 20.0,
            },
            ScalarPin {
                field: "computedRoot",
                index: 0,
                upstream: 60.0,
                css: 20.0,
            },
            ScalarPin {
                field: "computedLocal",
                index: 0,
                upstream: 60.0,
                css: 20.0,
            },
        ],
    },
    LayoutDivergence {
        fixture: "s7__flex-row__none__shape__participant__k-min-over-fill",
        reason: "layout-diverges:CSS flex-shrink 1 lets the empty Fixed sibling give up width: its 60 basis shrinks to 20 beside the subject's authored min-width 180 in a 200 row; the subject keeps its size and its origin shifts with the sibling.",
        first_difference: "Subject.world[4]: 110 != 150",
        pins: &[
            ScalarPin {
                field: "world",
                index: 4,
                upstream: 150.0,
                css: 110.0,
            },
            ScalarPin {
                field: "worldBounds",
                index: 0,
                upstream: 60.0,
                css: 20.0,
            },
            ScalarPin {
                field: "computedWorld",
                index: 0,
                upstream: 150.0,
                css: 110.0,
            },
            ScalarPin {
                field: "computedRoot",
                index: 0,
                upstream: 150.0,
                css: 110.0,
            },
            ScalarPin {
                field: "computedLocal",
                index: 0,
                upstream: 150.0,
                css: 110.0,
            },
        ],
    },
    LayoutDivergence {
        fixture: "s7__flex-column__none__layout-component__k-min-over-fill",
        reason: "layout-diverges:CSS flex-shrink 1 lets the empty Fixed sibling give up height: its 40 basis shrinks to zero beside the subject's authored min-height 150 in a 120 column; the subject keeps its size and its origin shifts with the sibling.",
        first_difference: "Subject.world[5]: 0 != 40",
        pins: &[
            ScalarPin {
                field: "world",
                index: 5,
                upstream: 40.0,
                css: 0.0,
            },
            ScalarPin {
                field: "layoutBounds",
                index: 1,
                upstream: 40.0,
                css: 0.0,
            },
            ScalarPin {
                field: "worldBounds",
                index: 1,
                upstream: 40.0,
                css: 0.0,
            },
            ScalarPin {
                field: "computedWorld",
                index: 1,
                upstream: 40.0,
                css: 0.0,
            },
            ScalarPin {
                field: "computedRoot",
                index: 1,
                upstream: 40.0,
                css: 0.0,
            },
            ScalarPin {
                field: "computedLocal",
                index: 1,
                upstream: 40.0,
                css: 0.0,
            },
        ],
    },
    LayoutDivergence {
        fixture: "s7__flex-column__none__shape__participant__k-min-over-fill",
        reason: "layout-diverges:CSS flex-shrink 1 lets the empty Fixed sibling give up height: its 40 basis shrinks to zero beside the subject's authored min-height 150 in a 120 column; the subject keeps its size and its origin shifts with the sibling.",
        first_difference: "Subject.world[5]: 75 != 115",
        pins: &[
            ScalarPin {
                field: "world",
                index: 5,
                upstream: 115.0,
                css: 75.0,
            },
            ScalarPin {
                field: "worldBounds",
                index: 1,
                upstream: 40.0,
                css: 0.0,
            },
            ScalarPin {
                field: "computedWorld",
                index: 1,
                upstream: 115.0,
                css: 75.0,
            },
            ScalarPin {
                field: "computedRoot",
                index: 1,
                upstream: 115.0,
                css: 75.0,
            },
            ScalarPin {
                field: "computedLocal",
                index: 1,
                upstream: 115.0,
                css: 75.0,
            },
        ],
    },
    LayoutDivergence {
        fixture: "s20__flex-row__none__layout-component__hug__leaf-content__over",
        reason: "layout-diverges:CSS min-width:auto keeps the Hug content floor at 260, capped by max-width where authored, above the 200 row offer; flex-shrink 1 reduces the empty Fixed sibling from its 60 basis to zero.",
        first_difference: "Subject.world[4]: 0 != 60",
        pins: &[
            ScalarPin {
                field: "world",
                index: 4,
                upstream: 60.0,
                css: 0.0,
            },
            ScalarPin {
                field: "layoutBounds",
                index: 0,
                upstream: 60.0,
                css: 0.0,
            },
            ScalarPin {
                field: "layoutBounds",
                index: 2,
                upstream: 200.0,
                css: 260.0,
            },
            ScalarPin {
                field: "localBounds",
                index: 2,
                upstream: 200.0,
                css: 260.0,
            },
            ScalarPin {
                field: "worldBounds",
                index: 0,
                upstream: 60.0,
                css: 0.0,
            },
            ScalarPin {
                field: "worldBounds",
                index: 2,
                upstream: 200.0,
                css: 260.0,
            },
            ScalarPin {
                field: "computedWorld",
                index: 0,
                upstream: 60.0,
                css: 0.0,
            },
            ScalarPin {
                field: "computedRoot",
                index: 0,
                upstream: 60.0,
                css: 0.0,
            },
            ScalarPin {
                field: "computedLocal",
                index: 0,
                upstream: 60.0,
                css: 0.0,
            },
            ScalarPin {
                field: "computedSize",
                index: 0,
                upstream: 200.0,
                css: 260.0,
            },
        ],
    },
    LayoutDivergence {
        fixture: "s20__flex-row__none__layout-component__hug__leaf-content__over__unbounded",
        reason: "layout-diverges:CSS min-width:auto keeps the Hug content floor at 260, capped by max-width where authored, above the 200 row offer; flex-shrink 1 reduces the empty Fixed sibling from its 60 basis to zero. The subject already kept its intrinsic width when unbounded; the moved values follow the shrinking sibling.",
        first_difference: "Subject.world[4]: 0 != 60",
        pins: &[
            ScalarPin {
                field: "world",
                index: 4,
                upstream: 60.0,
                css: 0.0,
            },
            ScalarPin {
                field: "layoutBounds",
                index: 0,
                upstream: 60.0,
                css: 0.0,
            },
            ScalarPin {
                field: "worldBounds",
                index: 0,
                upstream: 60.0,
                css: 0.0,
            },
            ScalarPin {
                field: "computedWorld",
                index: 0,
                upstream: 60.0,
                css: 0.0,
            },
            ScalarPin {
                field: "computedRoot",
                index: 0,
                upstream: 60.0,
                css: 0.0,
            },
            ScalarPin {
                field: "computedLocal",
                index: 0,
                upstream: 60.0,
                css: 0.0,
            },
        ],
    },
    LayoutDivergence {
        fixture: "s20__flex-row__none__shape__participant__hug__over",
        reason: "layout-diverges:CSS min-width:auto keeps the Hug content floor at 260, capped by max-width where authored, above the 200 row offer; flex-shrink 1 reduces the empty Fixed sibling from its 60 basis to zero.",
        first_difference: "Subject.world[0]: 1 != 0.7692",
        pins: &[
            ScalarPin {
                field: "world",
                index: 0,
                upstream: 0.7692,
                css: 1.0,
            },
            ScalarPin {
                field: "world",
                index: 4,
                upstream: 160.0,
                css: 130.0,
            },
            ScalarPin {
                field: "worldBounds",
                index: 0,
                upstream: 60.0,
                css: 0.0,
            },
            ScalarPin {
                field: "worldBounds",
                index: 2,
                upstream: 200.0,
                css: 260.0,
            },
            ScalarPin {
                field: "computedWorld",
                index: 0,
                upstream: 160.0,
                css: 130.0,
            },
            ScalarPin {
                field: "computedRoot",
                index: 0,
                upstream: 160.0,
                css: 130.0,
            },
            ScalarPin {
                field: "computedLocal",
                index: 0,
                upstream: 160.0,
                css: 130.0,
            },
        ],
    },
    LayoutDivergence {
        fixture: "s20__flex-row__none__shape__participant__hug__over__unbounded",
        reason: "layout-diverges:CSS min-width:auto keeps the Hug content floor at 260, capped by max-width where authored, above the 200 row offer; flex-shrink 1 reduces the empty Fixed sibling from its 60 basis to zero. The subject already kept its intrinsic width when unbounded; the moved values follow the shrinking sibling.",
        first_difference: "Subject.world[4]: 130 != 190",
        pins: &[
            ScalarPin {
                field: "world",
                index: 4,
                upstream: 190.0,
                css: 130.0,
            },
            ScalarPin {
                field: "worldBounds",
                index: 0,
                upstream: 60.0,
                css: 0.0,
            },
            ScalarPin {
                field: "computedWorld",
                index: 0,
                upstream: 190.0,
                css: 130.0,
            },
            ScalarPin {
                field: "computedRoot",
                index: 0,
                upstream: 190.0,
                css: 130.0,
            },
            ScalarPin {
                field: "computedLocal",
                index: 0,
                upstream: 190.0,
                css: 130.0,
            },
        ],
    },
    LayoutDivergence {
        fixture: "s20__flex-column__none__layout-component__hug__leaf-content__over",
        reason: "layout-diverges:CSS min-height:auto keeps the Hug content floor at 180, capped by max-height where authored, above the 120 column offer; flex-shrink 1 reduces the empty Fixed sibling from its 40 basis to zero.",
        first_difference: "Subject.world[5]: 0 != 40",
        pins: &[
            ScalarPin {
                field: "world",
                index: 5,
                upstream: 40.0,
                css: 0.0,
            },
            ScalarPin {
                field: "layoutBounds",
                index: 1,
                upstream: 40.0,
                css: 0.0,
            },
            ScalarPin {
                field: "layoutBounds",
                index: 3,
                upstream: 120.0,
                css: 180.0,
            },
            ScalarPin {
                field: "localBounds",
                index: 3,
                upstream: 120.0,
                css: 180.0,
            },
            ScalarPin {
                field: "worldBounds",
                index: 1,
                upstream: 40.0,
                css: 0.0,
            },
            ScalarPin {
                field: "worldBounds",
                index: 3,
                upstream: 120.0,
                css: 180.0,
            },
            ScalarPin {
                field: "computedWorld",
                index: 1,
                upstream: 40.0,
                css: 0.0,
            },
            ScalarPin {
                field: "computedRoot",
                index: 1,
                upstream: 40.0,
                css: 0.0,
            },
            ScalarPin {
                field: "computedLocal",
                index: 1,
                upstream: 40.0,
                css: 0.0,
            },
            ScalarPin {
                field: "computedSize",
                index: 1,
                upstream: 120.0,
                css: 180.0,
            },
        ],
    },
    LayoutDivergence {
        fixture: "s20__flex-column__none__layout-component__hug__leaf-content__over__unbounded",
        reason: "layout-diverges:CSS min-height:auto keeps the Hug content floor at 180, capped by max-height where authored, above the 120 column offer; flex-shrink 1 reduces the empty Fixed sibling from its 40 basis to zero. The subject already kept its intrinsic height when unbounded; the moved values follow the shrinking sibling.",
        first_difference: "Subject.world[5]: 0 != 40",
        pins: &[
            ScalarPin {
                field: "world",
                index: 5,
                upstream: 40.0,
                css: 0.0,
            },
            ScalarPin {
                field: "layoutBounds",
                index: 1,
                upstream: 40.0,
                css: 0.0,
            },
            ScalarPin {
                field: "worldBounds",
                index: 1,
                upstream: 40.0,
                css: 0.0,
            },
            ScalarPin {
                field: "computedWorld",
                index: 1,
                upstream: 40.0,
                css: 0.0,
            },
            ScalarPin {
                field: "computedRoot",
                index: 1,
                upstream: 40.0,
                css: 0.0,
            },
            ScalarPin {
                field: "computedLocal",
                index: 1,
                upstream: 40.0,
                css: 0.0,
            },
        ],
    },
    LayoutDivergence {
        fixture: "s20__flex-column__none__shape__participant__hug__over",
        reason: "layout-diverges:CSS min-height:auto keeps the Hug content floor at 180, capped by max-height where authored, above the 120 column offer; flex-shrink 1 reduces the empty Fixed sibling from its 40 basis to zero.",
        first_difference: "Subject.world[3]: 1 != 0.6667",
        pins: &[
            ScalarPin {
                field: "world",
                index: 3,
                upstream: 0.6667,
                css: 1.0,
            },
            ScalarPin {
                field: "world",
                index: 5,
                upstream: 100.0,
                css: 90.0,
            },
            ScalarPin {
                field: "worldBounds",
                index: 1,
                upstream: 40.0,
                css: 0.0,
            },
            ScalarPin {
                field: "worldBounds",
                index: 3,
                upstream: 120.0,
                css: 180.0,
            },
            ScalarPin {
                field: "computedWorld",
                index: 1,
                upstream: 100.0,
                css: 90.0,
            },
            ScalarPin {
                field: "computedRoot",
                index: 1,
                upstream: 100.0,
                css: 90.0,
            },
            ScalarPin {
                field: "computedLocal",
                index: 1,
                upstream: 100.0,
                css: 90.0,
            },
        ],
    },
    LayoutDivergence {
        fixture: "s20__flex-column__none__shape__participant__hug__over__unbounded",
        reason: "layout-diverges:CSS min-height:auto keeps the Hug content floor at 180, capped by max-height where authored, above the 120 column offer; flex-shrink 1 reduces the empty Fixed sibling from its 40 basis to zero. The subject already kept its intrinsic height when unbounded; the moved values follow the shrinking sibling.",
        first_difference: "Subject.world[5]: 90 != 130",
        pins: &[
            ScalarPin {
                field: "world",
                index: 5,
                upstream: 130.0,
                css: 90.0,
            },
            ScalarPin {
                field: "worldBounds",
                index: 1,
                upstream: 40.0,
                css: 0.0,
            },
            ScalarPin {
                field: "computedWorld",
                index: 1,
                upstream: 130.0,
                css: 90.0,
            },
            ScalarPin {
                field: "computedRoot",
                index: 1,
                upstream: 130.0,
                css: 90.0,
            },
            ScalarPin {
                field: "computedLocal",
                index: 1,
                upstream: 130.0,
                css: 90.0,
            },
        ],
    },
    LayoutDivergence {
        fixture: "s20__flex-row__none__layout-component__hug__k-max-over-fill__leaf-content__over__unbounded",
        reason: "layout-diverges:CSS caps the Hug content minimum 260 at max-width 180, then flex-shrink 1 reduces the empty Fixed sibling from its 60 basis to the remaining 20 in the 200 row. The subject width stays 180 and its origin shifts with the sibling.",
        first_difference: "Subject.world[4]: 20 != 60",
        pins: &[
            ScalarPin {
                field: "world",
                index: 4,
                upstream: 60.0,
                css: 20.0,
            },
            ScalarPin {
                field: "layoutBounds",
                index: 0,
                upstream: 60.0,
                css: 20.0,
            },
            ScalarPin {
                field: "worldBounds",
                index: 0,
                upstream: 60.0,
                css: 20.0,
            },
            ScalarPin {
                field: "computedWorld",
                index: 0,
                upstream: 60.0,
                css: 20.0,
            },
            ScalarPin {
                field: "computedRoot",
                index: 0,
                upstream: 60.0,
                css: 20.0,
            },
            ScalarPin {
                field: "computedLocal",
                index: 0,
                upstream: 60.0,
                css: 20.0,
            },
        ],
    },
    LayoutDivergence {
        fixture: "s20__flex-row__none__layout-component__hug__y-fixed__leaf-content__over__unbounded",
        reason: "layout-diverges:CSS min-width:auto keeps the Hug content floor at 260, capped by max-width where authored, above the 200 row offer; flex-shrink 1 reduces the empty Fixed sibling from its 60 basis to zero. The subject already kept its intrinsic width when unbounded; the moved values follow the shrinking sibling. The authored Fixed cross-axis height remains 40.",
        first_difference: "Subject.world[4]: 0 != 60",
        pins: &[
            ScalarPin {
                field: "world",
                index: 4,
                upstream: 60.0,
                css: 0.0,
            },
            ScalarPin {
                field: "layoutBounds",
                index: 0,
                upstream: 60.0,
                css: 0.0,
            },
            ScalarPin {
                field: "worldBounds",
                index: 0,
                upstream: 60.0,
                css: 0.0,
            },
            ScalarPin {
                field: "computedWorld",
                index: 0,
                upstream: 60.0,
                css: 0.0,
            },
            ScalarPin {
                field: "computedRoot",
                index: 0,
                upstream: 60.0,
                css: 0.0,
            },
            ScalarPin {
                field: "computedLocal",
                index: 0,
                upstream: 60.0,
                css: 0.0,
            },
        ],
    },
    LayoutDivergence {
        fixture: "s20__flex-row__none__shape__participant__hug__k-max-over-fill__over__unbounded",
        reason: "layout-diverges:CSS caps the Hug content minimum 260 at max-width 180, then flex-shrink 1 reduces the empty Fixed sibling from its 60 basis to the remaining 20 in the 200 row. The subject width stays 180 and its origin shifts with the sibling.",
        first_difference: "Subject.world[4]: 110 != 150",
        pins: &[
            ScalarPin {
                field: "world",
                index: 4,
                upstream: 150.0,
                css: 110.0,
            },
            ScalarPin {
                field: "worldBounds",
                index: 0,
                upstream: 60.0,
                css: 20.0,
            },
            ScalarPin {
                field: "computedWorld",
                index: 0,
                upstream: 150.0,
                css: 110.0,
            },
            ScalarPin {
                field: "computedRoot",
                index: 0,
                upstream: 150.0,
                css: 110.0,
            },
            ScalarPin {
                field: "computedLocal",
                index: 0,
                upstream: 150.0,
                css: 110.0,
            },
        ],
    },
    LayoutDivergence {
        fixture: "s20__flex-row__none__shape__participant__hug__y-fixed__over__unbounded",
        reason: "layout-diverges:CSS min-width:auto keeps the Hug content floor at 260, capped by max-width where authored, above the 200 row offer; flex-shrink 1 reduces the empty Fixed sibling from its 60 basis to zero. The subject already kept its intrinsic width when unbounded; the moved values follow the shrinking sibling. The authored Fixed cross-axis height remains 40.",
        first_difference: "Subject.world[4]: 130 != 190",
        pins: &[
            ScalarPin {
                field: "world",
                index: 4,
                upstream: 190.0,
                css: 130.0,
            },
            ScalarPin {
                field: "worldBounds",
                index: 0,
                upstream: 60.0,
                css: 0.0,
            },
            ScalarPin {
                field: "computedWorld",
                index: 0,
                upstream: 190.0,
                css: 130.0,
            },
            ScalarPin {
                field: "computedRoot",
                index: 0,
                upstream: 190.0,
                css: 130.0,
            },
            ScalarPin {
                field: "computedLocal",
                index: 0,
                upstream: 190.0,
                css: 130.0,
            },
        ],
    },
    LayoutDivergence {
        fixture: "s20__flex-row__none__shape__participant__fill__y-hug__sibling-fills__over",
        reason: "layout-diverges:CSS min-width:auto floors the shape at its intrinsic width 260 and the leading Fill sibling at its 50 content contribution, so they overflow the 200 row at widths 50 and 260. Fill fractions and bases are unchanged.",
        first_difference: "Subject.world[0]: 1 != 0.2885",
        pins: &[
            ScalarPin {
                field: "world",
                index: 0,
                upstream: 0.2885,
                css: 1.0,
            },
            ScalarPin {
                field: "world",
                index: 4,
                upstream: 162.5,
                css: 180.0,
            },
            ScalarPin {
                field: "worldBounds",
                index: 0,
                upstream: 125.0,
                css: 50.0,
            },
            ScalarPin {
                field: "worldBounds",
                index: 2,
                upstream: 75.0,
                css: 260.0,
            },
            ScalarPin {
                field: "computedWorld",
                index: 0,
                upstream: 162.5,
                css: 180.0,
            },
            ScalarPin {
                field: "computedRoot",
                index: 0,
                upstream: 162.5,
                css: 180.0,
            },
            ScalarPin {
                field: "computedLocal",
                index: 0,
                upstream: 162.5,
                css: 180.0,
            },
        ],
    },
    LayoutDivergence {
        fixture: "s20__flex-row__none__shape__participant__fill__y-hug__sibling-fills__over__unbounded",
        reason: "layout-diverges:CSS min-width:auto floors the shape at its intrinsic width 260 and the leading Fill sibling at its 50 content contribution, so they overflow the 200 row at widths 50 and 260. Fill fractions and bases are unchanged. Width is Fill, so hugUnbounded only changes the height mode.",
        first_difference: "Subject.world[0]: 1 != 0.2885",
        pins: &[
            ScalarPin {
                field: "world",
                index: 0,
                upstream: 0.2885,
                css: 1.0,
            },
            ScalarPin {
                field: "world",
                index: 4,
                upstream: 162.5,
                css: 180.0,
            },
            ScalarPin {
                field: "worldBounds",
                index: 0,
                upstream: 125.0,
                css: 50.0,
            },
            ScalarPin {
                field: "worldBounds",
                index: 2,
                upstream: 75.0,
                css: 260.0,
            },
            ScalarPin {
                field: "computedWorld",
                index: 0,
                upstream: 162.5,
                css: 180.0,
            },
            ScalarPin {
                field: "computedRoot",
                index: 0,
                upstream: 162.5,
                css: 180.0,
            },
            ScalarPin {
                field: "computedLocal",
                index: 0,
                upstream: 162.5,
                css: 180.0,
            },
        ],
    },
    LayoutDivergence {
        fixture: "s20__flex-row__none__layout-component__hug__over",
        reason: "layout-diverges:CSS min-width:auto keeps the Hug content floor at 260, capped by max-width where authored, above the 200 row offer; flex-shrink 1 reduces the empty Fixed sibling from its 60 basis to zero.",
        first_difference: "Subject.world[4]: 0 != 60",
        pins: &[
            ScalarPin {
                field: "world",
                index: 4,
                upstream: 60.0,
                css: 0.0,
            },
            ScalarPin {
                field: "layoutBounds",
                index: 0,
                upstream: 60.0,
                css: 0.0,
            },
            ScalarPin {
                field: "worldBounds",
                index: 0,
                upstream: 60.0,
                css: 0.0,
            },
            ScalarPin {
                field: "computedWorld",
                index: 0,
                upstream: 60.0,
                css: 0.0,
            },
            ScalarPin {
                field: "computedRoot",
                index: 0,
                upstream: 60.0,
                css: 0.0,
            },
            ScalarPin {
                field: "computedLocal",
                index: 0,
                upstream: 60.0,
                css: 0.0,
            },
        ],
    },
    LayoutDivergence {
        fixture: "s20__flex-row__none__layout-component__hug__over__unbounded",
        reason: "layout-diverges:CSS min-width:auto keeps the Hug content floor at 260, capped by max-width where authored, above the 200 row offer; flex-shrink 1 reduces the empty Fixed sibling from its 60 basis to zero. The subject already kept its intrinsic width when unbounded; the moved values follow the shrinking sibling.",
        first_difference: "Subject.world[4]: 0 != 60",
        pins: &[
            ScalarPin {
                field: "world",
                index: 4,
                upstream: 60.0,
                css: 0.0,
            },
            ScalarPin {
                field: "layoutBounds",
                index: 0,
                upstream: 60.0,
                css: 0.0,
            },
            ScalarPin {
                field: "worldBounds",
                index: 0,
                upstream: 60.0,
                css: 0.0,
            },
            ScalarPin {
                field: "computedWorld",
                index: 0,
                upstream: 60.0,
                css: 0.0,
            },
            ScalarPin {
                field: "computedRoot",
                index: 0,
                upstream: 60.0,
                css: 0.0,
            },
            ScalarPin {
                field: "computedLocal",
                index: 0,
                upstream: 60.0,
                css: 0.0,
            },
        ],
    },
    LayoutDivergence {
        fixture: "s18__flex-row__none__text__participant__tf-resizes-box",
        reason: "layout-diverges:CSS min-width:auto keeps the fitted text's unchanged unbounded width contribution 326.3672 rather than the 140 row remainder; the empty Fixed sibling shrinks from 60 to zero, placing the quarter-width origin at 81.5918. Fitted-font measurement remains the upstream protocol.",
        first_difference: "Subject.world[4]: 81.5918 != 95",
        pins: &[
            ScalarPin {
                field: "world",
                index: 4,
                upstream: 95.0,
                css: 81.5918,
            },
            ScalarPin {
                field: "computedWorld",
                index: 0,
                upstream: 95.0,
                css: 81.5918,
            },
            ScalarPin {
                field: "computedRoot",
                index: 0,
                upstream: 95.0,
                css: 81.5918,
            },
            ScalarPin {
                field: "computedLocal",
                index: 0,
                upstream: 95.0,
                css: 81.5918,
            },
            ScalarPin {
                field: "computedSize",
                index: 0,
                upstream: 140.0,
                css: 326.3672,
            },
        ],
    },
    LayoutDivergence {
        fixture: "s18__flex-row__none__text__participant__fill__y-hug__k-min-over-fill__tf-resizes-box",
        reason: "layout-diverges:CSS flex-shrink 1 lets the empty Fixed sibling give up width: its 60 basis shrinks to 20 beside the subject's authored min-width 180 in a 200 row; the subject keeps its size and its origin shifts with the sibling.",
        first_difference: "Subject.world[4]: 65 != 105",
        pins: &[
            ScalarPin {
                field: "world",
                index: 4,
                upstream: 105.0,
                css: 65.0,
            },
            ScalarPin {
                field: "computedWorld",
                index: 0,
                upstream: 105.0,
                css: 65.0,
            },
            ScalarPin {
                field: "computedRoot",
                index: 0,
                upstream: 105.0,
                css: 65.0,
            },
            ScalarPin {
                field: "computedLocal",
                index: 0,
                upstream: 105.0,
                css: 65.0,
            },
        ],
    },
    LayoutDivergence {
        fixture: "s18__flex-column__none__text__participant__tf-resizes-box",
        reason: "layout-diverges:CSS min-height:auto keeps the fitted text's unchanged unbounded height contribution 585 rather than the 80 column remainder; the empty Fixed sibling shrinks from 40 to zero, placing the three-quarter-height origin at 438.75. Fitted-font measurement remains the upstream protocol.",
        first_difference: "Subject.world[5]: 438.75 != 100",
        pins: &[
            ScalarPin {
                field: "world",
                index: 5,
                upstream: 100.0,
                css: 438.75,
            },
            ScalarPin {
                field: "computedWorld",
                index: 1,
                upstream: 100.0,
                css: 438.75,
            },
            ScalarPin {
                field: "computedRoot",
                index: 1,
                upstream: 100.0,
                css: 438.75,
            },
            ScalarPin {
                field: "computedLocal",
                index: 1,
                upstream: 100.0,
                css: 438.75,
            },
            ScalarPin {
                field: "computedSize",
                index: 1,
                upstream: 80.0,
                css: 585.0,
            },
        ],
    },
    LayoutDivergence {
        fixture: "s18__flex-column__none__text__participant__fill__y-hug__k-min-over-fill__tf-resizes-box",
        reason: "layout-diverges:CSS flex-shrink 1 lets the empty Fixed sibling give up height: its 40 basis shrinks to zero beside the subject's authored min-height 150 in a 120 column; the subject keeps its size and its origin shifts with the sibling.",
        first_difference: "Subject.world[5]: 112.5 != 152.5",
        pins: &[
            ScalarPin {
                field: "world",
                index: 5,
                upstream: 152.5,
                css: 112.5,
            },
            ScalarPin {
                field: "computedWorld",
                index: 1,
                upstream: 152.5,
                css: 112.5,
            },
            ScalarPin {
                field: "computedRoot",
                index: 1,
                upstream: 152.5,
                css: 112.5,
            },
            ScalarPin {
                field: "computedLocal",
                index: 1,
                upstream: 152.5,
                css: 112.5,
            },
        ],
    },
    LayoutDivergence {
        fixture: "s18__flex-row__none__text__participant__tf-keeps-box",
        reason: "layout-diverges:CSS min-width:auto keeps the fitted text's unchanged unbounded width contribution 326.3672 rather than the 140 row remainder; the empty Fixed sibling shrinks from 60 to zero, placing the quarter-width origin at 81.5918. Fitted-font measurement remains the upstream protocol.",
        first_difference: "Subject.world[4]: 81.5918 != 95",
        pins: &[
            ScalarPin {
                field: "world",
                index: 4,
                upstream: 95.0,
                css: 81.5918,
            },
            ScalarPin {
                field: "computedWorld",
                index: 0,
                upstream: 95.0,
                css: 81.5918,
            },
            ScalarPin {
                field: "computedRoot",
                index: 0,
                upstream: 95.0,
                css: 81.5918,
            },
            ScalarPin {
                field: "computedLocal",
                index: 0,
                upstream: 95.0,
                css: 81.5918,
            },
            ScalarPin {
                field: "computedSize",
                index: 0,
                upstream: 140.0,
                css: 326.3672,
            },
        ],
    },
    LayoutDivergence {
        fixture: "s18__flex-row__none__text__participant__fill__y-hug__k-min-over-fill__tf-keeps-box",
        reason: "layout-diverges:CSS flex-shrink 1 lets the empty Fixed sibling give up width: its 60 basis shrinks to 20 beside the subject's authored min-width 180 in a 200 row; the subject keeps its size and its origin shifts with the sibling.",
        first_difference: "Subject.world[4]: 65 != 105",
        pins: &[
            ScalarPin {
                field: "world",
                index: 4,
                upstream: 105.0,
                css: 65.0,
            },
            ScalarPin {
                field: "computedWorld",
                index: 0,
                upstream: 105.0,
                css: 65.0,
            },
            ScalarPin {
                field: "computedRoot",
                index: 0,
                upstream: 105.0,
                css: 65.0,
            },
            ScalarPin {
                field: "computedLocal",
                index: 0,
                upstream: 105.0,
                css: 65.0,
            },
        ],
    },
    LayoutDivergence {
        fixture: "s18__flex-column__none__text__participant__tf-keeps-box",
        reason: "layout-diverges:CSS min-height:auto keeps the fitted text's unchanged unbounded height contribution 585 rather than the 80 column remainder; the empty Fixed sibling shrinks from 40 to zero, placing the three-quarter-height origin at 438.75. Fitted-font measurement remains the upstream protocol.",
        first_difference: "Subject.world[5]: 438.75 != 100",
        pins: &[
            ScalarPin {
                field: "world",
                index: 5,
                upstream: 100.0,
                css: 438.75,
            },
            ScalarPin {
                field: "computedWorld",
                index: 1,
                upstream: 100.0,
                css: 438.75,
            },
            ScalarPin {
                field: "computedRoot",
                index: 1,
                upstream: 100.0,
                css: 438.75,
            },
            ScalarPin {
                field: "computedLocal",
                index: 1,
                upstream: 100.0,
                css: 438.75,
            },
            ScalarPin {
                field: "computedSize",
                index: 1,
                upstream: 80.0,
                css: 585.0,
            },
        ],
    },
    LayoutDivergence {
        fixture: "s18__flex-column__none__text__participant__fill__y-hug__k-min-over-fill__tf-keeps-box",
        reason: "layout-diverges:CSS flex-shrink 1 lets the empty Fixed sibling give up height: its 40 basis shrinks to zero beside the subject's authored min-height 150 in a 120 column; the subject keeps its size and its origin shifts with the sibling.",
        first_difference: "Subject.world[5]: 112.5 != 152.5",
        pins: &[
            ScalarPin {
                field: "world",
                index: 5,
                upstream: 152.5,
                css: 112.5,
            },
            ScalarPin {
                field: "computedWorld",
                index: 1,
                upstream: 152.5,
                css: 112.5,
            },
            ScalarPin {
                field: "computedRoot",
                index: 1,
                upstream: 152.5,
                css: 112.5,
            },
            ScalarPin {
                field: "computedLocal",
                index: 1,
                upstream: 152.5,
                css: 112.5,
            },
        ],
    },
    LayoutDivergence {
        fixture: "s19__flex-row__none__text__participant__wb-break-word",
        reason: "layout-diverges:CSS min-width:auto uses the true longest unbreakable run 263.14453 for Normal and BreakWord; emergency word breaks do not lower it to the 140 row remainder. The empty Fixed sibling shrinks from 60 to zero, so the quarter-width origin is 65.78613.",
        first_difference: "Subject.world[4]: 65.78613 != 95",
        pins: &[
            ScalarPin {
                field: "world",
                index: 4,
                upstream: 95.0,
                css: 65.78613,
            },
            ScalarPin {
                field: "computedWorld",
                index: 0,
                upstream: 95.0,
                css: 65.78613,
            },
            ScalarPin {
                field: "computedRoot",
                index: 0,
                upstream: 95.0,
                css: 65.78613,
            },
            ScalarPin {
                field: "computedLocal",
                index: 0,
                upstream: 95.0,
                css: 65.78613,
            },
            ScalarPin {
                field: "computedSize",
                index: 0,
                upstream: 140.0,
                css: 263.14453,
            },
        ],
    },
    LayoutDivergence {
        fixture: "s19__flex-row__none__text__participant__fill__y-hug__k-min-over-fill__wb-break-word",
        reason: "layout-diverges:CSS flex-shrink 1 lets the empty Fixed sibling give up width: its 60 basis shrinks to 20 beside the subject's authored min-width 180 in a 200 row; the subject keeps its size and its origin shifts with the sibling.",
        first_difference: "Subject.world[4]: 65 != 105",
        pins: &[
            ScalarPin {
                field: "world",
                index: 4,
                upstream: 105.0,
                css: 65.0,
            },
            ScalarPin {
                field: "computedWorld",
                index: 0,
                upstream: 105.0,
                css: 65.0,
            },
            ScalarPin {
                field: "computedRoot",
                index: 0,
                upstream: 105.0,
                css: 65.0,
            },
            ScalarPin {
                field: "computedLocal",
                index: 0,
                upstream: 105.0,
                css: 65.0,
            },
        ],
    },
    LayoutDivergence {
        fixture: "s19__flex-column__none__text__participant__wb-break-word",
        reason: "layout-diverges:CSS min-height:auto keeps the 87.75 text height produced by upstream BreakWord at the cross-axis width; the Fixed sibling shrinks from 40 to 32.25 instead of squeezing the text to 80. Its three-quarter-height origin is 98.0625.",
        first_difference: "Subject.world[5]: 98.0625 != 100",
        pins: &[
            ScalarPin {
                field: "world",
                index: 5,
                upstream: 100.0,
                css: 98.0625,
            },
            ScalarPin {
                field: "computedWorld",
                index: 1,
                upstream: 100.0,
                css: 98.0625,
            },
            ScalarPin {
                field: "computedRoot",
                index: 1,
                upstream: 100.0,
                css: 98.0625,
            },
            ScalarPin {
                field: "computedLocal",
                index: 1,
                upstream: 100.0,
                css: 98.0625,
            },
            ScalarPin {
                field: "computedSize",
                index: 1,
                upstream: 80.0,
                css: 87.75,
            },
        ],
    },
    LayoutDivergence {
        fixture: "s19__flex-column__none__text__participant__fill__y-hug__k-min-over-fill__wb-break-word",
        reason: "layout-diverges:CSS flex-shrink 1 lets the empty Fixed sibling give up height: its 40 basis shrinks to zero beside the subject's authored min-height 150 in a 120 column; the subject keeps its size and its origin shifts with the sibling.",
        first_difference: "Subject.world[5]: 112.5 != 152.5",
        pins: &[
            ScalarPin {
                field: "world",
                index: 5,
                upstream: 152.5,
                css: 112.5,
            },
            ScalarPin {
                field: "computedWorld",
                index: 1,
                upstream: 152.5,
                css: 112.5,
            },
            ScalarPin {
                field: "computedRoot",
                index: 1,
                upstream: 152.5,
                css: 112.5,
            },
            ScalarPin {
                field: "computedLocal",
                index: 1,
                upstream: 152.5,
                css: 112.5,
            },
        ],
    },
    LayoutDivergence {
        fixture: "s19__flex-row__none__text__participant__wb-normal",
        reason: "layout-diverges:CSS min-width:auto uses the true longest unbreakable run 263.14453 for Normal and BreakWord; emergency word breaks do not lower it to the 140 row remainder. The empty Fixed sibling shrinks from 60 to zero, so the quarter-width origin is 65.78613.",
        first_difference: "Subject.world[4]: 65.78613 != 95",
        pins: &[
            ScalarPin {
                field: "world",
                index: 4,
                upstream: 95.0,
                css: 65.78613,
            },
            ScalarPin {
                field: "computedWorld",
                index: 0,
                upstream: 95.0,
                css: 65.78613,
            },
            ScalarPin {
                field: "computedRoot",
                index: 0,
                upstream: 95.0,
                css: 65.78613,
            },
            ScalarPin {
                field: "computedLocal",
                index: 0,
                upstream: 95.0,
                css: 65.78613,
            },
            ScalarPin {
                field: "computedSize",
                index: 0,
                upstream: 140.0,
                css: 263.14453,
            },
        ],
    },
    LayoutDivergence {
        fixture: "s19__flex-row__none__text__participant__fill__y-hug__k-min-over-fill__wb-normal",
        reason: "layout-diverges:CSS flex-shrink 1 lets the empty Fixed sibling give up width: its 60 basis shrinks to 20 beside the subject's authored min-width 180 in a 200 row; the subject keeps its size and its origin shifts with the sibling.",
        first_difference: "Subject.world[4]: 65 != 105",
        pins: &[
            ScalarPin {
                field: "world",
                index: 4,
                upstream: 105.0,
                css: 65.0,
            },
            ScalarPin {
                field: "computedWorld",
                index: 0,
                upstream: 105.0,
                css: 65.0,
            },
            ScalarPin {
                field: "computedRoot",
                index: 0,
                upstream: 105.0,
                css: 65.0,
            },
            ScalarPin {
                field: "computedLocal",
                index: 0,
                upstream: 105.0,
                css: 65.0,
            },
        ],
    },
    LayoutDivergence {
        fixture: "s19__flex-column__none__text__participant__fill__y-hug__k-min-over-fill__wb-normal",
        reason: "layout-diverges:CSS flex-shrink 1 lets the empty Fixed sibling give up height: its 40 basis shrinks to zero beside the subject's authored min-height 150 in a 120 column; the subject keeps its size and its origin shifts with the sibling.",
        first_difference: "Subject.world[5]: 112.5 != 152.5",
        pins: &[
            ScalarPin {
                field: "world",
                index: 5,
                upstream: 152.5,
                css: 112.5,
            },
            ScalarPin {
                field: "computedWorld",
                index: 1,
                upstream: 152.5,
                css: 112.5,
            },
            ScalarPin {
                field: "computedRoot",
                index: 1,
                upstream: 152.5,
                css: 112.5,
            },
            ScalarPin {
                field: "computedLocal",
                index: 1,
                upstream: 152.5,
                css: 112.5,
            },
        ],
    },
    LayoutDivergence {
        fixture: "s19__flex-row__none__text__participant__fill__y-hug__k-min-over-fill__wb-break-all",
        reason: "layout-diverges:CSS flex-shrink 1 lets the empty Fixed sibling give up width: its 60 basis shrinks to 20 beside the subject's authored min-width 180 in a 200 row; the subject keeps its size and its origin shifts with the sibling.",
        first_difference: "Subject.world[4]: 65 != 105",
        pins: &[
            ScalarPin {
                field: "world",
                index: 4,
                upstream: 105.0,
                css: 65.0,
            },
            ScalarPin {
                field: "computedWorld",
                index: 0,
                upstream: 105.0,
                css: 65.0,
            },
            ScalarPin {
                field: "computedRoot",
                index: 0,
                upstream: 105.0,
                css: 65.0,
            },
            ScalarPin {
                field: "computedLocal",
                index: 0,
                upstream: 105.0,
                css: 65.0,
            },
        ],
    },
    LayoutDivergence {
        fixture: "s19__flex-column__none__text__participant__fill__y-hug__k-min-over-fill__wb-break-all",
        reason: "layout-diverges:CSS flex-shrink 1 lets the empty Fixed sibling give up height: its 40 basis shrinks to zero beside the subject's authored min-height 150 in a 120 column; the subject keeps its size and its origin shifts with the sibling.",
        first_difference: "Subject.world[5]: 112.5 != 152.5",
        pins: &[
            ScalarPin {
                field: "world",
                index: 5,
                upstream: 152.5,
                css: 112.5,
            },
            ScalarPin {
                field: "computedWorld",
                index: 1,
                upstream: 152.5,
                css: 112.5,
            },
            ScalarPin {
                field: "computedRoot",
                index: 1,
                upstream: 152.5,
                css: 112.5,
            },
            ScalarPin {
                field: "computedLocal",
                index: 1,
                upstream: 152.5,
                css: 112.5,
            },
        ],
    },
    // Previous CSS first difference: Subject.world[4]: 125 != 110
    LayoutDivergence {
        fixture: "s20__flex-row__none__text__participant__hug__k-max-over-host__over",
        reason: "layout-diverges:CSS min-width:auto keeps the Hug content floor at 260, capped by max-width where authored, above the 200 row offer; flex-shrink 1 reduces the empty Fixed sibling from its 60 basis to zero.",
        first_difference: "Subject.world[4]: 65 != 110",
        pins: &[
            // Previous CSS value: 125.0.
            ScalarPin {
                field: "world",
                index: 4,
                upstream: 110.0,
                css: 65.0,
            },
            // Previous CSS value: 125.0.
            ScalarPin {
                field: "computedWorld",
                index: 0,
                upstream: 110.0,
                css: 65.0,
            },
            // Previous CSS value: 125.0.
            ScalarPin {
                field: "computedRoot",
                index: 0,
                upstream: 110.0,
                css: 65.0,
            },
            // Previous CSS value: 125.0.
            ScalarPin {
                field: "computedLocal",
                index: 0,
                upstream: 110.0,
                css: 65.0,
            },
            ScalarPin {
                field: "computedSize",
                index: 0,
                upstream: 200.0,
                css: 260.0,
            },
        ],
    },
    LayoutDivergence {
        fixture: "s20__flex-row__none__text__participant__hug__k-max-over-host__over__unbounded",
        reason: "layout-diverges:CSS min-width:auto keeps the Hug content floor at 260, capped by max-width where authored, above the 200 row offer; flex-shrink 1 reduces the empty Fixed sibling from its 60 basis to zero. The subject already kept its intrinsic width when unbounded; the moved values follow the shrinking sibling.",
        first_difference: "Subject.world[4]: 65 != 125",
        pins: &[
            ScalarPin {
                field: "world",
                index: 4,
                upstream: 125.0,
                css: 65.0,
            },
            ScalarPin {
                field: "computedWorld",
                index: 0,
                upstream: 125.0,
                css: 65.0,
            },
            ScalarPin {
                field: "computedRoot",
                index: 0,
                upstream: 125.0,
                css: 65.0,
            },
            ScalarPin {
                field: "computedLocal",
                index: 0,
                upstream: 125.0,
                css: 65.0,
            },
        ],
    },
    // Previous CSS first difference: Subject.layoutBounds[2]: 260 != 200
    LayoutDivergence {
        fixture: "s20__flex-row__none__layout-component__hug__k-max-over-host__leaf-text__over",
        reason: "layout-diverges:CSS min-width:auto keeps the Hug content floor at 260, capped by max-width where authored, above the 200 row offer; flex-shrink 1 reduces the empty Fixed sibling from its 60 basis to zero.",
        first_difference: "Subject.world[4]: 0 != 60",
        pins: &[
            ScalarPin {
                field: "world",
                index: 4,
                upstream: 60.0,
                css: 0.0,
            },
            ScalarPin {
                field: "layoutBounds",
                index: 0,
                upstream: 60.0,
                css: 0.0,
            },
            ScalarPin {
                field: "layoutBounds",
                index: 2,
                upstream: 200.0,
                css: 260.0,
            },
            ScalarPin {
                field: "localBounds",
                index: 2,
                upstream: 200.0,
                css: 260.0,
            },
            ScalarPin {
                field: "worldBounds",
                index: 0,
                upstream: 60.0,
                css: 0.0,
            },
            ScalarPin {
                field: "worldBounds",
                index: 2,
                upstream: 200.0,
                css: 260.0,
            },
            ScalarPin {
                field: "computedWorld",
                index: 0,
                upstream: 60.0,
                css: 0.0,
            },
            ScalarPin {
                field: "computedRoot",
                index: 0,
                upstream: 60.0,
                css: 0.0,
            },
            ScalarPin {
                field: "computedLocal",
                index: 0,
                upstream: 60.0,
                css: 0.0,
            },
            ScalarPin {
                field: "computedSize",
                index: 0,
                upstream: 200.0,
                css: 260.0,
            },
        ],
    },
    LayoutDivergence {
        fixture: "s20__flex-row__none__layout-component__hug__k-max-over-host__leaf-text__over__unbounded",
        reason: "layout-diverges:CSS min-width:auto keeps the Hug content floor at 260, capped by max-width where authored, above the 200 row offer; flex-shrink 1 reduces the empty Fixed sibling from its 60 basis to zero. The subject already kept its intrinsic width when unbounded; the moved values follow the shrinking sibling.",
        first_difference: "Subject.world[4]: 0 != 60",
        pins: &[
            ScalarPin {
                field: "world",
                index: 4,
                upstream: 60.0,
                css: 0.0,
            },
            ScalarPin {
                field: "layoutBounds",
                index: 0,
                upstream: 60.0,
                css: 0.0,
            },
            ScalarPin {
                field: "worldBounds",
                index: 0,
                upstream: 60.0,
                css: 0.0,
            },
            ScalarPin {
                field: "computedWorld",
                index: 0,
                upstream: 60.0,
                css: 0.0,
            },
            ScalarPin {
                field: "computedRoot",
                index: 0,
                upstream: 60.0,
                css: 0.0,
            },
            ScalarPin {
                field: "computedLocal",
                index: 0,
                upstream: 60.0,
                css: 0.0,
            },
        ],
    },
    LayoutDivergence {
        fixture: "s20__flex-column__none__text__participant__hug__k-max-over-host__over",
        reason: "layout-diverges:CSS min-height:auto keeps the Hug content floor at 180, capped by max-height where authored, above the 120 column offer; flex-shrink 1 reduces the empty Fixed sibling from its 40 basis to zero. The existing fit-content inline cap remains 260.",
        first_difference: "Subject.world[4]: 65 != 50",
        pins: &[
            ScalarPin {
                field: "world",
                index: 4,
                upstream: 50.0,
                css: 65.0,
            },
            ScalarPin {
                field: "world",
                index: 5,
                upstream: 130.0,
                css: 135.0,
            },
            ScalarPin {
                field: "computedWorld",
                index: 0,
                upstream: 50.0,
                css: 65.0,
            },
            ScalarPin {
                field: "computedWorld",
                index: 1,
                upstream: 130.0,
                css: 135.0,
            },
            ScalarPin {
                field: "computedRoot",
                index: 0,
                upstream: 50.0,
                css: 65.0,
            },
            ScalarPin {
                field: "computedRoot",
                index: 1,
                upstream: 130.0,
                css: 135.0,
            },
            ScalarPin {
                field: "computedLocal",
                index: 0,
                upstream: 50.0,
                css: 65.0,
            },
            ScalarPin {
                field: "computedLocal",
                index: 1,
                upstream: 130.0,
                css: 135.0,
            },
            ScalarPin {
                field: "computedSize",
                index: 0,
                upstream: 200.0,
                css: 260.0,
            },
            ScalarPin {
                field: "computedSize",
                index: 1,
                upstream: 120.0,
                css: 180.0,
            },
        ],
    },
    LayoutDivergence {
        fixture: "s20__flex-column__none__text__participant__hug__k-max-over-host__over__unbounded",
        reason: "layout-diverges:CSS min-height:auto keeps the Hug content floor at 180, capped by max-height where authored, above the 120 column offer; flex-shrink 1 reduces the empty Fixed sibling from its 40 basis to zero. The subject already kept its intrinsic height when unbounded; the moved values follow the shrinking sibling. The existing fit-content inline cap remains 260.",
        first_difference: "Subject.world[5]: 135 != 175",
        pins: &[
            ScalarPin {
                field: "world",
                index: 5,
                upstream: 175.0,
                css: 135.0,
            },
            ScalarPin {
                field: "computedWorld",
                index: 1,
                upstream: 175.0,
                css: 135.0,
            },
            ScalarPin {
                field: "computedRoot",
                index: 1,
                upstream: 175.0,
                css: 135.0,
            },
            ScalarPin {
                field: "computedLocal",
                index: 1,
                upstream: 175.0,
                css: 135.0,
            },
        ],
    },
    // Previous CSS first difference: Subject.layoutBounds[2]: 260 != 200
    LayoutDivergence {
        fixture: "s20__flex-column__none__layout-component__hug__k-max-over-host__leaf-text__over",
        reason: "layout-diverges:CSS min-height:auto keeps the Hug content floor at 180, capped by max-height where authored, above the 120 column offer; flex-shrink 1 reduces the empty Fixed sibling from its 40 basis to zero. The existing fit-content inline cap remains 260.",
        first_difference: "Subject.world[5]: 0 != 40",
        pins: &[
            ScalarPin {
                field: "world",
                index: 5,
                upstream: 40.0,
                css: 0.0,
            },
            ScalarPin {
                field: "layoutBounds",
                index: 1,
                upstream: 40.0,
                css: 0.0,
            },
            ScalarPin {
                field: "layoutBounds",
                index: 2,
                upstream: 200.0,
                css: 260.0,
            },
            ScalarPin {
                field: "layoutBounds",
                index: 3,
                upstream: 120.0,
                css: 180.0,
            },
            ScalarPin {
                field: "localBounds",
                index: 2,
                upstream: 200.0,
                css: 260.0,
            },
            ScalarPin {
                field: "localBounds",
                index: 3,
                upstream: 120.0,
                css: 180.0,
            },
            ScalarPin {
                field: "worldBounds",
                index: 1,
                upstream: 40.0,
                css: 0.0,
            },
            ScalarPin {
                field: "worldBounds",
                index: 2,
                upstream: 200.0,
                css: 260.0,
            },
            ScalarPin {
                field: "worldBounds",
                index: 3,
                upstream: 120.0,
                css: 180.0,
            },
            ScalarPin {
                field: "computedWorld",
                index: 1,
                upstream: 40.0,
                css: 0.0,
            },
            ScalarPin {
                field: "computedRoot",
                index: 1,
                upstream: 40.0,
                css: 0.0,
            },
            ScalarPin {
                field: "computedLocal",
                index: 1,
                upstream: 40.0,
                css: 0.0,
            },
            ScalarPin {
                field: "computedSize",
                index: 0,
                upstream: 200.0,
                css: 260.0,
            },
            ScalarPin {
                field: "computedSize",
                index: 1,
                upstream: 120.0,
                css: 180.0,
            },
        ],
    },
    LayoutDivergence {
        fixture: "s20__flex-column__none__layout-component__hug__k-max-over-host__leaf-text__over__unbounded",
        reason: "layout-diverges:CSS min-height:auto keeps the Hug content floor at 180, capped by max-height where authored, above the 120 column offer; flex-shrink 1 reduces the empty Fixed sibling from its 40 basis to zero. The subject already kept its intrinsic height when unbounded; the moved values follow the shrinking sibling. The existing fit-content inline cap remains 260.",
        first_difference: "Subject.world[5]: 0 != 40",
        pins: &[
            ScalarPin {
                field: "world",
                index: 5,
                upstream: 40.0,
                css: 0.0,
            },
            ScalarPin {
                field: "layoutBounds",
                index: 1,
                upstream: 40.0,
                css: 0.0,
            },
            ScalarPin {
                field: "worldBounds",
                index: 1,
                upstream: 40.0,
                css: 0.0,
            },
            ScalarPin {
                field: "computedWorld",
                index: 1,
                upstream: 40.0,
                css: 0.0,
            },
            ScalarPin {
                field: "computedRoot",
                index: 1,
                upstream: 40.0,
                css: 0.0,
            },
            ScalarPin {
                field: "computedLocal",
                index: 1,
                upstream: 40.0,
                css: 0.0,
            },
        ],
    },
    LayoutDivergence {
        fixture: "s20__flex-row__none__text__participant__fill__y-hug__k-max-over-host__sibling-fills__over",
        reason: "layout-diverges:CSS min-width:auto floors the text at min(652.734375 min-content, 260 max-width) = 260 and the leading Fill sibling at its 50 content contribution, so they overflow the 200 row at widths 50 and 260. Fill fractions and bases are unchanged.",
        first_difference: "Subject.world[4]: 115 != 143.75",
        pins: &[
            ScalarPin {
                field: "world",
                index: 4,
                upstream: 143.75,
                css: 115.0,
            },
            ScalarPin {
                field: "computedWorld",
                index: 0,
                upstream: 143.75,
                css: 115.0,
            },
            ScalarPin {
                field: "computedRoot",
                index: 0,
                upstream: 143.75,
                css: 115.0,
            },
            ScalarPin {
                field: "computedLocal",
                index: 0,
                upstream: 143.75,
                css: 115.0,
            },
            ScalarPin {
                field: "computedSize",
                index: 0,
                upstream: 75.0,
                css: 260.0,
            },
        ],
    },
    LayoutDivergence {
        fixture: "s20__flex-row__none__text__participant__fill__y-hug__k-max-over-host__sibling-fills__over__unbounded",
        reason: "layout-diverges:CSS min-width:auto floors the text at min(652.734375 min-content, 260 max-width) = 260 and the leading Fill sibling at its 50 content contribution, so they overflow the 200 row at widths 50 and 260. Fill fractions and bases are unchanged. Width is Fill, so hugUnbounded only changes the height mode.",
        first_difference: "Subject.world[4]: 115 != 143.75",
        pins: &[
            ScalarPin {
                field: "world",
                index: 4,
                upstream: 143.75,
                css: 115.0,
            },
            ScalarPin {
                field: "computedWorld",
                index: 0,
                upstream: 143.75,
                css: 115.0,
            },
            ScalarPin {
                field: "computedRoot",
                index: 0,
                upstream: 143.75,
                css: 115.0,
            },
            ScalarPin {
                field: "computedLocal",
                index: 0,
                upstream: 143.75,
                css: 115.0,
            },
            ScalarPin {
                field: "computedSize",
                index: 0,
                upstream: 75.0,
                css: 260.0,
            },
        ],
    },
    // Previous CSS first difference: Subject.world[4]: 14.230129 != 40
    LayoutDivergence {
        fixture: "s20__flex-row__none__layout-component__fill__y-hug__k-max-over-host__sibling-fills__leaf-text__over",
        reason: "layout-diverges:CSS min-width:auto floors the text at min(652.734375 min-content, 260 max-width) = 260 and the leading Fill sibling at its 50 content contribution, so they overflow the 200 row at widths 50 and 260. Fill fractions and bases are unchanged.",
        first_difference: "Subject.world[4]: 50 != 40",
        pins: &[
            // Previous CSS value: 14.230129.
            ScalarPin {
                field: "world",
                index: 4,
                upstream: 40.0,
                css: 50.0,
            },
            // Previous CSS value: 14.230129.
            ScalarPin {
                field: "layoutBounds",
                index: 0,
                upstream: 40.0,
                css: 50.0,
            },
            // Previous CSS value: 185.7699.
            ScalarPin {
                field: "layoutBounds",
                index: 2,
                upstream: 160.0,
                css: 260.0,
            },
            // Previous CSS value: 185.7699.
            ScalarPin {
                field: "localBounds",
                index: 2,
                upstream: 160.0,
                css: 260.0,
            },
            // Previous CSS value: 14.230129.
            ScalarPin {
                field: "worldBounds",
                index: 0,
                upstream: 40.0,
                css: 50.0,
            },
            // Previous CSS value: 185.7699.
            ScalarPin {
                field: "worldBounds",
                index: 2,
                upstream: 160.0,
                css: 260.0,
            },
            // Previous CSS value: 14.230129.
            ScalarPin {
                field: "computedWorld",
                index: 0,
                upstream: 40.0,
                css: 50.0,
            },
            // Previous CSS value: 14.230129.
            ScalarPin {
                field: "computedRoot",
                index: 0,
                upstream: 40.0,
                css: 50.0,
            },
            // Previous CSS value: 14.230129.
            ScalarPin {
                field: "computedLocal",
                index: 0,
                upstream: 40.0,
                css: 50.0,
            },
            // Previous CSS value: 185.7699.
            ScalarPin {
                field: "computedSize",
                index: 0,
                upstream: 160.0,
                css: 260.0,
            },
        ],
    },
    // Previous CSS first difference: Subject.world[4]: 14.230129 != 40
    LayoutDivergence {
        fixture: "s20__flex-row__none__layout-component__fill__y-hug__k-max-over-host__sibling-fills__leaf-text__over__unbounded",
        reason: "layout-diverges:CSS min-width:auto floors the text at min(652.734375 min-content, 260 max-width) = 260 and the leading Fill sibling at its 50 content contribution, so they overflow the 200 row at widths 50 and 260. Fill fractions and bases are unchanged. Width is Fill, so hugUnbounded only changes the height mode.",
        first_difference: "Subject.world[4]: 50 != 40",
        pins: &[
            // Previous CSS value: 14.230129.
            ScalarPin {
                field: "world",
                index: 4,
                upstream: 40.0,
                css: 50.0,
            },
            // Previous CSS value: 14.230129.
            ScalarPin {
                field: "layoutBounds",
                index: 0,
                upstream: 40.0,
                css: 50.0,
            },
            // Previous CSS value: 185.7699.
            ScalarPin {
                field: "layoutBounds",
                index: 2,
                upstream: 160.0,
                css: 260.0,
            },
            // Previous CSS value: 185.7699.
            ScalarPin {
                field: "localBounds",
                index: 2,
                upstream: 160.0,
                css: 260.0,
            },
            // Previous CSS value: 14.230129.
            ScalarPin {
                field: "worldBounds",
                index: 0,
                upstream: 40.0,
                css: 50.0,
            },
            // Previous CSS value: 185.7699.
            ScalarPin {
                field: "worldBounds",
                index: 2,
                upstream: 160.0,
                css: 260.0,
            },
            // Previous CSS value: 14.230129.
            ScalarPin {
                field: "computedWorld",
                index: 0,
                upstream: 40.0,
                css: 50.0,
            },
            // Previous CSS value: 14.230129.
            ScalarPin {
                field: "computedRoot",
                index: 0,
                upstream: 40.0,
                css: 50.0,
            },
            // Previous CSS value: 14.230129.
            ScalarPin {
                field: "computedLocal",
                index: 0,
                upstream: 40.0,
                css: 50.0,
            },
            // Previous CSS value: 185.7699.
            ScalarPin {
                field: "computedSize",
                index: 0,
                upstream: 160.0,
                css: 260.0,
            },
        ],
    },
    LayoutDivergence {
        fixture: "s21__grid__none__layout-component__hug__k-max-over-host__auto__leaf-text__over__track-row-fr",
        reason: "layout-diverges:CSS fit-content keeps the unbreakable minimum above the 140 grid cell offer, then max-width 260 caps the box; Yoga clamps it to the offer.",
        first_difference: "Subject.layoutBounds[2]: 260 != 140",
        pins: &[
            ScalarPin {
                field: "layoutBounds",
                index: 2,
                upstream: 140.0,
                css: 260.0,
            },
            ScalarPin {
                field: "localBounds",
                index: 2,
                upstream: 140.0,
                css: 260.0,
            },
            ScalarPin {
                field: "worldBounds",
                index: 2,
                upstream: 140.0,
                css: 260.0,
            },
            ScalarPin {
                field: "computedSize",
                index: 0,
                upstream: 140.0,
                css: 260.0,
            },
        ],
    },
    LayoutDivergence {
        fixture: "s21__grid__none__layout-component__hug__k-max-over-host__auto__leaf-text__over__track-row-auto",
        reason: "layout-diverges:CSS fit-content keeps the unbreakable minimum above the 140 grid cell offer, then max-width 260 caps the box; Yoga clamps it to the offer.",
        first_difference: "Subject.layoutBounds[2]: 260 != 140",
        pins: &[
            ScalarPin {
                field: "layoutBounds",
                index: 2,
                upstream: 140.0,
                css: 260.0,
            },
            ScalarPin {
                field: "localBounds",
                index: 2,
                upstream: 140.0,
                css: 260.0,
            },
            ScalarPin {
                field: "worldBounds",
                index: 2,
                upstream: 140.0,
                css: 260.0,
            },
            ScalarPin {
                field: "computedSize",
                index: 0,
                upstream: 140.0,
                css: 260.0,
            },
        ],
    },
    LayoutDivergence {
        fixture: "s21__grid__none__text__participant__hug__k-max-over-host__auto__over__track-row-fr",
        reason: "layout-diverges:CSS fit-content keeps the unbreakable minimum above the 140 grid cell offer, then max-width 260 caps the box; Yoga clamps it to the offer.",
        first_difference: "Subject.world[4]: 125 != 95",
        pins: &[
            ScalarPin {
                field: "world",
                index: 4,
                upstream: 95.0,
                css: 125.0,
            },
            ScalarPin {
                field: "computedWorld",
                index: 0,
                upstream: 95.0,
                css: 125.0,
            },
            ScalarPin {
                field: "computedRoot",
                index: 0,
                upstream: 95.0,
                css: 125.0,
            },
            ScalarPin {
                field: "computedLocal",
                index: 0,
                upstream: 95.0,
                css: 125.0,
            },
            ScalarPin {
                field: "computedSize",
                index: 0,
                upstream: 140.0,
                css: 260.0,
            },
        ],
    },
    LayoutDivergence {
        fixture: "s21__grid__none__text__participant__hug__k-max-over-host__auto__over__track-row-auto",
        reason: "layout-diverges:CSS fit-content keeps the unbreakable minimum above the 140 grid cell offer, then max-width 260 caps the box; Yoga clamps it to the offer.",
        first_difference: "Subject.world[4]: 125 != 95",
        pins: &[
            ScalarPin {
                field: "world",
                index: 4,
                upstream: 95.0,
                css: 125.0,
            },
            ScalarPin {
                field: "computedWorld",
                index: 0,
                upstream: 95.0,
                css: 125.0,
            },
            ScalarPin {
                field: "computedRoot",
                index: 0,
                upstream: 95.0,
                css: 125.0,
            },
            ScalarPin {
                field: "computedLocal",
                index: 0,
                upstream: 95.0,
                css: 125.0,
            },
            ScalarPin {
                field: "computedSize",
                index: 0,
                upstream: 140.0,
                css: 260.0,
            },
        ],
    },
    LayoutDivergence {
        fixture: "s1__flex-row__none__image__participant",
        reason: "layout-diverges:CSS min-width:auto floors the Fill image at its natural width 442 rather than the 140 remainder of the 200 row; the empty Fixed sibling shrinks from 60 to zero, and image scale becomes 1 on this axis.",
        first_difference: "Subject.world[0]: 1 != 0.3167",
        pins: &[
            ScalarPin {
                field: "world",
                index: 0,
                upstream: 0.3167,
                css: 1.0,
            },
            ScalarPin {
                field: "world",
                index: 4,
                upstream: 130.0,
                css: 221.0,
            },
            ScalarPin {
                field: "computedWorld",
                index: 0,
                upstream: 130.0,
                css: 221.0,
            },
            ScalarPin {
                field: "computedRoot",
                index: 0,
                upstream: 130.0,
                css: 221.0,
            },
            ScalarPin {
                field: "computedLocal",
                index: 0,
                upstream: 130.0,
                css: 221.0,
            },
            ScalarPin {
                field: "computedSize",
                index: 0,
                upstream: 140.0,
                css: 442.0,
            },
        ],
    },
    LayoutDivergence {
        fixture: "s1__flex-row__group__image__participant",
        reason: "layout-diverges:CSS min-width:auto floors the Fill image at its natural width 442 rather than the 140 remainder of the 200 row; the empty Fixed sibling shrinks from 60 to zero, and image scale becomes 1 on this axis.",
        first_difference: "Subject.world[0]: 1 != 0.3167",
        pins: &[
            ScalarPin {
                field: "world",
                index: 0,
                upstream: 0.3167,
                css: 1.0,
            },
            ScalarPin {
                field: "world",
                index: 4,
                upstream: 130.0,
                css: 221.0,
            },
            ScalarPin {
                field: "computedWorld",
                index: 0,
                upstream: 130.0,
                css: 221.0,
            },
            ScalarPin {
                field: "computedRoot",
                index: 0,
                upstream: 130.0,
                css: 221.0,
            },
            ScalarPin {
                field: "computedLocal",
                index: 0,
                upstream: 130.0,
                css: 221.0,
            },
            ScalarPin {
                field: "computedSize",
                index: 0,
                upstream: 140.0,
                css: 442.0,
            },
        ],
    },
    LayoutDivergence {
        fixture: "s1__flex-row__solo-active__image__participant",
        reason: "layout-diverges:CSS min-width:auto floors the Fill image at its natural width 442 rather than the 140 remainder of the 200 row; the empty Fixed sibling shrinks from 60 to zero, and image scale becomes 1 on this axis.",
        first_difference: "Subject.world[0]: 1 != 0.3167",
        pins: &[
            ScalarPin {
                field: "world",
                index: 0,
                upstream: 0.3167,
                css: 1.0,
            },
            ScalarPin {
                field: "world",
                index: 4,
                upstream: 130.0,
                css: 221.0,
            },
            ScalarPin {
                field: "computedWorld",
                index: 0,
                upstream: 130.0,
                css: 221.0,
            },
            ScalarPin {
                field: "computedRoot",
                index: 0,
                upstream: 130.0,
                css: 221.0,
            },
            ScalarPin {
                field: "computedLocal",
                index: 0,
                upstream: 130.0,
                css: 221.0,
            },
            ScalarPin {
                field: "computedSize",
                index: 0,
                upstream: 140.0,
                css: 442.0,
            },
        ],
    },
    LayoutDivergence {
        fixture: "s1__flex-row__group-in-solo__image__participant",
        reason: "layout-diverges:CSS min-width:auto floors the Fill image at its natural width 442 rather than the 140 remainder of the 200 row; the empty Fixed sibling shrinks from 60 to zero, and image scale becomes 1 on this axis.",
        first_difference: "Subject.world[0]: 1 != 0.3167",
        pins: &[
            ScalarPin {
                field: "world",
                index: 0,
                upstream: 0.3167,
                css: 1.0,
            },
            ScalarPin {
                field: "world",
                index: 4,
                upstream: 130.0,
                css: 221.0,
            },
            ScalarPin {
                field: "computedWorld",
                index: 0,
                upstream: 130.0,
                css: 221.0,
            },
            ScalarPin {
                field: "computedRoot",
                index: 0,
                upstream: 130.0,
                css: 221.0,
            },
            ScalarPin {
                field: "computedLocal",
                index: 0,
                upstream: 130.0,
                css: 221.0,
            },
            ScalarPin {
                field: "computedSize",
                index: 0,
                upstream: 140.0,
                css: 442.0,
            },
        ],
    },
    LayoutDivergence {
        fixture: "s1__flex-column__none__image__participant",
        reason: "layout-diverges:CSS min-height:auto floors the Fill image at its natural height 412 rather than the 80 remainder of the 120 column; the empty Fixed sibling shrinks from 40 to zero, and image scale becomes 1 on this axis.",
        first_difference: "Subject.world[3]: 1 != 0.1942",
        pins: &[
            ScalarPin {
                field: "world",
                index: 3,
                upstream: 0.1942,
                css: 1.0,
            },
            ScalarPin {
                field: "world",
                index: 5,
                upstream: 80.0,
                css: 206.0,
            },
            ScalarPin {
                field: "computedWorld",
                index: 1,
                upstream: 80.0,
                css: 206.0,
            },
            ScalarPin {
                field: "computedRoot",
                index: 1,
                upstream: 80.0,
                css: 206.0,
            },
            ScalarPin {
                field: "computedLocal",
                index: 1,
                upstream: 80.0,
                css: 206.0,
            },
            ScalarPin {
                field: "computedSize",
                index: 1,
                upstream: 80.0,
                css: 412.0,
            },
        ],
    },
    LayoutDivergence {
        fixture: "s1__flex-column__group__image__participant",
        reason: "layout-diverges:CSS min-height:auto floors the Fill image at its natural height 412 rather than the 80 remainder of the 120 column; the empty Fixed sibling shrinks from 40 to zero, and image scale becomes 1 on this axis.",
        first_difference: "Subject.world[3]: 1 != 0.1942",
        pins: &[
            ScalarPin {
                field: "world",
                index: 3,
                upstream: 0.1942,
                css: 1.0,
            },
            ScalarPin {
                field: "world",
                index: 5,
                upstream: 80.0,
                css: 206.0,
            },
            ScalarPin {
                field: "computedWorld",
                index: 1,
                upstream: 80.0,
                css: 206.0,
            },
            ScalarPin {
                field: "computedRoot",
                index: 1,
                upstream: 80.0,
                css: 206.0,
            },
            ScalarPin {
                field: "computedLocal",
                index: 1,
                upstream: 80.0,
                css: 206.0,
            },
            ScalarPin {
                field: "computedSize",
                index: 1,
                upstream: 80.0,
                css: 412.0,
            },
        ],
    },
    LayoutDivergence {
        fixture: "s1__flex-column__solo-active__image__participant",
        reason: "layout-diverges:CSS min-height:auto floors the Fill image at its natural height 412 rather than the 80 remainder of the 120 column; the empty Fixed sibling shrinks from 40 to zero, and image scale becomes 1 on this axis.",
        first_difference: "Subject.world[3]: 1 != 0.1942",
        pins: &[
            ScalarPin {
                field: "world",
                index: 3,
                upstream: 0.1942,
                css: 1.0,
            },
            ScalarPin {
                field: "world",
                index: 5,
                upstream: 80.0,
                css: 206.0,
            },
            ScalarPin {
                field: "computedWorld",
                index: 1,
                upstream: 80.0,
                css: 206.0,
            },
            ScalarPin {
                field: "computedRoot",
                index: 1,
                upstream: 80.0,
                css: 206.0,
            },
            ScalarPin {
                field: "computedLocal",
                index: 1,
                upstream: 80.0,
                css: 206.0,
            },
            ScalarPin {
                field: "computedSize",
                index: 1,
                upstream: 80.0,
                css: 412.0,
            },
        ],
    },
    LayoutDivergence {
        fixture: "s1__flex-column__group-in-solo__image__participant",
        reason: "layout-diverges:CSS min-height:auto floors the Fill image at its natural height 412 rather than the 80 remainder of the 120 column; the empty Fixed sibling shrinks from 40 to zero, and image scale becomes 1 on this axis.",
        first_difference: "Subject.world[3]: 1 != 0.1942",
        pins: &[
            ScalarPin {
                field: "world",
                index: 3,
                upstream: 0.1942,
                css: 1.0,
            },
            ScalarPin {
                field: "world",
                index: 5,
                upstream: 80.0,
                css: 206.0,
            },
            ScalarPin {
                field: "computedWorld",
                index: 1,
                upstream: 80.0,
                css: 206.0,
            },
            ScalarPin {
                field: "computedRoot",
                index: 1,
                upstream: 80.0,
                css: 206.0,
            },
            ScalarPin {
                field: "computedLocal",
                index: 1,
                upstream: 80.0,
                css: 206.0,
            },
            ScalarPin {
                field: "computedSize",
                index: 1,
                upstream: 80.0,
                css: 412.0,
            },
        ],
    },
    LayoutDivergence {
        fixture: "s2__flex-row__none__image__participant__c-translation",
        reason: "layout-diverges:CSS min-width:auto floors the Fill image at its natural width 442 rather than the 140 remainder of the 200 row; the empty Fixed sibling shrinks from 60 to zero, and image scale becomes 1 on this axis. The authored constraint still owns the final position.",
        first_difference: "Subject.world[0]: 1 != 0.3167",
        pins: &[
            ScalarPin {
                field: "world",
                index: 0,
                upstream: 0.3167,
                css: 1.0,
            },
            ScalarPin {
                field: "computedSize",
                index: 0,
                upstream: 140.0,
                css: 442.0,
            },
        ],
    },
    LayoutDivergence {
        fixture: "s2__flex-row__none__image__participant__c-follow-path",
        reason: "layout-diverges:CSS min-width:auto floors the Fill image at its natural width 442 rather than the 140 remainder of the 200 row; the empty Fixed sibling shrinks from 60 to zero, and image scale becomes 1 on this axis. The authored constraint still owns the final position.",
        first_difference: "Subject.world[0]: 1 != 0.3167",
        pins: &[
            ScalarPin {
                field: "world",
                index: 0,
                upstream: 0.3167,
                css: 1.0,
            },
            ScalarPin {
                field: "computedSize",
                index: 0,
                upstream: 140.0,
                css: 442.0,
            },
        ],
    },
    LayoutDivergence {
        fixture: "s2__flex-row__group__image__participant__c-translation",
        reason: "layout-diverges:CSS min-width:auto floors the Fill image at its natural width 442 rather than the 140 remainder of the 200 row; the empty Fixed sibling shrinks from 60 to zero, and image scale becomes 1 on this axis. The authored constraint still owns the final position.",
        first_difference: "Subject.world[0]: 1 != 0.3167",
        pins: &[
            ScalarPin {
                field: "world",
                index: 0,
                upstream: 0.3167,
                css: 1.0,
            },
            ScalarPin {
                field: "computedSize",
                index: 0,
                upstream: 140.0,
                css: 442.0,
            },
        ],
    },
    LayoutDivergence {
        fixture: "s2__flex-row__group__image__participant__c-follow-path",
        reason: "layout-diverges:CSS min-width:auto floors the Fill image at its natural width 442 rather than the 140 remainder of the 200 row; the empty Fixed sibling shrinks from 60 to zero, and image scale becomes 1 on this axis. The authored constraint still owns the final position.",
        first_difference: "Subject.world[0]: 1 != 0.3167",
        pins: &[
            ScalarPin {
                field: "world",
                index: 0,
                upstream: 0.3167,
                css: 1.0,
            },
            ScalarPin {
                field: "computedSize",
                index: 0,
                upstream: 140.0,
                css: 442.0,
            },
        ],
    },
    LayoutDivergence {
        fixture: "s2__flex-column__none__image__participant__c-translation",
        reason: "layout-diverges:CSS min-height:auto floors the Fill image at its natural height 412 rather than the 80 remainder of the 120 column; the empty Fixed sibling shrinks from 40 to zero, and image scale becomes 1 on this axis. The authored constraint still owns the final position.",
        first_difference: "Subject.world[3]: 1 != 0.1942",
        pins: &[
            ScalarPin {
                field: "world",
                index: 3,
                upstream: 0.1942,
                css: 1.0,
            },
            ScalarPin {
                field: "computedSize",
                index: 1,
                upstream: 80.0,
                css: 412.0,
            },
        ],
    },
    LayoutDivergence {
        fixture: "s2__flex-column__none__image__participant__c-follow-path",
        reason: "layout-diverges:CSS min-height:auto floors the Fill image at its natural height 412 rather than the 80 remainder of the 120 column; the empty Fixed sibling shrinks from 40 to zero, and image scale becomes 1 on this axis. The authored constraint still owns the final position.",
        first_difference: "Subject.world[3]: 1 != 0.1942",
        pins: &[
            ScalarPin {
                field: "world",
                index: 3,
                upstream: 0.1942,
                css: 1.0,
            },
            ScalarPin {
                field: "computedSize",
                index: 1,
                upstream: 80.0,
                css: 412.0,
            },
        ],
    },
    LayoutDivergence {
        fixture: "s2__flex-column__group__image__participant__c-translation",
        reason: "layout-diverges:CSS min-height:auto floors the Fill image at its natural height 412 rather than the 80 remainder of the 120 column; the empty Fixed sibling shrinks from 40 to zero, and image scale becomes 1 on this axis. The authored constraint still owns the final position.",
        first_difference: "Subject.world[3]: 1 != 0.1942",
        pins: &[
            ScalarPin {
                field: "world",
                index: 3,
                upstream: 0.1942,
                css: 1.0,
            },
            ScalarPin {
                field: "computedSize",
                index: 1,
                upstream: 80.0,
                css: 412.0,
            },
        ],
    },
    LayoutDivergence {
        fixture: "s2__flex-column__group__image__participant__c-follow-path",
        reason: "layout-diverges:CSS min-height:auto floors the Fill image at its natural height 412 rather than the 80 remainder of the 120 column; the empty Fixed sibling shrinks from 40 to zero, and image scale becomes 1 on this axis. The authored constraint still owns the final position.",
        first_difference: "Subject.world[3]: 1 != 0.1942",
        pins: &[
            ScalarPin {
                field: "world",
                index: 3,
                upstream: 0.1942,
                css: 1.0,
            },
            ScalarPin {
                field: "computedSize",
                index: 1,
                upstream: 80.0,
                css: 412.0,
            },
        ],
    },
    LayoutDivergence {
        fixture: "s7__flex-row__none__image__participant__k-min-over-fill",
        reason: "layout-diverges:CSS flex-shrink 1 lets the empty Fixed sibling give up width: its 60 basis shrinks to 20 beside the subject's authored min-width 180 in a 200 row; the subject keeps its size and its origin shifts with the sibling.",
        first_difference: "Subject.world[4]: 110 != 150",
        pins: &[
            ScalarPin {
                field: "world",
                index: 4,
                upstream: 150.0,
                css: 110.0,
            },
            ScalarPin {
                field: "computedWorld",
                index: 0,
                upstream: 150.0,
                css: 110.0,
            },
            ScalarPin {
                field: "computedRoot",
                index: 0,
                upstream: 150.0,
                css: 110.0,
            },
            ScalarPin {
                field: "computedLocal",
                index: 0,
                upstream: 150.0,
                css: 110.0,
            },
        ],
    },
    LayoutDivergence {
        fixture: "s7__flex-column__none__image__participant__k-min-over-fill",
        reason: "layout-diverges:CSS flex-shrink 1 lets the empty Fixed sibling give up height: its 40 basis shrinks to zero beside the subject's authored min-height 150 in a 120 column; the subject keeps its size and its origin shifts with the sibling.",
        first_difference: "Subject.world[5]: 75 != 115",
        pins: &[
            ScalarPin {
                field: "world",
                index: 5,
                upstream: 115.0,
                css: 75.0,
            },
            ScalarPin {
                field: "computedWorld",
                index: 1,
                upstream: 115.0,
                css: 75.0,
            },
            ScalarPin {
                field: "computedRoot",
                index: 1,
                upstream: 115.0,
                css: 75.0,
            },
            ScalarPin {
                field: "computedLocal",
                index: 1,
                upstream: 115.0,
                css: 75.0,
            },
        ],
    },
];

pub fn divergence(fixture: &str) -> Option<&'static LayoutDivergence> {
    LAYOUT_DIVERGENCES
        .iter()
        .find(|entry| entry.fixture == fixture)
}

pub fn expected(
    divergence: Option<&LayoutDivergence>,
    component: &str,
    field: &str,
    index: usize,
    upstream: f32,
) -> f32 {
    let Some(entry) = divergence.filter(|_| component == "Subject") else {
        return upstream;
    };
    let Some(pin) = entry
        .pins
        .iter()
        .find(|pin| pin.field == field && pin.index == index)
    else {
        return upstream;
    };
    assert_eq!(
        upstream.to_bits(),
        pin.upstream.to_bits(),
        "upstream matrix expectation moved: {} {component}.{field}[{index}]",
        entry.fixture
    );
    pin.css
}
