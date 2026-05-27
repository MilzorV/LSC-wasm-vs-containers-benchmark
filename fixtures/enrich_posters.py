#!/usr/bin/env python3
"""Manage TMDb poster_path values in movies.json.

By default, copies poster_path from movies_metadata.csv (same as the Kaggle export).

Many CSV paths are stale and return 404 from image.tmdb.org. Use --refresh-tmdb with a
free API key from https://www.themoviedb.org/settings/api to fetch current paths.
"""

from __future__ import annotations

import argparse
import csv
import json
import os
import sys
import time
import urllib.error
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parent
CSV_PATH = ROOT / "movies_metadata.csv"
JSON_PATH = ROOT / "movies.json"
TMDB_MOVIE_URL = "https://api.themoviedb.org/3/movie/{movie_id}"
REQUEST_INTERVAL_S = 0.26  # ~4 req/s, under free-tier limits


def load_poster_paths_from_csv() -> dict[int, str | None]:
    posters: dict[int, str | None] = {}
    with CSV_PATH.open(newline="", encoding="utf-8") as handle:
        for row in csv.DictReader(handle):
            raw_id = (row.get("id") or "").strip()
            if not raw_id:
                continue
            try:
                movie_id = int(float(raw_id))
            except ValueError:
                continue
            poster = (row.get("poster_path") or "").strip()
            if poster in ("", "nan", "None", "null"):
                poster = None
            posters[movie_id] = poster
    return posters


def fetch_tmdb_poster_path(movie_id: int, api_key: str) -> str | None:
    url = TMDB_MOVIE_URL.format(movie_id=movie_id)
    request = urllib.request.Request(
        f"{url}?api_key={api_key}",
        headers={"accept": "application/json"},
    )
    with urllib.request.urlopen(request, timeout=20) as response:
        payload = json.load(response)
    poster = (payload.get("poster_path") or "").strip()
    return poster or None


def apply_csv_posters(movies: list[dict]) -> int:
    posters = load_poster_paths_from_csv()
    with_poster = 0
    for movie in movies:
        path = posters.get(movie["id"])
        if path:
            movie["poster_path"] = path
            with_poster += 1
        else:
            movie.pop("poster_path", None)
    return with_poster


def refresh_from_tmdb(movies: list[dict], api_key: str, limit: int | None) -> int:
    targets = movies if limit is None else movies[:limit]
    with_poster = 0
    failures = 0

    for index, movie in enumerate(targets, start=1):
        movie_id = movie["id"]
        try:
            path = fetch_tmdb_poster_path(movie_id, api_key)
        except urllib.error.HTTPError as error:
            failures += 1
            if error.code == 404:
                movie.pop("poster_path", None)
            else:
                print(f"HTTP {error.code} for movie {movie_id}", file=sys.stderr)
            time.sleep(REQUEST_INTERVAL_S)
            continue
        except urllib.error.URLError as error:
            print(f"network error for movie {movie_id}: {error}", file=sys.stderr)
            failures += 1
            time.sleep(REQUEST_INTERVAL_S)
            continue

        if path:
            movie["poster_path"] = path
            with_poster += 1
        else:
            movie.pop("poster_path", None)

        if index % 100 == 0:
            print(f"refreshed {index}/{len(targets)} ({with_poster} with posters, {failures} errors)")

        time.sleep(REQUEST_INTERVAL_S)

    return with_poster


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--refresh-tmdb",
        action="store_true",
        help="Fetch current poster_path from TMDb API (requires TMDB_API_KEY)",
    )
    parser.add_argument(
        "--limit",
        type=int,
        default=None,
        help="Only refresh the first N movies (for testing)",
    )
    args = parser.parse_args()

    movies = json.loads(JSON_PATH.read_text(encoding="utf-8"))

    if args.refresh_tmdb:
        api_key = os.environ.get("TMDB_API_KEY", "").strip()
        if not api_key:
            print("Set TMDB_API_KEY to refresh posters from TMDb.", file=sys.stderr)
            sys.exit(1)
        with_poster = refresh_from_tmdb(movies, api_key, args.limit)
        mode = "refreshed from TMDb"
    else:
        with_poster = apply_csv_posters(movies)
        mode = "copied from CSV"

    JSON_PATH.write_text(
        json.dumps(movies, indent=2, ensure_ascii=False) + "\n",
        encoding="utf-8",
    )
    print(f"Updated {len(movies)} movies ({with_poster} with poster_path, {mode})")


if __name__ == "__main__":
    main()
