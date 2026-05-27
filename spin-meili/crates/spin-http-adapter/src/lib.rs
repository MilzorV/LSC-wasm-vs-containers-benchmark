use std::sync::OnceLock;

use movie_search_core::{MovieSearch, SearchRequest, DEFAULT_LIMIT};
use serde::Serialize;
use spin_sdk::http::{IntoResponse, Method, Request, Response};
use spin_sdk::http_component;
use static_assets::lookup;

const MOVIES_JSON: &str = include_str!("../../../../fixtures/movies.json");

static ENGINE: OnceLock<MovieSearch> = OnceLock::new();

#[http_component]
fn handle_movie_search(req: Request) -> anyhow::Result<impl IntoResponse> {
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

    if method == "OPTIONS" && is_api_path(&segments) {
        return build_response(204, None, String::new());
    }

    let result = match (method, segments.as_slice()) {
        ("GET", ["health"]) => Ok(json(200, &engine().health())),
        ("GET", ["version"]) => Ok(json(200, &engine().version())),
        ("GET", ["stats"]) => Ok(json(200, &engine().stats())),
        ("GET", ["movies"]) => {
            let offset = query_usize(req.query(), "offset").unwrap_or(0);
            let limit = query_usize(req.query(), "limit").unwrap_or(DEFAULT_LIMIT);
            Ok(json(200, &engine().movies(offset, limit)))
        }
        ("POST", ["search"]) => handle_search(&req),
        ("GET", segs) => lookup(segs)
            .map(|file| static_file(file.contents, file.content_type))
            .ok_or_else(|| ApiError::new(404, "not_found", format!("route '{path}' was not found"))),
        (_, ["health"]) | (_, ["version"]) | (_, ["stats"]) | (_, ["movies"]) | (_, ["search"]) => {
            Err(ApiError::new(
                405,
                "method_not_allowed",
                format!("{method} is not allowed for {path}"),
            ))
        }
        _ => Err(ApiError::new(
            404,
            "not_found",
            format!("route '{path}' was not found"),
        )),
    };

    result.unwrap_or_else(|error| json(error.status, &error))
}

fn is_api_path(segments: &[&str]) -> bool {
    matches!(
        segments,
        ["health"] | ["version"] | ["stats"] | ["movies"] | ["search"]
    )
}

fn engine() -> &'static MovieSearch {
    ENGINE.get_or_init(|| {
        MovieSearch::from_json(MOVIES_JSON).expect("embedded movies fixture must be valid JSON")
    })
}

fn handle_search(req: &Request) -> Result<Response, ApiError> {
    let request = parse_body::<SearchRequest>(req)?;
    Ok(json(200, &engine().search(request)))
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
    serde_json::from_slice(req.body()).map_err(|err| {
        ApiError::new(
            400,
            "bad_request",
            format!("request body must be valid JSON: {err}"),
        )
    })
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

fn cors_headers(builder: &mut spin_sdk::http::ResponseBuilder) {
    builder
        .header("access-control-allow-origin", "*")
        .header("access-control-allow-methods", "GET, POST, OPTIONS")
        .header("access-control-allow-headers", "content-type");
}

fn build_response(status: u16, content_type: Option<&str>, body: String) -> Response {
    let mut builder = Response::builder();
    builder.status(status);
    cors_headers(&mut builder);
    if let Some(content_type) = content_type {
        builder.header("content-type", content_type);
    }
    builder.body(body).build()
}

fn build_response_bytes(status: u16, content_type: &str, body: Vec<u8>) -> Response {
    let mut builder = Response::builder();
    builder.status(status);
    cors_headers(&mut builder);
    builder.header("content-type", content_type);
    builder.body(body).build()
}

fn static_file(contents: &[u8], content_type: &str) -> Response {
    build_response_bytes(200, content_type, contents.to_vec())
}

fn json<T>(status: u16, value: &T) -> Response
where
    T: Serialize,
{
    let body = serde_json::to_string(value)
        .unwrap_or_else(|_| "{\"code\":\"internal\",\"message\":\"serialization failed\"}".into());

    build_response(status, Some("application/json"), body)
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ApiError {
    #[serde(skip)]
    status: u16,
    code: &'static str,
    message: String,
}

impl ApiError {
    fn new(status: u16, code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status,
            code,
            message: message.into(),
        }
    }
}
