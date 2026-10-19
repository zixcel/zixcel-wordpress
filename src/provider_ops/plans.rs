use super::{Mode, Op};
pub(super) const OPS: &[Op] = &[
    op!(
        "draft-plan",
        "draft-plan",
        &["draft", "plan"],
        "read",
        Mode::Plan("wordpress.draft-apply"),
        &[("input-file", true)]
    ),
    op!(
        "content-plan",
        "content-plan",
        &["content", "plan"],
        "read",
        Mode::Plan("wordpress.content-apply"),
        &[("input-file", true)]
    ),
    op!(
        "media-plan",
        "media-plan",
        &["media", "plan"],
        "read",
        Mode::Plan("wordpress.media-apply"),
        &[("file", true), ("input-file", true)]
    ),
    op!(
        "revoke-plan",
        "revoke-plan",
        &["auth", "revoke-plan"],
        "read",
        Mode::Plan("wordpress.revoke-apply"),
        &[]
    ),
    op!(
        "content-reconcile",
        "content-reconcile",
        &["content", "reconcile"],
        "read",
        Mode::Reconcile,
        &[("plan-file", true)]
    ),
];
