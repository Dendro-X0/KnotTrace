use crate::probe::measure_tcp_latency;
use crate::process::hidden_command;
use crate::types::*;
use std::time::{Duration, Instant};
use tokio::time::sleep;

const INTERNET_HOST: &str = "1.1.1.1";
const INTERNET_PORT: u16 = 443;
const LATENCY_ATTEMPTS: usize = 4;
const LOAD_DURATION: Duration = Duration::from_secs(2);
const LOAD_RAMP: Duration = Duration::from_millis(250);
const MTU_TEST_PAYLOADS: &[u16] = &[1472, 1400, 1300, 1200];
const IPV4_HEADER_OVERHEAD: u16 = 28;

pub async fn run_stability_probes() -> StabilityProbeResult {
    let started = Instant::now();
    let bufferbloat = probe_bufferbloat().await.ok();
    let mtu = probe_mtu_path().await.ok();

    StabilityProbeResult {
        bufferbloat,
        mtu,
        duration_ms: started.elapsed().as_millis() as u64,
    }
}

pub fn classify_bufferbloat(idle_ms: f64, loaded_ms: f64) -> BufferbloatProbe {
    let latency_delta_ms = (loaded_ms - idle_ms).max(0.0);
    let grade = bufferbloat_grade(latency_delta_ms);
    let summary = match grade {
        BufferbloatGrade::None => "Latency stays stable under light load.".to_string(),
        BufferbloatGrade::Mild => format!(
            "Latency rises {:.0} ms under load — mild buffer pressure.",
            latency_delta_ms
        ),
        BufferbloatGrade::Moderate => format!(
            "Latency rises {:.0} ms under load — likely router bufferbloat.",
            latency_delta_ms
        ),
        BufferbloatGrade::Severe => format!(
            "Latency rises {:.0} ms under load — severe bufferbloat affecting stability.",
            latency_delta_ms
        ),
    };

    BufferbloatProbe {
        idle_latency_ms: idle_ms,
        loaded_latency_ms: loaded_ms,
        latency_delta_ms,
        grade,
        summary,
    }
}

pub fn build_mtu_probe(largest_payload: Option<u16>) -> MtuProbe {
    let estimated_path_mtu = largest_payload.map(|payload| payload.saturating_add(IPV4_HEADER_OVERHEAD));
    let recommended_tcp_mss = estimated_path_mtu.map(|mtu| mtu.saturating_sub(40));
    let fragmentation_risk = estimated_path_mtu.is_some_and(|mtu| mtu < 1400);

    let summary = match estimated_path_mtu {
        Some(mtu) if fragmentation_risk => format!(
            "Estimated path MTU {mtu} bytes — tunnel/VPN fragmentation risk. Try MSS ~{}.",
            recommended_tcp_mss.unwrap_or(1360)
        ),
        Some(mtu) => format!("Estimated path MTU {mtu} bytes looks typical for broadband."),
        None => "Could not estimate path MTU from ICMP probes.".to_string(),
    };

    MtuProbe {
        estimated_path_mtu,
        recommended_tcp_mss,
        fragmentation_risk,
        summary,
    }
}

async fn probe_bufferbloat() -> Result<BufferbloatProbe, String> {
    let idle = measure_tcp_latency(INTERNET_HOST, INTERNET_PORT, LATENCY_ATTEMPTS).await?;
    let load = tokio::spawn(background_load(LOAD_DURATION));
    sleep(LOAD_RAMP).await;
    let loaded = measure_tcp_latency(INTERNET_HOST, INTERNET_PORT, LATENCY_ATTEMPTS).await?;
    load.abort();

    Ok(classify_bufferbloat(idle.avg_ms, loaded.avg_ms))
}

async fn background_load(duration: Duration) {
    let client = match reqwest::Client::builder()
        .timeout(Duration::from_secs(8))
        .build()
    {
        Ok(client) => client,
        Err(_) => return,
    };

    let deadline = Instant::now() + duration;
    while Instant::now() < deadline {
        if client
            .get("https://speed.cloudflare.com/__down?bytes=500000")
            .send()
            .await
            .and_then(|response| response.error_for_status())
            .is_err()
        {
            break;
        }
    }
}

async fn probe_mtu_path() -> Result<MtuProbe, String> {
    let mut largest: Option<u16> = None;

    for payload in MTU_TEST_PAYLOADS {
        if ping_df(*payload, INTERNET_HOST).await {
            largest = Some(*payload);
            break;
        }
    }

    Ok(build_mtu_probe(largest))
}

async fn ping_df(payload_size: u16, host: &str) -> bool {
    let host = host.to_string();
    tokio::task::spawn_blocking(move || ping_df_blocking(payload_size, &host))
        .await
        .unwrap_or(false)
}

fn ping_df_blocking(payload_size: u16, host: &str) -> bool {
    #[cfg(windows)]
    {
        let output = hidden_command("ping")
            .args([
                "-n",
                "1",
                "-f",
                "-w",
                "1500",
                "-l",
                &payload_size.to_string(),
                host,
            ])
            .output();

        return match output {
            Ok(result) => {
                let stdout = String::from_utf8_lossy(&result.stdout).to_ascii_lowercase();
                let stderr = String::from_utf8_lossy(&result.stderr).to_ascii_lowercase();
                result.status.success()
                    && !stdout.contains("fragment")
                    && !stdout.contains("need to fragment")
                    && !stderr.contains("fragment")
            }
            Err(_) => false,
        };
    }

    #[cfg(not(windows))]
    {
        let output = hidden_command("ping")
            .args([
                "-c",
                "1",
                "-W",
                "2",
                "-M",
                "do",
                "-s",
                &payload_size.to_string(),
                host,
            ])
            .output();

        match output {
            Ok(result) => {
                let combined = format!(
                    "{}{}",
                    String::from_utf8_lossy(&result.stdout),
                    String::from_utf8_lossy(&result.stderr)
                )
                .to_ascii_lowercase();
                result.status.success()
                    && !combined.contains("frag")
                    && !combined.contains("message too long")
            }
            Err(_) => false,
        }
    }
}

fn bufferbloat_grade(delta_ms: f64) -> BufferbloatGrade {
    if delta_ms < 30.0 {
        BufferbloatGrade::None
    } else if delta_ms < 80.0 {
        BufferbloatGrade::Mild
    } else if delta_ms < 150.0 {
        BufferbloatGrade::Moderate
    } else {
        BufferbloatGrade::Severe
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_bufferbloat_grades() {
        let mild = classify_bufferbloat(20.0, 55.0);
        assert_eq!(mild.grade, BufferbloatGrade::Mild);

        let severe = classify_bufferbloat(25.0, 200.0);
        assert_eq!(severe.grade, BufferbloatGrade::Severe);
    }

    #[test]
    fn flags_low_mtu_fragmentation_risk() {
        let mtu = build_mtu_probe(Some(1300));
        assert!(mtu.fragmentation_risk);
        assert_eq!(mtu.estimated_path_mtu, Some(1328));
    }
}
