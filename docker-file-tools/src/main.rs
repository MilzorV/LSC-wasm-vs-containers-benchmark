use axum::{
    body::Bytes,
    extract::Query,
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use image::imageops::FilterType;
use image::{DynamicImage, ImageFormat};
use serde::Deserialize;
use serde::Serialize;
use serde_json::{json, Map, Value};
use std::collections::BTreeSet;
use std::env;
use std::io::Cursor;
use std::net::SocketAddr;
use std::time::Instant;
use tower_http::trace::TraceLayer;

#[derive(Serialize)]
struct ApiError {
    code: &'static str,
    message: String,
}

#[derive(Serialize)]
struct Health {
    status: &'static str,
}

#[derive(Serialize)]
struct Version {
    name: &'static str,
    version: &'static str,
    runtime_target: &'static str,
}

#[derive(Debug, Deserialize)]
struct ImageParams {
    width: Option<u32>,
    height: Option<u32>,
    format: Option<String>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = env::var("PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(8081);

    let app = Router::new()
        .route("/", get(routes_handler))
        .route("/health", get(health_handler))
        .route("/version", get(version_handler))
        .route("/routes", get(routes_handler))
        .route("/ping", get(ping_handler))
        .route("/echo", post(echo_handler))
        .route("/validate/json", post(validate_json_handler))
        .route("/convert/json-to-csv", post(json_to_csv_handler))
        .route("/convert/csv-to-json", post(csv_to_json_handler))
        .route("/image/metadata", post(image_metadata_handler))
        .route("/image/grayscale", post(image_grayscale_handler))
        .route("/image/resize", post(image_resize_handler))
        .fallback(not_found)
        .layer(TraceLayer::new_for_http());

    let addr: SocketAddr = format!("{host}:{port}")
        .parse()
        .expect("HOST and PORT must form a valid socket address");

    println!("docker-file-tools listening on http://{addr}");

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("bind listener");

    axum::serve(listener, app).await.expect("serve app");
}

async fn health_handler() -> impl IntoResponse {
    Json(Health { status: "available" })
}

async fn version_handler() -> impl IntoResponse {
    Json(Version {
        name: "docker-file-tools",
        version: env!("CARGO_PKG_VERSION"),
        runtime_target: std::env::consts::ARCH,
    })
}

async fn ping_handler() -> impl IntoResponse {
    Json(json!({ "pong": true }))
}

async fn echo_handler(body: Bytes) -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/octet-stream"),
    );
    (StatusCode::OK, headers, body)
}

async fn routes_handler() -> impl IntoResponse {
    Json(routes())
}

async fn validate_json_handler(body: Bytes) -> Response {
    let started = Instant::now();
    let response = validate_json(&body);
    attach_processing_time(response, started)
}

async fn json_to_csv_handler(body: Bytes) -> Response {
    let started = Instant::now();
    let response = json_to_csv(&body);
    attach_processing_time(response, started)
}

async fn csv_to_json_handler(body: Bytes) -> Response {
    let started = Instant::now();
    let response = csv_to_json(&body);
    attach_processing_time(response, started)
}

async fn image_metadata_handler(body: Bytes) -> Response {
    let started = Instant::now();
    let response = image_metadata(&body);
    attach_processing_time(response, started)
}

async fn image_grayscale_handler(Query(params): Query<ImageParams>, body: Bytes) -> Response {
    let started = Instant::now();
    let response = image_transform(&body, params, ImageOp::Grayscale);
    attach_processing_time(response, started)
}

async fn image_resize_handler(Query(params): Query<ImageParams>, body: Bytes) -> Response {
    let started = Instant::now();
    let response = image_transform(&body, params, ImageOp::Resize);
    attach_processing_time(response, started)
}

async fn not_found() -> impl IntoResponse {
    json_error(404, "not_found", "route was not found")
}

fn routes() -> Value {
    json!({
        "routes": [
            { "method": "GET", "path": "/health" },
            { "method": "GET", "path": "/version" },
            { "method": "GET", "path": "/routes" },
            { "method": "GET", "path": "/ping" },
            { "method": "POST", "path": "/echo", "body": "raw bytes" },
            { "method": "POST", "path": "/validate/json", "body": "JSON object/value or {schema, document}" },
            { "method": "POST", "path": "/convert/json-to-csv", "body": "JSON array of flat objects" },
            { "method": "POST", "path": "/convert/csv-to-json", "body": "CSV with headers" },
            { "method": "POST", "path": "/image/metadata", "body": "PNG/JPEG bytes" },
            { "method": "POST", "path": "/image/grayscale?format=png|jpeg", "body": "PNG/JPEG bytes" },
            { "method": "POST", "path": "/image/resize?width=256&height=256&format=png|jpeg", "body": "PNG/JPEG bytes" }
        ]
    })
}

