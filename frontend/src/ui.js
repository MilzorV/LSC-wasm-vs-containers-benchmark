export function escapeHtml(value) {
  return String(value ?? "")
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;");
}

const TMDB_POSTER_BASE = "https://image.tmdb.org/t/p/w342";

/** Movie ids with a file under /posters/{id}.jpg (see fixtures/download_posters.py). */
let cachedPosterIds = null;

export async function loadCachedPosterIds() {
  if (cachedPosterIds !== null) {
    return cachedPosterIds;
  }
  try {
    const response = await fetch("/posters/manifest.json");
    cachedPosterIds = response.ok ? new Set(await response.json()) : new Set();
  } catch {
    cachedPosterIds = new Set();
  }
  return cachedPosterIds;
}

export function posterUrl(movie, size = "w342") {
  const path = movie?.poster_path;
  if (!path) {
    return null;
  }
  const base =
    size === "w342" ? TMDB_POSTER_BASE : `https://image.tmdb.org/t/p/${size}`;
  return `${base}${path}`;
}

function localPosterUrl(movieId) {
  return `/posters/${movieId}.jpg`;
}

function posterSrc(movie, large = false) {
  if (cachedPosterIds?.has(movie.id)) {
    return localPosterUrl(movie.id);
  }
  return posterUrl(movie, large ? "w500" : "w342");
}

function posterMarkup(movie, { large = false } = {}) {
  const cdnUrl = posterUrl(movie, large ? "w500" : "w342");
  const src = posterSrc(movie, large);
  const wrapClass = large ? "poster-wrap poster-wrap--large" : "poster-wrap";
  if (!src) {
    return `<div class="${wrapClass}"><div class="poster-fallback" aria-hidden="true">MS</div></div>`;
  }
  const alt = escapeHtml(movie.title);
  const cdnAttr = cdnUrl && cdnUrl !== src ? ` data-cdn="${escapeHtml(cdnUrl)}"` : "";
  return `<div class="${wrapClass}"><img class="poster" src="${escapeHtml(src)}"${cdnAttr} alt="${alt} poster" loading="lazy" decoding="async" onerror="if(this.dataset.cdn&&!this.dataset.triedCdn){this.dataset.triedCdn=1;this.src=this.dataset.cdn;return}this.hidden=true;this.nextElementSibling.hidden=false;this.nextElementSibling.removeAttribute('aria-hidden')" /><div class="poster-fallback" hidden aria-hidden="true">MS</div></div>`;
}

export function movieCard(movie) {
  const year = movie.year ? ` · ${movie.year}` : "";
  const formatted = movie._formatted || {};
  const title = formatted.title || escapeHtml(movie.title);
  const genre = formatted.genre || escapeHtml(movie.genre);
  const overviewValue = formatted.overview || escapeHtml(movie.overview);
  const overview = movie.overview
    ? `<p class="movie-overview">${overviewValue}</p>`
    : "";
  const encoded = encodeURIComponent(JSON.stringify(movie));
  return `
    <article class="movie-card" data-movie="${encoded}" tabindex="0">
      ${posterMarkup(movie)}
      <div class="movie-body">
        <h3>${title}</h3>
        <p class="movie-meta">${genre}${year} · #${movie.id}</p>
        ${overview}
      </div>
    </article>`;
}

export function renderMovieList(container, movies, emptyMessage) {
  if (!movies.length) {
    container.innerHTML = `<p class="placeholder">${escapeHtml(emptyMessage)}</p>`;
    return;
  }
  container.innerHTML = movies.map(movieCard).join("");
}

export function formatRange(offset, count, total) {
  if (!total) {
    return "0 results";
  }
  const from = offset + 1;
  const to = Math.min(offset + count, total);
  return `${from}–${to} of ${total.toLocaleString()}`;
}

export function openMovieDetail(dialog, detailRoot, movie) {
  const year = movie.year ? `<li>Year: ${escapeHtml(movie.year)}</li>` : "";
  detailRoot.innerHTML = `
    <div class="movie-detail">
      ${posterMarkup(movie, { large: true })}
      <div class="movie-detail-body">
        <h2>${escapeHtml(movie.title)}</h2>
        <ul class="detail-meta">
          <li>ID: ${movie.id}</li>
          <li>Genre: ${escapeHtml(movie.genre)}</li>
          ${year}
        </ul>
        <p>${escapeHtml(movie.overview || "No overview available.")}</p>
      </div>
    </div>
  `;
  dialog.showModal();
}

export function bindMovieCards(container, dialog, detailRoot) {
  container.addEventListener("click", (event) => {
    const card = event.target.closest(".movie-card");
    if (!card) {
      return;
    }
    openMovieDetail(dialog, detailRoot, JSON.parse(decodeURIComponent(card.dataset.movie)));
  });
  container.addEventListener("keydown", (event) => {
    if (event.key !== "Enter") {
      return;
    }
    const card = event.target.closest(".movie-card");
    if (!card) {
      return;
    }
    openMovieDetail(dialog, detailRoot, JSON.parse(decodeURIComponent(card.dataset.movie)));
  });
}
