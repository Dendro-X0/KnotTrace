# Slowdown Factors

This document captures common **system-side** and **path-side** reasons a user's internet can feel slow on Windows, Linux, and macOS. It is meant to guide future KnotTrace detection, diagnosis, and recommendation work.

## How "slow" usually appears

Users often describe different failure modes with the same phrase:

- **Slow page start**: DNS, captive portal, proxy, or TLS interception problems
- **Slow large downloads**: throughput limits, VPN overhead, duplex mismatch, ISP congestion
- **High ping while downloading/uploading**: bufferbloat or router saturation
- **Some sites/apps stall, others work**: MTU, DNS filtering, split tunneling, proxy issues
- **Only one device is slow**: local system settings, background traffic, adapter, or driver issue

KnotTrace should keep separating these cases instead of treating all slowness as a raw Mbps problem.

## Cross-platform common factors

### DNS latency or misconfiguration

Common symptoms:

- Pages pause before loading, then load normally
- Speed tests look fine but browsing feels slow
- Name lookups time out and retry

Common causes:

- Slow ISP resolver
- Broken split-DNS policy
- Captive portal interception
- Stale manual DNS settings
- DNS proxy or filter conflict

KnotTrace relevance:

- Already detects DNS latency and integrity issues
- Can continue recommending trusted DNS when safe

### Proxy, VPN, or Tor path overhead

Common symptoms:

- Higher latency than direct internet
- Inconsistent site reachability
- Good local network, poor real-world browsing
- Different behavior by app depending on route selection

Common causes:

- Distant exit node
- Overloaded proxy/VPN server
- SOCKS endpoint not fully working
- Split tunneling or policy routing mistakes
- MTU mismatch inside the tunnel

KnotTrace relevance:

- Already detects proxy, VPN hints, Tor, site reachability, and egress IP
- Should prefer recommendations over forced node changes

### Captive portals and guest/public Wi-Fi

Common symptoms:

- Connected to Wi-Fi but some or all sites fail
- HTTP redirects to a login page
- DNS answers look suspicious
- Public IP and path behavior are inconsistent

Common causes:

- Hotel, café, airport, school, or restaurant sign-in pages
- Guest isolation or traffic shaping
- Interception before full internet access

KnotTrace relevance:

- Already classifies captive portal and guest/public network context
- Should keep warning users before sensitive activity on untrusted Wi-Fi

### MTU and fragmentation

Common symptoms:

- Some sites stall while others load
- Video calls or uploads freeze intermittently
- Large transfers fail more than normal browsing

Common causes:

- VPN or overlay headers shrinking usable MTU
- PPPoE or tunnel overhead
- Broken path MTU discovery
- ICMP filtering on the path

KnotTrace relevance:

- Already surfaces MTU/fragmentation hints
- Future work can deepen platform-specific MTU guidance

### Bufferbloat and saturation

Common symptoms:

- Ping spikes under load
- Browsing becomes unusable during downloads/uploads
- Video calls lag while backups or updates run

Common causes:

- Router queue bloat
- Uplink saturation from cloud sync or backup
- Poor traffic shaping / no SQM

KnotTrace relevance:

- Already runs bufferbloat-lite probes
- Future recommendations can be more explicit about SQM/CAKE/fq_codel

### Physical and radio-layer problems

Common symptoms:

- Variable speed, packet loss, or retransmits
- Wi-Fi is much worse than Ethernet
- Ethernet link is unexpectedly capped

Common causes:

- Weak Wi-Fi signal
- Congested 2.4 GHz channel
- Bad Ethernet cable or port
- Duplex mismatch
- Adapter or router hardware faults

KnotTrace relevance:

- Already distinguishes Wi-Fi, cellular, proxy/VPN/Tor paths
- Could grow into stronger cable/link/duplex detection where OS support allows

### Background traffic and local contention

Common symptoms:

- Speed is poor only on one device
- Throughput drops during updates, sync, or backups
- Latency rises when another local app is active

