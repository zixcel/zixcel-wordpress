#[path = "installed/operator.rs"]
mod operator;
use serde_json::{Value, json};
use std::{fs, os::unix::fs::PermissionsExt, path::PathBuf, process::Command};
pub struct Context {
    pub root: tempfile::TempDir,
    pub directory: PathBuf,
}
impl Context {
    pub fn new() -> Self {
        let root = tempfile::tempdir().unwrap();
        fs::set_permissions(root.path(), fs::Permissions::from_mode(0o700)).unwrap();
        let directory = root.path().join("unpacked provider");
        fs::create_dir(&directory).unwrap();
        let binary = directory.join("zix-wordpress-fixture");
        fs::copy(env!("CARGO_BIN_EXE_zix-wordpress-contract-probe"), &binary).unwrap();
        fs::set_permissions(&binary, fs::Permissions::from_mode(0o700)).unwrap();
        // Only the immutable build output is executed to obtain metadata.
        // The unpacked copy is executed later through the dispatcher’s sealed snapshot.
        // This avoids inherited writable-copy descriptors during parallel fork/exec.
        let out = Command::new(env!("CARGO_BIN_EXE_zix-wordpress-contract-probe"))
            .arg("--manifest")
            .env_clear()
            .output()
            .unwrap();
        assert!(out.status.success());
        let manifest = directory.join("wordpress.extension.json");
        fs::write(&manifest, &out.stdout).unwrap();
        let hash = |path: &PathBuf| {
            use sha2::{Digest, Sha256};
            Sha256::digest(fs::read(path).unwrap())
                .iter()
                .map(|b| format!("{b:02x}"))
                .collect::<String>()
        };
        let c = Self { root, directory };
        let out = c
            .command()
            .args([
                "extensions",
                "add",
                "--manifest",
                manifest.to_str().unwrap(),
                "--manifest-sha256",
                &hash(&manifest),
                "--executable",
                binary.to_str().unwrap(),
                "--executable-sha256",
                &hash(&binary),
            ])
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        c
    }
    pub fn command(&self) -> Command {
        let mut c = Command::new(env!("CARGO_BIN_EXE_zix-dispatch-contract-probe"));
        c.env_clear().env("XDG_CONFIG_HOME", self.root.path()).env(
            "DBUS_SESSION_BUS_ADDRESS",
            "unix:path=/tmp/synthetic-no-real-bus",
        );
        c
    }
    pub fn save(&self, name: &str, value: &Value) -> String {
        let p = self.directory.join(name);
        fs::write(&p, serde_json::to_vec(value).unwrap()).unwrap();
        fs::set_permissions(&p, fs::Permissions::from_mode(0o600)).unwrap();
        p.to_str().unwrap().into()
    }
    pub fn run(&self, words: &[&str], extra: &[&str]) -> Value {
        let mut args = vec!["wordpress"];
        args.extend(words);
        args.extend([
            "--profile",
            "example",
            "--origin",
            "https://example.test",
            "--collection",
            "synthetic",
        ]);
        args.extend(extra);
        let out = self.command().args(args).output().unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        let r: Value = serde_json::from_slice(&out.stdout).unwrap();
        assert!(r.get("error").is_none(), "{r}");
        r["result"].clone()
    }
}
pub fn content_input() -> Value {
    json!({"kind":"posts","effect":"edit_draft","after":{"id":42,"status":"draft","title":"Changed","content":"Body"}})
}
