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
        fixture: "s20__flex-column__none__layout-component__hug__k-max-over-host__leaf-text__over",
        reason: "layout-diverges:CSS fit-content keeps the unbreakable minimum above the 200 column offer, then max-width 260 caps the box; Yoga clamps it to the offer.",
        first_difference: "Subject.layoutBounds[2]: 260 != 200",
        pins: &[
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
                index: 2,
                upstream: 200.0,
                css: 260.0,
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
        fixture: "s20__flex-column__none__text__participant__hug__k-max-over-host__over",
        reason: "layout-diverges:CSS fit-content keeps the unbreakable minimum above the 200 column offer, then max-width 260 caps the box; Yoga clamps it to the offer.",
        first_difference: "Subject.world[4]: 65 != 50",
        pins: &[
            ScalarPin {
                field: "world",
                index: 4,
                upstream: 50.0,
                css: 65.0,
            },
            ScalarPin {
                field: "computedWorld",
                index: 0,
                upstream: 50.0,
                css: 65.0,
            },
            ScalarPin {
                field: "computedRoot",
                index: 0,
                upstream: 50.0,
                css: 65.0,
            },
            ScalarPin {
                field: "computedLocal",
                index: 0,
                upstream: 50.0,
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
        fixture: "s20__flex-row__none__layout-component__fill__y-hug__k-max-over-host__sibling-fills__leaf-text__over",
        reason: "layout-diverges:CSS fit-content supplies the unbreakable content basis; stock Taffy flex shrink shares the 200 row as 185.7699 and 14.230129.",
        first_difference: "Subject.world[4]: 14.230129 != 40",
        pins: &[
            ScalarPin {
                field: "world",
                index: 4,
                upstream: 40.0,
                css: 14.230129,
            },
            ScalarPin {
                field: "layoutBounds",
                index: 0,
                upstream: 40.0,
                css: 14.230129,
            },
            ScalarPin {
                field: "layoutBounds",
                index: 2,
                upstream: 160.0,
                css: 185.7699,
            },
            ScalarPin {
                field: "localBounds",
                index: 2,
                upstream: 160.0,
                css: 185.7699,
            },
            ScalarPin {
                field: "worldBounds",
                index: 0,
                upstream: 40.0,
                css: 14.230129,
            },
            ScalarPin {
                field: "worldBounds",
                index: 2,
                upstream: 160.0,
                css: 185.7699,
            },
            ScalarPin {
                field: "computedWorld",
                index: 0,
                upstream: 40.0,
                css: 14.230129,
            },
            ScalarPin {
                field: "computedRoot",
                index: 0,
                upstream: 40.0,
                css: 14.230129,
            },
            ScalarPin {
                field: "computedLocal",
                index: 0,
                upstream: 40.0,
                css: 14.230129,
            },
            ScalarPin {
                field: "computedSize",
                index: 0,
                upstream: 160.0,
                css: 185.7699,
            },
        ],
    },
    LayoutDivergence {
        fixture: "s20__flex-row__none__layout-component__fill__y-hug__k-max-over-host__sibling-fills__leaf-text__over__unbounded",
        reason: "layout-diverges:Width is Fill, so hugUnbounded only unbounds the Hug height; the CSS content basis still shrinks to 185.7699 beside the 14.230129 sibling.",
        first_difference: "Subject.world[4]: 14.230129 != 40",
        pins: &[
            ScalarPin {
                field: "world",
                index: 4,
                upstream: 40.0,
                css: 14.230129,
            },
            ScalarPin {
                field: "layoutBounds",
                index: 0,
                upstream: 40.0,
                css: 14.230129,
            },
            ScalarPin {
                field: "layoutBounds",
                index: 2,
                upstream: 160.0,
                css: 185.7699,
            },
            ScalarPin {
                field: "localBounds",
                index: 2,
                upstream: 160.0,
                css: 185.7699,
            },
            ScalarPin {
                field: "worldBounds",
                index: 0,
                upstream: 40.0,
                css: 14.230129,
            },
            ScalarPin {
                field: "worldBounds",
                index: 2,
                upstream: 160.0,
                css: 185.7699,
            },
            ScalarPin {
                field: "computedWorld",
                index: 0,
                upstream: 40.0,
                css: 14.230129,
            },
            ScalarPin {
                field: "computedRoot",
                index: 0,
                upstream: 40.0,
                css: 14.230129,
            },
            ScalarPin {
                field: "computedLocal",
                index: 0,
                upstream: 40.0,
                css: 14.230129,
            },
            ScalarPin {
                field: "computedSize",
                index: 0,
                upstream: 160.0,
                css: 185.7699,
            },
        ],
    },
    LayoutDivergence {
        fixture: "s20__flex-row__none__layout-component__hug__k-max-over-host__leaf-text__over",
        reason: "layout-diverges:CSS fit-content keeps the unbreakable minimum above the 200 row offer, then max-width 260 caps the box; Yoga clamps it to the offer.",
        first_difference: "Subject.layoutBounds[2]: 260 != 200",
        pins: &[
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
                index: 2,
                upstream: 200.0,
                css: 260.0,
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
        fixture: "s20__flex-row__none__text__participant__hug__k-max-over-host__over",
        reason: "layout-diverges:CSS fit-content keeps the unbreakable minimum above the 200 row offer, then max-width 260 caps the box; Yoga clamps it to the offer.",
        first_difference: "Subject.world[4]: 125 != 110",
        pins: &[
            ScalarPin {
                field: "world",
                index: 4,
                upstream: 110.0,
                css: 125.0,
            },
            ScalarPin {
                field: "computedWorld",
                index: 0,
                upstream: 110.0,
                css: 125.0,
            },
            ScalarPin {
                field: "computedRoot",
                index: 0,
                upstream: 110.0,
                css: 125.0,
            },
            ScalarPin {
                field: "computedLocal",
                index: 0,
                upstream: 110.0,
                css: 125.0,
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
