import { getStats, getVersion, listMovies, searchMovies } from "./api.js";
import { bindMovieCards, formatRange, renderMovieList } from "./ui.js";

const searchForm = document.getElementById("search-form");
const searchInput = document.getElementById("search-input");
const searchResults = document.getElementById("search-results");
const searchMeta = document.getElementById("search-meta");
const searchPager = document.getElementById("search-pager");
const searchPrev = document.getElementById("search-prev");
const searchNext = document.getElementById("search-next");
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
    const [{ data: stats }, { data: version }] = await Promise.all([getStats(), getVersion()]);
    statusLine.textContent = `${stats.documentCount.toLocaleString()} movies in catalog`;
    engineLine.textContent = `${version.engine} · dataset ${version.datasetDocuments.toLocaleString()}`;
  } catch (error) {
    statusLine.textContent = `Backend unavailable: ${error.message}`;
  }
}

async function runSearch(offset = 0) {
  const q = searchInput.value.trim();
  searchResults.innerHTML = `<p class="placeholder">Searching…</p>`;
  searchMeta.hidden = true;
  searchPager.hidden = true;

  try {
    const { data, elapsed } = await searchMovies(q, offset);
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
    searchMeta.textContent = `${formatRange(data.offset, data.hits.length, data.estimatedTotalHits)} · ${elapsed} ms · page size ${data.limit}`;
    searchMeta.hidden = false;
    searchPager.hidden = false;
    searchPrev.disabled = data.offset <= 0;
    searchNext.disabled = data.offset + data.hits.length >= data.estimatedTotalHits;
  } catch (error) {
    searchResults.innerHTML = `<p class="placeholder error">${error.message}</p>`;
  }
}

async function loadBrowse(offset = 0) {
  browseResults.innerHTML = `<p class="placeholder">Loading…</p>`;
  try {
    const { data, elapsed } = await listMovies(offset);
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

loadStatus();
loadBrowse(0);
