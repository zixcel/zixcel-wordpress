use crate::{Error, Method, Result, Transport, WordPress};
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CredentialIdentity {
    pub uuid: String,
    pub name: String,
}
pub(crate) fn valid_uuid(s: &str) -> bool {
    s.len() == 36
        && s.bytes().enumerate().all(|(i, b)| {
            if [8, 13, 18, 23].contains(&i) {
                b == b'-'
            } else {
                b.is_ascii_hexdigit()
            }
        })
}
impl<T: Transport> WordPress<T> {
    pub fn credential_identity(&mut self) -> Result<(u64, CredentialIdentity)> {
        let user = self.login()?.user_id;
        let value = self.transport.request(
            Method::Get,
            "/wp-json/wp/v2/users/me/application-passwords/introspect?_fields=uuid,name",
            None,
            true,
        )?;
        let uuid = value
            .get("uuid")
            .and_then(serde_json::Value::as_str)
            .filter(|s| valid_uuid(s))
            .ok_or(Error::InvalidResponse)?
            .to_owned();
        let name = value
            .get("name")
            .and_then(serde_json::Value::as_str)
            .filter(|s| s.len() <= 256 && !s.chars().any(char::is_control))
            .ok_or(Error::InvalidResponse)?
            .to_owned();
        Ok((user, CredentialIdentity { uuid, name }))
    }
}