Common causes:

- OS updates
- Cloud storage sync
- Backup software
- Security scanners
- Browser or launcher background downloads

KnotTrace relevance:

- Mostly recommendation territory today
- Future work could add "local contention suspected" hints without intrusive process inspection

## Windows factors

### TCP receive window autotuning

If Windows autotuning is disabled or overly restricted, throughput can suffer badly on modern broadband and higher-latency paths.

Why it matters:

- Small fixed receive windows cap TCP throughput
- Mis-tuned values can make fast links underperform

### NIC driver, RSS, and offload behavior

Common factors:

- Outdated NIC driver
- Receive Side Scaling disabled or misconfigured
- Offload quirks on certain adapters

Why it matters:

- Traffic may bottleneck on one CPU core
- Driver bugs can cause drops, stalls, or poor throughput

### Power management

Common factors:

- Adapter power saving
- Aggressive laptop power plans

Why it matters:

- The NIC or CPU can be throttled in ways users experience as inconsistent speed

### System proxy and filter stacks

Common factors:

- WinINet proxy/PAC left behind by corporate software
- Endpoint security inserting filter drivers

Why it matters:

- Requests may be silently rerouted or delayed

## Linux factors

### Link negotiation and duplex mismatch

Common factors:

- Wrong negotiated speed
- Half-duplex mismatch
- Bad switch port or cable

Why it matters:

- Throughput can collapse even though the interface still shows "up"

### NIC errors and hardware counters

Common factors:

- CRC/input/drop errors
- Faulty USB Ethernet adapters
- Driver-specific packet handling bugs

Why it matters:

- Packet corruption and retries often look like general slowness

### Queue discipline and congestion control

Common factors:

- Suboptimal qdisc
- Bufferbloat under load
- Tunnel-heavy setups without sensible pacing

Why it matters:

- Linux can feel "slow" because latency explodes under contention, not because raw bandwidth is low

### Resolver stack complexity

Common factors:

- `systemd-resolved`, NetworkManager, local caches, and manual configs disagreeing
- Custom DNS proxies or ad blockers

Why it matters:

- Name resolution delay is often mistaken for internet speed loss

## macOS factors

### Network service order

Common factors:

- Wrong preferred interface when Wi-Fi and Ethernet are both active
- Unused services taking precedence

Why it matters:

- The Mac may route through a worse path than the user expects

### Wi-Fi diagnostics, channel congestion, and signal quality

Common factors:

- Congested channel
- Weak RSSI / high noise
- Captive or guest network association

Why it matters:

- Wi-Fi quality can degrade real throughput far more than ISP plan speed suggests

### Proxy, VPN, and Private Relay interactions

Common factors:

- Proxy settings left enabled
- VPN client route leaks or MTU issues
- Private Relay interacting badly with another tunnel

Why it matters:

- Routing and resolution behavior can become unpredictable

### DHCP, resolver cache, and network-profile drift

Common factors:

- Bad DHCP lease
- Stale resolver state
- Old remembered network settings

Why it matters:

- The system may be "connected" but still behave like the path is degraded

## What KnotTrace should detect vs recommend

### High-value signals to detect

- DNS latency and integrity
- Captive portal state
- Guest/public network context
- Public egress IP consistency
- Proxy/VPN/Tor path health
- MTU and fragmentation risk
- Bufferbloat under load
- Site reachability over the current path

### Better left as recommendations

- Router SQM/CAKE/fq_codel setup
- VPN provider/server choice
- Wi-Fi channel planning
- NIC driver updates
- Manual MTU changes
- Service-order cleanup on macOS
- Duplex or advanced offload tuning on Linux/Windows

## Product direction

This reference points to a future slowdown triage model:

1. **Classify the symptom** first
2. **Locate the bottleneck layer** second
3. **Recommend the safest reversible action** third

KnotTrace should continue favoring:

- observe-first diagnosis
- reversible assists
- opt-in higher-risk changes
- clear user guidance when the fix belongs outside the app
