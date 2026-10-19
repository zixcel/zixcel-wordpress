use crate::{Discovery, Error, Method, Permission, Profile, Result, Transport};
use serde_json::Value;
use std::collections::BTreeMap;
pub struct WordPress<T: Transport> {
    pub(crate) profile: Profile,
    pub(crate) transport: T,
}
impl<T: Transport> WordPress<T> {
    pub fn new(profile: Profile, transport: T) -> Result<Self> {
        profile.validate()?;
        if profile.origin != transport.origin() {
            return Err(Error::UnsafeTarget);
        }
        Ok(Self { profile, transport })
    }
    pub fn profile(&self) -> &Profile {
        &self.profile
    }
    pub fn discover(&mut self) -> Result<Discovery> {
        let value = self
            .transport
            .request(Method::Get, "/wp-json/", None, false)?;
        if value.get("home").and_then(Value::as_str) != Some(&self.profile.origin)
            || value.get("url").and_then(Value::as_str) != Some(&self.profile.origin)
            || !value
                .get("namespaces")
                .and_then(Value::as_array)
                .is_some_and(|ns| ns.iter().any(|s| s == "wp/v2"))
        {
            return Err(Error::Unsupported);
        }
        let authorization_url = value
            .pointer("/authentication/application-passwords/endpoints/authorization")
            .and_then(Value::as_str)
            .ok_or(Error::Unsupported)?;
        if authorization_url
            != format!("{}/wp-admin/authorize-application.php", self.profile.origin)
        {
            return Err(Error::UnsafeTarget);
        }
        Ok(Discovery {
            origin: self.profile.origin.clone(),
            authorization_url: authorization_url.to_owned(),
        })
    }
    pub fn login(&mut self) -> Result<Permission> {
        self.discover()?;
        let value = self.transport.request(
            Method::Get,
            "/wp-json/wp/v2/users/me?context=edit&_fields=id,capabilities",
            None,
            true,
        )?;
        let user_id = value
            .get("id")
            .and_then(Value::as_u64)
            .filter(|id| *id > 0)
            .ok_or(Error::InvalidResponse)?;
        let caps = value
            .get("capabilities")
            .and_then(Value::as_object)
            .ok_or(Error::InvalidResponse)?;
        let mut capabilities = BTreeMap::new();
        for name in [
            "edit_posts",
            "edit_pages",
            "publish_posts",
            "publish_pages",
            "upload_files",
        ] {
            capabilities.insert(
                name.to_owned(),
                caps.get(name).and_then(Value::as_bool).unwrap_or(false),
            );
        }
        Ok(Permission {
            user_id,
            capabilities,
        })
    }
}
