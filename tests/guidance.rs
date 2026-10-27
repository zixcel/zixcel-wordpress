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
        let entries = c["entries"].as_array().unwrap();
        assert_eq!(entries.len(), 17);
        for guide in entries {
            assert!(guide["requirements"].as_array().unwrap().iter().any(|r| {
                r["value"]
                    .as_str()
                    .unwrap()
                    .contains("zix-wordpress credential-import")
            }));
            assert!(
                guide["next_actions"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .all(|a| a["namespace"] != "credential")
            );
        }
    } else {
        assert!(!o.status.success());
    }
}
