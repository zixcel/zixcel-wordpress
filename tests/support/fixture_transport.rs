use serde_json::{Value, json};
use std::collections::VecDeque;
use zixcel_wordpress::{Error, Method, Result, Transport};
pub struct Fixture(VecDeque<Result<Value>>);
impl Transport for Fixture {
    fn origin(&self) -> &str {
        "https://example.test"
    }
    fn request(
        &mut self,
        _: Method,
        path: &str,
        _: Option<&Value>,
        authenticated: bool,
    ) -> Result<Value> {
        if !path.starts_with("/wp-json/")
            || path.contains("..")
            || path != "/wp-json/" && !authenticated
        {
            return Err(Error::UnsafeTarget);
        }
        self.0.pop_front().ok_or(Error::InvalidInput)?
    }
    fn upload(&mut self, _: &str, _: &str, _: &[u8]) -> Result<Value> {
        self.0.pop_front().ok_or(Error::InvalidInput)?
    }
}
fn content(item: &Value) -> Value {
    json!({"id":item["id"],"status":item["status"],"title":{"raw":item["title"]},"content":{"raw":item["content"]},"modified_gmt":"2026-01-01T00:00:00"})
}
pub fn for_operation(op: &str, source: Option<&[u8]>) -> Result<Fixture> {
    let value: Value = source
        .map(serde_json::from_slice)
        .transpose()
        .map_err(|_| Error::InvalidInput)?
        .unwrap_or(Value::Null);
    let discovery = json!({"home":"https://example.test","url":"https://example.test","namespaces":["wp/v2"],"authentication":{"application-passwords":{"endpoints":{"authorization":"https://example.test/wp-admin/authorize-application.php"}}}});
    let user = json!({"id":7,"capabilities":{"edit_posts":true,"edit_pages":true,"publish_posts":true,"publish_pages":true,"upload_files":true}});
    let mut rows = vec![Ok(discovery), Ok(user)];
    let media = json!({"id":8,"source_url":"https://example.test/x.png","mime_type":"image/png","alt_text":""});
    let identity = json!({"uuid":"aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee","name":"Synthetic"});
    match op {
        "draft-plan" | "media-plan" => {}
        "draft-apply" => {
            let item = content(
                &json!({"id":42,"status":"draft","title":value["title"],"content":value["content"]}),
            );
            rows.extend([Ok(item.clone()), Ok(item)]);
        }
        "content-plan" => {
            rows.push(Ok(content(
                &json!({"id":42,"status":"draft","title":"Before","content":"Before"}),
            )));
        }
        "content-apply" => {
            let after = content(&value["after"]);
            let before = content(&value["before"]["item"]);
            rows.extend([Ok(before), Ok(after.clone()), Ok(after)]);
        }
        "content-reconcile" => {
            rows.push(Ok(content(&value["after"])));
        }
        "media-apply" => rows.extend([Ok(media.clone()), Ok(media)]),
        "media-list" => rows.push(Ok(json!([media]))),
        "media-get" => rows.push(Ok(media)),
        "revoke-plan" | "auth-status" => rows.push(Ok(identity)),
        "revoke-apply" => rows.extend([
            Ok(identity),
            Ok(json!({"deleted":true})),
            Err(Error::Status(401)),
        ]),
        "posts-list" | "pages-list" => rows.push(Ok(
            json!([{"id":42,"status":"draft","title":{"raw":"Synthetic"}}]),
        )),
        "posts-get" | "pages-get" => rows.push(Ok(content(
            &json!({"id":42,"status":"draft","title":"Synthetic","content":"Body"}),
        ))),
        "login" => {}
        _ => return Err(Error::Unsupported),
    }
    Ok(Fixture(rows.into()))
}
