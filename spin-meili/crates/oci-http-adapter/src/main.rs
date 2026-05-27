use std::env;
use std::sync::OnceLock;

use movie_search_core::{MovieSearch, SearchRequest, DEFAULT_LIMIT};
use serde::Serialize;
use tiny_http::{Header, Request, Response, Server, StatusCode};

const MOVIES_JSON: &str = include_str!("../../../../fixtures/movies.json");
const DEFAULT_ADDR: &str = "0.0.0.0:7700";

static ENGINE: OnceLock<MovieSearch> = OnceLock::new();

fn main() -> anyhow::Result<()> {
    let addr = env::var("MOVIE_SEARCH_ADDR").unwrap_or_else(|_| DEFAULT_ADDR.to_string());
    let server = Server::http(&addr).map_err(|err| anyhow::anyhow!("{err}"))?;

    eprintln!("movie-search OCI adapter listening on http://{addr}");
    for request in server.incoming_requests() {
        handle(request);
    }

    Ok(())
}

fn handle(request: Request) {
    let response = route(request);
    let _ = response.request.respond(response.response);
}

fn route(mut request: Request) -> RoutedResponse {
    let method = request.method().to_string();
    let url = request.url().to_string();
    let (path, query) = url.split_once('?').unwrap_or((url.as_str(), ""));
    let segments = path
        .trim_matches('/')
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>();

    let response = match (method.as_str(), segments.as_slice()) {
        ("GET", ["health"]) => Ok(json(200, &engine().health())),
        ("GET", ["version"]) => Ok(json(200, &engine().version())),
        ("GET", ["stats"]) => Ok(json(200, &engine().stats())),
        ("GET", ["movies"]) => {
            let offset = query_usize(query, "offset").unwrap_or(0);
            let limit = query_usize(query, "limit").unwrap_or(DEFAULT_LIMIT);
            Ok(json(200, &engine().movies(offset, limit)))
        }
        ("POST", ["search"]) => handle_search(&mut request),
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
    }
    .unwrap_or_else(|error| json(error.status, &error));

    RoutedResponse { request, response }
}

fn engine() -> &'static MovieSearch {
    ENGINE.get_or_init(|| {
        MovieSearch::from_json(MOVIES_JSON).expect("embedded movies fixture must be valid JSON")
    })
}

fn handle_search(request: &mut Request) -> Result<Response<std::io::Cursor<Vec<u8>>>, ApiError> {
    let mut body = String::new();
    request
        .as_reader()
        .read_to_string(&mut body)
        .map_err(|err| {
            ApiError::new(
                400,
                "bad_request",
                format!("failed to read request body: {err}"),
            )
        })?;

    let search = serde_json::from_str::<SearchRequest>(&body).map_err(|err| {
        ApiError::new(
            400,
            "bad_request",
            format!("request body must be valid JSON: {err}"),
        )
    })?;

    Ok(json(200, &engine().search(search)))
}

fn query_usize(query: &str, name: &str) -> Option<usize> {
    query.split('&').find_map(|pair| {
        let (key, value) = pair.split_once('=')?;
        (key == name).then(|| value.parse::<usize>().ok()).flatten()
    })
}

fn json<T>(status: u16, value: &T) -> Response<std::io::Cursor<Vec<u8>>>
where
    T: Serialize,
{
    let body = serde_json::to_vec(value)
        .unwrap_or_else(|_| br#"{"code":"internal","message":"serialization failed"}"#.to_vec());

    Response::from_data(body)
        .with_status_code(StatusCode(status))
        .with_header(
            Header::from_bytes("content-type", "application/json")
                .expect("static header must be valid"),
        )
}

struct RoutedResponse {
    request: Request,
    response: Response<std::io::Cursor<Vec<u8>>>,
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
