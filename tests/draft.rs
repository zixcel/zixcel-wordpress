mod support;
use support::*;
use zixcel_wordpress::*;
#[test]
fn draft_plan_apply_readback_and_restart_replay_are_bound_to_one_site_and_user() {
    use std::os::unix::fs::PermissionsExt;
    let (mut c, calls) = fake(vec![
        Ok(discovery()),
        Ok(user()),
        Ok(discovery()),
        Ok(user()),
        Ok(draft()),
        Ok(draft()),
        Ok(discovery()),
        Ok(user()),
    ]);
    let plan = plan_draft(
        &mut c,
        ContentKind::Posts,
        "Synthetic acceptance".into(),
        "Not a public article.".into(),
        100,
        "a".repeat(32),
    )
    .unwrap();
    let dir = tempfile::tempdir().unwrap();
    std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    let receipt = apply_draft(&mut c, &plan, &plan.digest().unwrap(), 101, dir.path()).unwrap();
    assert!(receipt.read_back);
    assert_eq!(receipt.id, 42);
    assert!(matches!(
        apply_draft(&mut c, &plan, &plan.digest().unwrap(), 102, dir.path()),
        Err(Error::Replay)
    ));
    assert_eq!(
        calls
            .borrow()
            .iter()
            .filter(|c| c.0 == Method::Post)
            .count(),
        1
    );
}
#[test]
fn changed_origin_digest_user_or_expiry_never_writes() {
    let (mut c, calls) = fake(vec![Ok(discovery()), Ok(user())]);
    let plan = plan_draft(
        &mut c,
        ContentKind::Posts,
        "Synthetic acceptance".into(),
        String::new(),
        100,
        "a".repeat(32),
    )
    .unwrap();
    let d = tempfile::tempdir().unwrap();
    assert!(matches!(
        apply_draft(&mut c, &plan, "wrong", 101, d.path()),
        Err(Error::PlanMismatch)
    ));
    assert!(apply_draft(&mut c, &plan, &plan.digest().unwrap(), 701, d.path()).is_err());
    let mut other = plan.clone();
    other.profile.id = "other".into();
    assert!(matches!(
        apply_draft(&mut c, &other, &other.digest().unwrap(), 101, d.path()),
        Err(Error::PlanMismatch)
    ));
    assert_eq!(calls.borrow().len(), 2);
}
