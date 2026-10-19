use crate::Method;
pub(crate) fn allowed(method: Method, path: &str) -> bool {
    let mut parts = path.split('?');
    let route = parts.next().unwrap_or_default();
    if parts.clone().count() > 1 || path.len() > 512 || path.bytes().any(|c| c.is_ascii_control()) {
        return false;
    }
    let mut seen = std::collections::BTreeSet::new();
    for (name, value) in url::form_urlencoded::parse(parts.next().unwrap_or_default().as_bytes()) {
        if !seen.insert(name.to_string())
            || !match name.as_ref() {
                "context" => value == "edit",
                "per_page" => value == "25",
                "page" => value
                    .parse::<u16>()
                    .is_ok_and(|page| (1..=100).contains(&page)),
                "_fields" => matches!(
                    value.as_ref(),
                    "id,capabilities" | "id,status,title.raw" | "id,status,title.raw,content.raw"
                ),
                _ => false,
            }
        {
            return false;
        }
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
        for query in [
            "_method=DELETE",
            "%5Fmethod=DELETE",
            "_jsonp=callback",
            "_envelope=1",
            "context=edit&context=view",
        ] {
            assert!(!allowed(
                Method::Post,
                &format!("/wp-json/wp/v2/posts?{query}")
            ));
        }
    }
}
