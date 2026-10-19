mod support;
use serde_json::json;
use support::*;
use zixcel_wordpress::*;
fn identity() -> serde_json::Value {
    json!({"uuid":"aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee","name":"Synthetic client"})
}
#[test]
fn revocation_requires_exact_identity_and_observed_authentication_rejection() {
    use std::os::unix::fs::PermissionsExt;
    let (mut c, calls) = fake(vec![
        Ok(discovery()),
        Ok(user()),
        Ok(identity()),
        Ok(discovery()),
        Ok(user()),
        Ok(identity()),
        Ok(json!({"deleted":true})),
        Err(Error::Status(401)),
    ]);
    let p = plan_revoke(&mut c, 100, "a".repeat(32)).unwrap();
    let d = tempfile::tempdir().unwrap();
    std::fs::set_permissions(d.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    let r = apply_revoke(&mut c, &p, &p.digest().unwrap(), 101, d.path()).unwrap();
    assert!(r.server_deleted && r.credential_rejected && !r.local_forgotten);
    assert_eq!(
        calls
            .borrow()
            .iter()
            .filter(|x| x.0 == Method::Delete)
            .count(),
        1
    );
}
#[test]
fn wrong_identity_and_unknown_outcome_never_erase_local_custody_or_retry() {
    use std::os::unix::fs::PermissionsExt;
    for changed in [true, false] {
        let mut other = identity();
        if changed {
            other["uuid"] = json!("ffffffff-bbbb-cccc-dddd-eeeeeeeeeeee");
        }
        let (mut c, calls) = fake(vec![
            Ok(discovery()),
            Ok(user()),
            Ok(identity()),
            Ok(discovery()),
            Ok(user()),
            Ok(other),
            Err(Error::Transport),
        ]);
        let p = plan_revoke(&mut c, 100, "a".repeat(32)).unwrap();
        let d = tempfile::tempdir().unwrap();
        std::fs::set_permissions(d.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
        assert_eq!(
            apply_revoke(&mut c, &p, &p.digest().unwrap(), 101, d.path()).unwrap_err(),
            if changed {
                Error::Conflict
            } else {
                Error::UnknownResult
            }
        );
        assert_eq!(
            calls
                .borrow()
                .iter()
                .filter(|x| x.0 == Method::Delete)
                .count(),
            if changed { 0 } else { 1 }
        );
    }
}
