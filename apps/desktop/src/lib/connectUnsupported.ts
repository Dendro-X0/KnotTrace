/** Narrow connect recommendation shape for unsupported / switch gating. */
export interface ConnectSwitchSource {
  kernel?: { kind?: string | null } | null;
  should_apply?: boolean | null;
  recommended_switch?: { target_proxy?: string | null } | null;
}

export function isConnectUnsupported(
  rec: ConnectSwitchSource | null | undefined,
): boolean {
  return !rec?.kernel;
}

/** Only show Switch when a real controllable recommendation exists — never a disabled fake Apply. */
export function shouldShowConnectSwitch(
  rec: ConnectSwitchSource | null | undefined,
): boolean {
  if (isConnectUnsupported(rec)) return false;
  return Boolean(rec?.should_apply && rec?.recommended_switch?.target_proxy);
}
