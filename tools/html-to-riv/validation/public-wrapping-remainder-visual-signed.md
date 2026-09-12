# Direct visual review: public wrapping remainder, sheets 16–23

Viewed all eight actual PNG sheets at original image detail (32 Chrome/native pairs, frame 2 at 300×80). Comparisons were made within each labeled pair using the image origins, not between vertically offset sheet rows. No visible mismatch was identified in these reviewed pairs. This is direct image inspection, not a claim of exact geometric equality.

The campaign reports 256 passing pixel gates, while its separate geometry observations retain 552 differences among 7,040 values, up to approximately 0.007815px. Those subpixel differences remain evidence and are not negated by this visual review. The supplied diff panels retain a faint context image; they were not treated as blank raw-difference proofs. Thin bars, centered joins, reverse ordering and artboard crops were inspected in the actual Chrome/native images.

Coverage SHA-256: `22c2e6248e400b22b74fe6e0665250ddbcaa38f2272d357d5cebf27250bb3aa4`. The coverage receipt binds campaign receipt SHA-256 `d6a9bebb1b07fe35ba1af7d6a9975d4ce111122f5ca3096dab72996083bc75e3`. All listed sheet and member file hashes below were recomputed and matched the coverage manifest; no rendering or source changes were performed.

## Sheet 16

SHA-256: `6060390caf829a08d0f78022bed83e0d29f0705e79aad426fbe7a5a73acca9f7`.

The yellow/salmon row bars and three column bars retain matching widths, heights and gaps within each Chrome/native pair. Wrap-reverse reverses the color order as expected, with no additional crop or stray pixels visible.

- `public-remainder-k2-r1-row-wrap-single`, frame 2.
  - `frame-2.chrome.png`: `463f8aa306c488590fc4fdc254b3e157716e3b21df2e4d83e38cea737c3a32b7`
  - `frame-2.native.png`: `4088b7211829f8d4b748c9abeb74f5a6ab6b11946047d2e566643526baa26bf0`
  - `frame-2.diff.png`: `2c2f876acd33f8c7f5196f70513db1185a4d02da7665a681ba636e826a611cdd`
- `public-remainder-k2-r1-row-wrap-reverse-single`, frame 2.
  - `frame-2.chrome.png`: `1d7a7535992c6f08994fbaedb77a04bd787596f2189d4e504ece6f7c9bd3a566`
  - `frame-2.native.png`: `ed6e60d18494941237ec91178bb12029263a5c0b4b9000f1e525955825acdbd3`
  - `frame-2.diff.png`: `e9f72b14f35ec30b86c232daeb8abd81184f23a2c6f9dad36430cb42f81d48e3`
- `public-remainder-k3-r1-column-wrap-single`, frame 2.
  - `frame-2.chrome.png`: `89e69849fb90beb90233ab3625445d6fbeda212477f50717e4d8b27dfbf199b8`
  - `frame-2.native.png`: `6363a42a09005c4efd7a40a24aaf9fe6cdb815e4e9625ce2e32a9b73c0261422`
  - `frame-2.diff.png`: `2debb4bb6a2b6c2ec32d9ec980de5356188d5f9357dfd444ee1e163ba80cca5e`
- `public-remainder-k3-r1-column-wrap-reverse-single`, frame 2.
  - `frame-2.chrome.png`: `7e7c15ec97c58af9d512c7170021625967ab988becb14ce53da8735d1b165663`
  - `frame-2.native.png`: `3bccf65c4080599bc07c18788520c8e6c65b48ddd36419d62f13603c9441f786`
  - `frame-2.diff.png`: `e1f7d4087a9f070a458a36aa199b14ef171e7547e4c75bbd485dd23070194b6f`

## Sheet 17

SHA-256: `3545ab467fa31336929f768e8b90b8a1a888d36412d8fa207ae5673409b2a9ad`.

Reverse-main row bars have the same horizontal inset within each pair. The four column bars preserve the blue/green shared edge and the yellow/salmon gaps in both wrap directions.

