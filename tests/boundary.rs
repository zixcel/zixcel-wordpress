mod support;
use serde_json::json;
use support::*;
use zixcel_wordpress::*;
#[test]
fn a_different_authenticated_user_cannot_apply_an_approved_plan() {
    let mut other = user();
    other["id"] = json!(8);
    let (mut client, calls) = fake(vec![
        Ok(discovery()),
        Ok(user()),
        Ok(discovery()),
        Ok(other),
    ]);
    let plan = plan_draft(
        &mut client,
        ContentKind::Posts,
        "Synthetic".into(),
        String::new(),
        100,
        "a".repeat(32),
    )
    .unwrap();
    let directory = tempfile::tempdir().unwrap();
    assert!(matches!(
        apply_draft(
            &mut client,
            &plan,
            &plan.digest().unwrap(),
            101,
            directory.path()
        ),
        Err(Error::PlanMismatch)
    ));
    assert!(!calls.borrow().iter().any(|c| c.0 == Method::Post));
}
#[test]
fn authentication_rejection_is_not_an_empty_content_list() {
    let (mut client, calls) = fake(vec![Ok(discovery()), Err(Error::Status(401))]);
    assert!(matches!(
        client.list(ContentKind::Posts, 1),
        Err(Error::Status(401))
    ));
    assert_eq!(calls.borrow().len(), 2);
}
#[test]
fn write_readback_mismatch_preserves_the_journal_without_success() {
    use std::os::unix::fs::PermissionsExt;
    let mut mismatch = draft();
    mismatch["status"] = json!("publish");
    let (mut client, calls) = fake(vec![
        Ok(discovery()),
        Ok(user()),
        Ok(discovery()),
        Ok(user()),
        Ok(draft()),
        Ok(mismatch),
    ]);
    let plan = plan_draft(
        &mut client,
        ContentKind::Posts,
        "Synthetic acceptance".into(),
        "Not a public article.".into(),
        100,
        "a".repeat(32),
    )
    .unwrap();
    let directory = tempfile::tempdir().unwrap();
    std::fs::set_permissions(directory.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    assert!(matches!(
        apply_draft(
            &mut client,
            &plan,
            &plan.digest().unwrap(),
            101,
            directory.path()
        ),
        Err(Error::UnknownResult)
    ));
    let journal = std::fs::read_to_string(
        directory
            .path()
            .join(format!("{}.json", plan.digest().unwrap())),
    )
    .unwrap();
    assert!(journal.contains("do not replay"));
    assert_eq!(
        calls
            .borrow()
            .iter()
            .filter(|c| c.0 == Method::Post)
            .count(),
        1
    );
}