fn json_ok<T: Serialize>(value: &T) -> Response {
    json_status(StatusCode::OK, value)
}

fn json_status<T: Serialize>(status: StatusCode, value: &T) -> Response {
    let body = serde_json::to_vec_pretty(value)
        .unwrap_or_else(|_| b"{\"error\":\"serialization\"}".to_vec());

    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/json; charset=utf-8"),
    );

    (status, headers, body).into_response()
}

fn bytes_response(status: StatusCode, content_type: &'static str, body: Vec<u8>) -> Response {
    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, HeaderValue::from_static(content_type));
    (status, headers, body).into_response()
}

fn text_response(status: StatusCode, content_type: &'static str, body: String) -> Response {
    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, HeaderValue::from_static(content_type));
    (status, headers, body).into_response()
}

fn json_error(status: u16, code: &'static str, message: impl Into<String>) -> Response {
    let status = StatusCode::from_u16(status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    json_status(
        status,
        &ApiError {
            code,
            message: message.into(),
        },
    )
}

fn attach_processing_time(mut response: Response, started: Instant) -> Response {
    let elapsed_us = started.elapsed().as_micros().to_string();

    if let Ok(value) = HeaderValue::from_str(&elapsed_us) {
        response
            .headers_mut()
            .insert("x-internal-processing-us", value);
    }

    response
}

fn validate_json(body: &[u8]) -> Response {
    let parsed: Value = match serde_json::from_slice(body) {
        Ok(value) => value,
        Err(err) => {
            return json_status(
                StatusCode::BAD_REQUEST,
                &json!({
                    "valid": false,
                    "error": err.to_string()
                }),
            );
        }
    };

    if let Some(obj) = parsed.as_object() {
        if obj.contains_key("schema") && obj.contains_key("document") {
            let schema = &obj["schema"];
            let document = &obj["document"];
            let errors = validate_simple_schema(schema, document);
            return json_ok(
                &json!({
                    "valid": errors.is_empty(),
                    "errors": errors
                }),
            );
        }
    }

    json_ok(
        &json!({
            "valid": true,
            "kind": value_kind(&parsed)
        }),
    )
}

fn validate_simple_schema(schema: &Value, document: &Value) -> Vec<String> {
    let mut errors = Vec::new();

    if let Some(expected_type) = schema.get("type").and_then(Value::as_str) {
        if !matches_json_type(document, expected_type) {
            errors.push(format!(
                "document type is {}, expected {}",
                value_kind(document),
                expected_type
            ));
        }
    }

    if let Some(required) = schema.get("required").and_then(Value::as_array) {
        if let Some(doc_obj) = document.as_object() {
            for field in required.iter().filter_map(Value::as_str) {
                if !doc_obj.contains_key(field) {
                    errors.push(format!("missing required field '{}'", field));
                }
            }
        } else {
            errors.push("required can only be checked against an object document".to_string());
        }
    }

    if let (Some(properties), Some(doc_obj)) = (
        schema.get("properties").and_then(Value::as_object),
        document.as_object(),
    ) {
        for (field, rules) in properties {
            if let Some(value) = doc_obj.get(field) {
                if let Some(expected_type) = rules.get("type").and_then(Value::as_str) {
                    if !matches_json_type(value, expected_type) {
                        errors.push(format!(
                            "field '{}' type is {}, expected {}",
                            field,
                            value_kind(value),
                            expected_type
                        ));
                    }
                }
            }
        }
    }

    errors
}

fn matches_json_type(value: &Value, expected: &str) -> bool {
    match expected {
        "null" => value.is_null(),
        "boolean" | "bool" => value.is_boolean(),
        "number" => value.is_number(),
        "integer" | "int" => value.as_i64().is_some() || value.as_u64().is_some(),
        "string" => value.is_string(),
        "array" => value.is_array(),
        "object" => value.is_object(),
        _ => false,
    }
}

fn value_kind(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

fn json_to_csv(body: &[u8]) -> Response {
    let value: Value = match serde_json::from_slice(body) {
        Ok(value) => value,
        Err(err) => return json_error(400, "invalid_json", err.to_string()),
    };

    let rows = match value.as_array() {
        Some(rows) => rows,
        None => return json_error(400, "invalid_input", "expected a JSON array of flat objects"),
    };

    let mut headers = BTreeSet::new();
    for row in rows {
        let Some(obj) = row.as_object() else {
            return json_error(400, "invalid_input", "each array item must be an object");
        };

        for key in obj.keys() {
            headers.insert(key.clone());
        }
    }

    let headers_vec: Vec<String> = headers.into_iter().collect();
    let mut writer = csv::Writer::from_writer(vec![]);

    if let Err(err) = writer.write_record(&headers_vec) {
        return json_error(500, "csv_write_error", err.to_string());
    }

    for row in rows {
        let obj = row.as_object().unwrap();
        let record = headers_vec
            .iter()
            .map(|key| csv_cell(obj.get(key).unwrap_or(&Value::Null)))
            .collect::<Vec<_>>();

        if let Err(err) = writer.write_record(record) {
            return json_error(500, "csv_write_error", err.to_string());
        }
    }

    match writer.into_inner() {
        Ok(bytes) => text_response(
            StatusCode::OK,
            "text/csv; charset=utf-8",
            String::from_utf8_lossy(&bytes).to_string(),
        ),
        Err(err) => json_error(500, "csv_finalize_error", err.to_string()),
    }
}

fn csv_cell(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::Bool(v) => v.to_string(),
        Value::Number(v) => v.to_string(),
        Value::String(v) => v.clone(),
        Value::Array(_) | Value::Object(_) => serde_json::to_string(value).unwrap_or_default(),
    }
}

fn csv_to_json(body: &[u8]) -> Response {
    let mut reader = csv::Reader::from_reader(body);
    let headers = match reader.headers() {
        Ok(headers) => headers.clone(),
        Err(err) => return json_error(400, "invalid_csv", err.to_string()),
    };

    let mut out = Vec::new();

    for result in reader.records() {
        let record = match result {
            Ok(record) => record,
            Err(err) => return json_error(400, "invalid_csv_record", err.to_string()),
        };

        let mut map = Map::new();
        for (idx, header) in headers.iter().enumerate() {
            let raw = record.get(idx).unwrap_or("");
            map.insert(header.to_string(), parse_csv_value(raw));
        }
        out.push(Value::Object(map));
    }

    json_ok(&out)
}

fn parse_csv_value(raw: &str) -> Value {
    if raw.is_empty() {
        Value::Null
    } else if raw.eq_ignore_ascii_case("true") {
        Value::Bool(true)
    } else if raw.eq_ignore_ascii_case("false") {
        Value::Bool(false)
    } else if let Ok(n) = raw.parse::<i64>() {
        json!(n)
    } else if let Ok(n) = raw.parse::<f64>() {
        json!(n)
    } else if (raw.starts_with('{') && raw.ends_with('}'))
        || (raw.starts_with('[') && raw.ends_with(']'))
    {
        serde_json::from_str(raw).unwrap_or_else(|_| Value::String(raw.to_string()))
    } else {
        Value::String(raw.to_string())
    }
}

fn image_metadata(body: &[u8]) -> Response {
    match image::load_from_memory(body) {
        Ok(img) => json_ok(
            &json!({
                "width": img.width(),
                "height": img.height(),
                "color": format!("{:?}", img.color())
            }),
        ),
        Err(err) => json_error(400, "invalid_image", err.to_string()),
    }
}

enum ImageOp {
    Grayscale,
    Resize,
}

fn image_transform(body: &[u8], params: ImageParams, op: ImageOp) -> Response {
    let img = match image::load_from_memory(body) {
        Ok(img) => img,
        Err(err) => return json_error(400, "invalid_image", err.to_string()),
    };

    let out_format = image_format_from_param(params.format.as_deref()).unwrap_or(ImageFormat::Png);

    let transformed = match op {
        ImageOp::Grayscale => img.grayscale(),
        ImageOp::Resize => {
            let width = match params.width {
                Some(v) if v > 0 => v,
                _ => {
                    return json_error(
                        400,
                        "missing_width",
                        "query parameter 'width' must be a positive integer",
                    )
                }
            };

            let height = match params.height {
                Some(v) if v > 0 => v,
                _ => {
                    return json_error(
                        400,
                        "missing_height",
                        "query parameter 'height' must be a positive integer",
                    )
                }
            };

            img.resize_exact(width, height, FilterType::Triangle)
        }
    };

    encode_image(transformed, out_format)
}

fn encode_image(img: DynamicImage, format: ImageFormat) -> Response {
    let mut cursor = Cursor::new(Vec::new());

    if let Err(err) = img.write_to(&mut cursor, format) {
        return json_error(500, "image_encode_error", err.to_string());
    }

    let content_type = match format {
        ImageFormat::Jpeg => "image/jpeg",
        _ => "image/png",
    };

    bytes_response(StatusCode::OK, content_type, cursor.into_inner())
}

fn image_format_from_param(format: Option<&str>) -> Option<ImageFormat> {
    match format {
        Some("jpg") | Some("jpeg") => Some(ImageFormat::Jpeg),
        Some("png") | None => Some(ImageFormat::Png),
        Some(_) => None,
    }
}
