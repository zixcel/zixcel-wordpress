use zix_native_provider::GuidanceCatalog;
use zixcel_wordpress::{Error, Result};
pub(crate) fn catalog() -> Result<GuidanceCatalog> {
    let manifest = serde_json::from_value(crate::provider_manifest::manifest()?)
        .map_err(|_| Error::InvalidInput)?;
    zix_native_provider::guidance::catalog(&manifest, include_bytes!("../api/guidance.json"))
        .map_err(|_| Error::InvalidInput)
}
