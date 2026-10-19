use super::super::*;
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, VecDeque},
    os::unix::fs::PermissionsExt,
    path::Path,
};
use zixcel_wordpress::{Method, Profile, Transport, WordPress};
pub struct Fake(pub VecDeque<Value>);
impl Transport for Fake {
    fn origin(&self) -> &str {
        "https://example.test"
    }
    fn request(&mut self, _: Method, _: &str, _: Option<&Value>, _: bool) -> Result<Value> {
        Ok(self.0.pop_front().expect("unexpected fixture API call"))
    }
    fn upload(&mut self, _: &str, _: &str, _: &[u8]) -> Result<Value> {
        Ok(self.0.pop_front().expect("unexpected fixture upload"))
    }
}
pub fn client(rows: Vec<Value>) -> WordPress<Fake> {
    WordPress::new(
        Profile {
            id: "example".into(),
            origin: "https://example.test".into(),
        },
        Fake(rows.into()),
    )
    .unwrap()
}
pub fn discovery() -> Value {
    json!({"home":"https://example.test","url":"https://example.test","namespaces":["wp/v2"],"authentication":{"application-passwords":{"endpoints":{"authorization":"https://example.test/wp-admin/authorize-application.php"}}}})
}
pub fn user() -> Value {
    json!({"id":7,"capabilities":{"edit_posts":true,"edit_pages":true,"publish_posts":true,"upload_files":true}})
}
pub fn content() -> Value {
    json!({"id":42,"status":"draft","title":{"raw":"Synthetic"},"content":{"raw":"Body"}})
}
pub fn media() -> Value {
    json!({"id":7,"source_url":"https://example.test/x.png","mime_type":"image/png","alt_text":""})
}
pub fn op(id: &str) -> &'static Op {
    crate::provider_ops::ops().find(|o| o.id == id).unwrap()
}
pub fn args() -> BTreeMap<String, String> {
    [
        ("profile", "example"),
        ("origin", "https://example.test"),
        ("collection", "default"),
    ]
    .into_iter()
    .map(|(k, v)| (k.into(), v.into()))
    .collect()
}
pub fn binding() -> Binding {
    Binding::for_catalog(
        "zixcel-wordpress",
        env!("CARGO_PKG_VERSION"),
        &crate::provider_manifest::manifest().unwrap(),
        &"a".repeat(64),
    )
}
pub fn save(path: &Path, value: &Value) {
    std::fs::write(path, serde_json::to_vec(value).unwrap()).unwrap();
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600)).unwrap();
}
