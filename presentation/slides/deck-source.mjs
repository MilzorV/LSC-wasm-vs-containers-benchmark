const ROOT = process.env.PROJECT_ROOT;

const C = {
  bg: "#F6F5EF",
  ink: "#111827",
  muted: "#586070",
  pale: "#ECE8DE",
  line: "#D6D3C8",
  teal: "#0E7490",
  blue: "#1D4ED8",
  green: "#166534",
  amber: "#B45309",
  red: "#B91C1C",
  white: "#FFFFFF",
};

const slides = [
  {
    section: "LSC Project",
    title: "Movie Search on Spin/WASM vs OCI",
    subtitle: "Comparing WebAssembly as isolation for HTTP microservices",
    type: "title",
    notes:
      "Open with the core point: this is not a direct Meilisearch benchmark, but a comparison of the same Rust microservice in two isolation models.",
  },
  {
    section: "Research Question",
    title: "Can the same microservice run practically on Spin/WASI and OCI?",
    claim: "We prove functional parity first, then compare cold start, load, and memory.",
    bullets: [
      "Same application code: movie-search-core",
      "Same dataset: 44,471 movies after id deduplication",
      "Same HTTP endpoints and benchmark payloads",
      "Runtime/isolation comparison, not two different search engines",
    ],
    notes:
      "Set up the methodology: we care about isolation and startup differences, so we remove differences in application logic.",
  },
  {
    section: "Stack",
    title: "What are Spin, WASM, WASI, and wasmtime for?",
    type: "stack",
    notes:
      "Spin is a framework for WebAssembly apps and microservices. WASI provides controlled system interfaces, and wasmtime executes components outside the browser.",
  },
  {
    section: "Pivot",
    title: "Why did we move away from full Meilisearch?",
    claim: "Meilisearch was useful feasibility evidence, but not a fair 1:1 Spin benchmark.",
    bullets: [
      "Official Meilisearch stores data in LMDB and memory-mapped files",
      "A Spin fallback had different ranking semantics and a different data model",
      "The upstream WASI build stopped on runtime/storage dependencies",
      "Conclusion: measure a shared service instead of pretending parity",
    ],
    notes:
      "Explain the pivot honestly: the failed direct port became evidence that the benchmark method needed to change.",
  },
  {
    section: "Application",
    title: "What does movie-search do?",
    type: "application",
    notes:
      "The app is compact by design, but now includes opt-in Meilisearch-like features in the shared core for a stronger demo.",
  },
  {
    section: "Architektura",
    title: "One core, two runtime adapters",
    type: "architecture",
    notes:
      "This is the key technical slide: the shared core is the center, with Spin and OCI as adapters around the same logic.",
  },
  {
    section: "Methodology",
    title: "How did we measure?",
    type: "methodology",
    notes:
      "Show the full pipeline: tests, build, parity, cold start, load, memory, analysis. The full benchmark is not part of the live demo.",
  },
  {
    section: "Parity",
    title: "Both runtimes return the same results",
    type: "parity",
    notes:
      "This is the credibility gate. Parity covers engine, documentCount, estimatedTotalHits, hit.id lists, enhanced payloads, and suggestions.",
  },
  {
    section: "Cold start",
    title: "Cold start: OCI is ready sooner, Spin completes first search sooner",
    type: "cold",
    notes:
      "Explain the difference between /health readiness and the first real /search request. They are different metrics.",
  },
  {
    section: "Load: space",
    title: "Query space: linear scanning stresses both runtimes",
    type: "load-space",
    notes:
      "The space query exercises text scanning. Spin has higher request rate, but worse tail latency at high concurrency.",
  },
  {
    section: "Load: empty",
    title: "Empty query exposes runtime/HTTP path overhead",
    type: "load-empty",
    notes:
      "The empty query is application-light, so it strongly exposes runtime and HTTP server overhead.",
  },
  {
    section: "Memory",
    title: "Memory: useful operationally, not identical accounting",
    type: "memory",
    notes:
      "OCI is measured via docker stats; Spin is measured as process-tree RSS. Say this clearly so the conclusion stays honest.",
  },
  {
    section: "Demo",
    title: "Demo: show parity, not a full live benchmark run",
    type: "demo",
    notes:
      "The demo is a few minutes: health, version, stats, legacy search, enhanced search, compare_results, and saved charts.",
  },
  {
    section: "Conclusions",
    title: "Kiedy ten stack jest sensowny?",
    type: "conclusion",
    notes:
      "End with the nuanced conclusion: Spin/WASM is promising for small isolated services; OCI still wins on operational maturity.",
  },
];

