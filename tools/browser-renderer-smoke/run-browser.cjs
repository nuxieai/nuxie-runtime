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

const fullCases = [
  {
    path: "",
    expected: [
      "backend=webgpu",
      "browser-presentation=direct-webgpu exact-rgba=true surface=1 mapAsync=0 putImageData=0",
      "browser-readback=explicit rgba-bytes=48 exact=true surface=0 mapAsync=1 putImageData=0",
      "resize=webgpu in-flight=rejected extent=13x9",
      "direct-gpu-canvas=webgpu",
      "imported-gpu-canvas=webgpu",
      "gpu-canvas-interface=sync-rejected unrelated=clean valid=clean",
      "webgpu-uniform-limit=same-call-rejected unrelated=clean valid=clean",
      "stream=gm-rect backend=webgpu",
      "stream=gm-rect_grad backend=webgpu",
      "stream=gm-degengrad backend=webgpu",
      "stream=gm-CubicStroke backend=webgpu",
      "stream=gm-cliprects backend=webgpu",
      "stream=gm-poly_clockwise backend=webgpu",
      "stream=gm-poly_evenOdd backend=webgpu",
      "stream=gm-image backend=webgpu",
      "stream=gm-image_filter_options backend=webgpu",
      "stream=riv-scripted_color-frame-0 backend=webgpu",
    ],
  },
  {
    path: "?force-webgpu-compatibility=1",
    expected: [
      "backend=webgpu",
      "compatibility=selected order=core,compatibility requested-vertex-storage-limit=",
    ],
  },
  {
    path: "?force-webgpu-compatibility=1&force-no-ssbo=1",
    expected: [
      "backend=webgpu",
      "compatibility=selected order=core,compatibility vertex-storage-limit=0 polyfill=rendered",
    ],
  },
  {
    path: "?force-webgpu-unavailable=1",
    expected: ["webgpu-unavailable=adapter-error"],
  },
  {
    path: "?force-webgpu-no-adapter=1",
    expected: [
      "webgpu-no-adapter=adapter-error order=core,compatibility",
    ],
  },
  {
    path: "?force-webgpu-device-failure=1",
    expected: [
      "webgpu-device-failure=device-error order=core,compatibility",
    ],
  },
];

const cases = process.env.BROWSER_RENDERER_GPU_ONLY === "1"
  ? [{
      path: "gpu-only.html",
      expected: [
        "direct-gpu-canvas=webgpu",
        "imported-gpu-canvas=webgpu",
        "gpu-canvas-clean-error-scope=clean rendered-pixels=64 red-pixels=32",
        "gpu-canvas-error-scope=concrete-error-preserved",
        "gpu-canvas-interface=sync-rejected unrelated=clean valid=clean",
        "webgpu-uniform-limit=same-call-rejected unrelated=clean valid=clean",
      ],
    }]
  : fullCases;

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
