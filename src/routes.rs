use crate::Method;
pub(crate) fn allowed(method: Method, path: &str) -> bool {
    let mut parts = path.split('?');
    let route = parts.next().unwrap_or_default();
    if parts.clone().count() > 1 || path.len() > 512 || path.bytes().any(|c| c.is_ascii_control()) {
        return false;
    }
    if route == "/wp-json/" {
        return method == Method::Get;
    }
    let Some(suffix) = route.strip_prefix("/wp-json/wp/v2/") else {
        return false;
    };
    suffix == "users/me" && method == Method::Get
        || ["posts", "pages"].iter().any(|kind| {
            suffix == *kind
                || suffix
                    .strip_prefix(&format!("{kind}/"))
                    .is_some_and(|id| !id.is_empty() && id.bytes().all(|c| c.is_ascii_digit()))
                    && method == Method::Get
        })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn request_scope_refuses_other_apis_or_arbitrary_write_paths() {
        for path in [
            "https://other.example/",
            "/wp-json/wp/v2/users",
            "/wp-json/wp/v2/posts/../settings",
            "/wp-json/wp/v2/plugins",
            "/wp-json/wp/v2/posts/1",
        ] {
            assert!(!allowed(Method::Post, path));
        }
        assert!(allowed(Method::Get, "/wp-json/wp/v2/posts/1?context=edit"));
        assert!(allowed(Method::Post, "/wp-json/wp/v2/posts"));
        assert!(!allowed(Method::Get, "/wp-json/wp/v2/posts?x=y?z=bad"));
    }
}
