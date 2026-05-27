use std::cmp::Ordering;
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

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchRequest {
    #[serde(default)]
    pub q: Option<String>,
    #[serde(default)]
    pub offset: Option<usize>,
    #[serde(default)]
    pub limit: Option<usize>,
    #[serde(default, rename = "filter")]
    pub filters: Option<SearchFilter>,
    #[serde(default)]
    pub sort: Vec<String>,
    #[serde(default)]
    pub facets: Vec<String>,
    #[serde(default)]
    pub highlight: Vec<String>,
    #[serde(default)]
    pub typo_tolerance: bool,
    #[serde(default)]
    pub debug_ranking: bool,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuggestRequest {
    #[serde(default)]
    pub q: Option<String>,
    #[serde(default)]
    pub limit: Option<usize>,
    #[serde(default, rename = "filter")]
    pub filters: Option<SearchFilter>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchFilter {
    #[serde(default)]
    pub genre: Vec<String>,
    #[serde(default)]
    pub year: Option<YearFilter>,
}

#[derive(Debug, Clone, Copy, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct YearFilter {
    #[serde(default)]
    pub gte: Option<i32>,
    #[serde(default)]
    pub lte: Option<i32>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SearchResponse {
    pub hits: Vec<SearchHit>,
    pub query: String,
    pub offset: usize,
    pub limit: usize,
    pub estimated_total_hits: usize,
    pub processing_time_ms: u128,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub facet_distribution: BTreeMap<String, BTreeMap<String, usize>>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub facet_stats: BTreeMap<String, FacetStats>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ranking_info: Option<RankingInfo>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SearchHit {
    #[serde(flatten)]
    pub movie: Movie,
    #[serde(rename = "_formatted", skip_serializing_if = "Option::is_none")]
    pub formatted: Option<BTreeMap<String, String>>,
    #[serde(rename = "_rankingInfo", skip_serializing_if = "Option::is_none")]
    pub ranking_info: Option<HitRankingInfo>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FacetStats {
    pub min: i32,
    pub max: i32,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct RankingInfo {
    pub strategy: &'static str,
    pub typo_tolerance: bool,
    pub sort: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct HitRankingInfo {
    pub matched_tokens: usize,
    pub field_weight: usize,
    pub typo_matches: usize,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SuggestResponse {
    pub suggestions: Vec<String>,
    pub query: String,
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
        let highlight = normalize_requested_fields(&request.highlight);
        let facets = normalize_requested_fields(&request.facets);

        let mut matches = if tokens.is_empty() {
            self.movies
                .iter()
                .filter(|movie| matches_filter(movie, request.filters.as_ref()))
                .map(|movie| RankedMovie {
                    movie,
                    matched_tokens: 0,
                    field_weight: 0,
                    match_quality: 0,
                    typo_matches: 0,
                })
                .collect::<Vec<_>>()
        } else {
            self.movies
                .iter()
                .filter(|movie| matches_filter(movie, request.filters.as_ref()))
                .filter_map(|movie| rank_movie(movie, &tokens, request.typo_tolerance))
                .collect::<Vec<_>>()
        };

        let facet_distribution = facet_distribution(&matches, &facets);
        let facet_stats = facet_stats(&matches, &facets);

        sort_matches(&mut matches, &request.sort);

        let estimated_total_hits = matches.len();
        let hits = matches
            .into_iter()
            .skip(offset)
            .take(limit)
            .map(|ranked| SearchHit {
                movie: ranked.movie.clone(),
                formatted: formatted_fields(ranked.movie, &tokens, &highlight),
                ranking_info: request.debug_ranking.then_some(HitRankingInfo {
                    matched_tokens: ranked.matched_tokens,
                    field_weight: ranked.field_weight,
                    typo_matches: ranked.typo_matches,
                }),
            })
            .collect();

        SearchResponse {
            hits,
            query,
            offset,
            limit,
            estimated_total_hits,
            processing_time_ms: started.elapsed().as_millis(),
            facet_distribution,
            facet_stats,
            ranking_info: request.debug_ranking.then_some(RankingInfo {
                strategy: "matched_tokens, field_weight, match_quality, id",
                typo_tolerance: request.typo_tolerance,
                sort: request.sort,
            }),
        }
    }

    pub fn suggest(&self, request: SuggestRequest) -> SuggestResponse {
        let started = Instant::now();
        let query = request.q.unwrap_or_default();
        let normalized_query = query.trim().to_lowercase();
        let limit = request.limit.unwrap_or(5);

        let mut suggestions = if normalized_query.is_empty() {
            Vec::new()
        } else {
            self.movies
                .iter()
                .filter(|movie| matches_filter(movie, request.filters.as_ref()))
                .filter_map(|movie| {
                    let title = movie.title.to_lowercase();
                    if title.starts_with(&normalized_query) {
                        Some((0usize, movie.id, movie.title.clone()))
                    } else if title.contains(&normalized_query) {
                        Some((1usize, movie.id, movie.title.clone()))
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
        };

        suggestions.sort_by(|left, right| left.0.cmp(&right.0).then_with(|| left.1.cmp(&right.1)));
        let mut seen_titles = BTreeSet::new();

        SuggestResponse {
            suggestions: suggestions
                .into_iter()
                .filter(|(_, _, title)| seen_titles.insert(title.to_lowercase()))
                .take(limit)
                .map(|(_, _, title)| title)
                .collect(),
            query,
            processing_time_ms: started.elapsed().as_millis(),
        }
    }
}

struct RankedMovie<'movie> {
    movie: &'movie Movie,
    matched_tokens: usize,
    field_weight: usize,
    match_quality: usize,
    typo_matches: usize,
}

fn rank_movie<'movie>(
    movie: &'movie Movie,
    tokens: &BTreeSet<String>,
    typo_tolerance: bool,
) -> Option<RankedMovie<'movie>> {
    let title = searchable_field(&movie.title);
    let genre = searchable_field(&movie.genre);
    let overview = searchable_field(&movie.overview);

    let mut matched_tokens = 0;
    let mut field_weight = 0;
    let mut match_quality = 0;
    let mut typo_matches = 0;

    for token in tokens {
        if let Some(kind) = match_field(&title, token, typo_tolerance) {
            matched_tokens += 1;
            field_weight += 3;
            match_quality += kind.quality();
            typo_matches += usize::from(kind == MatchKind::Typo);
        } else if let Some(kind) = match_field(&genre, token, typo_tolerance) {
            matched_tokens += 1;
            field_weight += 2;
            match_quality += kind.quality();
            typo_matches += usize::from(kind == MatchKind::Typo);
        } else if let Some(kind) = match_field(&overview, token, typo_tolerance) {
            matched_tokens += 1;
            field_weight += 1;
            match_quality += kind.quality();
            typo_matches += usize::from(kind == MatchKind::Typo);
        }
    }

    (matched_tokens > 0).then_some(RankedMovie {
        movie,
        matched_tokens,
        field_weight,
        match_quality,
        typo_matches,
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

#[derive(Debug, Clone)]
struct SearchableField {
    text: String,
    tokens: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MatchKind {
    Exact,
    Prefix,
    Typo,
}

impl MatchKind {
    fn quality(self) -> usize {
        match self {
            MatchKind::Exact => 0,
            MatchKind::Prefix => 1,
            MatchKind::Typo => 2,
        }
    }
}

fn searchable_field(value: &str) -> SearchableField {
    SearchableField {
        text: value.to_lowercase(),
        tokens: value
            .split(|ch: char| !ch.is_alphanumeric())
            .map(str::trim)
            .filter(|token| !token.is_empty())
            .map(str::to_lowercase)
            .collect(),
    }
}

fn match_field(field: &SearchableField, token: &str, typo_tolerance: bool) -> Option<MatchKind> {
    if field.text.contains(token) {
        return Some(MatchKind::Exact);
    }
    if field
        .tokens
        .iter()
        .any(|field_token| field_token.starts_with(token))
    {
        return Some(MatchKind::Prefix);
    }
    if typo_tolerance
        && typo_threshold(token).is_some_and(|threshold| {
            field
                .tokens
                .iter()
                .any(|field_token| levenshtein(token, field_token) <= threshold)
        })
    {
        return Some(MatchKind::Typo);
    }
    None
}

fn typo_threshold(token: &str) -> Option<usize> {
    match token.chars().count() {
        0..=3 => None,
        4..=7 => Some(1),
        _ => Some(2),
    }
}

fn levenshtein(left: &str, right: &str) -> usize {
    let right_chars = right.chars().collect::<Vec<_>>();
    let mut previous = (0..=right_chars.len()).collect::<Vec<_>>();
    let mut current = vec![0; right_chars.len() + 1];

    for (left_index, left_ch) in left.chars().enumerate() {
        current[0] = left_index + 1;
        for (right_index, right_ch) in right_chars.iter().enumerate() {
            let substitution = previous[right_index] + usize::from(left_ch != *right_ch);
            let insertion = current[right_index] + 1;
            let deletion = previous[right_index + 1] + 1;
            current[right_index + 1] = substitution.min(insertion).min(deletion);
        }
        std::mem::swap(&mut previous, &mut current);
    }

    previous[right_chars.len()]
}

fn matches_filter(movie: &Movie, filters: Option<&SearchFilter>) -> bool {
    let Some(filters) = filters else {
        return true;
    };

    if !filters.genre.is_empty() && !genre_filter_matches(&movie.genre, &filters.genre) {
        return false;
    }

    if let Some(year_filter) = filters.year {
        let Some(year) = movie.year else {
            return false;
        };
        if year_filter.gte.is_some_and(|min| year < min) {
            return false;
        }
        if year_filter.lte.is_some_and(|max| year > max) {
            return false;
        }
    }

    true
}

fn genre_filter_matches(movie_genre: &str, requested: &[String]) -> bool {
    genre_values(movie_genre).any(|movie_genre| {
        requested
            .iter()
            .any(|requested_genre| movie_genre.eq_ignore_ascii_case(requested_genre.trim()))
    })
}

fn genre_values(movie_genre: &str) -> impl Iterator<Item = &str> {
    movie_genre
        .split(',')
        .map(str::trim)
        .filter(|genre| !genre.is_empty())
}

fn sort_matches(matches: &mut [RankedMovie<'_>], sort: &[String]) {
    matches.sort_by(|left, right| {
        compare_sort_fields(left.movie, right.movie, sort)
            .unwrap_or_else(|| relevance_order(left, right))
    });
}

fn relevance_order(left: &RankedMovie<'_>, right: &RankedMovie<'_>) -> Ordering {
    right
        .matched_tokens
        .cmp(&left.matched_tokens)
        .then_with(|| right.field_weight.cmp(&left.field_weight))
        .then_with(|| left.match_quality.cmp(&right.match_quality))
        .then_with(|| left.movie.id.cmp(&right.movie.id))
}

fn compare_sort_fields(left: &Movie, right: &Movie, sort: &[String]) -> Option<Ordering> {
    for raw in sort {
        let (field, direction) = raw.split_once(':').unwrap_or((raw.as_str(), "asc"));
        let ordering = match field {
            "id" => left.id.cmp(&right.id),
            "title" => left.title.to_lowercase().cmp(&right.title.to_lowercase()),
            "year" => compare_optional_year(left.year, right.year),
            _ => Ordering::Equal,
        };

        let ordering = if direction.eq_ignore_ascii_case("desc") {
            ordering.reverse()
        } else {
            ordering
        };
        if ordering != Ordering::Equal {
            return Some(ordering);
        }
    }
    None
}

fn compare_optional_year(left: Option<i32>, right: Option<i32>) -> Ordering {
    match (left, right) {
        (Some(left), Some(right)) => left.cmp(&right),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

fn normalize_requested_fields(fields: &[String]) -> BTreeSet<String> {
    fields.iter().map(|field| field.to_lowercase()).collect()
}

fn facet_distribution(
    matches: &[RankedMovie<'_>],
    facets: &BTreeSet<String>,
) -> BTreeMap<String, BTreeMap<String, usize>> {
    let mut result = BTreeMap::new();
    if facets.contains("genre") {
        let mut counts = BTreeMap::new();
        for ranked in matches {
            for genre in genre_values(&ranked.movie.genre) {
                *counts.entry(genre.to_string()).or_insert(0) += 1;
            }
        }
        result.insert("genre".to_string(), counts);
    }
    if facets.contains("year") {
        let mut counts = BTreeMap::new();
        for ranked in matches {
            if let Some(year) = ranked.movie.year {
                *counts.entry(year.to_string()).or_insert(0) += 1;
            }
        }
        result.insert("year".to_string(), counts);
    }
    result
}

fn facet_stats(
    matches: &[RankedMovie<'_>],
    facets: &BTreeSet<String>,
) -> BTreeMap<String, FacetStats> {
    let mut result = BTreeMap::new();
    if facets.contains("year") {
        let years = matches
            .iter()
            .filter_map(|ranked| ranked.movie.year)
            .collect::<Vec<_>>();
        if let (Some(min), Some(max)) = (years.iter().min(), years.iter().max()) {
            result.insert(
                "year".to_string(),
                FacetStats {
                    min: *min,
                    max: *max,
                },
            );
        }
    }
    result
}

fn formatted_fields(
    movie: &Movie,
    tokens: &BTreeSet<String>,
    highlight: &BTreeSet<String>,
) -> Option<BTreeMap<String, String>> {
    if highlight.is_empty() {
        return None;
    }
    let mut formatted = BTreeMap::new();
    for field in highlight {
        match field.as_str() {
            "title" => {
                formatted.insert(field.clone(), highlight_value(&movie.title, tokens));
            }
            "genre" => {
                formatted.insert(field.clone(), highlight_value(&movie.genre, tokens));
            }
            "overview" => {
                formatted.insert(field.clone(), highlight_value(&movie.overview, tokens));
            }
            _ => {}
        }
    }
    (!formatted.is_empty()).then_some(formatted)
}

fn highlight_value(value: &str, tokens: &BTreeSet<String>) -> String {
    let escaped = escape_html(value);
    let Some(first) = tokens
        .iter()
        .filter(|token| !token.is_empty())
        .find(|token| escaped.to_lowercase().contains(token.as_str()))
    else {
        return escaped;
    };

    highlight_ascii_case_insensitive(&escaped, first)
}

fn highlight_ascii_case_insensitive(value: &str, token: &str) -> String {
    let haystack = value.to_lowercase();
    let Some(start) = haystack.find(token) else {
        return value.to_string();
    };
    let end = start + token.len();
    format!(
        "{}<mark>{}</mark>{}",
        &value[..start],
        &value[start..end],
        &value[end..]
    )
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

#[cfg(test)]
mod tests {
    use super::{
        Movie, MovieSearch, SearchFilter, SearchRequest, SuggestRequest, YearFilter, ENGINE_NAME,
    };

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
            ..SearchRequest::default()
        });

        let ids = response
            .hits
            .iter()
            .map(|hit| hit.movie.id)
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
            ..SearchRequest::default()
        });

        assert_eq!(response.estimated_total_hits, 3);
        assert_eq!(response.hits.len(), 1);
        assert_eq!(response.hits[0].movie.id, 20);
    }

    #[test]
    fn legacy_fixture_search_ids_are_unchanged_without_new_options() {
        let engine = MovieSearch::from_json(FIXTURE).unwrap();

        let response = engine.search(SearchRequest {
            q: Some("space".to_string()),
            limit: Some(10),
            ..SearchRequest::default()
        });

        let ids = response
            .hits
            .iter()
            .map(|hit| hit.movie.id)
            .collect::<Vec<_>>();
        assert_eq!(
            ids,
            vec![62, 957, 1542, 2157, 2300, 2614, 5551, 6795, 7269, 7290]
        );
    }

    #[test]
    fn filters_narrow_by_genre_and_year() {
        let engine = MovieSearch::from_movies(vec![
            movie_with_year(1, "A", "Drama, Science Fiction", "space", 1999),
            movie_with_year(2, "B", "Comedy", "space", 2001),
            movie_with_year(3, "C", "Drama", "space", 2011),
        ]);

        let response = engine.search(SearchRequest {
            q: Some("space".to_string()),
            filters: Some(SearchFilter {
                genre: vec!["drama".to_string()],
                year: Some(YearFilter {
                    gte: Some(1990),
                    lte: Some(2000),
                }),
            }),
            ..SearchRequest::default()
        });

        assert_eq!(response.hits.len(), 1);
        assert_eq!(response.hits[0].movie.id, 1);
    }

    #[test]
    fn facets_include_genre_distribution_and_year_stats() {
        let engine = MovieSearch::from_movies(vec![
            movie_with_year(1, "A", "Drama", "space", 1999),
            movie_with_year(2, "B", "Comedy", "space", 2001),
            movie_with_year(3, "C", "Drama", "space", 2011),
        ]);

        let response = engine.search(SearchRequest {
            q: Some("space".to_string()),
            facets: vec!["genre".to_string(), "year".to_string()],
            ..SearchRequest::default()
        });

        assert_eq!(response.facet_distribution["genre"]["Drama"], 2);
        assert_eq!(response.facet_distribution["genre"]["Comedy"], 1);
        assert_eq!(response.facet_stats["year"].min, 1999);
        assert_eq!(response.facet_stats["year"].max, 2011);
    }

    #[test]
    fn sorting_can_override_relevance_order() {
        let engine = MovieSearch::from_movies(vec![
            movie_with_year(1, "Zulu", "Drama", "space", 1999),
            movie_with_year(2, "Alpha", "Drama", "space", 2010),
        ]);

        let by_year = engine.search(SearchRequest {
            q: Some("space".to_string()),
            sort: vec!["year:desc".to_string()],
            ..SearchRequest::default()
        });
        assert_eq!(by_year.hits[0].movie.id, 2);

        let by_title = engine.search(SearchRequest {
            q: Some("space".to_string()),
            sort: vec!["title:asc".to_string()],
            ..SearchRequest::default()
        });
        assert_eq!(by_title.hits[0].movie.id, 2);
    }

    #[test]
    fn typo_tolerance_is_opt_in() {
        let engine = MovieSearch::from_movies(vec![movie(1, "Space Movie", "Drama", "")]);

        let strict = engine.search(SearchRequest {
            q: Some("spce".to_string()),
            ..SearchRequest::default()
        });
        assert_eq!(strict.estimated_total_hits, 0);

        let tolerant = engine.search(SearchRequest {
            q: Some("spce".to_string()),
            typo_tolerance: true,
            ..SearchRequest::default()
        });
        assert_eq!(tolerant.hits[0].movie.id, 1);
    }

    #[test]
    fn highlighting_escapes_html_and_marks_matches() {
        let engine =
            MovieSearch::from_movies(vec![movie(1, "Space <Drama>", "Drama", "A space story")]);

        let response = engine.search(SearchRequest {
            q: Some("space".to_string()),
            highlight: vec!["title".to_string(), "overview".to_string()],
            ..SearchRequest::default()
        });

        let formatted = response.hits[0].formatted.as_ref().unwrap();
        assert_eq!(formatted["title"], "<mark>Space</mark> &lt;Drama&gt;");
        assert_eq!(formatted["overview"], "A <mark>space</mark> story");
    }

    #[test]
    fn suggestions_rank_prefix_before_substring() {
        let engine = MovieSearch::from_movies(vec![
            movie(20, "The Dark Knight", "Action", ""),
            movie(10, "Dark City", "Sci-Fi", ""),
            movie(30, "A Very Dark Tale", "Drama", ""),
        ]);

        let response = engine.suggest(SuggestRequest {
            q: Some("dark".to_string()),
            limit: Some(3),
            filters: None,
        });

        assert_eq!(
            response.suggestions,
            vec![
                "Dark City".to_string(),
                "The Dark Knight".to_string(),
                "A Very Dark Tale".to_string()
            ]
        );
    }

    #[test]
    fn suggestions_deduplicate_equal_titles() {
        let engine = MovieSearch::from_movies(vec![
            movie(20, "Dark City", "Action", ""),
            movie(10, "Dark City", "Drama", ""),
            movie(30, "Dark City 2", "Drama", ""),
        ]);

        let response = engine.suggest(SuggestRequest {
            q: Some("dark".to_string()),
            limit: Some(5),
            filters: None,
        });

        assert_eq!(
            response.suggestions,
            vec!["Dark City".to_string(), "Dark City 2".to_string()]
        );
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
        movie_with_year(id, title, genre, overview, 2026)
    }

    fn movie_with_year(id: u64, title: &str, genre: &str, overview: &str, year: i32) -> Movie {
        Movie {
            id,
            title: title.to_string(),
            overview: overview.to_string(),
            genre: genre.to_string(),
            year: Some(year),
            poster_path: None,
        }
    }
}
