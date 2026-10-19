use crate::{Error, Result};
use std::net::{IpAddr, SocketAddr, ToSocketAddrs};
/// Public DNS targets only. Never follow metadata/local/rebinding targets or proxies.
pub fn public_address(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v) => {
            let [a, b, c, _] = v.octets();
            !(a == 0
                || a == 10
                || a == 127
                || a >= 224
                || a == 100 && (64..128).contains(&b)
                || a == 169 && b == 254
                || a == 172 && (16..32).contains(&b)
                || a == 192 && (b == 168 || b == 0 || b == 88 && c == 99)
                || a == 198 && (b == 18 || b == 19 || b == 51 && c == 100)
                || a == 203 && b == 0 && c == 113)
        }
        IpAddr::V6(v) => {
            let s = v.segments();
            s[0] & 0xe000 == 0x2000
                && !(s[0] == 0x2001 && (s[1] < 0x200 || s[1] == 0xdb8))
                && !(s[0] == 0x2002)
        }
    }
}
pub(crate) fn resolve(host: &str) -> Result<Vec<SocketAddr>> {
    let addresses = (host, 443)
        .to_socket_addrs()
        .map_err(|_| Error::Transport)?
        .collect::<Vec<_>>();
    if addresses.is_empty()
        || addresses.len() > 32
        || addresses.iter().any(|a| !public_address(a.ip()))
    {
        return Err(Error::UnsafeTarget);
    }
    Ok(addresses)
}
