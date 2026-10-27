use std::process::Command;
#[test]
fn metadata_requires_built_custody_but_never_connects_to_it() {
    let o = Command::new(env!("CARGO_BIN_EXE_zix-wordpress"))
        .arg("--guidance")
        .env_clear()
        .output()
        .unwrap();
    if cfg!(all(target_os = "linux", feature = "os-keyring")) {
        assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
        let c: serde_json::Value = serde_json::from_slice(&o.stdout).unwrap();
        assert_eq!(c["namespace"], "wordpress");
        assert_eq!(c["entries"].as_array().unwrap().len(), 17);
    } else {
        assert!(!o.status.success());
    }
}
