import { remoteSearch } from "../api.js";
import { escapeHtml, movieCard } from "../ui.js";

const params = new URLSearchParams(location.search);
const SPIN_URL = params.get("spin") || "http://127.0.0.1:8080";
const OCI_URL = params.get("oci") || "http://127.0.0.1:8081";

document.getElementById("spin-label").textContent = SPIN_URL.replace(/^https?:\/\//, "");
document.getElementById("oci-label").textContent = OCI_URL.replace(/^https?:\/\//, "");

function hitIds(payload) {
  if (!payload || payload.error) {
    return null;
  }
  return (payload.data.hits || []).map((hit) => hit.id);
}

function updateParity(spinPayload, ociPayload) {
  const el = document.getElementById("parity");
  const spinIds = hitIds(spinPayload);
  const ociIds = hitIds(ociPayload);
  if (spinIds === null || ociIds === null) {
    el.className = "parity mismatch";
    el.textContent = "Could not compare — one or both backends failed";
    return;
  }
  const match =
    spinIds.length === ociIds.length && spinIds.every((id, index) => id === ociIds[index]);
  el.className = match ? "parity match" : "parity mismatch";
  el.textContent = match
    ? `Match — same ranking (${spinIds.length} hits on page)`
    : "Mismatch — hit order or count differs";
}

function renderCompare(container, payload) {
  if (payload.error) {
    const hint =
      payload.error.includes("Failed to fetch") || payload.error.includes("NetworkError")
        ? "<br><small>Is the backend running? Spin: <code>spin up</code> · OCI: <code>docker compose up</code></small>"
        : "";
    container.innerHTML = `<p class="placeholder error">${escapeHtml(payload.error)}${hint}</p>`;
    return;
  }
  const hits = payload.data.hits || [];
  const cards = hits.map((hit) => movieCard(hit)).join("");
  container.innerHTML =
    `<p class="results-meta">${hits.length} of ${payload.data.estimatedTotalHits} · ${payload.elapsed} ms</p>` +
    (cards || `<p class="placeholder">No hits</p>`);
}

async function compareSearch(query) {
  const parity = document.getElementById("parity");
  parity.className = "parity pending";
  parity.textContent = "Searching…";

  document.getElementById("spin-results").innerHTML = `<p class="placeholder">Searching…</p>`;
  document.getElementById("oci-results").innerHTML = `<p class="placeholder">Searching…</p>`;

  const [spinPayload, ociPayload] = await Promise.all([
    remoteSearch(SPIN_URL, query).catch((error) => ({ error: error.message })),
    remoteSearch(OCI_URL, query).catch((error) => ({ error: error.message })),
  ]);
  renderCompare(document.getElementById("spin-results"), spinPayload);
  renderCompare(document.getElementById("oci-results"), ociPayload);
  updateParity(spinPayload, ociPayload);
}

document.getElementById("compare-form").addEventListener("submit", (event) => {
  event.preventDefault();
  const query = document.getElementById("compare-query").value.trim();
  if (!query) {
    document.getElementById("parity").className = "parity pending";
    document.getElementById("parity").textContent = "Enter a query to search";
    return;
  }
  compareSearch(query);
});
