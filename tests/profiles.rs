use std::collections::VecDeque;
mod support;
use support::*;
use zixcel_wordpress::*;
#[test]
fn rejects_unsafe_profiles_and_transport_substitution() {
    for origin in [
        "http://example.com",
        "https://user:secret@example.com",
        "https://127.0.0.1",
        "https://example.com/path",
        "https://example.com?x=y",
        "https://example.com/#x",
    ] {
        assert!(
            Profile {
                id: "example".into(),
                origin: origin.into()
            }
            .validate()
            .is_err()
        );
    }
    let f = Fake {
        origin: "https://other.example".into(),
        responses: VecDeque::new(),
        calls: Default::default(),
    };
    assert!(matches!(
        WordPress::new(profile(), f),
        Err(Error::UnsafeTarget)
    ));
}
#[test]
fn rejects_nonpublic_destinations() {
    for ip in [
        "127.0.0.1",
        "10.0.0.1",
        "169.254.169.254",
        "192.168.0.1",
        "100.64.0.1",
        "::1",
        "::ffff:127.0.0.1",
        "fc00::1",
        "2001:db8::1",
    ] {
        assert!(!public_address(ip.parse().unwrap()), "{ip}");
    }
    assert!(public_address("8.8.8.8".parse().unwrap()));
    assert!(public_address("2606:4700:4700::1111".parse().unwrap()));
}
