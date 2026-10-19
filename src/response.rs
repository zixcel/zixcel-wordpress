use crate::{Draft, Error, Result};
use serde_json::Value;
pub(crate) fn parse_draft(value: &Value) -> Result<Draft> {
    let id = value
        .get("id")
        .and_then(Value::as_u64)
        .filter(|id| *id > 0)
        .ok_or(Error::InvalidResponse)?;
    let status = value
        .get("status")
        .and_then(Value::as_str)
        .filter(|s| {
            matches!(
                *s,
                "draft" | "publish" | "private" | "pending" | "future" | "trash"
            )
        })
        .ok_or(Error::InvalidResponse)?
        .to_owned();
    let text = |field| {
        value
            .pointer(field)
            .and_then(Value::as_str)
            .map(str::to_owned)
            .ok_or(Error::InvalidResponse)
    };
    Ok(Draft {
        id,
        status,
        title: text("/title/raw")?,
        content: text("/content/raw")?,
    })
}