- `public-remainder-k3-r2-row-reverse-wrap-single`, frame 2.
  - `frame-2.chrome.png`: `07a39c2294c0eeaf145488f028fcb8011657672b1c4875d1b36bd40f9eb4cb25`
  - `frame-2.native.png`: `3d3d797414e4cb32db801e4642f5dc6a316099b22323de04a5643d77e191a4c9`
  - `frame-2.diff.png`: `1437ea37606c922e214466dc595f8f185606aa72e2c3f4bbabc313cbb941991f`
- `public-remainder-k3-r2-row-reverse-wrap-reverse-single`, frame 2.
  - `frame-2.chrome.png`: `61ff118b183956441e9b36d6cf90f9d9ea18dc7279dac8e81cad163be4c422b9`
  - `frame-2.native.png`: `61d0ec0b1662609e07629c2ad213a75c8adf8ceb44e9a8d60e62d69f9789741f`
  - `frame-2.diff.png`: `46984f65077e6142fbfc5b2a548520939425aef3c0eca9017a06c41e1a48c91c`
- `public-remainder-k4-r1-column-reverse-wrap-single`, frame 2.
  - `frame-2.chrome.png`: `d0ac2988730e12cf1b6c9399ea4319f90ad54c43b9e0b79067b402ed42f1704d`
  - `frame-2.native.png`: `1688d048c32bfd9023f9b28284910e4149ce1fe72606c59c00e1da1d24aae41f`
  - `frame-2.diff.png`: `a51421f95f24b40d59282f303c1ce609058e5885f180d17b7416c8e93d0d58a4`
- `public-remainder-k4-r1-column-reverse-wrap-reverse-single`, frame 2.
  - `frame-2.chrome.png`: `e364062e793f5960d909dff8a7150eaf3142949802f143809350dad42b5b1590`
  - `frame-2.native.png`: `1bbe36e49e011c078117e7a8ae498c571a65bf4b699dcdf8bb67950d3584aabc`
  - `frame-2.diff.png`: `3f29cfe7e29c4c3b51c790245a0cbebdacb0492fcfe2a26cb282067d6cdc1b4d`

## Sheet 18

SHA-256: `62aa04b6fdf8257c324648da931f52ec74d2d92cf851e51d475bbb19c09c2b1b`.

Four-line row and column examples match in color order and edge placement. The lower row-wrap view cuts the final bar at the same artboard edge; reverse-wrap spacing and the slim yellow bar agree.

- `public-remainder-k4-r2-row-wrap-single`, frame 2.
  - `frame-2.chrome.png`: `b7e096db4f86fed68e21ecc7300395dba1053780a0ec9cc4849f42be0d1420f3`
  - `frame-2.native.png`: `741e2284f1465ebbb96bfc643e249dcdc3740bbcb073d67d8bd7b89b10b8fef0`
  - `frame-2.diff.png`: `aa429289af4a0274d0ab9c7bd78a6d2ecbe52596e52a620c62411401142adc40`
- `public-remainder-k4-r2-row-wrap-reverse-single`, frame 2.
  - `frame-2.chrome.png`: `cc6f00b634ce67067c1b2b254e111a2fd16226351abe9e35ed9b80630244d72e`
  - `frame-2.native.png`: `522a4a95f5165acb2cb885ff1622ca800ced5756eaa19fce3d58e9b1c411c485`
  - `frame-2.diff.png`: `636a2e5577d2badcfa404a924d14de228eb8c5cd3d2c6b7f6aebadcca7431881`
- `public-remainder-k4-r3-column-wrap-single`, frame 2.
  - `frame-2.chrome.png`: `77682c04956dee4254e3f70b90a9362e98b6a2412069e8a806e04b6c64949801`
  - `frame-2.native.png`: `66ac47b54da8e0ae761da82f688c9b502c33755ff6984c55c622c9f53f2c5348`
  - `frame-2.diff.png`: `682de798dd6eb7e7afc6ab59b1764784e43c85fb05aab45bb880fdffc77cbe0f`
- `public-remainder-k4-r3-column-wrap-reverse-single`, frame 2.
  - `frame-2.chrome.png`: `50416db488fda1bc225145563182ec85114dea73ec8ef94e8517f7d33580b79a`
  - `frame-2.native.png`: `4f4d4835f496a87882120995ba4eef70d67433cb2466f49ba3f449758df77a33`
  - `frame-2.diff.png`: `23604362aeaf202c0618f423ff2b27b65e55e446b03ff036357190175f833f44`

