use crate::{Error, MediaItem, Result, Transport, UploadPlan, WordPress};
use serde::Serialize;
use std::path::Path;
#[derive(Clone, Debug, Serialize)]
pub struct UploadReceipt {
    pub origin: String,
    pub profile: String,
    pub plan_sha256: String,
    pub file_sha256: String,
    pub media: MediaItem,
    pub publicly_accessible: bool,
    pub attached_to_post: bool,
    pub read_back: bool,
}
pub fn apply_upload<T: Transport>(
    client: &mut WordPress<T>,
    plan: &UploadPlan,
    approved: &str,
    bytes: &[u8],
    now: u64,
    directory: &Path,
) -> Result<UploadReceipt> {
    plan.validate(now, bytes)?;
    let digest = plan.digest()?;
    if digest != approved || plan.profile != *client.profile() {
        return Err(Error::PlanMismatch);
    }
    if client.media_permission()?.user_id != plan.user_id {
        return Err(Error::PlanMismatch);
    }
    let mut journal = crate::journal::Journal::begin(directory, &digest)?;
    let v = client
        .transport
        .upload(&plan.filename, &plan.mime, bytes)
        .map_err(|_| Error::UnknownResult)?;
    let created =
        crate::media::parse(&v, &plan.profile.origin).map_err(|_| Error::UnknownResult)?;
    let observed = client
        .media_read(created.id)
        .map_err(|_| Error::UnknownResult)?;
    if created != observed || observed.mime_type != plan.mime {
        return Err(Error::UnknownResult);
    }
    // Upload is deliberately standalone; attachment/publication requires another plan.
    let receipt = UploadReceipt {
        origin: plan.profile.origin.clone(),
        profile: plan.profile.id.clone(),
        plan_sha256: digest,
        file_sha256: plan.file_sha256.clone(),
        media: observed,
        publicly_accessible: true,
        attached_to_post: false,
        read_back: true,
    };
    journal.finish(&receipt)?;
    Ok(receipt)
}
