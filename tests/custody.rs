#![cfg(all(target_os = "linux", feature = "os-keyring"))]
use crowsi_credential_broker::{CredentialEntry, CredentialStore, MemoryStore, SecretValue};
use std::sync::Arc;
use zixcel_wordpress::{Profile, custody};
fn profile(origin: &str) -> Profile {
    Profile {
        id: "example-site".into(),
        origin: origin.into(),
    }
}
const LOGIN: &[u8] =
    br#"{"username":"synthetic-user","password":"synthetic-application-password"}"#;

#[test]
fn saved_login_is_origin_bound_and_rotation_is_observed() {
    let store = Arc::new(MemoryStore::default());
    let selected = profile("https://site.example.test");
    let revision = custody::enroll(&*store, &selected, LOGIN).unwrap();
    let login = custody::use_login(store.clone(), &selected, "login").unwrap();
    assert_eq!(login.username, "synthetic-user");
    let changed = br#"{"username":"synthetic-user","password":"rotated-synthetic-password"}"#;
    assert_ne!(
        revision,
        custody::enroll(&*store, &selected, changed).unwrap()
    );
    assert_eq!(
        custody::use_login(store.clone(), &selected, "login")
            .unwrap()
            .password,
        "rotated-synthetic-password"
    );
    assert!(
        custody::use_login(
            store.clone(),
            &profile("https://other.example.test"),
            "login"
        )
        .is_err()
    );
    store
        .delete(&custody::reference(&selected).unwrap())
        .unwrap();
    assert!(custody::use_login(store, &selected, "login").is_err());
}

#[test]
fn host_permission_and_embedded_origin_are_both_checked() {
    let selected = profile("https://site.example.test");
    let store = Arc::new(MemoryStore::default());
    for (host, payload) in [
        ("other.example.test", LOGIN.to_vec()),
        ("site.example.test", serde_json::to_vec(&serde_json::json!({"schema":"zix-wordpress-login/v1",
            "profile":profile("https://other.example.test"), "login":{"username":"synthetic","password":"synthetic"}})).unwrap()),
    ] {
        store.put(CredentialEntry::new(custody::reference(&selected).unwrap(), [host.into()], SecretValue::new(payload).unwrap()).unwrap()).unwrap();
        assert!(custody::use_login(store.clone(), &selected, "login").is_err());
    }
}

#[test]
fn invalid_enrollment_does_not_replace_a_valid_credential() {
    let store = Arc::new(MemoryStore::default());
    let selected = profile("https://site.example.test");
    custody::enroll(&*store, &selected, LOGIN).unwrap();
    for bytes in [
        br#"{"username":"","password":"synthetic"}"#.as_slice(),
        br#"{"username":"synthetic","password":"synthetic","extra":true}"#,
        b"not-json",
        &vec![0; 4097],
    ] {
        assert!(custody::enroll(&*store, &selected, bytes).is_err());
    }
    assert_eq!(
        custody::use_login(store, &selected, "login")
            .unwrap()
            .password,
        "synthetic-application-password"
    );
}
