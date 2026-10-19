#![cfg(feature = "contract-fixture")]
//! Actual dispatcher/provider processes and command/backend code; only network transport is synthetic.
#[path = "support/installed.rs"]
mod installed;
use installed::*;
use serde_json::json;
#[test]
fn unpacked_processes_discover_plan_confirm_apply_and_reject_replay() {
    let c = Context::new();
    let out = c
        .command()
        .args(["describe", "wordpress", "--format", "json"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let manifest: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(manifest["commands"].as_array().unwrap().len(), 17);
    c.run(&["posts", "list"], &["--page", "2"]);
    c.run(&["posts", "get"], &["--id", "42"]);
    let input = c.save(
        "input.json",
        &json!({"kind":"posts","title":"Synthetic","content":"Body"}),
    );
    let p = c.run(&["draft", "plan"], &["--input-file", &input]);
    assert!(!c.apply(&["draft", "apply"], &p, &[], false, false));
    assert!(c.apply(&["draft", "apply"], &p, &[], true, true));
    // A second confirmed invocation reaches the real persistent journal and is refused.
    assert!(!c.apply(&["draft", "apply"], &p, &[], true, false));
    let path = c.save("same-plan.json", &p["plan"]);
    let out = c
        .command()
        .args([
            "wordpress",
            "draft",
            "apply",
            "--profile",
            "example",
            "--origin",
            "https://example.test",
            "--collection",
            "synthetic",
            "--plan-file",
            &path,
            "--approval-sha256",
            p["sha256"].as_str().unwrap(),
            "--journal-directory",
            c.root.path().to_str().unwrap(),
        ])
        .output()
        .unwrap();
    assert!(!out.status.success());
    assert!(String::from_utf8_lossy(&out.stderr).contains("interactive_authorization_required"));
    let input = c.save("content.json", &content_input());
    let p = c.run(&["content", "plan"], &["--input-file", &input]);
    assert!(c.apply(&["content", "apply"], &p, &[], true, true));
    let path = c.save("observe.json", &p["plan"]);
    let r = c.run(&["content", "reconcile"], &["--plan-file", &path]);
    assert_eq!(r["value"], "desired_state_observed");
}
#[test]
fn media_and_revocation_cross_the_real_operator_boundary_with_no_real_credentials() {
    let c = Context::new();
    let file = c.directory.join("example.png");
    std::fs::write(&file, b"\x89PNG\r\n\x1a\nsynthetic").unwrap();
    let path = file.to_str().unwrap();
    c.run(&["media", "list"], &["--page", "1"]);
    c.run(&["media", "get"], &["--id", "8"]);
    let input = c.save(
        "media.json",
        &json!({"filename":"example.png","mime":"image/png"}),
    );
    let p = c.run(
        &["media", "plan"],
        &["--file", path, "--input-file", &input],
    );
    assert!(c.apply(&["media", "apply"], &p, &["--file", path], true, true));
    c.run(&["auth", "status"], &[]);
    let p = c.run(&["auth", "revoke-plan"], &[]);
    assert!(c.apply(&["auth", "revoke-apply"], &p, &[], true, true));
}
