#![forbid(unsafe_code)]
mod command;
mod input;
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
