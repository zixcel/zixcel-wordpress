#![allow(dead_code)]
pub mod mutations;
use serde_json::{Value, json};
use std::{cell::RefCell, collections::VecDeque, rc::Rc};
use zixcel_wordpress::*;
pub type Calls = Rc<RefCell<Vec<(Method, String, bool)>>>;
pub struct Fake {
    pub origin: String,
    pub responses: VecDeque<Result<Value>>,
    pub calls: Calls,
}
impl Transport for Fake {
    fn origin(&self) -> &str {
        &self.origin
    }
    fn request(
        &mut self,
        method: Method,
        path: &str,
        body: Option<&Value>,
        authenticated: bool,
    ) -> Result<Value> {
        self.calls
            .borrow_mut()
            .push((method, path.into(), authenticated));
        if method == Method::Post {
            assert!(matches!(
                body.unwrap()["status"].as_str(),
                Some("draft" | "publish")
            ));
        }
        self.responses
            .pop_front()
            .expect("no unexpected network request")
    }
}
pub fn profile() -> Profile {
    Profile {
        id: "example".into(),
        origin: "https://example.com".into(),
    }
}
pub fn discovery() -> Value {
    json!({"home":"https://example.com","url":"https://example.com","namespaces":["wp/v2"],
    "authentication":{"application-passwords":{"endpoints":{"authorization":"https://example.com/wp-admin/authorize-application.php"}}}})
}
pub fn user() -> Value {
    json!({"id":7,"capabilities":{"edit_posts":true,"edit_pages":true}})
}
pub fn draft() -> Value {
    json!({"id":42,"status":"draft","title":{"raw":"Synthetic acceptance"},"content":{"raw":"Not a public article."}})
}
pub fn fake(responses: Vec<Result<Value>>) -> (WordPress<Fake>, Calls) {
    let calls = Rc::new(RefCell::new(Vec::new()));
    (
        WordPress::new(
            profile(),
            Fake {
                origin: "https://example.com".into(),
                responses: responses.into(),
                calls: calls.clone(),
            },
        )
        .unwrap(),
        calls,
    )
}
