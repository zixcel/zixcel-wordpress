mod support;
use serde_json::json;
use support::media::*;
use support::*;
use zixcel_wordpress::*;
#[test]
fn invalid_files_and_changed_payload_never_upload() {
    for (name, mime, b) in [
        ("../x.png", "image/png", bytes()),
        ("x.svg", "image/svg+xml", b"<svg/>".to_vec()),
        ("x.png", "image/png", b"not PNG".to_vec()),
        ("x.png", "image/png", vec![0; 8 * 1024 * 1024 + 1]),
    ] {
        let (mut c, calls) = fake(vec![]);
        assert!(plan_upload(&mut c, name.into(), mime.into(), &b, 100, "a".repeat(32)).is_err());
        assert!(calls.borrow().is_empty());
    }
    let (mut c, calls) = fake(vec![Ok(discovery()), Ok(actor())]);
    let p = plan_upload(
        &mut c,
        "x.png".into(),
        "image/png".into(),
        &bytes(),
        100,
        "a".repeat(32),
    )
    .unwrap();
    let mut changed = bytes();
    changed.push(0);
    assert!(
        apply_upload(
            &mut c,
            &p,
            &p.digest().unwrap(),
            &changed,
            101,
            tempfile::tempdir().unwrap().path()
        )
        .is_err()
    );
    assert_eq!(calls.borrow().len(), 2);
}
#[test]
fn permission_or_partial_read_failure_never_looks_like_empty_media() {
    let (mut c, _) = fake(vec![Ok(discovery()), Ok(user())]);
    assert!(matches!(c.media_list(1), Err(Error::PermissionDenied)));
    let (mut c, calls) = fake(vec![]);
    assert!(c.media_list(0).is_err());
    assert!(calls.borrow().is_empty());
    let (mut c, _) = fake(vec![Ok(discovery()), Ok(actor()), Err(Error::Status(429))]);
    assert!(matches!(c.media_list(2), Err(Error::Status(429))));
    let mut foreign = media();
    foreign["source_url"] = json!("https://other.example/file.png");
    let (mut c, _) = fake(vec![Ok(discovery()), Ok(actor()), Ok(foreign)]);
    assert!(matches!(c.media_get(91), Err(Error::UnsafeTarget)));
}
