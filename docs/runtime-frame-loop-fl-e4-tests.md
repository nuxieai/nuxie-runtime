# FL-E4 upstream-test evidence

This note records the intrinsic-sizing and joystick-state slice of the pinned
`W65-unit-test-triage.md` assignments. Both Class B tests use the real fixtures
from pinned C++ commit `d788e8ec6e8b598526607d6a1e8818e8b637b60c`.

| W65 class | Upstream file | Assigned case | Rust evidence |
|---|---|---|---|
| B | `joystick_flags_test.cpp` | `joystick flags load as expected` | `upstream_joystick_flags_fixture_actions_are_ported` reads `joystick_flag_test.riv`, asserts all four authored flag masks, and repeats the six exact x/y animation actions and shape-x results |
| B | `layout_test.cpp` | `LayoutComponent with intrinsic size gets measured correctly` | `upstream_layout_intrinsic_measure_fixture_is_ported` opens artboard `hi` from `layout/measure_tests.riv`, requires both `TextLayout` and `HiText`, and repeats the four pinned `HiText.localBounds()` assertions: `0, 0, 62.48047, 72.62695` |

The owner mechanism also has two focused occurrence tests. The synthetic
custom-handle fixture proves `Joystick::update` composes parent/joystick world
space, caches its inverse, maps the retained source translation, and writes
both normalized axes. Its clone test mutates only one source occurrence and
proves clone-local source/matrix state. A unit test around the generated
double callback proves the property write publishes exactly root `Components`
dirt and no local joystick dirt. Structural ratchets reject the former central
callback/fallback shapes and keep the global path-epoch compensation ceiling
unchanged; together these delete the FL-G05-class fallback in E4 territory.

Rust uses typed arena handles in place of raw C++ pointers. Component/source
handles are rebuilt against the cloned occurrence; immutable linear-animation
definitions use the existing typed definition handle; nested remaps use a
private object-slot handle because `NestedRemapAnimation` is a Core object,
not a Component. The ordered y-then-x dependent list and duplicates are
preserved. This is an owner-safe representation difference, not a behavioral
divergence.

Adversarial coverage includes the pinned asymmetric zero-extent factor math
(`width == 0` returns x=0 while `height == 0` retains the C++ NaN), C++-ordered
NaN/signed-zero intrinsic minima, the existing graph differential that keeps
y-then-x duplicate nested-remap dependents, and the binary differential that
accepts a missing/wrong-type handle source as C++'s nonfatal `MissingObject`.
