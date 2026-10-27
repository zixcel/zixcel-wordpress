//! Typed installed adapter. Remote effects are declared independently of read-only planning.
pub(crate) use crate::provider_manifest::manifest;
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use zix_operation_contracts::Binding;
use zixcel_wordpress::{Error, Result};
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Request {
    schema: String,
    id: String,
    operation: String,
    arguments: BTreeMap<String, String>,
    limits: Limits,
    binding: Binding,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Limits {
    timeout_ms: u64,
    maximum_response_bytes: usize,
}
pub(crate) fn run() -> Result<Value> {
    let r: Request = crate::input::input()?;
    if r.schema != "zix-provider/v2"
        || r.id.len() != 32
        || !r
            .id
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        || r.limits.timeout_ms != 40000
        || r.limits.maximum_response_bytes != 131072
        || r.arguments.len() > 10
    {
        return Err(Error::InvalidInput);
    }
    let binding = crate::provider_manifest::binding()?;
    if r.binding != binding {
        return Err(Error::PlanMismatch);
    }
    let op = crate::provider_ops::ops()
        .find(|op| op.id == r.operation)
        .ok_or(Error::InvalidInput)?;
    let mut response = match crate::provider_invoke::invoke(
        op,
        &r.arguments,
        &binding,
        crate::command::execute,
    ) {
        Ok(value) => {
            json!({"schema":"zix-provider/v2","id":r.id,"operation":r.operation,"result":value})
        }
        Err(error) => {
            json!({"schema":"zix-provider/v2","id":r.id,"operation":r.operation,"error":crate::provider_error::code(error)})
        }
    };
    let catalog = crate::provider_guidance::catalog()?;
    let guide = catalog
        .select(&r.operation, &r.arguments)
        .ok_or(Error::InvalidInput)?;
    response["diagnostic"] = serde_json::to_value(
        zix_operation_contracts::guidance::Diagnostic::observe(guide.clone(), &response),
    )
    .map_err(|_| Error::InvalidResponse)?;
    if serde_json::to_vec(&response)
        .map_err(|_| Error::InvalidResponse)?
        .len()
        > r.limits.maximum_response_bytes
    {
        return Err(Error::ResponseLimit);
    }
    Ok(response)
}
