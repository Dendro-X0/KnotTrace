use network_core::{explain_locally, run_health_check};

#[tokio::main]
async fn main() {
    let explain_only = std::env::args().any(|arg| arg == "--explain");
    match run_health_check().await {
        Ok(report) => {
            if explain_only {
                let explain = explain_locally(&report);
                println!(
                    "{}",
                    serde_json::to_string_pretty(&explain).expect("serialize explain")
                );
            } else {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&report).expect("serialize report")
                );
            }
        }
        Err(error) => {
            eprintln!("health check failed: {error}");
            std::process::exit(1);
        }
    }
}
