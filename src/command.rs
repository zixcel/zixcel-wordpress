use crate::input::{DraftInput, input};
use serde_json::json;
use std::{
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};
use zixcel_wordpress::{
    ContentKind, DraftPlan, Error, Https, Profile, Result, WordPress, apply_draft, plan_draft,
};
pub(crate) fn run() -> Result<serde_json::Value> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args == ["--help"] || args == ["help"] {
        return Ok(crate::help::describe());
    }
    if args == ["--manifest"] {
        return crate::provider::manifest();
    }
    if args == ["--zix-provider-v1"] {
        return crate::provider::run();
    }
    execute(&args, None)
}
pub(crate) fn execute(args: &[String], collection: Option<&str>) -> Result<serde_json::Value> {
    if args == ["--version"] {
        return Ok(json!({"package":"zixcel-wordpress","version":env!("CARGO_PKG_VERSION")}));
    }
    if args.len() < 3 {
        return Err(Error::InvalidInput);
    }
    let profile = Profile {
        id: args[1].clone(),
        origin: args[2].clone(),
    };
    profile.validate()?;
    if args[0] == "login-browser" {
        return crate::auth_command::browser(profile, args);
    }
    if args[0].starts_with("credential-") {
        return crate::credentials::manage(&profile, args);
    }
    if args[0] == "discover" && args.len() == 3 {
        let mut client =
            WordPress::new(profile.clone(), Https::new(&profile, "unused", "unused")?)?;
        return serde_json::to_value(client.discover()?).map_err(|_| Error::InvalidResponse);
    }
    let (username, password) = crate::credentials::load(&profile, &args[0], collection)?;
    let mut client = WordPress::new(profile.clone(), Https::new(&profile, &username, &password)?)?;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| Error::LocalState)?
        .as_secs();
    let result = match args[0].as_str() {
        "login" if args.len() == 3 => serde_json::to_value(client.login()?),
        "posts-list" if args.len() == 3 => {
            serde_json::to_value(client.list(ContentKind::Posts, 1)?)
        }
        "pages-list" if args.len() == 3 => {
            serde_json::to_value(client.list(ContentKind::Pages, 1)?)
        }
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
            let value: DraftInput = input()?;
            let nonce = crate::content_command::nonce()?;
            let plan = plan_draft(
                &mut client,
                value.kind,
                value.title,
                value.content,
                now,
                nonce,
            )?;
            Ok(json!({"plan":plan,"sha256":plan.digest()?}))
        }
        "draft-apply" if args.len() == 5 => {
            let plan: DraftPlan = input()?;
            serde_json::to_value(apply_draft(
                &mut client,
                &plan,
                &args[3],
                now,
                Path::new(&args[4]),
            )?)
        }
        _ => Ok(crate::content_command::run(&mut client, args, now)?),
    }
    .map_err(|_| Error::InvalidResponse)?;
    if crate::redaction::reflected(&result, &username, &password) {
        return Err(Error::InvalidResponse);
    }
    Ok(result)
}
