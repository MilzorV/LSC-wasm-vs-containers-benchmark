const BENCH_UI = "http://127.0.0.1:8092";
const DASHBOARD_URL = "/benchmark-data/dashboard.json";

const statusEl = document.getElementById("bench-status");
const helperHint = document.getElementById("helper-hint");
const runLog = document.getElementById("run-log");
const metricsSection = document.getElementById("metrics-section");
const plotsSection = document.getElementById("plots-section");
const tablesSection = document.getElementById("tables-section");
const metricCards = document.getElementById("metric-cards");
const plotGrid = document.getElementById("plot-grid");
const summaryTables = document.getElementById("summary-tables");

let pollTimer = null;

function fmt(value, suffix = "") {
  if (value === "" || value === null || value === undefined) {
    return "—";
  }
  const num = Number(value);
  if (Number.isNaN(num)) {
    return String(value);
  }
  return `${num.toLocaleString(undefined, { maximumFractionDigits: 2 })}${suffix}`;
}

function pickMetric(rows, system, metric, field = "p95") {
  const row = rows.find((r) => r.system === system && r.metric === metric);
  return row ? row[field] : "";
}

function renderMetricCards(data) {
  const cold = data.cold || [];
  const load = data.load || [];
  const memory = data.memory || [];

  const cards = [
    {
      system: "spin",
      title: "Spin / WASM",
      rows: [
        ["Cold ready (p95)", pickMetric(cold, "spin", "ready_ms"), " ms"],
        ["Cold path (p95)", pickMetric(cold, "spin", "total_cold_path_ms"), " ms"],
        ["Load latency (p95)", loadRow(load, "spin", "p95"), " ms"],
        ["Throughput", loadRow(load, "spin", "request_rate"), " req/s"],
        ["Memory peak (host RSS)", memoryRow(memory, "spin"), " MiB"],
      ],
    },
    {
      system: "oci",
      title: "OCI / Docker",
      rows: [
        ["Cold ready (p95)", pickMetric(cold, "oci", "ready_ms"), " ms"],
        ["Cold path (p95)", pickMetric(cold, "oci", "total_cold_path_ms"), " ms"],
        ["Load latency (p95)", loadRow(load, "oci", "p95"), " ms"],
        ["Throughput", loadRow(load, "oci", "request_rate"), " req/s"],
        ["Memory peak (host RSS)", memoryRow(memory, "oci"), " MiB"],
      ],
    },
  ];

  metricCards.innerHTML = cards
    .map(
      (card) => `
    <article class="metric-card ${card.system}">
      <h3>${card.title}</h3>
      ${card.rows
        .map(
          ([label, value, suffix]) => `
        <div class="metric-row">
          <span>${label}</span>
          <span>${fmt(value)}${suffix && value !== "" ? suffix : ""}</span>
        </div>`
        )
        .join("")}
    </article>`
    )
    .join("");
  metricsSection.hidden = false;
}

function loadRow(rows, system, field) {
  const candidates = rows.filter((r) => r.system === system && r[field]);
  if (!candidates.length) {
    return "";
  }
  candidates.sort((a, b) => Number(b.concurrency) - Number(a.concurrency));
  return candidates[0][field];
}

function memoryRow(rows, system) {
  const row = rows.find(
    (r) => r.system === system && r.source === "host_process_rss" && r.phase === "idle"
  );
  if (!row || !row.max) {
    const fallback = rows.find((r) => r.system === system && r.source === "host_process_rss");
    if (!fallback?.max) {
      return "";
    }
    return (Number(fallback.max) / (1024 * 1024)).toFixed(1);
  }
  return (Number(row.max) / (1024 * 1024)).toFixed(1);
}

function renderPlots(plots) {
  if (!plots?.length) {
    plotsSection.hidden = true;
    return;
  }
  plotGrid.innerHTML = plots
    .map(
      (plot) => `
    <figure>
      <img src="/benchmark-data/${plot.path}" alt="${plot.title}" loading="lazy" />
      <figcaption>${plot.title}</figcaption>
    </figure>`
    )
    .join("");
  plotsSection.hidden = false;
}

