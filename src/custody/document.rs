use crate::{Error, Profile, Result};
use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Login {
    pub username: String,
    pub password: String,
}
impl Drop for Login {
    fn drop(&mut self) {
        self.username.zeroize();
        self.password.zeroize();
    }
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Envelope {
    schema: String,
    profile: Profile,
    login: Login,
}
#[derive(Serialize)]
struct Encoded<'a> {
    schema: &'a str,
    profile: &'a Profile,
    login: &'a Login,
}
impl Login {
    fn validate(&self) -> Result<()> {
        if self.username.is_empty()
            || self.username.len() > 128
            || self.username.chars().any(|c| c.is_control() || c == ':')
            || self.password.is_empty()
            || self.password.len() > 512
            || !self.password.bytes().all(|b| (32..127).contains(&b))
        {
            return Err(Error::InvalidCredential);
        }
        Ok(())
    }
    pub fn parse(bytes: &[u8]) -> Result<Self> {
        if bytes.len() > 4096 {
            return Err(Error::InvalidCredential);
        }
        let value: Self = serde_json::from_slice(bytes).map_err(|_| Error::InvalidCredential)?;
        value.validate()?;
        Ok(value)
    }
    pub(super) fn encode(&self, profile: &Profile) -> Result<Vec<u8>> {
        serde_json::to_vec(&Encoded {
            schema: "zix-wordpress-login/v1",
            profile,
            login: self,
        })
        .map_err(|_| Error::InvalidCredential)
    }
    pub(super) fn decode(profile: &Profile, bytes: &[u8]) -> Result<Self> {
        if bytes.len() > 8192 {
            return Err(Error::InvalidCredential);
        }
        let envelope: Envelope =
            serde_json::from_slice(bytes).map_err(|_| Error::InvalidCredential)?;
        if envelope.schema != "zix-wordpress-login/v1" || envelope.profile != *profile {
            return Err(Error::InvalidCredential);
        }
        envelope.login.validate()?;
        Ok(envelope.login)
    }
}
