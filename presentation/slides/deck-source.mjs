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
    section: "Projekt LSC",
    title: "Movie Search na Spin/WASM vs OCI",
    subtitle: "Porównanie WebAssembly jako izolacji dla mikroserwisów HTTP",
    type: "title",
    notes:
      "Otwieramy jasnym celem: to nie jest benchmark Meilisearch, tylko porównanie tego samego mikroserwisu Rust w dwóch modelach izolacji.",
  },
  {
    section: "Pytanie badawcze",
    title: "Czy ten sam mikroserwis działa praktycznie w Spin/WASI i OCI?",
    claim: "Najpierw dowodzimy zgodności funkcjonalnej, dopiero potem porównujemy cold start, load i pamięć.",
    bullets: [
      "Ten sam kod aplikacyjny: movie-search-core",
      "Ten sam dataset: 44 471 filmów po deduplikacji id",
      "Te same endpointy HTTP i te same zapytania benchmarkowe",
      "Porównanie runtime/isolation, nie dwóch różnych wyszukiwarek",
    ],
    notes:
      "Tu ustawiamy metodologię: interesuje nas różnica izolacji i uruchomienia, więc eliminujemy różnicę logiki aplikacji.",
  },
  {
    section: "Stack",
    title: "Do czego służy Spin/WASM/WASI/wasmtime?",
    type: "stack",
    notes:
      "Spin to framework dla aplikacji i mikroserwisów WebAssembly. WASI daje kontrolowane interfejsy systemowe, a wasmtime wykonuje komponenty poza przeglądarką.",
  },
  {
    section: "Pivot",
    title: "Dlaczego odeszliśmy od pełnego Meilisearch?",
    claim: "Meilisearch był dobrym case study feasibility, ale złym benchmarkiem 1:1 dla Spin.",
    bullets: [
      "Oficjalny Meilisearch opiera storage o LMDB i memory-mapped files",
      "Spin fallback miał inną semantykę rankingu i inny model danych",
      "Upstream WASI build zatrzymał się na zależnościach runtime/storage",
      "Wniosek: uczciwiej mierzyć własny wspólny serwis niż udawać parytet",
    ],
    notes:
      "Ten slajd uzasadnia decyzję projektową. Nie chowamy porażki portu: pokazujemy ją jako powód poprawienia metody.",
  },
  {
    section: "Aplikacja",
    title: "Co robi movie-search?",
    type: "application",
    notes:
      "Aplikacja jest prosta celowo. Dzięki temu wyniki wynikają z runtime'u i adapterów, a nie z ukrytej złożoności silnika wyszukiwarki.",
  },
  {
    section: "Architektura",
    title: "Jeden core, dwa adaptery runtime",
    type: "architecture",
    notes:
      "To najważniejszy slajd techniczny: shared core jest środkiem ciężkości, a Spin i OCI to adaptery wokół tej samej logiki.",
  },
  {
    section: "Metodyka",
    title: "Jak mierzyliśmy?",
    type: "methodology",
    notes:
      "Pokazujemy pełny pipeline: testy, build, parity, cold start, load, memory, analiza. Full benchmark nie jest częścią demo live.",
  },
  {
    section: "Parytet",
    title: "Oba runtime'y zwracają te same wyniki",
    type: "parity",
    notes:
      "To jest warunek wiarygodności. Parytet obejmuje engine, documentCount, estimatedTotalHits i listy hit.id.",
  },
  {
    section: "Cold start",
    title: "Cold start: OCI szybciej gotowe, Spin szybciej kończy pierwszy search",
    type: "cold",
    notes:
      "Wyjaśniamy różnicę między gotowością /health i pierwszym realnym requestem /search. To dwie różne metryki.",
  },
  {
    section: "Load: space",
    title: "Zapytanie space: liniowe skanowanie obciąża oba runtime'y",
    type: "load-space",
    notes:
      "Zapytanie space dotyka tekstowego skanowania. Spin ma wyższy request rate, ale przy dużej współbieżności ma gorszy ogon opóźnień.",
  },
  {
    section: "Load: empty",
    title: "Puste zapytanie pokazuje koszt ścieżki runtime/HTTP",
    type: "load-empty",
    notes:
      "Puste zapytanie jest lekkie aplikacyjnie, więc bardzo mocno ujawnia różnicę narzutu runtime i serwera HTTP.",
  },
  {
    section: "Pamięć",
    title: "Memory: metryki są porównywalne operacyjnie, ale nie identyczne",
    type: "memory",
    notes:
      "OCI mierzymy przez docker stats, Spin przez RSS drzewa procesów. To trzeba powiedzieć głośno, żeby wniosek był uczciwy.",
  },
  {
    section: "Demo",
    title: "Demo: pokazujemy zgodność, nie odpalamy pełnego benchmarku na żywo",
    type: "demo",
    notes:
      "Demo ma trwać kilka minut: health, version, stats, search, compare_results i pokazanie finalnych wykresów.",
  },
  {
    section: "Wnioski",
    title: "Kiedy ten stack jest sensowny?",
    type: "conclusion",
    notes:
      "Kończymy zniuansowanym wnioskiem: Spin/WASM jest obiecujący dla małych izolowanych usług, OCI nadal wygrywa dojrzałością operacyjną.",
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
  metric(slide, ctx, 74, 392, "44 471", "filmów w fixture", C.teal);
  metric(slide, ctx, 334, 392, "1 core", "wspólny Rust core", C.green);
  metric(slide, ctx, 594, 392, "2 runtime'y", "Spin/WASI i OCI", C.amber);
  ctx.addShape(slide, { x: 925, y: 95, w: 250, h: 380, fill: C.pale, line: ctx.line(C.line, 1) });
  ctx.addText(slide, {
    text: "Benchmark mikroserwisu HTTP\nbez udawania zgodności Meilisearch",
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
    text: "Pełny przebieg: 2026-05-24",
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
    ["WebAssembly", "przenośny format binarny i sandbox wykonania", C.blue],
    ["WASI", "kontrolowane interfejsy: HTTP, filesystem, clock, random", C.green],
    ["wasmtime", "runtime wykonujący komponenty poza przeglądarką", C.teal],
    ["Spin", "framework i CLI dla mikroserwisów WebAssembly", C.amber],
    ["OCI", "standard obrazu i runtime kontenera jako baseline", C.red],
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
  callout(slide, ctx, "Aplikacja jest prosta celowo: ma odsłonić koszt runtime'u, a nie imitować pełny silnik wyszukiwarki.", 70, 145, 1080, 70, C.green);
  metric(slide, ctx, 90, 260, "GET", "/health · /version · /stats · /movies", C.teal, 355);
  metric(slide, ctx, 505, 260, "POST", "/search { q, offset, limit }", C.amber, 315);
  metric(slide, ctx, 880, 260, "ranking", "tokeny, pola, id rosnąco", C.blue, 270);
  bulletList(slide, ctx, 120, 405, [
    "Deduplikacja id metodą last-write-wins",
    "Brak typo tolerance, fuzzy rankingu i reguł Meilisearch",
    "Puste zapytanie zwraca deterministyczną listę po id",
    "Zapytanie space skanuje pola title, genre i overview",
  ], 24, 46);
}

function architectureSlide(slide, ctx) {
  box(slide, ctx, 430, 155, 420, 92, "movie-search-core", "fixture · tokenizacja · ranking · response types", C.teal);
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
    ["1", "Testy i build", "cargo test, spin build, docker compose build"],
    ["2", "Parytet", "te same hit.id dla 5 zapytań"],
    ["3", "Cold start", "20 powtórzeń na runtime"],
    ["4", "Load", "concurrency 10, 50, 100, 200"],
    ["5", "Memory", "Docker stats vs host process RSS"],
  ];
  steps.forEach(([n, title, desc], i) => {
    const x = 92 + i * 220;
    ctx.addShape(slide, { x, y: 178, w: 150, h: 150, fill: C.white, line: ctx.line(C.line, 1) });
    ctx.addText(slide, { text: n, x: x + 48, y: 194, w: 54, h: 54, fontSize: 29, bold: true, color: C.white, align: "center", valign: "middle", fill: C.teal, insets: { left: 0, right: 0, top: 8, bottom: 0 } });
    ctx.addText(slide, { text: title, x: x + 12, y: 264, w: 126, h: 30, fontSize: 17, bold: true, color: C.ink, align: "center" });
    ctx.addText(slide, { text: desc, x: x + 12, y: 299, w: 126, h: 52, fontSize: 12, color: C.muted, align: "center" });
  });
  callout(slide, ctx, "Pełny benchmark: benchmarks/run_all.sh. Demo pokazuje zgodność i artefakty, nie uruchamia pełnego testu obciążeniowego.", 110, 420, 980, 82, C.amber);
}

function paritySlide(slide, ctx) {
  metric(slide, ctx, 84, 158, "0", "błędów walidacji", C.green, 260);
  metric(slide, ctx, 394, 158, "44 471", "documentCount", C.teal, 260);
  metric(slide, ctx, 704, 158, "5", "zapytań parity", C.blue, 260);
  ctx.addShape(slide, { x: 90, y: 330, w: 1030, h: 210, fill: C.ink, line: ctx.line() });
  ctx.addText(slide, {
    text: "'space': [62, 957, 1542, 2157, ...]\n'toy story': [862, 863, 10193, ...]\n'dark knight': [155, 29751, 49026, ...]\n'': [2, 3, 5, 6, 11, ...]\nSpin and OCI movie-search results match.",
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
  metric(slide, ctx, 92, 168, "28.4 ms", "OCI p95 /health", C.green, 245);
  metric(slide, ctx, 370, 168, "144.2 ms", "Spin p95 /health", C.teal, 245);
  metric(slide, ctx, 92, 332, "377.0 ms", "OCI p95 first search", C.amber, 245);
  metric(slide, ctx, 370, 332, "217.5 ms", "Spin p95 first search", C.blue, 245);
  ctx.addText(slide, { text: "Interpretacja: /health mierzy gotowość procesu, a pierwszy /search mierzy już realną ścieżkę aplikacji.", x: 110, y: 525, w: 1020, h: 42, fontSize: 20, color: C.muted, align: "center" });
}

async function loadSpaceSlide(slide, ctx) {
  await chart(slide, ctx, "load_latency_p95.png", 674, 148, 470, 345);
  metric(slide, ctx, 82, 168, "77.2 req/s", "Spin c=10", C.teal, 250);
  metric(slide, ctx, 372, 168, "54.0 req/s", "OCI c=10", C.green, 250);
  metric(slide, ctx, 82, 332, "168 ms", "Spin p95 c=10", C.blue, 250);
  metric(slide, ctx, 372, 332, "211 ms", "OCI p95 c=10", C.amber, 250);
  callout(slide, ctx, "Dla space oba warianty wykonują kosztowne liniowe skanowanie tekstu; przy wysokiej współbieżności rośnie tail latency.", 100, 540, 1000, 58, C.teal);
}

async function loadEmptySlide(slide, ctx) {
  await chart(slide, ctx, "load_throughput.png", 650, 145, 500, 350);
  metric(slide, ctx, 78, 168, "3315 req/s", "OCI c=10", C.green, 260);
  metric(slide, ctx, 378, 168, "133 req/s", "Spin c=10", C.teal, 260);
  metric(slide, ctx, 78, 332, "95.7 ms", "OCI p95 c=200", C.blue, 260);
  metric(slide, ctx, 378, 332, "3636 ms", "Spin p95 c=200", C.red, 260);
  callout(slide, ctx, "Puste zapytanie jest lekkie aplikacyjnie, więc mocno ujawnia narzut ścieżki HTTP/runtime.", 100, 540, 1000, 58, C.amber);
}

async function memorySlide(slide, ctx) {
  await chart(slide, ctx, "memory_peak.png", 704, 158, 430, 316);
  metric(slide, ctx, 92, 168, "31.9 MiB", "OCI load max", C.green, 245);
  metric(slide, ctx, 370, 168, "493.5 MiB", "Spin load max RSS", C.red, 245);
  bulletList(slide, ctx, 115, 360, [
    "OCI: docker stats / cgroup view",
    "Spin: host process tree RSS",
    "Wniosek: porównanie operacyjne, nie identyczna rachunkowość",
  ], 22, 42);
}

function demoSlide(slide, ctx) {
  const steps = [
    "Uruchom Spin :8080 i OCI :8081",
    "Pokaż /health, /version, /stats",
    "Wyślij POST /search dla q=space",
    "Uruchom benchmarks/compare_results.sh",
    "Pokaż CSV i wykresy z pełnego benchmarku",
  ];
  bulletList(slide, ctx, 105, 170, steps, 25, 58);
  ctx.addShape(slide, { x: 740, y: 180, w: 340, h: 260, fill: C.ink, line: ctx.line() });
  ctx.addText(slide, {
    text: "curl /version\ncurl /stats\ncurl -X POST /search\nbenchmarks/compare_results.sh\nls results/plots",
    x: 770,
    y: 218,
    w: 290,
    h: 180,
    fontSize: 21,
    color: "#E5E7EB",
    typeface: ctx.fonts.mono,
  });
  callout(slide, ctx, "Pełny benchmark trwa kilka-kilkanaście minut, więc live demo pokazuje procedurę i dowód parytetu.", 122, 540, 980, 58, C.blue);
}

function conclusionSlide(slide, ctx) {
  const cols = [
    ["Spin/WASM ma sens gdy", ["mały izolowany HTTP component", "edge/serverless/plugin", "ważna przenośność i sandboxing"], C.teal],
    ["OCI pozostaje lepsze gdy", ["potrzebna dojrzałość toolingowa", "storage-heavy service", "precyzyjna rachunkowość zasobów"], C.green],
    ["Nasz wynik", ["parytet funkcjonalny: tak", "wydajność: zależna od ścieżki", "Meilisearch port: feasibility blocker"], C.amber],
  ];
  cols.forEach(([title, bullets, color], i) => {
    const x = 78 + i * 390;
    ctx.addShape(slide, { x, y: 164, w: 330, h: 360, fill: C.white, line: ctx.line(C.line, 1) });
    ctx.addText(slide, { text: title, x: x + 24, y: 194, w: 280, h: 56, fontSize: 22, bold: true, color });
    bulletList(slide, ctx, x + 34, 276, bullets, 19, 52, 250);
  });
  ctx.addText(slide, { text: "Najważniejsze: benchmark jest uczciwy, bo porównuje ten sam kod aplikacyjny.", x: 130, y: 566, w: 940, h: 42, fontSize: 25, bold: true, color: C.ink, align: "center" });
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
