# B-6 Structural Audit — constraints

Pinned C++: /Users/levi/dev/oss/rive-runtime @ d788e8ec6e8b598526607d6a1e8818e8b637b60c. All 18 assigned C++ files and the complete mapped Rust module were read. Coverage sweep: crate-wide grep for generation/epoch/revision/dirty/observed/snapshot/candidate/alias plus constraint/scroll/virtualizer/follow-path/IK families, followed by sibling inspection of artboard.rs, components.rs, draw.rs, nuxie-graph/src/lib.rs, and view_model.rs where the virtualizer crosses RB-1 state. Fixed graph descriptors are AF-5 import-time constants; only cycle-written state is counted as compensation. The pinned C++ checkout was read-only.

## B6-0124

```json
{
  "row_id": "B6-0124",
  "cpp_files": [
    "src/constraints/constrainable_list.cpp"
  ],
  "rust_module": "crates/nuxie-runtime/src/constraints.rs",
  "subsystem_cluster": "constraints",
  "sibling_files_swept": [
    "crates/nuxie-runtime/src/constraints.rs",
    "crates/nuxie-runtime/src/artboard.rs",
    "crates/nuxie-runtime/src/components.rs",
    "crates/nuxie-runtime/src/draw.rs",
    "crates/nuxie-graph/src/lib.rs"
  ],
  "verdict": "ADAPTED",
  "axes": {
    "retained_identity": {
      "status": "adapted import-index identity",
      "idiom_rule": "AF-5 import-time devirtualization",
      "evidence": [
        "cpp@d788e8ec:src/constraints/constrainable_list.cpp:8-23",
        "crates/nuxie-graph/src/lib.rs:3084-3117"
      ]
    },
    "push_vs_poll": {
      "status": "adapted: one-time graph registration replaces the closed type switch and vector append",
      "cpp_pushes": false,
      "evidence": [
        "cpp@d788e8ec:src/constraints/constrainable_list.cpp:8-23",
        "crates/nuxie-graph/src/lib.rs:3084-3117"
      ]
    },
    "update_ordering": {
      "status": "isomorphic build-time registration",
      "phases_cpp": "onAdded/buildDependencies -> addListConstraint",
      "phases_rust": "graph build -> list_constraint_registrations"
    },
    "ownership": {
      "status": "adapted by-value registration vector",
      "evidence": [
        "cpp@d788e8ec:src/constraints/constrainable_list.cpp:18-23",
        "crates/nuxie-graph/src/lib.rs:884-890,3084-3117"
      ]
    },
    "compensation": {
      "status": "clear after crate-wide family grep and sibling sweep",
      "mechanisms": [],
      "import_time_constants": [
        {
          "name": "precomputed constraint membership/dependency descriptors",
          "idiom_rule": "AF-5 import-time devirtualization",
          "evidence": [
            "crates/nuxie-graph/src/lib.rs:3084-3143,3655-3716,4977-5023",
            "crates/nuxie-runtime/src/components.rs:477-512"
          ]
        }
      ]
    }
  },
  "idiom_rules_invoked": [
    "AF-5 import-time devirtualization",
    "AF-7 own-by-value"
  ],
  "confidence": "high",
  "notes": "The C++ pointer list is represented by immutable local/global IDs built once. generation/epoch/dirty/observed/snapshot/candidate/alias and constraint-family grep found no cycle-written drift tracker attributable to this row; all listed siblings were swept."
}
```

## B6-0125

```json
{
  "row_id": "B6-0125",
  "cpp_files": [
    "src/constraints/constraint.cpp"
  ],
  "rust_module": "crates/nuxie-runtime/src/constraints.rs",
  "subsystem_cluster": "constraints",
  "sibling_files_swept": [
    "crates/nuxie-runtime/src/constraints.rs",
    "crates/nuxie-runtime/src/artboard.rs",
    "crates/nuxie-runtime/src/components.rs",
    "crates/nuxie-runtime/src/draw.rs",
    "crates/nuxie-graph/src/lib.rs"
  ],
  "verdict": "DIVERGENT",
  "axes": {
    "retained_identity": {
      "status": "adapted-arena-identity",
      "idiom_rule": "AF-1 retained identity",
      "evidence": [
        "cpp@d788e8ec:src/constraints/constraint.cpp:9-40",
        "crates/nuxie-graph/src/lib.rs:4977-5023; crates/nuxie-runtime/src/components.rs:477-512"
      ]
    },
    "push_vs_poll": {
      "status": "isomorphic push for dependency dirt; extra counters are counted only under compensation",
      "cpp_pushes": true,
      "evidence": [
        "cpp@d788e8ec:src/constraints/constraint.cpp:22-40",
        "crates/nuxie-runtime/src/artboard.rs:3948-3990,4424-4451"
      ]
    },
    "update_ordering": {
      "status": "phase-sequence equivalent; representation divergent",
      "phases_cpp": "property change/dirt -> dependent cascade -> constrained component update",
      "phases_rust": "property write -> ComponentDirt cascade plus epoch writes -> constrained component update"
    },
    "ownership": {
      "status": "adapted arena IDs and owned dependency vectors",
      "evidence": [
        "cpp@d788e8ec:src/constraints/constraint.cpp:9-33",
        "crates/nuxie-graph/src/lib.rs:4977-5023"
      ]
    },
    "compensation": {
      "status": "present",
      "mechanisms": [
        {
          "name": "cache/prepared/command epoch fanout",
          "kind": "mutation-time parallel invalidation counters beside ComponentDirt",
          "mutation_gated": true,
          "cpp_counterpart": "none (the C++ constraint family uses retained ComponentDirt and markConstraintDirty)",
          "evidence": [
            "crates/nuxie-runtime/src/artboard.rs:2115-2134,3728-3755"
          ]
        }
      ],
      "import_time_constants": [
        {
          "name": "precomputed constraint membership/dependency descriptors",
          "idiom_rule": "AF-5 import-time devirtualization",
          "evidence": [
            "crates/nuxie-graph/src/lib.rs:3084-3143,3655-3716,4977-5023",
            "crates/nuxie-runtime/src/components.rs:477-512"
          ]
        }
      ]
    }
  },
  "idiom_rules_invoked": [
    "AF-1 retained identity",
    "AF-2 push never reconstruct",
    "AF-4 one dirt model",
    "AF-5 import-time devirtualization"
  ],
  "confidence": "high",
  "notes": "C++ has one constraint dirt path. Rust retains that path but also writes render/cache epoch state on live property mutation; family grep found its consumers in draw.rs. No remediation judgment."
}
```

## B6-0126

