use crate::credential_identity::valid_uuid;
use crate::{CredentialIdentity, Error, Method, Profile, Result, Transport, WordPress};
use serde::{Deserialize, Serialize};
use std::path::Path;
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RevokePlan {
    pub schema: String,
    pub profile: Profile,
    pub user_id: u64,
    pub credential: CredentialIdentity,
    pub expires_at: u64,
    pub nonce: String,
}
impl RevokePlan {
    pub fn digest(&self) -> Result<String> {
        Ok(crate::upload_plan::hash(
            &serde_json::to_vec(self).map_err(|_| Error::InvalidInput)?,
        ))
    }
}
pub fn plan_revoke<T: Transport>(
    client: &mut WordPress<T>,
    now: u64,
    nonce: String,
) -> Result<RevokePlan> {
    let (user_id, credential) = client.credential_identity()?;
    if nonce.len() != 32 || !nonce.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(Error::InvalidInput);
    }
    Ok(RevokePlan {
        schema: "zix-wordpress-revoke/v1".into(),
        profile: client.profile().clone(),
        user_id,
        credential,
        expires_at: now.checked_add(600).ok_or(Error::InvalidInput)?,
        nonce,
    })
}
#[derive(Debug, Serialize)]
pub struct RevokeReceipt {
    pub origin: String,
    pub uuid: String,
    pub plan_sha256: String,
    pub server_deleted: bool,
    pub credential_rejected: bool,
    pub local_forgotten: bool,
}
pub fn apply_revoke<T: Transport>(
    client: &mut WordPress<T>,
    plan: &RevokePlan,
    approved: &str,
    now: u64,
    directory: &Path,
) -> Result<RevokeReceipt> {
    let digest = plan.digest()?;
    if plan.schema != "zix-wordpress-revoke/v1"
        || plan.profile != *client.profile()
        || approved != digest
        || plan.expires_at <= now
        || plan.expires_at > now.saturating_add(600)
        || !valid_uuid(&plan.credential.uuid)
        || plan.nonce.len() != 32
        || !plan.nonce.bytes().all(|b| b.is_ascii_hexdigit())
    {
        return Err(Error::PlanMismatch);
    }
    if client.credential_identity()? != (plan.user_id, plan.credential.clone()) {
        return Err(Error::Conflict);
    }
    let mut journal = crate::journal::Journal::begin(directory, &digest)?;
    let value = client
        .transport
        .request(
            Method::Delete,
            &format!(
                "/wp-json/wp/v2/users/me/application-passwords/{}",
                plan.credential.uuid
            ),
            None,
            true,
        )
        .map_err(|_| Error::UnknownResult)?;
    if value.get("deleted").and_then(serde_json::Value::as_bool) != Some(true) {
        return Err(Error::UnknownResult);
    }
    if !matches!(
        client.transport.request(
            Method::Get,
            "/wp-json/wp/v2/users/me?context=edit&_fields=id,capabilities",
            None,
            true
        ),
        Err(Error::Status(401))
    ) {
        return Err(Error::UnknownResult);
    }
    let receipt = RevokeReceipt {
        origin: plan.profile.origin.clone(),
        uuid: plan.credential.uuid.clone(),
        plan_sha256: digest,
        server_deleted: true,
        credential_rejected: true,
        local_forgotten: false,
    };
    journal.finish(&receipt)?;
    Ok(receipt)
}
