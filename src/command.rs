use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};
use zixcel_wordpress::{Error, Https, Profile, Result, WordPress};
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
    execute(&args, None, None)
}
pub(crate) fn execute(
    args: &[String],
    collection: Option<&str>,
    source: Option<&[u8]>,
) -> Result<serde_json::Value> {
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
    let result = crate::command_ops::dispatch(&mut client, args, now, source)?;
    if crate::redaction::reflected(&result, &username, &password) {
        return Err(Error::InvalidResponse);
    }
    Ok(result)
}
