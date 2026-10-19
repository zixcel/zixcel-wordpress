use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use url::Url;
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    pub id: String,
    pub origin: String,
}
impl Profile {
    pub fn validate(&self) -> Result<Url> {
        let url = Url::parse(&self.origin).map_err(|_| Error::InvalidProfile)?;
        if self.id.is_empty()
            || self.id.len() > 48
            || !self
                .id
                .bytes()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
            || url.scheme() != "https"
            || !url.username().is_empty()
            || url.password().is_some()
            || url.port().is_some()
            || url.query().is_some()
            || url.fragment().is_some()
            || url.path() != "/"
            || url.origin().ascii_serialization() != self.origin
            || !matches!(url.host(), Some(url::Host::Domain(name)) if name.contains('.') && !name.ends_with('.'))
        {
            return Err(Error::InvalidProfile);
        }
        Ok(url)
    }
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ContentKind {
    Posts,
    Pages,
}
impl ContentKind {
    pub fn path(self) -> &'static str {
        match self {
            Self::Posts => "posts",
            Self::Pages => "pages",
        }
    }
    pub fn capability(self) -> &'static str {
        match self {
            Self::Posts => "edit_posts",
            Self::Pages => "edit_pages",
        }
    }
}
#[derive(Clone, Debug, Serialize)]
pub struct Discovery {
    pub origin: String,
    pub authorization_url: String,
}
#[derive(Clone, Debug, Serialize)]
pub struct Permission {
    pub user_id: u64,
    pub capabilities: BTreeMap<String, bool>,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Draft {
    pub id: u64,
    pub status: String,
    pub title: String,
    pub content: String,
}
