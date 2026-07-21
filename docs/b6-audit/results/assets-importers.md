# B-6 Structural Audit — assets-importers

Pinned C++: d788e8ec6e8b598526607d6a1e8818e8b637b60c. All 36 assigned C++ files and `crates/nuxie-runtime/src/objects.rs` were read completely. The coverage clause sweep traced the functional family through the listed runtime siblings and the importer implementation in `crates/nuxie-binary/src/lib.rs`; shader payload decode was also swept in `crates/nuxie-scripting/src/shader_asset.rs`. Crate-wide searches covered generation/epoch, dirty/dirt, observed, snapshot, candidate, cache, referencer, asset-update, and import/resolve families. File-finalization stack/status fields and lookup catalogs are written only during read/build or created as local read-side catalogs, so they fail the mutation-timing gate and are recorded as AF-5 import-time constants. The pinned upstream checkout was read-only.

## B6-0098

~~~yaml
row_id: B6-0098
cpp_files: ["src/assets/audio_asset.cpp"]
rust_module: "crates/nuxie-runtime/src/objects.rs"
subsystem_cluster: assets-importers
sibling_files_swept: ["crates/nuxie-runtime/src/objects.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/draw.rs", "crates/nuxie-runtime/src/text.rs", "crates/nuxie-runtime/src/view_model.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-binary/src/lib.rs", "crates/nuxie-scripting/src/shader_asset.rs"]
verdict: UNKNOWN
axes:
  retained_identity: {status: unknown, evidence: ["src/assets/audio_asset.cpp:10-19", "crates/nuxie-binary/src/lib.rs:9807-9812"], blocker: "Audio decode/playback lifecycle has no implementation in the mapped Rust runtime; only schema/import-stack recognition is present."}
  push_vs_poll: {status: unknown, cpp_pushes: false, evidence: ["src/assets/audio_asset.cpp:10-19", "crates/nuxie-binary/src/lib.rs:9807-9812"], blocker: "No complete mapped Rust lifecycle exists for the same relationship."}
  update_ordering: {status: unknown, phases_cpp: ["import/decode", "runtime lifecycle"], phases_rust: ["import immutable descriptor"], blocker: "The corresponding runtime phase is absent from the mapped seam."}
  ownership: {status: unknown, evidence: ["src/assets/audio_asset.cpp:10-19", "crates/nuxie-binary/src/lib.rs:9807-9812"], blocker: "No complete mapped Rust owner/lifecycle exists for this row."}
  compensation:
    status: unknown
    mechanisms: []
    import_time_constants:
      - {name: "import_status/import-stack descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-binary/src/lib.rs:9306-9319", "crates/nuxie-binary/src/lib.rs:9385-9426", "crates/nuxie-binary/src/lib.rs:9729-9812"]}
idiom_rules_invoked: ["AF-5 import-time devirtualization"]
confidence: high
notes: "Audio decode/playback lifecycle has no implementation in the mapped Rust runtime; only schema/import-stack recognition is present. UNKNOWN names the blocker and makes no remediation decision."
~~~

## B6-0099

