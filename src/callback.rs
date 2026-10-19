mod parser;
use crate::{AuthorizationSession, BrowserGrant, Error, Result};
use std::{
    io::{Read, Write},
    time::{Duration, Instant},
};
use zeroize::Zeroizing;
impl AuthorizationSession {
    pub fn receive(self) -> Result<BrowserGrant> {
        loop {
            if Instant::now() >= self.expires {
                return Err(Error::Expired);
            }
            match self.listener.accept() {
                Ok((mut stream, peer)) => {
                    if !peer.ip().is_loopback() {
                        return Err(Error::UnsafeTarget);
                    }
                    let remaining = self
                        .expires
                        .saturating_duration_since(Instant::now())
                        .min(Duration::from_secs(5));
                    stream
                        .set_read_timeout(Some(remaining))
                        .map_err(|_| Error::LocalState)?;
                    stream
                        .set_write_timeout(Some(remaining))
                        .map_err(|_| Error::LocalState)?;
                    let mut raw = Zeroizing::new(Vec::new());
                    let mut byte = [0u8; 1];
                    while !raw.ends_with(b"\r\n\r\n") && raw.len() < 8192 {
                        if stream.read(&mut byte).map_err(|_| Error::InvalidInput)? == 0 {
                            break;
                        }
                        raw.push(byte[0]);
                    }
                    let result = self.parse_callback(&raw);
                    // No URL/query, external assets or reflected error is returned to the browser.
                    let response=b"HTTP/1.1 303 See Other\r\nLocation: /complete\r\nCache-Control: no-store\r\nReferrer-Policy: no-referrer\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
                    let _ = stream.write_all(response);
                    return result;
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(Duration::from_millis(10))
                }
                Err(_) => return Err(Error::LocalState),
            }
        }
    }
}
