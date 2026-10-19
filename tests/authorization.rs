use std::{
    io::{Read, Write},
    net::TcpStream,
    time::Duration,
};
use zixcel_wordpress::*;
fn profile() -> Profile {
    Profile {
        id: "example".into(),
        origin: "https://example.com".into(),
    }
}
fn request(callback: &str, change: &str) -> String {
    let u = url::Url::parse(callback).unwrap();
    let mut path = format!(
        "{}?{}&site_url=https%3A%2F%2Fexample.com&user_login=person&password=synthetic-only",
        u.path(),
        u.query().unwrap()
    );
    path.push_str(change);
    format!(
        "GET {path} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\n\r\n",
        u.port().unwrap()
    )
}
fn send(session: AuthorizationSession, raw: String) -> Result<BrowserGrant> {
    let url = url::Url::parse(&session.callback_url()).unwrap();
    let handle = std::thread::spawn(move || session.receive());
    let mut stream = TcpStream::connect(("127.0.0.1", url.port().unwrap())).unwrap();
    stream.write_all(raw.as_bytes()).unwrap();
    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();
    assert!(response.contains("no-store") && response.contains("no-referrer"));
    assert!(!response.contains("synthetic-only"));
    handle.join().unwrap()
}
#[test]
fn bound_loopback_receives_secret_without_output_or_reusable_listener() {
    let s = AuthorizationSession::bind(profile(), "person".into(), true, Duration::from_secs(2))
        .unwrap();
    let callback = s.callback_url();
    let address = url::Url::parse(&callback).unwrap();
    let grant = send(s, request(&callback, "")).unwrap();
    assert_eq!(grant.username.as_str(), "person");
    assert_eq!(grant.password.as_str(), "synthetic-only");
    assert!(TcpStream::connect(("127.0.0.1", address.port().unwrap())).is_err());
}
#[test]
fn unsupported_expired_duplicate_cancel_and_substitution_fail_closed() {
    assert!(matches!(
        AuthorizationSession::bind(profile(), "person".into(), false, Duration::from_secs(1)),
        Err(Error::Unsupported)
    ));
    let s = AuthorizationSession::bind(profile(), "person".into(), true, Duration::from_millis(1))
        .unwrap();
    std::thread::sleep(Duration::from_millis(5));
    assert!(matches!(s.receive(), Err(Error::Expired)));
    for change in [
        "duplicate",
        "cancel",
        "user",
        "site",
        "host",
        "method",
        "path",
    ] {
        let s =
            AuthorizationSession::bind(profile(), "person".into(), true, Duration::from_secs(2))
                .unwrap();
        let mut raw = request(
            &s.callback_url(),
            if change == "duplicate" {
                "&password=other"
            } else {
                ""
            },
        );
        raw = match change {
            "cancel" => raw.replace("&site_url=", "&success=false&site_url="),
            "user" => raw.replace("user_login=person", "user_login=other"),
            "site" => raw.replace("example.com", "other.example"),
            "host" => raw.replace("Host: 127.0.0.1", "Host: localhost"),
            "method" => raw.replace("GET ", "POST "),
            "path" => raw.replace("/authorize/", "/other/"),
            _ => raw,
        };
        assert!(send(s, raw).is_err(), "{change}");
    }
}
