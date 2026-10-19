use serde_json::{Value, json};
use std::{
    io::Write,
    process::{Command, Output, Stdio},
};
fn request(value: &Value) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_zix-wordpress"))
        .arg("--zix-provider-v1")
        .env_clear()
        .env(
            "DBUS_SESSION_BUS_ADDRESS",
            "unix:path=/tmp/nonexistent-wordpress-test-bus",
        )
        .env("ZIX_WORDPRESS_USERNAME", "synthetic-legacy-user")
        .env(
            "ZIX_WORDPRESS_APPLICATION_PASSWORD",
            "synthetic-legacy-password",
        )
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(value.to_string().as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}
fn valid() -> Value {
    json!({"schema":"zix-provider/v1","id":"a".repeat(32),"operation":"wordpress.login",
        "arguments":{"profile":"example-site","origin":"https://site.example.test","collection":"default"},
        "limits":{"timeout_ms":40000,"maximum_response_bytes":131072},"binding":binding()})
}
fn binding() -> zix_operation_contracts::Binding {
    use sha2::{Digest, Sha256};
    let path = env!("CARGO_BIN_EXE_zix-wordpress");
    let out = Command::new(path).arg("--manifest").output().unwrap();
    let manifest: Value = serde_json::from_slice(&out.stdout).unwrap();
    let hash: String = Sha256::digest(std::fs::read(path).unwrap())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect();
    zix_operation_contracts::Binding::for_catalog(
        "zixcel-wordpress",
        env!("CARGO_PKG_VERSION"),
        &manifest,
        &hash,
    )
}
#[test]
fn only_built_capabilities_are_advertised_without_credentials() {
    let out = Command::new(env!("CARGO_BIN_EXE_zix-wordpress"))
        .arg("--manifest")
        .env_clear()
        .output()
        .unwrap();
    if cfg!(all(target_os = "linux", feature = "os-keyring")) {
        assert!(out.status.success());
        let manifest: Value = serde_json::from_slice(&out.stdout).unwrap();
        assert_eq!(
            manifest["runtime_environment"],
            json!(["DBUS_SESSION_BUS_ADDRESS"])
        );
        assert_eq!(manifest["credential_environment"], json!([]));
        assert!(
            manifest["commands"]
                .as_array()
                .unwrap()
                .iter()
                .all(|c| c["effect"] == "read" || c["effect"] == "operator-write")
        );
    } else {
        assert!(!out.status.success());
    }
}
#[test]
fn unsupported_operations_and_protocol_mutations_cannot_trigger_an_operation() {
    for field in ["schema", "id", "operation", "limits"] {
        let mut value = valid();
        value[field] = json!("invalid-synthetic");
        let out = request(&value);
        assert!(!out.status.success());
        assert!(out.stdout.is_empty());
    }
    let mut value = valid();
    value["operation"] = json!("wordpress.destroy");
    assert!(!request(&value).status.success());
}
#[test]
fn unavailable_custody_never_falls_back_to_password_environment_or_leaks_it() {
    let out = request(&valid());
    assert!(out.status.success());
    let value: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(value["error"], "wordpress_operation_failed");
    for bytes in [&out.stdout, &out.stderr] {
        assert!(!String::from_utf8_lossy(bytes).contains("synthetic-legacy-password"));
    }
    assert_eq!(value["id"], "a".repeat(32));
}
