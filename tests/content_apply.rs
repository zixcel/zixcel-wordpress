mod support;
use support::mutations::*;
use support::*;
use zixcel_wordpress::*;
#[test]
fn edit_publish_and_published_edit_have_separate_effects_and_one_use_receipts() {
    for (effect, before, after) in [
        (ContentEffect::EditDraft, "draft", "draft"),
        (ContentEffect::Publish, "draft", "publish"),
        (ContentEffect::EditPublished, "publish", "publish"),
    ] {
        let (mut c, calls) = fake(vec![
            Ok(discovery()),
            Ok(actor()),
            Ok(state(before, "Before")),
            Ok(discovery()),
            Ok(actor()),
            Ok(state(before, "Before")),
            Ok(state(after, "After")),
            Ok(state(after, "After")),
            Ok(discovery()),
            Ok(actor()),
            Ok(state(before, "Before")),
        ]);
        let p = plan_content(
            &mut c,
            ContentKind::Posts,
            effect,
            desired(after),
            100,
            "a".repeat(32),
        )
        .unwrap();
        let dir = directory();
        let r = apply_content(&mut c, &p, &p.digest().unwrap(), 101, dir.path()).unwrap();
        assert!(r.read_back);
        assert_eq!(r.effect, effect);
        assert_eq!(
            apply_content(&mut c, &p, &p.digest().unwrap(), 102, dir.path()).unwrap_err(),
            Error::Replay
        );
        assert_eq!(
            calls
                .borrow()
                .iter()
                .filter(|c| c.0 == Method::Post)
                .count(),
            1
        );
    }
}
#[test]
fn uncertain_mutation_preserves_replay_barrier_and_reconciliation_is_observation_only() {
    let (mut c, calls) = fake(vec![
        Ok(discovery()),
        Ok(actor()),
        Ok(state("draft", "Before")),
        Ok(discovery()),
        Ok(actor()),
        Ok(state("draft", "Before")),
        Err(Error::Transport),
        Ok(discovery()),
        Ok(actor()),
        Ok(state("publish", "After")),
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
        apply_content(&mut c, &p, &p.digest().unwrap(), 101, d.path()).unwrap_err(),
        Error::UnknownResult
    );
    assert_eq!(
        reconcile_content(&mut c, &p).unwrap(),
        Reconciliation::DesiredStateObserved
    );
    assert!(
        std::fs::read_to_string(d.path().join(format!("{}.json", p.digest().unwrap())))
            .unwrap()
            .contains("do not replay")
    );
    assert_eq!(
        calls
            .borrow()
            .iter()
            .filter(|c| c.0 == Method::Post)
            .count(),
        1
    );
}
