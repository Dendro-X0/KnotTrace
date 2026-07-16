use crate::types::{EnvironmentTag, HealthReport, MtuProbe};

pub const MIN_RECOMMENDED_MTU: u16 = 1200;
pub const MAX_RECOMMENDED_MTU: u16 = 1400;
const DEFAULT_TUNNEL_MTU: u16 = 1400;

pub fn tunnel_evidenced(report: &HealthReport) -> bool {
    let env = &report.environment;
    if env.proxy.enabled || env.tor.detected {
        return true;
    }
    if env.tags.contains(&EnvironmentTag::Vpn)
        || env.tags.contains(&EnvironmentTag::Tor)
        || env.tags.contains(&EnvironmentTag::Proxy)
    {
        return true;
    }
    if let Some(compare) = &report.tunnel_compare {
        if compare.vpn_detected || compare.tor_detected || compare.proxy_enabled {
            return true;
        }
    }
    false
}

pub fn recommend_mtu(probe: Option<&MtuProbe>) -> Option<u16> {
    let probe = probe?;
    if !probe.fragmentation_risk {
        return None;
    }
    Some(
        probe
            .estimated_path_mtu
            .unwrap_or(DEFAULT_TUNNEL_MTU)
            .clamp(MIN_RECOMMENDED_MTU, MAX_RECOMMENDED_MTU),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;

    #[test]
    fn parses_nothing_extra() {
        // Keep parsers module focused; clamp logic covered in mtu_assist tests.
        let probe = MtuProbe {
            estimated_path_mtu: Some(1328),
            recommended_tcp_mss: Some(1288),
            fragmentation_risk: true,
            summary: String::new(),
        };
        assert_eq!(recommend_mtu(Some(&probe)), Some(1328));
    }
}
