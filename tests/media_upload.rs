mod support;
use support::media::*;
use support::*;
use zixcel_wordpress::*;
#[test]
fn upload_is_explicitly_public_hash_bound_and_read_back_without_attachment() {
    use std::os::unix::fs::PermissionsExt;
    let (mut c, calls) = fake(vec![
        Ok(discovery()),
        Ok(actor()),
        Ok(discovery()),
        Ok(actor()),
        Ok(media()),
        Ok(media()),
        Ok(discovery()),
        Ok(actor()),
    ]);
    let b = bytes();
    let p = plan_upload(
        &mut c,
        "example.png".into(),
        "image/png".into(),
        &b,
        100,
        "a".repeat(32),
    )
    .unwrap();
    assert!(p.publicly_accessible);
    let d = tempfile::tempdir().unwrap();
    std::fs::set_permissions(d.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    let r = apply_upload(&mut c, &p, &p.digest().unwrap(), &b, 101, d.path()).unwrap();
    assert!(r.read_back && r.publicly_accessible && !r.attached_to_post);
    assert!(matches!(
        apply_upload(&mut c, &p, &p.digest().unwrap(), &b, 102, d.path()),
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
fn uncertain_upload_is_not_retried_or_automatically_deleted() {
    use std::os::unix::fs::PermissionsExt;
    let (mut c, calls) = fake(vec![
        Ok(discovery()),
        Ok(actor()),
        Ok(discovery()),
        Ok(actor()),
        Err(Error::Transport),
        Ok(discovery()),
        Ok(actor()),
    ]);
    let b = bytes();
    let p = plan_upload(
        &mut c,
        "x.png".into(),
        "image/png".into(),
        &b,
        100,
        "a".repeat(32),
    )
    .unwrap();
    let d = tempfile::tempdir().unwrap();
    std::fs::set_permissions(d.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    assert!(matches!(
        apply_upload(&mut c, &p, &p.digest().unwrap(), &b, 101, d.path()),
        Err(Error::UnknownResult)
    ));
    assert!(matches!(
        apply_upload(&mut c, &p, &p.digest().unwrap(), &b, 102, d.path()),
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
fn known_uploaded_id_is_retained_when_readback_is_unavailable() {
    use std::os::unix::fs::PermissionsExt;
    let (mut c, _) = fake(vec![
        Ok(discovery()),
        Ok(actor()),
        Ok(discovery()),
        Ok(actor()),
        Ok(media()),
        Err(Error::Transport),
    ]);
    let b = bytes();
    let p = plan_upload(
        &mut c,
        "x.png".into(),
        "image/png".into(),
        &b,
        100,
        "a".repeat(32),
    )
    .unwrap();
    let d = tempfile::tempdir().unwrap();
    std::fs::set_permissions(d.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    assert!(matches!(
        apply_upload(&mut c, &p, &p.digest().unwrap(), &b, 101, d.path()),
        Err(Error::UnknownResult)
    ));
    let receipt: serde_json::Value = serde_json::from_slice(
        &std::fs::read(d.path().join(format!("{}.json", p.digest().unwrap()))).unwrap(),
    )
    .unwrap();
    assert_eq!(receipt["id"], 91);
    assert_eq!(receipt["read_back"], false);
}
