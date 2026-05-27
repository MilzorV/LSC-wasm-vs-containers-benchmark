# Fixture Data

`movies.json` is the canonical benchmark fixture for both Spin and OCI.

- Documents: `44,471`
- Primary key: `id`
- Fields: `id`, `title`, `overview`, `genre`, `year`, `poster_path` (optional, from TMDb)
- Source data: `movies_metadata.csv`
- Deduplication rule: last record wins for duplicate `id` values

`movies_metadata.csv` is retained as the raw source export. Runtime services should load `movies.json`, not the CSV.

### Posters (no API key for images)

The UI builds public image URLs from `poster_path`:

```text
https://image.tmdb.org/t/p/w342<poster_path>
```

No auth is required to **display** images on the TMDb CDN. API keys are only needed if you want to **look up** fresh metadata.

Copy paths from the CSV:

```bash
python3 fixtures/enrich_posters.py
```

**Note:** many Kaggle `poster_path` values are outdated (~90% return 404 today). The app falls back to a placeholder when the CDN misses.

Optional — cache posters that still work on the CDN (no API key):

```bash
python3 fixtures/download_posters.py --limit 1000
make frontend-build
```

Optional — refresh stale paths from the TMDb API, then download:

```bash
export TMDB_API_KEY=your_key_here
python3 fixtures/enrich_posters.py --refresh-tmdb
python3 fixtures/download_posters.py
make frontend-build
```

Rebuild backends after changing `movies.json`: `make build` or `docker compose build`.
