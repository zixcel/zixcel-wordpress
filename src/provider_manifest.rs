use serde_json::{Value, json};
use zix_operation_contracts::Binding;
use zixcel_wordpress::{Error, Result};
pub(crate) fn manifest() -> Result<Value> {
    if !cfg!(all(target_os = "linux", feature = "os-keyring")) {
        return Err(Error::Unsupported);
    }
    let value = json!({"schema":"zix-extension/v2","package":"zixcel-wordpress","version":env!("CARGO_PKG_VERSION"),
 "namespace":"wordpress","platform":"linux-x86_64","protocol":"zix-provider/v2",
 "credential_environment":[],"runtime_environment":["DBUS_SESSION_BUS_ADDRESS"],"timeout_ms":40000,
 "commands":crate::provider_ops::ops().map(|op|{
  let mut args=vec![json!({"name":"profile","required":true,"maximum":48}),
   json!({"name":"origin","required":true,"maximum":512}),json!({"name":"collection","required":true,"maximum":64})];
  args.extend(op.extra.iter().map(|(name,required)|json!({"name":name,"required":required,"maximum":if *name=="approval-sha256"{64}else{2048}})));
  json!({"id":op.id,"words":op.words,"effect":op.effect,
    "description":if op.effect=="read"{"Read or plan only; explicit origin-bound OS custody, no automatic unlock."}else{"Explicit local operator write; exact plan approval and one-use journal required."},
    "required_permissions":[if op.effect=="read"{"wordpress:read"}else{"wordpress:write"}],"arguments":args})
 }).collect::<Vec<_>>()});
    let mut manifest = zix_native_provider::Manifest::parse(
        &serde_json::to_vec(&value).map_err(|_| Error::InvalidInput)?,
    )
    .map_err(|_| Error::InvalidInput)?;
    zix_native_provider::guidance::decorate(&mut manifest, include_bytes!("../api/guidance.json"))
        .map_err(|_| Error::InvalidInput)?;
    serde_json::to_value(manifest).map_err(|_| Error::InvalidInput)
}
pub(crate) fn binding() -> Result<Binding> {
    use std::io::Read;
    let bytes = std::fs::File::open("/proc/self/exe").map_err(|_| Error::LocalState)?;
    let mut executable = Vec::new();
    bytes
        .take(128 * 1024 * 1024 + 1)
        .read_to_end(&mut executable)
        .map_err(|_| Error::LocalState)?;
    if executable.len() > 128 * 1024 * 1024 {
        return Err(Error::ResponseLimit);
    }
    use sha2::{Digest, Sha256};
    let hash: String = Sha256::digest(&executable)
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    Ok(Binding::for_catalog(
        "zixcel-wordpress",
        env!("CARGO_PKG_VERSION"),
        &manifest()?,
        &hash,
    ))
}
