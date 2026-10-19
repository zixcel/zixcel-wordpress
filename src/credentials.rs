use zeroize::Zeroizing;
use zixcel_wordpress::{Error, Profile, Result};

pub(crate) fn load(
    profile: &Profile,
    operation: &str,
    collection: Option<&str>,
) -> Result<(Zeroizing<String>, Zeroizing<String>)> {
    let selected = collection
        .map(std::ffi::OsString::from)
        .or_else(|| std::env::var_os("ZIX_WORDPRESS_COLLECTION"));
    if let Some(alias) = selected {
        #[cfg(all(target_os = "linux", feature = "os-keyring"))]
        {
            let alias = alias.to_str().ok_or(Error::InvalidInput)?;
            let store = crowsi_credential_broker::LinuxCustodyStore::connect(alias)
                .map_err(|_| Error::LocalState)?;
            let mut login = zixcel_wordpress::custody::use_login(
                std::sync::Arc::new(store),
                profile,
                operation,
            )?;
            return Ok((
                Zeroizing::new(std::mem::take(&mut login.username)),
                Zeroizing::new(std::mem::take(&mut login.password)),
            ));
        }
        #[cfg(not(all(target_os = "linux", feature = "os-keyring")))]
        {
            let _ = (alias, profile, operation);
            return Err(Error::Unsupported);
        }
    }
    let _ = (profile, operation);
    Ok((
        Zeroizing::new(
            std::env::var("ZIX_WORDPRESS_USERNAME").map_err(|_| Error::InvalidCredential)?,
        ),
        Zeroizing::new(
            std::env::var("ZIX_WORDPRESS_APPLICATION_PASSWORD")
                .map_err(|_| Error::InvalidCredential)?,
        ),
    ))
}

pub(crate) fn manage(profile: &Profile, args: &[String]) -> Result<serde_json::Value> {
    #[cfg(all(target_os = "linux", feature = "os-keyring"))]
    {
        use crowsi_credential_broker::{CredentialStore, LinuxCustodyStore};
        use zixcel_wordpress::custody;
        if args.len() != 4 {
            return Err(Error::InvalidInput);
        }
        let store = LinuxCustodyStore::connect(&args[3]).map_err(|_| Error::LocalState)?;
        let reference = custody::reference(profile)?;
        match args[0].as_str() {
            "credential-import" => {
                match store.metadata(&reference) {
                    Err(crowsi_credential_broker::BrokerError::NotFound) => {}
                    Ok(_) => return Err(Error::Replay),
                    Err(_) => return Err(Error::LocalState),
                }
                let mut bytes = Zeroizing::new(Vec::new());
                use std::io::Read;
                std::io::stdin()
                    .take(4097)
                    .read_to_end(&mut bytes)
                    .map_err(|_| Error::InvalidInput)?;
                let revision = custody::enroll(&store, profile, &bytes)?;
                Ok(
                    serde_json::json!({"state":"stored","reference":reference.canonical_id(),"revision":revision}),
                )
            }
            "credential-unlock" => {
                store.unlock().map_err(|_| Error::LocalState)?;
                Ok(serde_json::json!({"state":"unlocked"}))
            }
            "credential-lock" => {
                store.lock().map_err(|_| Error::LocalState)?;
                Ok(serde_json::json!({"state":"locked"}))
            }
            "credential-status" => {
                let locked = store.is_locked().map_err(|_| Error::LocalState)?;
                if locked {
                    return Ok(serde_json::json!({"state":"locked"}));
                }
                let metadata = store
                    .metadata(&reference)
                    .map_err(|_| Error::InvalidCredential)?;
                Ok(
                    serde_json::json!({"state":"available","reference":reference.canonical_id(),"revision":metadata.revision().as_str()}),
                )
            }
            "credential-forget" => {
                store.delete(&reference).map_err(|_| Error::LocalState)?;
                Ok(serde_json::json!({"state":"forgotten","remoteRevoked":false}))
            }
            _ => Err(Error::InvalidInput),
        }
    }
    #[cfg(not(all(target_os = "linux", feature = "os-keyring")))]
    {
        let _ = (profile, args);
        Err(Error::Unsupported)
    }
}
