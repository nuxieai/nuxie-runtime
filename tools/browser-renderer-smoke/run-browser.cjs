const { chromium } = require("playwright");

const baseUrl = process.argv[2];
if (!baseUrl) {
  throw new Error("usage: node run-browser.cjs <base-url>");
}

const browserMode = process.env.BROWSER_RENDERER_BROWSER || "chrome";
const launchOptions = {
  headless: true,
  args: ["--enable-unsafe-webgpu"],
};
if (browserMode === "chrome") {
  launchOptions.channel = "chrome";
} else if (browserMode !== "chromium") {
  throw new Error(`unknown BROWSER_RENDERER_BROWSER ${browserMode}`);
}

const cases = [
  {
    path: "",
    expected: [
      "backend=webgpu fallback=false",
      "backend=webgl2 fallback=false",
      "stream=gm-rect backend=webgpu",
      "stream=gm-rect backend=webgl2",
      "stream=gm-image backend=webgpu",
      "stream=gm-image backend=webgl2",
      "stream=riv-scripted_color-frame-0 backend=webgpu",
      "stream=riv-scripted_color-frame-0 backend=webgl2",
      "unsupported=fail-closed recovery=clean abandoned=poisoned",
    ],
  },
  {
    path: "?force-webgl2-fallback=1",
    expected: ["backend=webgl2 fallback=true", "forced-webgpu=fail-closed"],
  },
  {
    path: "?force-webgpu-compatibility=1",
    expected: [
      "backend=webgpu fallback=false",
      "compatibility=selected requested-vertex-storage-limit=",
    ],
  },
  {
    path: "?force-webgpu-compatibility=1&force-no-ssbo=1",
    expected: [
      "backend=webgpu fallback=false",
      "compatibility=selected vertex-storage-limit=0 polyfill=rendered",
    ],
  },
];

(async () => {
  const browser = await chromium.launch(launchOptions);
  try {
    for (const testCase of cases) {
      const page = await browser.newPage();
      page.on("console", (message) =>
        console.log(`browser console ${message.type()}: ${message.text()}`),
      );
      page.on("pageerror", (error) =>
        console.error(`browser page error: ${error.stack || String(error)}`),
      );
      await page.goto(`${baseUrl}${testCase.path}`, { waitUntil: "networkidle" });
      await page.waitForFunction(
        () => ["passed", "failed"].includes(document.body.dataset.status),
        undefined,
        { timeout: 180_000 },
      );
      const state = await page.getAttribute("body", "data-status");
      const status = await page.textContent("#status");
      console.log(`browser case ${testCase.path || "default"}:\n${status}`);
      if (state !== "passed") {
        throw new Error(`browser smoke failed for ${testCase.path || "default"}: ${status}`);
      }
      for (const expected of testCase.expected) {
        if (!status.includes(expected)) {
          throw new Error(
            `browser smoke for ${testCase.path || "default"} omitted ${expected}`,
          );
        }
      }
      await page.close();
    }
  } finally {
    await browser.close();
  }
})().catch((error) => {
  console.error(error.stack || String(error));
  process.exitCode = 1;
});
