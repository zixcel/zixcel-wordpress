use serde_json::{Value, json};
pub(crate) fn describe() -> Value {
    let mut commands = vec![
        "discover PROFILE ORIGIN",
        "login PROFILE ORIGIN",
        "posts-list PROFILE ORIGIN",
        "pages-list PROFILE ORIGIN",
        "posts-get PROFILE ORIGIN ID",
        "pages-get PROFILE ORIGIN ID",
        "draft-plan PROFILE ORIGIN (JSON stdin)",
        "content-plan PROFILE ORIGIN (kind/effect/after JSON stdin)",
        "content-apply PROFILE ORIGIN APPROVED_SHA256 PRIVATE_JOURNAL_DIRECTORY (plan JSON stdin)",
        "content-reconcile PROFILE ORIGIN (plan JSON stdin; observation only)",
        "draft-apply PROFILE ORIGIN APPROVED_SHA256 PRIVATE_JOURNAL_DIRECTORY (plan JSON stdin)",
    ];
    if cfg!(all(target_os = "linux", feature = "os-keyring")) {
        commands.extend([
            "credential-unlock PROFILE ORIGIN COLLECTION",
            "credential-import PROFILE ORIGIN COLLECTION (username/password JSON stdin)",
            "credential-status PROFILE ORIGIN COLLECTION",
            "credential-lock PROFILE ORIGIN COLLECTION",
            "credential-forget PROFILE ORIGIN COLLECTION (local only; does not revoke remotely)",
        ]);
    }
    json!({"package":"zixcel-wordpress","version":env!("CARGO_PKG_VERSION"),"commands":commands,
        "authentication":"Select ZIX_WORDPRESS_COLLECTION for stored OS credentials; never auto-unlock or fall back when selected.",
        "approval":"Unlocking never authorizes a draft write. Review the current plan and pass its exact hash.",
        "runtime":"Native CI releases target Ubuntu 24.04-compatible GNU/Linux with OS-managed libsecret and GLib."})
}
