use crate::{ContentItem, ContentKind, Error, Profile, Result, Transport, WordPress};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Snapshot {
    pub item: ContentItem,
    pub modified_gmt: String,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ContentEffect {
    EditDraft,
    Publish,
    EditPublished,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ContentPlan {
    pub schema: String,
    pub profile: Profile,
    pub user_id: u64,
    pub kind: ContentKind,
    pub effect: ContentEffect,
    pub before: Snapshot,
    pub after: ContentItem,
    pub expires_at: u64,
    pub nonce: String,
}
impl ContentPlan {
    pub fn digest(&self) -> Result<String> {
        Ok(
            Sha256::digest(serde_json::to_vec(self).map_err(|_| Error::InvalidInput)?)
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect(),
        )
    }
    pub(crate) fn validate(&self, now: u64) -> Result<()> {
        self.profile.validate()?;
        let statuses = match self.effect {
            ContentEffect::EditDraft => ("draft", "draft"),
            ContentEffect::Publish => ("draft", "publish"),
            ContentEffect::EditPublished => ("publish", "publish"),
        };
        if self.schema != "zix-wordpress-content/v1"
            || self.user_id == 0
            || self.before.item.id == 0
            || self.before.item.id != self.after.id
            || self.before.item.status != statuses.0
            || self.after.status != statuses.1
            || self.before.modified_gmt.len() != 19
            || self.before.item.content.len() > 65536
            || self.after.title.is_empty()
            || self.after.title.len() > 1024
            || self.after.content.len() > 65536
            || self.after.title.contains('\0')
            || self.after.content.contains('\0')
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
pub fn plan_content<T: Transport>(
    client: &mut WordPress<T>,
    kind: ContentKind,
    effect: ContentEffect,
    after: ContentItem,
    now: u64,
    nonce: String,
) -> Result<ContentPlan> {
    let permission = client.content_permission(kind, effect)?;
    let before = client.snapshot(kind, after.id)?;
    let plan = ContentPlan {
        schema: "zix-wordpress-content/v1".into(),
        profile: client.profile().clone(),
        user_id: permission.user_id,
        kind,
        effect,
        before,
        after,
        expires_at: now.checked_add(600).ok_or(Error::InvalidInput)?,
        nonce,
    };
    plan.validate(now)?;
    Ok(plan)
}
