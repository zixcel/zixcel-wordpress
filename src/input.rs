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
    decode(None)
}
pub(crate) fn decode<T: serde::de::DeserializeOwned>(source: Option<&[u8]>) -> Result<T> {
    let mut bytes = Zeroizing::new(Vec::new());
    if let Some(input) = source {
        if input.len() > 1048576 {
            return Err(Error::InvalidInput);
        }
        bytes.extend_from_slice(input);
    } else {
        io::stdin()
            .take(1048577)
            .read_to_end(&mut bytes)
            .map_err(|_| Error::InvalidInput)?;
    }
    if bytes.len() > 1048576 {
        return Err(Error::InvalidInput);
    }
    serde_json::from_slice(&bytes).map_err(|_| Error::InvalidInput)
}