```json
{
  "row_id": "B6-0126",
  "cpp_files": [
    "src/constraints/distance_constraint.cpp"
  ],
  "rust_module": "crates/nuxie-runtime/src/constraints.rs",
  "subsystem_cluster": "constraints",
  "sibling_files_swept": [
    "crates/nuxie-runtime/src/constraints.rs",
    "crates/nuxie-runtime/src/artboard.rs",
    "crates/nuxie-runtime/src/components.rs",
    "crates/nuxie-runtime/src/draw.rs",
    "crates/nuxie-graph/src/lib.rs"
  ],
  "verdict": "DIVERGENT",
  "axes": {
    "retained_identity": {
      "status": "adapted-arena-identity",
      "idiom_rule": "AF-1 retained identity",
      "evidence": [
        "cpp@d788e8ec:src/constraints/distance_constraint.cpp:15-57",
        "crates/nuxie-runtime/src/constraints.rs:1890-1968"
      ]
    },
    "push_vs_poll": {
      "status": "divergent for authored distance/mode writes: C++ pushes markConstraintDirty; Rust reads live values when the parent next updates",
      "cpp_pushes": true,
      "evidence": [
        "cpp@d788e8ec:src/constraints/distance_constraint.cpp:59-61",
        "crates/nuxie-runtime/src/constraints.rs:1928-1946",
        "crates/nuxie-runtime/src/artboard.rs:4962-4984"
      ]
    },
    "update_ordering": {
      "status": "mixed",
      "phases_cpp": "distance/mode change -> parent transform dirt -> constrain",
      "phases_rust": "property mutation -> epoch invalidation; constrain only on a later parent WORLD_TRANSFORM update"
    },
    "ownership": {
      "status": "adapted arena target ID and in-place component transform",
      "evidence": [
        "cpp@d788e8ec:src/constraints/distance_constraint.cpp:15-57",
        "crates/nuxie-runtime/src/constraints.rs:1903-1967"
      ]
    },
    "compensation": {
      "status": "present",
      "mechanisms": [
        {
          "name": "cache/prepared/command epoch fanout",
          "kind": "mutation-time parallel invalidation counters beside ComponentDirt",
          "mutation_gated": true,
          "cpp_counterpart": "none (the C++ constraint family uses retained ComponentDirt and markConstraintDirty)",
          "evidence": [
            "crates/nuxie-runtime/src/artboard.rs:2115-2134,3728-3755"
          ]
        }
      ],
      "import_time_constants": [
        {
          "name": "precomputed constraint membership/dependency descriptors",
          "idiom_rule": "AF-5 import-time devirtualization",
          "evidence": [
            "crates/nuxie-graph/src/lib.rs:3084-3143,3655-3716,4977-5023",
            "crates/nuxie-runtime/src/components.rs:477-512"
          ]
        }
      ]
    }
  },
  "idiom_rules_invoked": [
    "AF-1 retained identity",
    "AF-2 push never reconstruct",
    "AF-4 one dirt model",
    "AF-5 import-time devirtualization"
  ],
  "confidence": "high",
  "notes": "Behavior-bug note: current explicit Rust property-change dispatch covers ScrollConstraint and FollowPathConstraint, not DistanceConstraint.distance/modeValue. Audit records evidence only."
}
```

## B6-0127

```json
{
  "row_id": "B6-0127",
  "cpp_files": [
    "src/constraints/draggable_constraint.cpp"
  ],
  "rust_module": "crates/nuxie-runtime/src/constraints.rs",
  "subsystem_cluster": "constraints",
  "sibling_files_swept": [
    "crates/nuxie-runtime/src/constraints.rs",
    "crates/nuxie-runtime/src/artboard.rs",
    "crates/nuxie-runtime/src/components.rs",
    "crates/nuxie-runtime/src/draw.rs",
    "crates/nuxie-graph/src/lib.rs"
  ],
  "verdict": "UNKNOWN",
  "axes": {
    "retained_identity": {
      "status": "unknown: no mapped Rust listener/proxy identity",
      "evidence": [
        "cpp@d788e8ec:src/constraints/draggable_constraint.cpp:8-85",
        "crates/nuxie-runtime/src/constraints.rs:107-125,1717-1723"
      ]
    },
    "push_vs_poll": {
      "status": "unknown: Rust only reads directionValue for scroll math; no processEvent/drag listener seam exists",
      "cpp_pushes": true,
      "evidence": [
        "cpp@d788e8ec:src/constraints/draggable_constraint.cpp:32-85",
        "crates/nuxie-runtime/src/constraints.rs:107-125"
      ]
    },
    "update_ordering": {
      "status": "unknown",
      "phases_cpp": "pointer event -> listener phase -> start/drag/end",
      "phases_rust": "blocker: no mapped event/gesture implementation"
    },
    "ownership": {
      "status": "unknown: C++ allocates listener groups/proxies; no Rust counterpart found",
      "evidence": [
        "cpp@d788e8ec:src/constraints/draggable_constraint.cpp:8-29",
        "crates/nuxie-runtime/src/constraints.rs:1-4012"
      ]
    },
    "compensation": {
      "status": "not assessed beyond absent mapped seam",
      "mechanisms": [],
      "import_time_constants": [
        {
          "name": "precomputed constraint membership/dependency descriptors",
          "idiom_rule": "AF-5 import-time devirtualization",
          "evidence": [
            "crates/nuxie-graph/src/lib.rs:3084-3143,3655-3716,4977-5023",
            "crates/nuxie-runtime/src/components.rs:477-512"
          ]
        }
      ]
    }
  },
  "idiom_rules_invoked": [
    "AF-5 import-time devirtualization"
  ],
  "confidence": "high",
  "notes": "Blocker: constraints.rs has no RuntimeDraggableConstraint, listener group, pointer phase, or drag lifecycle. Crate-wide sibling grep found only scroll direction/property consumers, so a structural verdict would be a guess."
}
```

## B6-0128

```json
{
  "row_id": "B6-0128",
  "cpp_files": [
    "src/constraints/follow_path_constraint.cpp"
  ],
  "rust_module": "crates/nuxie-runtime/src/constraints.rs",
  "subsystem_cluster": "constraints",
  "sibling_files_swept": [
    "crates/nuxie-runtime/src/constraints.rs",
    "crates/nuxie-runtime/src/artboard.rs",
    "crates/nuxie-runtime/src/components.rs",
    "crates/nuxie-runtime/src/draw.rs",
    "crates/nuxie-graph/src/lib.rs"
  ],
  "verdict": "DIVERGENT",
  "axes": {
    "retained_identity": {
      "status": "adapted-arena-identity",
      "idiom_rule": "AF-1 retained identity",
      "evidence": [
        "cpp@d788e8ec:src/constraints/follow_path_constraint.cpp:21-190",
        "crates/nuxie-runtime/src/constraints.rs:12-18,741-817,2413-2695"
      ]
    },
    "push_vs_poll": {
      "status": "isomorphic push for path/constraint dependencies; extra counters counted under compensation",
      "cpp_pushes": true,
      "evidence": [
        "cpp@d788e8ec:src/constraints/follow_path_constraint.cpp:18-20,122-190",
        "crates/nuxie-graph/src/lib.rs:3709-3716,4023-4113",
        "crates/nuxie-runtime/src/artboard.rs:4627-4632,4977-4982"
      ]
    },
    "update_ordering": {
      "status": "phase-sequence equivalent; path measure representation adapted",
      "phases_cpp": "path dirt -> FollowPath update/path measure -> constrain",
      "phases_rust": "path/dependency dirt -> geometry command/measure construction -> constrain"
    },
    "ownership": {
      "status": "adapted: target/path identities are arena IDs and immutable geometry descriptors",
      "evidence": [
        "cpp@d788e8ec:src/constraints/follow_path_constraint.cpp:122-190",
        "crates/nuxie-runtime/src/constraints.rs:741-817,2474-2499"
      ]
    },
    "compensation": {
      "status": "present",
      "mechanisms": [
        {
          "name": "cache/prepared/command epoch fanout",
          "kind": "mutation-time parallel invalidation counters beside ComponentDirt",
          "mutation_gated": true,
          "cpp_counterpart": "none (the C++ constraint family uses retained ComponentDirt and markConstraintDirty)",
          "evidence": [
            "crates/nuxie-runtime/src/artboard.rs:2115-2134,3728-3755"
          ]
        }
      ],
      "import_time_constants": [
        {
          "name": "precomputed constraint membership/dependency descriptors",
          "idiom_rule": "AF-5 import-time devirtualization",
          "evidence": [
            "crates/nuxie-graph/src/lib.rs:3084-3143,3655-3716,4977-5023",
            "crates/nuxie-runtime/src/components.rs:477-512"
          ]
        },
        {
          "name": "RuntimeFollowPathTargetKind and path descriptor list",
          "idiom_rule": "AF-5 import-time devirtualization",
          "evidence": [
            "crates/nuxie-runtime/src/constraints.rs:419-430,741-817"
          ]
        }
      ]
    }
  },
  "idiom_rules_invoked": [
    "AF-1 retained identity",
    "AF-4 one dirt model",
    "AF-5 import-time devirtualization"
  ],
  "confidence": "high",
  "notes": "Target kind/path membership is build-time devirtualization and does not pass the mutation gate. The separate mutation-time epoch fanout does."
}
```

