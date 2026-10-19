use crate::{ContentSummary, Draft, Error, Result};
use serde_json::Value;
pub(crate) fn parse_summary(value: &Value) -> Result<ContentSummary> {
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
    let title = value
        .pointer("/title/raw")
        .and_then(Value::as_str)
        .filter(|s| s.len() <= 16384)
        .ok_or(Error::InvalidResponse)?
        .to_owned();
    Ok(ContentSummary { id, status, title })
}
pub(crate) fn parse_draft(value: &Value) -> Result<Draft> {
    let summary = parse_summary(value)?;
    let content = value
        .pointer("/content/raw")
        .and_then(Value::as_str)
        .ok_or(Error::InvalidResponse)?
        .to_owned();
    Ok(Draft {
        id: summary.id,
        status: summary.status,
        title: summary.title,
        content,
    })
}
