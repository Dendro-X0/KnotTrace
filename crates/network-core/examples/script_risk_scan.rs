//! One-shot bounded script-risk scan (indicators only — not antivirus).
use network_core::run_script_risk_scan;

fn main() {
    let snap = run_script_risk_scan();
    println!("source: {}", snap.source);
    println!("summary: {}", snap.summary);
    println!();
    print_hits("persistence / script hosts", &snap.persistence_hits);
    print_hits("mining-like processes", &snap.mining_hits);
    print_hits("risky scripts on disk", &snap.disk_hits);
}

fn print_hits(title: &str, hits: &[network_core::SuspiciousActivityHit]) {
    println!("== {title} ({}) ==", hits.len());
    if hits.is_empty() {
        println!("  (none)");
        println!();
        return;
    }
    for hit in hits {
        println!("  - {}", hit.name);
        println!("    @ {}", hit.location);
        if let Some(detail) = &hit.detail {
            println!("    detail: {detail}");
        }
    }
    println!();
}