## Sheet 19

SHA-256: `3b9aa6d127504fb15b5ab4ea7530a72612cfa07e38bd005bb9dcf721bb93f449`.

Seven-line row cases are clipped by the 80px artboard height. The reverse-wrap view retains the same thin purple strip at the bottom in both images. Seven column colors, narrow white gaps, and reverse order agree.

- `public-remainder-k7-r1-row-reverse-wrap-single`, frame 2.
  - `frame-2.chrome.png`: `90532d0249d8e0a533f92a5ef9d8e6fbf5fadceb116ecb0a073842cd6e3db74a`
  - `frame-2.native.png`: `94135054744476bd22f9183736c7fe9fb34f9ea4677d6ee832f49b70d23e24f0`
  - `frame-2.diff.png`: `108b7589959f8414a39a4513185fb4b1b3543c324a6650b34aceafcbdcd9fcf1`
- `public-remainder-k7-r1-row-reverse-wrap-reverse-single`, frame 2.
  - `frame-2.chrome.png`: `be5f3d1fb371afcfcb8da6b359c073b3c0cc3c2362b65903dec11e57300a505c`
  - `frame-2.native.png`: `c05fd3bdb6a31d92cc5e8f4bfe1e9ea0b74f9808dd78dd072f44d966edf0d8ee`
  - `frame-2.diff.png`: `6161cd1f70e83a02081fcdbb2ac2b32ff7be9d2250ddabd7a79f5b014c178049`
- `public-remainder-k7-r2-column-reverse-wrap-single`, frame 2.
  - `frame-2.chrome.png`: `2f00b999b02e58ea0b8061c88f99209e8fe5152679905349b3d386737429c4c3`
  - `frame-2.native.png`: `3a7f49300d2156eaf6d0dee44c393f326fff26aa3f4b514c49d9329dac8569e6`
  - `frame-2.diff.png`: `1838416fc5cf6e967d74090882f634973476638746227edd838aca25035539cd`
- `public-remainder-k7-r2-column-reverse-wrap-reverse-single`, frame 2.
  - `frame-2.chrome.png`: `4a495ee03c2dfa432b93a1f84293ea95ef1a99a495d47cd166e9bf24aee43f32`
  - `frame-2.native.png`: `1bfb8fba2d1bec29363edd6c5ff5209cf98f6ec0c8b8964cfc998a0beb9e9458`
  - `frame-2.diff.png`: `615cb19a86ac76b171bea7f855c2b8a9a6e6259920d600c713582eb4dc986a0e`

## Sheet 20

SHA-256: `e7199150d91e6bec77ba706fa99c15f6bbb09d5fd6a4cf73230d6f628579d5af`.

The seven-line row crop and reverse-wrap thin purple strip agree within both pairs. Column remainder-four views preserve all seven widths, gaps and reversed order without an extra edge pixel apparent at this scale.

- `public-remainder-k7-r3-row-wrap-single`, frame 2.
  - `frame-2.chrome.png`: `b7e096db4f86fed68e21ecc7300395dba1053780a0ec9cc4849f42be0d1420f3`
  - `frame-2.native.png`: `741e2284f1465ebbb96bfc643e249dcdc3740bbcb073d67d8bd7b89b10b8fef0`
  - `frame-2.diff.png`: `aa429289af4a0274d0ab9c7bd78a6d2ecbe52596e52a620c62411401142adc40`
- `public-remainder-k7-r3-row-wrap-reverse-single`, frame 2.
  - `frame-2.chrome.png`: `0e5f3ebf629a0fe44d17693e7c3b7c3e0527378c1472ae6f5ba9cfe5ad0d6756`
  - `frame-2.native.png`: `99f81d937efea7dac8ec85d96b2dea08a7d87ce1f316a1c453a5b82804d3057e`
  - `frame-2.diff.png`: `3fd7dd8f2c25fdf3fe6022b248d982a2814235dc3d83fd0a048914ba50d730e8`
