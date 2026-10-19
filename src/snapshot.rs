use crate::{
    ContentEffect, ContentKind, Error, Method, Permission, Result, Snapshot, Transport, WordPress,
};
impl<T: Transport> WordPress<T> {
    pub(crate) fn content_permission(
        &mut self,
        kind: ContentKind,
        effect: ContentEffect,
    ) -> Result<Permission> {
        let p = self.require(kind)?;
        let publish = if kind == ContentKind::Posts {
            "publish_posts"
        } else {
            "publish_pages"
        };
        if effect != ContentEffect::EditDraft && p.capabilities.get(publish) != Some(&true) {
            return Err(Error::PermissionDenied);
        }
        Ok(p)
    }
    pub(crate) fn snapshot(&mut self, kind: ContentKind, id: u64) -> Result<Snapshot> {
        if id == 0 {
            return Err(Error::InvalidInput);
        }
        let value = self.transport.request(Method::Get,
            &format!("/wp-json/wp/v2/{}/{id}?context=edit&_fields=id,status,title.raw,content.raw,modified_gmt", kind.path()), None, true)?;
        let item = crate::response::parse_content(&value)?;
        if item.id != id {
            return Err(Error::InvalidResponse);
        }
        let modified_gmt = value
            .get("modified_gmt")
            .and_then(serde_json::Value::as_str)
            .filter(|s| {
                s.len() == 19
                    && s.bytes().enumerate().all(|(i, b)| {
                        if [4, 7].contains(&i) {
                            b == b'-'
                        } else if i == 10 {
                            b == b'T'
                        } else if [13, 16].contains(&i) {
                            b == b':'
                        } else {
                            b.is_ascii_digit()
                        }
                    })
            })
            .ok_or(Error::InvalidResponse)?
            .to_owned();
        Ok(Snapshot { item, modified_gmt })
    }
}