## B6-0129

```json
{
  "row_id": "B6-0129",
  "cpp_files": [
    "src/constraints/ik_constraint.cpp"
  ],
  "rust_module": "crates/nuxie-runtime/src/constraints.rs",
  "subsystem_cluster": "constraints",
  "sibling_files_swept": [
    "crates/nuxie-runtime/src/constraints.rs",
    "crates/nuxie-runtime/src/artboard.rs",
    "crates/nuxie-runtime/src/components.rs",
    "crates/nuxie-runtime/src/draw.rs",
    "crates/nuxie-graph/src/lib.rs"
  ],
  "verdict": "DIVERGENT",
  "axes": {
    "retained_identity": {
      "status": "adapted-arena-identity",
      "idiom_rule": "AF-1 retained identity",
      "evidence": [
        "cpp@d788e8ec:src/constraints/ik_constraint.cpp:23-73,219-290",
        "crates/nuxie-runtime/src/constraints.rs:433-450,819-860,2697-2933"
      ]
    },
    "push_vs_poll": {
      "status": "mixed: target/tip-child dependency pushes are preserved; invertDirection mutation has no row-specific Rust dirt dispatch",
      "cpp_pushes": true,
      "evidence": [
        "cpp@d788e8ec:src/constraints/ik_constraint.cpp:10-21,76-86,186",
        "crates/nuxie-graph/src/lib.rs:3691-3706,3921-4021,4657-4695",
        "crates/nuxie-runtime/src/artboard.rs:4962-4984"
      ]
    },
    "update_ordering": {
      "status": "phase math equivalent; mutation scheduling mixed",
      "phases_cpp": "target/property dirt -> chain reset/decompose -> solve -> strength mix",
      "phases_rust": "dependency dirt -> chain reset/decompose -> solve -> strength mix, plus epoch invalidation"
    },
    "ownership": {
      "status": "adapted owned chain of arena IDs",
      "evidence": [
        "cpp@d788e8ec:src/constraints/ik_constraint.cpp:30-52",
        "crates/nuxie-runtime/src/constraints.rs:819-860"
      ]
    },
    "compensation": {
      "status": "present",
      "mechanisms": [
        {
          "name": "cache/prepared/command epoch fanout",
          "kind": "mutation-time parallel invalidation counters beside ComponentDirt",
          "mutation_gated": true,
          "cpp_counterpart": "none (the C++ constraint family uses retained ComponentDirt and markConstraintDirty)",
          "evidence": [
            "crates/nuxie-runtime/src/artboard.rs:2115-2134,3728-3755"
          ]
        }
      ],
      "import_time_constants": [
        {
          "name": "precomputed constraint membership/dependency descriptors",
          "idiom_rule": "AF-5 import-time devirtualization",
          "evidence": [
            "crates/nuxie-graph/src/lib.rs:3084-3143,3655-3716,4977-5023",
            "crates/nuxie-runtime/src/components.rs:477-512"
          ]
        },
        {
          "name": "precomputed IK chain IDs",
          "idiom_rule": "AF-5 import-time devirtualization",
          "evidence": [
            "crates/nuxie-runtime/src/constraints.rs:819-860"
          ]
        }
      ]
    }
  },
  "idiom_rules_invoked": [
    "AF-1 retained identity",
    "AF-4 one dirt model",
    "AF-5 import-time devirtualization",
    "AF-7 own-by-value"
  ],
  "confidence": "high",
  "notes": "Behavior-bug note: current Rust property dispatch has no explicit IKConstraint.invertDirection parent-dirt arm. Chain math and dependency ordering were otherwise mechanically matched."
}
```

## B6-0130

```json
{
  "row_id": "B6-0130",
  "cpp_files": [
    "src/constraints/list_constraint.cpp"
  ],
  "rust_module": "crates/nuxie-runtime/src/constraints.rs",
  "subsystem_cluster": "constraints",
  "sibling_files_swept": [
    "crates/nuxie-runtime/src/constraints.rs",
    "crates/nuxie-runtime/src/artboard.rs",
    "crates/nuxie-runtime/src/components.rs",
    "crates/nuxie-runtime/src/draw.rs",
    "crates/nuxie-graph/src/lib.rs"
  ],
  "verdict": "ADAPTED",
  "axes": {
    "retained_identity": {
      "status": "adapted import-time type descriptor",
      "idiom_rule": "AF-5 import-time devirtualization",
      "evidence": [
        "cpp@d788e8ec:src/constraints/list_constraint.cpp:7-14",
        "crates/nuxie-graph/src/lib.rs:3084-3117"
      ]
    },
    "push_vs_poll": {
      "status": "isomorphic: no cycle observer in this closed type switch",
      "cpp_pushes": false,
      "evidence": [
        "cpp@d788e8ec:src/constraints/list_constraint.cpp:7-14",
        "crates/nuxie-graph/src/lib.rs:3091-3115"
      ]
    },
    "update_ordering": {
      "status": "isomorphic import/build classification",
      "phases_cpp": "closed coreType switch",
      "phases_rust": "schema/type check during graph build"
    },
    "ownership": {
      "status": "adapted by-value registration descriptor",
      "evidence": [
        "cpp@d788e8ec:src/constraints/list_constraint.cpp:7-14",
        "crates/nuxie-graph/src/lib.rs:884-890"
      ]
    },
    "compensation": {
      "status": "clear after crate-wide family grep and sibling sweep",
      "mechanisms": [],
      "import_time_constants": [
        {
          "name": "precomputed constraint membership/dependency descriptors",
          "idiom_rule": "AF-5 import-time devirtualization",
          "evidence": [
            "crates/nuxie-graph/src/lib.rs:3084-3143,3655-3716,4977-5023",
            "crates/nuxie-runtime/src/components.rs:477-512"
          ]
        }
      ]
    }
  },
  "idiom_rules_invoked": [
    "AF-5 import-time devirtualization",
    "AF-7 own-by-value"
  ],
  "confidence": "high",
  "notes": "The keyword/type discriminants are fixed at graph build and therefore remain import_time_constants. All listed siblings were swept and clean for this row."
}
```

