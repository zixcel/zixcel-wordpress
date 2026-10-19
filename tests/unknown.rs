mod support;
use support::*;
use zixcel_wordpress::*;
#[test]
fn uncertain_write_is_journaled_and_never_retried() {
    use std::os::unix::fs::PermissionsExt;
    let (mut c, calls) = fake(vec![
        Ok(discovery()),
        Ok(user()),
        Ok(discovery()),
        Ok(user()),
        Err(Error::Transport),
        Ok(discovery()),
        Ok(user()),
    ]);
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
    std::fs::set_permissions(d.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    assert!(matches!(
        apply_draft(&mut c, &plan, &plan.digest().unwrap(), 101, d.path()),
        Err(Error::UnknownResult)
    ));
    assert!(matches!(
        apply_draft(&mut c, &plan, &plan.digest().unwrap(), 102, d.path()),
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
