//! Installed zix read adapter. Credential mutations and draft writes use explicit local commands.
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use zixcel_wordpress::{Error, Result};
const OPERATIONS: [(&str, &str, &[&str]); 3] = [
    ("wordpress.login", "login", &["status"]),
    ("wordpress.posts-list", "posts-list", &["posts", "list"]),
    ("wordpress.pages-list", "pages-list", &["pages", "list"]),
];
pub(crate) fn manifest() -> Result<Value> {
    if !cfg!(all(target_os = "linux", feature = "os-keyring")) {
        return Err(Error::Unsupported);
    }
    Ok(
        json!({"schema":"zix-extension/v1", "package":"zixcel-wordpress", "version":env!("CARGO_PKG_VERSION"),
        "namespace":"wordpress", "platform":"linux-x86_64", "protocol":"zix-provider/v1",
        "credential_environment":[], "runtime_environment":["DBUS_SESSION_BUS_ADDRESS"], "timeout_ms":40000,
        "commands": OPERATIONS.iter().map(|(id,_,words)| json!({"id":id,"words":words,"effect":"read",
            "description":"Read a selected WordPress origin using its stored credential; never auto-unlock.",
            "required_permissions":["wordpress:read"], "arguments":[
                {"name":"profile","required":true,"maximum":48},
                {"name":"origin","required":true,"maximum":512},
                {"name":"collection","required":true,"maximum":64}]})).collect::<Vec<_>>()}),
    )
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    schema: String,
    id: String,
    operation: String,
    arguments: BTreeMap<String, String>,
    limits: Limits,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Limits {
    timeout_ms: u64,
    maximum_response_bytes: usize,
}
pub(crate) fn run() -> Result<Value> {
    let request: Request = crate::input::input()?;
    if request.schema != "zix-provider/v1"
        || request.id.len() != 32
        || !request
            .id
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        || request.limits.timeout_ms != 40000
        || request.limits.maximum_response_bytes != 131072
        || request.arguments.len() != 3
    {
        return Err(Error::InvalidInput);
    }
    let field = |name| {
        request
            .arguments
            .get(name)
            .filter(|v| !v.is_empty() && v.len() <= 2048 && !v.chars().any(char::is_control))
            .map(String::as_str)
            .ok_or(Error::InvalidInput)
    };
    let operation = OPERATIONS
        .iter()
        .find(|(id, _, _)| *id == request.operation)
        .ok_or(Error::InvalidInput)?
        .1;
    let args = [
        operation.to_owned(),
        field("profile")?.to_owned(),
        field("origin")?.to_owned(),
    ];
    let result = crate::command::execute(&args, Some(field("collection")?));
    let response = match result {
        Ok(value) => {
            json!({"schema":"zix-provider/v1","id":request.id,"operation":request.operation,"result":value})
        }
        Err(_) => {
            json!({"schema":"zix-provider/v1","id":request.id,"operation":request.operation,"error":"wordpress_operation_failed"})
        }
    };
    if serde_json::to_vec(&response)
        .map_err(|_| Error::InvalidResponse)?
        .len()
        > request.limits.maximum_response_bytes
    {
        return Err(Error::ResponseLimit);
    }
    Ok(response)
}
