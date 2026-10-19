use crate::{ContentKind, Error, Result, Transport, WordPress};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DraftPlan {
    pub schema: String,
    pub profile: crate::Profile,
    pub user_id: u64,
    pub kind: ContentKind,
    pub title: String,
    pub content: String,
    pub expires_at: u64,
    pub nonce: String,
}
#[derive(Clone, Debug, Serialize)]
pub struct DraftReceipt {
    pub origin: String,
    pub profile: String,
    pub id: u64,
    pub status: String,
    pub plan_sha256: String,
    pub read_back: bool,
}
impl DraftPlan {
    pub fn digest(&self) -> Result<String> {
        let bytes = serde_json::to_vec(self).map_err(|_| Error::InvalidInput)?;
        Ok(Sha256::digest(bytes)
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect())
    }
    pub(crate) fn validate(&self, now: u64) -> Result<()> {
        self.profile.validate()?;
        if self.schema != "zix-wordpress-draft/v1"
            || self.user_id == 0
            || self.title.is_empty()
            || self.title.len() > 1024
            || self.content.len() > 65536
            || self.title.contains('\0')
            || self.content.contains('\0')
            || self.expires_at <= now
            || self.expires_at > now + 600
            || self.nonce.len() != 32
            || !self.nonce.bytes().all(|c| c.is_ascii_hexdigit())
        {
            return Err(Error::InvalidInput);
        }
        Ok(())
    }
}
pub fn plan_draft<T: Transport>(
    client: &mut WordPress<T>,
    kind: ContentKind,
    title: String,
    content: String,
    now: u64,
    nonce: String,
) -> Result<DraftPlan> {
    let permission = client.require(kind)?;
    let plan = DraftPlan {
        schema: "zix-wordpress-draft/v1".into(),
        profile: client.profile().clone(),
        user_id: permission.user_id,
        kind,
        title,
        content,
        expires_at: now + 600,
        nonce,
    };
    plan.validate(now)?;
    Ok(plan)
}
