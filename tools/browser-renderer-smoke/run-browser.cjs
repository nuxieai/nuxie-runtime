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
      "browser-presentation=direct-webgpu alpha=premultiplied exact-rgba=true composite=over-blue surface=1 mapAsync=0 putImageData=0",
      "browser-readback=explicit rgba-bytes=48 exact=true surface=0 mapAsync=1 putImageData=0",
      "direct-gpu-canvas=webgpu",
      "imported-gpu-canvas=webgpu",
      "gpu-canvas-physical-shader=rejected lua-values=zero device=clean valid=clean",
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
        "gpu-canvas-physical-shader=rejected lua-values=zero device=clean valid=clean",
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
      // Uncaptured WebGPU errors never reject a page promise — Chrome only
      // reports them on the console. Collect them so a device poisoned by a
      // rejected shader (UNIV-1764) fails the case instead of passing on
      // pixels alone.
      const uncapturedGpuErrors = [];
      page.on("console", (message) => {
        console.log(`browser console ${message.type()}: ${message.text()}`);
        if (
          message.type() === "error" &&
          /webgpu|wgpu|uncaptured/i.test(message.text())
        ) {
          uncapturedGpuErrors.push(message.text());
        }
      });
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
      if (uncapturedGpuErrors.length > 0) {
        throw new Error(
          `browser smoke for ${testCase.path || "default"} left uncaptured WebGPU errors:\n${uncapturedGpuErrors.join("\n")}`,
        );
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
