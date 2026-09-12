# Normalized boundary direct visual review — root

Directly inspected sheets00–12 at full image resolution using view_image. Chrome/native pairs and diff panels were viewed, not inferred from metric results.

Saturation-only scenes have matching white coverage; spanning scenes have matching yellow extents. Zero/limit/next-above/next-below scenes no longer show the extra native yellow hairline. Thin-above and half-extent retain the same one-pixel paint in both renderings. Row/column accumulated-decimal scenes have matching edge placement, including the next-line wrap; the former edge band is absent. Faint filled-region residuals remain in teal/yellow diff panels, so this is not an identical-RGBA claim. No missing paint or unexpected clipping observed in these sheets. Alpha overlap scenes are assigned to the complementary agent review.

Inspected sheet and constituent image identities follow:

```json
[
  {
    "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/visual/sheet-00.png",
    "sha256": "43acd6a246a87dc7afc5c04fca14e56a3b076950271f532ff6c8f0c3fc818893",
    "members": [
      {
        "name": "boundary-row-negative-saturation",
        "frame": 0,
        "files": [
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-negative-saturation/frame-0.chrome.png",
            "sha256": "edf0f885bfb44d995fa5eb08149fd103e3efc728de089f2e8fb0a7be8022ceb4"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-negative-saturation/frame-0.native.png",
            "sha256": "da676042d72c2d11398a0ee55532d04050c903d54cd8bfe40aa8f747c128f426"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-negative-saturation/frame-0.diff.png",
            "sha256": "27356741c7369acd914ea2e2476776b18ff864f74893f9086058843e709deb57"
          }
        ],
        "pixelFailures": []
      },
      {
        "name": "boundary-row-positive-saturation",
        "frame": 0,
        "files": [
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-positive-saturation/frame-0.chrome.png",
            "sha256": "edf0f885bfb44d995fa5eb08149fd103e3efc728de089f2e8fb0a7be8022ceb4"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-positive-saturation/frame-0.native.png",
            "sha256": "da676042d72c2d11398a0ee55532d04050c903d54cd8bfe40aa8f747c128f426"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-positive-saturation/frame-0.diff.png",
            "sha256": "27356741c7369acd914ea2e2476776b18ff864f74893f9086058843e709deb57"
          }
        ],
        "pixelFailures": []
      },
      {
        "name": "boundary-row-spanning",
        "frame": 0,
        "files": [
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-spanning/frame-0.chrome.png",
            "sha256": "1e00934e5c61a782105eb3a1fb2b8e21a29b18d2efead6d1749ab8e669157be2"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-spanning/frame-0.native.png",
            "sha256": "ef323ab394062331118607546fbd3d40aadbb313bbccd8b47a3c48f5b9a4e881"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-spanning/frame-0.diff.png",
            "sha256": "32fd9a3bf7c92acfe6c7ecd2ba042d7a0170f842d7e453a8952f879b455cee61"
          }
        ],
        "pixelFailures": []
      },
      {
        "name": "boundary-row-negative-spanning",
        "frame": 0,
        "files": [
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-negative-spanning/frame-0.chrome.png",
            "sha256": "1e00934e5c61a782105eb3a1fb2b8e21a29b18d2efead6d1749ab8e669157be2"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-negative-spanning/frame-0.native.png",
            "sha256": "ef323ab394062331118607546fbd3d40aadbb313bbccd8b47a3c48f5b9a4e881"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-negative-spanning/frame-0.diff.png",
            "sha256": "32fd9a3bf7c92acfe6c7ecd2ba042d7a0170f842d7e453a8952f879b455cee61"
          }
        ],
        "pixelFailures": []
      }
    ]
  },
  {
    "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/visual/sheet-01.png",
    "sha256": "bd0ce4eed70ab8ebc65c231f7550f169462b0c0f0ec772744a009dfa9743746b",
    "members": [
      {
        "name": "boundary-row-zero",
        "frame": 0,
        "files": [
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-zero/frame-0.chrome.png",
            "sha256": "edf0f885bfb44d995fa5eb08149fd103e3efc728de089f2e8fb0a7be8022ceb4"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-zero/frame-0.native.png",
            "sha256": "da676042d72c2d11398a0ee55532d04050c903d54cd8bfe40aa8f747c128f426"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-zero/frame-0.diff.png",
            "sha256": "27356741c7369acd914ea2e2476776b18ff864f74893f9086058843e709deb57"
          }
        ],
        "pixelFailures": []
      },
      {
        "name": "boundary-row-thin-limit",
        "frame": 0,
        "files": [
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-thin-limit/frame-0.chrome.png",
            "sha256": "edf0f885bfb44d995fa5eb08149fd103e3efc728de089f2e8fb0a7be8022ceb4"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-thin-limit/frame-0.native.png",
            "sha256": "da676042d72c2d11398a0ee55532d04050c903d54cd8bfe40aa8f747c128f426"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-thin-limit/frame-0.diff.png",
            "sha256": "27356741c7369acd914ea2e2476776b18ff864f74893f9086058843e709deb57"
          }
        ],
        "pixelFailures": []
      },
      {
        "name": "boundary-row-thin-next-above",
        "frame": 0,
        "files": [
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-thin-next-above/frame-0.chrome.png",
            "sha256": "edf0f885bfb44d995fa5eb08149fd103e3efc728de089f2e8fb0a7be8022ceb4"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-thin-next-above/frame-0.native.png",
            "sha256": "da676042d72c2d11398a0ee55532d04050c903d54cd8bfe40aa8f747c128f426"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-thin-next-above/frame-0.diff.png",
            "sha256": "27356741c7369acd914ea2e2476776b18ff864f74893f9086058843e709deb57"
          }
        ],
        "pixelFailures": []
      },
      {
        "name": "boundary-row-thin-next-below",
        "frame": 0,
        "files": [
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-thin-next-below/frame-0.chrome.png",
            "sha256": "edf0f885bfb44d995fa5eb08149fd103e3efc728de089f2e8fb0a7be8022ceb4"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-thin-next-below/frame-0.native.png",
            "sha256": "da676042d72c2d11398a0ee55532d04050c903d54cd8bfe40aa8f747c128f426"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-thin-next-below/frame-0.diff.png",
            "sha256": "27356741c7369acd914ea2e2476776b18ff864f74893f9086058843e709deb57"
          }
        ],
        "pixelFailures": []
      }
    ]
  },
  {
    "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/visual/sheet-02.png",
    "sha256": "18736024414dd6c603aa6d0aad73c3703112728d80ccb8af2f85f5fa19063c03",
    "members": [
      {
        "name": "boundary-row-thin-above",
        "frame": 0,
        "files": [
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-thin-above/frame-0.chrome.png",
            "sha256": "906fdd0d36c21020c18d89f1bcb99c51ae32dbdf2bf8c37dfbf889cf28378865"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-thin-above/frame-0.native.png",
            "sha256": "056f2dd79b63b77a3f7d0b76d56b60f497ed09a9483f02761e43895ebafc6957"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-thin-above/frame-0.diff.png",
            "sha256": "fde3e95125c6756312af58c7f2a53b2dce1b17ae0638a127035821b2de5e2e74"
          }
        ],
        "pixelFailures": []
      },
      {
        "name": "boundary-row-half-extent",
        "frame": 0,
        "files": [
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-half-extent/frame-0.chrome.png",
            "sha256": "906fdd0d36c21020c18d89f1bcb99c51ae32dbdf2bf8c37dfbf889cf28378865"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-half-extent/frame-0.native.png",
            "sha256": "056f2dd79b63b77a3f7d0b76d56b60f497ed09a9483f02761e43895ebafc6957"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-half-extent/frame-0.diff.png",
            "sha256": "fde3e95125c6756312af58c7f2a53b2dce1b17ae0638a127035821b2de5e2e74"
          }
        ],
        "pixelFailures": []
      },
      {
        "name": "boundary-row-accumulated-exact",
        "frame": 0,
        "files": [
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-accumulated-exact/frame-0.chrome.png",
            "sha256": "ef6d705b5dcf3568b4fe350dab281fae8c0085b82aabdc4384bcaf402d32fe7e"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-accumulated-exact/frame-0.native.png",
            "sha256": "b0b93619f70ad9ccae06be95591c68603be3fc5d0e8f93acd6fbf6a92ac7e554"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-accumulated-exact/frame-0.diff.png",
            "sha256": "a05b90f54d595e7e8e34c8b73dd89319426a6eda8cc49532144de5bf39b37078"
          }
        ],
        "pixelFailures": []
      },
      {
        "name": "boundary-row-accumulated-decimal",
        "frame": 0,
        "files": [
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-accumulated-decimal/frame-0.chrome.png",
            "sha256": "245369a943180741d227d3fee112b219987b9a39b0fad04bbfd581d50133e31c"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-accumulated-decimal/frame-0.native.png",
            "sha256": "54a3bd03decf2f042bf7962f2c994d1578a66facd77215884ca2526a23acff90"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-accumulated-decimal/frame-0.diff.png",
            "sha256": "75e1aec0ec203139c66d15aee7cff15f7a8224878f83ac27b804f47097ea63cb"
          }
        ],
        "pixelFailures": []
      }
    ]
  },
  {
    "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/visual/sheet-03.png",
    "sha256": "402863948176e39a2d6ae187aaac6037b95d9fd353d659f05549ae7c3ef08497",
    "members": [
      {
        "name": "boundary-column-negative-saturation",
        "frame": 0,
        "files": [
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-column-negative-saturation/frame-0.chrome.png",
            "sha256": "edf0f885bfb44d995fa5eb08149fd103e3efc728de089f2e8fb0a7be8022ceb4"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-column-negative-saturation/frame-0.native.png",
            "sha256": "da676042d72c2d11398a0ee55532d04050c903d54cd8bfe40aa8f747c128f426"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-column-negative-saturation/frame-0.diff.png",
            "sha256": "27356741c7369acd914ea2e2476776b18ff864f74893f9086058843e709deb57"
          }
        ],
        "pixelFailures": []
      },
      {
        "name": "boundary-column-positive-saturation",
        "frame": 0,
        "files": [
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-column-positive-saturation/frame-0.chrome.png",
            "sha256": "edf0f885bfb44d995fa5eb08149fd103e3efc728de089f2e8fb0a7be8022ceb4"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-column-positive-saturation/frame-0.native.png",
            "sha256": "da676042d72c2d11398a0ee55532d04050c903d54cd8bfe40aa8f747c128f426"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-column-positive-saturation/frame-0.diff.png",
            "sha256": "27356741c7369acd914ea2e2476776b18ff864f74893f9086058843e709deb57"
          }
        ],
        "pixelFailures": []
      },
      {
        "name": "boundary-column-spanning",
        "frame": 0,
        "files": [
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-column-spanning/frame-0.chrome.png",
            "sha256": "f26d3dc91deea5791f72a61d0db03a116bd15504f300fd253b6525d7a90d35f7"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-column-spanning/frame-0.native.png",
            "sha256": "7bf0fc93be5e39ec11b29e217572b3ea0c5813027f63a1c3d5fc266f92f71ba6"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-column-spanning/frame-0.diff.png",
            "sha256": "57075a5eaa594720a9857c02ddbcf1b0d65c2401ff6ef9bee037a05cc13fdef9"
          }
        ],
        "pixelFailures": []
      },
      {
        "name": "boundary-column-negative-spanning",
        "frame": 0,
        "files": [
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-column-negative-spanning/frame-0.chrome.png",
            "sha256": "f26d3dc91deea5791f72a61d0db03a116bd15504f300fd253b6525d7a90d35f7"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-column-negative-spanning/frame-0.native.png",
            "sha256": "7bf0fc93be5e39ec11b29e217572b3ea0c5813027f63a1c3d5fc266f92f71ba6"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-column-negative-spanning/frame-0.diff.png",
            "sha256": "57075a5eaa594720a9857c02ddbcf1b0d65c2401ff6ef9bee037a05cc13fdef9"
          }
        ],
        "pixelFailures": []
      }
    ]
  },
  {
    "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/visual/sheet-04.png",
    "sha256": "926a4d36138b00c2502427cea42d0b10c257b3d1ab77a3f30ba69a56018386d7",
    "members": [
      {
        "name": "boundary-column-zero",
        "frame": 0,
        "files": [
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-column-zero/frame-0.chrome.png",
            "sha256": "edf0f885bfb44d995fa5eb08149fd103e3efc728de089f2e8fb0a7be8022ceb4"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-column-zero/frame-0.native.png",
            "sha256": "da676042d72c2d11398a0ee55532d04050c903d54cd8bfe40aa8f747c128f426"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-column-zero/frame-0.diff.png",
            "sha256": "27356741c7369acd914ea2e2476776b18ff864f74893f9086058843e709deb57"
          }
        ],
        "pixelFailures": []
      },
      {
        "name": "boundary-column-thin-limit",
        "frame": 0,
        "files": [
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-column-thin-limit/frame-0.chrome.png",
            "sha256": "edf0f885bfb44d995fa5eb08149fd103e3efc728de089f2e8fb0a7be8022ceb4"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-column-thin-limit/frame-0.native.png",
            "sha256": "da676042d72c2d11398a0ee55532d04050c903d54cd8bfe40aa8f747c128f426"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-column-thin-limit/frame-0.diff.png",
            "sha256": "27356741c7369acd914ea2e2476776b18ff864f74893f9086058843e709deb57"
          }
        ],
        "pixelFailures": []
      },
      {
        "name": "boundary-column-thin-next-above",
        "frame": 0,
        "files": [
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-column-thin-next-above/frame-0.chrome.png",
            "sha256": "edf0f885bfb44d995fa5eb08149fd103e3efc728de089f2e8fb0a7be8022ceb4"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-column-thin-next-above/frame-0.native.png",
            "sha256": "da676042d72c2d11398a0ee55532d04050c903d54cd8bfe40aa8f747c128f426"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-column-thin-next-above/frame-0.diff.png",
            "sha256": "27356741c7369acd914ea2e2476776b18ff864f74893f9086058843e709deb57"
          }
        ],
        "pixelFailures": []
      },
      {
        "name": "boundary-column-thin-next-below",
        "frame": 0,
        "files": [
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-column-thin-next-below/frame-0.chrome.png",
            "sha256": "edf0f885bfb44d995fa5eb08149fd103e3efc728de089f2e8fb0a7be8022ceb4"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-column-thin-next-below/frame-0.native.png",
            "sha256": "da676042d72c2d11398a0ee55532d04050c903d54cd8bfe40aa8f747c128f426"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-column-thin-next-below/frame-0.diff.png",
            "sha256": "27356741c7369acd914ea2e2476776b18ff864f74893f9086058843e709deb57"
          }
        ],
        "pixelFailures": []
      }
    ]
  },
  {
    "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/visual/sheet-05.png",
    "sha256": "6c95df2c2101e49e70f8e6eb48f6a712bdac685eddf05888f334845cfa35d68c",
    "members": [
      {
        "name": "boundary-column-thin-above",
        "frame": 0,
        "files": [
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-column-thin-above/frame-0.chrome.png",
            "sha256": "68263645687bba1dbdf1de6e5daddf2bae3f749300fa89881a43f384df7e3673"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-column-thin-above/frame-0.native.png",
            "sha256": "4fd9386d7bba777babc69eb6d6f3efaa909c6dfc3d0dafacd217f7e1987ad562"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-column-thin-above/frame-0.diff.png",
            "sha256": "2f7edd5f243de343698f07c181c430c3f0a9254795f25ec4471fb6a9bfb22d00"
          }
        ],
        "pixelFailures": []
      },
      {
        "name": "boundary-column-half-extent",
        "frame": 0,
        "files": [
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-column-half-extent/frame-0.chrome.png",
            "sha256": "68263645687bba1dbdf1de6e5daddf2bae3f749300fa89881a43f384df7e3673"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-column-half-extent/frame-0.native.png",
            "sha256": "4fd9386d7bba777babc69eb6d6f3efaa909c6dfc3d0dafacd217f7e1987ad562"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-column-half-extent/frame-0.diff.png",
            "sha256": "2f7edd5f243de343698f07c181c430c3f0a9254795f25ec4471fb6a9bfb22d00"
          }
        ],
        "pixelFailures": []
      },
      {
        "name": "boundary-column-accumulated-exact",
        "frame": 0,
        "files": [
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-column-accumulated-exact/frame-0.chrome.png",
            "sha256": "ee14d3f4c257b15cdc9f0e639559f585c9591dbc220ce6f0207f326f5f273bd9"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-column-accumulated-exact/frame-0.native.png",
            "sha256": "7024958f90ee9bffd1e390acd79c8ae818d36108424765a0d58f88ae7badca39"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-column-accumulated-exact/frame-0.diff.png",
            "sha256": "50fed1c5b12337621b953a70eb336af51290f5ffa04b084182808b966dae9105"
          }
        ],
        "pixelFailures": []
      },
      {
        "name": "boundary-column-accumulated-decimal",
        "frame": 0,
        "files": [
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-column-accumulated-decimal/frame-0.chrome.png",
            "sha256": "da2ee9140846cd58445d9e81b957b3906498b05264112f76aeb568d37bcbca30"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-column-accumulated-decimal/frame-0.native.png",
            "sha256": "925bef45dcab5781119036a72fdbff48e1c4861d739a04a119ebaccb30e259d7"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-column-accumulated-decimal/frame-0.diff.png",
            "sha256": "e4bf9f4e2178ffd9262382019edf325096cdb39c0dad42570db795084bca8ce7"
          }
        ],
        "pixelFailures": []
      }
    ]
  },
  {
    "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/visual/sheet-06.png",
    "sha256": "1a39151bb6ca765f7135c6521e9492a15e40407c0a29637a61183fe095ca55ca",
    "members": [
      {
        "name": "boundary-row-negative-saturation",
        "frame": 1,
        "files": [
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-negative-saturation/frame-1.chrome.png",
            "sha256": "b522ff5bd6bb80f2e1465bdb2aa58877c8745f74fef8217ce80c593aaad341a7"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-negative-saturation/frame-1.native.png",
            "sha256": "1cfd703fb83b58b540579d34bd0f896bbda0b433854be6126b2ea321b1a2e899"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-negative-saturation/frame-1.diff.png",
            "sha256": "b646b1fea9d171b42ed832bb58539a75ea6cd016d26f8e485c8720b11f6caa2b"
          }
        ],
        "pixelFailures": []
      },
      {
        "name": "boundary-row-positive-saturation",
        "frame": 1,
        "files": [
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-positive-saturation/frame-1.chrome.png",
            "sha256": "b522ff5bd6bb80f2e1465bdb2aa58877c8745f74fef8217ce80c593aaad341a7"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-positive-saturation/frame-1.native.png",
            "sha256": "1cfd703fb83b58b540579d34bd0f896bbda0b433854be6126b2ea321b1a2e899"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-positive-saturation/frame-1.diff.png",
            "sha256": "b646b1fea9d171b42ed832bb58539a75ea6cd016d26f8e485c8720b11f6caa2b"
          }
        ],
        "pixelFailures": []
      },
      {
        "name": "boundary-row-spanning",
        "frame": 1,
        "files": [
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-spanning/frame-1.chrome.png",
            "sha256": "bcacde139d04ac733d04efe39e03aef2e54f4f42b353250c0e52445e1f46f335"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-spanning/frame-1.native.png",
            "sha256": "7df590055e367e97e186cc40048a571fca58c08e3fd02d90ffe882f850476b74"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-spanning/frame-1.diff.png",
            "sha256": "bb6fed9aa3b4a480f21c4dbe8e0130c57f1e034ebd5c86388705d5be11d17217"
          }
        ],
        "pixelFailures": []
      },
      {
        "name": "boundary-row-negative-spanning",
        "frame": 1,
        "files": [
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-negative-spanning/frame-1.chrome.png",
            "sha256": "bcacde139d04ac733d04efe39e03aef2e54f4f42b353250c0e52445e1f46f335"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-negative-spanning/frame-1.native.png",
            "sha256": "7df590055e367e97e186cc40048a571fca58c08e3fd02d90ffe882f850476b74"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-negative-spanning/frame-1.diff.png",
            "sha256": "bb6fed9aa3b4a480f21c4dbe8e0130c57f1e034ebd5c86388705d5be11d17217"
          }
        ],
        "pixelFailures": []
      }
    ]
  },
  {
    "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/visual/sheet-07.png",
    "sha256": "f6ea66faa7e479c39025a196204718843b07fbfec01eb85f954f0605a8e2bee3",
    "members": [
      {
        "name": "boundary-row-negative-saturation",
        "frame": 2,
        "files": [
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-negative-saturation/frame-2.chrome.png",
            "sha256": "21e163aa59319a6a7e15e172f89e20ee5506da089bf06b75d9f7736f0822c15d"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-negative-saturation/frame-2.native.png",
            "sha256": "505380d8f9c5512f2e9972d6653202f05d9b3d9af0643688a6ca4b4293c2b7b3"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-negative-saturation/frame-2.diff.png",
            "sha256": "7365d3407f93e299b75e62e0b5aad7516aae14a4734ac6d8a438f91b914ae6a4"
          }
        ],
        "pixelFailures": []
      },
      {
        "name": "boundary-row-positive-saturation",
        "frame": 2,
        "files": [
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-positive-saturation/frame-2.chrome.png",
            "sha256": "21e163aa59319a6a7e15e172f89e20ee5506da089bf06b75d9f7736f0822c15d"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-positive-saturation/frame-2.native.png",
            "sha256": "505380d8f9c5512f2e9972d6653202f05d9b3d9af0643688a6ca4b4293c2b7b3"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-positive-saturation/frame-2.diff.png",
            "sha256": "7365d3407f93e299b75e62e0b5aad7516aae14a4734ac6d8a438f91b914ae6a4"
          }
        ],
        "pixelFailures": []
      },
      {
        "name": "boundary-row-spanning",
        "frame": 2,
        "files": [
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-spanning/frame-2.chrome.png",
            "sha256": "8ea28460e7014b47422889862d235a3ac97ff54924588bbdd3511d91842468bd"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-spanning/frame-2.native.png",
            "sha256": "4482878528052263a685beab27bbf5bb9f961383a6940ed026575b04b1e7d0a5"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-spanning/frame-2.diff.png",
            "sha256": "174338cd2c17058c64b5cbd9574a37d49e4d9b83326bc493c858a6e856985e1d"
          }
        ],
        "pixelFailures": []
      },
      {
        "name": "boundary-row-negative-spanning",
        "frame": 2,
        "files": [
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-negative-spanning/frame-2.chrome.png",
            "sha256": "8ea28460e7014b47422889862d235a3ac97ff54924588bbdd3511d91842468bd"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-negative-spanning/frame-2.native.png",
            "sha256": "4482878528052263a685beab27bbf5bb9f961383a6940ed026575b04b1e7d0a5"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-negative-spanning/frame-2.diff.png",
            "sha256": "174338cd2c17058c64b5cbd9574a37d49e4d9b83326bc493c858a6e856985e1d"
          }
        ],
        "pixelFailures": []
      }
    ]
  },
  {
    "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/visual/sheet-08.png",
    "sha256": "7dedee7093b5d3c961363972f22cd969badbc7cc145048649e00124bf4d7d144",
    "members": [
      {
        "name": "boundary-row-zero",
        "frame": 1,
        "files": [
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-zero/frame-1.chrome.png",
            "sha256": "91b3bbe774ea824b19948603e163c80c270c3bc991bd998c573ba3fb99767645"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-zero/frame-1.native.png",
            "sha256": "a1a3bd08d1f580e9f1310198484e7fbd166c3455e4eb4cf88d46c604c7f85ac5"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-zero/frame-1.diff.png",
            "sha256": "b6282331a39035345037d15bc8425b2c82bdd713e27f44edfa373ba44560e6ae"
          }
        ],
        "pixelFailures": []
      },
      {
        "name": "boundary-row-thin-limit",
        "frame": 1,
        "files": [
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-thin-limit/frame-1.chrome.png",
            "sha256": "91b3bbe774ea824b19948603e163c80c270c3bc991bd998c573ba3fb99767645"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-thin-limit/frame-1.native.png",
            "sha256": "a1a3bd08d1f580e9f1310198484e7fbd166c3455e4eb4cf88d46c604c7f85ac5"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-thin-limit/frame-1.diff.png",
            "sha256": "b6282331a39035345037d15bc8425b2c82bdd713e27f44edfa373ba44560e6ae"
          }
        ],
        "pixelFailures": []
      },
      {
        "name": "boundary-row-thin-next-above",
        "frame": 1,
        "files": [
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-thin-next-above/frame-1.chrome.png",
            "sha256": "91b3bbe774ea824b19948603e163c80c270c3bc991bd998c573ba3fb99767645"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-thin-next-above/frame-1.native.png",
            "sha256": "a1a3bd08d1f580e9f1310198484e7fbd166c3455e4eb4cf88d46c604c7f85ac5"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-thin-next-above/frame-1.diff.png",
            "sha256": "b6282331a39035345037d15bc8425b2c82bdd713e27f44edfa373ba44560e6ae"
          }
        ],
        "pixelFailures": []
      },
      {
        "name": "boundary-row-thin-next-below",
        "frame": 1,
        "files": [
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-thin-next-below/frame-1.chrome.png",
            "sha256": "91b3bbe774ea824b19948603e163c80c270c3bc991bd998c573ba3fb99767645"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-thin-next-below/frame-1.native.png",
            "sha256": "a1a3bd08d1f580e9f1310198484e7fbd166c3455e4eb4cf88d46c604c7f85ac5"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-thin-next-below/frame-1.diff.png",
            "sha256": "b6282331a39035345037d15bc8425b2c82bdd713e27f44edfa373ba44560e6ae"
          }
        ],
        "pixelFailures": []
      }
    ]
  },
  {
    "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/visual/sheet-09.png",
    "sha256": "732c85814f310a21d8c7e1c4653a806fa6fe0a1a751f5be56fd99d36eb8d7ef2",
    "members": [
      {
        "name": "boundary-row-thin-above",
        "frame": 1,
        "files": [
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-thin-above/frame-1.chrome.png",
            "sha256": "3e7b648cf06f166f289f46cb6a3e162d925299be434aba809946f2c32af8c885"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-thin-above/frame-1.native.png",
            "sha256": "3e6c385b7ccc914a2646cc49e2dd11b001a10bdd45b07068262fb1edf4ca59e9"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-thin-above/frame-1.diff.png",
            "sha256": "1c1fd6777084a025a98c2b087ed9aa21035c6cfad9904cabe6616e67ad5fecca"
          }
        ],
        "pixelFailures": []
      },
      {
        "name": "boundary-row-half-extent",
        "frame": 1,
        "files": [
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-half-extent/frame-1.chrome.png",
            "sha256": "3e7b648cf06f166f289f46cb6a3e162d925299be434aba809946f2c32af8c885"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-half-extent/frame-1.native.png",
            "sha256": "3e6c385b7ccc914a2646cc49e2dd11b001a10bdd45b07068262fb1edf4ca59e9"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-half-extent/frame-1.diff.png",
            "sha256": "1c1fd6777084a025a98c2b087ed9aa21035c6cfad9904cabe6616e67ad5fecca"
          }
        ],
        "pixelFailures": []
      }
    ]
  },
  {
    "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/visual/sheet-10.png",
    "sha256": "25f51f1bb106658f02526b1a57019159e30e3a272af646b52a4cc7b80f968fa9",
    "members": [
      {
        "name": "boundary-row-zero",
        "frame": 2,
        "files": [
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-zero/frame-2.chrome.png",
            "sha256": "0d5e9a2bdffd868e9c99391832d0d71bcc39665a1e355fe2d15b00aa555fa043"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-zero/frame-2.native.png",
            "sha256": "e21b9c6aef25175deaf811267757c8cf220440a0921c39372657dd85a6a859f1"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-zero/frame-2.diff.png",
            "sha256": "7afffa6b5b4fceb882315514b3c26138b2600dc87ebf10f4bf95a5db08fc4c63"
          }
        ],
        "pixelFailures": []
      },
      {
        "name": "boundary-row-thin-limit",
        "frame": 2,
        "files": [
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-thin-limit/frame-2.chrome.png",
            "sha256": "0d5e9a2bdffd868e9c99391832d0d71bcc39665a1e355fe2d15b00aa555fa043"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-thin-limit/frame-2.native.png",
            "sha256": "e21b9c6aef25175deaf811267757c8cf220440a0921c39372657dd85a6a859f1"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-thin-limit/frame-2.diff.png",
            "sha256": "7afffa6b5b4fceb882315514b3c26138b2600dc87ebf10f4bf95a5db08fc4c63"
          }
        ],
        "pixelFailures": []
      },
      {
        "name": "boundary-row-thin-next-above",
        "frame": 2,
        "files": [
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-thin-next-above/frame-2.chrome.png",
            "sha256": "0d5e9a2bdffd868e9c99391832d0d71bcc39665a1e355fe2d15b00aa555fa043"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-thin-next-above/frame-2.native.png",
            "sha256": "e21b9c6aef25175deaf811267757c8cf220440a0921c39372657dd85a6a859f1"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-thin-next-above/frame-2.diff.png",
            "sha256": "7afffa6b5b4fceb882315514b3c26138b2600dc87ebf10f4bf95a5db08fc4c63"
          }
        ],
        "pixelFailures": []
      },
      {
        "name": "boundary-row-thin-next-below",
        "frame": 2,
        "files": [
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-thin-next-below/frame-2.chrome.png",
            "sha256": "0d5e9a2bdffd868e9c99391832d0d71bcc39665a1e355fe2d15b00aa555fa043"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-thin-next-below/frame-2.native.png",
            "sha256": "e21b9c6aef25175deaf811267757c8cf220440a0921c39372657dd85a6a859f1"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-thin-next-below/frame-2.diff.png",
            "sha256": "7afffa6b5b4fceb882315514b3c26138b2600dc87ebf10f4bf95a5db08fc4c63"
          }
        ],
        "pixelFailures": []
      }
    ]
  },
  {
    "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/visual/sheet-11.png",
    "sha256": "224f5d7441ccaf27ef25ba08eaf674e0c9c42ffcf2c8b421230c886006a9332e",
    "members": [
      {
        "name": "boundary-row-thin-above",
        "frame": 2,
        "files": [
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-thin-above/frame-2.chrome.png",
            "sha256": "3b514405da2d56b101d4b7c79624a55d899a69c8d8cfe455b72775c665cc6e65"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-thin-above/frame-2.native.png",
            "sha256": "1cce03055e3e398426216fb19786be832d9e42fb1bdcf684f821c7ed2cc313ce"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-thin-above/frame-2.diff.png",
            "sha256": "9ee08b5d2e36e26e449ad854c1828fd3bb3d6a7decc2cc71bc98213bc6028bce"
          }
        ],
        "pixelFailures": []
      },
      {
        "name": "boundary-row-half-extent",
        "frame": 2,
        "files": [
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-half-extent/frame-2.chrome.png",
            "sha256": "3b514405da2d56b101d4b7c79624a55d899a69c8d8cfe455b72775c665cc6e65"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-half-extent/frame-2.native.png",
            "sha256": "1cce03055e3e398426216fb19786be832d9e42fb1bdcf684f821c7ed2cc313ce"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-half-extent/frame-2.diff.png",
            "sha256": "9ee08b5d2e36e26e449ad854c1828fd3bb3d6a7decc2cc71bc98213bc6028bce"
          }
        ],
        "pixelFailures": []
      },
      {
        "name": "boundary-row-accumulated-exact",
        "frame": 2,
        "files": [
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-accumulated-exact/frame-2.chrome.png",
            "sha256": "1063c67f709648c4c090fc945705554249ec82c93cb182268f28365a406dc6fa"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-accumulated-exact/frame-2.native.png",
            "sha256": "8d57775d411311a60794a282f55985e7530a1dd76f12732e1c143af2f8bc571f"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-accumulated-exact/frame-2.diff.png",
            "sha256": "2e48829aad4e7b5170c06e50c298ebe01e6ff5d66bf6e5cda358fea3caadcf01"
          }
        ],
        "pixelFailures": []
      },
      {
        "name": "boundary-row-accumulated-decimal",
        "frame": 2,
        "files": [
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-accumulated-decimal/frame-2.chrome.png",
            "sha256": "1063c67f709648c4c090fc945705554249ec82c93cb182268f28365a406dc6fa"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-accumulated-decimal/frame-2.native.png",
            "sha256": "8d57775d411311a60794a282f55985e7530a1dd76f12732e1c143af2f8bc571f"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-accumulated-decimal/frame-2.diff.png",
            "sha256": "2e48829aad4e7b5170c06e50c298ebe01e6ff5d66bf6e5cda358fea3caadcf01"
          }
        ],
        "pixelFailures": []
      }
    ]
  },
  {
    "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/visual/sheet-12.png",
    "sha256": "31e57e784f773680bfc01e204e9339222e08ad716e9a41f367bb710eeb0b0ad7",
    "members": [
      {
        "name": "boundary-row-accumulated-exact",
        "frame": 1,
        "files": [
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-accumulated-exact/frame-1.chrome.png",
            "sha256": "39d5cb9eb9b1eaed68faf79abbf4140be8512a9534dbfb75f67e62318207b506"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-accumulated-exact/frame-1.native.png",
            "sha256": "564c5cdc76c259a7d2848a14bd6a3bb94086cffb8b0191b034c775f5528eb047"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-accumulated-exact/frame-1.diff.png",
            "sha256": "01f08017c9a27c180e411fd98cd4a4f1452e1c668313b64fb3111e6ad23819ed"
          }
        ],
        "pixelFailures": []
      },
      {
        "name": "boundary-row-accumulated-decimal",
        "frame": 1,
        "files": [
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-accumulated-decimal/frame-1.chrome.png",
            "sha256": "c46135d693339f6ec9a32593ef10b14a93f2461f050f8e9841d247f8ea8e629e"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-accumulated-decimal/frame-1.native.png",
            "sha256": "fdc3cc477339a85f76ca0797e1e09c8e2f8aebf1ce54d4045e5af321525f1b19"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-normalized-boundary-r1/boundary-row-accumulated-decimal/frame-1.diff.png",
            "sha256": "6886da12623233b24a488d46511983463007eedbd44611ff776a92567c1e93c1"
          }
        ],
        "pixelFailures": []
      }
    ]
  }
]
```
