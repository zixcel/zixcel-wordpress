use super::{Mode, Op};
pub(super) const OPS: &[Op] = &[
    op!("login", "login", &["status"], "read", Mode::Read, &[]),
    op!(
        "posts-list",
        "posts-list",
        &["posts", "list"],
        "read",
        Mode::Read,
        &[("page", false)]
    ),
    op!(
        "pages-list",
        "pages-list",
        &["pages", "list"],
        "read",
        Mode::Read,
        &[("page", false)]
    ),
    op!(
        "posts-get",
        "posts-get",
        &["posts", "get"],
        "read",
        Mode::Read,
        &[("id", true)]
    ),
    op!(
        "pages-get",
        "pages-get",
        &["pages", "get"],
        "read",
        Mode::Read,
        &[("id", true)]
    ),
    op!(
        "media-list",
        "media-list",
        &["media", "list"],
        "read",
        Mode::Read,
        &[("page", true)]
    ),
    op!(
        "media-get",
        "media-get",
        &["media", "get"],
        "read",
        Mode::Read,
        &[("id", true)]
    ),
    op!(
        "auth-status",
        "auth-status",
        &["auth", "status"],
        "read",
        Mode::Read,
        &[]
    ),
];
