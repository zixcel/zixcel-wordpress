#![forbid(unsafe_code)]
mod auth_command;
mod command;
mod content_command;
mod credentials;
mod help;
mod input;
mod local_file;
mod media_command;
mod provider;
mod redaction;
use serde_json::json;
use std::process::ExitCode;
fn main() -> ExitCode {
    match command::run() {
        Ok(value) => {
            println!("{value}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{}", json!({"error":format!("{error:?}")}));
            ExitCode::from(2)
        }
    }
}
