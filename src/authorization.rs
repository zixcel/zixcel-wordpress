use crate::{Error, Profile, Result};
use std::{
    net::{Ipv4Addr, SocketAddrV4, TcpListener},
    time::{Duration, Instant},
};
use zeroize::Zeroizing;
pub struct BrowserGrant {
    pub username: Zeroizing<String>,
    pub password: Zeroizing<String>,
}
pub struct AuthorizationSession {
    pub(crate) listener: TcpListener,
    pub(crate) profile: Profile,
    pub(crate) expected_user: String,
    pub(crate) path: String,
    pub(crate) state: String,
    pub(crate) expires: Instant,
    pub(crate) address: SocketAddrV4,
}
impl AuthorizationSession {
    /// Explicitly supported core loopback flow only; unknown/older versions keep manual import.
    pub fn bind(
        profile: Profile,
        expected_user: String,
        loopback_supported: bool,
        ttl: Duration,
    ) -> Result<Self> {
        profile.validate()?;
        if !loopback_supported {
            return Err(Error::Unsupported);
        }
        if expected_user.is_empty()
            || expected_user.len() > 128
            || expected_user.chars().any(char::is_control)
            || ttl.is_zero()
            || ttl > Duration::from_secs(300)
        {
            return Err(Error::InvalidInput);
        }
        let listener =
            TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).map_err(|_| Error::LocalState)?;
        listener
            .set_nonblocking(true)
            .map_err(|_| Error::LocalState)?;
        let std::net::SocketAddr::V4(address) =
            listener.local_addr().map_err(|_| Error::LocalState)?
        else {
            return Err(Error::LocalState);
        };
        let mut random = [0u8; 32];
        getrandom::fill(&mut random).map_err(|_| Error::LocalState)?;
        let state: String = random.iter().map(|b| format!("{b:02x}")).collect();
        Ok(Self {
            listener,
            profile,
            expected_user,
            path: format!("/authorize/{state}"),
            state,
            address,
            expires: Instant::now() + ttl,
        })
    }
    pub fn callback_url(&self) -> String {
        format!("http://{}{}?state={}", self.address, self.path, self.state)
    }
    pub fn authorization_url(&self, discovery: &crate::Discovery) -> Result<String> {
        if discovery.origin != self.profile.origin
            || discovery.authorization_url
                != format!("{}/wp-admin/authorize-application.php", self.profile.origin)
        {
            return Err(Error::UnsafeTarget);
        }
        let mut url =
            url::Url::parse(&discovery.authorization_url).map_err(|_| Error::UnsafeTarget)?;
        url.query_pairs_mut()
            .append_pair("app_name", "Zix WordPress")
            .append_pair("success_url", &self.callback_url());
        Ok(url.into())
    }
}
