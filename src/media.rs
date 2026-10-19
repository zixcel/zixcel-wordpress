use crate::{Error, Method, Result, Transport, WordPress};
use serde::{Deserialize, Serialize};
use serde_json::Value;
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct MediaItem {
    pub id: u64,
    pub source_url: String,
    pub mime_type: String,
    pub alt_text: String,
}
pub(crate) fn parse(value: &Value, origin: &str) -> Result<MediaItem> {
    let id = value
        .get("id")
        .and_then(Value::as_u64)
        .filter(|id| *id > 0)
        .ok_or(Error::InvalidResponse)?;
    let text = |key| {
        value
            .get(key)
            .and_then(Value::as_str)
            .filter(|s| s.len() <= 4096 && !s.chars().any(char::is_control))
            .map(str::to_owned)
            .ok_or(Error::InvalidResponse)
    };
    let item = MediaItem {
        id,
        source_url: text("source_url")?,
        mime_type: text("mime_type")?,
        alt_text: text("alt_text")?,
    };
    let url = url::Url::parse(&item.source_url).map_err(|_| Error::InvalidResponse)?;
    // Do not fetch or execute a returned URL. Initial policy admits only this exact origin.
    if url.origin().ascii_serialization() != origin
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return Err(Error::UnsafeTarget);
    }
    Ok(item)
}
impl<T: Transport> WordPress<T> {
    pub(crate) fn media_permission(&mut self) -> Result<crate::Permission> {
        let p = self.login()?;
        if p.capabilities.get("upload_files") != Some(&true) {
            return Err(Error::PermissionDenied);
        }
        Ok(p)
    }
    pub fn media_list(&mut self, page: u16) -> Result<Vec<MediaItem>> {
        if !(1..=100).contains(&page) {
            return Err(Error::InvalidInput);
        }
        self.media_permission()?;
        let v=self.transport.request(Method::Get,&format!("/wp-json/wp/v2/media?context=edit&per_page=25&page={page}&_fields=id,source_url,mime_type,alt_text"),None,true)?;
        v.as_array()
            .filter(|v| v.len() <= 25)
            .ok_or(Error::InvalidResponse)?
            .iter()
            .map(|v| parse(v, &self.profile.origin))
            .collect()
    }
    pub fn media_get(&mut self, id: u64) -> Result<MediaItem> {
        if id == 0 {
            return Err(Error::InvalidInput);
        }
        self.media_permission()?;
        self.media_read(id)
    }
    pub(crate) fn media_read(&mut self, id: u64) -> Result<MediaItem> {
        let v = self.transport.request(
            Method::Get,
            &format!(
                "/wp-json/wp/v2/media/{id}?context=edit&_fields=id,source_url,mime_type,alt_text"
            ),
            None,
            true,
        )?;
        let item = parse(&v, &self.profile.origin)?;
        if item.id != id {
            return Err(Error::InvalidResponse);
        }
        Ok(item)
    }
}
