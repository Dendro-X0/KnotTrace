# Slow-Speed Triage

This document converts slowdown research into a practical triage model for KnotTrace.

Goal:

1. Identify the **symptom shape**
2. Infer the **most likely bottleneck layer**
3. Attach **observable signals**
4. Recommend the **safest next action**

This is not a promise that KnotTrace will detect every cause automatically. It is a guide for future detection, diagnosis text, and assist policy.

## Triage principles

- **Do not treat all slowness as throughput loss**
- **Prefer confidence-ranked hints over guessed fixes**
- **Separate reversible actions from manual/system changes**
- **Do not silently bypass policy networks, captive portals, or VPN intent**

## Symptom matrix

### 1. Pages pause before loading, then load quickly

Likely layers:

- DNS
- Captive portal
- Proxy / PAC
- Resolver cache or split-DNS

High-value signals:

- DNS latency high
- DNS integrity mismatches
- Site reachability mixed or partial
- Captive portal suspected or confirmed
- Proxy enabled

Confidence rules:

- **High**: high DNS latency plus integrity mismatch or repeated lookup failures
- **Medium**: DNS latency high but integrity clean
- **Low**: only generic user complaint with no probe evidence

Safe recommendations:

- Review DNS Assist recommendation
- Complete captive portal sign-in if detected
- Review proxy settings if browsing fails only on proxied paths

Safe auto-actions:

- Trusted DNS apply when integrity is caution/suspicious and policy allows

Manual or opt-in actions:

- Change proxy/PAC settings
- Flush resolver cache

### 2. Speed test is fine, but browsing/video calls feel laggy during downloads or uploads

Likely layers:

- Bufferbloat
- Router queue saturation
- Local background traffic

High-value signals:

- Idle latency normal, loaded latency much higher
- Health drops only under load
- Throughput okay but responsiveness poor

Confidence rules:

- **High**: bufferbloat probe moderate/severe
- **Medium**: repeated reports of lag under load with mild latency inflation
- **Low**: no load-sensitive data captured

Safe recommendations:

- Explain bufferbloat in plain language
- Suggest checking router SQM/QoS
- Suggest pausing large uploads/sync tasks during calls

Safe auto-actions:

- None today

Manual or opt-in actions:

- Router SQM/CAKE/fq_codel configuration

### 3. Some sites/apps stall, others work

Likely layers:

- MTU / fragmentation
- Proxy path failure
- Split tunneling
- DNS filtering

High-value signals:

- Site reachability degraded
- MTU fragmentation risk
- Proxy enabled with failing verification domains
- VPN/Tor path active
- Egress IP differs by path

Confidence rules:

- **High**: MTU hint plus tunnel present, or proxy path failures clustered on verification sites
- **Medium**: site failures without MTU hint
- **Low**: only app-specific complaint

Safe recommendations:

- Review MTU/tunnel guidance
- Compare direct vs proxy/VPN/Tor path behavior
- Use Connect Assist manually if a proxy path is clearly worse

Safe auto-actions:

- None by default

Manual or opt-in actions:

- Lower tunnel MTU
- Switch proxy/VPN server
- Adjust split tunneling

### 4. Connected to Wi-Fi, but internet is partial or blocked

Likely layers:

- Captive portal
- Guest/public Wi-Fi restrictions
- DNS interception

High-value signals:

- Captive portal suspected or confirmed
- Guest Wi-Fi/public network context
- DNS integrity concern
- Site reachability failures

Confidence rules:

- **High**: captive portal redirect detected
- **Medium**: guest network plus DNS/reachability anomalies
- **Low**: Wi-Fi path only, no anomalies

Safe recommendations:

- Open browser and complete sign-in
- Avoid sensitive activity until the path is trusted or VPN-protected
- Re-run the check after sign-in

Safe auto-actions:

- Trusted DNS apply if hijacking signals remain after sign-in and policy allows

Manual or opt-in actions:

- Join a different network
- Enable a trusted VPN

### 5. Only one device is slow

Likely layers:

- Local OS settings
- Background traffic
- NIC power saving
- Driver or adapter issue

High-value signals:

