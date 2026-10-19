use super::*;
use serde_json::{Value, json};
pub fn actor() -> Value {
    let mut v = user();
    v["capabilities"]["upload_files"] = json!(true);
    v
}
pub fn media() -> Value {
    json!({"id":91,"source_url":"https://example.com/uploads/example.png","mime_type":"image/png","alt_text":""})
}
pub fn bytes() -> Vec<u8> {
    b"\x89PNG\r\n\x1a\nsynthetic signature fixture".to_vec()
}
