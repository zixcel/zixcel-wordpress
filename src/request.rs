use crate::routes::allowed;
use crate::{Error, Https, Method, Result, Transport};
use crowsi_transport_foundation::http::SensitiveBodyCollector;
use serde_json::Value;
use std::{io::Read, time::Instant};
impl Transport for Https {
    fn origin(&self) -> &str {
        &self.origin
    }
    fn request(
        &mut self,
        method: Method,
        path: &str,
        body: Option<&Value>,
        authenticated: bool,
    ) -> Result<Value> {
        if !allowed(method, path) || self.remaining == 0 || Instant::now() >= self.deadline {
            return Err(Error::UnsafeTarget);
        }
        self.remaining -= 1;
        let mut request = match method {
            Method::Get => self.client.get(format!("{}{path}", self.origin)),
            Method::Post => self.client.post(format!("{}{path}", self.origin)),
            Method::Delete => self.client.delete(format!("{}{path}", self.origin)),
        };
        request = request.header("Accept", "application/json");
        request = request.timeout(
            self.deadline
                .saturating_duration_since(Instant::now())
                .min(std::time::Duration::from_secs(20)),
        );
        if authenticated {
            request = request.header("Authorization", self.authorization.clone());
        }
        if let Some(body) = body {
            request = request.json(body);
        }
        let response = request.send().map_err(|_| {
            if method != Method::Get {
                Error::UnknownResult
            } else {
                Error::Transport
            }
        })?;
        collect(response, method)
    }
    fn upload(&mut self, filename: &str, mime: &str, bytes: &[u8]) -> Result<Value> {
        crate::upload_plan::validate_file(filename, mime, bytes)?;
        if self.remaining == 0 || Instant::now() >= self.deadline {
            return Err(Error::UnsafeTarget);
        }
        self.remaining -= 1;
        let response = self
            .client
            .post(format!("{}/wp-json/wp/v2/media", self.origin))
            .header("Authorization", self.authorization.clone())
            .header("Accept", "application/json")
            .header("Content-Type", mime)
            .header(
                "Content-Disposition",
                format!("attachment; filename=\"{filename}\""),
            )
            .timeout(
                self.deadline
                    .saturating_duration_since(Instant::now())
                    .min(std::time::Duration::from_secs(20)),
            )
            .body(bytes.to_vec())
            .send()
            .map_err(|_| Error::UnknownResult)?;
        collect(response, Method::Post)
    }
}
fn collect(mut response: reqwest::blocking::Response, method: Method) -> Result<Value> {
    if !response.status().is_success() {
        return Err(Error::Status(response.status().as_u16()));
    }
    let mut collector = SensitiveBodyCollector::new(1024 * 1024, response.content_length())
        .map_err(|_| Error::ResponseLimit)?;
    let mut chunk = [0u8; 8192];
    loop {
        let count = response.read(&mut chunk).map_err(|_| {
            if method != Method::Get {
                Error::UnknownResult
            } else {
                Error::Transport
            }
        })?;
        if count == 0 {
            break;
        }
        collector
            .push(&chunk[..count])
            .map_err(|_| Error::ResponseLimit)?;
    }
    serde_json::from_slice(&collector.finish()).map_err(|_| Error::InvalidResponse)
}
