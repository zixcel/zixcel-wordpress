use crate::{DraftPlan, DraftReceipt, Error, Result, Transport, WordPress};
use std::{fs::OpenOptions, io::Write, path::Path};
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
    let meta = std::fs::symlink_metadata(journal).map_err(|_| Error::LocalState)?;
    use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};
    if !meta.is_dir() || meta.file_type().is_symlink() || meta.permissions().mode() & 0o077 != 0 {
        return Err(Error::LocalState);
    }
    // Require a journal owned by the current effective user (obtained without unsafe code).
    let own = std::fs::metadata("/proc/self")
        .map_err(|_| Error::LocalState)?
        .uid();
    if meta.uid() != own {
        return Err(Error::LocalState);
    }
    let file = journal.join(format!("{digest}.json"));
    let mut log = OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(file)
        .map_err(|e| {
            if e.kind() == std::io::ErrorKind::AlreadyExists {
                Error::Replay
            } else {
                Error::LocalState
            }
        })?;
    log.write_all(b"{\"state\":\"write-starting; do not replay\"}\n")
        .and_then(|_| log.sync_all())
        .map_err(|_| Error::LocalState)?;
    let created = client.create(plan.kind, &plan.title, &plan.content)?;
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
    let value = serde_json::to_vec(&receipt).map_err(|_| Error::LocalState)?;
    log.set_len(0).map_err(|_| Error::LocalState)?;
    use std::io::{Seek, SeekFrom};
    log.seek(SeekFrom::Start(0))
        .and_then(|_| log.write_all(&value))
        .and_then(|_| log.sync_all())
        .map_err(|_| Error::LocalState)?;
    Ok(receipt)
}
