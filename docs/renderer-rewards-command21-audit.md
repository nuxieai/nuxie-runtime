# Rewards Command-21 Audit

Date: 2026-07-15

## Verdict

`riv-rewards_demo-frame-0-clockwise-atomic` no longer represents an
advanced-feather implementation gap. Its remaining output is the reviewed
Metal/WebGPU subpixel clip-edge boundary. The row stays gated under its
unchanged native Metal reference and `2/32` contract.

The rejected mixed artifact harness was not integrated. This audit uses a
fresh empty-directory recapture plus focused Rust tests and records only the
resulting evidence.

## Provenance

- C++ runtime: `7c778d13c5d903b3b74eec1dd6bb68a811dea5f2`
- C++ Dawn oracle executable:
  `9a554fdda32f2dc11a685652b64bc9680320f3bdde1fe73cbafbb8e8b5b97ca9`
- Rewards stream:
  `3c565fd9e0fe99a4a00c966e3d5ca1165339792e5403eba550b2bed0d5270140`
- Hardware: Apple M5 Max, Dawn WebGPU on Metal versus wgpu on Metal

The fresh recapture reproduced every prior artifact hash exactly:

| Artifact | SHA-256 |
| --- | --- |
| command-16 inputs | `3c0eb270575727860907a7a85db8e9dac71ca20d30e9de2247617a9723429cbe` |
| command-16 pixels | `3b58158f24b2f6d18e1e7440b7111b476d6e145470c3c1a5a8b491059ba677c0` |
| command-21 preparation | `21af4b559139e044b65556560f5fb9cbff791d71295f1a5f8377b393cfaf1a88` |
| command-21 clipped pixels | `70b6e1492c6bc7b26104c3cd4af9a7b079749433e912faea628b9012306cd7c8` |
| command-21 unclipped inputs | `c1da51a8161fdf8ea7098e65a5609ebde6bbd915b5ba2982747fd4594fc5e62d` |
| command-21 unclipped pixels | `8c553020e95e37613c4641139f4886689e4fa0f924ee418a268ca797d1d4ac8a` |
| command-21 spans | `ca0768108c4fdd9ed47d9830b462c927cae396894ab7e1891808fc4373a76a14` |
| command-21 coverage plane | `4460e056d512e3cb515b34b678c0abd001380415ac95a6d5b735ac2fcd7cb91e` |
| command-21 clip plane | `bfecf45b730bbf8d67ede3be980bbafa5930cd5e1af4d834e9214cdb9749763a` |

## Findings

The command-21 schedule is exact: initialize, 104 outer-curve patches, 99
interior-triangle vertices, 496 feather patches, then resolve. C++ and Rust
CPU span geometry and preparation checks pass. The unclipped pixel control
also passes the unchanged `2/32` contract.

The clipped comparison has 254 pixels beyond delta 2, max delta 94. Its clip
plane differs at 802 words in 283 small components, largest 15 words. Of those,
797 encode the same clip ID with C++ partial coverage and Rust full coverage.
After the existing cleared-word normalization, the coverage plane differs at
only six words, none of which explains a visible pixel. Every one of the 254
visible differences is located on a differing clip-plane word.

The full native Metal comparison remains 1,575 pixels/max delta 33, distributed
across 1,517 components with a largest component of six pixels. Only 132 of
those pixels overlap command 21; the rest have the same sparse edge topology
elsewhere in the frame. This is not an interior color, draw-order, weighted
triangle, or advanced-feather failure.

## Reproduction

Run the pinned C++ oracle into a newly created empty directory for
`direct-rewards-command16`, `direct-rewards-command21`,
`direct-rewards-command21-unclipped`, and
`direct-rewards-command21-planes`. Then run these focused Rust tests against
the fresh paths:

```sh
cargo test -p nuxie-renderer \
  configured_cpp_rewards_command_21_cpu_spans_match_rust_record_for_record \
  -- --ignored --nocapture --test-threads=1
cargo test -p nuxie-renderer \
  configured_cpp_rewards_command_21_preparation_matches_rust \
  -- --ignored --nocapture --test-threads=1
cargo test -p nuxie-renderer \
  configured_cpp_rewards_command_21_unclipped_blit_matches_rust \
  -- --ignored --nocapture --test-threads=1
```

The clipped pixel and plane tests intentionally report the measured boundary;
their outputs are the inputs to the component analysis above.
