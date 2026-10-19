//! Origin-bound credentials delegated to Crowsi's OS custody and one-use broker.
mod document;
use crate::{Error, Profile, Result};
use crowsi_credential_broker::{
    AccessRequest, Broker, CredentialEntry, CredentialStore, LeaseBinding, SecretRef, SecretScope,
    SecretValue,
};
pub use document::Login;
use sha2::{Digest, Sha256};
use std::sync::Arc;

pub fn reference(profile: &Profile) -> Result<SecretRef> {
    profile.validate()?;
    let id = Sha256::digest(profile.origin.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>();
    let scope = SecretScope::new(
        &profile.id,
        "wordpress",
        "application-password",
        "wordpress-cli",
    )
    .map_err(|_| Error::InvalidProfile)?;
    SecretRef::new(&id, scope).map_err(|_| Error::InvalidProfile)
}

pub fn enroll(store: &impl CredentialStore, profile: &Profile, bytes: &[u8]) -> Result<String> {
    let login = Login::parse(bytes)?;
    let host = profile
        .validate()?
        .host_str()
        .ok_or(Error::InvalidProfile)?
        .to_owned();
    let payload = login.encode(profile)?;
    let entry = CredentialEntry::new(
        reference(profile)?,
        [host],
        SecretValue::new(payload).map_err(|_| Error::InvalidCredential)?,
    )
    .map_err(|_| Error::InvalidCredential)?;
    let revision = entry.metadata().revision().as_str().to_owned();
    store.put(entry).map_err(|_| Error::LocalState)?;
    Ok(revision)
}

pub fn use_login<S: CredentialStore>(
    store: Arc<S>,
    profile: &Profile,
    operation: &str,
) -> Result<Login> {
    let host = profile
        .validate()?
        .host_str()
        .ok_or(Error::InvalidProfile)?
        .to_owned();
    let resource = reference(profile)?;
    let binding = LeaseBinding::new(
        "local-user",
        "local-session",
        "spiffe://zix/local/wordpress",
        "invocation",
        resource.id(),
        operation,
        "os-session",
    )
    .map_err(|_| Error::InvalidInput)?;
    let request = AccessRequest::new(resource, "wordpress-cli", &host, 30)
        .map_err(|_| Error::InvalidInput)?;
    let broker = Broker::new(store).map_err(|_| Error::LocalState)?;
    let grant = broker
        .issue_bound(request, binding.clone())
        .map_err(|_| Error::InvalidCredential)?;
    let used = broker
        .consume_bound(grant.token(), "wordpress-cli", &host, &binding)
        .map_err(|_| Error::InvalidCredential)?;
    used.secret.expose(|bytes| Login::decode(profile, bytes))
}
