use crate::{ContentPlan, Error, Method, Result, Transport, WordPress};
use serde::{Deserialize, Serialize};
use std::path::Path;
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ContentReceipt {
    pub origin: String,
    pub profile: String,
    pub id: u64,
    pub status: String,
    pub effect: crate::ContentEffect,
    pub plan_sha256: String,
    pub read_back: bool,
}
pub fn apply_content<T: Transport>(
    client: &mut WordPress<T>,
    plan: &ContentPlan,
    approved: &str,
    now: u64,
    directory: &Path,
) -> Result<ContentReceipt> {
    plan.validate(now)?;
    let digest = plan.digest()?;
    if approved != digest || plan.profile != *client.profile() {
        return Err(Error::PlanMismatch);
    }
    if client.content_permission(plan.kind, plan.effect)?.user_id != plan.user_id {
        return Err(Error::PlanMismatch);
    }
    if client.snapshot(plan.kind, plan.after.id)? != plan.before {
        return Err(Error::Conflict);
    }
    let mut journal = crate::journal::Journal::begin(directory, &digest)?;
    client.transport.request(Method::Post,
        &format!("/wp-json/wp/v2/{}/{}", plan.kind.path(), plan.after.id),
        Some(&serde_json::json!({"title":plan.after.title,"content":plan.after.content,"status":plan.after.status})), true)
        .map_err(|_| Error::UnknownResult)?;
    let observed = client
        .snapshot(plan.kind, plan.after.id)
        .map_err(|_| Error::UnknownResult)?;
    if observed.item != plan.after {
        return Err(Error::UnknownResult);
    }
    let receipt = ContentReceipt {
        origin: plan.profile.origin.clone(),
        profile: plan.profile.id.clone(),
        id: observed.item.id,
        status: observed.item.status,
        effect: plan.effect,
        plan_sha256: digest,
        read_back: true,
    };
    journal.finish(&receipt)?;
    Ok(receipt)
}
#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Reconciliation {
    DesiredStateObserved,
    UnchangedObserved,
    Conflict,
}
/// Observation only: matching state does not prove which actor wrote it or permit a resend.
pub fn reconcile_content<T: Transport>(
    client: &mut WordPress<T>,
    plan: &ContentPlan,
) -> Result<Reconciliation> {
    if plan.profile != *client.profile() {
        return Err(Error::PlanMismatch);
    }
    if client.content_permission(plan.kind, plan.effect)?.user_id != plan.user_id {
        return Err(Error::PlanMismatch);
    }
    let current = client.snapshot(plan.kind, plan.after.id)?;
    Ok(if current.item == plan.after {
        Reconciliation::DesiredStateObserved
    } else if current == plan.before {
        Reconciliation::UnchangedObserved
    } else {
        Reconciliation::Conflict
    })
}
