#![cfg(all(target_os = "linux", feature = "os-keyring"))]
use crowsi_credential_broker::LinuxCustodyStore;
use serde_json::Value;
use std::{
    io::Write,
    process::{Command, Output, Stdio},
    sync::Arc,
};
use zixcel_wordpress::{Profile, custody};
fn call(operation: &str, input: &[u8]) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_zix-wordpress"))
        .args([
            operation,
            "example-site",
            "https://site.example.test",
            "default",
        ])
        .env_clear()
        .env(
            "DBUS_SESSION_BUS_ADDRESS",
            std::env::var("DBUS_SESSION_BUS_ADDRESS").unwrap(),
        )
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(input).unwrap();
    child.wait_with_output().unwrap()
}
#[test]
#[ignore = "requires isolated real Secret Service; run scripts/acceptance/keyring.sh"]
fn distinct_cli_processes_share_os_custody_without_secret_output() {
    assert_eq!(std::env::var("ZIX_ISOLATED_KEYRING").as_deref(), Ok("1"));
    let imported = call(
        "credential-import",
        br#"{"username":"synthetic-user","password":"synthetic-password"}"#,
    );
    assert!(
        imported.status.success(),
        "import must succeed without printing raw output"
    );
    for output in [&imported.stdout, &imported.stderr] {
        assert!(!String::from_utf8_lossy(output).contains("synthetic-password"));
    }
    let status = call("credential-status", b"");
    assert!(status.status.success());
    assert!(
        !call(
            "credential-import",
            br#"{"username":"synthetic-user","password":"replacement"}"#
        )
        .status
        .success()
    );
    assert_eq!(
        serde_json::from_slice::<Value>(&status.stdout).unwrap()["state"],
        "available"
    );
    let profile = Profile {
        id: "example-site".into(),
        origin: "https://site.example.test".into(),
    };
    let store = Arc::new(LinuxCustodyStore::connect("default").unwrap());
    assert_eq!(
        custody::use_login(store.clone(), &profile, "login")
            .unwrap()
            .password,
        "synthetic-password"
    );
    assert!(call("credential-lock", b"").status.success());
    assert!(custody::use_login(store, &profile, "login").is_err());
    assert!(
        !call(
            "credential-import",
            br#"{"username":"synthetic","password":"replacement"}"#
        )
        .status
        .success()
    );
    assert_eq!(
        serde_json::from_slice::<Value>(&call("credential-status", b"").stdout).unwrap()["state"],
        "locked"
    );
}
