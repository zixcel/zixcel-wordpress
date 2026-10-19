use crate::{AuthorizationSession, BrowserGrant, Error, Result};
use std::{collections::BTreeMap, time::Instant};
use zeroize::Zeroizing;
impl AuthorizationSession {
    pub(crate) fn parse_callback(&self, raw: &[u8]) -> Result<BrowserGrant> {
        if Instant::now() >= self.expires {
            return Err(Error::Expired);
        }
        if raw.len() >= 8192 || !raw.ends_with(b"\r\n\r\n") {
            return Err(Error::InvalidInput);
        }
        let text = std::str::from_utf8(raw).map_err(|_| Error::InvalidInput)?;
        let mut lines = text.split("\r\n");
        let words = lines
            .next()
            .ok_or(Error::InvalidInput)?
            .split(' ')
            .collect::<Vec<_>>();
        if words.len() != 3 || words[0] != "GET" || words[2] != "HTTP/1.1" {
            return Err(Error::InvalidInput);
        }
        let mut headers = BTreeMap::new();
        for line in lines.filter(|l| !l.is_empty()) {
            let (key, value) = line.split_once(':').ok_or(Error::InvalidInput)?;
            if headers
                .insert(key.to_ascii_lowercase(), value.trim())
                .is_some()
            {
                return Err(Error::InvalidInput);
            }
        }
        if headers.get("host").copied() != Some(self.address.to_string().as_str())
            || headers.contains_key("transfer-encoding")
            || headers.get("content-length").is_some_and(|v| *v != "0")
        {
            return Err(Error::UnsafeTarget);
        }
        let (path, query) = words[1].split_once('?').ok_or(Error::InvalidInput)?;
        if path != self.path {
            return Err(Error::UnsafeTarget);
        }
        let mut fields = BTreeMap::new();
        for (key, value) in url::form_urlencoded::parse(query.as_bytes()) {
            if !["state", "success", "site_url", "user_login", "password"].contains(&key.as_ref())
                || fields
                    .insert(key.into_owned(), Zeroizing::new(value.into_owned()))
                    .is_some()
            {
                return Err(Error::InvalidInput);
            }
        }
        if fields.get("state").map(|s| s.as_str()) != Some(self.state.as_str()) {
            return Err(Error::PlanMismatch);
        }
        if fields.get("success").is_some_and(|s| s.as_str() == "false") {
            return Err(Error::Cancelled);
        }
        if fields.len() != 4
            || fields.get("site_url").map(|s| s.trim_end_matches('/'))
                != Some(self.profile.origin.as_str())
            || fields.get("user_login").map(|s| s.as_str()) != Some(self.expected_user.as_str())
        {
            return Err(Error::UnsafeTarget);
        }
        let password = fields.remove("password").ok_or(Error::InvalidCredential)?;
        if password.is_empty()
            || password.len() > 512
            || !password.bytes().all(|b| (32..127).contains(&b))
        {
            return Err(Error::InvalidCredential);
        }
        Ok(BrowserGrant {
            username: fields
                .remove("user_login")
                .ok_or(Error::InvalidCredential)?,
            password,
        })
    }
}