export async function makeSlide(number, presentation, ctx) {
  if (!ROOT) {
    throw new Error("PROJECT_ROOT environment variable is required");
  }
  const model = slides[number - 1];
  if (!model) {
    throw new Error(`No model for slide ${number}`);
  }
  const slide = presentation.slides.add();
  base(slide, ctx, model);

  switch (model.type) {
    case "title":
      titleSlide(slide, ctx, model);
      break;
    case "stack":
      stackSlide(slide, ctx, model);
      break;
    case "application":
      applicationSlide(slide, ctx, model);
      break;
    case "architecture":
      architectureSlide(slide, ctx, model);
      break;
    case "methodology":
      methodologySlide(slide, ctx, model);
      break;
    case "parity":
      paritySlide(slide, ctx, model);
      break;
    case "cold":
      await coldSlide(slide, ctx, model);
      break;
    case "load-space":
      await loadSpaceSlide(slide, ctx, model);
      break;
    case "load-empty":
      await loadEmptySlide(slide, ctx, model);
      break;
    case "memory":
      await memorySlide(slide, ctx, model);
      break;
    case "demo":
      demoSlide(slide, ctx, model);
      break;
    case "conclusion":
      conclusionSlide(slide, ctx, model);
      break;
    default:
      bulletsSlide(slide, ctx, model);
  }

  footer(slide, ctx, number);
  slide.speakerNotes.setText(model.notes);
  return slide;
}

function base(slide, ctx, model) {
  ctx.addShape(slide, { x: 0, y: 0, w: 1280, h: 720, fill: C.bg, line: ctx.line() });
  ctx.addShape(slide, { x: 0, y: 0, w: 1280, h: 10, fill: C.teal, line: ctx.line() });
  ctx.addText(slide, {
    text: model.section,
    x: 56,
    y: 28,
    w: 520,
    h: 24,
    fontSize: 15,
    bold: true,
    color: C.teal,
  });
  if (model.type !== "title") {
    ctx.addText(slide, {
      text: model.title,
      x: 56,
      y: 66,
      w: 1080,
      h: 74,
      fontSize: 31,
      bold: true,
      color: C.ink,
      typeface: ctx.fonts.title,
    });
  }
}

function footer(slide, ctx, number) {
  ctx.addShape(slide, { x: 56, y: 665, w: 1110, h: 1, fill: C.line, line: ctx.line() });
  ctx.addText(slide, {
    text: "LSC 2026 · Spin/WASM vs OCI · movie-search",
    x: 56,
    y: 676,
    w: 600,
    h: 22,
    fontSize: 12,
    color: C.muted,
  });
  ctx.addText(slide, {
    text: String(number).padStart(2, "0"),
    x: 1140,
    y: 674,
    w: 70,
    h: 24,
    fontSize: 13,
    bold: true,
    align: "right",
    color: C.muted,
  });
}

function titleSlide(slide, ctx, model) {
  ctx.addText(slide, {
    text: model.title,
    x: 72,
    y: 136,
    w: 840,
    h: 120,
    fontSize: 46,
    bold: true,
    color: C.ink,
    typeface: ctx.fonts.title,
  });
  ctx.addText(slide, {
    text: model.subtitle,
    x: 74,
    y: 270,
    w: 760,
    h: 60,
    fontSize: 23,
    color: C.muted,
  });
  metric(slide, ctx, 74, 392, "44,471", "movies in fixture", C.teal);
  metric(slide, ctx, 334, 392, "1 core", "shared Rust core", C.green);
  metric(slide, ctx, 594, 392, "2 runtimes", "Spin/WASI and OCI", C.amber);
  ctx.addShape(slide, { x: 925, y: 95, w: 250, h: 380, fill: C.pale, line: ctx.line(C.line, 1) });
  ctx.addText(slide, {
    text: "HTTP microservice benchmark\nwithout pretending full Meilisearch parity",
    x: 955,
    y: 150,
    w: 190,
    h: 130,
    fontSize: 25,
    bold: true,
    color: C.ink,
    align: "center",
    valign: "middle",
  });
  ctx.addText(slide, {
    text: "Full run: 2026-05-24",
    x: 955,
    y: 334,
    w: 190,
    h: 36,
    fontSize: 16,
    color: C.muted,
    align: "center",
  });
}

