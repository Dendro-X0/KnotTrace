# Platform support

## Desktop targets

Tauri bundles for Windows, macOS, and Linux. **Primary development and testing target is Windows.**

Release automation builds desktop installers for all three platforms on tag pushes.

| Feature | Windows | macOS | Linux |
|---------|---------|-------|-------|
| Health monitor & probes | Yes | Yes | Yes |
| Link facts (speed/duplex) | Yes (`Get-NetAdapter`) | Yes (`networksetup -getMedia`) | Yes (`sysfs`) |
| Local caps (TCP / NIC power) | **Yes** (opt-in repair) | Observe N/A | Observe N/A |
| MTU assist (opt-in clamp) | **Yes** | **Yes** (`networksetup`) | **Yes** (`ip` / sysfs) |
| Tunnel path compare | Yes | Yes | Yes |
| Upstream pool proof | Yes | Yes | Yes |
| Diagnosis & stability | Yes | Yes | Yes |
| DNS integrity (observe) | Yes | Yes | Yes |
| Benchmark snapshots | Yes | Yes | Yes |
| Throughput test (on-demand) | Yes | Yes | Yes |
| Connect Assist (Clash API) | Yes | Yes* | Yes* |
| Protect & notifications | Yes | Yes | Yes |
| **DNS Assist apply/restore** | **Yes** | **Yes** (`networksetup`) | **Yes** (`nmcli` or `resolvectl`) |

\* Requires a local Mihomo, sing-box, or compatible proxy with `external-controller` enabled.

## DNS Assist

| Platform | Backend | Notes |
|----------|---------|-------|
| Windows | `dns_platform/windows.rs` | `Set-DnsClientServerAddress` via PowerShell |
| macOS | `dns_platform/macos.rs` | Maps device → network service; `networksetup -setdnsservers` / `empty` to restore DHCP |
| Linux | `dns_platform/linux.rs` | Prefers NetworkManager `nmcli device modify`; falls back to `resolvectl dns` / `revert` |

Recommendations and integrity checks work everywhere. Apply/restore still require sufficient privileges (UAC, admin prompt, or polkit).

## Local assists (opt-in only)

| Assist | Platforms | Backup file | Auto from Protect? |
|--------|-----------|-------------|--------------------|
| Local caps | Windows | `local_caps_backup.json` | **No** |
| MTU clamp | Windows, macOS, Linux | `mtu_assist_backup.json` | **No** |
| DNS Assist | Windows, macOS, Linux | `dns_backup.json` | Yes, when auto-apply enabled |

## Permissions

- **DNS apply** may require administrator / polkit approval depending on OS policy.
- **MTU assist** and **Windows local caps** may require elevation; always backup first and restore from the Network page.
- **Proxy switching** requires the proxy app's Clash API to be reachable on localhost.
- **Throughput test** downloads/uploads a few MB to Cloudflare speed endpoints when you trigger it manually.

## What we do not do

- Corporate VPN or firewall bypass
- Silent MTU / TCP / registry edits (opt-in panels only)
- Background bandwidth saturation tests
- Auto node hopping / “optimize my airport”
- Guaranteed ISP or proxy speed improvements

## Mobile roadmap

- **Android**: signed APK on every release tag (upload keystore in CI secrets)
- **iOS**: signed IPA on every release tag (distribution cert + provisioning profile)
- Install/trust steps: [mobile-signing.md](mobile-signing.md)
