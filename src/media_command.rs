use serde::Deserialize;
use serde_json::{Value, json};
use std::path::Path;
use zixcel_wordpress::{
    Error, Result, Transport, UploadPlan, WordPress, apply_upload, plan_upload,
};
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Input {
    filename: String,
    mime: String,
}
pub(crate) fn run<T: Transport>(
    client: &mut WordPress<T>,
    args: &[String],
    now: u64,
    source: Option<&[u8]>,
) -> Result<Value> {
    let result = match args[0].as_str() {
        "media-list" if args.len() == 4 => serde_json::to_value(
            client.media_list(args[3].parse().map_err(|_| Error::InvalidInput)?)?,
        ),
        "media-get" if args.len() == 4 => serde_json::to_value(
            client.media_get(args[3].parse().map_err(|_| Error::InvalidInput)?)?,
        ),
        "media-plan" if args.len() == 4 => {
            let input: Input = crate::input::decode(source)?;
            let bytes = crate::local_file::read(&args[3])?;
            let plan = plan_upload(
                client,
                input.filename,
                input.mime,
                &bytes,
                now,
                crate::content_command::nonce()?,
            )?;
            return Ok(json!({"sha256":plan.digest()?,"plan":plan}));
        }
        "media-apply" if args.len() == 6 => {
            let plan: UploadPlan = crate::input::decode(source)?;
            let bytes = crate::local_file::read(&args[3])?;
            serde_json::to_value(apply_upload(
                client,
                &plan,
                &args[4],
                &bytes,
                now,
                Path::new(&args[5]),
            )?)
        }
        _ => return crate::auth_command::run(client, args, now, source),
    };
    result.map_err(|_| Error::InvalidResponse)
}
