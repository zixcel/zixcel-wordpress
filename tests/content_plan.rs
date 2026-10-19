mod support;
use serde_json::json;
use support::mutations::*;
use support::*;
use zixcel_wordpress::*;
#[test]
fn drift_and_permission_loss_stop_before_writing() {
    for field in ["title", "status", "modified_gmt", "permission"] {
        let mut changed = state("draft", "Before");
        let mut permission = actor();
        match field {
            "title" => changed["title"]["raw"] = json!("Other"),
            "status" => changed["status"] = json!("publish"),
            "modified_gmt" => changed["modified_gmt"] = json!("2026-01-01T00:00:01"),
            _ => permission["capabilities"]["publish_posts"] = json!(false),
        }
        let (mut c, calls) = fake(vec![
            Ok(discovery()),
            Ok(actor()),
            Ok(state("draft", "Before")),
            Ok(discovery()),
            Ok(permission),
            Ok(changed),
        ]);
        let p = plan_content(
            &mut c,
            ContentKind::Posts,
            ContentEffect::Publish,
            desired("publish"),
            100,
            "a".repeat(32),
        )
        .unwrap();
        let e =
            apply_content(&mut c, &p, &p.digest().unwrap(), 101, directory().path()).unwrap_err();
        assert_eq!(
            e,
            if field == "permission" {
                Error::PermissionDenied
            } else {
                Error::Conflict
            }
        );
        assert!(!calls.borrow().iter().any(|c| c.0 == Method::Post));
    }
}
#[test]
fn altered_digest_expiry_effect_or_site_never_writes() {
    let (mut c, calls) = fake(vec![
        Ok(discovery()),
        Ok(actor()),
        Ok(state("draft", "Before")),
    ]);
    let p = plan_content(
        &mut c,
        ContentKind::Posts,
        ContentEffect::Publish,
        desired("publish"),
        100,
        "a".repeat(32),
    )
    .unwrap();
    let d = directory();
    assert_eq!(
        apply_content(&mut c, &p, "wrong", 101, d.path()).unwrap_err(),
        Error::PlanMismatch
    );
    assert!(apply_content(&mut c, &p, &p.digest().unwrap(), 701, d.path()).is_err());
    let mut other = p.clone();
    other.profile.id = "another".into();
    assert_eq!(
        apply_content(&mut c, &other, &other.digest().unwrap(), 101, d.path()).unwrap_err(),
        Error::PlanMismatch
    );
    other = p.clone();
    other.effect = ContentEffect::EditDraft;
    assert!(apply_content(&mut c, &other, &other.digest().unwrap(), 101, d.path()).is_err());
    assert_eq!(calls.borrow().len(), 3);
}
