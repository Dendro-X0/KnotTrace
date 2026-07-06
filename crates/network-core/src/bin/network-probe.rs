use network_core::run_health_check;

#[tokio::main]
async fn main() {
    match run_health_check().await {
        Ok(report) => {
            println!("{}", serde_json::to_string_pretty(&report).expect("serialize report"));
        }
        Err(error) => {
            eprintln!("health check failed: {error}");
            std::process::exit(1);
        }
    }
}
