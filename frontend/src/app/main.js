import { createApiClient } from "../api.js";
import { bindMovieCards, formatRange, loadCachedPosterIds, renderMovieList } from "../ui.js";

const body = document.body;
const API_BASE = body.dataset.apiBase ?? "";
const RUNTIME_LABEL = body.dataset.runtime ?? "Movie Search";
const api = createApiClient(API_BASE);

const searchForm = document.getElementById("search-form");
const searchInput = document.getElementById("search-input");
const searchResults = document.getElementById("search-results");
const searchMeta = document.getElementById("search-meta");
const searchPager = document.getElementById("search-pager");
const searchPrev = document.getElementById("search-prev");
const searchNext = document.getElementById("search-next");
const suggestionsRoot = document.createElement("div");
suggestionsRoot.id = "suggestions";
suggestionsRoot.className = "suggestions";
const browseResults = document.getElementById("browse-results");
const browseMeta = document.getElementById("browse-meta");
const browsePrev = document.getElementById("browse-prev");
const browseNext = document.getElementById("browse-next");
const statusLine = document.getElementById("status-line");
const engineLine = document.getElementById("engine-line");
const dialog = document.getElementById("movie-dialog");
const detailRoot = document.getElementById("movie-detail");

let searchState = { q: "", offset: 0, pageSize: 20, total: 0 };
let browseState = { offset: 0, pageSize: 20, total: 0 };
let suggestTimer = null;

const enhancedControls = document.createElement("section");
enhancedControls.className = "enhanced-controls";
enhancedControls.innerHTML = `
  <label>
    Genre
    <input id="filter-genre" type="text" placeholder="Science Fiction, Drama" />
  </label>
  <label>
    Year from
    <input id="filter-year-min" type="number" inputmode="numeric" placeholder="1970" />
  </label>
  <label>
    Year to
    <input id="filter-year-max" type="number" inputmode="numeric" placeholder="2026" />
  </label>
  <label>
    Sort
    <select id="sort-order">
      <option value="">Relevance</option>
      <option value="year:desc">Newest first</option>
      <option value="year:asc">Oldest first</option>
      <option value="title:asc">Title A-Z</option>
      <option value="title:desc">Title Z-A</option>
      <option value="id:asc">ID ascending</option>
      <option value="id:desc">ID descending</option>
    </select>
  </label>
  <label class="check-option">
    <input id="typo-toggle" type="checkbox" />
    Typo tolerance
  </label>
  <label class="check-option">
    <input id="highlight-toggle" type="checkbox" />
    Highlights
  </label>
  <label class="check-option">
    <input id="facets-toggle" type="checkbox" />
    Facets
  </label>
`;
searchForm.after(suggestionsRoot);
suggestionsRoot.after(enhancedControls);

document.querySelectorAll(".nav-tab").forEach((tab) => {
  tab.addEventListener("click", () => {
    document.querySelectorAll(".nav-tab").forEach((item) => item.classList.remove("active"));
    document.querySelectorAll(".view").forEach((view) => view.classList.remove("active"));
    tab.classList.add("active");
    document.getElementById(`view-${tab.dataset.view}`).classList.add("active");
    if (tab.dataset.view === "browse" && browseState.total === 0) {
      loadBrowse(0);
    }
  });
});

bindMovieCards(searchResults, dialog, detailRoot);
bindMovieCards(browseResults, dialog, detailRoot);

async function loadStatus() {
  try {
    const [{ data: stats }, { data: version }] = await Promise.all([api.getStats(), api.getVersion()]);
    statusLine.textContent = `${stats.documentCount.toLocaleString()} movies in catalog`;
    engineLine.textContent = `${RUNTIME_LABEL} · ${version.engine} · dataset ${version.datasetDocuments.toLocaleString()}`;
  } catch (error) {
    statusLine.textContent = `Backend unavailable: ${error.message}`;
    engineLine.textContent = RUNTIME_LABEL;
  }
}

async function runSearch(offset = 0) {
  const q = searchInput.value.trim();
  searchResults.innerHTML = `<p class="placeholder">Searching…</p>`;
  searchMeta.hidden = true;
  searchPager.hidden = true;

  try {
    const options = collectSearchOptions();
    const { data, elapsed } = await api.searchMovies(q, offset, options);
    searchState = {
      q,
      offset: data.offset,
      pageSize: data.limit,
      total: data.estimatedTotalHits,
    };
    renderMovieList(
      searchResults,
      data.hits,
      q ? "No movies matched your search." : "No movies found."
    );
    searchMeta.innerHTML = [
      `${formatRange(data.offset, data.hits.length, data.estimatedTotalHits)} · ${elapsed} ms · page size ${data.limit}`,
      renderFacetSummary(data),
      data.rankingInfo ? "ranking debug on" : "",
    ]
      .filter(Boolean)
      .join(" · ");
    searchMeta.hidden = false;
    searchPager.hidden = false;
    searchPrev.disabled = data.offset <= 0;
    searchNext.disabled = data.offset + data.hits.length >= data.estimatedTotalHits;
  } catch (error) {
    searchResults.innerHTML = `<p class="placeholder error">${error.message}</p>`;
  }
}