## B6-0131

```json
{
  "row_id": "B6-0131",
  "cpp_files": [
    "src/constraints/list_follow_path_constraint.cpp"
  ],
  "rust_module": "crates/nuxie-runtime/src/constraints.rs",
  "subsystem_cluster": "constraints",
  "sibling_files_swept": [
    "crates/nuxie-runtime/src/constraints.rs",
    "crates/nuxie-runtime/src/artboard.rs",
    "crates/nuxie-runtime/src/components.rs",
    "crates/nuxie-runtime/src/draw.rs",
    "crates/nuxie-graph/src/lib.rs"
  ],
  "verdict": "DIVERGENT",
  "axes": {
    "retained_identity": {
      "status": "adapted-arena-identity",
      "idiom_rule": "AF-1 retained identity",
      "evidence": [
        "cpp@d788e8ec:src/constraints/list_follow_path_constraint.cpp:14-64",
        "crates/nuxie-runtime/src/constraints.rs:20-24,479-497,2556-2632"
      ]
    },
    "push_vs_poll": {
      "status": "divergent for distanceEnd/distanceOffset: C++ pushes parent dirt; Rust recomputes from live values during list transform preparation",
      "cpp_pushes": true,
      "evidence": [
        "cpp@d788e8ec:src/constraints/list_follow_path_constraint.cpp:8-12,56-64",
        "crates/nuxie-runtime/src/constraints.rs:2574-2602",
        "crates/nuxie-runtime/src/artboard.rs:4962-4984"
      ]
    },
    "update_ordering": {
      "status": "mixed",
      "phases_cpp": "property dirt/layout -> updateConstraints -> mutate retained item transforms",
      "phases_rust": "list transform preparation/update -> read live properties -> rewrite item transforms, plus epoch invalidation"
    },
    "ownership": {
      "status": "adapted arena IDs with owned runtime registration",
      "evidence": [
        "cpp@d788e8ec:src/constraints/list_follow_path_constraint.cpp:14-38,56-64",
        "crates/nuxie-runtime/src/constraints.rs:479-497,2556-2632"
      ]
    },
    "compensation": {
      "status": "present",
      "mechanisms": [
        {
          "name": "cache/prepared/command epoch fanout",
          "kind": "mutation-time parallel invalidation counters beside ComponentDirt",
          "mutation_gated": true,
          "cpp_counterpart": "none (the C++ constraint family uses retained ComponentDirt and markConstraintDirty)",
          "evidence": [
            "crates/nuxie-runtime/src/artboard.rs:2115-2134,3728-3755"
          ]
        }
      ],
      "import_time_constants": [
        {
          "name": "precomputed constraint membership/dependency descriptors",
          "idiom_rule": "AF-5 import-time devirtualization",
          "evidence": [
            "crates/nuxie-graph/src/lib.rs:3084-3143,3655-3716,4977-5023",
            "crates/nuxie-runtime/src/components.rs:477-512"
          ]
        }
      ]
    }
  },
  "idiom_rules_invoked": [
    "AF-1 retained identity",
    "AF-2 push never reconstruct",
    "AF-4 one dirt model",
    "AF-5 import-time devirtualization"
  ],
  "confidence": "high",
  "notes": "Behavior-bug note: no explicit Rust parent-dirt dispatch exists for ListFollowPathConstraint.distanceEnd/distanceOffset; current transform preparation reads them live."
}
```

## B6-0132

```json
{
  "row_id": "B6-0132",
  "cpp_files": [
    "src/constraints/rotation_constraint.cpp"
  ],
  "rust_module": "crates/nuxie-runtime/src/constraints.rs",
  "subsystem_cluster": "constraints",
  "sibling_files_swept": [
    "crates/nuxie-runtime/src/constraints.rs",
    "crates/nuxie-runtime/src/artboard.rs",
    "crates/nuxie-runtime/src/components.rs",
    "crates/nuxie-runtime/src/draw.rs",
    "crates/nuxie-graph/src/lib.rs"
  ],
  "verdict": "DIVERGENT",
  "axes": {
    "retained_identity": {
      "status": "adapted-arena-identity",
      "idiom_rule": "AF-1 retained identity",
      "evidence": [
        "cpp@d788e8ec:src/constraints/rotation_constraint.cpp:8-115",
        "crates/nuxie-runtime/src/constraints.rs:1970-2127"
      ]
    },
    "push_vs_poll": {
      "status": "isomorphic push for target dependency; live property reads occur inside the scheduled constrain",
      "cpp_pushes": true,
      "evidence": [
        "cpp@d788e8ec:src/constraints/rotation_constraint.cpp:8-115",
        "crates/nuxie-graph/src/lib.rs:3655-3688",
        "crates/nuxie-runtime/src/artboard.rs:3948-3990"
      ]
    },
    "update_ordering": {
      "status": "phase-sequence equivalent; representation divergent",
      "phases_cpp": "target/property dirt -> world update -> constrain",
      "phases_rust": "dependency dirt/property mutation -> epoch writes -> world update -> constrain"
    },
    "ownership": {
      "status": "adapted arena target identity and by-value decomposed temporaries",
      "evidence": [
        "cpp@d788e8ec:src/constraints/rotation_constraint.cpp:14-114",
        "crates/nuxie-runtime/src/constraints.rs:1982-2126"
      ]
    },
    "compensation": {
      "status": "present",
      "mechanisms": [
        {
          "name": "cache/prepared/command epoch fanout",
          "kind": "mutation-time parallel invalidation counters beside ComponentDirt",
          "mutation_gated": true,
          "cpp_counterpart": "none (the C++ constraint family uses retained ComponentDirt and markConstraintDirty)",
          "evidence": [
            "crates/nuxie-runtime/src/artboard.rs:2115-2134,3728-3755"
          ]
        }
      ],
      "import_time_constants": [
        {
          "name": "precomputed constraint membership/dependency descriptors",
          "idiom_rule": "AF-5 import-time devirtualization",
          "evidence": [
            "crates/nuxie-graph/src/lib.rs:3084-3143,3655-3716,4977-5023",
            "crates/nuxie-runtime/src/components.rs:477-512"
          ]
        }
      ]
    }
  },
  "idiom_rules_invoked": [
    "AF-1 retained identity",
    "AF-4 one dirt model",
    "AF-5 import-time devirtualization",
    "AF-7 own-by-value"
  ],
  "confidence": "high",
  "notes": "The rotation/conversion math is structurally direct. DIVERGENT is based only on the separately mutation-gated epoch family, not the by-value temporary components."
}
```

## B6-0133

