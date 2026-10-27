//! Test-only provider: real protocol/router/backend with a synthetic transport, never production credentials/network.
#![forbid(unsafe_code)]
// Browser login belongs to the regular native host, not this transport fixture.
#[allow(dead_code)]
#[path = "../../src/auth_command.rs"]
mod auth_command;
#[path = "../../src/command_ops.rs"]
mod command_ops;
#[path = "../../src/content_command.rs"]
mod content_command;
#[path = "fixture_transport.rs"]
mod fixture_transport;
#[path = "../../src/input.rs"]
mod input;
#[path = "../../src/local_file.rs"]
mod local_file;
#[path = "../../src/media_command.rs"]
mod media_command;
#[path = "../../src/provider.rs"]
mod provider;
#[path = "../../src/provider_error.rs"]
mod provider_error;
#[path = "../../src/provider_guidance.rs"]
mod provider_guidance;
#[path = "../../src/provider_invoke.rs"]
mod provider_invoke;
#[path = "../../src/provider_manifest.rs"]
mod provider_manifest;
#[path = "../../src/provider_ops.rs"]
mod provider_ops;
#[path = "../../src/provider_plan.rs"]
mod provider_plan;
#[path = "../../src/provider_receipt.rs"]
mod provider_receipt;
mod command {
    pub(crate) fn execute(
        args: &[String],
        collection: Option<&str>,
        source: Option<&[u8]>,
    ) -> zixcel_wordpress::Result<serde_json::Value> {
        fixture(args, collection, source)
    }
    fn fixture(
        args: &[String],
        collection: Option<&str>,
        source: Option<&[u8]>,
    ) -> zixcel_wordpress::Result<serde_json::Value> {
        use zixcel_wordpress::{Error, Profile, WordPress};
        if args.get(1).map(String::as_str) != Some("example")
            || args.get(2).map(String::as_str) != Some("https://example.test")
            || collection != Some("synthetic")
        {
            return Err(Error::InvalidInput);
        }
        let transport = crate::fixture_transport::for_operation(&args[0], source)?;
        let mut client = WordPress::new(
            Profile {
                id: "example".into(),
                origin: "https://example.test".into(),
            },
            transport,
        )?;
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| Error::LocalState)?
            .as_secs();
        crate::command_ops::dispatch(&mut client, args, now, source)
    }
}
fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let result = if args == ["--manifest"] {
        provider::manifest()
    } else if args == ["--guidance"] {
        provider_guidance::catalog().and_then(|c| {
            serde_json::to_value(c).map_err(|_| zixcel_wordpress::Error::InvalidInput)
        })
    } else if args == ["--zix-provider-v2"] {
        provider::run()
    } else {
        Err(zixcel_wordpress::Error::Unsupported)
    };
    match result {
        Ok(value) => println!("{value}"),
        Err(_) => {
            eprintln!("fixture input rejected");
            std::process::exit(2)
        }
    }
}
