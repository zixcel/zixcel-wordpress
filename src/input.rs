use serde::Deserialize;
use std::io::{self, Read};
use zeroize::Zeroizing;
use zixcel_wordpress::{ContentKind, Error, Result};
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DraftInput {
    pub kind: ContentKind,
    pub title: String,
    pub content: String,
}
pub(crate) fn input<T: serde::de::DeserializeOwned>() -> Result<T> {
    let mut bytes = Zeroizing::new(Vec::new());
    io::stdin()
        .take(70001)
        .read_to_end(&mut bytes)
        .map_err(|_| Error::InvalidInput)?;
    if bytes.len() > 70000 {
        return Err(Error::InvalidInput);
    }
    serde_json::from_slice(&bytes).map_err(|_| Error::InvalidInput)
}
