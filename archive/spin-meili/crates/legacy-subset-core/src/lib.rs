use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

pub const DEFAULT_PRIMARY_KEY: &str = "id";
pub const DEFAULT_LIMIT: usize = 20;
const STATIC_TIMESTAMP: &str = "2026-05-11T00:00:00.000000000Z";

#[derive(Debug, Clone, Serialize)]
pub struct ApiError {
    #[serde(skip)]
    pub status: u16,
    pub message: String,
    pub code: String,
    #[serde(rename = "type")]
    pub error_type: String,
    pub link: String,
}

impl ApiError {
    pub fn new(
        status: u16,
        code: impl Into<String>,
        message: impl Into<String>,
        error_type: impl Into<String>,
    ) -> Self {
        let code = code.into();
        Self {
            status,
            message: message.into(),
            link: format!("https://docs.meilisearch.com/errors#{code}"),
            code,
            error_type: error_type.into(),
        }
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new(400, "bad_request", message, "invalid_request")
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(404, "not_found", message, "invalid_request")
    }

    pub fn method_not_allowed(message: impl Into<String>) -> Self {
        Self::new(405, "method_not_allowed", message, "invalid_request")
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(500, "internal", message, "internal")
    }

    pub fn missing_auth() -> Self {
        Self::new(
            401,
            "missing_authorization_header",
            "The Authorization header is missing. It must use the bearer authorization method.",
            "auth",
        )
    }

