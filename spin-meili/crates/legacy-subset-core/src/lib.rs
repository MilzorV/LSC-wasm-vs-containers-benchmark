use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const DEFAULT_PRIMARY_KEY: &str = "id";
pub const DEFAULT_LIMIT: usize = 20;

#[derive(Debug, Clone, Serialize)]
pub struct ApiError {
    #[serde(skip)]
    pub status: u16,
    pub code: String,
    pub message: String,
}

impl ApiError {
    pub fn new(status: u16, code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            status,
            code: code.into(),
            message: message.into(),
        }
    }

    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new(400, "bad_request", message)
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(404, "not_found", message)
    }

    pub fn method_not_allowed(message: impl Into<String>) -> Self {
        Self::new(405, "method_not_allowed", message)
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(500, "internal", message)
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
    pub build_date: Option<&'static str>,
    pub engine: &'static str,
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
    pub primary_key: String,
    pub number_of_documents: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SearchRequest {
    #[serde(default)]
    pub q: Option<String>,
    #[serde(default)]
    pub offset: Option<usize>,
    #[serde(default)]
    pub limit: Option<usize>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub task_uid: u64,
    pub index_uid: String,
    pub status: String,
    #[serde(rename = "type")]
    pub task_type: String,
    pub details: TaskDetails,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaskDetails {
    pub received_documents: usize,
    pub indexed_documents: usize,
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
            build_date: option_env!("BUILD_DATE"),
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
            limit: total,
            total,
        }
    }

    pub fn add_documents(&mut self, uid: &str, docs: Vec<Value>) -> Result<Task, ApiError> {
        if uid.trim().is_empty() {
            return Err(ApiError::bad_request("index uid must not be empty"));
        }

        let index = self
            .indexes
            .entry(uid.to_string())
            .or_insert_with(|| Index::new(uid));

        let received_documents = docs.len();
        let indexed_documents = index.add_documents(docs)?;
        let task = self.completed_document_task(uid, received_documents, indexed_documents);
        self.tasks.push(task.clone());
        Ok(task)
    }

    pub fn search(&self, uid: &str, request: SearchRequest) -> Result<SearchResponse, ApiError> {
        let index = self
            .indexes
            .get(uid)
            .ok_or_else(|| ApiError::not_found(format!("index '{uid}' was not found")))?;

        Ok(index.search(request))
    }

    pub fn stats(&self) -> StatsResponse {
        let indexes = self
            .indexes
            .iter()
            .map(|(uid, index)| {
                (
                    uid.clone(),
                    IndexStats {
                        number_of_documents: index.documents.len(),
                        is_indexing: false,
                    },
                )
            })
            .collect::<BTreeMap<_, _>>();

        let document_count = self
            .indexes
            .values()
            .map(|index| index.documents.len())
            .sum::<usize>();

        StatsResponse {
            database_size: document_count,
            used_database_size: document_count,
            last_update: None,
            indexes,
        }
    }

    pub fn tasks(&self) -> TasksResponse {
        TasksResponse {
            results: self.tasks.clone(),
            limit: self.tasks.len(),
            from: self.tasks.first().map(|task| task.task_uid),
            next: None,
        }
    }

    fn completed_document_task(
        &mut self,
        uid: &str,
        received_documents: usize,
        indexed_documents: usize,
    ) -> Task {
        let task_uid = self.next_task_uid;
        self.next_task_uid += 1;

        Task {
            task_uid,
            index_uid: uid.to_string(),
            status: "succeeded".to_string(),
            task_type: "documentAdditionOrUpdate".to_string(),
            details: TaskDetails {
                received_documents,
                indexed_documents,
            },
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Index {
    uid: String,
    primary_key: String,
    documents: BTreeMap<String, Value>,
}

impl Index {
    fn new(uid: &str) -> Self {
        Self {
            uid: uid.to_string(),
            primary_key: DEFAULT_PRIMARY_KEY.to_string(),
            documents: BTreeMap::new(),
        }
    }

    fn info(&self) -> IndexInfo {
        IndexInfo {
            uid: self.uid.clone(),
            primary_key: self.primary_key.clone(),
            number_of_documents: self.documents.len(),
        }
    }

    fn add_documents(&mut self, docs: Vec<Value>) -> Result<usize, ApiError> {
        let mut indexed = 0;

        for doc in docs {
            let id = document_id(&doc, &self.primary_key)?;
            self.documents.insert(id, doc);
            indexed += 1;
        }

        Ok(indexed)
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

        matches.sort_by(|left, right| right.1.cmp(&left.1).then_with(|| left.0.cmp(&right.0)));

        let estimated_total_hits = matches.len();
        let hits = matches
            .into_iter()
            .skip(offset)
            .take(limit)
            .map(|(_, _, doc)| doc)
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

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{SearchEngine, SearchRequest};

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
        assert_eq!(error.code, "bad_request");
    }
}
