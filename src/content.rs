use crate::response::{parse_content, parse_summary};
use crate::{
    ContentItem, ContentKind, ContentSummary, Error, Method, Permission, Result, Transport,
    WordPress,
};
use serde_json::json;
impl<T: Transport> WordPress<T> {
    pub fn list(&mut self, kind: ContentKind, page: u16) -> Result<Vec<ContentSummary>> {
        if !(1..=100).contains(&page) {
            return Err(Error::InvalidInput);
        }
        self.require(kind)?;
        let value = self.transport.request(Method::Get, &format!("/wp-json/wp/v2/{}?context=edit&per_page=25&page={page}&_fields=id,status,title.raw", kind.path()), None, true)?;
        let rows = value
            .as_array()
            .filter(|rows| rows.len() <= 25)
            .ok_or(Error::InvalidResponse)?;
        rows.iter().map(parse_summary).collect()
    }
    pub fn get(&mut self, kind: ContentKind, id: u64) -> Result<ContentItem> {
        if id == 0 {
            return Err(Error::InvalidInput);
        }
        self.require(kind)?;
        let item = self.read(kind, id)?;
        if item.id != id {
            return Err(Error::InvalidResponse);
        }
        Ok(item)
    }
    pub(crate) fn require(&mut self, kind: ContentKind) -> Result<Permission> {
        let permission = self.login()?;
        if permission.capabilities.get(kind.capability()) != Some(&true) {
            return Err(Error::PermissionDenied);
        }
        Ok(permission)
    }
    pub(crate) fn create(
        &mut self,
        kind: ContentKind,
        title: &str,
        content: &str,
    ) -> Result<ContentItem> {
        let value = self
            .transport
            .request(
                Method::Post,
                &format!(
                    "/wp-json/wp/v2/{}?_fields=id,status,title.raw,content.raw",
                    kind.path()
                ),
                Some(&json!({"status":"draft","title":title,"content":content})),
                true,
            )
            .map_err(|_| Error::UnknownResult)?;
        parse_content(&value).map_err(|_| Error::UnknownResult)
    }
    pub(crate) fn read(&mut self, kind: ContentKind, id: u64) -> Result<ContentItem> {
        let value = self.transport.request(
            Method::Get,
            &format!(
                "/wp-json/wp/v2/{}/{id}?context=edit&_fields=id,status,title.raw,content.raw",
                kind.path()
            ),
            None,
            true,
        )?;
        parse_content(&value)
    }
}
