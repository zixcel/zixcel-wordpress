//! Test-only host for the actual dispatcher library; excluded from release bundles.
#![forbid(unsafe_code)]
#[tokio::main(flavor = "current_thread")]
async fn main() {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    match zix_cli::command_line::run(&args).await {
        Ok(true) => {}
        Ok(false) => std::process::exit(2),
        Err(e) => {
            eprintln!("{}", serde_json::json!({"error":e.0}));
            std::process::exit(2)
        }
    }
}
