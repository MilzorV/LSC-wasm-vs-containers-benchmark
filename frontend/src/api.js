async function fetchJson(path, options) {
  const started = performance.now();
  const response = await fetch(path, options);
  const elapsed = Math.round(performance.now() - started);
  if (!response.ok) {
    const text = await response.text();
    throw new Error(`${response.status} ${text}`);
  }
  const data = await response.json();
  return { data, elapsed };
}

export async function getHealth() {
  return fetchJson("/health");
}

export async function getVersion() {
  return fetchJson("/version");
}

export async function getStats() {
  return fetchJson("/stats");
}

export async function searchMovies(query, offset = 0) {
  const body = { q: query };
  if (offset > 0) {
    body.offset = offset;
  }
  return fetchJson("/search", {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify(body),
  });
}

export async function listMovies(offset = 0) {
  const params = new URLSearchParams();
  if (offset > 0) {
    params.set("offset", String(offset));
  }
  const query = params.toString();
  return fetchJson(query ? `/movies?${query}` : "/movies");
}

export async function remoteSearch(baseUrl, query) {
  const url = baseUrl.replace(/\/$/, "");
  const started = performance.now();
  const response = await fetch(`${url}/search`, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ q: query }),
  });
  const elapsed = Math.round(performance.now() - started);
  if (!response.ok) {
    const text = await response.text();
    throw new Error(`${response.status} ${text}`);
  }
  const data = await response.json();
  return { data, elapsed };
}
