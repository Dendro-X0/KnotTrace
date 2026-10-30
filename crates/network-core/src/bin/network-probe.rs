use network_core::{
    build_diagnostic_bundle, explain_locally, parse_args, run_health_check_with_settings, usage,
    CliCommand, CliOutput, DiagnosticBundleOptions, JsonStyle,
};

#[tokio::main]
async fn main() {
    let command = match parse_args(std::env::args()) {
        Ok(cmd) => cmd,
        Err(error) => {
            eprintln!("usage error: {}", error.message);
            eprintln!("{}", usage());
            std::process::exit(2);
        }
    };

    match command {
        CliCommand::Help => {
            print!("{}", usage());
        }
        CliCommand::Check {
            profile,
            output,
            style,
        } => match run_health_check_with_settings(None, profile).await {
            Ok(report) => {
                let payload = match output {
                    CliOutput::Report => emit(&report, style),
                    CliOutput::Explain => emit(&explain_locally(&report), style),
                    CliOutput::Bundle => emit(
                        &build_diagnostic_bundle(&report, DiagnosticBundleOptions::default()),
                        style,
                    ),
                };
                match payload {
                    Ok(text) => println!("{text}"),
                    Err(error) => {
                        eprintln!("serialize failed: {error}");
                        std::process::exit(1);
                    }
                }
            }
            Err(error) => {
                eprintln!("health check failed: {error}");
                std::process::exit(1);
            }
        },
    }
}

fn emit<T: serde::Serialize>(value: &T, style: JsonStyle) -> Result<String, serde_json::Error> {
    match style {
        JsonStyle::Compact => serde_json::to_string(value),
        JsonStyle::Pretty => serde_json::to_string_pretty(value),
    }
}