```json
{
  "row_id": "B6-0133",
  "cpp_files": [
    "src/constraints/scale_constraint.cpp"
  ],
  "rust_module": "crates/nuxie-runtime/src/constraints.rs",
  "subsystem_cluster": "constraints",
  "sibling_files_swept": [
    "crates/nuxie-runtime/src/constraints.rs",
    "crates/nuxie-runtime/src/artboard.rs",
    "crates/nuxie-runtime/src/components.rs",
    "crates/nuxie-runtime/src/draw.rs",
    "crates/nuxie-graph/src/lib.rs"
  ],
  "verdict": "DIVERGENT",
  "axes": {
    "retained_identity": {
      "status": "adapted-arena-identity",
      "idiom_rule": "AF-1 retained identity",
      "evidence": [
        "cpp@d788e8ec:src/constraints/scale_constraint.cpp:8-131",
        "crates/nuxie-runtime/src/constraints.rs:2129-2359"
      ]
    },
    "push_vs_poll": {
      "status": "isomorphic push for target dependency; extra counters counted under compensation",
      "cpp_pushes": true,
      "evidence": [
        "cpp@d788e8ec:src/constraints/scale_constraint.cpp:8-131",
        "crates/nuxie-graph/src/lib.rs:3655-3688",
        "crates/nuxie-runtime/src/artboard.rs:3948-3990"
      ]
    },
    "update_ordering": {
      "status": "phase-sequence equivalent; representation divergent",
      "phases_cpp": "target/property dirt -> world update -> constrain",
      "phases_rust": "dependency dirt/property mutation -> epoch writes -> world update -> constrain"
    },
    "ownership": {
      "status": "adapted arena target identity and by-value decomposed temporaries",
      "evidence": [
        "cpp@d788e8ec:src/constraints/scale_constraint.cpp:14-130",
        "crates/nuxie-runtime/src/constraints.rs:2141-2359"
      ]
    },
    "compensation": {
      "status": "present",
      "mechanisms": [
        {
          "name": "cache/prepared/command epoch fanout",
          "kind": "mutation-time parallel invalidation counters beside ComponentDirt",
          "mutation_gated": true,
          "cpp_counterpart": "none (the C++ constraint family uses retained ComponentDirt and markConstraintDirty)",
          "evidence": [
            "crates/nuxie-runtime/src/artboard.rs:2115-2134,3728-3755"
          ]
        }
      ],
      "import_time_constants": [
        {
          "name": "precomputed constraint membership/dependency descriptors",
          "idiom_rule": "AF-5 import-time devirtualization",
          "evidence": [
            "crates/nuxie-graph/src/lib.rs:3084-3143,3655-3716,4977-5023",
            "crates/nuxie-runtime/src/components.rs:477-512"
          ]
        }
      ]
    }
  },
  "idiom_rules_invoked": [
    "AF-1 retained identity",
    "AF-4 one dirt model",
    "AF-5 import-time devirtualization",
    "AF-7 own-by-value"
  ],
  "confidence": "high",
  "notes": "The scale math matches the retained transform flow. The finding is the off-file mutation-time epoch family only."
}
```

## B6-0134

```json
{
  "row_id": "B6-0134",
  "cpp_files": [
    "src/constraints/scrolling/clamped_scroll_physics.cpp"
  ],
  "rust_module": "crates/nuxie-runtime/src/constraints.rs",
  "subsystem_cluster": "constraints",
  "sibling_files_swept": [
    "crates/nuxie-runtime/src/constraints.rs",
    "crates/nuxie-runtime/src/artboard.rs",
    "crates/nuxie-runtime/src/components.rs",
    "crates/nuxie-runtime/src/draw.rs",
    "crates/nuxie-graph/src/lib.rs"
  ],
  "verdict": "UNKNOWN",
  "axes": {
    "retained_identity": {
      "status": "unknown: no Rust ClampedScrollPhysics state",
      "evidence": [
        "cpp@d788e8ec:src/constraints/scrolling/clamped_scroll_physics.cpp:6-31",
        "crates/nuxie-runtime/src/constraints.rs:1494-1506"
      ]
    },
    "push_vs_poll": {
      "status": "unknown: pure clamped_scroll_offset is not the C++ physics run/advance lifecycle",
      "cpp_pushes": false,
      "evidence": [
        "cpp@d788e8ec:src/constraints/scrolling/clamped_scroll_physics.cpp:6-31",
        "crates/nuxie-runtime/src/constraints.rs:1494-1506"
      ]
    },
    "update_ordering": {
      "status": "unknown",
      "phases_cpp": "run stores m_value -> advance stops and returns it",
      "phases_rust": "blocker: only a stateless clamp helper exists"
    },
    "ownership": {
      "status": "unknown: C++ inherited physics object has retained m_value; no Rust object counterpart",
      "evidence": [
        "cpp@d788e8ec:src/constraints/scrolling/clamped_scroll_physics.cpp:6-31",
        "crates/nuxie-runtime/src/constraints.rs:1-4012"
      ]
    },
    "compensation": {
      "status": "not assessed beyond absent mapped seam",
      "mechanisms": [],
      "import_time_constants": [
        {
          "name": "precomputed constraint membership/dependency descriptors",
          "idiom_rule": "AF-5 import-time devirtualization",
          "evidence": [
            "crates/nuxie-graph/src/lib.rs:3084-3143,3655-3716,4977-5023",
            "crates/nuxie-runtime/src/components.rs:477-512"
          ]
        }
      ]
    }
  },
  "idiom_rules_invoked": [
    "AF-5 import-time devirtualization"
  ],
  "confidence": "high",
  "notes": "Blocker: the mapped Rust module contains finite offset clamping but no ClampedScrollPhysics run/advance/stop object, so equivalence cannot be judged honestly."
}
```

## B6-0138

