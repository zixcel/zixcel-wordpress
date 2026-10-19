use super::{Mode, Op};
pub(super) const OPS: &[Op] = &[
    op!(
        "draft-apply",
        "draft-apply",
        &["draft", "apply"],
        "operator-write",
        Mode::Apply,
        &[
            ("plan-file", true),
            ("approval-sha256", true),
            ("journal-directory", true)
        ]
    ),
    op!(
        "content-apply",
        "content-apply",
        &["content", "apply"],
        "operator-write",
        Mode::Apply,
        &[
            ("plan-file", true),
            ("approval-sha256", true),
            ("journal-directory", true)
        ]
    ),
    op!(
        "media-apply",
        "media-apply",
        &["media", "apply"],
        "operator-write",
        Mode::Apply,
        &[
            ("file", true),
            ("plan-file", true),
            ("approval-sha256", true),
            ("journal-directory", true)
        ]
    ),
    op!(
        "revoke-apply",
        "revoke-apply",
        &["auth", "revoke-apply"],
        "operator-write",
        Mode::Apply,
        &[
            ("plan-file", true),
            ("approval-sha256", true),
            ("journal-directory", true)
        ]
    ),
];
