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
      "backend=webgpu fallback=false",
      "backend=webgl2 fallback=false",
      "direct-gpu-canvas=webgpu",
      "direct-gpu-canvas=webgl2",
      "image-mesh=webgl2 indexed=cropped general-triangles=applied transform=applied clip-layer=applied opacity=applied samplers=nearest+bilinear+repeat unsupported=mirror+advanced-blend-fail-closed",
      "imported-gpu-canvas=webgpu selected=webgpu",
      "imported-gpu-canvas=webgl2 selected=webgl2",
      "gpu-canvas-interface=sync-rejected unrelated=clean valid=clean",
      "webgl2-gpu-canvas-interface=attributes+uniforms+interstage-rejected valid=clean",
      "imported-gpu-canvas-uniform-animation=webgl2 frames=2 first-instance=5 reversed-slots=applied programs=1 vaos=1 buffers=3 contexts=2",
      "webgpu-uniform-limit=same-call-rejected unrelated=clean valid=clean",
      "imported-gpu-canvas-stress=webgl2 frames=32 keys=2 programs=2 vaos=2 buffers=0 contexts=2",
      "resize=webgpu in-flight=rejected extent=13x9",
      "resize=webgl2 in-flight=rejected extent=13x9",
      "stream=gm-rect backend=webgpu",
      "stream=gm-rect backend=webgl2",
      "stream=gm-image backend=webgpu",
      "stream=gm-image backend=webgl2",
      "stream=gm-image_filter_options backend=webgpu",
      "stream=gm-image_filter_options backend=webgl2",
      "stream=riv-scripted_color-frame-0 backend=webgpu",
      "stream=riv-scripted_color-frame-0 backend=webgl2",
      "path-clip=exact unsupported=fail-closed recovery=clean abandoned=poisoned",
    ],
  },
  {
    path: "?force-webgl2-fallback=1",
    expected: ["backend=webgl2 fallback=true", "forced-webgpu=fail-closed"],
  },
];
const cases = process.env.BROWSER_RENDERER_GPU_ONLY === "1"
  ? [{
      path: "gpu-only.html",
      expected: [
        "direct-gpu-canvas=webgpu",
        "direct-gpu-canvas=webgl2",
        "imported-gpu-canvas=webgpu selected=webgpu",
        "imported-gpu-canvas=webgl2 selected=webgl2",
        "gpu-canvas-interface=sync-rejected unrelated=clean valid=clean",
        "webgl2-gpu-canvas-interface=attributes+uniforms+interstage-rejected valid=clean",
        "imported-gpu-canvas-uniform-animation=webgl2 frames=2 first-instance=5 reversed-slots=applied programs=1 vaos=1 buffers=3 contexts=2",
        "webgpu-uniform-limit=same-call-rejected unrelated=clean valid=clean",
        "imported-gpu-canvas-stress=webgl2 frames=32 keys=2 programs=2 vaos=2 buffers=0 contexts=2",
      ],
    }]
  : fullCases;

(async () => {
  const browser = await chromium.launch(launchOptions);
  try {
    for (const testCase of cases) {
      const page = await browser.newPage();
      page.on("console", (message) => {
        if (
          message.type() === "error"
          || message.type() === "warning"
          || message.text().startsWith("gpu-smoke:")
        ) {
          console.log(`browser ${message.type()}: ${message.text()}`);
        }
      });
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
