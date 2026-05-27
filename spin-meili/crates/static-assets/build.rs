fn main() {
    let dist = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../frontend/dist");
    if !dist.join("index.html").exists() {
        panic!(
            "frontend/dist is missing. Build the UI first:\n  cd frontend && npm install && npm run build"
        );
    }
}