- `public-remainder-k7-r4-column-wrap-single`, frame 2.
  - `frame-2.chrome.png`: `35c8e5e53cea61a1c942fca36f008da5846181c897899579cb95510ff6dd5a4c`
  - `frame-2.native.png`: `0d19f8388d27a3595d6ec8f4581192be1e17d513d24b6a9c84cfe289449f9486`
  - `frame-2.diff.png`: `28036f54869322cd04c22c039f2a7e26406ce7931459ca63157057a5266f4165`
- `public-remainder-k7-r4-column-wrap-reverse-single`, frame 2.
  - `frame-2.chrome.png`: `cdefd85afbb61c079910b7817f48cf2acd127ccfb9e16ba4e11bd54164bef3d1`
  - `frame-2.native.png`: `2cd1002528d6bf986ae3fa352e8f6a431f6d5edacbbabe304ad126cd01a8df3d`
  - `frame-2.diff.png`: `4e4f9c78579585204c848c392243bb3c2d8c120b144aefe4b65bcff8b457a8cd`

## Sheet 21

SHA-256: `cdda51c7cafba6d4baaf3a5375652c063d011a78952930bdefdac65a3e60c2e0`.

Reverse-main row insets and bottom-edge crops agree. The seven-column remainder-six views retain matching narrow gaps and color ordering under wrap reversal.

- `public-remainder-k7-r5-row-reverse-wrap-single`, frame 2.
  - `frame-2.chrome.png`: `90532d0249d8e0a533f92a5ef9d8e6fbf5fadceb116ecb0a073842cd6e3db74a`
  - `frame-2.native.png`: `94135054744476bd22f9183736c7fe9fb34f9ea4677d6ee832f49b70d23e24f0`
  - `frame-2.diff.png`: `108b7589959f8414a39a4513185fb4b1b3543c324a6650b34aceafcbdcd9fcf1`
- `public-remainder-k7-r5-row-reverse-wrap-reverse-single`, frame 2.
  - `frame-2.chrome.png`: `be5f3d1fb371afcfcb8da6b359c073b3c0cc3c2362b65903dec11e57300a505c`
  - `frame-2.native.png`: `c05fd3bdb6a31d92cc5e8f4bfe1e9ea0b74f9808dd78dd072f44d966edf0d8ee`
  - `frame-2.diff.png`: `6161cd1f70e83a02081fcdbb2ac2b32ff7be9d2250ddabd7a79f5b014c178049`
- `public-remainder-k7-r6-column-reverse-wrap-single`, frame 2.
  - `frame-2.chrome.png`: `2f00b999b02e58ea0b8061c88f99209e8fe5152679905349b3d386737429c4c3`
  - `frame-2.native.png`: `3a7f49300d2156eaf6d0dee44c393f326fff26aa3f4b514c49d9329dac8569e6`
  - `frame-2.diff.png`: `1838416fc5cf6e967d74090882f634973476638746227edd838aca25035539cd`
- `public-remainder-k7-r6-column-reverse-wrap-reverse-single`, frame 2.
  - `frame-2.chrome.png`: `4a495ee03c2dfa432b93a1f84293ea95ef1a99a495d47cd166e9bf24aee43f32`
  - `frame-2.native.png`: `1bfb8fba2d1bec29363edd6c5ff5209cf98f6ec0c8b8964cfc998a0beb9e9458`
  - `frame-2.diff.png`: `615cb19a86ac76b171bea7f855c2b8a9a6e6259920d600c713582eb4dc986a0e`

## Sheet 22

SHA-256: `f5446dbd5d55f32d4b3747c4f15b8b8119b64052a5dbfdb5bde401c4b3e698b3`.

Paired items show deliberately unequal heights/widths and alignment steps. The thinner centered yellow/salmon/blue members line up with the corresponding larger member in the other image. Reverse-column pair crops and thin white channels agree.

- `public-remainder-k2-r1-row-wrap-pairs`, frame 2.
  - `frame-2.chrome.png`: `4be93a24a7e02b7bf40935c9e8385dd5ba5bd83c33fdc260c1cfee8967fc53eb`
  - `frame-2.native.png`: `8afd1dcb8f1557e9e83d531ba21081c0cb19caee7592f875232d90d41d0130dd`
  - `frame-2.diff.png`: `cf47e1b2a6d6ef41b9955d948398462be71689186606db35ae368b9b142fccfc`
