use include_dir::{include_dir, Dir};

static FRONTEND: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../../../frontend/dist");

pub struct StaticFile {
    pub contents: &'static [u8],
    pub content_type: &'static str,
}

pub fn lookup(segments: &[&str]) -> Option<StaticFile> {
    let path = match segments {
        [] => "index.html",
        ["demo"] | ["demo", "index.html"] => "index.html",
        ["benchmarks"] | ["benchmarks", "index.html"] => "benchmarks/index.html",
        ["assets", rest @ ..] if !rest.is_empty() => {
            return lookup_path(&format!("assets/{}", join_segments(rest)));
        }
        ["benchmark-data", rest @ ..] if !rest.is_empty() => {
            return lookup_path(&format!("benchmark-data/{}", join_segments(rest)));
        }
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
    use super::{lookup, FRONTEND};

    #[test]
    fn serves_app_and_benchmark_entrypoints() {
        assert!(lookup(&[]).is_some());
        assert!(lookup(&["demo"]).is_some());
        assert!(lookup(&["benchmarks"]).is_some());
    }

    #[test]
    fn serves_hashed_assets_under_assets_prefix() {
        let assets_dir = FRONTEND
            .get_dir("assets")
            .expect("run make frontend-build before cargo test");
        let file = assets_dir
            .files()
            .next()
            .expect("dist/assets must contain built files");
        let name = file
            .path()
            .file_name()
            .and_then(|n| n.to_str())
            .expect("asset file name");
        let served = lookup(&["assets", name]).expect("GET /assets/{name} must resolve");
        assert!(!served.contents.is_empty());
    }

    #[test]
    fn serves_benchmark_data_when_present() {
        if lookup(&["benchmark-data", "dashboard.json"]).is_some() {
            let json = lookup(&["benchmark-data", "dashboard.json"]).unwrap();
            assert_eq!(json.content_type, "application/json; charset=utf-8");
        }
    }
}
