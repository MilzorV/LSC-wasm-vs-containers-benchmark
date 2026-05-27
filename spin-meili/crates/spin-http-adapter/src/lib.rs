use std::sync::OnceLock;

use movie_search_core::{MovieSearch, SearchRequest, DEFAULT_LIMIT};
use serde::Serialize;
use spin_sdk::http::{IntoResponse, Method, Request, Response};
use spin_sdk::http_component;

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