```json
{
  "row_id": "B6-0138",
  "cpp_files": [
    "src/constraints/scrolling/scroll_constraint.cpp"
  ],
  "rust_module": "crates/nuxie-runtime/src/constraints.rs",
  "subsystem_cluster": "constraints",
  "sibling_files_swept": [
    "crates/nuxie-runtime/src/constraints.rs",
    "crates/nuxie-runtime/src/artboard.rs",
    "crates/nuxie-runtime/src/components.rs",
    "crates/nuxie-runtime/src/draw.rs",
    "crates/nuxie-graph/src/lib.rs"
  ],
  "verdict": "DIVERGENT",
  "axes": {
    "retained_identity": {
      "status": "partial retained scroll state",
      "idiom_rule": "AF-1 retained identity",
      "evidence": [
        "cpp@d788e8ec:src/constraints/scrolling/scroll_constraint.cpp:14-237,401-641",
        "crates/nuxie-runtime/src/constraints.rs:26-176,500-739,1698-1888"
      ]
    },
    "push_vs_poll": {
      "status": "partial: scroll property dirt is pushed, but physics/drag advance is absent; readiness uses a Rust latch",
      "cpp_pushes": true,
      "evidence": [
        "cpp@d788e8ec:src/constraints/scrolling/scroll_constraint.cpp:170-187,194-237,299-335",
        "crates/nuxie-runtime/src/artboard.rs:4962-4975",
        "crates/nuxie-runtime/src/constraints.rs:1698-1888"
      ]
    },
    "update_ordering": {
      "status": "core constrain/intents equivalent; lifecycle incomplete",
      "phases_cpp": "advance physics/drag -> resolve intents -> clamp/constrain children -> virtualize",
      "phases_rust": "property/update -> layout_initialized latch -> resolve intents -> clamp/constrain children; no physics advance"
    },
    "ownership": {
      "status": "partial: IDs/intents retained by value; no ScrollPhysics/proxy ownership",
      "evidence": [
        "cpp@d788e8ec:src/constraints/scrolling/scroll_constraint.cpp:14-23,364-423",
        "crates/nuxie-runtime/src/constraints.rs:26-56,500-527"
      ]
    },
    "compensation": {
      "status": "present",
      "mechanisms": [
        {
          "name": "layout_initialized readiness latch",
          "kind": "update-written lifecycle flag gating later intent resolution",
          "mutation_gated": true,
          "cpp_counterpart": "none",
          "evidence": [
            "crates/nuxie-runtime/src/constraints.rs:27-34,517-524,625-638,1766-1772"
          ]
        },
        {
          "name": "cache/prepared/command epoch fanout",
          "kind": "mutation-time parallel invalidation counters beside ComponentDirt",
          "mutation_gated": true,
          "cpp_counterpart": "none (the C++ constraint family uses retained ComponentDirt and markConstraintDirty)",
          "evidence": [
            "crates/nuxie-runtime/src/artboard.rs:2115-2134,3728-3755"
          ]
        }
      ],
      "import_time_constants": [
        {
          "name": "precomputed constraint membership/dependency descriptors",
          "idiom_rule": "AF-5 import-time devirtualization",
          "evidence": [
            "crates/nuxie-graph/src/lib.rs:3084-3143,3655-3716,4977-5023",
            "crates/nuxie-runtime/src/components.rs:477-512"
          ]
        },
        {
          "name": "scroll property-key OnceLock",
          "idiom_rule": "AF-5 import-time devirtualization",
          "evidence": [
            "crates/nuxie-runtime/src/constraints.rs:77-105"
          ]
        }
      ]
    }
  },
  "idiom_rules_invoked": [
    "AF-1 retained identity",
    "AF-4 one dirt model",
    "AF-5 import-time devirtualization",
    "AF-8 no invented lifecycles"
  ],
  "confidence": "high",
  "notes": "Behavior-bug note: C++ drag/velocity/snap/physics advance surfaces (299-335,401-465,986-1082) have no mapped Rust implementation. Verdict is supported independently by the mutation-gated readiness latch."
}
```

## B6-0139

```json
{
  "row_id": "B6-0139",
  "cpp_files": [
    "src/constraints/scrolling/scroll_constraint_proxy.cpp"
  ],
  "rust_module": "crates/nuxie-runtime/src/constraints.rs",
  "subsystem_cluster": "constraints",
  "sibling_files_swept": [
    "crates/nuxie-runtime/src/constraints.rs",
    "crates/nuxie-runtime/src/artboard.rs",
    "crates/nuxie-runtime/src/components.rs",
    "crates/nuxie-runtime/src/draw.rs",
    "crates/nuxie-graph/src/lib.rs"
  ],
  "verdict": "UNKNOWN",
  "axes": {
    "retained_identity": {
      "status": "unknown: no Rust ViewportDraggableProxy identity",
      "evidence": [
        "cpp@d788e8ec:src/constraints/scrolling/scroll_constraint_proxy.cpp:8-81",
        "crates/nuxie-runtime/src/constraints.rs:1-4012"
      ]
    },
    "push_vs_poll": {
      "status": "unknown: no pointer drag proxy event surface exists",
      "cpp_pushes": true,
      "evidence": [
        "cpp@d788e8ec:src/constraints/scrolling/scroll_constraint_proxy.cpp:8-81",
        "crates/nuxie-runtime/src/constraints.rs:1-4012"
      ]
    },
    "update_ordering": {
      "status": "unknown",
      "phases_cpp": "startDrag -> thresholded drag -> endDrag/runPhysics",
      "phases_rust": "blocker: no mapped proxy/event lifecycle"
    },
    "ownership": {
      "status": "unknown: C++ proxy owns lastPosition/isDragging and points to constraint; no Rust counterpart",
      "evidence": [
        "cpp@d788e8ec:src/constraints/scrolling/scroll_constraint_proxy.cpp:8-81",
        "crates/nuxie-runtime/src/constraints.rs:1-4012"
      ]
    },
    "compensation": {
      "status": "not assessed beyond absent mapped seam",
      "mechanisms": [],
      "import_time_constants": [
        {
          "name": "precomputed constraint membership/dependency descriptors",
          "idiom_rule": "AF-5 import-time devirtualization",
          "evidence": [
            "crates/nuxie-graph/src/lib.rs:3084-3143,3655-3716,4977-5023",
            "crates/nuxie-runtime/src/components.rs:477-512"
          ]
        }
      ]
    }
  },
  "idiom_rules_invoked": [
    "AF-5 import-time devirtualization"
  ],
  "confidence": "high",
  "notes": "Blocker: crate-wide constraint/scroll/proxy/drag grep found no ViewportDraggableProxy or equivalent start/drag/end state machine."
}
```

## B6-0140

```json
{
  "row_id": "B6-0140",
  "cpp_files": [
    "src/constraints/scrolling/scroll_physics.cpp"
  ],
  "rust_module": "crates/nuxie-runtime/src/constraints.rs",
  "subsystem_cluster": "constraints",
  "sibling_files_swept": [
    "crates/nuxie-runtime/src/constraints.rs",
    "crates/nuxie-runtime/src/artboard.rs",
    "crates/nuxie-runtime/src/components.rs",
    "crates/nuxie-runtime/src/draw.rs",
    "crates/nuxie-graph/src/lib.rs"
  ],
  "verdict": "UNKNOWN",
  "axes": {
    "retained_identity": {
      "status": "unknown: no Rust ScrollPhysics state object",
      "evidence": [
        "cpp@d788e8ec:src/constraints/scrolling/scroll_physics.cpp:8-65",
        "crates/nuxie-runtime/src/constraints.rs:1-4012"
      ]
    },
    "push_vs_poll": {
      "status": "unknown: no accumulated delta/time/speed/acceleration seam exists",
      "cpp_pushes": false,
      "evidence": [
        "cpp@d788e8ec:src/constraints/scrolling/scroll_physics.cpp:8-50",
        "crates/nuxie-runtime/src/constraints.rs:1-4012"
      ]
    },
    "update_ordering": {
      "status": "unknown",
      "phases_cpp": "accumulate during drag -> run -> advance/reset",
      "phases_rust": "blocker: no mapped physics lifecycle"
    },
    "ownership": {
      "status": "unknown: C++ physics is cloned/owned by ScrollConstraint; no Rust counterpart",
      "evidence": [
        "cpp@d788e8ec:src/constraints/scrolling/scroll_physics.cpp:53-65",
        "cpp@d788e8ec:src/constraints/scrolling/scroll_constraint.cpp:14-23,364-398"
      ]
    },
    "compensation": {
      "status": "not assessed beyond absent mapped seam",
      "mechanisms": [],
      "import_time_constants": [
        {
          "name": "precomputed constraint membership/dependency descriptors",
          "idiom_rule": "AF-5 import-time devirtualization",
          "evidence": [
            "crates/nuxie-graph/src/lib.rs:3084-3143,3655-3716,4977-5023",
            "crates/nuxie-runtime/src/components.rs:477-512"
          ]
        }
      ]
    }
  },
  "idiom_rules_invoked": [
    "AF-5 import-time devirtualization"
  ],
  "confidence": "high",
  "notes": "Blocker: constraints.rs has no velocity, acceleration, timestamp, deterministic-mode, run, reset, or imported physics object. UNKNOWN avoids treating a pure clamp helper as a port."
}
```

