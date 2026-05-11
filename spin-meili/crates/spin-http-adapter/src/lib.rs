use meili_spin_legacy_subset_core::{
    ApiError, CreateIndexRequest, DocumentsFetchRequest, MultiSearchRequest, SearchEngine,
    SearchRequest,
};
use serde::Serialize;
use serde_json::Value;
use spin_sdk::http::{IntoResponse, Method, Request, Response};
use spin_sdk::http_component;
use spin_sdk::key_value::Store;
use spin_sdk::variables;

const ENGINE_STORE_KEY: &str = "engine";
const DEFAULT_MASTER_KEY: &str = "MASTER_KEY";

#[http_component]
fn handle_meili_spin_port(req: Request) -> anyhow::Result<impl IntoResponse> {
    Ok(route(req))
}

fn route(req: Request) -> Response {
    let method = method_name(req.method());
    let path = req.path();

    if method == "GET" {
        if let Some(response) = static_asset(path) {
            return response;
        }
    }

    if is_protected(path) {
        if let Err(error) = authorize(&req) {
            return json(error.status, &error);
        }
    }

    let segments = path
        .trim_matches('/')
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();

    let result = match (method, segments.as_slice()) {
        ("GET", ["health"]) => Ok(json(200, &SearchEngine::new().health())),
        ("GET", ["version"]) => Ok(json(200, &SearchEngine::new().version())),
        ("GET", ["indexes"]) => with_engine(|engine| Ok(json(200, &engine.list_indexes()))),
        ("POST", ["indexes"]) => handle_create_index(&req),
        ("GET", ["indexes", uid]) => {
            with_engine(|engine| engine.get_index(uid).map(|index| json(200, &index)))
        }
        ("DELETE", ["indexes", uid]) => {
            with_engine(|engine| engine.delete_index(uid).map(|task| json(202, &task)))
        }
        ("POST", ["indexes", uid, "documents"]) => handle_add_documents(&req, uid),
        ("GET", ["indexes", uid, "documents"]) => handle_list_documents(&req, uid),
        ("POST", ["indexes", uid, "documents", "fetch"]) => handle_fetch_documents(&req, uid),
        ("GET", ["indexes", uid, "documents", id]) => {
            with_engine(|engine| engine.get_document(uid, id).map(|doc| json(200, &doc)))
        }
        ("DELETE", ["indexes", uid, "documents", id]) => {
            with_engine(|engine| engine.delete_document(uid, id).map(|task| json(202, &task)))
        }
        ("POST", ["indexes", uid, "search"]) => handle_search(&req, uid),
        ("POST", ["multi-search"]) => handle_multi_search(&req),
        ("GET", ["indexes", uid, "stats"]) => {
            with_engine(|engine| engine.index_stats(uid).map(|stats| json(200, &stats)))
        }
        ("GET", ["indexes", uid, "settings"]) => {
            with_engine(|engine| engine.settings(uid).map(|settings| json(200, &settings)))
        }
        ("PATCH", ["indexes", uid, "settings"]) => handle_patch_settings(&req, uid),
        ("GET", ["indexes", uid, "settings", setting]) => {
            with_engine(|engine| engine.setting(uid, setting).map(|value| json(200, &value)))
        }
        ("PATCH", ["indexes", uid, "settings", setting]) => {
            handle_patch_setting(&req, uid, setting)
        }
        ("GET", ["stats"]) => with_engine(|engine| Ok(json(200, &engine.stats()))),
        ("GET", ["tasks"]) => with_engine(|engine| Ok(json(200, &engine.tasks()))),
        (_, ["health"])
        | (_, ["version"])
        | (_, ["indexes"])
        | (_, ["indexes", _])
        | (_, ["indexes", _, "documents"])
        | (_, ["indexes", _, "documents", "fetch"])
        | (_, ["indexes", _, "search"])
        | (_, ["multi-search"])
        | (_, ["indexes", _, "stats"])
        | (_, ["indexes", _, "settings"])
        | (_, ["indexes", _, "settings", _])
        | (_, ["stats"])
        | (_, ["tasks"]) => Err(ApiError::method_not_allowed(format!(
            "{method} is not allowed for {path}"
        ))),
        _ if method == "GET" && !looks_like_api_path(path) => Ok(static_index()),
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

fn authorize(req: &Request) -> Result<(), ApiError> {
    let Some(header) = req
        .header("authorization")
        .and_then(|value| value.as_str())
        .map(str::trim)
    else {
        return Err(ApiError::missing_auth());
    };

    let Some(token) = header.strip_prefix("Bearer ").map(str::trim) else {
        return Err(ApiError::missing_auth());
    };

    if token == master_key() {
        Ok(())
    } else {
        Err(ApiError::invalid_api_key())
    }
}

fn master_key() -> String {
    variables::get("meili_master_key").unwrap_or_else(|_| DEFAULT_MASTER_KEY.to_string())
}

fn is_protected(path: &str) -> bool {
    !matches!(
        path,
        "/" | "/health" | "/manifest.json" | "/robots.txt" | "/logo.svg" | "/favicon-32x32.png"
    ) && !path.starts_with("/assets/")
        && !path.starts_with("/fonts/")
        && looks_like_api_path(path)
}

fn looks_like_api_path(path: &str) -> bool {
    matches!(
        path.trim_start_matches('/')
            .split('/')
            .next()
            .unwrap_or_default(),
        "health" | "version" | "indexes" | "multi-search" | "stats" | "tasks"
    )
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

fn handle_create_index(req: &Request) -> Result<Response, ApiError> {
    let request = parse_body::<CreateIndexRequest>(req)?;
    with_engine(|engine| engine.create_index(request).map(|task| json(202, &task)))
}

fn handle_add_documents(req: &Request, uid: &str) -> Result<Response, ApiError> {
    let docs = parse_body::<Vec<Value>>(req)?;
    with_engine(|engine| engine.add_documents(uid, docs).map(|task| json(202, &task)))
}

fn handle_list_documents(req: &Request, uid: &str) -> Result<Response, ApiError> {
    let offset = query_usize(req.query(), "offset").unwrap_or(0);
    let limit = query_usize(req.query(), "limit").unwrap_or(20);
    with_engine(|engine| {
        engine
            .list_documents(uid, offset, limit)
            .map(|response| json(200, &response))
    })
}

fn handle_fetch_documents(req: &Request, uid: &str) -> Result<Response, ApiError> {
    let request = parse_body::<DocumentsFetchRequest>(req)?;
    with_engine(|engine| {
        engine
            .fetch_documents(uid, request)
            .map(|response| json(200, &response))
    })
}

fn handle_search(req: &Request, uid: &str) -> Result<Response, ApiError> {
    let request = parse_body::<SearchRequest>(req)?;
    with_engine(|engine| {
        engine
            .search(uid, request)
            .map(|response| json(200, &response))
    })
}

fn handle_multi_search(req: &Request) -> Result<Response, ApiError> {
    let request = parse_body::<MultiSearchRequest>(req)?;
    with_engine(|engine| {
        engine
            .multi_search(request)
            .map(|response| json(200, &response))
    })
}

fn handle_patch_settings(req: &Request, uid: &str) -> Result<Response, ApiError> {
    let patch = parse_body::<Value>(req)?;
    with_engine(|engine| {
        engine
            .patch_settings(uid, patch)
            .map(|task| json(202, &task))
    })
}

fn handle_patch_setting(req: &Request, uid: &str, setting: &str) -> Result<Response, ApiError> {
    let value = parse_body::<Value>(req)?;
    with_engine(|engine| {
        engine
            .patch_setting(uid, setting, value)
            .map(|task| json(202, &task))
    })
}

fn query_usize(query: &str, name: &str) -> Option<usize> {
    query.split('&').find_map(|pair| {
        let (key, value) = pair.split_once('=')?;
        (key == name).then(|| value.parse::<usize>().ok()).flatten()
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

fn static_asset(path: &str) -> Option<Response> {
    match path {
        "/" => Some(static_index()),
        "/assets/index-dDwzADMz.js" => Some(bytes(
            200,
            "application/javascript; charset=utf-8",
            include_bytes!("../../../static/mini-dashboard/assets/index-dDwzADMz.js").as_slice(),
        )),
        "/manifest.json" => Some(bytes(
            200,
            "application/manifest+json",
            include_bytes!("../../../static/mini-dashboard/manifest.json").as_slice(),
        )),
        "/robots.txt" => Some(bytes(
            200,
            "text/plain; charset=utf-8",
            include_bytes!("../../../static/mini-dashboard/robots.txt").as_slice(),
        )),
        "/logo.svg" => Some(bytes(
            200,
            "image/svg+xml",
            include_bytes!("../../../static/mini-dashboard/logo.svg").as_slice(),
        )),
        "/favicon-32x32.png" => Some(bytes(
            200,
            "image/png",
            include_bytes!("../../../static/mini-dashboard/favicon-32x32.png").as_slice(),
        )),
        "/fonts/Barlow/regular.woff2" => Some(bytes(
            200,
            "font/woff2",
            include_bytes!("../../../static/mini-dashboard/fonts/Barlow/regular.woff2").as_slice(),
        )),
        "/fonts/Work_Sans/bold.woff2" => Some(bytes(
            200,
            "font/woff2",
            include_bytes!("../../../static/mini-dashboard/fonts/Work_Sans/bold.woff2").as_slice(),
        )),
        "/fonts/Work_Sans/light.woff2" => Some(bytes(
            200,
            "font/woff2",
            include_bytes!("../../../static/mini-dashboard/fonts/Work_Sans/light.woff2").as_slice(),
        )),
        "/fonts/Work_Sans/medium.woff2" => Some(bytes(
            200,
            "font/woff2",
            include_bytes!("../../../static/mini-dashboard/fonts/Work_Sans/medium.woff2")
                .as_slice(),
        )),
        "/fonts/Work_Sans/regular.woff2" => Some(bytes(
            200,
            "font/woff2",
            include_bytes!("../../../static/mini-dashboard/fonts/Work_Sans/regular.woff2")
                .as_slice(),
        )),
        _ => None,
    }
}

fn static_index() -> Response {
    bytes(
        200,
        "text/html; charset=utf-8",
        include_bytes!("../../../static/mini-dashboard/index.html").as_slice(),
    )
}

fn bytes(status: u16, content_type: &str, body: &[u8]) -> Response {
    Response::builder()
        .status(status)
        .header("content-type", content_type)
        .body(body.to_vec())
        .build()
}
