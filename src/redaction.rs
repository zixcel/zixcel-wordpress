use base64::{Engine, engine::general_purpose::STANDARD};
use serde_json::Value;
use zeroize::Zeroizing;
pub(crate) fn reflected(value: &Value, username: &str, password: &str) -> bool {
    let pair = Zeroizing::new(format!("{username}:{password}"));
    let basic = Zeroizing::new(STANDARD.encode(pair.as_bytes()));
    contains(value, &[password, &basic])
}
fn contains(value: &Value, secrets: &[&str]) -> bool {
    let includes = |s: &str| {
        secrets
            .iter()
            .any(|secret| !secret.is_empty() && s.contains(secret))
    };
    match value {
        Value::String(s) => includes(s),
        Value::Array(a) => a.iter().any(|v| contains(v, secrets)),
        Value::Object(o) => o
            .iter()
            .any(|(key, value)| includes(key) || contains(value, secrets)),
        _ => false,
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn escaped_json_nested_fields_and_basic_auth_are_never_output() {
        let secret = "synthetic-\"password";
        let value = serde_json::json!({"items":[{"content":secret}]});
        assert!(reflected(&value, "synthetic-user", secret));
        let raw = STANDARD.encode(format!("synthetic-user:{secret}"));
        assert!(reflected(
            &serde_json::json!({"header":format!("Basic {raw}")}),
            "synthetic-user",
            secret
        ));
        assert!(!reflected(
            &serde_json::json!({"state":"available"}),
            "synthetic-user",
            secret
        ));
    }
}
