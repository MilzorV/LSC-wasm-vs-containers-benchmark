#!/usr/bin/env python3
"""Download TMDb posters that still resolve on the public image CDN.

No API key required — only HEAD/GET against image.tmdb.org using poster_path
from movies.json. Saves files to frontend/public/posters/{id}.jpg and writes
manifest.json for the UI.

Most Kaggle poster_path values are stale (404). Expect only a fraction to
download successfully unless you refresh paths first (enrich_posters.py --refresh-tmdb).
"""

from __future__ import annotations

import argparse
import json
import sys
import time
import urllib.error
import urllib.request
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
JSON_PATH = ROOT / "fixtures" / "movies.json"
POSTER_DIR = ROOT / "frontend" / "public" / "posters"
MANIFEST_PATH = POSTER_DIR / "manifest.json"
TMDB_SIZE = "w500"


def poster_cdn_url(poster_path: str, size: str = TMDB_SIZE) -> str:
    return f"https://image.tmdb.org/t/p/{size}{poster_path}"


def download_poster(url: str, dest: Path) -> bool:
    try:
        with urllib.request.urlopen(url, timeout=20) as response:
            if response.status != 200:
                return False
            dest.write_bytes(response.read())
        return True
    except urllib.error.HTTPError:
        return False
    except urllib.error.URLError as error:
        print(f"network error {url}: {error}", file=sys.stderr)
        return False


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--limit", type=int, default=None, help="Only process first N movies")
    parser.add_argument("--delay", type=float, default=0.05, help="Seconds between requests")
    args = parser.parse_args()

    movies = json.loads(JSON_PATH.read_text(encoding="utf-8"))
    if args.limit is not None:
        movies = movies[: args.limit]

    POSTER_DIR.mkdir(parents=True, exist_ok=True)
    cached_ids: list[int] = []

    for index, movie in enumerate(movies, start=1):
        path = movie.get("poster_path")
        if not path:
            continue

        dest = POSTER_DIR / f"{movie['id']}.jpg"
        if dest.exists() and dest.stat().st_size > 0:
            cached_ids.append(movie["id"])
            continue

        url = poster_cdn_url(path)
        if download_poster(url, dest):
            cached_ids.append(movie["id"])
        elif dest.exists():
            dest.unlink()

        if index % 200 == 0:
            print(f"processed {index}/{len(movies)}, cached {len(cached_ids)}")

        time.sleep(args.delay)

    MANIFEST_PATH.write_text(json.dumps(sorted(cached_ids)) + "\n", encoding="utf-8")
    print(f"Cached {len(cached_ids)} posters under {POSTER_DIR}")
    print(f"Manifest: {MANIFEST_PATH}")
    print("Run: make frontend-build")


if __name__ == "__main__":
    main()
