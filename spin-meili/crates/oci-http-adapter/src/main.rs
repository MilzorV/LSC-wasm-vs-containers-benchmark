use std::env;
use std::sync::OnceLock;

use movie_search_core::{MovieSearch, SearchRequest, SuggestRequest, DEFAULT_LIMIT};
use serde::Serialize;
use static_assets::lookup;
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

    if method == "OPTIONS" && is_api_path(&segments) {
        return RoutedResponse {
            request,
            response: with_cors(Response::from_data(Vec::new()).with_status_code(StatusCode(204))),
        };
    }

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
        ("POST", ["suggest"]) => handle_suggest(&mut request),
        ("GET", segs) => lookup(segs)
            .map(|file| static_file(file.contents, file.content_type))
            .ok_or_else(|| {
                ApiError::new(404, "not_found", format!("route '{path}' was not found"))
            }),
        (_, ["health"])
        | (_, ["version"])
        | (_, ["stats"])
        | (_, ["movies"])
        | (_, ["search"])
        | (_, ["suggest"]) => Err(ApiError::new(
            405,
            "method_not_allowed",
            format!("{method} is not allowed for {path}"),
        )),
        _ => Err(ApiError::new(
            404,
            "not_found",
            format!("route '{path}' was not found"),
        )),
    }
    .unwrap_or_else(|error| json(error.status, &error));

    RoutedResponse { request, response }
}

fn is_api_path(segments: &[&str]) -> bool {
    matches!(
        segments,
        ["health"] | ["version"] | ["stats"] | ["movies"] | ["search"] | ["suggest"]
    )
}

fn engine() -> &'static MovieSearch {
    ENGINE.get_or_init(|| {
        MovieSearch::from_json(MOVIES_JSON).expect("embedded movies fixture must be valid JSON")
    })
}

fn handle_search(request: &mut Request) -> Result<Response<std::io::Cursor<Vec<u8>>>, ApiError> {
    let search = read_json_body::<SearchRequest>(request)?;
    Ok(json(200, &engine().search(search)))
}

fn handle_suggest(request: &mut Request) -> Result<Response<std::io::Cursor<Vec<u8>>>, ApiError> {
    let suggest = read_json_body::<SuggestRequest>(request)?;
    Ok(json(200, &engine().suggest(suggest)))
}

fn read_json_body<T>(request: &mut Request) -> Result<T, ApiError>
where
    T: serde::de::DeserializeOwned,
{
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

    serde_json::from_str::<T>(&body).map_err(|err| {
        ApiError::new(
            400,
            "bad_request",
            format!("request body must be valid JSON: {err}"),
        )
    })
}

fn query_usize(query: &str, name: &str) -> Option<usize> {
    query.split('&').find_map(|pair| {
        let (key, value) = pair.split_once('=')?;
        (key == name).then(|| value.parse::<usize>().ok()).flatten()
    })
}

fn with_cors(response: Response<std::io::Cursor<Vec<u8>>>) -> Response<std::io::Cursor<Vec<u8>>> {
    response
        .with_header(cors_header("access-control-allow-origin", "*"))
        .with_header(cors_header(
            "access-control-allow-methods",
            "GET, POST, OPTIONS",
        ))
        .with_header(cors_header("access-control-allow-headers", "content-type"))
}

fn cors_header(name: &str, value: &str) -> Header {
    Header::from_bytes(name, value).expect("static cors header must be valid")
}

fn static_file(contents: &[u8], content_type: &str) -> Response<std::io::Cursor<Vec<u8>>> {
    with_cors(
        Response::from_data(contents.to_vec())
            .with_status_code(StatusCode(200))
            .with_header(
                Header::from_bytes("content-type", content_type)
                    .expect("static header must be valid"),
            ),
    )
}

fn json<T>(status: u16, value: &T) -> Response<std::io::Cursor<Vec<u8>>>
where
    T: Serialize,
{
    let body = serde_json::to_vec(value)
        .unwrap_or_else(|_| br#"{"code":"internal","message":"serialization failed"}"#.to_vec());

    with_cors(
        Response::from_data(body)
            .with_status_code(StatusCode(status))
            .with_header(
                Header::from_bytes("content-type", "application/json")
                    .expect("static header must be valid"),
            ),
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