## B6-0141

```json
{
  "row_id": "B6-0141",
  "cpp_files": [
    "src/constraints/scrolling/scroll_virtualizer.cpp"
  ],
  "rust_module": "crates/nuxie-runtime/src/constraints.rs",
  "subsystem_cluster": "constraints",
  "sibling_files_swept": [
    "crates/nuxie-runtime/src/constraints.rs",
    "crates/nuxie-runtime/src/artboard.rs",
    "crates/nuxie-runtime/src/components.rs",
    "crates/nuxie-runtime/src/draw.rs",
    "crates/nuxie-graph/src/lib.rs"
  ],
  "verdict": "DIVERGENT",
  "axes": {
    "retained_identity": {
      "status": "mixed: mounted ArtboardInstances are retained, but the visible window is reconstructed/diffed",
      "evidence": [
        "cpp@d788e8ec:src/constraints/scrolling/scroll_virtualizer.cpp:8-10,54-370",
        "crates/nuxie-runtime/src/artboard.rs:500-517,2611-2717",
        "crates/nuxie-runtime/src/constraints.rs:1223-1338,1534-1661"
      ]
    },
    "push_vs_poll": {
      "status": "divergent: C++ mutates retained visible ranges/recycles directly; Rust rebuilds a desired window and polls identity/generation equality",
      "cpp_pushes": true,
      "evidence": [
        "cpp@d788e8ec:src/constraints/scrolling/scroll_virtualizer.cpp:63-75,178-330",
        "crates/nuxie-runtime/src/artboard.rs:2611-2667",
        "crates/nuxie-runtime/src/constraints.rs:1534-1661"
      ]
    },
    "update_ordering": {
      "status": "divergent representation with bounded feedback",
      "phases_cpp": "constrain -> update retained visible range -> recycle/add/move -> virtualizableChanged",
      "phases_rust": "bind/update -> reconstruct desired window -> generation/diff check -> reuse/rebuild mounted vector -> bounded layout feedback"
    },
    "ownership": {
      "status": "mixed: retained child identity is preserved by occurrence_identity, while window topology is rebuilt as owned vectors",
      "evidence": [
        "cpp@d788e8ec:src/constraints/scrolling/scroll_virtualizer.cpp:178-370",
        "crates/nuxie-runtime/src/artboard.rs:2634-2808"
      ]
    },
    "compensation": {
      "status": "present",
      "mechanisms": [
        {
          "name": "desired-window reconstruction and full diff",
          "kind": "update-time reconstruct-and-compare loop",
          "mutation_gated": true,
          "cpp_counterpart": "none",
          "evidence": [
            "crates/nuxie-runtime/src/constraints.rs:1223-1338,1534-1661",
            "crates/nuxie-runtime/src/artboard.rs:2611-2667"
          ]
        },
        {
          "name": "mutation_generation window resync",
          "kind": "generation counter plus bind/update-time equality poll",
          "mutation_gated": true,
          "cpp_counterpart": "none",
          "evidence": [
            "crates/nuxie-runtime/src/view_model.rs:7009-7016",
            "crates/nuxie-runtime/src/artboard.rs:2634-2650,2683-2713"
          ]
        },
        {
          "name": "virtual-window layout/prepared epoch fanout",
          "kind": "window-change counters consumed by later layout/render preparation",
          "mutation_gated": true,
          "cpp_counterpart": "none",
          "evidence": [
            "crates/nuxie-runtime/src/artboard.rs:2809-2814,3728-3785"
          ]
        }
      ],
      "import_time_constants": [
        {
          "name": "precomputed constraint membership/dependency descriptors",
          "idiom_rule": "AF-5 import-time devirtualization",
          "evidence": [
            "crates/nuxie-graph/src/lib.rs:3084-3143,3655-3716,4977-5023",
            "crates/nuxie-runtime/src/components.rs:477-512"
          ]
        }
      ]
    }
  },
  "idiom_rules_invoked": [
    "AF-1 retained identity",
    "AF-2 push never reconstruct",
    "AF-4 one dirt model",
    "AF-5 import-time devirtualization",
    "AF-8 no invented lifecycles"
  ],
  "confidence": "high",
  "notes": "Current mid-RB-1 state recorded mechanically: the mutation_generation member is scheduled by the #RB-1 mini-queue at docs/parity-closeout-status.md:210-225, not re-triaged here. The C++ visible-range state and Rust retained mounted children were both included in the comparison."
}
```

## B6-0142

```json
{
  "row_id": "B6-0142",
  "cpp_files": [
    "src/constraints/targeted_constraint.cpp"
  ],
  "rust_module": "crates/nuxie-runtime/src/constraints.rs",
  "subsystem_cluster": "constraints",
  "sibling_files_swept": [
    "crates/nuxie-runtime/src/constraints.rs",
    "crates/nuxie-runtime/src/artboard.rs",
    "crates/nuxie-runtime/src/components.rs",
    "crates/nuxie-runtime/src/draw.rs",
    "crates/nuxie-graph/src/lib.rs"
  ],
  "verdict": "ADAPTED",
  "axes": {
    "retained_identity": {
      "status": "adapted arena-ID identity",
      "idiom_rule": "AF-1 retained identity",
      "evidence": [
        "cpp@d788e8ec:src/constraints/targeted_constraint.cpp:7-48",
        "crates/nuxie-graph/src/lib.rs:3655-3688; crates/nuxie-runtime/src/constraints.rs:3324-3332"
      ]
    },
    "push_vs_poll": {
      "status": "adapted: target dependency remains push-driven; target pointer is an arena lookup by stable local ID",
      "cpp_pushes": true,
      "evidence": [
        "cpp@d788e8ec:src/constraints/targeted_constraint.cpp:41-48",
        "crates/nuxie-graph/src/lib.rs:3655-3688",
        "crates/nuxie-runtime/src/artboard.rs:3948-3990"
      ]
    },
    "update_ordering": {
      "status": "isomorphic dependency phase",
      "phases_cpp": "validate/resolve target -> register target dependent -> constrain after target",
      "phases_rust": "validate graph IDs -> precompute target edge -> constrain after target"
    },
    "ownership": {
      "status": "adapted non-owning pointer to stable arena ID",
      "evidence": [
        "cpp@d788e8ec:src/constraints/targeted_constraint.cpp:23-38",
        "crates/nuxie-runtime/src/constraints.rs:3324-3332"
      ]
    },
    "compensation": {
      "status": "clear after crate-wide family grep and sibling sweep",
      "mechanisms": [],
      "import_time_constants": [
        {
          "name": "precomputed constraint membership/dependency descriptors",
          "idiom_rule": "AF-5 import-time devirtualization",
          "evidence": [
            "crates/nuxie-graph/src/lib.rs:3084-3143,3655-3716,4977-5023",
            "crates/nuxie-runtime/src/components.rs:477-512"
          ]
        }
      ]
    }
  },
  "idiom_rules_invoked": [
    "AF-1 retained identity",
    "AF-5 import-time devirtualization"
  ],
  "confidence": "high",
  "notes": "The per-apply local-ID lookup is the arena form of C++ non-owning identity, not a drift poll. All listed siblings were swept; no target-specific cycle-written generation/snapshot/latch was found."
}
```

