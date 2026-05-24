# Fixture Data

`movies.json` is the canonical benchmark fixture for both Spin and OCI.

- Documents: `44,471`
- Primary key: `id`
- Fields: `id`, `title`, `overview`, `genre`, `year`
- Source data: `movies_metadata.csv`
- Deduplication rule: last record wins for duplicate `id` values

`movies_metadata.csv` is retained as the raw source export. Runtime services should load `movies.json`, not the CSV.
