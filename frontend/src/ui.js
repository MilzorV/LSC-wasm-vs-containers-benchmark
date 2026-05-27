export function escapeHtml(value) {
  return String(value ?? "")
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;");
}

export function movieCard(movie) {
  const year = movie.year ? ` · ${movie.year}` : "";
  const overview = movie.overview
    ? `<p class="movie-overview">${escapeHtml(movie.overview)}</p>`
    : "";
  const encoded = encodeURIComponent(JSON.stringify(movie));
  return `
    <article class="movie-card" data-movie="${encoded}" tabindex="0">
      <div class="poster" aria-hidden="true">MS</div>
      <div class="movie-body">
        <h3>${escapeHtml(movie.title)}</h3>
        <p class="movie-meta">${escapeHtml(movie.genre)}${year} · #${movie.id}</p>
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
    <h2>${escapeHtml(movie.title)}</h2>
    <ul class="detail-meta">
      <li>ID: ${movie.id}</li>
      <li>Genre: ${escapeHtml(movie.genre)}</li>
      ${year}
    </ul>
    <p>${escapeHtml(movie.overview || "No overview available.")}</p>
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
