use crate::provider_ops::{Mode, Op};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use zix_operation_contracts::{Binding, LocalPlan};
use zixcel_wordpress::{ContentPlan, DraftPlan, Error, Result, RevokePlan, UploadPlan};
fn inner_digest(native: &str, payload: &Value) -> Result<String> {
    macro_rules! parse {
        ($t:ty) => {
            serde_json::from_value::<$t>(payload.clone())
                .map_err(|_| Error::InvalidInput)?
                .digest()
        };
    }
    match native {
        "draft-apply" => parse!(DraftPlan),
        "content-apply" => parse!(ContentPlan),
        "media-apply" => parse!(UploadPlan),
        "revoke-apply" => parse!(RevokePlan),
        _ => Err(Error::InvalidInput),
    }
}
pub(crate) fn invoke<F>(
    op: &Op,
    values: &BTreeMap<String, String>,
    binding: &Binding,
    execute: F,
) -> Result<Value>
where
    F: FnOnce(&[String], Option<&str>, Option<&[u8]>) -> Result<Value>,
{
    let get = |name| {
        values
            .get(name)
            .map(String::as_str)
            .ok_or(Error::InvalidInput)
    };
    for name in values.keys() {
        if !["profile", "origin", "collection"].contains(&name.as_str())
            && !op.extra.iter().any(|(key, _)| key == name)
        {
            return Err(Error::InvalidInput);
        }
    }
    for (name, required) in op.extra {
        if *required && !values.contains_key(*name) {
            return Err(Error::InvalidInput);
        }
    }
    if values
        .values()
        .any(|s| s.is_empty() || s.len() > 2048 || s.chars().any(char::is_control))
    {
        return Err(Error::InvalidInput);
    }
    let mut args = vec![
        op.native.into(),
        get("profile")?.into(),
        get("origin")?.into(),
    ];
    let mut input = None;
    match op.mode {
        Mode::Read => {
            for key in ["page", "id"] {
                if let Some(value) = values.get(key) {
                    args.push(value.clone());
                }
            }
        }
        Mode::Plan(_) => {
            if let Some(path) = values.get("input-file") {
                input = Some(crate::local_file::read(path)?);
            }
            if let Some(file) = values.get("file") {
                args.push(file.clone());
            }
        }
        Mode::Apply | Mode::Reconcile => {
            let bytes = crate::local_file::plan(get("plan-file")?)?;
            let plan: LocalPlan =
                serde_json::from_slice(&bytes).map_err(|_| Error::InvalidInput)?;
            let action = if matches!(op.mode, Mode::Reconcile) {
                "wordpress.content-apply"
            } else {
                op.id
            };
            let approved = if matches!(op.mode, Mode::Reconcile) {
                plan.digest()
            } else {
                get("approval-sha256")?.into()
            };
            if !plan.matches(action, binding, &approved) {
                return Err(Error::PlanMismatch);
            }
            if matches!(op.mode, Mode::Apply) {
                if let Some(file) = values.get("file") {
                    args.push(file.clone());
                }
                args.push(inner_digest(op.native, &plan.plan)?);
                args.push(get("journal-directory")?.into());
            }
            input = Some(serde_json::to_vec(&plan.plan).map_err(|_| Error::InvalidInput)?);
        }
    }
    let value = execute(&args, Some(get("collection")?), input.as_deref())?;
    if let Mode::Plan(action) = op.mode {
        let plan = LocalPlan::new(
            action.into(),
            binding.clone(),
            value.get("plan").cloned().ok_or(Error::InvalidResponse)?,
        );
        return Ok(json!({"sha256":plan.digest(),"plan":plan}));
    }
    Ok(json!({"value":value,"invocation":{"operation":op.id,"binding":binding}}))
}

#[cfg(test)]
#[path = "provider_invoke/tests/mod.rs"]
mod tests;