function bulletsSlide(slide, ctx, model) {
  callout(slide, ctx, model.claim, 70, 150, 1080, 70, C.teal);
  bulletList(slide, ctx, 100, 260, model.bullets, 27, 62);
}

function stackSlide(slide, ctx) {
  const items = [
    ["WebAssembly", "portable binary format and execution sandbox", C.blue],
    ["WASI", "controlled interfaces: HTTP, filesystem, clock, random", C.green],
    ["wasmtime", "runtime executing components outside the browser", C.teal],
    ["Spin", "framework and CLI for WebAssembly microservices", C.amber],
    ["OCI", "container image and runtime standard used as the baseline", C.red],
  ];
  items.forEach(([name, desc, color], i) => {
    const y = 156 + i * 86;
    ctx.addShape(slide, { x: 80, y, w: 260, h: 58, fill: color, line: ctx.line() });
    ctx.addText(slide, { text: name, x: 100, y: y + 13, w: 220, h: 30, fontSize: 23, bold: true, color: C.white, align: "center" });
    ctx.addShape(slide, { x: 365, y, w: 760, h: 58, fill: C.white, line: ctx.line(C.line, 1) });
    ctx.addText(slide, { text: desc, x: 390, y: y + 16, w: 720, h: 28, fontSize: 20, color: C.ink });
  });
}

function applicationSlide(slide, ctx) {
  callout(slide, ctx, "The app is compact by design: it exposes runtime cost while still offering opt-in search-product features for the demo.", 70, 145, 1080, 70, C.green);
  metric(slide, ctx, 90, 260, "GET", "/health · /version · /stats · /movies", C.teal, 355);
  metric(slide, ctx, 505, 260, "POST", "/search and /suggest", C.amber, 315);
  metric(slide, ctx, 880, 260, "ranking", "tokens, fields, id ascending", C.blue, 270);
  bulletList(slide, ctx, 120, 405, [
    "Last-write-wins deduplication by id",
    "Filters, facets, sorting, highlights, suggestions, typo tolerance",
    "Empty query returns a deterministic id-ordered list",
    "Legacy benchmark queries stay backward compatible",
  ], 24, 46);
}

function architectureSlide(slide, ctx) {
  box(slide, ctx, 430, 155, 420, 92, "movie-search-core", "fixture · ranking · filters · facets · suggestions", C.teal);
  box(slide, ctx, 120, 345, 330, 92, "Spin adapter", "WASI HTTP component\n127.0.0.1:8080", C.blue);
  box(slide, ctx, 830, 345, 330, 92, "OCI adapter", "native Rust HTTP server\n127.0.0.1:8081", C.green);
  box(slide, ctx, 410, 505, 460, 74, "Benchmark harness", "smoke · parity · cold start · load · memory", C.amber);
  segment(slide, ctx, 638, 247, 4, 54);
  segment(slide, ctx, 285, 300, 710, 4);
  segment(slide, ctx, 283, 300, 4, 45);
  segment(slide, ctx, 993, 300, 4, 45);
  segment(slide, ctx, 638, 437, 4, 68);
  segment(slide, ctx, 285, 468, 710, 4);
  segment(slide, ctx, 283, 437, 4, 35);
  segment(slide, ctx, 993, 437, 4, 35);
}

function methodologySlide(slide, ctx) {
  const steps = [
    ["1", "Tests and build", "cargo test, spin build, docker compose build"],
    ["2", "Parity", "same hit.id plus enhanced payload"],
    ["3", "Cold start", "20 repetitions per runtime"],
    ["4", "Load", "space, empty, enhanced; c=10/50/100/200"],
    ["5", "Memory", "Docker stats vs host process RSS"],
  ];
  steps.forEach(([n, title, desc], i) => {
    const x = 92 + i * 220;
    ctx.addShape(slide, { x, y: 178, w: 150, h: 150, fill: C.white, line: ctx.line(C.line, 1) });
    ctx.addText(slide, { text: n, x: x + 48, y: 194, w: 54, h: 54, fontSize: 29, bold: true, color: C.white, align: "center", valign: "middle", fill: C.teal, insets: { left: 0, right: 0, top: 8, bottom: 0 } });
    ctx.addText(slide, { text: title, x: x + 12, y: 264, w: 126, h: 30, fontSize: 17, bold: true, color: C.ink, align: "center" });
    ctx.addText(slide, { text: desc, x: x + 12, y: 299, w: 126, h: 52, fontSize: 12, color: C.muted, align: "center" });
  });
  callout(slide, ctx, "Full benchmark: benchmarks/run_all.sh. The demo shows parity and artifacts; it does not run the full load test live.", 110, 420, 980, 82, C.amber);
}

