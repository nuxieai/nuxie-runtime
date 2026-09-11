# Add the border-repeat sentinel after the active full run terminates

The comparator is intentionally scoped to the existing fractional-border fixture's
one-device-pixel transparent border at DPR1. It samples six points away from
corners using Chrome geometry and measured used border widths; it retains the
existing local hard-stop limit of two channel units. It is not a generic border
antialias comparator. The Python gate's separate opaque-border control is unchanged.

In `validation/browser.spec.mjs`, add:

```js
import {compareGradientBorderPixels} from './gradient-border-pixels.mjs';
```

Immediately after `const {diff, failures} = comparison;`, add:

```js
if (fixture.name === 'linear-gradient-composition-fractional-border') {
  const usedBorder = await page.evaluate(() => {
    const element = document.getElementById('gradient');
    if (!element) throw new Error('Missing gradient border sentinel');
    const style = getComputedStyle(element);
    return {
      borderLeftWidth: parseFloat(style.borderLeftWidth),
      borderRightWidth: parseFloat(style.borderRightWidth),
    };
  });
  const borderRepeat = compareGradientBorderPixels(reference, actual, {
    viewport: {width, height},
    box: browser.gradient,
    ...usedBorder,
  });
  comparison.metrics.gradientBorderRepeat = borderRepeat;
  if (!borderRepeat.passed) failures.push('gradient border repeat mismatch');
}
```

This keeps the existing aggregate metrics and thresholds intact while recording
and enforcing the local failure. Missing geometry, wrong image dimensions,
changed used border widths and offscreen sample positions throw before a pass.

Controls: `node tools/html-to-riv/validation/test-gradient-border-pixels.mjs`.
The immutable receipt is
`output/playwright/html-to-riv/linear-gradient-border-js-controls.json`:
three preserved r1 failures, three corrected r3 passes, six browser-self passes,
and thirty malformed-geometry/dimension/border rejection controls.
The script refuses to overwrite its control receipt.
