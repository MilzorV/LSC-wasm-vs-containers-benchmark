use std::collections::{BTreeMap, BTreeSet};
use std::time::Instant;

use serde::{Deserialize, Serialize};

pub const ENGINE_NAME: &str = "movie-search-core";
pub const DEFAULT_LIMIT: usize = 20;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Movie {
    pub id: u64,
    pub title: String,
    pub overview: String,
    pub genre: String,
    pub year: Option<i32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub poster_path: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchRequest {
    #[serde(default)]
    pub q: Option<String>,
    #[serde(default)]
    pub offset: Option<usize>,
    #[serde(default)]
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SearchResponse {
    pub hits: Vec<Movie>,
    pub query: String,
    pub offset: usize,
    pub limit: usize,
    pub estimated_total_hits: usize,
    pub processing_time_ms: u128,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MoviesResponse {
    pub results: Vec<Movie>,
    pub offset: usize,
    pub limit: usize,
    pub total: usize,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct StatsResponse {
    pub document_count: usize,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct HealthResponse {
    pub status: &'static str,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct VersionResponse {
    pub engine: &'static str,
    pub pkg_version: &'static str,
    pub dataset_documents: usize,
}

#[derive(Debug, Clone)]
pub struct MovieSearch {
    movies: Vec<Movie>,
}

impl MovieSearch {
    pub fn from_json(input: &str) -> Result<Self, serde_json::Error> {
        let movies = serde_json::from_str::<Vec<Movie>>(input)?;
        Ok(Self::from_movies(movies))
    }

    pub fn from_movies(movies: Vec<Movie>) -> Self {
        let mut by_id = BTreeMap::new();
        for movie in movies {
            by_id.insert(movie.id, movie);
        }

        Self {
            movies: by_id.into_values().collect(),
        }
    }

    pub fn document_count(&self) -> usize {
        self.movies.len()
    }

    pub fn health(&self) -> HealthResponse {
        HealthResponse {
            status: "available",
        }
    }

    pub fn version(&self) -> VersionResponse {
        VersionResponse {
            engine: ENGINE_NAME,
            pkg_version: env!("CARGO_PKG_VERSION"),
            dataset_documents: self.document_count(),
        }
    }

    pub fn stats(&self) -> StatsResponse {
        StatsResponse {
            document_count: self.document_count(),
        }
    }

    pub fn movies(&self, offset: usize, limit: usize) -> MoviesResponse {
        MoviesResponse {
            results: self
                .movies
                .iter()
                .skip(offset)
                .take(limit)
                .cloned()
                .collect(),
            offset,
            limit,
            total: self.movies.len(),
        }
    }

    pub fn search(&self, request: SearchRequest) -> SearchResponse {
        let started = Instant::now();
        let query = request.q.unwrap_or_default();
        let offset = request.offset.unwrap_or(0);
        let limit = request.limit.unwrap_or(DEFAULT_LIMIT);
        let tokens = tokenize(&query);

        let mut matches = if tokens.is_empty() {
            self.movies
                .iter()
                .map(|movie| RankedMovie {
                    movie,
                    matched_tokens: 0,
                    field_weight: 0,
                })
                .collect::<Vec<_>>()
        } else {
            self.movies
                .iter()
                .filter_map(|movie| rank_movie(movie, &tokens))
                .collect::<Vec<_>>()
        };

        matches.sort_by(|left, right| {
            right
                .matched_tokens
                .cmp(&left.matched_tokens)
                .then_with(|| right.field_weight.cmp(&left.field_weight))
                .then_with(|| left.movie.id.cmp(&right.movie.id))
        });

        let estimated_total_hits = matches.len();
        let hits = matches
            .into_iter()
            .skip(offset)
            .take(limit)
            .map(|ranked| ranked.movie.clone())
            .collect();

        SearchResponse {
            hits,
            query,
            offset,
            limit,
            estimated_total_hits,
            processing_time_ms: started.elapsed().as_millis(),
        }
    }
}

struct RankedMovie<'movie> {
    movie: &'movie Movie,
    matched_tokens: usize,
    field_weight: usize,
}

fn rank_movie<'movie>(
    movie: &'movie Movie,
    tokens: &BTreeSet<String>,
) -> Option<RankedMovie<'movie>> {
    let title = movie.title.to_lowercase();
    let genre = movie.genre.to_lowercase();
    let overview = movie.overview.to_lowercase();

    let mut matched_tokens = 0;
    let mut field_weight = 0;

    for token in tokens {
        if title.contains(token) {
            matched_tokens += 1;
            field_weight += 3;
        } else if genre.contains(token) {
            matched_tokens += 1;
            field_weight += 2;
        } else if overview.contains(token) {
            matched_tokens += 1;
            field_weight += 1;
        }
    }

    (matched_tokens > 0).then_some(RankedMovie {
        movie,
        matched_tokens,
        field_weight,
    })
}

fn tokenize(input: &str) -> BTreeSet<String> {
    input
        .split(|ch: char| !ch.is_alphanumeric())
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .map(str::to_lowercase)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{Movie, MovieSearch, SearchRequest, ENGINE_NAME};

    const FIXTURE: &str = include_str!("../../../../fixtures/movies.json");

    #[test]
    fn loads_canonical_fixture() {
        let engine = MovieSearch::from_json(FIXTURE).unwrap();

        assert_eq!(engine.document_count(), 44_471);
        assert_eq!(engine.version().engine, ENGINE_NAME);
    }

    #[test]
    fn deduplicates_movies_by_id_with_last_write_winning() {
        let engine = MovieSearch::from_movies(vec![
            movie(7, "Old Title", "Drama", "first"),
            movie(3, "Other", "Comedy", "second"),
            movie(7, "New Title", "Drama", "third"),
        ]);

        let response = engine.movies(0, 10);

        assert_eq!(response.total, 2);
        assert_eq!(response.results[0].id, 3);
        assert_eq!(response.results[1].id, 7);
        assert_eq!(response.results[1].title, "New Title");
    }

    #[test]
    fn ranks_by_token_count_field_weight_then_id() {
        let engine = MovieSearch::from_movies(vec![
            movie(30, "Space", "Quiet Drama", "orbital story"),
            movie(10, "Quiet", "Science Fiction", "space survival mission"),
            movie(20, "Quiet Space", "Drama", "family story"),
        ]);

        let response = engine.search(SearchRequest {
            q: Some("quiet space".to_string()),
            offset: None,
            limit: Some(3),
        });

        let ids = response
            .hits
            .iter()
            .map(|movie| movie.id)
            .collect::<Vec<_>>();
        assert_eq!(ids, vec![20, 30, 10]);
    }

    #[test]
    fn empty_query_is_paginated_by_id() {
        let engine = MovieSearch::from_movies(vec![
            movie(30, "C", "Drama", ""),
            movie(10, "A", "Drama", ""),
            movie(20, "B", "Drama", ""),
        ]);

        let response = engine.search(SearchRequest {
            q: Some(String::new()),
            offset: Some(1),
            limit: Some(1),
        });

        assert_eq!(response.estimated_total_hits, 3);
        assert_eq!(response.hits.len(), 1);
        assert_eq!(response.hits[0].id, 20);
    }

    #[test]
    fn list_movies_uses_same_pagination_contract() {
        let engine =
            MovieSearch::from_movies(vec![movie(1, "A", "Drama", ""), movie(2, "B", "Drama", "")]);

        let response = engine.movies(1, 10);

        assert_eq!(response.total, 2);
        assert_eq!(response.results.len(), 1);
        assert_eq!(response.results[0].id, 2);
    }

    fn movie(id: u64, title: &str, genre: &str, overview: &str) -> Movie {
        Movie {
            id,
            title: title.to_string(),
            overview: overview.to_string(),
            genre: genre.to_string(),
            year: Some(2026),
            poster_path: None,
        }
    }
}
