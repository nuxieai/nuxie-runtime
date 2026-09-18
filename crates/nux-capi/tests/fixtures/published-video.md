# Published video fixture

`published-video.nux` is the Nuxie production publisher's neutral Video Frame
fixture, copied from `sdks/nuxie-ios/fixtures/video/greeting.nux` in nuxie-dev.
SHA-256: `242a0ebc242f2617d923a9e1e04a7cf01b9d5d6a8e62cc934f1f26df6efef4db`.

Regenerate in nuxie-dev with
`pnpm exec tsx tests/e2e/ios/scripts/generate-sdk-video-fixture.mts`.
The artboard is 320 × 640; video component 5 references an external 64 × 32
asset. The visibility test intentionally supplies no decoded media: decoder
admission must identify the visible occurrence before opening its decoder.
