mod support;
use support::media::*;
use support::*;
use zixcel_wordpress::*;
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
