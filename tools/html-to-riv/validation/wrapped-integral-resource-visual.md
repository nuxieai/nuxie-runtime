# Integral resource and mixed paint visual review

Root directly inspected all six full-resolution Chrome/native/diff sheets using view_image: 18 distinct pairs and 30 exact repeat transfers. The 1/3/8/34-owner scenes retain matching tile positions, row breaks, colors and blank background. Mixed row and column scenes retain the fractional middle paint and wider final paint; no missing replica, edge band or incorrect draw ordering is visible. Faint filled-region diff context remains, so this does not claim identical RGBA. These viewports do not wrap the three-owner mixed scenes; other wrapping coverage is separate.

Inspected image identities:

```json
[
  {
    "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-integral-resource-r1/visual/sheet-00.png",
    "sha256": "b8ea01e7d0b783919cde2ac59066789a4d3702cf1772b3b6013a62fc171e9952",
    "members": [
      {
        "name": "integral-resource-1",
        "frame": 0,
        "files": [
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-integral-resource-r1/integral-resource-1/frame-0.chrome.png",
            "sha256": "c4ca99f0ba05e214380329c5d845176dd049cbbdf29aba9a7eb71825146bcf3c"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-integral-resource-r1/integral-resource-1/frame-0.native.png",
            "sha256": "c3b0746788e74266666b3fd690bccdca6baba223bc860284ca61d9a9b79dd3a7"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-integral-resource-r1/integral-resource-1/frame-0.diff.png",
            "sha256": "aabe0886ca6702136782e26b8af858380c5c63934b67fab2875e8e936bfd910c"
          }
        ],
        "pixelFailures": []
      },
      {
        "name": "integral-resource-3",
        "frame": 0,
        "files": [
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-integral-resource-r1/integral-resource-3/frame-0.chrome.png",
            "sha256": "2f8660182a7736ab5dd13ae4c6aa0f1924c2363b122bf00e3c48a73d25a3c78a"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-integral-resource-r1/integral-resource-3/frame-0.native.png",
            "sha256": "55695ddedc7fb16106fcf8fe23f015e961edeb59561e71ade76096dd1b525378"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-integral-resource-r1/integral-resource-3/frame-0.diff.png",
            "sha256": "857975547ca7010e4c154c6b2e64c2653416aee6a6e2afb389a17a945bec88a0"
          }
        ],
        "pixelFailures": []
      },
      {
        "name": "integral-resource-8",
        "frame": 0,
        "files": [
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-integral-resource-r1/integral-resource-8/frame-0.chrome.png",
            "sha256": "51080b6ad2f8fb0f3aef80e00adafa1aa82081c9d690a71e079f58eebc09c4d5"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-integral-resource-r1/integral-resource-8/frame-0.native.png",
            "sha256": "b7bcd5a81ba40f2f85c8a8f7902ce95548eaf66fa63674cf8e79d773255a9da0"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-integral-resource-r1/integral-resource-8/frame-0.diff.png",
            "sha256": "2b92a9c2f0fd5ace52dba66cec638c8a9289588e29630ad4ebfb9e8124796d2c"
          }
        ],
        "pixelFailures": []
      },
      {
        "name": "integral-resource-34",
        "frame": 0,
        "files": [
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-integral-resource-r1/integral-resource-34/frame-0.chrome.png",
            "sha256": "9d0c5e9b82bd96e28e8413e5a8a2b5463688dba1c1dd93dceee802d747347c09"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-integral-resource-r1/integral-resource-34/frame-0.native.png",
            "sha256": "42d6fa6600b24380fbe5dbde4a68b1eda6f6d70abceea9f973502525375d0611"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-integral-resource-r1/integral-resource-34/frame-0.diff.png",
            "sha256": "b6f13783226efe418b053122c04b16b382bcbbff24ce280464e2ec7755a8195e"
          }
        ],
        "pixelFailures": []
      }
    ]
  },
  {
    "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-integral-resource-r1/visual/sheet-01.png",
    "sha256": "08830c48f59ada0b8d6fb0e26ba86775413a84286484a682010b8e65eb67fe0e",
    "members": [
      {
        "name": "integral-mixed-row",
        "frame": 0,
        "files": [
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-integral-resource-r1/integral-mixed-row/frame-0.chrome.png",
            "sha256": "5fcd8bbefef3161b80066deb1f5151901733fa14b83840739a05ff398fa1d9dc"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-integral-resource-r1/integral-mixed-row/frame-0.native.png",
            "sha256": "f3cd7c416a8b846b53337c09fa33bc6bfaded58315bd301b9bf5ff4dff50f721"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-integral-resource-r1/integral-mixed-row/frame-0.diff.png",
            "sha256": "c71d0749c48cbce20ddf4814f8a8949cf047da2c4561104862b437414cdf9775"
          }
        ],
        "pixelFailures": []
      },
      {
        "name": "integral-mixed-column",
        "frame": 0,
        "files": [
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-integral-resource-r1/integral-mixed-column/frame-0.chrome.png",
            "sha256": "cf2cfa92edbac465aff6d07337a0ed04bd94ecc7e31802fbbcd3cbe9edb7f582"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-integral-resource-r1/integral-mixed-column/frame-0.native.png",
            "sha256": "761464e499153b82a5335dbc3171e959289737b12050ad64e596707032ac553d"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-integral-resource-r1/integral-mixed-column/frame-0.diff.png",
            "sha256": "c0402a91c6b8927182699a4489c30a426f1b08608933be0987d23e997d542726"
          }
        ],
        "pixelFailures": []
      }
    ]
  },
  {
    "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-integral-resource-r1/visual/sheet-02.png",
    "sha256": "7e5ca65d4eb73591bec4f51c3c23523a4e6e81139456cc3d92320ea5bd558a82",
    "members": [
      {
        "name": "integral-resource-1",
        "frame": 1,
        "files": [
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-integral-resource-r1/integral-resource-1/frame-1.chrome.png",
            "sha256": "e1980b2a9e64860a93a7adb2cf15dc2529635fb1954cdc82bbdacfcbdfc74470"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-integral-resource-r1/integral-resource-1/frame-1.native.png",
            "sha256": "ec7c4564d1a5ce54ee2dc0a93fb2b0ae09d1c60bcb6550abef74bc353b8e9a79"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-integral-resource-r1/integral-resource-1/frame-1.diff.png",
            "sha256": "94c4bf3af1d812cfc698bb3a6aac9d2c5189b0ceb3f66951056c3f2d96f954d7"
          }
        ],
        "pixelFailures": []
      },
      {
        "name": "integral-resource-3",
        "frame": 1,
        "files": [
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-integral-resource-r1/integral-resource-3/frame-1.chrome.png",
            "sha256": "8b3d552d87ae10b8fef87225f930440945e0547c555d76ef7a67bc66389be0b1"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-integral-resource-r1/integral-resource-3/frame-1.native.png",
            "sha256": "f086c05c933559b2c9c15201b8490edbe34bcff63fea4dd5bdade5fd7d938e16"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-integral-resource-r1/integral-resource-3/frame-1.diff.png",
            "sha256": "788c41776b1038645b5b706f6e427a930f6cfc44354b4c0ac43c18fc8e50282b"
          }
        ],
        "pixelFailures": []
      },
      {
        "name": "integral-resource-8",
        "frame": 1,
        "files": [
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-integral-resource-r1/integral-resource-8/frame-1.chrome.png",
            "sha256": "d9522526d93637cc4b259921905c90cf23a453275f2a73c5c1f62f5e3867a6a5"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-integral-resource-r1/integral-resource-8/frame-1.native.png",
            "sha256": "afac2e5af9fbc4bab578d60450fcd7b129da034cfbb86f87ca1ed5bbc1cb1a23"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-integral-resource-r1/integral-resource-8/frame-1.diff.png",
            "sha256": "936b4a4f26b276cf62237c8ed94c585173e8c1340b35bcef8851de145714f0e9"
          }
        ],
        "pixelFailures": []
      },
      {
        "name": "integral-resource-34",
        "frame": 1,
        "files": [
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-integral-resource-r1/integral-resource-34/frame-1.chrome.png",
            "sha256": "ef7a2b2e1b7521c6e6c309deb6f8077a04d32928bc6b4f5dc3dee90ce0d020af"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-integral-resource-r1/integral-resource-34/frame-1.native.png",
            "sha256": "de4a202e8c3adf06807ac77c6864e04e6736983b3cce9a1733a4fa76a6b4b61c"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-integral-resource-r1/integral-resource-34/frame-1.diff.png",
            "sha256": "04129ccf79296fa70c5de0356ae3e414bb36e37419e992125534755828ec9a15"
          }
        ],
        "pixelFailures": []
      }
    ]
  },
  {
    "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-integral-resource-r1/visual/sheet-03.png",
    "sha256": "fa02a04ad34ffb3abc2de5f76fb758ca82dc1115d31afdcd3436e127a9d28c17",
    "members": [
      {
        "name": "integral-mixed-row",
        "frame": 1,
        "files": [
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-integral-resource-r1/integral-mixed-row/frame-1.chrome.png",
            "sha256": "ea4ea9de7b85b96080debe4fb57ba1ecc4007a2556f0630dc6c110138ff3643a"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-integral-resource-r1/integral-mixed-row/frame-1.native.png",
            "sha256": "99ecfd9c22881ca6d58f76c25f1a7702920006c43ba4801190306a7cb075392c"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-integral-resource-r1/integral-mixed-row/frame-1.diff.png",
            "sha256": "d6d685d9f35920a5ed2edc56b22424087c37df20bc77560e9532e6d1370e3575"
          }
        ],
        "pixelFailures": []
      },
      {
        "name": "integral-mixed-column",
        "frame": 1,
        "files": [
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-integral-resource-r1/integral-mixed-column/frame-1.chrome.png",
            "sha256": "d9a134e0a0d72a32d40c13ded3d25b229156e6e0291c657654921c9dac599623"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-integral-resource-r1/integral-mixed-column/frame-1.native.png",
            "sha256": "65e88ef5e0974dad4e3a2f3f91fef2bd4c93e7ff629a66fefdb46576885cf0ca"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-integral-resource-r1/integral-mixed-column/frame-1.diff.png",
            "sha256": "8754a4d9084f6be05f3f782cee210f859ca49b7e756af351be82b7035d2a40af"
          }
        ],
        "pixelFailures": []
      }
    ]
  },
  {
    "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-integral-resource-r1/visual/sheet-04.png",
    "sha256": "87bd0930a663742e18a1165d970bcccbea53b52e6183a50718bf7101da1be089",
    "members": [
      {
        "name": "integral-resource-1",
        "frame": 2,
        "files": [
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-integral-resource-r1/integral-resource-1/frame-2.chrome.png",
            "sha256": "801671af125cd6dbeaf672f9495ac636a42807a85c4e01a294aa20832d294d73"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-integral-resource-r1/integral-resource-1/frame-2.native.png",
            "sha256": "071a886c47eeea0ab8a69448205de9fc78760f3db3498fa7e91f1c27871b93db"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-integral-resource-r1/integral-resource-1/frame-2.diff.png",
            "sha256": "82af9b77925f88f838a36d16b4f82290e5ab364abf8ad2c0316cae8fee41d137"
          }
        ],
        "pixelFailures": []
      },
      {
        "name": "integral-resource-3",
        "frame": 2,
        "files": [
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-integral-resource-r1/integral-resource-3/frame-2.chrome.png",
            "sha256": "f4ac2feaacba430fdfd25ec328466d3d524b96b0a8b1bce1905f963915ddea59"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-integral-resource-r1/integral-resource-3/frame-2.native.png",
            "sha256": "1b25f2093bfe2a8f3619a311d167f38bdc0d2ef61674a5bb267ee17a194231ae"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-integral-resource-r1/integral-resource-3/frame-2.diff.png",
            "sha256": "471ec1219b434862d49c0e03dc091b8f6b9e77cd74a9b333874e032bd3ddd089"
          }
        ],
        "pixelFailures": []
      },
      {
        "name": "integral-resource-8",
        "frame": 2,
        "files": [
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-integral-resource-r1/integral-resource-8/frame-2.chrome.png",
            "sha256": "360fcec14081b885d3405625a19d131c06c67db07e6866a856553c4ee222d5f7"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-integral-resource-r1/integral-resource-8/frame-2.native.png",
            "sha256": "2913b46241504261636cc202fefa69de3fd8926bc1982ed58516d99f359cecca"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-integral-resource-r1/integral-resource-8/frame-2.diff.png",
            "sha256": "288063ff1dbe410f224cb22564732e4f78598c1209c0053d169698f0744164a9"
          }
        ],
        "pixelFailures": []
      },
      {
        "name": "integral-resource-34",
        "frame": 2,
        "files": [
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-integral-resource-r1/integral-resource-34/frame-2.chrome.png",
            "sha256": "8643cd326904175ea3114b4083e5bb8e5ff709d6b751481a56a1a5998ef1bbc2"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-integral-resource-r1/integral-resource-34/frame-2.native.png",
            "sha256": "1eebef64c093cf1316edc60424c059f1036e5e45729f880b65fa128d8c795cbc"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-integral-resource-r1/integral-resource-34/frame-2.diff.png",
            "sha256": "de51779dd39de31e9d196c8a65f9eb0c08fdf2ad911d607d6e66d37b98661f60"
          }
        ],
        "pixelFailures": []
      }
    ]
  },
  {
    "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-integral-resource-r1/visual/sheet-05.png",
    "sha256": "e643fb88b294b97d18c1d9daa422c26ca711c374f83fe18ac14113d796f5ce22",
    "members": [
      {
        "name": "integral-mixed-row",
        "frame": 2,
        "files": [
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-integral-resource-r1/integral-mixed-row/frame-2.chrome.png",
            "sha256": "38cb29aa8444fbce0ffbe8575d6e76e72a129c7b0b9aea3d6966005d9cd2f955"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-integral-resource-r1/integral-mixed-row/frame-2.native.png",
            "sha256": "452f74de13ba74175bbd3ad2740ffe9aa0483127f671c31b77b160dabeefbd86"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-integral-resource-r1/integral-mixed-row/frame-2.diff.png",
            "sha256": "4e080b73b8f5055af40ed04544360e1af7f976bcc87e1e22f057ccb2931c9007"
          }
        ],
        "pixelFailures": []
      },
      {
        "name": "integral-mixed-column",
        "frame": 2,
        "files": [
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-integral-resource-r1/integral-mixed-column/frame-2.chrome.png",
            "sha256": "0d120459baedf392c25e8c6b00c58b4fdb788370c59a61f50340eb50c7c872a6"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-integral-resource-r1/integral-mixed-column/frame-2.native.png",
            "sha256": "68c3df6ff7b0b5c4b665a8ccb850dfa1be6cdc2d6dde60eb6036bad6dfa4810c"
          },
          {
            "path": "/Users/levi/.codex/worktrees/html-css-immutable/tools/html-to-riv/output/playwright/wrapped-integral-resource-r1/integral-mixed-column/frame-2.diff.png",
            "sha256": "6e969aa5d1096b5a44a6ccafe07107a2d9ce0da9d7d7c5c4e49b429e581d2d03"
          }
        ],
        "pixelFailures": []
      }
    ]
  }
]
```
