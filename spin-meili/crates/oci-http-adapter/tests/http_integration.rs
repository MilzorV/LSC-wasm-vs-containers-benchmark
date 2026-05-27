use std::net::TcpListener;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::Duration;

fn free_port() -> u16 {
    TcpListener::bind("127.0.0.1:0")
        .expect("bind ephemeral port")
        .local_addr()
        .expect("local addr")
        .port()
}

fn start_server(port: u16) -> Child {
    Command::new(env!("CARGO_BIN_EXE_oci-http-adapter"))
        .env("MOVIE_SEARCH_ADDR", format!("127.0.0.1:{port}"))
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("start oci-http-adapter")
}

fn wait_for_health(port: u16) {
    let url = format!("http://127.0.0.1:{port}/health");
    for _ in 0..120 {
        if ureq::get(&url).call().is_ok() {
            return;
        }
        thread::sleep(Duration::from_millis(100));
    }
    panic!("timed out waiting for {url}");
}

fn stop_server(mut child: Child) {
    let _ = child.kill();
    let _ = child.wait();
}

#[test]
fn health_version_stats_and_search_match_contract() {
    let port = free_port();
    let child = start_server(port);
    wait_for_health(port);
    let base = format!("http://127.0.0.1:{port}");

    let health: serde_json::Value = ureq::get(&format!("{base}/health"))
        .call()
        .expect("health")
        .into_json()
        .expect("health json");
    assert_eq!(health["status"], "available");

    let version: serde_json::Value = ureq::get(&format!("{base}/version"))
        .call()
        .expect("version")
        .into_json()
        .expect("version json");
    assert_eq!(version["engine"], "movie-search-core");
    assert_eq!(version["datasetDocuments"].as_u64(), Some(44_471));

    let stats: serde_json::Value = ureq::get(&format!("{base}/stats"))
        .call()
        .expect("stats")
        .into_json()
        .expect("stats json");
    assert_eq!(stats["documentCount"].as_u64(), Some(44_471));

    let search: serde_json::Value = ureq::post(&format!("{base}/search"))
        .set("content-type", "application/json")
        .send_json(serde_json::json!({"q": "space", "limit": 3}))
        .expect("search")
        .into_json()
        .expect("search json");
    let hit_ids = search["hits"]
        .as_array()
        .expect("hits array")
        .iter()
        .map(|hit| hit["id"].as_u64().expect("hit id"))
        .collect::<Vec<_>>();
    assert_eq!(hit_ids, vec![62, 957, 1542]);

    stop_server(child);
}

#[test]
fn app_is_served_at_root() {
    let port = free_port();
    let child = start_server(port);
    wait_for_health(port);

    let response = ureq::get(&format!("http://127.0.0.1:{port}/"))
        .call()
        .expect("app root");
    assert_eq!(response.status(), 200);
    let body = response.into_string().expect("app body");
    assert!(body.contains("movie-search-compare"));
    assert!(body.contains("Runtime comparison"));

    stop_server(child);
}

#[test]
fn demo_alias_serves_compare_app() {
    let port = free_port();
    let child = start_server(port);
    wait_for_health(port);

    let response = ureq::get(&format!("http://127.0.0.1:{port}/demo"))
        .call()
        .expect("demo route");
    assert_eq!(response.status(), 200);
    let body = response.into_string().expect("demo body");
    assert!(body.contains("movie-search-compare"));

    stop_server(child);
}

#[test]
fn benchmarks_page_is_served() {
    let port = free_port();
    let child = start_server(port);
    wait_for_health(port);

    let response = ureq::get(&format!("http://127.0.0.1:{port}/benchmarks"))
        .call()
        .expect("benchmarks route");
    assert_eq!(response.status(), 200);
    let body = response.into_string().expect("benchmarks body");
    assert!(body.contains("movie-search-benchmarks"));

    stop_server(child);
}

#[test]
fn assets_are_served_under_assets_prefix() {
    let port = free_port();
    let child = start_server(port);
    wait_for_health(port);

    let root = ureq::get(&format!("http://127.0.0.1:{port}/"))
        .call()
        .expect("root")
        .into_string()
        .expect("root body");
    let asset_path = root
        .split("href=\"/assets/")
        .nth(1)
        .and_then(|rest| rest.split('"').next())
        .expect("built index should reference /assets/");
    let response = ureq::get(&format!("http://127.0.0.1:{port}/assets/{asset_path}"))
        .call()
        .expect("asset");
    assert_eq!(response.status(), 200);

    stop_server(child);
}

#[test]
fn unknown_route_returns_not_found() {
    let port = free_port();
    let child = start_server(port);
    wait_for_health(port);

    let response = ureq::get(&format!("http://127.0.0.1:{port}/missing")).call();
    match response {
        Err(ureq::Error::Status(404, _)) => {}
        Ok(resp) => panic!("expected 404, got {}", resp.status()),
        Err(err) => panic!("unexpected error: {err}"),
    }

    stop_server(child);
}