- Other devices on same network are fine
- Local Wi-Fi/Ethernet path looks weaker than expected
- DNS and internet latency unstable only on this host
- Known proxy/VPN/filter stack present

Confidence rules:

- **High**: strong local-only evidence from comparative testing
- **Medium**: device-local anomalies but no comparison data
- **Low**: user report only

Safe recommendations:

- Check OS updates, cloud sync, or backup traffic
- Review proxy/VPN/security software
- Compare Ethernet vs Wi-Fi

Safe auto-actions:

- None

Manual or opt-in actions:

- Disable NIC power saving
- Update NIC driver
- Change service order or renew DHCP

### 6. Ethernet is unexpectedly capped or unstable

Likely layers:

- Duplex mismatch
- Link negotiated too low
- Bad cable / port / adapter

High-value signals:

- Ethernet path detected
- Throughput unexpectedly low
- Packet loss or retransmit symptoms
- Platform-specific NIC/link counters in future versions

Confidence rules:

- **High**: explicit negotiated-speed/duplex error from OS tools
- **Medium**: Ethernet only underperforms while Wi-Fi behaves normally
- **Low**: throughput low with no physical evidence

Safe recommendations:

- Check cable and switch/router port
- Prefer auto-negotiation unless both ends must be forced
- Compare with another cable or adapter

Safe auto-actions:

- None

Manual or opt-in actions:

- Duplex/speed tuning
- Driver and firmware changes

### 7. VPN or Tor is active, and everything is slower than expected

Likely layers:

- Tunnel overhead
- Distant exit node
- Overloaded server
- Tunnel MTU
- SOCKS/bootstrap issues

High-value signals:

- VPN/Tor detected
- Egress differs from system path
- Higher latency than baseline
- Site reachability degraded only on tunneled path
- Tor SOCKS unreachable

Confidence rules:

- **High**: tunnel detected plus degraded path metrics
- **Medium**: tunnel detected plus user complaint
- **Low**: no measurable degradation

Safe recommendations:

- Explain that privacy routes trade throughput for security/anonymity
- Suggest another server/node manually
- Suggest checking MTU if large transfers stall

Safe auto-actions:

- None by default

Manual or opt-in actions:

- Switch VPN server
- Switch proxy node
- Adjust Tor or SOCKS config

## Detection roadmap

### Strong current signals

- DNS latency
- DNS integrity
- Site reachability
- Proxy/VPN/Tor presence
- Guest/public network context
- Captive portal hint
- Public egress IP consistency
- Bufferbloat-lite
- MTU fragmentation risk

### Medium-term signals

- Comparative direct vs proxy path timing
- Local contention suspicion
- Better Wi-Fi quality hints
- Per-platform resolver state detail

### Longer-term / optional signals

- Windows autotuning / RSS / power plan inspection
- Linux negotiated link speed / duplex / error counters
- macOS service-order and Private Relay inspection
- Driver and adapter capability hints

## Suggested diagnosis voice

KnotTrace should phrase results like:

- "Browsing is delayed before pages start loading. DNS or captive portal behavior is the most likely cause."
- "Your connection has enough bandwidth, but latency spikes badly under load. This looks more like bufferbloat than a raw speed problem."
- "Some sites are failing on the current proxy or tunnel path. This suggests a path or MTU issue, not a total internet outage."

Avoid phrasing like:

- "Internet is slow" without naming the layer
- "Try everything" style shotgun advice
- Implied certainty when only weak signals exist

## Safe-action policy

KnotTrace should keep a three-tier action model:

### Tier 1: Observe only

- Public IP checks
- Captive portal hints
- Guest/public network classification
- DNS integrity findings
- VPN/Tor/proxy detection

### Tier 2: Safe reversible assists

- Trusted DNS apply with backup/rollback
- Re-check after an assist

### Tier 3: Recommend or require opt-in

- Proxy node switching
- MTU changes
- Driver tuning
- Router QoS/SQM changes
- VPN server changes
- macOS service-order changes

## Product fit

This triage model supports future work in:

- diagnosis ranking
- recommendations prioritization
- protect alert wording
- platform-specific capability expansion
- deciding what KnotTrace should automate versus explain
