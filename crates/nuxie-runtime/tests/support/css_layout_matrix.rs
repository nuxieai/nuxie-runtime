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

// Restored to the original exact C++ comparison after the minimum-contract fix:
// s20__flex-row__none__shape__participant__fill__y-hug__sibling-fills__over
// s20__flex-row__none__shape__participant__fill__y-hug__sibling-fills__over__unbounded
// s18__flex-row__none__text__participant__tf-resizes-box
// s18__flex-column__none__text__participant__tf-resizes-box
// s18__flex-row__none__text__participant__tf-keeps-box
// s18__flex-column__none__text__participant__tf-keeps-box
// s1__flex-row__none__image__participant
// s1__flex-row__group__image__participant
// s1__flex-row__solo-active__image__participant
// s1__flex-row__group-in-solo__image__participant
// s2__flex-row__none__image__participant__c-translation
// s2__flex-row__none__image__participant__c-follow-path
// s2__flex-row__group__image__participant__c-translation
// s2__flex-row__group__image__participant__c-follow-path

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
    // Previous reviewed first difference: Subject.world[4]: 0 != 60
    LayoutDivergence {
        fixture: "s20__flex-row__none__layout-component__hug__leaf-content__over",
        reason: "layout-diverges:Shape measurement has no minimum contract and keeps zero automatic minimum. CSS flex-shrink 1 shares the 200-point row in proportion to each item basis: the 60-point Fixed sibling and the Hug shape. The subject resolves to 143.88489 points wide. Rive controlSize writes the solved primitive size back into its measurement; the retained second solve uses that smaller basis.",
        first_difference: "Subject.world[4]: 56.115105 != 60",
        pins: &[
            // Previous reviewed CSS value: 0.0.
            ScalarPin {
                field: "world",
                index: 4,
                upstream: 60.0,
                css: 56.115105,
            },
            // Previous reviewed CSS value: 0.0.
            ScalarPin {
                field: "layoutBounds",
                index: 0,
                upstream: 60.0,
                css: 56.115105,
            },
            // Previous reviewed CSS value: 260.0.
            ScalarPin {
                field: "layoutBounds",
                index: 2,
                upstream: 200.0,
                css: 143.88489,
            },
            // Previous reviewed CSS value: 260.0.
            ScalarPin {
                field: "localBounds",
                index: 2,
                upstream: 200.0,
                css: 143.88489,
            },
            // Previous reviewed CSS value: 0.0.
            ScalarPin {
                field: "worldBounds",
                index: 0,
                upstream: 60.0,
                css: 56.115105,
            },
            // Previous reviewed CSS value: 260.0.
            ScalarPin {
                field: "worldBounds",
                index: 2,
                upstream: 200.0,
                css: 143.88489,
            },
            // Previous reviewed CSS value: 0.0.
            ScalarPin {
                field: "computedWorld",
                index: 0,
                upstream: 60.0,
                css: 56.115105,
            },
            // Previous reviewed CSS value: 0.0.
            ScalarPin {
                field: "computedRoot",
                index: 0,
                upstream: 60.0,
                css: 56.115105,
            },
            // Previous reviewed CSS value: 0.0.
            ScalarPin {
                field: "computedLocal",
                index: 0,
                upstream: 60.0,
                css: 56.115105,
            },
            // Previous reviewed CSS value: 260.0.
            ScalarPin {
                field: "computedSize",
                index: 0,
                upstream: 200.0,
                css: 143.88489,
            },
        ],
    },
    // Previous reviewed first difference: Subject.world[4]: 0 != 60
    LayoutDivergence {
        fixture: "s20__flex-row__none__layout-component__hug__leaf-content__over__unbounded",
        reason: "layout-diverges:Shape measurement has no minimum contract and keeps zero automatic minimum. CSS flex-shrink 1 shares the 200-point row in proportion to each item basis: the 60-point Fixed sibling and the Hug shape. The subject resolves to 146.06741 points wide. Rive controlSize writes the solved primitive size back into its measurement; the retained second solve uses that smaller basis.",
        first_difference: "Subject.world[4]: 53.932583 != 60",
        pins: &[
            // Previous reviewed CSS value: 0.0.
            ScalarPin {
                field: "world",
                index: 4,
                upstream: 60.0,
                css: 53.932583,
            },
            // Previous reviewed CSS value: 0.0.
            ScalarPin {
                field: "layoutBounds",
                index: 0,
                upstream: 60.0,
                css: 53.932583,
            },
            ScalarPin {
                field: "layoutBounds",
                index: 2,
                upstream: 260.0,
                css: 146.06741,
            },
            ScalarPin {
                field: "localBounds",
                index: 2,
                upstream: 260.0,
                css: 146.06741,
            },
            // Previous reviewed CSS value: 0.0.
            ScalarPin {
                field: "worldBounds",
                index: 0,
                upstream: 60.0,
                css: 53.932583,
            },
            ScalarPin {
                field: "worldBounds",
                index: 2,
                upstream: 260.0,
                css: 146.06741,
            },
            // Previous reviewed CSS value: 0.0.
            ScalarPin {
                field: "computedWorld",
                index: 0,
                upstream: 60.0,
                css: 53.932583,
            },
            // Previous reviewed CSS value: 0.0.
            ScalarPin {
                field: "computedRoot",
                index: 0,
                upstream: 60.0,
                css: 53.932583,
            },
            // Previous reviewed CSS value: 0.0.
            ScalarPin {
                field: "computedLocal",
                index: 0,
                upstream: 60.0,
                css: 53.932583,
            },
            ScalarPin {
                field: "computedSize",
                index: 0,
                upstream: 260.0,
                css: 146.06741,
            },
        ],
    },
    // Previous reviewed first difference: Subject.world[0]: 1 != 0.7692
    LayoutDivergence {
        fixture: "s20__flex-row__none__shape__participant__hug__over",
        reason: "layout-diverges:Shape measurement has no minimum contract and keeps zero automatic minimum. CSS flex-shrink 1 shares the 200-point row in proportion to each item basis: the 60-point Fixed sibling and the Hug shape. The subject resolves to 153.84616 points wide. The participant retains the natural primitive basis.",
        first_difference: "Subject.world[0]: 0.591716 != 0.7692",
        pins: &[
            // Previous reviewed CSS value: 1.0.
            ScalarPin {
                field: "world",
                index: 0,
                upstream: 0.7692,
                css: 0.591716,
            },
            // Previous reviewed CSS value: 130.0.
            ScalarPin {
                field: "world",
                index: 4,
                upstream: 160.0,
                css: 123.07693,
            },
            // Previous reviewed CSS value: 0.0.
            ScalarPin {
                field: "worldBounds",
                index: 0,
                upstream: 60.0,
                css: 46.153847,
            },
            // Previous reviewed CSS value: 260.0.
            ScalarPin {
                field: "worldBounds",
                index: 2,
                upstream: 200.0,
                css: 153.84616,
            },
            // Previous reviewed CSS value: 130.0.
            ScalarPin {
                field: "computedWorld",
                index: 0,
                upstream: 160.0,
                css: 123.07693,
            },
            // Previous reviewed CSS value: 130.0.
            ScalarPin {
                field: "computedRoot",
                index: 0,
                upstream: 160.0,
                css: 123.07693,
            },
            // Previous reviewed CSS value: 130.0.
            ScalarPin {
                field: "computedLocal",
                index: 0,
                upstream: 160.0,
                css: 123.07693,
            },
        ],
    },
    // Previous reviewed first difference: Subject.world[4]: 130 != 190
    LayoutDivergence {
        fixture: "s20__flex-row__none__shape__participant__hug__over__unbounded",
        reason: "layout-diverges:Shape measurement has no minimum contract and keeps zero automatic minimum. CSS flex-shrink 1 shares the 200-point row in proportion to each item basis: the 60-point Fixed sibling and the Hug shape. The subject resolves to 162.5 points wide. The participant retains the natural primitive basis.",
        first_difference: "Subject.world[0]: 0.625 != 1",
        pins: &[
            ScalarPin {
                field: "world",
                index: 0,
                upstream: 1.0,
                css: 0.625,
            },
            // Previous reviewed CSS value: 130.0.
            ScalarPin {
                field: "world",
                index: 4,
                upstream: 190.0,
                css: 118.75,
            },
            // Previous reviewed CSS value: 0.0.
            ScalarPin {
                field: "worldBounds",
                index: 0,
                upstream: 60.0,
                css: 37.5,
            },
            ScalarPin {
                field: "worldBounds",
                index: 2,
                upstream: 260.0,
                css: 162.5,
            },
            // Previous reviewed CSS value: 130.0.
            ScalarPin {
                field: "computedWorld",
                index: 0,
                upstream: 190.0,
                css: 118.75,
            },
            // Previous reviewed CSS value: 130.0.
            ScalarPin {
                field: "computedRoot",
                index: 0,
                upstream: 190.0,
                css: 118.75,
            },
            // Previous reviewed CSS value: 130.0.
            ScalarPin {
                field: "computedLocal",
                index: 0,
                upstream: 190.0,
                css: 118.75,
            },
        ],
    },
    // Previous reviewed first difference: Subject.world[5]: 0 != 40
    LayoutDivergence {
        fixture: "s20__flex-column__none__layout-component__hug__leaf-content__over",
        reason: "layout-diverges:Shape measurement has no minimum contract and keeps zero automatic minimum. CSS flex-shrink 1 shares the 120-point column between the 40-point Fixed sibling and the Hug shape, weighted by their bases. The subject resolves to 83.07692 points high. Rive controlSize writes the solved primitive size back into its measurement, so the retained second solve uses that smaller basis.",
        first_difference: "Subject.world[5]: 36.923077 != 40",
        pins: &[
            // Previous reviewed CSS value: 0.0.
            ScalarPin {
                field: "world",
                index: 5,
                upstream: 40.0,
                css: 36.923077,
            },
            // Previous reviewed CSS value: 0.0.
            ScalarPin {
                field: "layoutBounds",
                index: 1,
                upstream: 40.0,
                css: 36.923077,
            },
            // Previous reviewed CSS value: 180.0.
            ScalarPin {
                field: "layoutBounds",
                index: 3,
                upstream: 120.0,
                css: 83.07692,
            },
            // Previous reviewed CSS value: 180.0.
            ScalarPin {
                field: "localBounds",
                index: 3,
                upstream: 120.0,
                css: 83.07692,
            },
            // Previous reviewed CSS value: 0.0.
            ScalarPin {
                field: "worldBounds",
                index: 1,
                upstream: 40.0,
                css: 36.923077,
            },
            // Previous reviewed CSS value: 180.0.
            ScalarPin {
                field: "worldBounds",
                index: 3,
                upstream: 120.0,
                css: 83.07692,
            },
            // Previous reviewed CSS value: 0.0.
            ScalarPin {
                field: "computedWorld",
                index: 1,
                upstream: 40.0,
                css: 36.923077,
            },
            // Previous reviewed CSS value: 0.0.
            ScalarPin {
                field: "computedRoot",
                index: 1,
                upstream: 40.0,
                css: 36.923077,
            },
            // Previous reviewed CSS value: 0.0.
            ScalarPin {
                field: "computedLocal",
                index: 1,
                upstream: 40.0,
                css: 36.923077,
            },
            // Previous reviewed CSS value: 180.0.
            ScalarPin {
                field: "computedSize",
                index: 1,
                upstream: 120.0,
                css: 83.07692,
            },
        ],
    },
    // Previous reviewed first difference: Subject.world[5]: 0 != 40
    LayoutDivergence {
        fixture: "s20__flex-column__none__layout-component__hug__leaf-content__over__unbounded",
        reason: "layout-diverges:Shape measurement has no minimum contract and keeps zero automatic minimum. CSS flex-shrink 1 shares the 120-point column between the 40-point Fixed sibling and the Hug shape, weighted by their bases. The subject resolves to 85.26315 points high. Rive controlSize writes the solved primitive size back into its measurement, so the retained second solve uses that smaller basis.",
        first_difference: "Subject.world[5]: 34.73684 != 40",
        pins: &[
            // Previous reviewed CSS value: 0.0.
            ScalarPin {
                field: "world",
                index: 5,
                upstream: 40.0,
                css: 34.73684,
            },
            // Previous reviewed CSS value: 0.0.
            ScalarPin {
                field: "layoutBounds",
                index: 1,
                upstream: 40.0,
                css: 34.73684,
            },
            ScalarPin {
                field: "layoutBounds",
                index: 3,
                upstream: 180.0,
                css: 85.26315,
            },
            ScalarPin {
                field: "localBounds",
                index: 3,
                upstream: 180.0,
                css: 85.26315,
            },
            // Previous reviewed CSS value: 0.0.
            ScalarPin {
                field: "worldBounds",
                index: 1,
                upstream: 40.0,
                css: 34.73684,
            },
            ScalarPin {
                field: "worldBounds",
                index: 3,
                upstream: 180.0,
                css: 85.26315,
            },
            // Previous reviewed CSS value: 0.0.
            ScalarPin {
                field: "computedWorld",
                index: 1,
                upstream: 40.0,
                css: 34.73684,
            },
            // Previous reviewed CSS value: 0.0.
            ScalarPin {
                field: "computedRoot",
                index: 1,
                upstream: 40.0,
                css: 34.73684,
            },
            // Previous reviewed CSS value: 0.0.
            ScalarPin {
                field: "computedLocal",
                index: 1,
                upstream: 40.0,
                css: 34.73684,
            },
            ScalarPin {
                field: "computedSize",
                index: 1,
                upstream: 180.0,
                css: 85.26315,
            },
        ],
    },
    // Previous reviewed first difference: Subject.world[3]: 1 != 0.6667
    LayoutDivergence {
        fixture: "s20__flex-column__none__shape__participant__hug__over",
        reason: "layout-diverges:Shape measurement has no minimum contract and keeps zero automatic minimum. CSS flex-shrink 1 shares the 120-point column between the 40-point Fixed sibling and the Hug shape, weighted by their bases. The subject resolves to 90 points high. The participant retains the natural primitive basis.",
        first_difference: "Subject.world[3]: 0.5 != 0.6667",
        pins: &[
            // Previous reviewed CSS value: 1.0.
            ScalarPin {
                field: "world",
                index: 3,
                upstream: 0.6667,
                css: 0.5,
            },
            // Previous reviewed CSS value: 90.0.
            ScalarPin {
                field: "world",
                index: 5,
                upstream: 100.0,
                css: 75.0,
            },
            // Previous reviewed CSS value: 0.0.
            ScalarPin {
                field: "worldBounds",
                index: 1,
                upstream: 40.0,
                css: 30.0,
            },
            // Previous reviewed CSS value: 180.0.
            ScalarPin {
                field: "worldBounds",
                index: 3,
                upstream: 120.0,
                css: 90.0,
            },
            // Previous reviewed CSS value: 90.0.
            ScalarPin {
                field: "computedWorld",
                index: 1,
                upstream: 100.0,
                css: 75.0,
            },
            // Previous reviewed CSS value: 90.0.
            ScalarPin {
                field: "computedRoot",
                index: 1,
                upstream: 100.0,
                css: 75.0,
            },
            // Previous reviewed CSS value: 90.0.
            ScalarPin {
                field: "computedLocal",
                index: 1,
                upstream: 100.0,
                css: 75.0,
            },
        ],
    },
    // Previous reviewed first difference: Subject.world[5]: 90 != 130
    LayoutDivergence {
        fixture: "s20__flex-column__none__shape__participant__hug__over__unbounded",
        reason: "layout-diverges:Shape measurement has no minimum contract and keeps zero automatic minimum. CSS flex-shrink 1 shares the 120-point column between the 40-point Fixed sibling and the Hug shape, weighted by their bases. The subject resolves to 98.18182 points high. The participant retains the natural primitive basis.",
        first_difference: "Subject.world[3]: 0.54545456 != 1",
        pins: &[
            ScalarPin {
                field: "world",
                index: 3,
                upstream: 1.0,
                css: 0.54545456,
            },
            // Previous reviewed CSS value: 90.0.
            ScalarPin {
                field: "world",
                index: 5,
                upstream: 130.0,
                css: 70.909096,
            },
            // Previous reviewed CSS value: 0.0.
            ScalarPin {
                field: "worldBounds",
                index: 1,
                upstream: 40.0,
                css: 21.818184,
            },
            ScalarPin {
                field: "worldBounds",
                index: 3,
                upstream: 180.0,
                css: 98.18182,
            },
            // Previous reviewed CSS value: 90.0.
            ScalarPin {
                field: "computedWorld",
                index: 1,
                upstream: 130.0,
                css: 70.909096,
            },
            // Previous reviewed CSS value: 90.0.
            ScalarPin {
                field: "computedRoot",
                index: 1,
                upstream: 130.0,
                css: 70.909096,
            },
            // Previous reviewed CSS value: 90.0.
            ScalarPin {
                field: "computedLocal",
                index: 1,
                upstream: 130.0,
                css: 70.909096,
            },
        ],
    },
    // Previous reviewed first difference: Subject.world[4]: 20 != 60
    LayoutDivergence {
        fixture: "s20__flex-row__none__layout-component__hug__k-max-over-fill__leaf-content__over__unbounded",
        reason: "layout-diverges:Shape measurement has no minimum contract and keeps zero automatic minimum. CSS flex-shrink 1 shares the 200-point row in proportion to each item basis: the 60-point Fixed sibling and the Hug shape. The subject resolves to 146.06741 points wide. Rive controlSize writes the solved primitive size back into its measurement; the retained second solve uses that smaller basis. The authored 180 maximum remains in force and is above this allocation.",
        first_difference: "Subject.world[4]: 53.932583 != 60",
        pins: &[
            // Previous reviewed CSS value: 20.0.
            ScalarPin {
                field: "world",
                index: 4,
                upstream: 60.0,
                css: 53.932583,
            },
            // Previous reviewed CSS value: 20.0.
            ScalarPin {
                field: "layoutBounds",
                index: 0,
                upstream: 60.0,
                css: 53.932583,
            },
            ScalarPin {
                field: "layoutBounds",
                index: 2,
                upstream: 180.0,
                css: 146.06741,
            },
            ScalarPin {
                field: "localBounds",
                index: 2,
                upstream: 180.0,
                css: 146.06741,
            },
            // Previous reviewed CSS value: 20.0.
            ScalarPin {
                field: "worldBounds",
                index: 0,
                upstream: 60.0,
                css: 53.932583,
            },
            ScalarPin {
                field: "worldBounds",
                index: 2,
                upstream: 180.0,
                css: 146.06741,
            },
            // Previous reviewed CSS value: 20.0.
            ScalarPin {
                field: "computedWorld",
                index: 0,
                upstream: 60.0,
                css: 53.932583,
            },
            // Previous reviewed CSS value: 20.0.
            ScalarPin {
                field: "computedRoot",
                index: 0,
                upstream: 60.0,
                css: 53.932583,
            },
            // Previous reviewed CSS value: 20.0.
            ScalarPin {
                field: "computedLocal",
                index: 0,
                upstream: 60.0,
                css: 53.932583,
            },
            ScalarPin {
                field: "computedSize",
                index: 0,
                upstream: 180.0,
                css: 146.06741,
            },
        ],
    },
    // Previous reviewed first difference: Subject.world[4]: 0 != 60
    LayoutDivergence {
        fixture: "s20__flex-row__none__layout-component__hug__y-fixed__leaf-content__over__unbounded",
        reason: "layout-diverges:Shape measurement has no minimum contract and keeps zero automatic minimum. CSS flex-shrink 1 shares the 200-point row in proportion to each item basis: the 60-point Fixed sibling and the Hug shape. The subject resolves to 146.06741 points wide. Rive controlSize writes the solved primitive size back into its measurement; the retained second solve uses that smaller basis. The authored cross-axis height stays 40.",
        first_difference: "Subject.world[4]: 53.932583 != 60",
        pins: &[
            // Previous reviewed CSS value: 0.0.
            ScalarPin {
                field: "world",
                index: 4,
                upstream: 60.0,
                css: 53.932583,
            },
            // Previous reviewed CSS value: 0.0.
            ScalarPin {
                field: "layoutBounds",
                index: 0,
                upstream: 60.0,
                css: 53.932583,
            },
            ScalarPin {
                field: "layoutBounds",
                index: 2,
                upstream: 260.0,
                css: 146.06741,
            },
            ScalarPin {
                field: "localBounds",
                index: 2,
                upstream: 260.0,
                css: 146.06741,
            },
            // Previous reviewed CSS value: 0.0.
            ScalarPin {
                field: "worldBounds",
                index: 0,
                upstream: 60.0,
                css: 53.932583,
            },
            ScalarPin {
                field: "worldBounds",
                index: 2,
                upstream: 260.0,
                css: 146.06741,
            },
            // Previous reviewed CSS value: 0.0.
            ScalarPin {
                field: "computedWorld",
                index: 0,
                upstream: 60.0,
                css: 53.932583,
            },
            // Previous reviewed CSS value: 0.0.
            ScalarPin {
                field: "computedRoot",
                index: 0,
                upstream: 60.0,
                css: 53.932583,
            },
            // Previous reviewed CSS value: 0.0.
            ScalarPin {
                field: "computedLocal",
                index: 0,
                upstream: 60.0,
                css: 53.932583,
            },
            ScalarPin {
                field: "computedSize",
                index: 0,
                upstream: 260.0,
                css: 146.06741,
            },
        ],
    },
    // Previous reviewed first difference: Subject.world[4]: 110 != 150
    LayoutDivergence {
        fixture: "s20__flex-row__none__shape__participant__hug__k-max-over-fill__over__unbounded",
        reason: "layout-diverges:Shape measurement has no minimum contract and keeps zero automatic minimum. CSS flex-shrink 1 shares the 200-point row in proportion to each item basis: the 60-point Fixed sibling and the Hug shape. The subject resolves to 162.5 points wide. The participant retains the natural primitive basis. The authored 180 maximum remains in force and is above this allocation.",
        first_difference: "Subject.world[0]: 0.625 != 0.6923",
        pins: &[
            ScalarPin {
                field: "world",
                index: 0,
                upstream: 0.6923,
                css: 0.625,
            },
            // Previous reviewed CSS value: 110.0.
            ScalarPin {
                field: "world",
                index: 4,
                upstream: 150.0,
                css: 118.75,
            },
            // Previous reviewed CSS value: 20.0.
            ScalarPin {
                field: "worldBounds",
                index: 0,
                upstream: 60.0,
                css: 37.5,
            },
            ScalarPin {
                field: "worldBounds",
                index: 2,
                upstream: 180.0,
                css: 162.5,
            },
            // Previous reviewed CSS value: 110.0.
            ScalarPin {
                field: "computedWorld",
                index: 0,
                upstream: 150.0,
                css: 118.75,
            },
            // Previous reviewed CSS value: 110.0.
            ScalarPin {
                field: "computedRoot",
                index: 0,
                upstream: 150.0,
                css: 118.75,
            },
            // Previous reviewed CSS value: 110.0.
            ScalarPin {
                field: "computedLocal",
                index: 0,
                upstream: 150.0,
                css: 118.75,
            },
        ],
    },
    // Previous reviewed first difference: Subject.world[4]: 130 != 190
    LayoutDivergence {
        fixture: "s20__flex-row__none__shape__participant__hug__y-fixed__over__unbounded",
        reason: "layout-diverges:Shape measurement has no minimum contract and keeps zero automatic minimum. CSS flex-shrink 1 shares the 200-point row in proportion to each item basis: the 60-point Fixed sibling and the Hug shape. The subject resolves to 162.5 points wide. The participant retains the natural primitive basis. The authored cross-axis height stays 40.",
        first_difference: "Subject.world[0]: 0.625 != 1",
        pins: &[
            ScalarPin {
                field: "world",
                index: 0,
                upstream: 1.0,
                css: 0.625,
            },
            // Previous reviewed CSS value: 130.0.
            ScalarPin {
                field: "world",
                index: 4,
                upstream: 190.0,
                css: 118.75,
            },
            // Previous reviewed CSS value: 0.0.
            ScalarPin {
                field: "worldBounds",
                index: 0,
                upstream: 60.0,
                css: 37.5,
            },
            ScalarPin {
                field: "worldBounds",
                index: 2,
                upstream: 260.0,
                css: 162.5,
            },
            // Previous reviewed CSS value: 130.0.
            ScalarPin {
                field: "computedWorld",
                index: 0,
                upstream: 190.0,
                css: 118.75,
            },
            // Previous reviewed CSS value: 130.0.
            ScalarPin {
                field: "computedRoot",
                index: 0,
                upstream: 190.0,
                css: 118.75,
            },
            // Previous reviewed CSS value: 130.0.
            ScalarPin {
                field: "computedLocal",
                index: 0,
                upstream: 190.0,
                css: 118.75,
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
    LayoutDivergence {
        fixture: "s20__flex-row__none__text__participant__hug__k-max-over-host__over",
        reason: "layout-diverges:CSS min-width:auto keeps the Hug content floor at 260, capped by max-width where authored, above the 200 row offer; flex-shrink 1 reduces the empty Fixed sibling from its 60 basis to zero.",
        first_difference: "Subject.world[4]: 65 != 110",
        pins: &[
            ScalarPin {
                field: "world",
                index: 4,
                upstream: 110.0,
                css: 65.0,
            },
            ScalarPin {
                field: "computedWorld",
                index: 0,
                upstream: 110.0,
                css: 65.0,
            },
            ScalarPin {
                field: "computedRoot",
                index: 0,
                upstream: 110.0,
                css: 65.0,
            },
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
        reason: "layout-diverges:CSS min-width:auto floors the text at min(526.28906 longest-word min-content, 260 max-width) = 260 and the leading Fill sibling at its 50 content contribution, so they overflow the 200 row at widths 50 and 260. Fill fractions and bases are unchanged. The full phrase max-content width is 652.734375.",
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
        reason: "layout-diverges:CSS min-width:auto floors the text at min(526.28906 longest-word min-content, 260 max-width) = 260 and the leading Fill sibling at its 50 content contribution, so they overflow the 200 row at widths 50 and 260. Fill fractions and bases are unchanged. Width is Fill, so hugUnbounded only changes the height mode. The full phrase max-content width is 652.734375.",
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
        fixture: "s20__flex-row__none__layout-component__fill__y-hug__k-max-over-host__sibling-fills__leaf-text__over",
        reason: "layout-diverges:CSS min-width:auto floors the text at min(526.28906 longest-word min-content, 260 max-width) = 260 and the leading Fill sibling at its 50 content contribution, so they overflow the 200 row at widths 50 and 260. Fill fractions and bases are unchanged. The full phrase max-content width is 652.734375.",
        first_difference: "Subject.world[4]: 50 != 40",
        pins: &[
            ScalarPin {
                field: "world",
                index: 4,
                upstream: 40.0,
                css: 50.0,
            },
            ScalarPin {
                field: "layoutBounds",
                index: 0,
                upstream: 40.0,
                css: 50.0,
            },
            ScalarPin {
                field: "layoutBounds",
                index: 2,
                upstream: 160.0,
                css: 260.0,
            },
            ScalarPin {
                field: "localBounds",
                index: 2,
                upstream: 160.0,
                css: 260.0,
            },
            ScalarPin {
                field: "worldBounds",
                index: 0,
                upstream: 40.0,
                css: 50.0,
            },
            ScalarPin {
                field: "worldBounds",
                index: 2,
                upstream: 160.0,
                css: 260.0,
            },
            ScalarPin {
                field: "computedWorld",
                index: 0,
                upstream: 40.0,
                css: 50.0,
            },
            ScalarPin {
                field: "computedRoot",
                index: 0,
                upstream: 40.0,
                css: 50.0,
            },
            ScalarPin {
                field: "computedLocal",
                index: 0,
                upstream: 40.0,
                css: 50.0,
            },
            ScalarPin {
                field: "computedSize",
                index: 0,
                upstream: 160.0,
                css: 260.0,
            },
        ],
    },
    LayoutDivergence {
        fixture: "s20__flex-row__none__layout-component__fill__y-hug__k-max-over-host__sibling-fills__leaf-text__over__unbounded",
        reason: "layout-diverges:CSS min-width:auto floors the text at min(526.28906 longest-word min-content, 260 max-width) = 260 and the leading Fill sibling at its 50 content contribution, so they overflow the 200 row at widths 50 and 260. Fill fractions and bases are unchanged. Width is Fill, so hugUnbounded only changes the height mode. The full phrase max-content width is 652.734375.",
        first_difference: "Subject.world[4]: 50 != 40",
        pins: &[
            ScalarPin {
                field: "world",
                index: 4,
                upstream: 40.0,
                css: 50.0,
            },
            ScalarPin {
                field: "layoutBounds",
                index: 0,
                upstream: 40.0,
                css: 50.0,
            },
            ScalarPin {
                field: "layoutBounds",
                index: 2,
                upstream: 160.0,
                css: 260.0,
            },
            ScalarPin {
                field: "localBounds",
                index: 2,
                upstream: 160.0,
                css: 260.0,
            },
            ScalarPin {
                field: "worldBounds",
                index: 0,
                upstream: 40.0,
                css: 50.0,
            },
            ScalarPin {
                field: "worldBounds",
                index: 2,
                upstream: 160.0,
                css: 260.0,
            },
            ScalarPin {
                field: "computedWorld",
                index: 0,
                upstream: 40.0,
                css: 50.0,
            },
            ScalarPin {
                field: "computedRoot",
                index: 0,
                upstream: 40.0,
                css: 50.0,
            },
            ScalarPin {
                field: "computedLocal",
                index: 0,
                upstream: 40.0,
                css: 50.0,
            },
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
    // Previous reviewed first difference: Subject.world[3]: 1 != 0.1942
    LayoutDivergence {
        fixture: "s1__flex-column__none__image__participant",
        reason: "layout-diverges:CSS min-height:auto uses the transferred image minimum min(412, 200 * 412 / 442) = 186.42534, rather than the old 80-point remainder of the 120 column. The empty Fixed sibling shrinks from 40 to zero. Image scale on this axis is 200 / 442 = 0.4524887.",
        first_difference: "Subject.world[3]: 0.4524887 != 0.1942",
        pins: &[
            // Previous reviewed CSS value: 1.0.
            ScalarPin {
                field: "world",
                index: 3,
                upstream: 0.1942,
                css: 0.4524887,
            },
            // Previous reviewed CSS value: 206.0.
            ScalarPin {
                field: "world",
                index: 5,
                upstream: 80.0,
                css: 93.21267,
            },
            // Previous reviewed CSS value: 206.0.
            ScalarPin {
                field: "computedWorld",
                index: 1,
                upstream: 80.0,
                css: 93.21267,
            },
            // Previous reviewed CSS value: 206.0.
            ScalarPin {
                field: "computedRoot",
                index: 1,
                upstream: 80.0,
                css: 93.21267,
            },
            // Previous reviewed CSS value: 206.0.
            ScalarPin {
                field: "computedLocal",
                index: 1,
                upstream: 80.0,
                css: 93.21267,
            },
            // Previous reviewed CSS value: 412.0.
            ScalarPin {
                field: "computedSize",
                index: 1,
                upstream: 80.0,
                css: 186.42534,
            },
        ],
    },
    // Previous reviewed first difference: Subject.world[3]: 1 != 0.1942
    LayoutDivergence {
        fixture: "s1__flex-column__group__image__participant",
        reason: "layout-diverges:CSS min-height:auto uses the transferred image minimum min(412, 200 * 412 / 442) = 186.42534, rather than the old 80-point remainder of the 120 column. The empty Fixed sibling shrinks from 40 to zero. Image scale on this axis is 200 / 442 = 0.4524887.",
        first_difference: "Subject.world[3]: 0.4524887 != 0.1942",
        pins: &[
            // Previous reviewed CSS value: 1.0.
            ScalarPin {
                field: "world",
                index: 3,
                upstream: 0.1942,
                css: 0.4524887,
            },
            // Previous reviewed CSS value: 206.0.
            ScalarPin {
                field: "world",
                index: 5,
                upstream: 80.0,
                css: 93.21267,
            },
            // Previous reviewed CSS value: 206.0.
            ScalarPin {
                field: "computedWorld",
                index: 1,
                upstream: 80.0,
                css: 93.21267,
            },
            // Previous reviewed CSS value: 206.0.
            ScalarPin {
                field: "computedRoot",
                index: 1,
                upstream: 80.0,
                css: 93.21267,
            },
            // Previous reviewed CSS value: 206.0.
            ScalarPin {
                field: "computedLocal",
                index: 1,
                upstream: 80.0,
                css: 93.21267,
            },
            // Previous reviewed CSS value: 412.0.
            ScalarPin {
                field: "computedSize",
                index: 1,
                upstream: 80.0,
                css: 186.42534,
            },
        ],
    },
    // Previous reviewed first difference: Subject.world[3]: 1 != 0.1942
    LayoutDivergence {
        fixture: "s1__flex-column__solo-active__image__participant",
        reason: "layout-diverges:CSS min-height:auto uses the transferred image minimum min(412, 200 * 412 / 442) = 186.42534, rather than the old 80-point remainder of the 120 column. The empty Fixed sibling shrinks from 40 to zero. Image scale on this axis is 200 / 442 = 0.4524887.",
        first_difference: "Subject.world[3]: 0.4524887 != 0.1942",
        pins: &[
            // Previous reviewed CSS value: 1.0.
            ScalarPin {
                field: "world",
                index: 3,
                upstream: 0.1942,
                css: 0.4524887,
            },
            // Previous reviewed CSS value: 206.0.
            ScalarPin {
                field: "world",
                index: 5,
                upstream: 80.0,
                css: 93.21267,
            },
            // Previous reviewed CSS value: 206.0.
            ScalarPin {
                field: "computedWorld",
                index: 1,
                upstream: 80.0,
                css: 93.21267,
            },
            // Previous reviewed CSS value: 206.0.
            ScalarPin {
                field: "computedRoot",
                index: 1,
                upstream: 80.0,
                css: 93.21267,
            },
            // Previous reviewed CSS value: 206.0.
            ScalarPin {
                field: "computedLocal",
                index: 1,
                upstream: 80.0,
                css: 93.21267,
            },
            // Previous reviewed CSS value: 412.0.
            ScalarPin {
                field: "computedSize",
                index: 1,
                upstream: 80.0,
                css: 186.42534,
            },
        ],
    },
    // Previous reviewed first difference: Subject.world[3]: 1 != 0.1942
    LayoutDivergence {
        fixture: "s1__flex-column__group-in-solo__image__participant",
        reason: "layout-diverges:CSS min-height:auto uses the transferred image minimum min(412, 200 * 412 / 442) = 186.42534, rather than the old 80-point remainder of the 120 column. The empty Fixed sibling shrinks from 40 to zero. Image scale on this axis is 200 / 442 = 0.4524887.",
        first_difference: "Subject.world[3]: 0.4524887 != 0.1942",
        pins: &[
            // Previous reviewed CSS value: 1.0.
            ScalarPin {
                field: "world",
                index: 3,
                upstream: 0.1942,
                css: 0.4524887,
            },
            // Previous reviewed CSS value: 206.0.
            ScalarPin {
                field: "world",
                index: 5,
                upstream: 80.0,
                css: 93.21267,
            },
            // Previous reviewed CSS value: 206.0.
            ScalarPin {
                field: "computedWorld",
                index: 1,
                upstream: 80.0,
                css: 93.21267,
            },
            // Previous reviewed CSS value: 206.0.
            ScalarPin {
                field: "computedRoot",
                index: 1,
                upstream: 80.0,
                css: 93.21267,
            },
            // Previous reviewed CSS value: 206.0.
            ScalarPin {
                field: "computedLocal",
                index: 1,
                upstream: 80.0,
                css: 93.21267,
            },
            // Previous reviewed CSS value: 412.0.
            ScalarPin {
                field: "computedSize",
                index: 1,
                upstream: 80.0,
                css: 186.42534,
            },
        ],
    },
    // Previous reviewed first difference: Subject.world[3]: 1 != 0.1942
    LayoutDivergence {
        fixture: "s2__flex-column__none__image__participant__c-translation",
        reason: "layout-diverges:CSS min-height:auto uses the transferred image minimum min(412, 200 * 412 / 442) = 186.42534, rather than the old 80-point remainder of the 120 column. The empty Fixed sibling shrinks from 40 to zero. Image scale on this axis is 200 / 442 = 0.4524887. The authored constraint still owns the final position.",
        first_difference: "Subject.world[3]: 0.4524887 != 0.1942",
        pins: &[
            // Previous reviewed CSS value: 1.0.
            ScalarPin {
                field: "world",
                index: 3,
                upstream: 0.1942,
                css: 0.4524887,
            },
            // Previous reviewed CSS value: 412.0.
            ScalarPin {
                field: "computedSize",
                index: 1,
                upstream: 80.0,
                css: 186.42534,
            },
        ],
    },
    // Previous reviewed first difference: Subject.world[3]: 1 != 0.1942
    LayoutDivergence {
        fixture: "s2__flex-column__none__image__participant__c-follow-path",
        reason: "layout-diverges:CSS min-height:auto uses the transferred image minimum min(412, 200 * 412 / 442) = 186.42534, rather than the old 80-point remainder of the 120 column. The empty Fixed sibling shrinks from 40 to zero. Image scale on this axis is 200 / 442 = 0.4524887. The authored constraint still owns the final position.",
        first_difference: "Subject.world[3]: 0.45248866 != 0.1942",
        pins: &[
            // Previous reviewed CSS value: 1.0.
            ScalarPin {
                field: "world",
                index: 3,
                upstream: 0.1942,
                css: 0.45248866,
            },
            // Previous reviewed CSS value: 412.0.
            ScalarPin {
                field: "computedSize",
                index: 1,
                upstream: 80.0,
                css: 186.42534,
            },
        ],
    },
    // Previous reviewed first difference: Subject.world[3]: 1 != 0.1942
    LayoutDivergence {
        fixture: "s2__flex-column__group__image__participant__c-translation",
        reason: "layout-diverges:CSS min-height:auto uses the transferred image minimum min(412, 200 * 412 / 442) = 186.42534, rather than the old 80-point remainder of the 120 column. The empty Fixed sibling shrinks from 40 to zero. Image scale on this axis is 200 / 442 = 0.4524887. The authored constraint still owns the final position.",
        first_difference: "Subject.world[3]: 0.4524887 != 0.1942",
        pins: &[
            // Previous reviewed CSS value: 1.0.
            ScalarPin {
                field: "world",
                index: 3,
                upstream: 0.1942,
                css: 0.4524887,
            },
            // Previous reviewed CSS value: 412.0.
            ScalarPin {
                field: "computedSize",
                index: 1,
                upstream: 80.0,
                css: 186.42534,
            },
        ],
    },
    // Previous reviewed first difference: Subject.world[3]: 1 != 0.1942
    LayoutDivergence {
        fixture: "s2__flex-column__group__image__participant__c-follow-path",
        reason: "layout-diverges:CSS min-height:auto uses the transferred image minimum min(412, 200 * 412 / 442) = 186.42534, rather than the old 80-point remainder of the 120 column. The empty Fixed sibling shrinks from 40 to zero. Image scale on this axis is 200 / 442 = 0.4524887. The authored constraint still owns the final position.",
        first_difference: "Subject.world[3]: 0.45248866 != 0.1942",
        pins: &[
            // Previous reviewed CSS value: 1.0.
            ScalarPin {
                field: "world",
                index: 3,
                upstream: 0.1942,
                css: 0.45248866,
            },
            // Previous reviewed CSS value: 412.0.
            ScalarPin {
                field: "computedSize",
                index: 1,
                upstream: 80.0,
                css: 186.42534,
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