## B6-0143

```json
{
  "row_id": "B6-0143",
  "cpp_files": [
    "src/constraints/transform_constraint.cpp"
  ],
  "rust_module": "crates/nuxie-runtime/src/constraints.rs",
  "subsystem_cluster": "constraints",
  "sibling_files_swept": [
    "crates/nuxie-runtime/src/constraints.rs",
    "crates/nuxie-runtime/src/artboard.rs",
    "crates/nuxie-runtime/src/components.rs",
    "crates/nuxie-runtime/src/draw.rs",
    "crates/nuxie-graph/src/lib.rs"
  ],
  "verdict": "DIVERGENT",
  "axes": {
    "retained_identity": {
      "status": "adapted-arena-identity",
      "idiom_rule": "AF-1 retained identity",
      "evidence": [
        "cpp@d788e8ec:src/constraints/transform_constraint.cpp:9-90",
        "crates/nuxie-runtime/src/constraints.rs:2362-2411,2935-3026"
      ]
    },
    "push_vs_poll": {
      "status": "isomorphic push for target dependency; extra counters counted under compensation",
      "cpp_pushes": true,
      "evidence": [
        "cpp@d788e8ec:src/constraints/transform_constraint.cpp:18-20,22-54",
        "crates/nuxie-graph/src/lib.rs:3655-3688",
        "crates/nuxie-runtime/src/artboard.rs:3948-3990"
      ]
    },
    "update_ordering": {
      "status": "phase-sequence equivalent; representation divergent",
      "phases_cpp": "target/origin dirt -> world update -> constrain",
      "phases_rust": "dependency/property mutation -> epoch writes -> world update -> constrain"
    },
    "ownership": {
      "status": "adapted arena target identity and by-value matrix/components",
      "evidence": [
        "cpp@d788e8ec:src/constraints/transform_constraint.cpp:9-90",
        "crates/nuxie-runtime/src/constraints.rs:2362-2411,2935-3026"
      ]
    },
    "compensation": {
      "status": "present",
      "mechanisms": [
        {
          "name": "cache/prepared/command epoch fanout",
          "kind": "mutation-time parallel invalidation counters beside ComponentDirt",
          "mutation_gated": true,
          "cpp_counterpart": "none (the C++ constraint family uses retained ComponentDirt and markConstraintDirty)",
          "evidence": [
            "crates/nuxie-runtime/src/artboard.rs:2115-2134,3728-3755"
          ]
        }
      ],
      "import_time_constants": [
        {
          "name": "precomputed constraint membership/dependency descriptors",
          "idiom_rule": "AF-5 import-time devirtualization",
          "evidence": [
            "crates/nuxie-graph/src/lib.rs:3084-3143,3655-3716,4977-5023",
            "crates/nuxie-runtime/src/components.rs:477-512"
          ]
        }
      ]
    }
  },
  "idiom_rules_invoked": [
    "AF-1 retained identity",
    "AF-4 one dirt model",
    "AF-5 import-time devirtualization",
    "AF-7 own-by-value"
  ],
  "confidence": "high",
  "notes": "Behavior-bug note: C++ originX/originY callbacks push constraint dirt; the current explicit Rust double-property dispatch has no TransformConstraint origin arm."
}
```

## B6-0144

```json
{
  "row_id": "B6-0144",
  "cpp_files": [
    "src/constraints/translation_constraint.cpp"
  ],
  "rust_module": "crates/nuxie-runtime/src/constraints.rs",
  "subsystem_cluster": "constraints",
  "sibling_files_swept": [
    "crates/nuxie-runtime/src/constraints.rs",
    "crates/nuxie-runtime/src/artboard.rs",
    "crates/nuxie-runtime/src/components.rs",
    "crates/nuxie-runtime/src/draw.rs",
    "crates/nuxie-graph/src/lib.rs"
  ],
  "verdict": "DIVERGENT",
  "axes": {
    "retained_identity": {
      "status": "adapted-arena-identity",
      "idiom_rule": "AF-1 retained identity",
      "evidence": [
        "cpp@d788e8ec:src/constraints/translation_constraint.cpp:9-115",
        "crates/nuxie-runtime/src/constraints.rs:3028-3257"
      ]
    },
    "push_vs_poll": {
      "status": "isomorphic push for target dependency; extra counters counted under compensation",
      "cpp_pushes": true,
      "evidence": [
        "cpp@d788e8ec:src/constraints/translation_constraint.cpp:9-115",
        "crates/nuxie-graph/src/lib.rs:3655-3688",
        "crates/nuxie-runtime/src/artboard.rs:3948-3990"
      ]
    },
    "update_ordering": {
      "status": "phase-sequence equivalent; representation divergent",
      "phases_cpp": "target/property dirt -> world update -> constrain",
      "phases_rust": "dependency/property mutation -> epoch writes -> world update -> constrain"
    },
    "ownership": {
      "status": "adapted arena target identity and by-value vector/matrix temporaries",
      "evidence": [
        "cpp@d788e8ec:src/constraints/translation_constraint.cpp:15-114",
        "crates/nuxie-runtime/src/constraints.rs:3040-3256"
      ]
    },
    "compensation": {
      "status": "present",
      "mechanisms": [
        {
          "name": "cache/prepared/command epoch fanout",
          "kind": "mutation-time parallel invalidation counters beside ComponentDirt",
          "mutation_gated": true,
          "cpp_counterpart": "none (the C++ constraint family uses retained ComponentDirt and markConstraintDirty)",
          "evidence": [
            "crates/nuxie-runtime/src/artboard.rs:2115-2134,3728-3755"
          ]
        }
      ],
      "import_time_constants": [
        {
          "name": "precomputed constraint membership/dependency descriptors",
          "idiom_rule": "AF-5 import-time devirtualization",
          "evidence": [
            "crates/nuxie-graph/src/lib.rs:3084-3143,3655-3716,4977-5023",
            "crates/nuxie-runtime/src/components.rs:477-512"
          ]
        }
      ]
    }
  },
  "idiom_rules_invoked": [
    "AF-1 retained identity",
    "AF-4 one dirt model",
    "AF-5 import-time devirtualization",
    "AF-7 own-by-value"
  ],
  "confidence": "high",
  "notes": "The translate/copy/clamp/interpolate math is structurally direct. DIVERGENT is based on the off-file mutation-gated epoch family, not ordinary by-value temporaries."
}
```

