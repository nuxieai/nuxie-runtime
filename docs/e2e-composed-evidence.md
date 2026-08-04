# Composed runtime differential evidence

Measured 2026-08-04 on macOS arm64 against pinned rive-runtime
`4ac7b32798da0482e441ef09304dc3b480ed3ee5`.

## Contract

`make e2e-composed-compare` builds both scripted golden runners in release
mode and executes each `e2e-composed.toml` fixture once per runtime. Within
that one loaded session, the existing `docs/side-channel-format.md` verb
grammar drives a host resize, pointer input, and typed view-model mutation at
ordered timestamps. Both runners advance around those inputs, drain complete
semantic diffs, sample, and draw. The comparator checks the entire emitted
stream under the existing exact-stream rules.

The gate rejects a row unless both actual streams contain records proving all
of these stages ran: `advance`, pointer `input`, `viewModel`, `resize`,
`semantics`, `sample`, and `frame`. It also rejects tolerant/structural modes,
missing scripts, known-divergence side-channel suppression, and semantic-only
stream projection.

## Enrollment and result

| fixture | composed features |
|---|---|
| `listener_view_model.riv` | listeners, data binding, numeric VM mutation |
| `data_converter_to_number.riv` | text, data converter, numeric VM mutation |
| `scripted_boolean.riv` | animation, boolean VM mutation |
| `relative_data_binding.riv` | nested artboard, nested VM path, data binding |
| `scripted_color.riv` | animation, data binding, color VM mutation |
| `scripted_enum.riv` | animation, custom enum VM mutation |
| `scripted_string.riv` | animation, string VM mutation |
| `collapsable_data_binding.riv` | solo, data binding, numeric VM mutation |
| `data_converter_interpolator_reset.riv` | converter interpolation and reset |
| `library_data_enum_test.riv` | events, data binding, enum VM mutation |

Result:

```text
golden-compare summary: entries=10 exact=10 exact-segments=40 side-channel-segments=40 diverges=0 unsupported-feature=0 not-yet=0
golden-compare composed sessions: exact=10/10
```

No tolerance or comparison rule changed. The dedicated manifest uses exact
status for every row and compares every draw and side-channel record.
