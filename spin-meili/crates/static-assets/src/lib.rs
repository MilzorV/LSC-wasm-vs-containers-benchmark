use include_dir::{include_dir, Dir};

static FRONTEND: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../../../frontend/dist");

pub struct StaticFile {
    pub contents: &'static [u8],
    pub content_type: &'static str,
}

pub fn lookup(segments: &[&str]) -> Option<StaticFile> {
    let path = match segments {
        [] => "index.html",
        ["demo"] => "demo/index.html",
        ["assets", rest @ ..] if !rest.is_empty() => return lookup_path(&join_segments(rest)),
        _ => return None,
    };

    lookup_path(path)
}

fn join_segments(segments: &[&str]) -> String {
    segments.join("/")
}

fn lookup_path(path: &str) -> Option<StaticFile> {
    let file = FRONTEND.get_file(path)?;
    Some(StaticFile {
        contents: file.contents(),
        content_type: content_type_for_path(path),
    })
}

fn content_type_for_path(path: &str) -> &'static str {
    if path.ends_with(".html") {
        "text/html; charset=utf-8"
    } else if path.ends_with(".js") {
        "application/javascript; charset=utf-8"
    } else if path.ends_with(".css") {
        "text/css; charset=utf-8"
    } else if path.ends_with(".svg") {
        "image/svg+xml"
    } else if path.ends_with(".png") {
        "image/png"
    } else if path.ends_with(".ico") {
        "image/x-icon"
    } else if path.ends_with(".json") {
        "application/json; charset=utf-8"
    } else if path.ends_with(".woff2") {
        "font/woff2"
    } else {
        "application/octet-stream"
    }
}

#[cfg(test)]
mod tests {
    use super::lookup;

    #[test]
    fn serves_app_and_demo_entrypoints() {
        assert!(lookup(&[]).is_some());
        assert!(lookup(&["demo"]).is_some());
    }
}
