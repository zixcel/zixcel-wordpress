use crate::{Error, Profile, Result, Transport, WordPress};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
pub(crate) fn hash(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}
pub(crate) fn validate_file(filename: &str, mime: &str, bytes: &[u8]) -> Result<()> {
    let extension = filename.rsplit('.').next().unwrap_or_default();
    let supported = match mime {
        "image/png" => extension == "png" && bytes.starts_with(b"\x89PNG\r\n\x1a\n"),
        "image/jpeg" => {
            ["jpg", "jpeg"].contains(&extension) && bytes.starts_with(&[0xff, 0xd8, 0xff])
        }
        "application/pdf" => extension == "pdf" && bytes.starts_with(b"%PDF-"),
        _ => false,
    };
    if filename.is_empty()
        || filename.len() > 128
        || filename.starts_with('.')
        || filename.contains("..")
        || !filename
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
        || bytes.is_empty()
        || bytes.len() > 8 * 1024 * 1024
        || !supported
    {
        return Err(Error::InvalidInput);
    }
    Ok(())
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UploadPlan {
    pub schema: String,
    pub profile: Profile,
    pub user_id: u64,
    pub filename: String,
    pub mime: String,
    pub file_sha256: String,
    pub size: usize,
    pub publicly_accessible: bool,
    pub expires_at: u64,
    pub nonce: String,
}
impl UploadPlan {
    pub fn digest(&self) -> Result<String> {
        Ok(hash(
            &serde_json::to_vec(self).map_err(|_| Error::InvalidInput)?,
        ))
    }
    pub(crate) fn validate(&self, now: u64, bytes: &[u8]) -> Result<()> {
        self.profile.validate()?;
        validate_file(&self.filename, &self.mime, bytes)?;
        if self.schema != "zix-wordpress-upload/v1"
            || self.user_id == 0
            || !self.publicly_accessible
            || self.size != bytes.len()
            || self.file_sha256 != hash(bytes)
            || self.expires_at <= now
            || self.expires_at > now.saturating_add(600)
            || self.nonce.len() != 32
            || !self.nonce.bytes().all(|b| b.is_ascii_hexdigit())
        {
            return Err(Error::InvalidInput);
        }
        Ok(())
    }
}
pub fn plan_upload<T: Transport>(
    client: &mut WordPress<T>,
    filename: String,
    mime: String,
    bytes: &[u8],
    now: u64,
    nonce: String,
) -> Result<UploadPlan> {
    validate_file(&filename, &mime, bytes)?;
    let p = client.media_permission()?;
    let plan = UploadPlan {
        schema: "zix-wordpress-upload/v1".into(),
        profile: client.profile().clone(),
        user_id: p.user_id,
        filename,
        mime,
        file_sha256: hash(bytes),
        size: bytes.len(),
        publicly_accessible: true,
        expires_at: now.checked_add(600).ok_or(Error::InvalidInput)?,
        nonce,
    };
    plan.validate(now, bytes)?;
    Ok(plan)
}
