use serde_json::{Value, json};
use std::path::Path;
use zixcel_wordpress::{
    Error, Result, RevokePlan, Transport, WordPress, apply_revoke, plan_revoke,
};
pub(crate) fn run<T: Transport>(
    client: &mut WordPress<T>,
    args: &[String],
    now: u64,
    source: Option<&[u8]>,
) -> Result<Value> {
    match args[0].as_str() {
        "auth-status" if args.len() == 3 => {
            let (user_id, credential) = client.credential_identity()?;
            Ok(json!({"user_id":user_id,"credential":credential,"authenticated":true}))
        }
        "revoke-plan" if args.len() == 3 => {
            let p = plan_revoke(client, now, crate::content_command::nonce()?)?;
            Ok(json!({"sha256":p.digest()?,"plan":p}))
        }
        "revoke-apply" if args.len() == 5 => {
            let p: RevokePlan = crate::input::decode(source)?;
            serde_json::to_value(apply_revoke(
                client,
                &p,
                &args[3],
                now,
                Path::new(&args[4]),
            )?)
            .map_err(|_| Error::InvalidResponse)
        }
        _ => Err(Error::InvalidInput),
    }
}
pub(crate) fn browser(profile: zixcel_wordpress::Profile, args: &[String]) -> Result<Value> {
    #[cfg(all(target_os = "linux", feature = "os-keyring"))]
    {
        use crowsi_credential_broker::{BrokerError, CredentialStore, LinuxCustodyStore};
        use std::{io::IsTerminal, time::Duration};
        use zixcel_wordpress::{AuthorizationSession, Https, custody};
        if args.len() != 7
            || args[5] != "--interactive"
            || args[6] != "--loopback-supported"
            || !std::io::stdin().is_terminal()
            || std::env::var_os("GITHUB_ACTIONS").is_some()
        {
            return Err(Error::InvalidInput);
        }
        let store = LinuxCustodyStore::connect(&args[3]).map_err(|_| Error::LocalState)?;
        if store.is_locked().map_err(|_| Error::LocalState)? {
            return Err(Error::InvalidCredential);
        }
        match store.metadata(&custody::reference(&profile)?) {
            Err(BrokerError::NotFound) => {}
            Ok(_) => return Err(Error::Replay),
            Err(_) => return Err(Error::LocalState),
        }
        let mut discovery =
            WordPress::new(profile.clone(), Https::new(&profile, "unused", "unused")?)?;
        let session = AuthorizationSession::bind(
            profile.clone(),
            args[4].clone(),
            true,
            Duration::from_secs(180),
        )?;
        eprintln!(
            "Open this authorization URL in your browser; no credential is in this URL: {}",
            session.authorization_url(&discovery.discover()?)?
        );
        let login = session.receive()?;
        let mut client = WordPress::new(
            profile.clone(),
            Https::new(&profile, &login.username, &login.password)?,
        )?;
        let permission = client.login()?;
        let bytes = zeroize::Zeroizing::new(
            serde_json::to_vec(&json_borrowed(&login.username, &login.password))
                .map_err(|_| Error::InvalidInput)?,
        );
        // Recheck before enrollment; do not replace an existing slot after a concurrent import.
        match store.metadata(&custody::reference(&profile)?) {
            Err(BrokerError::NotFound) => {}
            _ => return Err(Error::Replay),
        }
        let revision = custody::enroll(&store, &profile, &bytes)?;
        Ok(
            json!({"state":"stored","origin":profile.origin,"user_id":permission.user_id,"revision":revision}),
        )
    }
    #[cfg(not(all(target_os = "linux", feature = "os-keyring")))]
    {
        let _ = (profile, args);
        Err(Error::Unsupported)
    }
}
#[cfg(all(target_os = "linux", feature = "os-keyring"))]
#[derive(serde::Serialize)]
struct BorrowedLogin<'a> {
    username: &'a str,
    password: &'a str,
}
#[cfg(all(target_os = "linux", feature = "os-keyring"))]
fn json_borrowed<'a>(username: &'a str, password: &'a str) -> BorrowedLogin<'a> {
    BorrowedLogin { username, password }
}