    pub fn invalid_api_key() -> Self {
        Self::new(
            403,
            "invalid_api_key",
            "The provided API key is invalid.",
            "auth",
        )
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct HealthResponse {
    pub status: &'static str,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionResponse {
    pub pkg_version: &'static str,
    pub commit_sha: Option<&'static str>,
    pub commit_date: Option<&'static str>,
    pub engine: &'static str,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateIndexRequest {
    pub uid: String,
    #[serde(default)]
    pub primary_key: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct IndexesResponse {
    pub results: Vec<IndexInfo>,
    pub offset: usize,
    pub limit: usize,
    pub total: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexInfo {
    pub uid: String,
    pub created_at: String,
    pub updated_at: String,
    pub primary_key: String,
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
    #[serde(default)]
    pub attributes_to_retrieve: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchResponse {
    pub hits: Vec<Value>,
    pub query: String,
    pub processing_time_ms: u64,
    pub limit: usize,
    pub offset: usize,
    pub estimated_total_hits: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MultiSearchRequest {
    #[serde(default)]
    pub queries: Vec<MultiSearchQuery>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MultiSearchQuery {
    #[serde(default, alias = "indexName")]
    pub index_uid: Option<String>,
    #[serde(default)]
    pub q: Option<String>,
    #[serde(default)]
    pub offset: Option<usize>,
    #[serde(default)]
    pub limit: Option<usize>,
    #[serde(default)]
    pub attributes_to_retrieve: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct MultiSearchResponse {
    pub results: Vec<SearchResponse>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentsFetchRequest {
    #[serde(default)]
    pub offset: Option<usize>,
    #[serde(default)]
    pub limit: Option<usize>,
    #[serde(default)]
    pub fields: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DocumentsResponse {
    pub results: Vec<Value>,
    pub offset: usize,
    pub limit: usize,
    pub total: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub task_uid: u64,
    #[serde(default)]
    pub index_uid: Option<String>,
    #[serde(default = "succeeded_status")]
    pub status: String,
    #[serde(rename = "type")]
    pub task_type: String,
    #[serde(default)]
    pub canceled_by: Option<u64>,
    pub details: Value,
    #[serde(default)]
    pub error: Option<Value>,
    #[serde(default)]
    pub duration: Option<String>,
    #[serde(default = "default_timestamp")]
    pub enqueued_at: String,
    #[serde(default = "default_timestamp")]
    pub started_at: String,
    #[serde(default = "default_timestamp")]
    pub finished_at: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct TasksResponse {
    pub results: Vec<Task>,
    pub limit: usize,
    pub from: Option<u64>,
    pub next: Option<u64>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatsResponse {
    pub database_size: usize,
    pub used_database_size: usize,
    pub last_update: Option<String>,
    pub indexes: BTreeMap<String, IndexStats>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexStats {
    pub number_of_documents: usize,
    pub is_indexing: bool,
    pub field_distribution: BTreeMap<String, usize>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct SearchEngine {
    indexes: BTreeMap<String, Index>,
    tasks: Vec<Task>,
    next_task_uid: u64,
}

impl SearchEngine {
    pub fn new() -> Self {
        Self {
            indexes: BTreeMap::new(),
            tasks: Vec::new(),
            next_task_uid: 0,
        }
    }

    pub fn health(&self) -> HealthResponse {
        HealthResponse {
            status: "available",
        }
    }

    pub fn version(&self) -> VersionResponse {
        VersionResponse {
            pkg_version: env!("CARGO_PKG_VERSION"),
            commit_sha: option_env!("GIT_COMMIT_SHA"),
            commit_date: option_env!("BUILD_DATE"),
            engine: "meili-spin-legacy-subset",
        }
    }

    pub fn list_indexes(&self) -> IndexesResponse {
        let results = self
            .indexes
            .values()
            .map(Index::info)
            .collect::<Vec<IndexInfo>>();
        let total = results.len();

        IndexesResponse {
            results,
            offset: 0,
            limit: DEFAULT_LIMIT,
            total,
        }
    }

    pub fn create_index(&mut self, request: CreateIndexRequest) -> Result<Task, ApiError> {
        if request.uid.trim().is_empty() {
            return Err(ApiError::bad_request("index uid must not be empty"));
        }

        self.indexes
            .entry(request.uid.clone())
            .or_insert_with(|| Index::new(&request.uid, request.primary_key.as_deref()));

        let task = self.task(
            Some(&request.uid),
            "indexCreation",
            json!({"primaryKey": request.primary_key.unwrap_or_else(|| DEFAULT_PRIMARY_KEY.to_string())}),
        );
        self.tasks.push(task.clone());
        Ok(task)
    }

    pub fn get_index(&self, uid: &str) -> Result<IndexInfo, ApiError> {
        self.index(uid).map(Index::info)
    }

    pub fn delete_index(&mut self, uid: &str) -> Result<Task, ApiError> {
        self.index(uid)?;
        self.indexes.remove(uid);
        let task = self.task(Some(uid), "indexDeletion", json!({}));
        self.tasks.push(task.clone());
        Ok(task)
    }

    pub fn add_documents(&mut self, uid: &str, docs: Vec<Value>) -> Result<Task, ApiError> {
        if uid.trim().is_empty() {
            return Err(ApiError::bad_request("index uid must not be empty"));
        }

        let index = self
            .indexes
            .entry(uid.to_string())
            .or_insert_with(|| Index::new(uid, Some(DEFAULT_PRIMARY_KEY)));

        let received_documents = docs.len();
        let indexed_documents = index.add_documents(docs)?;
        let task = self.task(
            Some(uid),
            "documentAdditionOrUpdate",
            json!({
                "receivedDocuments": received_documents,
                "indexedDocuments": indexed_documents
            }),
        );
        self.tasks.push(task.clone());
        Ok(task)
    }

    pub fn list_documents(
        &self,
        uid: &str,
        offset: usize,
        limit: usize,
    ) -> Result<DocumentsResponse, ApiError> {
        self.index(uid).map(|index| index.documents(offset, limit))
    }

    pub fn fetch_documents(
        &self,
        uid: &str,
        request: DocumentsFetchRequest,
    ) -> Result<DocumentsResponse, ApiError> {
        self.list_documents(
            uid,
            request.offset.unwrap_or(0),
            request.limit.unwrap_or(DEFAULT_LIMIT),
        )
    }

    pub fn get_document(&self, uid: &str, id: &str) -> Result<Value, ApiError> {
        self.index(uid)?.document(id)
    }

    pub fn delete_document(&mut self, uid: &str, id: &str) -> Result<Task, ApiError> {
        self.index_mut(uid)?.delete_document(id)?;
        let task = self.task(
            Some(uid),
            "documentDeletion",
            json!({"providedIds": 1, "deletedDocuments": 1}),
        );
        self.tasks.push(task.clone());
        Ok(task)
    }

    pub fn search(&self, uid: &str, request: SearchRequest) -> Result<SearchResponse, ApiError> {
        self.index(uid).map(|index| index.search(request))
    }

    pub fn multi_search(
        &self,
        request: MultiSearchRequest,
    ) -> Result<MultiSearchResponse, ApiError> {
        let mut results = Vec::with_capacity(request.queries.len());

        for query in request.queries {
            let Some(uid) = query.index_uid else {
                return Err(ApiError::bad_request(
                    "multi-search query is missing indexUid",
                ));
            };
            results.push(self.search(
                &uid,
                SearchRequest {
                    q: query.q,
                    offset: query.offset,
                    limit: query.limit,
                    attributes_to_retrieve: query.attributes_to_retrieve,
                },
            )?);
        }

        Ok(MultiSearchResponse { results })
    }

    pub fn stats(&self) -> StatsResponse {
        let indexes = self
            .indexes
            .iter()
            .map(|(uid, index)| (uid.clone(), index.stats()))
            .collect::<BTreeMap<_, _>>();

        let document_count = self
            .indexes
            .values()
            .map(|index| index.documents.len())
            .sum::<usize>();

        StatsResponse {
            database_size: document_count,
            used_database_size: document_count,
            last_update: Some(STATIC_TIMESTAMP.to_string()),
            indexes,
        }
    }

    pub fn index_stats(&self, uid: &str) -> Result<IndexStats, ApiError> {
        self.index(uid).map(Index::stats)
    }

    pub fn tasks(&self) -> TasksResponse {
        TasksResponse {
            results: self.tasks.clone(),
            limit: self.tasks.len(),
            from: self.tasks.first().map(|task| task.task_uid),
            next: None,
        }
    }

    pub fn settings(&self, uid: &str) -> Result<Value, ApiError> {
        Ok(self.index(uid)?.settings.clone())
    }

    pub fn setting(&self, uid: &str, key: &str) -> Result<Value, ApiError> {
        let setting_key = setting_key(key);
        Ok(self
            .index(uid)?
            .settings
            .get(setting_key)
            .cloned()
            .unwrap_or(Value::Null))
    }

    pub fn patch_settings(&mut self, uid: &str, patch: Value) -> Result<Task, ApiError> {
        merge_object(&mut self.index_mut(uid)?.settings, patch);
        let task = self.task(Some(uid), "settingsUpdate", json!({}));
        self.tasks.push(task.clone());
        Ok(task)
    }

    pub fn patch_setting(&mut self, uid: &str, key: &str, value: Value) -> Result<Task, ApiError> {
        let setting_key = setting_key(key).to_string();
        if let Some(settings) = self.index_mut(uid)?.settings.as_object_mut() {
            settings.insert(setting_key, value);
        }
        let task = self.task(Some(uid), "settingsUpdate", json!({}));
        self.tasks.push(task.clone());
        Ok(task)
    }

    fn index(&self, uid: &str) -> Result<&Index, ApiError> {
        self.indexes
            .get(uid)
            .ok_or_else(|| ApiError::not_found(format!("index '{uid}' was not found")))
    }

    fn index_mut(&mut self, uid: &str) -> Result<&mut Index, ApiError> {
        self.indexes
            .get_mut(uid)
            .ok_or_else(|| ApiError::not_found(format!("index '{uid}' was not found")))
    }

    fn task(&mut self, uid: Option<&str>, task_type: &str, details: Value) -> Task {
        let task_uid = self.next_task_uid;
        self.next_task_uid += 1;

        Task {
            task_uid,
            index_uid: uid.map(str::to_string),
            status: "succeeded".to_string(),
            task_type: task_type.to_string(),
            canceled_by: None,
            details,
            error: None,
            duration: Some("PT0S".to_string()),
            enqueued_at: STATIC_TIMESTAMP.to_string(),
            started_at: STATIC_TIMESTAMP.to_string(),
            finished_at: STATIC_TIMESTAMP.to_string(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Index {
    uid: String,
    #[serde(default = "default_primary_key")]
    primary_key: String,
    #[serde(default = "default_timestamp")]
    created_at: String,
    #[serde(default = "default_timestamp")]
    updated_at: String,
    documents: BTreeMap<String, Value>,
    #[serde(default = "default_settings")]
    settings: Value,
}

impl Index {
    fn new(uid: &str, primary_key: Option<&str>) -> Self {
        Self {
            uid: uid.to_string(),
            primary_key: primary_key.unwrap_or(DEFAULT_PRIMARY_KEY).to_string(),
            created_at: STATIC_TIMESTAMP.to_string(),
            updated_at: STATIC_TIMESTAMP.to_string(),
            documents: BTreeMap::new(),
            settings: default_settings(),
        }
    }

    fn info(&self) -> IndexInfo {
        IndexInfo {
            uid: self.uid.clone(),
            created_at: self.created_at.clone(),
            updated_at: self.updated_at.clone(),
            primary_key: self.primary_key.clone(),
        }
    }

    fn stats(&self) -> IndexStats {
        IndexStats {
            number_of_documents: self.documents.len(),
            is_indexing: false,
            field_distribution: self.field_distribution(),
        }
    }

    fn add_documents(&mut self, docs: Vec<Value>) -> Result<usize, ApiError> {
        let mut indexed = 0;

        for doc in docs {
            let id = document_id(&doc, &self.primary_key)?;
            self.documents.insert(id, doc);
            indexed += 1;
        }
        self.updated_at = STATIC_TIMESTAMP.to_string();

        Ok(indexed)
    }

    fn documents(&self, offset: usize, limit: usize) -> DocumentsResponse {
        let mut documents = self.documents.iter().collect::<Vec<_>>();
        documents.sort_by(|left, right| natural_id_cmp(left.0, right.0));

        let results = documents
            .into_iter()
            .map(|(_, doc)| doc)
            .skip(offset)
            .take(limit)
            .cloned()
            .collect::<Vec<_>>();

        DocumentsResponse {
            results,
            offset,
            limit,
            total: self.documents.len(),
        }
    }

    fn document(&self, id: &str) -> Result<Value, ApiError> {
        self.documents
            .get(id)
            .cloned()
            .ok_or_else(|| ApiError::not_found(format!("document '{id}' was not found")))
    }

    fn delete_document(&mut self, id: &str) -> Result<(), ApiError> {
        self.documents
            .remove(id)
            .map(|_| ())
            .ok_or_else(|| ApiError::not_found(format!("document '{id}' was not found")))
    }

    fn search(&self, request: SearchRequest) -> SearchResponse {
        let query = request.q.unwrap_or_default();
        let offset = request.offset.unwrap_or(0);
        let limit = request.limit.unwrap_or(DEFAULT_LIMIT);
        let tokens = tokenize(&query);

        let mut matches = if tokens.is_empty() {
            self.documents
                .iter()
                .map(|(id, doc)| (id.clone(), 0_usize, doc.clone()))
                .collect::<Vec<_>>()
        } else {
            self.documents
                .iter()
                .filter_map(|(id, doc)| {
                    let searchable_text = searchable_text(doc);
                    let score = tokens
                        .iter()
                        .filter(|token| searchable_text.contains(token.as_str()))
                        .count();

                    (score > 0).then(|| (id.clone(), score, doc.clone()))
                })
                .collect::<Vec<_>>()
        };

        matches.sort_by(|left, right| {
            right
                .1
                .cmp(&left.1)
                .then_with(|| natural_id_cmp(&left.0, &right.0))
        });

        let estimated_total_hits = matches.len();
        let hits = matches
            .into_iter()
            .skip(offset)
            .take(limit)
            .map(|(_, _, doc)| with_highlight_result(doc))
            .collect::<Vec<_>>();

        SearchResponse {
            hits,
            query,
            processing_time_ms: 0,
            limit,
            offset,
            estimated_total_hits,
        }
    }

    fn field_distribution(&self) -> BTreeMap<String, usize> {
        let mut fields = BTreeMap::new();
        for doc in self.documents.values().filter_map(Value::as_object) {
            for key in doc.keys() {
                *fields.entry(key.clone()).or_insert(0) += 1;
            }
        }
        fields
    }
}

fn default_settings() -> Value {
    json!({
        "displayedAttributes": ["*"],
        "searchableAttributes": ["*"],
        "filterableAttributes": [],
        "sortableAttributes": [],
        "rankingRules": [
            "words",
            "typo",
            "proximity",
            "attribute",
            "sort",
            "exactness"
        ],
        "stopWords": [],
        "synonyms": {},
        "distinctAttribute": null,
        "typoTolerance": {
            "enabled": true,
            "minWordSizeForTypos": {
                "oneTypo": 5,
                "twoTypos": 9
            },
            "disableOnWords": [],
            "disableOnAttributes": []
        },
        "faceting": {
            "maxValuesPerFacet": 100,
            "sortFacetValuesBy": {}
        },
        "pagination": {
            "maxTotalHits": 1000
        },
        "proximityPrecision": "byWord",
        "separatorTokens": null,
        "nonSeparatorTokens": null,
        "dictionary": [],
        "embedders": {},
        "searchCutoffMs": null,
        "localizedAttributes": null,
        "facetSearch": true,
        "prefixSearch": "indexingTime",
        "chat": {}
    })
}

fn default_primary_key() -> String {
    DEFAULT_PRIMARY_KEY.to_string()
}

fn default_timestamp() -> String {
    STATIC_TIMESTAMP.to_string()
}

fn succeeded_status() -> String {
    "succeeded".to_string()
}

fn setting_key(path_segment: &str) -> &str {
    match path_segment {
        "displayed-attributes" => "displayedAttributes",
        "searchable-attributes" => "searchableAttributes",
        "filterable-attributes" => "filterableAttributes",
        "sortable-attributes" => "sortableAttributes",
        "ranking-rules" => "rankingRules",
        "stop-words" => "stopWords",
        "distinct-attribute" => "distinctAttribute",
        "typo-tolerance" => "typoTolerance",
        "separator-tokens" => "separatorTokens",
        "non-separator-tokens" => "nonSeparatorTokens",
        "proximity-precision" => "proximityPrecision",
        "search-cutoff-ms" => "searchCutoffMs",
        "localized-attributes" => "localizedAttributes",
        "facet-search" => "facetSearch",
        "prefix-search" => "prefixSearch",
        other => other,
    }
}

fn merge_object(target: &mut Value, patch: Value) {
    if let (Some(target), Some(patch)) = (target.as_object_mut(), patch.as_object()) {
        for (key, value) in patch {
            target.insert(key.clone(), value.clone());
        }
    }
}

fn document_id(doc: &Value, primary_key: &str) -> Result<String, ApiError> {
    let object = doc
        .as_object()
        .ok_or_else(|| ApiError::bad_request("each document must be a JSON object"))?;
    let value = object.get(primary_key).ok_or_else(|| {
        ApiError::bad_request(format!("document is missing primary key '{primary_key}'"))
    })?;

    match value {
        Value::String(value) => Ok(value.clone()),
        Value::Number(value) => Ok(value.to_string()),
        _ => Err(ApiError::bad_request(format!(
            "primary key '{primary_key}' must be a string or number"
        ))),
    }
}

fn tokenize(input: &str) -> BTreeSet<String> {
    input
        .split(|ch: char| !ch.is_alphanumeric())
        .map(str::trim)
        .filter(|token| !token.is_empty())
        .map(str::to_lowercase)
        .collect()
}

fn searchable_text(value: &Value) -> String {
    let mut parts = Vec::new();
    collect_searchable_text(value, &mut parts);
    parts.join(" ").to_lowercase()
}

fn collect_searchable_text(value: &Value, parts: &mut Vec<String>) {
    match value {
        Value::Null => {}
        Value::Bool(value) => parts.push(value.to_string()),
        Value::Number(value) => parts.push(value.to_string()),
        Value::String(value) => parts.push(value.clone()),
        Value::Array(values) => values
            .iter()
            .for_each(|value| collect_searchable_text(value, parts)),
        Value::Object(object) => object
            .values()
            .for_each(|value| collect_searchable_text(value, parts)),
    }
}

fn with_highlight_result(mut doc: Value) -> Value {
    let Some(object) = doc.as_object_mut() else {
        return doc;
    };

    let highlight = object
        .iter()
        .filter(|(key, _)| key.as_str() != "_highlightResult")
        .filter_map(|(key, value)| {
            highlight_value(value).map(|value| {
                (
                    key.clone(),
                    json!({
                        "value": value,
                        "matchLevel": "none",
                        "matchedWords": []
                    }),
                )
            })
        })
        .collect::<serde_json::Map<_, _>>();

    object.insert("_highlightResult".to_string(), Value::Object(highlight));
    doc
}

fn highlight_value(value: &Value) -> Option<String> {
    match value {
        Value::Null | Value::Array(_) | Value::Object(_) => None,
        Value::Bool(value) => Some(value.to_string()),
        Value::Number(value) => Some(value.to_string()),
        Value::String(value) => Some(value.clone()),
    }
}

fn natural_id_cmp(left: &str, right: &str) -> std::cmp::Ordering {
    match (left.parse::<u64>(), right.parse::<u64>()) {
        (Ok(left), Ok(right)) => left.cmp(&right),
        _ => left.cmp(right),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{ApiError, SearchEngine, SearchRequest};

    #[test]
    fn indexes_and_searches_documents() {
        let mut engine = SearchEngine::new();
        engine
            .add_documents(
                "movies",
                vec![
                    json!({"id": 1, "title": "Moon Run", "overview": "A quiet space drama"}),
                    json!({"id": 2, "title": "City Heat", "overview": "A street thriller"}),
                ],
            )
            .unwrap();

        let response = engine
            .search(
                "movies",
                SearchRequest {
                    q: Some("space".to_string()),
                    offset: None,
                    limit: None,
                    attributes_to_retrieve: None,
                },
            )
            .unwrap();

        assert_eq!(response.estimated_total_hits, 1);
        assert_eq!(response.hits[0]["id"], 1);
    }

    #[test]
    fn placeholder_search_is_paginated() {
        let mut engine = SearchEngine::new();
        engine
            .add_documents(
                "movies",
                vec![
                    json!({"id": 1, "title": "A"}),
                    json!({"id": 2, "title": "B"}),
                    json!({"id": 3, "title": "C"}),
                ],
            )
            .unwrap();

        let response = engine
            .search(
                "movies",
                SearchRequest {
                    q: Some(String::new()),
                    offset: Some(1),
                    limit: Some(1),
                    attributes_to_retrieve: None,
                },
            )
            .unwrap();

        assert_eq!(response.estimated_total_hits, 3);
        assert_eq!(response.hits.len(), 1);
        assert_eq!(response.hits[0]["id"], 2);
    }

    #[test]
    fn rejects_documents_without_primary_key() {
        let mut engine = SearchEngine::new();
        let error = engine
            .add_documents("movies", vec![json!({"title": "No ID"})])
            .unwrap_err();

        assert_eq!(error.status, 400);
    }

    #[test]
    fn mirrors_auth_error_codes() {
        let missing = ApiError::missing_auth();
        let invalid = ApiError::invalid_api_key();

        assert_eq!(missing.status, 401);
        assert_eq!(missing.code, "missing_authorization_header");
        assert_eq!(invalid.status, 403);
        assert_eq!(invalid.code, "invalid_api_key");
    }

    #[test]
    fn lists_documents_for_dashboard() {
        let mut engine = SearchEngine::new();
        engine
            .add_documents(
                "movies",
                vec![
                    json!({"id": 1, "title": "A"}),
                    json!({"id": 2, "title": "B"}),
                ],
            )
            .unwrap();

        let docs = engine.list_documents("movies", 0, 1).unwrap();
        assert_eq!(docs.total, 2);
        assert_eq!(docs.results.len(), 1);
    }
}
