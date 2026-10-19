use super::*;
use serde_json::{Value, json};
use zixcel_wordpress::*;
pub fn state(status: &str, title: &str) -> Value {
    json!({"id":42,"status":status,"title":{"raw":title},"content":{"raw":"Body"},"modified_gmt":"2026-01-01T00:00:00"})
}
pub fn actor() -> Value {
    let mut v = user();
    v["capabilities"]["publish_posts"] = json!(true);
    v
}
pub fn desired(status: &str) -> ContentItem {
    ContentItem {
        id: 42,
        status: status.into(),
        title: "After".into(),
        content: "Body".into(),
    }
}
pub fn directory() -> tempfile::TempDir {
    use std::os::unix::fs::PermissionsExt;
    let d = tempfile::tempdir().unwrap();
    std::fs::set_permissions(d.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    d
}
