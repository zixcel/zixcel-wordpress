use super::Context;
use serde_json::Value;
use std::{
    io::Write,
    process::{Command, Stdio},
};
impl Context {
    pub fn apply(
        &self,
        words: &[&str],
        plan: &Value,
        extra: &[&str],
        approve: bool,
        expected: bool,
    ) -> bool {
        let path = self.save("approved.json", &plan["plan"]);
        let hash = plan["sha256"].as_str().unwrap();
        let mut args = vec![
            env!("CARGO_BIN_EXE_zix-dispatch-contract-probe"),
            "wordpress",
        ];
        args.extend(words);
        args.extend([
            "--profile",
            "example",
            "--origin",
            "https://example.test",
            "--collection",
            "synthetic",
            "--plan-file",
            &path,
            "--approval-sha256",
            hash,
            "--journal-directory",
            self.root.path().to_str().unwrap(),
        ]);
        args.extend(extra);
        let quote = |s: &str| format!("'{}'", s.replace('\'', "'\\''"));
        let command = args.into_iter().map(quote).collect::<Vec<_>>().join(" ");
        let mut child = Command::new("script")
            .args(["--quiet", "--return", "--command", &command, "/dev/null"])
            .env_clear()
            .env("PATH", "/usr/bin:/bin")
            .env("XDG_CONFIG_HOME", self.root.path())
            .env(
                "DBUS_SESSION_BUS_ADDRESS",
                "unix:path=/tmp/synthetic-no-real-bus",
            )
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("script must be available; do not skip terminal acceptance");
        writeln!(
            child.stdin.take().unwrap(),
            "{}",
            if approve { hash } else { "" }
        )
        .unwrap();
        let out = child.wait_with_output().unwrap();
        assert_eq!(
            out.status.success(),
            expected,
            "{}",
            String::from_utf8_lossy(&out.stdout)
        );
        if expected {
            let output = String::from_utf8_lossy(&out.stdout);
            let start = output.find('{').expect("protocol receipt missing");
            let receipt: Value =
                serde_json::from_str(output[start..].trim()).expect("invalid protocol receipt");
            assert_eq!(
                receipt["result"]["invocation"]["approved_plan_sha256"],
                hash
            );

            assert!(
                out.status.success(),
                "{}",
                String::from_utf8_lossy(&out.stdout)
            );
            assert!(
                String::from_utf8_lossy(&out.stdout).contains("read_back")
                    || String::from_utf8_lossy(&out.stdout).contains("credential_rejected")
            );
        }
        if approve && !expected {
            assert!(String::from_utf8_lossy(&out.stdout).contains("replay_refused"));
        }
        out.status.success()
    }
}