function paritySlide(slide, ctx) {
  metric(slide, ctx, 84, 158, "0", "validation errors", C.green, 260);
  metric(slide, ctx, 394, 158, "44,471", "documentCount", C.teal, 260);
  metric(slide, ctx, 704, 158, "5+", "parity queries", C.blue, 260);
  ctx.addShape(slide, { x: 90, y: 330, w: 1030, h: 210, fill: C.ink, line: ctx.line() });
  ctx.addText(slide, {
    text: "'space': [62, 957, 1542, 2157, ...]\n'toy story': [862, 863, 10193, ...]\n'dark knight': [155, 29751, 49026, ...]\nenhanced: filters + facets + highlights\n/suggest: same ranked suggestions\nSpin and OCI movie-search results match.",
    x: 120,
    y: 356,
    w: 970,
    h: 160,
    fontSize: 22,
    color: "#E5E7EB",
    typeface: ctx.fonts.mono,
  });
}

async function coldSlide(slide, ctx) {
  await chart(slide, ctx, "cold_start_p95.png", 705, 166, 440, 300);
  metric(slide, ctx, 92, 168, "38.7 ms", "OCI p95 /health", C.green, 245);
  metric(slide, ctx, 370, 168, "202.7 ms", "Spin p95 /health", C.teal, 245);
  metric(slide, ctx, 92, 332, "854.8 ms", "OCI p95 first search", C.amber, 245);
  metric(slide, ctx, 370, 332, "446.3 ms", "Spin p95 first search", C.blue, 245);
  ctx.addText(slide, { text: "Interpretation: /health measures process readiness; first /search measures the real application path.", x: 110, y: 525, w: 1020, h: 42, fontSize: 20, color: C.muted, align: "center" });
}

async function loadSpaceSlide(slide, ctx) {
  await chart(slide, ctx, "load_latency_p95.png", 674, 148, 470, 345);
  metric(slide, ctx, 82, 168, "16.8 req/s", "Spin c=10", C.teal, 250);
  metric(slide, ctx, 372, 168, "6.0 req/s", "OCI c=10", C.green, 250);
  metric(slide, ctx, 82, 332, "1098 ms", "Spin p95 c=10", C.blue, 250);
  metric(slide, ctx, 372, 332, "2443 ms", "OCI p95 c=10", C.amber, 250);
  callout(slide, ctx, "For space, both variants perform expensive linear text scanning; at c=200 both hit client-side timeouts.", 100, 540, 1000, 58, C.teal);
}

async function loadEmptySlide(slide, ctx) {
  await chart(slide, ctx, "load_throughput.png", 650, 145, 500, 350);
  metric(slide, ctx, 78, 168, "2225 req/s", "OCI c=10", C.green, 260);
  metric(slide, ctx, 378, 168, "84 req/s", "Spin c=10", C.teal, 260);
  metric(slide, ctx, 78, 332, "142 ms", "OCI p95 c=200", C.blue, 260);
  metric(slide, ctx, 378, 332, "5219 ms", "Spin p95 c=200", C.red, 260);
  callout(slide, ctx, "The empty query is application-light, so it exposes HTTP/runtime overhead strongly.", 100, 540, 1000, 58, C.amber);
}

async function memorySlide(slide, ctx) {
  await chart(slide, ctx, "memory_peak.png", 704, 158, 430, 316);
  metric(slide, ctx, 92, 168, "37.4 MiB", "OCI load max", C.green, 245);
  metric(slide, ctx, 370, 168, "553.1 MiB", "Spin load max RSS", C.red, 245);
  bulletList(slide, ctx, 115, 360, [
    "OCI: docker stats / cgroup view",
    "Spin: host process tree RSS",
    "Conclusion: operational comparison, not identical accounting",
  ], 22, 42);
}

