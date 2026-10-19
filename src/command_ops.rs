use crate::input::DraftInput;
use serde_json::json;
use std::path::Path;
use zixcel_wordpress::{
    ContentKind, DraftPlan, Error, Result, Transport, WordPress, apply_draft, plan_draft,
};
pub(crate) fn dispatch<T: Transport>(
    client: &mut WordPress<T>,
    args: &[String],
    now: u64,
    source: Option<&[u8]>,
) -> Result<serde_json::Value> {
    let result = match args[0].as_str() {
        "login" if args.len() == 3 => serde_json::to_value(client.login()?),
        "posts-list" if (3..=4).contains(&args.len()) => serde_json::to_value(
            client.list(
                ContentKind::Posts,
                args.get(3)
                    .map(|s| s.parse())
                    .transpose()
                    .map_err(|_| Error::InvalidInput)?
                    .unwrap_or(1),
            )?,
        ),
        "pages-list" if (3..=4).contains(&args.len()) => serde_json::to_value(
            client.list(
                ContentKind::Pages,
                args.get(3)
                    .map(|s| s.parse())
                    .transpose()
                    .map_err(|_| Error::InvalidInput)?
                    .unwrap_or(1),
            )?,
        ),
        "posts-get" | "pages-get" if args.len() == 4 => {
            let id = args[3].parse::<u64>().map_err(|_| Error::InvalidInput)?;
            let kind = if args[0] == "posts-get" {
                ContentKind::Posts
            } else {
                ContentKind::Pages
            };
            serde_json::to_value(client.get(kind, id)?)
        }
        "draft-plan" if args.len() == 3 => {
            let value: DraftInput = crate::input::decode(source)?;
            let nonce = crate::content_command::nonce()?;
            let plan = plan_draft(client, value.kind, value.title, value.content, now, nonce)?;
            Ok(json!({"plan":plan,"sha256":plan.digest()?}))
        }
        "draft-apply" if args.len() == 5 => {
            let plan: DraftPlan = crate::input::decode(source)?;
            serde_json::to_value(apply_draft(
                client,
                &plan,
                &args[3],
                now,
                Path::new(&args[4]),
            )?)
        }
        _ => Ok(crate::content_command::run(client, args, now, source)?),
    }
    .map_err(|_| Error::InvalidResponse)?;

    Ok(result)
}
