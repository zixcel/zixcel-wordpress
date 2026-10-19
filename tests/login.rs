mod support;
use serde_json::json;
use support::*;
use zixcel_wordpress::*;
#[test]
fn checks_discovery_before_any_credential_is_sent() {
    let mut wrong = discovery();
    wrong["home"] = json!("https://other.example");
    let (mut c, calls) = fake(vec![Ok(wrong)]);
    assert!(c.login().is_err());
    assert_eq!(calls.borrow().len(), 1);
    assert!(!calls.borrow()[0].2);
    let (mut c, calls) = fake(vec![Ok(discovery()), Ok(user())]);
    let permission = c.login().unwrap();
    assert_eq!(permission.user_id, 7);
    assert!(!permission.capabilities["publish_posts"]);
    assert!(calls.borrow()[1].2);
}
#[test]
fn permission_and_pagination_failure_prevent_content_requests() {
    let mut denied = user();
    denied["capabilities"]["edit_posts"] = json!(false);
    let (mut c, calls) = fake(vec![Ok(discovery()), Ok(denied)]);
    assert!(matches!(
        c.list(ContentKind::Posts, 1),
        Err(Error::PermissionDenied)
    ));
    assert_eq!(calls.borrow().len(), 2);
    assert!(c.list(ContentKind::Posts, 0).is_err());
    assert_eq!(calls.borrow().len(), 2);
}