function demoSlide(slide, ctx) {
  const steps = [
    "Start Spin :8080 and OCI :8081",
    "Show /health, /version, /stats",
    "Send POST /search for q=space",
    "Enable enhanced search and call /suggest",
    "Run benchmarks/compare_results.sh",
    "Show CSVs and plots from the full benchmark",
  ];
  bulletList(slide, ctx, 105, 170, steps, 25, 58);
  ctx.addShape(slide, { x: 740, y: 180, w: 340, h: 260, fill: C.ink, line: ctx.line() });
  ctx.addText(slide, {
    text: "curl /version\ncurl /stats\ncurl -X POST /search\ncurl -X POST /suggest\nbenchmarks/compare_results.sh\nls results/plots",
    x: 770,
    y: 218,
    w: 290,
    h: 180,
    fontSize: 21,
    color: "#E5E7EB",
    typeface: ctx.fonts.mono,
  });
  callout(slide, ctx, "The full benchmark takes several minutes or more, so the live demo shows the procedure and parity proof.", 122, 540, 980, 58, C.blue);
}

function conclusionSlide(slide, ctx) {
  const cols = [
    ["Spin/WASM fits when", ["small isolated HTTP component", "edge/serverless/plugin", "portability and sandboxing matter"], C.teal],
    ["OCI remains better when", ["mature tooling is required", "storage-heavy service", "precise resource accounting"], C.green],
    ["Our result", ["functional parity: yes", "performance: endpoint-dependent", "Meilisearch port: feasibility blocker"], C.amber],
  ];
  cols.forEach(([title, bullets, color], i) => {
    const x = 78 + i * 390;
    ctx.addShape(slide, { x, y: 164, w: 330, h: 360, fill: C.white, line: ctx.line(C.line, 1) });
    ctx.addText(slide, { text: title, x: x + 24, y: 194, w: 280, h: 56, fontSize: 22, bold: true, color });
    bulletList(slide, ctx, x + 34, 276, bullets, 19, 52, 250);
  });
  ctx.addText(slide, { text: "Most important: the benchmark is fair because it compares the same application code.", x: 130, y: 566, w: 940, h: 42, fontSize: 25, bold: true, color: C.ink, align: "center" });
}

function bulletList(slide, ctx, x, y, items, size = 22, gap = 50, width = 900) {
  items.forEach((item, i) => {
    const top = y + i * gap;
    ctx.addShape(slide, { x, y: top + 9, w: 10, h: 10, fill: C.teal, line: ctx.line() });
    ctx.addText(slide, { text: item, x: x + 28, y: top, w: width, h: gap - 6, fontSize: size, color: C.ink });
  });
}

function metric(slide, ctx, x, y, value, label, color, width = 230) {
  ctx.addShape(slide, { x, y, w: width, h: 112, fill: C.white, line: ctx.line(C.line, 1) });
  ctx.addShape(slide, { x, y, w: 8, h: 112, fill: color, line: ctx.line() });
  ctx.addText(slide, { text: value, x: x + 24, y: y + 18, w: width - 40, h: 44, fontSize: 29, bold: true, color: C.ink });
  ctx.addText(slide, { text: label, x: x + 24, y: y + 68, w: width - 40, h: 30, fontSize: 15, color: C.muted });
}

function callout(slide, ctx, text, x, y, w, h, color) {
  ctx.addShape(slide, { x, y, w, h, fill: C.white, line: ctx.line(color, 2) });
  ctx.addText(slide, { text, x: x + 22, y: y + 12, w: w - 44, h: h - 20, fontSize: 22, bold: true, color: C.ink, valign: "middle" });
}

function box(slide, ctx, x, y, w, h, title, subtitle, color) {
  ctx.addShape(slide, { x, y, w, h, fill: C.white, line: ctx.line(color, 2) });
  ctx.addText(slide, { text: title, x: x + 18, y: y + 16, w: w - 36, h: 28, fontSize: 22, bold: true, color });
  ctx.addText(slide, { text: subtitle, x: x + 18, y: y + 48, w: w - 36, h: h - 54, fontSize: 16, color: C.muted });
}

function segment(slide, ctx, x, y, w, h) {
  ctx.addShape(slide, { x, y, w, h, fill: C.line, line: ctx.line() });
}

async function chart(slide, ctx, file, x, y, w, h) {
  await ctx.addImage(slide, {
    path: `${ROOT}/results/plots/${file}`,
    x,
    y,
    w,
    h,
    fit: "contain",
    alt: file,
  });
}
