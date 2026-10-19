use serde::Deserialize;
use serde_json::{Value, json};
use std::{io::Read, path::Path};
use zixcel_wordpress::{
    ContentEffect, ContentItem, ContentKind, ContentPlan, Error, Result, Transport, WordPress,
    apply_content, plan_content, reconcile_content,
};
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Input {
    kind: ContentKind,
    effect: ContentEffect,
    after: ContentItem,
}
pub(crate) fn nonce() -> Result<String> {
    let mut bytes = [0u8; 16];
    std::fs::File::open("/dev/urandom")
        .and_then(|mut f| f.read_exact(&mut bytes))
        .map_err(|_| Error::LocalState)?;
    Ok(bytes.iter().map(|b| format!("{b:02x}")).collect())
}
pub(crate) fn run<T: Transport>(
    client: &mut WordPress<T>,
    args: &[String],
    now: u64,
) -> Result<Value> {
    match args[0].as_str() {
        "content-plan" if args.len() == 3 => {
            let input: Input = crate::input::input()?;
            let plan = plan_content(client, input.kind, input.effect, input.after, now, nonce()?)?;
            Ok(json!({"sha256":plan.digest()?,"plan":plan}))
        }
        "content-apply" if args.len() == 5 => {
            let plan: ContentPlan = crate::input::input()?;
            serde_json::to_value(apply_content(
                client,
                &plan,
                &args[3],
                now,
                Path::new(&args[4]),
            )?)
            .map_err(|_| Error::InvalidResponse)
        }
        "content-reconcile" if args.len() == 3 => {
            let plan: ContentPlan = crate::input::input()?;
            serde_json::to_value(reconcile_content(client, &plan)?)
                .map_err(|_| Error::InvalidResponse)
        }
        _ => crate::media_command::run(client, args, now),
    }
}