function renderTable(title, rows, columns) {
  if (!rows?.length) {
    return "";
  }
  const head = columns.map((c) => `<th>${c.label}</th>`).join("");
  const body = rows
    .map((row) => {
      const cls = row.system === "spin" ? "spin" : row.system === "oci" ? "oci" : "";
      const cells = columns.map((c) => `<td>${escape(String(row[c.key] ?? ""))}</td>`).join("");
      return `<tr class="${cls}">${cells}</tr>`;
    })
    .join("");
  return `
    <div class="summary-table-wrap">
      <h3>${title}</h3>
      <table class="summary-table">
        <thead><tr>${head}</tr></thead>
        <tbody>${body}</tbody>
      </table>
    </div>`;
}

function renderTables(data) {
  summaryTables.innerHTML =
    renderTable("Cold start", data.cold, [
      { key: "system", label: "System" },
      { key: "metric", label: "Metric" },
      { key: "p50", label: "p50" },
      { key: "p95", label: "p95" },
      { key: "mean", label: "mean" },
    ]) +
    renderTable("Load", data.load, [
      { key: "system", label: "System" },
      { key: "query", label: "Query" },
      { key: "concurrency", label: "C" },
      { key: "p95", label: "p95 ms" },
      { key: "request_rate", label: "req/s" },
    ]) +
    renderTable("Memory (host RSS)", (data.memory || []).filter((r) => r.source === "host_process_rss"), [
      { key: "system", label: "System" },
      { key: "phase", label: "Phase" },
      { key: "max", label: "max bytes" },
      { key: "mean", label: "mean" },
    ]);
  tablesSection.hidden = false;
}

async function loadDashboard() {
  statusEl.textContent = "Loading results…";
  try {
    const response = await fetch(DASHBOARD_URL);
    if (!response.ok) {
      throw new Error(`${response.status} — run make analyze && make frontend-build`);
    }
    const data = await response.json();
    statusEl.textContent = data.generatedAt
      ? `Last analyzed: ${new Date(data.generatedAt).toLocaleString()}`
      : "Benchmark summaries loaded";
    renderMetricCards(data);
    renderPlots(data.plots);
    renderTables(data);
  } catch (error) {
    statusEl.textContent = `No dashboard data: ${error.message}`;
    metricsSection.hidden = true;
    plotsSection.hidden = true;
    tablesSection.hidden = true;
  }
}

async function benchUiFetch(path, options) {
  const response = await fetch(`${BENCH_UI}${path}`, options);
  if (!response.ok) {
    const text = await response.text();
    throw new Error(text || `${response.status}`);
  }
  return response.json();
}

async function checkHelper() {
  try {
    await benchUiFetch("/status");
    helperHint.hidden = true;
    return true;
  } catch {
    helperHint.hidden = false;
    return false;
  }
}

async function startRun(profile) {
  if (!(await checkHelper())) {
    return;
  }
  runLog.hidden = false;
  runLog.textContent = `Starting ${profile} run…\n`;
  document.getElementById("run-pilot").disabled = true;
  document.getElementById("run-full").disabled = true;

  try {
    await benchUiFetch("/run", {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ profile }),
    });
    pollStatus();
  } catch (error) {
    runLog.textContent += `Error: ${error.message}\n`;
    document.getElementById("run-pilot").disabled = false;
    document.getElementById("run-full").disabled = false;
  }
}

function pollStatus() {
  if (pollTimer) {
    clearInterval(pollTimer);
  }
  pollTimer = setInterval(async () => {
    try {
      const status = await benchUiFetch("/status");
      runLog.textContent = status.logTail || "(no log yet)";
      if (!status.running) {
        clearInterval(pollTimer);
        pollTimer = null;
        document.getElementById("run-pilot").disabled = false;
        document.getElementById("run-full").disabled = false;
        if (status.exitCode === 0) {
          runLog.textContent += "\nDone. Refreshing dashboard…";
          await loadDashboard();
        } else {
          runLog.textContent += `\nFailed (exit ${status.exitCode}).`;
        }
      }
    } catch (error) {
      runLog.textContent += `\nPoll error: ${error.message}`;
      clearInterval(pollTimer);
      pollTimer = null;
      document.getElementById("run-pilot").disabled = false;
      document.getElementById("run-full").disabled = false;
    }
  }, 2000);
}

document.getElementById("run-pilot").addEventListener("click", () => startRun("pilot"));
document.getElementById("run-full").addEventListener("click", () => startRun("full"));
document.getElementById("refresh-data").addEventListener("click", () => loadDashboard());

checkHelper();
loadDashboard();