~~~yaml
row_id: B6-0099
cpp_files: ["src/assets/blob_asset.cpp"]
rust_module: "crates/nuxie-runtime/src/objects.rs"
subsystem_cluster: assets-importers
sibling_files_swept: ["crates/nuxie-runtime/src/objects.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/draw.rs", "crates/nuxie-runtime/src/text.rs", "crates/nuxie-runtime/src/view_model.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-binary/src/lib.rs", "crates/nuxie-scripting/src/shader_asset.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-1 stable arena identity; AF-7 owned payload", evidence: ["src/assets/blob_asset.cpp:5-11", "crates/nuxie-binary/src/lib.rs:419-430", "crates/nuxie-runtime/src/objects.rs:16-23", "crates/nuxie-runtime/src/objects.rs:59-75"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/assets/blob_asset.cpp:5-11", "crates/nuxie-binary/src/lib.rs:419-430"], note: "This import relationship is established once; no advance/update/bind poll, generation comparison, or rescan was found for it."}
  update_ordering: {status: isomorphic, phases_cpp: ["read/import", "resolve"], phases_rust: ["read/compute import status", "validate/apply import mutations", "build indexed runtime descriptors"]}
  ownership: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/assets/blob_asset.cpp:5-11", "crates/nuxie-binary/src/lib.rs:419-430", "crates/nuxie-binary/src/lib.rs:413-430"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "import_status/import-stack descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-binary/src/lib.rs:9306-9319", "crates/nuxie-binary/src/lib.rs:9385-9426", "crates/nuxie-binary/src/lib.rs:9729-9812"]}
      - {name: "latest importer/ordinal and lookup catalogs", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-binary/src/lib.rs:4766-4935"], note: "locals/read-side catalogs, not cycle-persistent drift trackers"}
idiom_rules_invoked: ["AF-1 retained identity via arena id", "AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "C++ moves decoded bytes into the asset; Rust retains imported contents as owned byte storage and exposes borrowed slices. The full sibling sweep found no mutation-gated compensation for this row."
~~~

## B6-0100

~~~yaml
row_id: B6-0100
cpp_files: ["src/assets/file_asset.cpp"]
rust_module: "crates/nuxie-runtime/src/objects.rs"
subsystem_cluster: assets-importers
sibling_files_swept: ["crates/nuxie-runtime/src/objects.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/draw.rs", "crates/nuxie-runtime/src/text.rs", "crates/nuxie-runtime/src/view_model.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-binary/src/lib.rs", "crates/nuxie-scripting/src/shader_asset.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-1 stable arena identity", evidence: ["src/assets/file_asset.cpp:11-24", "crates/nuxie-binary/src/lib.rs:13254-13304", "crates/nuxie-runtime/src/objects.rs:16-23", "crates/nuxie-runtime/src/objects.rs:59-75"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/assets/file_asset.cpp:11-24", "crates/nuxie-binary/src/lib.rs:13254-13304"], note: "This import relationship is established once; no advance/update/bind poll, generation comparison, or rescan was found for it."}
  update_ordering: {status: isomorphic, phases_cpp: ["read/import", "resolve"], phases_rust: ["read/compute import status", "validate/apply import mutations", "build indexed runtime descriptors"]}
  ownership: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/assets/file_asset.cpp:11-24", "crates/nuxie-binary/src/lib.rs:13254-13304", "crates/nuxie-binary/src/lib.rs:413-430"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "import_status/import-stack descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-binary/src/lib.rs:9306-9319", "crates/nuxie-binary/src/lib.rs:9385-9426", "crates/nuxie-binary/src/lib.rs:9729-9812"]}
      - {name: "latest importer/ordinal and lookup catalogs", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-binary/src/lib.rs:4766-4935"], note: "locals/read-side catalogs, not cycle-persistent drift trackers"}
idiom_rules_invoked: ["AF-1 retained identity via arena id", "AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "C++ retains the FileAsset in Backboard and normalizes duplicate asset ids; Rust owns one RuntimeObject in the file arena and performs the same normalization once at finalize. The full sibling sweep found no mutation-gated compensation for this row."
~~~

## B6-0101

~~~yaml
row_id: B6-0101
cpp_files: ["src/assets/file_asset_contents.cpp"]
rust_module: "crates/nuxie-runtime/src/objects.rs"
subsystem_cluster: assets-importers
sibling_files_swept: ["crates/nuxie-runtime/src/objects.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/draw.rs", "crates/nuxie-runtime/src/text.rs", "crates/nuxie-runtime/src/view_model.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-binary/src/lib.rs", "crates/nuxie-scripting/src/shader_asset.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/assets/file_asset_contents.cpp:8-19", "crates/nuxie-binary/src/lib.rs:4766-4827", "crates/nuxie-runtime/src/objects.rs:16-23", "crates/nuxie-runtime/src/objects.rs:59-75"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/assets/file_asset_contents.cpp:8-19", "crates/nuxie-binary/src/lib.rs:4766-4827"], note: "This import relationship is established once; no advance/update/bind poll, generation comparison, or rescan was found for it."}
  update_ordering: {status: isomorphic, phases_cpp: ["read/import", "resolve"], phases_rust: ["read/compute import status", "validate/apply import mutations", "build indexed runtime descriptors"]}
  ownership: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/assets/file_asset_contents.cpp:8-19", "crates/nuxie-binary/src/lib.rs:4766-4827", "crates/nuxie-binary/src/lib.rs:413-430"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "import_status/import-stack descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-binary/src/lib.rs:9306-9319", "crates/nuxie-binary/src/lib.rs:9385-9426", "crates/nuxie-binary/src/lib.rs:9729-9812"]}
      - {name: "latest importer/ordinal and lookup catalogs", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-binary/src/lib.rs:4766-4935"], note: "locals/read-side catalogs, not cycle-persistent drift trackers"}
idiom_rules_invoked: ["AF-1 retained identity via arena id", "AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "C++ transfers unique ownership of FileAssetContents to the current importer; Rust associates owned bytes with the latest importer-owning asset during the immutable file scan. The full sibling sweep found no mutation-gated compensation for this row."
~~~

## B6-0102

~~~yaml
row_id: B6-0102
cpp_files: ["src/assets/file_asset_referencer.cpp"]
rust_module: "crates/nuxie-runtime/src/objects.rs"
subsystem_cluster: assets-importers
sibling_files_swept: ["crates/nuxie-runtime/src/objects.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/draw.rs", "crates/nuxie-runtime/src/text.rs", "crates/nuxie-runtime/src/view_model.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-binary/src/lib.rs", "crates/nuxie-scripting/src/shader_asset.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-1 stable arena identity", evidence: ["src/assets/file_asset_referencer.cpp:16-39", "crates/nuxie-binary/src/lib.rs:4937-4958", "crates/nuxie-runtime/src/objects.rs:16-23", "crates/nuxie-runtime/src/objects.rs:59-75"]}
  push_vs_poll: {status: adapted, cpp_pushes: true, evidence: ["src/assets/file_asset_referencer.cpp:16-39", "src/assets/font_asset.cpp:16-27", "src/assets/image_asset.cpp:37-50", "crates/nuxie-binary/src/lib.rs:4937-4958"], note: "C++ registers referencers for later asset-update pushes; Rust devirtualizes the imported link to stable ids because the mapped file asset is immutable. No advance/update/bind poll, generation comparison, or rescan was found."}
  update_ordering: {status: isomorphic, phases_cpp: ["read/import", "resolve"], phases_rust: ["read/compute import status", "validate/apply import mutations", "build indexed runtime descriptors"]}
  ownership: {status: adapted, idiom_rule: "AF-7 unique children by value; AF-1 shared links by arena id", evidence: ["src/assets/file_asset_referencer.cpp:16-39", "crates/nuxie-binary/src/lib.rs:4937-4958", "crates/nuxie-binary/src/lib.rs:413-430"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "import_status/import-stack descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-binary/src/lib.rs:9306-9319", "crates/nuxie-binary/src/lib.rs:9385-9426", "crates/nuxie-binary/src/lib.rs:9729-9812"]}
      - {name: "latest importer/ordinal and lookup catalogs", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-binary/src/lib.rs:4766-4935"], note: "locals/read-side catalogs, not cycle-persistent drift trackers"}
idiom_rules_invoked: ["AF-1 retained identity via arena id", "AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "C++ registers a raw referencer and later installs an rcp asset; Rust retains the referencer and asset as stable arena objects and resolves their imported index relationship without a cycle-time rescan. The full sibling sweep found no mutation-gated compensation for this row."
~~~

## B6-0103

~~~yaml
row_id: B6-0103
cpp_files: ["src/assets/font_asset.cpp"]
rust_module: "crates/nuxie-runtime/src/objects.rs"
subsystem_cluster: assets-importers
sibling_files_swept: ["crates/nuxie-runtime/src/objects.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/draw.rs", "crates/nuxie-runtime/src/text.rs", "crates/nuxie-runtime/src/view_model.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-binary/src/lib.rs", "crates/nuxie-scripting/src/shader_asset.rs"]
verdict: UNKNOWN
axes:
  retained_identity: {status: unknown, evidence: ["src/assets/font_asset.cpp:9-27", "crates/nuxie-runtime/src/text.rs:5923-5950"], blocker: "C++ installs a mutable decoded Font and pushes TextShape dirt to registered referencers; the mapped Rust seam has immutable imported bytes plus a host snapshot API, but no asset-owned decode/referencer lifecycle to compare honestly."}
  push_vs_poll: {status: unknown, cpp_pushes: true, evidence: ["src/assets/font_asset.cpp:9-27", "crates/nuxie-runtime/src/text.rs:5923-5950"], blocker: "No complete mapped Rust lifecycle exists for the same relationship."}
  update_ordering: {status: unknown, phases_cpp: ["import/decode", "runtime lifecycle"], phases_rust: ["import immutable descriptor"], blocker: "The corresponding runtime phase is absent from the mapped seam."}
  ownership: {status: unknown, evidence: ["src/assets/font_asset.cpp:9-27", "crates/nuxie-runtime/src/text.rs:5923-5950"], blocker: "No complete mapped Rust owner/lifecycle exists for this row."}
  compensation:
    status: unknown
    mechanisms: []
    import_time_constants:
      - {name: "import_status/import-stack descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-binary/src/lib.rs:9306-9319", "crates/nuxie-binary/src/lib.rs:9385-9426", "crates/nuxie-binary/src/lib.rs:9729-9812"]}
      - {name: "external_font_assets snapshot", idiom_rule: "AF-8 explicit host replacement lifecycle", evidence: ["crates/nuxie-runtime/src/artboard.rs:1071-1105"], note: "mutated only by the explicit host API, not during advance/update/bind; does not pass the gate"}
idiom_rules_invoked: ["AF-5 import-time devirtualization", "AF-8 no invented lifecycles"]
confidence: high
notes: "C++ installs a mutable decoded Font and pushes TextShape dirt to registered referencers; the mapped Rust seam has immutable imported bytes plus a host snapshot API, but no asset-owned decode/referencer lifecycle to compare honestly. UNKNOWN names the blocker and makes no remediation decision."
~~~

## B6-0104

~~~yaml
row_id: B6-0104
cpp_files: ["src/assets/image_asset.cpp"]
rust_module: "crates/nuxie-runtime/src/objects.rs"
subsystem_cluster: assets-importers
sibling_files_swept: ["crates/nuxie-runtime/src/objects.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/draw.rs", "crates/nuxie-runtime/src/text.rs", "crates/nuxie-runtime/src/view_model.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-binary/src/lib.rs", "crates/nuxie-scripting/src/shader_asset.rs"]
verdict: UNKNOWN
axes:
  retained_identity: {status: unknown, evidence: ["src/assets/image_asset.cpp:19-50", "crates/nuxie-runtime/src/draw.rs:46-76"], blocker: "C++ owns a mutable RenderImage and pushes assetUpdated on sync/async decode; Rust builds an immutable image catalog, with decoding delegated outside the mapped runtime seam and no comparable async asset lifecycle."}
  push_vs_poll: {status: unknown, cpp_pushes: true, evidence: ["src/assets/image_asset.cpp:19-50", "crates/nuxie-runtime/src/draw.rs:46-76"], blocker: "No complete mapped Rust lifecycle exists for the same relationship."}
  update_ordering: {status: unknown, phases_cpp: ["import/decode", "runtime lifecycle"], phases_rust: ["import immutable descriptor"], blocker: "The corresponding runtime phase is absent from the mapped seam."}
  ownership: {status: unknown, evidence: ["src/assets/image_asset.cpp:19-50", "crates/nuxie-runtime/src/draw.rs:46-76"], blocker: "No complete mapped Rust owner/lifecycle exists for this row."}
  compensation:
    status: unknown
    mechanisms: []
    import_time_constants:
      - {name: "import_status/import-stack descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-binary/src/lib.rs:9306-9319", "crates/nuxie-binary/src/lib.rs:9385-9426", "crates/nuxie-binary/src/lib.rs:9729-9812"]}
idiom_rules_invoked: ["AF-5 import-time devirtualization"]
confidence: high
notes: "C++ owns a mutable RenderImage and pushes assetUpdated on sync/async decode; Rust builds an immutable image catalog, with decoding delegated outside the mapped runtime seam and no comparable async asset lifecycle. UNKNOWN names the blocker and makes no remediation decision."
~~~

## B6-0105

~~~yaml
row_id: B6-0105
cpp_files: ["src/assets/manifest_asset.cpp"]
rust_module: "crates/nuxie-runtime/src/objects.rs"
subsystem_cluster: assets-importers
sibling_files_swept: ["crates/nuxie-runtime/src/objects.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/draw.rs", "crates/nuxie-runtime/src/text.rs", "crates/nuxie-runtime/src/view_model.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-binary/src/lib.rs", "crates/nuxie-scripting/src/shader_asset.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-1 stable arena identity; AF-7 owned maps", evidence: ["src/assets/manifest_asset.cpp:12-163", "crates/nuxie-binary/src/lib.rs:13859-13945", "crates/nuxie-runtime/src/objects.rs:16-23", "crates/nuxie-runtime/src/objects.rs:59-75"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/assets/manifest_asset.cpp:12-163", "crates/nuxie-binary/src/lib.rs:13859-13945"], note: "This import relationship is established once; no advance/update/bind poll, generation comparison, or rescan was found for it."}
  update_ordering: {status: isomorphic, phases_cpp: ["read/import", "resolve"], phases_rust: ["read/compute import status", "validate/apply import mutations", "build indexed runtime descriptors"]}
  ownership: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/assets/manifest_asset.cpp:12-163", "crates/nuxie-binary/src/lib.rs:13859-13945", "crates/nuxie-binary/src/lib.rs:413-430"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "import_status/import-stack descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-binary/src/lib.rs:9306-9319", "crates/nuxie-binary/src/lib.rs:9385-9426", "crates/nuxie-binary/src/lib.rs:9729-9812"]}
      - {name: "latest importer/ordinal and lookup catalogs", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-binary/src/lib.rs:4766-4935"], note: "locals/read-side catalogs, not cycle-persistent drift trackers"}
idiom_rules_invoked: ["AF-1 retained identity via arena id", "AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "Both sides decode names and paths once and subsequently resolve by keyed lookup; Rust stores owned maps by value. The full sibling sweep found no mutation-gated compensation for this row."
~~~

## B6-0106

~~~yaml
row_id: B6-0106
cpp_files: ["src/assets/script_asset.cpp"]
rust_module: "crates/nuxie-runtime/src/objects.rs"
subsystem_cluster: assets-importers
sibling_files_swept: ["crates/nuxie-runtime/src/objects.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/draw.rs", "crates/nuxie-runtime/src/text.rs", "crates/nuxie-runtime/src/view_model.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-binary/src/lib.rs", "crates/nuxie-scripting/src/shader_asset.rs"]
verdict: UNKNOWN
axes:
  retained_identity: {status: unknown, evidence: ["src/assets/script_asset.cpp:23-221", "crates/nuxie-runtime/src/objects.rs:59-75"], blocker: "This C++ row includes VM registration, hydration, and scripted-object initialization, while the mapped objects.rs seam only stores imported fields; the separate scripting subsystem is outside this row mapping, so a complete lifecycle comparison is blocked."}
  push_vs_poll: {status: unknown, cpp_pushes: false, evidence: ["src/assets/script_asset.cpp:23-221", "crates/nuxie-runtime/src/objects.rs:59-75"], blocker: "No complete mapped Rust lifecycle exists for the same relationship."}
  update_ordering: {status: unknown, phases_cpp: ["import/decode", "runtime lifecycle"], phases_rust: ["import immutable descriptor"], blocker: "The corresponding runtime phase is absent from the mapped seam."}
  ownership: {status: unknown, evidence: ["src/assets/script_asset.cpp:23-221", "crates/nuxie-runtime/src/objects.rs:59-75"], blocker: "No complete mapped Rust owner/lifecycle exists for this row."}
  compensation:
    status: unknown
    mechanisms: []
    import_time_constants:
      - {name: "import_status/import-stack descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-binary/src/lib.rs:9306-9319", "crates/nuxie-binary/src/lib.rs:9385-9426", "crates/nuxie-binary/src/lib.rs:9729-9812"]}
idiom_rules_invoked: ["AF-5 import-time devirtualization"]
confidence: med
notes: "This C++ row includes VM registration, hydration, and scripted-object initialization, while the mapped objects.rs seam only stores imported fields; the separate scripting subsystem is outside this row mapping, so a complete lifecycle comparison is blocked. UNKNOWN names the blocker and makes no remediation decision."
~~~

## B6-0107

~~~yaml
row_id: B6-0107
cpp_files: ["src/assets/shader_asset.cpp"]
rust_module: "crates/nuxie-runtime/src/objects.rs"
subsystem_cluster: assets-importers
sibling_files_swept: ["crates/nuxie-runtime/src/objects.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/draw.rs", "crates/nuxie-runtime/src/text.rs", "crates/nuxie-runtime/src/view_model.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-binary/src/lib.rs", "crates/nuxie-scripting/src/shader_asset.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-1 stable arena identity; AF-7 owned decoded payload", evidence: ["src/assets/shader_asset.cpp:6-125", "crates/nuxie-scripting/src/shader_asset.rs:80-223", "crates/nuxie-runtime/src/objects.rs:16-23", "crates/nuxie-runtime/src/objects.rs:59-75"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/assets/shader_asset.cpp:6-125", "crates/nuxie-scripting/src/shader_asset.rs:80-223"], note: "This import relationship is established once; no advance/update/bind poll, generation comparison, or rescan was found for it."}
  update_ordering: {status: isomorphic, phases_cpp: ["read/import", "resolve"], phases_rust: ["read/compute import status", "validate/apply import mutations", "build indexed runtime descriptors"]}
  ownership: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/assets/shader_asset.cpp:6-125", "crates/nuxie-scripting/src/shader_asset.rs:80-223", "crates/nuxie-binary/src/lib.rs:413-430"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "import_status/import-stack descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-binary/src/lib.rs:9306-9319", "crates/nuxie-binary/src/lib.rs:9385-9426", "crates/nuxie-binary/src/lib.rs:9729-9812"]}
      - {name: "latest importer/ordinal and lookup catalogs", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-binary/src/lib.rs:4766-4935"], note: "locals/read-side catalogs, not cycle-persistent drift trackers"}
idiom_rules_invoked: ["AF-1 retained identity via arena id", "AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "C++ and Rust decode immutable shader payload descriptors once; Rust's bounded decoder owns the decoded stages by value. The full sibling sweep found no mutation-gated compensation for this row."
~~~

## B6-0212

~~~yaml
row_id: B6-0212
cpp_files: ["src/importers/artboard_importer.cpp"]
rust_module: "crates/nuxie-runtime/src/objects.rs"
subsystem_cluster: assets-importers
sibling_files_swept: ["crates/nuxie-runtime/src/objects.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/draw.rs", "crates/nuxie-runtime/src/text.rs", "crates/nuxie-runtime/src/view_model.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-binary/src/lib.rs", "crates/nuxie-scripting/src/shader_asset.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-1 stable arena identity", evidence: ["src/importers/artboard_importer.cpp:12-46", "crates/nuxie-runtime/src/artboard.rs:835-854", "crates/nuxie-runtime/src/objects.rs:16-23", "crates/nuxie-runtime/src/objects.rs:59-75"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/importers/artboard_importer.cpp:12-46", "crates/nuxie-runtime/src/artboard.rs:835-854"], note: "This import relationship is established once; no advance/update/bind poll, generation comparison, or rescan was found for it."}
  update_ordering: {status: isomorphic, phases_cpp: ["read/import", "resolve"], phases_rust: ["read/compute import status", "validate/apply import mutations", "build indexed runtime descriptors"]}
  ownership: {status: adapted, idiom_rule: "AF-7 unique children by value; AF-1 shared links by arena id", evidence: ["src/importers/artboard_importer.cpp:12-46", "crates/nuxie-runtime/src/artboard.rs:835-854", "crates/nuxie-binary/src/lib.rs:413-430"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "import_status/import-stack descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-binary/src/lib.rs:9306-9319", "crates/nuxie-binary/src/lib.rs:9385-9426", "crates/nuxie-binary/src/lib.rs:9729-9812"]}
      - {name: "latest importer/ordinal and lookup catalogs", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-binary/src/lib.rs:4766-4935"], note: "locals/read-side catalogs, not cycle-persistent drift trackers"}
idiom_rules_invoked: ["AF-1 retained identity via arena id", "AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "Artboard child membership and resolve ordering are represented by stable file/local ids and owned vectors built from the finalized object stream. The full sibling sweep found no mutation-gated compensation for this row."
~~~

## B6-0213

~~~yaml
row_id: B6-0213
cpp_files: ["src/importers/backboard_importer.cpp"]
rust_module: "crates/nuxie-runtime/src/objects.rs"
subsystem_cluster: assets-importers
sibling_files_swept: ["crates/nuxie-runtime/src/objects.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/draw.rs", "crates/nuxie-runtime/src/text.rs", "crates/nuxie-runtime/src/view_model.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-binary/src/lib.rs", "crates/nuxie-scripting/src/shader_asset.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-1 stable arena identity", evidence: ["src/importers/backboard_importer.cpp:23-192", "crates/nuxie-binary/src/lib.rs:9306-9319", "crates/nuxie-runtime/src/objects.rs:16-23", "crates/nuxie-runtime/src/objects.rs:59-75"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/importers/backboard_importer.cpp:23-192", "crates/nuxie-binary/src/lib.rs:9306-9319"], note: "This import relationship is established once; no advance/update/bind poll, generation comparison, or rescan was found for it."}
  update_ordering: {status: isomorphic, phases_cpp: ["read/import", "resolve"], phases_rust: ["read/compute import status", "validate/apply import mutations", "build indexed runtime descriptors"]}
  ownership: {status: adapted, idiom_rule: "AF-7 unique children by value; AF-1 shared links by arena id", evidence: ["src/importers/backboard_importer.cpp:23-192", "crates/nuxie-binary/src/lib.rs:9306-9319", "crates/nuxie-binary/src/lib.rs:413-430"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "import_status/import-stack descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-binary/src/lib.rs:9306-9319", "crates/nuxie-binary/src/lib.rs:9385-9426", "crates/nuxie-binary/src/lib.rs:9729-9812"]}
      - {name: "latest importer/ordinal and lookup catalogs", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-binary/src/lib.rs:4766-4935"], note: "locals/read-side catalogs, not cycle-persistent drift trackers"}
idiom_rules_invoked: ["AF-1 retained identity via arena id", "AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "Backboard importer collections and link resolution are folded into the one-time import-status/normalization pass and stable arena lookups. The full sibling sweep found no mutation-gated compensation for this row."
~~~

## B6-0214

~~~yaml
row_id: B6-0214
cpp_files: ["src/importers/bindable_property_importer.cpp"]
rust_module: "crates/nuxie-runtime/src/objects.rs"
subsystem_cluster: assets-importers
sibling_files_swept: ["crates/nuxie-runtime/src/objects.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/draw.rs", "crates/nuxie-runtime/src/text.rs", "crates/nuxie-runtime/src/view_model.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-binary/src/lib.rs", "crates/nuxie-scripting/src/shader_asset.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-1 stable arena identity", evidence: ["src/importers/bindable_property_importer.cpp:7-9", "crates/nuxie-binary/src/lib.rs:9794-9799", "crates/nuxie-runtime/src/objects.rs:16-23", "crates/nuxie-runtime/src/objects.rs:59-75"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/importers/bindable_property_importer.cpp:7-9", "crates/nuxie-binary/src/lib.rs:9794-9799"], note: "This import relationship is established once; no advance/update/bind poll, generation comparison, or rescan was found for it."}
  update_ordering: {status: isomorphic, phases_cpp: ["read/import", "resolve"], phases_rust: ["read/compute import status", "validate/apply import mutations", "build indexed runtime descriptors"]}
  ownership: {status: adapted, idiom_rule: "AF-7 unique children by value; AF-1 shared links by arena id", evidence: ["src/importers/bindable_property_importer.cpp:7-9", "crates/nuxie-binary/src/lib.rs:9794-9799", "crates/nuxie-binary/src/lib.rs:413-430"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "import_status/import-stack descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-binary/src/lib.rs:9306-9319", "crates/nuxie-binary/src/lib.rs:9385-9426", "crates/nuxie-binary/src/lib.rs:9729-9812"]}
      - {name: "latest importer/ordinal and lookup catalogs", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-binary/src/lib.rs:4766-4935"], note: "locals/read-side catalogs, not cycle-persistent drift trackers"}
idiom_rules_invoked: ["AF-1 retained identity via arena id", "AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "The constructor-only importer context becomes a build-time type/index discriminant in the finalized Rust object graph. The full sibling sweep found no mutation-gated compensation for this row."
~~~

## B6-0215

~~~yaml
row_id: B6-0215
cpp_files: ["src/importers/data_bind_path_importer.cpp"]
rust_module: "crates/nuxie-runtime/src/objects.rs"
subsystem_cluster: assets-importers
sibling_files_swept: ["crates/nuxie-runtime/src/objects.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/draw.rs", "crates/nuxie-runtime/src/text.rs", "crates/nuxie-runtime/src/view_model.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-binary/src/lib.rs", "crates/nuxie-scripting/src/shader_asset.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-1 stable arena identity", evidence: ["src/importers/data_bind_path_importer.cpp:6-14", "crates/nuxie-binary/src/lib.rs:9797-9799", "crates/nuxie-runtime/src/objects.rs:16-23", "crates/nuxie-runtime/src/objects.rs:59-75"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/importers/data_bind_path_importer.cpp:6-14", "crates/nuxie-binary/src/lib.rs:9797-9799"], note: "This import relationship is established once; no advance/update/bind poll, generation comparison, or rescan was found for it."}
  update_ordering: {status: isomorphic, phases_cpp: ["read/import", "resolve"], phases_rust: ["read/compute import status", "validate/apply import mutations", "build indexed runtime descriptors"]}
  ownership: {status: adapted, idiom_rule: "AF-7 unique children by value; AF-1 shared links by arena id", evidence: ["src/importers/data_bind_path_importer.cpp:6-14", "crates/nuxie-binary/src/lib.rs:9797-9799", "crates/nuxie-binary/src/lib.rs:413-430"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "import_status/import-stack descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-binary/src/lib.rs:9306-9319", "crates/nuxie-binary/src/lib.rs:9385-9426", "crates/nuxie-binary/src/lib.rs:9729-9812"]}
      - {name: "latest importer/ordinal and lookup catalogs", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-binary/src/lib.rs:4766-4935"], note: "locals/read-side catalogs, not cycle-persistent drift trackers"}
idiom_rules_invoked: ["AF-1 retained identity via arena id", "AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "C++ claims the path object from its importer; Rust records the latest path context during import and later owns the path descriptor by value. The full sibling sweep found no mutation-gated compensation for this row."
~~~

## B6-0216

~~~yaml
row_id: B6-0216
cpp_files: ["src/importers/data_converter_formula_importer.cpp"]
rust_module: "crates/nuxie-runtime/src/objects.rs"
subsystem_cluster: assets-importers
sibling_files_swept: ["crates/nuxie-runtime/src/objects.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/draw.rs", "crates/nuxie-runtime/src/text.rs", "crates/nuxie-runtime/src/view_model.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-binary/src/lib.rs", "crates/nuxie-scripting/src/shader_asset.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-1 stable arena identity", evidence: ["src/importers/data_converter_formula_importer.cpp:7-15", "crates/nuxie-binary/src/lib.rs:9762-9764", "crates/nuxie-runtime/src/objects.rs:16-23", "crates/nuxie-runtime/src/objects.rs:59-75"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/importers/data_converter_formula_importer.cpp:7-15", "crates/nuxie-binary/src/lib.rs:9762-9764"], note: "This import relationship is established once; no advance/update/bind poll, generation comparison, or rescan was found for it."}
  update_ordering: {status: isomorphic, phases_cpp: ["read/import", "resolve"], phases_rust: ["read/compute import status", "validate/apply import mutations", "build indexed runtime descriptors"]}
  ownership: {status: adapted, idiom_rule: "AF-7 unique children by value; AF-1 shared links by arena id", evidence: ["src/importers/data_converter_formula_importer.cpp:7-15", "crates/nuxie-binary/src/lib.rs:9762-9764", "crates/nuxie-binary/src/lib.rs:413-430"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "import_status/import-stack descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-binary/src/lib.rs:9306-9319", "crates/nuxie-binary/src/lib.rs:9385-9426", "crates/nuxie-binary/src/lib.rs:9729-9812"]}
      - {name: "latest importer/ordinal and lookup catalogs", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-binary/src/lib.rs:4766-4935"], note: "locals/read-side catalogs, not cycle-persistent drift trackers"}
idiom_rules_invoked: ["AF-1 retained identity via arena id", "AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "Formula importer resolution is folded into the finalized import graph and owned formula/token descriptors. The full sibling sweep found no mutation-gated compensation for this row."
~~~

## B6-0217

~~~yaml
row_id: B6-0217
cpp_files: ["src/importers/data_converter_group_importer.cpp"]
rust_module: "crates/nuxie-runtime/src/objects.rs"
subsystem_cluster: assets-importers
sibling_files_swept: ["crates/nuxie-runtime/src/objects.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/draw.rs", "crates/nuxie-runtime/src/text.rs", "crates/nuxie-runtime/src/view_model.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-binary/src/lib.rs", "crates/nuxie-scripting/src/shader_asset.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-1 stable arena identity", evidence: ["src/importers/data_converter_group_importer.cpp:7-9", "crates/nuxie-binary/src/lib.rs:9761-9764", "crates/nuxie-runtime/src/objects.rs:16-23", "crates/nuxie-runtime/src/objects.rs:59-75"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/importers/data_converter_group_importer.cpp:7-9", "crates/nuxie-binary/src/lib.rs:9761-9764"], note: "This import relationship is established once; no advance/update/bind poll, generation comparison, or rescan was found for it."}
  update_ordering: {status: isomorphic, phases_cpp: ["read/import", "resolve"], phases_rust: ["read/compute import status", "validate/apply import mutations", "build indexed runtime descriptors"]}
  ownership: {status: adapted, idiom_rule: "AF-7 unique children by value; AF-1 shared links by arena id", evidence: ["src/importers/data_converter_group_importer.cpp:7-9", "crates/nuxie-binary/src/lib.rs:9761-9764", "crates/nuxie-binary/src/lib.rs:413-430"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "import_status/import-stack descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-binary/src/lib.rs:9306-9319", "crates/nuxie-binary/src/lib.rs:9385-9426", "crates/nuxie-binary/src/lib.rs:9729-9812"]}
      - {name: "latest importer/ordinal and lookup catalogs", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-binary/src/lib.rs:4766-4935"], note: "locals/read-side catalogs, not cycle-persistent drift trackers"}
idiom_rules_invoked: ["AF-1 retained identity via arena id", "AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "The constructor-only group context is represented by import-time stack state and owned group items. The full sibling sweep found no mutation-gated compensation for this row."
~~~

## B6-0218

~~~yaml
row_id: B6-0218
cpp_files: ["src/importers/enum_importer.cpp"]
rust_module: "crates/nuxie-runtime/src/objects.rs"
subsystem_cluster: assets-importers
sibling_files_swept: ["crates/nuxie-runtime/src/objects.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/draw.rs", "crates/nuxie-runtime/src/text.rs", "crates/nuxie-runtime/src/view_model.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-binary/src/lib.rs", "crates/nuxie-scripting/src/shader_asset.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-1 stable arena identity", evidence: ["src/importers/enum_importer.cpp:7-14", "crates/nuxie-binary/src/lib.rs:9761-9762", "crates/nuxie-runtime/src/objects.rs:16-23", "crates/nuxie-runtime/src/objects.rs:59-75"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/importers/enum_importer.cpp:7-14", "crates/nuxie-binary/src/lib.rs:9761-9762"], note: "This import relationship is established once; no advance/update/bind poll, generation comparison, or rescan was found for it."}
  update_ordering: {status: isomorphic, phases_cpp: ["read/import", "resolve"], phases_rust: ["read/compute import status", "validate/apply import mutations", "build indexed runtime descriptors"]}
  ownership: {status: adapted, idiom_rule: "AF-7 unique children by value; AF-1 shared links by arena id", evidence: ["src/importers/enum_importer.cpp:7-14", "crates/nuxie-binary/src/lib.rs:9761-9762", "crates/nuxie-binary/src/lib.rs:413-430"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "import_status/import-stack descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-binary/src/lib.rs:9306-9319", "crates/nuxie-binary/src/lib.rs:9385-9426", "crates/nuxie-binary/src/lib.rs:9729-9812"]}
      - {name: "latest importer/ordinal and lookup catalogs", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-binary/src/lib.rs:4766-4935"], note: "locals/read-side catalogs, not cycle-persistent drift trackers"}
idiom_rules_invoked: ["AF-1 retained identity via arena id", "AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "C++ appends raw value pointers owned by the enum graph; Rust keeps values as stable imported objects and derives owned/indexed collections after import. The full sibling sweep found no mutation-gated compensation for this row."
~~~

## B6-0219

~~~yaml
row_id: B6-0219
cpp_files: ["src/importers/file_asset_importer.cpp"]
rust_module: "crates/nuxie-runtime/src/objects.rs"
subsystem_cluster: assets-importers
sibling_files_swept: ["crates/nuxie-runtime/src/objects.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/draw.rs", "crates/nuxie-runtime/src/text.rs", "crates/nuxie-runtime/src/view_model.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-binary/src/lib.rs", "crates/nuxie-scripting/src/shader_asset.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-1 stable asset id; AF-7 owned contents", evidence: ["src/importers/file_asset_importer.cpp:10-51", "crates/nuxie-binary/src/lib.rs:4766-4935", "crates/nuxie-runtime/src/objects.rs:16-23", "crates/nuxie-runtime/src/objects.rs:59-75"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/importers/file_asset_importer.cpp:10-51", "crates/nuxie-binary/src/lib.rs:4766-4935"], note: "This import relationship is established once; no advance/update/bind poll, generation comparison, or rescan was found for it."}
  update_ordering: {status: isomorphic, phases_cpp: ["read/import", "resolve"], phases_rust: ["read/compute import status", "validate/apply import mutations", "build indexed runtime descriptors"]}
  ownership: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/importers/file_asset_importer.cpp:10-51", "crates/nuxie-binary/src/lib.rs:4766-4935", "crates/nuxie-binary/src/lib.rs:413-430"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "import_status/import-stack descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-binary/src/lib.rs:9306-9319", "crates/nuxie-binary/src/lib.rs:9385-9426", "crates/nuxie-binary/src/lib.rs:9729-9812"]}
      - {name: "latest importer/ordinal and lookup catalogs", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-binary/src/lib.rs:4766-4935"], note: "locals/read-side catalogs, not cycle-persistent drift trackers"}
idiom_rules_invoked: ["AF-1 retained identity via arena id", "AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "C++ temporarily owns the latest contents and resolves loader/decode once; Rust associates contents to the latest importer-owning asset in a one-time scan. The full sibling sweep found no mutation-gated compensation for this row."
~~~

## B6-0220

~~~yaml
row_id: B6-0220
cpp_files: ["src/importers/keyed_object_importer.cpp"]
rust_module: "crates/nuxie-runtime/src/objects.rs"
subsystem_cluster: assets-importers
sibling_files_swept: ["crates/nuxie-runtime/src/objects.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/draw.rs", "crates/nuxie-runtime/src/text.rs", "crates/nuxie-runtime/src/view_model.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-binary/src/lib.rs", "crates/nuxie-scripting/src/shader_asset.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-1 stable arena identity", evidence: ["src/importers/keyed_object_importer.cpp:8-15", "crates/nuxie-binary/src/lib.rs:9404-9406", "crates/nuxie-runtime/src/objects.rs:16-23", "crates/nuxie-runtime/src/objects.rs:59-75"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/importers/keyed_object_importer.cpp:8-15", "crates/nuxie-binary/src/lib.rs:9404-9406"], note: "This import relationship is established once; no advance/update/bind poll, generation comparison, or rescan was found for it."}
  update_ordering: {status: isomorphic, phases_cpp: ["read/import", "resolve"], phases_rust: ["read/compute import status", "validate/apply import mutations", "build indexed runtime descriptors"]}
  ownership: {status: adapted, idiom_rule: "AF-7 unique children by value; AF-1 shared links by arena id", evidence: ["src/importers/keyed_object_importer.cpp:8-15", "crates/nuxie-binary/src/lib.rs:9404-9406", "crates/nuxie-binary/src/lib.rs:413-430"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "import_status/import-stack descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-binary/src/lib.rs:9306-9319", "crates/nuxie-binary/src/lib.rs:9385-9426", "crates/nuxie-binary/src/lib.rs:9729-9812"]}
      - {name: "latest importer/ordinal and lookup catalogs", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-binary/src/lib.rs:4766-4935"], note: "locals/read-side catalogs, not cycle-persistent drift trackers"}
idiom_rules_invoked: ["AF-1 retained identity via arena id", "AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "C++ transfers unique_ptr KeyedProperty values; Rust owns equivalent keyed-property descriptors by value. The full sibling sweep found no mutation-gated compensation for this row."
~~~

## B6-0221

~~~yaml
row_id: B6-0221
cpp_files: ["src/importers/keyed_property_importer.cpp"]
rust_module: "crates/nuxie-runtime/src/objects.rs"
subsystem_cluster: assets-importers
sibling_files_swept: ["crates/nuxie-runtime/src/objects.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/draw.rs", "crates/nuxie-runtime/src/text.rs", "crates/nuxie-runtime/src/view_model.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-binary/src/lib.rs", "crates/nuxie-scripting/src/shader_asset.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-1 stable arena identity", evidence: ["src/importers/keyed_property_importer.cpp:8-23", "crates/nuxie-binary/src/lib.rs:9435-9441", "crates/nuxie-runtime/src/objects.rs:16-23", "crates/nuxie-runtime/src/objects.rs:59-75"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/importers/keyed_property_importer.cpp:8-23", "crates/nuxie-binary/src/lib.rs:9435-9441"], note: "This import relationship is established once; no advance/update/bind poll, generation comparison, or rescan was found for it."}
  update_ordering: {status: isomorphic, phases_cpp: ["read/import", "resolve"], phases_rust: ["read/compute import status", "validate/apply import mutations", "build indexed runtime descriptors"]}
  ownership: {status: adapted, idiom_rule: "AF-7 unique children by value; AF-1 shared links by arena id", evidence: ["src/importers/keyed_property_importer.cpp:8-23", "crates/nuxie-binary/src/lib.rs:9435-9441", "crates/nuxie-binary/src/lib.rs:413-430"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "import_status/import-stack descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-binary/src/lib.rs:9306-9319", "crates/nuxie-binary/src/lib.rs:9385-9426", "crates/nuxie-binary/src/lib.rs:9729-9812"]}
      - {name: "latest importer/ordinal and lookup catalogs", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-binary/src/lib.rs:4766-4935"], note: "locals/read-side catalogs, not cycle-persistent drift trackers"}
idiom_rules_invoked: ["AF-1 retained identity via arena id", "AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "C++ transfers unique_ptr keyframes and ignores null frames; Rust owns imported keyframe descriptors by value and records null/import status once. The full sibling sweep found no mutation-gated compensation for this row."
~~~

## B6-0222

~~~yaml
row_id: B6-0222
cpp_files: ["src/importers/layer_state_importer.cpp"]
rust_module: "crates/nuxie-runtime/src/objects.rs"
subsystem_cluster: assets-importers
sibling_files_swept: ["crates/nuxie-runtime/src/objects.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/draw.rs", "crates/nuxie-runtime/src/text.rs", "crates/nuxie-runtime/src/view_model.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-binary/src/lib.rs", "crates/nuxie-scripting/src/shader_asset.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-1 stable arena identity", evidence: ["src/importers/layer_state_importer.cpp:9-49", "crates/nuxie-binary/src/lib.rs:9677-9683", "crates/nuxie-runtime/src/objects.rs:16-23", "crates/nuxie-runtime/src/objects.rs:59-75"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/importers/layer_state_importer.cpp:9-49", "crates/nuxie-binary/src/lib.rs:9677-9683"], note: "This import relationship is established once; no advance/update/bind poll, generation comparison, or rescan was found for it."}
  update_ordering: {status: isomorphic, phases_cpp: ["read/import", "resolve"], phases_rust: ["read/compute import status", "validate/apply import mutations", "build indexed runtime descriptors"]}
  ownership: {status: adapted, idiom_rule: "AF-7 unique children by value; AF-1 shared links by arena id", evidence: ["src/importers/layer_state_importer.cpp:9-49", "crates/nuxie-binary/src/lib.rs:9677-9683", "crates/nuxie-binary/src/lib.rs:413-430"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "import_status/import-stack descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-binary/src/lib.rs:9306-9319", "crates/nuxie-binary/src/lib.rs:9385-9426", "crates/nuxie-binary/src/lib.rs:9729-9812"]}
      - {name: "latest importer/ordinal and lookup catalogs", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-binary/src/lib.rs:4766-4935"], note: "locals/read-side catalogs, not cycle-persistent drift trackers"}
idiom_rules_invoked: ["AF-1 retained identity via arena id", "AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "Transition and blend-animation links are resolved once from stable ids, replacing C++ raw-pointer attachment without cycle-time reconstruction. The full sibling sweep found no mutation-gated compensation for this row."
~~~

## B6-0223

~~~yaml
row_id: B6-0223
cpp_files: ["src/importers/linear_animation_importer.cpp"]
rust_module: "crates/nuxie-runtime/src/objects.rs"
subsystem_cluster: assets-importers
sibling_files_swept: ["crates/nuxie-runtime/src/objects.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/draw.rs", "crates/nuxie-runtime/src/text.rs", "crates/nuxie-runtime/src/view_model.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-binary/src/lib.rs", "crates/nuxie-scripting/src/shader_asset.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-1 stable arena identity", evidence: ["src/importers/linear_animation_importer.cpp:8-15", "crates/nuxie-binary/src/lib.rs:9741-9743", "crates/nuxie-runtime/src/objects.rs:16-23", "crates/nuxie-runtime/src/objects.rs:59-75"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/importers/linear_animation_importer.cpp:8-15", "crates/nuxie-binary/src/lib.rs:9741-9743"], note: "This import relationship is established once; no advance/update/bind poll, generation comparison, or rescan was found for it."}
  update_ordering: {status: isomorphic, phases_cpp: ["read/import", "resolve"], phases_rust: ["read/compute import status", "validate/apply import mutations", "build indexed runtime descriptors"]}
  ownership: {status: adapted, idiom_rule: "AF-7 unique children by value; AF-1 shared links by arena id", evidence: ["src/importers/linear_animation_importer.cpp:8-15", "crates/nuxie-binary/src/lib.rs:9741-9743", "crates/nuxie-binary/src/lib.rs:413-430"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "import_status/import-stack descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-binary/src/lib.rs:9306-9319", "crates/nuxie-binary/src/lib.rs:9385-9426", "crates/nuxie-binary/src/lib.rs:9729-9812"]}
      - {name: "latest importer/ordinal and lookup catalogs", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-binary/src/lib.rs:4766-4935"], note: "locals/read-side catalogs, not cycle-persistent drift trackers"}
idiom_rules_invoked: ["AF-1 retained identity via arena id", "AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "C++ transfers unique_ptr KeyedObject values; Rust owns animation descriptors by value. The full sibling sweep found no mutation-gated compensation for this row."
~~~

## B6-0224

~~~yaml
row_id: B6-0224
cpp_files: ["src/importers/listener_input_type_gamepad_importer.cpp"]
rust_module: "crates/nuxie-runtime/src/objects.rs"
subsystem_cluster: assets-importers
sibling_files_swept: ["crates/nuxie-runtime/src/objects.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/draw.rs", "crates/nuxie-runtime/src/text.rs", "crates/nuxie-runtime/src/view_model.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-binary/src/lib.rs", "crates/nuxie-scripting/src/shader_asset.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-1 stable arena identity", evidence: ["src/importers/listener_input_type_gamepad_importer.cpp:5-13", "crates/nuxie-binary/src/lib.rs:9749-9751", "crates/nuxie-runtime/src/objects.rs:16-23", "crates/nuxie-runtime/src/objects.rs:59-75"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/importers/listener_input_type_gamepad_importer.cpp:5-13", "crates/nuxie-binary/src/lib.rs:9749-9751"], note: "This import relationship is established once; no advance/update/bind poll, generation comparison, or rescan was found for it."}
  update_ordering: {status: isomorphic, phases_cpp: ["read/import", "resolve"], phases_rust: ["read/compute import status", "validate/apply import mutations", "build indexed runtime descriptors"]}
  ownership: {status: adapted, idiom_rule: "AF-7 unique children by value; AF-1 shared links by arena id", evidence: ["src/importers/listener_input_type_gamepad_importer.cpp:5-13", "crates/nuxie-binary/src/lib.rs:9749-9751", "crates/nuxie-binary/src/lib.rs:413-430"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "import_status/import-stack descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-binary/src/lib.rs:9306-9319", "crates/nuxie-binary/src/lib.rs:9385-9426", "crates/nuxie-binary/src/lib.rs:9729-9812"]}
      - {name: "latest importer/ordinal and lookup catalogs", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-binary/src/lib.rs:4766-4935"], note: "locals/read-side catalogs, not cycle-persistent drift trackers"}
idiom_rules_invoked: ["AF-1 retained identity via arena id", "AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "Resolve-time input linking is folded into the import-time listener-input context and stable ids. The full sibling sweep found no mutation-gated compensation for this row."
~~~

## B6-0225

~~~yaml
row_id: B6-0225
cpp_files: ["src/importers/listener_input_type_keyboard_importer.cpp"]
rust_module: "crates/nuxie-runtime/src/objects.rs"
subsystem_cluster: assets-importers
sibling_files_swept: ["crates/nuxie-runtime/src/objects.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/draw.rs", "crates/nuxie-runtime/src/text.rs", "crates/nuxie-runtime/src/view_model.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-binary/src/lib.rs", "crates/nuxie-scripting/src/shader_asset.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-1 stable arena identity", evidence: ["src/importers/listener_input_type_keyboard_importer.cpp:5-13", "crates/nuxie-binary/src/lib.rs:9752-9754", "crates/nuxie-runtime/src/objects.rs:16-23", "crates/nuxie-runtime/src/objects.rs:59-75"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/importers/listener_input_type_keyboard_importer.cpp:5-13", "crates/nuxie-binary/src/lib.rs:9752-9754"], note: "This import relationship is established once; no advance/update/bind poll, generation comparison, or rescan was found for it."}
  update_ordering: {status: isomorphic, phases_cpp: ["read/import", "resolve"], phases_rust: ["read/compute import status", "validate/apply import mutations", "build indexed runtime descriptors"]}
  ownership: {status: adapted, idiom_rule: "AF-7 unique children by value; AF-1 shared links by arena id", evidence: ["src/importers/listener_input_type_keyboard_importer.cpp:5-13", "crates/nuxie-binary/src/lib.rs:9752-9754", "crates/nuxie-binary/src/lib.rs:413-430"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "import_status/import-stack descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-binary/src/lib.rs:9306-9319", "crates/nuxie-binary/src/lib.rs:9385-9426", "crates/nuxie-binary/src/lib.rs:9729-9812"]}
      - {name: "latest importer/ordinal and lookup catalogs", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-binary/src/lib.rs:4766-4935"], note: "locals/read-side catalogs, not cycle-persistent drift trackers"}
idiom_rules_invoked: ["AF-1 retained identity via arena id", "AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "Resolve-time input linking is folded into the import-time listener-input context and stable ids. The full sibling sweep found no mutation-gated compensation for this row."
~~~

## B6-0226

~~~yaml
row_id: B6-0226
cpp_files: ["src/importers/listener_input_type_semantic_importer.cpp"]
rust_module: "crates/nuxie-runtime/src/objects.rs"
subsystem_cluster: assets-importers
sibling_files_swept: ["crates/nuxie-runtime/src/objects.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/draw.rs", "crates/nuxie-runtime/src/text.rs", "crates/nuxie-runtime/src/view_model.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-binary/src/lib.rs", "crates/nuxie-scripting/src/shader_asset.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-1 stable arena identity", evidence: ["src/importers/listener_input_type_semantic_importer.cpp:5-13", "crates/nuxie-binary/src/lib.rs:9755-9757", "crates/nuxie-runtime/src/objects.rs:16-23", "crates/nuxie-runtime/src/objects.rs:59-75"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/importers/listener_input_type_semantic_importer.cpp:5-13", "crates/nuxie-binary/src/lib.rs:9755-9757"], note: "This import relationship is established once; no advance/update/bind poll, generation comparison, or rescan was found for it."}
  update_ordering: {status: isomorphic, phases_cpp: ["read/import", "resolve"], phases_rust: ["read/compute import status", "validate/apply import mutations", "build indexed runtime descriptors"]}
  ownership: {status: adapted, idiom_rule: "AF-7 unique children by value; AF-1 shared links by arena id", evidence: ["src/importers/listener_input_type_semantic_importer.cpp:5-13", "crates/nuxie-binary/src/lib.rs:9755-9757", "crates/nuxie-binary/src/lib.rs:413-430"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "import_status/import-stack descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-binary/src/lib.rs:9306-9319", "crates/nuxie-binary/src/lib.rs:9385-9426", "crates/nuxie-binary/src/lib.rs:9729-9812"]}
      - {name: "latest importer/ordinal and lookup catalogs", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-binary/src/lib.rs:4766-4935"], note: "locals/read-side catalogs, not cycle-persistent drift trackers"}
idiom_rules_invoked: ["AF-1 retained identity via arena id", "AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "Resolve-time input linking is folded into the import-time listener-input context and stable ids. The full sibling sweep found no mutation-gated compensation for this row."
~~~

## B6-0227

~~~yaml
row_id: B6-0227
cpp_files: ["src/importers/scripted_object_importer.cpp"]
rust_module: "crates/nuxie-runtime/src/objects.rs"
subsystem_cluster: assets-importers
sibling_files_swept: ["crates/nuxie-runtime/src/objects.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/draw.rs", "crates/nuxie-runtime/src/text.rs", "crates/nuxie-runtime/src/view_model.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-binary/src/lib.rs", "crates/nuxie-scripting/src/shader_asset.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-1 stable arena identity", evidence: ["src/importers/scripted_object_importer.cpp:9-22", "crates/nuxie-binary/src/lib.rs:9800-9802", "crates/nuxie-runtime/src/objects.rs:16-23", "crates/nuxie-runtime/src/objects.rs:59-75"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/importers/scripted_object_importer.cpp:9-22", "crates/nuxie-binary/src/lib.rs:9800-9802"], note: "This import relationship is established once; no advance/update/bind poll, generation comparison, or rescan was found for it."}
  update_ordering: {status: isomorphic, phases_cpp: ["read/import", "resolve"], phases_rust: ["read/compute import status", "validate/apply import mutations", "build indexed runtime descriptors"]}
  ownership: {status: adapted, idiom_rule: "AF-7 unique children by value; AF-1 shared links by arena id", evidence: ["src/importers/scripted_object_importer.cpp:9-22", "crates/nuxie-binary/src/lib.rs:9800-9802", "crates/nuxie-binary/src/lib.rs:413-430"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "import_status/import-stack descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-binary/src/lib.rs:9306-9319", "crates/nuxie-binary/src/lib.rs:9385-9426", "crates/nuxie-binary/src/lib.rs:9729-9812"]}
      - {name: "latest importer/ordinal and lookup catalogs", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-binary/src/lib.rs:4766-4935"], note: "locals/read-side catalogs, not cycle-persistent drift trackers"}
idiom_rules_invoked: ["AF-1 retained identity via arena id", "AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "C++ appends input property pointers during import; Rust records scripted-object context once and owns/imports descriptors by stable id. The full sibling sweep found no mutation-gated compensation for this row."
~~~

## B6-0228

~~~yaml
row_id: B6-0228
cpp_files: ["src/importers/state_machine_importer.cpp"]
rust_module: "crates/nuxie-runtime/src/objects.rs"
subsystem_cluster: assets-importers
sibling_files_swept: ["crates/nuxie-runtime/src/objects.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/draw.rs", "crates/nuxie-runtime/src/text.rs", "crates/nuxie-runtime/src/view_model.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-binary/src/lib.rs", "crates/nuxie-scripting/src/shader_asset.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-1 stable arena identity", evidence: ["src/importers/state_machine_importer.cpp:10-47", "crates/nuxie-binary/src/lib.rs:9744-9747", "crates/nuxie-runtime/src/objects.rs:16-23", "crates/nuxie-runtime/src/objects.rs:59-75"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/importers/state_machine_importer.cpp:10-47", "crates/nuxie-binary/src/lib.rs:9744-9747"], note: "This import relationship is established once; no advance/update/bind poll, generation comparison, or rescan was found for it."}
  update_ordering: {status: isomorphic, phases_cpp: ["read/import", "resolve"], phases_rust: ["read/compute import status", "validate/apply import mutations", "build indexed runtime descriptors"]}
  ownership: {status: adapted, idiom_rule: "AF-7 unique children by value; AF-1 shared links by arena id", evidence: ["src/importers/state_machine_importer.cpp:10-47", "crates/nuxie-binary/src/lib.rs:9744-9747", "crates/nuxie-binary/src/lib.rs:413-430"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "import_status/import-stack descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-binary/src/lib.rs:9306-9319", "crates/nuxie-binary/src/lib.rs:9385-9426", "crates/nuxie-binary/src/lib.rs:9729-9812"]}
      - {name: "latest importer/ordinal and lookup catalogs", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-binary/src/lib.rs:4766-4935"], note: "locals/read-side catalogs, not cycle-persistent drift trackers"}
idiom_rules_invoked: ["AF-1 retained identity via arena id", "AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "C++ unique_ptr layers, inputs, listeners, and data binds map to owned vectors; scripted-object raw links map to stable ids. The full sibling sweep found no mutation-gated compensation for this row."
~~~

## B6-0229

~~~yaml
row_id: B6-0229
cpp_files: ["src/importers/state_machine_layer_component_importer.cpp"]
rust_module: "crates/nuxie-runtime/src/objects.rs"
subsystem_cluster: assets-importers
sibling_files_swept: ["crates/nuxie-runtime/src/objects.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/draw.rs", "crates/nuxie-runtime/src/text.rs", "crates/nuxie-runtime/src/view_model.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-binary/src/lib.rs", "crates/nuxie-scripting/src/shader_asset.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-1 stable arena identity", evidence: ["src/importers/state_machine_layer_component_importer.cpp:8-31", "crates/nuxie-binary/src/lib.rs:9770-9772", "crates/nuxie-runtime/src/objects.rs:16-23", "crates/nuxie-runtime/src/objects.rs:59-75"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/importers/state_machine_layer_component_importer.cpp:8-31", "crates/nuxie-binary/src/lib.rs:9770-9772"], note: "This import relationship is established once; no advance/update/bind poll, generation comparison, or rescan was found for it."}
  update_ordering: {status: isomorphic, phases_cpp: ["read/import", "resolve"], phases_rust: ["read/compute import status", "validate/apply import mutations", "build indexed runtime descriptors"]}
  ownership: {status: adapted, idiom_rule: "AF-7 unique children by value; AF-1 shared links by arena id", evidence: ["src/importers/state_machine_layer_component_importer.cpp:8-31", "crates/nuxie-binary/src/lib.rs:9770-9772", "crates/nuxie-binary/src/lib.rs:413-430"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "import_status/import-stack descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-binary/src/lib.rs:9306-9319", "crates/nuxie-binary/src/lib.rs:9385-9426", "crates/nuxie-binary/src/lib.rs:9729-9812"]}
      - {name: "latest importer/ordinal and lookup catalogs", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-binary/src/lib.rs:4766-4935"], note: "locals/read-side catalogs, not cycle-persistent drift trackers"}
idiom_rules_invoked: ["AF-1 retained identity via arena id", "AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "Owned listener actions map to values, while raw fire-event links map to stable imported ids. The full sibling sweep found no mutation-gated compensation for this row."
~~~

## B6-0230

~~~yaml
row_id: B6-0230
cpp_files: ["src/importers/state_machine_layer_importer.cpp"]
rust_module: "crates/nuxie-runtime/src/objects.rs"
subsystem_cluster: assets-importers
sibling_files_swept: ["crates/nuxie-runtime/src/objects.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/draw.rs", "crates/nuxie-runtime/src/text.rs", "crates/nuxie-runtime/src/view_model.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-binary/src/lib.rs", "crates/nuxie-scripting/src/shader_asset.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-1 stable arena identity", evidence: ["src/importers/state_machine_layer_importer.cpp:9-61", "crates/nuxie-binary/src/lib.rs:9435-9441", "crates/nuxie-runtime/src/objects.rs:16-23", "crates/nuxie-runtime/src/objects.rs:59-75"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/importers/state_machine_layer_importer.cpp:9-61", "crates/nuxie-binary/src/lib.rs:9435-9441"], note: "This import relationship is established once; no advance/update/bind poll, generation comparison, or rescan was found for it."}
  update_ordering: {status: isomorphic, phases_cpp: ["read/import", "resolve"], phases_rust: ["read/compute import status", "validate/apply import mutations", "build indexed runtime descriptors"]}
  ownership: {status: adapted, idiom_rule: "AF-7 unique children by value; AF-1 shared links by arena id", evidence: ["src/importers/state_machine_layer_importer.cpp:9-61", "crates/nuxie-binary/src/lib.rs:9435-9441", "crates/nuxie-binary/src/lib.rs:413-430"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "import_status/import-stack descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-binary/src/lib.rs:9306-9319", "crates/nuxie-binary/src/lib.rs:9385-9426", "crates/nuxie-binary/src/lib.rs:9729-9812"]}
      - {name: "latest importer/ordinal and lookup catalogs", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-binary/src/lib.rs:4766-4935"], note: "locals/read-side catalogs, not cycle-persistent drift trackers"}
idiom_rules_invoked: ["AF-1 retained identity via arena id", "AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "C++ state attachment and resolve pass are folded into stable ids and one-time import validation; null placeholders remain explicit. The full sibling sweep found no mutation-gated compensation for this row."
~~~

## B6-0231

~~~yaml
row_id: B6-0231
cpp_files: ["src/importers/state_machine_listener_importer.cpp"]
rust_module: "crates/nuxie-runtime/src/objects.rs"
subsystem_cluster: assets-importers
sibling_files_swept: ["crates/nuxie-runtime/src/objects.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/draw.rs", "crates/nuxie-runtime/src/text.rs", "crates/nuxie-runtime/src/view_model.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-binary/src/lib.rs", "crates/nuxie-scripting/src/shader_asset.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-1 stable arena identity", evidence: ["src/importers/state_machine_listener_importer.cpp:8-25", "crates/nuxie-binary/src/lib.rs:9780-9782", "crates/nuxie-runtime/src/objects.rs:16-23", "crates/nuxie-runtime/src/objects.rs:59-75"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/importers/state_machine_listener_importer.cpp:8-25", "crates/nuxie-binary/src/lib.rs:9780-9782"], note: "This import relationship is established once; no advance/update/bind poll, generation comparison, or rescan was found for it."}
  update_ordering: {status: isomorphic, phases_cpp: ["read/import", "resolve"], phases_rust: ["read/compute import status", "validate/apply import mutations", "build indexed runtime descriptors"]}
  ownership: {status: adapted, idiom_rule: "AF-7 unique children by value; AF-1 shared links by arena id", evidence: ["src/importers/state_machine_listener_importer.cpp:8-25", "crates/nuxie-binary/src/lib.rs:9780-9782", "crates/nuxie-binary/src/lib.rs:413-430"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "import_status/import-stack descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-binary/src/lib.rs:9306-9319", "crates/nuxie-binary/src/lib.rs:9385-9426", "crates/nuxie-binary/src/lib.rs:9729-9812"]}
      - {name: "latest importer/ordinal and lookup catalogs", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-binary/src/lib.rs:4766-4935"], note: "locals/read-side catalogs, not cycle-persistent drift trackers"}
idiom_rules_invoked: ["AF-1 retained identity via arena id", "AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "C++ transfers unique_ptr actions/input types; Rust owns equivalent descriptors by value after import. The full sibling sweep found no mutation-gated compensation for this row."
~~~

## B6-0232

~~~yaml
row_id: B6-0232
cpp_files: ["src/importers/state_transition_importer.cpp"]
rust_module: "crates/nuxie-runtime/src/objects.rs"
subsystem_cluster: assets-importers
sibling_files_swept: ["crates/nuxie-runtime/src/objects.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/draw.rs", "crates/nuxie-runtime/src/text.rs", "crates/nuxie-runtime/src/view_model.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-binary/src/lib.rs", "crates/nuxie-scripting/src/shader_asset.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-1 stable arena identity", evidence: ["src/importers/state_transition_importer.cpp:7-15", "crates/nuxie-binary/src/lib.rs:9773-9775", "crates/nuxie-runtime/src/objects.rs:16-23", "crates/nuxie-runtime/src/objects.rs:59-75"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/importers/state_transition_importer.cpp:7-15", "crates/nuxie-binary/src/lib.rs:9773-9775"], note: "This import relationship is established once; no advance/update/bind poll, generation comparison, or rescan was found for it."}
  update_ordering: {status: isomorphic, phases_cpp: ["read/import", "resolve"], phases_rust: ["read/compute import status", "validate/apply import mutations", "build indexed runtime descriptors"]}
  ownership: {status: adapted, idiom_rule: "AF-7 unique children by value; AF-1 shared links by arena id", evidence: ["src/importers/state_transition_importer.cpp:7-15", "crates/nuxie-binary/src/lib.rs:9773-9775", "crates/nuxie-binary/src/lib.rs:413-430"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "import_status/import-stack descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-binary/src/lib.rs:9306-9319", "crates/nuxie-binary/src/lib.rs:9385-9426", "crates/nuxie-binary/src/lib.rs:9729-9812"]}
      - {name: "latest importer/ordinal and lookup catalogs", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-binary/src/lib.rs:4766-4935"], note: "locals/read-side catalogs, not cycle-persistent drift trackers"}
idiom_rules_invoked: ["AF-1 retained identity via arena id", "AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "Raw condition links are represented as stable imported ids resolved once, with no cycle-time observer or poll. The full sibling sweep found no mutation-gated compensation for this row."
~~~

## B6-0233

~~~yaml
row_id: B6-0233
cpp_files: ["src/importers/text_asset_importer.cpp"]
rust_module: "crates/nuxie-runtime/src/objects.rs"
subsystem_cluster: assets-importers
sibling_files_swept: ["crates/nuxie-runtime/src/objects.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/draw.rs", "crates/nuxie-runtime/src/text.rs", "crates/nuxie-runtime/src/view_model.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-binary/src/lib.rs", "crates/nuxie-scripting/src/shader_asset.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-1 stable asset id; AF-7 owned contents", evidence: ["src/importers/text_asset_importer.cpp:36-114", "crates/nuxie-binary/src/lib.rs:4766-4935", "crates/nuxie-runtime/src/objects.rs:16-23", "crates/nuxie-runtime/src/objects.rs:59-75"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/importers/text_asset_importer.cpp:36-114", "crates/nuxie-binary/src/lib.rs:4766-4935"], note: "This import relationship is established once; no advance/update/bind poll, generation comparison, or rescan was found for it."}
  update_ordering: {status: isomorphic, phases_cpp: ["read/import", "resolve"], phases_rust: ["read/compute import status", "validate/apply import mutations", "build indexed runtime descriptors"]}
  ownership: {status: adapted, idiom_rule: "AF-7 own-by-value", evidence: ["src/importers/text_asset_importer.cpp:36-114", "crates/nuxie-binary/src/lib.rs:4766-4935", "crates/nuxie-binary/src/lib.rs:413-430"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "import_status/import-stack descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-binary/src/lib.rs:9306-9319", "crates/nuxie-binary/src/lib.rs:9385-9426", "crates/nuxie-binary/src/lib.rs:9729-9812"]}
      - {name: "latest importer/ordinal and lookup catalogs", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-binary/src/lib.rs:4766-4935"], note: "locals/read-side catalogs, not cycle-persistent drift trackers"}
idiom_rules_invoked: ["AF-1 retained identity via arena id", "AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "C++ uniquely owns in-band contents and resolves verification/decode at import; Rust retains immutable contents and performs downstream validation from the finalized file. The full sibling sweep found no mutation-gated compensation for this row."
~~~

## B6-0234

~~~yaml
row_id: B6-0234
cpp_files: ["src/importers/transition_viewmodel_condition_importer.cpp"]
rust_module: "crates/nuxie-runtime/src/objects.rs"
subsystem_cluster: assets-importers
sibling_files_swept: ["crates/nuxie-runtime/src/objects.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/draw.rs", "crates/nuxie-runtime/src/text.rs", "crates/nuxie-runtime/src/view_model.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-binary/src/lib.rs", "crates/nuxie-scripting/src/shader_asset.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-1 stable arena identity", evidence: ["src/importers/transition_viewmodel_condition_importer.cpp:8-22", "crates/nuxie-binary/src/lib.rs:9791-9793", "crates/nuxie-runtime/src/objects.rs:16-23", "crates/nuxie-runtime/src/objects.rs:59-75"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/importers/transition_viewmodel_condition_importer.cpp:8-22", "crates/nuxie-binary/src/lib.rs:9791-9793"], note: "This import relationship is established once; no advance/update/bind poll, generation comparison, or rescan was found for it."}
  update_ordering: {status: isomorphic, phases_cpp: ["read/import", "resolve"], phases_rust: ["read/compute import status", "validate/apply import mutations", "build indexed runtime descriptors"]}
  ownership: {status: adapted, idiom_rule: "AF-7 unique children by value; AF-1 shared links by arena id", evidence: ["src/importers/transition_viewmodel_condition_importer.cpp:8-22", "crates/nuxie-binary/src/lib.rs:9791-9793", "crates/nuxie-binary/src/lib.rs:413-430"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "import_status/import-stack descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-binary/src/lib.rs:9306-9319", "crates/nuxie-binary/src/lib.rs:9385-9426", "crates/nuxie-binary/src/lib.rs:9729-9812"]}
      - {name: "latest importer/ordinal and lookup catalogs", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-binary/src/lib.rs:4766-4935"], note: "locals/read-side catalogs, not cycle-persistent drift trackers"}
idiom_rules_invoked: ["AF-1 retained identity via arena id", "AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "Comparator attachment and resolve are folded into a stable import-time condition context. The full sibling sweep found no mutation-gated compensation for this row."
~~~

## B6-0235

~~~yaml
row_id: B6-0235
cpp_files: ["src/importers/viewmodel_importer.cpp"]
rust_module: "crates/nuxie-runtime/src/objects.rs"
subsystem_cluster: assets-importers
sibling_files_swept: ["crates/nuxie-runtime/src/objects.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/draw.rs", "crates/nuxie-runtime/src/text.rs", "crates/nuxie-runtime/src/view_model.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-binary/src/lib.rs", "crates/nuxie-scripting/src/shader_asset.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-1 stable arena identity", evidence: ["src/importers/viewmodel_importer.cpp:7-15", "crates/nuxie-binary/src/lib.rs:9758-9760", "crates/nuxie-runtime/src/objects.rs:16-23", "crates/nuxie-runtime/src/objects.rs:59-75"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/importers/viewmodel_importer.cpp:7-15", "crates/nuxie-binary/src/lib.rs:9758-9760"], note: "This import relationship is established once; no advance/update/bind poll, generation comparison, or rescan was found for it."}
  update_ordering: {status: isomorphic, phases_cpp: ["read/import", "resolve"], phases_rust: ["read/compute import status", "validate/apply import mutations", "build indexed runtime descriptors"]}
  ownership: {status: adapted, idiom_rule: "AF-7 unique children by value; AF-1 shared links by arena id", evidence: ["src/importers/viewmodel_importer.cpp:7-15", "crates/nuxie-binary/src/lib.rs:9758-9760", "crates/nuxie-binary/src/lib.rs:413-430"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "import_status/import-stack descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-binary/src/lib.rs:9306-9319", "crates/nuxie-binary/src/lib.rs:9385-9426", "crates/nuxie-binary/src/lib.rs:9729-9812"]}
      - {name: "latest importer/ordinal and lookup catalogs", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-binary/src/lib.rs:4766-4935"], note: "locals/read-side catalogs, not cycle-persistent drift trackers"}
idiom_rules_invoked: ["AF-1 retained identity via arena id", "AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "C++ appends property pointers; Rust retains properties as stable imported ids and builds owned/indexed view-model storage. The full sibling sweep found no mutation-gated compensation for this row."
~~~

## B6-0236

~~~yaml
row_id: B6-0236
cpp_files: ["src/importers/viewmodel_instance_importer.cpp"]
rust_module: "crates/nuxie-runtime/src/objects.rs"
subsystem_cluster: assets-importers
sibling_files_swept: ["crates/nuxie-runtime/src/objects.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/draw.rs", "crates/nuxie-runtime/src/text.rs", "crates/nuxie-runtime/src/view_model.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-binary/src/lib.rs", "crates/nuxie-scripting/src/shader_asset.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-1 stable arena identity", evidence: ["src/importers/viewmodel_instance_importer.cpp:7-16", "crates/nuxie-binary/src/lib.rs:9758-9760", "crates/nuxie-runtime/src/objects.rs:16-23", "crates/nuxie-runtime/src/objects.rs:59-75"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/importers/viewmodel_instance_importer.cpp:7-16", "crates/nuxie-binary/src/lib.rs:9758-9760"], note: "This import relationship is established once; no advance/update/bind poll, generation comparison, or rescan was found for it."}
  update_ordering: {status: isomorphic, phases_cpp: ["read/import", "resolve"], phases_rust: ["read/compute import status", "validate/apply import mutations", "build indexed runtime descriptors"]}
  ownership: {status: adapted, idiom_rule: "AF-7 unique children by value; AF-1 shared links by arena id", evidence: ["src/importers/viewmodel_instance_importer.cpp:7-16", "crates/nuxie-binary/src/lib.rs:9758-9760", "crates/nuxie-binary/src/lib.rs:413-430"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "import_status/import-stack descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-binary/src/lib.rs:9306-9319", "crates/nuxie-binary/src/lib.rs:9385-9426", "crates/nuxie-binary/src/lib.rs:9729-9812"]}
      - {name: "latest importer/ordinal and lookup catalogs", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-binary/src/lib.rs:4766-4935"], note: "locals/read-side catalogs, not cycle-persistent drift trackers"}
idiom_rules_invoked: ["AF-1 retained identity via arena id", "AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "C++ appends instance-value pointers; Rust retains stable imported ids and builds owned/indexed instance storage. The full sibling sweep found no mutation-gated compensation for this row."
~~~

## B6-0237

~~~yaml
row_id: B6-0237
cpp_files: ["src/importers/viewmodel_instance_list_importer.cpp"]
rust_module: "crates/nuxie-runtime/src/objects.rs"
subsystem_cluster: assets-importers
sibling_files_swept: ["crates/nuxie-runtime/src/objects.rs", "crates/nuxie-runtime/src/artboard.rs", "crates/nuxie-runtime/src/artboard_data_bind.rs", "crates/nuxie-runtime/src/draw.rs", "crates/nuxie-runtime/src/text.rs", "crates/nuxie-runtime/src/view_model.rs", "crates/nuxie-runtime/src/state_machine.rs", "crates/nuxie-runtime/src/state_machine/instance.rs", "crates/nuxie-binary/src/lib.rs", "crates/nuxie-scripting/src/shader_asset.rs"]
verdict: ADAPTED
axes:
  retained_identity: {status: adapted, idiom_rule: "AF-1 stable arena identity", evidence: ["src/importers/viewmodel_instance_list_importer.cpp:7-16", "crates/nuxie-binary/src/lib.rs:9758-9761", "crates/nuxie-runtime/src/objects.rs:16-23", "crates/nuxie-runtime/src/objects.rs:59-75"]}
  push_vs_poll: {status: isomorphic, cpp_pushes: false, evidence: ["src/importers/viewmodel_instance_list_importer.cpp:7-16", "crates/nuxie-binary/src/lib.rs:9758-9761"], note: "This import relationship is established once; no advance/update/bind poll, generation comparison, or rescan was found for it."}
  update_ordering: {status: isomorphic, phases_cpp: ["read/import", "resolve"], phases_rust: ["read/compute import status", "validate/apply import mutations", "build indexed runtime descriptors"]}
  ownership: {status: adapted, idiom_rule: "AF-7 unique children by value; AF-1 shared links by arena id", evidence: ["src/importers/viewmodel_instance_list_importer.cpp:7-16", "crates/nuxie-binary/src/lib.rs:9758-9761", "crates/nuxie-binary/src/lib.rs:413-430"]}
  compensation:
    status: adapted
    mechanisms: []
    import_time_constants:
      - {name: "import_status/import-stack descriptor", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-binary/src/lib.rs:9306-9319", "crates/nuxie-binary/src/lib.rs:9385-9426", "crates/nuxie-binary/src/lib.rs:9729-9812"]}
      - {name: "latest importer/ordinal and lookup catalogs", idiom_rule: "AF-5 import-time devirtualization", evidence: ["crates/nuxie-binary/src/lib.rs:4766-4935"], note: "locals/read-side catalogs, not cycle-persistent drift trackers"}
idiom_rules_invoked: ["AF-1 retained identity via arena id", "AF-5 import-time devirtualization", "AF-7 own-by-value"]
confidence: high
notes: "C++ retains rcp list-item identity; Rust resolves list items through stable arena ids rather than copying and re-synchronizing them. The full sibling sweep found no mutation-gated compensation for this row."
~~~

