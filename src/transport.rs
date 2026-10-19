use crate::{Error, Profile, Result, network};
use base64::{Engine, engine::general_purpose::STANDARD};
use reqwest::{blocking::Client, header::HeaderValue};
use serde_json::Value;
use std::time::{Duration, Instant};
use zeroize::Zeroizing;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Method {
    Get,
    Post,
    Delete,
}
pub trait Transport {
    fn origin(&self) -> &str;
    fn request(
        &mut self,
        method: Method,
        path: &str,
        body: Option<&Value>,
        authenticated: bool,
    ) -> Result<Value>;
    fn upload(&mut self, _filename: &str, _mime: &str, _bytes: &[u8]) -> Result<Value> {
        Err(Error::Unsupported)
    }
}
pub struct Https {
    pub(crate) origin: String,
    pub(crate) client: Client,
    pub(crate) authorization: HeaderValue,
    pub(crate) deadline: Instant,
    pub(crate) remaining: u8,
}
impl Https {
    pub fn new(profile: &Profile, username: &str, password: &str) -> Result<Self> {
        let url = profile.validate()?;
        if username.is_empty()
            || username.len() > 128
            || username.chars().any(|c| c.is_control() || c == ':')
            || password.is_empty()
            || password.len() > 512
            || !password.bytes().all(|c| (32..127).contains(&c))
        {
            return Err(Error::InvalidCredential);
        }
        let host = url.host_str().ok_or(Error::InvalidProfile)?;
        let addresses = network::resolve(host)?;
        let basic = Zeroizing::new(format!("{username}:{password}"));
        let encoded = Zeroizing::new(STANDARD.encode(basic.as_bytes()));
        let header = Zeroizing::new(format!("Basic {}", *encoded));
        let mut authorization =
            HeaderValue::from_str(&header).map_err(|_| Error::InvalidCredential)?;
        authorization.set_sensitive(true);
        let client = Client::builder()
            .https_only(true)
            .no_proxy()
            .redirect(reqwest::redirect::Policy::none())
            .retry(reqwest::retry::never())
            .connect_timeout(Duration::from_secs(5))
            .timeout(Duration::from_secs(20))
            .resolve_to_addrs(host, &addresses)
            .build()
            .map_err(|_| Error::Transport)?;
        Ok(Self {
            origin: profile.origin.clone(),
            client,
            authorization,
            deadline: Instant::now() + Duration::from_secs(90),
            remaining: 12,
        })
    }
}
