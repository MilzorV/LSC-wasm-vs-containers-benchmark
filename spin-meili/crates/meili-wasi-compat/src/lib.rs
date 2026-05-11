//! Compatibility boundary for the upstream Meilisearch-on-Spin port.
//!
//! The current Week 1 Spin service still uses the local subset engine as a
//! fallback. This crate is intentionally small: it names the porting boundary
//! where upstream Meilisearch APIs, storage, and runtime assumptions should be
//! adapted once the `wasm32-wasip2` feasibility checks identify the highest
//! reusable upstream layer.

pub const UPSTREAM_MEILISEARCH_TAG: &str = "v1.43.0";
pub const OCI_IMAGE: &str = "getmeili/meilisearch:v1.43.0";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PortingBoundary {
    pub upstream_tag: &'static str,
    pub oci_image: &'static str,
    pub expected_blockers: &'static [&'static str],
}

impl PortingBoundary {
    pub fn current() -> Self {
        Self {
            upstream_tag: UPSTREAM_MEILISEARCH_TAG,
            oci_image: OCI_IMAGE,
            expected_blockers: &[
                "LMDB/heed memory-mapped storage",
                "native HTTP server runtime",
                "background task scheduler",
                "filesystem-oriented snapshots/dumps",
                "telemetry and host process configuration",
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PortingBoundary;

    #[test]
    fn records_pinned_upstream_version() {
        let boundary = PortingBoundary::current();

        assert_eq!(boundary.upstream_tag, "v1.43.0");
        assert_eq!(boundary.oci_image, "getmeili/meilisearch:v1.43.0");
        assert!(boundary
            .expected_blockers
            .iter()
            .any(|blocker| blocker.contains("LMDB")));
    }
}