function collectSearchOptions() {
  const options = {};
  const genre = document
    .getElementById("filter-genre")
    .value.split(",")
    .map((value) => value.trim())
    .filter(Boolean);
  const min = parseYearInput("filter-year-min");
  const max = parseYearInput("filter-year-max");
  if (genre.length || min !== null || max !== null) {
    options.filter = {};
    if (genre.length) {
      options.filter.genre = genre;
    }
    if (min !== null || max !== null) {
      options.filter.year = {};
      if (min !== null) {
        options.filter.year.gte = min;
      }
      if (max !== null) {
        options.filter.year.lte = max;
      }
    }
  }
  const sort = document.getElementById("sort-order").value;
  if (sort) {
    options.sort = [sort];
  }
  if (document.getElementById("typo-toggle").checked) {
    options.typoTolerance = true;
  }
  if (document.getElementById("highlight-toggle").checked) {
    options.highlight = ["title", "genre", "overview"];
  }
  if (document.getElementById("facets-toggle").checked) {
    options.facets = ["genre", "year"];
  }
  return options;
}

function parseYearInput(id) {
  const value = document.getElementById(id).value.trim();
  return value ? Number.parseInt(value, 10) : null;
}

function renderFacetSummary(data) {
  const genreFacet = data.facetDistribution?.genre;
  const yearStats = data.facetStats?.year;
  const parts = [];
  if (genreFacet) {
    const topGenres = Object.entries(genreFacet)
      .sort((left, right) => right[1] - left[1])
      .slice(0, 3)
      .map(([genre, count]) => `${genre}: ${count}`);
    parts.push(`top genres ${topGenres.join(", ")}`);
  }
  if (yearStats) {
    parts.push(`years ${yearStats.min}-${yearStats.max}`);
  }
  return parts.join(" · ");
}

function scheduleSuggestions() {
  window.clearTimeout(suggestTimer);
  const q = searchInput.value.trim();
  if (q.length < 2) {
    suggestionsRoot.innerHTML = "";
    return;
  }
  suggestTimer = window.setTimeout(async () => {
    try {
      const { data } = await api.suggestMovies(q, {
        limit: 5,
        filter: collectSearchOptions().filter,
      });
      renderSuggestions(data.suggestions || []);
    } catch {
      suggestionsRoot.innerHTML = "";
    }
  }, 160);
}

function renderSuggestions(suggestions) {
  if (!suggestions.length) {
    suggestionsRoot.innerHTML = "";
    return;
  }
  suggestionsRoot.innerHTML = suggestions
    .map(
      (suggestion) =>
        `<button type="button" class="suggestion-chip">${suggestion.replaceAll("&", "&amp;").replaceAll("<", "&lt;").replaceAll(">", "&gt;")}</button>`
    )
    .join("");
}

async function loadBrowse(offset = 0) {
  browseResults.innerHTML = `<p class="placeholder">Loading…</p>`;
  try {
    const { data, elapsed } = await api.listMovies(offset);
    browseState = {
      offset: data.offset,
      pageSize: data.limit,
      total: data.total,
    };
    renderMovieList(browseResults, data.results, "No movies in catalog.");
    browseMeta.textContent = `${formatRange(data.offset, data.results.length, data.total)} · ${elapsed} ms`;
    browsePrev.disabled = data.offset <= 0;
    browseNext.disabled = data.offset + data.results.length >= data.total;
  } catch (error) {
    browseResults.innerHTML = `<p class="placeholder error">${error.message}</p>`;
    browseMeta.textContent = "";
  }
}

searchForm.addEventListener("submit", (event) => {
  event.preventDefault();
  runSearch(0);
});

searchInput.addEventListener("input", scheduleSuggestions);

suggestionsRoot.addEventListener("click", (event) => {
  const button = event.target.closest(".suggestion-chip");
  if (!button) {
    return;
  }
  searchInput.value = button.textContent;
  suggestionsRoot.innerHTML = "";
  runSearch(0);
});

searchPrev.addEventListener("click", () => {
  runSearch(Math.max(0, searchState.offset - searchState.pageSize));
});

searchNext.addEventListener("click", () => {
  runSearch(searchState.offset + searchState.pageSize);
});

browsePrev.addEventListener("click", () => {
  loadBrowse(Math.max(0, browseState.offset - browseState.pageSize));
});

browseNext.addEventListener("click", () => {
  loadBrowse(browseState.offset + browseState.pageSize);
});

loadCachedPosterIds().then(loadStatus);
