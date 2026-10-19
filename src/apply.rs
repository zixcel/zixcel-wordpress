use crate::{DraftPlan, DraftReceipt, Error, Result, Transport, WordPress};
use std::path::Path;
/// The caller creates a private journal directory. An existing digest journal always forbids replay.
pub fn apply_draft<T: Transport>(
    client: &mut WordPress<T>,
    plan: &DraftPlan,
    approved_digest: &str,
    now: u64,
    journal: &Path,
) -> Result<DraftReceipt> {
    plan.validate(now)?;
    let digest = plan.digest()?;
    if digest != approved_digest || plan.profile != *client.profile() {
        return Err(Error::PlanMismatch);
    }
    let permission = client.require(plan.kind)?;
    if permission.user_id != plan.user_id {
        return Err(Error::PlanMismatch);
    }
    let mut journal = crate::journal::Journal::begin(journal, &digest)?;
    let created = client.create(plan.kind, &plan.title, &plan.content)?;
    journal.accepted(&plan.profile.origin, created.id, &digest)?;
    if created.status != "draft" {
        return Err(Error::UnknownResult);
    }
    let observed = client
        .read(plan.kind, created.id)
        .map_err(|_| Error::UnknownResult)?;
    if observed.id != created.id
        || observed.status != "draft"
        || observed.title != plan.title
        || observed.content != plan.content
    {
        return Err(Error::UnknownResult);
    }
    let receipt = DraftReceipt {
        origin: plan.profile.origin.clone(),
        profile: plan.profile.id.clone(),
        id: observed.id,
        status: "draft".into(),
        plan_sha256: digest,
        read_back: true,
    };
    journal.finish(&receipt)?;
    Ok(receipt)
}
