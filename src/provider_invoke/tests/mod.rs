use super::*;
mod support;
use support::*;
#[test]
fn actual_command_router_plans_and_applies_one_fixture_draft_through_the_bound_adapter() {
    let directory = tempfile::tempdir().unwrap();
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(directory.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    let input = directory.path().join("input.json");
    save(
        &input,
        &json!({"kind":"posts","title":"Synthetic","content":"Body"}),
    );
    let mut arguments = args();
    arguments.insert("input-file".into(), input.display().to_string());
    let mut c = client(vec![discovery(), user()]);
    let b = binding();
    let planned = invoke(op("wordpress.draft-plan"), &arguments, &b, |a, _, s| {
        crate::command_ops::dispatch(&mut c, a, 100, s)
    })
    .unwrap();
    let plan = directory.path().join("plan.json");
    save(&plan, &planned["plan"]);
    arguments = args();
    arguments.insert("plan-file".into(), plan.display().to_string());
    arguments.insert(
        "approval-sha256".into(),
        planned["sha256"].as_str().unwrap().into(),
    );
    arguments.insert(
        "journal-directory".into(),
        directory.path().display().to_string(),
    );
    let mut c = client(vec![discovery(), user(), content(), content()]);
    let applied = invoke(op("wordpress.draft-apply"), &arguments, &b, |a, _, s| {
        crate::command_ops::dispatch(&mut c, a, 101, s)
    })
    .unwrap();
    assert_eq!(applied["value"]["read_back"], true);
    assert_eq!(
        applied["invocation"]["approved_plan_sha256"],
        planned["sha256"]
    );
    assert_eq!(applied["invocation"]["operation"], "wordpress.draft-apply");
    let mut changed = planned["plan"].clone();
    changed["plan"]["title"] = json!("Changed");
    save(&plan, &changed);
    assert!(
        invoke(
            op("wordpress.draft-apply"),
            &arguments,
            &b,
            |_, _, _| panic!("tampered plan must stop before authentication")
        )
        .is_err()
    );
}
#[test]
fn explicit_page_and_resource_arguments_are_not_ignored() {
    for (id, key, value) in [
        ("wordpress.posts-list", "page", "2"),
        ("wordpress.pages-get", "id", "42"),
        ("wordpress.media-list", "page", "3"),
    ] {
        let mut a = args();
        a.insert(key.into(), value.into());
        invoke(op(id), &a, &binding(), |args, collection, _| {
            assert_eq!(args[3], value);
            assert_eq!(collection, Some("default"));
            Ok(json!([]))
        })
        .unwrap();
    }
    let mut a = args();
    a.insert("unknown".into(), "ignored?".into());
    assert!(
        invoke(op("wordpress.login"), &a, &binding(), |_, _, _| panic!(
            "unknown input must fail"
        ))
        .is_err()
    );
}
#[test]
fn media_plan_and_apply_use_actual_typed_backend_with_an_explicit_file() {
    let d = tempfile::tempdir().unwrap();
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(d.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    let file = d.path().join("x.png");
    std::fs::write(&file, b"\x89PNG\r\n\x1a\nsynthetic").unwrap();
    let input = d.path().join("input.json");
    save(&input, &json!({"filename":"x.png","mime":"image/png"}));
    let mut a = args();
    a.insert("file".into(), file.display().to_string());
    a.insert("input-file".into(), input.display().to_string());
    let mut c = client(vec![discovery(), user()]);
    let b = binding();
    let p = invoke(op("wordpress.media-plan"), &a, &b, |a, _, s| {
        crate::command_ops::dispatch(&mut c, a, 100, s)
    })
    .unwrap();
    let path = d.path().join("plan.json");
    save(&path, &p["plan"]);
    a.remove("input-file");
    a.insert("plan-file".into(), path.display().to_string());
    a.insert(
        "approval-sha256".into(),
        p["sha256"].as_str().unwrap().into(),
    );
    a.insert("journal-directory".into(), d.path().display().to_string());
    let mut c = client(vec![discovery(), user(), media(), media()]);
    let r = invoke(op("wordpress.media-apply"), &a, &b, |a, _, s| {
        crate::command_ops::dispatch(&mut c, a, 101, s)
    })
    .unwrap();
    assert_eq!(r["value"]["read_back"], true);
    assert_eq!(r["value"]["publicly_accessible"], true);
}
