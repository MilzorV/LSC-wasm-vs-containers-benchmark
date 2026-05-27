function resolveBase(baseUrl) {
  const raw = baseUrl?.trim() || window.location.origin;
  return raw.replace(/\/$/, "");
}

async function fetchJson(baseUrl, path, options) {
  const started = performance.now();
  const response = await fetch(`${resolveBase(baseUrl)}${path}`, options);
  const elapsed = Math.round(performance.now() - started);
  if (!response.ok) {
    const text = await response.text();
    throw new Error(`${response.status} ${text}`);
  }
  const data = await response.json();
  return { data, elapsed };
}

export function createApiClient(baseUrl = "") {
  return {
    getHealth: () => fetchJson(baseUrl, "/health"),
    getVersion: () => fetchJson(baseUrl, "/version"),
    getStats: () => fetchJson(baseUrl, "/stats"),
    searchMovies: (query, offset = 0) => {
      const body = { q: query };
      if (offset > 0) {
        body.offset = offset;
      }
      return fetchJson(baseUrl, "/search", {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify(body),
      });
    },
    listMovies: (offset = 0) => {
      const params = new URLSearchParams();
      if (offset > 0) {
        params.set("offset", String(offset));
      }
      const query = params.toString();
      return fetchJson(baseUrl, query ? `/movies?${query}` : "/movies");
    },
  };
}

export async function getHealth(baseUrl = "") {
  return createApiClient(baseUrl).getHealth();
}

export async function getVersion(baseUrl = "") {
  return createApiClient(baseUrl).getVersion();
}

export async function getStats(baseUrl = "") {
  return createApiClient(baseUrl).getStats();
}

export async function searchMovies(query, offset = 0, baseUrl = "") {
  return createApiClient(baseUrl).searchMovies(query, offset);
}

export async function listMovies(offset = 0, baseUrl = "") {
  return createApiClient(baseUrl).listMovies(offset);
}

export async function remoteSearch(baseUrl, query) {
  const url = resolveBase(baseUrl);
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
