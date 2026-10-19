use serde_json::Value;
use zixcel_wordpress::{ContentPlan, DraftPlan, Error, Result, RevokePlan, UploadPlan};
pub(crate) fn inner_digest(native: &str, payload: &Value) -> Result<String> {
    macro_rules! parse {
        ($t:ty) => {
            serde_json::from_value::<$t>(payload.clone())
                .map_err(|_| Error::InvalidInput)?
                .digest()
        };
    }
    match native {
        "draft-apply" => parse!(DraftPlan),
        "content-apply" => parse!(ContentPlan),
        "media-apply" => parse!(UploadPlan),
        "revoke-apply" => parse!(RevokePlan),
        _ => Err(Error::InvalidInput),
    }
}
