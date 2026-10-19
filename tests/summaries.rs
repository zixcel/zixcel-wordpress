mod support;
use serde_json::json;
use support::*;
use zixcel_wordpress::*;
#[test]
fn list_requests_only_summary_fields_and_accepts_rows_without_bodies() {
    for kind in [ContentKind::Posts, ContentKind::Pages] {
        let row = json!({"id":42,"status":"publish","title":{"raw":"Synthetic summary"}});
        let (mut client, calls) = fake(vec![Ok(discovery()), Ok(user()), Ok(json!([row]))]);
        let result = client.list(kind, 2).unwrap();
        assert_eq!(result[0].title, "Synthetic summary");
        let calls = calls.borrow();
        let (_, path, authenticated) = &calls[2];
        assert!(path.contains("per_page=25&page=2&_fields=id,status,title.raw"));
        assert!(!path.contains("content"));
        assert!(*authenticated);
        assert!(
            serde_json::to_value(&result).unwrap()[0]
                .get("content")
                .is_none()
        );
    }
}
#[test]
fn full_content_requires_an_explicit_single_resource_request() {
    let (mut client, calls) = fake(vec![Ok(discovery()), Ok(user()), Ok(draft())]);
    assert_eq!(
        client.get(ContentKind::Posts, 42).unwrap().content,
        "Not a public article."
    );
    let calls = calls.borrow();
    assert!(calls[2].1.starts_with("/wp-json/wp/v2/posts/42?"));
    assert!(
        calls[2]
            .1
            .ends_with("_fields=id,status,title.raw,content.raw")
    );
}
#[test]
fn summary_projection_preserves_limits_and_rejects_invalid_shapes() {
    let row = json!({"id":42,"status":"publish","title":{"raw":"Synthetic"}});
    let (mut client, _) = fake(vec![Ok(discovery()), Ok(user()), Ok(json!(vec![row; 26]))]);
    assert!(matches!(
        client.list(ContentKind::Posts, 1),
        Err(Error::InvalidResponse)
    ));
    let (mut client, _) = fake(vec![Ok(discovery()), Ok(user()), Err(Error::ResponseLimit)]);
    assert!(matches!(
        client.list(ContentKind::Posts, 1),
        Err(Error::ResponseLimit)
    ));
}
