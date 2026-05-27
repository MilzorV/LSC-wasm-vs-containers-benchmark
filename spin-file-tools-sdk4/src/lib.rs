use anyhow::Result;
use image::imageops::FilterType;
use image::{DynamicImage, ImageFormat};
use serde::Serialize;
use serde_json::{json, Map, Value};
use spin_sdk::http::{IntoResponse, Method, Request, Response};
use spin_sdk::http_component;
use std::collections::BTreeSet;
use std::io::Cursor;

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

#[http_component]
fn handle_spin_file_tools(req: Request) -> Result<impl IntoResponse> {
    Ok(route(req))
}

fn route(req: Request) -> Response {
    let method = req.method().clone();
    let path = req.path().trim_end_matches('/').to_string();
    let query = req.query();
    let body = req.body();

    match (method, path.as_str()) {
        (Method::Get, "") | (Method::Get, "/") => json_response(200, &routes()),
        (Method::Get, "/health") => json_response(200, &Health { status: "available" }),
        (Method::Get, "/version") => json_response(
            200,
            &Version {
                name: "spin-file-tools",
                version: env!("CARGO_PKG_VERSION"),
                runtime_target: "wasm32-wasip2",
            },
        ),
        (Method::Get, "/routes") => json_response(200, &routes()),

        (Method::Post, "/validate/json") => validate_json(body),
        (Method::Post, "/convert/json-to-csv") => json_to_csv(body),
        (Method::Post, "/convert/csv-to-json") => csv_to_json(body),
        (Method::Post, "/image/metadata") => image_metadata(body),
        (Method::Post, "/image/grayscale") => image_transform(body, query, ImageOp::Grayscale),
        (Method::Post, "/image/resize") => image_transform(body, query, ImageOp::Resize),

        (
            _,
            "/health"
            | "/version"
            | "/routes"
            | "/validate/json"
            | "/convert/json-to-csv"
            | "/convert/csv-to-json"
            | "/image/metadata"
            | "/image/grayscale"
            | "/image/resize",
        ) => json_error(405, "method_not_allowed", "method is not allowed for this route"),

        _ => json_error(404, "not_found", format!("route '{}' was not found", path)),
    }
}

fn routes() -> Value {
    json!({
        "routes": [
            { "method": "GET", "path": "/health" },
            { "method": "GET", "path": "/version" },
            { "method": "GET", "path": "/routes" },
            { "method": "POST", "path": "/validate/json", "body": "JSON object/value or {schema, document}" },
            { "method": "POST", "path": "/convert/json-to-csv", "body": "JSON array of flat objects" },
            { "method": "POST", "path": "/convert/csv-to-json", "body": "CSV with headers" },
            { "method": "POST", "path": "/image/metadata", "body": "PNG/JPEG bytes" },
            { "method": "POST", "path": "/image/grayscale?format=png|jpeg", "body": "PNG/JPEG bytes" },
            { "method": "POST", "path": "/image/resize?width=256&height=256&format=png|jpeg", "body": "PNG/JPEG bytes" }
        ]
    })
}

fn json_response<T: Serialize>(status: u16, value: &T) -> Response {
    let body = serde_json::to_vec_pretty(value)
        .unwrap_or_else(|_| b"{\"error\":\"serialization\"}".to_vec());

    Response::builder()
        .status(status)
        .header("content-type", "application/json; charset=utf-8")
        .body(body)
        .build()
}

fn bytes_response(status: u16, content_type: &'static str, body: Vec<u8>) -> Response {
    Response::builder()
        .status(status)
        .header("content-type", content_type)
        .body(body)
        .build()
}

fn text_response(status: u16, content_type: &'static str, body: String) -> Response {
    Response::builder()
        .status(status)
        .header("content-type", content_type)
        .body(body)
        .build()
}

fn json_error(status: u16, code: &'static str, message: impl Into<String>) -> Response {
    json_response(
        status,
        &ApiError {
            code,
            message: message.into(),
        },
    )
}

fn validate_json(body: &[u8]) -> Response {
    let parsed: Value = match serde_json::from_slice(body) {
        Ok(value) => value,
        Err(err) => {
            return json_response(
                400,
                &json!({
                    "valid": false,
                    "error": err.to_string()
                }),
            );
        }
    };

    // Optional lightweight schema mode:
    // {
    //   "schema": {
    //     "type": "object",
    //     "required": ["id", "title"],
    //     "properties": { "id": {"type":"integer"}, "title": {"type":"string"} }
    //   },
    //   "document": { ... }
    // }
    if let Some(obj) = parsed.as_object() {
        if obj.contains_key("schema") && obj.contains_key("document") {
            let schema = &obj["schema"];
            let document = &obj["document"];
            let errors = validate_simple_schema(schema, document);
            return json_response(
                200,
                &json!({
                    "valid": errors.is_empty(),
                    "errors": errors
                }),
            );
        }
    }

    json_response(
        200,
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
            200,
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

    json_response(200, &out)
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
        Ok(img) => json_response(
            200,
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

fn image_transform(body: &[u8], query: &str, op: ImageOp) -> Response {
    let img = match image::load_from_memory(body) {
        Ok(img) => img,
        Err(err) => return json_error(400, "invalid_image", err.to_string()),
    };

    let out_format = image_format_from_query(query).unwrap_or(ImageFormat::Png);
    let transformed = match op {
        ImageOp::Grayscale => img.grayscale(),
        ImageOp::Resize => {
            let width = match query_param_u32(query, "width") {
                Some(v) if v > 0 => v,
                _ => {
                    return json_error(
                        400,
                        "missing_width",
                        "query parameter 'width' must be a positive integer",
                    )
                }
            };

            let height = match query_param_u32(query, "height") {
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

    bytes_response(200, content_type, cursor.into_inner())
}

fn image_format_from_query(query: &str) -> Option<ImageFormat> {
    match query_param(query, "format").as_deref() {
        Some("jpg") | Some("jpeg") => Some(ImageFormat::Jpeg),
        Some("png") | None => Some(ImageFormat::Png),
        Some(_) => None,
    }
}

fn query_param_u32(query: &str, key: &str) -> Option<u32> {
    query_param(query, key)?.parse::<u32>().ok()
}

fn query_param(query: &str, key: &str) -> Option<String> {
    query
        .split('&')
        .filter(|part| !part.is_empty())
        .find_map(|part| {
            let mut pieces = part.splitn(2, '=');
            let k = pieces.next().unwrap_or("");
            let v = pieces.next().unwrap_or("");

            if k == key {
                Some(percent_decode(v))
            } else {
                None
            }
        })
}

fn percent_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;

    while i < bytes.len() {
        match bytes[i] {
            b'+' => {
                out.push(b' ');
                i += 1;
            }
            b'%' if i + 2 < bytes.len() => {
                if let (Some(h), Some(l)) = (hex(bytes[i + 1]), hex(bytes[i + 2])) {
                    out.push(h * 16 + l);
                    i += 3;
                } else {
                    out.push(bytes[i]);
                    i += 1;
                }
            }
            b => {
                out.push(b);
                i += 1;
            }
        }
    }

    String::from_utf8_lossy(&out).to_string()
}

fn hex(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}
