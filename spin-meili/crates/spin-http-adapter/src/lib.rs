use meili_spin_core::{ApiError, SearchEngine, SearchRequest};
use serde::Serialize;
use serde_json::Value;
use spin_sdk::http::{IntoResponse, Method, Request, Response};
use spin_sdk::http_component;
use spin_sdk::key_value::Store;

const ENGINE_STORE_KEY: &str = "engine";

#[http_component]
fn handle_meili_spin_subset(req: Request) -> anyhow::Result<impl IntoResponse> {
    Ok(route(req))
}

fn route(req: Request) -> Response {
    let method = method_name(req.method());
    let path = req.path();
    let segments = path
        .trim_matches('/')
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();

    let result = match (method, segments.as_slice()) {
        ("GET", ["health"]) => Ok(json(200, &SearchEngine::new().health())),
        ("GET", ["version"]) => Ok(json(200, &SearchEngine::new().version())),
        ("GET", ["indexes"]) => with_engine(|engine| Ok(json(200, &engine.list_indexes()))),
        ("POST", ["indexes", uid, "documents"]) => handle_add_documents(&req, uid),
        ("POST", ["indexes", uid, "search"]) => handle_search(&req, uid),
        ("GET", ["stats"]) => with_engine(|engine| Ok(json(200, &engine.stats()))),
        ("GET", ["tasks"]) => with_engine(|engine| Ok(json(200, &engine.tasks()))),
        (_, ["health"])
        | (_, ["version"])
        | (_, ["indexes"])
        | (_, ["indexes", _, "documents"])
        | (_, ["indexes", _, "search"])
        | (_, ["stats"])
        | (_, ["tasks"]) => Err(ApiError::method_not_allowed(format!(
            "{method} is not allowed for {path}"
        ))),
        _ => Err(ApiError::not_found(format!("route '{path}' was not found"))),
    };

    result.unwrap_or_else(error)
}

fn with_engine<T>(
    operation: impl FnOnce(&mut SearchEngine) -> Result<T, ApiError>,
) -> Result<T, ApiError> {
    let store = Store::open_default()
        .map_err(|err| ApiError::internal(format!("failed to open key-value store: {err:?}")))?;
    let mut engine = match store
        .get(ENGINE_STORE_KEY)
        .map_err(|err| ApiError::internal(format!("failed to load engine state: {err:?}")))?
    {
        Some(bytes) => serde_json::from_slice(&bytes)
            .map_err(|err| ApiError::internal(format!("failed to decode engine state: {err}")))?,
        None => SearchEngine::new(),
    };

    let output = operation(&mut engine)?;
    let bytes = serde_json::to_vec(&engine)
        .map_err(|err| ApiError::internal(format!("failed to encode engine state: {err}")))?;
    store
        .set(ENGINE_STORE_KEY, &bytes)
        .map_err(|err| ApiError::internal(format!("failed to save engine state: {err:?}")))?;

    Ok(output)
}

fn method_name(method: &Method) -> &'static str {
    match method {
        Method::Get => "GET",
        Method::Post => "POST",
        Method::Put => "PUT",
        Method::Delete => "DELETE",
        Method::Patch => "PATCH",
        Method::Head => "HEAD",
        Method::Options => "OPTIONS",
        Method::Connect => "CONNECT",
        Method::Trace => "TRACE",
        Method::Other(_) => "OTHER",
    }
}

fn handle_add_documents(req: &Request, uid: &str) -> Result<Response, ApiError> {
    let docs = parse_body::<Vec<Value>>(req)?;
    with_engine(|engine| engine.add_documents(uid, docs).map(|task| json(202, &task)))
}

fn handle_search(req: &Request, uid: &str) -> Result<Response, ApiError> {
    let request = parse_body::<SearchRequest>(req)?;
    with_engine(|engine| {
        engine
            .search(uid, request)
            .map(|response| json(200, &response))
    })
}

fn parse_body<T>(req: &Request) -> Result<T, ApiError>
where
    T: serde::de::DeserializeOwned,
{
    serde_json::from_slice(req.body())
        .map_err(|err| ApiError::bad_request(format!("request body must be valid JSON: {err}")))
}

fn json<T>(status: u16, value: &T) -> Response
where
    T: Serialize,
{
    let body = serde_json::to_string(value)
        .unwrap_or_else(|_| "{\"code\":\"internal\",\"message\":\"serialization failed\"}".into());

    Response::builder()
        .status(status)
        .header("content-type", "application/json")
        .body(body)
        .build()
}

fn error(error: ApiError) -> Response {
    json(error.status, &error)
}
