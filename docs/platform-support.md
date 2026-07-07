# Platform support

## Desktop targets

Tauri bundles for Windows, macOS, and Linux. **Primary development and testing target is Windows.**

Release automation now builds desktop installers for all three platforms on tag pushes.

| Feature | Windows | macOS / Linux |
|---------|---------|---------------|
| Health monitor & probes | Yes | Yes |
| Diagnosis & stability | Yes | Yes |
| DNS integrity (observe) | Yes | Yes |
| Benchmark snapshots | Yes | Yes |
| Throughput test (on-demand) | Yes | Yes |
| Connect Assist (Clash API) | Yes | Yes* |
| Protect & notifications | Yes | Yes |
| **DNS Assist apply/restore** | **Yes** | **Not yet** (observe + recommend only) |

\* Requires a local Mihomo, sing-box, or compatible proxy with `external-controller` enabled.

## DNS Assist (Windows)

Apply and restore use the Windows network configuration API (`dns_platform/windows.rs`). On other platforms, the stub reports: *"DNS assist is not available on this platform yet."*

Recommendations and integrity checks still work everywhere.

## Permissions

- **DNS apply** may require running as administrator on some Windows adapters.
- **Proxy switching** requires the proxy app's Clash API to be reachable on localhost.
- **Throughput test** downloads/uploads a few MB to Cloudflare speed endpoints when you trigger it manually.

## What we do not do

- Corporate VPN or firewall bypass
- Automatic MTU/MSS repair
- Background bandwidth saturation tests
- Guaranteed ISP speed improvements

## Mobile roadmap

- **Android**: manual release workflow exists (`release-android.yml`), not part of default desktop release path
- **iOS**: planned for future release workflow and installer/signing setup