- `public-remainder-k2-r1-column-reverse-wrap-reverse-pairs`, frame 2.
  - `frame-2.chrome.png`: `e2b95964a5f693943b4be67046f98349cee05bb067cfaa58add99898fadaaa60`
  - `frame-2.native.png`: `94922cc14b6c1d2d86137fb54f7498c7d25e7bed7e80de15cbefd356aa19ed6d`
  - `frame-2.diff.png`: `68c380c076461eaad170893b6300dde78604c98699b71a1b3ed736fd1233658a`
- `public-remainder-k3-r1-row-reverse-wrap-pairs`, frame 2.
  - `frame-2.chrome.png`: `56517b2ad78b8d6dc1c34f5ad4babcacb9e8d5a94fd4fad445d46f885d38edcd`
  - `frame-2.native.png`: `d88f52066819768278228e82eb4299dfe72d534b15cbf343a93577f73ba00201`
  - `frame-2.diff.png`: `b02ba256cb5f08cefc6cf6516b5121e992e3b51234f0d7d5f24eb6dc05e9e75e`
- `public-remainder-k3-r2-column-wrap-reverse-pairs`, frame 2.
  - `frame-2.chrome.png`: `c62001d0431bf02cd63dcb0e62446d272648e036486871ce252665e359c47935`
  - `frame-2.native.png`: `b440e0ee7bf34c942f164eb5d85540f61087d8ff358e73b82e9887f526831783`
  - `frame-2.diff.png`: `3f74297e71062c413199ff9776f64370bad47b099b37319cfb19f16eaf07582e`

## Sheet 23

SHA-256: `b12fe0dc61aa1a6aeb7b2cf01dd68ccedc5b0258d334140dc58247ec63097195`.

Paired column items preserve their staggered joins and white channels. Reverse-row pairs show matching thin yellow and salmon members, centered offsets and order. Seven-line reverse-column steps agree; the cropped reverse-row dark teal and magenta members end at the same artboard boundary.

- `public-remainder-k4-r1-column-wrap-pairs`, frame 2.
  - `frame-2.chrome.png`: `c8d4e5ebc82364da6b211371058e9cc8fc2d03cc9ddc6e49d2d73973fe4157b7`
  - `frame-2.native.png`: `bcda8eab69eb7d4c26f925307934bb61634c95f54249faabe61dd8f0b190e861`
  - `frame-2.diff.png`: `335c436e4fac01d018f5c50a4b184d03e53b84f78a698d85ee15e06205472e85`
- `public-remainder-k4-r3-row-reverse-wrap-reverse-pairs`, frame 2.
  - `frame-2.chrome.png`: `055e9f3adacf5daa5fd6b7367a166c51a1dfe738b385822abd5c15be21db4dbb`
  - `frame-2.native.png`: `5ab86023800dff1be71c598ffb491d66d0fe6b2cf13f53790d6c89b2036eead7`
  - `frame-2.diff.png`: `cc4688225cc4cbeacbf26da3b9f758e148ad5923e0d199ceb8437cb3a2788b04`
- `public-remainder-k7-r1-column-reverse-wrap-pairs`, frame 2.
  - `frame-2.chrome.png`: `82389ac151e7280328eb6ba00b0121dab05f58e6c203905fac5b230d60330fd5`
  - `frame-2.native.png`: `a9dae7429869ebee352a016cf3bf4e690fe4cccddc47ebcb899a5bdde1716a96`
  - `frame-2.diff.png`: `5e583139411419e1fa022b4e2811a0f74468cdc575c78478376e61585d0254de`
- `public-remainder-k7-r6-row-wrap-reverse-pairs`, frame 2.
  - `frame-2.chrome.png`: `fa49d2aeaa4c2be2a53404def8bb6093d88aaa9686bb7ab78fb2e8784472d2f4`
  - `frame-2.native.png`: `45da0cf512a8d544c9675ee67931fa9249b20491db8fd7403e2f51a5c9838933`
  - `frame-2.diff.png`: `78dc3d2c878150f6384a18939749afef06a9802eb03e759a5fba7bb9f718097f`
